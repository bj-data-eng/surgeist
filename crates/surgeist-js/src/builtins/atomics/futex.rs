//! Waiter lists for Atomics, keyed by byte offset in one shared backing.

#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, VecDeque, btree_map::Entry},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    rc::Rc,
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError, atomic::Ordering},
};

use surgeist_js_string::JsString;

use crate::{
    Context, JsNativeError, JsResult, JsValue,
    builtins::{
        array_buffer::{SharedArrayBuffer, utils::SliceRef},
        promise::ResolvingFunctions,
        typed_array::Element,
    },
    job::{CancellationToken, NativeAsyncJob, TimeoutJob},
    js_string,
    sys::time::{Duration, Instant},
};

/// The result of waiting on a shared element.
#[derive(Debug, Clone, Copy)]
pub(super) enum AtomicsWaitResult {
    NotEqual,
    TimedOut,
    Ok,
}

impl AtomicsWaitResult {
    pub(super) fn to_js_string(self) -> JsString {
        match self {
            Self::NotEqual => js_string!("not-equal"),
            Self::TimedOut => js_string!("timed-out"),
            Self::Ok => js_string!("ok"),
        }
    }
}

type WaitersByOffset = BTreeMap<usize, VecDeque<Waiter>>;

#[derive(Debug)]
struct Waiter {
    identity: Arc<()>,
    signal: Signal,
}

#[derive(Debug)]
enum Signal {
    Blocking(Arc<Condvar>),
    Async(oneshot::Sender<AtomicsWaitResult>),
}

impl Signal {
    fn send(self, result: AtomicsWaitResult) {
        match self {
            Self::Blocking(ready) => ready.notify_one(),
            Self::Async(sender) => drop(sender.send(result)),
        }
    }
}

/// Each shared backing owns its own FIFO waiter lists.
#[derive(Debug, Default)]
pub(in crate::builtins) struct WaitQueues(Mutex<WaitersByOffset>);

impl WaitQueues {
    fn lock(&self) -> JsResult<MutexGuard<'_, WaitersByOffset>> {
        self.0.lock().map_err(|_| synchronization_error().into())
    }
}

fn synchronization_error() -> JsNativeError {
    JsNativeError::typ().with_message("failed to synchronize with the agent cluster")
}

/// Removing a registration wins the race with any future notify or timeout.
/// The queue stores only its token and signal, so it cannot retain the backing.
struct Registration {
    backing: SharedArrayBuffer,
    offset: usize,
    identity: Arc<()>,
}

impl Registration {
    fn insert(
        lists: &mut WaitersByOffset,
        backing: &SharedArrayBuffer,
        offset: usize,
        signal: Signal,
    ) -> Self {
        let identity = Arc::new(());
        lists.entry(offset).or_default().push_back(Waiter {
            identity: identity.clone(),
            signal,
        });
        Self {
            backing: backing.clone(),
            offset,
            identity,
        }
    }

    fn remove_from(&self, lists: &mut WaitersByOffset) -> Option<Signal> {
        let Entry::Occupied(mut list) = lists.entry(self.offset) else {
            return None;
        };
        let index = list
            .get()
            .iter()
            .position(|waiter| Arc::ptr_eq(&waiter.identity, &self.identity))?;
        let waiter = list.get_mut().remove(index)?;
        if list.get().is_empty() {
            list.remove();
        }
        Some(waiter.signal)
    }

    fn remove(&self) -> Option<Signal> {
        // Cancellation must release the entry even if a worker unwound.
        let mut lists = self
            .backing
            .wait_queues()
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        self.remove_from(&mut lists)
    }

    fn suspend(
        &self,
        ready: &Condvar,
        deadline: Option<(Instant, Duration)>,
    ) -> JsResult<AtomicsWaitResult> {
        let mut lists = self.backing.wait_queues().lock()?;
        loop {
            let pending = lists.get(&self.offset).is_some_and(|list| {
                list.iter()
                    .any(|waiter| Arc::ptr_eq(&waiter.identity, &self.identity))
            });
            if !pending {
                return Ok(AtomicsWaitResult::Ok);
            }

            if let Some((start, timeout)) = deadline {
                let remaining = timeout.saturating_sub(start.elapsed());
                if remaining.is_zero() {
                    self.remove_from(&mut lists);
                    return Ok(AtomicsWaitResult::TimedOut);
                }
                lists = ready
                    .wait_timeout(lists, remaining)
                    .map_err(|_| synchronization_error())?
                    .0;
            } else {
                lists = ready.wait(lists).map_err(|_| synchronization_error())?;
            }
        }
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        drop(self.remove());
    }
}

