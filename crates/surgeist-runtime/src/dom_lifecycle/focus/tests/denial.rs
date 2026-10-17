//! Real callback adoption into a retired Document with an actual retained Window.
use super::*;
use crate::{
    DomEventCallback, DomEventListenerOptions, DomEventType, DomSourceRuntime,
    DomSourceRuntimeError,
};
use std::error::Error;
type Failure = Box<dyn Error>;
const RUNTIME_VISITS: u64 = 20;

struct Parked {
    session: DomSession,
    request: DomSourceDialogRequest,
    parent: Option<DomSourceEventRequest>,
    host: DomHostedHostId,
    original_global: DomHostedGlobalId,
    current_global: DomHostedGlobalId,
    current_document: DocumentId,
    subject: NodeId,
    sentinel: NodeId,
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
fn start(
    session: &mut DomSession,
    target: NodeId,
    policy: DomSourceLimits,
    scope: Option<&DomSourceExecutionScope>,
) -> Result<DomSourceEventRequest, Failure> {
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(event),
        ..
    } = session.source_execution().apply(
        &mut session.dom(),
        DomSourceEdit::show_dialog(target),
        policy,
        &mut DomSourceServices {
            execution_scope: scope,
            style_policy: &mut |_| None,
            url: None,
        },
        &AtomicBool::new(false),
    )?
    else {
        panic!("real beforetoggle observation required")
    };
    Ok(event)
}
fn element(
    session: &mut DomSession,
    document: DocumentId,
    parent: NodeId,
) -> Result<NodeId, Failure> {
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, parent, None);
    Ok(session
        .dom()
        .apply(batch, &AtomicBool::new(false))?
        .created(&dialog)?
        .unwrap())
}
fn next(
    lease: &mut DomSourceLease<'_>,
    action: HtmlDialogAction,
) -> Result<DomSourceDialogRequest, Failure> {
    let DomSourceProgress::Request(DomSourceRequest::Dialog(request)) =
        lease.advance(&AtomicBool::new(false))?
    else {
        panic!("actual Dialog request required")
    };
    assert_eq!(request.action(), action);
    Ok(request)
}
fn parked(
    work: u64,
    parent_work: Option<u64>,
    prelude: bool,
    retain_window: bool,
) -> Result<Parked, Failure> {
    let (mut session, host, parent_subject) = fixture();
    let cancel = AtomicBool::new(false);
    let mut hosted = session.hosted_execution();
    let a = hosted.create_top_level(&mut session.dom(), host, &cancel)?;
    let b = hosted.create_top_level(&mut session.dom(), host, &cancel)?;
    let snapshot = session.snapshot();
    let body = |document: DocumentId| {
        let html = snapshot.node(document.node()).unwrap().children()[0];
        snapshot.node(html).unwrap().children()[1]
    };
    let subject = element(&mut session, a.document(), body(a.document()))?;
    let body_b = body(b.document());
    let sentinel = element(&mut session, b.document(), body_b)?;
    if retain_window {
        let opened = session.source_execution().apply(
            &mut session.dom(),
            DomSourceEdit::set_attribute(sentinel, "open", ""),
            source_limits(10_000_000),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )?;
        assert!(matches!(opened, DomSourceDriveProgress::Complete(_)));
    }
    let queued = hosted.destroy_top_level(&mut session.dom(), b.navigable(), &cancel)?;
    let mut completed = false;
    for _ in 0..32 {
        if matches!(hosted.run_next(&mut session.dom(), queued.traversable(), &cancel)?,
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            completed = true;
            break;
        }
    }
    assert!(completed, "the real B destruction must complete");
    let (event, parent) = if let Some(cap) = parent_work {
        struct Child {
            source: DomSourceRuntime,
            subject: NodeId,
            work: u64,
            event: Option<DomSourceEventRequest>,
            error: Option<DomSourceRuntimeError>,
            calls: usize,
            unexpected: Option<String>,
        }
        let scope = session.source_execution_scope(host, &cancel)?;
        let parent = start(
            &mut session,
            parent_subject,
            source_limits(cap),
            Some(&scope),
        )?;
        let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 1,
            pending_events: 1,
            nested_dispatches: 1,
        })?;
        registry.add_listener(
            &session.dom(),
            parent_subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|dom, _, context: &mut Child, _| {
                context.calls += 1;
                match context.source.apply(
                    dom,
                    DomSourceEdit::show_dialog(context.subject),
                    source_limits(context.work),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &AtomicBool::new(false),
                ) {
                    Ok(DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::Event(event),
                        ..
                    }) => context.event = Some(event),
                    Ok(other) => {
                        context.unexpected =
                            Some(format!("unexpected genuine child entry: {other:?}"))
                    }
                    Err(error) => context.error = Some(error),
                }
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )?;
        let mut context = Child {
            source: session.source_execution(),
            subject,
            work,
            event: None,
            error: None,
            calls: 0,
            unexpected: None,
        };
        let result = session.source_execution().service_event(
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
        let _ = result?;
        assert_eq!(context.calls, 1);
        assert!(context.unexpected.is_none(), "{:?}", context.unexpected);
        (
            context
                .event
                .expect("the real parent callback started its child"),
            Some(parent),
        )
    } else {
        (
            start(&mut session, subject, source_limits(work), None)?,
            None,
        )
    };
    struct Adopt {
        calls: usize,
        result: Option<Result<DomCommit, DomError>>,
    }
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })?;
    registry.add_listener(
        &session.dom(),
        subject,
        DomEventType::BeforeToggle,
        DomEventCallback::new(move |dom, _, context: &mut Adopt, _| {
            context.calls += 1;
            let mut batch = DomBatch::new();
            batch.insert_before(subject, body_b, None);
            context.result = Some(dom.apply(batch, &AtomicBool::new(false)));
            Ok(())
        }),
        DomEventListenerOptions {
            capture: false,
            once: false,
        },
    )?;
    let mut context = Adopt {
        calls: 0,
        result: None,
    };
    let progress = session.source_execution().service_event(
        &mut session.dom(),
        &event,
        &mut registry,
        &mut context,
        DomSourceEventServiceLimits {
            source: source_limits(work),
            dispatch: dispatch_limits(),
        },
        &cancel,
    )?;
    assert!(matches!(
        progress,
        DomSourceEventServiceProgress::Serviced { .. }
    ));
    assert_eq!(context.calls, 1);
    drop(context.result.unwrap()?);
    let tasks = session.element_task_execution();
    let request = {
        let mut dom = session.dom();
        let mut lease =
            dom.raw_mut()
                .resume_source_edit(event.operation(), source_limits(work), &cancel)?;
        let DomSourceProgress::Request(DomSourceRequest::Dialog(queue)) = lease.advance(&cancel)?
        else {
            panic!("real QueueToggle required")
        };
        tasks.queue_dialog(&mut lease, &queue, &cancel)?;
        let remembered = next(&mut lease, HtmlDialogAction::RememberFocusedElement)?;
        assert!(hosted.service_dialog_focus(&mut lease, &remembered, &cancel)?);
        let focusing = next(&mut lease, HtmlDialogAction::RunDialogFocusingSteps)?;
        assert_eq!(
            lease.status().parent_operation(),
            parent.as_ref().map(DomSourceEventRequest::operation)
        );
        focusing
    };
    let parked = Parked {
        session,
        request,
        parent,
        host,
        original_global: a.global(),
        current_global: b.global(),
        current_document: b.document(),
        subject,
        sentinel,
    };
    if prelude {
        let mut parked = parked;
        canonical_prelude(&mut parked)?;
        Ok(parked)
    } else {
        Ok(parked)
    }
}
// Canonical preparation and actual relative formation only. No Runtime denial
// service, retained-bank traversal, Window scan or observation is calibrated.
fn canonical_prelude(case: &mut Parked) -> Result<(u64, u64), Failure> {
    let hosted = case.session.hosted_execution();
    let owner = hosted.state()?;
    let source = &owner.borrow().source;
    let mut dom = case.session.dom();
    let mut lease = dom.raw_mut().resume_source_edit(
        case.request.operation(),
        source_limits(10_000_000),
        &AtomicBool::new(false),
    )?;
    let before = lease.status().spent_work();
    let payload = lease.retain_hosted_runtime_payload(
        source,
        size_of::<FocusFormation>(),
        &AtomicBool::new(false),
    )?;
    let preparation = lease
        .prepare_dialog_focus(&case.request, source, &AtomicBool::new(false))?
        .unwrap();
    assert_eq!(
        preparation.target(),
        DomDialogFocusTarget::RetiredDocumentWindow {
            document: case.current_document,
            global: case.current_global,
            host: case.host,
        }
    );
    let formation = FocusFormation {
        preparation,
        payload,
    };
    let spent = lease.status().spent_work() - before;
    drop(formation);
    Ok((before, spent))
}
fn assert_real_shape(case: &Parked) {
    let owner = case.session.hosted_execution().state().unwrap();
    let state = owner.borrow();
    assert_eq!(state.hosts.len(), 1);
    assert_eq!(state.hosts[0].top_levels.len(), 2);
    assert!(!state.destruction.pending());
    assert!(state.children.is_empty());
    let a = state.hosts[0].top_levels[0].group.as_ref().unwrap();
    let b = state.hosts[0].top_levels[1].group.as_ref().unwrap();
    assert_eq!(a.clusters.len(), 1);
    assert_eq!(b.clusters.len(), 1);
    assert_eq!(a.clusters[0].agent.globals.len(), 1);
    assert!(b.clusters[0].agent.globals.is_empty());
    assert_eq!(a.clusters[0].agent.windows.rows.len(), 1);
    assert_eq!(b.clusters[0].agent.windows.rows.len(), 1);
    assert_eq!(b.clusters[0].agent.windows.rows[0].id, case.current_global);
    assert!(b.clusters[0].agent.windows.rows[0].retired);
    // Independent service tariff for this real shape: availability1; retained
    // owner traversal Host1 + A(comp/cluster/Window/Global/settings=5)
    // + B(comp/cluster/Window/watcher-group/original-member-record-body=5);
    // find B(host1+A3+B3=7); actual observation1. Total1+11+7+1=20,
    // independently counting the same original Rc record body exactly once.
    assert_eq!(RUNTIME_VISITS, 1 + (1 + 5 + 5) + (1 + 3 + 3) + 1);
    let snapshot = case.session.snapshot();
    assert_eq!(
        snapshot.node(case.subject).unwrap().owner_document(),
        Some(case.current_document)
    );
    assert_eq!(
        snapshot.relevant_global(case.subject).unwrap(),
        Some(case.original_global)
    );
    assert!(
        snapshot
            .node(case.sentinel)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
}

#[test]
fn denial_work_boundary_retains_exact_result_blocks_cleanup_and_releases_retired_window() {
    let mut calibration = parked(10_000_000, None, false, true).unwrap();
    assert_real_shape(&calibration);
    let (scaffold, canonical) = canonical_prelude(&mut calibration).unwrap();
    for short in [true, false] {
        let cap = scaffold + canonical + RUNTIME_VISITS - u64::from(short);
        let mut case = parked(cap, None, false, true).unwrap();
        assert_real_shape(&case);
        let cancel = AtomicBool::new(false);
        let hosted = case.session.hosted_execution();
        let owner = hosted.state().unwrap();
        let old_loop = {
            let state = owner.borrow();
            Rc::downgrade(
                &state.hosts[0].top_levels[1]
                    .group
                    .as_ref()
                    .unwrap()
                    .clusters[0]
                    .agent
                    .event_loop,
            )
        };
        let basis = case.session.snapshot().stamp();
        let queue = case
            .session
            .element_task_execution()
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap();
        {
            let mut dom = case.session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(case.request.operation(), source_limits(cap), &cancel)
                .unwrap();
            assert_eq!(lease.status().spent_work(), scaffold);
            assert!(
                matches!(hosted.service_dialog_focus(&mut lease, &case.request, &cancel),
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit { limit }))) if limit == cap)
            );
            assert_eq!(lease.status().spent_work(), cap);
            assert_eq!(lease.snapshot().stamp(), basis);
        }
        if short {
            assert!(
                owner.borrow().focus.pending.is_none(),
                "one-short refuses before any owned result"
            );
        } else {
            assert_eq!(
                owner.borrow().focus.pending.as_ref().unwrap().request(),
                &case.request
            );
            let cleanup = case.session.source_execution().apply(
                &mut case.session.dom(),
                DomSourceEdit::remove_attribute(case.sentinel, "open"),
                source_limits(10_000_000),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            );
            assert!(matches!(
                cleanup,
                Err(DomSourceRuntimeError::Dom(DomError::SourceOperationPending))
            ));
            assert!(
                old_loop.upgrade().is_some(),
                "the blocked cleanup cannot drop the actual Window/loop"
            );
        }
        let replacement = case.session.hosted_execution();
        {
            let mut dom = case.session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
                .unwrap();
            let before = owner.borrow().work;
            assert!(matches!(
                replacement.service_dialog_focus(&mut lease, &case.request, &AtomicBool::new(true)),
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                    DomError::Cancelled
                ))) | Err(DomHostedRuntimeError::Cancelled)
            ));
            assert_eq!(owner.borrow().work, before);
            assert_eq!(owner.borrow().focus.pending.is_some(), !short);
            assert!(
                replacement
                    .service_dialog_focus(&mut lease, &case.request, &cancel)
                    .unwrap()
            );
            assert!(owner.borrow().focus.pending.is_none());
            if !short {
                assert_eq!(
                    owner.borrow().work,
                    before,
                    "retained denial transfers without re-reading the Window"
                );
            }
            assert_eq!(lease.snapshot().stamp(), basis);
            assert_eq!(
                lease.advance(&AtomicBool::new(true)).unwrap_err(),
                DomError::Cancelled
            );
        }
        let done = {
            let mut dom = case.session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
                .unwrap();
            let before_settlement = owner.borrow().work;
            let settlement = replacement
                .prepare_watcher_settlement(&lease, &cancel)
                .unwrap();
            // The original driver's mandatory settlement is separate work:
            // first preparation scan8 + second scan3 + finish prepayment20.
            // The latter is first pass16 (Host1/compositions2/clusters2,
            // Windows6/groups3/member2) + prune pass4 (Host1/compositions2,
            // retired B cluster1; no trailing composition to move).
            assert_eq!(owner.borrow().work - before_settlement, 8 + 3 + 20);
            let before = owner.borrow().work;
            let completion = lease.advance(&cancel).unwrap();
            settlement.finish(&lease.snapshot());
            assert_eq!(
                owner.borrow().work,
                before,
                "accepted receipt consumption performs no further Runtime observation or unadmitted cleanup"
            );
            let DomSourceProgress::Complete(done) = completion else {
                panic!("actual negative allow-focus must close its original Show")
            };
            done
        };
        assert!(!done.cancellation_requested());
        assert_eq!(done.operation(), case.request.operation());
        assert!(case.session.dom().pending_source_edit().is_none());
        assert!(
            old_loop.upgrade().is_some(),
            "the unrelated real watcher still owns Window B"
        );
        assert_eq!(
            case.session
                .element_task_execution()
                .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
                .unwrap(),
            queue
        );
        let cleaned = case
            .session
            .source_execution()
            .apply(
                &mut case.session.dom(),
                DomSourceEdit::remove_attribute(case.sentinel, "open"),
                source_limits(10_000_000),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        assert!(matches!(cleaned, DomSourceDriveProgress::Complete(_)));
        assert!(
            !case
                .session
                .snapshot()
                .node(case.sentinel)
                .unwrap()
                .as_dialog()
                .unwrap()
                .has_close_watcher()
        );
        assert_eq!(owner.borrow().hosts[0].top_levels.len(), 1);
        assert!(
            old_loop.upgrade().is_none(),
            "exact clear settlement physically releases B's composition/WindowLoop"
        );
        assert_eq!(
            case.session
                .element_task_execution()
                .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
                .unwrap(),
            queue
        );
    }
}

