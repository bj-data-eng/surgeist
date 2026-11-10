#![allow(unused_crate_dependencies)]

use std::cell::{Cell, RefCell};

use surgeist_js::{
    Context, ContextAccess, JsData, JsError, JsNativeError, JsNativeErrorKind, JsResult, JsValue,
    Source, boa_class, boa_module,
    gc::heap::Trace as HeapTrace,
    js_object, js_string, js_value,
    object::builtins::JsFunction,
    value::{TryFromJs, TryIntoJs},
};

#[derive(Debug, TryFromJs, TryIntoJs)]
struct ConvertedRecord {
    #[boa(
        rename = "left",
        from_js_with = "decode_left",
        into_js_with = "encode_left"
    )]
    first: i32,
    #[boa(
        rename = "right",
        from_js_with = "decode_right",
        into_js_with = "encode_right"
    )]
    second: i32,
    #[boa(skip)]
    hidden: i32,
}

fn decode_left(value: &JsValue, mut access: ContextAccess<'_>) -> JsResult<i32> {
    access.eval(Source::from_bytes("conversionLog.push('decode-left')"))?;
    i32::try_from_js(value, access.reborrow())
}

fn decode_right(value: &JsValue, mut access: ContextAccess<'_>) -> JsResult<i32> {
    access.eval(Source::from_bytes("conversionLog.push('decode-right')"))?;
    value.try_js_into(access.reborrow())
}

fn encode_left(value: &i32, mut access: ContextAccess<'_>) -> JsResult<JsValue> {
    access.eval(Source::from_bytes("conversionLog.push('encode-left')"))?;
    Ok(JsValue::new(*value))
}

fn encode_right(value: &i32, mut access: ContextAccess<'_>) -> JsResult<JsValue> {
    access.eval(Source::from_bytes("conversionLog.push('encode-right')"))?;
    Ok(JsValue::new(*value))
}

#[test]
fn derived_conversions_reborrow_for_each_custom_field_and_preserve_rename_and_skip() {
    let mut owner = Context::builder().build().unwrap();
    let value = owner
        .eval(Source::from_bytes(
            "globalThis.conversionLog = []; ({ left: 7, right: 35, hidden: 42 })",
        ))
        .unwrap();
    let mut access = owner.access();
    let record = ConvertedRecord::try_from_js(&value, access.reborrow()).unwrap();
    assert_eq!(record.first + record.second, 42);
    assert_eq!(record.hidden, 42);
    let output = record
        .try_into_js(access.reborrow())
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(
        output.get(js_string!("left"), access.reborrow()).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        output.get(js_string!("right"), access.reborrow()).unwrap(),
        JsValue::new(35)
    );
    assert!(
        !output
            .has_own_property(js_string!("hidden"), access.reborrow())
            .unwrap()
    );
    assert_eq!(
        access
            .eval(Source::from_bytes("conversionLog.join(',')"))
            .unwrap(),
        JsValue::new(js_string!(
            "decode-left,decode-right,encode-left,encode-right"
        ))
    );
}

#[derive(Debug, JsData, HeapTrace)]
struct Gauge {
    value: i32,
}

#[boa_class]
impl Gauge {
    #[boa(constructor)]
    fn new(value: i32, mut access: ContextAccess<'_>) -> Self {
        access
            .eval(Source::from_bytes("events.push('constructor')"))
            .expect("constructor reentry");
        Self { value }
    }

    fn add(&self, delta: i32, mut access: ContextAccess<'_>) -> JsResult<i32> {
        access.eval(Source::from_bytes("events.push('method')"))?;
        Ok(self.value + delta)
    }

    #[boa(static)]
    fn answer(mut access: ContextAccess<'_>) -> JsResult<i32> {
        access.eval(Source::from_bytes("events.push('static')"))?;
        Ok(42)
    }

    #[boa(getter)]
    fn value(&self, mut access: ContextAccess<'_>) -> JsResult<i32> {
        access.eval(Source::from_bytes("events.push('getter')"))?;
        Ok(self.value)
    }

    #[boa(setter)]
    #[boa(rename = "value")]
    fn set_value(&mut self, value: i32, mut access: ContextAccess<'_>) -> JsResult<()> {
        self.value = value;
        access.eval(Source::from_bytes("events.push('setter')"))?;
        Ok(())
    }
}