struct AsyncRegistration {
    registration: Rc<Registration>,
    timeout: Option<CancellationToken>,
}

impl Drop for AsyncRegistration {
    fn drop(&mut self) {
        if let Some(token) = self.timeout.take() {
            drop(self.registration.remove());
            token.revoke();
        }
    }
}

/// Waits for a notification or timeout at the selected shared element.
///
/// The offset and element width are checked against the supplied length witness.
pub(super) fn wait<E: Element + PartialEq>(
    buffer: &SharedArrayBuffer,
    buf_len: usize,
    offset: usize,
    check: E,
    timeout: Option<Duration>,
) -> JsResult<AtomicsWaitResult> {
    let mut lists = buffer.wait_queues().lock()?;
    let bytes = buffer.bytes_with_len(buf_len).subslice(offset..);
    let value = E::read(SliceRef::AtomicSlice(bytes), Ordering::SeqCst);
    if check != value {
        return Ok(AtomicsWaitResult::NotEqual);
    }

    let deadline = timeout.map(|timeout| (Instant::now(), timeout));
    let ready = Arc::new(Condvar::new());
    let registration =
        Registration::insert(&mut lists, buffer, offset, Signal::Blocking(ready.clone()));
    drop(lists);
    registration.suspend(&ready, deadline)
}

/// Registers a wait whose result is resolved on the waiting agent's executor.
///
/// The offset and element width are checked against the supplied length witness.
pub(super) fn wait_async<E: Element + PartialEq>(
    buffer: &SharedArrayBuffer,
    buf_len: usize,
    offset: usize,
    check: E,
    timeout: Option<Duration>,
    functions: ResolvingFunctions,
    context: &mut Context,
) -> JsResult<AtomicsWaitResult> {
    let mut lists = buffer.wait_queues().lock()?;
    let bytes = buffer.bytes_with_len(buf_len).subslice(offset..);
    let value = E::read(SliceRef::AtomicSlice(bytes), Ordering::SeqCst);
    if check != value {
        return Ok(AtomicsWaitResult::NotEqual);
    }
    if timeout.is_some_and(|timeout| timeout.is_zero()) {
        return Ok(AtomicsWaitResult::TimedOut);
    }

    let (sender, receiver) = oneshot::async_channel();
    let registration = Rc::new(Registration::insert(
        &mut lists,
        buffer,
        offset,
        Signal::Async(sender),
    ));
    drop(lists);

    let mut pending = AsyncRegistration {
        registration,
        timeout: None,
    };
    let realm = context.realm().clone();
    if let Some(timeout) = timeout {
        let registration = Rc::downgrade(&pending.registration);
        let job = TimeoutJob::with_realm(
            move |_| {
                if let Some(signal) = registration.upgrade().and_then(|entry| entry.remove()) {
                    signal.send(AtomicsWaitResult::TimedOut);
                }
                Ok(JsValue::undefined())
            },
            realm.clone(),
            timeout,
        );
        pending.timeout = Some(job.cancellation_token().clone());
        context.enqueue_job(job.into());
    }

    context.enqueue_job(
        NativeAsyncJob::with_realm(
            async move |context| {
                // The queued job owns this before its first poll. Discarding the
                // job removes the registration even if the Promise is still live.
                let mut pending = pending;
                if let Ok(result) = receiver.await {
                    functions.resolve.call(
                        &JsValue::undefined(),
                        &[result.to_js_string().into()],
                        context.borrow_mut().access(),
                    )?;
                }

                if let Some(token) = pending.timeout.take() {
                    token.cancel(&mut context.borrow_mut());
                }

                Ok(JsValue::undefined())
            },
            realm,
        )
        .into(),
    );

    Ok(AtomicsWaitResult::Ok)
}

