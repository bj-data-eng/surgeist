//! Literal HTML877d task ordering, reentry and retirement through real DOM edits.
use super::*;
use crate::dom_event::{
    DomEventCallback, DomEventCallbackError, DomEventListenerOptions, DomEventPhase,
    DomEventRegistryLimits, DomEventResource, DomEventType, DomToggleEvent,
};
use crate::dom_lifecycle::DomHostedRuntimeLimits;
use crate::dom_style::{DomSession, DomSessionLimits, DomStyleRuntimeLimits};
use surgeist_dom::{DocumentId, Dom, DomBatch, DomCommit, DomLimits, Namespace, NodeId};

fn session(dom: Dom) -> DomSession {
    DomSession::new(
        dom,
        DomSessionLimits {
            source_events: crate::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 32,
                completions: 32,
                reports: 32,
                storage_bytes: 16 * 1024 * 1024,
                work: 1_000_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            element_tasks: limits(),
            hosted: DomHostedRuntimeLimits {
                canonical: surgeist_dom::DomHostedLimits::new(
                    surgeist_dom::DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    16 * 1024 * 1024,
                    16 * 1024 * 1024,
                    1_000_000,
                ),
                tasks: 32,
                storage_bytes: 16 * 1024 * 1024,
                scratch_bytes: 16 * 1024 * 1024,
                work: 1_000_000,
            },
        },
    )
    .unwrap()
}

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomElementTaskRuntimeLimits {
    DomElementTaskRuntimeLimits {
        tasks: 32,
        storage_bytes: 65536,
        work: 1_000_000,
    }
}
fn dispatch_limits() -> DomEventDispatchLimits {
    DomEventDispatchLimits {
        path_items: 64,
        snapshot_listeners: 64,
        failure_records: 64,
        callbacks: 64,
        work: 100_000,
    }
}
fn apply(dom: &mut DomSession, batch: DomBatch) -> DomCommit {
    dom.dom().apply(batch, &cancel()).unwrap()
}
fn fixture() -> (DomSession, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(
            256,
            65536,
            1024,
            1024,
            512,
            16,
            1_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            surgeist_dom::DomDispatchCustodyLimits::new(
                64, 4096, 16_000_000, 16_000_000, 4_000_000,
            )
            .unwrap(),
        )
        .with_details_toggle_intentions(128)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let a = batch.create_element_ns(&document, Namespace::html(), "details");
    let b = batch.create_element_ns(&document, Namespace::html(), "details");
    batch.insert_before(&a, &root, None);
    batch.insert_before(&b, &root, None);
    batch.insert_before(&root, &document, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    (
        session(dom),
        commit.document(&document).unwrap(),
        commit.created(&a).unwrap().unwrap(),
        commit.created(&b).unwrap().unwrap(),
    )
}
struct Context {
    element_tasks: DomElementTaskRuntime,
    log: Vec<(NodeId, DomToggleEvent, DomEventPhase)>,
    reenter: bool,
    refused_limit_change: bool,
    panic_next_access: bool,
    replacement_session: Option<DomSession>,
}
impl DomElementTaskEventContext for Context {
    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
        if std::mem::take(&mut self.panic_next_access) {
            panic!("actual Details context accessor unwind");
        }
        &mut self.element_tasks
    }
}
fn context(dom: &mut DomSession) -> Context {
    Context {
        element_tasks: dom.element_task_execution(),
        log: Vec::new(),
        reenter: false,
        refused_limit_change: false,
        panic_next_access: false,
        replacement_session: None,
    }
}
fn registry() -> DomEventDispatcher<Context> {
    DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 64,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap()
}
fn ingest(dom: &mut DomSession, context: &mut Context) {
    context
        .element_tasks
        .ingest(&mut dom.dom(), &cancel())
        .unwrap();
}
fn open(dom: &mut DomSession, target: NodeId) {
    let mut batch = DomBatch::new();
    batch.set_attribute(target, "open", "");
    apply(dom, batch);
}
fn close(dom: &mut DomSession, target: NodeId) {
    let mut batch = DomBatch::new();
    batch.remove_attribute(target, "open");
    apply(dom, batch);
}
fn record() -> DomEventCallback<Context> {
    DomEventCallback::new(|_, _, context: &mut Context, event| {
        // Assertions of actual delivery remain outside callbacks: dispatcher
        // records callback unwinds instead of allowing them to hide test failures.
        context.log.push((
            event.current_target().unwrap().node().unwrap(),
            *event.toggle_event().unwrap(),
            event.phase(),
        ));
        Ok(())
    })
}
fn complete(
    dom: &mut DomSession,
    registry: &mut DomEventDispatcher<Context>,
    c: &mut Context,
) -> (DomElementTask, DomEventDispatchReceipt) {
    match DomElementTaskRuntime::run_next(
        &mut dom.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        registry,
        c,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap()
    {
        DomElementTaskProgress::Complete {
            task,
            receipt,
            dialog_toggle_disposition,
            ..
        } => {
            assert_eq!(
                dialog_toggle_disposition, None,
                "Details has no Dialog cleanup result"
            );
            assert!(
                receipt.failures().is_empty(),
                "positive fixture listener failed: {:?}",
                receipt.failures()
            );
            (task, receipt)
        }
        other => panic!("expected actual delivery, got {other:?}"),
    }
}

#[test]
fn coalescing_preserves_first_old_latest_new_and_requeues_after_other_targets() {
    let (mut dom, document, a, b) = fixture();
    let mut c = context(&mut dom);
    // Three actual raw transitions fit two task slots because A is replaced.
    c.element_tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 2,
            ..limits()
        })
        .unwrap();
    let mut registry = registry();
    registry
        .add_listener(
            &dom.dom(),
            document.node(),
            DomEventType::Toggle,
            record(),
            DomEventListenerOptions {
                capture: true,
                once: false,
            },
        )
        .unwrap();
    for target in [a, b] {
        registry
            .add_listener(
                &dom.dom(),
                target,
                DomEventType::Toggle,
                record(),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "name", "shared");
    batch.set_attribute(b, "name", "shared");
    batch.set_attribute(a, "open", "");
    // Opening B emits B's notification before the real group hook closes A.
    batch.set_attribute(b, "open", "");
    apply(&mut dom, batch);
    ingest(&mut dom, &mut c);
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        2
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap()
            .id()
            .target(),
        b
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 1)
            .unwrap()
            .unwrap()
            .id()
            .target(),
        a
    );
    let (first, r1) = complete(&mut dom, &mut registry, &mut c);
    let (second, r2) = complete(&mut dom, &mut registry, &mut c);
    assert_eq!(
        first.details().unwrap().event().old_state(),
        HtmlDetailsState::Closed
    );
    assert_eq!(
        first.details().unwrap().event().new_state(),
        HtmlDetailsState::Open
    );
    assert_eq!(
        second.details().unwrap().event().old_state().as_str(),
        "closed"
    );
    assert_eq!(
        second.details().unwrap().event().new_state().as_str(),
        "closed"
    );
    assert_eq!(r1.callbacks(), 2);
    assert_eq!(r2.callbacks(), 2);
    assert!(r1.failures().is_empty() && r2.failures().is_empty());
    assert_eq!(
        c.log
            .iter()
            .map(|(id, _, phase)| (*id, *phase))
            .collect::<Vec<_>>(),
        vec![
            (document.node(), DomEventPhase::Capturing),
            (b, DomEventPhase::AtTarget),
            (document.node(), DomEventPhase::Capturing),
            (a, DomEventPhase::AtTarget)
        ]
    );
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn reentrant_replacement_is_untracked_by_postdispatch_clear_but_still_delivers() {
    let (mut dom, _, a, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    c.reenter = true;
    registry
        .add_listener(
            &dom.dom(),
            a,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, c: &mut Context, event| {
                c.log.push((
                    event.current_target().unwrap().node().unwrap(),
                    *event.toggle_event().unwrap(),
                    event.phase(),
                ));
                if c.reenter {
                    c.reenter = false;
                    c.refused_limit_change = c
                        .element_tasks
                        .set_limits(DomElementTaskRuntimeLimits {
                            tasks: 64,
                            ..limits()
                        })
                        .is_err();
                    let mut batch = DomBatch::new();
                    batch.remove_attribute(a, "open");
                    dom.apply(batch, &cancel()).unwrap();
                    c.element_tasks.ingest(dom, &cancel()).unwrap();
                }
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    let (_, receipt) = complete(&mut dom, &mut registry, &mut c);
    assert!(receipt.failures().is_empty());
    assert!(c.refused_limit_change);
    let replacement = c
        .element_tasks
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    assert_eq!(
        replacement.details().unwrap().event().old_state(),
        HtmlDetailsState::Closed
    );
    assert_eq!(
        replacement.details().unwrap().event().new_state(),
        HtmlDetailsState::Closed
    );
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        2
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap(),
        replacement
    );
    assert_ne!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 1)
            .unwrap()
            .unwrap()
            .id(),
        replacement.id()
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 1)
            .unwrap()
            .unwrap()
            .details()
            .unwrap()
            .event()
            .old_state(),
        HtmlDetailsState::Closed
    );
    complete(&mut dom, &mut registry, &mut c);
    complete(&mut dom, &mut registry, &mut c);
    assert_eq!(
        c.log
            .iter()
            .map(|(_, event, _)| (event.old_state(), event.new_state()))
            .collect::<Vec<_>>(),
        vec![("closed", "open"), ("closed", "closed"), ("closed", "open")]
    );
    // With all prior tasks finished, a new transition has no tracked old state.
    // It must use this actual Open→Closed transition, not the earlier Closed.
    close(&mut dom, a);
    ingest(&mut dom, &mut c);
    let (fresh_close, _) = complete(&mut dom, &mut registry, &mut c);
    assert_eq!(
        fresh_close.details().unwrap().event().old_state(),
        HtmlDetailsState::Open
    );
    assert_eq!(
        fresh_close.details().unwrap().event().new_state(),
        HtmlDetailsState::Closed
    );
    assert_eq!(c.log.len(), 4);
    assert_eq!(
        c.log[3].1.old_state(),
        fresh_close.details().unwrap().event().old_state().as_str()
    );
    assert_eq!(
        c.log[3].1.new_state(),
        fresh_close.details().unwrap().event().new_state().as_str()
    );
    assert_eq!(c.log[3].1.source(), None);
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn detach_and_adopt_preserve_target_but_retired_generation_is_skipped() {
    let (mut dom, _, a, b) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    for target in [a, b] {
        registry
            .add_listener(
                &dom.dom(),
                target,
                DomEventType::Toggle,
                record(),
                DomEventListenerOptions::default(),
            )
            .unwrap();
    }
    open(&mut dom, a);
    open(&mut dom, b);
    ingest(&mut dom, &mut c);
    let mut batch = DomBatch::new();
    let other = batch.create_document();
    batch.detach(a);
    batch.adopt(a, &other);
    batch.detach(b);
    batch.destroy(b);
    apply(&mut dom, batch);
    let (task, receipt) = complete(&mut dom, &mut registry, &mut c);
    assert_eq!(task.id().target(), a);
    assert!(receipt.failures().is_empty());
    let document = dom.snapshot().node(a).unwrap().owner_document().unwrap();
    let mut batch = DomBatch::new();
    let replacement = batch.create_element_ns(document, Namespace::html(), "details");
    let commit = apply(&mut dom, batch);
    let replacement = commit.created(&replacement).unwrap().unwrap();
    assert_ne!(replacement, b);
    match DomElementTaskRuntime::run_next(
        &mut dom.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut c,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap()
    {
        DomElementTaskProgress::SkippedTargetDestroyed(task) => assert_eq!(task.id().target(), b),
        other => panic!("{other:?}"),
    }
    assert_eq!(c.log.len(), 1);
    assert_eq!(c.log[0].0, a);
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn destruction_during_actual_listener_preserves_delivery_and_callback_failure_cleanup() {
    let (mut dom, _, a, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &dom.dom(),
            a,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, c: &mut Context, event| {
                c.log
                    .push((a, *event.toggle_event().unwrap(), event.phase()));
                let mut batch = DomBatch::new();
                batch.detach(a);
                batch.destroy(a);
                dom.apply(batch, &cancel()).unwrap();
                Err(DomEventCallbackError { code: 77 })
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    registry
        .add_listener(
            &dom.dom(),
            a,
            DomEventType::Toggle,
            record(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    let (task, receipt) = match DomElementTaskRuntime::run_next(
        &mut dom.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut c,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap()
    {
        DomElementTaskProgress::Complete { task, receipt, .. } => (task, receipt),
        other => panic!("expected real delivery with reported failure: {other:?}"),
    };
    assert_eq!(task.id().target(), a);
    assert_eq!(receipt.callbacks(), 2);
    assert_eq!(receipt.failures().len(), 1);
    assert_eq!(c.log.len(), 2);
    assert!(matches!(dom.snapshot().node(a), Err(DomError::StaleNode)));
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn intake_refusal_returns_whole_transfer_without_losing_prior_order_or_identity() {
    let (mut dom, _, a, b) = fixture();
    let mut c = context(&mut dom);
    c.element_tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 1,
            ..limits()
        })
        .unwrap();
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    let first = c
        .element_tasks
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    open(&mut dom, b);
    close(&mut dom, a);
    let transfer = dom
        .dom()
        .take_details_toggle_intentions(128, &cancel())
        .unwrap();
    let ops = transfer
        .intentions()
        .iter()
        .map(|i| i.operation())
        .collect::<Vec<_>>();
    let rejected = c
        .element_tasks
        .ingest_transfer(&mut dom.dom(), transfer, &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomElementTaskRuntimeError::Admission
    ));
    let transfer = rejected.into_transfer();
    assert_eq!(
        transfer
            .intentions()
            .iter()
            .map(|i| i.operation())
            .collect::<Vec<_>>(),
        ops
    );
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        1
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(first)
    );
    // Increase the same owner's finite bound: the original pending A tracker
    // still supplies the first closed state when the exact transfer is retried.
    c.element_tasks.set_limits(limits()).unwrap();
    assert_eq!(
        c.element_tasks
            .ingest_transfer(&mut dom.dom(), transfer, &cancel())
            .unwrap(),
        2
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap()
            .id()
            .target(),
        b
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 1)
            .unwrap()
            .unwrap()
            .details()
            .unwrap()
            .event()
            .old_state(),
        HtmlDetailsState::Closed
    );
    assert_eq!(
        c.element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 1)
            .unwrap()
            .unwrap()
            .details()
            .unwrap()
            .event()
            .new_state(),
        HtmlDetailsState::Closed
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "");
    apply(&mut dom, batch);
    let transfer = dom
        .dom()
        .take_details_toggle_intentions(128, &cancel())
        .unwrap();
    let stopped = AtomicBool::new(true);
    let rejected = c
        .element_tasks
        .ingest_transfer(&mut dom.dom(), transfer, &stopped)
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomElementTaskRuntimeError::Dom(DomError::Cancelled)
    ));
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        2
    );
    c.element_tasks
        .ingest_transfer(&mut dom.dom(), rejected.into_transfer(), &cancel())
        .unwrap();
}

