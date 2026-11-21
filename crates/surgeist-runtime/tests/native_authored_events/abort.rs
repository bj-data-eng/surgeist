//! Abort uses the live native listener generations and the original dispatcher.

use super::*;
use std::{any::Any, rc::Rc};
use surgeist_runtime::event::{
    DomAbortController, DomAbortDefaultReason, DomAbortError, DomAbortProgress, DomAbortReason,
    DomAbortValue, DomEventCallbackError, DomEventDispatchError, DomEventListenerFailureKind,
    DomEventResource, DomSignalListenerRegistration,
};

type Trace = Vec<&'static str>;

fn controller<C>(
    session: &mut DomSession,
    registry: &mut DomEventDispatcher<C>,
    origin: DomHostedTopLevel,
    algorithm_limit: usize,
) -> DomAbortController<C> {
    session
        .hosted_execution()
        .create_abort_controller(
            &mut session.dom(),
            registry,
            origin.global(),
            algorithm_limit,
            &cancel(),
        )
        .unwrap()
}

fn recording(label: &'static str) -> DomEventCallback<Trace> {
    DomEventCallback::new(move |_, _, calls: &mut Trace, _| {
        calls.push(label);
        Ok(())
    })
}

fn begin_abort<C>(
    controller: &DomAbortController<C>,
    session: &mut DomSession,
    registry: &mut DomEventDispatcher<C>,
    context: &mut C,
    reason: Option<DomAbortValue>,
) -> DomAbortProgress {
    controller
        .abort(
            &mut session.dom(),
            registry,
            context,
            reason,
            limits(),
            &cancel(),
        )
        .unwrap()
}

fn delivered(progress: DomAbortProgress) -> DomEventDispatchReceipt {
    let DomAbortProgress::Delivery(outcome) = progress else {
        panic!("first abort must drive its real event");
    };
    let receipt = complete(outcome);
    assert!(receipt.failures().is_empty());
    receipt
}

fn default_reason<C>(controller: &DomAbortController<C>) -> DomAbortDefaultReason {
    let Some(DomAbortReason::AbortError(reason)) = controller.signal().reason().unwrap() else {
        panic!("omitted native abort value must establish one default AbortError");
    };
    reason
}

