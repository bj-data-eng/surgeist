//! Native initial-blank construction and queued lifecycle use the actual session.
//! HTML877d's BC/Document creator, Details notification and destruction clauses
//! supply the literal states and ordering below.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_dom::{
    Dom, DomBatch, DomCommit, DomError, DomSnapshot,
    capture::DomHtmlCaptureLimits,
    custom_element::DomRegistryBinding,
    document::{DomDocumentClass, DomDocumentCreationCause},
    hosted::{DomHostedError, DomHostedLimits, DomHostedRecordLimits},
    identity::{DocumentId, NodeId},
    limits::DomLimits,
    shadow::{DomShadowRootInit, DomTemplateDocumentRole},
    source::{DomSourceEdit, DomSourceLimits},
    style::{DomStyleCssomRoot, DomStyleInlineCspObservation, DomStyleSource},
};
use surgeist_html::construction::HtmlDocumentMode;
use surgeist_html::{
    construction::HtmlShadowRootMode,
    values::{DocumentString, Namespace},
};
use surgeist_runtime::{
    DomSession,
    element_task::{
        DomElementTaskEventContext, DomElementTaskProgress, DomElementTaskRunLimits,
        DomElementTaskRuntime, DomElementTaskRuntimeError, DomElementTaskRuntimeLimits,
    },
    event::{
        DomEventCallback, DomEventDispatchError, DomEventDispatchLimits, DomEventDispatcher,
        DomEventDom, DomEventListenerOptions, DomEventPhase, DomEventRegistryLimits,
        DomEventTarget, DomEventType, DomToggleEvent, NativeDomEvent,
    },
    hosted::{
        DomHostedDestructionQueued, DomHostedRuntimeError, DomHostedRuntimeLimits,
        DomHostedTaskProgress, DomHostedTopLevel, HostedRuntime,
    },
    session::{DomSessionLimits, DomShutdownOutcome, DomShutdownReason},
    source::{DomSourceDriveProgress, DomSourceServices},
    style::DomStyleRuntimeLimits,
};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomSessionLimits {
    DomSessionLimits {
        authored_events: surgeist_runtime::event::DomEventStorageLimits {
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        source_events: surgeist_runtime::source::DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 32,
            reports: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: 16_000_000,
            work: 1_000_000,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                16_000_000,
                16_000_000,
                1_000_000,
            ),
            tasks: 32,
            storage_bytes: 16_000_000,
            scratch_bytes: 16_000_000,
            work: 1_000_000,
        },
    }
}
fn session() -> DomSession {
    let dom = Dom::new(
        DomLimits::new(
            512,
            200_000,
            1024,
            1024,
            1024,
            64,
            4_000_000,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            surgeist_dom::event::DomDispatchCustodyLimits::new(
                64, 4096, 16_000_000, 16_000_000, 4_000_000,
            )
            .unwrap(),
        )
        .with_details_toggle_intentions(32)
        .unwrap(),
    )
    .unwrap();
    DomSession::new(dom, limits()).unwrap()
}
fn create(session: &mut DomSession) -> DomHostedTopLevel {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel()).unwrap();
    let top = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
    assert_eq!(top.host(), host);
    top
}
fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let root = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let children = snapshot.node(root).unwrap().children();
    assert_eq!(children.len(), 2);
    children[1]
}
fn event_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 128,
        snapshot_listeners: 128,
        failure_records: 32,
        callbacks: 128,
        work: 100_000,
    }
}
#[derive(Debug, Eq, PartialEq)]
struct Observation {
    label: &'static str,
    current: Option<DomEventTarget>,
    target: Option<NodeId>,
    phase: DomEventPhase,
    payload: Option<DomToggleEvent>,
    bubbles: bool,
}
struct Context {
    element_tasks: DomElementTaskRuntime,
    seen: Vec<Observation>,
}
impl DomElementTaskEventContext for Context {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.element_tasks
    }
}
fn registry() -> DomEventDispatcher<Context> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 32,
        pending_events: 8,
        nested_dispatches: 8,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap()
}
fn listen(
    registry: &mut DomEventDispatcher<Context>,
    dom: &mut DomEventDom<'_>,
    target: DomEventTarget,
    label: &'static str,
    capture: bool,
) {
    registry
        .add_listener(
            dom,
            target,
            DomEventType::TOGGLE,
            DomEventCallback::new(move |_, _, context: &mut Context, event| {
                context.seen.push(Observation {
                    label,
                    current: event.current_target(),
                    target: event.target().and_then(|target| target.node()),
                    phase: event.phase(),
                    payload: event.toggle_event().cloned(),
                    bubbles: event.bubbles(),
                });
                Ok(())
            }),
            DomEventListenerOptions {
                capture,
                once: false,
                passive: None,
            },
        )
        .unwrap();
}
fn run_details(
    session: &mut DomSession,
    registry: &mut DomEventDispatcher<Context>,
    context: &mut Context,
) {
    let mut dom = session.dom();
    match DomElementTaskRuntime::run_next(
        &mut dom,
        surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
        registry,
        context,
        crate::DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: surgeist_dom::source::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap()
    {
        DomElementTaskProgress::Complete { receipt, .. } => assert!(receipt.failures().is_empty()),
        other => panic!("actual Toggle task must dispatch: {other:?}"),
    }
}

#[test]
fn initial_blank_has_real_html_tree_registry_and_current_global_settings() {
    let mut session = session();
    let top = create(&mut session);
    let snapshot = session.snapshot();
    let document = snapshot.document(top.document()).unwrap();
    let metadata = document.metadata();
    assert_eq!(metadata.class(), DomDocumentClass::Html);
    assert!(matches!(metadata.creation_cause(),
        DomDocumentCreationCause::HostedInitialAboutBlank { source, .. }
            if *source == top.host().source()));
    assert_eq!(metadata.content_type(), &DocumentString::from("text/html"));
    assert_eq!(metadata.encoding(), &DocumentString::from("UTF-8"));
    assert_eq!(metadata.url(), &DocumentString::from("about:blank"));
    assert_eq!(metadata.mode(), HtmlDocumentMode::Quirks);
    assert!(metadata.allow_declarative_shadow_roots());
    assert!(document.is_fully_active().unwrap());
    assert_eq!(
        snapshot.global_document(top.global()).unwrap(),
        top.document()
    );
    let children = snapshot.node(top.document().node()).unwrap().children();
    assert_eq!(children.len(), 1);
    let html = snapshot.node(children[0]).unwrap();
    assert_eq!(
        html.as_element().unwrap().name().local_name(),
        &"html".into()
    );
    assert_eq!(
        html.as_element().unwrap().name().namespace_uri(),
        Namespace::html().uri()
    );
    assert_eq!(html.children().len(), 2);
    for (node, local) in html.children().iter().zip(["head", "body"]) {
        let node = snapshot.node(*node).unwrap();
        assert_eq!(
            node.as_element().unwrap().name().local_name(),
            &local.into()
        );
        assert_eq!(
            node.as_element().unwrap().name().namespace_uri(),
            Namespace::html().uri()
        );
        assert!(node.children().is_empty());
    }
    let DomRegistryBinding::Registry(registry) = metadata.custom_element_registry() else {
        panic!("initial Global must supply its actual nonnull registry")
    };
    let registry = snapshot.registry(registry).unwrap();
    assert_eq!(registry.definition_count(), 0);
    assert_eq!(
        registry.scope(),
        surgeist_html_semantics::HtmlRegistryScope::Global
    );
    assert!(
        registry
            .lookup_definition(&Namespace::html(), &"x-native".into(), None)
            .unwrap()
            .is_none()
    );
    assert!(!registry.is_scoped());
    assert!(!registry.definition_is_running());
    assert_eq!(registry.scoped_document_count(), 0);
    assert_eq!(registry.when_defined_count(), 0);
    let canonical_registry = registry.id();
    let captured = snapshot
        .html_capture(
            top.document(),
            None,
            surgeist_html_semantics::HtmlCaptureLimits::new(
                128, 512, 512, 100_000, 128, 32, 32, 16_777_216,
            ),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &cancel(),
        )
        .unwrap();
    let producer = captured.capture();
    assert_eq!(producer.usage().registries(), 1);
    assert_eq!(producer.usage().definitions(), 0);
    let document_subject = captured.subject(top.document().node()).unwrap();
    assert_eq!(
        captured.node(&document_subject).unwrap(),
        top.document().node()
    );
    let captured_document = producer.node(&document_subject).unwrap();
    let surgeist_html_semantics::HtmlCapturedNodeKind::Document(facts) = captured_document.kind()
    else {
        panic!("checked canonical Document must capture as Document")
    };
    assert_eq!(
        facts.scripting(),
        surgeist_html_semantics::HtmlScriptingStatus::Disabled
    );
    assert_eq!(
        facts.nonce_environment().browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Present
    );
    let captured_registry = captured_document.lookup_registry();
    let surgeist_html_semantics::HtmlCapturedRegistry::Registry(id) = &captured_registry else {
        panic!("actual Global registry remains non-null in producer capture")
    };
    assert_eq!(
        producer.registry_scope(id).unwrap(),
        surgeist_html_semantics::HtmlRegistryScope::Global
    );
    let surgeist_html_semantics::HtmlCapturedDefinitions::Complete(definitions) =
        producer.definitions(id).unwrap()
    else {
        panic!("the actual empty Global registry is complete")
    };
    assert!(definitions.is_empty());
    assert!(
        producer
            .lookup_definition(
                &captured_registry,
                &Namespace::html(),
                &"x-native".into(),
                None
            )
            .unwrap()
            .is_none()
    );
    assert_eq!(
        captured
            .source()
            .document(top.document())
            .unwrap()
            .metadata()
            .custom_element_registry(),
        DomRegistryBinding::Registry(canonical_registry)
    );
    for node in std::iter::once(html.id()).chain(html.children().iter().copied()) {
        let subject = captured.subject(node).unwrap();
        assert_eq!(captured.node(&subject).unwrap(), node);
        assert_eq!(
            captured
                .source()
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .custom_element_registry(),
            DomRegistryBinding::Registry(canonical_registry)
        );
        let captured_element = producer.node(&subject).unwrap();
        assert!(matches!(
            captured_element.kind(),
            surgeist_html_semantics::HtmlCapturedNodeKind::Element(_)
        ));
        assert_eq!(captured_element.lookup_registry(), captured_registry);
    }
    let hosted = session.hosted_execution();
    let dom = session.dom();
    assert_eq!(
        hosted.top_level(&dom, top.navigable(), &cancel()).unwrap(),
        top
    );
    let settings = hosted.settings(&dom, top.global(), &cancel()).unwrap();
    assert_eq!(settings.global(), top.global());
    assert_eq!(settings.document().id(), top.document());
    assert_eq!(settings.document().metadata().origin(), metadata.origin());
    assert!(settings.environment_ready());
    assert_eq!(settings.clock_source(), top.host().source());
    assert_eq!(settings.time_origin_microseconds() % 100, 0);
    assert_eq!(
        settings.creation_url().serialization(),
        &surgeist_html::values::DocumentString::from("about:blank")
    );
    assert_eq!(settings.top_level_creation_url(), settings.creation_url());
    assert!(!settings.cross_origin_isolated());
    assert!(!settings.cross_site());
    let entry = hosted.history_entry(&dom, top.entry(), &cancel()).unwrap();
    assert_eq!(entry.entry(), top.entry());
    assert_eq!(entry.document(), top.document());
    assert_eq!(entry.origin(), metadata.origin());
    assert_eq!(entry.step(), Some(0));
    for value in [entry.key(), entry.id()] {
        assert_eq!(value.len(), 36);
        for (index, unit) in value.iter().copied().enumerate() {
            if [8, 13, 18, 23].contains(&index) {
                assert_eq!(unit, u16::from(b'-'));
            } else {
                assert!(matches!(unit, 0x30..=0x39 | 0x61..=0x66));
            }
        }
        assert_eq!(value[14], u16::from(b'4'));
        assert!(matches!(value[19], 0x38 | 0x39 | 0x61 | 0x62));
    }
    let again = hosted.history_entry(&dom, top.entry(), &cancel()).unwrap();
    assert_eq!(again.entry(), entry.entry());
    assert_eq!(again.document(), entry.document());
    assert_eq!(again.origin(), entry.origin());
    assert_eq!(again.step(), entry.step());
    assert_eq!(again.key(), entry.key());
    assert_eq!(again.id(), entry.id());
}

#[test]
fn nonbubbling_toggle_captures_global_then_document_and_real_ancestors() {
    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        seen: Vec::new(),
    };
    let mut registry = registry();
    let target = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(top.document(), Namespace::html(), "details");
        batch.insert_before(&details, parent, None);
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        for (recipient, label, capture) in [
            (DomEventTarget::Global(top.global()), "global", true),
            (
                DomEventTarget::Node(top.document().node()),
                "document",
                true,
            ),
            (DomEventTarget::Node(parent), "ancestor", true),
            (DomEventTarget::Node(target), "target-capture", true),
            (DomEventTarget::Node(target), "target", false),
            (DomEventTarget::Global(top.global()), "global-bubble", false),
        ] {
            listen(&mut registry, &mut dom, recipient, label, capture);
        }
        let mut batch = DomBatch::new();
        batch.set_attribute(target, "open", "");
        dom.apply(batch, &cancel()).unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        target
    };
    run_details(&mut session, &mut registry, &mut context);
    assert_eq!(
        context.seen.iter().map(|o| o.label).collect::<Vec<_>>(),
        ["global", "document", "ancestor", "target-capture", "target"]
    );
    assert_eq!(
        context.seen[0].current,
        Some(DomEventTarget::Global(top.global()))
    );
    assert_eq!(
        context.seen[1].current,
        Some(DomEventTarget::Node(top.document().node()))
    );
    assert_eq!(
        context.seen.iter().map(|o| o.phase).collect::<Vec<_>>(),
        [
            DomEventPhase::Capturing,
            DomEventPhase::Capturing,
            DomEventPhase::Capturing,
            DomEventPhase::AtTarget,
            DomEventPhase::AtTarget,
        ]
    );
    for observation in &context.seen {
        assert_eq!(observation.target, Some(target));
        assert!(!observation.bubbles);
        let payload = observation.payload.as_ref().unwrap();
        assert_eq!(
            payload.old_state(),
            "closed".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            payload.new_state(),
            "open".encode_utf16().collect::<Vec<_>>()
        );
    }
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn zero_child_destruction_still_queues_and_survives_permission_drop_and_shutdown() {
    let mut session = session();
    let top = create(&mut session);
    let historical = session.snapshot();
    let mut hosted = session.hosted_execution();
    let queued;
    {
        let mut dom = session.dom();
        queued = hosted
            .destroy_top_level(&mut dom, top.navigable(), &cancel())
            .unwrap();
        assert_eq!(queued.host(), top.host());
        assert_eq!(queued.navigable(), top.navigable());
        assert_eq!(queued.traversable(), top.navigable());
    }
    assert!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    drop(hosted);
    session = match session.try_shutdown() {
        DomShutdownOutcome::Pending { session, .. } => session,
        DomShutdownOutcome::Complete => panic!("queued parent-Global task retains custody"),
    };
    let mut hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        assert!(
            matches!(pump_destruction(&mut hosted, &mut dom, queued).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        );
    }
    let current = session.snapshot();
    assert!(current.node(top.document().node()).is_ok());
    assert!(
        !current
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        historical
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let dom = session.dom();
    assert!(hosted.top_level(&dom, top.navigable(), &cancel()).is_err());
    assert!(hosted.settings(&dom, top.global(), &cancel()).is_err());
    assert_eq!(
        historical.global_document(top.global()).unwrap(),
        top.document()
    );
}

#[test]
fn queued_toggle_keeps_its_identity_but_adoption_uses_the_current_global_event_path() {
    let mut session = session();
    let first = create(&mut session);
    let second = create(&mut session);
    let first_body = body(&session.snapshot(), first.document());
    let second_body = body(&session.snapshot(), second.document());
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        seen: Vec::new(),
    };
    let mut registry = registry();
    let (target, queued) = {
        let mut dom = session.dom();
        listen(
            &mut registry,
            &mut dom,
            DomEventTarget::Global(first.global()),
            "original-global",
            true,
        );
        listen(
            &mut registry,
            &mut dom,
            DomEventTarget::Global(second.global()),
            "current-global",
            true,
        );
        listen(
            &mut registry,
            &mut dom,
            DomEventTarget::Node(second.document().node()),
            "current-document",
            true,
        );
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(first.document(), Namespace::html(), "details");
        batch.insert_before(&details, first_body, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        let queued = context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        let association = queued.queue_association().unwrap();
        assert_eq!(association.global(), first.global());
        assert_eq!(association.document(), first.document());
        let mut batch = DomBatch::new();
        batch.adopt(target, second.document());
        batch.insert_before(target, second_body, None);
        dom.apply(batch, &cancel()).unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            0
        );
        assert_eq!(
            context
                .element_tasks
                .pending_task(
                    surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                    0
                )
                .unwrap()
                .unwrap()
                .id(),
            queued.id()
        );
        assert_eq!(
            context
                .element_tasks
                .pending_task(
                    surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                    0
                )
                .unwrap()
                .unwrap()
                .queue_association(),
            Some(association)
        );
        (target, queued)
    };
    assert_eq!(
        session.snapshot().node(target).unwrap().owner_document(),
        Some(second.document())
    );
    run_details(&mut session, &mut registry, &mut context);
    assert_eq!(
        context.seen.iter().map(|o| o.label).collect::<Vec<_>>(),
        ["current-global", "current-document"]
    );
    assert_eq!(queued.id().target(), target);
    assert_eq!(
        context.seen[0].current,
        Some(DomEventTarget::Global(second.global()))
    );
    assert_eq!(
        context.seen[1].current,
        Some(DomEventTarget::Node(second.document().node()))
    );
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    let hosted = session.hosted_execution();
    let dom = session.dom();
    assert_eq!(
        hosted
            .settings(&dom, first.global(), &cancel())
            .unwrap()
            .document()
            .id(),
        first.document()
    );
    assert_eq!(
        hosted
            .settings(&dom, second.global(), &cancel())
            .unwrap()
            .document()
            .id(),
        second.document()
    );
}

