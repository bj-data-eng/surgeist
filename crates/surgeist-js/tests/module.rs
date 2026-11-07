#![allow(unused_crate_dependencies, missing_docs)]

use std::cell::RefCell;
use std::future;
use std::rc::Rc;

use surgeist_js::builtins::promise::PromiseState;
use surgeist_js::module::{ModuleLoader, Referrer};
use surgeist_js::{Context, JsResult, JsString, Module, Source, js_string};

#[test]
fn dropping_a_parsed_module_preserves_shared_scope_identities() {
    let mut context = Context::default();
    let root = context.realm().scope().clone();
    let baseline = surgeist_js_ast::scope::Scope::new(root.clone(), false).unique_id();
    let module = Module::parse(
        Source::from_bytes("export function retained() { return 42; }"),
        None,
        &mut context,
    )
    .expect("valid module syntax parses");
    drop(module);
    let next = surgeist_js_ast::scope::Scope::new(root, false).unique_id();
    assert_eq!(next, baseline + 1);
}

#[test]
fn test_json_module_from_str() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: surgeist_js::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");
            let src = self.0.clone();

            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_string = js_string!(r#"{"key":"value","other":123}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_string.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        import basic_json from 'basic';
        export let json = basic_json;
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Pending => {}
        PromiseState::Fulfilled(v) => {
            assert!(v.is_undefined());
        }
        PromiseState::Rejected(e) => {
            panic!("Unexpected error: {:?}", e.to_string(&mut context).unwrap());
        }
    }

    let json = module
        .namespace(&mut context)
        .unwrap()
        .get(js_string!("json"), &mut context)
        .unwrap();

    assert_eq!(
        JsString::from(json.to_json(&mut context).unwrap().unwrap().to_string()),
        json_string
    );
}

#[test]
fn test_json_module_dynamic_import() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: surgeist_js::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            // Verify attributes were passed correctly
            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_content = js_string!(r#"{"key":"value","other":123}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_content.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export let p = import('basic', { with: { type: 'json' } });
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    // Get the exported promise 'p'
    let p = module
        .namespace(&mut context)
        .unwrap()
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Fulfilled(module_ns) => {
            let default_export = module_ns
                .as_object()
                .unwrap()
                .get(js_string!("default"), &mut context)
                .unwrap();

            assert_eq!(
                JsString::from(
                    default_export
                        .to_json(&mut context)
                        .unwrap()
                        .unwrap()
                        .to_string()
                ),
                json_content
            );
        }
        PromiseState::Rejected(e) => {
            panic!(
                "Dynamic import failed: {:?}",
                e.to_string(&mut context).unwrap()
            );
        }
        PromiseState::Pending => panic!("Dynamic import is still pending"),
    }
}

#[test]
fn test_json_module_static_import_with_attributes() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: surgeist_js::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_string = js_string!(r#"{"static":"import"}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_string.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        import json from 'basic' with { type: 'json' };
        export let value = json;
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    assert_eq!(
        promise.state(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );

    let value = module
        .namespace(&mut context)
        .unwrap()
        .get(js_string!("value"), &mut context)
        .unwrap();

    assert_eq!(
        JsString::from(value.to_json(&mut context).unwrap().unwrap().to_string()),
        json_string
    );
}

#[test]
fn test_json_module_reexport_with_attributes() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: surgeist_js::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_string = js_string!(r#"{"re":"export"}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_string.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export { default as json } from 'basic' with { type: 'json' };
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    assert_eq!(
        promise.state(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );

    let json = module
        .namespace(&mut context)
        .unwrap()
        .get(js_string!("json"), &mut context)
        .unwrap();

    assert_eq!(
        JsString::from(json.to_json(&mut context).unwrap().unwrap().to_string()),
        json_string
    );
}

#[test]
fn test_dynamic_import_invalid_options() {
    struct TestModuleLoader;
    impl ModuleLoader for TestModuleLoader {
        async fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            _request: surgeist_js::module::ModuleRequest,
            _context: &RefCell<&mut Context>,
        ) -> JsResult<Module> {
            panic!("Module loading should not be triggered for invalid options");
        }
    }

    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export let p = import('basic', 'invalid-option-string');
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    // Get the exported promise 'p'
    let p = module
        .namespace(&mut context)
        .unwrap()
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Rejected(e) => {
            let error = e.as_object().unwrap();
            let name = error.get(js_string!("name"), &mut context).unwrap();
            assert_eq!(name.as_string().unwrap(), js_string!("TypeError"));
        }
        state => panic!("Dynamic import should be rejected with TypeError, got {state:?}"),
    }
}