#[test]
fn generated_class_constructor_methods_static_and_accessors_receive_reentrant_access() {
    let mut owner = Context::builder().build().unwrap();
    owner
        .eval(Source::from_bytes("globalThis.events = []"))
        .unwrap();
    owner.register_global_class::<Gauge>().unwrap();
    assert_eq!(owner.eval(Source::from_bytes(
        "const gauge = new Gauge(40); const answer = gauge.add(2); gauge.value = 7; answer === 42 && gauge.value === 7 && Gauge.answer() === 42",
    )).unwrap(), JsValue::new(true));
    assert_eq!(
        owner.eval(Source::from_bytes("events.join(',')")).unwrap(),
        JsValue::new(js_string!("constructor,method,setter,getter,static"))
    );
}

#[boa_module]
mod protected_module {
    use surgeist_js::{ContextAccess, JsResult, Source};

    fn answer(mut access: ContextAccess<'_>) -> JsResult<u32> {
        access.eval(Source::from_bytes("moduleLog.push('function')"))?;
        Ok(42)
    }

    type Gauge = super::Gauge;

    #[boa(rename = "marker")]
    const MARK: i32 = 7;
}

#[test]
fn generated_module_exports_protected_functions_classes_and_renamed_constants() {
    let mut owner = Context::builder().build().unwrap();
    owner
        .eval(Source::from_bytes(
            "globalThis.events = []; globalThis.moduleLog = []",
        ))
        .unwrap();
    let module = protected_module::boa_module(None, owner.access()).unwrap();
    module
        .load_link_evaluate(owner.access())
        .unwrap()
        .await_blocking(owner.access())
        .unwrap();
    let function = module
        .get_value(js_string!("answer"), owner.access())
        .unwrap()
        .as_object()
        .unwrap();
    let function = JsFunction::from_object(function).unwrap();
    assert_eq!(
        function
            .call(&JsValue::undefined(), &[], owner.access())
            .unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        module
            .get_value(js_string!("marker"), owner.access())
            .unwrap(),
        JsValue::new(7)
    );
    let constructor = module
        .get_value(js_string!("Gauge"), owner.access())
        .unwrap()
        .as_object()
        .unwrap();
    let object = constructor
        .construct(&[JsValue::new(7)], None, owner.access())
        .unwrap();
    assert_eq!(
        object.get(js_string!("value"), owner.access()).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        owner
            .eval(Source::from_bytes("moduleLog.join(',')"))
            .unwrap(),
        JsValue::new(js_string!("function"))
    );
}

#[test]
fn literal_context_expression_runs_once_before_values_keys_and_nested_construction() {
    let mut owner = Context::builder().build().unwrap();
    let order = RefCell::new(Vec::new());
    let value = || {
        order.borrow_mut().push("value");
        42
    };
    let key = || {
        order.borrow_mut().push("key");
        js_string!("answer")
    };
    let object = js_object!({
        [key()]: value(),
        nested: [7, { answer: 35 }],
    }, { order.borrow_mut().push("access"); owner.access() })
    .unwrap();
    assert_eq!(order.into_inner(), ["access", "value", "key"]);
    assert_eq!(
        object.get(js_string!("answer"), owner.access()).unwrap(),
        JsValue::new(42)
    );
    let array = object
        .get(js_string!("nested"), owner.access())
        .unwrap()
        .as_object()
        .unwrap();
    assert_eq!(array.get(0, owner.access()).unwrap(), JsValue::new(7));
    let nested = array.get(1, owner.access()).unwrap().as_object().unwrap();
    assert_eq!(
        nested.get(js_string!("answer"), owner.access()).unwrap(),
        JsValue::new(35)
    );
}

struct AccessExpression<'a> {
    access: ContextAccess<'a>,
}

#[test]
fn literals_accept_general_moved_and_temporary_owner_capability_expressions() {
    let mut owner = Context::builder().build().unwrap();
    let object =
        js_object!({ answer: 42 }, AccessExpression { access: owner.access() }.access).unwrap();
    assert_eq!(
        object.get(js_string!("answer"), owner.access()).unwrap(),
        JsValue::new(42)
    );
    let moved = owner.access();
    let array = js_value!([7, 35], moved).unwrap().as_object().unwrap();
    assert_eq!(array.get(1, owner.access()).unwrap(), JsValue::new(35));
    let runtime = owner.runtime().clone();
    let temporary =
        js_object!({ answer: 42 }, Context::builder().runtime(runtime).build().unwrap().access())
            .unwrap();
    assert_eq!(
        temporary.get(js_string!("answer"), owner.access()).unwrap(),
        JsValue::new(42)
    );
}