#[test]
fn cancelled_and_foreign_host_permission_does_not_publish_a_document_then_same_owner_recovers() {
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let before = session.snapshot();
    {
        let mut dom = session.dom();
        assert!(matches!(
            hosted.create_host(&mut dom, &AtomicBool::new(true)),
            Err(DomHostedRuntimeError::Cancelled)
        ));
    }
    assert_eq!(session.snapshot().stamp(), before.stamp());
    let mut foreign = self::session();
    {
        let mut dom = foreign.dom();
        assert!(matches!(
            hosted.create_host(&mut dom, &cancel()),
            Err(DomHostedRuntimeError::Binding)
        ));
    }
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(foreign.snapshot().nodes().count(), 0);
    let top = create(&mut session);
    assert!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let dom = session.dom();
    assert_eq!(
        hosted.top_level(&dom, top.navigable(), &cancel()).unwrap(),
        top
    );
}

#[test]
fn zero_host_admission_refuses_before_any_document_or_global_is_published() {
    let dom = Dom::new(
        DomLimits::new(
            128,
            20_000,
            128,
            128,
            128,
            16,
            1_000_000,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            surgeist_dom::event::DomDispatchCustodyLimits::new(
                64, 4096, 16_000_000, 16_000_000, 4_000_000,
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let mut policy = limits();
    policy.hosted.canonical = DomHostedLimits::new(
        DomHostedRecordLimits::new(0, 32, 32, 32, 32, 32, 32),
        16_000_000,
        16_000_000,
        1_000_000,
    );
    let mut session = DomSession::new(dom, policy).unwrap();
    let before = session.snapshot();
    let mut hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        assert!(matches!(
            hosted.create_host(&mut dom, &cancel()),
            Err(DomHostedRuntimeError::Admission)
        ));
    }
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(session.snapshot().nodes().count(), 0);
}

#[test]
fn destroying_recorded_task_document_cancels_toggle_after_target_adoption() {
    let mut session = session();
    let first = create(&mut session);
    let second = create(&mut session);
    let first_body = body(&session.snapshot(), first.document());
    let second_body = body(&session.snapshot(), second.document());
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        seen: Vec::new(),
    };
    let mut registry = registry();
    let target = {
        let mut dom = session.dom();
        for (recipient, label) in [
            (DomEventTarget::Global(first.global()), "original-global"),
            (DomEventTarget::Global(second.global()), "current-global"),
            (
                DomEventTarget::Node(second.document().node()),
                "current-document",
            ),
        ] {
            listen(&mut registry, &mut dom, recipient, label, true);
        }
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(first.document(), Namespace::html(), "details");
        batch.insert_before(&details, first_body, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        listen(
            &mut registry,
            &mut dom,
            DomEventTarget::Node(target),
            "target",
            false,
        );
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        let queued = context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        let association = queued.queue_association().unwrap();
        assert_eq!(association.global(), first.global());
        assert_eq!(association.document(), first.document());
        let mut batch = DomBatch::new();
        batch.adopt(target, second.document());
        batch.insert_before(target, second_body, None);
        dom.apply(batch, &cancel()).unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            0
        );
        let after = context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        assert_eq!(after.id(), queued.id());
        assert_eq!(after.queue_association(), Some(association));
        target
    };
    let mut hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        let queued = hosted
            .destroy_top_level(&mut dom, first.navigable(), &cancel())
            .unwrap();
        assert_eq!(queued.host(), first.host());
        assert_eq!(queued.navigable(), first.navigable());
        assert_eq!(queued.traversable(), first.navigable());
        assert_eq!(
            context
                .element_tasks
                .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
                .unwrap(),
            1
        );
        assert!(
            matches!(pump_destruction(&mut hosted, &mut dom, queued).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        );
        assert!(matches!(
            DomElementTaskRuntime::run_next(
                &mut dom,
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                &mut registry,
                &mut context,
                crate::DomElementTaskRunLimits {
                    dispatch: event_limits(),
                    source: surgeist_dom::source::DomSourceLimits::new(
                        16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                    )
                },
                &cancel()
            )
            .unwrap(),
            DomElementTaskProgress::Empty
        ));
    }
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert!(context.seen.is_empty());
    let snapshot = session.snapshot();
    assert!(
        !snapshot
            .document(first.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        snapshot
            .document(second.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let current = snapshot.node(target).unwrap();
    assert_eq!(current.owner_document(), Some(second.document()));
    assert_eq!(current.parent(), Some(second_body));
    assert!(current.is_connected());
    assert!(current.attribute(&"open".into()).is_some());
}

#[test]
fn destroying_one_top_level_preserves_another_top_level_in_the_same_host() {
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let (first, second) = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel()).unwrap();
        let first = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
        let second = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
        assert_eq!(first.host(), host);
        assert_eq!(second.host(), host);
        assert_ne!(first.browsing_context(), second.browsing_context());
        assert_ne!(first.navigable(), second.navigable());
        assert_ne!(first.document(), second.document());
        let queued = hosted
            .destroy_top_level(&mut dom, first.navigable(), &cancel())
            .unwrap();
        assert_eq!(queued.host(), first.host());
        assert_eq!(queued.navigable(), first.navigable());
        assert_eq!(queued.traversable(), first.navigable());
        assert!(
            matches!(pump_destruction(&mut hosted, &mut dom, queued).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        );
        assert_eq!(
            hosted
                .top_level(&dom, second.navigable(), &cancel())
                .unwrap(),
            second
        );
        assert_eq!(
            hosted
                .settings(&dom, second.global(), &cancel())
                .unwrap()
                .document()
                .id(),
            second.document()
        );
        (first, second)
    };
    let snapshot = session.snapshot();
    assert!(
        !snapshot
            .document(first.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        snapshot
            .document(second.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let parent = body(&snapshot, second.document());
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        seen: Vec::new(),
    };
    let mut registry = registry();
    {
        let mut dom = session.dom();
        listen(
            &mut registry,
            &mut dom,
            DomEventTarget::Global(second.global()),
            "surviving-global",
            true,
        );
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(second.document(), Namespace::html(), "details");
        batch.insert_before(&details, parent, None);
        batch.set_attribute(&details, "open", "");
        dom.apply(batch, &cancel()).unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
    }
    run_details(&mut session, &mut registry, &mut context);
    assert_eq!(context.seen.len(), 1);
    assert_eq!(context.seen[0].label, "surviving-global");
    assert_eq!(
        context.seen[0].current,
        Some(DomEventTarget::Global(second.global()))
    );
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn pending_slot_checkpoint_retains_delivered_task_without_replaying_toggle() {
    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        seen: Vec::new(),
    };
    let mut registry = registry();
    let (slot, target, queued) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let host = batch.create_element_ns(top.document(), Namespace::html(), "div");
        batch.insert_before(&host, parent, None);
        let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
        let slot = batch.create_element_ns(top.document(), Namespace::html(), "slot");
        batch.insert_before(&slot, &shadow, None);
        let light = batch.create_text(top.document(), "assigned");
        batch.insert_before(&light, &host, None);
        let details = batch.create_element_ns(top.document(), Namespace::html(), "details");
        batch.insert_before(&details, parent, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let slot = commit.created(&slot).unwrap().unwrap();
        let light = commit.created(&light).unwrap().unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        let snapshot = dom.snapshot();
        assert_eq!(snapshot.slot(slot).unwrap().assigned_nodes(), &[light]);
        assert_eq!(snapshot.pending_slot_signals(), &[slot]);
        listen(
            &mut registry,
            &mut dom,
            DomEventTarget::Node(target),
            "target",
            false,
        );
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        let queued = context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        assert_eq!(queued.queue_association().unwrap().global(), top.global());
        (slot, target, queued)
    };
    for _ in 0..2 {
        let mut dom = session.dom();
        assert!(matches!(
            DomElementTaskRuntime::run_next(
                &mut dom,
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                &mut registry,
                &mut context,
                crate::DomElementTaskRunLimits {
                    dispatch: event_limits(),
                    source: surgeist_dom::source::DomSourceLimits::new(
                        16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                    )
                },
                &cancel()
            ),
            Err(DomElementTaskRuntimeError::Hosted(
                DomHostedRuntimeError::CheckpointUnavailable
            ))
        ));
        assert_eq!(context.seen.len(), 1);
        assert_eq!(context.seen[0].target, Some(target));
        assert_eq!(
            context.seen[0].payload.as_ref().unwrap().new_state(),
            "open".encode_utf16().collect::<Vec<_>>()
        );
        let retained = context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        assert_eq!(retained.id(), queued.id());
        assert_eq!(retained.queue_association(), queued.queue_association());
        assert_eq!(dom.snapshot().pending_slot_signals(), &[slot]);
    }
    let session = match session.try_shutdown() {
        DomShutdownOutcome::Pending { session, reason } => {
            assert_eq!(reason, DomShutdownReason::DomElementTask(queued.id()));
            session
        }
        DomShutdownOutcome::Complete => {
            panic!("undelivered Slot checkpoint retains the original task")
        }
    };
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(context.seen.len(), 1);
    assert!(
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(session.snapshot().pending_slot_signals(), &[slot]);
}

#[test]
fn retired_document_retains_canonical_creation_and_text_without_live_execution() {
    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let text = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let text = batch.create_text(top.document(), "before retirement");
        batch.insert_before(&text, parent, None);
        dom.apply(batch, &cancel())
            .unwrap()
            .created(&text)
            .unwrap()
            .unwrap()
    };
    let historical = session.snapshot();
    let mut hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        let queued = hosted
            .destroy_top_level(&mut dom, top.navigable(), &cancel())
            .unwrap();
        assert_eq!(queued.host(), top.host());
        assert_eq!(queued.navigable(), top.navigable());
        assert_eq!(queued.traversable(), top.navigable());
        assert!(
            matches!(pump_destruction(&mut hosted, &mut dom, queued).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        );
    }
    let retired = session.snapshot();
    assert!(
        historical
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        !retired
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        retired.node(text).unwrap().data(),
        Some(&"before retirement".into())
    );
    {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let span = batch.create_element_ns(top.document(), Namespace::html(), "span");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let span = commit.created(&span).unwrap().unwrap();
        let created = dom.snapshot();
        let node = created.node(span).unwrap();
        assert_eq!(node.owner_document(), Some(top.document()));
        assert_eq!(node.parent(), None);
        assert_eq!(node.node_name(), DocumentString::from("SPAN"));
        assert_eq!(created.relevant_global(span).unwrap(), Some(top.global()));
        assert_eq!(
            node.as_element().unwrap().custom_element_registry(),
            retired
                .document(top.document())
                .unwrap()
                .metadata()
                .custom_element_registry()
        );
        assert_ne!(created.stamp(), retired.stamp());
        let mut batch = DomBatch::new();
        batch.replace_data(text, 0, usize::MAX, "after retirement");
        dom.apply(batch, &cancel()).unwrap();
    }
    let current = session.snapshot();
    assert_eq!(
        current.node(text).unwrap().data(),
        Some(&"after retirement".into())
    );
    assert_eq!(
        current.node(text).unwrap().owner_document(),
        Some(top.document())
    );
    assert!(
        !current
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        historical.node(text).unwrap().data(),
        Some(&"before retirement".into())
    );
    assert!(
        historical
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        hosted
            .settings(&session.dom(), top.global(), &cancel())
            .is_err()
    );
}

#[test]
fn associated_html_creator_and_template_owner_do_not_copy_live_hosted_association() {
    let mut session = session();
    let top = create(&mut session);
    let source = session.snapshot();
    let source_metadata = source.document(top.document()).unwrap().metadata();
    let (copy, template) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let copy = batch.create_html_document(top.document(), None);
        let template = batch.create_element_ns(top.document(), Namespace::html(), "template");
        let commit = dom.apply(batch, &cancel()).unwrap();
        (
            commit.document(&copy).unwrap(),
            commit.created(&template).unwrap().unwrap(),
        )
    };
    let snapshot = session.snapshot();
    let copied = snapshot.document(copy).unwrap();
    assert_eq!(copied.metadata().origin(), source_metadata.origin());
    assert_eq!(
        copied.metadata().document_environment().input_platform(),
        source_metadata.document_environment().input_platform()
    );
    assert_eq!(
        copied.metadata().document_environment().browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent
    );
    assert!(
        !copied
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert_eq!(
        copied.metadata().custom_element_registry(),
        DomRegistryBinding::Null
    );
    assert!(!copied.is_fully_active().unwrap());
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(inert, top.document());
    assert_ne!(inert, copy);
    let inert_document = snapshot.document(inert).unwrap();
    assert_eq!(
        inert_document.template_role(),
        DomTemplateDocumentRole::InertTemplate
    );
    assert_eq!(
        inert_document
            .metadata()
            .document_environment()
            .input_platform(),
        source_metadata.document_environment().input_platform()
    );
    assert_eq!(
        inert_document
            .metadata()
            .document_environment()
            .browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent
    );
    assert!(
        !inert_document
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert_eq!(
        inert_document.metadata().custom_element_registry(),
        DomRegistryBinding::Null
    );
    assert!(!inert_document.is_fully_active().unwrap());
    assert!(
        snapshot
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot.global_document(top.global()).unwrap(),
        top.document()
    );
}

#[test]
fn owned_shadow_retains_the_real_global_registry_and_capture_deduplicates_it() {
    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (host, shadow, child) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let host = batch.create_element_ns(top.document(), Namespace::html(), "div");
        batch.insert_before(&host, parent, None);
        let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
        let child = batch.create_element_ns(top.document(), Namespace::html(), "span");
        batch.insert_before(&child, &shadow, None);
        let commit = dom.apply(batch, &cancel()).unwrap();
        (
            commit.created(&host).unwrap().unwrap(),
            commit.attached_shadow_root(&shadow).unwrap().unwrap(),
            commit.created(&child).unwrap().unwrap(),
        )
    };
    let snapshot = session.snapshot();
    let canonical = snapshot
        .document(top.document())
        .unwrap()
        .metadata()
        .custom_element_registry();
    assert!(matches!(canonical, DomRegistryBinding::Registry(_)));
    assert_eq!(
        snapshot
            .shadow_root(shadow)
            .unwrap()
            .state()
            .custom_element_registry(),
        canonical
    );
    assert_eq!(
        snapshot
            .node(child)
            .unwrap()
            .as_element()
            .unwrap()
            .custom_element_registry(),
        canonical
    );
    let capture = snapshot
        .html_capture(
            top.document(),
            None,
            surgeist_html_semantics::HtmlCaptureLimits::new(
                128, 512, 512, 100_000, 128, 32, 32, 16_777_216,
            ),
            DomHtmlCaptureLimits::new(128, 1_000_000),
            &cancel(),
        )
        .unwrap();
    assert_eq!(capture.capture().usage().registries(), 1);
    let document_subject = capture.subject(top.document().node()).unwrap();
    let registry = capture
        .capture()
        .node(&document_subject)
        .unwrap()
        .lookup_registry();
    assert!(matches!(
        registry,
        surgeist_html_semantics::HtmlCapturedRegistry::Registry(_)
    ));
    for node in [host, shadow, child] {
        let subject = capture.subject(node).unwrap();
        assert_eq!(capture.node(&subject).unwrap(), node);
        assert_eq!(
            capture.capture().node(&subject).unwrap().lookup_registry(),
            registry
        );
    }
    let shadow_subject = capture.subject(shadow).unwrap();
    let captured_shadow = capture.capture().node(&shadow_subject).unwrap();
    let Some(surgeist_html_semantics::HtmlCapturedHost::Element(actual_host)) =
        captured_shadow.host()
    else {
        panic!("actual owned shadow must retain its reciprocal host")
    };
    assert_eq!(capture.node(actual_host.subject()).unwrap(), host);
}

#[test]
fn unsupported_child_navigable_and_svg_processing_profiles_refuse_connection_atomically() {
    use surgeist_html::{elements::native::HtmlElementClass, vocabulary::KnownHtmlElement};
    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let closed = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let closed = batch.create_html_document(top.document(), None);
        dom.apply(batch, &cancel())
            .unwrap()
            .document(&closed)
            .unwrap()
    };
    assert!(
        !session
            .snapshot()
            .document(closed)
            .unwrap()
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    for (namespace, local, requirement, expected_connection) in [
        (
            Namespace::html(),
            "frame",
            HtmlElementClass::HtmlFrame,
            DomError::HostedSourceUnavailable,
        ),
        (
            Namespace::html(),
            "iframe",
            HtmlElementClass::KnownHtml(KnownHtmlElement::Iframe),
            DomError::HostedSourceUnavailable,
        ),
        (
            Namespace::svg(),
            "script",
            HtmlElementClass::SvgScript,
            DomError::SvgScriptProcessingUnavailable,
        ),
    ] {
        if local == "script" {
            let before = session.snapshot();
            let mut dom = session.dom();
            let usage = dom.usage();
            let mut batch = DomBatch::new();
            batch.set_attribute(parent, "data-unpublished", "prefix");
            batch.create_element_ns(top.document(), namespace.clone(), local);
            assert_eq!(
                dom.apply(batch, &cancel()).unwrap_err(),
                DomError::NativeInitializationRequired(requirement)
            );
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
        }
        // A closed creator can initialize the native node. Adoption
        // retains that state, but connection must qualify its CURRENT Doc.
        let node = {
            let mut dom = session.dom();
            let mut batch = DomBatch::new();
            let node = batch.create_element_ns(closed, namespace, local);
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&node)
                .unwrap()
                .unwrap()
        };
        let before = session.snapshot();
        let mut dom = session.dom();
        let usage = dom.usage();
        let mut import = DomBatch::new();
        import.import_node(&before, node, top.document(), false);
        assert_eq!(
            dom.apply(import, &cancel()).unwrap_err(),
            DomError::NativeInitializationRequired(requirement)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);

        let mut batch = DomBatch::new();
        batch.set_attribute(parent, "data-unpublished", "prefix");
        batch.adopt(node, top.document());
        batch.insert_before(node, parent, None);
        assert_eq!(
            dom.apply(batch, &cancel()).unwrap_err(),
            expected_connection
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert_eq!(dom.snapshot().node(node).unwrap().parent(), None);
        assert_eq!(
            dom.snapshot().node(node).unwrap().owner_document(),
            Some(closed)
        );
        assert!(
            dom.snapshot()
                .node(parent)
                .unwrap()
                .attribute(&"data-unpublished".into())
                .is_none()
        );
        if local == "script" {
            assert!(
                !dom.snapshot()
                    .node(node)
                    .unwrap()
                    .as_svg_script()
                    .unwrap()
                    .flags()
                    .already_processed()
            );
        }
    }
}

#[test]
fn html_script_in_present_native_context_stops_at_explicit_disabled_scripting() {
    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let script = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let script = batch.create_element_ns(top.document(), Namespace::html(), "script");
        let text = batch.create_text(top.document(), "void 0;");
        batch.insert_before(&text, &script, None);
        batch.insert_before(&script, parent, None);
        dom.apply(batch, &cancel())
            .unwrap()
            .created(&script)
            .unwrap()
            .unwrap()
    };
    let snapshot = session.snapshot();
    let environment = snapshot
        .document(top.document())
        .unwrap()
        .metadata()
        .document_environment();
    assert_eq!(
        environment.browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Present
    );
    assert_eq!(
        snapshot.document(top.document()).unwrap().scripting(),
        surgeist_html_semantics::HtmlScriptingStatus::Disabled
    );
    // HTML877d prepare-the-script steps9,15,16 precede the explicit step18
    // disabled-scripting return; an actual BC does not bypass those writes.
    let state = snapshot.node(script).unwrap().as_html_script().unwrap();
    assert_eq!(
        state.script_type(),
        Some(surgeist_html::elements::script::HtmlScriptKind::Classic)
    );
    assert!(state.flags().already_started());
    assert!(state.flags().force_async());
    assert!(!state.flags().external_file());
    assert!(!state.flags().ready_to_be_parser_executed());
    assert!(!state.flags().delaying_load());
    assert_eq!(state.parser_document(), None);
    assert_eq!(state.preparation_document(), Some(top.document()));
    assert_eq!(
        state.result(),
        surgeist_html::elements::script::HtmlScriptResult::Uninitialized
    );
    assert_eq!(state.steps_when_result_ready(), None);
}

#[test]
fn live_original_task_refuses_a_retired_adoption_destination_before_callbacks() {
    let mut session = session();
    let first = create(&mut session);
    let second = create(&mut session);
    let first_body = body(&session.snapshot(), first.document());
    let second_body = body(&session.snapshot(), second.document());
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        seen: Vec::new(),
    };
    let mut registry = registry();
    let (target, queued) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(first.document(), Namespace::html(), "details");
        batch.insert_before(&details, first_body, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        let queued = context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        let association = queued.queue_association().unwrap();
        assert_eq!(association.document(), first.document());
        assert_eq!(association.global(), first.global());
        let mut batch = DomBatch::new();
        batch.adopt(target, second.document());
        batch.insert_before(target, second_body, None);
        dom.apply(batch, &cancel()).unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            0
        );
        for (recipient, label, capture) in [
            (
                DomEventTarget::Global(first.global()),
                "original-global",
                true,
            ),
            (
                DomEventTarget::Global(second.global()),
                "destination-global",
                true,
            ),
            (
                DomEventTarget::Node(second.document().node()),
                "destination-document",
                true,
            ),
            (DomEventTarget::Node(target), "target", false),
        ] {
            listen(&mut registry, &mut dom, recipient, label, capture);
        }
        (target, queued)
    };
    let mut hosted = session.hosted_execution();
    {
        let mut dom = session.dom();
        let queued = hosted
            .destroy_top_level(&mut dom, second.navigable(), &cancel())
            .unwrap();
        assert_eq!(queued.host(), second.host());
        assert_eq!(queued.navigable(), second.navigable());
        assert_eq!(queued.traversable(), second.navigable());
        assert!(
            matches!(pump_destruction(&mut hosted, &mut dom, queued).unwrap(),
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        );
    }
    let before = session.snapshot();
    assert!(
        before
            .document(first.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        !before
            .document(second.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        before.node(target).unwrap().owner_document(),
        Some(second.document())
    );
    assert!(before.node(target).unwrap().is_connected());
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    let retained = context
        .element_tasks
        .pending_task(
            surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
            0,
        )
        .unwrap()
        .unwrap();
    assert_eq!(retained.id(), queued.id());
    assert_eq!(retained.queue_association(), queued.queue_association());
    let retired = before.document(second.document()).unwrap();
    let environment = retired.metadata().document_environment();
    assert!(environment.has_ever_had_navigable());
    assert_eq!(
        environment.browsing_context(),
        surgeist_html::security::nonce::HtmlBrowsingContextPresence::Absent
    );
    // The standard parent is null, but the selected native task must still
    // qualify the actual retired destination before invoking Rust callbacks.
    let result = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: surgeist_dom::source::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    );
    assert!(
        matches!(
            result,
            Err(DomElementTaskRuntimeError::Dispatch(
                DomEventDispatchError::Hosted(DomHostedError::StaleIdentity)
            ))
        ),
        "the retired destination must refuse before the original task starts: {result:?}"
    );
    assert!(context.seen.is_empty());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(
        context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0
            )
            .unwrap()
            .unwrap()
            .id(),
        queued.id()
    );
    assert_eq!(
        context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0
            )
            .unwrap()
            .unwrap()
            .queue_association(),
        queued.queue_association()
    );
}

#[test]
fn destination_retired_in_global_capture_prevents_later_callbacks_without_replaying_capture() {
    struct RetireContext {
        element_tasks: DomElementTaskRuntime,
        hosted: HostedRuntime,
        destination: DomHostedTopLevel,
        seen: Vec<&'static str>,
        observed: Vec<NativeDomEvent>,
        destruction: Vec<Result<DomHostedDestructionQueued, DomHostedRuntimeError>>,
        settlement: Vec<Result<DomHostedTaskProgress, DomHostedRuntimeError>>,
    }
    impl DomElementTaskEventContext for RetireContext {
        fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
            &mut self.element_tasks
        }
    }
    let mut session = session();
    let first = create(&mut session);
    let second = create(&mut session);
    let first_body = body(&session.snapshot(), first.document());
    let second_body = body(&session.snapshot(), second.document());
    let mut context = RetireContext {
        element_tasks: session.element_task_execution(),
        hosted: session.hosted_execution(),
        destination: second,
        seen: Vec::new(),
        observed: Vec::new(),
        destruction: Vec::new(),
        settlement: Vec::new(),
    };
    let mut registry = DomEventDispatcher::<RetireContext>::new(DomEventRegistryLimits {
        registrations: 32,
        pending_events: 8,
        nested_dispatches: 8,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let (target, queued) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(first.document(), Namespace::html(), "details");
        batch.insert_before(&details, first_body, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        let queued = context
            .element_tasks
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            queued.queue_association().unwrap().document(),
            first.document()
        );
        let mut batch = DomBatch::new();
        batch.adopt(target, second.document());
        batch.insert_before(target, second_body, None);
        dom.apply(batch, &cancel()).unwrap();
        registry
            .add_listener(
                &mut dom,
                DomEventTarget::Global(second.global()),
                DomEventType::TOGGLE,
                DomEventCallback::new(|dom, _, context: &mut RetireContext, event| {
                    context.seen.push("destination-global");
                    context.observed.push(event.clone());
                    let first = context.hosted.destroy_top_level(
                        dom,
                        context.destination.navigable(),
                        &cancel(),
                    );
                    if let Ok(queued) = &first {
                        context.settlement.push(pump_destruction(
                            &mut context.hosted,
                            dom,
                            *queued,
                        ));
                    }
                    context.destruction.push(first);
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                    passive: None,
                },
            )
            .unwrap();
        for (recipient, label, capture) in [
            (
                DomEventTarget::Node(second.document().node()),
                "destination-document",
                true,
            ),
            (DomEventTarget::Node(target), "target", false),
        ] {
            registry
                .add_listener(
                    &mut dom,
                    recipient,
                    DomEventType::TOGGLE,
                    DomEventCallback::new(move |_, _, context: &mut RetireContext, _| {
                        context.seen.push(label);
                        Ok(())
                    }),
                    DomEventListenerOptions {
                        capture,
                        once: false,
                        passive: None,
                    },
                )
                .unwrap();
        }
        (target, queued)
    };
    let result = {
        let mut dom = session.dom();
        DomElementTaskRuntime::run_next(
            &mut dom,
            surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::source::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        )
    };
    // Check the real nested operation OUTSIDE callbacks: panic capture cannot
    // hide an unsuccessful teardown or make the no-delivery oracle pass.
    assert_eq!(context.destruction.len(), 1);
    let destruction_queued = context.destruction[0].as_ref().unwrap();
    assert_eq!(destruction_queued.host(), second.host());
    assert_eq!(destruction_queued.navigable(), second.navigable());
    assert_eq!(destruction_queued.traversable(), second.navigable());
    assert_eq!(context.settlement.len(), 1);
    assert!(
        matches!(&context.settlement[0], Ok(DomHostedTaskProgress::Complete { operation }) if *operation == destruction_queued.operation())
    );
    let snapshot = session.snapshot();
    assert!(
        snapshot
            .document(first.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        !snapshot
            .document(second.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot.node(target).unwrap().owner_document(),
        Some(second.document())
    );
    assert_eq!(context.seen, ["destination-global"]);
    assert_eq!(context.observed.len(), 1);
    assert_eq!(context.observed[0].target(), Some(target.into()));
    assert_eq!(
        context.observed[0].toggle_event().unwrap().new_state(),
        "open".encode_utf16().collect::<Vec<_>>()
    );
    if let Ok(DomElementTaskProgress::Complete { receipt, .. }) = &result {
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
    }
    // Completion after skipping inactive recipients, or retention at a current
    // authority boundary, must preserve the delivered capture. Retry cannot
    // invoke that capture again or reach later ineligible recipients.
    {
        let mut dom = session.dom();
        let _retry = DomElementTaskRuntime::run_next(
            &mut dom,
            surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::source::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        );
    }
    assert_eq!(context.seen, ["destination-global"]);
    assert_eq!(context.observed.len(), 1);
    if let Some(retained) = context
        .element_tasks
        .pending_task(
            surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
            0,
        )
        .unwrap()
    {
        assert_eq!(retained.id(), queued.id());
        assert_eq!(retained.queue_association(), queued.queue_association());
    }
}

#[test]
fn adopted_target_retirement_delivers_once_using_its_active_destination_document() {
    struct DestroyTargetContext {
        element_tasks: DomElementTaskRuntime,
        target: NodeId,
        seen: Vec<&'static str>,
        applied: Vec<Result<DomCommit, DomError>>,
    }
    impl DomElementTaskEventContext for DestroyTargetContext {
        fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
            &mut self.element_tasks
        }
    }
    let mut session = session();
    let first = create(&mut session);
    let second = create(&mut session);
    let first_body = body(&session.snapshot(), first.document());
    let second_body = body(&session.snapshot(), second.document());
    let mut details = session.element_task_execution();
    let (target, queued) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let target = batch.create_element_ns(first.document(), Namespace::html(), "details");
        batch.insert_before(&target, first_body, None);
        batch.set_attribute(&target, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&target).unwrap().unwrap();
        assert_eq!(details.ingest(&mut dom, &cancel()).unwrap(), 1);
        let queued = details
            .pending_task(
                surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
                0,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            queued.queue_association().unwrap().document(),
            first.document()
        );
        assert_eq!(queued.queue_association().unwrap().global(), first.global());
        let mut batch = DomBatch::new();
        batch.adopt(target, second.document());
        batch.insert_before(target, second_body, None);
        dom.apply(batch, &cancel()).unwrap();
        (target, queued)
    };
    let mut context = DestroyTargetContext {
        element_tasks: details,
        target,
        seen: Vec::new(),
        applied: Vec::new(),
    };
    let mut registry = DomEventDispatcher::<DestroyTargetContext>::new(DomEventRegistryLimits {
        registrations: 32,
        pending_events: 8,
        nested_dispatches: 8,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let target_once = {
        let mut dom = session.dom();
        registry
            .add_listener(
                &mut dom,
                DomEventTarget::Global(second.global()),
                DomEventType::TOGGLE,
                DomEventCallback::new(|dom, _, context: &mut DestroyTargetContext, _| {
                    context.seen.push("destination-global");
                    let mut batch = DomBatch::new();
                    batch.detach(context.target);
                    batch.destroy(context.target);
                    context.applied.push(dom.apply(batch, &cancel()));
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: true,
                    passive: None,
                },
            )
            .unwrap();
        registry
            .add_listener(
                &mut dom,
                DomEventTarget::Node(target),
                DomEventType::TOGGLE,
                DomEventCallback::new(|_, _, context: &mut DestroyTargetContext, _| {
                    context.seen.push("destroyed-target");
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                    passive: None,
                },
            )
            .unwrap()
    };
    let adopted = session.snapshot();
    assert_eq!(
        adopted.node(target).unwrap().owner_document(),
        Some(second.document())
    );
    assert_eq!(
        adopted.global_document(second.global()).unwrap(),
        second.document()
    );
    assert!(
        adopted
            .document(second.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let still_queued = context
        .element_tasks
        .pending_task(
            surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
            0,
        )
        .unwrap()
        .unwrap();
    assert_eq!(still_queued.id(), queued.id());
    assert_eq!(still_queued.queue_association(), queued.queue_association());
    let progress = {
        let mut dom = session.dom();
        DomElementTaskRuntime::run_next(
            &mut dom,
            surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::source::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        )
        .unwrap()
    };
    // The fixed started path uses the exact successful retirement association,
    // not stale-node absence or its original queue Document. The destination
    // Document remains active, so its applicable once listener still runs.
    match progress {
        DomElementTaskProgress::Complete {
            task,
            receipt,
            displaced_owner: None,
            dialog_toggle_disposition: None,
        } => {
            assert_eq!(task.id(), queued.id());
            assert_eq!(task.queue_association(), queued.queue_association());
            assert_eq!(receipt.callbacks(), 2);
            assert!(receipt.failures().is_empty());
            assert!(!receipt.default_prevented());
        }
        other => panic!("proven retired target in its active destination must complete: {other:?}"),
    }
    // Actual edit and callback effects are inspected outside the callbacks.
    assert_eq!(context.applied.len(), 1);
    assert!(context.applied[0].is_ok());
    assert_eq!(context.seen, ["destination-global", "destroyed-target"]);
    let retired = session.snapshot();
    assert!(matches!(retired.node(target), Err(DomError::StaleNode)));
    assert!(
        retired
            .document(second.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        retired.global_document(second.global()).unwrap(),
        second.document()
    );
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert!(
        !registry.remove_listener(target_once),
        "the actual once invocation consumed registration"
    );
    let next = {
        let mut dom = session.dom();
        DomElementTaskRuntime::run_next(
            &mut dom,
            surgeist_dom::task::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::source::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        )
        .unwrap()
    };
    assert!(matches!(next, DomElementTaskProgress::Empty));
    assert_eq!(context.seen, ["destination-global", "destroyed-target"]);
    assert_eq!(context.applied.len(), 1);
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::task::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

fn pump_destruction(
    hosted: &mut HostedRuntime,
    dom: &mut DomEventDom<'_>,
    queued: DomHostedDestructionQueued,
) -> Result<DomHostedTaskProgress, DomHostedRuntimeError> {
    let mut last = DomHostedTaskProgress::Idle;
    for _ in 0..256 {
        last = hosted.run_next(dom, queued.traversable(), &cancel())?;
        if matches!(last, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            break;
        }
    }
    Ok(last)
}

fn plain_adoption_subtree(session: &mut DomSession, document: DocumentId) -> (NodeId, NodeId) {
    let parent = body(&session.snapshot(), document);
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(document, Namespace::html(), "div");
    let text = batch.create_text(document, "preserved adoption text");
    batch.insert_before(&text, &element, None);
    batch.insert_before(&element, parent, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    (
        commit.created(&element).unwrap().unwrap(),
        commit.created(&text).unwrap().unwrap(),
    )
}

fn teardown_adoption_destination(session: &mut DomSession, top: DomHostedTopLevel) {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let queued = hosted
        .destroy_top_level(&mut dom, top.navigable(), &cancel())
        .unwrap();
    assert!(matches!(
        pump_destruction(&mut hosted, &mut dom, queued).unwrap(),
        DomHostedTaskProgress::Complete { operation } if operation == queued.operation()
    ));
}

fn assert_retired_adoption_destination(
    session: &mut DomSession,
    top: DomHostedTopLevel,
    binding: DomRegistryBinding,
) {
    let snapshot = session.snapshot();
    let document = snapshot.document(top.document()).unwrap();
    assert!(!document.is_fully_active().unwrap());
    assert!(
        document
            .metadata()
            .document_environment()
            .has_ever_had_navigable()
    );
    assert_eq!(document.metadata().custom_element_registry(), binding);
    let DomRegistryBinding::Registry(id) = binding else {
        panic!("the hosted destination retains its nonnull registry");
    };
    let registry = snapshot.registry(id).unwrap();
    assert_eq!(
        registry.scope(),
        surgeist_html_semantics::HtmlRegistryScope::Global
    );
    assert_eq!(registry.definition_count(), 0);
    assert!(!registry.is_scoped());
    assert_eq!(
        registry
            .lookup_definition(&Namespace::html(), &"div".into(), None)
            .unwrap(),
        None
    );
    assert!(matches!(
        session
            .hosted_execution()
            .settings(&session.dom(), top.global(), &cancel()),
        Err(DomHostedRuntimeError::Dom(DomHostedError::StaleIdentity))
    ));
}

fn assert_adopted_plain_subtree(
    snapshot: &DomSnapshot,
    target: NodeId,
    text: NodeId,
    destination: DocumentId,
    binding: DomRegistryBinding,
) {
    let element = snapshot.node(target).unwrap();
    assert_eq!(element.owner_document(), Some(destination));
    assert_eq!(element.parent(), None);
    assert_eq!(element.children(), &[text]);
    let state = element.as_element().unwrap();
    assert_eq!(state.name().namespace_uri(), Namespace::html().uri());
    assert_eq!(state.name().local_name(), &DocumentString::from("div"));
    assert_eq!(state.custom_element_registry(), binding);
    let child = snapshot.node(text).unwrap();
    assert_eq!(child.owner_document(), Some(destination));
    assert_eq!(child.parent(), Some(target));
    assert_eq!(
        child.data(),
        Some(&DocumentString::from("preserved adoption text"))
    );
}

fn assert_plain_adoption_unchanged(
    before: &DomSnapshot,
    after: &DomSnapshot,
    target: NodeId,
    text: NodeId,
) {
    assert_eq!(after.stamp(), before.stamp());
    for id in [target, text] {
        let old = before.node(id).unwrap();
        let current = after.node(id).unwrap();
        assert_eq!(current.owner_document(), old.owner_document());
        assert_eq!(current.parent(), old.parent());
        assert_eq!(current.children(), old.children());
        assert_eq!(current.kind(), old.kind());
    }
    let parent = before.node(target).unwrap().parent().unwrap();
    assert_eq!(
        after.node(parent).unwrap().children(),
        before.node(parent).unwrap().children()
    );
}

#[test]
fn canonical_adoption_after_hosted_teardown_preserves_identity_and_actual_registry() {
    let mut session = session();
    let original = create(&mut session);
    let destination = create(&mut session);
    let (target, text) = plain_adoption_subtree(&mut session, original.document());
    let binding = session
        .snapshot()
        .document(destination.document())
        .unwrap()
        .metadata()
        .custom_element_registry();
    teardown_adoption_destination(&mut session, destination);
    assert_retired_adoption_destination(&mut session, destination, binding);
    let before = session.snapshot();
    let usage = session.dom().usage();
    let old_parent = before.node(target).unwrap().parent().unwrap();
    let mut batch = DomBatch::new();
    batch.adopt(target, destination.document());
    session.dom().apply(batch, &cancel()).expect(
        "canonical adoption into a surviving Document needs its actual registry, not live execution authority",
    );
    let after = session.snapshot();
    assert_adopted_plain_subtree(&after, target, text, destination.document(), binding);
    assert!(!after.node(old_parent).unwrap().children().contains(&target));
    assert_eq!(
        before.node(target).unwrap().owner_document(),
        Some(original.document())
    );
    assert_eq!(before.node(target).unwrap().parent(), Some(old_parent));
    assert_eq!(session.dom().usage().managed_nodes(), usage.managed_nodes());
    assert_eq!(
        session.dom().usage().string_code_units(),
        usage.string_code_units()
    );
    assert_retired_adoption_destination(&mut session, destination, binding);
    assert!(
        session
            .snapshot()
            .document(original.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}

#[test]
fn canonical_adoption_late_batch_error_after_hosted_teardown_rolls_back_registry_and_tree() {
    let mut session = session();
    let original = create(&mut session);
    let destination = create(&mut session);
    let (target, text) = plain_adoption_subtree(&mut session, original.document());
    let binding = session
        .snapshot()
        .document(destination.document())
        .unwrap()
        .metadata()
        .custom_element_registry();
    teardown_adoption_destination(&mut session, destination);
    assert_retired_adoption_destination(&mut session, destination, binding);
    let before = session.snapshot();
    let usage = session.dom().usage();
    let mut batch = DomBatch::new();
    batch.adopt(target, destination.document());
    // This fails only after the preceding adoption and registry lookup staged.
    batch.destroy(destination.document());
    assert_eq!(
        session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::InvalidDestruction
    );
    assert_plain_adoption_unchanged(&before, &session.snapshot(), target, text);
    assert_eq!(session.dom().usage(), usage);
    assert_retired_adoption_destination(&mut session, destination, binding);
}

#[test]
fn canonical_adoption_cancellation_after_hosted_teardown_publishes_nothing() {
    let mut session = session();
    let original = create(&mut session);
    let destination = create(&mut session);
    let (target, text) = plain_adoption_subtree(&mut session, original.document());
    let binding = session
        .snapshot()
        .document(destination.document())
        .unwrap()
        .metadata()
        .custom_element_registry();
    teardown_adoption_destination(&mut session, destination);
    assert_retired_adoption_destination(&mut session, destination, binding);
    let before = session.snapshot();
    let usage = session.dom().usage();
    let mut batch = DomBatch::new();
    batch.adopt(target, destination.document());
    assert_eq!(
        session
            .dom()
            .apply(batch, &AtomicBool::new(true))
            .unwrap_err(),
        DomError::Cancelled
    );
    assert_plain_adoption_unchanged(&before, &session.snapshot(), target, text);
    assert_eq!(session.dom().usage(), usage);
    assert_retired_adoption_destination(&mut session, destination, binding);
}

#[test]
fn canonical_adoption_active_and_never_hosted_destinations_remain_controls() {
    let mut session = session();
    let original = create(&mut session);
    let active = create(&mut session);
    let closed = {
        let mut batch = DomBatch::new();
        let document = batch.create_html_document(original.document(), None);
        session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .document(&document)
            .unwrap()
    };
    for destination in [active.document(), closed] {
        let (target, text) = plain_adoption_subtree(&mut session, original.document());
        let binding = session
            .snapshot()
            .document(destination)
            .unwrap()
            .metadata()
            .custom_element_registry();
        let mut batch = DomBatch::new();
        batch.adopt(target, destination);
        session.dom().apply(batch, &cancel()).unwrap();
        assert_adopted_plain_subtree(&session.snapshot(), target, text, destination, binding);
    }
    let snapshot = session.snapshot();
    assert!(
        snapshot
            .document(active.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        !snapshot
            .document(closed)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        snapshot
            .document(closed)
            .unwrap()
            .metadata()
            .custom_element_registry(),
        DomRegistryBinding::Null
    );
}

#[test]
fn canonical_adoption_style_resource_refusal_after_hosted_teardown_preserves_real_sheet() {
    use surgeist_cssom::CssomStore;

    let mut session = session();
    let destination = create(&mut session);
    let binding = session
        .snapshot()
        .document(destination.document())
        .unwrap()
        .metadata()
        .custom_element_registry();
    // Use the existing supported native Style environment, with a real CSSOM
    // owner. Hosted Style base/CSP services are a separate contribution.
    let (document, parent, style, text) = {
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let parent = batch.create_element_ns(&document, Namespace::html(), "div");
        batch.insert_before(&parent, &document, None);
        let style = batch.create_element_ns(&document, Namespace::html(), "style");
        let text = batch.create_text(&document, "p { color: red }");
        batch.insert_before(&text, &style, None);
        let commit = session.dom().apply(batch, &cancel()).unwrap();
        (
            commit.document(&document).unwrap(),
            commit.created(&parent).unwrap().unwrap(),
            commit.created(&style).unwrap().unwrap(),
            commit.created(&text).unwrap().unwrap(),
        )
    };
    let mut styles = session.style_execution();
    let mut source = session.source_execution();
    let mut root = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    styles
        .bind_root(
            &session.dom(),
            "adoption-refusal-root".into(),
            &mut root,
            &cancel(),
        )
        .unwrap();
    assert!(root.is_none());
    let mut policy =
        |source: &DomStyleSource| Some(DomStyleInlineCspObservation::complete_empty(source, 1));
    let progress = source
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, parent, None),
            DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut policy,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    teardown_adoption_destination(&mut session, destination);
    assert_retired_adoption_destination(&mut session, destination, binding);
    let before = session.snapshot();
    let before_style = before.node(style).unwrap().as_style().unwrap();
    assert!(before.node(style).unwrap().is_connected());
    assert!(!before_style.state().creation_state().on_parser_stack());
    assert_eq!(before_style.parser_document(), None);
    let associated = before_style.sheet().unwrap().unwrap().clone();
    let before_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(before_cssom.sheets(), std::slice::from_ref(associated.id()));
    assert_eq!(before_cssom.sheet_rules(associated.id()).unwrap().len(), 1);
    let before_owner = before_cssom
        .sheet(associated.id())
        .unwrap()
        .inputs()
        .owner
        .clone();
    assert!(before_owner.is_some());
    let mut batch = DomBatch::new();
    batch.adopt(style, destination.document());
    assert_eq!(
        session.dom().apply(batch, &cancel()).unwrap_err(),
        DomError::StyleCompositionRequired(
            surgeist_html::elements::style::HtmlStyleUpdateAction::RemoveAssociatedSheet
        )
    );
    let after = session.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(after.node(style).unwrap().parent(), Some(parent));
    assert_eq!(after.node(style).unwrap().owner_document(), Some(document));
    assert_eq!(after.node(style).unwrap().children(), &[text]);
    assert_eq!(
        after.node(parent).unwrap().children(),
        before.node(parent).unwrap().children()
    );
    assert_eq!(
        after
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap(),
        Some(&associated)
    );
    let after_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(after_cssom.revision(), before_cssom.revision());
    assert_eq!(after_cssom.sheet_list(), before_cssom.sheet_list());
    assert_eq!(after_cssom.sheets(), before_cssom.sheets());
    assert_eq!(
        after_cssom.sheet(associated.id()).unwrap().inputs().owner,
        before_owner
    );
    assert_retired_adoption_destination(&mut session, destination, binding);
}

#[test]
fn native_object_tracks_its_canonical_form_owner_after_source_creation() {
    use surgeist_html::values::HtmlValueInterpreter;
    use surgeist_html_semantics::{
        HtmlCaptureLimits, HtmlFormOwnerOrigin, HtmlSemanticCancellation, HtmlSemanticFact,
        HtmlSemanticLimits, HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
    };

    let mut session = session();
    let top = create(&mut session);
    let mut setup = DomBatch::new();
    let closed = setup.create_html_document(top.document(), None);
    let commit = session.dom().apply(setup, &cancel()).unwrap();
    let closed = commit.document(&closed).unwrap();
    let body = body(&session.snapshot(), closed);
    let mut setup = DomBatch::new();
    let nearest = setup.create_element_ns(closed, Namespace::html(), "form");
    let external = setup.create_element_ns(closed, Namespace::html(), "form");
    setup.set_attribute(&external, "id", "external");
    setup.insert_before(&nearest, body, None);
    setup.insert_before(&external, body, None);
    let commit = session.dom().apply(setup, &cancel()).unwrap();
    let [nearest, external] =
        [nearest, external].map(|form| commit.created(&form).unwrap().unwrap());

    let mut execution = session.source_execution();
    let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let mut apply = |session: &mut DomSession, edit| {
        let DomSourceDriveProgress::Complete(result) = execution
            .apply(
                &mut session.dom(),
                edit,
                limits,
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap()
        else {
            panic!("expected a complete native Object edit");
        };
        result
    };
    let object = apply(
        &mut session,
        DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "object"),
    )
    .published_node()
    .unwrap();
    apply(&mut session, DomSourceEdit::adopt(object, closed.node()));
    apply(
        &mut session,
        DomSourceEdit::insert_before(object, nearest, None),
    );

    let assert_owner = |snapshot: &DomSnapshot, expected: Option<NodeId>| {
        let capture = snapshot
            .html_capture(
                closed,
                None,
                HtmlCaptureLimits::new(128, 512, 512, 100_000, 128, 32, 32, 16_777_216),
                DomHtmlCaptureLimits::new(128, 1_000_000),
                &cancel(),
            )
            .unwrap();
        let report = capture
            .query(
                &[HtmlSemanticRequest::new(
                    capture.subject(object).unwrap(),
                    HtmlSemanticQuery::FormOwner,
                )],
                HtmlValueInterpreter::new(32_768),
                HtmlSemanticLimits::new(1_000_000, 16, 1024, 512, 1_000_000, 1024),
                HtmlSemanticCancellation::new(),
            )
            .unwrap();
        let fact = report.results()[0].fact();
        let HtmlSemanticFact::FormOwner(HtmlSemanticValue::Known(owner)) = fact else {
            panic!("Object must supply its canonical form owner: {fact:?}");
        };
        assert_eq!(owner.origin(), HtmlFormOwnerOrigin::EstablishedCurrent);
        assert_eq!(owner.owner().map(|id| capture.node(id).unwrap()), expected);
    };

    let under_nearest = session.snapshot();
    assert_owner(&under_nearest, Some(nearest));
    apply(
        &mut session,
        DomSourceEdit::set_attribute(object, "form", "external"),
    );
    let under_external = session.snapshot();
    assert_owner(&under_external, Some(external));
    apply(
        &mut session,
        DomSourceEdit::set_attribute(external, "id", "renamed"),
    );
    assert_owner(&session.snapshot(), None);
    assert_owner(&under_nearest, Some(nearest));
    assert_owner(&under_external, Some(external));
}

#[test]
fn native_object_creation_queues_its_original_task_on_an_active_host() {
    use surgeist_dom::task::DomElementTaskSourceKind;
    use surgeist_html::elements::object::HtmlObjectRepresentation;

    let mut session = session();
    let top = create(&mut session);
    let mut execution = session.source_execution();
    let tasks = session.element_task_execution();
    let source = DomElementTaskSourceKind::DomManipulation;
    assert_eq!(tasks.pending_tasks(source).unwrap(), 0);
    let mut style_policy = |_: &DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut style_policy,
        url: None,
    };
    let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let result = execution
        .apply(
            &mut session.dom(),
            DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "object"),
            limits,
            &mut services,
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(completion) = result else {
        panic!("the original task owner must admit the detached Object: {result:?}");
    };
    let object = completion.published_node().unwrap();
    let snapshot = session.snapshot();
    let node = snapshot.node(object).unwrap();
    assert!(!node.is_connected());
    assert_eq!(node.owner_document(), Some(top.document()));
    let element = node.as_element().unwrap();
    assert_eq!(
        element.object_representation(),
        Some(HtmlObjectRepresentation::Undetermined)
    );
    assert_eq!(element.child_navigable(), Some(None));
    let task = tasks.pending_task(source, 0).unwrap().unwrap();
    assert_eq!(task.id().target(), object);
    assert_eq!(task.source(), source);
    let original = task.queue_association().unwrap();
    assert_eq!(original.document(), top.document());
    assert_eq!(original.global(), top.global());
    assert_eq!(tasks.pending_tasks(source).unwrap(), 1);

    let parent = body(&snapshot, top.document());
    let mut insertion = DomBatch::new();
    insertion.set_attribute(parent, "id", "private prefix");
    insertion.insert_before(object, parent, None);
    assert_eq!(
        session.dom().apply(insertion, &cancel()).unwrap_err(),
        DomError::ObjectHostedTransitionUnavailable
    );
    assert_eq!(session.snapshot().stamp(), snapshot.stamp());

    let mut fallback = DomBatch::new();
    let closed = fallback.create_html_document(top.document(), None);
    let embed = fallback.create_element_ns(top.document(), Namespace::html(), "embed");
    fallback.insert_before(&embed, object, None);
    let commit = session.dom().apply(fallback, &cancel()).unwrap();
    let embed = commit.created(&embed).unwrap().unwrap();
    let closed = commit.document(&closed).unwrap();
    let closed_body = body(&session.snapshot(), closed);
    for edit in [
        DomSourceEdit::adopt(object, closed.node()),
        DomSourceEdit::insert_before(object, closed_body, None),
    ] {
        assert!(matches!(
            execution
                .apply(&mut session.dom(), edit, limits, &mut services, &cancel())
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
    }
    let before_task = session.snapshot();
    assert_eq!(
        before_task.node(object).unwrap().owner_document(),
        Some(closed)
    );
    assert!(before_task.node(object).unwrap().is_connected());
    assert_eq!(tasks.pending_tasks(source).unwrap(), 3);

    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let mut context = Context {
        element_tasks: tasks.clone(),
        seen: Vec::new(),
    };
    let no_work = DomSourceLimits::new(16_000_000, 1_000_000, 0, 128, 4096, 4096, 1);
    let refused = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut dispatcher,
        &mut context,
        DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: no_work,
        },
        &cancel(),
    )
    .unwrap_err();
    assert!(matches!(
        refused,
        DomElementTaskRuntimeError::Dom(DomError::SourceWorkLimit { limit: 0 })
    ));
    assert_eq!(session.snapshot().stamp(), before_task.stamp());
    assert_eq!(
        tasks.pending_task(source, 0).unwrap().unwrap().id(),
        task.id()
    );
    let no_storage = DomSourceLimits::new(0, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let refused = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut dispatcher,
        &mut context,
        DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: no_storage,
        },
        &cancel(),
    )
    .unwrap_err();
    assert!(matches!(
        refused,
        DomElementTaskRuntimeError::Dom(DomError::ResourceLimit {
            resource: surgeist_dom::limits::DomResource::SourceRetainedBytes,
            limit: 0,
        })
    ));
    assert_eq!(session.snapshot().stamp(), before_task.stamp());
    assert_eq!(
        tasks.pending_task(source, 0).unwrap().unwrap().id(),
        task.id()
    );
    let outcome = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut dispatcher,
        &mut context,
        DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: limits,
        },
        &cancel(),
    )
    .unwrap();
    assert!(matches!(outcome,
        DomElementTaskProgress::ObjectRepresentationComplete { task: info, displaced_owner: None }
            if info.id() == task.id() && info.queued_node_document() == top.document()
    ));
    assert!(context.seen.is_empty());
    assert_eq!(tasks.pending_tasks(source).unwrap(), 2);
    let current = session.snapshot();
    let element = current.node(object).unwrap().as_element().unwrap();
    assert_eq!(
        element.object_representation(),
        Some(HtmlObjectRepresentation::FallbackContent)
    );
    assert_eq!(element.child_navigable(), Some(None));
    assert!(
        current
            .node(embed)
            .unwrap()
            .as_element()
            .unwrap()
            .is_inactive_embed()
    );
    assert_eq!(
        snapshot
            .node(object)
            .unwrap()
            .as_element()
            .unwrap()
            .object_representation(),
        Some(HtmlObjectRepresentation::Undetermined)
    );
}

#[test]
fn retiring_object_cannot_retarget_its_queued_task() {
    use surgeist_dom::task::DomElementTaskSourceKind;

    let mut session = session();
    let top = create(&mut session);
    let mut runtime = session.source_execution();
    let tasks = session.element_task_execution();
    let source = DomElementTaskSourceKind::DomManipulation;
    let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let result = runtime
        .apply(
            &mut session.dom(),
            DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "object"),
            limits,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(result) = result else {
        panic!("expected original Object creation");
    };
    let object = result.published_node().unwrap();
    let before = session.snapshot();
    let queued = tasks.pending_task(source, 0).unwrap().unwrap();
    let mut retire = DomBatch::new();
    retire.destroy(object);
    session.dom().apply(retire, &cancel()).unwrap();
    let mut replacement = DomBatch::new();
    let node = replacement.create_element(top.document(), "span");
    let node = session
        .dom()
        .apply(replacement, &cancel())
        .unwrap()
        .created(&node)
        .unwrap()
        .unwrap();
    assert_ne!(node, object);
    assert!(matches!(
        session.snapshot().node(object),
        Err(DomError::StaleNode)
    ));
    let mut context = Context {
        element_tasks: tasks.clone(),
        seen: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let outcome = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut dispatcher,
        &mut context,
        DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: limits,
        },
        &cancel(),
    )
    .unwrap();
    assert!(matches!(outcome,
        DomElementTaskProgress::SkippedTargetDestroyed(task) if task.id() == queued.id()));
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    assert!(context.seen.is_empty());
    assert!(
        before
            .node(object)
            .unwrap()
            .as_element()
            .unwrap()
            .object_representation()
            .is_some()
    );
    assert!(
        session
            .snapshot()
            .node(node)
            .unwrap()
            .as_element()
            .unwrap()
            .object_representation()
            .is_none()
    );
}

#[test]
fn text_selection_uses_atomic_noncoalesced_user_interaction_tasks() {
    use surgeist_dom::task::DomElementTaskSourceKind::UserInteraction;
    use surgeist_html::elements::input::{
        HtmlInputSelectionDirection as Direction, HtmlTextSelectionEdit as Edit,
    };
    use surgeist_runtime::source::{DomSourceDriveError, DomSourceRuntimeError};

    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (control, textarea) = {
        let mut batch = DomBatch::new();
        let control = batch.create_element_ns(top.document(), Namespace::html(), "input");
        let textarea = batch.create_element_ns(top.document(), Namespace::html(), "textarea");
        batch.set_attribute(&control, "value", "abcdef");
        batch.insert_before(&control, parent, None);
        batch.insert_before(&textarea, parent, None);
        let commit = session.dom().apply(batch, &cancel()).unwrap();
        (
            commit.created(&control).unwrap().unwrap(),
            commit.created(&textarea).unwrap().unwrap(),
        )
    };
    let mut execution = session.source_execution();
    let mut tasks = session.element_task_execution();
    let task_limits = limits().element_tasks;
    tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 0,
            ..task_limits
        })
        .unwrap();
    let mut style_policy = |_: &DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut style_policy,
        url: None,
    };
    let source = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    for (node, edit) in [(control, Edit::End(None)), (textarea, Edit::All)] {
        let result = execution
            .apply(
                &mut session.dom(),
                DomSourceEdit::edit_text_selection(node, edit),
                source,
                &mut services,
                &cancel(),
            )
            .expect("an empty-range representation change does not need a task slot");
        assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    }
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    let before = session.snapshot();
    let range = Edit::Range {
        start: 1,
        end: 3,
        direction: Direction::Backward,
    };
    assert!(matches!(
        execution.apply(
            &mut session.dom(),
            DomSourceEdit::edit_text_selection(control, range),
            source,
            &mut services,
            &cancel(),
        ),
        Err(DomSourceDriveError::Source(
            DomSourceRuntimeError::ElementTasks(DomElementTaskRuntimeError::Admission)
        )),
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    let operation = session.dom().pending_source_edit().unwrap().operation();
    tasks.set_limits(task_limits).unwrap();
    let DomSourceDriveProgress::Complete(completion) = execution
        .resume(
            &mut session.dom(),
            operation,
            source,
            &mut services,
            &cancel(),
        )
        .unwrap()
    else {
        panic!("the original selection must complete with its admitted task")
    };
    assert_eq!(completion.operation(), operation);
    assert_eq!(
        session
            .snapshot()
            .node(control)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .selection()
            .unwrap()
            .selection_range(),
        Some((1, 3, Direction::Backward)),
    );
    let after_first = session.snapshot().stamp();
    tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 1,
            ..task_limits
        })
        .unwrap();
    assert!(matches!(
        execution.apply(
            &mut session.dom(),
            DomSourceEdit::edit_text_selection(control, Edit::Direction(Direction::Forward)),
            source,
            &mut services,
            &cancel(),
        ),
        Err(DomSourceDriveError::Source(
            DomSourceRuntimeError::ElementTasks(DomElementTaskRuntimeError::Admission)
        )),
    ));
    let abandoned = session.dom().pending_source_edit().unwrap().operation();
    execution
        .cancel_pending_element_task_admission(&mut session.dom(), abandoned)
        .unwrap();
    assert_eq!(session.snapshot().stamp(), after_first);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    tasks.set_limits(task_limits).unwrap();
    for edit in [range, Edit::Direction(Direction::Forward)] {
        let result = execution
            .apply(
                &mut session.dom(),
                DomSourceEdit::edit_text_selection(control, edit),
                source,
                &mut services,
                &cancel(),
            )
            .unwrap();
        assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
        if edit == range {
            assert_eq!(session.snapshot().stamp(), after_first);
            assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
        }
    }
    let first = tasks.pending_task(UserInteraction, 0).unwrap().unwrap();
    let second = tasks.pending_task(UserInteraction, 1).unwrap().unwrap();
    assert_ne!(first.id(), second.id());
    assert_eq!(first.id().target(), control);
    assert_eq!(first.queue_association().unwrap().global(), top.global());
    assert_eq!(second.queue_association(), first.queue_association());
    assert_pending_text_selection_dispatch(
        &mut session,
        tasks,
        parent,
        control,
        [first, second],
        source,
    );
}

fn assert_pending_text_selection_dispatch(
    session: &mut DomSession,
    tasks: DomElementTaskRuntime,
    parent: NodeId,
    control: NodeId,
    queued: [surgeist_runtime::element_task::DomElementTask; 2],
    source: DomSourceLimits,
) {
    use surgeist_dom::task::DomElementTaskSourceKind::UserInteraction;

    let mut registry = registry();
    let mut context = Context {
        element_tasks: tasks.clone(),
        seen: Vec::new(),
    };
    for (node, label) in [(control, "control"), (parent, "ancestor")] {
        registry
            .add_listener(
                &mut session.dom(),
                DomEventTarget::Node(node),
                DomEventType::SELECT,
                DomEventCallback::new(move |_, _, context: &mut Context, event| {
                    assert!(event.is_trusted());
                    assert!(event.bubbles());
                    assert!(!event.cancelable());
                    assert!(!event.composed());
                    assert_eq!(event.target(), Some(DomEventTarget::Node(control)));
                    assert_eq!(event.current_target(), Some(DomEventTarget::Node(node)));
                    context.seen.push(Observation {
                        label,
                        current: event.current_target(),
                        target: event.target().and_then(|target| target.node()),
                        phase: event.phase(),
                        payload: event.toggle_event().cloned(),
                        bubbles: event.bubbles(),
                    });
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: false,
                    passive: None,
                },
            )
            .unwrap();
    }
    let mut change = DomBatch::new();
    change.set_attribute(control, "type", "number");
    session.dom().apply(change, &cancel()).unwrap();
    assert!(
        session
            .snapshot()
            .node(control)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .selection()
            .is_none()
    );
    assert!(context.seen.is_empty());
    for occurrence in queued {
        let run = DomElementTaskRuntime::run_next(
            &mut session.dom(),
            UserInteraction,
            &mut registry,
            &mut context,
            DomElementTaskRunLimits {
                dispatch: event_limits(),
                source,
            },
            &cancel(),
        )
        .unwrap();
        assert!(matches!(run, DomElementTaskProgress::Complete { task, .. } if task == occurrence));
    }
    assert_eq!(
        context
            .seen
            .iter()
            .map(|event| (event.label, event.phase))
            .collect::<Vec<_>>(),
        [
            ("control", DomEventPhase::AtTarget),
            ("ancestor", DomEventPhase::Bubbling),
            ("control", DomEventPhase::AtTarget),
            ("ancestor", DomEventPhase::Bubbling),
        ],
    );
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
}

#[test]
fn select_listener_queues_an_original_range_edit_and_retirement_skips_its_task() {
    use std::cell::RefCell;
    use surgeist_dom::task::DomElementTaskSourceKind::UserInteraction;
    use surgeist_html::elements::input::{
        HtmlRangeTextMode as Mode, HtmlRangeTextTarget as Target, HtmlTextSelectionEdit as Edit,
    };

    let mut session = session();
    let top = create(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let mut batch = DomBatch::new();
    let control = batch.create_element_ns(top.document(), Namespace::html(), "input");
    batch.set_attribute(&control, "value", "abcd");
    batch.insert_before(&control, parent, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let control = commit.created(&control).unwrap().unwrap();
    let tasks = session.element_task_execution();
    let execution = RefCell::new(session.source_execution());
    let source = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let apply = move |dom: &mut DomEventDom<'_>, edit| {
        let mut style_policy = |_: &DomStyleSource| None;
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut style_policy,
            url: None,
        };
        assert!(matches!(
            execution
                .borrow_mut()
                .apply(dom, edit, source, &mut services, &cancel())
                .unwrap(),
            DomSourceDriveProgress::Complete(_),
        ));
    };
    apply(
        &mut session.dom(),
        DomSourceEdit::edit_text_selection(control, Edit::All),
    );
    let first = tasks.pending_task(UserInteraction, 0).unwrap().unwrap();
    let mut registry = registry();
    registry
        .add_listener(
            &mut session.dom(),
            DomEventTarget::Node(control),
            DomEventType::SELECT,
            DomEventCallback::new(move |dom, _, context: &mut Context, event| {
                assert!(context.seen.is_empty());
                assert!(event.is_trusted());
                apply(
                    dom,
                    DomSourceEdit::set_range_text(
                        control,
                        "",
                        Target::Offsets {
                            start: 2,
                            end: 4,
                            mode: Mode::Preserve,
                        },
                    ),
                );
                apply(
                    dom,
                    DomSourceEdit::set_range_text(
                        control,
                        "Z",
                        Target::Offsets {
                            start: 0,
                            end: 1,
                            mode: Mode::Select,
                        },
                    ),
                );
                context.seen.push(Observation {
                    label: "selection",
                    current: event.current_target(),
                    target: event.target().and_then(|target| target.node()),
                    phase: event.phase(),
                    payload: None,
                    bubbles: event.bubbles(),
                });
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
                passive: None,
            },
        )
        .unwrap();
    let mut context = Context {
        element_tasks: tasks.clone(),
        seen: Vec::new(),
    };
    let mut run = |session: &mut DomSession, context: &mut Context| {
        DomElementTaskRuntime::run_next(
            &mut session.dom(),
            UserInteraction,
            &mut registry,
            context,
            DomElementTaskRunLimits {
                dispatch: event_limits(),
                source,
            },
            &cancel(),
        )
        .unwrap()
    };
    assert!(
        matches!(run(&mut session, &mut context), DomElementTaskProgress::Complete { task, .. } if task == first)
    );
    let historical = session.snapshot();
    assert_eq!(
        historical
            .node(control)
            .unwrap()
            .as_input()
            .unwrap()
            .value(),
        [90, 98]
    );
    // The preserve edit's final range already matched the value-change
    // clamp. Only the second edit changed selection in the explicit step.
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    let followup = tasks.pending_task(UserInteraction, 0).unwrap().unwrap();
    assert_ne!(followup.id(), first.id());
    assert_eq!(followup.id().target(), control);
    let mut retire = DomBatch::new();
    retire.detach(control);
    retire.destroy(control);
    session.dom().apply(retire, &cancel()).unwrap();
    assert!(matches!(
        run(&mut session, &mut context),
        DomElementTaskProgress::SkippedTargetDestroyed(task) if task == followup,
    ));
    assert_eq!(context.seen.len(), 1);
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    assert!(historical.node(control).is_ok());
}

#[test]
fn a_full_element_task_bank_refuses_object_before_publishing_and_retries_the_original_edit() {
    use surgeist_runtime::source::{DomSourceDriveError, DomSourceRuntimeError};

    let mut session = session();
    let top = create(&mut session);
    let mut execution = session.source_execution();
    let mut tasks = session.element_task_execution();
    let task_limits = limits().element_tasks;
    tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 0,
            ..task_limits
        })
        .unwrap();
    let before = session.snapshot();
    let mut style_policy = |_: &DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut style_policy,
        url: None,
    };
    let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let refusal = execution
        .apply(
            &mut session.dom(),
            DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "object"),
            limits,
            &mut services,
            &cancel(),
        )
        .unwrap_err();
    assert!(matches!(
        refusal,
        DomSourceDriveError::Source(DomSourceRuntimeError::ElementTasks(
            DomElementTaskRuntimeError::Admission
        ))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
    let original = session.dom().pending_source_edit().unwrap().operation();
    tasks.set_limits(task_limits).unwrap();
    let result = execution
        .resume(
            &mut session.dom(),
            original,
            limits,
            &mut services,
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(result) = result else {
        panic!("the original operation must now complete");
    };
    assert_eq!(result.operation(), original);
    assert!(result.published_node().is_some());
    assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
    assert!(session.dom().pending_source_edit().is_none());

    tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 1,
            ..task_limits
        })
        .unwrap();
    let before_second = session.snapshot();
    assert!(matches!(
        execution.apply(
            &mut session.dom(),
            DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "object"),
            limits,
            &mut services,
            &cancel(),
        ),
        Err(DomSourceDriveError::Source(
            DomSourceRuntimeError::ElementTasks(DomElementTaskRuntimeError::Admission)
        ))
    ));
    let unpublished = session.dom().pending_source_edit().unwrap().operation();
    execution
        .cancel_pending_element_task_admission(&mut session.dom(), unpublished)
        .unwrap();
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(session.snapshot().stamp(), before_second.stamp());
    let source = surgeist_dom::task::DomElementTaskSourceKind::DomManipulation;
    let first_task = tasks.pending_task(source, 0).unwrap().unwrap();
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 4,
        nested_dispatches: 4,
        storage_bytes: 16_000_000,
        work: 1_000_000,
    })
    .unwrap();
    let mut context = Context {
        element_tasks: tasks.clone(),
        seen: Vec::new(),
    };
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        source,
        &mut dispatcher,
        &mut context,
        DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: limits,
        },
        &cancel(),
    )
    .unwrap();
    assert!(
        matches!(progress, DomElementTaskProgress::ObjectRepresentationComplete { task, .. }
        if task.id() == first_task.id())
    );
    assert_eq!(tasks.pending_total_tasks().unwrap(), 0);
}

