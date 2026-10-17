//! Real task-source allocation custody through the original Source admission.
//! These test the producer retention boundary, not fabricated queued delivery.
use super::admission_tests::update_limits;
use super::tests::{attest, fixture, limits, request};
use super::*;

struct Owner {
    source: crate::DomManipulationTaskSource,
    retained: Option<crate::DomManipulationTaskRetention>,
}
struct Allocation {
    values: Vec<NodeId>,
    _retained: crate::DomManipulationTaskRetention,
}
fn owner(dom: &mut Dom) -> Owner {
    let source = dom
        .bind_manipulation_task_source(&AtomicBool::new(false))
        .unwrap();
    let retained = source
        .retain_payload(dom, size_of::<Owner>(), &AtomicBool::new(false))
        .unwrap();
    Owner {
        source,
        retained: Some(retained),
    }
}
fn extent(count: usize) -> usize {
    size_of::<Allocation>() + count * size_of::<NodeId>()
}
fn allocate(
    owner: &Owner,
    dom: &mut Dom,
    target: NodeId,
    count: usize,
) -> Result<Allocation, DomError> {
    let retained = owner
        .source
        .retain_payload(dom, extent(count), &AtomicBool::new(false))?;
    let mut values = Vec::new();
    values.try_reserve_exact(count).unwrap();
    assert_eq!(
        values.capacity(),
        count,
        "physical owning fixture has this exact backing"
    );
    values.resize(count, target);
    Ok(Allocation {
        values,
        _retained: retained,
    })
}
#[test]
fn manipulation_actual_allocation_exact_source_limit_and_one_short_preserve_original_owner() {
    for short in [true, false] {
        let (mut dom, _executor, _, first, _) = fixture();
        let owner = owner(&mut dom);
        let event = request(&mut dom, first);
        let baseline = dom
            .source_observation_total(storage::Payload::default(), &AtomicBool::new(false))
            .unwrap();
        assert_eq!(
            owner.source.binding.bytes.load(Ordering::Relaxed),
            size_of::<Owner>()
        );
        assert!(owner.retained.is_some());
        let mut tight = limits();
        tight.retained_bytes = baseline.bytes + extent(13) - usize::from(short);
        update_limits(&mut dom, &event, tight);
        let before = dom.snapshot().stamp();
        let result = allocate(&owner, &mut dom, first, 13);
        if short {
            assert!(
                matches!(result, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == tight.retained_bytes)
            );
            assert_eq!(
                owner.source.binding.bytes.load(Ordering::Relaxed),
                size_of::<Owner>()
            );
        } else {
            let allocation = result.unwrap();
            assert_eq!(allocation.values.len(), 13);
            assert_eq!(
                dom.source_observation_total(storage::Payload::default(), &AtomicBool::new(false))
                    .unwrap()
                    .bytes,
                baseline.bytes + extent(13)
            );
            drop(allocation); // actual Vec first, its relative admission last
            assert_eq!(
                owner.source.binding.bytes.load(Ordering::Relaxed),
                size_of::<Owner>()
            );
            assert_eq!(
                dom.source_observation_total(storage::Payload::default(), &AtomicBool::new(false))
                    .unwrap()
                    .bytes,
                baseline.bytes
            );
        }
        assert_eq!(dom.snapshot().stamp(), before);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            event.operation()
        );
    }
}
#[test]
fn manipulation_allocation_survives_original_child_close_and_releases_under_same_parent() {
    let cancel = AtomicBool::new(false);
    let (mut dom, executor, _, first, second) = fixture();
    let owner = owner(&mut dom);
    let parent = request(&mut dom, first);
    let parent_permit = executor.begin_dispatch(&dom, &parent, &cancel).unwrap();
    let parent_scope = executor
        .enter_dispatch(&dom, &parent_permit, &cancel)
        .unwrap();
    let child = request(&mut dom, second);
    let before = dom
        .source_observation_total(storage::Payload::default(), &cancel)
        .unwrap();
    let allocation = allocate(&owner, &mut dom, first, 17).unwrap();
    assert_eq!(
        dom.source_observation_total(storage::Payload::default(), &cancel)
            .unwrap()
            .bytes,
        before.bytes + extent(17)
    );
    // Actual trusted-executor canonical protocol, explicitly no Runtime listener
    // claim: canceled Show finishes normally and restores the original parent.
    attest(&mut dom, &executor, &child, DomSourceEventOutcome::Canceled);
    let completion = dom
        .resume_source_edit(child.operation(), limits(), &cancel)
        .unwrap()
        .advance(&cancel)
        .unwrap();
    assert!(matches!(completion, DomSourceProgress::Complete(_)));
    assert_eq!(
        dom.pending_source_edit().unwrap().operation(),
        parent.operation()
    );
    assert_eq!(
        owner.source.binding.bytes.load(Ordering::Relaxed),
        size_of::<Owner>() + extent(17)
    );
    let retained = dom
        .source_observation_total(storage::Payload::default(), &cancel)
        .unwrap();
    drop(allocation);
    let released = dom
        .source_observation_total(storage::Payload::default(), &cancel)
        .unwrap();
    assert_eq!(retained.bytes - released.bytes, extent(17));
    assert_eq!(
        owner.source.binding.bytes.load(Ordering::Relaxed),
        size_of::<Owner>()
    );
    drop(parent_scope);
}

