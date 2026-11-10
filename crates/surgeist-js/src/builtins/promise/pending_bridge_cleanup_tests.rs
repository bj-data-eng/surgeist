use super::*;
use crate::{
    ContextAccess,
    context::HostHooks,
    error::EngineError,
    execution::ExecutionOptions,
    gc::heap::Trace,
    job::{JobCallback, JobRetirement, NativeJob},
    native_function::NativeCapture,
    object::builtins::{JsFunction, JsPromise},
};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    pin::Pin,
    rc::Rc,
    sync::{Arc, Condvar, Mutex},
    task::{Context as TaskContext, Poll, Wake, Waker},
    time::{Duration, Instant},
};

#[derive(Debug)]
struct BridgeCaptureUnwind(usize);
struct BridgeDropState {
    armed: Cell<bool>,
    throw_once: [Cell<bool>; 2],
    disposals: [Cell<usize>; 2],
    first: Cell<Option<usize>>,
}
struct BridgeLiveCapture {
    shared: Rc<BridgeDropState>,
    tag: usize,
}
#[derive(Trace)]
struct BridgeStoredCapture {
    #[heap_trace(skip)]
    shared: Rc<BridgeDropState>,
    #[heap_trace(skip)]
    tag: usize,
}
impl NativeCapture for BridgeLiveCapture {
    type Stored = BridgeStoredCapture;
    fn store(&self) -> Self::Stored {
        BridgeStoredCapture {
            shared: self.shared.clone(),
            tag: self.tag,
        }
    }
    fn restore(stored: &Self::Stored) -> Result<Self, crate::error::RootingError> {
        Ok(Self {
            shared: stored.shared.clone(),
            tag: stored.tag,
        })
    }
}
impl Drop for BridgeLiveCapture {
    fn drop(&mut self) {
        if self.shared.armed.get() {
            let count = &self.shared.disposals[self.tag];
            count.set(count.get() + 1);
            if self.shared.first.get().is_none() {
                self.shared.first.set(Some(self.tag));
            }
            if self.shared.throw_once[self.tag].replace(false) {
                panic_any(BridgeCaptureUnwind(self.tag));
            }
        }
    }
}
struct BridgeHooks {
    shared: Rc<BridgeDropState>,
    handlers: RefCell<Option<[JsFunction; 2]>>,
}
impl HostHooks for BridgeHooks {
    fn make_job_callback(&self, callback: JsFunction, _: ContextAccess<'_>) -> JobCallback {
        let tag = self.handlers.borrow().as_ref().and_then(|handlers| {
            handlers
                .iter()
                .position(|handler| handler.identity() == callback.identity())
        });
        match tag {
            Some(tag) => JobCallback::new(
                callback,
                BridgeLiveCapture {
                    shared: self.shared.clone(),
                    tag,
                },
            ),
            None => JobCallback::new(callback, ()),
        }
    }
}
#[derive(Default)]
struct BridgeWake {
    count: Mutex<usize>,
    changed: Condvar,
}
impl BridgeWake {
    fn snapshot(&self) -> usize {
        *self.count.lock().unwrap()
    }
    fn await_after(&self, before: usize) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut count = self.count.lock().unwrap();
        while *count <= before {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "native bridge wait received no new notification"
            );
            count = self.changed.wait_timeout(count, remaining).unwrap().0;
        }
    }
}
impl Wake for BridgeWake {
    fn wake(self: Arc<Self>) {
        *self.count.lock().unwrap() += 1;
        self.changed.notify_all();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        *self.count.lock().unwrap() += 1;
        self.changed.notify_all();
    }
}
fn bridge_poll<F: Future + Unpin>(future: &mut F, waker: &Waker) -> Poll<F::Output> {
    Pin::new(future).poll(&mut TaskContext::from_waker(waker))
}

