//! Fixed two-feature inheritance and actual denied Dialog focus composition.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlValueInterpreter;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 4_000_000, 20_000_000, 128, 8192, 8192, 4)
}
fn dispatch_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 32,
        failure_records: 32,
        callbacks: 32,
        work: 100_000,
    }
}
fn headless_top() -> (DomSession, DomHostedTopLevel) {
    let dom = Dom::new(
        DomLimits::new(
            512,
            2_000_000,
            2048,
            2048,
            2048,
            64,
            8_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 32_000_000, 32_000_000, 8_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut session = DomSession::new(
        dom,
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
            element_tasks: DomElementTaskRuntimeLimits {
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
    .unwrap();
    let mut hosted = session.hosted_execution();
    let host = hosted
        .create_headless_host(&mut session.dom(), &cancel())
        .unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap();
    (session, top)
}

fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let html = snapshot.node(document.node()).unwrap().children()[0];
    snapshot.node(html).unwrap().children()[1]
}

fn container(
    session: &mut DomSession,
    parent: DocumentId,
    local: &str,
    allow: Option<&str>,
    sandbox: Option<&str>,
    legacy: bool,
) -> (NodeId, DocumentId) {
    let destination = body(&session.snapshot(), parent);
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(parent, None);
    let element = batch.create_element_ns(&detached, Namespace::html(), local);
    if let Some(allow) = allow {
        batch.set_attribute(&element, "allow", allow);
    }
    if let Some(sandbox) = sandbox {
        batch.set_attribute(&element, "sandbox", sandbox);
    }
    if legacy {
        batch.set_attribute(&element, "allowfullscreen", "");
    }
    let element = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&element)
        .unwrap()
        .unwrap();
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(element, destination, None),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(progress, DomSourceDriveProgress::Complete(_)),
        "actual initial child construction must complete: {progress:?}"
    );
    let snapshot = session.snapshot();
    let child = snapshot
        .node(element)
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
    (element, document)
}
fn policy(snapshot: &DomSnapshot, document: DocumentId) -> DomInitialPermissionsPolicy {
    let policy = snapshot
        .document(document)
        .unwrap()
        .metadata()
        .initial_permissions_policy()
        .unwrap();
    assert_eq!(policy.profile(),DomInitialPermissionsPolicyProfile::FullscreenAndFocusWithoutUserActivationSelfOpaqueInitial);
    policy
}
fn dialog(session: &mut DomSession, document: DocumentId) -> NodeId {
    let parent = body(&session.snapshot(), document);
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&subject, parent, None);
    session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap()
}
fn drive(
    session: &mut DomSession,
    edit: DomSourceEdit,
    cancelable: bool,
    calls: &mut Vec<bool>,
) -> DomSourceDriveProgress {
    let event = match session
        .source_execution()
        .apply(
            &mut session.dom(),
            edit,
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(event),
            ..
        } => event,
        other => panic!("real Dialog method must first fire beforetoggle: {other:?}"),
    };
    assert_eq!(event.cancelable(), cancelable);
    assert_eq!(event.toggle_event().unwrap().source(), None);
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            event.subject(),
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |_, _, calls: &mut Vec<bool>, event| {
                assert_eq!(event.cancelable(), cancelable);
                calls.push(event.cancelable());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    match session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &event,
            &mut registry,
            calls,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: dispatch_limits(),
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceEventServiceProgress::Serviced { receipt } => {
            assert_eq!(receipt.callbacks(), 1);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
        }
        other => panic!("actual dispatcher receipt must finish: {other:?}"),
    }
    session
        .source_execution()
        .resume(
            &mut session.dom(),
            event.operation(),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
}
fn completed(progress: DomSourceDriveProgress) {
    match progress {
        DomSourceDriveProgress::Complete(done) => assert!(!done.cancellation_requested()),
        other => panic!("denied focus must reach actual completion: {other:?}"),
    }
}
fn document_inert(snapshot: &DomSnapshot, document: DocumentId) -> HtmlSemanticFact {
    // This actual hosted child contributes its one complete-empty Global
    // custom-element registry, independently of having no definitions.
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(128, 4096, 256, 32768, 128, 1, 0, 16_777_216),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &cancel(),
        )
        .unwrap();
    capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(document.node()).unwrap(),
                HtmlSemanticQuery::Inertness,
            )],
            HtmlValueInterpreter::new(16384),
            HtmlSemanticLimits::new(1_000_000, 16, 1024, 256, 1_000_000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}