#[test]
fn queue_preparation_requires_known_prefix_and_closing_phase_headroom_before_effect() {
    let cancel = AtomicBool::new(false);
    // Queue changes ElementState + the incomplete marker. Reserve both known
    // possible prefix changes and one final close: five output records total.
    for (phases, outputs, failure) in [
        (2, 4096, Some(DomError::SourcePublicationLimit)),
        (3, 4, Some(DomError::SourceOutputLimit)),
        (3, 5, None),
    ] {
        let (mut dom, executor, _, first, _) = fixture();
        let owner = owner(&mut dom);
        let event = request(&mut dom, first);
        attest(
            &mut dom,
            &executor,
            &event,
            DomSourceEventOutcome::NotCanceled,
        );
        let mut policy = limits();
        policy.phases = phases;
        policy.outputs = outputs;
        let stamp = dom.snapshot().stamp();
        let mut lease = dom
            .resume_source_edit(event.operation(), policy, &cancel)
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Dialog(request)) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("the original uncanceled Show must reach QueueToggle");
        };
        let plan = lease
            .dialog_toggle_queue_plan(&request, &owner.source, 1, &cancel)
            .unwrap();
        assert!(matches!(
            lease.prepare_dialog_toggle_queue(
                &request,
                &owner.source,
                plan,
                &AtomicBool::new(true)
            ),
            Err(DomError::Cancelled)
        ));
        {
            let result = lease.prepare_dialog_toggle_queue(&request, &owner.source, plan, &cancel);
            if let Some(expected) = failure {
                match result {
                    Err(actual) => assert_eq!(actual, expected),
                    Ok(_) => panic!("insufficient original headroom admitted a queue candidate"),
                }
            } else {
                // A real admitted candidate exists; dropping it is pre-effect
                // rollback, not a fabricated Runtime queue publication.
                drop(result.unwrap());
            }
        }
        drop(lease);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.snapshot()
                .node(first)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker(),
            None
        );
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            event.operation()
        );
        let op = dom.source_operation.as_ref().unwrap();
        assert_eq!(
            op.draft.dialog_show.unwrap().cursor.action(),
            surgeist_html::HtmlDialogAction::QueueToggle {
                old: surgeist_html::HtmlDialogState::Closed,
                new: surgeist_html::HtmlDialogState::Open,
                source: surgeist_html::HtmlDialogEventSource::Null,
            }
        );
    }
}

#[test]
fn queue_preparation_reserves_both_show_focus_occurrences_before_any_queue_effect() {
    let cancel = AtomicBool::new(false);
    for remaining in [1_u64, 2] {
        let (mut dom, executor, _, first, _) = fixture();
        let owner = owner(&mut dom);
        let event = request(&mut dom, first);
        attest(
            &mut dom,
            &executor,
            &event,
            DomSourceEventOutcome::NotCanceled,
        );
        let stamp = dom.snapshot().stamp();
        {
            let mut lease = dom
                .resume_source_edit(event.operation(), limits(), &cancel)
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Dialog(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("the genuine original Show must reach QueueToggle")
            };
            // The real event receipt is rebased by advance above. Only now
            // does this actual operation own a Draft; place the private
            // near-exhaustion counter at the queue-preparation boundary.
            lease
                .dom
                .source_operation
                .as_mut()
                .unwrap()
                .draft
                .source_invocation = u64::MAX - remaining;
            let plan = lease
                .dialog_toggle_queue_plan(&request, &owner.source, 1, &cancel)
                .unwrap();
            let result = lease.prepare_dialog_toggle_queue(&request, &owner.source, plan, &cancel);
            if remaining == 1 {
                assert!(matches!(result, Err(DomError::RepresentationExhausted)));
            } else {
                drop(result.expect("Remember plus focusing fit exactly two original ordinals"));
            }
        }
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.snapshot()
                .node(first)
                .unwrap()
                .as_dialog()
                .unwrap()
                .toggle_task_tracker(),
            None
        );
        assert_eq!(
            dom.source_operation
                .as_ref()
                .unwrap()
                .draft
                .source_invocation,
            u64::MAX - remaining
        );
        assert!(matches!(
            dom.source_operation
                .as_ref()
                .unwrap()
                .draft
                .dialog_show
                .unwrap()
                .invocation,
            Invocation::Unissued
        ));
    }
}