#[test]
fn pruned_original_window_does_not_become_a_negative_focus_observation() {
    let mut case = parked(10_000_000, None, false, false).unwrap();
    let cancel = AtomicBool::new(false);
    let hosted = case.session.hosted_execution();
    let owner = hosted.state().unwrap();
    assert_eq!(
        owner.borrow().hosts[0].top_levels.len(),
        1,
        "the genuinely empty retired B composition was pruned"
    );
    let basis = case.session.snapshot().stamp();
    let mut dom = case.session.dom();
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
        .unwrap();
    {
        let prepared = lease
            .prepare_dialog_focus(&case.request, &owner.borrow().source, &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(
            prepared.target(),
            DomDialogFocusTarget::RetiredDocumentWindow {
                document: case.current_document,
                global: case.current_global,
                host: case.host,
            }
        );
    }
    for _ in 0..2 {
        assert!(
            !hosted
                .service_dialog_focus(&mut lease, &case.request, &cancel)
                .unwrap()
        );
        assert!(owner.borrow().focus.pending.is_none());
        assert_eq!(lease.snapshot().stamp(), basis);
        assert_eq!(
            request(&mut lease, HtmlDialogAction::RunDialogFocusingSteps),
            case.request
        );
    }
}

#[test]
fn denial_hosted_formation_storage_scratch_and_twenty_visits_admit_exactly() {
    for kind in 0..3 {
        let mut case = parked(10_000_000, None, false, true).unwrap();
        assert_real_shape(&case);
        let cancel = AtomicBool::new(false);
        let mut hosted = case.session.hosted_execution();
        let owner = hosted.state().unwrap();
        // Only the unchanged existing owner projection supplies this fixture's
        // baseline (including opaque watcher bodies). The new actual temporary
        // is independently sizeof its owned preparation plus relative guard.
        let baseline = owner.borrow_mut().retained_bytes(&cancel).unwrap();
        let candidate = size_of::<FocusFormation>();
        let mut policy = session_limits().hosted;
        policy.storage_bytes = baseline + candidate - usize::from(kind == 0);
        policy.scratch_bytes = candidate - usize::from(kind == 1);
        hosted
            .set_limits(&mut case.session.dom(), policy, &cancel)
            .unwrap();
        if kind == 2 {
            let mut state = owner.borrow_mut();
            state.limits.work = state.work + RUNTIME_VISITS - 1;
        }
        let basis = case.session.snapshot().stamp();
        {
            let mut dom = case.session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
                .unwrap();
            assert!(matches!(
                hosted.service_dialog_focus(&mut lease, &case.request, &cancel),
                Err(DomHostedRuntimeError::Admission)
            ));
            assert!(
                owner.borrow().focus.pending.is_none(),
                "no owned result may precede full admission"
            );
            assert_eq!(lease.snapshot().stamp(), basis);
            assert_eq!(
                request(&mut lease, HtmlDialogAction::RunDialogFocusingSteps),
                case.request
            );
        }
        policy.storage_bytes = baseline + candidate;
        policy.scratch_bytes = candidate;
        policy.work = 1_000_000;
        hosted
            .set_limits(&mut case.session.dom(), policy, &cancel)
            .unwrap();
        let before = owner.borrow().work;
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        assert!(
            hosted
                .service_dialog_focus(&mut lease, &case.request, &cancel)
                .unwrap()
        );
        assert_eq!(owner.borrow().work - before, RUNTIME_VISITS);
        assert!(owner.borrow().focus.pending.is_none());
        assert_eq!(lease.snapshot().stamp(), basis);
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
}

#[test]
fn denial_source_bytes_price_temporary_before_ownership_and_release_on_transfer() {
    for short in [true, false] {
        let mut case = parked(10_000_000, None, false, true).unwrap();
        let hosted = case.session.hosted_execution();
        let owner = hosted.state().unwrap();
        let cancel = AtomicBool::new(false);
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
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
                    assert_eq!(limit, source_limits(10_000_000).retained_bytes());
                    high = middle - 1;
                }
                Err(error) => panic!("wrong existing canonical baseline refusal: {error:?}"),
            }
        }
        let candidate = size_of::<FocusFormation>();
        assert!(low > candidate);
        let value = filler(
            &mut lease,
            &owner.borrow().source,
            low - candidate + usize::from(short),
        )
        .unwrap();
        let basis = lease.snapshot().stamp();
        let result = hosted.service_dialog_focus(&mut lease, &case.request, &cancel);
        if short {
            assert!(matches!(result,
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes, limit,
                }))) if limit == source_limits(10_000_000).retained_bytes()));
            assert!(owner.borrow().focus.pending.is_none());
        } else {
            assert!(result.unwrap());
            assert!(owner.borrow().focus.pending.is_none());
            assert!(candidate >= size_of::<Filler>());
            let released = filler(
                &mut lease,
                &owner.borrow().source,
                candidate - size_of::<Filler>(),
            )
            .unwrap();
            drop(released);
        }
        assert_eq!(lease.snapshot().stamp(), basis);
        drop(value);
        if short {
            assert!(
                hosted
                    .service_dialog_focus(&mut lease, &case.request, &cancel)
                    .unwrap()
            );
        }
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
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
fn observing_parent_work_governs_actual_denial_and_retained_receipt_transfer() {
    // Calibrate only setup and canonical preparation/relative retention. All
    // actual new Runtime visits remain the independently derived literal20.
    let mut low = 0;
    let mut high = 10_000_000;
    while low < high {
        let middle = low + (high - low) / 2;
        match parked(10_000_000, Some(middle), true, true) {
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
        let cap = scaffold + RUNTIME_VISITS - u64::from(short);
        let mut case = parked(10_000_000, Some(cap), false, true).unwrap();
        assert_real_shape(&case);
        let parent = case.parent.as_ref().unwrap().clone();
        let hosted = case.session.hosted_execution();
        let owner = hosted.state().unwrap();
        let cancel = AtomicBool::new(false);
        let basis = case.session.snapshot().stamp();
        {
            let mut dom = case.session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
                .unwrap();
            assert!(
                matches!(hosted.service_dialog_focus(&mut lease, &case.request, &cancel),
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit { limit }))) if limit == cap)
            );
            assert_eq!(lease.status().parent_operation(), Some(parent.operation()));
            assert!(lease.status().spent_work() < source_limits(10_000_000).work());
            assert_eq!(lease.snapshot().stamp(), basis);
        }
        assert_eq!(owner.borrow().focus.pending.is_some(), !short);
        case.session
            .source_execution()
            .raise_event_limits(
                &mut case.session.dom(),
                &parent,
                source_limits(10_000_000),
                &cancel,
            )
            .unwrap();
        let before = owner.borrow().work;
        let mut dom = case.session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(case.request.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        assert!(
            hosted
                .service_dialog_focus(&mut lease, &case.request, &cancel)
                .unwrap()
        );
        if !short {
            assert_eq!(
                owner.borrow().work,
                before,
                "retained child denial is not observed again"
            );
        }
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            parent.operation()
        );
        assert!(owner.borrow().focus.pending.is_none());
    }
}
