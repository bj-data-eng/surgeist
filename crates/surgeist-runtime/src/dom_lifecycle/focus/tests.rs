//! Actual headless Host producer, original Source receipt, and physical extents.
use super::*;
use crate::{
    DomElementTaskRuntimeLimits, DomEventDispatchLimits, DomEventDispatcher,
    DomEventRegistryLimits, DomSession, DomSessionLimits, DomSourceDriveProgress,
    DomSourceEventRuntimeLimits, DomSourceEventServiceLimits, DomSourceEventServiceProgress,
    DomSourceServices, DomStyleRuntimeLimits,
};
use surgeist_dom::*;
use surgeist_html::{HtmlDialogAction, HtmlDialogEventSource, HtmlDialogState};

fn source_limits(work: u64) -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, work, 128, 4096, 4096, 4)
}
fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 4,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: surgeist_css::CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 8,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}
fn fixture() -> (DomSession, DomHostedHostId, NodeId) {
    let mut session = DomSession::new(
        Dom::new(
            DomLimits::new(
                128,
                100_000,
                512,
                512,
                512,
                32,
                4_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(16, 256, 16_000_000, 16_000_000, 4_000_000).unwrap(),
            ),
        )
        .unwrap(),
        session_limits(),
    )
    .unwrap();
    let host = session
        .hosted_execution()
        .create_headless_host(&mut session.dom(), &AtomicBool::new(false))
        .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, &document, None);
    let subject = session
        .dom()
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    (session, host, subject)
}
fn parked(work: u64) -> (DomSession, DomHostedHostId, NodeId, DomFocusRequest) {
    let (mut session, host, subject) = fixture();
    let cancel = AtomicBool::new(false);
    let scope = session.source_execution_scope(host, &cancel).unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(event),
        ..
    } = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(subject),
            source_limits(work),
            &mut DomSourceServices {
                execution_scope: Some(&scope),
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("actual beforetoggle required")
    };
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 0,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    assert!(matches!(
        session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &event,
                &mut registry,
                &mut (),
                DomSourceEventServiceLimits {
                    source: source_limits(work),
                    dispatch: DomEventDispatchLimits {
                        path_items: 8,
                        snapshot_listeners: 0,
                        failure_records: 0,
                        callbacks: 0,
                        work: 100_000
                    }
                },
                &cancel
            )
            .unwrap(),
        DomSourceEventServiceProgress::Serviced { .. }
    ));
    let tasks = session.element_task_execution();
    let request = {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(event.operation(), source_limits(work), &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Dialog(queue)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual queue required")
        };
        assert_eq!(
            queue.action(),
            HtmlDialogAction::QueueToggle {
                old: HtmlDialogState::Closed,
                new: HtmlDialogState::Open,
                source: HtmlDialogEventSource::Null
            }
        );
        tasks.queue_dialog(&mut lease, &queue, &cancel).unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Dialog(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("actual remembered-focus wait required")
        };
        assert_eq!(request.action(), HtmlDialogAction::RememberFocusedElement);
        DomFocusRequest::RememberDialog(request)
    };
    (session, host, subject, request)
}
fn request(lease: &mut DomSourceLease<'_>, action: HtmlDialogAction) -> DomSourceDialogRequest {
    let DomSourceProgress::Request(DomSourceRequest::Dialog(request)) =
        lease.advance(&AtomicBool::new(false)).unwrap()
    else {
        panic!("actual Dialog request required")
    };
    assert_eq!(request.action(), action);
    request
}
fn focusing_request(
    lease: &mut DomSourceLease<'_>,
    action: surgeist_html::HtmlDialogFocusingAction,
) -> DomSourceDialogFocusingRequest {
    let DomSourceProgress::Request(DomSourceRequest::DialogFocusing(request)) =
        lease.advance(&AtomicBool::new(false)).unwrap()
    else {
        panic!("actual child Dialog focusing request required");
    };
    assert_eq!(request.action(), action);
    request
}
fn allow_request(lease: &mut DomSourceLease<'_>) -> DomFocusRequest {
    DomFocusRequest::AllowDialog(focusing_request(
        lease,
        surgeist_html::HtmlDialogFocusingAction::ReadAllowFocus,
    ))
}
// Measure only canonical preparation/retention and one resume on this exact
// original owner. Materialization-dependent work from another owner is not a
// transferable tariff. Runtime visits remain independently enumerated below.
fn source_focus_boundary(
    session: &mut DomSession,
    request: &DomFocusRequest,
    runtime_visits: u64,
    short: bool,
) -> (u64, u64, DomFocusTarget) {
    source_focus_boundary_with_limits(session, request, runtime_visits, short, source_limits)
}
fn source_focus_boundary_with_limits(
    session: &mut DomSession,
    request: &DomFocusRequest,
    runtime_visits: u64,
    short: bool,
    limits: fn(u64) -> DomSourceLimits,
) -> (u64, u64, DomFocusTarget) {
    let cancel = AtomicBool::new(false);
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    assert!(owner.borrow().focus.pending.is_none());
    let before_resume = session.dom().pending_source_edit().unwrap().spent_work();
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(request.operation(), limits(10_000_000), &cancel)
        .unwrap();
    let scaffold = lease.status().spent_work();
    let resume = scaffold - before_resume;
    let state = owner.borrow();
    let payload = lease
        .retain_hosted_runtime_payload(&state.source, size_of::<FocusFormation>(), &cancel)
        .unwrap();
    let preparation = lease
        .prepare_focus(request, &state.source, &cancel)
        .unwrap()
        .unwrap();
    let target = preparation.target();
    let formation = FocusFormation {
        preparation,
        payload,
    };
    let canonical = lease.status().spent_work() - scaffold;
    drop(formation);
    let next_scaffold = lease.status().spent_work() + resume;
    (
        next_scaffold + canonical + runtime_visits - u64::from(short),
        next_scaffold,
        target,
    )
}

