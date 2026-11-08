#![allow(unused)]

use surgeist_js::{
    ContextAccess, JsResult, JsValue,
    value::{TryFromJs, TryIntoJs},
};

#[derive(TryFromJs, TryIntoJs)]
struct Blah {
    #[boa(into_js_with = "my_custom_converter_to_js")]
    #[boa(from_js_with = "my_custom_converter_from_js")]
    x: i32,
}

fn my_custom_converter_to_js(value: &i32, _context: ContextAccess<'_>) -> JsResult<JsValue> {
    Ok(JsValue::new(*value))
}

fn my_custom_converter_from_js(value: &JsValue, context: ContextAccess<'_>) -> JsResult<i32> {
    value.try_js_into(context)
}

fn main() {}
