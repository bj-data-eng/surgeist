//! Started native Details dispatch retains historical targets after retirement.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_dom::*;
use surgeist_html::Namespace;
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn session() -> DomSession {
    let dom = Dom::new(
        DomLimits::new(128, 200_000, 256, 256, 256, 32, 1_000_000)
            .unwrap()
            .with_dispatch_custody(
                surgeist_dom::DomDispatchCustodyLimits::new(
                    64, 4096, 16_000_000, 16_000_000, 4_000_000,
                )
                .unwrap(),
            )
            .with_details_toggle_intentions(16)
            .unwrap(),
    )
    .unwrap();
    DomSession::new(
        dom,
        DomSessionLimits {
            source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 16,
                reports: 16,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 16,
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
                tasks: 16,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap()
}

fn dispatch_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 64,
        snapshot_listeners: 16,
        failure_records: 16,
        callbacks: 16,
        work: 100_000,
    }
}

struct Context {
    element_tasks: DomElementTaskRuntime,
    target: NodeId,
    observations: Vec<(Option<DomEventTarget>, Option<NodeId>, DomEventPhase)>,
    edits: Vec<Result<DomCommit, DomError>>,
}

impl DomElementTaskEventContext for Context {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.element_tasks
    }
}

#[test]
fn started_hosted_details_dispatch_delivers_later_listener_after_target_retirement() {
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel()).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel()).unwrap()
    };
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut details = session.element_task_execution();
    let target = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(top.document(), Namespace::html(), "details");
        batch.insert_before(&target, body, None);
        batch.set_attribute(&target, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&target).unwrap().unwrap();
        assert_eq!(details.ingest(&mut dom, &cancel()).unwrap(), 1);
        target
    };
    let queued = details
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let association = queued.queue_association().unwrap();
    assert_eq!(association.document(), top.document());
    assert_eq!(association.global(), top.global());
    let mut context = Context {
        element_tasks: details,
        target,
        observations: Vec::new(),
        edits: Vec::new(),
    };
    let mut registry = DomEventDispatcher::<Context>::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    {
        let dom = session.dom();
        registry
            .add_listener(
                &dom,
                target,
                DomEventType::Toggle,
                DomEventCallback::new(|dom, _, context: &mut Context, event| {
                    context.observations.push((
                        event.current_target(),
                        event.target(),
                        event.phase(),
                    ));
                    let mut batch = DomBatch::new();
                    batch.detach(context.target);
                    batch.destroy(context.target);
                    context.edits.push(dom.apply(batch, &cancel()));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        registry
            .add_listener(
                &dom,
                target,
                DomEventType::Toggle,
                DomEventCallback::new(|dom, _, context: &mut Context, event| {
                    context.observations.push((
                        event.current_target(),
                        event.target(),
                        event.phase(),
                    ));
                    let mut batch = DomBatch::new();
                    batch.set_attribute(context.target, "retired-write", "forbidden");
                    context.edits.push(dom.apply(batch, &cancel()));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }

    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap();
    let DomElementTaskProgress::Complete {
        task,
        receipt,
        displaced_owner: None,
        dialog_toggle_disposition: None,
    } = progress
    else {
        panic!("started dispatch must deliver its retained exact target and finish: {progress:?}")
    };
    assert_eq!(task, queued);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert_eq!(
        context.observations,
        [
            (Some(target.into()), Some(target), DomEventPhase::AtTarget),
            (Some(target.into()), Some(target), DomEventPhase::AtTarget),
        ]
    );
    assert_eq!(context.edits.len(), 2);
    assert!(context.edits[0].is_ok());
    assert!(matches!(context.edits[1], Err(DomError::StaleNode)));
    assert!(matches!(
        session.snapshot().node(target),
        Err(DomError::StaleNode)
    ));
    let settings = hosted
        .settings(&session.dom(), top.global(), &cancel())
        .unwrap();
    assert_eq!(settings.document().id(), top.document());
    assert!(settings.environment_ready());
    assert!(settings.document().is_fully_active().unwrap());
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert!(matches!(
        DomElementTaskRuntime::run_next(
            &mut session.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                )
            },
            &cancel(),
        )
        .unwrap(),
        DomElementTaskProgress::Empty
    ));
}

#[derive(Clone, Copy)]
enum TargetEdit {
    Destroy,
    AdoptThenDestroy,
    AdoptAndDestroy,
    Detach,
}

struct ReentryContext {
    element_tasks: DomElementTaskRuntime,
    target: NodeId,
    destination: DocumentId,
    edit: TargetEdit,
    observations: Vec<(Option<DomEventTarget>, Option<NodeId>, DomEventPhase)>,
    edits: Vec<Result<DomCommit, DomError>>,
    adopted_documents: Vec<Result<Option<DocumentId>, DomError>>,
    replacement_callbacks: usize,
}

impl DomElementTaskEventContext for ReentryContext {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.element_tasks
    }
}

fn create_top(session: &mut DomSession) -> DomHostedTopLevel {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel()).unwrap();
    hosted.create_top_level(&mut dom, host, &cancel()).unwrap()
}

fn body(session: &DomSession, document: DocumentId) -> NodeId {
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    snapshot.node(html).unwrap().children()[1]
}

fn retire_top(session: &mut DomSession, top: DomHostedTopLevel) {
    let mut hosted = session.hosted_execution();
    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
        .unwrap();
    let mut completion = None;
    for _ in 0..256 {
        let progress = hosted
            .run_next(&mut session.dom(), queued.traversable(), &cancel())
            .unwrap();
        if matches!(progress, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            completion = Some(progress);
            break;
        }
    }
    assert!(matches!(
        completion,
        Some(DomHostedTaskProgress::Complete { .. })
    ));
    assert!(
        !session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}

fn queue_details(
    session: &mut DomSession,
    document: DocumentId,
    parent: NodeId,
) -> (NodeId, DomElementTaskRuntime, DomElementTask) {
    let mut details = session.element_task_execution();
    let mut dom = session.dom();
    let mut batch = DomBatch::new();
    let target = batch.create_element_ns(document, Namespace::html(), "details");
    batch.insert_before(&target, parent, None);
    batch.set_attribute(&target, "open", "");
    let commit = dom.apply(batch, &cancel()).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    assert_eq!(details.ingest(&mut dom, &cancel()).unwrap(), 1);
    let queued = details
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    (target, details, queued)
}

fn reentry_registry(
    session: &mut DomSession,
    target: NodeId,
) -> DomEventDispatcher<ReentryContext> {
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    let dom = session.dom();
    registry
        .add_listener(
            &dom,
            target,
            DomEventType::Toggle,
            DomEventCallback::new(|dom, _, context: &mut ReentryContext, event| {
                context
                    .observations
                    .push((event.current_target(), event.target(), event.phase()));
                let mut batch = DomBatch::new();
                batch.detach(context.target);
                match context.edit {
                    TargetEdit::Destroy => batch.destroy(context.target),
                    TargetEdit::AdoptThenDestroy => {
                        batch.adopt(context.target, context.destination);
                    }
                    TargetEdit::AdoptAndDestroy => {
                        batch.adopt(context.target, context.destination);
                        batch.destroy(context.target);
                    }
                    TargetEdit::Detach => {}
                }
                context.edits.push(dom.apply(batch, &cancel()));
                if matches!(context.edit, TargetEdit::AdoptThenDestroy) {
                    context.adopted_documents.push(
                        dom.snapshot()
                            .node(context.target)
                            .map(|node| node.owner_document()),
                    );
                    let mut batch = DomBatch::new();
                    batch.destroy(context.target);
                    context.edits.push(dom.apply(batch, &cancel()));
                }
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    registry
        .add_listener(
            &dom,
            target,
            DomEventType::Toggle,
            DomEventCallback::new(|dom, _, context: &mut ReentryContext, event| {
                context
                    .observations
                    .push((event.current_target(), event.target(), event.phase()));
                let mut batch = DomBatch::new();
                batch.set_attribute(context.target, "later-listener", "delivered");
                context.edits.push(dom.apply(batch, &cancel()));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    registry
}

fn assert_complete(progress: DomElementTaskProgress, queued: DomElementTask, callbacks: usize) {
    match progress {
        DomElementTaskProgress::Complete {
            task,
            receipt,
            displaced_owner: None,
            dialog_toggle_disposition: None,
        } => {
            assert_eq!(task, queued);
            assert_eq!(receipt.callbacks(), callbacks);
            assert!(receipt.failures().is_empty());
        }
        other => panic!("the original started Details dispatch must complete: {other:?}"),
    }
}

fn adoption_retirement_case(retire_before_resume: bool, same_edit: bool, closed_start: bool) {
    let mut session = session();
    let original = create_top(&mut session);
    let destination = create_top(&mut session);
    let document = if closed_start {
        let mut batch = DomBatch::new();
        let document = batch.create_html_document(original.document(), None);
        session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .document(&document)
            .unwrap()
    } else {
        original.document()
    };
    let parent = body(&session, document);
    let (target, details, queued) = queue_details(&mut session, document, parent);
    if closed_start {
        assert_eq!(queued.queue_association(), None);
    } else {
        let association = queued.queue_association().unwrap();
        assert_eq!(association.document(), original.document());
        assert_eq!(association.global(), original.global());
    }
    let mut context = ReentryContext {
        element_tasks: details,
        target,
        destination: destination.document(),
        edit: if same_edit {
            TargetEdit::AdoptAndDestroy
        } else {
            TargetEdit::AdoptThenDestroy
        },
        observations: Vec::new(),
        edits: Vec::new(),
        adopted_documents: Vec::new(),
        replacement_callbacks: 0,
    };
    let mut registry = reentry_registry(&mut session, target);
    let mut first_limits = dispatch_limits();
    if retire_before_resume {
        first_limits.callbacks = 1;
    }
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: first_limits,
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap();
    let canonical_edits = if same_edit { 1 } else { 2 };
    assert!(
        context.edits.len() >= canonical_edits,
        "the callback must attempt its actual adoption/destruction edits: {:?}",
        context.edits
    );
    for (index, edit) in context.edits[..canonical_edits].iter().enumerate() {
        assert!(
            edit.is_ok(),
            "actual callback canonical edit {index} must succeed before delivery eligibility can be assessed: {edit:?}"
        );
    }
    if !same_edit {
        assert_eq!(
            context.adopted_documents,
            [Ok(Some(destination.document()))],
            "the real first edit must establish the destination node Document"
        );
    }
    assert!(
        matches!(session.snapshot().node(target), Err(DomError::StaleNode)),
        "the actual target must retire before testing the retained invocation"
    );
    let progress = if retire_before_resume {
        assert!(matches!(
            progress,
            DomElementTaskProgress::Retained {
                task,
                dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks),
                displaced_owner: None,
            } if task == queued
        ));
        assert_eq!(context.observations.len(), 1);
        assert_eq!(context.edits.len(), canonical_edits);
        assert!(
            session
                .snapshot()
                .document(destination.document())
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        // Adoption and target retirement succeeded while the destination was
        // active. Its actual lifecycle now changes current activity while the
        // original dispatch retains its exact next listener and task Document.
        retire_top(&mut session, destination);
        DomElementTaskRuntime::run_next(
            &mut session.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        )
        .unwrap()
    } else {
        progress
    };
    let callbacks = if retire_before_resume { 1 } else { 2 };
    assert_complete(progress, queued, callbacks);
    assert_eq!(
        context.observations,
        vec![(Some(target.into()), Some(target), DomEventPhase::AtTarget); callbacks]
    );
    assert_eq!(context.edits.len(), canonical_edits + callbacks - 1);
    if retire_before_resume {
        // The original path and queued task Document stay unchanged. Eligibility
        // follows that same distinct Document's current inactive state.
        assert_eq!(context.edits.len(), canonical_edits);
    } else {
        assert!(matches!(
            context.edits[canonical_edits],
            Err(DomError::StaleNode)
        ));
    }
    if same_edit {
        assert!(context.adopted_documents.is_empty());
    } else {
        assert_eq!(
            context.adopted_documents,
            [Ok(Some(destination.document()))]
        );
    }
    assert!(matches!(
        session.snapshot().node(target),
        Err(DomError::StaleNode)
    ));
    let snapshot = session.snapshot();
    assert!(
        snapshot
            .document(original.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot
            .document(destination.document())
            .unwrap()
            .is_fully_active()
            .unwrap(),
        !retire_before_resume
    );
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert!(matches!(
        DomElementTaskRuntime::run_next(
            &mut session.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                )
            },
            &cancel()
        )
        .unwrap(),
        DomElementTaskProgress::Empty
    ));
}

#[test]
fn adoption_to_active_document_then_retirement_preserves_historical_delivery() {
    adoption_retirement_case(false, false, false);
}

#[test]
fn adoption_and_retirement_in_one_edit_use_the_active_destination() {
    adoption_retirement_case(false, true, false);
}

#[test]
fn adopted_then_retired_target_skips_later_listener_after_destination_becomes_inactive() {
    adoption_retirement_case(true, false, false);
}

#[test]
fn same_edit_adoption_and_retirement_use_destination_activity_at_resume() {
    adoption_retirement_case(true, true, false);
}

#[test]
fn never_hosted_retired_target_uses_current_hosted_activity_after_resume() {
    adoption_retirement_case(true, true, true);
}

#[test]
fn detach_during_started_dispatch_keeps_live_target_delivery_and_mutation() {
    let mut session = session();
    let top = create_top(&mut session);
    let parent = body(&session, top.document());
    let (target, details, queued) = queue_details(&mut session, top.document(), parent);
    let mut context = ReentryContext {
        element_tasks: details,
        target,
        destination: top.document(),
        edit: TargetEdit::Detach,
        observations: Vec::new(),
        edits: Vec::new(),
        adopted_documents: Vec::new(),
        replacement_callbacks: 0,
    };
    let mut registry = reentry_registry(&mut session, target);
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap();
    assert_complete(progress, queued, 2);
    assert_eq!(
        context.observations,
        vec![(Some(target.into()), Some(target), DomEventPhase::AtTarget); 2]
    );
    assert_eq!(context.edits.len(), 2);
    assert!(context.edits.iter().all(Result::is_ok));
    let snapshot = session.snapshot();
    let node = snapshot.node(target).unwrap();
    assert_eq!(node.parent(), None);
    assert_eq!(node.owner_document(), Some(top.document()));
    assert_eq!(
        node.attribute(&"later-listener".into()),
        Some(&"delivered".into())
    );
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn retained_retirement_resumes_without_replay_or_replacement_delivery_after_wrong_authority() {
    let mut session = session();
    let top = create_top(&mut session);
    let parent = body(&session, top.document());
    let (target, details, queued) = queue_details(&mut session, top.document(), parent);
    let mut context = ReentryContext {
        element_tasks: details,
        target,
        destination: top.document(),
        edit: TargetEdit::Destroy,
        observations: Vec::new(),
        edits: Vec::new(),
        adopted_documents: Vec::new(),
        replacement_callbacks: 0,
    };
    let mut registry = reentry_registry(&mut session, target);
    let mut low = dispatch_limits();
    low.callbacks = 1;
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: low,
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap();
    assert!(
        matches!(progress, DomElementTaskProgress::Retained { task, dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks), displaced_owner: None } if task == queued)
    );
    assert_eq!(context.observations.len(), 1);
    assert_eq!(context.edits.len(), 1);
    assert!(context.edits[0].is_ok());
    assert!(matches!(
        session.snapshot().node(target),
        Err(DomError::StaleNode)
    ));
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );

    // Drop/reacquire only weak execution permission. The original session owns
    // the exact retained task, event, registry binding and retirement evidence.
    let old = std::mem::replace(&mut context.element_tasks, session.element_task_execution());
    drop(old);
    let mut foreign = self::session();
    let foreign_before = foreign.snapshot().stamp();
    assert!(matches!(
        DomElementTaskRuntime::run_next(
            &mut foreign.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                )
            },
            &cancel()
        ),
        Err(DomElementTaskRuntimeError::Binding)
    ));
    assert_eq!(foreign.snapshot().stamp(), foreign_before);
    let mut wrong_registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    assert!(matches!(
        DomElementTaskRuntime::run_next(
            &mut session.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut wrong_registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                )
            },
            &cancel()
        ),
        Err(DomElementTaskRuntimeError::Dispatch(
            DomEventDispatchError::WrongContinuation
        ))
    ));
    assert_eq!(context.observations.len(), 1);
    assert_eq!(context.edits.len(), 1);
    assert_eq!(
        context
            .element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(queued)
    );

    let replacement = {
        let mut batch = DomBatch::new();
        let replacement = batch.create_element_ns(top.document(), Namespace::html(), "details");
        batch.insert_before(&replacement, parent, None);
        session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&replacement)
            .unwrap()
            .unwrap()
    };
    assert_ne!(replacement, target);
    registry
        .add_listener(
            &session.dom(),
            replacement,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut ReentryContext, _| {
                context.replacement_callbacks += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    assert!(matches!(
        registry.add_listener(
            &session.dom(),
            target,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, _: &mut ReentryContext, _| Ok(())),
            DomEventListenerOptions::default()
        ),
        Err(DomEventDispatchError::Dom(DomError::StaleNode))
    ));
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap();
    assert_complete(progress, queued, 2);
    assert_eq!(
        context.observations,
        vec![(Some(target.into()), Some(target), DomEventPhase::AtTarget); 2]
    );
    assert_eq!(context.edits.len(), 2);
    assert!(matches!(context.edits[1], Err(DomError::StaleNode)));
    assert_eq!(context.replacement_callbacks, 0);
    assert_eq!(
        session
            .snapshot()
            .node(replacement)
            .unwrap()
            .attribute(&"later-listener".into()),
        None
    );
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert!(matches!(
        DomElementTaskRuntime::run_next(
            &mut session.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                )
            },
            &cancel()
        )
        .unwrap(),
        DomElementTaskProgress::Empty
    ));
}