#[test]
fn pending_bridge_cleanup_disposes_both_callback_captures_before_resuming_its_first_unwind() {
    let shared = Rc::new(BridgeDropState {
        armed: Cell::new(false),
        throw_once: [Cell::new(true), Cell::new(true)],
        disposals: [Cell::new(0), Cell::new(0)],
        first: Cell::new(None),
    });
    let hooks = Rc::new(BridgeHooks {
        shared: shared.clone(),
        handlers: RefCell::new(None),
    });
    let builder = Context::builder().host_hooks(hooks.clone());
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(100_000);
    let mut context = builder.build().unwrap();
    let capabilities = context
        .with_execution(ExecutionOptions::new(4096), |access| {
            let context = access.into_owner();
            let constructor = context
                .intrinsics()
                .constructors()
                .promise()
                .constructor()
                .unwrap();
            Ok((
                PromiseCapability::new(&constructor, context)?,
                PromiseCapability::new(&constructor, context)?,
            ))
        })
        .unwrap()
        .result
        .unwrap();
    let (source, target) = capabilities;
    let source_identity = source.promise().identity();
    let target_identity = target.promise().identity();
    assert_ne!(source_identity, target_identity);
    let source_job = NativeJob::new(|_| Ok(JsValue::undefined()));
    let previous_target_job = NativeJob::new(|_| Ok(JsValue::undefined()));
    context
        .with_execution(ExecutionOptions::new(4096), |access| {
            let context = access.into_owner();
            source.associate_primary_job(&source_job.observer(), context)?;
            target.associate_primary_job(&previous_target_job.observer(), context)
        })
        .unwrap()
        .result
        .unwrap();
    let source_promise = JsPromise::from_object(source.promise().clone()).unwrap();
    let target_promise = JsPromise::from_object(target.promise().clone()).unwrap();
    let mut source_future = source_promise
        .clone()
        .into_js_future(context.access())
        .unwrap();
    let mut target_future = target_promise
        .clone()
        .into_js_future(context.access())
        .unwrap();
    let source_wake = Arc::new(BridgeWake::default());
    let target_wake = Arc::new(BridgeWake::default());
    let source_waker = Waker::from(source_wake.clone());
    let target_waker = Waker::from(target_wake.clone());
    assert!(bridge_poll(&mut source_future, &source_waker).is_pending());
    assert!(bridge_poll(&mut target_future, &target_waker).is_pending());
    hooks
        .handlers
        .replace(Some([target.resolve().clone(), target.reject().clone()]));
    let prepared = context
        .with_execution(ExecutionOptions::new(4096), |access| {
            Promise::prepare_pending_then(
                &source.native_promise()?,
                target.resolve().clone(),
                target.reject().clone(),
                &target,
                access.into_owner(),
            )
        })
        .unwrap()
        .result
        .unwrap();

    shared.armed.set(true);
    let result = catch_unwind(AssertUnwindSafe(|| prepared.notify()));
    shared.armed.set(false);
    let unwind =
        result.expect_err("each original bridge callback capture has an armed Drop unwind");
    assert_eq!(shared.disposals[0].get(), 1);
    assert_eq!(shared.disposals[1].get(), 1);
    assert_eq!(
        unwind.downcast_ref::<BridgeCaptureUnwind>().unwrap().0,
        shared.first.get().unwrap()
    );
    assert_eq!(source.promise().identity(), source_identity);
    assert_eq!(target.promise().identity(), target_identity);
    assert_eq!(source_promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(target_promise.state().unwrap(), PromiseState::Pending);

    // The real target handoff has replaced its old primary job even though
    // disposal unwound. That old job cannot retire the committed source bridge.
    drop(previous_target_job);
    assert!(bridge_poll(&mut source_future, &source_waker).is_pending());
    assert!(bridge_poll(&mut target_future, &target_waker).is_pending());
    let source_before = source_wake.snapshot();
    let target_before = target_wake.snapshot();
    drop(source_job);
    source_wake.await_after(source_before);
    target_wake.await_after(target_before);
    for result in [
        bridge_poll(&mut source_future, &source_waker),
        bridge_poll(&mut target_future, &target_waker),
    ] {
        let Poll::Ready(Err(error)) = result else {
            panic!("the actual source job loss must reach both native bridge waits")
        };
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::JobRetired(JobRetirement::Dropped))
        );
    }
    assert_eq!(source.promise().identity(), source_identity);
    assert_eq!(target.promise().identity(), target_identity);
    assert_eq!(source_promise.state().unwrap(), PromiseState::Pending);
    assert_eq!(target_promise.state().unwrap(), PromiseState::Pending);
}
