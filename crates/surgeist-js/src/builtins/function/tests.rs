use crate::{
    JsNativeErrorKind, JsValue, TestAction,
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    object::{FunctionObjectBuilder, JsObject},
    property::{Attribute, PropertyDescriptor},
    run_test_actions,
};
use indoc::indoc;
use surgeist_js_macros::js_str;

#[test]
fn native_callbacks_cannot_return_or_throw_foreign_objects() {
    use crate::{Context, JsError, Runtime};

    let mut context = Context::default();
    let function = FunctionObjectBuilder::new(
        context.realm(),
        NativeFunction::from_fn_ptr(|_, args, _| {
            let object = JsObject::with_null_proto(&Runtime::default()).into();
            match args[0].as_number() {
                Some(1.0) => Err(JsError::from_opaque(object)?),
                Some(2.0) => {
                    Err(JsNativeError::aggregate(vec![JsError::from_opaque(object)?]).into())
                }
                _ => Ok(object),
            }
        }),
    )
    .build()
    .unwrap();
    for outcome in 0..3 {
        let error = function
            .call(&JsValue::undefined(), &[outcome.into()], context.access())
            .unwrap_err();
        assert!(
            error
                .as_native()
                .is_some_and(|error| error.kind() == &JsNativeErrorKind::Type)
        );
    }
}

#[test]
fn native_async_job_owns_captures_after_the_function_drops() {
    use crate::{Context, builtins::promise::PromiseState, object::JsPromise};

    let mut context = Context::default();
    let capture = JsObject::with_null_proto(context.runtime());
    let weak_capture = capture.edge();
    let callback = NativeFunction::from_async_fn_with_captures(
        |_, _, capture, _| {
            Ok(crate::job::ContextTask::from_future(async move {
                Ok(capture.into())
            }))
        },
        capture,
    );
    let promise = callback
        .call(&JsValue::undefined(), &[], context.access())
        .unwrap()
        .as_object()
        .unwrap();
    let promise = JsPromise::from_object(promise).unwrap();
    drop(callback);

    context.runtime().collect().unwrap();
    assert!(weak_capture.is_live().unwrap());
    context.run_jobs().unwrap();
    {
        let PromiseState::Fulfilled(value) = promise.state().unwrap() else {
            panic!("native job did not fulfill its promise");
        };
        let captured = weak_capture.upgrade().unwrap().unwrap();
        assert_eq!(value.as_object(), Some(captured));
    }
    drop(promise);
    context.runtime().collect().unwrap();
    assert!(!weak_capture.is_live().unwrap());
}

#[test]
fn nested_native_invocation_restores_the_outer_activation() {
    use crate::{Context, JsError};

    let inner_context = Context::default();
    let inner = FunctionObjectBuilder::new(
        inner_context.realm(),
        NativeFunction::from_fn_with_captures(
            |_, args, inner_realm, context| {
                assert!(context.realm() == inner_realm);
                context.runtime().collect().unwrap();
                let value = JsObject::with_null_proto(context.runtime()).into();
                if args[0].as_boolean() == Some(true) {
                    Err(JsError::from_opaque(value)?)
                } else {
                    Ok(value)
                }
            },
            inner_context.realm().clone(),
        ),
    )
    .constructor(true)
    .build()
    .unwrap();
    let context = &mut Context::builder()
        .runtime(inner_context.runtime().clone())
        .build()
        .unwrap();
    let outer = NativeFunction::from_fn_with_captures(
        |_, _, inner, mut context| {
            let active = context
                .reborrow()
                .into_owner()
                .active_function_object()
                .expect("outer activation");
            let realm = context.realm().clone();
            for construct in [false, true] {
                for throws in [false, true] {
                    let args = [JsValue::from(throws)];
                    let outcome = if construct {
                        inner
                            .construct(&args, None, context.reborrow())
                            .map(JsValue::from)
                    } else {
                        inner.call(&JsValue::undefined(), &args, context.reborrow())
                    };
                    assert_eq!(outcome.is_err(), throws);
                    assert!(context.realm() == &realm);
                    assert_eq!(
                        context
                            .reborrow()
                            .into_owner()
                            .active_function_object()
                            .as_ref(),
                        Some(&active)
                    );
                }
            }
            Ok(JsValue::undefined())
        },
        inner,
    )
    .to_js_function(context.realm())
    .unwrap();
    outer
        .call(&JsValue::undefined(), &[], context.access())
        .expect("outer invocation");
}

#[allow(clippy::float_cmp)]
#[test]
fn arguments_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function jason(a, b) {
                    return arguments[0];
                }
            "#}),
        TestAction::assert_eq("jason(100, 6)", 100),
    ]);
}

#[test]
fn self_mutating_function_when_calling() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn self_mutating_function_when_constructing() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function x() {
                    x.y = 3;
                }
                new x();
            "#}),
        TestAction::assert_eq("x.y", 3),
    ]);
}

