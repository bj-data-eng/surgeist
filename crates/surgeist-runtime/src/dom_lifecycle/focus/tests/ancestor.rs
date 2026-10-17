//! Genuine retained observing parent with an independently scoped child Show.
use super::*;
use crate::{
    DomEventCallback, DomEventListenerOptions, DomEventType, DomSourceRuntime,
    DomSourceRuntimeError,
};
use std::error::Error;
use surgeist_dom::DomSourceEventRequest;
type Failure = Box<dyn Error>;
struct Child {
    source: DomSourceRuntime,
    scope: DomSourceExecutionScope,
    target: NodeId,
    event: Option<DomSourceEventRequest>,
    error: Option<DomSourceRuntimeError>,
    unexpected: Option<String>,
    calls: usize,
}
struct Ancestor {
    session: DomSession,
    parent: DomSourceEventRequest,
    child: DomSourceDialogRequest,
}
fn child_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4)
}
fn dispatch_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 16,
        snapshot_listeners: 4,
        failure_records: 4,
        callbacks: 4,
        work: 100_000,
    }
}
fn ancestor(cap: u64, prelude: bool) -> Result<Ancestor, Failure> {
    let (mut session, host, parent) = fixture();
    let cancel = AtomicBool::new(false);
    let document = session.snapshot().node(parent)?.owner_document().unwrap();
    let mut batch = DomBatch::new();
    let child = batch.create_element_ns(document, Namespace::html(), "dialog");
    let child = session
        .dom()
        .apply(batch, &cancel)?
        .created(&child)?
        .unwrap();
    let scope = session.source_execution_scope(host, &cancel)?;
    let mut context = Child {
        source: session.source_execution(),
        scope: scope.clone(),
        target: child,
        event: None,
        error: None,
        unexpected: None,
        calls: 0,
    };
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 2,
        nested_dispatches: 2,
    })?;
    registry.add_listener(
        &session.dom(),
        parent,
        DomEventType::BeforeToggle,
        DomEventCallback::new(|dom, _, context: &mut Child, _| {
            context.calls += 1;
            match context.source.apply(
                dom,
                DomSourceEdit::show_dialog(context.target),
                child_limits(),
                &mut DomSourceServices {
                    execution_scope: Some(&context.scope),
                    style_policy: &mut |_| None,
                    url: None,
                },
                &AtomicBool::new(false),
            ) {
                Ok(DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(event),
                    ..
                }) => context.event = Some(event),
                Err(error) => context.error = Some(error),
                Ok(other) => context.unexpected = Some(format!("{other:?}")),
            }
            Ok(())
        }),
        DomEventListenerOptions::default(),
    )?;
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(parent),
        ..
    } = session.source_execution().apply(
        &mut session.dom(),
        DomSourceEdit::show_dialog(parent),
        source_limits(cap),
        &mut DomSourceServices {
            execution_scope: Some(&scope),
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel,
    )?
    else {
        panic!("actual parent observation required")
    };
    let outer = session.source_execution().service_event(
        &mut session.dom(),
        &parent,
        &mut registry,
        &mut context,
        DomSourceEventServiceLimits {
            source: source_limits(cap),
            dispatch: dispatch_limits(),
        },
        &cancel,
    );
    if let Some(error) = context.error {
        return Err(Box::new(error));
    }
    let _ = outer?;
    assert_eq!(context.calls, 1);
    assert!(
        context.unexpected.is_none(),
        "unexpected child entry: {:?}",
        context.unexpected
    );
    let child = context.event.expect("real callback started its child");
    let mut empty = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 0,
        pending_events: 1,
        nested_dispatches: 1,
    })?;
    assert!(matches!(
        session.source_execution().service_event(
            &mut session.dom(),
            &child,
            &mut empty,
            &mut (),
            DomSourceEventServiceLimits {
                source: child_limits(),
                dispatch: dispatch_limits()
            },
            &cancel
        )?,
        DomSourceEventServiceProgress::Serviced { .. }
    ));
    let tasks = session.dom_manipulation_execution();
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let child = {
        let mut dom = session.dom();
        let mut lease =
            dom.raw_mut()
                .resume_source_edit(child.operation(), child_limits(), &cancel)?;
        let DomSourceProgress::Request(DomSourceRequest::Dialog(queue)) = lease.advance(&cancel)?
        else {
            panic!("actual child Queue required")
        };
        tasks.queue_dialog(&mut lease, &queue, &cancel)?;
        let DomSourceProgress::Request(DomSourceRequest::Dialog(child)) = lease.advance(&cancel)?
        else {
            panic!("actual child Remember required")
        };
        assert_eq!(child.action(), HtmlDialogAction::RememberFocusedElement);
        assert_eq!(lease.status().parent_operation(), Some(parent.operation()));
        child
    };
    if prelude {
        // Match the second actual resume used by the service tests. Its
        // original parent admission work belongs to the canonical scaffold.
        let mut dom = session.dom();
        let mut lease =
            dom.raw_mut()
                .resume_source_edit(child.operation(), child_limits(), &cancel)?;
        // Canonical preparation/real retention only, excluding all three
        // Runtime qualification/retained-Host/observation visits under test.
        let source = &owner.borrow().source;
        let payload =
            lease.retain_hosted_runtime_payload(source, size_of::<FocusFormation>(), &cancel)?;
        let preparation = lease
            .prepare_dialog_focus(&child, source, &cancel)?
            .unwrap();
        let formation = FocusFormation {
            preparation,
            payload,
        };
        drop(formation);
    }
    Ok(Ancestor {
        session,
        parent,
        child,
    })
}
fn exhausted(error: &(dyn Error + 'static), cap: u64) -> bool {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(DomError::SourceWorkLimit { limit }) = error.downcast_ref::<DomError>() {
            assert_eq!(*limit, cap);
            return true;
        }
        if let Some(DomHostedError::Dom(DomError::SourceWorkLimit { limit })) =
            error.downcast_ref::<DomHostedError>()
        {
            assert_eq!(*limit, cap);
            return true;
        }
        current = error.source();
    }
    false
}
#[test]
fn observing_parent_bytes_exact_and_one_short_govern_child_observation_formation() {
    for short in [true, false] {
        let Ancestor {
            mut session,
            parent,
            child,
        } = ancestor(10_000_000, false).unwrap();
        let hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(child.operation(), child_limits(), &AtomicBool::new(false))
            .unwrap();
        let mut low = 0;
        let mut high = source_limits(10_000_000).retained_bytes();
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            match filler(&mut lease, &owner.borrow().source, middle) {
                Ok(value) => {
                    drop(value);
                    low = middle;
                }
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit,
                }) => {
                    assert!(limit > 0 && limit <= source_limits(10_000_000).retained_bytes());
                    high = middle - 1;
                }
                Err(error) => panic!("wrong ancestor baseline refusal: {error:?}"),
            }
        }
        // Observe only the independent canonical front at its own boundary.
        // Nested admission subtracts retained observer bodies from the parent
        // allowance; Runtime cannot inspect those private DOM body layouts.
        let effective = match filler(&mut lease, &owner.borrow().source, low + 1) {
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit,
            }) => limit,
            Err(error) => panic!("wrong canonical boundary refusal: {error:?}"),
            Ok(_) => panic!("canonical one-short boundary unexpectedly admitted"),
        };
        assert!(effective > 0);
        assert!(effective <= source_limits(10_000_000).retained_bytes());
        assert!(source_limits(10_000_000).retained_bytes() < child_limits().retained_bytes());
        let candidate = size_of::<FocusFormation>();
        let value = filler(
            &mut lease,
            &owner.borrow().source,
            low - candidate + usize::from(short),
        )
        .unwrap();
        let basis = lease.snapshot().stamp();
        let result = hosted.service_dialog_focus(&mut lease, &child, &AtomicBool::new(false));
        if short {
            assert!(matches!(
                result,
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit
                    }
                ))) if limit == effective
            ));
            assert!(owner.borrow().focus.pending.is_none());
        } else {
            assert!(result.unwrap());
            assert!(owner.borrow().focus.pending.is_none());
        }
        assert_eq!(lease.snapshot().stamp(), basis);
        assert_eq!(lease.status().parent_operation(), Some(parent.operation()));
        assert!(lease.status().spent_work() < child_limits().work());
        drop(value);
        if short {
            assert!(
                hosted
                    .service_dialog_focus(&mut lease, &child, &AtomicBool::new(false))
                    .unwrap()
            );
        }
        assert_eq!(
            request(&mut lease, HtmlDialogAction::RunDialogFocusingSteps).operation(),
            child.operation()
        );
    }
}
#[test]
fn observing_parent_work_prices_three_actual_runtime_visits_and_keeps_one_pending_result() {
    // Search only canonical setup/prepare/retention on fresh real owners.
    // Runtime remembered-focus service never runs during this calibration.
    let mut low = 0;
    let mut high = 10_000_000;
    while low < high {
        let middle = low + (high - low) / 2;
        match ancestor(middle, true) {
            Ok(_) => high = middle,
            Err(error) => {
                assert!(
                    exhausted(error.as_ref(), middle),
                    "wrong canonical setup error: {error:?}"
                );
                low = middle + 1;
            }
        }
    }
    let scaffold = low;
    assert!(scaffold > 0);
    for short in [true, false] {
        let cap = scaffold + 3 - u64::from(short);
        let Ancestor {
            mut session,
            parent,
            child,
        } = ancestor(cap, false).unwrap();
        let cancel = AtomicBool::new(false);
        let hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(child.operation(), child_limits(), &cancel)
                .unwrap();
            let basis = lease.snapshot().stamp();
            let error = hosted
                .service_dialog_focus(&mut lease, &child, &cancel)
                .unwrap_err();
            assert!(exhausted(&error, cap), "wrong ancestor refusal: {error:?}");
            assert_eq!(lease.snapshot().stamp(), basis);
            assert_eq!(lease.status().parent_operation(), Some(parent.operation()));
            assert!(lease.status().spent_work() < child_limits().work());
        }
        if short {
            assert!(
                owner.borrow().focus.pending.is_none(),
                "one short refuses before observation"
            );
        } else {
            assert_eq!(
                owner.borrow().focus.pending.as_ref().unwrap().request(),
                &child,
                "exact formation visits produce the authentic result before acceptance needs more work"
            );
            let pending = session.source_execution().resume(
                &mut session.dom(),
                parent.operation(),
                source_limits(10_000_000),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            );
            assert!(matches!(
                pending,
                Err(DomSourceRuntimeError::Dom(DomError::InvalidSourceOperation))
            ));
            assert_eq!(
                owner.borrow().focus.pending.as_ref().unwrap().request(),
                &child
            );
        }
        session
            .source_execution()
            .raise_event_limits(
                &mut session.dom(),
                &parent,
                source_limits(10_000_000),
                &cancel,
            )
            .unwrap();
        let before = owner.borrow().work;
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(child.operation(), child_limits(), &cancel)
            .unwrap();
        assert!(
            hosted
                .service_dialog_focus(&mut lease, &child, &cancel)
                .unwrap()
        );
        assert!(owner.borrow().focus.pending.is_none());
        if !short {
            assert_eq!(
                owner.borrow().work,
                before,
                "retained result transfers without a second observation"
            );
        }
        assert_eq!(
            request(&mut lease, HtmlDialogAction::RunDialogFocusingSteps).operation(),
            child.operation()
        );
        assert_eq!(lease.status().parent_operation(), Some(parent.operation()));
    }
}

