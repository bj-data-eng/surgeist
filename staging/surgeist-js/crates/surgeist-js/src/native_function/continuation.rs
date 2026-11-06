use std::ops::ControlFlow;

use surgeist_js_gc::{Finalize, Gc, Trace};

use crate::{Context, JsError, JsResult, JsValue, vm::CompletionRecord};

/// Utility trait to make it easy to break from a coroutine using the `?` operator.
pub(crate) trait CoroutineBranch<T> {
    fn branch(self) -> ControlFlow<JsResult<()>, T>;
}

impl<T, E> CoroutineBranch<T> for Result<T, E>
where
    E: Into<JsError>,
{
    fn branch(self) -> ControlFlow<JsResult<()>, T> {
        match self {
            Ok(v) => ControlFlow::Continue(v),
            Err(e) => ControlFlow::Break(Err(e.into())),
        }
    }
}

impl CoroutineBranch<JsValue> for CompletionRecord {
    fn branch(self) -> ControlFlow<JsResult<()>, JsValue> {
        match self {
            CompletionRecord::Normal(val) => ControlFlow::Continue(val),
            CompletionRecord::Return(_) => ControlFlow::Break(Ok(())),
            CompletionRecord::Throw(err) => ControlFlow::Break(Err(err)),
        }
    }
}

pub(crate) type CoroutineState = ControlFlow<JsResult<()>, JsValue>;

trait TraceableCoroutine: Trace {
    fn call(&self, completion: CompletionRecord, context: &mut Context) -> CoroutineState;
}

#[derive(Trace, Finalize)]
struct Coroutine<T: Trace> {
    // A function pointer holds no JavaScript state. Captures are traced separately.
    #[unsafe_ignore_trace]
    callback: fn(CompletionRecord, &T, &mut Context) -> CoroutineState,
    captures: T,
}

impl<T: Trace> TraceableCoroutine for Coroutine<T> {
    fn call(&self, completion: CompletionRecord, context: &mut Context) -> CoroutineState {
        (self.callback)(completion, &self.captures, context)
    }
}

/// A callable Rust coroutine that can be used to await promises.
///
/// Owned coroutine state is supplied as explicit captures for tracing.
#[derive(Clone, Trace, Finalize)]
pub(crate) struct NativeCoroutine {
    inner: Gc<Box<dyn TraceableCoroutine>>,
}

impl std::fmt::Debug for NativeCoroutine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeCoroutine").finish_non_exhaustive()
    }
}

impl NativeCoroutine {
    /// Creates a callback with all stored state in its traceable captures.
    pub(crate) fn from_fn_with_captures<T: Trace + 'static>(
        callback: fn(CompletionRecord, &T, &mut Context) -> CoroutineState,
        captures: T,
    ) -> Self {
        Self {
            inner: Gc::new(Box::new(Coroutine { callback, captures })),
        }
    }

    /// Calls this `NativeCoroutine`, forwarding the arguments to the corresponding function.
    #[inline]
    pub(crate) fn call(
        &self,
        completion: CompletionRecord,
        context: &mut Context,
    ) -> CoroutineState {
        self.inner.call(completion, context)
    }
}
