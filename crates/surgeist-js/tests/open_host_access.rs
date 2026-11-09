#![allow(unused_crate_dependencies)]

use surgeist_js::{
    Context, ContextAccess, IntoJsFunction, IntoJsFunctionWithCaptures, JsData, JsNativeError,
    JsNativeErrorKind, JsObject, JsResult, JsValue, NativeFunction, Source, TryIntoJsResult,
    builtins::promise::PromiseState,
    class::{Class, ClassBuilder},
    error::{EngineError, RuntimeLimitError},
    execution::ExecutionOptions,
    gc::heap::Trace as HeapTrace,
    interop::TryFromJsArgument,
    js_string,
    module::IntoJsModule,
    object::{
        ConstructorBuilder, ObjectInitializer,
        builtins::{JsFunction, JsPromise, TryIntoJsArguments},
    },
    property::Attribute,
    value::{TryFromJs, TryIntoJs},
};

fn instruction_error(limit: u64) -> EngineError {
    EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit })
}

#[test]
fn plain_and_captured_native_reentry_share_exact_instruction_limits() {
    let control = Context::default()
        .eval_with_budget(Source::from_bytes("40 + 2"), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    let exact = control.report.instructions().unwrap();
    let plain =
        NativeFunction::from_fn_ptr(|_, _, mut access| access.eval(Source::from_bytes("40 + 2")));
    let captured = NativeFunction::from_fn_with_captures(
        |_, _, captured, mut access| {
            assert_eq!(*captured, 7_u32);
            access.eval(Source::from_bytes("40 + 2"))
        },
        7_u32,
    );
    for native in [plain, captured] {
        for limit in [exact, exact - 1] {
            let mut owner = Context::default();
            let outcome = owner
                .with_execution(ExecutionOptions::new(limit), |mut access| {
                    native.call(&JsValue::undefined(), &[], access.reborrow())
                })
                .unwrap();
            assert_eq!(outcome.report.instructions().unwrap(), limit);
            if limit == exact {
                assert_eq!(outcome.result.unwrap(), JsValue::new(42));
            } else {
                assert_eq!(
                    outcome.result.unwrap_err().as_engine(),
                    Some(&instruction_error(limit))
                );
            }
        }
    }
}

#[test]
fn native_callbacks_cannot_replace_the_first_stop_with_success_or_another_error() {
    let plain = NativeFunction::from_fn_ptr(|_, _, mut access| {
        let first = access.eval(Source::from_bytes("globalThis.committed = 7; try { while (true) {} } finally { globalThis.finalized = true; }")).unwrap_err();
        assert_eq!(first.as_engine(), Some(&instruction_error(32)));
        Ok(JsValue::new(42))
    });
    let captured = NativeFunction::from_fn_with_captures(
        |_, _, captured, mut access| {
            assert_eq!(*captured, 7_u32);
            let first = access.eval(Source::from_bytes("globalThis.committed = 7; try { while (true) {} } finally { globalThis.finalized = true; }")).unwrap_err();
            assert_eq!(first.as_engine(), Some(&instruction_error(32)));
            Err(JsNativeError::typ().with_message("later host error").into())
        },
        7_u32,
    );
    for native in [plain, captured] {
        let mut owner = Context::default();
        let outcome = owner
            .with_execution(ExecutionOptions::new(32), |mut access| {
                native.call(&JsValue::undefined(), &[], access.reborrow())
            })
            .unwrap();
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&instruction_error(32))
        );
        assert_eq!(outcome.report.instructions().unwrap(), 32);
        assert_eq!(
            owner
                .eval(Source::from_bytes(
                    "committed === 7 && typeof finalized === 'undefined'"
                ))
                .unwrap(),
            JsValue::new(true)
        );
        assert_eq!(
            owner.eval(Source::from_bytes("42")).unwrap(),
            JsValue::new(42)
        );
    }
}

struct LoggedArgument(i32);

impl TryIntoJsResult for LoggedArgument {
    fn try_into_js_result(self, mut access: ContextAccess<'_>) -> JsResult<JsValue> {
        access.eval(Source::from_bytes("trace.push('argument')"))?;
        Ok(JsValue::new(self.0))
    }
}

struct LoggedArguments;

impl TryIntoJsArguments for LoggedArguments {
    fn into_js_args(self, mut access: ContextAccess<'_>) -> JsResult<Vec<JsValue>> {
        access.eval(Source::from_bytes("trace.push('arguments')"))?;
        Ok(vec![
            LoggedArgument(40).try_into_js_result(access.reborrow())?,
        ])
    }
}

