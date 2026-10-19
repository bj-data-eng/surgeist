//! A handed-off Select intention keeps its retirement outcome across Session adoption.
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

fn session_limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: Default::default(),
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
            tasks: 16,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}

#[test]
fn runtime_select_completion_preserves_a_real_handoff_retirement_skip() {
    for foreign_optional_scope in [false, true] {
        let cancel = AtomicBool::new(false);
        let checkpoint = DomNativeCheckpointLimits::new(
            4_000_000,
            10_000_000,
            DomSlotQueryLimits::new(128, 512).unwrap(),
        )
        .unwrap();
        let mut dom = Dom::new(
            DomLimits::new(128, 32768, 512, 512, 128, 16, 1_000_000)
                .unwrap()
                .with_select_processing(32, 4_000_000, 10_000_000)
                .unwrap()
                .with_select_value_queries(
                    surgeist_html_semantics::HtmlCaptureLimits::new(
                        128, 512, 512, 32768, 128, 0, 0, 16_777_216,
                    ),
                    surgeist_html_semantics::HtmlSemanticLimits::new(
                        10_000_000, 1, 512, 0, 4_000_000, 512,
                    ),
                ),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element_ns(&document, Namespace::html(), "div");
        let select = batch.create_element_ns(&document, Namespace::html(), "select");
        let option = batch.create_element_ns(&document, Namespace::html(), "option");
        batch.set_attribute(&option, "selected", "");
        batch.insert_before(&option, &select, None);
        batch.insert_before(&select, &root, None);
        let survivor = batch.create_element_ns(&document, Namespace::html(), "select");
        let survivor_option = batch.create_element_ns(&document, Namespace::html(), "option");
        batch.set_attribute(&survivor_option, "selected", "");
        batch.insert_before(&survivor_option, &survivor, None);
        batch.insert_before(&survivor, &root, None);
        batch.insert_before(&root, &document, None);
        let setup = dom.apply(batch, &cancel).unwrap();
        let document = setup.document(&document).unwrap();
        let root = setup.created(&root).unwrap().unwrap();
        let select = setup.created(&select).unwrap().unwrap();
        let option = setup.created(&option).unwrap().unwrap();
        let survivor = setup.created(&survivor).unwrap().unwrap();
        let survivor_option = setup.created(&survivor_option).unwrap().unwrap();
        let mut batch = DomBatch::new();
        batch.detach(option);
        batch.detach(survivor_option);
        dom.apply(batch, &cancel).unwrap();
        assert_eq!(dom.pending_native_tasks(), 2);
        let DomNativeCheckpoint::Select(task) =
            dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
        else {
            panic!("the first real Option removal must hand off its captured Select");
        };
        let operation = task.operation();
        assert_eq!(task.select(), select);
        let historical = task.source().clone();
        assert_eq!(historical.node(select).unwrap().parent(), Some(root));
        assert!(historical.node(option).unwrap().parent().is_none());

        // Move the ORIGINAL unbound owner, with its actual active authorization and
        // later FIFO intention, into the public durable Runtime composition.
        let mut session = DomSession::new(dom, session_limits()).unwrap();
        assert_eq!(session.snapshot().owner(), historical.owner());
        let mut batch = DomBatch::new();
        batch.detach(select);
        batch.destroy(select);
        let retired = session.dom().apply(batch, &cancel).unwrap();
        assert!(
            retired
                .changes()
                .changes()
                .contains(&DomChange::SelectTaskCancelled {
                    operation,
                    select,
                    reason: DomSelectTaskCancellationReason::TargetDestroyed,
                })
        );
        let mut batch = DomBatch::new();
        let replacement = batch.create_element_ns(document, Namespace::html(), "select");
        batch.set_attribute(&replacement, "data-replacement", "unchanged");
        batch.insert_before(&replacement, root, None);
        let setup = session.dom().apply(batch, &cancel).unwrap();
        let replacement = setup.created(&replacement).unwrap().unwrap();
        assert_ne!(replacement, select);
        let before = session.snapshot();
        assert!(matches!(before.node(select), Err(DomError::StaleNode)));
        assert!(session.dom().pending_source_edit().is_none());
        let mut runtime = session.source_execution();
        let mut unexpected_policy = |_: &DomStyleSource| -> Option<DomStyleInlineCspObservation> {
            panic!("a retired Select skip must not request unrelated Style services")
        };
        let mut foreign_session = DomSession::new(
            Dom::new(DomLimits::new(32, 4096, 32, 32, 32, 8, 100_000).unwrap()).unwrap(),
            session_limits(),
        )
        .unwrap();
        let mut foreign_runtime = foreign_session.source_execution();
        assert!(matches!(
            foreign_runtime.complete_select(
                &mut session.dom(),
                &task,
                checkpoint,
                DomSourceLimits::new(0, 0, 0, 0, 0, 0, 0),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut unexpected_policy,
                    url: None,
                },
                &cancel,
            ),
            Err(DomSourceRuntimeError::ElementTasks(
                DomElementTaskRuntimeError::Binding
            ))
        ));
        assert!(matches!(
            foreign_runtime.complete_select(
                &mut foreign_session.dom(),
                &task,
                checkpoint,
                DomSourceLimits::new(0, 0, 0, 0, 0, 0, 0),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut unexpected_policy,
                    url: None,
                },
                &cancel,
            ),
            Err(DomSourceRuntimeError::Dom(DomError::ForeignOwner))
        ));
        for (checkpoint, cancelled, expected) in [
            (checkpoint, true, DomError::Cancelled),
            (
                DomNativeCheckpointLimits::new(
                    4_000_000,
                    0,
                    DomSlotQueryLimits::new(128, 512).unwrap(),
                )
                .unwrap(),
                false,
                DomError::SelectWorkLimit { limit: 0 },
            ),
            (
                DomNativeCheckpointLimits::new(
                    0,
                    10_000_000,
                    DomSlotQueryLimits::new(128, 512).unwrap(),
                )
                .unwrap(),
                false,
                DomError::ResourceLimit {
                    resource: DomResource::SelectScratch,
                    limit: 0,
                },
            ),
        ] {
            assert!(matches!(
                runtime.complete_select(
                    &mut session.dom(),
                    &task,
                    checkpoint,
                    DomSourceLimits::new(0, 0, 0, 0, 0, 0, 0),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut unexpected_policy,
                        url: None,
                    },
                    &AtomicBool::new(cancelled),
                ),
                Err(DomSourceRuntimeError::Dom(actual)) if actual == expected
            ));
            assert_eq!(session.snapshot().stamp(), before.stamp());
            assert!(session.dom().pending_source_edit().is_none());
        }
        // A real optional scope from another Session is irrelevant to this dead
        // task. It must not be qualified as though the skip required live services.
        let foreign_host = foreign_session
            .hosted_execution()
            .create_headless_host(&mut foreign_session.dom(), &cancel)
            .unwrap();
        let foreign_scope = foreign_session
            .source_execution_scope(foreign_host, &cancel)
            .unwrap();
        let result = runtime.complete_select(
            &mut session.dom(),
            &task,
            checkpoint,
            DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1),
            &mut DomSourceServices {
                execution_scope: foreign_optional_scope.then_some(&foreign_scope),
                style_policy: &mut unexpected_policy,
                url: None,
            },
            &cancel,
        );
        // The authorized executor consumes the canonical historical skip before
        // live-only Source admission or unrelated optional service qualification.
        assert!(
            result.is_ok(),
            "actual retired handoff must complete as a skip: {result:?}"
        );
        let DomSelectCompletionProgress::Skipped(completion) = result.unwrap() else {
            panic!("retirement must not fabricate a completed Source operation");
        };
        assert_eq!(
            completion.outcome(),
            DomSelectTaskOutcome::SkippedTargetDestroyed
        );
        assert!(completion.changes().is_none());
        assert!(matches!(
            runtime.complete_select(
                &mut session.dom(),
                &task,
                checkpoint,
                DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut unexpected_policy,
                    url: None,
                },
                &cancel,
            ),
            Err(DomSourceRuntimeError::Dom(DomError::InvalidNativeTask))
        ));
        let after = session.snapshot();
        assert_eq!(after.stamp(), before.stamp());
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(task.operation(), operation);
        assert_eq!(task.select(), select);
        assert_eq!(task.source().stamp(), historical.stamp());
        assert_eq!(historical.node(select).unwrap().parent(), Some(root));
        for id in [replacement, survivor, survivor_option] {
            assert_eq!(
                after.node(id).unwrap().parent(),
                before.node(id).unwrap().parent()
            );
            assert_eq!(after.node(id).unwrap().owner_document(), Some(document));
        }
        assert_eq!(
            after.node(root).unwrap().children(),
            &[survivor, replacement]
        );
        assert_eq!(
            after
                .node(replacement)
                .unwrap()
                .attribute(&"data-replacement".into()),
            Some(&"unchanged".into())
        );
        // The later live captured intention still belongs to its survivor. There is
        // no public Session checkpoint drain; retire it through the real owner and
        // require its distinct cancellation evidence rather than inventing a turn.
        let mut batch = DomBatch::new();
        batch.detach(survivor);
        batch.destroy(survivor);
        let cleanup = session.dom().apply(batch, &cancel).unwrap();
        let canceled: Vec<_> = cleanup
            .changes()
            .changes()
            .iter()
            .filter_map(|change| match change {
                DomChange::SelectTaskCancelled {
                    operation,
                    select,
                    reason,
                } => {
                    assert_eq!(*reason, DomSelectTaskCancellationReason::TargetDestroyed);
                    Some((*operation, *select))
                }
                _ => None,
            })
            .collect();
        assert_eq!(canceled.len(), 1);
        assert_eq!(canceled[0].1, survivor);
        assert_ne!(canceled[0].0, operation);
        assert!(matches!(
            session.try_shutdown(),
            DomShutdownOutcome::Complete
        ));
        // The foreign control Host still owns real lifecycle state. Terminal drop
        // disposes that unrelated fixture; this is not graceful hosted teardown.
        drop(foreign_session);
    }
}

