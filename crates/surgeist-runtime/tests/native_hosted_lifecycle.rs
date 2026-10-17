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
use surgeist_css::CssUtf16InputLimits;
use surgeist_dom::*;
use surgeist_html::HtmlDocumentMode;
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomSessionLimits {
    DomSessionLimits {
        source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
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
        DomLimits::new(512, 200_000, 1024, 1024, 1024, 64, 4_000_000)
            .unwrap()
            .with_dispatch_custody(
                surgeist_dom::DomDispatchCustodyLimits::new(
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
    let root = snapshot.node(document.node()).unwrap().children()[0];
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
    })
    .unwrap()
}
fn listen(
    registry: &mut DomEventDispatcher<Context>,
    dom: &DomEventDom<'_>,
    target: DomEventTarget,
    label: &'static str,
    capture: bool,
) {
    registry
        .add_listener(
            dom,
            target,
            DomEventType::Toggle,
            DomEventCallback::new(move |_, _, context: &mut Context, event| {
                context.seen.push(Observation {
                    label,
                    current: event.current_target(),
                    target: event.target(),
                    phase: event.phase(),
                    payload: event.toggle_event().copied(),
                    bubbles: event.bubbles(),
                });
                Ok(())
            }),
            DomEventListenerOptions {
                capture,
                once: false,
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
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        registry,
        context,
        crate::DomElementTaskRunLimits {
            dispatch: event_limits(),
            source: surgeist_dom::DomSourceLimits::new(
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
        html.as_element().unwrap().name().namespace(),
        &Namespace::html()
    );
    assert_eq!(html.children().len(), 2);
    for (node, local) in html.children().iter().zip(["head", "body"]) {
        let node = snapshot.node(*node).unwrap();
        assert_eq!(
            node.as_element().unwrap().name().local_name(),
            &local.into()
        );
        assert_eq!(
            node.as_element().unwrap().name().namespace(),
            &Namespace::html()
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
            surgeist_html_semantics::HtmlCaptureLimits::new(128, 512, 512, 100_000, 128, 32, 32),
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
        surgeist_html::HtmlBrowsingContextPresence::Present
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
        &surgeist_html::DocumentString::from("about:blank")
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
            listen(&mut registry, &dom, recipient, label, capture);
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
        let payload = observation.payload.unwrap();
        assert_eq!(payload.old_state(), "closed");
        assert_eq!(payload.new_state(), "open");
    }
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
            &dom,
            DomEventTarget::Global(first.global()),
            "original-global",
            true,
        );
        listen(
            &mut registry,
            &dom,
            DomEventTarget::Global(second.global()),
            "current-global",
            true,
        );
        listen(
            &mut registry,
            &dom,
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
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
                .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
                .unwrap()
                .unwrap()
                .id(),
            queued.id()
        );
        assert_eq!(
            context
                .element_tasks
                .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
        DomLimits::new(128, 20_000, 128, 128, 128, 16, 1_000_000)
            .unwrap()
            .with_dispatch_custody(
                surgeist_dom::DomDispatchCustodyLimits::new(
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
            listen(&mut registry, &dom, recipient, label, true);
        }
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(first.document(), Namespace::html(), "details");
        batch.insert_before(&details, first_body, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        listen(
            &mut registry,
            &dom,
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
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
                .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
                surgeist_dom::DomElementTaskSourceKind::DomManipulation,
                &mut registry,
                &mut context,
                crate::DomElementTaskRunLimits {
                    dispatch: event_limits(),
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
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
    assert!(
        current
            .as_element()
            .unwrap()
            .attribute(&"open".into())
            .is_some()
    );
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
            &dom,
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
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
            &dom,
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
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
                surgeist_dom::DomElementTaskSourceKind::DomManipulation,
                &mut registry,
                &mut context,
                crate::DomElementTaskRunLimits {
                    dispatch: event_limits(),
                    source: surgeist_dom::DomSourceLimits::new(
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
        assert_eq!(context.seen[0].payload.unwrap().new_state(), "open");
        let retained = context
            .element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
        surgeist_html::HtmlBrowsingContextPresence::Absent
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
        surgeist_html::HtmlBrowsingContextPresence::Absent
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
            commit.created(&shadow).unwrap().unwrap(),
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
            surgeist_html_semantics::HtmlCaptureLimits::new(128, 512, 512, 100_000, 128, 32, 32),
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
    use surgeist_html::{HtmlElementClass, KnownHtmlElement};
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
        // The selected native hosted profile refuses direct initialization; it
        // does not manufacture closed-environment family evidence in this Doc.
        let before = session.snapshot();
        {
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
        // A genuine closed creator can initialize the native node. Adoption
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
                .as_element()
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
        surgeist_html::HtmlBrowsingContextPresence::Present
    );
    assert_eq!(
        environment.scripting(),
        surgeist_html_semantics::HtmlScriptingStatus::Disabled
    );
    // HTML877d prepare-the-script steps9,15,16 precede the explicit step18
    // disabled-scripting return; an actual BC does not bypass those writes.
    let state = snapshot.node(script).unwrap().as_html_script().unwrap();
    assert_eq!(
        state.script_type(),
        Some(surgeist_html::HtmlScriptKind::Classic)
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
        surgeist_html::HtmlScriptResult::Uninitialized
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
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
            listen(&mut registry, &dom, recipient, label, capture);
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
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    let retained = context
        .element_tasks
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(retained.id(), queued.id());
    assert_eq!(retained.queue_association(), queued.queue_association());
    // DOM's null-BC parent branch cannot supply fresh closed-native authority:
    // this current node Document has an actual retired Global. The bounded
    // native path must qualify it before invoking Rust callbacks (innerinvoke).
    {
        let mut dom = session.dom();
        assert!(matches!(
            DomElementTaskRuntime::run_next(
                &mut dom,
                surgeist_dom::DomElementTaskSourceKind::DomManipulation,
                &mut registry,
                &mut context,
                crate::DomElementTaskRunLimits {
                    dispatch: event_limits(),
                    source: surgeist_dom::DomSourceLimits::new(
                        16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                    )
                },
                &cancel()
            ),
            Err(DomElementTaskRuntimeError::Dispatch(
                DomEventDispatchError::Hosted(DomHostedError::StaleIdentity)
            ))
        ));
    }
    assert!(context.seen.is_empty());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(
        context
            .element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap()
            .id(),
        queued.id()
    );
    assert_eq!(
        context
            .element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
                &dom,
                DomEventTarget::Global(second.global()),
                DomEventType::Toggle,
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
                    &dom,
                    recipient,
                    DomEventType::Toggle,
                    DomEventCallback::new(move |_, _, context: &mut RetireContext, _| {
                        context.seen.push(label);
                        Ok(())
                    }),
                    DomEventListenerOptions {
                        capture,
                        once: false,
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
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::DomSourceLimits::new(
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
    assert_eq!(context.observed[0].target(), Some(target));
    assert_eq!(
        context.observed[0].toggle_event().unwrap().new_state(),
        "open"
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
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::DomSourceLimits::new(
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
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
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
    })
    .unwrap();
    let target_once = {
        let dom = session.dom();
        registry
            .add_listener(
                &dom,
                DomEventTarget::Global(second.global()),
                DomEventType::Toggle,
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
                },
            )
            .unwrap();
        registry
            .add_listener(
                &dom,
                DomEventTarget::Node(target),
                DomEventType::Toggle,
                DomEventCallback::new(|_, _, context: &mut DestroyTargetContext, _| {
                    context.seen.push("destroyed-target");
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
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
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(still_queued.id(), queued.id());
    assert_eq!(still_queued.queue_association(), queued.queue_association());
    let progress = {
        let mut dom = session.dom();
        DomElementTaskRuntime::run_next(
            &mut dom,
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::DomSourceLimits::new(
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
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: event_limits(),
                source: surgeist_dom::DomSourceLimits::new(
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
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
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
        panic!("the genuine hosted destination retains its nonnull registry");
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
    assert_eq!(state.name().namespace(), &Namespace::html());
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
            surgeist_html::HtmlStyleUpdateAction::RemoveAssociatedSheet
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