#[derive(Debug)]
struct LoggedResult(i32);

impl TryFromJs for LoggedResult {
    fn try_from_js(value: &JsValue, mut access: ContextAccess<'_>) -> JsResult<Self> {
        access.eval(Source::from_bytes("trace.push('result')"))?;
        Ok(Self(i32::try_from_js(value, access.reborrow())?))
    }
}

#[test]
fn typed_call_keeps_argument_guest_and_result_conversion_order_and_one_budget() {
    let mut owner = Context::default();
    let function = owner
        .eval(Source::from_bytes(
            "globalThis.trace = []; (function(value) { trace.push('call'); return value + 2; })",
        ))
        .unwrap()
        .as_object()
        .unwrap();
    let function = JsFunction::from_object(function)
        .unwrap()
        .typed::<LoggedArguments, LoggedResult>();
    let control = owner
        .with_execution(ExecutionOptions::new(512), |mut access| {
            function.call(access.reborrow(), LoggedArguments)
        })
        .unwrap();
    assert_eq!(control.result.unwrap().0, 42);
    assert_eq!(
        owner.eval(Source::from_bytes("trace.join(',')")).unwrap(),
        JsValue::new(js_string!("arguments,argument,call,result"))
    );
    let exact = control.report.instructions().unwrap();
    assert!(exact > 1);
    for limit in [exact, exact - 1] {
        owner.eval(Source::from_bytes("trace = []")).unwrap();
        let outcome = owner
            .with_execution(ExecutionOptions::new(limit), |mut access| {
                function.call(access.reborrow(), LoggedArguments)
            })
            .unwrap();
        assert_eq!(outcome.report.instructions().unwrap(), limit);
        if limit == exact {
            assert_eq!(outcome.result.unwrap().0, 42);
        } else {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&instruction_error(limit))
            );
        }
    }
}

struct HostArgument(i32);

impl<'a> TryFromJsArgument<'a> for HostArgument {
    fn try_from_js_argument(
        _: &'a JsValue,
        rest: &'a [JsValue],
        mut access: ContextAccess<'_>,
    ) -> JsResult<(Self, &'a [JsValue])> {
        access.eval(Source::from_bytes("trace.push('host-argument')"))?;
        let (first, remaining) = rest.split_first().expect("one authored argument");
        Ok((Self(i32::try_from_js(first, access.reborrow())?), remaining))
    }
}

struct HostReturn(i32);

impl TryIntoJs for HostReturn {
    fn try_into_js(&self, mut access: ContextAccess<'_>) -> JsResult<JsValue> {
        access.eval(Source::from_bytes("trace.push('host-result')"))?;
        Ok(JsValue::new(self.0))
    }
}

#[test]
fn typed_native_adapter_reborrows_for_argument_callback_and_result_hooks() {
    let callback: for<'a> fn(HostArgument, ContextAccess<'a>) -> JsResult<HostReturn> =
        |value, mut access| {
            access.eval(Source::from_bytes("trace.push('host-call')"))?;
            Ok(HostReturn(value.0 + 2))
        };
    let captured: for<'a> fn(&u32, HostArgument, ContextAccess<'a>) -> JsResult<HostReturn> =
        |captured, value, mut access| {
            access.eval(Source::from_bytes("trace.push('host-call')"))?;
            Ok(HostReturn(value.0 + i32::try_from(*captured).unwrap()))
        };
    for native in [
        callback.into_js_function(),
        captured.into_js_function_with_captures(2_u32),
    ] {
        let mut owner = Context::default();
        owner
            .eval(Source::from_bytes("globalThis.trace = []"))
            .unwrap();
        let outcome = owner
            .with_execution(ExecutionOptions::new(256), |mut access| {
                native.call(
                    &JsValue::undefined(),
                    &[JsValue::new(40)],
                    access.reborrow(),
                )
            })
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::new(42));
        assert!(outcome.report.instructions().unwrap() > 0);
        assert_eq!(
            owner.eval(Source::from_bytes("trace.join(',')")).unwrap(),
            JsValue::new(js_string!("host-argument,host-call,host-result"))
        );
    }
}

struct IndependentlySelectedArguments;

