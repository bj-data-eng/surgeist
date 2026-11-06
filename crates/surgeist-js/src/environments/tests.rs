use super::{DeclarativeEnvironment, EnvironmentStack};
use crate::{
    Context, JsNativeErrorKind, JsObject, JsValue, NativeFunction, Runtime, Source, TestAction,
    js_string, run_test_actions,
};
use indoc::indoc;
use surgeist_js_ast::scope::BindingLocatorScope;
use surgeist_js_gc::heap::AllocationCollection;

#[test]
fn saved_and_captured_scopes_keep_outer_bindings_alive() {
    let runtime = Runtime::default();
    let heap = &runtime.heap;
    heap.set_allocation_collection(AllocationCollection::BeforeAllocation)
        .unwrap();
    let global = heap.allocate(DeclarativeEnvironment::global()).unwrap();

    let mut stack = EnvironmentStack::new();
    let outer = stack.push_lexical(1, &global);
    let object = JsObject::with_null_proto(&runtime);
    let object_edge = object.edge();
    stack.put_lexical_value(BindingLocatorScope::Stack(outer), 0, object.into(), &global);
    stack.push_lexical(0, &global);

    let saved = stack.pop_to_global();
    heap.collect().unwrap();
    assert!(object_edge.upgrade().unwrap().is_some());
    stack.restore_from_saved(saved);

    let captured = heap.allocate(stack.stored()).unwrap();
    drop(stack);
    heap.collect().unwrap();

    let outer = captured
        .to_stack()
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
    let mut context = Context::default();
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
