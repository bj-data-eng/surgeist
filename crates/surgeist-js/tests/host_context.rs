#![allow(unused_crate_dependencies)]

use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
};

use surgeist_js::{
    Context, JsResult, JsValue, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionTermination},
};

#[test]
fn repeated_holder_leases_and_reborrows_share_the_original_guest_state() {
    let mut owner = Context::builder().build().unwrap();
    let mut independent = Context::builder().build().unwrap();
    {
        let holder = owner.host_access();
        holder
            .with(|mut access| {
                access.eval(Source::from_bytes("globalThis.value = 40"))?;
                let mut nested = access.reborrow();
                assert_eq!(
                    nested.eval(Source::from_bytes("value += 2"))?,
                    JsValue::new(42)
                );
                Ok(())
            })
            .unwrap();
        assert_eq!(
            holder
                .with(|mut access| access.eval(Source::from_bytes("value")))
                .unwrap(),
            JsValue::new(42)
        );
        assert_eq!(
            independent
                .eval(Source::from_bytes("globalThis.value = 7"))
                .unwrap(),
            JsValue::new(7)
        );
        assert_eq!(
            holder
                .with(|mut access| access.eval(Source::from_bytes("value")))
                .unwrap(),
            JsValue::new(42)
        );
    }
    assert_eq!(
        owner.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        independent.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(7)
    );
}

#[test]
fn nested_same_holder_is_busy_without_invoking_callback_or_charging_work() {
    let control = Context::builder()
        .build()
        .unwrap()
        .eval_with_budget(Source::from_bytes("40 + 2"), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    let instructions = control.report.instructions().unwrap();
    let mut owner = Context::builder().build().unwrap();
    let called = Cell::new(false);
    let outcome = owner
        .with_execution(ExecutionOptions::new(64), |context| {
            let holder = context.into_host_access();
            holder.with(|mut access| {
                let busy = holder
                    .with(|mut competing| {
                        called.set(true);
                        competing.eval(Source::from_bytes(
                            "globalThis.unexpected = 1; while (true) {}",
                        ))
                    })
                    .unwrap_err();
                assert_eq!(busy.as_engine(), Some(&EngineError::ContextAccessBusy));
                // The original short lease remains usable after the rejected one.
                access.eval(Source::from_bytes("40 + 2"))
            })
        })
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_eq!(outcome.report.instructions().unwrap(), instructions);
    assert!(!called.get());
    assert!(matches!(
        outcome.report.termination(),
        ExecutionTermination::Completed
    ));
    assert_eq!(
        owner
            .eval(Source::from_bytes("typeof unexpected === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn holder_restores_its_lease_after_callback_panic_without_rolling_back_effects() {
    let mut owner = Context::builder().build().unwrap();
    {
        let holder = owner.host_access();
        let panic = catch_unwind(AssertUnwindSafe(|| {
            holder.with(|mut access| -> JsResult<()> {
                access.eval(Source::from_bytes("globalThis.committed = 7"))?;
                panic!("holder callback unwind");
            })
        }))
        .unwrap_err();
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"holder callback unwind")
        );
        assert_eq!(
            holder
                .with(|mut access| access.eval(Source::from_bytes("committed + 35")))
                .unwrap(),
            JsValue::new(42)
        );
        assert_eq!(
            holder
                .with(|mut access| access.eval(Source::from_bytes("committed")))
                .unwrap(),
            JsValue::new(7)
        );
    }
    assert_eq!(
        owner.eval(Source::from_bytes("committed")).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        owner.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn consuming_access_into_holder_retains_the_selected_owner() {
    let mut owner = Context::builder().build().unwrap();
    {
        let mut access = owner.access();
        access
            .eval(Source::from_bytes("globalThis.converted = 7"))
            .unwrap();
        let holder = access.into_host_access();
        assert_eq!(
            holder
                .with(|mut access| access.eval(Source::from_bytes("converted *= 6")))
                .unwrap(),
            JsValue::new(42)
        );
        assert_eq!(
            holder
                .with(|mut access| access.eval(Source::from_bytes("converted")))
                .unwrap(),
            JsValue::new(42)
        );
    }
    assert_eq!(
        owner.eval(Source::from_bytes("converted")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn retained_instruction_stop_wins_over_later_holder_busy_at_operation_completion() {
    let mut owner = Context::builder().build().unwrap();
    let called = Cell::new(false);
    let outcome = owner
        .with_execution(ExecutionOptions::new(32), |context| {
            let holder = context.into_host_access();
            holder.with(|mut access| {
                let first = access
                    .eval(Source::from_bytes(
                        "globalThis.beforeStop = 7; while (true) {}",
                    ))
                    .unwrap_err();
                assert_eq!(
                    first.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 32
                    }))
                );
                let busy = holder
                    .with(|_| {
                        called.set(true);
                        Ok(42_u32)
                    })
                    .unwrap_err();
                assert_eq!(busy.as_engine(), Some(&EngineError::ContextAccessBusy));
                Err::<u32, _>(busy)
            })
        })
        .unwrap();
    assert!(!called.get());
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 32
        }))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 32);
    assert!(matches!(
        outcome.report.termination(),
        ExecutionTermination::Completed
    ));
    assert_eq!(
        owner.eval(Source::from_bytes("beforeStop")).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        owner.eval(Source::from_bytes("42")).unwrap(),
        JsValue::new(42)
    );
}