impl TryIntoJsArguments for IndependentlySelectedArguments {
    fn into_js_args(self, mut access: ContextAccess<'_>) -> JsResult<Vec<JsValue>> {
        let mut other = Context::default();
        let mut selected = access.reborrow();
        let _original = std::mem::replace(&mut selected, other.access());
        assert_eq!(
            selected.eval(Source::from_bytes("globalThis.otherValue = 42"))?,
            JsValue::new(42)
        );
        Ok(vec![JsValue::new(2)])
    }
}

#[test]
fn custom_argument_converter_cannot_redirect_the_typed_adapters_original_owner() {
    let mut owner = Context::default();
    let function = owner
        .eval(Source::from_bytes(
            "(function(value) { globalThis.called = true; return value + 40; })",
        ))
        .unwrap()
        .as_object()
        .unwrap();
    let function = JsFunction::from_object(function)
        .unwrap()
        .typed::<IndependentlySelectedArguments, u32>();
    let outcome = owner
        .with_execution(ExecutionOptions::new(0), |mut access| {
            function.call(access.reborrow(), IndependentlySelectedArguments)
        })
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&instruction_error(0))
    );
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(
        owner
            .eval(Source::from_bytes(
                "typeof called === 'undefined' && typeof otherValue === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
    let fresh = owner
        .with_execution(ExecutionOptions::new(64), |mut access| {
            function.call(access.reborrow(), IndependentlySelectedArguments)
        })
        .unwrap();
    assert_eq!(fresh.result.unwrap(), 42);
    assert!(fresh.report.instructions().unwrap() > 0);
}

#[derive(Debug, JsData, HeapTrace)]
struct HookClass {
    value: i32,
}

impl Class for HookClass {
    const NAME: &'static str = "HookClass";

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class
            .context()
            .eval(Source::from_bytes("classLog.push('init')"))?;
        class.property(js_string!("prototypeValue"), 35, Attribute::all());
        Ok(())
    }

    fn data_constructor(
        _: &JsValue,
        args: &[JsValue],
        mut access: ContextAccess<'_>,
    ) -> JsResult<Self> {
        access.eval(Source::from_bytes("classLog.push('data')"))?;
        Ok(Self {
            value: i32::try_from_js(&args[0], access.reborrow())?,
        })
    }

    fn object_constructor(
        instance: &JsObject<Self>,
        _: &[JsValue],
        mut access: ContextAccess<'_>,
    ) -> JsResult<()> {
        access.eval(Source::from_bytes("classLog.push('object')"))?;
        let object = instance.clone().upcast();
        let value = object.downcast_ref::<Self>().unwrap().unwrap().value;
        object.set(js_string!("answer"), value, true, access.reborrow())?;
        Ok(())
    }

    fn construct(
        new_target: &JsValue,
        args: &[JsValue],
        mut access: ContextAccess<'_>,
    ) -> JsResult<JsObject> {
        access.eval(Source::from_bytes("classLog.push('construct')"))?;
        let data = Self::data_constructor(new_target, args, access.reborrow())?;
        Self::from_data(data, access.reborrow())
    }

    fn from_data(data: Self, mut access: ContextAccess<'_>) -> JsResult<JsObject> {
        access.eval(Source::from_bytes("classLog.push('from-data')"))?;
        let prototype = access.get_global_class::<Self>().unwrap().prototype();
        let object = JsObject::new(access.root_shape(), prototype, data)?;
        Self::object_constructor(&object, &[], access.reborrow())?;
        Ok(object.upcast())
    }
}

#[test]
fn every_class_hook_and_short_builder_getter_operates_on_the_reserved_context() {
    let mut owner = Context::default();
    owner
        .eval(Source::from_bytes("globalThis.classLog = []"))
        .unwrap();
    owner.register_global_class::<HookClass>().unwrap();
    let outcome = owner
        .with_execution(ExecutionOptions::new(512), |mut access| {
            let constructed = access
                .eval(Source::from_bytes("new HookClass(42)"))?
                .as_object()
                .unwrap();
            assert_eq!(
                constructed.get(js_string!("answer"), access.reborrow())?,
                JsValue::new(42)
            );
            assert_eq!(
                constructed.get(js_string!("prototypeValue"), access.reborrow())?,
                JsValue::new(35)
            );
            let direct = HookClass::from_data(HookClass { value: 7 }, access.reborrow())?;
            assert_eq!(
                direct.get(js_string!("answer"), access.reborrow())?,
                JsValue::new(7)
            );
            let object = {
                let mut builder = ObjectInitializer::new(access.reborrow());
                builder
                    .context()
                    .eval(Source::from_bytes("classLog.push('object-builder')"))?;
                builder.property(js_string!("answer"), 42, Attribute::all());
                builder.build()?
            };
            assert_eq!(
                object.get(js_string!("answer"), access.reborrow())?,
                JsValue::new(42)
            );
            let mut constructor = ConstructorBuilder::new(
                access.reborrow(),
                NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())),
            );
            constructor
                .context()
                .eval(Source::from_bytes("classLog.push('constructor-builder')"))?;
            let _constructor = constructor.build()?;
            Ok(())
        })
        .unwrap();
    outcome.result.unwrap();
    assert!(outcome.report.instructions().unwrap() > 0);
    assert_eq!(
        owner
            .eval(Source::from_bytes("classLog.join(',')"))
            .unwrap(),
        JsValue::new(js_string!(
            "init,construct,data,from-data,object,from-data,object,object-builder,constructor-builder"
        ))
    );
}