#[test]
fn exact_feature_directives_inherit_independently_with_only_fullscreen_legacy_fallback() {
    use DomPermissionsPolicyValue::{Disabled as D, Enabled as E};
    for (allow, legacy, fullscreen, focus) in [
        (None, false, E, E),
        (Some(""), false, E, E),
        (Some("focus-without-user-activation 'none'"), false, E, D),
        (Some("fullscreen 'none'"), false, D, E),
        (
            Some("fullscreen 'none'; focus-without-user-activation 'none'"),
            false,
            D,
            D,
        ),
        (Some("focus-without-user-activation *"), false, E, E),
        (
            Some(
                "focus-without-user-activation 'none'; fullscreen 'none'; focus-without-user-activation 'self'",
            ),
            false,
            D,
            E,
        ),
        (
            Some("fullscreen 'none'; focus-without-user-activation 'none'; fullscreen *"),
            false,
            E,
            D,
        ),
        (Some("focus-without-user-activation 'none'"), true, E, D),
        (Some("FOCUS-WITHOUT-USER-ACTIVATION 'none'"), false, E, E),
        (Some("focus-without-user-activation 'SeLf'"), false, E, E),
    ] {
        let (mut session, top) = headless_top();
        assert_eq!(
            policy(&session.snapshot(), top.document()),
            DomInitialPermissionsPolicy::new(E, E)
        );
        let (_, child) = container(&mut session, top.document(), "iframe", allow, None, legacy);
        assert_eq!(
            policy(&session.snapshot(), child),
            DomInitialPermissionsPolicy::new(fullscreen, focus),
            "raw allow={allow:?}, allowfullscreen={legacy}"
        );
    }
}
#[test]
fn actual_sandbox_origins_frame_algorithm_and_parent_denial_keep_feature_values_separate() {
    use DomPermissionsPolicyValue::{Disabled as D, Enabled as E};
    for (sandbox, allow, legacy, fullscreen, focus, same_origin) in [
        ("", None, false, D, D, false),
        ("", None, true, E, D, false),
        (
            "",
            Some("focus-without-user-activation *"),
            false,
            D,
            E,
            false,
        ),
        ("allow-same-origin", None, false, E, E, true),
    ] {
        let (mut session, top) = headless_top();
        let before = session.snapshot();
        let (_, child) = container(
            &mut session,
            top.document(),
            "iframe",
            allow,
            Some(sandbox),
            legacy,
        );
        let snapshot = session.snapshot();
        assert_eq!(
            snapshot.document(child).unwrap().metadata().origin()
                == before.document(top.document()).unwrap().metadata().origin(),
            same_origin
        );
        assert_eq!(
            policy(&snapshot, child),
            DomInitialPermissionsPolicy::new(fullscreen, focus)
        );
    }
    let (mut session, top) = headless_top();
    let (_, frame) = container(
        &mut session,
        top.document(),
        "frame",
        Some("fullscreen 'none'; focus-without-user-activation 'none'"),
        Some(""),
        false,
    );
    assert_eq!(
        policy(&session.snapshot(), frame),
        DomInitialPermissionsPolicy::new(E, E)
    );
    for (parent_allow, expected) in [
        (
            "focus-without-user-activation 'none'",
            DomInitialPermissionsPolicy::new(E, D),
        ),
        ("fullscreen 'none'", DomInitialPermissionsPolicy::new(D, E)),
    ] {
        let (mut session, top) = headless_top();
        let (_, parent) = container(
            &mut session,
            top.document(),
            "iframe",
            Some(parent_allow),
            None,
            false,
        );
        let (_, child) = container(
            &mut session,
            parent,
            "iframe",
            Some("fullscreen *; focus-without-user-activation *"),
            None,
            false,
        );
        assert_eq!(policy(&session.snapshot(), child), expected);
    }
}
#[test]
fn later_raw_attribute_changes_do_not_recompute_the_complete_constructor_policy() {
    use DomPermissionsPolicyValue::{Disabled as D, Enabled as E};
    let (mut session, top) = headless_top();
    let (iframe, child) = container(
        &mut session,
        top.document(),
        "iframe",
        Some("fullscreen 'none'; focus-without-user-activation 'none'"),
        None,
        false,
    );
    let before = session.snapshot();
    assert_eq!(
        policy(&before, child),
        DomInitialPermissionsPolicy::new(D, D)
    );
    for edit in [
        DomSourceEdit::set_attribute(
            iframe,
            "allow",
            "fullscreen *; focus-without-user-activation *",
        ),
        DomSourceEdit::set_attribute(iframe, "allowfullscreen", ""),
    ] {
        let progress = session
            .source_execution()
            .apply(
                &mut session.dom(),
                edit,
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    }
    assert_eq!(
        policy(&session.snapshot(), child),
        DomInitialPermissionsPolicy::new(D, D)
    );
    assert_eq!(
        policy(&before, child),
        DomInitialPermissionsPolicy::new(D, D)
    );
    assert_eq!(
        policy(&session.snapshot(), top.document()),
        DomInitialPermissionsPolicy::new(E, E)
    );
}
#[test]
fn enabled_permission_keeps_positive_focusing_pending_even_without_any_activation() {
    use surgeist_html::{HtmlDialogFocusControl, HtmlDialogFocusingAction};
    let (mut session, top) = headless_top();
    let (_, child) = container(&mut session, top.document(), "iframe", None, None, false);
    let subject = dialog(&mut session, child);
    let mut calls = Vec::new();
    let progress = drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    );
    let expected = HtmlDialogFocusingAction::RunFocusingSteps {
        control: HtmlDialogFocusControl::Subject,
    };
    assert!(matches!(progress, DomSourceDriveProgress::Pending {
        request: DomSourceRequest::DialogFocusing(request),
        reason: DomSourceUnavailable::DialogFocusing(action),
    } if request.subject() == subject && action == expected && request.action() == expected));
    assert_eq!(calls, [true]);
    assert!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .is_modal()
    );
    assert!(matches!(
        session.snapshot().document(child).unwrap().top_layer(),
        Err(DomError::IncompleteSourceOperation)
    ));
}
#[test]
fn real_denied_modal_close_and_reshow_keep_pending_raw_membership_without_readding() {
    let (mut session, top) = headless_top();
    let (_, child) = container(
        &mut session,
        top.document(),
        "iframe",
        Some("focus-without-user-activation 'none'"),
        None,
        false,
    );
    let subject = dialog(&mut session, child);
    let mut calls = Vec::new();
    completed(drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    ));
    let shown = session.snapshot();
    let shown_dialog = shown.node(subject).unwrap().as_dialog().unwrap();
    let first_watcher = shown_dialog.close_watcher().unwrap();
    assert!(shown_dialog.is_modal());
    assert_eq!(
        shown
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members(),
        [subject]
    );
    assert_eq!(
        shown
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .ua_rules()
            .collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(subject)
        ]
    );
    completed(drive(
        &mut session,
        DomSourceEdit::close_dialog(subject, None),
        false,
        &mut calls,
    ));
    let closed = session.snapshot();
    let node = closed.node(subject).unwrap();
    let closed_dialog = node.as_dialog().unwrap();
    assert_eq!(node.attribute(&"open".into()), None);
    assert!(!closed_dialog.is_modal());
    assert!(!closed_dialog.has_close_watcher());
    let layer = closed.document(child).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    assert_eq!(layer.pending_removals(), [subject]);
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [DomTopLayerUaRule::UniversalOverlayNone]
    );
    assert_eq!(
        document_inert(&closed, child),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    let tasks = session.element_task_execution();
    let close_task = tasks
        .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
        .unwrap()
        .unwrap();
    assert_eq!(close_task.dialog_close().unwrap().id().target(), subject);
    completed(drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    ));
    let reshown = session.snapshot();
    let dialog = reshown.node(subject).unwrap().as_dialog().unwrap();
    assert!(dialog.is_modal());
    assert_ne!(dialog.close_watcher().unwrap(), first_watcher);
    let layer = reshown.document(child).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    assert_eq!(layer.pending_removals(), [subject]);
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [DomTopLayerUaRule::UniversalOverlayNone]
    );
    assert_eq!(
        document_inert(&reshown, child),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(calls, [true, false, true]);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 2);
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::UserInteraction, 0)
            .unwrap(),
        Some(close_task)
    );
    assert_eq!(
        shown
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .pending_removals(),
        []
    );
    assert_eq!(
        closed
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .pending_removals(),
        [subject]
    );
}

