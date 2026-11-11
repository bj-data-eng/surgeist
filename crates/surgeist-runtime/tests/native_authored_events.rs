//! Actual hosted construction, exact UTF-16 registration and ordinary Node
//! dispatch share the selected native kernel and its affine custody.
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

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}
fn session() -> DomSession {
    session_with_event_capacity(64, 128)
}
fn session_with_event_capacity(node_scopes: usize, recipient_slots: usize) -> DomSession {
    let dom = Dom::new(
        DomLimits::new(
            512,
            200_000,
            1024,
            1024,
            1024,
            64,
            8_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_event_recipients(
            DomEventRecipientLimits::new(recipient_slots, 16_000_000, 16_000_000, 4_000_000)
                .unwrap(),
        )
        .with_dispatch_custody(
            DomDispatchCustodyLimits::new(node_scopes, 4096, 16_000_000, 16_000_000, 4_000_000)
                .unwrap(),
        )
        .with_details_toggle_intentions(32)
        .unwrap(),
    )
    .unwrap();
    DomSession::new(
        dom,
        DomSessionLimits {
            authored_events: DomEventStorageLimits {
                storage_bytes: 16_000_000,
                work: 4_000_000,
            },
            source_events: DomSourceEventRuntimeLimits {
                jobs: 32,
                storage_bytes: 16_000_000,
                work: 4_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 32,
                reports: 32,
                storage_bytes: 16_000_000,
                work: 4_000_000,
                css_input: surgeist_css::source::CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 16_000_000,
                work: 4_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    16_000_000,
                    16_000_000,
                    4_000_000,
                ),
                tasks: 32,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 4_000_000,
            },
        },
    )
    .unwrap()
}
fn top(session: &mut DomSession) -> DomHostedTopLevel {
    let mut hosted = session.hosted_execution();
    let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
    hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap()
}
fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let root = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    snapshot.node(root).unwrap().children()[1]
}
fn new_registry<C>() -> DomEventDispatcher<C> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 128,
        pending_events: 16,
        nested_dispatches: 8,
        storage_bytes: 16_000_000,
        work: 4_000_000,
    })
    .unwrap()
}
fn limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 128,
        snapshot_listeners: 128,
        failure_records: 128,
        callbacks: 128,
        work: 1_000_000,
    }
}
fn complete(outcome: DomEventDispatchOutcome) -> DomEventDispatchReceipt {
    let DomEventDispatchOutcome::Complete(receipt) = outcome else {
        panic!("actual completion required: {outcome:?}")
    };
    receipt
}
fn fire<C>(
    session: &mut DomSession,
    registry: &mut DomEventDispatcher<C>,
    target: NodeId,
    event: &NativeDomEvent,
    context: &mut C,
) -> DomEventDispatchReceipt {
    let mut invocation = registry
        .prepare_dispatch(&mut session.dom(), target, event, &cancel())
        .unwrap();
    complete(
        invocation
            .advance(&mut session.dom(), registry, context, limits(), &cancel())
            .unwrap(),
    )
}