#[test]
fn retained_dispatch_survives_callback_coordinator_replacement_and_resumes_once() {
    let (mut dom, _, a, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &dom.dom(),
            a,
            DomEventType::Toggle,
            DomEventCallback::new(move |_dom, _, c: &mut Context, event| {
                c.log
                    .push((a, *event.toggle_event().unwrap(), event.phase()));
                let (replacement_session, _, _, _) = fixture();
                let replacement = replacement_session.element_task_execution();
                c.replacement_session = Some(replacement_session);
                drop(std::mem::replace(&mut c.element_tasks, replacement));
                Ok(())
            }),
            DomEventListenerOptions {
                once: true,
                capture: false,
            },
        )
        .unwrap();
    registry
        .add_listener(
            &dom.dom(),
            a,
            DomEventType::Toggle,
            record(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    let original = c
        .element_tasks
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let mut progress = DomElementTaskRuntime::run_next(
        &mut dom.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut c,
        crate::DomElementTaskRunLimits {
            dispatch: DomEventDispatchLimits {
                callbacks: 1,
                ..dispatch_limits()
            },
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap();
    assert!(matches!(
        &progress,
        DomElementTaskProgress::Retained {
            dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks),
            ..
        }
    ));
    assert_eq!(c.log.len(), 1);
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    let original_owner = match &mut progress {
        DomElementTaskProgress::Retained {
            displaced_owner, ..
        } => displaced_owner
            .take()
            .expect("original owner returned after callback dropped it"),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        original_owner
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(original)
    );
    c.element_tasks = original_owner;
    let (task, receipt) = complete(&mut dom, &mut registry, &mut c);
    assert_eq!(task, original);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert_eq!(c.log.len(), 2);
    assert_eq!(
        c.element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn real_transfer_qualification_and_finite_banks_fail_without_partial_intake() {
    let (mut dom, _, a, b) = fixture();
    open(&mut dom, a);
    let older = dom
        .dom()
        .take_details_toggle_intentions(128, &cancel())
        .unwrap();
    open(&mut dom, b);
    let newer = dom
        .dom()
        .take_details_toggle_intentions(128, &cancel())
        .unwrap();
    let mut owner = dom.element_task_execution();
    owner
        .ingest_transfer(&mut dom.dom(), newer, &cancel())
        .unwrap();
    let kept = owner
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let rejected = owner
        .ingest_transfer(&mut dom.dom(), older, &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomElementTaskRuntimeError::OutOfOrderTransfer
    ));
    assert_eq!(
        owner
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(kept)
    );
    let (mut foreign, _, _, _) = fixture();
    let mut foreign_owner = foreign.element_task_execution();
    let rejected = foreign_owner
        .ingest_transfer(&mut foreign.dom(), rejected.into_transfer(), &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomElementTaskRuntimeError::Binding
    ));
    assert_eq!(
        foreign_owner
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );

    // The actual same durable owner retains its transfer across resource refusal.
    let (mut limited_dom, _, target, _) = fixture();
    let mut limited = limited_dom.element_task_execution();
    limited
        .set_limits(DomElementTaskRuntimeLimits {
            storage_bytes: ElementTaskState::fixed_bytes().unwrap(),
            ..limits()
        })
        .unwrap();
    open(&mut limited_dom, target);
    let transfer = limited_dom
        .dom()
        .take_details_toggle_intentions(128, &cancel())
        .unwrap();
    let rejected = limited
        .ingest_transfer(&mut limited_dom.dom(), transfer, &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomElementTaskRuntimeError::Admission
    ));
    assert_eq!(
        limited
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    let spent = limited.work_used().unwrap();
    limited
        .set_limits(DomElementTaskRuntimeLimits {
            work: spent,
            ..limits()
        })
        .unwrap();
    let rejected = limited
        .ingest_transfer(&mut limited_dom.dom(), rejected.into_transfer(), &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomElementTaskRuntimeError::Admission
    ));
    assert_eq!(
        limited
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert_eq!(limited.work_used().unwrap(), spent);
    limited.set_limits(limits()).unwrap();
    limited
        .ingest_transfer(&mut limited_dom.dom(), rejected.into_transfer(), &cancel())
        .unwrap();
    assert_eq!(
        limited
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap()
            .unwrap()
            .id()
            .target(),
        target
    );

    // Exercise actual checked issuance on a separate original owner, without
    // resetting a consumed serial or reconstructing the limited owner above.
    let (mut exhausted_dom, _, exhausted_target, _) = fixture();
    let mut exhausted = exhausted_dom.element_task_execution();
    exhausted.state().unwrap().borrow_mut().next = u64::MAX;
    open(&mut exhausted_dom, exhausted_target);
    let transfer = exhausted_dom
        .dom()
        .take_details_toggle_intentions(128, &cancel())
        .unwrap();
    let rejected = exhausted
        .ingest_transfer(&mut exhausted_dom.dom(), transfer, &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomElementTaskRuntimeError::IdentityExhausted
    ));
    assert_eq!(
        exhausted
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn empty_zero_task_and_work_bounds_need_no_task_cleanup_admission() {
    let (mut dom, _, _, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    c.element_tasks
        .set_limits(DomElementTaskRuntimeLimits {
            tasks: 0,
            storage_bytes: ElementTaskState::fixed_bytes().unwrap(),
            work: 0,
        })
        .unwrap();
    assert!(matches!(
        DomElementTaskRuntime::run_next(
            &mut dom.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut c,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                )
            },
            &cancel()
        )
        .unwrap(),
        DomElementTaskProgress::Empty
    ));
    assert_eq!(c.element_tasks.work_used().unwrap(), 0);
}

#[test]
fn accessor_unwind_after_completed_dispatch_keeps_actual_receipt_and_releases_running_without_replay()
 {
    let (mut dom, _, target, _) = fixture();
    let mut context = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &dom.dom(),
            target,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut Context, event| {
                context.log.push((
                    event.current_target().unwrap().node().unwrap(),
                    *event.toggle_event().unwrap(),
                    event.phase(),
                ));
                context.panic_next_access = true;
                Ok(())
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, target);
    ingest(&mut dom, &mut context);
    let original = context
        .element_tasks
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        DomElementTaskRuntime::run_next(
            &mut dom.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: dispatch_limits(),
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        )
    }));
    assert!(failed.is_err());
    assert_eq!(context.log.len(), 1);
    assert_eq!(
        context
            .element_tasks
            .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
            .unwrap(),
        Some(original)
    );
    let (task, receipt) = complete(&mut dom, &mut registry, &mut context);
    assert_eq!(task, original);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert_eq!(context.log.len(), 1);
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn target_destroyed_after_prestart_dispatch_refusal_is_skipped_without_delivery() {
    let (mut dom, _, target, _) = fixture();
    let mut context = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &dom.dom(),
            target,
            DomEventType::Toggle,
            record(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, target);
    ingest(&mut dom, &mut context);
    let original = context
        .element_tasks
        .pending_task(surgeist_dom::DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let mut low = dispatch_limits();
    low.work = 0;
    assert!(matches!(
        DomElementTaskRuntime::run_next(
            &mut dom.dom(),
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: low,
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64
                )
            },
            &cancel()
        ),
        Err(DomElementTaskRuntimeError::Dispatch(
            DomEventDispatchError::Admission(DomEventResource::Work)
        ))
    ));
    let mut batch = DomBatch::new();
    batch.detach(target);
    batch.destroy(target);
    apply(&mut dom, batch);
    let result = DomElementTaskRuntime::run_next(
        &mut dom.dom(),
        surgeist_dom::DomElementTaskSourceKind::DomManipulation,
        &mut registry,
        &mut context,
        crate::DomElementTaskRunLimits {
            dispatch: dispatch_limits(),
            source: surgeist_dom::DomSourceLimits::new(
                16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
            ),
        },
        &cancel(),
    )
    .unwrap();
    assert!(
        matches!(result,DomElementTaskProgress::SkippedTargetDestroyed(task) if task==original)
    );
    assert!(context.log.is_empty());
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
}

