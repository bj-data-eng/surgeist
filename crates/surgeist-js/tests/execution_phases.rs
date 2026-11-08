#![allow(unused_crate_dependencies)]

use std::{cell::Cell, time::Duration};

use surgeist_js::{
    Context, JsValue, NativeFunction, Source,
    execution::{ExecutionOptions, ProfileMode},
    js_string,
};

thread_local! {
    static TIME: Cell<Duration> = const { Cell::new(Duration::ZERO) };
    static CALLBACKS: Cell<usize> = const { Cell::new(0) };
}

fn clock() -> Duration {
    TIME.get()
}

fn context_with_clock_callback() -> Context {
    let mut context = Context::default();
    context
        .register_global_builtin_callable(
            js_string!("advanceClock"),
            0,
            NativeFunction::from_fn_ptr(|_, _, _| {
                CALLBACKS.set(CALLBACKS.get() + 1);
                TIME.set(TIME.get() + Duration::from_millis(7));
                Ok(JsValue::new(41))
            }),
        )
        .unwrap();
    context
}

#[test]
fn profiled_evaluation_observes_vm_active_time_across_a_native_callback() {
    const SOURCE: &str = "advanceClock() + 1";
    TIME.set(Duration::ZERO);
    CALLBACKS.set(0);
    let control = context_with_clock_callback()
        .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    assert_eq!(CALLBACKS.get(), 1);
    let instructions = control.report.instructions();
    assert!(instructions > 0 && instructions <= 64);

    TIME.set(Duration::ZERO);
    CALLBACKS.set(0);
    let outcome = context_with_clock_callback()
        .eval_with_budget(
            Source::from_bytes(SOURCE),
            ExecutionOptions::new(64)
                .with_profile(ProfileMode::Summary { timing: true })
                .with_profile_clock(clock),
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert_eq!(CALLBACKS.get(), 1);
    assert_eq!(outcome.report.instructions(), instructions);
    assert!(outcome.report.effects_started());
    let profile = outcome.report.profile().unwrap();
    assert_eq!(profile.elapsed, Some(Duration::from_millis(7)));
    // Only executing the registered callback advances time. Parsing and
    // compilation finish at zero, before this single synchronous VM slice.
    let active = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "vm-active")
        .expect("a profiled evaluation must expose VM active execution");
    assert_eq!(active.phases, 1);
    assert_eq!(active.phase_elapsed, Some(Duration::from_millis(7)));
}