#[test]
fn real_abort_keeps_reason_identity_and_removes_its_own_listener_before_firing() {
    let mut session = session();
    let origin = top(&mut session);
    let mut registry = new_registry::<Trace>();
    let controller = controller(&mut session, &mut registry, origin, 1);
    let signal = controller.signal();
    let target = signal.event_target();
    assert_eq!(signal, controller.signal());
    assert_eq!(target, signal.clone().event_target());
    assert!(!signal.aborted().unwrap());
    assert!(signal.reason().unwrap().is_none());
    assert_eq!(target.relevant_global(), origin.global());

    let clock = session
        .hosted_execution()
        .settings(&session.dom(), origin.global(), &cancel())
        .unwrap();
    let global = origin.global();
    let source = clock.clock_source();
    let time_origin = clock.time_origin_microseconds();
    let supplied: Rc<dyn Any> = Rc::new(String::from("original native abort reason"));
    let expected = supplied.clone();
    let selected = signal.clone();
    let event_target: DomEventTarget = target.clone().into();
    let expected_target = event_target.clone();
    let name = units("abort");
    registry
        .add_listener(
            &mut session.dom(),
            target.clone(),
            &name,
            DomEventCallback::new(move |_, _, calls: &mut Trace, event| {
                assert_eq!(event.event_type().code_units(), units("abort"));
                assert_eq!(event.target(), Some(expected_target.clone()));
                assert_eq!(event.current_target(), Some(expected_target.clone()));
                assert_eq!(event.phase(), DomEventPhase::AtTarget);
                if !event.is_trusted() {
                    assert!(!selected.aborted().unwrap());
                    assert!(selected.reason().unwrap().is_none());
                    calls.push("authored");
                    return Ok(());
                }
                assert!(!event.bubbles());
                assert!(!event.cancelable());
                assert!(!event.composed());
                assert!(selected.aborted().unwrap());
                let Some(DomAbortReason::Explicit(DomAbortValue::Native(observed))) =
                    selected.reason().unwrap()
                else {
                    panic!("native reason must already be established at the abort callback");
                };
                assert!(Rc::ptr_eq(&observed, &expected));
                let creation = event.creation().unwrap();
                assert_eq!(creation.global(), global);
                assert_eq!(creation.clock_source(), source);
                assert_eq!(creation.time_origin_microseconds(), time_origin);
                assert!(event.time_stamp().unwrap().is_finite());
                calls.push("trusted");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let authored = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            global,
            &name,
            DomEventInit::default(),
            &cancel(),
        )
        .unwrap();
    let mut calls = Trace::new();
    let receipt = fire(
        &mut session,
        &mut registry,
        target.clone(),
        &authored,
        &mut calls,
    );
    assert!(receipt.failures().is_empty());
    assert_eq!(calls, ["authored"]);
    assert!(!signal.aborted().unwrap());

    let registration = registry.add_listener_with_signal(
        &mut session.dom(),
        target,
        &name,
        recording("removed before abort"),
        DomEventListenerOptions::default(),
        &signal,
    );
    assert!(matches!(
        registration,
        Ok(DomSignalListenerRegistration::Inserted(_))
    ));
    let receipt = delivered(begin_abort(
        &controller,
        &mut session,
        &mut registry,
        &mut calls,
        Some(DomAbortValue::Native(supplied.clone())),
    ));
    assert_eq!(receipt.original_target(), event_target);
    assert_eq!(receipt.event_type().code_units(), name);
    assert_eq!(receipt.callbacks(), 1);
    assert!(!receipt.default_prevented());
    assert_eq!(calls, ["authored", "trusted"]);

    assert!(matches!(
        begin_abort(
            &controller,
            &mut session,
            &mut registry,
            &mut calls,
            Some(DomAbortValue::Null)
        ),
        DomAbortProgress::AlreadyAborted
    ));
    assert_eq!(calls, ["authored", "trusted"]);
    let Some(DomAbortReason::Explicit(DomAbortValue::Native(observed))) = signal.reason().unwrap()
    else {
        panic!("repeated abort changed the kind of the original reason");
    };
    assert!(Rc::ptr_eq(&observed, &supplied));
}

#[test]
fn default_reason_is_one_stable_record_for_each_signal() {
    let mut session = session();
    let origin = top(&mut session);
    let mut registry = new_registry::<()>();
    let first = controller(&mut session, &mut registry, origin, 0);
    let second = controller(&mut session, &mut registry, origin, 0);
    assert_ne!(first.signal(), second.signal());
    assert_ne!(
        first.signal().event_target(),
        second.signal().event_target()
    );
    for value in [&first, &second] {
        let receipt = delivered(begin_abort(
            value,
            &mut session,
            &mut registry,
            &mut (),
            None,
        ));
        assert_eq!(
            receipt.original_target(),
            value.signal().event_target().into()
        );
        assert_eq!(receipt.callbacks(), 0);
    }
    let reason = default_reason(&first);
    assert_eq!(reason.global(), origin.global());
    assert_eq!(reason, default_reason(&first));
    assert_ne!(reason, default_reason(&second));
    assert!(matches!(
        begin_abort(
            &first,
            &mut session,
            &mut registry,
            &mut (),
            Some(DomAbortValue::Null)
        ),
        DomAbortProgress::AlreadyAborted
    ));
    assert_eq!(reason, default_reason(&first));
}

#[test]
fn duplicate_signal_step_uses_capacity_without_claiming_the_original_listener() {
    let mut session = session();
    let origin = top(&mut session);
    let target = body(&session.snapshot(), origin.document());
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 4_000_000,
    })
    .unwrap();
    let original = controller(&mut session, &mut registry, origin, 1);
    let duplicate = controller(&mut session, &mut registry, origin, 1);
    let spare = controller(&mut session, &mut registry, origin, 1);
    let callback = recording("original");
    let flags = DomEventListenerOptions::default();
    let name = units("probe");
    let other = units("other");
    let DomSignalListenerRegistration::Inserted(id) = registry
        .add_listener_with_signal(
            &mut session.dom(),
            target,
            &name,
            callback.clone(),
            flags,
            &original.signal(),
        )
        .unwrap()
    else {
        panic!("first actual listener");
    };
    let once = DomEventListenerOptions {
        once: true,
        ..flags
    };
    assert!(matches!(
        registry.add_listener_with_signal(
            &mut session.dom(), target, &name, callback.clone(), once, &duplicate.signal(),
        ),
        Ok(DomSignalListenerRegistration::Existing(existing)) if existing == id
    ));
    // There is no listener append in either call. The signal-backed duplicate's
    // separate occurrence carries the only abort step, while an ordinary duplicate still succeeds.
    assert_eq!(
        registry
            .add_listener(&mut session.dom(), target, &name, callback.clone(), flags)
            .unwrap(),
        id,
    );
    assert!(matches!(
        registry.add_listener_with_signal(
            &mut session.dom(),
            target,
            &name,
            callback.clone(),
            once,
            &duplicate.signal(),
        ),
        Err(DomEventDispatchError::Admission(
            DomEventResource::Registrations
        ))
    ));
    assert!(matches!(
        registry.add_listener_with_signal(
            &mut session.dom(),
            target,
            &other,
            callback.clone(),
            flags,
            &spare.signal(),
        ),
        Err(DomEventDispatchError::Admission(
            DomEventResource::Registrations
        ))
    ));

    let mut calls = Trace::new();
    delivered(begin_abort(
        &duplicate,
        &mut session,
        &mut registry,
        &mut calls,
        Some(DomAbortValue::Null),
    ));
    assert!(matches!(
        duplicate.signal().reason().unwrap(),
        Some(DomAbortReason::Explicit(DomAbortValue::Null))
    ));
    assert!(matches!(
        registry.add_listener_with_signal(
            &mut session.dom(),
            target,
            &other,
            callback.clone(),
            flags,
            &duplicate.signal(),
        ),
        Ok(DomSignalListenerRegistration::SignalAborted)
    ));
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            origin.global(),
            &name,
            DomEventInit::default(),
            &cancel(),
        )
        .unwrap();
    for _ in 0..2 {
        assert_eq!(
            fire(&mut session, &mut registry, target, &event, &mut calls).callbacks(),
            1
        );
    }
    assert_eq!(calls, ["original", "original"]);
    delivered(begin_abort(
        &original,
        &mut session,
        &mut registry,
        &mut calls,
        None,
    ));
    assert_eq!(
        fire(&mut session, &mut registry, target, &event, &mut calls).callbacks(),
        0
    );
    assert!(!registry.remove_listener(id));

    // Refusal at the full listener list left the spare signal's sole step free.
    let DomSignalListenerRegistration::Inserted(old) = registry
        .add_listener_with_signal(
            &mut session.dom(),
            target,
            &name,
            callback.clone(),
            flags,
            &spare.signal(),
        )
        .unwrap()
    else {
        panic!("freed slot and still unused signal step");
    };
    assert!(registry.remove_listener(old));
    let replacement = registry
        .add_listener(&mut session.dom(), target, &name, callback, flags)
        .unwrap();
    assert_ne!(replacement, old);
    delivered(begin_abort(
        &spare,
        &mut session,
        &mut registry,
        &mut calls,
        None,
    ));
    assert!(!registry.remove_listener(old));
    assert_eq!(
        fire(&mut session, &mut registry, target, &event, &mut calls).callbacks(),
        1
    );
    assert_eq!(calls, ["original", "original", "original"]);
    assert!(registry.remove_listener(replacement));
}

