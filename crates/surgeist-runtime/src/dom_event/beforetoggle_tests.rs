//! HTML877d Show fires cancelable closed→open beforetoggle; DOM3071's canceled
//! flag is independent of reported listener failures. New private producer
//! construction has no previously callable cancelable-event RED boundary.
use super::*;
use surgeist_html::{HtmlDetailsState, HtmlDetailsToggleEvent, HtmlDialogState};

fn dialog() -> Ordinary {
    let mut dom = dom();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::html(), "div");
    let target = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&parent, &document, None);
    batch.insert_before(&target, &parent, None);
    let commit = apply(&mut dom, batch);
    Ordinary {
        document: commit.document(&document).unwrap(),
        parent: commit.created(&parent).unwrap().unwrap(),
        target: commit.created(&target).unwrap().unwrap(),
        dom,
    }
}
fn show_event(target: NodeId) -> NativeDomEventOwner {
    NativeDomEvent::dialog_before_toggle(
        target,
        HtmlDialogState::Closed,
        HtmlDialogState::Open,
        true,
    )
}
#[derive(Debug, Eq, PartialEq)]
struct Seen {
    label: u8,
    current: DomEventTarget,
    phase: DomEventPhase,
    payload: DomToggleEvent,
    canceled: bool,
    cancelable: bool,
    bubbles: bool,
    composed: bool,
}
fn listener(label: u8, prevent: bool) -> DomEventCallback<Vec<Seen>> {
    DomEventCallback::new(move |_, _, log: &mut Vec<Seen>, event| {
        if prevent {
            event.prevent_default();
        }
        log.push(Seen {
            label,
            current: event.current_target().unwrap(),
            phase: event.phase(),
            payload: *event.toggle_event().unwrap(),
            canceled: event.default_prevented(),
            cancelable: event.cancelable(),
            bubbles: event.bubbles(),
            composed: event.composed(),
        });
        Ok(())
    })
}
fn assert_show_payload(log: &[Seen]) {
    for observation in log {
        assert_eq!(observation.payload.old_state(), "closed");
        assert_eq!(observation.payload.new_state(), "open");
        assert_eq!(observation.payload.source(), None);
        assert!(observation.cancelable);
        assert!(!observation.bubbles && !observation.composed);
    }
}

