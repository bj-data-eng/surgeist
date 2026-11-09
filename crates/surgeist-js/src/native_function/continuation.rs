use std::{cell::RefCell, ops::ControlFlow};

use surgeist_js_gc::heap::{AccessError, RecordRef, Rooted, Trace};

use crate::{Context, JsError, JsResult, JsValue, error::RootingError, vm::CompletionRecord};

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
    pub(crate) fn take(&self) -> Result<Option<S>, RootingError> {
        let (state, stored) = {
            let mut slot = self
                .0
                .try_borrow_mut()
                .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
            let state = slot.as_ref().map(S::restore).transpose()?;
            (state, slot.take())
        };
        drop(stored);
        Ok(state)
    }

    pub(crate) fn set(&self, state: S) -> Result<(), RootingError> {
        let stored = state.store();
        let replaced = {
            let mut slot = self
                .0
                .try_borrow_mut()
                .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
            slot.replace(stored)
        };
        drop(replaced);
        Ok(())
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
    fn call(
        &self,
        completion: CompletionRecord,
        enter: &mut dyn FnMut(),
        context: &mut Context,
    ) -> Result<CoroutineState, RootingError>;
}

#[derive(Trace)]
struct Coroutine<C: NativeCapture, S: NativeCapture> {
    #[heap_trace(skip)]
    callback:
        fn(CompletionRecord, &C, Option<S>, &CoroutineSlot<S>, &mut Context) -> CoroutineState,
    captures: C::Stored,
    state: CoroutineSlot<S>,
}

impl<C: NativeCapture, S: NativeCapture> TraceableCoroutine for Coroutine<C, S> {
    fn call(
        &self,
        completion: CompletionRecord,
        enter: &mut dyn FnMut(),
        context: &mut Context,
    ) -> Result<CoroutineState, RootingError> {
        let captures = C::restore(&self.captures)?;
        let state = self.state.take()?;
        enter();
        Ok((self.callback)(
            completion,
            &captures,
            state,
            &self.state,
            context,
        ))
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

    fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
        Ok(Self(
            stored
                .0
                .upgrade()
                .map_err(RootingError::Access)?
                .ok_or(RootingError::Expired)?,
        ))
    }
}

impl NativeCoroutine {
    /// Creates a coroutine with explicit immutable captures and mutable state.
    pub(crate) fn from_fn_with_state<C: NativeCapture, S: NativeCapture>(
        callback: fn(
            CompletionRecord,
            &C,
            Option<S>,
            &CoroutineSlot<S>,
            &mut Context,
        ) -> CoroutineState,
        captures: C,
        state: S,
        context: &Context,
    ) -> Result<Self, RootingError> {
        let record = CoroutineRecord {
            coroutine: Box::new(Coroutine::<C, S> {
                callback,
                captures: captures.store(),
                state: CoroutineSlot(RefCell::new(Some(state.store()))),
            }),
        };
        Ok(Self(context.runtime().heap.allocate(record).map_err(
            |rejected| RootingError::Access(rejected.reason),
        )?))
    }

