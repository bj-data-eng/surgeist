use std::{cell::RefCell, ops::ControlFlow};

use surgeist_js_gc::heap::{RecordRef, Rooted, Trace};

use crate::{Context, JsError, JsResult, JsValue, vm::CompletionRecord};

use super::NativeCapture;

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

/// Stores suspended state as heap edges. Resumption takes live roots before
/// removing the stored state so it can call into JavaScript without borrowing
/// the slot or losing any of the captured objects.
pub(crate) struct CoroutineSlot<S: NativeCapture>(RefCell<Option<S::Stored>>);

impl<S: NativeCapture> CoroutineSlot<S> {
    pub(crate) fn take(&self) -> Option<S> {
        let state = self.0.borrow().as_ref().map(S::restore);
        *self.0.borrow_mut() = None;
        state
    }

    pub(crate) fn set(&self, state: S) {
        *self.0.borrow_mut() = Some(state.store());
    }
}

impl<S: NativeCapture> Trace for CoroutineSlot<S> {
    fn trace(
        &self,
        edges: &mut surgeist_js_gc::heap::EdgeVisitor<'_>,
    ) -> Result<(), surgeist_js_gc::heap::AccessError> {
        self.0.trace(edges)
    }
}

trait TraceableCoroutine: Trace {
    fn call(&self, completion: CompletionRecord, context: &mut Context) -> CoroutineState;
}

#[derive(Trace)]
struct Coroutine<C: NativeCapture, S: NativeCapture> {
    #[heap_trace(skip)]
    callback: fn(CompletionRecord, &C, &CoroutineSlot<S>, &mut Context) -> CoroutineState,
    captures: C::Stored,
    state: CoroutineSlot<S>,
}

impl<C: NativeCapture, S: NativeCapture> TraceableCoroutine for Coroutine<C, S> {
    fn call(&self, completion: CompletionRecord, context: &mut Context) -> CoroutineState {
        (self.callback)(
            completion,
            &C::restore(&self.captures),
            &self.state,
            context,
        )
    }
}

#[derive(Trace)]
struct CoroutineRecord {
    coroutine: Box<dyn TraceableCoroutine>,
}

/// A callable Rust coroutine that can be used to await promises.
#[derive(Clone, Debug)]
pub(crate) struct NativeCoroutine(Rooted<CoroutineRecord>);

#[derive(Debug, Trace)]
pub(crate) struct StoredNativeCoroutine(RecordRef<CoroutineRecord>);

impl NativeCapture for NativeCoroutine {
    type Stored = StoredNativeCoroutine;

    fn store(&self) -> Self::Stored {
        StoredNativeCoroutine(self.0.record_ref())
    }

    fn restore(stored: &Self::Stored) -> Self {
        Self(
            stored
                .0
                .upgrade()
                .expect("continuation heap is available")
                .expect("live owner retains continuation"),
        )
    }
}

impl NativeCoroutine {
    /// Creates a coroutine with explicit immutable captures and mutable state.
    pub(crate) fn from_fn_with_state<C: NativeCapture, S: NativeCapture>(
        callback: fn(CompletionRecord, &C, &CoroutineSlot<S>, &mut Context) -> CoroutineState,
        captures: C,
        state: S,
        context: &Context,
    ) -> Self {
        let record = CoroutineRecord {
            coroutine: Box::new(Coroutine::<C, S> {
                callback,
                captures: captures.store(),
                state: CoroutineSlot(RefCell::new(Some(state.store()))),
            }),
        };
        Self(
            context
                .runtime()
                .heap
                .allocate(record)
                .expect("continuation references belong to its runtime"),
        )
    }

    /// Calls this coroutine while its record is rooted.
    pub(crate) fn call(
        &self,
        completion: CompletionRecord,
        context: &mut Context,
    ) -> CoroutineState {
        self.0.coroutine.call(completion, context)
    }
}
