//! Actual Source suspension and observer reentry share the canonical SVG gate.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_cssom::CssomStore;
use surgeist_dom::*;
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4)
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
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 8,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
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

fn dispatcher<C>() -> DomEventDispatcher<C> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
    })
    .unwrap()
}

struct Fixture {
    session: DomSession,
    document: DocumentId,
    parent: NodeId,
    dialog: NodeId,
    script: NodeId,
    style: NodeId,
}

fn fixture() -> Fixture {
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            100_000,
            256,
            128,
            128,
            32,
            4_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        )
        .with_event_recipients(
            DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000).unwrap(),
        ),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let parent = batch.create_element_ns(&document, Namespace::html(), "div");
    let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
    let script = batch.create_element_ns(&document, Namespace::svg(), "script");
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let text = batch.create_text(&document, "p { color: red }");
    batch.insert_before(&parent, &document, None);
    batch.insert_before(&dialog, &parent, None);
    batch.insert_before(&text, &style, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    Fixture {
        session: DomSession::new(dom, session_limits()).unwrap(),
        document: commit.document(&document).unwrap(),
        parent: commit.created(&parent).unwrap().unwrap(),
        dialog: commit.created(&dialog).unwrap().unwrap(),
        script: commit.created(&script).unwrap().unwrap(),
        style: commit.created(&style).unwrap().unwrap(),
    }
}

fn assert_detached_script(snapshot: &DomSnapshot, script: NodeId, document: DocumentId) {
    let node = snapshot.node(script).unwrap();
    assert_eq!(node.owner_document(), Some(document));
    assert_eq!(node.parent(), None);
    assert!(!node.is_connected());
    assert!(!node.as_svg_script().unwrap().flags().already_processed());
    assert_eq!(
        node.as_svg_script().unwrap().origin(),
        DomSvgScriptOrigin::Native
    );
}