#[test]
fn document_activity_refusal_follows_the_adopted_object_even_with_a_foreign_task_global() {
    use surgeist_dom::task::DomElementTaskSourceKind;

    let mut session = session();
    let from = create(&mut session);
    let to = create(&mut session);
    let mut native = session.source_execution();
    let tasks = session.element_task_execution();
    let source = DomElementTaskSourceKind::DomManipulation;
    let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let mut apply = |session: &mut DomSession, edit| {
        let DomSourceDriveProgress::Complete(result) = native
            .apply(
                &mut session.dom(),
                edit,
                limits,
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            )
            .unwrap()
        else {
            panic!("expected one complete native Source edit")
        };
        result
    };
    let object = apply(
        &mut session,
        DomSourceEdit::create_element_ns(from.document().node(), Namespace::html(), "object"),
    )
    .published_node()
    .unwrap();
    apply(
        &mut session,
        DomSourceEdit::adopt(object, to.document().node()),
    );
    let before = session.snapshot();
    assert_eq!(
        before.node(object).unwrap().owner_document(),
        Some(to.document())
    );
    assert_eq!(tasks.pending_tasks(source).unwrap(), 1);
    let original = tasks.pending_task(source, 0).unwrap().unwrap();
    assert_eq!(
        original.queue_association().unwrap().global(),
        from.global()
    );
    let mut hosted = session.hosted_execution();
    assert!(matches!(
        hosted.destroy_top_level(&mut session.dom(), to.navigable(), &cancel()),
        Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
            DomError::ObjectHostedTransitionUnavailable
        )))
    ));
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        tasks.pending_task(source, 0).unwrap().unwrap().id(),
        original.id()
    );

    let mut retire = DomBatch::new();
    retire.destroy(object);
    session.dom().apply(retire, &cancel()).unwrap();
    hosted
        .destroy_top_level(&mut session.dom(), to.navigable(), &cancel())
        .unwrap();
}

