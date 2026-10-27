//! Fullscreen7c38 exit lines 833–973: synchronous admission/selection and the
//! disconnected step-7 event/unfullscreen prefix precede parallel orientation.
//! Pending method return and passive scheduler quiescence never certify exit.
//!
//! Actual orientation/viewport completion, service replies and canonical
//! Document-identity retirement have no admitted public producer here.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;

use surgeist_dom::*;
use surgeist_runtime::*;

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 4_000_000, 100_000_000, 128, 8192, 8192, 8)
}
fn hosted_limits(tasks: usize) -> DomHostedRuntimeLimits {
    DomHostedRuntimeLimits {
        canonical: DomHostedLimits::new(
            DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
            32_000_000,
            32_000_000,
            20_000_000,
        ),
        tasks,
        storage_bytes: 32_000_000,
        scratch_bytes: 32_000_000,
        work: 20_000_000,
    }
}
fn session(exits: usize, events: usize) -> DomSession {
    DomSession::new(
        Dom::new(
            DomLimits::new(
                512,
                1_000_000,
                1024,
                1024,
                1024,
                128,
                16_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(64, 4096, 32_000_000, 32_000_000, 8_000_000).unwrap(),
            )
            .with_fullscreen_exits(exits)
            .unwrap()
            .with_fullscreen_events(events)
            .unwrap(),
        )
        .unwrap(),
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 8,
                storage_bytes: 32_000_000,
                work: 20_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 8,
                completions: 32,
                reports: 32,
                storage_bytes: 32_000_000,
                work: 20_000_000,
                css_input: surgeist_css::source::CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 32_000_000,
                work: 20_000_000,
            },
            hosted: hosted_limits(32),
        },
    )
    .unwrap()
}
fn top_level(session: &mut DomSession) -> DomHostedTopLevel {
    let cancel = AtomicBool::new(false);
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel).unwrap();
    hosted.create_top_level(&mut dom, host, &cancel).unwrap()
}
fn apply(session: &mut DomSession, batch: DomBatch) -> DomCommit {
    session.dom().apply(batch, &AtomicBool::new(false)).unwrap()
}
#[expect(
    clippy::result_large_err,
    reason = "the helper preserves the actual root and original producer failure for exact driving refusal assertions"
)]
fn source_result(
    session: &mut DomSession,
    edit: DomSourceEdit,
    limits: DomSourceLimits,
    cancel: &AtomicBool,
) -> Result<DomSourceDriveProgress, DomSourceDriveError> {
    session.source_execution().apply(
        &mut session.dom(),
        edit,
        limits,
        &mut DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        cancel,
    )
}
fn complete(session: &mut DomSession, edit: DomSourceEdit) -> DomSourceCompletion {
    let progress = source_result(session, edit, source_limits(), &AtomicBool::new(false)).unwrap();
    let DomSourceDriveProgress::Complete(done) = progress else {
        panic!("the synchronous call must complete after authentic handoff");
    };
    done
}
fn exit(session: &mut DomSession, document: DocumentId) -> DomSourceCompletion {
    complete(session, DomSourceEdit::exit_fullscreen(document))
}
fn pending(done: &DomSourceCompletion) -> DomFullscreenExitId {
    match done.fullscreen_exit_return() {
        Some(DomFullscreenExitReturn::Pending(id)) => id,
        other => panic!("actual active model state must return a native Pending result: {other:?}"),
    }
}
fn root(session: &DomSession, document: DocumentId) -> NodeId {
    session
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap()
}
fn detached(session: &mut DomSession, document: DocumentId) -> NodeId {
    let mut batch = DomBatch::new();
    let node = batch.create_element_ns(document, Namespace::html(), "div");
    apply(session, batch).created(&node).unwrap().unwrap()
}
fn fullscreen(session: &mut DomSession, node: NodeId) {
    let mut batch = DomBatch::new();
    batch.fullscreen_element_model(node);
    apply(session, batch);
}
fn unfullscreen(session: &mut DomSession, node: NodeId) {
    let mut batch = DomBatch::new();
    batch.unfullscreen_element_model(node);
    apply(session, batch);
}
fn subjects(snapshot: &DomSnapshot, document: DocumentId) -> Vec<NodeId> {
    let events = snapshot
        .document(document)
        .unwrap()
        .pending_fullscreen_events()
        .unwrap();
    for event in events {
        assert_eq!(event.document(), document);
        assert_eq!(event.kind(), DomFullscreenEventKind::Change);
    }
    events.iter().map(|event| event.subject()).collect()
}
fn status(session: &mut DomSession, id: DomFullscreenExitId, document: DocumentId, resize: bool) {
    let view = session.dom().fullscreen_exit_status(id).unwrap();
    assert_eq!(view.id(), id);
    assert_eq!(view.document(), document);
    assert_eq!(view.resize(), resize);
    assert_eq!(
        view.step(),
        DomFullscreenExitStep::FullyUnlockScreenOrientation
    );
}
fn first_orientation(
    session: &mut DomSession,
    traversable: DomHostedNavigableId,
    id: DomFullscreenExitId,
) {
    assert_eq!(
        session
            .hosted_execution()
            .run_next(&mut session.dom(), traversable, &AtomicBool::new(false))
            .unwrap(),
        DomHostedTaskProgress::FullscreenExitPending {
            invocation: id,
            step: DomFullscreenExitStep::FullyUnlockScreenOrientation,
        }
    );
}
fn quiescent(session: &mut DomSession, traversable: DomHostedNavigableId) {
    assert_eq!(
        session
            .hosted_execution()
            .run_next(&mut session.dom(), traversable, &AtomicBool::new(false))
            .unwrap(),
        DomHostedTaskProgress::Quiescent
    );
}
fn iframe(
    session: &mut DomSession,
    top: DomHostedTopLevel,
) -> (NodeId, DomHostedNavigableId, DocumentId) {
    let snapshot = session.snapshot();
    let html = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let closed = batch.create_html_document(top.document(), None);
    let container = batch.create_element_ns(&closed, Namespace::html(), "iframe");
    let container = apply(session, batch).created(&container).unwrap().unwrap();
    complete(session, DomSourceEdit::insert_before(container, body, None));
    let snapshot = session.snapshot();
    let child = snapshot
        .node(container)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let document = snapshot
        .hosted_navigable(child)
        .unwrap()
        .active_document()
        .unwrap();
    assert!(
        snapshot
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    (container, child, document)
}
fn settle_initial_history(session: &mut DomSession, traversable: DomHostedNavigableId) {
    for _ in 0..64 {
        if session
            .hosted_execution()
            .run_next(&mut session.dom(), traversable, &AtomicBool::new(false))
            .unwrap()
            == DomHostedTaskProgress::Idle
        {
            return;
        }
    }
    panic!("actual initial history did not settle within 64 advances");
}

#[test]
fn active_document_without_raw_fullscreen_returns_type_error_without_optional_capacity_or_job() {
    let mut session = session(0, 0);
    let top = top_level(&mut session);
    let before = session.snapshot();
    assert!(
        before
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let done = exit(&mut session, top.document());
    assert_eq!(
        done.fullscreen_exit_return(),
        Some(DomFullscreenExitReturn::RejectedTypeError)
    );
    assert!(done.commit().changes().changes().is_empty());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert!(subjects(&session.snapshot(), top.document()).is_empty());
    assert_eq!(
        session
            .hosted_execution()
            .run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(false))
            .unwrap(),
        DomHostedTaskProgress::Idle
    );
}

#[test]
fn disconnected_step_seven_queues_and_unfullscreens_before_the_actual_orientation_requirement() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let node = detached(&mut session, top.document());
    fullscreen(&mut session, node);
    let old = session.snapshot();
    let done = exit(&mut session, top.document());
    let id = pending(&done);
    assert_eq!(done.output(), None);
    let after = session.snapshot();
    assert_eq!(subjects(&after, top.document()), [node]);
    assert!(
        !after
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(
        after
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    assert!(
        after
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .pending_removals()
            .is_empty()
    );
    assert!(after.incomplete_source_operation().is_none());
    assert!(session.dom().pending_source_edit().is_none());
    assert!(
        old.node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(subjects(&old, top.document()).is_empty());
    status(&mut session, id, top.document(), true);
    first_orientation(&mut session, top.navigable(), id);
    let waiting = session.snapshot();
    quiescent(&mut session, top.navigable());
    assert_eq!(session.snapshot().stamp(), waiting.stamp());
    status(&mut session, id, top.document(), true);
}

#[test]
fn nonsimple_document_retains_false_resize_and_unfullscreens_only_the_disconnected_current_entry() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let connected = root(&session, top.document());
    let detached = detached(&mut session, top.document());
    fullscreen(&mut session, connected);
    fullscreen(&mut session, detached);
    assert!(
        !session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_simple_fullscreen_document()
            .unwrap()
    );
    let id = pending(&exit(&mut session, top.document()));
    status(&mut session, id, top.document(), false);
    let after = session.snapshot();
    assert!(
        after
            .node(connected)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(
        !after
            .node(detached)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        after
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[connected]
    );
    assert_eq!(subjects(&after, top.document()), [detached]);
    first_orientation(&mut session, top.navigable(), id);
}

#[test]
fn existing_source_removal_with_admitted_exit_capacity_finishes_its_outer_tail_after_handoff() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let root = root(&session, top.document());
    fullscreen(&mut session, root);
    let old = session.snapshot();
    let done = complete(&mut session, DomSourceEdit::detach(root));
    assert_eq!(done.fullscreen_exit_return(), None);
    let after = session.snapshot();
    assert_eq!(after.node(root).unwrap().parent(), None);
    assert_eq!(
        after.document(top.document()).unwrap().document_element(),
        None
    );
    assert!(
        !after
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(subjects(&after, top.document()), [root]);
    assert!(
        after
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    assert!(session.dom().pending_source_edit().is_none());
    assert!(after.incomplete_source_operation().is_none());
    assert_eq!(
        old.node(root).unwrap().parent(),
        Some(top.document().node())
    );
    let progress = session
        .hosted_execution()
        .run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(false))
        .unwrap();
    let DomHostedTaskProgress::FullscreenExitPending { invocation, step } = progress else {
        panic!("completed removal must retain its real asynchronous exit: {progress:?}");
    };
    assert_eq!(step, DomFullscreenExitStep::FullyUnlockScreenOrientation);
    status(&mut session, invocation, top.document(), true);
    let mut batch = DomBatch::new();
    batch.set_attribute(root, "data-after-removal", "kept");
    apply(&mut session, batch);
    quiescent(&mut session, top.navigable());
    assert_eq!(
        session
            .snapshot()
            .node(root)
            .unwrap()
            .attribute(&"data-after-removal".into()),
        Some(&"kept".into())
    );
}

#[test]
fn result_and_executor_drop_preserve_one_job_without_replaying_step_seven_or_overwriting_edits() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let a = detached(&mut session, top.document());
    let b = detached(&mut session, top.document());
    fullscreen(&mut session, a);
    let done = exit(&mut session, top.document());
    let id = pending(&done);
    drop(done);
    let mut first_permission = session.hosted_execution();
    assert_eq!(
        first_permission
            .run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(false))
            .unwrap(),
        DomHostedTaskProgress::FullscreenExitPending {
            invocation: id,
            step: DomFullscreenExitStep::FullyUnlockScreenOrientation
        }
    );
    drop(first_permission);
    fullscreen(&mut session, b);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "data-independent", "survives");
    apply(&mut session, batch);
    complete(
        &mut session,
        DomSourceEdit::create_comment(top.document().node(), "independent Source"),
    );
    let edited = session.snapshot();
    for _ in 0..3 {
        quiescent(&mut session, top.navigable());
        status(&mut session, id, top.document(), true);
    }
    let after = session.snapshot();
    assert_eq!(after.stamp(), edited.stamp());
    assert!(
        after
            .node(b)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        after
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[b]
    );
    assert_eq!(subjects(&after, top.document()), [a]);
    assert_eq!(
        after.node(a).unwrap().attribute(&"data-independent".into()),
        Some(&"survives".into())
    );
    assert!(session.dom().pending_source_edit().is_none());
}

