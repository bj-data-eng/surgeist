#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    future::{Future, pending},
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    rc::Rc,
    task::{Context as TaskContext, Poll, Waker},
};

use surgeist_js::{
    Context, JsError, JsNativeError, JsValue, NativeFunction, Script, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{
        EventFilter, ExecutionOptions, ExecutionReceipt, ExecutionSetupError, ExecutionTermination,
        ProfileMode,
    },
    job::SimpleJobExecutor,
    js_string,
};

fn poll<F: Future + Unpin>(future: &mut F) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(Waker::noop()))
}

fn complete<F: Future + Unpin>(future: &mut F) -> (F::Output, usize) {
    for pending_polls in 0..4096 {
        if let Poll::Ready(value) = poll(future) {
            return (value, pending_polls);
        }
    }
    panic!("execution was still pending after 4096 cooperative polls");
}

fn instruction_error(limit: u64) -> EngineError {
    EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit })
}

#[test]
fn synchronous_scope_returns_generic_values_and_reentry_uses_the_same_owner() {
    let mut context = Context::builder().build().unwrap();
    let denied_callback = Cell::new(false);
    let outcome = context
        .with_execution(ExecutionOptions::new(128), |mut context| {
            assert!(matches!(
                context.with_execution(ExecutionOptions::new(128), |_| {
                    denied_callback.set(true);
                    Ok(99_u32)
                }),
                Err(ExecutionSetupError::AlreadyActive)
            ));
            assert!(matches!(
                context.execution_task(ExecutionOptions::new(128), async |_| Ok(99_u32)),
                Err(ExecutionSetupError::AlreadyActive)
            ));
            assert!(matches!(
                context.eval_with_budget(Source::from_bytes("0"), ExecutionOptions::new(128)),
                Err(ExecutionSetupError::AlreadyActive)
            ));
            assert_eq!(
                context.eval(Source::from_bytes("20 + 22"))?,
                JsValue::new(42)
            );
            Ok(42_u32)
        })
        .unwrap();
    assert_eq!(outcome.result.unwrap(), 42);
    assert!(!denied_callback.get());
    assert!(outcome.report.instructions().unwrap() > 0);
    assert!(matches!(
        outcome.report.termination(),
        ExecutionTermination::Completed
    ));
    let unit = context
        .with_execution(ExecutionOptions::new(0), |_| Ok(()))
        .unwrap();
    unit.result.unwrap();
    assert_eq!(unit.report.instructions().unwrap(), 0);
    assert!(!unit.report.effects_started());
}

