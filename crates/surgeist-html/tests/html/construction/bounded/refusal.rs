//! A refused consuming step hands its owner back unchanged, together with the cause.
use super::*;
use std::error::Error;

fn expect_not_ready<T>(refusal: &HtmlConstructionRefusal<T>) {
    assert!(refusal.reason().cause() == HtmlConstructionCause::OperationNotReady);
}

#[test]
fn the_carrier_exposes_the_cause_the_owner_and_its_parts_and_describes_itself_once() {
    let refusal = HtmlConstructionStorage::begin().try_ready().unwrap_err();
    expect_not_ready(&refusal);
    assert_eq!(
        refusal.to_string(),
        "construction step refused: construction operation still has pending actions"
    );
    let as_error: &dyn Error = &refusal;
    assert!(as_error.source().is_none());
    let (mut initialization, reason) = refusal.into_parts();
    assert_eq!(reason.cause(), HtmlConstructionCause::OperationNotReady);
    assert!(initialization.next_action().unwrap().is_some());
}

#[test]
fn a_refused_initialization_is_unchanged_and_can_still_complete() {
    let refusal = HtmlConstructionStorage::begin().try_ready().unwrap_err();
    let (mut initialization, _) = refusal.into_parts();
    while let Some(action) = initialization.next_action().unwrap() {
        action.perform().accept();
    }
    assert!(initialization.try_ready().unwrap().commit().is_empty());
}

#[test]
fn a_refused_operation_keeps_its_command_and_pending_actions() {
    let operation = storage().prepare(HtmlConstructionCommand::CreateFragment);
    let refusal = operation.try_ready().unwrap_err();
    expect_not_ready(&refusal);
    let (mut operation, _) = refusal.into_parts();
    let mut performed = 0;
    while let Some(action) = operation.next_action().unwrap() {
        action.perform().unwrap().accept();
        performed += 1;
    }
    assert!(performed > 0, "no action had run before the refusal");
    let HtmlConstructionCommit::Changed { storage, created } =
        operation.try_ready().unwrap().commit().unwrap()
    else {
        panic!("creation");
    };
    assert!(created.is_some());
    assert_eq!(storage.len(), 1);
}

#[test]
fn a_failed_operation_refuses_with_its_latched_cause_and_still_aborts_to_its_storage() {
    let (other, foreign) = create(storage(), element("foreign"));
    drop(other);
    let (owner, parent) = create(storage(), element("parent"));
    let mut operation = owner.prepare(HtmlConstructionCommand::Insert {
        node: foreign,
        parent,
        before: None,
    });
    let failure = loop {
        match operation.next_action().unwrap() {
            Some(action) => {
                if let Err(failure) = action.perform() {
                    break failure;
                }
            }
            None => panic!("a foreign operand was admitted"),
        }
    };
    let refusal = operation.try_ready().unwrap_err();
    assert_eq!(refusal.reason(), failure.reason());
    let (operation, _) = refusal.into_parts();
    let (storage, command, retired) = operation.abort().unwrap().into_parts();
    assert_eq!(storage.len(), 1);
    assert!(matches!(command, HtmlConstructionCommand::Insert { .. }));
    assert!(retired.is_none());
}

#[test]
fn a_refused_text_append_returns_its_operands() {
    let (owner, text) = create(
        storage(),
        HtmlConstructionCommand::Create {
            leaf: HtmlConstructionLeaf::Text("ab".into()),
        },
    );
    let append = HtmlConstructionTextAppend::begin(owner, text, "cd".into());
    let refusal = append.try_ready().unwrap_err();
    expect_not_ready(&refusal);
    let (mut append, _) = refusal.into_parts();
    assert!(append.next_action().unwrap().is_some());
    let (owner, _node, incoming, discarded) = append.abort().unwrap().into_parts();
    assert_eq!(incoming.code_units(), [0x63, 0x64]);
    assert!(discarded.is_none());
    assert_eq!(owner.len(), 1);
}

#[test]
fn a_refused_recovery_hands_back_the_result_and_the_lease() {
    let (owner, root) = fragment(storage());
    let (result, lease) = publish(owner, root);
    let (other, other_root) = fragment(storage());
    let (other_result, other_lease) = publish(other, other_root);
    let refusal = HtmlConstructionLease::try_recover(result, other_lease).unwrap_err();
    assert!(refusal.reason().cause() == HtmlConstructionCause::ForeignAllocation);
    let ((result, other_lease), _) = refusal.into_parts();
    assert_eq!(result.len(), 1);
    drop((other_result, other_lease));
    let recovery = HtmlConstructionLease::try_recover(result, lease).unwrap();
    let (storage, _receipt) = recovery.into_parts();
    assert_eq!(storage.len(), 1);
}
