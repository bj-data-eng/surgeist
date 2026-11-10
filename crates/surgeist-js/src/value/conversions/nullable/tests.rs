use crate::value::TryIntoJs;
use surgeist_js::value::Nullable;
use surgeist_js::{Context, JsResult, JsValue};

#[test]
fn not_null() {
    let context = &mut Context::builder().build().unwrap();
    let v: Nullable<i32> = JsValue::new(42)
        .try_js_into(context.access())
        .expect("Failed to convert value from js");

    assert!(!v.is_null());
    assert!(v.is_not_null());
    assert_eq!(v, Nullable::NonNull(42));

    assert_eq!(v.try_into_js(context.access()).unwrap(), JsValue::new(42));
}

#[test]
fn null() {
    let context = &mut Context::builder().build().unwrap();
    let v: Nullable<i32> = JsValue::null()
        .try_js_into(context.access())
        .expect("Failed to convert value from js");

    assert!(v.is_null());
    assert!(!v.is_not_null());
    assert_eq!(v, Nullable::Null);

    assert_eq!(v.try_into_js(context.access()).unwrap(), JsValue::null());
}

#[test]
fn invalid() {
    let context = &mut Context::builder().build().unwrap();
    let v: JsResult<Nullable<i32>> = JsValue::undefined().try_js_into(context.access());

    assert!(v.is_err());
}
