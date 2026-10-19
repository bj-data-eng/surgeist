//! The DOM issuer boundary uses the same trusted native publication inputs as
//! the Fullscreen fixture. It executes the original child constructor and seals
//! real issuer receipts; it does not claim Runtime Window or event-loop service.
use super::*;
use crate::{
    DomHostedChildStep, DomInitialPermissionsPolicy, DomPermissionsPolicyValue, DomSourceEdit,
};
use std::cell::Cell;

thread_local! {
    static CANCEL_HISTORY: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn after_history_assignment(cancel: &AtomicBool) {
    CANCEL_HISTORY.with(|armed| {
        if armed.replace(false) {
            cancel.store(true, Ordering::Release);
        }
    });
}

fn limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 4_000_000, 100_000_000, 128, 4096, 4096, 4)
}

fn uuid(last: u16) -> [u16; 36] {
    let mut value = [0; 36];
    for (out, unit) in value
        .iter_mut()
        .zip("00000000-0000-4000-8000-000000000000".encode_utf16())
    {
        *out = unit;
    }
    value[35] = last;
    value
}

#[test]
fn child_history_publication_cancellation_restores_graph_and_retries_the_original_receipt() {
    let cancel = AtomicBool::new(false);
    let (mut dom, mut source, document, _) = super::super::fullscreen::native_fixture(
        DomLimits::new(
            128,
            1_000_000,
            256,
            128,
            128,
            32,
            4_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_fullscreen_exits(4)
        .unwrap()
        .with_fullscreen_events(4)
        .unwrap(),
    );
    let parent = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let mut batch = DomBatch::new();
    // A closed HTML Document permits native construction without a hosted
    // initializer. Moving the Frame into the active tree then starts the real
    // child-construction protocol.
    let closed = batch.create_html_document(document, None);
    let iframe = batch.create_element_ns(&closed, crate::Namespace::html(), "frame");
    let iframe = dom
        .apply(batch, &cancel)
        .unwrap()
        .created(&iframe)
        .unwrap()
        .unwrap();
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(iframe, parent, None),
            limits(),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual connected Frame reaches its child constructor");
    };
    let preparation = lease
        .prepare_hosted_child(&mut source, &child, &cancel)
        .unwrap();
    let mut prepared = Some(
        source
            .seal_child_preparation(&child, &mut Some(preparation), &cancel)
            .unwrap(),
    );
    lease
        .accept_hosted_child_preparation(&mut source, &child, &mut prepared, &cancel)
        .unwrap();
    assert!(prepared.is_none());

    let mut publication = None;
    let mut operation = None;
    for _ in 0..8 {
        let value = match lease.hosted_child_step(&child).unwrap() {
            DomHostedChildStep::PermissionsPolicy(request) => {
                let mut policy = Some(
                    source
                        .seal_permissions_policy(
                            request,
                            DomInitialPermissionsPolicy::new(
                                DomPermissionsPolicyValue::Enabled,
                                DomPermissionsPolicyValue::Enabled,
                            ),
                            &cancel,
                        )
                        .unwrap(),
                );
                lease
                    .accept_hosted_child_policy(&mut source, &child, &mut policy, &cancel)
                    .unwrap();
                assert!(policy.is_none());
                continue;
            }
            DomHostedChildStep::NativeExecution(request) => {
                let global = source.issue_global(request, &cancel).unwrap();
                source
                    .seal_native_execution(request, global, 200, &cancel)
                    .unwrap()
            }
            DomHostedChildStep::MakeActive(request) => {
                source.seal_make_active(request, &cancel).unwrap()
            }
            DomHostedChildStep::FinishLoading(request) => {
                source.seal_finish_loading(request, 300, &cancel).unwrap()
            }
            DomHostedChildStep::InitialEntry(request) => {
                let entry = source.issue_entry(request, &cancel).unwrap();
                let navigable = source.issue_navigable(request, &cancel).unwrap();
                let mut initialization = Some(DomHostedEntryInitialization::new(
                    uuid(u16::from(b'3')),
                    uuid(u16::from(b'4')),
                ));
                source
                    .seal_initial_entry(request, entry, navigable, &mut initialization, &cancel)
                    .unwrap()
            }
            DomHostedChildStep::QueueHistory(request) => {
                operation = Some(request.operation());
                publication = Some(source.seal_child_history(request, &cancel).unwrap());
                break;
            }
            _ => panic!("unexpected child phase before its history handoff"),
        };
        let mut value = Some(value);
        lease
            .accept_hosted_child_publication(&mut source, &child, &mut value, &cancel)
            .unwrap();
        assert!(value.is_none());
    }
    let operation = operation.expect("original constructor reaches QueueHistory");
    assert!(publication.is_some());
    let before = lease.snapshot();
    let graph = lease
        .dom
        .source_operation
        .as_ref()
        .unwrap()
        .draft
        .hosted_history
        .clone()
        .unwrap();
    let revision = source.revision().unwrap();
    let spent = lease.status().spent_work();
    CANCEL_HISTORY.set(true);
    assert_eq!(
        lease
            .accept_hosted_child_history(&mut source, &child, &mut publication, &cancel)
            .unwrap_err(),
        DomHostedError::Dom(DomError::Cancelled),
    );
    assert!(cancel.load(Ordering::Acquire));
    assert!(
        publication.is_some(),
        "failed publication keeps its authentic receipt"
    );
    assert_eq!(source.revision().unwrap(), revision);
    assert_eq!(lease.snapshot().stamp(), before.stamp());
    assert!(Arc::ptr_eq(
        &graph,
        lease
            .dom
            .source_operation
            .as_ref()
            .unwrap()
            .draft
            .hosted_history
            .as_ref()
            .unwrap()
    ));
    assert!(matches!(
        lease.hosted_child_step(&child).unwrap(),
        DomHostedChildStep::QueueHistory(_)
    ));
    assert!(lease.status().spent_work() > spent);

    cancel.store(false, Ordering::Release);
    let queued = lease
        .accept_hosted_child_history(&mut source, &child, &mut publication, &cancel)
        .unwrap();
    assert_eq!(queued.operation(), operation);
    assert!(publication.is_none());
    assert_ne!(lease.snapshot().stamp(), before.stamp());
    crate::store::oracle_tests::assert_full_canonical(lease.dom);
    assert!(matches!(
        lease.hosted_child_step(&child).unwrap(),
        DomHostedChildStep::NeedsUrl(_)
    ));
}