#[test]
fn synchronous_nested_abort_skips_the_removed_outer_snapshot_entry() {
    nested_abort(false);
}

#[test]
fn parked_abort_finishes_before_the_original_outer_cursor_can_advance() {
    nested_abort(true);
}

fn nested_abort(retain: bool) {
    let mut session = session();
    let origin = top(&mut session);
    let target = body(&session.snapshot(), origin.document());
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 2,
        nested_dispatches: 2,
        storage_bytes: 16_000_000,
        work: 4_000_000,
    })
    .unwrap();
    let controller = Rc::new(controller(&mut session, &mut registry, origin, 1));
    let signal = controller.signal();
    let flags = DomEventListenerOptions::default();
    let outer = units("outer");
    let abort = units("abort");
    let original = controller.clone();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            &outer,
            DomEventCallback::new(move |dom, registry, calls: &mut Trace, _| {
                calls.push("outer first");
                let limits = DomEventDispatchLimits {
                    callbacks: if retain { 1 } else { limits().callbacks },
                    ..limits()
                };
                let result = original
                    .abort(
                        dom,
                        registry,
                        calls,
                        Some(DomAbortValue::Null),
                        limits,
                        &cancel(),
                    )
                    .unwrap();
                if retain {
                    assert!(matches!(
                        result,
                        DomAbortProgress::Delivery(DomEventDispatchOutcome::Retained(
                            DomEventResource::Callbacks
                        ))
                    ));
                } else {
                    assert_eq!(delivered(result).callbacks(), 2);
                }
                calls.push("outer resumed");
                Ok(())
            }),
            flags,
        )
        .unwrap();
    assert!(matches!(
        registry
            .add_listener_with_signal(
                &mut session.dom(),
                target,
                &outer,
                recording("removed"),
                flags,
                &signal,
            )
            .unwrap(),
        DomSignalListenerRegistration::Inserted(_)
    ));
    registry
        .add_listener(
            &mut session.dom(),
            target,
            &outer,
            recording("outer tail"),
            flags,
        )
        .unwrap();
    let target_signal = signal.event_target();
    registry
        .add_listener(
            &mut session.dom(),
            target_signal.clone(),
            &abort,
            recording("abort prefix"),
            flags,
        )
        .unwrap();
    let suffix = registry
        .add_listener(
            &mut session.dom(),
            target_signal,
            &abort,
            DomEventCallback::new(move |_, _, calls: &mut Trace, _| {
                calls.push("abort suffix");
                if retain {
                    Err(DomEventCallbackError { code: 17 })
                } else {
                    Ok(())
                }
            }),
            flags,
        )
        .unwrap();

    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            origin.global(),
            &outer,
            DomEventInit::default(),
            &cancel(),
        )
        .unwrap();
    let mut invocation = registry
        .prepare_dispatch(&mut session.dom(), target, &event, &cancel())
        .unwrap();
    let mut calls = Trace::new();
    let first = invocation.advance(
        &mut session.dom(),
        &mut registry,
        &mut calls,
        DomEventDispatchLimits {
            callbacks: if retain { 1 } else { limits().callbacks },
            ..limits()
        },
        &cancel(),
    );
    if retain {
        assert!(
            matches!(first, Ok(DomEventDispatchOutcome::RetainedAbortDelivery)),
            "{first:?}"
        );
        assert!(signal.take_abort_receipt().unwrap().is_none());
        assert!(matches!(
            signal.resume_abort(
                &mut session.dom(),
                &mut registry,
                &mut calls,
                limits(),
                &cancel()
            ),
            Err(DomAbortError::Event(
                DomEventDispatchError::WrongContinuation
            ))
        ));
        assert_eq!(calls, ["outer first", "abort prefix", "outer resumed"]);
        assert!(matches!(
            signal.reason().unwrap(),
            Some(DomAbortReason::Explicit(DomAbortValue::Null))
        ));
        let mut foreign = new_registry::<Trace>();
        assert!(
            signal
                .resume_abort(
                    &mut session.dom(),
                    &mut foreign,
                    &mut calls,
                    limits(),
                    &cancel()
                )
                .is_err()
        );
        assert!(matches!(
            begin_abort(&controller, &mut session, &mut registry, &mut calls, None),
            DomAbortProgress::AlreadyAborted
        ));
        assert_eq!(calls, ["outer first", "abort prefix", "outer resumed"]);
        let receipt = complete(
            invocation
                .advance(
                    &mut session.dom(),
                    &mut registry,
                    &mut calls,
                    limits(),
                    &cancel(),
                )
                .unwrap(),
        );
        assert!(receipt.failures().is_empty());
        assert_eq!(receipt.callbacks(), 2);
        let nested_receipt = signal.take_abort_receipt().unwrap().unwrap();
        assert_eq!(
            nested_receipt.original_target(),
            signal.event_target().into()
        );
        assert_eq!(nested_receipt.callbacks(), 2);
        assert_eq!(nested_receipt.failures().len(), 1);
        assert_eq!(nested_receipt.failures()[0].registration, suffix);
        assert_eq!(
            nested_receipt.failures()[0].kind,
            DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 17 })
        );
        assert!(signal.take_abort_receipt().unwrap().is_none());
        assert_eq!(
            calls,
            [
                "outer first",
                "abort prefix",
                "outer resumed",
                "abort suffix",
                "outer tail"
            ]
        );
    } else {
        let receipt = complete(first.unwrap());
        assert!(receipt.failures().is_empty());
        assert_eq!(receipt.callbacks(), 2);
        assert_eq!(
            calls,
            [
                "outer first",
                "abort prefix",
                "abort suffix",
                "outer resumed",
                "outer tail"
            ]
        );
    }
    assert!(signal.aborted().unwrap());
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
}

