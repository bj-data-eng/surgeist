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
use surgeist_html::{HtmlDialogAction, HtmlValueInterpreter};
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
        DomLimits::new(512, 2_000_000, 2048, 2048, 2048, 64, 8_000_000)
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
            &session.dom(),
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
            HtmlCaptureLimits::new(128, 4096, 256, 32768, 128, 1, 0),
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
    assert!(
        matches!(progress,DomSourceDriveProgress::Pending{request:DomSourceRequest::Dialog(request),reason:DomSourceUnavailable::Dialog(HtmlDialogAction::RunDialogFocusingSteps)} if request.subject()==subject && request.action()==HtmlDialogAction::RunDialogFocusingSteps)
    );
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
