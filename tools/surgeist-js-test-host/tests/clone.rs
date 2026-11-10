#![allow(unused_crate_dependencies, missing_docs)]

use rstest::rstest;
use std::path::PathBuf;
use surgeist_js::{Context, Source};
use surgeist_js_test_host::RuntimeExtension;

#[rstest]
fn clone(#[files("tests/clone/**/*.js")] path: PathBuf) {
    let context = &mut Context::builder().build().unwrap();
    surgeist_js_test_host::clone::register(None, context).expect("Could not register runtime");
    surgeist_js_test_host::extensions::ConsoleExtension::default()
        .register(None, context)
        .expect("Could not register console");

    let harness_path = PathBuf::from("./assets/harness.js");
    let harness = Source::from_filepath(&harness_path).expect("Could not load harness");
    context.eval(harness).expect("Could not eval source");

    let source = Source::from_filepath(&path).expect("Could not load source");

    if let Err(e) = context.eval(source) {
        panic!("Evaluation failed: {e}");
    }

    if let Err(e) = context.run_jobs() {
        panic!("Execution error: {e}");
    }
}