#[test]
fn actual_history_execution_and_checkpoint_are_available_while_orientation_is_parked() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let (_, child, document) = iframe(&mut session, top);
    let global = session.snapshot().hosted_navigable(child).unwrap().global();
    let node = detached(&mut session, top.document());
    fullscreen(&mut session, node);
    let id = pending(&exit(&mut session, top.document()));
    let mut history_operation = None;
    let mut checkpoint_completed = false;
    let mut orientation_reached = false;
    for _ in 0..64 {
        let progress = session
            .hosted_execution()
            .run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(false))
            .unwrap();
        match progress {
            DomHostedTaskProgress::Executed {
                operation,
                document: actual_document,
                global: actual_global,
                kind: DomHostedTaskKind::History,
            } if actual_document == document && actual_global == global => {
                history_operation = Some(operation);
            }
            DomHostedTaskProgress::Complete { operation }
                if Some(operation) == history_operation =>
            {
                checkpoint_completed = true;
            }
            DomHostedTaskProgress::FullscreenExitPending { invocation, step } => {
                assert_eq!(invocation, id);
                assert_eq!(step, DomFullscreenExitStep::FullyUnlockScreenOrientation);
                orientation_reached = true;
            }
            _ => {}
        }
        if checkpoint_completed && orientation_reached {
            break;
        }
    }
    assert!(
        checkpoint_completed,
        "the real child History task and checkpoint were blocked by passive fullscreen work"
    );
    assert!(
        orientation_reached,
        "the admitted fullscreen job never reached its real requirement"
    );
    status(&mut session, id, top.document(), true);
    assert_eq!(subjects(&session.snapshot(), top.document()), [node]);
    assert!(session.dom().pending_source_edit().is_none());
}

