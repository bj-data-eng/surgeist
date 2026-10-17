//! Actual sealed headless absence through original Source and event owners.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlDialogAction;
use surgeist_runtime::*;
#[path = "dom_source_focus/empty_popovers.rs"]
mod empty_popovers;
#[path = "dom_source_focus/lifetime.rs"]
mod lifetime;
fn limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 4_000_000, 20_000_000, 128, 8192, 8192, 4)
}
fn session() -> DomSession {
    DomSession::new(
        Dom::new(
            DomLimits::new(512, 2_000_000, 2048, 2048, 2048, 64, 8_000_000)
                .unwrap()
                .with_dispatch_custody(
                    DomDispatchCustodyLimits::new(64, 4096, 32_000_000, 32_000_000, 8_000_000)
                        .unwrap(),
                ),
        )
        .unwrap(),
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 8,
                storage_bytes: 32_000_000,
                work: 4_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 8,
                completions: 32,
                reports: 32,
                storage_bytes: 32_000_000,
                work: 4_000_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            dom_manipulation: DomManipulationRuntimeLimits {
                tasks: 32,
                storage_bytes: 32_000_000,
                work: 4_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    32_000_000,
                    32_000_000,
                    4_000_000,
                ),
                tasks: 32,
                storage_bytes: 32_000_000,
                scratch_bytes: 32_000_000,
                work: 4_000_000,
            },
        },
    )
    .unwrap()
}
fn native_dialog(session: &mut DomSession) -> NodeId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    batch.insert_before(&dialog, &document, None);
    session
        .dom()
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap()
}
fn hosted_dialog(
    session: &mut DomSession,
    headless: bool,
) -> (DomHostedHostId, DomHostedTopLevel, NodeId) {
    let mut hosted = session.hosted_execution();
    let host = if headless {
        hosted
            .create_headless_host(&mut session.dom(), &AtomicBool::new(false))
            .unwrap()
    } else {
        hosted
            .create_host(&mut session.dom(), &AtomicBool::new(false))
            .unwrap()
    };
    let top = hosted
        .create_top_level(&mut session.dom(), host, &AtomicBool::new(false))
        .unwrap();
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&dialog, body, None);
    let dialog = session
        .dom()
        .apply(batch, &AtomicBool::new(false))
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    (host, top, dialog)
}
fn start(
    session: &mut DomSession,
    subject: NodeId,
    scope: Option<&DomSourceExecutionScope>,
) -> DomSourceEventRequest {
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(subject),
            limits(),
            &mut DomSourceServices {
                execution_scope: scope,
                style_policy: &mut |_| None,
                url: None,
            },
            &AtomicBool::new(false),
        )
        .unwrap();
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } => request,
        other => panic!("actual beforetoggle required: {other:?}"),
    }
}
fn dispatch(session: &mut DomSession, request: &DomSourceEventRequest) -> DomEventDispatchReceipt {
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    match session
        .source_execution()
        .service_event(
            &mut session.dom(),
            request,
            &mut registry,
            &mut (),
            DomSourceEventServiceLimits {
                source: limits(),
                dispatch: DomEventDispatchLimits {
                    path_items: 32,
                    snapshot_listeners: 32,
                    failure_records: 32,
                    callbacks: 32,
                    work: 100_000,
                },
            },
            &AtomicBool::new(false),
        )
        .unwrap()
    {
        DomSourceEventServiceProgress::Serviced { receipt } => receipt,
        other => panic!("actual dispatch required: {other:?}"),
    }
}
fn resume(
    session: &mut DomSession,
    operation: DomSourceOperationId,
    scope: Option<&DomSourceExecutionScope>,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
    session.source_execution().resume(
        &mut session.dom(),
        operation,
        limits(),
        &mut DomSourceServices {
            execution_scope: scope,
            style_policy: &mut |_| None,
            url: None,
        },
        &AtomicBool::new(false),
    )
}
fn pending_action(
    progress: DomSourceDriveProgress,
    expected: HtmlDialogAction,
) -> DomSourceDialogRequest {
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Dialog(request),
            reason: DomSourceUnavailable::Dialog(action),
        } => {
            assert_eq!(action, expected);
            assert_eq!(request.action(), expected);
            request
        }
        other => panic!("actual Dialog wait required: {other:?}"),
    }
}
#[test]
fn native_explicit_headless_scope_remembers_authentic_none_then_waits_at_document_capture() {
    let mut session = session();
    let subject = native_dialog(&mut session);
    assert_eq!(session.snapshot().relevant_global(subject).unwrap(), None);
    let host = session
        .hosted_execution()
        .create_headless_host(&mut session.dom(), &AtomicBool::new(false))
        .unwrap();
    let scope = session
        .source_execution_scope(host, &AtomicBool::new(false))
        .unwrap();
    let before = session.snapshot();
    let event = start(&mut session, subject, Some(&scope));
    let receipt = dispatch(&mut session, &event);
    assert_eq!(receipt.callbacks(), 0);
    assert!(!receipt.default_prevented());
    assert!(receipt.failures().is_empty());
    let next = pending_action(
        resume(&mut session, event.operation(), Some(&scope)).unwrap(),
        HtmlDialogAction::CaptureNodeDocument,
    );
    assert_eq!(next.subject(), subject);
    assert_eq!(next.operation(), event.operation());
    let after = session.snapshot();
    let element = after.node(subject).unwrap().as_element().unwrap();
    assert_eq!(element.attribute(&"open".into()), Some(&"".into()));
    assert_eq!(
        element
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    assert_eq!(
        before.node(subject).unwrap().attribute(&"open".into()),
        None
    );
    assert_eq!(after.relevant_global(subject).unwrap(), None);
    let task = session
        .dom_manipulation_execution()
        .pending_task(0)
        .unwrap()
        .unwrap();
    assert!(task.dialog().unwrap().queue_association().is_none());
    drop(scope);
    pending_action(
        resume(&mut session, event.operation(), None).unwrap(),
        HtmlDialogAction::CaptureNodeDocument,
    );
    assert_eq!(session.snapshot().stamp(), after.stamp());
    assert_eq!(
        session
            .dom_manipulation_execution()
            .pending_task(0)
            .unwrap(),
        Some(task)
    );
}
#[test]
fn original_hosted_headless_scope_establishes_watcher_then_remembers_none() {
    let mut session = session();
    let (_, top, subject) = hosted_dialog(&mut session, true);
    let before = session.snapshot();
    let event = start(&mut session, subject, None);
    dispatch(&mut session, &event);
    pending_action(
        resume(&mut session, event.operation(), None).unwrap(),
        HtmlDialogAction::CaptureNodeDocument,
    );
    let after = session.snapshot();
    let dialog = after.node(subject).unwrap().as_dialog().unwrap();
    let watcher = dialog.close_watcher().unwrap();
    assert_eq!(watcher.global(), top.global());
    assert_eq!(watcher.subject(), subject);
    assert_eq!(
        after
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
    assert!(
        !before
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    let task = session
        .dom_manipulation_execution()
        .pending_task(0)
        .unwrap()
        .unwrap()
        .dialog()
        .unwrap();
    assert_eq!(task.queue_association().unwrap().global(), top.global());
    assert_eq!(task.queue_association().unwrap().document(), top.document());
}
#[test]
fn unbound_and_external_host_scopes_do_not_infer_absence_or_bind_on_resume() {
    let mut native = session();
    let subject = native_dialog(&mut native);
    let host = native
        .hosted_execution()
        .create_headless_host(&mut native.dom(), &AtomicBool::new(false))
        .unwrap();
    let scope = native
        .source_execution_scope(host, &AtomicBool::new(false))
        .unwrap();
    let event = start(&mut native, subject, None);
    dispatch(&mut native, &event);
    assert!(matches!(
        resume(&mut native, event.operation(), Some(&scope)),
        Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
            DomHostedError::Dom(DomError::InvalidSourceOperation)
        )))
    ));
    pending_action(
        resume(&mut native, event.operation(), None).unwrap(),
        HtmlDialogAction::RememberFocusedElement,
    );
    let mut external = session();
    let (_, _, subject) = hosted_dialog(&mut external, false);
    let event = start(&mut external, subject, None);
    dispatch(&mut external, &event);
    pending_action(
        resume(&mut external, event.operation(), None).unwrap(),
        HtmlDialogAction::RememberFocusedElement,
    );
}
#[test]
fn foreign_and_conflicting_host_permissions_preserve_the_original_operation() {
    let mut current = session();
    let subject = native_dialog(&mut current);
    let host = current
        .hosted_execution()
        .create_headless_host(&mut current.dom(), &AtomicBool::new(false))
        .unwrap();
    let scope = current
        .source_execution_scope(host, &AtomicBool::new(false))
        .unwrap();
    let mut foreign = session();
    let host = foreign
        .hosted_execution()
        .create_headless_host(&mut foreign.dom(), &AtomicBool::new(false))
        .unwrap();
    let wrong = foreign
        .source_execution_scope(host, &AtomicBool::new(false))
        .unwrap();
    let before = current.snapshot();
    let result = current.source_execution().apply(
        &mut current.dom(),
        DomSourceEdit::show_dialog(subject),
        limits(),
        &mut DomSourceServices {
            execution_scope: Some(&wrong),
            style_policy: &mut |_| None,
            url: None,
        },
        &AtomicBool::new(false),
    );
    assert!(matches!(
        result,
        Err(DomSourceRuntimeError::Hosted(
            DomHostedRuntimeError::Binding
        ))
    ));
    assert_eq!(current.snapshot().stamp(), before.stamp());
    assert!(current.dom().pending_source_edit().is_none());
    let other = current
        .hosted_execution()
        .create_headless_host(&mut current.dom(), &AtomicBool::new(false))
        .unwrap();
    let conflict = current
        .source_execution_scope(other, &AtomicBool::new(false))
        .unwrap();
    let event = start(&mut current, subject, Some(&scope));
    dispatch(&mut current, &event);
    let before = current.snapshot();
    assert!(resume(&mut current, event.operation(), Some(&conflict)).is_err());
    assert_eq!(current.snapshot().stamp(), before.stamp());
    pending_action(
        resume(&mut current, event.operation(), Some(&scope)).unwrap(),
        HtmlDialogAction::CaptureNodeDocument,
    );
}