#[test]
fn enabled_autofocus_preserves_actual_focusing_wait_and_completes_same_origin_tail() {
    use surgeist_html::{HtmlDialogFocusControl, HtmlDialogFocusingAction};
    for (sandbox, inert, expected) in [
        (
            Some(""),
            false,
            Some(HtmlDialogFocusingAction::RunFocusingSteps {
                control: HtmlDialogFocusControl::Subject,
            }),
        ),
        (None, true, None),
    ] {
        let (mut session, top) = headless_top();
        let (_, child) = container(
            &mut session,
            top.document(),
            "iframe",
            Some("focus-without-user-activation *"),
            sandbox,
            false,
        );
        let subject = dialog(&mut session, child);
        for name in ["autofocus"].into_iter().chain(inert.then_some("inert")) {
            let progress = session
                .source_execution()
                .apply(
                    &mut session.dom(),
                    DomSourceEdit::set_attribute(subject, name, ""),
                    source_limits(),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel(),
                )
                .unwrap();
            assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
        }
        let before = session.snapshot();
        assert_eq!(
            policy(&before, child).inherited_focus_without_user_activation(),
            DomPermissionsPolicyValue::Enabled
        );
        assert_eq!(
            before.document(child).unwrap().metadata().origin()
                == before.document(top.document()).unwrap().metadata().origin(),
            sandbox.is_none()
        );
        let mut calls = Vec::new();
        let progress = drive(
            &mut session,
            DomSourceEdit::show_modal_dialog(subject),
            true,
            &mut calls,
        );
        let operation = match progress {
            DomSourceDriveProgress::Pending {
                request: DomSourceRequest::DialogFocusing(request),
                reason: DomSourceUnavailable::DialogFocusing(action),
            } if !inert => {
                assert_eq!(Some(action), expected);
                assert_eq!(Some(request.action()), expected);
                assert_eq!(request.subject(), subject);
                request.operation()
            }
            DomSourceDriveProgress::Complete(done) if inert => {
                assert!(!done.cancellation_requested());
                done.operation()
            }
            other => panic!(
                "actual focusing remains pending without inert proof; the same-origin no-change path completes its autofocus writes: {other:?}"
            ),
        };
        assert_eq!(calls, [true]);
        let current = session.snapshot();
        if inert {
            assert_eq!(current.incomplete_source_operation(), None);
            assert!(session.dom().pending_source_edit().is_none());
            assert_eq!(
                current
                    .document(child)
                    .unwrap()
                    .top_layer()
                    .unwrap()
                    .raw_members(),
                [subject]
            );
        } else {
            assert_eq!(current.incomplete_source_operation(), Some(operation));
            assert!(matches!(
                current.document(child).unwrap().top_layer(),
                Err(DomError::IncompleteSourceOperation)
            ));
        }
        let dialog = current.node(subject).unwrap().as_dialog().unwrap();
        assert!(dialog.is_modal());
        assert!(dialog.has_close_watcher());
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            1
        );
        assert!(
            !before
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .is_modal()
        );
        assert!(
            before
                .document(child)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members()
                .is_empty()
        );
    }
}

