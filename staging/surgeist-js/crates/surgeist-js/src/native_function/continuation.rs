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
struct Coroutine<F, T>
where
    F: Fn(CompletionRecord, &T, &mut Context) -> CoroutineState + Copy,
    T: Trace,
{
    // A Copy callback cannot own collector handles. Captures are traced separately.
    #[unsafe_ignore_trace]
    f: F,
    captures: T,
}

impl<F, T> TraceableCoroutine for Coroutine<F, T>
where
    F: Fn(CompletionRecord, &T, &mut Context) -> CoroutineState + Copy,
    T: Trace,
{
    fn call(&self, completion: CompletionRecord, context: &mut Context) -> CoroutineState {
        (self.f)(completion, &self.captures, context)
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
    /// Creates a `NativeCoroutine` from a `Copy` closure and a list of traceable captures.
    pub(crate) fn from_copy_closure_with_captures<F, T>(closure: F, captures: T) -> Self
    where
        F: Fn(CompletionRecord, &T, &mut Context) -> CoroutineState + Copy + 'static,
        T: Trace + 'static,
    {
        Self {
            inner: Gc::new(Box::new(Coroutine {
                f: closure,
                captures,
            })),
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