#[test]
fn show_beforetoggle_capture_and_both_target_invocations_can_cancel() {
    for cancel_at in 0..3 {
        let mut f = dialog();
        let mut registry = registry();
        for (label, node, capture) in [
            (0, f.parent, true),
            (1, f.target, true),
            (2, f.target, false),
        ] {
            registry
                .add_listener(
                    &DomEventDom::borrow(&mut f.dom),
                    node,
                    DomEventType::BeforeToggle,
                    listener(label, label == cancel_at),
                    DomEventListenerOptions {
                        capture,
                        once: false,
                    },
                )
                .unwrap();
        }
        let mut log = Vec::new();
        let event = show_event(f.target);
        assert_eq!(event.event_type().as_str(), "beforetoggle");
        assert!(!event.default_prevented());
        let receipt = complete(
            registry
                .dispatch(
                    &mut f.dom,
                    &mut log,
                    &event,
                    limits(),
                    &AtomicBool::new(false),
                )
                .unwrap(),
        );
        assert_eq!(receipt.event_type(), DomEventType::BeforeToggle);
        assert_eq!(receipt.callbacks(), 3);
        assert!(receipt.failures().is_empty());
        assert!(receipt.default_prevented() && event.default_prevented());
        assert_eq!(log.iter().map(|o| o.label).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(
            log.iter().map(|o| o.canceled).collect::<Vec<_>>(),
            (0..3).map(|index| index >= cancel_at).collect::<Vec<_>>()
        );
        assert_eq!(log[0].current, f.parent.into());
        assert_eq!(log[0].phase, DomEventPhase::Capturing);
        for observation in &log[1..] {
            assert_eq!(observation.current, f.target.into());
            assert_eq!(observation.phase, DomEventPhase::AtTarget);
        }
        assert_show_payload(&log);
    }
}

#[test]
fn noncancelable_details_toggle_ignores_prevention_and_keeps_exact_payload() {
    let mut f = ordinary();
    let mut batch = DomBatch::new();
    let details = batch.create_element_ns(f.document, Namespace::html(), "details");
    batch.insert_before(&details, f.parent, None);
    let commit = apply(&mut f.dom, batch);
    let details = commit.created(&details).unwrap().unwrap();
    let mut registry = registry();
    registry
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            details,
            DomEventType::Toggle,
            listener(0, true),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = NativeDomEvent::details_toggle(
        details,
        HtmlDetailsToggleEvent::queued(HtmlDetailsState::Closed, HtmlDetailsState::Closed, None),
    );
    let mut log = Vec::new();
    let receipt = complete(
        registry
            .dispatch(
                &mut f.dom,
                &mut log,
                &event,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap(),
    );
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented() && !event.default_prevented());
    assert_eq!(log.len(), 1);
    assert!(!log[0].cancelable && !log[0].canceled);
    assert_eq!(log[0].payload.old_state(), "closed");
    assert_eq!(log[0].payload.new_state(), "closed");
    assert_eq!(log[0].payload.source(), None);
}

#[test]
fn retained_beforetoggle_keeps_cancellation_and_never_replays_delivered_capture() {
    let mut f = dialog();
    let mut registry = registry();
    for (label, node, capture) in [
        (0, f.parent, true),
        (1, f.target, true),
        (2, f.target, false),
    ] {
        registry
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                node,
                DomEventType::BeforeToggle,
                listener(label, label == 0),
                DomEventListenerOptions {
                    capture,
                    once: false,
                },
            )
            .unwrap();
    }
    let mut log = Vec::new();
    let event = show_event(f.target);
    assert!(matches!(
        registry
            .dispatch(
                &mut f.dom,
                &mut log,
                &event,
                DomEventDispatchLimits {
                    callbacks: 1,
                    ..limits()
                },
                &AtomicBool::new(false)
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    assert_eq!(log.iter().map(|o| o.label).collect::<Vec<_>>(), [0]);
    assert!(event.default_prevented());
    assert!(matches!(
        registry
            .resume(
                &mut f.dom,
                &mut log,
                &event,
                DomEventDispatchLimits {
                    callbacks: 0,
                    ..limits()
                },
                &AtomicBool::new(false)
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    assert_eq!(log.len(), 1);
    assert!(event.default_prevented());
    let receipt = complete(
        registry
            .resume(
                &mut f.dom,
                &mut log,
                &event,
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap(),
    );
    assert_eq!(receipt.callbacks(), 3);
    assert!(receipt.failures().is_empty());
    assert!(receipt.default_prevented());
    assert_eq!(log.iter().map(|o| o.label).collect::<Vec<_>>(), [0, 1, 2]);
    assert!(log.iter().all(|observation| observation.canceled));
    assert_show_payload(&log);
}

#[test]
fn beforetoggle_listener_error_and_unwind_are_independent_from_cancellation() {
    for cancel in [false, true] {
        let mut f = dialog();
        let mut registry = registry();
        registry
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |_, _, _: &mut Vec<Seen>, event| {
                    if cancel {
                        event.prevent_default();
                    }
                    Err(DomEventCallbackError { code: 17 })
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        registry
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::BeforeToggle,
                DomEventCallback::new(|_, _, _: &mut Vec<Seen>, _| {
                    panic!("actual listener unwind")
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        registry
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::BeforeToggle,
                listener(2, false),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let event = show_event(f.target);
        let mut log = Vec::new();
        let receipt = complete(
            registry
                .dispatch(
                    &mut f.dom,
                    &mut log,
                    &event,
                    limits(),
                    &AtomicBool::new(false),
                )
                .unwrap(),
        );
        assert_eq!(receipt.callbacks(), 3);
        assert_eq!(
            receipt
                .failures()
                .iter()
                .map(|failure| failure.kind)
                .collect::<Vec<_>>(),
            [
                DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 17 }),
                DomEventListenerFailureKind::Panicked,
            ]
        );
        assert_eq!(receipt.default_prevented(), cancel);
        assert_eq!(event.default_prevented(), cancel);
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].canceled, cancel);
        assert_show_payload(&log);
    }
}