#[test]
fn step_six_selects_the_real_simple_top_document_and_preserves_original_child_state() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let (_, _, child_document) = iframe(&mut session, top);
    settle_initial_history(&mut session, top.navigable());
    let top_root = root(&session, top.document());
    let child_root = root(&session, child_document);
    fullscreen(&mut session, top_root);
    fullscreen(&mut session, child_root);
    let id = pending(&exit(&mut session, child_document));
    status(&mut session, id, top.document(), true);
    assert!(subjects(&session.snapshot(), top.document()).is_empty());
    assert!(subjects(&session.snapshot(), child_document).is_empty());
    assert!(
        session
            .snapshot()
            .node(top_root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(
        session
            .snapshot()
            .node(child_root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    first_orientation(&mut session, top.navigable(), id);
}

#[test]
fn a_model_only_child_with_unflagged_parent_reports_the_actual_collector_assertion() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let (_, _, child_document) = iframe(&mut session, top);
    settle_initial_history(&mut session, top.navigable());
    let child_root = root(&session, child_document);
    fullscreen(&mut session, child_root);
    let before = session.snapshot();
    let error = source_result(
        &mut session,
        DomSourceEdit::exit_fullscreen(child_document),
        source_limits(),
        &AtomicBool::new(false),
    )
    .unwrap_err();
    assert!(
        matches!(error, surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Dom(DomError::FullscreenExitAssertionFailed { document })) if document == top.document())
    );
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert!(
        session
            .snapshot()
            .node(child_root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert!(subjects(&session.snapshot(), top.document()).is_empty());
    assert!(subjects(&session.snapshot(), child_document).is_empty());
}

#[test]
fn genuine_a_b_a_prefixes_form_an_ordered_set_and_transfer_retains_exact_historical_subjects() {
    let cancel = AtomicBool::new(false);
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let a = detached(&mut session, top.document());
    let b = detached(&mut session, top.document());
    let mut ids = Vec::new();
    for subject in [a, b, a] {
        fullscreen(&mut session, subject);
        let done = exit(&mut session, top.document());
        let id = pending(&done);
        assert!(!ids.contains(&id));
        ids.push(id);
        assert!(
            !session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_element()
                .unwrap()
                .fullscreen_flag()
        );
    }
    let old = session.snapshot();
    assert_eq!(subjects(&old, top.document()), [a, b]);
    let mut batch = DomBatch::new();
    batch.destroy(a);
    apply(&mut session, batch);
    let replacement = detached(&mut session, top.document());
    assert_ne!(replacement, a);
    let before_drain = session.snapshot();
    assert!(before_drain.node(a).is_err());
    assert_eq!(subjects(&before_drain, top.document()), [a, b]);
    assert_eq!(
        session
            .dom()
            .take_fullscreen_events(top.document(), 1, &cancel)
            .unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::FullscreenEvents,
            limit: 1
        }
    );
    assert_eq!(session.snapshot().stamp(), before_drain.stamp());
    assert_eq!(
        session
            .dom()
            .take_fullscreen_events(top.document(), 2, &AtomicBool::new(true))
            .unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(session.snapshot().stamp(), before_drain.stamp());
    let transferred = session
        .dom()
        .take_fullscreen_events(top.document(), 2, &cancel)
        .unwrap();
    assert_eq!(
        transferred.basis(),
        before_drain.document(top.document()).unwrap().stamp()
    );
    assert_eq!(
        transferred
            .events()
            .iter()
            .map(|event| event.subject())
            .collect::<Vec<_>>(),
        [a, b]
    );
    for event in transferred.events() {
        assert_eq!(event.document(), top.document());
        assert_eq!(event.kind(), DomFullscreenEventKind::Change);
    }
    assert!(!transferred.changes().changes().is_empty());
    assert_eq!(transferred.changes().to(), session.snapshot().stamp());
    assert!(subjects(&session.snapshot(), top.document()).is_empty());
    assert_eq!(subjects(&old, top.document()), [a, b]);
    let empty = session
        .dom()
        .take_fullscreen_events(top.document(), 0, &cancel)
        .unwrap();
    assert!(empty.events().is_empty());
    assert!(empty.changes().changes().is_empty());
    for id in ids {
        first_orientation(&mut session, top.navigable(), id);
    }
    quiescent(&mut session, top.navigable());
    assert!(subjects(&session.snapshot(), top.document()).is_empty());
    assert_eq!(
        transferred
            .events()
            .iter()
            .map(|event| event.subject())
            .collect::<Vec<_>>(),
        [a, b]
    );
    assert!(
        !session
            .snapshot()
            .node(replacement)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
}

#[test]
fn zero_optional_exit_or_event_capacity_refuses_before_disconnected_prefix_or_runnable_job() {
    for (exits, events, resource) in [
        (0, 8, DomResource::FullscreenExits),
        (8, 0, DomResource::FullscreenEvents),
    ] {
        let mut session = session(exits, events);
        let top = top_level(&mut session);
        let node = detached(&mut session, top.document());
        fullscreen(&mut session, node);
        let before = session.snapshot();
        let error = source_result(
            &mut session,
            DomSourceEdit::exit_fullscreen(top.document()),
            source_limits(),
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert!(
            matches!(error, surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Dom(DomError::ResourceLimit { resource: actual, limit: 0 })) if actual == resource)
        );
        let after = session.snapshot();
        assert_eq!(after.stamp(), before.stamp());
        assert!(
            after
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .fullscreen_flag()
        );
        assert_eq!(
            after
                .document(top.document())
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            &[node]
        );
        assert!(subjects(&after, top.document()).is_empty());
        assert_eq!(
            session
                .hosted_execution()
                .run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(false))
                .unwrap(),
            DomHostedTaskProgress::Idle
        );
    }
}

#[test]
fn failed_runtime_job_reservation_preserves_prefix_and_retry_hands_off_the_original_source_call() {
    let cancel = AtomicBool::new(false);
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let node = detached(&mut session, top.document());
    fullscreen(&mut session, node);
    let mut hosted = session.hosted_execution();
    hosted
        .set_limits(&mut session.dom(), hosted_limits(0), &cancel)
        .unwrap();
    let before = session.snapshot();
    let error = source_result(
        &mut session,
        DomSourceEdit::exit_fullscreen(top.document()),
        source_limits(),
        &cancel,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Hosted(
            DomHostedRuntimeError::Admission
        ))
    ));
    let refused = session.snapshot();
    assert_eq!(refused.stamp(), before.stamp());
    assert!(
        session
            .snapshot()
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    let original = session
        .dom()
        .pending_source_edit()
        .expect("the observed handoff retains its original Source owner")
        .operation();
    assert_eq!(refused.incomplete_source_operation(), Some(original));
    assert_eq!(
        refused
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap_err(),
        DomError::IncompleteSourceOperation
    );
    assert_eq!(
        refused
            .document(top.document())
            .unwrap()
            .pending_fullscreen_events()
            .unwrap_err(),
        DomError::IncompleteSourceOperation
    );
    // Observed handoff custody qualifies the current capture as incomplete
    // without changing its semantic revision. The unchanged stamp preserves
    // the prior complete capture's membership and queue evidence.
    assert_eq!(
        before
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[node]
    );
    assert!(subjects(&before, top.document()).is_empty());
    // The observed handoff still owns its synchronous retry guard. A refused
    // live hosted lookup is not an Idle observation or a job-count assertion.
    assert_eq!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap_err(),
        DomHostedRuntimeError::Dom(DomHostedError::Dom(DomError::SourceOperationPending))
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        original
    );
    assert_eq!(session.snapshot().stamp(), before.stamp());
    hosted
        .set_limits(&mut session.dom(), hosted_limits(32), &cancel)
        .unwrap();
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        original
    );
    let progress = session
        .source_execution()
        .resume(
            &mut session.dom(),
            original,
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = progress else {
        panic!("reserved job must finish the original synchronous handoff");
    };
    assert_eq!(done.operation(), original);
    let id = pending(&done);
    assert!(session.dom().pending_source_edit().is_none());
    assert!(session.snapshot().incomplete_source_operation().is_none());
    assert!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    assert_eq!(subjects(&session.snapshot(), top.document()), [node]);
    assert!(
        !session
            .snapshot()
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    first_orientation(&mut session, top.navigable(), id);
    quiescent(&mut session, top.navigable());
}

#[test]
fn preprefix_cancellation_preserves_model_and_queue_and_does_not_enqueue_background_work() {
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let node = detached(&mut session, top.document());
    fullscreen(&mut session, node);
    let before = session.snapshot();
    let error = source_result(
        &mut session,
        DomSourceEdit::exit_fullscreen(top.document()),
        source_limits(),
        &AtomicBool::new(true),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Dom(
            DomError::Cancelled
        )) | surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Style(
            DomStyleRuntimeError::Dom(DomError::Cancelled)
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert!(
        session
            .snapshot()
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
    assert_eq!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        &[node]
    );
    assert!(subjects(&session.snapshot(), top.document()).is_empty());
    assert_eq!(
        session
            .hosted_execution()
            .run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(false))
            .unwrap(),
        DomHostedTaskProgress::Idle
    );
    let id = pending(&exit(&mut session, top.document()));
    first_orientation(&mut session, top.navigable(), id);
}

#[test]
fn foreign_exit_id_cannot_observe_or_replace_the_local_invocation() {
    let mut local = session(8, 8);
    let local_top = top_level(&mut local);
    let local_node = detached(&mut local, local_top.document());
    fullscreen(&mut local, local_node);
    let local_id = pending(&exit(&mut local, local_top.document()));
    let mut foreign = session(8, 8);
    let foreign_top = top_level(&mut foreign);
    let foreign_node = detached(&mut foreign, foreign_top.document());
    fullscreen(&mut foreign, foreign_node);
    let foreign_id = pending(&exit(&mut foreign, foreign_top.document()));
    let before = local.snapshot();
    assert_eq!(
        local.dom().fullscreen_exit_status(foreign_id).unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(local.snapshot().stamp(), before.stamp());
    status(&mut local, local_id, local_top.document(), true);
    status(&mut foreign, foreign_id, foreign_top.document(), true);
    assert_eq!(
        subjects(&local.snapshot(), local_top.document()),
        [local_node]
    );
    first_orientation(&mut local, local_top.navigable(), local_id);
    quiescent(&mut local, local_top.navigable());
}

#[test]
fn original_receiver_hosted_retirement_is_not_canonical_identity_retirement_of_selected_top_document()
 {
    let cancel = AtomicBool::new(false);
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let (container, child, document) = iframe(&mut session, top);
    settle_initial_history(&mut session, top.navigable());
    let top_root = root(&session, top.document());
    let child_root = root(&session, document);
    fullscreen(&mut session, top_root);
    fullscreen(&mut session, child_root);
    let id = pending(&exit(&mut session, document));
    status(&mut session, id, top.document(), true);
    first_orientation(&mut session, top.navigable(), id);
    unfullscreen(&mut session, child_root);
    complete(&mut session, DomSourceEdit::detach(container));
    let mut retired = false;
    for _ in 0..128 {
        session
            .hosted_execution()
            .run_next(&mut session.dom(), top.navigable(), &cancel)
            .unwrap();
        if session
            .snapshot()
            .hosted_navigable(child)
            .unwrap()
            .is_retired()
        {
            retired = true;
            break;
        }
    }
    assert!(
        retired,
        "the original receiver's actual hosted teardown did not complete"
    );
    let after = session.snapshot();
    assert!(
        after.document(document).is_ok(),
        "hosted retirement does not retire canonical Document identity"
    );
    assert!(!after.document(document).unwrap().is_fully_active().unwrap());
    assert!(
        after
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(subjects(&after, document).is_empty());
    assert!(subjects(&after, top.document()).is_empty());
    status(&mut session, id, top.document(), true);
    // The selected top's original orientation work stays parked. Neither an
    // unrelated receiver teardown nor changed fullscreen flags supplies it.
    assert!(
        after
            .node(top_root)
            .unwrap()
            .as_element()
            .unwrap()
            .fullscreen_flag()
    );
}

#[test]
fn parked_exit_slots_and_pending_event_rows_each_enforce_their_independent_one_short_bound() {
    for (exits, events, resource) in [
        (1, 8, DomResource::FullscreenExits),
        (8, 1, DomResource::FullscreenEvents),
    ] {
        let mut session = session(exits, events);
        let top = top_level(&mut session);
        let a = detached(&mut session, top.document());
        let b = detached(&mut session, top.document());
        fullscreen(&mut session, a);
        let first = pending(&exit(&mut session, top.document()));
        first_orientation(&mut session, top.navigable(), first);
        fullscreen(&mut session, b);
        let before = session.snapshot();
        let error = source_result(
            &mut session,
            DomSourceEdit::exit_fullscreen(top.document()),
            source_limits(),
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert!(
            matches!(error, surgeist_runtime::DomSourceDriveError::Source(DomSourceRuntimeError::Dom(DomError::ResourceLimit { resource: actual, limit: 1 })) if actual == resource)
        );
        let after = session.snapshot();
        assert_eq!(after.stamp(), before.stamp());
        assert!(
            after
                .node(b)
                .unwrap()
                .as_element()
                .unwrap()
                .fullscreen_flag()
        );
        assert_eq!(
            after
                .document(top.document())
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            &[b]
        );
        assert_eq!(subjects(&after, top.document()), [a]);
        status(&mut session, first, top.document(), true);
        quiescent(&mut session, top.navigable());
    }
}

#[test]
fn nested_exit_handoff_restores_the_enclosing_real_dispatch_source_guard() {
    struct Context {
        source: DomSourceRuntime,
        document: DocumentId,
        invocation: Option<DomFullscreenExitId>,
        outer: DomSourceOperationId,
        calls: usize,
    }
    let cancel = AtomicBool::new(false);
    let mut session = session(8, 8);
    let top = top_level(&mut session);
    let node = detached(&mut session, top.document());
    fullscreen(&mut session, node);
    let snapshot = session.snapshot();
    let html = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&dialog, body, None);
    let dialog = apply(&mut session, batch)
        .created(&dialog)
        .unwrap()
        .unwrap();
    let progress = source_result(
        &mut session,
        DomSourceEdit::show_dialog(dialog),
        source_limits(),
        &cancel,
    )
    .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        reason: DomSourceUnavailable::Event,
    } = progress
    else {
        panic!("ordinary Show must reach its authentic beforetoggle dispatch");
    };
    let mut listeners = DomEventDispatcher::<Context>::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    listeners
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                context.calls += 1;
                assert_eq!(
                    dom.pending_source_edit().unwrap().operation(),
                    context.outer
                );
                let progress = context
                    .source
                    .apply(
                        dom,
                        DomSourceEdit::exit_fullscreen(context.document),
                        source_limits(),
                        &mut DomSourceServices {
                            meta: None,
                            execution_scope: None,
                            style_policy: &mut |_| None,
                            url: None,
                        },
                        &AtomicBool::new(false),
                    )
                    .unwrap();
                let DomSourceDriveProgress::Complete(done) = progress else {
                    panic!("parallel exit handoff must return to the actual outer callback");
                };
                context.invocation = Some(pending(&done));
                assert_eq!(
                    dom.pending_source_edit().unwrap().operation(),
                    context.outer
                );
                let mut batch = DomBatch::new();
                batch.set_attribute(dialog, "data-outer-callback", "kept");
                dom.apply(batch, &AtomicBool::new(false)).unwrap();
                event.prevent_default();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut context = Context {
        source: session.source_execution(),
        document: top.document(),
        invocation: None,
        outer: request.operation(),
        calls: 0,
    };
    let progress = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &request,
            &mut listeners,
            &mut context,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: DomEventDispatchLimits {
                    path_items: 32,
                    snapshot_listeners: 32,
                    failure_records: 32,
                    callbacks: 32,
                    work: 100_000,
                },
            },
            &cancel,
        )
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = progress else {
        panic!("a returned parallel call cannot strand the enclosing dispatch");
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.default_prevented());
    assert_eq!(context.calls, 1);
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        request.operation()
    );
    let progress = session
        .source_execution()
        .resume(
            &mut session.dom(),
            request.operation(),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = progress else {
        panic!("the actual canceled outer Show must complete its synchronous tail");
    };
    assert_eq!(done.fullscreen_exit_return(), None);
    assert_eq!(context.calls, 1);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"data-outer-callback".into()),
        Some(&"kept".into())
    );
    assert_eq!(subjects(&session.snapshot(), top.document()), [node]);
    first_orientation(&mut session, top.navigable(), context.invocation.unwrap());
    quiescent(&mut session, top.navigable());
    assert_eq!(context.calls, 1);
}