#[test]
fn refused_transition_keeps_listener_and_reason_then_accepted_abort_releases_the_exact_step() {
    let mut session = session();
    let origin = top(&mut session);
    let mut registry = new_registry::<Trace>();
    let controller = controller(&mut session, &mut registry, origin, 1);
    let signal = controller.signal();
    let kind = units("abort");
    let flags = DomEventListenerOptions::default();
    let removed = registry
        .add_listener_with_signal(
            &mut session.dom(),
            signal.event_target(),
            &kind,
            recording("removed"),
            flags,
            &signal,
        )
        .unwrap();
    assert!(matches!(
        removed,
        DomSignalListenerRegistration::Inserted(_)
    ));
    let mut calls = Trace::new();
    for (limit, cancelled) in [(1, false), (2, true)] {
        let result = controller.abort(
            &mut session.dom(),
            &mut registry,
            &mut calls,
            Some(DomAbortValue::Null),
            DomEventDispatchLimits {
                work: limit,
                ..limits()
            },
            &AtomicBool::new(cancelled),
        );
        if cancelled {
            assert!(
                matches!(
                    result,
                    Err(DomAbortError::Event(DomEventDispatchError::Dom(
                        surgeist_dom::DomError::Cancelled
                    )))
                ),
                "{result:?}"
            );
        } else {
            assert!(
                matches!(
                    result,
                    Err(DomAbortError::Event(DomEventDispatchError::Admission(
                        DomEventResource::Work
                    )))
                ),
                "{result:?}"
            );
        }
        assert!(!signal.aborted().unwrap());
        assert!(signal.reason().unwrap().is_none());
        assert!(calls.is_empty());
    }
    let accepted = controller.abort(
        &mut session.dom(),
        &mut registry,
        &mut calls,
        Some(DomAbortValue::Null),
        DomEventDispatchLimits {
            work: 2,
            ..limits()
        },
        &cancel(),
    );
    assert!(
        matches!(
            accepted,
            Err(DomAbortError::Event(DomEventDispatchError::Admission(
                DomEventResource::Work
            )))
        ),
        "{accepted:?}"
    );
    assert!(signal.aborted().unwrap());
    let DomSignalListenerRegistration::Inserted(removed) = removed else {
        unreachable!()
    };
    assert!(!registry.remove_listener(removed));
    let result = signal
        .resume_abort(
            &mut session.dom(),
            &mut registry,
            &mut calls,
            limits(),
            &cancel(),
        )
        .unwrap();
    assert_eq!(complete(result).callbacks(), 0);
    assert!(calls.is_empty());
}