#[test]
fn session_bootstrap_distinguishes_actual_nonempty_raw_transfer_from_empty_or_refused_take() {
    for route in 0..3 {
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                4096,
                128,
                128,
                128,
                8,
                100_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_dispatch_custody(
                surgeist_dom::DomDispatchCustodyLimits::new(
                    64, 4096, 16_000_000, 16_000_000, 4_000_000,
                )
                .unwrap(),
            )
            .with_details_toggle_intentions(8)
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let details = batch.create_element_ns(&document, Namespace::html(), "details");
        if route != 0 {
            batch.set_attribute(&details, "open", "");
        }
        let commit = dom.apply(batch, &cancel()).unwrap();
        let target = commit.created(&details).unwrap().unwrap();
        let transfer = if route == 1 {
            assert!(dom.take_details_toggle_intentions(0, &cancel()).is_err());
            None
        } else {
            Some(dom.take_details_toggle_intentions(8, &cancel()).unwrap())
        };
        let owner = dom.owner();
        let stamp = dom.snapshot().stamp();
        let pending = dom.pending_details_toggle_intentions();
        let limits = DomSessionLimits {
            source_events: crate::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 8,
                completions: 8,
                reports: 8,
                storage_bytes: 16 * 1024 * 1024,
                work: 1_000_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            element_tasks: self::limits(),
            hosted: DomHostedRuntimeLimits {
                canonical: surgeist_dom::DomHostedLimits::new(
                    surgeist_dom::DomHostedRecordLimits::new(8, 8, 8, 8, 8, 8, 8),
                    16 * 1024 * 1024,
                    16 * 1024 * 1024,
                    1_000_000,
                ),
                tasks: 8,
                storage_bytes: 16 * 1024 * 1024,
                scratch_bytes: 16 * 1024 * 1024,
                work: 1_000_000,
            },
        };
        if route == 2 {
            let transfer = transfer.expect("the genuine standalone transfer remains caller-owned");
            assert_eq!(transfer.len(), 1);
            assert_eq!(transfer.intentions()[0].target(), target);
            let rejected = DomSession::new(dom, limits).unwrap_err();
            assert!(matches!(
                rejected.error(),
                crate::dom_style::DomSessionCreateFailure::ElementTasks(
                    DomElementTaskRuntimeError::PretransferredSession
                )
            ));
            let dom = rejected.into_dom();
            assert_eq!(dom.owner(), owner);
            assert_eq!(dom.snapshot().stamp(), stamp);
            assert_eq!(dom.pending_details_toggle_intentions(), pending);
            assert_eq!(transfer.intentions()[0].target(), target);
        } else {
            let session = DomSession::new(dom, limits).unwrap();
            assert_eq!(session.snapshot().owner(), owner);
            assert_eq!(session.snapshot().stamp(), stamp);
            assert_eq!(
                session
                    .snapshot()
                    .node(target)
                    .unwrap()
                    .as_details()
                    .unwrap()
                    .state(),
                if route == 0 {
                    HtmlDetailsState::Closed
                } else {
                    HtmlDetailsState::Open
                }
            );
        }
    }
}

mod queue_tests;

mod close_tests;
