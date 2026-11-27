use super::*;
use crate::limits::DomNameLimits;
use surgeist_html::values::Namespace;

fn stage_previous_focus(
    draft: &mut Draft,
    receiver: Key,
    target: Key,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    draft.materialize_node(receiver, cancel)?;
    let undo = draft.prepare_undo(cancel)?;
    let debit = draft
        .custody_ledger()
        .retain(custody::Usage::ZERO, |usage| {
            draft.admit_custody_usage(usage, cancel)
        })?;
    draft.commit_selected(
        receiver,
        custody::Held::new(FocusVersion::Staged(target), debit),
        |value| Overlay::Focus(value),
        |_, _, staged, _| {
            let (value, debit) = staged.into_parts();
            drop(debit);
            Ok(FieldUndo::Focus(value))
        },
        undo,
        cancel,
    )
}

fn observed_focus(snapshot: &DomSnapshot, node: NodeId) -> Option<NodeId> {
    snapshot
        .node(node)
        .unwrap()
        .as_element()
        .unwrap()
        .common_state()
        .html()
        .unwrap()
        .previously_focused_element()
        .copied()
}

#[test]
fn retiring_an_exact_previous_focus_target_clears_only_its_surviving_referrers() {
    let cancel = AtomicBool::new(false);
    let mut dom = focus_dom();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let tickets = ["dialog", "dialog", "div", "button", "button"]
        .map(|tag| batch.create_element_ns(&doc, Namespace::html(), tag));
    let made = dom.apply(batch, &cancel).unwrap();
    let [left, right, unrelated, target, other] =
        tickets.map(|ticket| made.created(&ticket).unwrap().unwrap());
    dom.edit_native_state(&cancel, |draft| {
        for (receiver, remembered) in [(left, target), (right, target), (unrelated, other)] {
            stage_previous_focus(
                draft,
                Key::Existing(receiver),
                Key::Existing(remembered),
                &cancel,
            )?;
        }
        Ok(())
    })
    .unwrap();
    let before = dom.snapshot();
    assert_eq!(observed_focus(&before, left), Some(target));
    assert_eq!(observed_focus(&before, right), Some(target));
    assert_eq!(observed_focus(&before, unrelated), Some(other));

    let mut retire = DomBatch::new();
    retire.destroy(target);
    let committed = dom.apply(retire, &cancel).unwrap();
    let after = dom.snapshot();
    assert!(matches!(after.node(target), Err(DomError::StaleNode)));
    assert_eq!(observed_focus(&after, left), None);
    assert_eq!(observed_focus(&after, right), None);
    assert_eq!(observed_focus(&after, unrelated), Some(other));
    assert_eq!(observed_focus(&before, left), Some(target));
    assert_eq!(observed_focus(&before, right), Some(target));
    assert_eq!(committed.changes().from(), before.stamp());
    assert_eq!(committed.changes().to(), after.stamp());
    assert!(
        committed
            .changes()
            .changes()
            .contains(&DomChange::ElementState(left))
    );
    assert!(
        committed
            .changes()
            .changes()
            .contains(&DomChange::ElementState(right))
    );
    assert!(
        !committed
            .changes()
            .changes()
            .contains(&DomChange::ElementState(unrelated))
    );
}

fn focus_dom() -> Dom {
    Dom::new(
        DomLimits::new(
            32,
            4096,
            128,
            32,
            32,
            16,
            250_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn retiring_a_staged_previous_focus_target_does_not_require_a_public_id() {
    let cancel = AtomicBool::new(false);
    let mut dom = focus_dom();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let receiver = setup.create_element_ns(&document, Namespace::html(), "dialog");
    let committed = dom.apply(setup, &cancel).unwrap();
    let document = committed.document(&document).unwrap();
    let receiver = committed.created(&receiver).unwrap().unwrap();
    let before = dom.snapshot();

    let mut batch = DomBatch::new();
    let target = batch.create_element_ns(document, Namespace::html(), "button");
    batch.destroy(&target);
    let mut draft = Draft::new(&dom, batch.owner.clone(), &cancel).unwrap();
    draft
        .apply(&batch.commands[0], dom.limits(), &cancel)
        .unwrap();
    let key = draft.resolve((&target).into()).unwrap();
    assert!(matches!(key, Key::Created(_)));
    assert_eq!(draft.published_id(key), None);
    let receiver_key = Key::Existing(receiver);
    stage_previous_focus(&mut draft, receiver_key, key, &cancel).unwrap();
    let staged_focus = |draft: &Draft| {
        draft
            .node(receiver_key)
            .unwrap()
            .element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element()
            .copied()
    };
    assert_eq!(staged_focus(&draft), Some(key));
    draft
        .apply(&batch.commands[1], dom.limits(), &cancel)
        .unwrap();
    assert!(matches!(draft.node(key), Err(DomError::StaleNode)));
    assert_eq!(staged_focus(&draft), None);

    let prepared = draft.finish(&dom, &cancel).unwrap();
    let commit = dom.publish(prepared, &cancel).unwrap();
    assert_eq!(commit.created(&target).unwrap(), None);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(observed_focus(&dom.snapshot(), receiver), None);
}
