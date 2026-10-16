//! Literal HTML877d task ordering, reentry and retirement through real DOM edits.
use super::*;
use crate::dom_event::{
    DomEventCallback, DomEventCallbackError, DomEventListenerOptions, DomEventPhase,
    DomEventRegistryLimits, DomEventResource, DomEventType,
};
use surgeist_dom::{DocumentId, Dom, DomBatch, DomCommit, DomLimits, Namespace};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomDetailsRuntimeLimits {
    DomDetailsRuntimeLimits {
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
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId) {
    let mut dom = Dom::new(
        DomLimits::new(256, 65536, 1024, 1024, 512, 16, 1_000_000)
            .unwrap()
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
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&a).unwrap().unwrap(),
        commit.created(&b).unwrap().unwrap(),
    )
}
struct Context {
    details: DomDetailsRuntime,
    log: Vec<(NodeId, HtmlDetailsToggleEvent, DomEventPhase)>,
    reenter: bool,
    refused_limit_change: bool,
    panic_next_access: bool,
}
impl DomDetailsEventContext for Context {
    fn details(&mut self) -> &mut DomDetailsRuntime {
        if std::mem::take(&mut self.panic_next_access) {
            panic!("actual Details context accessor unwind");
        }
        &mut self.details
    }
}
fn context(dom: &mut Dom) -> Context {
    Context {
        details: DomDetailsRuntime::new(&DomEventDom::borrow(dom), limits()).unwrap(),
        log: Vec::new(),
        reenter: false,
        refused_limit_change: false,
        panic_next_access: false,
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
fn ingest(dom: &mut Dom, context: &mut Context) {
    let transfer = dom.take_details_toggle_intentions(128, &cancel()).unwrap();
    context.details.ingest(transfer, &cancel()).unwrap();
}
fn open(dom: &mut Dom, target: NodeId) {
    let mut batch = DomBatch::new();
    batch.set_attribute(target, "open", "");
    apply(dom, batch);
}
fn close(dom: &mut Dom, target: NodeId) {
    let mut batch = DomBatch::new();
    batch.remove_attribute(target, "open");
    apply(dom, batch);
}
fn record() -> DomEventCallback<Context> {
    DomEventCallback::new(|_, _, context: &mut Context, event| {
        // Assertions of actual delivery remain outside callbacks: dispatcher
        // records callback unwinds instead of allowing them to hide test failures.
        context.log.push((
            event.current_target().unwrap(),
            *event.toggle_event().unwrap(),
            event.phase(),
        ));
        Ok(())
    })
}
fn complete(
    dom: &mut Dom,
    registry: &mut DomEventDispatcher<Context>,
    c: &mut Context,
) -> (DomDetailsTask, DomEventDispatchReceipt) {
    match DomDetailsRuntime::run_next(
        &mut DomEventDom::borrow(dom),
        registry,
        c,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomDetailsProgress::Complete { task, receipt, .. } => {
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
    c.details
        .set_limits(DomDetailsRuntimeLimits {
            tasks: 2,
            ..limits()
        })
        .unwrap();
    let mut registry = registry();
    registry
        .add_listener(
            &DomEventDom::borrow(&mut dom),
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
                &DomEventDom::borrow(&mut dom),
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
    assert_eq!(c.details.pending_tasks(), 2);
    assert_eq!(c.details.pending_task(0).unwrap().id().target(), b);
    assert_eq!(c.details.pending_task(1).unwrap().id().target(), a);
    let (first, r1) = complete(&mut dom, &mut registry, &mut c);
    let (second, r2) = complete(&mut dom, &mut registry, &mut c);
    assert_eq!(first.event().old_state(), HtmlDetailsState::Closed);
    assert_eq!(first.event().new_state(), HtmlDetailsState::Open);
    assert_eq!(second.event().old_state().as_str(), "closed");
    assert_eq!(second.event().new_state().as_str(), "closed");
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
    assert_eq!(c.details.pending_tasks(), 0);
}

#[test]
fn reentrant_replacement_is_untracked_by_postdispatch_clear_but_still_delivers() {
    let (mut dom, _, a, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    c.reenter = true;
    registry
        .add_listener(
            &DomEventDom::borrow(&mut dom),
            a,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, c: &mut Context, event| {
                c.log.push((
                    event.current_target().unwrap(),
                    *event.toggle_event().unwrap(),
                    event.phase(),
                ));
                if c.reenter {
                    c.reenter = false;
                    c.refused_limit_change = c
                        .details
                        .set_limits(DomDetailsRuntimeLimits {
                            tasks: 64,
                            ..limits()
                        })
                        .is_err();
                    close(dom.raw_mut(), a);
                    ingest(dom.raw_mut(), c);
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
    let replacement = c.details.pending_task(0).unwrap();
    assert_eq!(replacement.event().old_state(), HtmlDetailsState::Closed);
    assert_eq!(replacement.event().new_state(), HtmlDetailsState::Closed);
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    assert_eq!(c.details.pending_tasks(), 2);
    assert_eq!(c.details.pending_task(0).unwrap(), replacement);
    assert_ne!(c.details.pending_task(1).unwrap().id(), replacement.id());
    assert_eq!(
        c.details.pending_task(1).unwrap().event().old_state(),
        HtmlDetailsState::Closed
    );
    complete(&mut dom, &mut registry, &mut c);
    complete(&mut dom, &mut registry, &mut c);
    assert_eq!(
        c.log
            .iter()
            .map(|(_, event, _)| (event.old_state(), event.new_state()))
            .collect::<Vec<_>>(),
        vec![
            (HtmlDetailsState::Closed, HtmlDetailsState::Open),
            (HtmlDetailsState::Closed, HtmlDetailsState::Closed),
            (HtmlDetailsState::Closed, HtmlDetailsState::Open)
        ]
    );
    // With all prior tasks finished, a new transition has no tracked old state.
    // It must use this actual Open→Closed transition, not the earlier Closed.
    close(&mut dom, a);
    ingest(&mut dom, &mut c);
    let (fresh_close, _) = complete(&mut dom, &mut registry, &mut c);
    assert_eq!(fresh_close.event().old_state(), HtmlDetailsState::Open);
    assert_eq!(fresh_close.event().new_state(), HtmlDetailsState::Closed);
    assert_eq!(c.log.len(), 4);
    assert_eq!(c.log[3].1, fresh_close.event());
    assert_eq!(c.details.pending_tasks(), 0);
}

#[test]
fn detach_and_adopt_preserve_target_but_retired_generation_is_skipped() {
    let (mut dom, _, a, b) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    for target in [a, b] {
        registry
            .add_listener(
                &DomEventDom::borrow(&mut dom),
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
    match DomDetailsRuntime::run_next(
        &mut DomEventDom::borrow(&mut dom),
        &mut registry,
        &mut c,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomDetailsProgress::SkippedTargetDestroyed(task) => assert_eq!(task.id().target(), b),
        other => panic!("{other:?}"),
    }
    assert_eq!(c.log.len(), 1);
    assert_eq!(c.log[0].0, a);
    assert_eq!(c.details.pending_tasks(), 0);
}

#[test]
fn destruction_during_actual_listener_preserves_delivery_and_callback_failure_cleanup() {
    let (mut dom, _, a, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &DomEventDom::borrow(&mut dom),
            a,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, c: &mut Context, event| {
                c.log
                    .push((a, *event.toggle_event().unwrap(), event.phase()));
                let mut batch = DomBatch::new();
                batch.detach(a);
                batch.destroy(a);
                apply(dom.raw_mut(), batch);
                Err(DomEventCallbackError { code: 77 })
            }),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    registry
        .add_listener(
            &DomEventDom::borrow(&mut dom),
            a,
            DomEventType::Toggle,
            record(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    let (task, receipt) = match DomDetailsRuntime::run_next(
        &mut DomEventDom::borrow(&mut dom),
        &mut registry,
        &mut c,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
    {
        DomDetailsProgress::Complete { task, receipt, .. } => (task, receipt),
        other => panic!("expected real delivery with reported failure: {other:?}"),
    };
    assert_eq!(task.id().target(), a);
    assert_eq!(receipt.callbacks(), 2);
    assert_eq!(receipt.failures().len(), 1);
    assert_eq!(c.log.len(), 2);
    assert!(matches!(dom.snapshot().node(a), Err(DomError::StaleNode)));
    assert_eq!(c.details.pending_tasks(), 0);
}

#[test]
fn intake_refusal_returns_whole_transfer_without_losing_prior_order_or_identity() {
    let (mut dom, _, a, b) = fixture();
    let mut c = context(&mut dom);
    c.details = DomDetailsRuntime::new(
        &DomEventDom::borrow(&mut dom),
        DomDetailsRuntimeLimits {
            tasks: 1,
            ..limits()
        },
    )
    .unwrap();
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    let first = c.details.pending_task(0).unwrap();
    open(&mut dom, b);
    close(&mut dom, a);
    let transfer = dom.take_details_toggle_intentions(128, &cancel()).unwrap();
    let ops = transfer
        .intentions()
        .iter()
        .map(|i| i.operation())
        .collect::<Vec<_>>();
    let rejected = c.details.ingest(transfer, &cancel()).unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomDetailsRuntimeError::Admission
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
    assert_eq!(c.details.pending_tasks(), 1);
    assert_eq!(c.details.pending_task(0), Some(first));
    // Increase the same owner's finite bound: the original pending A tracker
    // still supplies the first closed state when the exact transfer is retried.
    c.details.set_limits(limits()).unwrap();
    assert_eq!(c.details.ingest(transfer, &cancel()).unwrap(), 2);
    assert_eq!(c.details.pending_task(0).unwrap().id().target(), b);
    assert_eq!(
        c.details.pending_task(1).unwrap().event().old_state(),
        HtmlDetailsState::Closed
    );
    assert_eq!(
        c.details.pending_task(1).unwrap().event().new_state(),
        HtmlDetailsState::Closed
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "open", "");
    apply(&mut dom, batch);
    let transfer = dom.take_details_toggle_intentions(128, &cancel()).unwrap();
    let stopped = AtomicBool::new(true);
    let rejected = c.details.ingest(transfer, &stopped).unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomDetailsRuntimeError::Dom(DomError::Cancelled)
    ));
    assert_eq!(c.details.pending_tasks(), 2);
    c.details
        .ingest(rejected.into_transfer(), &cancel())
        .unwrap();
}

#[test]
fn retained_dispatch_survives_callback_coordinator_replacement_and_resumes_once() {
    let (mut dom, _, a, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &DomEventDom::borrow(&mut dom),
            a,
            DomEventType::Toggle,
            DomEventCallback::new(move |dom, _, c: &mut Context, event| {
                c.log
                    .push((a, *event.toggle_event().unwrap(), event.phase()));
                let replacement = DomDetailsRuntime::new(dom, limits()).unwrap();
                drop(std::mem::replace(&mut c.details, replacement));
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
            &DomEventDom::borrow(&mut dom),
            a,
            DomEventType::Toggle,
            record(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, a);
    ingest(&mut dom, &mut c);
    let original = c.details.pending_task(0).unwrap();
    let mut progress = DomDetailsRuntime::run_next(
        &mut DomEventDom::borrow(&mut dom),
        &mut registry,
        &mut c,
        DomEventDispatchLimits {
            callbacks: 1,
            ..dispatch_limits()
        },
        &cancel(),
    )
    .unwrap();
    assert!(matches!(
        &progress,
        DomDetailsProgress::Retained {
            dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks),
            ..
        }
    ));
    assert_eq!(c.log.len(), 1);
    assert_eq!(c.details.pending_tasks(), 0);
    let original_owner = match &mut progress {
        DomDetailsProgress::Retained {
            displaced_owner, ..
        } => displaced_owner
            .take()
            .expect("original owner returned after callback dropped it"),
        other => panic!("{other:?}"),
    };
    assert_eq!(original_owner.pending_task(0), Some(original));
    c.details = original_owner;
    let (task, receipt) = complete(&mut dom, &mut registry, &mut c);
    assert_eq!(task, original);
    assert_eq!(receipt.callbacks(), 2);
    assert!(receipt.failures().is_empty());
    assert_eq!(c.log.len(), 2);
    assert_eq!(c.details.pending_tasks(), 0);
}

#[test]
fn real_transfer_qualification_and_finite_banks_fail_without_partial_intake() {
    let (mut dom, _, a, b) = fixture();
    open(&mut dom, a);
    let older = dom.take_details_toggle_intentions(128, &cancel()).unwrap();
    open(&mut dom, b);
    let newer = dom.take_details_toggle_intentions(128, &cancel()).unwrap();
    let mut owner = DomDetailsRuntime::new(&DomEventDom::borrow(&mut dom), limits()).unwrap();
    owner.ingest(newer, &cancel()).unwrap();
    let kept = owner.pending_task(0).unwrap();
    let rejected = owner.ingest(older, &cancel()).unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomDetailsRuntimeError::OutOfOrderTransfer
    ));
    assert_eq!(owner.pending_task(0), Some(kept));
    let transfer = rejected.into_transfer();
    let (mut foreign, _, _, _) = fixture();
    let mut foreign_owner =
        DomDetailsRuntime::new(&DomEventDom::borrow(&mut foreign), limits()).unwrap();
    let rejected = foreign_owner.ingest(transfer, &cancel()).unwrap_err();
    assert!(matches!(rejected.error(), DomDetailsRuntimeError::Binding));
    let mut no_storage = DomDetailsRuntime::new(
        &DomEventDom::borrow(&mut dom),
        DomDetailsRuntimeLimits {
            storage_bytes: 0,
            ..limits()
        },
    )
    .unwrap();
    let rejected = no_storage
        .ingest(rejected.into_transfer(), &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomDetailsRuntimeError::Admission
    ));
    assert_eq!(no_storage.pending_tasks(), 0);
    let mut no_work = DomDetailsRuntime::new(
        &DomEventDom::borrow(&mut dom),
        DomDetailsRuntimeLimits {
            work: 1,
            ..limits()
        },
    )
    .unwrap();
    let rejected = no_work
        .ingest(rejected.into_transfer(), &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomDetailsRuntimeError::Admission
    ));
    assert_eq!(no_work.pending_tasks(), 0);
    let mut exhausted = DomDetailsRuntime::new(&DomEventDom::borrow(&mut dom), limits()).unwrap();
    exhausted.state.borrow_mut().next = u64::MAX;
    let rejected = exhausted
        .ingest(rejected.into_transfer(), &cancel())
        .unwrap_err();
    assert!(matches!(
        rejected.error(),
        DomDetailsRuntimeError::IdentityExhausted
    ));
    assert_eq!(exhausted.pending_tasks(), 0);
    let mut admitted = DomDetailsRuntime::new(&DomEventDom::borrow(&mut dom), limits()).unwrap();
    admitted
        .ingest(rejected.into_transfer(), &cancel())
        .unwrap();
    assert_eq!(admitted.pending_task(0).unwrap().id().target(), a);
}

#[test]
fn empty_zero_bound_owner_needs_no_task_cleanup_admission() {
    let (mut dom, _, _, _) = fixture();
    let mut c = context(&mut dom);
    let mut registry = registry();
    c.details = DomDetailsRuntime::new(
        &DomEventDom::borrow(&mut dom),
        DomDetailsRuntimeLimits {
            tasks: 0,
            storage_bytes: 0,
            work: 0,
        },
    )
    .unwrap();
    assert!(matches!(
        DomDetailsRuntime::run_next(
            &mut DomEventDom::borrow(&mut dom),
            &mut registry,
            &mut c,
            dispatch_limits(),
            &cancel()
        )
        .unwrap(),
        DomDetailsProgress::Empty
    ));
    assert_eq!(c.details.work_used(), 0);
}

#[test]
fn accessor_unwind_after_completed_dispatch_keeps_actual_receipt_and_releases_running_without_replay()
 {
    let (mut dom, _, target, _) = fixture();
    let mut context = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &DomEventDom::borrow(&mut dom),
            target,
            DomEventType::Toggle,
            DomEventCallback::new(|_, _, context: &mut Context, event| {
                context.log.push((
                    event.current_target().unwrap(),
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
    let original = context.details.pending_task(0).unwrap();
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        DomDetailsRuntime::run_next(
            &mut DomEventDom::borrow(&mut dom),
            &mut registry,
            &mut context,
            dispatch_limits(),
            &cancel(),
        )
    }));
    assert!(failed.is_err());
    assert_eq!(context.log.len(), 1);
    assert_eq!(context.details.pending_task(0), Some(original));
    let (task, receipt) = complete(&mut dom, &mut registry, &mut context);
    assert_eq!(task, original);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert_eq!(context.log.len(), 1);
    assert_eq!(context.details.pending_tasks(), 0);
}

#[test]
fn target_destroyed_after_prestart_dispatch_refusal_is_skipped_without_delivery() {
    let (mut dom, _, target, _) = fixture();
    let mut context = context(&mut dom);
    let mut registry = registry();
    registry
        .add_listener(
            &DomEventDom::borrow(&mut dom),
            target,
            DomEventType::Toggle,
            record(),
            DomEventListenerOptions::default(),
        )
        .unwrap();
    open(&mut dom, target);
    ingest(&mut dom, &mut context);
    let original = context.details.pending_task(0).unwrap();
    let mut low = dispatch_limits();
    low.work = 0;
    assert!(matches!(
        DomDetailsRuntime::run_next(
            &mut DomEventDom::borrow(&mut dom),
            &mut registry,
            &mut context,
            low,
            &cancel()
        ),
        Err(DomDetailsRuntimeError::Dispatch(
            DomEventDispatchError::Admission(DomEventResource::Work)
        ))
    ));
    let mut batch = DomBatch::new();
    batch.detach(target);
    batch.destroy(target);
    apply(&mut dom, batch);
    let result = DomDetailsRuntime::run_next(
        &mut DomEventDom::borrow(&mut dom),
        &mut registry,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap();
    assert!(matches!(result,DomDetailsProgress::SkippedTargetDestroyed(task) if task==original));
    assert!(context.log.is_empty());
    assert_eq!(context.details.pending_tasks(), 0);
}