#[test]
fn function_prototype() {
    run_test_actions([
        TestAction::assert_eq("Function.prototype.name", js_string!()),
        TestAction::assert_eq("Function.prototype.length", 0),
        TestAction::assert_eq("Function.prototype()", JsValue::undefined()),
        TestAction::assert_eq(
            "Function.prototype(1, '', new String(''))",
            JsValue::undefined(),
        ),
        TestAction::assert_native_error(
            "new Function.prototype()",
            JsNativeErrorKind::Type,
            "not a constructor",
        ),
    ]);
}

#[test]
fn function_prototype_call() {
    run_test_actions([TestAction::assert_eq(
        "Object.prototype.toString.call(new Error())",
        js_str!("[object Error]"),
    )]);
}

#[test]
fn function_prototype_call_throw() {
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            let call = Function.prototype.call;
            call(call)
        "#},
        JsNativeErrorKind::Type,
        "undefined is not a function",
    )]);
}

#[test]
fn function_prototype_call_multiple_args() {
    run_test_actions([
        TestAction::run(indoc! {r#"
            function f(a, b) {
                this.a = a;
                this.b = b;
            }
            let o = {a: 0, b: 0};
            f.call(o, 1, 2);
        "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn function_prototype_apply() {
    run_test_actions([
        TestAction::run("const numbers = [6, 7, 3, 4, 2]"),
        TestAction::assert_eq("Math.max.apply(null, numbers)", 7),
        TestAction::assert_eq("Math.min.apply(null, numbers)", 2),
    ]);
}

#[test]
fn function_prototype_apply_on_object() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function f(a, b) {
                    this.a = a;
                    this.b = b;
                }
                let o = {a: 0, b: 0};
                f.apply(o, [1, 2]);
            "#}),
        TestAction::assert_eq("o.a", 1),
        TestAction::assert_eq("o.b", 2),
    ]);
}

#[test]
fn closure_capture_clone() {
    run_test_actions([
        TestAction::inspect_context(|ctx| {
            let string = js_string!("Hello");
            let object = JsObject::with_object_proto(ctx.intrinsics());
            object
                .define_property_or_throw(
                    js_string!("key"),
                    PropertyDescriptor::builder()
                        .value(js_string!(" world!"))
                        .writable(false)
                        .enumerable(false)
                        .configurable(false),
                    ctx.access(),
                )
                .unwrap();

            let func = FunctionObjectBuilder::new(
                ctx.realm(),
                NativeFunction::from_fn_with_captures(
                    |_, _, captures, mut context| {
                        let (string, object) = &captures;

                        let hw = js_string!(
                            string,
                            &object
                                .__get_own_property__(
                                    &js_string!("key").into(),
                                    &mut context.reborrow().into_owner().into()
                                )?
                                .and_then(|prop| prop.value().and_then(JsValue::as_string))
                                .ok_or_else(
                                    || JsNativeError::typ().with_message("invalid `key` property")
                                )?
                        );
                        Ok(hw.into())
                    },
                    (string, object),
                ),
            )
            .name(js_str!("closure"))
            .build()
            .unwrap();

            ctx.register_global_property(js_str!("closure"), func, Attribute::default())
                .unwrap();
        }),
        TestAction::assert_eq("closure()", js_str!("Hello world!")),
    ]);
}

#[test]
fn function_constructor_early_errors_super() {
    run_test_actions([
        TestAction::assert_native_error(
            "Function('super()')()",
            JsNativeErrorKind::Syntax,
            "invalid `super` call",
        ),
        TestAction::assert_native_error(
            "Function('super.a')()",
            JsNativeErrorKind::Syntax,
            "invalid `super` reference",
        ),
    ]);
}

#[test]
fn dynamic_function_private_parameters_reject_undeclared_names() {
    use crate::{Context, Source};

    let mut context = Context::default();
    let mut unexpected = Vec::new();
    for constructor in [
        "Function",
        "(function*() {}).constructor",
        "(async function() {}).constructor",
        "(async function*() {}).constructor",
    ] {
        for (initializer, body) in [
            ("arg.#missing", ""),
            ("(() => arg.#missing)", "return value;"),
        ] {
            let source = format!("{constructor}('arg', 'value = {initializer}', '{body}');");
            match context.eval(Source::from_bytes(&source)) {
                Err(error)
                    if error
                        .as_native()
                        .is_some_and(|error| error.kind() == &JsNativeErrorKind::Syntax) => {}
                result => unexpected.push((source, result)),
            }
        }

        let source = format!(
            "typeof {constructor}('value = class {{ #ok; get() {{ return this.#ok; }} }}', '')"
        );
        assert_eq!(
            context.eval(Source::from_bytes(&source)).unwrap(),
            JsValue::from(js_string!("function"))
        );
    }

    assert!(unexpected.is_empty(), "{unexpected:?}");
}
