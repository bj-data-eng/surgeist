//! HTML877d request-close early-return guards through the public Source owner.
//! These native no-Global cases do not attest hosted watcher/event execution.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4)
}

#[test]
fn closed_disconnected_and_inactive_dialogs_return_before_request_state_or_services() {
    // Source46713–46715: open precedes connectedness, which precedes activity.
    for (open, connected) in [(false, true), (true, false), (true, true)] {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                100_000,
                64,
                0,
                0,
                16,
                1_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let dialog = setup.create_element_ns(&document, Namespace::html(), "dialog");
        setup.set_dialog_return_value(&dialog, "kept");
        if open {
            setup.set_attribute(&dialog, "open", "");
        }
        if connected {
            setup.insert_before(&dialog, &document, None);
        }
        let created = dom.apply(setup, &cancel).unwrap();
        let dialog = created.created(&dialog).unwrap().unwrap();
        let before = dom.snapshot();
        let completion = {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::request_close_dialog(dialog, Some("unused".into())),
                    source_limits(),
                    &cancel,
                )
                .unwrap();
            let DomSourceProgress::Complete(completion) = lease.advance(&cancel).unwrap() else {
                panic!("guard must return without requesting event or watcher service");
            };
            completion
        };
        assert!(completion.commit().changes().changes().is_empty());
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(dom.pending_source_edit().is_none());
        let snapshot = dom.snapshot();
        let current = snapshot.node(dialog).unwrap().as_dialog().unwrap();
        assert_eq!(current.return_value(), &DocumentString::from("kept"));
        assert_eq!(current.request_close_return_value(), None);
        assert_eq!(current.request_close_source(), None);
        assert!(!current.force_close_watcher_enabled());
        assert!(!current.has_close_watcher());
    }
}

#[test]
fn request_close_rejects_a_non_dialog_without_retaining_an_unpublished_operation() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            16,
            100_000,
            32,
            0,
            0,
            8,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let node = setup.create_element_ns(&document, Namespace::html(), "div");
    let created = dom.apply(setup, &cancel).unwrap();
    let node = created.created(&node).unwrap().unwrap();
    let before = dom.snapshot().stamp();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::request_close_dialog(node, None),
                source_limits(),
                &cancel,
            )
            .unwrap();
        assert_eq!(
            lease.advance(&cancel).unwrap_err(),
            DomError::DialogInapplicable
        );
    }
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(dom.snapshot().stamp(), before);
}