#[test]
fn object_attribute_and_clone_occurrences_follow_source_precedence_and_fifo() {
    use surgeist_dom::task::DomElementTaskSourceKind;
    use surgeist_html::elements::object::HtmlObjectRepresentation;

    let mut session = session();
    let top = create(&mut session);
    let mut setup = DomBatch::new();
    let details = setup.create_element_ns(top.document(), Namespace::html(), "details");
    setup.set_attribute(&details, "open", "");
    let details = session
        .dom()
        .apply(setup, &cancel())
        .unwrap()
        .created(&details)
        .unwrap()
        .unwrap();
    let mut native = session.source_execution();
    let tasks = session.element_task_execution();
    let source = DomElementTaskSourceKind::DomManipulation;
    let mut style_policy = |_: &DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut style_policy,
        url: None,
    };
    let limits = DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1);
    let mut apply = |session: &mut DomSession, edit: DomSourceEdit| {
        let result = native
            .apply(&mut session.dom(), edit, limits, &mut services, &cancel())
            .unwrap();
        let DomSourceDriveProgress::Complete(result) = result else {
            panic!("expected complete native Object operation");
        };
        result
    };
    let object = apply(
        &mut session,
        DomSourceEdit::create_element_ns(top.document().node(), Namespace::html(), "object"),
    )
    .published_node()
    .unwrap();
    assert_eq!(tasks.pending_tasks(source).unwrap(), 2);
    assert_eq!(
        tasks
            .pending_task(source, 0)
            .unwrap()
            .unwrap()
            .id()
            .target(),
        details
    );
    assert_eq!(
        tasks
            .pending_task(source, 1)
            .unwrap()
            .unwrap()
            .id()
            .target(),
        object
    );
    assert_eq!(session.dom().pending_details_toggle_intentions(), 0);
    for (edit, after) in [
        (
            DomSourceEdit::set_native_attribute(object, "type", "image/png"),
            3,
        ),
        (DomSourceEdit::set_native_attribute(object, "data", ""), 4),
        (
            DomSourceEdit::set_native_attribute(object, "type", "image/svg+xml"),
            4,
        ),
        (
            DomSourceEdit::set_native_attribute(object, "classid", ""),
            5,
        ),
        (
            DomSourceEdit::set_native_attribute(object, "classid", ""),
            6,
        ),
        (
            DomSourceEdit::set_native_attribute(object, "data", "raw"),
            6,
        ),
        (DomSourceEdit::remove_attribute(object, "classid"), 7),
        (
            DomSourceEdit::set_native_attribute_ns(
                object,
                Namespace::from_uri("urn:other"),
                "p:classid",
                "",
            ),
            7,
        ),
    ] {
        apply(&mut session, edit);
        assert_eq!(tasks.pending_tasks(source).unwrap(), after);
    }
    let cloned = apply(&mut session, DomSourceEdit::clone_node(object, false))
        .published_node()
        .unwrap();
    assert_ne!(cloned, object);
    assert_eq!(tasks.pending_tasks(source).unwrap(), 10);
    for ordinal in 7..10 {
        assert_eq!(
            tasks
                .pending_task(source, ordinal)
                .unwrap()
                .unwrap()
                .id()
                .target(),
            cloned
        );
    }
    let mut distinct = Vec::new();
    for ordinal in 0..10 {
        let id = tasks.pending_task(source, ordinal).unwrap().unwrap().id();
        assert!(!distinct.contains(&id));
        distinct.push(id);
    }
    let snapshot = session.snapshot();
    let cloned = snapshot.node(cloned).unwrap();
    assert_eq!(cloned.attribute(&"data".into()), Some(&"raw".into()));
    assert_eq!(
        cloned.as_element().unwrap().object_representation(),
        Some(HtmlObjectRepresentation::Undetermined)
    );

    let mut create = DomBatch::new();
    let inactive = create.create_html_document(top.document(), None);
    let inactive = session
        .dom()
        .apply(create, &cancel())
        .unwrap()
        .document(&inactive)
        .unwrap();
    apply(
        &mut session,
        DomSourceEdit::adopt(cloned.id(), inactive.node()),
    );
    let capture = session.snapshot();
    let adopted = capture.node(cloned.id()).unwrap();
    assert_eq!(adopted.owner_document(), Some(inactive));
    assert_eq!(adopted.as_element().unwrap().child_navigable(), Some(None));
    assert_eq!(cloned.owner_document(), Some(top.document()));
    assert_eq!(tasks.pending_tasks(source).unwrap(), 11);
    let custody = tasks.pending_task(source, 10).unwrap().unwrap();
    let surgeist_runtime::element_task::DomElementTask::ObjectRepresentation(info) = custody else {
        panic!("the change of Document activity must queue its own task");
    };
    assert_eq!(info.id().target(), cloned.id());
    assert_eq!(info.queue_association().global(), top.global());
    assert_eq!(info.queue_association().document(), top.document());
    assert_eq!(info.queued_node_document(), inactive);

    let closed_body = body(&capture, inactive);
    apply(
        &mut session,
        DomSourceEdit::insert_before(cloned.id(), closed_body, None),
    );
    assert!(session.snapshot().node(cloned.id()).unwrap().is_connected());
    assert_eq!(tasks.pending_tasks(source).unwrap(), 12);
    apply(&mut session, DomSourceEdit::detach(cloned.id()));
    assert!(!session.snapshot().node(cloned.id()).unwrap().is_connected());
    assert_eq!(tasks.pending_tasks(source).unwrap(), 13);
    for ordinal in 11..13 {
        let task = tasks.pending_task(source, ordinal).unwrap().unwrap();
        let surgeist_runtime::element_task::DomElementTask::ObjectRepresentation(info) = task
        else {
            panic!("the ordinary closed-Document tree hook queues Object work");
        };
        assert_eq!(info.id().target(), cloned.id());
        assert_eq!(info.queued_node_document(), inactive);
        assert_eq!(info.queue_association().global(), top.global());
    }
}