#[test]
fn observing_parent_prices_old_and_replacement_host_backings_until_physical_release() {
    for short in [true, false] {
        let Ancestor {
            mut session,
            parent,
            child,
        } = ancestor(10_000_000, false).unwrap();
        let cancel = AtomicBool::new(false);
        let mut hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let original = {
            let state = owner.borrow();
            assert_eq!(state.hosts.len(), 1);
            assert_eq!(state.hosts.capacity(), 1);
            assert_eq!(
                state.hosts[0].focus,
                HostFocusDisposition::HeadlessNoContentSystemFocus
            );
            state.hosts[0].id
        };
        // The existing actual one-slot backing remains Source-owned while the
        // two-slot replacement is allocated; no per-field delta is used.
        let old_backing = size_of::<records::Host>();
        let replacement = 2 * size_of::<records::Host>();
        let (value, effective) = {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(child.operation(), child_limits(), &cancel)
                .unwrap();
            assert_eq!(
                lease.execution_entry_scope().unwrap(),
                DomSourceExecutionEntryScope::ExplicitHost(original)
            );
            let mut low = 0;
            let mut high = source_limits(10_000_000).retained_bytes();
            while low < high {
                let middle = low + (high - low).div_ceil(2);
                match filler(&mut lease, &owner.borrow().source, middle) {
                    Ok(value) => {
                        drop(value);
                        low = middle;
                    }
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit,
                    }) => {
                        assert!(limit > 0 && limit <= source_limits(10_000_000).retained_bytes());
                        high = middle - 1;
                    }
                    Err(error) => {
                        panic!("wrong canonical Host-overlap baseline refusal: {error:?}")
                    }
                }
            }
            let effective = match filler(&mut lease, &owner.borrow().source, low + 1) {
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    limit,
                }) => limit,
                Err(error) => panic!("wrong canonical Host-overlap boundary refusal: {error:?}"),
                Ok(_) => panic!("canonical boundary unexpectedly admitted"),
            };
            assert!(effective > 0 && effective <= source_limits(10_000_000).retained_bytes());
            assert!(source_limits(10_000_000).retained_bytes() < child_limits().retained_bytes());
            assert!(low > replacement);
            let value = filler(
                &mut lease,
                &owner.borrow().source,
                low - replacement + usize::from(short),
            )
            .unwrap();
            (value, effective)
        }; // Only the lease ends: the actual allocation and relative guard stay live.
        let basis = session.snapshot().stamp();
        let issuer = owner.borrow().source.revision().unwrap();
        let result = hosted.create_headless_host(&mut session.dom(), &cancel);
        let second = if short {
            assert!(
                matches!(result, Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }))) if limit == effective)
            );
            {
                let state = owner.borrow();
                assert_eq!(state.source.revision().unwrap(), issuer);
                assert_eq!(state.hosts.len(), 1);
                assert_eq!(state.hosts.capacity(), 1);
                assert_eq!(state.hosts[0].id, original);
                assert!(state.focus.pending.is_none());
            }
            assert_eq!(session.snapshot().stamp(), basis);
            drop(value);
            // The same original counter/bank can retry once the genuine caller
            // allocation releases; no Source policy increase or reset is used.
            hosted
                .create_headless_host(&mut session.dom(), &cancel)
                .unwrap()
        } else {
            let second = result.expect("exact Source old+replacement overlap must admit");
            {
                let state = owner.borrow();
                assert_ne!(state.source.revision().unwrap(), issuer);
                assert_eq!(state.hosts.len(), 2);
                assert_eq!(state.hosts.capacity(), 2);
                assert_eq!(state.hosts[0].id, original);
                assert_eq!(state.hosts[1].id, second);
            }
            // The original allocation has physically dropped after replacement.
            // While the original Filler remains live, exactly that old backing
            // can be used by a second genuine caller allocation. Its own inline
            // descriptor/guard is included, rather than treated as free.
            assert!(old_backing >= size_of::<Filler>());
            let released = {
                let mut dom = session.dom();
                // Retain through the original source directly: no extra resume
                // formation changes the independently calibrated byte baseline.
                let retained = owner
                    .borrow()
                    .source
                    .retain_runtime_payload(dom.raw_mut(), old_backing, &cancel)
                    .unwrap();
                let mut bytes = Vec::new();
                bytes
                    .try_reserve_exact(old_backing - size_of::<Filler>())
                    .unwrap();
                assert_eq!(bytes.capacity(), old_backing - size_of::<Filler>());
                Filler {
                    _bytes: bytes,
                    _retention: retained,
                }
            };
            drop(released);
            drop(value);
            second
        };
        assert_ne!(second, original);
        {
            let state = owner.borrow();
            assert_eq!(state.hosts.capacity(), 2);
            assert_eq!(state.hosts[0].id, original);
            assert_eq!(
                state.hosts[0].focus,
                HostFocusDisposition::HeadlessNoContentSystemFocus
            );
            assert_eq!(state.hosts[1].id, second);
            assert!(state.focus.host_backing.is_some());
            assert!(state.focus.pending.is_none());
        }
        assert_eq!(session.snapshot().stamp(), basis);
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(child.operation(), child_limits(), &cancel)
            .unwrap();
        assert_eq!(
            lease.execution_entry_scope().unwrap(),
            DomSourceExecutionEntryScope::ExplicitHost(original)
        );
        assert_eq!(lease.status().parent_operation(), Some(parent.operation()));
        assert!(
            hosted
                .service_dialog_focus(&mut lease, &child, &cancel)
                .unwrap()
        );
        assert_eq!(
            request(&mut lease, HtmlDialogAction::RunDialogFocusingSteps).operation(),
            child.operation()
        );
        assert_eq!(lease.snapshot().stamp(), basis);
    }
}