#[test]
fn runtime_live_select_completion_returns_its_actual_source_progress() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(64, 8192, 128, 128, 64, 8, 1_000_000)
            .unwrap()
            .with_select_processing(8, 4_000_000, 10_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.set_attribute(&option, "selected", "");
    batch.insert_before(&option, &select, None);
    batch.insert_before(&select, &document, None);
    let setup = dom.apply(batch, &cancel).unwrap();
    let select = setup.created(&select).unwrap().unwrap();
    let option = setup.created(&option).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.detach(option);
    dom.apply(batch, &cancel).unwrap();
    let checkpoint = DomNativeCheckpointLimits::new(
        4_000_000,
        10_000_000,
        DomSlotQueryLimits::new(64, 128).unwrap(),
    )
    .unwrap();
    let DomNativeCheckpoint::Select(task) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("real selected Option removal queued its captured Select");
    };
    assert_eq!(task.select(), select);
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    let mut runtime = session.source_execution();
    let mut policy = |_: &DomStyleSource| -> Option<DomStyleInlineCspObservation> {
        panic!("the empty Select traversal has no Style boundary")
    };
    assert!(matches!(
        runtime
            .complete_select(
                &mut session.dom(),
                &task,
                checkpoint,
                DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel,
            )
            .unwrap(),
        DomSelectCompletionProgress::Source(DomSourceDriveProgress::Complete(_))
    ));
    assert!(session.dom().pending_source_edit().is_none());
    assert!(
        session
            .snapshot()
            .node(select)
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(task.select(), select);
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}