#[test]
fn refused_acceptance_retains_original_absence_across_progress_and_permission_replacement() {
    let (mut session, host, subject, original) = parked(10_000_000);
    // Host qualification, retained Host, and actual headless observation.
    let (cap, scaffold, target) = source_focus_boundary(&mut session, &original, 3, false);
    assert_eq!(target, DomFocusTarget::RememberedHost(host));
    let cancel = AtomicBool::new(false);
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let basis = session.snapshot().stamp();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(original.operation(), source_limits(cap), &cancel)
            .unwrap();
        assert_eq!(lease.status().spent_work(), scaffold);
        assert!(
            matches!(hosted.service_focus(&mut lease,&original,&cancel),Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit{limit}))) if limit==cap)
        );
        assert_eq!(lease.snapshot().stamp(), basis);
        assert_eq!(lease.status().spent_work(), cap);
    }
    assert_eq!(
        owner.borrow().focus.pending.as_ref().unwrap().request(),
        &original
    );
    let observed_work = owner.borrow().work;
    // Unrelated real issuer progress cannot invalidate an immutable Host fact.
    // Raise the original Source policy first; spent work is never reset.
    {
        let mut dom = session.dom();
        let _lease = dom
            .raw_mut()
            .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
    }
    let mut replacement = session.hosted_execution();
    let second = replacement
        .create_host(&mut session.dom(), &cancel)
        .unwrap();
    assert_ne!(second, host);
    assert_eq!(session.snapshot().stamp(), basis);
    let after_progress = owner.borrow().work;
    assert!(after_progress > observed_work);
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(
            replacement
                .service_focus(&mut lease, &original, &cancel)
                .is_err()
        );
        cancel.store(false, std::sync::atomic::Ordering::Relaxed);
        assert!(owner.borrow().focus.pending.is_some());
        assert_eq!(owner.borrow().work, after_progress);
        assert!(
            replacement
                .service_focus(&mut lease, &original, &cancel)
                .unwrap()
        );
        assert!(owner.borrow().focus.pending.is_none());
        assert_eq!(
            owner.borrow().work,
            after_progress,
            "accepted retained result must not resample"
        );
        let next = allow_request(&mut lease);
        assert_eq!(next.subject(), subject);
        assert_eq!(
            lease.snapshot().stamp(),
            basis,
            "already-null observation is a semantic no-op"
        );
        assert!(matches!(
            replacement.service_focus(&mut lease, &original, &cancel),
            Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                DomError::InvalidSourceOperation
            )))
        ));
        // Old requests still qualify one matching Host and inspect both retained
        // Host rows. Canonical action rejection precedes a new observation.
        assert_eq!(owner.borrow().work, after_progress + 3);
        assert!(owner.borrow().focus.pending.is_none());
        assert_eq!(lease.snapshot().stamp(), basis);
        assert_eq!(allow_request(&mut lease), next);
        assert_eq!(
            lease
                .snapshot()
                .node(subject)
                .unwrap()
                .as_element()
                .unwrap()
                .common_state()
                .html()
                .unwrap()
                .previously_focused_element(),
            None
        );
    }
}
#[test]
fn retired_and_reused_host_cannot_validate_a_retained_original_result() {
    let (mut session, host, _, original) = parked(10_000_000);
    let (cap, _, target) = source_focus_boundary(&mut session, &original, 3, false);
    assert_eq!(target, DomFocusTarget::RememberedHost(host));
    let cancel = AtomicBool::new(false);
    let mut hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(original.operation(), source_limits(cap), &cancel)
            .unwrap();
        assert!(
            hosted
                .service_focus(&mut lease, &original, &cancel)
                .is_err()
        );
    }
    assert!(owner.borrow().focus.pending.is_some());
    {
        let mut dom = session.dom();
        let _lease = dom
            .raw_mut()
            .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
    }
    let basis = session.snapshot().stamp();
    hosted
        .destroy_host(&mut session.dom(), host, &cancel)
        .unwrap();
    let reused = hosted
        .create_headless_host(&mut session.dom(), &cancel)
        .unwrap();
    assert_ne!(reused, host);
    assert_eq!(session.snapshot().stamp(), basis);
    let work = owner.borrow().work;
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
        .unwrap();
    assert!(matches!(
        hosted.service_focus(&mut lease, &original, &cancel),
        Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
            DomError::HostedSourceUnavailable
        )))
    ));
    assert_eq!(
        owner.borrow().focus.pending.as_ref().unwrap().request(),
        &original
    );
    assert_eq!(owner.borrow().work, work);
}
#[test]
fn host_replacement_prices_both_real_backings_before_effect_and_releases_old_extent() {
    for scratch_short in [false, true] {
        let (mut session, host, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        // One actual bank slot is live; replacement needs two slots while the
        // original allocation remains live. The fixed owner includes FocusState.
        assert_eq!(owner.borrow().hosts.capacity(), 1);
        let baseline = size_of::<RefCell<HostedState>>() + size_of::<records::Host>();
        let candidate = 2 * size_of::<records::Host>();
        let mut limits = session_limits().hosted;
        limits.storage_bytes = baseline + candidate - usize::from(!scratch_short);
        limits.scratch_bytes = candidate - usize::from(scratch_short);
        hosted
            .set_limits(&mut session.dom(), limits, &cancel)
            .unwrap();
        let stamp = session.snapshot().stamp();
        assert!(matches!(
            hosted.create_headless_host(&mut session.dom(), &cancel),
            Err(DomHostedRuntimeError::Admission)
        ));
        {
            let state = owner.borrow();
            assert_eq!(state.hosts.len(), 1);
            assert_eq!(state.hosts[0].id, host);
            assert_eq!(state.hosts.capacity(), 1);
            assert!(state.focus.host_backing.is_some());
        }
        assert_eq!(session.snapshot().stamp(), stamp);
        limits.storage_bytes = baseline + candidate;
        limits.scratch_bytes = candidate;
        hosted
            .set_limits(&mut session.dom(), limits, &cancel)
            .unwrap();
        let second = hosted
            .create_headless_host(&mut session.dom(), &cancel)
            .unwrap();
        {
            let state = owner.borrow();
            assert_eq!(state.hosts.capacity(), 2);
            assert_eq!(state.hosts.len(), 2);
            assert_eq!(state.hosts[1].id, second);
        }
        let actual = owner.borrow_mut().retained_bytes(&cancel).unwrap();
        assert_eq!(actual, size_of::<RefCell<HostedState>>() + candidate);
        hosted
            .destroy_host(&mut session.dom(), second, &cancel)
            .unwrap();
        hosted
            .destroy_host(&mut session.dom(), host, &cancel)
            .unwrap();
        assert!(owner.borrow().hosts.is_empty());
        assert_eq!(
            owner.borrow().hosts.capacity(),
            2,
            "empty retained bank still owns its backing"
        );
        // A weak relative guard cannot prolong the Hosted owner or issuer.
        let weak = Rc::downgrade(&owner);
        drop(owner);
        drop(session);
        assert!(weak.upgrade().is_none());
        assert!(matches!(
            hosted.state(),
            Err(DomHostedRuntimeError::OwnerDisposed)
        ));
    }
}
#[test]
fn hosted_producer_storage_scratch_and_actual_three_visits_refuse_before_observation() {
    for kind in 0..3 {
        let (mut session, _, _, original) = parked(10_000_000);
        let cancel = AtomicBool::new(false);
        let mut hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let baseline = size_of::<RefCell<HostedState>>() + size_of::<records::Host>();
        let candidate = size_of::<FocusFormation>();
        let mut limits = session_limits().hosted;
        match kind {
            0 => limits.storage_bytes = baseline + candidate - 1,
            1 => limits.scratch_bytes = candidate - 1,
            _ => {}
        }
        hosted
            .set_limits(&mut session.dom(), limits, &cancel)
            .unwrap();
        if kind == 2 {
            let mut state = owner.borrow_mut();
            limits.work = state.work + 2;
            state.limits = limits;
        }
        let stamp = session.snapshot().stamp();
        {
            let mut dom = session.dom();
            let mut lease = dom
                .raw_mut()
                .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
                .unwrap();
            let result = hosted.service_focus(&mut lease, &original, &cancel);
            assert!(matches!(result, Err(DomHostedRuntimeError::Admission)));
            assert!(owner.borrow().focus.pending.is_none());
            assert_eq!(lease.snapshot().stamp(), stamp);
        }
        limits.storage_bytes = baseline + candidate;
        limits.scratch_bytes = candidate;
        limits.work = 1_000_000;
        hosted
            .set_limits(&mut session.dom(), limits, &cancel)
            .unwrap();
        let before = owner.borrow().work;
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        assert!(
            hosted
                .service_focus(&mut lease, &original, &cancel)
                .unwrap()
        );
        assert_eq!(owner.borrow().work - before, 3);
        assert!(owner.borrow().focus.pending.is_none());
        assert_eq!(allow_request(&mut lease).operation(), original.operation());
    }
}

// Genuine caller-owned temporary allocation used only to establish the canonical
// baseline. The byte oracle under test is the independent producer struct size.
// Value precedes guard so its physical backing disappears before its debit.
struct Filler {
    _bytes: Vec<u8>,
    _retention: DomHostedRuntimeRetention,
}
fn filler(
    lease: &mut DomSourceLease<'_>,
    source: &DomHostedSource,
    bytes: usize,
) -> Result<Filler, DomError> {
    let retention = lease.retain_hosted_runtime_payload(
        source,
        size_of::<Filler>() + bytes,
        &AtomicBool::new(false),
    )?;
    let mut value = Vec::new();
    value
        .try_reserve_exact(bytes)
        .expect("finite real fixture allocation");
    assert_eq!(value.capacity(), bytes);
    Ok(Filler {
        _bytes: value,
        _retention: retention,
    })
}
#[test]
fn original_source_bytes_price_actual_producer_header_exactly_and_release_temporary() {
    for short in [true, false] {
        let (mut session, _, _, original) = parked(10_000_000);
        let hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(
                original.operation(),
                source_limits(10_000_000),
                &AtomicBool::new(false),
            )
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
                    ..
                }) => high = middle - 1,
                Err(error) => {
                    panic!("canonical baseline calibration refused unexpectedly: {error:?}")
                }
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
        let result = hosted.service_focus(&mut lease, &original, &AtomicBool::new(false));
        if short {
            assert!(matches!(
                result,
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        limit: 32_000_000
                    }
                )))
            ));
            assert!(owner.borrow().focus.pending.is_none());
        } else {
            assert!(result.unwrap());
            assert!(owner.borrow().focus.pending.is_none());
        }
        assert_eq!(lease.snapshot().stamp(), basis);
        drop(value);
        if short {
            assert!(
                hosted
                    .service_focus(&mut lease, &original, &AtomicBool::new(false))
                    .unwrap()
            );
        }
        assert_eq!(allow_request(&mut lease).operation(), original.operation());
        // The real temporary formation released its debit after transfer. This
        // actual allocation independently demonstrates reclaimed capacity.
        let recovered = filler(&mut lease, &owner.borrow().source, candidate).unwrap();
        drop(recovered);
    }
}

