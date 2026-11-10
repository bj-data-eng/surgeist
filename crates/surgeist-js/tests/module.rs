#![allow(unused_crate_dependencies, missing_docs)]

use std::rc::Rc;

use surgeist_js::builtins::promise::PromiseState;
use surgeist_js::job::ContextTask;
use surgeist_js::module::{ModuleLoader, Referrer};
use surgeist_js::{Context, JsString, Module, Source, js_string};

#[test]
fn dropping_a_parsed_module_preserves_shared_scope_identities() {
    let mut context = Context::builder().build().unwrap();
    let root = context.realm().scope().clone();
    let baseline = surgeist_js_ast::scope::Scope::new(root.clone(), false).unique_id();
    let module = Module::parse(
        Source::from_bytes("export function retained() { return 42; }"),
        None,
        context.access(),
    )
    .expect("valid module syntax parses");
    drop(module);
    let next = surgeist_js_ast::scope::Scope::new(root, false).unique_id();
    assert_eq!(next, baseline + 1);
}

#[test]
fn failed_linking_preserves_empty_synthetic_dependency_scope_identities() {
    let loader = Rc::new(surgeist_js::module::MapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();
    let dependency = Module::synthetic(
        &[],
        surgeist_js::module::SyntheticModuleInitializer::from_fn_with_captures(
            |_, (), _| Ok(()),
            (),
        ),
        None,
        None,
        context.access(),
    )
    .unwrap();
    let peer = Module::parse(
        Source::from_bytes("export const present = 1;"),
        None,
        context.access(),
    )
    .unwrap();
    let module = Module::parse(
        Source::from_bytes("import 'empty'; export { absent } from 'peer';"),
        None,
        context.access(),
    )
    .unwrap();
    assert!(loader.insert("empty", dependency).is_none());
    assert!(loader.insert("peer", peer).is_none());
    let loaded = module.load(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    let root = context.realm().scope().clone();
    let baseline = surgeist_js_ast::scope::Scope::new(root.clone(), false).unique_id();
    let error = module.link(context.access()).unwrap_err();
    assert_eq!(
        error.as_native().unwrap().message(),
        "could not find export `absent`"
    );
    let next = surgeist_js_ast::scope::Scope::new(root, false).unique_id();
    assert_eq!(next, baseline + 1);
}

#[test]
fn one_short_module_preparation_preserves_identities_and_retained_source_retry() {
    fn setup() -> (Context, Module, u32) {
        let mut context = Context::builder().build().unwrap();
        let module = Module::parse(
            Source::from_bytes(
                "export function answer(value = 1) { return value + 41; } export const value = 42;",
            ),
            None,
            context.access(),
        )
        .unwrap();
        let loaded = module.load(context.access()).unwrap();
        context.run_jobs().unwrap();
        assert_eq!(
            loaded.state().unwrap(),
            PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
        );
        context
            .register_global_builtin_callable(
                js_string!("linkModule"),
                0,
                surgeist_js::NativeFunction::from_fn_with_captures(
                    |_, _, module, mut context| {
                        module.link(context.reborrow())?;
                        Ok(surgeist_js::JsValue::undefined())
                    },
                    module.clone(),
                ),
            )
            .unwrap();
        let baseline =
            surgeist_js_ast::scope::Scope::new(context.realm().scope().clone(), false).unique_id();
        (context, module, baseline)
    }
    let (mut control, _, _) = setup();
    let complete = control
        .eval_with_budget(
            Source::from_bytes("linkModule()"),
            surgeist_js::execution::ExecutionOptions::new(256),
        )
        .unwrap();
    complete.result.unwrap();
    let limit = complete
        .report
        .scope_work()
        .unwrap()
        .checked_sub(1)
        .unwrap();

    let (mut context, module, baseline) = setup();
    let stopped = context
        .eval_with_budget(
            Source::from_bytes("linkModule()"),
            surgeist_js::execution::ExecutionOptions::new(256).with_scope_work_limit(limit),
        )
        .unwrap();
    assert_eq!(
        stopped.result.unwrap_err().as_engine(),
        Some(&surgeist_js::error::EngineError::RuntimeLimit(
            surgeist_js::error::RuntimeLimitError::ScopeWork { limit }
        ))
    );
    assert!(stopped.report.effects_started());
    let next =
        surgeist_js_ast::scope::Scope::new(context.realm().scope().clone(), false).unique_id();
    assert_eq!(next, baseline + 1);

    module.link(context.access()).unwrap();
    let evaluated = module.evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        evaluated.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    assert_eq!(
        module
            .get_value(js_string!("value"), context.access())
            .unwrap(),
        42.into()
    );
}

#[test]
fn linking_prepares_captures_without_entering_a_guest_frame() {
    let mut context = Context::builder().build().unwrap();
    let module = Module::parse(
        Source::from_bytes("export function answer() { return 42; }"),
        None,
        context.access(),
    )
    .unwrap();
    let loaded = module.load(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    let limits = context.runtime_limits();
    context.runtime_limits_mut().set_recursion_limit(0);
    context.runtime_limits_mut().set_stack_size_limit(0);
    module.link(context.access()).unwrap();
    *context.runtime_limits_mut() = limits;
    let answer = module
        .get_value(js_string!("answer"), context.access())
        .unwrap();
    let answer = answer.as_function().unwrap();
    assert_eq!(
        answer
            .call(&surgeist_js::JsValue::undefined(), &[], context.access())
            .unwrap(),
        42.into()
    );
}

#[test]
fn pending_cyclic_modules_use_the_current_realm_and_preserve_top_level_await() {
    let loader = Rc::new(surgeist_js::module::MapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();
    let root = Module::parse(
        Source::from_bytes(
            "import { g } from 'peer';
             export function f() { return g() + intervening; }
             export const result = await Promise.resolve(f());",
        ),
        None,
        context.access(),
    )
    .unwrap();
    let peer = Module::parse(
        Source::from_bytes(
            "import { f } from 'root';
             export function g() { return 41; }
             export function throughCycle() { return f(); }",
        ),
        None,
        context.access(),
    )
    .unwrap();
    assert!(loader.insert("root", root.clone()).is_none());
    assert!(loader.insert("peer", peer.clone()).is_none());
    context
        .eval(Source::from_bytes("let intervening = 1;"))
        .unwrap();
    let evaluated = root.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        evaluated.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    assert_eq!(
        root.get_value(js_string!("result"), context.access())
            .unwrap(),
        42.into()
    );
    let through_cycle = peer
        .get_value(js_string!("throughCycle"), context.access())
        .unwrap();
    assert_eq!(
        through_cycle
            .as_function()
            .unwrap()
            .call(&surgeist_js::JsValue::undefined(), &[], context.access())
            .unwrap(),
        42.into()
    );
}

#[test]
fn failed_linking_preserves_an_initialized_synthetic_binding() {
    let loader = Rc::new(surgeist_js::module::MapModuleLoader::new());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();
    let stable = Module::synthetic(
        &[js_string!("value")],
        surgeist_js::module::SyntheticModuleInitializer::from_fn_with_captures(
            |module, (), _| module.set_export(&js_string!("value"), 17.into()),
            (),
        ),
        None,
        None,
        context.access(),
    )
    .unwrap();
    stable.link(context.access()).unwrap();
    let initialized = stable.evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        initialized.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    let peer = Module::parse(
        Source::from_bytes("export const present = 1;"),
        None,
        context.access(),
    )
    .unwrap();
    let root = Module::parse(
        Source::from_bytes("import 'stable'; export { absent } from 'peer';"),
        None,
        context.access(),
    )
    .unwrap();
    assert!(loader.insert("stable", stable.clone()).is_none());
    assert!(loader.insert("peer", peer).is_none());
    let loaded = root.load(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    let scope = context.realm().scope().clone();
    let baseline = surgeist_js_ast::scope::Scope::new(scope.clone(), false).unique_id();
    assert_eq!(
        root.link(context.access())
            .unwrap_err()
            .as_native()
            .unwrap()
            .message(),
        "could not find export `absent`"
    );
    assert_eq!(
        stable
            .get_value(js_string!("value"), context.access())
            .unwrap(),
        17.into()
    );
    let next = surgeist_js_ast::scope::Scope::new(scope, false).unique_id();
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
        ) -> ContextTask<Module> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");
            let src = self.0.clone();

            ContextTask::from_future(async move { Ok(src) }).then(Module::parse_json)
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

    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    match promise.state().unwrap() {
        PromiseState::Pending => {}
        PromiseState::Fulfilled(v) => {
            assert!(v.is_undefined());
        }
        PromiseState::Rejected(e) => {
            panic!(
                "Unexpected error: {:?}",
                e.to_string(context.access()).unwrap()
            );
        }
    }

    let json = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("json"), context.access())
        .unwrap();

    assert_eq!(
        JsString::from(json.to_json(context.access()).unwrap().unwrap().to_string()),
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
        ) -> ContextTask<Module> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            // Verify attributes were passed correctly
            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            ContextTask::from_future(async move { Ok(src) }).then(Module::parse_json)
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

    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    match promise.state().unwrap() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    // Get the exported promise 'p'
    let p = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("p"), context.access())
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state().unwrap() {
        PromiseState::Fulfilled(module_ns) => {
            let default_export = module_ns
                .as_object()
                .unwrap()
                .get(js_string!("default"), context.access())
                .unwrap();

            assert_eq!(
                JsString::from(
                    default_export
                        .to_json(context.access())
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
                e.to_string(context.access()).unwrap()
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
        ) -> ContextTask<Module> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            ContextTask::from_future(async move { Ok(src) }).then(Module::parse_json)
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

    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
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
        ) -> ContextTask<Module> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            let type_attr = request
                .get_attribute("type")
                .expect("should have type attribute");
            assert_eq!(type_attr.to_std_string_escaped(), "json");

            let src = self.0.clone();
            ContextTask::from_future(async move { Ok(src) }).then(Module::parse_json)
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

    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );

    let json = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("json"), context.access())
        .unwrap();

    assert_eq!(
        JsString::from(json.to_json(context.access()).unwrap().unwrap().to_string()),
        json_string
    );
}

#[test]
fn test_dynamic_import_invalid_options() {
    struct TestModuleLoader;
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            _request: surgeist_js::module::ModuleRequest,
        ) -> ContextTask<Module> {
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

    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    match promise.state().unwrap() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    // Get the exported promise 'p'
    let p = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("p"), context.access())
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state().unwrap() {
        PromiseState::Rejected(e) => {
            let error = e.as_object().unwrap();
            let name = error.get(js_string!("name"), context.access()).unwrap();
            assert_eq!(name.as_string().unwrap(), js_string!("TypeError"));
        }
        state => panic!("Dynamic import should be rejected with TypeError, got {state:?}"),
    }
}

#[test]
fn test_dynamic_import_non_string_attribute_value() {
    struct TestModuleLoader;
    impl ModuleLoader for TestModuleLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            _request: surgeist_js::module::ModuleRequest,
        ) -> ContextTask<Module> {
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

    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    match promise.state().unwrap() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    let p = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("p"), context.access())
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state().unwrap() {
        PromiseState::Rejected(e) => {
            let error = e.as_object().unwrap();
            let name = error.get(js_string!("name"), context.access()).unwrap();
            assert_eq!(name.as_string().unwrap(), js_string!("TypeError"));
            let message = error.get(js_string!("message"), context.access()).unwrap();
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
        ) -> ContextTask<Module> {
            assert_eq!(request.specifier().to_std_string_escaped(), "basic");

            // Verify attributes were passed correctly (symbol key should be ignored)
            assert!(request.get_attribute("type").is_none());

            let src = self.0.clone();
            ContextTask::from_future(async move { Ok(src) }).then(Module::parse_json)
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

    let module = Module::parse(source, None, context.access()).unwrap();
    let promise = module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    match promise.state().unwrap() {
        PromiseState::Fulfilled(_) => {}
        _ => panic!("Module evaluation failed"),
    }

    let p = module
        .namespace(context.access())
        .unwrap()
        .get(js_string!("p"), context.access())
        .unwrap();

    let p_obj = p.as_promise().unwrap();
    context.run_jobs().unwrap();

    match p_obj.state().unwrap() {
        PromiseState::Fulfilled(module_ns) => {
            let default_export = module_ns
                .as_object()
                .unwrap()
                .get(js_string!("default"), context.access())
                .unwrap();

            assert_eq!(
                JsString::from(
                    default_export
                        .to_json(context.access())
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
                e.to_string(context.access()).unwrap()
            );
        }
        PromiseState::Pending => panic!("Dynamic import is still pending"),
    }
}

#[test]
fn a_cold_module_links_and_evaluates_in_a_peer_context() {
    let mut owner = Context::builder().build().unwrap();
    let mut peer = Context::builder()
        .runtime(owner.runtime().clone())
        .build()
        .unwrap();
    let module = Module::parse(
        Source::from_bytes("export const retainedModuleValue = 'owner module literal';"),
        None,
        owner.access(),
    )
    .unwrap();
    drop(owner);

    assert_eq!(
        module.load(peer.access()).unwrap().state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    module.link(peer.access()).unwrap();
    assert_eq!(
        module.evaluate(peer.access()).unwrap().state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );
    assert_eq!(
        module
            .get_value(js_string!("retainedModuleValue"), peer.access())
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
        ) -> ContextTask<Module> {
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
            ContextTask::from_future(async move { Ok(module) })
        }
    }

    let mut owner = Context::builder().build().unwrap();
    let named = Module::parse(
        Source::from_bytes("const nameInDependency = 17; export { nameInDependency as '*' };"),
        None,
        owner.access(),
    )
    .unwrap();
    let namespace = Module::parse(
        Source::from_bytes("export const peerNamespaceValue = 29;"),
        None,
        owner.access(),
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
        peer.access(),
    )
    .unwrap();

    let evaluation = root.load_link_evaluate(peer.access()).unwrap();
    peer.run_jobs().unwrap();
    assert_eq!(
        evaluation.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );

    for name in [js_string!("reexportedValue"), js_string!("importedValue")] {
        assert_eq!(root.get_value(name, owner.access()).unwrap(), 17.into());
    }
    assert_eq!(
        root.get_value(js_string!("peerNamespaceValue"), owner.access())
            .unwrap(),
        29.into()
    );
    let namespace = root
        .get_value(js_string!("peerNamespace"), owner.access())
        .unwrap();
    assert_eq!(
        namespace
            .as_object()
            .unwrap()
            .get(js_string!("peerNamespaceValue"), peer.access())
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
        context.access(),
    )
    .unwrap();
    let child = Module::parse(
        Source::from_bytes("import 'cycle-root'; export const presentInCycle = 1;"),
        None,
        context.access(),
    )
    .unwrap();
    let peer = Module::parse(
        Source::from_bytes("import 'cycle-child';"),
        None,
        context.access(),
    )
    .unwrap();
    assert!(loader.insert("cycle-root", root.clone()).is_none());
    assert!(loader.insert("cycle-child", child).is_none());
    assert!(loader.insert("cycle-peer", peer.clone()).is_none());

    let loaded = root.load(context.access()).unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        loaded.state().unwrap(),
        PromiseState::Fulfilled(surgeist_js::JsValue::undefined())
    );

    for module in [root, peer] {
        let error = module.link(context.access()).unwrap_err();
        let error = error.as_native().unwrap();
        assert_eq!(error.kind(), &surgeist_js::JsNativeErrorKind::Syntax);
        assert_eq!(error.message(), "could not find export `missingFromCycle`");
    }
}
