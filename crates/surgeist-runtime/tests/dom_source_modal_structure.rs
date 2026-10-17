//! Standard ShowModal publishes its structural modal prefix before focusing.
//! Pinned HTML877d46602–46615 orders watcher assertion, modal=true, Document
//! blocking, conditional top-layer addition, remembered focus/popovers, focusing.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlDialogFocusingAction, HtmlDialogState};
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
fn headless_dialog() -> (DomSession, DomHostedTopLevel, NodeId) {
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
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let body = snapshot.node(html).unwrap().children()[1];
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(top.document(), Namespace::html(), "dialog");
    batch.insert_before(&dialog, body, None);
    let dialog = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&dialog)
        .unwrap()
        .unwrap();
    (session, top, dialog)
}

#[test]
fn active_show_modal_publishes_modal_after_real_watcher_before_focusing_wait() {
    let (mut session, top, subject) = headless_dialog();
    assert!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        session.snapshot().node(subject).unwrap().owner_document(),
        Some(top.document())
    );
    assert_eq!(
        session.snapshot().relevant_global(subject).unwrap(),
        Some(top.global())
    );
    let scope = session
        .source_execution_scope(top.host(), &cancel())
        .unwrap();
    let request = match session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_modal_dialog(subject),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: Some(&scope),
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(request),
            reason: DomSourceUnavailable::Event,
        } => request,
        other => panic!("active connected ShowModal must fire actual beforetoggle: {other:?}"),
    };
    assert_eq!(request.subject(), subject);
    assert!(request.cancelable());
    let payload = request.toggle_event().unwrap();
    assert_eq!(payload.old_state(), HtmlDialogState::Closed);
    assert_eq!(payload.new_state(), HtmlDialogState::Open);
    assert_eq!(payload.source(), None);
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
            DomEventCallback::new(move |_, _, calls: &mut usize, event| {
                *calls += 1;
                assert_eq!(event.target(), Some(subject));
                assert!(event.cancelable());
                assert!(!event.default_prevented());
                assert_eq!(
                    event.composed_path().last(),
                    Some(DomEventTarget::Global(top.global()))
                );
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let mut calls = 0;
    match session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &request,
            &mut registry,
            &mut calls,
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
        other => panic!("the real cancelable dispatch must finish: {other:?}"),
    }
    let progress = session
        .source_execution()
        .resume(
            &mut session.dom(),
            request.operation(),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: Some(&scope),
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let snapshot = session.snapshot();
    let node = snapshot.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    let dialog = node.as_dialog().unwrap();
    let watcher = dialog
        .close_watcher()
        .expect("actual AddOpen setup must establish its original watcher");
    assert_eq!(watcher.subject(), subject);
    assert_eq!(watcher.global(), top.global());
    assert_eq!(watcher.window_document(), top.document());
    assert_eq!(calls, 1);
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(task.dialog_toggle().unwrap().id().target(), subject);
    assert_eq!(task.queue_association().unwrap().document(), top.document());
    // The source performs literal modal=true after its satisfied watcher
    // assertion, before the shared remembered-focus/popover/focusing tail.
    assert!(
        dialog.is_modal(),
        "ShowModal must publish modal=true after its real watcher assertion and before focusing"
    );
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::DialogFocusing(pending),
            reason:
                DomSourceUnavailable::DialogFocusing(HtmlDialogFocusingAction::FindFocusDelegate),
        } => {
            assert_eq!(pending.operation(), request.operation());
            assert_eq!(pending.subject(), subject);
            assert_eq!(
                pending.action(),
                HtmlDialogFocusingAction::FindFocusDelegate
            );
        }
        other => panic!(
            "structural modal effects must reach the genuine remaining focusing wait: {other:?}"
        ),
    }
    assert_eq!(
        snapshot.incomplete_source_operation(),
        Some(request.operation())
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        request.operation()
    );
}
