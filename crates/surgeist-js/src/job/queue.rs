//! Resident queue custody; a temporary driver owns claims, not pending tasks.
use super::{
    ContextId, FinalizationRegistryCleanupJob, GenericJob, Job, JobCall, JobExecutor,
    JobInvocation, JobObserver, JobRetirement, JobStep, NativeJob, asynchronous::AsyncIdentity,
};
use crate::{
    Context, ContextAccess, HostContext, JsResult,
    context::time::JsInstant,
    error::{EngineError, EngineResourceError},
};
use futures_lite::future;
use portable_atomic::AtomicBool;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    rc::{Rc, Weak},
    sync::Arc,
    sync::atomic::Ordering,
    task::Poll,
};
use surgeist_js_gc::heap::ObjectId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Promise,
    Async,
    Generic,
    Clock(JsInstant),
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum Status {
    Available,
    Claimed,
    Retired,
}
struct Slot {
    index: usize,
    status: Cell<Status>,
    payload: RefCell<Option<Job>>,
    observer: Option<JobObserver>,
    native: Option<AsyncIdentity>,
}
impl Slot {
    fn retire(&self, reason: JobRetirement) {
        self.status.set(Status::Retired);
        if let Some(observer) = &self.observer {
            observer.retire(reason);
        }
        if let Some(native) = &self.native {
            native.retire(reason);
        }
        let payload = self.payload.borrow_mut().take();
        drop(payload);
    }
    fn owner(&self) -> Option<ContextId> {
        self.native.as_ref().and_then(AsyncIdentity::owner)
    }
}
#[derive(Default)]
struct Bucket {
    head: Option<usize>,
    tail: Option<usize>,
    reservations: usize,
}
struct Node {
    slot: Rc<Slot>,
    kind: Kind,
    previous: Option<usize>,
    next: Option<usize>,
}
enum Entry {
    Free(Option<usize>),
    Node(Node),
}
#[derive(Default)]
struct Store {
    entries: Vec<Entry>,
    free: Option<usize>,
    ordinary: [Bucket; 3],
    clocks: BTreeMap<JsInstant, Bucket>,
    len: usize,
}
impl Store {
    fn bucket(&self, kind: Kind) -> &Bucket {
        match kind {
            Kind::Promise => &self.ordinary[0],
            Kind::Async => &self.ordinary[1],
            Kind::Generic => &self.ordinary[2],
            Kind::Clock(time) => self.clocks.get(&time).expect("clock bucket exists"),
        }
    }
    fn bucket_mut(&mut self, kind: Kind) -> &mut Bucket {
        match kind {
            Kind::Promise => &mut self.ordinary[0],
            Kind::Async => &mut self.ordinary[1],
            Kind::Generic => &mut self.ordinary[2],
            Kind::Clock(time) => self.clocks.entry(time).or_default(),
        }
    }
    fn node(&self, index: usize) -> &Node {
        let Entry::Node(node) = &self.entries[index] else {
            unreachable!("occupied queue node")
        };
        node
    }
    fn node_mut(&mut self, index: usize) -> &mut Node {
        let Entry::Node(node) = &mut self.entries[index] else {
            unreachable!("occupied queue node")
        };
        node
    }
    fn insert(&mut self, job: Job, kind: Kind) {
        let index = if let Some(index) = self.free {
            let Entry::Free(next) = self.entries[index] else {
                unreachable!("free queue node")
            };
            self.free = next;
            index
        } else {
            self.entries.reserve(1);
            let index = self.entries.len();
            self.entries.push(Entry::Free(None));
            index
        };
        let (observer, native) = match &job {
            Job::PromiseJob(job) => (Some(job.observer()), None),
            Job::GenericJob(job) => (Some(job.observer()), None),
            Job::TimeoutJob(job) => (Some(job.observer()), None),
            Job::AsyncJob(job) => (Some(job.observer()), Some(job.identity())),
            _ => (None, None),
        };
        let slot = Rc::new(Slot {
            index,
            status: Cell::new(Status::Available),
            payload: RefCell::new(Some(job)),
            observer,
            native,
        });
        let previous = self.bucket_mut(kind).tail;
        self.entries[index] = Entry::Node(Node {
            slot,
            kind,
            previous,
            next: None,
        });
        if let Some(previous) = previous {
            self.node_mut(previous).next = Some(index);
        }
        let bucket = self.bucket_mut(kind);
        if bucket.head.is_none() {
            bucket.head = Some(index);
        }
        bucket.tail = Some(index);
        self.len += 1;
    }
    fn unlink(&mut self, index: usize) -> Rc<Slot> {
        let Entry::Node(node) = std::mem::replace(&mut self.entries[index], Entry::Free(self.free))
        else {
            unreachable!("remove occupied node")
        };
        self.free = Some(index);
        if let Some(previous) = node.previous {
            self.node_mut(previous).next = node.next;
        }
        if let Some(next) = node.next {
            self.node_mut(next).previous = node.previous;
        }
        let bucket = self.bucket_mut(node.kind);
        if bucket.head == Some(index) {
            bucket.head = node.next;
        }
        if bucket.tail == Some(index) {
            bucket.tail = node.previous;
        }
        let remove = bucket.head.is_none() && bucket.reservations == 0;
        if remove && let Kind::Clock(time) = node.kind {
            self.clocks.remove(&time);
        }
        self.len -= 1;
        node.slot
    }
    fn move_clock(&mut self, index: usize, time: JsInstant) {
        let (old_kind, previous, next) = {
            let node = self.node(index);
            (node.kind, node.previous, node.next)
        };
        if let Some(previous) = previous {
            self.node_mut(previous).next = next;
        }
        if let Some(next) = next {
            self.node_mut(next).previous = previous;
        }
        let old = self.bucket_mut(old_kind);
        if old.head == Some(index) {
            old.head = next;
        }
        if old.tail == Some(index) {
            old.tail = previous;
        }
        let remove = old.head.is_none() && old.reservations == 0;
        if remove && let Kind::Clock(old_time) = old_kind {
            self.clocks.remove(&old_time);
        }
        let kind = Kind::Clock(time);
        let previous = self.bucket(kind).tail;
        if let Some(previous) = previous {
            self.node_mut(previous).next = Some(index);
        }
        let node = self.node_mut(index);
        node.kind = kind;
        node.previous = previous;
        node.next = None;
        let bucket = self.bucket_mut(kind);
        if bucket.head.is_none() {
            bucket.head = Some(index);
        }
        bucket.tail = Some(index);
    }
    fn claim(&self, kinds: impl Iterator<Item = Kind>) -> JsResult<Batch> {
        let mut slots = Vec::new();
        slots
            .try_reserve(self.len)
            .map_err(|_| EngineResourceError::Allocation)?;
        for kind in kinds {
            let mut cursor = self.bucket(kind).head;
            while let Some(index) = cursor {
                let node = self.node(index);
                cursor = node.next;
                if node.slot.status.get() == Status::Available {
                    slots.push(node.slot.clone());
                }
            }
        }
        for slot in &slots {
            slot.status.set(Status::Claimed);
        }
        Ok(Batch { slots })
    }
}
struct Batch {
    slots: Vec<Rc<Slot>>,
}
impl Drop for Batch {
    fn drop(&mut self) {
        for slot in &self.slots {
            if slot.status.get() == Status::Claimed {
                slot.status.set(Status::Available);
            }
        }
    }
}
struct Payload {
    slot: Rc<Slot>,
    job: Option<Job>,
}
impl Payload {
    fn take(slot: &Rc<Slot>) -> Option<Self> {
        if slot.status.get() == Status::Retired {
            return None;
        }
        let job = slot.payload.borrow_mut().take()?;
        Some(Self {
            slot: slot.clone(),
            job: Some(job),
        })
    }
}
impl Drop for Payload {
    fn drop(&mut self) {
        if self.slot.status.get() != Status::Retired {
            *self.slot.payload.borrow_mut() = self.job.take();
        }
    }
}
struct ClockReservation {
    queues: Rc<RuntimeJobQueues>,
    time: JsInstant,
}
impl Drop for ClockReservation {
    fn drop(&mut self) {
        let mut store = self.queues.store.borrow_mut();
        if let Some(bucket) = store.clocks.get_mut(&self.time) {
            bucket.reservations -= 1;
            if bucket.head.is_none() && bucket.reservations == 0 {
                store.clocks.remove(&self.time);
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct RuntimeJobQueues {
    store: RefCell<Store>,
    finalization_registry_subscriptions: RefCell<VecDeque<FinalizationRegistryCleanupJob>>,
    active_drains: Cell<usize>,
}
impl RuntimeJobQueues {
    fn finish(&self, slot: &Rc<Slot>) {
        if slot.status.replace(Status::Retired) == Status::Retired {
            return;
        }
        let removed = {
            let mut store = self.store.borrow_mut();
            let same = matches!(&store.entries[slot.index], Entry::Node(node) if Rc::ptr_eq(&node.slot, slot));
            same.then(|| store.unlink(slot.index))
        };
        drop(removed);
    }
    pub(crate) fn retire_owner(&self, owner: &ContextId) {
        self.retire_matching(JobRetirement::OwnerDropped, |slot| {
            slot.owner().as_ref() == Some(owner)
        });
    }
    fn retire_matching(&self, reason: JobRetirement, matches: impl Fn(&Slot) -> bool) {
        let bound = {
            let store = self.store.borrow();
            for entry in &store.entries {
                if let Entry::Node(node) = entry
                    && matches(&node.slot)
                {
                    node.slot.status.set(Status::Retired);
                }
            }
            store.entries.len()
        };
        for index in 0..bound {
            let slot = {
                let mut store = self.store.borrow_mut();
                let should_remove = matches!(&store.entries[index], Entry::Node(node) if node.slot.status.get() == Status::Retired);
                should_remove.then(|| store.unlink(index))
            };
            if let Some(slot) = slot {
                slot.retire(reason);
            }
        }
    }
    fn prune_cancelled_clocks(&self) {
        self.retire_matching(JobRetirement::Cancelled, |slot| {
            match slot.payload.borrow().as_ref() {
                Some(Job::TimeoutJob(job)) => job.cancelled(),
                Some(Job::IntervalJob(job)) => job.cancelled(),
                _ => false,
            }
        });
    }
    fn clear(&self) {
        self.retire_matching(JobRetirement::Cancelled, |_| true);
    }
    fn take_ready_cleanup(&self) -> Option<NativeJob> {
        let mut subscriptions = self.finalization_registry_subscriptions.borrow_mut();
        for _ in 0..subscriptions.len() {
            let subscription = subscriptions
                .pop_front()
                .expect("one subscription per iteration");
            match subscription.try_take() {
                Poll::Pending => subscriptions.push_back(subscription),
                Poll::Ready(Some(job)) => {
                    subscriptions.push_back(subscription);
                    return Some(job);
                }
                Poll::Ready(None) => {}
            }
        }
        None
    }
    fn reserve_clock(self: &Rc<Self>, time: JsInstant) -> ClockReservation {
        self.store
            .borrow_mut()
            .clocks
            .entry(time)
            .or_default()
            .reservations += 1;
        ClockReservation {
            queues: self.clone(),
            time,
        }
    }
}
struct DrainGuard(Rc<RuntimeJobQueues>);
impl Drop for DrainGuard {
    fn drop(&mut self) {
        self.0.active_drains.set(self.0.active_drains.get() - 1);
    }
}

/// A FIFO executor retaining accepted work independently of temporary drains.
#[derive(Default)]
pub struct SimpleJobExecutor {
    queues: RefCell<Vec<(ObjectId, Weak<RuntimeJobQueues>)>>,
    stop: Arc<AtomicBool>,
}
impl std::fmt::Debug for SimpleJobExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleJobExecutor").finish_non_exhaustive()
    }
}
impl SimpleJobExecutor {
    /// Creates an empty executor.
    pub fn new() -> Self {
        Self::default()
    }
    /// Explicit destructive cancellation for the runtime whose drain observes it.
    pub fn get_cancellation_token(&self) -> Arc<AtomicBool> {
        self.stop.clone()
    }
    pub(crate) fn attach_context(&self, context: &mut Context) -> Rc<RuntimeJobQueues> {
        let queues = self.queues_for(context);
        context.retain_simple_job_queue(&queues);
        queues
    }
    fn queues_for(&self, context: &Context) -> Rc<RuntimeJobQueues> {
        let identity = context.realm().identity();
        let mut queues = self.queues.borrow_mut();
        if let Some(queue) = queues
            .iter()
            .find(|(key, _)| key.same_heap(identity))
            .and_then(|(_, queue)| queue.upgrade())
        {
            return queue;
        }
        queues.retain(|(_, queue)| queue.strong_count() != 0);
        let queue = Rc::new(RuntimeJobQueues::default());
        queues.push((identity, Rc::downgrade(&queue)));
        queue
    }
}
impl Drop for SimpleJobExecutor {
    fn drop(&mut self) {
        let queues = std::mem::take(self.queues.get_mut());
        for (_, queue) in queues {
            if let Some(queue) = queue.upgrade() {
                queue.clear();
                drop(queue.finalization_registry_subscriptions.take());
            }
        }
    }
}
impl Drop for RuntimeJobQueues {
    fn drop(&mut self) {
        self.clear();
    }
}

fn invoke_owned<J>(
    host: &HostContext<'_>,
    job: J,
    invoke: impl for<'call> FnOnce(J, ContextAccess<'call>) -> JobCall<J>,
) -> JobCall<J> {
    let mut pending = Some(job);
    match host.with(|access| {
        Ok(invoke(
            pending.take().expect("owned callback enters once"),
            access,
        ))
    }) {
        Ok(outcome) => outcome,
        Err(error) => JobCall::Deferred {
            job: pending
                .take()
                .expect("unavailable holder did not consume callback"),
            error,
        },
    }
}

impl JobExecutor for SimpleJobExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, access: ContextAccess<'_>) {
        let context = access.into_owner();
        let queues = self.attach_context(context);
        if let Job::FinalizationRegistryCleanupJob(subscription) = job {
            queues
                .finalization_registry_subscriptions
                .borrow_mut()
                .push_back(subscription);
            return;
        }
        let kind = match &job {
            Job::PromiseJob(_) => Kind::Promise,
            Job::AsyncJob(_) => Kind::Async,
            Job::GenericJob(_) => Kind::Generic,
            Job::TimeoutJob(job) => Kind::Clock(context.clock().now() + job.timeout()),
            Job::IntervalJob(job) => Kind::Clock(context.clock().now() + job.interval()),
            Job::FinalizationRegistryCleanupJob(_) => unreachable!(),
        };
        queues.store.borrow_mut().insert(job, kind);
    }
    fn run_jobs(self: Rc<Self>, access: ContextAccess<'_>) -> JsResult<()> {
        future::block_on(self.run_jobs_async(&access.into_host_access()))
    }
    async fn run_jobs_async(self: Rc<Self>, host: &HostContext<'_>) -> JsResult<()>
    where
        Self: Sized,
    {
        let queues = host.with(|access| {
            let context = access.into_owner();
            context.check_execution_stop()?;
            Ok(self.attach_context(context))
        })?;
        queues.active_drains.set(queues.active_drains.get() + 1);
        let _drain = DrainGuard(queues.clone());
        loop {
            if self.stop.load(Ordering::Relaxed) && self.stop.swap(false, Ordering::Relaxed) {
                queues.clear();
                return Ok(());
            }
            host.with(|access| access.into_owner().check_execution_stop())?;
            let async_batch = queues.store.borrow().claim(std::iter::once(Kind::Async))?;
            let mut did_work = !async_batch.slots.is_empty();
            // Prepare all captured requests before polling tasks or invoking other jobs.
            for slot in &async_batch.slots {
                let Some(mut payload) = Payload::take(slot) else {
                    continue;
                };
                if let Some(Job::AsyncJob(job)) = payload.job.as_mut() {
                    if !job.is_prepared() {
                        let step = future::poll_fn(|cx| {
                            job.register_driver(cx.waker());
                            Poll::Ready(job.run_step(cx, host))
                        })
                        .await;
                        match step {
                            JobStep::Prepared => {}
                            JobStep::Deferred(error) => return Err(error),
                            JobStep::Complete(result) => {
                                queues.finish(slot);
                                result?;
                            }
                            JobStep::Finished(status) => {
                                queues.finish(slot);
                                if let Err(fault) = status {
                                    return host.with(|access| {
                                        access.into_owner().check_execution_stop()?;
                                        Err(fault.into())
                                    });
                                }
                            }
                            JobStep::Pending => unreachable!("factory step cannot poll"),
                        }
                    } else if let Some(required) = job.owner() {
                        let actual = host.with(|access| Ok(access.id()))?;
                        if actual != required {
                            return Err(EngineError::JobContextRequired { required }.into());
                        }
                    }
                }
            }
            let clock = host.with(|access| Ok(access.into_owner().clock_handle()))?;
            let now = clock.now();
            queues.prune_cancelled_clocks();
            let clock_batch = {
                let store = queues.store.borrow();
                store.claim(
                    store
                        .clocks
                        .range(..now)
                        .map(|(time, _)| Kind::Clock(*time)),
                )?
            };
            did_work |= !clock_batch.slots.is_empty();
            for slot in &clock_batch.slots {
                let Some(mut payload) = Payload::take(slot) else {
                    continue;
                };
                let Some(job) = payload.job.take() else {
                    continue;
                };
                match job {
                    Job::TimeoutJob(job) => {
                        if job.cancelled() {
                            queues.finish(slot);
                            continue;
                        }
                        match invoke_owned(host, job, |job, access| job.call(access)) {
                            JobCall::Deferred { job, error } => {
                                payload.job = Some(Job::TimeoutJob(job));
                                return Err(error);
                            }
                            JobCall::Complete(result) => {
                                queues.finish(slot);
                                result?;
                            }
                        }
                    }
                    Job::IntervalJob(job) => {
                        if job.cancelled() {
                            queues.finish(slot);
                            continue;
                        }
                        let clock = host.with(|access| Ok(access.into_owner().clock_handle()))?;
                        let deadline = clock.now() + job.interval();
                        let _reservation = queues.reserve_clock(deadline);
                        match host
                            .with(|access| Ok(job.call(access)))
                            .unwrap_or_else(JobInvocation::Deferred)
                        {
                            JobInvocation::Deferred(error) => {
                                payload.job = Some(Job::IntervalJob(job));
                                return Err(error);
                            }
                            JobInvocation::Complete(result) => {
                                if let Err(error) = result {
                                    queues.finish(slot);
                                    return Err(error);
                                }
                                if slot.status.get() != Status::Retired {
                                    queues.store.borrow_mut().move_clock(slot.index, deadline);
                                    payload.job = Some(Job::IntervalJob(job));
                                }
                            }
                        }
                    }
                    _ => unreachable!("clock slot"),
                }
            }
            // Keep the former idle boundary before task polling and the
            // promise/generic batches. A completed final ordinary batch still
            // clears kept objects and yields before the next empty turn exits.
            if queues.store.borrow().len == 0 {
                if let Some(job) = queues.take_ready_cleanup() {
                    queues
                        .store
                        .borrow_mut()
                        .insert(Job::GenericJob(GenericJob(job)), Kind::Generic);
                    did_work = true;
                } else {
                    break;
                }
            }
            for slot in &async_batch.slots {
                let Some(mut payload) = Payload::take(slot) else {
                    continue;
                };
                if let Some(Job::AsyncJob(job)) = payload.job.as_mut() {
                    let step = future::poll_fn(|cx| {
                        job.register_driver(cx.waker());
                        if !job.is_ready() {
                            return Poll::Ready(None);
                        }
                        Poll::Ready(Some(job.run_step(cx, host)))
                    })
                    .await;
                    match step {
                        None | Some(JobStep::Pending) => {}
                        Some(JobStep::Prepared) => unreachable!("prepared before poll phase"),
                        Some(JobStep::Deferred(error)) => return Err(error),
                        Some(JobStep::Complete(result)) => {
                            queues.finish(slot);
                            result?;
                        }
                        Some(JobStep::Finished(status)) => {
                            queues.finish(slot);
                            if let Err(fault) = status {
                                return host.with(|access| {
                                    access.into_owner().check_execution_stop()?;
                                    Err(fault.into())
                                });
                            }
                        }
                    }
                }
            }
            for kind in [Kind::Promise, Kind::Generic] {
                let batch = queues.store.borrow().claim(std::iter::once(kind))?;
                did_work |= !batch.slots.is_empty();
                for slot in &batch.slots {
                    let Some(mut payload) = Payload::take(slot) else {
                        continue;
                    };
                    let Some(job) = payload.job.take() else {
                        continue;
                    };
                    match job {
                        Job::PromiseJob(job) => {
                            match invoke_owned(host, job, |job, access| job.call(access)) {
                                JobCall::Deferred { job, error } => {
                                    payload.job = Some(Job::PromiseJob(job));
                                    return Err(error);
                                }
                                JobCall::Complete(result) => {
                                    queues.finish(slot);
                                    result?;
                                }
                            }
                        }
                        Job::GenericJob(job) => {
                            match invoke_owned(host, job, |job, access| job.call(access)) {
                                JobCall::Deferred { job, error } => {
                                    payload.job = Some(Job::GenericJob(job));
                                    return Err(error);
                                }
                                JobCall::Complete(result) => {
                                    queues.finish(slot);
                                    result?;
                                }
                            }
                        }
                        _ => unreachable!("synchronous bucket"),
                    }
                }
            }
            if !did_work && queues.active_drains.get() > 1 {
                break;
            }
            host.with(|mut access| {
                access.clear_kept_objects();
                Ok(())
            })?;
            future::yield_now().await;
        }
        Ok(())
    }
}