#[test]
fn setup_failure_never_invokes_sync_or_async_callbacks() {
    let options = ExecutionOptions::new(64).with_profile(ProfileMode::Detailed {
        timing: false,
        event_limit: usize::MAX,
        filter: EventFilter::All,
    });
    let mut context = Context::builder().build().unwrap();
    let called = Cell::new(false);
    assert!(matches!(
        context.with_execution(options, |_| {
            called.set(true);
            Ok(())
        }),
        Err(ExecutionSetupError::Allocation)
    ));
    assert!(matches!(
        context.execution_task(options, async |_| {
            called.set(true);
            Ok(())
        }),
        Err(ExecutionSetupError::Allocation)
    ));
    assert!(!called.get());
    assert_eq!(
        context.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn ordinary_guest_error_is_a_completed_result_without_an_abandoned_stop() {
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .with_execution(ExecutionOptions::new(0), |_| {
            Err::<(), _>(JsError::from_opaque(JsValue::new(23)).unwrap())
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_opaque(),
        Some(&JsValue::new(23))
    );
    assert!(matches!(
        outcome.report.termination(),
        ExecutionTermination::Completed
    ));
    assert_eq!(outcome.report.instructions().unwrap(), 0);
}

#[test]
fn synchronous_scope_keeps_the_first_stop_over_swallowed_success_or_later_error() {
    for later_error in [false, true] {
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .with_execution(ExecutionOptions::new(32), |mut context| {
                let first = context
                    .eval(Source::from_bytes("while (true) {}"))
                    .unwrap_err();
                assert_eq!(first.as_engine(), Some(&instruction_error(32)));
                if later_error {
                    Err(JsNativeError::typ()
                        .with_message("later guest error")
                        .into())
                } else {
                    Ok(42_u32)
                }
            })
            .unwrap();
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&instruction_error(32))
        );
        assert_eq!(outcome.report.instructions().unwrap(), 32);
        // Completed includes an ordinary delivered Err; the stop is not also
        // copied into an abandoned report.
        assert!(matches!(
            outcome.report.termination(),
            ExecutionTermination::Completed
        ));
        assert_eq!(
            context.eval(Source::from_bytes("42")).unwrap(),
            JsValue::new(42)
        );
    }
}

#[test]
fn independently_prepared_scripts_compose_across_awaits_on_one_reserved_context() {
    let mut context = Context::builder().build().unwrap();
    let first = Script::parse(
        Source::from_bytes("globalThis.partial = 20; partial"),
        None,
        context.access(),
    )
    .unwrap();
    let second = Script::parse(Source::from_bytes("partial + 22"), None, context.access()).unwrap();
    let mut task = context
        .execution_task(ExecutionOptions::new(256), async move |mut context| {
            assert_eq!(
                first
                    .evaluate_async_with_budget(context.reborrow(), 1)
                    .await?,
                JsValue::new(20)
            );
            second
                .evaluate_async_with_budget(context.reborrow(), 1)
                .await
        })
        .unwrap();
    let (outcome, pending_polls) = complete(&mut task);
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert!(pending_polls > 2);
    assert!(outcome.report.instructions().unwrap() > 0);
    assert_eq!(outcome.report.source_reads().unwrap(), 0);
    drop(task);
    assert_eq!(
        context.eval(Source::from_bytes("partial")).unwrap(),
        JsValue::new(20)
    );
    assert_eq!(context.stack_trace().count(), 0);
}

#[test]
fn script_quantum_polls_share_exact_and_one_short_instruction_limits() {
    const SOURCE: &str = "let total = 0; for (let i = 0; i < 6; i++) total += i; total";
    let control = Context::builder()
        .build()
        .unwrap()
        .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(4096))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(15));
    let exact = control.report.instructions().unwrap();
    assert!(exact > 2);
    for limit in [exact, exact - 1] {
        let mut context = Context::builder().build().unwrap();
        let mut task = context
            .execution_task(ExecutionOptions::new(limit), async |mut context| {
                let script = Script::parse(Source::from_bytes(SOURCE), None, context.reborrow())?;
                script
                    .evaluate_async_with_budget(context.reborrow(), 1)
                    .await
            })
            .unwrap();
        let receipt = task.receipt();
        assert!(receipt.report().is_none());
        let (outcome, pending_polls) = complete(&mut task);
        assert!(
            pending_polls > 2,
            "the script did not cross multiple VM suspensions"
        );
        assert_eq!(outcome.report.instructions().unwrap(), limit);
        if limit == exact {
            assert_eq!(outcome.result.unwrap(), JsValue::new(15));
        } else {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&instruction_error(limit))
            );
        }
        assert!(matches!(
            outcome.report.termination(),
            ExecutionTermination::Completed
        ));
        let published = receipt.report().unwrap();
        assert_eq!(published.instructions().unwrap(), limit);
        assert!(matches!(
            published.termination(),
            ExecutionTermination::Completed
        ));
        drop(task);
        assert_eq!(context.stack_trace().count(), 0);
        assert_eq!(
            context.eval(Source::from_bytes("42")).unwrap(),
            JsValue::new(42)
        );
    }
}

struct PublicationProbe {
    receipt: Rc<RefCell<Option<ExecutionReceipt>>>,
    drops: Rc<Cell<usize>>,
    unpublished_at_drop: Rc<Cell<bool>>,
}

impl Drop for PublicationProbe {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        let unpublished = self
            .receipt
            .borrow()
            .as_ref()
            .is_some_and(|receipt| receipt.report().is_none());
        self.unpublished_at_drop.set(unpublished);
    }
}