#[test]
fn actual_constructor_and_registry_preserve_every_utf16_type_exactly() {
    let mut session = session();
    let top = top(&mut session);
    let target = body(&session.snapshot(), top.document());
    let mut hosted = session.hosted_execution();
    let settings = hosted
        .settings(&session.dom(), top.global(), &cancel())
        .unwrap();
    let mut registry = new_registry::<Vec<usize>>();
    let names = [
        Vec::new(),
        vec![0xd800],
        vec![0xdc00, 0, 0xd800],
        units("load"),
        units("Load"),
        units("toggle"),
        units("click"),
    ];
    for (index, name) in names.into_iter().enumerate() {
        let mut other = name.clone();
        other.push(33);
        registry
            .add_listener(
                &mut session.dom(),
                target,
                &name,
                DomEventCallback::new(move |_, _, seen: &mut Vec<usize>, _| {
                    seen.push(index);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        registry
            .add_listener(
                &mut session.dom(),
                target,
                &other,
                DomEventCallback::new(move |_, _, seen: &mut Vec<usize>, _| {
                    seen.push(1000 + index);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let event = hosted
            .create_event(
                &mut session.dom(),
                top.global(),
                &name,
                DomEventInit::default(),
                &cancel(),
            )
            .unwrap();
        assert_eq!(event.event_type().code_units(), name);
        assert!(!event.is_trusted());
        assert!(event.toggle_event().is_none());
        assert_eq!(event.target(), None);
        let creation = event.creation().unwrap();
        assert_eq!(creation.global(), top.global());
        assert_eq!(creation.clock_source(), settings.clock_source());
        assert_eq!(
            creation.time_origin_microseconds(),
            settings.time_origin_microseconds()
        );
        assert_eq!(creation.occurrence_microseconds() % 100, 0);
        assert!(creation.relative_microseconds() >= 0);
        assert!(event.time_stamp().unwrap().is_finite());
        let mut seen = Vec::new();
        let receipt = fire(&mut session, &mut registry, target, &event, &mut seen);
        assert_eq!(seen, [index]);
        assert_eq!(receipt.event_type().code_units(), name);
        assert_eq!(receipt.callbacks(), 1);
    }
}

#[test]
fn real_node_dispatch_orders_capture_and_bubble_and_reuses_sticky_cancellation() {
    let mut session = session();
    let top = top(&mut session);
    let snapshot = session.snapshot();
    let root = snapshot
        .document(top.document())
        .unwrap()
        .document_element()
        .unwrap();
    let body = body(&snapshot, top.document());
    let mut batch = DomBatch::new();
    let target = batch.create_element_ns(top.document(), Namespace::html(), "span");
    batch.insert_before(&target, body, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let name = units("ordinary");
    let mut registry = new_registry::<Vec<&'static str>>();
    for (endpoint, capture_label, bubble_label) in [
        (
            DomEventTarget::Global(top.global()),
            "global-capture",
            "global-bubble",
        ),
        (
            DomEventTarget::Node(top.document().node()),
            "document-capture",
            "document-bubble",
        ),
        (DomEventTarget::Node(root), "root-capture", "root-bubble"),
        (DomEventTarget::Node(body), "body-capture", "body-bubble"),
        (
            DomEventTarget::Node(target),
            "target-capture",
            "target-bubble",
        ),
    ] {
        for (capture, label) in [(true, capture_label), (false, bubble_label)] {
            registry
                .add_listener(
                    &mut session.dom(),
                    endpoint.clone(),
                    &name,
                    DomEventCallback::new(move |_, _, trace: &mut Vec<&'static str>, event| {
                        trace.push(label);
                        if label == "target-capture" {
                            event.prevent_default();
                        }
                        Ok(())
                    }),
                    DomEventListenerOptions {
                        capture,
                        once: false,
                    },
                )
                .unwrap();
        }
    }
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            top.global(),
            &name,
            DomEventInit {
                bubbles: true,
                cancelable: true,
                composed: true,
            },
            &cancel(),
        )
        .unwrap();
    let creation = *event.creation().unwrap();
    let timestamp = event.time_stamp().unwrap();
    let mut trace = Vec::new();
    let receipt = fire(&mut session, &mut registry, target, &event, &mut trace);
    assert_eq!(
        trace,
        [
            "global-capture",
            "document-capture",
            "root-capture",
            "body-capture",
            "target-capture",
            "target-bubble",
            "body-bubble",
            "root-bubble",
            "document-bubble",
            "global-bubble"
        ]
    );
    assert!(receipt.default_prevented());
    assert_eq!(event.target(), Some(target.into()));
    assert_eq!(event.src_element(), Some(target.into()));
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert_eq!(event.composed_path().count(), 0);
    event.set_return_value(true);
    assert!(!event.return_value());
    trace.clear();
    assert!(fire(&mut session, &mut registry, body, &event, &mut trace).default_prevented());
    assert!(!trace.contains(&"target-capture"));
    drop(registry);
    let destination = crate::top(&mut session);
    let destination_node = crate::body(&session.snapshot(), destination.document());
    assert_ne!(destination.global(), top.global());
    let mut replacement = new_registry::<usize>();
    replacement
        .add_listener(
            &mut session.dom(),
            destination.global(),
            &name,
            DomEventCallback::new(|_, _, calls: &mut usize, _| {
                *calls += 1;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                once: false,
            },
        )
        .unwrap();
    let mut calls = 0;
    assert!(
        fire(
            &mut session,
            &mut replacement,
            destination_node,
            &event,
            &mut calls
        )
        .default_prevented()
    );
    assert_eq!(calls, 1);
    assert_eq!(event.target(), Some(destination_node.into()));
    assert_eq!(event.creation(), Some(&creation));
    assert_eq!(event.time_stamp().unwrap(), timestamp);
}

#[test]
fn active_reentry_and_legacy_initialization_obey_the_same_real_event_identity() {
    let mut session = session();
    let top = top(&mut session);
    let target = body(&session.snapshot(), top.document());
    let name = units("guarded");
    let replacement = vec![0xd800, 0, 0xdc00];
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            top.global(),
            &name,
            DomEventInit {
                bubbles: true,
                cancelable: true,
                composed: true,
            },
            &cancel(),
        )
        .unwrap();
    let timestamp = event.time_stamp().unwrap();
    let original_name = name.clone();
    let ignored_name = replacement.clone();
    let mut registry = new_registry::<usize>();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            &name,
            DomEventCallback::new(move |dom, registry, calls: &mut usize, event| {
                *calls += 1;
                assert!(matches!(
                    registry.prepare_dispatch(dom, target, event, &cancel()),
                    Err(DomEventDispatchError::AlreadyDispatching)
                ));
                event
                    .init_event(dom, &ignored_name, false, false, &cancel())
                    .unwrap();
                assert_eq!(event.event_type().code_units(), original_name);
                assert!(event.bubbles());
                assert!(event.cancelable());
                event.set_return_value(false);
                event.stop_propagation();
                event.set_cancel_bubble(false);
                assert!(event.cancel_bubble());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut calls = 0;
    assert!(fire(&mut session, &mut registry, target, &event, &mut calls).default_prevented());
    assert_eq!(calls, 1);
    assert!(!event.cancel_bubble());
    event
        .init_event(&mut session.dom(), &replacement, false, false, &cancel())
        .unwrap();
    assert_eq!(event.event_type().code_units(), replacement);
    assert!(!event.bubbles());
    assert!(!event.cancelable());
    assert!(event.composed());
    assert!(!event.default_prevented());
    assert_eq!(event.target(), None);
    assert_eq!(event.time_stamp().unwrap(), timestamp);
    event.stop_propagation();
    event.set_cancel_bubble(false);
    assert!(event.cancel_bubble());
    assert_eq!(
        fire(&mut session, &mut registry, target, &event, &mut calls).callbacks(),
        0
    );
    assert!(!event.cancel_bubble());
}

#[test]
fn terminal_dispatcher_teardown_cannot_enable_reentry_before_stack_cleanup() {
    let mut session = session();
    let top = top(&mut session);
    let target = body(&session.snapshot(), top.document());
    let name = units("teardown");
    let original = name.clone();
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            top.global(),
            &name,
            DomEventInit::default(),
            &cancel(),
        )
        .unwrap();
    let mut dispatcher = new_registry::<Vec<&'static str>>();
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            &name,
            DomEventCallback::new(
                move |dom, dispatcher, calls: &mut Vec<&'static str>, event| {
                    calls.push("callback");
                    drop(std::mem::replace(dispatcher, new_registry()));
                    assert!(matches!(
                        dispatcher.prepare_dispatch(dom, target, event, &cancel()),
                        Err(DomEventDispatchError::AlreadyDispatching)
                    ));
                    event
                        .init_event(dom, &[0xd800], true, true, &cancel())
                        .unwrap();
                    assert_eq!(event.event_type().code_units(), original);
                    calls.push("after-drop");
                    Ok(())
                },
            ),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut invocation = dispatcher
        .prepare_dispatch(&mut session.dom(), target, &event, &cancel())
        .unwrap();
    let mut calls = Vec::new();
    assert!(matches!(
        invocation.advance(
            &mut session.dom(),
            &mut dispatcher,
            &mut calls,
            limits(),
            &cancel()
        ),
        Err(DomEventDispatchError::Abandoned)
    ));
    assert_eq!(calls, ["callback", "after-drop"]);
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert_eq!(event.composed_path().count(), 0);
    assert!(matches!(
        invocation.advance(
            &mut session.dom(),
            &mut dispatcher,
            &mut calls,
            limits(),
            &cancel()
        ),
        Err(DomEventDispatchError::WrongContinuation)
    ));
    assert_eq!(
        fire(&mut session, &mut dispatcher, target, &event, &mut calls).callbacks(),
        0
    );
}

#[test]
fn host_unavailability_preserves_once_and_cursor_until_real_native_resume() {
    let mut session = session();
    let top = top(&mut session);
    let target = body(&session.snapshot(), top.document());
    let name = units("host-boundary");
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            top.global(),
            &name,
            DomEventInit::default(),
            &cancel(),
        )
        .unwrap();
    let mut registry = new_registry::<Vec<&'static str>>();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            &name,
            DomEventCallback::new(|_, _, calls: &mut Vec<&'static str>, _| {
                calls.push("prefix");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let host = registry
        .add_listener(
            &mut session.dom(),
            target,
            &name,
            DomEventCallback::host(41),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            &name,
            DomEventCallback::new(|_, _, calls: &mut Vec<&'static str>, _| {
                calls.push("suffix");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut invocation = registry
        .prepare_dispatch(&mut session.dom(), target, &event, &cancel())
        .unwrap();
    let mut calls = Vec::new();
    assert!(matches!(
        invocation
            .advance(
                &mut session.dom(),
                &mut registry,
                &mut calls,
                limits(),
                &cancel()
            )
            .unwrap(),
        DomEventDispatchOutcome::RetainedHostUnavailable
    ));
    assert_eq!(calls, ["prefix"]);
    assert!(matches!(
        registry.prepare_dispatch(&mut session.dom(), target, &event, &cancel()),
        Err(DomEventDispatchError::AlreadyDispatching)
    ));
    assert!(registry.remove_listener(host));
    let receipt = complete(
        invocation
            .advance(
                &mut session.dom(),
                &mut registry,
                &mut calls,
                limits(),
                &cancel(),
            )
            .unwrap(),
    );
    assert_eq!(calls, ["prefix", "suffix"]);
    assert_eq!(receipt.callbacks(), 2);
    assert!(matches!(
        invocation.advance(
            &mut session.dom(),
            &mut registry,
            &mut calls,
            limits(),
            &cancel()
        ),
        Err(DomEventDispatchError::AlreadyCompleted)
    ));
}

#[derive(Debug, Eq, PartialEq)]
struct PathObservation {
    label: &'static str,
    phase: DomEventPhase,
    target: Option<NodeId>,
    path: Vec<DomEventTarget>,
}

#[test]
fn actual_composed_shadow_paths_retarget_and_filter_both_root_modes() {
    for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
        let mut session = session();
        let top = top(&mut session);
        let snapshot = session.snapshot();
        let html = snapshot
            .document(top.document())
            .unwrap()
            .document_element()
            .unwrap();
        let body_node = body(&snapshot, top.document());
        let mut batch = DomBatch::new();
        let host = batch.create_element_ns(top.document(), Namespace::html(), "div");
        let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(mode));
        let inner = batch.create_element_ns(top.document(), Namespace::html(), "span");
        batch.insert_before(&inner, &shadow, None);
        batch.insert_before(&host, body_node, None);
        let commit = session.dom().apply(batch, &cancel()).unwrap();
        let host = commit.created(&host).unwrap().unwrap();
        let shadow = commit.created(&shadow).unwrap().unwrap();
        let inner = commit.created(&inner).unwrap().unwrap();
        let name = units("shadow-boundary");
        let mut registry = new_registry::<Vec<PathObservation>>();
        for (endpoint, capture, label) in [
            (DomEventTarget::Global(top.global()), true, "global"),
            (DomEventTarget::Node(host), false, "host"),
            (DomEventTarget::Node(inner), false, "inner"),
        ] {
            registry
                .add_listener(
                    &mut session.dom(),
                    endpoint,
                    &name,
                    DomEventCallback::new(move |_, _, seen: &mut Vec<PathObservation>, event| {
                        seen.push(PathObservation {
                            label,
                            phase: event.phase(),
                            target: event.target().and_then(|target| target.node()),
                            path: event.composed_path().collect(),
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
        let event = session
            .hosted_execution()
            .create_event(
                &mut session.dom(),
                top.global(),
                &name,
                DomEventInit::default(),
                &cancel(),
            )
            .unwrap();
        let mut seen = Vec::new();
        fire(&mut session, &mut registry, inner, &event, &mut seen);
        assert_eq!(
            seen,
            [PathObservation {
                label: "inner",
                phase: DomEventPhase::AtTarget,
                target: Some(inner),
                path: vec![inner.into(), shadow.into()]
            }]
        );
        assert_eq!(event.target(), None);
        let composed = session
            .hosted_execution()
            .create_event(
                &mut session.dom(),
                top.global(),
                &name,
                DomEventInit {
                    composed: true,
                    ..DomEventInit::default()
                },
                &cancel(),
            )
            .unwrap();
        seen.clear();
        fire(&mut session, &mut registry, inner, &composed, &mut seen);
        let all = vec![
            inner.into(),
            shadow.into(),
            host.into(),
            body_node.into(),
            html.into(),
            top.document().node().into(),
            top.global().into(),
        ];
        let outside = if mode == HtmlShadowRootMode::Closed {
            vec![
                host.into(),
                body_node.into(),
                html.into(),
                top.document().node().into(),
                top.global().into(),
            ]
        } else {
            all.clone()
        };
        assert_eq!(
            seen,
            [
                PathObservation {
                    label: "global",
                    phase: DomEventPhase::Capturing,
                    target: Some(host),
                    path: outside.clone()
                },
                PathObservation {
                    label: "inner",
                    phase: DomEventPhase::AtTarget,
                    target: Some(inner),
                    path: all
                },
                PathObservation {
                    label: "host",
                    phase: DomEventPhase::AtTarget,
                    target: Some(host),
                    path: outside
                },
            ]
        );
        assert_eq!(composed.target(), Some(host.into()));
    }
}

#[test]
fn exact_document_load_exception_differs_from_case_distinct_load() {
    let mut session = session();
    let top = top(&mut session);
    let target = body(&session.snapshot(), top.document());
    let mut registry = new_registry::<usize>();
    for name in [units("load"), units("Load")] {
        registry
            .add_listener(
                &mut session.dom(),
                top.global(),
                &name,
                DomEventCallback::new(|_, _, calls: &mut usize, _| {
                    *calls += 1;
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
        let event = session
            .hosted_execution()
            .create_event(
                &mut session.dom(),
                top.global(),
                &name,
                DomEventInit::default(),
                &cancel(),
            )
            .unwrap();
        let mut calls = 0;
        let receipt = fire(&mut session, &mut registry, target, &event, &mut calls);
        let expected = usize::from(name == units("Load"));
        assert_eq!(calls, expected);
        assert_eq!(receipt.callbacks(), expected);
    }
}

struct NativeContext {
    tasks: DomElementTaskRuntime,
    observed: Option<NativeDomEvent>,
    calls: usize,
}
impl DomElementTaskEventContext for NativeContext {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        &mut self.tasks
    }
}

#[test]
fn native_toggle_payload_and_receipt_survive_ordinary_body_reinitialization() {
    let mut session = session();
    let top = top(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let mut batch = DomBatch::new();
    let target = batch.create_element_ns(top.document(), Namespace::html(), "details");
    batch.insert_before(&target, parent, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let mut registry = new_registry::<NativeContext>();
    registry
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::TOGGLE,
            DomEventCallback::new(|_, _, context: &mut NativeContext, event| {
                assert_eq!(event.is_trusted(), context.calls == 0);
                assert!(event.time_stamp().is_err());
                context.observed = Some(event.clone());
                context.calls += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    batch.set_attribute(target, "open", "");
    session.dom().apply(batch, &cancel()).unwrap();
    let mut context = NativeContext {
        tasks: session.element_task_execution(),
        observed: None,
        calls: 0,
    };
    context.tasks.ingest(&mut session.dom(), &cancel()).unwrap();
    let progress = DomElementTaskRuntime::run_next(
        &mut session.dom(),
        DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        DomElementTaskRunLimits {
            dispatch: limits(),
            source: DomSourceLimits::new(16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64),
        },
        &cancel(),
    )
    .unwrap();
    let DomElementTaskProgress::Complete { receipt, .. } = progress else {
        panic!("actual native toggle completion required: {progress:?}")
    };
    assert_eq!(context.calls, 1);
    let event = context.observed.take().unwrap();
    let payload = *event.toggle_event().unwrap();
    let original_receipt_type = receipt.event_type();
    assert!(event.is_trusted());
    fire(&mut session, &mut registry, target, &event, &mut context);
    assert!(!event.is_trusted());
    let name = units("renamed-native");
    event
        .init_event(&mut session.dom(), &name, true, true, &cancel())
        .unwrap();
    assert_eq!(event.toggle_event(), Some(&payload));
    assert_eq!(receipt.event_type(), original_receipt_type);
    assert_eq!(receipt.event_type(), DomEventType::TOGGLE);
    assert_eq!(receipt.original_target(), target.into());
    registry
        .add_listener(
            &mut session.dom(),
            target,
            &name,
            DomEventCallback::new(|_, _, context: &mut NativeContext, event| {
                assert!(!event.is_trusted());
                context.calls += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    fire(&mut session, &mut registry, target, &event, &mut context);
    assert_eq!(context.calls, 3);
}

#[test]
fn stale_and_dropped_preparations_preserve_idle_event_state() {
    let mut session = session();
    let top = top(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let mut batch = DomBatch::new();
    let target = batch.create_element_ns(top.document(), Namespace::html(), "span");
    batch.insert_before(&target, parent, None);
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let original = units("prepared");
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            top.global(),
            &original,
            DomEventInit {
                cancelable: true,
                ..DomEventInit::default()
            },
            &cancel(),
        )
        .unwrap();
    event.prevent_default();
    event.stop_propagation();
    let mut registry = new_registry::<usize>();
    let mut stale = registry
        .prepare_dispatch(&mut session.dom(), target, &event, &cancel())
        .unwrap();
    let mut foreign = new_registry::<usize>();
    let mut calls = 0;
    assert!(matches!(
        stale.advance(
            &mut session.dom(),
            &mut foreign,
            &mut calls,
            limits(),
            &cancel()
        ),
        Err(DomEventDispatchError::WrongContinuation)
    ));
    let mut foreign_dom = crate::session();
    assert!(matches!(
        stale.advance(
            &mut foreign_dom.dom(),
            &mut registry,
            &mut calls,
            limits(),
            &cancel()
        ),
        Err(DomEventDispatchError::Dom(DomError::ForeignOwner))
    ));
    let mut batch = DomBatch::new();
    batch.detach(target);
    batch.destroy(target);
    session.dom().apply(batch, &cancel()).unwrap();
    assert!(matches!(
        stale.advance(
            &mut session.dom(),
            &mut registry,
            &mut calls,
            limits(),
            &cancel()
        ),
        Err(DomEventDispatchError::Dom(DomError::StaleNode))
    ));
    drop(stale);
    assert!(event.cancel_bubble());
    assert!(event.default_prevented());
    assert_eq!(event.event_type().code_units(), original);
    assert_eq!(event.target(), None);
    let mut ready = registry
        .prepare_dispatch(&mut session.dom(), parent, &event, &cancel())
        .unwrap();
    let replacement = units("initialized-while-ready");
    event
        .init_event(&mut session.dom(), &replacement, false, false, &cancel())
        .unwrap();
    registry
        .add_listener(
            &mut session.dom(),
            parent,
            &replacement,
            DomEventCallback::new(|_, _, calls: &mut usize, _| {
                *calls += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let receipt = complete(
        ready
            .advance(
                &mut session.dom(),
                &mut registry,
                &mut calls,
                limits(),
                &cancel(),
            )
            .unwrap(),
    );
    assert_eq!(calls, 1);
    assert_eq!(receipt.event_type().code_units(), replacement);
    assert!(!receipt.default_prevented());
}

#[test]
fn direct_global_load_has_one_at_target_path_without_node_custody() {
    let mut session = session_with_event_capacity(0, 128);
    let top = top(&mut session);
    let global = DomEventTarget::Global(top.global());
    let event = session
        .hosted_execution()
        .create_event(
            &mut session.dom(),
            top.global(),
            DomEventType::LOAD.code_units(),
            DomEventInit {
                cancelable: true,
                ..DomEventInit::default()
            },
            &cancel(),
        )
        .unwrap();
    let mut registry = new_registry::<Vec<(&'static str, DomEventPhase)>>();
    for (target, capture, label) in [
        (global.clone(), false, "global noncapture"),
        (
            DomEventTarget::Node(top.document().node()),
            true,
            "document capture",
        ),
        (global.clone(), true, "global capture"),
    ] {
        let expected = global.clone();
        registry
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::LOAD,
                DomEventCallback::new(move |_, _, calls: &mut Vec<_>, event| {
                    calls.push((label, event.phase()));
                    assert_eq!(event.target(), Some(expected.clone()));
                    assert_eq!(event.current_target(), Some(expected.clone()));
                    let path = event.composed_path().collect::<Vec<_>>();
                    assert_eq!(path.as_slice(), std::slice::from_ref(&expected));
                    event.prevent_default();
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture,
                    once: false,
                },
            )
            .unwrap();
    }
    let mut calls = Vec::new();
    let mut dispatch = registry
        .prepare_dispatch(&mut session.dom(), global.clone(), &event, &cancel())
        .unwrap();
    let receipt = complete(
        dispatch
            .advance(
                &mut session.dom(),
                &mut registry,
                &mut calls,
                DomEventDispatchLimits {
                    path_items: 1,
                    ..limits()
                },
                &cancel(),
            )
            .unwrap(),
    );
    assert_eq!(
        calls,
        [
            ("global capture", DomEventPhase::AtTarget),
            ("global noncapture", DomEventPhase::AtTarget)
        ]
    );
    assert_eq!(receipt.original_target(), global);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert!(receipt.default_prevented());
    assert_eq!(event.target(), Some(global));
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    assert_eq!(event.composed_path().count(), 0);
}

#[test]
fn standalone_uses_its_own_global_and_registry_without_a_dom_recipient() {
    let mut session = session_with_event_capacity(0, 0);
    let event_origin = top(&mut session);
    let target_origin = top(&mut session);
    let mut hosted = session.hosted_execution();
    let mut registry = new_registry::<Vec<(DomEventTarget, DomEventComposedPath)>>();
    let first = hosted
        .create_event_target(
            &mut session.dom(),
            &registry,
            target_origin.global(),
            &cancel(),
        )
        .unwrap();
    let second = hosted
        .create_event_target(
            &mut session.dom(),
            &registry,
            target_origin.global(),
            &cancel(),
        )
        .unwrap();
    assert_ne!(first, second);
    assert_eq!(first.relevant_global(), target_origin.global());
    let first = DomEventTarget::Standalone(first);
    let second = DomEventTarget::Standalone(second);
    let event = hosted
        .create_event(
            &mut session.dom(),
            event_origin.global(),
            DomEventType::LOAD.code_units(),
            DomEventInit::default(),
            &cancel(),
        )
        .unwrap();
    let callback = DomEventCallback::new(|dom, registry, seen: &mut Vec<_>, event| {
        let target = event.current_target().unwrap();
        assert_eq!(event.target(), Some(target.clone()));
        assert_eq!(event.phase(), DomEventPhase::AtTarget);
        assert!(matches!(
            registry.prepare_dispatch(dom, target.clone(), event, &cancel()),
            Err(DomEventDispatchError::AlreadyDispatching)
        ));
        seen.push((target, event.composed_path()));
        Ok(())
    });
    for target in [first.clone(), second.clone()] {
        registry
            .add_listener(
                &mut session.dom(),
                target,
                DomEventType::LOAD,
                callback.clone(),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let mut seen = Vec::new();
    for target in [first.clone(), second.clone()] {
        let mut dispatch = registry
            .prepare_dispatch(&mut session.dom(), target.clone(), &event, &cancel())
            .unwrap();
        let receipt = complete(
            dispatch
                .advance(
                    &mut session.dom(),
                    &mut registry,
                    &mut seen,
                    DomEventDispatchLimits {
                        path_items: 1,
                        ..limits()
                    },
                    &cancel(),
                )
                .unwrap(),
        );
        assert_eq!(receipt.original_target(), target);
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
    }
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].0, first);
    assert_eq!(seen[1].0, second);
    assert_eq!(event.creation().unwrap().global(), event_origin.global());

    let mut retired_global = registry
        .prepare_dispatch(
            &mut session.dom(),
            target_origin.global(),
            &event,
            &cancel(),
        )
        .unwrap();
    let retirement = hosted
        .destroy_top_level(&mut session.dom(), target_origin.navigable(), &cancel())
        .unwrap();
    assert!((0..256).any(|_| matches!(
        hosted.run_next(&mut session.dom(), retirement.traversable(), &cancel()).unwrap(),
        DomHostedTaskProgress::Complete { operation } if operation == retirement.operation()
    )));
    assert!(matches!(
        retired_global.advance(
            &mut session.dom(),
            &mut registry,
            &mut seen,
            limits(),
            &cancel()
        ),
        Err(DomEventDispatchError::Hosted(DomHostedError::StaleIdentity))
    ));
    let mut inactive = registry
        .prepare_dispatch(&mut session.dom(), second.clone(), &event, &cancel())
        .unwrap();
    let receipt = complete(
        inactive
            .advance(
                &mut session.dom(),
                &mut registry,
                &mut seen,
                DomEventDispatchLimits {
                    path_items: 1,
                    ..limits()
                },
                &cancel(),
            )
            .unwrap(),
    );
    assert_eq!(receipt.callbacks(), 0);
    assert_eq!(receipt.original_target(), second);
    assert_eq!(seen.len(), 2);

    drop(registry);
    let mut replacement = new_registry::<()>();
    let escaped = seen.remove(0).1.collect::<Vec<_>>();
    assert_eq!(escaped, [first]);
    assert!(matches!(
        replacement.prepare_dispatch(&mut session.dom(), escaped[0].clone(), &event, &cancel()),
        Err(DomEventDispatchError::WrongTargetRegistry)
    ));
}

#[test]
fn omitted_wheel_listener_is_passive_on_the_first_html_body_or_frameset() {
    let mut session = session();
    let top = top(&mut session);
    let document = top.document();
    let snapshot = session.snapshot();
    let html = snapshot
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    let &[head, later_body] = snapshot.node(html).unwrap().children() else {
        panic!("the hosted HTML root must start with its head and body");
    };
    let mut batch = DomBatch::new();
    let frameset = batch.create_element_ns(document, Namespace::html(), "frameset");
    batch.insert_before(&frameset, html, Some(later_body.into()));
    let commit = session.dom().apply(batch, &cancel()).unwrap();
    let frameset = commit.created(&frameset).unwrap().unwrap();

    // The HTML body getter selects the first direct HTML body or frameset.
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.node(html).unwrap().children(),
        [head, frameset, later_body]
    );
    let html_namespace = DocumentString::from("http://www.w3.org/1999/xhtml");
    for (node, local_name) in [(head, "head"), (frameset, "frameset"), (later_body, "body")] {
        let view = snapshot.node(node).unwrap();
        assert_eq!(view.owner_document(), Some(document));
        assert_eq!(view.parent(), Some(html));
        let name = view.as_element().unwrap().name();
        assert_eq!(name.namespace_uri(), Some(&html_namespace));
        assert_eq!(name.local_name(), &DocumentString::from(local_name));
    }

    let mut hosted = session.hosted_execution();
    let mut registry = new_registry::<Vec<(DomEventPhase, Option<DomEventTarget>, bool, bool)>>();
    // A later body and a case-distinct type on the selected frameset are
    // independent controls. Both must be able to cancel through the same API.
    for (target, name, canceled) in [
        (later_body, "wheel", true),
        (frameset, "Wheel", true),
        (frameset, "wheel", false),
    ] {
        let name_units = units(name);
        registry
            .add_listener(
                &mut session.dom(),
                target,
                &name_units,
                DomEventCallback::new(|_, _, trace: &mut Vec<_>, event| {
                    let before = event.default_prevented();
                    event.prevent_default();
                    trace.push((
                        event.phase(),
                        event.current_target(),
                        before,
                        event.default_prevented(),
                    ));
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let event = hosted
            .create_event(
                &mut session.dom(),
                top.global(),
                &name_units,
                DomEventInit {
                    cancelable: true,
                    ..DomEventInit::default()
                },
                &cancel(),
            )
            .unwrap();
        assert!(event.cancelable());
        let mut trace = Vec::new();
        let receipt = fire(&mut session, &mut registry, target, &event, &mut trace);
        assert_eq!(receipt.callbacks(), 1, "{name} on {target:?}");
        assert!(receipt.failures().is_empty());
        assert_eq!(trace.len(), 1);
        let (phase, current, before, after) = &trace[0];
        assert_eq!(*phase, DomEventPhase::AtTarget);
        assert_eq!(*current, Some(target.into()));
        assert!(!*before);
        assert_eq!(
            (
                after,
                receipt.default_prevented(),
                event.default_prevented()
            ),
            (&canceled, canceled, canceled),
            "{name} on {target:?}"
        );
    }
}