#[test]
fn callback_adoption_preserves_original_host_scope_in_both_dispositions() {
    struct Context {
        adopted: Option<Result<DomCommit, DomError>>,
    }
    for original_headless in [true, false] {
        let mut session = session();
        let (_, first, subject) = hosted_dialog(&mut session, original_headless);
        let (_, second, _) = hosted_dialog(&mut session, !original_headless);
        let snapshot = session.snapshot();
        let html = snapshot.node(second.document().node()).unwrap().children()[0];
        let destination = snapshot.node(html).unwrap().children()[1];
        let event = start(&mut session, subject, None);
        let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 4,
            nested_dispatches: 4,
        })
        .unwrap();
        registry
            .add_listener(
                &session.dom(),
                subject,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |dom, _, context: &mut Context, _| {
                    let mut batch = DomBatch::new();
                    batch.insert_before(subject, destination, None);
                    context.adopted = Some(dom.apply(batch, &AtomicBool::new(false)));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let mut context = Context { adopted: None };
        let serviced = session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &event,
                &mut registry,
                &mut context,
                DomSourceEventServiceLimits {
                    source: limits(),
                    dispatch: DomEventDispatchLimits {
                        path_items: 32,
                        snapshot_listeners: 32,
                        failure_records: 32,
                        callbacks: 32,
                        work: 100_000,
                    },
                },
                &AtomicBool::new(false),
            )
            .unwrap();
        let DomSourceEventServiceProgress::Serviced { receipt } = serviced else {
            panic!("actual completed beforetoggle required")
        };
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
        assert!(context.adopted.unwrap().is_ok());
        let expected = if original_headless {
            HtmlDialogAction::CaptureNodeDocument
        } else {
            HtmlDialogAction::RememberFocusedElement
        };
        pending_action(
            resume(&mut session, event.operation(), None).unwrap(),
            expected,
        );
        let after = session.snapshot();
        assert_eq!(
            after.node(subject).unwrap().owner_document(),
            Some(second.document())
        );
        assert_eq!(after.node(subject).unwrap().parent(), Some(destination));
        assert_eq!(
            after.relevant_global(subject).unwrap(),
            Some(first.global())
        );
        assert_eq!(
            after
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .close_watcher()
                .unwrap()
                .global(),
            first.global()
        );
        assert_eq!(
            session
                .dom_manipulation_execution()
                .pending_task(0)
                .unwrap()
                .unwrap()
                .dialog()
                .unwrap()
                .queue_association()
                .unwrap()
                .global(),
            first.global()
        );
    }
}
#[test]
fn disposed_scope_permission_cannot_supply_absence_to_another_live_session() {
    let mut abandoned = session();
    let host = abandoned
        .hosted_execution()
        .create_headless_host(&mut abandoned.dom(), &AtomicBool::new(false))
        .unwrap();
    let stale = abandoned
        .source_execution_scope(host, &AtomicBool::new(false))
        .unwrap();
    drop(abandoned);
    let mut live = session();
    let subject = native_dialog(&mut live);
    let before = live.snapshot();
    let result = live.source_execution().apply(
        &mut live.dom(),
        DomSourceEdit::show_dialog(subject),
        limits(),
        &mut DomSourceServices {
            execution_scope: Some(&stale),
            style_policy: &mut |_| None,
            url: None,
        },
        &AtomicBool::new(false),
    );
    assert!(matches!(
        result,
        Err(DomSourceRuntimeError::Hosted(
            DomHostedRuntimeError::OwnerDisposed
        ))
    ));
    assert_eq!(live.snapshot().stamp(), before.stamp());
    assert!(live.dom().pending_source_edit().is_none());
}
#[test]
fn nested_native_show_selects_its_own_scope_instead_of_inheriting_parent_absence() {
    struct Context {
        source: DomSourceRuntime,
        scope: Option<DomSourceExecutionScope>,
        child: Option<Result<DomSourceDriveProgress, DomSourceRuntimeError>>,
        calls: usize,
    }
    for child_headless in [None, Some(false), Some(true)] {
        let mut session = session();
        let parent = native_dialog(&mut session);
        let child = native_dialog(&mut session);
        let parent_host = session
            .hosted_execution()
            .create_headless_host(&mut session.dom(), &AtomicBool::new(false))
            .unwrap();
        let parent_scope = session
            .source_execution_scope(parent_host, &AtomicBool::new(false))
            .unwrap();
        let scope = child_headless.map(|headless| {
            let host = if headless {
                session
                    .hosted_execution()
                    .create_headless_host(&mut session.dom(), &AtomicBool::new(false))
                    .unwrap()
            } else {
                session
                    .hosted_execution()
                    .create_host(&mut session.dom(), &AtomicBool::new(false))
                    .unwrap()
            };
            session
                .source_execution_scope(host, &AtomicBool::new(false))
                .unwrap()
        });
        let outer = start(&mut session, parent, Some(&parent_scope));
        let mut context = Context {
            source: session.source_execution(),
            scope,
            child: None,
            calls: 0,
        };
        let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 4,
            pending_events: 4,
            nested_dispatches: 4,
        })
        .unwrap();
        registry
            .add_listener(
                &session.dom(),
                parent,
                DomEventType::BeforeToggle,
                DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                    context.calls += 1;
                    event.prevent_default();
                    context.child = Some(context.source.apply(
                        dom,
                        DomSourceEdit::show_dialog(child),
                        limits(),
                        &mut DomSourceServices {
                            execution_scope: context.scope.as_ref(),
                            style_policy: &mut |_| None,
                            url: None,
                        },
                        &AtomicBool::new(false),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let result = session
            .source_execution()
            .service_event(
                &mut session.dom(),
                &outer,
                &mut registry,
                &mut context,
                DomSourceEventServiceLimits {
                    source: limits(),
                    dispatch: DomEventDispatchLimits {
                        path_items: 32,
                        snapshot_listeners: 32,
                        failure_records: 32,
                        callbacks: 32,
                        work: 100_000,
                    },
                },
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(matches!(
            result,
            DomSourceEventServiceProgress::Retained {
                dispatch: DomEventDispatchOutcome::RetainedSourceOperation(_)
            }
        ));
        assert_eq!(context.calls, 1);
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(inner),
            reason: DomSourceUnavailable::Event,
        } = context.child.take().unwrap().unwrap()
        else {
            panic!("actual child beforetoggle required")
        };
        assert_eq!(inner.parent_operation(), Some(outer.operation()));
        dispatch(&mut session, &inner);
        let expected = if child_headless == Some(true) {
            HtmlDialogAction::CaptureNodeDocument
        } else {
            HtmlDialogAction::RememberFocusedElement
        };
        pending_action(
            resume(&mut session, inner.operation(), None).unwrap(),
            expected,
        );
        let status = session.dom().pending_source_edit().unwrap();
        assert_eq!(status.operation(), inner.operation());
        assert_eq!(status.parent_operation(), Some(outer.operation()));
        assert_eq!(context.calls, 1);
        assert_eq!(
            session
                .snapshot()
                .node(parent)
                .unwrap()
                .attribute(&"open".into()),
            None
        );
    }
}