#[test]
fn unpolled_task_disposes_callback_captures_before_publishing_zero_work_abandonment() {
    let mut context = Context::builder().build().unwrap();
    let receipt_slot = Rc::new(RefCell::new(None));
    let drops = Rc::new(Cell::new(0));
    let unpublished_at_drop = Rc::new(Cell::new(false));
    let probe = PublicationProbe {
        receipt: receipt_slot.clone(),
        drops: drops.clone(),
        unpublished_at_drop: unpublished_at_drop.clone(),
    };
    let called = Rc::new(Cell::new(false));
    let callback_called = called.clone();
    let task = context
        .execution_task(
            ExecutionOptions::new(64).with_profile(ProfileMode::Summary { timing: false }),
            async move |mut context| {
                let _probe = probe;
                callback_called.set(true);
                context.eval(Source::from_bytes("globalThis.unpolledEffect = 1"))
            },
        )
        .unwrap();
    let receipt = task.receipt();
    *receipt_slot.borrow_mut() = Some(receipt.clone());
    assert!(receipt.report().is_none());
    drop(task);
    assert_eq!(drops.get(), 1);
    assert!(unpublished_at_drop.get());
    assert!(!called.get());
    let report = receipt.report().unwrap();
    assert!(matches!(
        report.termination(),
        ExecutionTermination::Abandoned { first_stop: None }
    ));
    assert_eq!(report.instructions().unwrap(), 0);
    assert!(!report.effects_started());
    assert!(
        report
            .profile()
            .unwrap()
            .unwrap()
            .scopes
            .iter()
            .all(|scope| scope.phases == 0)
    );
    assert_eq!(
        context
            .eval(Source::from_bytes("typeof unpolledEffect === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn cancel_drops_polled_inner_work_before_publishing_and_keeps_committed_effects() {
    let mut context = Context::builder().build().unwrap();
    let receipt_slot = Rc::new(RefCell::new(None));
    let drops = Rc::new(Cell::new(0));
    let unpublished_at_drop = Rc::new(Cell::new(false));
    let inner_receipt = receipt_slot.clone();
    let inner_drops = drops.clone();
    let inner_unpublished = unpublished_at_drop.clone();
    let mut task = context
        .execution_task(
            ExecutionOptions::new(64).with_profile(ProfileMode::Summary { timing: false }),
            async move |mut context| {
                let _probe = PublicationProbe {
                    receipt: inner_receipt,
                    drops: inner_drops,
                    unpublished_at_drop: inner_unpublished,
                };
                context.eval(Source::from_bytes("globalThis.innerEffect = 7"))?;
                pending::<()>().await;
                Ok(())
            },
        )
        .unwrap();
    let receipt = task.receipt();
    *receipt_slot.borrow_mut() = Some(receipt.clone());
    assert!(poll(&mut task).is_pending());
    assert!(receipt.report().is_none());
    let report = task.cancel();
    assert_eq!(drops.get(), 1);
    assert!(unpublished_at_drop.get());
    assert!(matches!(
        report.termination(),
        ExecutionTermination::Abandoned { first_stop: None }
    ));
    assert!(report.instructions().unwrap() > 0);
    assert!(std::ptr::eq(
        report.profile().unwrap().unwrap(),
        receipt.report().unwrap().profile().unwrap().unwrap()
    ));
    assert_eq!(
        context.eval(Source::from_bytes("innerEffect")).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(context.stack_trace().count(), 0);
}

#[test]
fn canceling_suspended_script_keeps_transferred_jobs_and_skips_guest_cleanup() {
    let executor = Rc::new(SimpleJobExecutor::new());
    let mut context = Context::builder().job_executor(executor).build().unwrap();
    let committed = Rc::new(Cell::new(false));
    let _ = context.insert_data(committed.clone());
    context
        .register_global_builtin_callable(
            js_string!("markCommitted"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                context.get_data::<Rc<Cell<bool>>>().unwrap().set(true);
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();
    let mut task = context.execution_task(ExecutionOptions::new(4096), async |mut context| {
        let script = Script::parse(Source::from_bytes(
            "globalThis.committedValue = 7; Promise.resolve().then(() => { globalThis.jobValue = 42; }); markCommitted(); try { while (true) {} } finally { globalThis.finalized = true; }",
        ), None, context.reborrow())?;
        script.evaluate_async_with_budget(context.reborrow(), 1).await
    }).unwrap();
    let receipt = task.receipt();
    for _ in 0..256 {
        assert!(
            poll(&mut task).is_pending(),
            "script ended before cancellation boundary"
        );
        if committed.get() {
            break;
        }
    }
    assert!(
        committed.get(),
        "script never committed its effect and job transfer"
    );
    let report = task.cancel();
    assert!(matches!(
        report.termination(),
        ExecutionTermination::Abandoned { first_stop: None }
    ));
    assert!(report.effects_started());
    assert!(receipt.report().is_some());
    assert_eq!(context.stack_trace().count(), 0);
    assert_eq!(context.eval(Source::from_bytes("committedValue === 7 && typeof finalized === 'undefined' && typeof jobValue === 'undefined'" )).unwrap(), JsValue::new(true));
    context.run_jobs().unwrap();
    assert_eq!(
        context.eval(Source::from_bytes("jobValue")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn completion_and_receipt_share_one_immutable_profile_body() {
    let mut context = Context::builder().build().unwrap();
    let receipt_slot = Rc::new(RefCell::new(None));
    let drops = Rc::new(Cell::new(0));
    let unpublished_at_drop = Rc::new(Cell::new(false));
    let inner_receipt = receipt_slot.clone();
    let inner_drops = drops.clone();
    let inner_unpublished = unpublished_at_drop.clone();
    let mut task = context
        .execution_task(
            ExecutionOptions::new(64).with_profile(ProfileMode::Summary { timing: false }),
            async move |mut context| {
                let _probe = PublicationProbe {
                    receipt: inner_receipt,
                    drops: inner_drops,
                    unpublished_at_drop: inner_unpublished,
                };
                context.eval(Source::from_bytes("42"))?;
                Ok(42_u32)
            },
        )
        .unwrap();
    let receipt = task.receipt();
    *receipt_slot.borrow_mut() = Some(receipt.clone());
    let retained = receipt.clone();
    assert!(receipt.report().is_none());
    let (outcome, _) = complete(&mut task);
    assert_eq!(outcome.result.unwrap(), 42);
    assert_eq!(drops.get(), 1);
    assert!(unpublished_at_drop.get());
    let report = receipt.report().unwrap();
    assert!(std::ptr::eq(
        outcome.report.profile().unwrap().unwrap(),
        report.profile().unwrap().unwrap()
    ));
    let canceled_after_ready = task.cancel();
    assert!(matches!(
        canceled_after_ready.termination(),
        ExecutionTermination::Completed
    ));
    assert!(std::ptr::eq(
        report.profile().unwrap().unwrap(),
        canceled_after_ready.profile().unwrap().unwrap()
    ));
    drop(outcome.report);
    let later = retained.report().unwrap();
    assert!(std::ptr::eq(
        report.profile().unwrap().unwrap(),
        later.profile().unwrap().unwrap()
    ));
    assert!(matches!(
        later.termination(),
        ExecutionTermination::Completed
    ));
    assert_eq!(
        context.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn canceling_after_a_swallowed_stop_moves_it_into_abandoned_report() {
    let mut context = Context::builder().build().unwrap();
    let mut task = context.execution_task(ExecutionOptions::new(32), async |mut context| {
        let first = context.eval(Source::from_bytes(
            "globalThis.beforeStop = 7; try { while (true) {} } finally { globalThis.finalized = true; }",
        )).unwrap_err();
        assert_eq!(first.as_engine(), Some(&instruction_error(32)));
        pending::<()>().await;
        Ok(())
    }).unwrap();
    let receipt = task.receipt();
    assert!(poll(&mut task).is_pending());
    assert!(receipt.report().is_none());
    let report = task.cancel();
    let ExecutionTermination::Abandoned {
        first_stop: Some(first_stop),
    } = report.termination()
    else {
        panic!("cancel must retain the undelivered first stop");
    };
    assert_eq!(first_stop.as_engine(), Some(&instruction_error(32)));
    assert_eq!(report.instructions().unwrap(), 32);
    let published = receipt.report().unwrap();
    assert!(std::ptr::eq(report.termination(), published.termination()));
    assert_eq!(context.stack_trace().count(), 0);
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "beforeStop === 7 && typeof finalized === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        context.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn native_unwind_publishes_abandonment_immediately_while_task_is_retained() {
    let mut context = Context::builder().build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("panicHost"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| panic!("native task unwind")),
        )
        .unwrap();
    let mut task = context
        .execution_task(ExecutionOptions::new(64), async |mut context| {
            context.eval(Source::from_bytes(
                "globalThis.beforePanic = 7; panicHost()",
            ))
        })
        .unwrap();
    let receipt = task.receipt();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = poll(&mut task);
    }))
    .unwrap_err();
    assert_eq!(panic.downcast_ref::<&str>(), Some(&"native task unwind"));
    let report = receipt
        .report()
        .expect("unwinding poll must finalize before returning to the host");
    assert!(matches!(
        report.termination(),
        ExecutionTermination::Abandoned { first_stop: None }
    ));
    assert!(report.effects_started());
    drop(task);
    assert_eq!(context.stack_trace().count(), 0);
    assert_eq!(
        context.eval(Source::from_bytes("beforePanic")).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        context.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}

#[derive(Debug)]
struct DiscardedOutput {
    drops: Rc<Cell<usize>>,
    panic_on_drop: bool,
}

impl Drop for DiscardedOutput {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
        if self.panic_on_drop {
            panic!("discarded generic output");
        }
    }
}

#[test]
fn retained_stop_discards_generic_success_once_and_is_delivered_as_completed_error() {
    let mut context = Context::builder().build().unwrap();
    let drops = Rc::new(Cell::new(0));
    let output_drops = drops.clone();
    let mut task = context
        .execution_task(ExecutionOptions::new(32), async move |mut context| {
            let _ = context.eval(Source::from_bytes("while (true) {}"));
            Ok(DiscardedOutput {
                drops: output_drops,
                panic_on_drop: false,
            })
        })
        .unwrap();
    let receipt = task.receipt();
    let (outcome, _) = complete(&mut task);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&instruction_error(32))
    );
    assert_eq!(drops.get(), 1);
    assert!(matches!(
        receipt.report().unwrap().termination(),
        ExecutionTermination::Completed
    ));
    drop(task);
}

#[test]
fn panicking_discard_of_generic_success_still_publishes_the_first_stop_as_abandoned() {
    let mut context = Context::builder().build().unwrap();
    let drops = Rc::new(Cell::new(0));
    let output_drops = drops.clone();
    let mut task = context
        .execution_task(ExecutionOptions::new(32), async move |mut context| {
            let _ = context.eval(Source::from_bytes("while (true) {}"));
            Ok(DiscardedOutput {
                drops: output_drops,
                panic_on_drop: true,
            })
        })
        .unwrap();
    let receipt = task.receipt();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = poll(&mut task);
    }))
    .unwrap_err();
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"discarded generic output")
    );
    assert_eq!(drops.get(), 1);
    let report = receipt
        .report()
        .expect("the finalizer must stay armed while discarding the output");
    let ExecutionTermination::Abandoned {
        first_stop: Some(first_stop),
    } = report.termination()
    else {
        panic!("unreturned first stop must move into abandonment");
    };
    assert_eq!(first_stop.as_engine(), Some(&instruction_error(32)));
    assert_eq!(report.instructions().unwrap(), 32);
    drop(task);
    assert_eq!(
        context.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}
