//! A genuine initial Iframe focus-policy denial ends Dialog focusing before
//! target selection: HTML877d3029,24293–24300,60636–60641,46762–46782.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::{HtmlDialogState, HtmlValueInterpreter};
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

#[test]
fn initial_iframe_focus_policy_denial_completes_modal_before_focus_target_selection() {
    let (mut session, top) = headless_top();
    let parent = body(&session.snapshot(), top.document());
    // Native construction in an actual associated detached HTML Document,
    // followed by Source insertion, runs the original Iframe child constructor.
    let mut batch = DomBatch::new();
    let detached = batch.create_html_document(top.document(), None);
    let iframe = batch.create_element_ns(&detached, Namespace::html(), "iframe");
    batch.set_attribute(&iframe, "allow", "focus-without-user-activation 'none'");
    let iframe = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&iframe)
        .unwrap()
        .unwrap();
    let insertion = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(iframe, parent, None),
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
        matches!(insertion, DomSourceDriveProgress::Complete(_)),
        "real initial child insertion must complete: {insertion:?}"
    );
    let created = session.snapshot();
    assert_eq!(created.incomplete_source_operation(), None);
    assert_eq!(created.incomplete_hosted_operation(), None);
    assert_eq!(
        created.node(iframe).unwrap().attribute(&"allow".into()),
        Some(&"focus-without-user-activation 'none'".into())
    );
    let child = created
        .node(iframe)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let mut documents = created.nodes().filter_map(|node| {
        let document = created.node(node).unwrap().as_document()?;
        (document.navigable() == Some(child)).then_some(document.id())
    });
    let document = documents.next().expect("actual child Document association");
    assert_eq!(documents.next(), None);
    assert_ne!(document, top.document());
    assert!(
        created
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let child_body = body(&created, document);
    let mut batch = DomBatch::new();
    let subject = batch.create_element_ns(document, Namespace::html(), "dialog");
    batch.insert_before(&subject, child_body, None);
    let subject = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&subject)
        .unwrap()
        .unwrap();
    let ready = session.snapshot();
    let global = ready.relevant_global(subject).unwrap().unwrap();
    assert_ne!(global, top.global());
    assert_eq!(ready.global_document(global).unwrap(), document);
    assert!(
        session
            .hosted_execution()
            .settings(&session.dom(), global, &cancel())
            .is_ok()
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
        other => panic!("connected active child Dialog must fire real beforetoggle: {other:?}"),
    };
    assert!(request.cancelable());
    assert_eq!(request.subject(), subject);
    let toggle = request.toggle_event().unwrap();
    assert_eq!(toggle.old_state(), HtmlDialogState::Closed);
    assert_eq!(toggle.new_state(), HtmlDialogState::Open);
    assert_eq!(toggle.source(), None);
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
                    Some(DomEventTarget::Global(global))
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
        other => panic!(
            "actual beforetoggle must finish with its authentic uncanceled receipt: {other:?}"
        ),
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
    let dialog = node.as_dialog().unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    assert!(dialog.is_modal());
    let watcher = dialog
        .close_watcher()
        .expect("real open setup must establish child Window watcher");
    assert_eq!(watcher.subject(), subject);
    assert_eq!(watcher.global(), global);
    assert_eq!(watcher.window_document(), document);
    assert_eq!(calls, 1);
    let tasks = session.element_task_execution();
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let task = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(task.dialog_toggle().unwrap().id().target(), subject);
    assert_eq!(task.queue_association().unwrap().global(), global);
    assert_eq!(task.queue_association().unwrap().document(), document);
    // This initial Window has received no activation-triggering input. The raw
    // inherited feature denial and false transient activation make allow-focus
    // return false before selecting a focus target, so this real method ends.
    let done = match progress {
        DomSourceDriveProgress::Complete(done) => done,
        other => panic!(
            "denied focus policy and no activation must complete ShowModal before focus-target selection: {other:?}"
        ),
    };
    assert_eq!(done.operation(), request.operation());
    assert!(!done.cancellation_requested());
    drop(done);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(snapshot.incomplete_source_operation(), None);
    let doc = snapshot.document(document).unwrap();
    let layer = doc.top_layer().unwrap();
    assert_eq!(layer.stamp(), doc.stamp());
    assert_eq!(layer.raw_members(), [subject]);
    assert!(layer.pending_removals().is_empty());
    assert_eq!(
        layer.ua_rules().collect::<Vec<_>>(),
        [
            DomTopLayerUaRule::UniversalOverlayNone,
            DomTopLayerUaRule::ElementOverlayAuto(subject)
        ]
    );
    let capture = snapshot
        .html_capture(
            document,
            None,
            // The hosted child has one actual complete-empty Global registry.
            HtmlCaptureLimits::new(128, 4096, 256, 32768, 128, 1, 0, 16_777_216),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &cancel(),
        )
        .unwrap();
    assert_eq!(capture.basis(), doc.stamp());
    let report = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(document.node()).unwrap(),
                HtmlSemanticQuery::Inertness,
            )],
            HtmlValueInterpreter::new(16384),
            HtmlSemanticLimits::new(1_000_000, 16, 1024, 256, 1_000_000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    assert_eq!(
        report.results()[0].fact(),
        &HtmlSemanticFact::Inertness(HtmlSemanticValue::Known(true))
    );
    assert_eq!(calls, 1);
    assert_eq!(
        tasks
            .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(task)
    );
}
