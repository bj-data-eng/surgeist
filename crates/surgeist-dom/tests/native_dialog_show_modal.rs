//! Native ShowModal entry guards, without an invented active Window.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
fn limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4)
}
#[test]
fn inactive_closed_and_already_open_nonmodal_dialogs_fail_before_dispatch_or_modal_effects() {
    for open in [false, true] {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let dialog = setup.create_element_ns(&document, Namespace::html(), "dialog");
        setup.insert_before(&dialog, &document, None);
        setup.set_dialog_return_value(&dialog, "kept");
        if open {
            setup.set_attribute(&dialog, "open", "");
        }
        let created = dom.apply(setup, &cancel).unwrap();
        let dialog = created.created(&dialog).unwrap().unwrap();
        let before = dom.snapshot();
        {
            let mut lease = dom
                .begin_source_edit(DomSourceEdit::show_modal_dialog(dialog), limits(), &cancel)
                .unwrap();
            assert_eq!(
                lease.advance(&cancel).unwrap_err(),
                DomError::DialogInvalidState
            );
        }
        assert!(dom.pending_source_edit().is_none());
        let current = dom.snapshot();
        assert_eq!(current.stamp(), before.stamp());
        let view = current.node(dialog).unwrap().as_dialog().unwrap();
        assert_eq!(view.return_value(), &DocumentString::from("kept"));
        assert!(!view.is_modal());
        assert!(!view.has_close_watcher());
        assert!(!view.has_toggle_task_tracker());
        assert_eq!(view.request_close_return_value(), None);
        assert_eq!(
            current
                .node(dialog)
                .unwrap()
                .attribute(&"open".into())
                .is_some(),
            open
        );
    }
}
#[test]
fn wrong_family_and_canceled_entry_leave_no_source_guard_or_canonical_changes() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let div = setup.create_element_ns(&document, Namespace::html(), "div");
    let dialog = setup.create_element_ns(&document, Namespace::html(), "dialog");
    let created = dom.apply(setup, &cancel).unwrap();
    let div = created.created(&div).unwrap().unwrap();
    let dialog = created.created(&dialog).unwrap().unwrap();
    let before = dom.snapshot().stamp();
    {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::show_modal_dialog(div), limits(), &cancel)
            .unwrap();
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::DialogInapplicable
        );
    }
    assert!(matches!(
        dom.begin_source_edit(
            DomSourceEdit::show_modal_dialog(dialog),
            limits(),
            &AtomicBool::new(true)
        ),
        Err(DomError::Cancelled)
    ));
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.snapshot().stamp(), before);
}
