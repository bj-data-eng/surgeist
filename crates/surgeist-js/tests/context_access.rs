#![allow(unused_crate_dependencies)]

use std::{
    cell::Cell,
    future::Future,
    task::{Context as TaskContext, Poll, Waker},
};

use surgeist_js::{
    Context, JsValue, Script, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{ExecutionOptions, ExecutionTermination, ProfileMode},
};

#[test]
fn access_and_reborrow_evaluate_against_the_same_committed_guest_state() {
    let mut owner = Context::default();
    {
        let mut access = owner.access();
        assert_eq!(
            access
                .eval(Source::from_bytes("globalThis.value = 40"))
                .unwrap(),
            JsValue::new(40)
        );
        {
            let mut nested = access.reborrow();
            assert_eq!(
                nested.eval(Source::from_bytes("value += 2")).unwrap(),
                JsValue::new(42)
            );
        }
        assert_eq!(
            access.eval(Source::from_bytes("value")).unwrap(),
            JsValue::new(42)
        );
    }
    assert_eq!(
        owner.eval(Source::from_bytes("value")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn typed_host_data_mutations_through_views_remain_owned_by_the_context() {
    let mut owner = Context::default();
    assert!(owner.insert_data(Cell::new(1_u32)).is_none());
    {
        let mut access = owner.access();
        assert!(access.has_data::<Cell<u32>>());
        access.get_data::<Cell<u32>>().unwrap().set(20);
        {
            let mut nested = access.reborrow();
            assert_eq!(nested.get_data::<Cell<u32>>().unwrap().get(), 20);
            let previous = nested.insert_data(Cell::new(40_u32)).unwrap();
            assert_eq!(previous.get(), 20);
            nested.get_data::<Cell<u32>>().unwrap().set(42);
            let retained = nested.remove_data::<Cell<u32>>().unwrap();
            assert_eq!(retained.get(), 42);
            assert!(!nested.has_data::<Cell<u32>>());
            assert!(nested.insert_data(*retained).is_none());
        }
        assert_eq!(access.get_data::<Cell<u32>>().unwrap().get(), 42);
    }
    assert!(owner.has_data::<Cell<u32>>());
    assert_eq!(owner.get_data::<Cell<u32>>().unwrap().get(), 42);
    assert_eq!(owner.remove_data::<Cell<u32>>().unwrap().get(), 42);
    assert!(!owner.has_data::<Cell<u32>>());
}

#[test]
fn idle_owner_move_and_swap_preserve_state_reports_and_selected_operation_limits() {
    let mut first = Context::default();
    let completed = {
        let mut access = first.access();
        access
            .eval(Source::from_bytes("globalThis.ownerValue = 40"))
            .unwrap();
        access
            .eval_with_budget(
                Source::from_bytes("ownerValue += 2; ownerValue"),
                ExecutionOptions::new(64).with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap()
    };
    assert_eq!(completed.result.unwrap(), JsValue::new(42));
    let instructions = completed.report.instructions();
    assert!(instructions > 0);
    let retained_report = completed.report.clone();
    let retained_batches = retained_report.profile().unwrap().scopes[0].accepted_batches;
    assert!(retained_batches > 0);

    let mut second = Context::default();
    second
        .access()
        .eval(Source::from_bytes("globalThis.ownerValue = -1"))
        .unwrap();
    let mut moved = first;
    std::mem::swap(&mut moved, &mut second);
    assert_eq!(
        second
            .access()
            .eval(Source::from_bytes("ownerValue"))
            .unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        moved
            .access()
            .eval(Source::from_bytes("ownerValue"))
            .unwrap(),
        JsValue::new(-1)
    );

    // The zero budget selects the original first owner in its new binding.
    // Its prior completed report does not attach a budget to either binding.
    let denied = second
        .access()
        .eval_with_budget(
            Source::from_bytes("globalThis.budgetEffect = 1"),
            ExecutionOptions::new(0),
        )
        .unwrap();
    assert_eq!(
        denied.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 0
        }))
    );
    assert_eq!(denied.report.instructions(), 0);
    let accepted = moved
        .access()
        .eval_with_budget(
            Source::from_bytes("globalThis.budgetEffect = 42; budgetEffect"),
            ExecutionOptions::new(64),
        )
        .unwrap();
    assert_eq!(accepted.result.unwrap(), JsValue::new(42));
    assert!(accepted.report.instructions() > 0);
    assert_eq!(
        second
            .access()
            .eval(Source::from_bytes(
                "ownerValue === 42 && typeof budgetEffect === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        moved
            .access()
            .eval(Source::from_bytes(
                "ownerValue === -1 && budgetEffect === 42"
            ))
            .unwrap(),
        JsValue::new(true)
    );

    assert_eq!(retained_report.instructions(), instructions);
    assert_eq!(
        retained_report.profile().unwrap().scopes[0].accepted_batches,
        retained_batches
    );
    assert!(matches!(
        retained_report.termination(),
        ExecutionTermination::Completed
    ));
    assert!(std::ptr::eq(
        completed.report.profile().unwrap(),
        retained_report.profile().unwrap()
    ));
}

#[test]
fn script_compilation_and_async_evaluation_compose_through_one_outer_access() {
    let mut owner = Context::default();
    {
        let mut access = owner.access();
        let script = Script::parse(
            Source::from_bytes(
                "globalThis.accessEffect = 0; for (let i = 0; i < 6; i++) accessEffect += 7; accessEffect",
            ),
            None,
            access.reborrow(),
        )
        .unwrap();
        let _compiled = script.codeblock(access.reborrow()).unwrap();
        let mut future = Box::pin(script.evaluate_async_with_budget(access.reborrow(), 1));
        let mut result = None;
        let mut pending_polls = 0;
        for _ in 0..1024 {
            match future
                .as_mut()
                .poll(&mut TaskContext::from_waker(Waker::noop()))
            {
                Poll::Pending => pending_polls += 1,
                Poll::Ready(completion) => {
                    result = Some(completion);
                    break;
                }
            }
        }
        let result = result.expect("script remained pending after 1024 cooperative polls");
        drop(future);
        assert!(
            pending_polls > 1,
            "script must cross multiple real suspensions"
        );
        assert_eq!(result.unwrap(), JsValue::new(42));
        assert_eq!(
            access.eval(Source::from_bytes("accessEffect")).unwrap(),
            JsValue::new(42)
        );
    }
    assert_eq!(
        owner.eval(Source::from_bytes("accessEffect")).unwrap(),
        JsValue::new(42)
    );
}