fn replace_local_access<'a>(
    binding: &mut ContextAccess<'a>,
    replacement: ContextAccess<'a>,
) -> i32 {
    let _previous = std::mem::replace(binding, replacement);
    binding
        .eval(Source::from_bytes("globalThis.rebound = 7"))
        .unwrap();
    42
}

#[test]
fn literal_custody_is_hygienic_and_cannot_be_redirected_by_a_caller_capability_binding() {
    let mut owner = Context::builder().build().unwrap();
    let mut second = Context::builder().build().unwrap();
    let mut third = Context::builder().build().unwrap();
    let object = 40;
    let value = 2;
    let element = 7;
    let context = 35;
    let result = {
        let mut caller = owner.access();
        js_object!({
            answer: object + value,
            rebound: replace_local_access(&mut caller, third.access()),
            element: element,
            context: context,
        }, std::mem::replace(&mut caller, second.access()))
        .unwrap()
    };
    assert_eq!(
        result.get(js_string!("answer"), owner.access()).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        result.get(js_string!("rebound"), owner.access()).unwrap(),
        JsValue::new(42)
    );
    assert_eq!(
        result.get(js_string!("element"), owner.access()).unwrap(),
        JsValue::new(7)
    );
    assert_eq!(
        result.get(js_string!("context"), owner.access()).unwrap(),
        JsValue::new(35)
    );
    assert_eq!(
        owner
            .eval(Source::from_bytes("typeof rebound === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
    assert_eq!(
        third.eval(Source::from_bytes("rebound")).unwrap(),
        JsValue::new(7)
    );
}

fn return_from_literal(owner: &mut Context, leave: bool) -> u32 {
    let _ = js_value!([{ value: if leave { return 42 } else { 7 } }], owner.access()).unwrap();
    7
}

fn question_from_literal(
    owner: &mut Context,
    fail: bool,
    siblings: &Cell<i32>,
) -> JsResult<JsValue> {
    js_value!([
        if fail { Err::<i32, JsError>(JsNativeError::typ().with_message("literal failure").into())? } else { 42 },
        { next: siblings.replace(siblings.get() + 1) },
    ], owner.access())
}

#[test]
fn embedded_return_break_and_question_mark_keep_the_callers_control_flow() {
    let mut owner = Context::builder().build().unwrap();
    assert_eq!(return_from_literal(&mut owner, true), 42);
    assert_eq!(return_from_literal(&mut owner, false), 7);
    let siblings = Cell::new(0);
    let error = question_from_literal(&mut owner, true, &siblings).unwrap_err();
    assert_eq!(error.as_native().unwrap().message(), "literal failure");
    assert_eq!(siblings.get(), 0);
    question_from_literal(&mut owner, false, &siblings).unwrap();
    assert_eq!(siblings.get(), 1);
    let mut iterations = 0;
    for number in 0..3 {
        let _ = js_value!([{ value: if number == 1 { break } else { number } }], owner.access())
            .unwrap();
        iterations += 1;
    }
    assert_eq!(iterations, 1);
}

struct Probe<'a>(&'a RefCell<Vec<&'static str>>);

impl Drop for Probe<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().push("drop");
    }
}

#[test]
fn literal_arrays_keep_temporaries_and_sibling_evaluation_order_on_nested_failure() {
    let mut owner = Context::builder().build().unwrap();
    let foreign = Context::builder().build().unwrap();
    let foreign_object = surgeist_js::JsObject::with_null_proto(foreign.runtime()).unwrap();
    for variant in 0..3 {
        let order = RefCell::new(Vec::new());
        let first = |probe: &Probe<'_>| {
            probe.0.borrow_mut().push("first");
            1
        };
        let next = || {
            order.borrow_mut().push("next");
            2
        };
        let result = match variant {
            0 => js_value!([first(&Probe(&order)), next()], owner.access()),
            1 => js_value!([[first(&Probe(&order))], next()], owner.access()),
            _ => js_value!(
                [[first(&Probe(&order)), foreign_object.clone()], next()],
                owner.access()
            ),
        };
        if variant == 2 {
            assert_eq!(
                result.unwrap_err().as_native().unwrap().kind(),
                &JsNativeErrorKind::Type
            );
        } else {
            result.unwrap();
        }
        assert_eq!(order.into_inner(), ["first", "next", "drop"]);
    }
}

#[test]
fn protected_open_boundaries_reject_owner_and_lifetime_escape() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/protected_access/raw_context_tail.rs");
    cases.compile_fail("tests/protected_access/converter_access_escape.rs");
    cases.compile_fail("tests/protected_access/builder_access_escape.rs");
}
