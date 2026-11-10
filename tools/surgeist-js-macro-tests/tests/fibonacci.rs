#![allow(unused_crate_dependencies)]
//! A test that goes back and forth between JavaScript and Rust.

// You can execute this example with `cargo run --example gcd`

use std::path::PathBuf;
use surgeist_js::object::builtins::{JsFunction, TypedJsFunction};
use surgeist_js::{
    Context, ContextAccess, IntoJsFunction, JsResult, Module, Source, js_error, js_string,
};

#[allow(clippy::needless_pass_by_value)]
fn fibonacci(
    a: usize,
    cb_a: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    cb_b: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    mut context: ContextAccess<'_>,
) -> JsResult<usize> {
    if a <= 1 {
        Ok(a)
    } else {
        Ok(cb_a.call(
            context.reborrow(),
            (a - 1, cb_b.clone().into(), cb_a.clone().into()),
        )? + cb_b.call(
            context.reborrow(),
            (a - 2, cb_b.clone().into(), cb_a.clone().into()),
        )?)
    }
}

fn fibonacci_throw(
    a: usize,
    cb_a: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    cb_b: TypedJsFunction<(usize, JsFunction, JsFunction), usize>,
    context: ContextAccess<'_>,
) -> JsResult<usize> {
    if a < 5 {
        Err(js_error!("a is too small")?)
    } else {
        fibonacci(a, cb_a, cb_b, context)
    }
}

#[test]
fn fibonacci_test() {
    type JsStep = TypedJsFunction<(usize, JsFunction, JsFunction), usize>;
    type RustStep = fn(usize, JsStep, JsStep, ContextAccess<'_>) -> JsResult<usize>;

    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    // Create the engine.
    let context = &mut Context::builder().build().unwrap();

    // Load the JavaScript code.
    let gcd_path = assets_dir.join("fibonacci.js");
    let source = Source::from_filepath(&gcd_path).unwrap();
    let module = Module::parse(source, None, context.access()).unwrap();
    module
        .load_link_evaluate(context.access())
        .unwrap()
        .await_blocking(context.access())
        .unwrap();

    let fibonacci_js = module
        .get_typed_fn::<(usize, JsFunction, JsFunction), usize>(
            js_string!("fibonacci"),
            context.access(),
        )
        .unwrap();

    let fibonacci_rust = (fibonacci as RustStep)
        .into_js_function()
        .to_js_function(context.realm())
        .unwrap();

    assert_eq!(
        fibonacci_js
            .call(
                context.access(),
                (
                    10,
                    fibonacci_rust.clone(),
                    fibonacci_js.as_js_function().clone()
                )
            )
            .unwrap(),
        55
    );

    let fibonacci_throw = (fibonacci_throw as RustStep)
        .into_js_function()
        .to_js_function(context.realm())
        .unwrap();
    assert!(
        fibonacci_js
            .call(
                context.access(),
                (
                    10,
                    fibonacci_throw.clone(),
                    fibonacci_js.as_js_function().clone()
                )
            )
            .unwrap_err()
            .to_string()
            .contains("\"a is too small\""),
    );
}
