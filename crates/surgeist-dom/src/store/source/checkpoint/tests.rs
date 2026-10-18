//! Literal outcomes through the original Source command interpreter.
use super::*;

fn fixture() -> (Dom, NodeId, Draft) {
    let cancel = AtomicBool::new(false);
    let limits = DomLimits::new(8, 4096, 64, 0, 0, 16, 1_000_000).unwrap();
    let mut dom = Dom::new(limits).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let text = setup.create_text(&document, "original");
    let commit = dom.apply(setup, &cancel).unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let commands = DomBatch::new();
    let policy = crate::DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 128, 4096, 4096, 4);
    let draft = Draft::new_source(
        &dom,
        commands.owner.clone(),
        &commands.commands,
        policy,
        &cancel,
    )
    .unwrap()
    .0;
    (dom, text, draft)
}

fn replace(draft: &mut Draft, dom: &Dom, text: NodeId, value: &str) {
    let mut edit = DomBatch::new();
    edit.replace_data(text, 0, usize::MAX, value);
    draft
        .apply(&edit.commands[0], dom.limits, &AtomicBool::new(false))
        .unwrap();
}

fn assert_text(draft: &Draft, text: NodeId, expected: &str) {
    let key = draft.resolve(text.into()).unwrap();
    let node = draft.node(key).unwrap();
    let DomNodeKind::Text(value) = &node.kind else {
        panic!("original text record changed kind");
    };
    assert_eq!(value, &DocumentString::from(expected));
}

#[test]
fn nested_savepoints_restore_each_original_same_record_version_without_refunding_work() {
    let cancel = AtomicBool::new(false);
    let (dom, text, mut draft) = fixture();
    let captured = dom.snapshot();
    let outer = draft.source_checkpoint(0, 0, &cancel).unwrap();
    replace(&mut draft, &dom, text, "outer");
    let inner = draft.source_checkpoint(0, 0, &cancel).unwrap();
    replace(&mut draft, &dom, text, "inner");
    assert_text(&draft, text, "inner");
    let spent = draft.source_work.load(Ordering::Relaxed);
    // Rollback is mandatory after refusal/cancellation. It cannot consult the
    // cancelled token or require even one remaining work unit.
    cancel.store(true, Ordering::Relaxed);
    draft.source_limits.as_mut().unwrap().work = spent;
    inner.restore(&mut draft);
    assert_text(&draft, text, "outer");
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    outer.restore(&mut draft);
    assert_text(&draft, text, "original");
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    assert_eq!(dom.snapshot().stamp(), captured.stamp());
    assert_eq!(
        dom.snapshot().node(text).unwrap().data(),
        captured.node(text).unwrap().data()
    );
}

#[test]
fn sealing_inner_savepoint_keeps_outer_original_for_later_rollback() {
    let cancel = AtomicBool::new(false);
    let (dom, text, mut draft) = fixture();
    let outer = draft.source_checkpoint(0, 0, &cancel).unwrap();
    replace(&mut draft, &dom, text, "first");
    let inner = draft.source_checkpoint(0, 0, &cancel).unwrap();
    replace(&mut draft, &dom, text, "second");
    drop(inner);
    assert_text(&draft, text, "second");
    replace(&mut draft, &dom, text, "third");
    let spent = draft.source_work.load(Ordering::Relaxed);
    outer.restore(&mut draft);
    assert_text(&draft, text, "original");
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
}

#[test]
fn original_dialog_interpreter_mark_restores_an_unstarted_method_after_its_event_boundary() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(8, 4096, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let dialog = setup.create_element_ns(&document, crate::Namespace::html(), "dialog");
    let done = dom.apply(setup, &cancel).unwrap();
    let dialog = done.created(&dialog).unwrap().unwrap();
    let mut edit = DomBatch::new();
    edit.show_dialog(dialog);
    let mut draft = Draft::new_source(
        &dom,
        edit.owner.clone(),
        &edit.commands,
        crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 128, 4096, 4096, 4),
        &cancel,
    )
    .unwrap()
    .0;
    assert!(draft.method.is_none());
    let mark = draft.source_checkpoint(0, 0, &cancel).unwrap();
    assert!(matches!(
        draft.apply(&edit.commands[0], dom.limits, &cancel),
        Err(DomError::NeedsEventfulOperation { .. })
    ));
    assert!(matches!(draft.method, Some(method::Frame::Dialog(_))));
    let spent = draft.source_work.load(Ordering::Relaxed);
    cancel.store(true, Ordering::Relaxed);
    mark.restore(&mut draft);
    assert!(draft.method.is_none());
    assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
    assert_eq!(
        draft
            .element(Key::Existing(dialog))
            .unwrap()
            .attribute(&"open".into()),
        None
    );
}