#[test]
fn test_dynamic_import_non_string_attribute_value() {
    struct TestModuleLoader;
    impl ModuleLoader for TestModuleLoader {
        async fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            _request: surgeist_js::module::ModuleRequest,
            _context: &RefCell<&mut Context>,
        ) -> JsResult<Module> {
            panic!("Module loading should not be triggered for invalid attribute values");
        }
    }

    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        export let p = import('basic', { with: { type: 123 } });
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    let p = module
        .namespace(&mut context)
        .unwrap()
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Rejected(e) => {
            let error = e.as_object().unwrap();
            let name = error.get(js_string!("name"), &mut context).unwrap();
            assert_eq!(name.as_string().unwrap(), js_string!("TypeError"));
            let message = error.get(js_string!("message"), &mut context).unwrap();
            assert_eq!(
                message.as_string().unwrap(),
                js_string!("import attribute value must be a string")
            );
        }
        state => panic!("Dynamic import should be rejected with TypeError, got {state:?}"),
    }
}

#[test]
fn test_dynamic_import_symbol_key() {
    struct TestModuleLoader(JsString);
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: surgeist_js::module::ModuleRequest,
            context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            // Verify attributes were passed correctly (symbol key should be ignored)
            assert!(request.get_attribute("type").is_none());

            let src = self.0.clone();
            future::ready(Ok(
                Module::parse_json(src, &mut context.borrow_mut()).unwrap()
            ))
        }
    }

    let json_content = js_string!(r#"{"ignore":"symbol"}"#);
    let mut context = Context::builder()
        .module_loader(Rc::new(TestModuleLoader(json_content.clone())))
        .build()
        .unwrap();

    let source = Source::from_bytes(
        b"
        let sym = Symbol('type');
        export let p = import('basic', { with: { [sym]: 'json' } });
    ",
    );

    let module = Module::parse(source, None, &mut context).unwrap();
    let promise = module.load_link_evaluate(&mut context);
    context.run_jobs().unwrap();

    match promise.state() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    let p = module
        .namespace(&mut context)
        .unwrap()
        .get(js_string!("p"), &mut context)
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state() {
        PromiseState::Fulfilled(module_ns) => {
            let default_export = module_ns
                .as_object()
                .unwrap()
                .get(js_string!("default"), &mut context)
                .unwrap();

            assert_eq!(
                JsString::from(
                    default_export
                        .to_json(&mut context)
                        .unwrap()
                        .unwrap()
                        .to_string()
                ),
                json_content
            );
        }
        PromiseState::Rejected(e) => {
            panic!(
                "Dynamic import failed: {:?}",
                e.to_string(&mut context).unwrap()
            );
        }
        PromiseState::Pending => panic!("Dynamic import is still pending"),
    }
}