#[test]
fn beforetoggle_removing_actual_own_inert_preserves_called_focusing_wait_and_exact_retry() {
    use surgeist_html::{HtmlDialogFocusControl, HtmlDialogFocusingAction};
    let (mut session, top) = headless_top();
    let (_, child) = container(
        &mut session,
        top.document(),
        "iframe",
        Some("focus-without-user-activation *"),
        Some(""),
        false,
    );
    let subject = dialog(&mut session, child);
    for name in ["inert", "autofocus"] {
        let result = session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(subject, name, ""),
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    }
    let before = session.snapshot();
    let event = match session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_modal_dialog(subject),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(event),
            ..
        } => event,
        other => panic!("actual beforetoggle required: {other:?}"),
    };
    struct Context {
        source: DomSourceRuntime,
        calls: usize,
        removed: Option<Result<DomSourceDriveProgress, DomSourceRuntimeError>>,
    }
    let mut context = Context {
        source: session.source_execution(),
        calls: 0,
        removed: None,
    };
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                assert!(event.cancelable());
                context.calls += 1;
                context.removed = Some(context.source.apply(
                    dom,
                    DomSourceEdit::remove_attribute(subject, "inert"),
                    source_limits(),
                    &mut DomSourceServices {
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel(),
                ));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    match session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &event,
            &mut registry,
            &mut context,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: dispatch_limits(),
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceEventServiceProgress::Serviced { receipt } => {
            assert_eq!(receipt.callbacks(), 1);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
        }
        other => panic!("actual completed beforetoggle required: {other:?}"),
    }
    assert!(matches!(
        context.removed.take().unwrap().unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    assert_eq!(context.calls, 1);
    let child_request = match session
        .source_execution()
        .resume(
            &mut session.dom(),
            event.operation(),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::DialogFocusing(request),
            reason: DomSourceUnavailable::DialogFocusing(action),
        } => {
            assert_eq!(
                action,
                HtmlDialogFocusingAction::RunFocusingSteps {
                    control: HtmlDialogFocusControl::Subject
                }
            );
            assert_eq!(request.action(), action);
            request
        }
        other => {
            panic!("lost actual inert proof must preserve the called focusing wait: {other:?}")
        }
    };
    let current = session.snapshot();
    assert_eq!(
        current.node(subject).unwrap().attribute(&"inert".into()),
        None
    );
    assert_eq!(
        before.node(subject).unwrap().attribute(&"inert".into()),
        Some(&"".into())
    );
    assert_eq!(
        current.incomplete_source_operation(),
        Some(event.operation())
    );
    assert_eq!(
        policy(&current, child).inherited_focus_without_user_activation(),
        DomPermissionsPolicyValue::Enabled
    );
    let dialog = current.node(subject).unwrap().as_dialog().unwrap();
    assert!(dialog.is_modal());
    let watcher = dialog.close_watcher().unwrap();
    let task = session
        .element_task_execution()
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap();
    let retry = session
        .source_execution()
        .resume(
            &mut session.dom(),
            event.operation(),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(retry,DomSourceDriveProgress::Pending {request:DomSourceRequest::DialogFocusing(request),reason:DomSourceUnavailable::DialogFocusing(action)} if request==child_request && action==child_request.action())
    );
    assert_eq!(session.snapshot().stamp(), current.stamp());
    assert_eq!(
        session
            .snapshot()
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .close_watcher(),
        Some(watcher)
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        task
    );
    assert_eq!(context.calls, 1);
}

