#![allow(unused_crate_dependencies)]
//! A test that mimics the `surgeist_js`'s GCD test with a typed callback.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::{path::PathBuf, rc::Rc};
use surgeist_js::interop::ContextData;
use surgeist_js::object::builtins::JsFunction;
use surgeist_js::{
    Context, IntoJsFunction, JsData, Module, Source, gc::heap::Trace as HeapTrace, js_string,
};

#[derive(Clone, JsData, HeapTrace)]
struct CallbackResult(#[heap_trace(skip)] Rc<AtomicUsize>);

fn callback_from_js(ContextData(r): ContextData<CallbackResult>, result: usize) {
    r.0.store(result, Ordering::Relaxed);
}

#[test]
fn gcd_callback() {
    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    // Create the engine.
    let context = &mut Context::builder().build().unwrap();
    let result = Rc::new(AtomicUsize::new(0));
    context.insert_data(CallbackResult(result.clone()));

    // Load the JavaScript code.
    let gcd_path = assets_dir.join("gcd_callback.js");
    let source = Source::from_filepath(&gcd_path).unwrap();
    let module = Module::parse(source, None, context.access()).unwrap();
    module
        .load_link_evaluate(context.access())
        .unwrap()
        .await_blocking(context.access())
        .unwrap();

    let js_gcd = module
        .get_typed_fn::<(i32, i32, JsFunction), ()>(js_string!("gcd_callback"), context.access())
        .unwrap();

    let function = (callback_from_js as fn(ContextData<CallbackResult>, usize))
        .into_js_function()
        .to_js_function(context.realm())
        .unwrap();

    result.store(0, Ordering::Relaxed);
    assert_eq!(
        js_gcd.call(context.access(), (6, 9, function.clone())),
        Ok(())
    );
    assert_eq!(result.load(Ordering::Relaxed), 3);

    result.store(0, Ordering::Relaxed);
    assert_eq!(js_gcd.call(context.access(), (9, 6, function)), Ok(()));
    assert_eq!(result.load(Ordering::Relaxed), 3);
}