#[test]
fn dispatcher_teardown_releases_the_detached_duplicate_callback_even_with_a_live_signal() {
    let mut session = session();
    let origin = top(&mut session);
    let mut registry = new_registry::<Trace>();
    let controller = controller(&mut session, &mut registry, origin, 1);
    let signal = controller.signal();
    let owner = Rc::new(());
    let observed = Rc::downgrade(&owner);
    let captured_signal = signal.clone();
    let callback = DomEventCallback::new(move |_, _, _: &mut Trace, _| {
        std::hint::black_box((&captured_signal, &owner));
        Ok(())
    });
    let event = units("test");
    let target = signal.event_target();
    let flags = DomEventListenerOptions::default();
    let original = registry
        .add_listener(
            &mut session.dom(),
            target.clone(),
            &event,
            callback.clone(),
            flags,
        )
        .unwrap();
    assert_eq!(
        registry
            .add_listener_with_signal(&mut session.dom(), target, &event, callback, flags, &signal)
            .unwrap(),
        DomSignalListenerRegistration::Existing(original),
    );
    assert!(registry.remove_listener(original));
    assert!(observed.upgrade().is_some());
    drop(registry);
    assert!(observed.upgrade().is_none());
    assert!(!signal.aborted().unwrap());
    assert!(signal.reason().unwrap().is_none());
    let mut replacement = new_registry::<Trace>();
    assert!(matches!(
        controller.abort(
            &mut session.dom(),
            &mut replacement,
            &mut Trace::new(),
            None,
            limits(),
            &cancel()
        ),
        Err(DomAbortError::Event(
            DomEventDispatchError::WrongTargetRegistry
        ))
    ));
}