#[test]
fn enabled_cross_origin_inert_autofocus_dialog_completes_show_modal() {
    use DomPermissionsPolicyValue::{Disabled, Enabled};
    let (mut session, top) = headless_top();
    let (_, child) = container(
        &mut session,
        top.document(),
        "iframe",
        Some("focus-without-user-activation *"),
        Some(""),
        false,
    );
    let subject = dialog(&mut session, child);
    let before_attributes = session.snapshot();
    let expected_policy = DomInitialPermissionsPolicy::new(Disabled, Enabled);
    assert_eq!(policy(&before_attributes, child), expected_policy);
    assert_ne!(
        before_attributes
            .document(child)
            .unwrap()
            .metadata()
            .origin(),
        before_attributes
            .document(top.document())
            .unwrap()
            .metadata()
            .origin()
    );
    // These are actual attribute changes after connected insertion. No
    // insertion-time autofocus processing is part of this fixture.
    for name in ["inert", "autofocus"] {
        let progress = session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(subject, name, ""),
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    }
    let ready = session.snapshot();
    assert_eq!(
        ready.node(subject).unwrap().attribute(&"inert".into()),
        Some(&"".into())
    );
    assert_eq!(
        ready.node(subject).unwrap().attribute(&"autofocus".into()),
        Some(&"".into())
    );
    assert_eq!(ready.node(subject).unwrap().attribute(&"open".into()), None);
    assert!(!ready.node(subject).unwrap().as_dialog().unwrap().is_modal());
    assert!(
        ready
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    let mut calls = Vec::new();
    // HTML877d raw46762–46782: enabled permission selects the autofocus
    // subject. Its own inert attribute still applies while modal (59128–31).
    // Called focusing cannot update focus: an inert subject's scroll regions
    // are not focusable areas (59702–05), and get-focusable-area has no other
    // matching branch for this ordinary Dialog (60085–60109). Even if the
    // subject itself were an area, its inert DOM anchor returns (60182).
    // The real child/top origin mismatch then returns before autofocus writes.
    completed(drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    ));
    let shown = session.snapshot();
    assert_eq!(calls, [true]);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(shown.incomplete_source_operation(), None);
    assert_eq!(policy(&shown, child), expected_policy);
    let node = shown.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    let watcher = dialog.close_watcher().expect("actual standard watcher");
    assert_eq!(watcher.subject(), subject);
    assert_eq!(
        watcher.global(),
        shown.relevant_global(subject).unwrap().unwrap()
    );
    let layer = shown.document(child).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    assert!(layer.pending_removals().is_empty());
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(subject)
        ]
    );
    assert_eq!(
        document_inert(&shown, child),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        1
    );
    assert_eq!(
        before_attributes
            .node(subject)
            .unwrap()
            .attribute(&"inert".into()),
        None
    );
    assert_eq!(
        before_attributes
            .node(subject)
            .unwrap()
            .attribute(&"autofocus".into()),
        None
    );
    assert_eq!(policy(&before_attributes, child), expected_policy);
    assert!(
        ready
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    assert!(!ready.node(subject).unwrap().as_dialog().unwrap().is_modal());
}

