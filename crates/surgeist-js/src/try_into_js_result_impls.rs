//! Declare implementations of [`TryIntoJsResult`] trait for various types.

use crate::ContextAccess;
use crate::value::TryIntoJs;
use crate::{JsResult, JsValue, TryIntoJsResult};

impl<T> TryIntoJsResult for T
where
    T: TryIntoJs,
{
    fn try_into_js_result(self, mut ctx: ContextAccess<'_>) -> JsResult<JsValue> {
        self.try_into_js(ctx.reborrow())
    }
}

impl<T> TryIntoJsResult for JsResult<T>
where
    T: TryIntoJsResult,
{
    fn try_into_js_result(self, mut cx: ContextAccess<'_>) -> JsResult<JsValue> {
        self.and_then(|value| value.try_into_js_result(cx.reborrow()))
    }
}
