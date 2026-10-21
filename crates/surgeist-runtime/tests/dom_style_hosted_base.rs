//! Hosted constructor Base facts must reach the original native Style capture.
//! HTML877d Style12138–12219 and the native initial Document contracts supply
//! the actual about:blank/known-about-base inputs; CSSOM remains the real owner.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_cssom::CssomStore;
use surgeist_dom::*;
use surgeist_html_semantics::HtmlAboutBaseUrl;
use surgeist_runtime::*;

#[path = "dom_style_hosted_base/idle_destruction.rs"]
mod idle_destruction;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 2)
}
fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 4_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 8,
            reports: 8,
            storage_bytes: 16_000_000,
            work: 4_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 8,
            storage_bytes: 16_000_000,
            work: 4_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(16, 16, 16, 16, 16, 16, 16),
                16_000_000,
                16_000_000,
                4_000_000,
            ),
            tasks: 16,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 4_000_000,
        },
    }
}
fn new_session() -> DomSession {
    DomSession::new(
        Dom::new(
            DomLimits::new(
                256,
                200_000,
                1024,
                1024,
                1024,
                32,
                4_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_event_recipients(
                surgeist_dom::DomEventRecipientLimits::new(64, 16_000_000, 16_000_000, 4_000_000)
                    .unwrap(),
            )
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(16, 1024, 16_000_000, 16_000_000, 4_000_000).unwrap(),
            ),
        )
        .unwrap(),
        limits(),
    )
    .unwrap()
}
fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let html = snapshot.node(document.node()).unwrap().children()[0];
    snapshot.node(html).unwrap().children()[1]
}
fn finish_hosted(
    session: &mut DomSession,
    top: DomHostedTopLevel,
    expected: Option<DomHostedOperationId>,
) {
    let mut hosted = session.hosted_execution();
    let mut target = expected;
    let mut last = DomHostedTaskProgress::Idle;
    let mut completed = false;
    for _ in 0..256 {
        last = hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel())
            .unwrap();
        assert!(
            !matches!(last, DomHostedTaskProgress::Idle),
            "actual queued work disappeared before completion"
        );
        if target.is_none() {
            target = last.operation();
        }
        if matches!(last, DomHostedTaskProgress::Complete { operation } if Some(operation) == target)
        {
            completed = true;
            break;
        }
    }
    assert!(
        completed,
        "original native operation did not finish in 256 advances: {last:?}"
    );
    assert!(matches!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel())
            .unwrap(),
        DomHostedTaskProgress::Idle
    ));
}
struct Notifications {
    styles: DomStyleRuntime,
    delivered: Vec<NodeId>,
}
impl DomStyleEventContext for Notifications {
    fn styles(&mut self) -> &mut DomStyleRuntime {
        &mut self.styles
    }
}
fn deliver_load(session: &mut DomSession, style: NodeId) {
    let mut context = Notifications {
        styles: session.style_execution(),
        delivered: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let once = dispatcher
        .add_listener(
            &mut session.dom(),
            style,
            DomEventType::Load,
            DomEventCallback::new(move |_, _, context: &mut Notifications, event| {
                assert_eq!(event.target(), Some(style));
                assert_eq!(event.event_type(), DomEventType::Load);
                assert!(!event.cancelable());
                context.delivered.push(style);
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let policy = DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 8,
        failure_records: 8,
        callbacks: 8,
        work: 100_000,
    };
    let result = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        policy,
        &cancel(),
    )
    .unwrap();
    let DomStyleNetworkingProgress::Complete { receipt, .. } = result else {
        panic!("real parsed sheet must deliver its Load: {result:?}")
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty(), "{:?}", receipt.failures());
    assert!(!receipt.default_prevented());
    assert_eq!(context.delivered, [style]);
    assert!(!dispatcher.remove_listener(once));
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut context,
            policy,
            &cancel()
        )
        .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
}
fn hosted_style_uses_current_constructor_base(child: bool) {
    let mut session = new_session();
    let mut hosted = session.hosted_execution();
    let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap();
    let initial = session.snapshot();
    let parent_base = initial
        .document(top.document())
        .unwrap()
        .document_base_record()
        .unwrap()
        .clone();
    assert_eq!(
        parent_base.serialization(),
        &DocumentString::from("about:blank")
    );
    let document = if child {
        let mut batch = DomBatch::new();
        let closed = batch.create_html_document(top.document(), None);
        let iframe = batch.create_element_ns(&closed, Namespace::html(), "iframe");
        let iframe = session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&iframe)
            .unwrap()
            .unwrap();
        let result = session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(iframe, body(&initial, top.document()), None),
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_: &DomStyleSource| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap();
        assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
        let snapshot = session.snapshot();
        let navigable = snapshot
            .node(iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable()
            .unwrap()
            .unwrap();
        let document = snapshot
            .hosted_navigable(navigable)
            .unwrap()
            .active_document()
            .unwrap();
        finish_hosted(&mut session, top, None);
        document
    } else {
        top.document()
    };
    let ready = session.snapshot();
    let view = ready.document(document).unwrap();
    assert!(view.is_fully_active().unwrap());
    assert!(
        view.metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert_eq!(view.first_base().unwrap(), None);
    let facts = view
        .base_document_facts()
        .expect("actual native constructor payload");
    assert!(!facts.iframe_srcdoc());
    assert_eq!(view.metadata().encoding(), &DocumentString::from("UTF-8"));
    let selected = view.document_base_record().unwrap().clone();
    assert_eq!(
        selected.serialization(),
        &DocumentString::from("about:blank")
    );
    if child {
        let HtmlAboutBaseUrl::KnownRecord(about) = facts.about_base_url() else {
            panic!("child retains creator's actual selected about-base")
        };
        assert_eq!(about, &parent_base);
        assert_eq!(&selected, about);
        assert_ne!(
            facts.document_url().unwrap(),
            &selected,
            "child URL and inherited about-base have distinct issued identities"
        );
    } else {
        assert!(matches!(
            facts.about_base_url(),
            HtmlAboutBaseUrl::KnownNull
        ));
        assert_eq!(facts.document_url(), Some(&selected));
    }
    let global = ready
        .relevant_global(body(&ready, document))
        .unwrap()
        .unwrap();
    let settings = hosted.settings(&session.dom(), global, &cancel()).unwrap();
    assert_eq!(
        settings.document_base_url().unwrap(),
        selected.serialization()
    );
    assert_eq!(
        settings.document().document_base_record().unwrap(),
        &selected
    );
    if child {
        assert_ne!(settings.creation_url(), &selected);
    }
    let mut styles = session.style_execution();
    let mut input = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    styles
        .bind_root(
            &session.dom(),
            "hosted-style-base".into(),
            &mut input,
            &cancel(),
        )
        .unwrap();
    assert!(input.is_none());
    let cssom_before = styles.snapshot(document.node()).unwrap().unwrap();
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let style = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let detached = session.snapshot();
    let mut observed = Vec::new();
    let result = {
        let mut policy = |source: &DomStyleSource| {
            assert_eq!(source.element(), style);
            assert_eq!(source.document(), document);
            assert_eq!(source.root(), document.node());
            assert!(
                source
                    .document_metadata()
                    .document_environment()
                    .has_ever_had_navigable()
            );
            assert_eq!(source.base_url(), selected.serialization());
            assert_eq!(source.base_url(), &DocumentString::from("about:blank"));
            assert_eq!(
                source.text().as_slice(),
                "p { color: red }".encode_utf16().collect::<Vec<_>>()
            );
            observed.push((source.operation(), source.base_url().clone()));
            Some(DomStyleInlineCspObservation::complete_empty(source, 1))
        };
        session.source_execution().apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, body(&ready, document), None),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel(),
        )
    };
    let result = result.unwrap_or_else(|error| panic!("constructor-qualified hosted Style must reach CSP and complete (observed {} CSP requests): {error:?}", observed.len()));
    let DomSourceDriveProgress::Complete(done) = result else {
        panic!("actual hosted Style association must complete: {result:?}")
    };
    assert_eq!(
        observed,
        [(done.operation(), selected.serialization().clone())]
    );
    assert!(!done.cancellation_requested());
    assert!(session.dom().pending_source_edit().is_none());
    let published = session.snapshot();
    assert_eq!(published.incomplete_source_operation(), None);
    let association = published
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap()
        .clone();
    assert_eq!(association.root(), document.node());
    let cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(cssom.sheets(), std::slice::from_ref(association.id()));
    assert_eq!(
        cssom
            .sheet(association.id())
            .unwrap()
            .authored()
            .rules()
            .len(),
        1
    );
    assert!(
        cssom
            .sheet(association.id())
            .unwrap()
            .diagnostics()
            .is_empty()
    );
    assert!(
        cssom
            .sheet(association.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert_eq!(
        cssom.sheet(association.id()).unwrap().inputs().base_url,
        None,
        "ordinary sheet has no constructed-base override"
    );
    assert_eq!(
        published
            .document(document)
            .unwrap()
            .document_base_record()
            .unwrap(),
        &selected
    );
    assert_eq!(
        settings.document().document_base_record().unwrap(),
        &selected
    );
    assert!(
        detached
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(cssom_before.sheets().is_empty());
    deliver_load(&mut session, style);
    let removed = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::detach(style),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_: &DomStyleSource| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(removed, DomSourceDriveProgress::Complete(_)));
    assert!(
        session
            .snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    let current = styles.snapshot(document.node()).unwrap().unwrap();
    assert!(current.sheets().is_empty());
    assert!(
        current
            .sheet(association.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    assert!(
        cssom
            .sheet(association.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert_eq!(styles.pending_sheet(0).unwrap(), None);
    assert_eq!(styles.waiting_resources().unwrap(), 0);
    assert!(matches!(
        hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel())
            .unwrap(),
        DomHostedTaskProgress::Idle
    ));
    let settled = session.snapshot();
    assert!(
        settled
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        settled
            .document(document)
            .unwrap()
            .document_base_record()
            .unwrap(),
        &selected
    );
    // DOM's hosted cleanup guard rejects every still-bound CSSOM root, even
    // this empty store. There is no public Document-root unbind. Source, Load
    // and child-history work is settled, but Top/Host remain live resources;
    // do not start an unsupported retirement or claim graceful completion.
    let session = match session.try_shutdown() {
        DomShutdownOutcome::Pending { session, reason } => {
            assert_eq!(reason, DomShutdownReason::HostedLifecycle);
            session
        }
        other => panic!("live hosted resources and bound empty CSSOM root remain: {other:?}"),
    };
    // Explicit terminal disposal releases the composed owners without
    // reporting Global destruction or executing synthetic cleanup callbacks.
    drop(session);
    assert!(matches!(
        styles.snapshot(document.node()),
        Err(DomStyleRuntimeError::SessionUnavailable)
    ));
    let mut control = new_session();
    assert!(matches!(
        hosted.settings(&control.dom(), global, &cancel()),
        Err(DomHostedRuntimeError::OwnerDisposed)
    ));
    assert!(matches!(
        control.try_shutdown(),
        DomShutdownOutcome::Complete
    ));
}

#[test]
fn top_constructor_current_base_reaches_real_style_csp_and_cssom_association() {
    hosted_style_uses_current_constructor_base(false);
}

#[test]
fn child_known_about_base_reaches_real_style_csp_without_substituting_its_creation_url() {
    hosted_style_uses_current_constructor_base(true);
}
