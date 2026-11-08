use surgeist_js::{Context, IntoJsFunction, JsResult, JsValue};

fn raw_owner(_: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}

fn main() {
    let pointer: fn(&mut Context) -> JsResult<JsValue> = raw_owner;
    let _ = pointer.into_js_function();
}