#[derive(Debug, JsData, HeapTrace)]
struct ForeignClass;

impl Class for ForeignClass {
    const NAME: &'static str = "ForeignClass";
    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let foreign = *class.context().remove_data::<JsObject>().unwrap();
        class.static_property(js_string!("foreign"), foreign, Attribute::all());
        Ok(())
    }
    fn data_constructor(_: &JsValue, _: &[JsValue], _: ContextAccess<'_>) -> JsResult<Self> {
        Ok(Self)
    }
}

#[test]
fn protected_class_builder_still_rejects_a_foreign_runtime_property() {
    let foreign = Context::default();
    let object = JsObject::with_null_proto(foreign.runtime());
    let mut owner = Context::default();
    assert!(owner.insert_data(object).is_none());
    let error = owner.register_global_class::<ForeignClass>().unwrap_err();
    assert_eq!(error.as_native().unwrap().kind(), &JsNativeErrorKind::Type);
    assert_eq!(
        owner
            .eval(Source::from_bytes("typeof ForeignClass === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn promise_executor_reenters_through_access_and_retains_the_first_stop() {
    let mut owner = Context::default();
    let completed = owner
        .with_execution(ExecutionOptions::new(64), |mut access| {
            JsPromise::new(
                |resolvers, mut executor| {
                    let result = executor.eval(Source::from_bytes("40 + 2"))?;
                    resolvers.resolve.call(
                        &JsValue::undefined(),
                        &[result],
                        executor.reborrow(),
                    )?;
                    Ok(JsValue::undefined())
                },
                access.reborrow(),
            )
        })
        .unwrap();
    assert!(completed.report.instructions().unwrap() > 0);
    assert_eq!(
        completed.result.unwrap().state().unwrap(),
        PromiseState::Fulfilled(JsValue::new(42))
    );
    let stopped = owner
        .with_execution(ExecutionOptions::new(0), |mut access| {
            JsPromise::new(
                |_, mut executor| {
                    assert_eq!(
                        executor
                            .eval(Source::from_bytes("0"))
                            .unwrap_err()
                            .as_engine(),
                        Some(&instruction_error(0))
                    );
                    Err(JsNativeError::typ()
                        .with_message("later executor error")
                        .into())
                },
                access.reborrow(),
            )
        })
        .unwrap();
    assert_eq!(
        stopped.result.unwrap_err().as_engine(),
        Some(&instruction_error(0))
    );
    assert_eq!(stopped.report.instructions().unwrap(), 0);
}

#[test]
fn open_module_conversion_exports_a_protected_reentrant_native_function() {
    let exported =
        NativeFunction::from_fn_ptr(|_, _, mut access| access.eval(Source::from_bytes("40 + 2")));
    let mut owner = Context::default();
    let outcome = owner
        .with_execution(ExecutionOptions::new(128), |mut access| {
            let module =
                vec![(js_string!("answer"), exported)].into_js_module(access.reborrow())?;
            module
                .load_link_evaluate(access.reborrow())?
                .await_blocking(access.reborrow())?;
            let answer = module
                .get_value(js_string!("answer"), access.reborrow())?
                .as_callable()
                .unwrap();
            answer.call(&JsValue::undefined(), &[], access.reborrow())
        })
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(42));
    assert!(outcome.report.instructions().unwrap() > 0);
}