#[test]
fn nested_limits_refuse_the_abort_before_publishing_any_reason() {
    for (nesting, pending, refused) in [
        (1, 2, DomEventResource::NestedDispatches),
        (2, 1, DomEventResource::PendingEvents),
    ] {
        nested_refusal(nesting, pending, refused);
    }
}

fn nested_refusal(nesting: usize, pending: usize, refused: DomEventResource) {
    let mut session = session();
    let origin = top(&mut session);
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: pending,
        nested_dispatches: nesting,
        storage_bytes: 16_000_000,
        work: 4_000_000,
    })
    .unwrap();
    let controller = Rc::new(controller(&mut session, &mut registry, origin, 0));
    let signal = controller.signal();
    let target = body(&session.snapshot(), origin.document());
    let outer = units("limit");
    let inside = controller.clone();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            &outer,
            DomEventCallback::new(move |dom, registry, trace: &mut Trace, _| {
                assert!(matches!(
                    inside.abort(dom, registry, trace, None, limits(), &cancel()),
                    Err(DomAbortError::Event(DomEventDispatchError::Admission(resource)))
                        if resource == refused
                ));
                trace.push("refused");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            origin.global(),
            &outer,
            DomEventInit::default(),
            &cancel(),
        )
        .unwrap();
    let mut calls = Trace::new();
    assert!(
        fire(&mut session, &mut registry, target, &event, &mut calls)
            .failures()
            .is_empty()
    );
    assert_eq!(calls, ["refused"]);
    assert!(!signal.aborted().unwrap());
    assert!(signal.reason().unwrap().is_none());
    assert_eq!(
        delivered(begin_abort(
            &controller,
            &mut session,
            &mut registry,
            &mut calls,
            None
        ))
        .callbacks(),
        0
    );
}

#[test]
fn missing_mandatory_admission_preserves_the_original_live_signal() {
    let mut session = session();
    let origin = top(&mut session);
    let flags = DomEventListenerOptions::default();
    let event_type = units("abort");
    for (pending_events, work, refused) in [
        (0, 100, DomEventResource::PendingEvents),
        (1, 1, DomEventResource::Work),
    ] {
        let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 1,
            pending_events,
            nested_dispatches: 1,
            storage_bytes: 16_000_000,
            work,
        })
        .unwrap();
        let controller = controller(&mut session, &mut registry, origin, 1);
        let signal = controller.signal();
        let mut calls = Trace::new();
        let registration = if pending_events == 0 {
            Some(
                registry
                    .add_listener_with_signal(
                        &mut session.dom(),
                        signal.event_target(),
                        &event_type,
                        recording("kept"),
                        flags,
                        &signal,
                    )
                    .unwrap(),
            )
        } else {
            None
        };
        let result = controller.abort(
            &mut session.dom(),
            &mut registry,
            &mut calls,
            None,
            limits(),
            &cancel(),
        );
        assert!(
            matches!(result, Err(DomAbortError::Event(DomEventDispatchError::Admission(resource))) if resource == refused),
            "{result:?}"
        );
        assert!(
            !signal.aborted().unwrap(),
            "refusal of {refused:?} set the reason"
        );
        assert!(signal.reason().unwrap().is_none());
        assert!(calls.is_empty());
        if let Some(DomSignalListenerRegistration::Inserted(id)) = registration {
            assert!(registry.remove_listener(id), "refusal removed the listener");
        }
    }
}

#[test]
fn a_signal_cannot_transition_through_a_registry_without_canonical_authority() {
    let mut session = session();
    let origin = top(&mut session);
    let mut original = new_registry::<Trace>();
    let controller = controller(&mut session, &mut original, origin, 0);
    let signal = controller.signal();
    let mut current = new_registry::<Trace>();
    current.bind(&mut session.dom(), &cancel()).unwrap();
    let result = controller.abort(
        &mut session.dom(),
        &mut original,
        &mut Trace::new(),
        None,
        limits(),
        &cancel(),
    );
    assert!(result.is_err(), "{result:?}");
    assert!(!signal.aborted().unwrap());
    assert!(signal.reason().unwrap().is_none());
    drop(current);
    assert_eq!(
        delivered(begin_abort(
            &controller,
            &mut session,
            &mut original,
            &mut Trace::new(),
            None
        ))
        .callbacks(),
        0
    );
}