    /// Prepares rooted captures and state before entering the coroutine body.
    ///
    /// Preparation refusal leaves the stored state present and does not call
    /// `enter`. Every successful result comes from an entered body.
    pub(crate) fn call<F: FnOnce()>(
        &self,
        completion: CompletionRecord,
        enter: F,
        context: &mut Context,
    ) -> Result<CoroutineState, RootingError> {
        let mut enter = Some(enter);
        self.0.coroutine.call(
            completion,
            &mut || {
                if let Some(enter) = enter.take() {
                    enter();
                }
            },
            context,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use super::*;

    #[derive(Default)]
    struct Observations {
        restores: Cell<usize>,
        live_restorations: Cell<usize>,
        bodies: Cell<usize>,
    }

    /// A capture whose live snapshot requires reading a real stored payload.
    struct Snapshot {
        source: Rooted<RefCell<u32>>,
        value: u32,
        observations: Rc<Observations>,
        restored: bool,
    }

    #[derive(Trace)]
    struct StoredSnapshot {
        source: RecordRef<RefCell<u32>>,
        #[heap_trace(skip)]
        observations: Rc<Observations>,
    }

    impl NativeCapture for Snapshot {
        type Stored = StoredSnapshot;

        fn store(&self) -> Self::Stored {
            StoredSnapshot {
                source: self.source.record_ref(),
                observations: self.observations.clone(),
            }
        }

        fn restore(stored: &Self::Stored) -> Result<Self, RootingError> {
            let observations = stored.observations.clone();
            observations.restores.set(observations.restores.get() + 1);
            let source = Rooted::restore(&stored.source)?;
            let value = *source
                .try_borrow()
                .map_err(|_| RootingError::Access(AccessError::PayloadBorrowed))?;
            observations
                .live_restorations
                .set(observations.live_restorations.get() + 1);
            Ok(Self {
                source,
                value,
                observations,
                restored: true,
            })
        }
    }

    impl Drop for Snapshot {
        fn drop(&mut self) {
            if self.restored {
                self.observations
                    .live_restorations
                    .set(self.observations.live_restorations.get() - 1);
            }
        }
    }

    fn snapshot(context: &Context, value: u32) -> Snapshot {
        Snapshot {
            source: context
                .runtime()
                .heap
                .allocate(RefCell::new(value))
                .unwrap(),
            value,
            observations: Rc::default(),
            restored: false,
        }
    }

    fn refusal_preserves_state_and_retry_enters_once(block_capture: bool) {
        let mut context = Context::default();
        let capture = snapshot(&context, 7);
        let state = snapshot(&context, 11);
        let coroutine = Coroutine::<Snapshot, Snapshot> {
            callback: |_, capture: &Snapshot, prepared, slot, _| {
                capture
                    .observations
                    .bodies
                    .set(capture.observations.bodies.get() + 1);
                assert_eq!(capture.value, 7);
                assert_eq!(prepared.unwrap().value, 11);
                // The entry hook and body run after the state borrow ends.
                assert!(slot.0.try_borrow_mut().is_ok());
                CoroutineState::Break(Ok(()))
            },
            captures: capture.store(),
            state: CoroutineSlot(RefCell::new(Some(state.store()))),
        };
        let held_source = if block_capture {
            &capture.source
        } else {
            &state.source
        };
        let held_payload = held_source.borrow_mut();
        let entries = Cell::new(0);
        let mut enter = || {
            assert!(coroutine.state.0.try_borrow_mut().is_ok());
            entries.set(entries.get() + 1);
        };

        assert_eq!(
            coroutine.call(
                CompletionRecord::Normal(JsValue::undefined()),
                &mut enter,
                &mut context,
            ),
            Err(RootingError::Access(AccessError::PayloadBorrowed))
        );
        assert_eq!(entries.get(), 0);
        assert_eq!(capture.observations.bodies.get(), 0);
        assert_eq!(capture.observations.restores.get(), 1);
        assert_eq!(
            state.observations.restores.get(),
            usize::from(!block_capture)
        );
        assert_eq!(capture.observations.live_restorations.get(), 0);
        assert_eq!(state.observations.live_restorations.get(), 0);
        assert!(coroutine.state.0.borrow().is_some());

        drop(held_payload);
        assert_eq!(
            coroutine
                .call(
                    CompletionRecord::Normal(JsValue::undefined()),
                    &mut enter,
                    &mut context,
                )
                .unwrap(),
            CoroutineState::Break(Ok(()))
        );
        assert_eq!(entries.get(), 1);
        assert_eq!(capture.observations.bodies.get(), 1);
        assert_eq!(capture.observations.restores.get(), 2);
        assert_eq!(
            state.observations.restores.get(),
            1 + usize::from(!block_capture)
        );
        assert_eq!(capture.observations.live_restorations.get(), 0);
        assert_eq!(state.observations.live_restorations.get(), 0);
        assert!(coroutine.state.0.borrow().is_none());
    }

    #[test]
    fn borrowed_capture_payload_refuses_before_state_or_entry_and_retry_succeeds() {
        refusal_preserves_state_and_retry_enters_once(true);
    }

    #[test]
    fn borrowed_state_payload_preserves_stored_state_and_releases_capture_prefix() {
        refusal_preserves_state_and_retry_enters_once(false);
    }
}