#[test]
fn enabled_same_origin_inert_autofocus_dialog_completes_its_autofocus_tail() {
    use DomPermissionsPolicyValue::Enabled;
    let (mut session, top) = headless_top();
    let (_, child) = container(&mut session, top.document(), "iframe", None, None, false);
    let subject = dialog(&mut session, child);
    let inserted = session.snapshot();
    let expected_policy = DomInitialPermissionsPolicy::new(Enabled, Enabled);
    assert_eq!(policy(&inserted, child), expected_policy);
    assert_eq!(
        inserted.document(child).unwrap().metadata().origin(),
        inserted
            .document(top.document())
            .unwrap()
            .metadata()
            .origin()
    );
    for name in ["inert", "autofocus"] {
        let result = session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(subject, name, ""),
                source_limits(),
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    }
    let ready = session.snapshot();
    assert_eq!(
        ready.node(subject).unwrap().attribute(&"inert".into()),
        Some(&"".into())
    );
    assert_eq!(
        ready.node(subject).unwrap().attribute(&"autofocus".into()),
        Some(&"".into())
    );
    assert_eq!(ready.node(subject).unwrap().attribute(&"open".into()), None);
    assert!(
        ready
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    let mut calls = Vec::new();
    // Actual enabled permission selects this autofocus subject; its own inert
    // attribute makes the called focusing invocation a no-change path. The
    // selected control and actual top Document share one origin, so HTML877d
    // raw46777–46779 continues through EmptyAutofocusCandidates and
    // SetAutofocusProcessed, then the original ShowModal must return.
    let done = match drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    ) {
        DomSourceDriveProgress::Complete(done) => done,
        other => panic!(
            "same-origin no-change focusing must finish its actual autofocus tail: {other:?}"
        ),
    };
    assert!(!done.cancellation_requested());
    let shown = session.snapshot();
    assert_eq!(calls, [true]);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(shown.incomplete_source_operation(), None);
    assert_eq!(done.commit().changes().to(), shown.stamp());
    assert!(
        done.commit()
            .changes()
            .changes()
            .contains(&DomChange::SourceOperation)
    );
    assert_eq!(policy(&shown, child), expected_policy);
    assert_eq!(
        shown
            .hosted_navigable(top.navigable())
            .unwrap()
            .active_document(),
        Some(top.document())
    );
    let node = shown.node(subject).unwrap();
    assert_eq!(node.owner_document(), Some(child));
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    let watcher = dialog.close_watcher().expect("actual standard watcher");
    let global = shown.relevant_global(subject).unwrap().unwrap();
    assert_eq!(watcher.global(), global);
    assert_eq!(watcher.subject(), subject);
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    let layer = shown.document(child).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    assert!(layer.pending_removals().is_empty());
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(subject)
        ]
    );
    assert_eq!(
        document_inert(&shown, child),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap()
        .dialog_toggle()
        .unwrap();
    assert_eq!(task.id().target(), subject);
    assert_eq!(task.source(), None);
    assert_eq!(task.queue_association().unwrap().document(), child);
    assert_eq!(task.queue_association().unwrap().global(), global);
    assert_eq!(dialog.toggle_task_tracker().unwrap().task(), task.id());
    assert_eq!(
        inserted.node(subject).unwrap().attribute(&"inert".into()),
        None
    );
    assert_eq!(
        inserted
            .node(subject)
            .unwrap()
            .attribute(&"autofocus".into()),
        None
    );
    assert_eq!(policy(&inserted, child), expected_policy);
    assert_eq!(ready.node(subject).unwrap().attribute(&"open".into()), None);
    assert!(!ready.node(subject).unwrap().as_dialog().unwrap().is_modal());
    assert!(
        ready
            .document(child)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
}

#[path = "native_focus_permissions_policy/autofocus.rs"]
mod autofocus;

#[path = "native_focus_permissions_policy/delegates.rs"]
mod delegates;

#[test]
fn enabled_same_origin_inert_dialog_without_autofocus_completes_after_empty_delegate_traversals() {
    use DomPermissionsPolicyValue::Enabled;
    let (mut session, top) = headless_top();
    let document = top.document();
    let subject = dialog(&mut session, document);
    let inserted = session.snapshot();
    let expected_policy = DomInitialPermissionsPolicy::new(Enabled, Enabled);
    assert_eq!(policy(&inserted, document), expected_policy);
    assert!(
        inserted
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        inserted.relevant_global(subject).unwrap(),
        Some(top.global())
    );
    let node = inserted.node(subject).unwrap();
    assert!(node.children().is_empty());
    assert_eq!(node.hosted_shadow_root(), None);
    assert_eq!(node.attribute(&"autofocus".into()), None);
    assert_eq!(node.attribute(&"inert".into()), None);
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    // Set only inert after insertion. No autofocus insertion enrollment, focus
    // movement, or external focusability observation supplies the null delegate.
    let changed = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::set_attribute(subject, "inert", ""),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(changed, DomSourceDriveProgress::Complete(_)));
    let ready = session.snapshot();
    let node = ready.node(subject).unwrap();
    assert!(node.children().is_empty());
    assert_eq!(node.hosted_shadow_root(), None);
    assert_eq!(node.attribute(&"inert".into()), Some(&"".into()));
    assert_eq!(node.attribute(&"autofocus".into()), None);
    assert_eq!(node.attribute(&"open".into()), None);
    assert!(!node.as_dialog().unwrap().is_modal());
    let initial_autofocus = ready.document(document).unwrap().autofocus().unwrap();
    assert!(initial_autofocus.candidates().is_empty());
    assert!(!initial_autofocus.processed());
    assert!(
        ready
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
    let mut calls = Vec::new();
    // HTML877d60113–60159 runs the autofocus-delegate traversal, then the
    // ordinary focus-delegate traversal. Both visit this actual empty ordinary
    // descendant domain and return null. HTML877d46762–46779 then selects the
    // subject; its own inert makes called focusing a no-change path, and its
    // actual same-origin top Document permits the existing autofocus tail.
    let progress = drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    );
    let prefix = session.snapshot();
    let reached = prefix.node(subject).unwrap();
    assert_eq!(reached.attribute(&"open".into()), Some(&"".into()));
    let reached_dialog = reached.as_dialog().unwrap();
    assert!(reached_dialog.is_modal());
    assert_eq!(
        reached_dialog.close_watcher().unwrap().global(),
        top.global()
    );
    assert_eq!(reached_dialog.close_watcher().unwrap().subject(), subject);
    assert_eq!(
        reached
            .as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    let done = match progress {
        DomSourceDriveProgress::Complete(done) => done,
        other => {
            panic!("empty delegate traversals must let the real inert Dialog complete: {other:?}")
        }
    };
    assert!(!done.cancellation_requested());
    let shown = session.snapshot();
    assert_eq!(calls, [true]);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(shown.incomplete_source_operation(), None);
    assert_eq!(done.commit().changes().to(), shown.stamp());
    assert_eq!(policy(&shown, document), expected_policy);
    assert_eq!(
        shown
            .hosted_navigable(top.navigable())
            .unwrap()
            .active_document(),
        Some(document)
    );
    let node = shown.node(subject).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    assert!(node.children().is_empty());
    assert_eq!(node.hosted_shadow_root(), None);
    assert_eq!(node.attribute(&"autofocus".into()), None);
    assert_eq!(node.attribute(&"inert".into()), Some(&"".into()));
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    let watcher = dialog.close_watcher().expect("actual standard watcher");
    assert_eq!(watcher.global(), top.global());
    assert_eq!(watcher.subject(), subject);
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    let layer = shown.document(document).unwrap().top_layer().unwrap();
    assert_eq!(layer.raw_members(), [subject]);
    assert!(layer.pending_removals().is_empty());
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(subject),
        ]
    );
    let autofocus = shown.document(document).unwrap().autofocus().unwrap();
    assert!(autofocus.candidates().is_empty());
    assert!(autofocus.processed());
    assert_eq!(
        document_inert(&shown, document),
        HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap()
        .dialog_toggle()
        .unwrap();
    assert_eq!(task.id().target(), subject);
    assert_eq!(task.old_state(), surgeist_html::HtmlDialogState::Closed);
    assert_eq!(task.new_state(), surgeist_html::HtmlDialogState::Open);
    assert_eq!(task.source(), None);
    assert_eq!(task.queue_association().unwrap().document(), document);
    assert_eq!(task.queue_association().unwrap().global(), top.global());
    assert_eq!(dialog.toggle_task_tracker().unwrap().task(), task.id());
    // Historical complete snapshots retain the pre-method tree and history.
    assert_eq!(
        inserted.node(subject).unwrap().attribute(&"inert".into()),
        None
    );
    assert_eq!(ready.node(subject).unwrap().attribute(&"open".into()), None);
    assert_eq!(
        ready.node(subject).unwrap().attribute(&"autofocus".into()),
        None
    );
    assert!(!ready.node(subject).unwrap().as_dialog().unwrap().is_modal());
    assert!(
        ready
            .document(document)
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    let historical_autofocus = ready.document(document).unwrap().autofocus().unwrap();
    assert!(historical_autofocus.candidates().is_empty());
    assert!(!historical_autofocus.processed());
    assert_eq!(policy(&ready, document), expected_policy);
}