mod ancestor;

fn retained_result() -> (DomSession, DomFocusRequest) {
    let (mut session, host, _, original) = parked(10_000_000);
    let (cap, _, target) = source_focus_boundary(&mut session, &original, 3, false);
    assert_eq!(target, DomFocusTarget::RememberedHost(host));
    let hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(
                original.operation(),
                source_limits(cap),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(
            matches!(hosted.service_focus(&mut lease,&original,&AtomicBool::new(false)),Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceWorkLimit {limit}))) if limit==cap)
        );
    }
    assert_eq!(
        hosted
            .state()
            .unwrap()
            .borrow()
            .focus
            .pending
            .as_ref()
            .unwrap()
            .request(),
        &original
    );
    (session, original)
}
#[test]
fn authentic_foreign_publication_refuses_without_consuming_either_original_result() {
    let (mut first, first_request) = retained_result();
    let (mut second, second_request) = retained_result();
    let cancel = AtomicBool::new(false);
    let first_hosted = first.hosted_execution();
    let first_owner = first_hosted.state().unwrap();
    let second_hosted = second.hosted_execution();
    let second_owner = second_hosted.state().unwrap();
    let mut foreign = first_owner.borrow_mut().focus.pending.take();
    let second_basis = second.snapshot().stamp();
    let second_work = second_owner.borrow().work;
    {
        let mut dom = second.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(
                second_request.operation(),
                source_limits(10_000_000),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            lease.accept_focus(
                &second_request,
                &second_owner.borrow().source,
                &mut foreign,
                &cancel
            ),
            Err(DomError::InvalidSourceOperation)
        ));
        assert_eq!(foreign.as_ref().unwrap().request(), &first_request);
        assert_eq!(lease.snapshot().stamp(), second_basis);
    }
    assert_eq!(
        second_owner
            .borrow()
            .focus
            .pending
            .as_ref()
            .unwrap()
            .request(),
        &second_request
    );
    assert_eq!(second_owner.borrow().work, second_work);
    first_owner.borrow_mut().focus.pending = foreign;
    for (session, original) in [(&mut first, &first_request), (&mut second, &second_request)] {
        let hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let work = owner.borrow().work;
        let mut dom = session.dom();
        let mut lease = dom
            .raw_mut()
            .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
            .unwrap();
        assert!(hosted.service_focus(&mut lease, original, &cancel).unwrap());
        assert!(owner.borrow().focus.pending.is_none());
        assert_eq!(owner.borrow().work, work);
        assert_eq!(allow_request(&mut lease).operation(), original.operation());
    }
}
#[test]
fn retained_result_cannot_cross_canonical_mutation_while_original_source_is_busy() {
    let (mut session, original) = retained_result();
    let cancel = AtomicBool::new(false);
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let basis = session.snapshot().stamp();
    let mut batch = DomBatch::new();
    batch.set_attribute(original.subject(), "data-unrelated", "");
    assert!(matches!(
        session.dom().apply(batch, &cancel),
        Err(DomError::SourceOperationPending)
    ));
    let result = session.source_execution().apply(
        &mut session.dom(),
        DomSourceEdit::set_attribute(original.subject(), "data-unrelated", ""),
        source_limits(10_000_000),
        &mut DomSourceServices {
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel,
    );
    assert!(matches!(
        result,
        Err(crate::DomSourceRuntimeError::Dom(
            DomError::SourceOperationPending
        ))
    ));
    assert_eq!(session.snapshot().stamp(), basis);
    assert_eq!(
        owner.borrow().focus.pending.as_ref().unwrap().request(),
        &original
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        original.operation()
    );
    let work = owner.borrow().work;
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
        .unwrap();
    assert!(
        hosted
            .service_focus(&mut lease, &original, &cancel)
            .unwrap()
    );
    assert_eq!(owner.borrow().work, work);
    assert_eq!(allow_request(&mut lease).operation(), original.operation());
}

#[test]
fn focus_preparation_rejects_wrong_observation_and_native_focusing_stays_unavailable() {
    let cancel = AtomicBool::new(false);
    let (mut session, host, _, original) = parked(10_000_000);
    let hosted = session.hosted_execution();
    let owner = hosted.state().unwrap();
    let source = &owner.borrow().source;
    let mut dom = session.dom();
    let mut lease = dom
        .raw_mut()
        .resume_source_edit(original.operation(), source_limits(10_000_000), &cancel)
        .unwrap();
    let stamp = lease.snapshot().stamp();
    let prepared = lease
        .prepare_focus(&original, source, &cancel)
        .unwrap()
        .unwrap();
    assert_eq!(prepared.target(), DomFocusTarget::RememberedHost(host));
    assert!(matches!(
        prepared.publish(DomFocusObservation::Denied),
        Err(DomError::InvalidSourceOperation)
    ));
    assert_eq!(lease.snapshot().stamp(), stamp);
    let prepared = lease
        .prepare_focus(&original, source, &cancel)
        .unwrap()
        .unwrap();
    let mut publication = Some(prepared.publish(DomFocusObservation::KnownNone).unwrap());
    lease
        .accept_focus(&original, source, &mut publication, &cancel)
        .unwrap();
    assert!(publication.is_none());
    let focusing = allow_request(&mut lease);
    assert_ne!(
        focusing, original,
        "each actual Show action has its own original occurrence"
    );
    assert!(
        lease
            .prepare_focus(&focusing, source, &cancel)
            .unwrap()
            .is_none()
    );
    assert_eq!(allow_request(&mut lease), focusing);
}

mod autofocus;
mod denial;
mod policy;
mod task_cleanup;
