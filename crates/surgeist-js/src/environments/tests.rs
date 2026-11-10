use super::{DeclarativeEnvironment, EnvironmentStack};
use crate::{
    Context, JsNativeErrorKind, JsObject, JsValue, NativeFunction, Runtime, Source, TestAction,
    js_string, run_test_actions,
};
use indoc::indoc;
use surgeist_js_ast::scope::BindingLocatorScope;
use surgeist_js_gc::heap::AllocationCollection;

#[test]
fn cloned_and_captured_scopes_keep_outer_bindings_alive() {
    let runtime = Runtime::default();
    let heap = &runtime.heap;
    heap.set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let global = heap.allocate(DeclarativeEnvironment::global()).unwrap();

    let mut stack = EnvironmentStack::new();
    let outer = stack.push_lexical(1, &global);
    let object = JsObject::with_null_proto(&runtime).unwrap();
    let object_edge = object.edge();
    stack.put_lexical_value(BindingLocatorScope::Stack(outer), 0, object.into(), &global);
    stack.push_lexical(0, &global);

    let saved = stack.clone();
    stack = EnvironmentStack::new();
    assert_eq!(stack.len(), 0);
    heap.collect().unwrap();
    assert!(object_edge.upgrade().unwrap().is_some());
    stack = saved;

    let captured = heap.allocate(stack.stored()).unwrap();
    drop(stack);
    heap.collect().unwrap();

    let outer = captured
        .to_stack()
        .unwrap()
        .get(outer as usize)
        .and_then(super::Environment::into_declarative)
        .unwrap();
    assert_eq!(
        outer.get(0).unwrap().as_object().unwrap().identity(),
        object_edge.identity(),
    );

    drop(outer);
    drop(captured);
    heap.collect().unwrap();
    assert!(object_edge.upgrade().unwrap().is_none());
}

#[test]
fn captured_name_reads_preserve_identity_and_initialization_after_collection() {
    let mut context = Context::builder().build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("collect"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                context.runtime().collect().unwrap();
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();
    let result = context
        .eval(Source::from_bytes(indoc! {r#"
            function outer(box) {
                let empty;
                function readBox() { return box; }
                return function read() {
                    let own = {};
                    function readOwn() { return own; }
                    function readLater() { return later; }
                    collect();
                    let rejected = false;
                    try { readLater(); }
                    catch (error) { rejected = error instanceof ReferenceError; }
                    let later;
                    return rejected && readLater() === undefined && empty === undefined
                        && own === readOwn() && box === readBox();
                };
            }
            const read = outer({});
            read();
        "#}))
        .unwrap();
    assert_eq!(result, JsValue::from(true));
}

#[test]
fn let_is_block_scoped() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            {
              let bar = "bar";
            }
            bar;
        "#},
        JsNativeErrorKind::Reference,
        "bar is not defined",
    )]);
}

#[test]
fn const_is_block_scoped() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            {
            const bar = "bar";
            }
            bar;
        "#},
        JsNativeErrorKind::Reference,
        "bar is not defined",
    )]);
}

#[test]
fn var_not_block_scoped() {
    run_test_actions([TestAction::assert(indoc! {r#"
            {
              var bar = "bar";
            }
            bar == "bar";
        "#})]);
}

#[test]
fn functions_use_declaration_scope() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            function foo() {
                bar;
            }
            {
                let bar = "bar";
                foo();
            }
        "#},
        JsNativeErrorKind::Reference,
        "bar is not defined",
    )]);
}

#[test]
fn set_outer_var_in_block_scope() {
    run_test_actions([TestAction::assert(indoc! {r#"
            var bar;
            {
                bar = "foo";
            }
            bar == "foo";
        "#})]);
}

#[test]
fn set_outer_let_in_block_scope() {
    run_test_actions([TestAction::assert(indoc! {r#"
            let bar;
            {
                bar = "foo";
            }
            bar == "foo";
        "#})]);
}

#[test]
fn private_module_captures_reject_foreign_scopes_and_code_identity() {
    use std::rc::Rc;
    use surgeist_js_ast::scope::{
        PreparedScopes, ScopeAnalysisError, ScopeHandle, ScopeStateError,
    };
    fn module_scope(scopes: &mut PreparedScopes) -> ScopeHandle {
        scopes
            .with_access(None, |access| -> Result<_, ScopeAnalysisError<()>> {
                let root = access.root_handle()?;
                Ok(access.create_module_scope(&root, &[])?.0)
            })
            .unwrap()
    }
    let context = Context::builder().build().unwrap();
    let mut own = PreparedScopes::new(context.realm().scope());
    let mut foreign = PreparedScopes::new(context.realm().scope());
    let own_scope = module_scope(&mut own);
    let foreign_scope = module_scope(&mut foreign);
    let environment =
        super::PreparedModuleEnvironment::new(own_scope.clone(), 0, &context.runtime().heap)
            .unwrap();
    let code = |scope| {
        let mut code = crate::vm::CodeBlock::new(js_string!("private"), 0, false);
        code.scope_association = scope;
        Rc::new(code)
    };
    let owned_code = code(own_scope.clone());
    let foreign_code = code(foreign_scope);
    let rejected: Result<_, ScopeAnalysisError<()>> = own.with_access(None, |access| {
        environment.capture_for(&foreign_code, access)
    });
    assert!(matches!(
        rejected,
        Err(ScopeAnalysisError::State(
            ScopeStateError::AssociationMismatch
        ))
    ));
    let capture: Result<_, ScopeAnalysisError<()>> =
        own.with_access(None, |access| environment.capture_for(&owned_code, access));
    let capture = capture.unwrap();
    assert_eq!(
        capture
            .stored_for(&owned_code)
            .unwrap()
            .to_stack()
            .unwrap()
            .len(),
        1
    );
    let other_code = code(own_scope);
    assert_eq!(
        capture.stored_for(&other_code).unwrap_err().as_engine(),
        Some(&crate::error::EngineError::InvalidEntryPermit)
    );
    let foreign_permit = foreign
        .prepare_publication(None::<&mut dyn FnMut(_) -> Result<(), ()>>)
        .unwrap();
    assert_eq!(
        environment.validate_publication(&foreign_permit),
        Err(ScopeStateError::AssociationMismatch)
    );
}
