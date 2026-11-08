#![allow(unused_crate_dependencies)]

use std::path::Path;

use surgeist_js::{
    Context, JsError, JsObject, JsValue, Module, Source,
    builtins::promise::PromiseState,
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    js_string,
    object::builtins::JsPromise,
};

const INTERRUPTING_LIMIT: u64 = 64;
const LATER_LIMIT: u64 = 100_000;

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    context
        .global_object()
        .set(js_string!("moduleEffects"), 0, true, context.access())
        .unwrap();
    context
}

fn linked_module(source: &str, path: &str, context: &mut Context) -> Module {
    // Parsing, loading and linking precede the selected evaluation operation.
    let module = Module::parse(
        Source::from_bytes(source).with_path(Path::new(path)),
        None,
        context.access(),
    )
    .unwrap();
    let loaded = module.load(context.access()).unwrap();
    assert_eq!(
        loaded.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    module.link(context.access()).unwrap();
    module
}

fn effects(context: &mut Context) -> JsValue {
    context
        .global_object()
        .get(js_string!("moduleEffects"), context.access())
        .unwrap()
}

fn assert_instruction_stop(error: &JsError, limit: u64) {
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit
        }))
    );
}

fn stop_evaluation(module: &Module, context: &mut Context, limit: u64) {
    let outcome = context
        .with_execution(ExecutionOptions::new(limit), |access| {
            let result = module.evaluate(access);
            assert_instruction_stop(
                result
                    .as_ref()
                    .expect_err("entered evaluation must return its original instruction stop"),
                limit,
            );
            result
        })
        .unwrap();
    assert_instruction_stop(&outcome.result.unwrap_err(), limit);
    assert_eq!(outcome.report.instructions(), limit);
}

fn assert_later_evaluation_is_terminal(module: &Module, context: &mut Context) {
    let outcome = context
        .with_execution(ExecutionOptions::new(LATER_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap();
    let error = outcome
        .result
        .expect_err("an interrupted instance must refuse a fresh higher-budget evaluation");
    assert!(error.as_engine().is_some());
    assert_eq!(outcome.report.instructions(), 0);
}

#[test]
fn synchronous_stop_preserves_its_committed_effect_and_refuses_higher_budget_reentry() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1;
         for (let step = 0; step < 1000; step++) {}
         export const completed = true;",
        "sync-interruption.js",
        &mut context,
    );

    stop_evaluation(&module, &mut context, INTERRUPTING_LIMIT);
    assert_eq!(effects(&mut context), 1.into());
    assert_later_evaluation_is_terminal(&module, &mut context);
    assert_eq!(
        effects(&mut context),
        1.into(),
        "evaluation must never replay"
    );
}

#[test]
fn zero_instruction_stop_after_evaluation_entry_cannot_be_retried_as_unstarted_work() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1; export const completed = true;",
        "zero-instruction-interruption.js",
        &mut context,
    );

    // There is no previously retained stop. Evaluate enters its lifecycle before
    // attempting to admit the body's first instruction.
    stop_evaluation(&module, &mut context, 0);
    assert_eq!(effects(&mut context), 0.into());
    assert_later_evaluation_is_terminal(&module, &mut context);
    assert_eq!(effects(&mut context), 0.into());
}

#[test]
fn initial_async_module_body_returns_its_original_typed_host_stop() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1;
         for (let step = 0; step < 1000; step++) {}
         await 0;",
        "initial-tla-interruption.js",
        &mut context,
    );

    stop_evaluation(&module, &mut context, INTERRUPTING_LIMIT);
    assert_eq!(effects(&mut context), 1.into());
    assert_later_evaluation_is_terminal(&module, &mut context);
    assert_eq!(effects(&mut context), 1.into());
}

#[test]
fn ordinary_guest_throw_keeps_the_same_cached_rejected_promise_and_reason() {
    let mut context = context();
    let reason = JsValue::from(JsObject::with_null_proto(context.runtime()));
    context
        .global_object()
        .set(
            js_string!("moduleGuestError"),
            reason.clone(),
            true,
            context.access(),
        )
        .unwrap();
    let module = linked_module(
        "globalThis.moduleEffects += 1; throw moduleGuestError;",
        "ordinary-module-rejection.js",
        &mut context,
    );

    let first = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.state(), PromiseState::Rejected(reason.clone()));
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(second.state(), PromiseState::Rejected(reason));
    assert_eq!(effects(&mut context), 1.into());
}

fn evaluate_with_fresh_budget(module: &Module, context: &mut Context) -> JsPromise {
    context
        .with_execution(ExecutionOptions::new(LATER_LIMIT), |access| {
            module.evaluate(access)
        })
        .unwrap()
        .result
        .unwrap()
}

#[test]
fn stop_retained_before_evaluation_preserves_the_linked_instances_first_valid_attempt() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1; export const completed = true;",
        "pre-entry-refusal.js",
        &mut context,
    );
    let stopped = context
        .with_execution(ExecutionOptions::new(0), |mut access| {
            let prior = access.eval(Source::from_bytes("0")).unwrap_err();
            assert_instruction_stop(&prior, 0);
            module.evaluate(access)
        })
        .unwrap();
    assert_instruction_stop(&stopped.result.unwrap_err(), 0);
    assert_eq!(stopped.report.instructions(), 0);
    assert_eq!(effects(&mut context), 0.into());

    let first = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.state(), PromiseState::Fulfilled(JsValue::undefined()));
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(effects(&mut context), 1.into());
}

#[test]
fn completed_success_is_preserved_after_an_unrelated_operations_stop() {
    let mut context = context();
    let module = linked_module(
        "globalThis.moduleEffects += 1; export const completed = true;",
        "completed-before-unrelated-stop.js",
        &mut context,
    );
    let first = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.state(), PromiseState::Fulfilled(JsValue::undefined()));

    let unrelated = context
        .eval_with_budget(Source::from_bytes("0"), ExecutionOptions::new(0))
        .unwrap();
    assert_instruction_stop(&unrelated.result.unwrap_err(), 0);
    let second = evaluate_with_fresh_budget(&module, &mut context);
    assert_eq!(first.identity(), second.identity());
    assert_eq!(
        second.state(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(effects(&mut context), 1.into());
}
