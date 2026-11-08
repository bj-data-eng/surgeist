use std::cell::RefCell;

use std::task::Poll;
use surgeist_js::{Context, ContextAccess, JsResult, JsValue, NativeFunction, job::ContextTask};

fn callback(_: JsValue, _: Vec<JsValue>, _: ContextAccess<'_>) -> JsResult<ContextTask<JsValue>> {
    Ok(ContextTask::from_poll(|_, host| {
        let _owner_cell: &RefCell<&mut Context> = host;
        Poll::Ready(Ok(JsValue::undefined()))
    }))
}

fn main() {
    let _callback = NativeFunction::from_async_fn(callback);
}