#[test]
fn parked_style_source_refuses_new_svg_recipients_without_resetting_the_original_operation() {
    let Fixture {
        mut session,
        document,
        parent,
        dialog,
        script,
        style,
    } = fixture();
    let mut styles = session.style_execution();
    let mut root = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    styles
        .bind_root(
            &session.dom(),
            "actual-style-root".into(),
            &mut root,
            &cancel(),
        )
        .unwrap();
    assert!(root.is_none());
    let old_dom = session.snapshot();
    let old_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    let mut source = session.source_execution();
    let mut no_policy = |_: &DomStyleSource| None;
    let pending = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut no_policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Style(request),
        reason: DomSourceUnavailable::Style(DomStyleUnavailable::InlinePolicy),
    } = pending
    else {
        panic!("the real Style CSP request must remain unacknowledged")
    };
    assert_eq!(request.source().element(), style);
    assert_eq!(request.source().document(), document);
    assert_eq!(
        request.source().text().as_slice(),
        "p { color: red }".encode_utf16().collect::<Vec<_>>()
    );
    let parked = session.dom().pending_source_edit().unwrap();
    assert_eq!(parked.operation(), request.source().operation());
    assert!(parked.spent_work() > 0);
    let parked_snapshot = session.snapshot();
    let usage = session.dom().usage();
    let mut recipients = dispatcher::<()>();
    for event_type in [DomEventType::Load, DomEventType::Error] {
        assert!(matches!(
            recipients.add_listener(
                &mut session.dom(),
                script,
                event_type,
                DomEventCallback::new(|_, _, _: &mut (), _| Ok(())),
                DomEventListenerOptions::default()
            ),
            Err(DomEventDispatchError::Dom(DomError::SourceOperationPending))
        ));
        assert_eq!(session.dom().pending_source_edit(), Some(parked));
        assert_eq!(session.snapshot().stamp(), parked_snapshot.stamp());
        assert_eq!(session.dom().usage(), usage);
        assert_detached_script(&session.snapshot(), script, document);
    }
    // The driver resumes its original resource owner and authentic CSP inputs.
    let mut policy_calls = 0;
    let mut policy = |actual: &DomStyleSource| {
        policy_calls += 1;
        assert_eq!(actual.operation(), request.source().operation());
        assert_eq!(actual.ordinal(), request.source().ordinal());
        assert_eq!(actual.stamp(), request.source().stamp());
        assert!(Arc::ptr_eq(actual.text(), request.source().text()));
        Some(DomStyleInlineCspObservation::complete_empty(actual, 1))
    };
    let result = source
        .resume(
            &mut session.dom(),
            request.source().operation(),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = result else {
        panic!("original Style insertion completes")
    };
    assert_eq!(policy_calls, 1);
    assert_eq!(done.operation(), parked.operation());
    assert!(!done.cancellation_requested());
    assert_eq!(done.commit().changes().to(), session.snapshot().stamp());
    assert!(session.dom().pending_source_edit().is_none());
    let current = session.snapshot();
    assert_eq!(current.node(parent).unwrap().children(), &[dialog, style]);
    let sheet = current
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap();
    let cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(cssom.sheets(), std::slice::from_ref(sheet.id()));
    assert_eq!(sheet.revision(), cssom.revision());
    assert_eq!(sheet.association().element(), style);
    assert!(cssom.sheet(sheet.id()).unwrap().inputs().owner.is_some());
    assert!(old_cssom.sheets().is_empty());
    assert!(
        old_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert_detached_script(&old_dom, script, document);
    assert_detached_script(&current, script, document);
    // Neither refused attempt installed a hidden listener; registration is now
    // possible on the same dispatcher after the original operation completes.
    for event_type in [DomEventType::Load, DomEventType::Error] {
        let id = recipients
            .add_listener(
                &mut session.dom(),
                script,
                event_type,
                DomEventCallback::new(|_, _, _: &mut (), _| Ok(())),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        assert!(recipients.remove_listener(id));
        assert!(!recipients.remove_listener(id));
    }
}

struct Reentry {
    recipients: DomEventDispatcher<()>,
    registrations: Vec<DomEventRegistrationId>,
    callbacks: usize,
    refused: usize,
}

#[test]
fn observed_beforetoggle_reentry_retains_svg_recipients_and_atomically_refuses_processing() {
    let Fixture {
        mut session,
        document,
        parent,
        dialog,
        script,
        ..
    } = fixture();
    let old = session.snapshot();
    let mut listeners = dispatcher();
    listeners
        .add_listener(
            &mut session.dom(),
            dialog,
            DomEventType::BeforeToggle,
            DomEventCallback::new(move |dom, _, context: &mut Reentry, event| {
                context.callbacks += 1;
                event.prevent_default();
                for event_type in [DomEventType::Load, DomEventType::Error] {
                    let id = context
                        .recipients
                        .add_listener(
                            dom,
                            script,
                            event_type,
                            DomEventCallback::new(|_, _, _: &mut (), _| Ok(())),
                            DomEventListenerOptions::default(),
                        )
                        .unwrap();
                    context.registrations.push(id);
                }
                let before = dom.snapshot();
                let usage = dom.usage();
                let outer = dom.pending_source_edit().unwrap();
                let mut batch = DomBatch::new();
                batch.insert_before(script, parent, None);
                assert_eq!(
                    dom.apply(batch, &cancel()).unwrap_err(),
                    DomError::SvgScriptProcessingUnavailable
                );
                context.refused += 1;
                assert_eq!(dom.snapshot().stamp(), before.stamp());
                assert_eq!(dom.usage(), usage);
                assert_eq!(
                    dom.pending_source_edit().unwrap().operation(),
                    outer.operation()
                );
                assert_detached_script(&dom.snapshot(), script, document);
                assert_eq!(dom.snapshot().node(parent).unwrap().children(), &[dialog]);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut source = session.source_execution();
    let mut no_policy = |_: &DomStyleSource| None;
    let progress = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_dialog(dialog),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut no_policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Pending {
        request: DomSourceRequest::Event(request),
        reason: DomSourceUnavailable::Event,
    } = progress
    else {
        panic!("actual beforetoggle observation is required")
    };
    assert_eq!(request.subject(), dialog);
    assert!(request.cancelable());
    let toggle = request.toggle_event().unwrap();
    assert_eq!(toggle.old_state(), surgeist_html::HtmlDialogState::Closed);
    assert_eq!(toggle.new_state(), surgeist_html::HtmlDialogState::Open);
    let mut context = Reentry {
        recipients: dispatcher(),
        registrations: Vec::new(),
        callbacks: 0,
        refused: 0,
    };
    let serviced = source
        .service_event(
            &mut session.dom(),
            &request,
            &mut listeners,
            &mut context,
            DomSourceEventServiceLimits {
                source: source_limits(),
                dispatch: dispatch_limits(),
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = serviced else {
        panic!("original real observer delivery must finish")
    };
    assert_eq!(receipt.original_target(), dialog);
    assert_eq!(receipt.event_type(), DomEventType::BeforeToggle);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
    assert!(receipt.default_prevented());
    assert_eq!(context.callbacks, 1);
    assert_eq!(context.refused, 1);
    assert_eq!(context.registrations.len(), 2);
    let result = source
        .resume(
            &mut session.dom(),
            request.operation(),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut no_policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(done) = result else {
        panic!("prevented original show completes once")
    };
    assert_eq!(done.operation(), request.operation());
    assert!(done.commit().changes().changes().is_empty());
    assert_eq!(session.snapshot().stamp(), old.stamp());
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .snapshot()
            .node(dialog)
            .unwrap()
            .attribute(&"open".into()),
        None
    );
    assert_detached_script(&old, script, document);
    assert_detached_script(&session.snapshot(), script, document);
    for id in context.registrations.drain(..) {
        assert!(context.recipients.remove_listener(id));
        assert!(!context.recipients.remove_listener(id));
    }
    assert!(
        source
            .service_event(
                &mut session.dom(),
                &request,
                &mut listeners,
                &mut context,
                DomSourceEventServiceLimits {
                    source: source_limits(),
                    dispatch: dispatch_limits()
                },
                &cancel(),
            )
            .is_err()
    );
    assert_eq!(context.callbacks, 1);
    assert_eq!(context.refused, 1);
}

struct OnceLoad {
    styles: DomStyleRuntime,
    calls: usize,
}

impl DomStyleEventContext for OnceLoad {
    fn styles(&mut self) -> &mut DomStyleRuntime {
        &mut self.styles
    }
}

#[test]
fn real_style_load_consumes_once_ancestor_recipient_before_svg_processing_retry() {
    let Fixture {
        mut session,
        document,
        parent,
        dialog,
        script,
        style,
    } = fixture();
    let mut context = OnceLoad {
        styles: session.style_execution(),
        calls: 0,
    };
    let mut root = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    context
        .styles
        .bind_root(
            &session.dom(),
            "once-load-root".into(),
            &mut root,
            &cancel(),
        )
        .unwrap();
    assert!(root.is_none());
    let mut listeners = dispatcher();
    let once = listeners
        .add_listener(
            &mut session.dom(),
            parent,
            DomEventType::Load,
            DomEventCallback::new(move |_, _, context: &mut OnceLoad, event| {
                assert_eq!(event.event_type(), DomEventType::Load);
                assert_eq!(event.target(), Some(style));
                context.calls += 1;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                once: true,
            },
        )
        .unwrap();
    let mut source = session.source_execution();
    let mut policy =
        |actual: &DomStyleSource| Some(DomStyleInlineCspObservation::complete_empty(actual, 1));
    let completed = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(completed, DomSourceDriveProgress::Complete(_)));
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(context.calls, 0);
    assert!(context.styles.pending_sheet(0).unwrap().is_some());
    let before = session.snapshot();
    assert_eq!(before.node(parent).unwrap().children(), &[dialog, style]);
    assert!(
        before
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_some()
    );
    assert_detached_script(&before, script, document);
    let usage = session.dom().usage();
    let mut batch = DomBatch::new();
    batch.insert_before(script, parent, None);
    assert_eq!(
        session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::SvgScriptProcessingUnavailable
    );
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(session.dom().usage(), usage);
    assert!(session.dom().pending_source_edit().is_none());
    assert_detached_script(&session.snapshot(), script, document);
    let progress = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut listeners,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap();
    let DomStyleNetworkingProgress::Complete { receipt, .. } = progress else {
        panic!("the queued real Style load must be delivered")
    };
    assert_eq!(receipt.original_target(), style);
    assert_eq!(receipt.event_type(), DomEventType::Load);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
    assert!(!receipt.default_prevented());
    assert_eq!(context.calls, 1);
    assert!(!listeners.remove_listener(once));
    assert!(context.styles.pending_sheet(0).unwrap().is_none());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    let mut batch = DomBatch::new();
    batch.insert_before(script, parent, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let current = session.snapshot();
    assert_eq!(commit.changes().from(), before.stamp());
    assert_eq!(commit.changes().to(), current.stamp());
    assert_eq!(
        current.stamp().revision().get(),
        before.stamp().revision().get() + 1
    );
    assert_eq!(
        current.node(parent).unwrap().children(),
        &[dialog, style, script]
    );
    assert_eq!(current.node(script).unwrap().parent(), Some(parent));
    assert!(current.node(script).unwrap().is_connected());
    assert!(
        current
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .flags()
            .already_processed()
    );
    assert_eq!(
        current
            .node(script)
            .unwrap()
            .as_svg_script()
            .unwrap()
            .origin(),
        DomSvgScriptOrigin::Native
    );
    assert_detached_script(&before, script, document);
    assert_eq!(before.node(parent).unwrap().children(), &[dialog, style]);
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut listeners,
            &mut context,
            dispatch_limits(),
            &cancel(),
        )
        .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(context.calls, 1);
}
