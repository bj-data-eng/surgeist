use std::cell::RefCell;

use surgeist_js::{Context, HostContext, JsValue, NativeFunction, job::BoxedFuture};

fn callback<'a>(_: &'a JsValue, _: &'a [JsValue], host: &'a HostContext<'_>) -> BoxedFuture<'a> {
    let _owner_cell: &RefCell<&mut Context> = host;
    Box::pin(async { Ok(JsValue::undefined()) })
}

fn main() {
    let _callback = NativeFunction::from_async_fn(callback);
}
