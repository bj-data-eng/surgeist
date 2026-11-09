#![allow(unused_crate_dependencies, missing_docs)]

use std::path::PathBuf;
use std::rc::Rc;

use surgeist_js::builtins::promise::PromiseState;
use surgeist_js::module::SimpleModuleLoader;
use surgeist_js::{Context, JsString, JsValue, Source, js_string};

/// Test that relative imports work with the simple module loader.
#[test]
fn subdirectories() {
    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    let loader = Rc::new(SimpleModuleLoader::new(assets_dir).unwrap());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let source = Source::from_bytes(b"export { file1 } from 'file1.js';");
    let module = surgeist_js::Module::parse(source, None, context.access()).unwrap();
    let result = module.load_link_evaluate(context.access()).unwrap();

    context.run_jobs().unwrap();
    match result.state().unwrap() {
        PromiseState::Pending => {}
        PromiseState::Fulfilled(v) => {
            assert!(v.is_undefined());

            let foo_value = module
                .namespace(context.access())
                .unwrap()
                .get(js_string!("file1"), context.access())
                .unwrap()
                .as_callable()
                .unwrap()
                .call(&JsValue::undefined(), &[], context.access())
                .unwrap();

            assert_eq!(foo_value, js_string!("file1..file1_1.file1_2").into());
        }
        PromiseState::Rejected(reason) => {
            panic!("Module failed to load: {}", reason.display());
        }
    }
}

#[test]
fn json_import_attributes_are_part_of_the_cache_key() {
    let assets_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/assets");

    let loader = Rc::new(SimpleModuleLoader::new(assets_dir).unwrap());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        import json from 'data.json' with { type: 'json' };
        export let value = json;
        export let p = import('data.json');
    ",
    );
    let module = surgeist_js::Module::parse(source, None, context.access()).unwrap();
    let result = module.load_link_evaluate(context.access()).unwrap();

    context.run_jobs().unwrap();
    assert_eq!(
        result.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );

    let value = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("value"), context.access())
        .unwrap();
    assert_eq!(
        JsString::from(
            value
                .to_json(context.access())
                .unwrap()
                .unwrap()
                .to_string()
        ),
        js_string!(r#"{"ok":true}"#)
    );

    let p = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("p"), context.access())
        .unwrap();
    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state().unwrap() {
        PromiseState::Rejected(e) => {
            let error = e
                .as_object()
                .expect("expected rejection to be an Error object");
            let name = error.get(js_string!("name"), context.access()).unwrap();
            assert_eq!(name.as_string().unwrap(), js_string!("TypeError"));
            let message = error.get(js_string!("message"), context.access()).unwrap();
            assert_eq!(
                message.as_string().unwrap(),
                js_string!("module `data.json` needs an import attribute of type \"json\"")
            );
        }
        state => panic!("expected dynamic import to reject, got {state:?}"),
    }
}
