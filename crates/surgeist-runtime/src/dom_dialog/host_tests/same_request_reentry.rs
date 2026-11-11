//! Draft owning test. Place under dom_dialog/host_tests after review freeze.
use super::*;
use crate::DomSourceRuntimeError;

#[derive(Default)]
struct Context {
    request: Option<DomSourceEventRequest>,
    calls: Vec<&'static str>,
    repeats: Vec<Result<DomSourceEventServiceProgress, DomSourceRuntimeError>>,
}

fn attempt(storage_bytes: usize, repeat_request: bool) -> bool {
    let cancel = AtomicBool::new(false);
    let (mut session, _, target, _) = fixture_with_source_storage(storage_bytes);
    let mut source = session.source_execution();
    let nested = session.source_execution();
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 2,
        pending_events: 1,
        nested_dispatches: 2,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::new(move |dom, dispatcher, context: &mut Context, event| {
                let first = context.calls.is_empty();
                context.calls.push("prefix");
                if repeat_request && first {
                    let request = context.request.as_ref().unwrap().clone();
                    let stamp = dom.snapshot().stamp();
                    for _ in 0..2 {
                        let outcome = nested.service_event(
                            dom,
                            &request,
                            dispatcher,
                            context,
                            limits(),
                            &cancel,
                        );
                        context.repeats.push(outcome);
                        assert_eq!(dom.snapshot().stamp(), stamp);
                        assert_eq!(
                            dom.pending_source_edit().unwrap().operation(),
                            request.operation()
                        );
                        assert_eq!(event.current_target(), Some(target.into()));
                        assert_eq!(event.phase(), DomEventPhase::AtTarget);
                    }
                }
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::BEFORE_TOGGLE,
            DomEventCallback::new(|_, _, context: &mut Context, event| {
                context.calls.push("suffix");
                assert!(event.default_prevented());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let request = show(&mut source, &mut session, target);
    let mut context = Context {
        request: Some(request.clone()),
        ..Context::default()
    };
    let cancel = AtomicBool::new(false);
    let receipt = match source.service_event(
        &mut session.dom(),
        &request,
        &mut registry,
        &mut context,
        limits(),
        &cancel,
    ) {
        Err(DomSourceRuntimeError::SourceEvent(DomSourceEventRuntimeError::Admission(
            DomSourceEventResource::StorageBytes,
        ))) => {
            assert!(context.calls.is_empty());
            assert_eq!(
                session.dom().pending_source_edit().unwrap().operation(),
                request.operation()
            );
            return false;
        }
        Ok(DomSourceEventServiceProgress::Serviced { receipt }) => receipt,
        other => panic!("the actual original Show dispatch must finish: {other:?}"),
    };
    assert_eq!(receipt.original_target(), target);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(receipt.default_prevented());
    assert_eq!(context.calls, ["prefix", "suffix"]);
    assert_eq!(context.repeats.len(), if repeat_request { 2 } else { 0 });
    let pending = session.dom().pending_source_edit().unwrap();
    assert_eq!(pending.operation(), request.operation());
    assert_eq!(
        pending.phase(),
        surgeist_dom::DomSourcePhase::EventReceiptAccepted
    );
    canceled_tail(&mut source, &mut session, &request);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(context.calls, ["prefix", "suffix"]);
    for outcome in context.repeats {
        assert!(
            matches!(
                outcome,
                Err(DomSourceRuntimeError::SourceEvent(
                    DomSourceEventRuntimeError::Dom(DomError::InvalidSourceEvent)
                ))
            ),
            "same live request must reach its actual Source scope refusal: {outcome:?}"
        );
    }
    true
}

#[test]
fn exact_storage_same_request_reentry_preserves_original_source_scope_refusal() {
    // Find the smallest owner cap for the same real dispatch with no recursive
    // request. This places the cut independently of the recursive error oracle.
    let mut refused = State::fixed_bytes();
    let mut admitted = 65_536;
    assert!(!attempt(refused, false));
    assert!(attempt(admitted, false));
    assert!(attempt(admitted, true));
    while admitted - refused > 1 {
        let bytes = refused + (admitted - refused) / 2;
        if attempt(bytes, false) {
            admitted = bytes;
        } else {
            refused = bytes;
        }
    }
    assert!(!attempt(admitted - 1, false));
    assert!(attempt(admitted, true));
}
