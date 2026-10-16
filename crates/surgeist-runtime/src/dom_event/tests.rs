//! Literal behavioral oracles from retained DOM3071 dispatch/invoke/parent and
//! composedPath algorithms. This wholly new API has no executable prior RED.
use super::*;
use std::sync::atomic::AtomicBool;
use surgeist_dom::{DocumentId, DomBatch, DomCommit, DomLimits, Namespace};

fn limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 64,
        snapshot_listeners: 64,
        failure_records: 64,
        callbacks: 64,
        work: 100_000,
    }
}
fn registry<C>() -> DomEventDispatcher<C> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 64,
        pending_events: 8,
        nested_dispatches: 8,
    })
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn dom() -> Dom {
    Dom::new(DomLimits::new(128, 32768, 512, 128, 128, 32, 131072).unwrap()).unwrap()
}
struct Ordinary {
    dom: Dom,
    document: DocumentId,
    parent: NodeId,
    target: NodeId,
}
fn ordinary() -> Ordinary {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let p = b.create_element(&d, "parent");
    let t = b.create_element(&d, "target");
    b.insert_before(&p, &d, None);
    b.insert_before(&t, &p, None);
    let c = apply(&mut dom, b);
    Ordinary {
        document: c.document(&d).unwrap(),
        parent: c.created(&p).unwrap().unwrap(),
        target: c.created(&t).unwrap().unwrap(),
        dom,
    }
}
fn complete(outcome: DomEventDispatchOutcome) -> DomEventDispatchReceipt {
    match outcome {
        DomEventDispatchOutcome::Complete(receipt) => receipt,
        other => panic!("expected actual completion: {other:?}"),
    }
}
#[derive(Debug, Eq, PartialEq)]
struct Observation {
    current: NodeId,
    target: NodeId,
    phase: DomEventPhase,
    path: Vec<NodeId>,
}
fn observer() -> DomEventCallback<Vec<Observation>> {
    DomEventCallback::new(|_, _, log: &mut Vec<Observation>, event| {
        log.push(Observation {
            current: event.current_target().unwrap(),
            target: event.target().unwrap(),
            phase: event.phase(),
            path: event.composed_path().collect(),
        });
        assert!(!event.bubbles() && !event.cancelable() && !event.composed());
        event.prevent_default();
        assert!(!event.default_prevented());
        Ok(())
    })
}
#[test]
fn nonbubbling_source_events_capture_ancestors_then_both_target_phases() {
    for kind in [DomEventType::Load, DomEventType::Error] {
        let mut f = ordinary();
        let mut listeners = registry();
        let mut log = Vec::new();
        for node in [f.document.node(), f.parent, f.target] {
            for capture in [false, true] {
                listeners
                    .add_listener(
                        &DomEventDom::borrow(&mut f.dom),
                        node,
                        kind,
                        observer(),
                        DomEventListenerOptions {
                            capture,
                            once: false,
                        },
                    )
                    .unwrap();
            }
        }
        let event = if kind == DomEventType::Load {
            NativeDomEvent::style_load(f.target)
        } else {
            NativeDomEvent::style_error(f.target)
        };
        let receipt = complete(
            listeners
                .dispatch(&mut f.dom, &mut log, &event, limits())
                .unwrap(),
        );
        let path = vec![f.target, f.parent, f.document.node()];
        assert_eq!(
            log,
            vec![
                Observation {
                    current: f.document.node(),
                    target: f.target,
                    phase: DomEventPhase::Capturing,
                    path: path.clone()
                },
                Observation {
                    current: f.parent,
                    target: f.target,
                    phase: DomEventPhase::Capturing,
                    path: path.clone()
                },
                Observation {
                    current: f.target,
                    target: f.target,
                    phase: DomEventPhase::AtTarget,
                    path: path.clone()
                },
                Observation {
                    current: f.target,
                    target: f.target,
                    phase: DomEventPhase::AtTarget,
                    path
                },
            ]
        );
        assert_eq!(receipt.original_target(), f.target);
        assert_eq!(receipt.event_type(), kind);
        assert_eq!(receipt.callbacks(), 4);
        assert!(receipt.failures().is_empty());
        assert!(!receipt.default_prevented());
        assert_eq!(event.current_target(), None);
        assert_eq!(event.phase(), DomEventPhase::None);
        assert_eq!(event.target(), Some(f.target));
        assert_eq!(event.composed_path().count(), 0);
        assert!(matches!(
            listeners.dispatch(&mut f.dom, &mut log, &event, limits()),
            Err(DomEventDispatchError::AlreadyCompleted)
        ));
    }
}
#[test]
fn duplicate_key_ignores_once_change_but_capture_is_distinct_and_ids_do_not_alias_reuse() {
    let mut f = ordinary();
    let mut listeners = registry();
    let callback = DomEventCallback::new(|_, _, count: &mut usize, _| {
        *count += 1;
        Ok(())
    });
    let first = listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            callback.clone(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let duplicate = listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            callback.clone(),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    assert_eq!(first, duplicate);
    let capture = listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            callback.clone(),
            DomEventListenerOptions {
                capture: true,
                once: false,
            },
        )
        .unwrap();
    assert_ne!(first, capture);
    let mut count = 0;
    for _ in 0..2 {
        complete(
            listeners
                .dispatch(
                    &mut f.dom,
                    &mut count,
                    &NativeDomEvent::style_load(f.target),
                    limits(),
                )
                .unwrap(),
        );
    }
    assert_eq!(count, 4);
    assert!(listeners.remove_listener(first));
    let replacement = listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            callback,
            DomEventListenerOptions::default(),
        )
        .unwrap();
    assert_ne!(first, replacement);
    assert!(!listeners.remove_listener(first));
    assert!(listeners.remove_listener(replacement));
}
#[test]
fn additions_are_snapshotted_at_each_invoke_and_not_at_dispatch_start() {
    let mut f = ordinary();
    let mut listeners = registry();
    let target = f.target;
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.document.node(),
            DomEventType::Load,
            DomEventCallback::new(move |dom, listeners, log: &mut Vec<&'static str>, _| {
                log.push("document");
                listeners
                    .add_listener(
                        dom,
                        target,
                        DomEventType::Load,
                        DomEventCallback::new(move |dom, listeners, log: &mut Vec<&'static str>, _| {
                            log.push("target capture");
                            listeners
                                .add_listener(
                                    dom,
                                    target,
                                    DomEventType::Load,
                                    DomEventCallback::new(move |dom, listeners, log: &mut Vec<&'static str>, _| {
                                        log.push("target noncapture");
                                        listeners
                                            .add_listener(
                                                dom,
                                                target,
                                                DomEventType::Load,
                                                DomEventCallback::new(|_, _, log: &mut Vec<&'static str>, _| {
                                                    log.push("too late");
                                                    Ok(())
                                                }),
                                                DomEventListenerOptions::default(),
                                            )
                                            .unwrap();
                                        Ok(())
                                    }),
                                    DomEventListenerOptions::default(),
                                )
                                .unwrap();
                            Ok(())
                        }),
                        DomEventListenerOptions {
                            capture: true,
                            once: false,
                        },
                    )
                    .unwrap();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                once: false,
            },
        )
        .unwrap();
    let mut log = Vec::new();
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut log,
                &NativeDomEvent::style_load(f.target),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(log, ["document", "target capture", "target noncapture"]);
}
#[derive(Default)]
struct Removal {
    log: Vec<&'static str>,
    remove: Option<DomEventRegistrationId>,
}
#[test]
fn removed_snapshot_entry_is_skipped_even_when_its_slot_is_reused() {
    let mut f = ordinary();
    let mut listeners = registry();
    let target = f.target;
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            target,
            DomEventType::Load,
            DomEventCallback::new(move |dom, listeners, c: &mut Removal, _| {
                c.log.push("first");
                if let Some(id) = c.remove.take() {
                    assert!(listeners.remove_listener(id));
                    listeners
                        .add_listener(
                            dom,
                            target,
                            DomEventType::Load,
                            DomEventCallback::new(|_, _, c: &mut Removal, _| {
                                c.log.push("replacement");
                                Ok(())
                            }),
                            DomEventListenerOptions::default(),
                        )
                        .unwrap();
                }
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let second = listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, c: &mut Removal, _| {
                c.log.push("removed");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut c = Removal {
        remove: Some(second),
        ..Removal::default()
    };
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &NativeDomEvent::style_load(target),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(c.log, ["first"]);
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &NativeDomEvent::style_load(target),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(c.log, ["first", "first", "replacement"]);
}
#[test]
fn once_is_removed_before_nested_dispatch_and_same_event_redispatch_is_rejected() {
    let mut f = ordinary();
    let mut listeners = registry();
    let target = f.target;
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            target,
            DomEventType::Load,
            DomEventCallback::new(move |dom, listeners, c: &mut Vec<&'static str>, event| {
                c.push("once");
                assert!(matches!(
                    listeners.dispatch(dom.raw_mut(), c, &event.clone(), limits()),
                    Err(DomEventDispatchError::AlreadyDispatching)
                ));
                assert!(matches!(
                    listeners.resume(dom.raw_mut(), c, event, limits()),
                    Err(DomEventDispatchError::AlreadyDispatching)
                ));
                complete(
                    listeners
                        .dispatch(
                            dom.raw_mut(),
                            c,
                            &NativeDomEvent::style_load(target),
                            limits(),
                        )
                        .unwrap(),
                );
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, c: &mut Vec<&'static str>, _| {
                c.push("ordinary");
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut c = Vec::new();
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &NativeDomEvent::style_load(target),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(c, ["once", "ordinary", "ordinary"]);
}
#[test]
fn stop_controls_apply_at_source_invoke_boundaries_including_target_second_invoke() {
    for immediate in [false, true] {
        let mut f = ordinary();
        let mut listeners = registry();
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, c: &mut Vec<&'static str>, event| {
                    c.push("first capture");
                    if immediate {
                        event.stop_immediate_propagation();
                    } else {
                        event.stop_propagation();
                    }
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
        for (capture, label) in [(true, "second capture"), (false, "noncapture")] {
            listeners
                .add_listener(
                    &DomEventDom::borrow(&mut f.dom),
                    f.target,
                    DomEventType::Load,
                    DomEventCallback::new(move |_, _, c: &mut Vec<&'static str>, _| {
                        c.push(label);
                        Ok(())
                    }),
                    DomEventListenerOptions {
                        capture,
                        once: false,
                    },
                )
                .unwrap();
        }
        let mut c = Vec::new();
        complete(
            listeners
                .dispatch(
                    &mut f.dom,
                    &mut c,
                    &NativeDomEvent::style_load(f.target),
                    limits(),
                )
                .unwrap(),
        );
        assert_eq!(
            c,
            if immediate {
                vec!["first capture"]
            } else {
                vec!["first capture", "second capture"]
            }
        );
    }
    let mut f = ordinary();
    let mut listeners = registry();
    for (node, stop) in [
        (f.document.node(), true),
        (f.parent, false),
        (f.target, false),
    ] {
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                node,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, c: &mut Vec<NodeId>, event| {
                    c.push(event.current_target().unwrap());
                    if stop {
                        event.stop_propagation();
                    }
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
    }
    let mut c = Vec::new();
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &NativeDomEvent::style_load(f.target),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(c, [f.document.node()]);
}
#[derive(Default)]
struct Edits {
    log: Vec<NodeId>,
    nested: bool,
    nested_log: Vec<NodeId>,
}
#[test]
fn callback_edits_real_dom_and_adopts_target_while_outer_path_stays_fixed_and_nested_path_changes()
{
    let mut f = ordinary();
    let mut b = DomBatch::new();
    let other = b.create_document();
    let c = apply(&mut f.dom, b);
    let other = c.document(&other).unwrap();
    let mut listeners = registry();
    let target = f.target;
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.document.node(),
            DomEventType::Load,
            DomEventCallback::new(move |dom, listeners, c: &mut Edits, event| {
                c.log.push(event.current_target().unwrap());
                let mut b = DomBatch::new();
                b.adopt(target, other);
                b.set_attribute(target, "observed", "actual callback edit");
                apply(dom.raw_mut(), b);
                c.nested = true;
                complete(
                    listeners
                        .dispatch(
                            dom.raw_mut(),
                            c,
                            &NativeDomEvent::style_load(target),
                            limits(),
                        )
                        .unwrap(),
                );
                c.nested = false;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                once: false,
            },
        )
        .unwrap();
    for node in [f.parent, f.target] {
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                node,
                DomEventType::Load,
                DomEventCallback::new(|_, _, c: &mut Edits, event| {
                    if c.nested {
                        c.nested_log.push(event.current_target().unwrap());
                    } else {
                        c.log.push(event.current_target().unwrap());
                    }
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
    }
    let mut c = Edits::default();
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &NativeDomEvent::style_load(target),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(c.log, [f.document.node(), f.parent, target]);
    assert_eq!(c.nested_log, [target]);
    let s = f.dom.snapshot();
    assert_eq!(s.node(target).unwrap().owner_document(), Some(other));
    assert_eq!(s.node(target).unwrap().parent(), None);
    assert_eq!(
        s.node(target).unwrap().attribute(&"observed".into()),
        Some(&"actual callback edit".into())
    );
}
struct Shadow {
    dom: Dom,
    document: DocumentId,
    host: NodeId,
    root: NodeId,
    slot: NodeId,
    light: NodeId,
    inside: NodeId,
}
fn shadow(mode: HtmlShadowRootMode) -> Shadow {
    shadow_config(mode, surgeist_dom::HtmlSlotAssignment::Named)
}
fn shadow_config(mode: HtmlShadowRootMode, assignment: surgeist_dom::HtmlSlotAssignment) -> Shadow {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let h = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&h, &d, None);
    let r = b.attach_shadow(
        &h,
        surgeist_dom::HtmlShadowRootProperties::new(mode, assignment, false, false, false),
    );
    let s = b.create_element_ns(&d, Namespace::html(), "slot");
    b.insert_before(&s, &r, None);
    let l = b.create_text(&d, "light");
    b.insert_before(&l, &h, None);
    if assignment == surgeist_dom::HtmlSlotAssignment::Manual {
        b.assign_slot(&s, vec![(&l).into()]);
    }
    let i = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&i, &r, None);
    let c = apply(&mut dom, b);
    Shadow {
        document: c.document(&d).unwrap(),
        host: c.created(&h).unwrap().unwrap(),
        root: c.created(&r).unwrap().unwrap(),
        slot: c.created(&s).unwrap().unwrap(),
        light: c.created(&l).unwrap().unwrap(),
        inside: c.created(&i).unwrap().unwrap(),
        dom,
    }
}
#[test]
fn noncomposed_shadow_origin_stops_at_its_first_target_root_in_both_modes_and_clears_target() {
    for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
        let mut f = shadow(mode);
        let mut listeners = registry();
        for node in [f.document.node(), f.host, f.root, f.inside] {
            listeners
                .add_listener(
                    &DomEventDom::borrow(&mut f.dom),
                    node,
                    DomEventType::Load,
                    observer(),
                    DomEventListenerOptions {
                        capture: true,
                        once: false,
                    },
                )
                .unwrap();
        }
        let event = NativeDomEvent::style_load(f.inside);
        let mut c = Vec::new();
        complete(
            listeners
                .dispatch(&mut f.dom, &mut c, &event, limits())
                .unwrap(),
        );
        assert_eq!(
            c,
            [
                Observation {
                    current: f.root,
                    target: f.inside,
                    phase: DomEventPhase::Capturing,
                    path: vec![f.inside, f.root]
                },
                Observation {
                    current: f.inside,
                    target: f.inside,
                    phase: DomEventPhase::AtTarget,
                    path: vec![f.inside, f.root]
                },
            ]
        );
        assert_eq!(event.target(), None);
    }
}
#[test]
fn slotted_light_origin_crosses_shadow_host_and_closed_composed_path_hides_only_internal_segment() {
    for mode in [HtmlShadowRootMode::Open, HtmlShadowRootMode::Closed] {
        let mut f = shadow(mode);
        assert_eq!(
            f.dom
                .snapshot()
                .node(f.light)
                .unwrap()
                .stored_assigned_slot()
                .unwrap(),
            Some(f.slot)
        );
        let mut listeners = registry();
        for node in [f.document.node(), f.host, f.root, f.slot, f.light] {
            listeners
                .add_listener(
                    &DomEventDom::borrow(&mut f.dom),
                    node,
                    DomEventType::Error,
                    observer(),
                    DomEventListenerOptions {
                        capture: true,
                        once: false,
                    },
                )
                .unwrap();
        }
        let mut c = Vec::new();
        let event = NativeDomEvent::style_error(f.light);
        complete(
            listeners
                .dispatch(&mut f.dom, &mut c, &event, limits())
                .unwrap(),
        );
        let full = vec![f.light, f.slot, f.root, f.host, f.document.node()];
        let outer = if mode == HtmlShadowRootMode::Closed {
            vec![f.light, f.host, f.document.node()]
        } else {
            full.clone()
        };
        assert_eq!(
            c.iter().map(|r| r.current).collect::<Vec<_>>(),
            [f.document.node(), f.host, f.root, f.slot, f.light]
        );
        for observation in &c {
            assert_eq!(observation.target, f.light);
            assert_eq!(
                observation.path,
                if [f.root, f.slot].contains(&observation.current) {
                    full.clone()
                } else {
                    outer.clone()
                }
            );
        }
        assert_eq!(event.target(), Some(f.light));
    }
}
#[test]
fn stored_assignment_not_fresh_slot_query_controls_event_path() {
    let mut f = shadow_config(
        HtmlShadowRootMode::Open,
        surgeist_dom::HtmlSlotAssignment::Manual,
    );
    let mut b = DomBatch::new();
    let h2 = b.create_element_ns(f.document, Namespace::html(), "div");
    b.insert_before(&h2, f.host, None);
    let r2 = b.attach_shadow(
        &h2,
        surgeist_dom::HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Open,
            surgeist_dom::HtmlSlotAssignment::Manual,
            false,
            false,
            false,
        ),
    );
    let s2 = b.create_element_ns(f.document, Namespace::html(), "slot");
    b.insert_before(&s2, &r2, None);
    // A manual assignment in an unrelated root leaves the original stored
    // assignment until that source root's reassignment algorithm is reached.
    b.assign_slot(&s2, vec![f.light.into()]);
    apply(&mut f.dom, b);
    let s = f.dom.snapshot();
    assert_eq!(
        s.node(f.light).unwrap().stored_assigned_slot().unwrap(),
        Some(f.slot)
    );
    assert_eq!(
        s.node(f.light)
            .unwrap()
            .assigned_slot(&AtomicBool::new(false))
            .unwrap(),
        None
    );
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.slot,
            DomEventType::Load,
            observer(),
            DomEventListenerOptions {
                capture: true,
                once: false,
            },
        )
        .unwrap();
    let mut c = Vec::new();
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &NativeDomEvent::style_load(f.light),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].current, f.slot);
}
#[test]
fn document_load_and_error_stop_at_actual_native_no_browsing_context_boundary() {
    for kind in [DomEventType::Load, DomEventType::Error] {
        let mut f = ordinary();
        let mut listeners = registry();
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.document.node(),
                kind,
                observer(),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
        let event = if kind == DomEventType::Load {
            NativeDomEvent::style_load(f.document.node())
        } else {
            NativeDomEvent::style_error(f.document.node())
        };
        let mut c = Vec::new();
        complete(
            listeners
                .dispatch(&mut f.dom, &mut c, &event, limits())
                .unwrap(),
        );
        assert_eq!(
            c,
            [Observation {
                current: f.document.node(),
                target: f.document.node(),
                phase: DomEventPhase::AtTarget,
                path: vec![f.document.node()]
            }]
        );
    }
}
#[test]
fn callback_failure_and_unwind_are_reported_separately_while_later_real_edits_complete() {
    let mut f = ordinary();
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|dom, _, _: &mut (), event| {
                let mut b = DomBatch::new();
                b.set_attribute(event.current_target().unwrap(), "before-failure", "kept");
                apply(dom.raw_mut(), b);
                Err(DomEventCallbackError { code: 7 })
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, _, _| panic!("listener unwind")),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|dom, _, _, event| {
                let mut b = DomBatch::new();
                b.set_attribute(event.current_target().unwrap(), "delivered", "yes");
                apply(dom.raw_mut(), b);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = NativeDomEvent::style_load(f.target);
    let receipt = complete(
        listeners
            .dispatch(&mut f.dom, &mut (), &event, limits())
            .unwrap(),
    );
    assert_eq!(receipt.callbacks(), 3);
    assert_eq!(
        receipt
            .failures()
            .iter()
            .map(|f| f.kind)
            .collect::<Vec<_>>(),
        [
            DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 7 }),
            DomEventListenerFailureKind::Panicked
        ]
    );
    assert!(!receipt.default_prevented());
    assert_eq!(
        f.dom
            .snapshot()
            .node(f.target)
            .unwrap()
            .attribute(&"delivered".into()),
        Some(&"yes".into())
    );
    assert_eq!(
        f.dom
            .snapshot()
            .node(f.target)
            .unwrap()
            .attribute(&"before-failure".into()),
        Some(&"kept".into())
    );
    assert_eq!(event.current_target(), None);
    // No running/pending guard was stranded by the panic.
    complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut (),
                &NativeDomEvent::style_load(f.target),
                limits(),
            )
            .unwrap(),
    );
}
#[test]
fn admission_before_delivery_preserves_event_and_after_delivery_resumes_without_replay() {
    let mut f = ordinary();
    let mut listeners = registry();
    for label in [1, 2, 3] {
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, c: &mut Vec<u8>, _| {
                    c.push(label);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let event = NativeDomEvent::style_load(f.target);
    let mut c = Vec::new();
    assert!(matches!(
        listeners.dispatch(
            &mut f.dom,
            &mut c,
            &event,
            DomEventDispatchLimits {
                path_items: 1,
                ..limits()
            }
        ),
        Err(DomEventDispatchError::Admission(
            DomEventResource::PathItems
        ))
    ));
    assert!(c.is_empty());
    let one = DomEventDispatchLimits {
        callbacks: 1,
        ..limits()
    };
    assert!(matches!(
        listeners.dispatch(&mut f.dom, &mut c, &event, one).unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    assert_eq!(c, [1]);
    assert_eq!(event.current_target(), None);
    assert!(matches!(
        listeners.dispatch(&mut f.dom, &mut c, &event, limits()),
        Err(DomEventDispatchError::AlreadyDispatching)
    ));
    let mut different = registry();
    assert!(matches!(
        different.resume(&mut f.dom, &mut c, &event, limits()),
        Err(DomEventDispatchError::WrongContinuation)
    ));
    assert!(matches!(
        listeners.resume(&mut f.dom, &mut c, &event, one).unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    assert_eq!(c, [1, 2]);
    let receipt = complete(listeners.resume(&mut f.dom, &mut c, &event, one).unwrap());
    assert_eq!(c, [1, 2, 3]);
    assert_eq!(receipt.callbacks(), 3);
}
#[test]
fn retained_current_snapshot_continues_after_stop_propagation_and_does_not_replay() {
    let mut f = ordinary();
    let mut listeners = registry();
    for n in [1, 2] {
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, c: &mut Vec<u8>, event| {
                    c.push(n);
                    event.stop_propagation();
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: true,
                    once: false,
                },
            )
            .unwrap();
    }
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, c: &mut Vec<u8>, _| {
                c.push(3);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut c = Vec::new();
    let event = NativeDomEvent::style_load(f.target);
    assert!(matches!(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &event,
                DomEventDispatchLimits {
                    callbacks: 1,
                    ..limits()
                }
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    complete(
        listeners
            .resume(&mut f.dom, &mut c, &event, limits())
            .unwrap(),
    );
    assert_eq!(c, [1, 2]);
}
#[test]
fn per_invoke_snapshot_and_failure_output_admission_are_retained_before_callback() {
    for resource in [
        DomEventResource::SnapshotListeners,
        DomEventResource::FailureRecords,
        DomEventResource::Work,
    ] {
        let mut f = ordinary();
        let mut listeners = registry();
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::Load,
                DomEventCallback::new(|_, _, count: &mut usize, _| {
                    *count += 1;
                    Err(DomEventCallbackError { code: 9 })
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
        let event = NativeDomEvent::style_load(f.target);
        let low = match resource {
            DomEventResource::SnapshotListeners => DomEventDispatchLimits {
                snapshot_listeners: 0,
                ..limits()
            },
            DomEventResource::FailureRecords => DomEventDispatchLimits {
                failure_records: 0,
                ..limits()
            },
            DomEventResource::Work => DomEventDispatchLimits {
                work: 0,
                ..limits()
            },
            _ => unreachable!(),
        };
        let mut count = 0;
        let outcome = listeners.dispatch(&mut f.dom, &mut count, &event, low);
        assert_eq!(count, 0);
        if resource == DomEventResource::Work {
            assert!(matches!(
                outcome,
                Err(DomEventDispatchError::Admission(DomEventResource::Work))
            ));
            complete(
                listeners
                    .dispatch(&mut f.dom, &mut count, &event, limits())
                    .unwrap(),
            );
        } else {
            assert!(
                matches!(outcome.unwrap(), DomEventDispatchOutcome::Retained(r) if r == resource)
            );
            let receipt = complete(
                listeners
                    .resume(&mut f.dom, &mut count, &event, limits())
                    .unwrap(),
            );
            assert_eq!(receipt.failures().len(), 1);
        }
        assert_eq!(count, 1);
    }
}
#[test]
fn dropping_retained_event_releases_pending_admission_and_nested_limits_reject_before_delivery() {
    let mut f = ordinary();
    let mut listeners = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 4,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let target = f.target;
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            target,
            DomEventType::Load,
            DomEventCallback::new(move |dom, listeners, count: &mut usize, _| {
                *count += 1;
                assert!(matches!(
                    listeners.dispatch(
                        dom.raw_mut(),
                        count,
                        &NativeDomEvent::style_error(target),
                        limits()
                    ),
                    Err(DomEventDispatchError::Admission(
                        DomEventResource::NestedDispatches
                    ))
                ));
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = NativeDomEvent::style_load(target);
    let mut count = 0;
    assert!(matches!(
        listeners
            .dispatch(
                &mut f.dom,
                &mut count,
                &event,
                DomEventDispatchLimits {
                    callbacks: 0,
                    ..limits()
                }
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    assert!(matches!(
        listeners.dispatch(
            &mut f.dom,
            &mut count,
            &NativeDomEvent::style_error(target),
            limits()
        ),
        Err(DomEventDispatchError::Admission(
            DomEventResource::PendingEvents
        ))
    ));
    let observation: NativeDomEvent = (*event).clone();
    // Observer payload can be borrowed/debugged while task custody is dropped.
    // Terminal cleanup must release its reservation without borrowing that cell.
    let observation_borrow = observation.0.path.borrow();
    drop(event);
    assert!(format!("{observation:?}").contains("Abandoned"));
    assert!(matches!(
        listeners.resume(&mut f.dom, &mut count, &observation, limits()),
        Err(DomEventDispatchError::Abandoned)
    ));
    assert_eq!(count, 0);
    drop(observation_borrow);
    let receipt = complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut count,
                &NativeDomEvent::style_load(target),
                limits(),
            )
            .unwrap(),
    );
    assert!(
        receipt.failures().is_empty(),
        "callback assertions must not be swallowed as reported failures"
    );
    assert_eq!(count, 1);
}
#[test]
fn retirement_keeps_started_exact_path_and_never_delivers_or_mutates_reused_generation() {
    let mut f = ordinary();
    let target = f.target;
    let document = f.document;
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.document.node(),
            DomEventType::Load,
            DomEventCallback::new(move |dom, listeners, c: &mut Vec<NodeId>, _| {
                let mut b = DomBatch::new();
                b.detach(target);
                b.destroy(target);
                apply(dom.raw_mut(), b);
                let mut b = DomBatch::new();
                let replacement = b.create_element(document, "replacement");
                let result = apply(dom.raw_mut(), b);
                let replacement = result.created(&replacement).unwrap().unwrap();
                assert_ne!(replacement, target);
                c.push(replacement);
                listeners
                    .add_listener(
                        dom,
                        replacement,
                        DomEventType::Load,
                        DomEventCallback::new(|_, _, _, _| {
                            panic!("reused generation received old dispatch")
                        }),
                        DomEventListenerOptions::default(),
                    )
                    .unwrap();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                once: true,
            },
        )
        .unwrap();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            target,
            DomEventType::Load,
            DomEventCallback::new(move |dom, _, c: &mut Vec<NodeId>, event| {
                assert_eq!(event.current_target(), Some(target));
                assert_eq!(event.target(), Some(target));
                c.push(target);
                let mut b = DomBatch::new();
                b.set_attribute(target, "stale", "never");
                assert!(dom.apply(b, &AtomicBool::new(false)).is_err());
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut c = Vec::new();
    let receipt = complete(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &NativeDomEvent::style_load(target),
                limits(),
            )
            .unwrap(),
    );
    assert_eq!(receipt.callbacks(), 2);
    assert_eq!(c.len(), 2);
    assert_eq!(c[1], target);
    assert_eq!(
        f.dom
            .snapshot()
            .node(c[0])
            .unwrap()
            .attribute(&"stale".into()),
        None
    );
    assert!(matches!(
        listeners.dispatch(
            &mut f.dom,
            &mut c,
            &NativeDomEvent::style_load(target),
            limits()
        ),
        Err(DomEventDispatchError::Dom(_))
    ));
}

fn style_edit_limits() -> surgeist_dom::DomStyleEditLimits {
    surgeist_dom::DomStyleEditLimits::new(16_000_000, 100_000, 10_000_000, 64, 4096, 4096)
}
fn park_real_style_tail(dom: &mut Dom, document: DocumentId) -> surgeist_dom::DomStyleOperationId {
    let mut b = DomBatch::new();
    let style = b.create_element_ns(document, Namespace::html(), "style");
    let parent = dom
        .snapshot()
        .document(document)
        .unwrap()
        .document_element()
        .unwrap();
    b.insert_before(&style, parent, None);
    let mut lease = dom
        .begin_style_batch(b, style_edit_limits(), &AtomicBool::new(false))
        .unwrap();
    let operation = lease.operation();
    assert!(matches!(
        lease.advance(&AtomicBool::new(false)).unwrap(),
        surgeist_dom::DomStyleProgress::Request(_)
    ));
    assert_eq!(
        lease.snapshot().incomplete_source_operation(),
        Some(operation)
    );
    operation
}
fn finish_real_style_tail(dom: &mut Dom, operation: surgeist_dom::DomStyleOperationId) {
    let mut lease = dom
        .resume_style_edit(operation, style_edit_limits(), &AtomicBool::new(false))
        .unwrap();
    let surgeist_dom::DomStyleProgress::Request(request) =
        lease.advance(&AtomicBool::new(false)).unwrap()
    else {
        panic!("retained actual CSP request")
    };
    lease
        .accept_inline_csp(
            &request,
            &surgeist_dom::DomStyleInlineCspObservation::evaluated(
                request.source(),
                std::sync::Arc::new(vec!["style-src 'none'".into()]),
                19,
                false,
            ),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&AtomicBool::new(false)).unwrap(),
        surgeist_dom::DomStyleProgress::Complete(_)
    ));
}
#[test]
fn incomplete_source_before_dispatch_refuses_delivery_then_same_ready_event_can_start() {
    let mut f = ordinary();
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, n: &mut usize, _| {
                *n += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = NativeDomEvent::style_load(f.target);
    let operation = park_real_style_tail(&mut f.dom, f.document);
    let mut count = 0;
    assert!(matches!(
        listeners.dispatch(&mut f.dom, &mut count, &event, limits()),
        Err(DomEventDispatchError::Dom(
            DomError::IncompleteSourceOperation
        ))
    ));
    assert_eq!(count, 0);
    finish_real_style_tail(&mut f.dom, operation);
    complete(
        listeners
            .dispatch(&mut f.dom, &mut count, &event, limits())
            .unwrap(),
    );
    assert_eq!(count, 1);
}
#[test]
fn callback_published_source_tail_blocks_later_listener_until_exact_tail_finishes_without_replay() {
    let mut f = ordinary();
    let document = f.document;
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(
                move |dom, _, c: &mut Vec<surgeist_dom::DomStyleOperationId>, _| {
                    c.push(park_real_style_tail(dom.raw_mut(), document));
                    Ok(())
                },
            ),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(
                |dom, _, c: &mut Vec<surgeist_dom::DomStyleOperationId>, _| {
                    assert_eq!(dom.snapshot().incomplete_source_operation(), None);
                    c.push(c[0]);
                    Ok(())
                },
            ),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = NativeDomEvent::style_load(f.target);
    let mut c = Vec::new();
    let DomEventDispatchOutcome::RetainedSourceOperation(operation) = listeners
        .dispatch(&mut f.dom, &mut c, &event, limits())
        .unwrap()
    else {
        panic!("actual source tail must retain dispatch")
    };
    assert_eq!(c, [operation]);
    assert!(
        matches!(listeners.resume(&mut f.dom, &mut c, &event, limits()).unwrap(),
        DomEventDispatchOutcome::RetainedSourceOperation(id) if id == operation)
    );
    assert_eq!(c, [operation]);
    finish_real_style_tail(&mut f.dom, operation);
    let receipt = complete(
        listeners
            .resume(&mut f.dom, &mut c, &event, limits())
            .unwrap(),
    );
    assert_eq!(c, [operation, operation]);
    assert_eq!(receipt.callbacks(), 2);
}
#[test]
fn unfinished_hook_tail_after_last_listener_retains_event_cleanup_and_receipt() {
    let mut f = ordinary();
    let document = f.document;
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(move |dom, _, count: &mut usize, event| {
                *count += 1;
                park_real_style_tail(dom.raw_mut(), document);
                event.stop_immediate_propagation();
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = NativeDomEvent::style_load(f.target);
    let mut count = 0;
    let DomEventDispatchOutcome::RetainedSourceOperation(operation) = listeners
        .dispatch(&mut f.dom, &mut count, &event, limits())
        .unwrap()
    else {
        panic!("cleanup must wait for actual hook tail")
    };
    assert_eq!(count, 1);
    finish_real_style_tail(&mut f.dom, operation);
    let receipt = complete(
        listeners
            .resume(&mut f.dom, &mut count, &event, limits())
            .unwrap(),
    );
    assert_eq!(count, 1);
    assert_eq!(receipt.callbacks(), 1);
}

#[test]
fn failure_output_after_delivered_callback_is_retained_then_completed_without_replay() {
    let mut f = ordinary();
    let mut listeners = registry();
    for n in [1, 2] {
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, c: &mut Vec<u8>, _| {
                    c.push(n);
                    Err(DomEventCallbackError { code: u64::from(n) })
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let event = NativeDomEvent::style_load(f.target);
    let mut c = Vec::new();
    assert!(matches!(
        listeners
            .dispatch(
                &mut f.dom,
                &mut c,
                &event,
                DomEventDispatchLimits {
                    failure_records: 1,
                    ..limits()
                }
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::FailureRecords)
    ));
    assert_eq!(c, [1]);
    let receipt = complete(
        listeners
            .resume(&mut f.dom, &mut c, &event, limits())
            .unwrap(),
    );
    assert_eq!(c, [1, 2]);
    assert_eq!(receipt.failures().len(), 2);
    assert_eq!(receipt.callbacks(), 2);
}
#[test]
fn finite_work_exhaustion_after_callback_keeps_exact_next_listener_without_replay() {
    let mut f = ordinary();
    let mut listeners = registry();
    for n in [1, 2, 3] {
        listeners
            .add_listener(
                &DomEventDom::borrow(&mut f.dom),
                f.target,
                DomEventType::Load,
                DomEventCallback::new(move |_, _, c: &mut Vec<u8>, _| {
                    c.push(n);
                    Ok(())
                }),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    // The oracle is preservation of actual progress at every finite work cut,
    // independent of the implementation's current work threshold.
    let mut found_partial = false;
    for work in 0..256 {
        let event = NativeDomEvent::style_load(f.target);
        let mut c = Vec::new();
        let outcome = listeners.dispatch(
            &mut f.dom,
            &mut c,
            &event,
            DomEventDispatchLimits { work, ..limits() },
        );
        if matches!(
            outcome,
            Ok(DomEventDispatchOutcome::Retained(DomEventResource::Work))
        ) {
            let before = c.clone();
            let receipt = complete(
                listeners
                    .resume(&mut f.dom, &mut c, &event, limits())
                    .unwrap(),
            );
            assert_eq!(c, [1, 2, 3]);
            assert_eq!(receipt.callbacks(), 3);
            if !before.is_empty() && before.len() < 3 {
                found_partial = true;
            }
        }
    }
    assert!(
        found_partial,
        "finite work cuts must include an actual partial dispatch"
    );
}
struct PanicWhenDropped;
impl Drop for PanicWhenDropped {
    fn drop(&mut self) {
        panic!("panic payload destructor");
    }
}
#[test]
fn panic_payload_cleanup_unwind_retains_next_listener_and_unstrands_running_guard() {
    let mut f = ordinary();
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, c: &mut Vec<u8>, _| {
                c.push(1);
                std::panic::panic_any(PanicWhenDropped)
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, c: &mut Vec<u8>, _| {
                c.push(2);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut c = Vec::new();
    let event = NativeDomEvent::style_load(f.target);
    assert!(matches!(
        listeners
            .dispatch(&mut f.dom, &mut c, &event, limits())
            .unwrap(),
        DomEventDispatchOutcome::RetainedAfterUnwind
    ));
    assert_eq!(c, [1]);
    assert_eq!(event.current_target(), None);
    assert_eq!(event.phase(), DomEventPhase::None);
    let receipt = complete(
        listeners
            .resume(&mut f.dom, &mut c, &event, limits())
            .unwrap(),
    );
    assert_eq!(c, [1, 2]);
    assert_eq!(receipt.callbacks(), 2);
    assert_eq!(receipt.failures().len(), 1);
    assert_eq!(
        receipt.failures()[0].kind,
        DomEventListenerFailureKind::Panicked
    );
}
#[test]
fn retained_dispatch_refuses_foreign_canonical_owner_without_losing_progress() {
    let mut f = ordinary();
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, n: &mut usize, _| {
                *n += 1;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let event = NativeDomEvent::style_load(f.target);
    let mut count = 0;
    assert!(matches!(
        listeners
            .dispatch(
                &mut f.dom,
                &mut count,
                &event,
                DomEventDispatchLimits {
                    callbacks: 0,
                    ..limits()
                }
            )
            .unwrap(),
        DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
    ));
    let mut other = dom();
    assert!(matches!(
        listeners.resume(&mut other, &mut count, &event, limits()),
        Err(DomEventDispatchError::Dom(DomError::ForeignOwner))
    ));
    assert_eq!(count, 0);
    complete(
        listeners
            .resume(&mut f.dom, &mut count, &event, limits())
            .unwrap(),
    );
    assert_eq!(count, 1);
}

struct RegistryReplacement {
    original: Option<DomEventDispatcher<RegistryReplacement>>,
    calls: Vec<&'static str>,
}
#[test]
fn callback_registry_replacement_retains_original_dispatch_and_restored_registry_resumes_without_replay()
 {
    let mut f = ordinary();
    let target = f.target;
    let mut listeners = registry();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            f.document.node(),
            DomEventType::Load,
            DomEventCallback::new(move |dom, listeners, c: &mut RegistryReplacement, _| {
                c.calls.push("document capture");
                let mut b = DomBatch::new();
                b.set_attribute(target, "capture-delivered", "yes");
                apply(dom.raw_mut(), b);
                c.original = Some(std::mem::replace(listeners, registry()));
                listeners
                    .add_listener(
                        dom,
                        target,
                        DomEventType::Load,
                        DomEventCallback::new(move |dom, _, c: &mut RegistryReplacement, _| {
                            c.calls.push("replacement");
                            let mut b = DomBatch::new();
                            b.set_attribute(target, "replacement-delivered", "unexpected");
                            apply(dom.raw_mut(), b);
                            Ok(())
                        }),
                        DomEventListenerOptions::default(),
                    )
                    .unwrap();
                Ok(())
            }),
            DomEventListenerOptions {
                capture: true,
                once: false,
            },
        )
        .unwrap();
    listeners
        .add_listener(
            &DomEventDom::borrow(&mut f.dom),
            target,
            DomEventType::Load,
            DomEventCallback::new(move |dom, _, c: &mut RegistryReplacement, _| {
                c.calls.push("original target");
                let mut b = DomBatch::new();
                b.set_attribute(target, "original-delivered", "yes");
                apply(dom.raw_mut(), b);
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    let mut c = RegistryReplacement {
        original: None,
        calls: Vec::new(),
    };
    let event = NativeDomEvent::style_load(target);
    let outcome = listeners
        .dispatch(&mut f.dom, &mut c, &event, limits())
        .unwrap();
    assert_eq!(
        f.dom
            .snapshot()
            .node(target)
            .unwrap()
            .attribute(&"replacement-delivered".into()),
        None,
        "a replacement registry must not execute listeners on the original registry's dispatch path"
    );
    assert!(
        matches!(outcome, DomEventDispatchOutcome::RetainedForeignRegistry),
        "registry replacement must retain the already-started dispatch obligation"
    );
    assert!(matches!(
        listeners.resume(&mut f.dom, &mut c, &event, limits()),
        Err(DomEventDispatchError::WrongContinuation)
    ));
    assert_eq!(c.calls, ["document capture"]);
    assert_eq!(
        f.dom
            .snapshot()
            .node(target)
            .unwrap()
            .attribute(&"capture-delivered".into()),
        Some(&"yes".into())
    );
    let original = c.original.take().unwrap();
    let _replacement = std::mem::replace(&mut listeners, original);
    let receipt = complete(
        listeners
            .resume(&mut f.dom, &mut c, &event, limits())
            .unwrap(),
    );
    assert_eq!(c.calls, ["document capture", "original target"]);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert_eq!(
        f.dom
            .snapshot()
            .node(target)
            .unwrap()
            .attribute(&"original-delivered".into()),
        Some(&"yes".into())
    );
    assert_eq!(
        f.dom
            .snapshot()
            .node(target)
            .unwrap()
            .attribute(&"replacement-delivered".into()),
        None
    );
}