/// Removes and notifies at most `count` waiters in FIFO order.
pub(super) fn notify(buffer: &SharedArrayBuffer, offset: usize, count: u64) -> JsResult<u64> {
    let waiters = {
        let mut lists = buffer.wait_queues().lock()?;
        let Entry::Occupied(mut list) = lists.entry(offset) else {
            return Ok(0);
        };
        let count = usize::try_from(count)
            .unwrap_or(usize::MAX)
            .min(list.get().len());
        let waiters: Vec<_> = list.get_mut().drain(..count).collect();
        if list.get().is_empty() {
            list.remove();
        }
        waiters
    };

    // Wake blocking agents before invoking any host waker. All selected
    // registrations have already been removed, so one host panic must not
    // leave the remaining agents without their results.
    let count = waiters.len() as u64;
    for waiter in &waiters {
        if let Signal::Blocking(ready) = &waiter.signal {
            ready.notify_one();
        }
    }
    let mut panics = Vec::new();
    for waiter in waiters {
        if let Signal::Async(sender) = waiter.signal
            && let Err(panic) = catch_unwind(AssertUnwindSafe(|| {
                drop(sender.send(AtomicsWaitResult::Ok));
            }))
        {
            panics.push(panic);
        }
    }
    let mut panics = panics.into_iter();
    let first = panics.next();
    for secondary in panics {
        // A host can panic with an owned payload whose destructor also panics.
        if let Err(cleanup_panic) = catch_unwind(AssertUnwindSafe(|| drop(secondary))) {
            std::mem::forget(cleanup_panic);
        }
    }
    if let Some(first) = first {
        resume_unwind(first);
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::{
        future::Future,
        panic::{AssertUnwindSafe, catch_unwind, panic_any},
        pin::Pin,
        sync::Arc,
        task::{Context as TaskContext, Wake, Waker},
    };

    use super::{AtomicsWaitResult, Context, Registration, Signal, notify};
    use crate::object::JsSharedArrayBuffer;

    #[test]
    fn notify_selects_waiters_in_registration_order() {
        let mut context = Context::default();
        let backing = JsSharedArrayBuffer::new(4, context.access())
            .unwrap()
            .inner();
        let (first, first_result) = oneshot::channel();
        let (second, second_result) = oneshot::channel();
        let (_first, _second) = {
            let mut lists = backing.wait_queues().lock().unwrap();
            (
                Registration::insert(&mut lists, &backing, 0, Signal::Async(first)),
                Registration::insert(&mut lists, &backing, 0, Signal::Async(second)),
            )
        };

        assert_eq!(notify(&backing.clone(), 0, 1).unwrap(), 1);
        assert!(matches!(first_result.try_recv(), Ok(AtomicsWaitResult::Ok)));
        assert!(matches!(
            second_result.try_recv(),
            Err(oneshot::TryRecvError::Empty)
        ));
        assert_eq!(notify(&backing, 0, 1).unwrap(), 1);
        assert!(matches!(
            second_result.try_recv(),
            Ok(AtomicsWaitResult::Ok)
        ));
        assert_eq!(notify(&backing, 0, 1).unwrap(), 0);
    }

    #[test]
    fn notify_delivers_remaining_results_when_a_host_waker_panics() {
        struct PanickingWake;
        struct SecondaryWake;
        struct PanickingPayload;

        impl Drop for PanickingPayload {
            fn drop(&mut self) {
                panic!("secondary payload destructor failed");
            }
        }

        impl Wake for PanickingWake {
            fn wake(self: Arc<Self>) {
                panic!("host waker failed");
            }
        }

        impl Wake for SecondaryWake {
            fn wake(self: Arc<Self>) {
                panic_any(PanickingPayload);
            }
        }

        let mut context = Context::default();
        let backing = JsSharedArrayBuffer::new(4, context.access())
            .unwrap()
            .inner();
        let (first, mut first_result) = oneshot::async_channel();
        let first_waker = Waker::from(Arc::new(PanickingWake));
        assert!(
            Pin::new(&mut first_result)
                .poll(&mut TaskContext::from_waker(&first_waker))
                .is_pending()
        );
        let (secondary, mut secondary_result) = oneshot::async_channel();
        let secondary_waker = Waker::from(Arc::new(SecondaryWake));
        assert!(
            Pin::new(&mut secondary_result)
                .poll(&mut TaskContext::from_waker(&secondary_waker))
                .is_pending()
        );
        let (remaining, remaining_result) = oneshot::channel();
        let (_first, _secondary, _remaining) = {
            let mut lists = backing.wait_queues().lock().unwrap();
            (
                Registration::insert(&mut lists, &backing, 0, Signal::Async(first)),
                Registration::insert(&mut lists, &backing, 0, Signal::Async(secondary)),
                Registration::insert(&mut lists, &backing, 0, Signal::Async(remaining)),
            )
        };

        let result = catch_unwind(AssertUnwindSafe(|| notify(&backing, 0, 3)));
        let panic = result.unwrap_err();
        assert_eq!(panic.downcast_ref::<&str>(), Some(&"host waker failed"));
        assert!(matches!(
            remaining_result.try_recv(),
            Ok(AtomicsWaitResult::Ok)
        ));
        assert_eq!(notify(&backing, 0, 3).unwrap(), 0);
    }
}