#[test]
fn a_cold_module_links_and_evaluates_in_a_peer_context() {
    let mut owner = Context::default();
    let mut peer = Context::builder()
        .runtime(owner.runtime().clone())
        .build()
        .unwrap();
    let module = Module::parse(
        Source::from_bytes("export const retainedModuleValue = 'owner module literal';"),
        None,
        &mut owner,
    )
    .unwrap();
    drop(owner);

    assert_eq!(
        module.load(&mut peer).state(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    module.link(&mut peer).unwrap();
    assert_eq!(
        module.evaluate(&mut peer).unwrap().state(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    assert_eq!(
        module
            .get_value(js_string!("retainedModuleValue"), &mut peer)
            .unwrap(),
        js_string!("owner module literal").into()
    );
}

#[test]
fn a_mixed_context_graph_resolves_named_and_namespace_exports_with_attributes() {
    struct PeerModuleLoader {
        named: Module,
        namespace: Module,
    }

    impl ModuleLoader for PeerModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            request: surgeist_js::module::ModuleRequest,
            _context: &RefCell<&mut Context>,
        ) -> impl Future<Output = JsResult<Module>> {
            assert_eq!(request.specifier(), &js_string!("peer-dependency"));
            let module = match request
                .get_attribute("kind")
                .unwrap()
                .to_std_string_escaped()
                .as_str()
            {
                "named" => self.named.clone(),
                "namespace" => self.namespace.clone(),
                kind => panic!("unexpected import attribute {kind}"),
            };
            future::ready(Ok(module))
        }
    }

    let mut owner = Context::default();
    let named = Module::parse(
        Source::from_bytes("const nameInDependency = 17; export { nameInDependency as '*' };"),
        None,
        &mut owner,
    )
    .unwrap();
    let namespace = Module::parse(
        Source::from_bytes("export const peerNamespaceValue = 29;"),
        None,
        &mut owner,
    )
    .unwrap();
    let mut peer = Context::builder()
        .runtime(owner.runtime().clone())
        .module_loader(Rc::new(PeerModuleLoader { named, namespace }))
        .build()
        .unwrap();
    let root = Module::parse(
        Source::from_bytes(
            "
            import { '*' as localValue } from 'peer-dependency' with { kind: 'named' };
            export { localValue as reexportedValue };
            export const importedValue = localValue;
            export * as peerNamespace from 'peer-dependency' with { kind: 'namespace' };
            export * from 'peer-dependency' with { kind: 'namespace' };
            ",
        ),
        None,
        &mut peer,
    )
    .unwrap();

    let evaluation = root.load_link_evaluate(&mut peer);
    peer.run_jobs().unwrap();
    assert_eq!(
        evaluation.state(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );

    for name in [js_string!("reexportedValue"), js_string!("importedValue")] {
        assert_eq!(root.get_value(name, &mut owner).unwrap(), 17.into());
    }
    assert_eq!(
        root.get_value(js_string!("peerNamespaceValue"), &mut owner)
            .unwrap(),
        29.into()
    );
    let namespace = root
        .get_value(js_string!("peerNamespace"), &mut owner)
        .unwrap();
    assert_eq!(
        namespace
            .as_object()
            .unwrap()
            .get(js_string!("peerNamespaceValue"), &mut peer)
            .unwrap(),
        29.into()
    );
}

#[test]
fn failed_cyclic_linking_reports_the_missing_export_on_retry() {
    let loader = Rc::new(surgeist_js::module::MapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let root = Module::parse(
        Source::from_bytes(
            "export { absent as missingFromCycle } from 'cycle-child'; import 'cycle-peer';",
        ),
        None,
        &mut context,
    )
    .unwrap();
    let child = Module::parse(
        Source::from_bytes("import 'cycle-root'; export const presentInCycle = 1;"),
        None,
        &mut context,
    )
    .unwrap();
    let peer = Module::parse(
        Source::from_bytes("import 'cycle-child';"),
        None,
        &mut context,
    )
    .unwrap();
    assert!(loader.insert("cycle-root", root.clone()).is_none());
    assert!(loader.insert("cycle-child", child).is_none());
    assert!(loader.insert("cycle-peer", peer.clone()).is_none());

    let loaded = root.load(&mut context);
    context.run_jobs().unwrap();
    assert_eq!(
        loaded.state(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );

    for module in [root, peer] {
        let error = module.link(&mut context).unwrap_err();
        let error = error.as_native().unwrap();
        assert_eq!(error.kind(), &surgeist_js::JsNativeErrorKind::Syntax);
        assert_eq!(error.message(), "could not find export `missingFromCycle`");
    }
}