#[test]
fn runtime_live_select_copy_resumes_the_same_real_style_csp_request_once() {
    let cancel = AtomicBool::new(false);
    let source_limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 1);
    let checkpoint = DomNativeCheckpointLimits::new(
        4_000_000,
        10_000_000,
        DomSlotQueryLimits::new(256, 1024).unwrap(),
    )
    .unwrap();
    let mut dom = Dom::new(
        DomLimits::new(256, 100_000, 1024, 0, 0, 16, 1_000_000)
            .unwrap()
            .with_select_processing(32, 4_000_000, 10_000_000)
            .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.set_attribute(&option, "selected", "");
    let original_style = batch.create_element_ns(&document, Namespace::html(), "style");
    batch.set_attribute(&original_style, "type", "text/plain");
    let text = batch.create_text(&document, "a { color: red }");
    batch.insert_before(&text, &original_style, None);
    let other = batch.create_element_ns(&document, Namespace::html(), "option");
    batch.insert_before(&original_style, &other, None);
    let content = batch.create_element_ns(&document, Namespace::html(), "selectedcontent");
    batch.insert_before(&option, &select, None);
    batch.insert_before(&other, &select, None);
    batch.insert_before(&content, &select, None);
    batch.insert_before(&select, &document, None);
    let setup = dom.apply(batch, &cancel).unwrap();
    let select = setup.created(&select).unwrap().unwrap();
    let option = setup.created(&option).unwrap().unwrap();
    let original_style = setup.created(&original_style).unwrap().unwrap();
    let content = setup.created(&content).unwrap().unwrap();
    let mut batch = DomBatch::new();
    // Real selected-Option removal queues the captured Select. Its remaining
    // Option supplies a Style copy only when deferred selectedcontent runs.
    batch.detach(option);
    batch.set_attribute(original_style, "type", "text/css");
    dom.apply(batch, &cancel).unwrap();
    let DomNativeCheckpoint::Select(task) =
        dom.native_checkpoint_step(checkpoint, &cancel).unwrap()
    else {
        panic!("actual queued Select update");
    };
    let task_operation = task.operation();
    assert_eq!(task.select(), select);
    let historical = task.source().clone();
    assert!(historical.node(content).unwrap().children().is_empty());
    let mut session = DomSession::new(dom, session_limits()).unwrap();
    let mut runtime = session.source_execution();
    let DomSelectCompletionProgress::Source(DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Style(request),
        reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
    }) = runtime
        .complete_select(
            &mut session.dom(),
            &task,
            checkpoint,
            source_limits,
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("live Select must park at the copied Style's real CSP boundary");
    };
    assert_eq!(
        request.action(),
        DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp,)
    );
    let operation = request.source().operation();
    let ordinal = request.source().ordinal();
    let copied_style = request.source().element();
    assert_ne!(copied_style, original_style);
    let partial = session.snapshot();
    assert_eq!(partial.incomplete_source_operation(), Some(operation));
    assert_eq!(partial.node(copied_style).unwrap().parent(), Some(content));
    assert_eq!(partial.node(content).unwrap().children(), &[copied_style]);
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    drop(runtime);
    let mut runtime = session.source_execution();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Style(retry),
        reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
    } = runtime
        .resume_select(
            &mut session.dom(),
            operation,
            &task,
            source_limits,
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel,
        )
        .unwrap()
    else {
        panic!("same original Select continuation must remain pending");
    };
    assert_eq!(retry.source().operation(), operation);
    assert_eq!(retry.source().ordinal(), ordinal);
    assert_eq!(retry.source().stamp(), request.source().stamp());
    assert_eq!(retry.source().element(), copied_style);
    assert_eq!(session.snapshot().stamp(), partial.stamp());
    let mut evaluations = 0;
    let mut block = |source: &DomStyleSource| {
        assert_eq!(source.operation(), operation);
        assert_eq!(source.ordinal(), ordinal);
        assert_eq!(source.stamp(), request.source().stamp());
        assert_eq!(source.element(), copied_style);
        evaluations += 1;
        // A qualified full external CSP evaluation of the literal policy that
        // prohibits inline Style. Runtime does not parse/guess this outcome.
        Some(DomStyleInlineCspObservation::evaluated(
            source,
            std::sync::Arc::new(vec!["style-src 'none'".into()]),
            19,
            false,
        ))
    };
    assert!(matches!(
        runtime
            .resume_select(
                &mut session.dom(),
                operation,
                &task,
                source_limits,
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut block,
                    url: None,
                },
                &cancel,
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    assert_eq!(evaluations, 1);
    assert!(session.dom().pending_source_edit().is_none());
    let complete = session.snapshot();
    assert!(complete.incomplete_source_operation().is_none());
    assert_eq!(complete.node(content).unwrap().children(), &[copied_style]);
    assert!(
        complete
            .node(copied_style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(
        complete
            .node(original_style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    let copied_children = complete.node(copied_style).unwrap().children();
    assert_eq!(copied_children.len(), 1);
    assert_eq!(
        complete.node(copied_children[0]).unwrap().data(),
        Some(&"a { color: red }".into())
    );
    assert!(historical.node(content).unwrap().children().is_empty());
    assert_eq!(task.operation(), task_operation);
    assert_eq!(task.select(), select);
    assert!(
        session
            .style_execution()
            .pending_sheet(0)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        runtime.complete_select(
            &mut session.dom(),
            &task,
            checkpoint,
            source_limits,
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| panic!("consumed Select must not replay CSP"),
                url: None,
            },
            &cancel,
        ),
        Err(DomSourceRuntimeError::Dom(DomError::InvalidNativeTask))
    ));
    assert_eq!(session.snapshot().stamp(), complete.stamp());
    assert!(matches!(
        session.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}
