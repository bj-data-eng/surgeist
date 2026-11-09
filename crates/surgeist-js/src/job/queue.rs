//! Resident queue custody; a temporary driver owns claims, not pending tasks.
use super::{
    ContextId, FinalizationRegistryCleanupJob, GenericJob, Job, JobAcceptance, JobCall,
    JobExecutor, JobFault, JobFaultCause, JobFaultOwner, JobInvocation, JobObserver,
    JobPublicationCause, JobPublicationError, JobRetirement, JobStep, UnwindCleanup,
    asynchronous::AsyncIdentity,
};
use crate::{
    Context, ContextAccess, HostContext, JsResult,
    context::time::{JsDuration, JsInstant},
    error::{EngineError, EngineResourceError},
    execution::{ExecutionPhase, JobClaimWork},
};
use futures_lite::future;
use portable_atomic::AtomicBool;
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    mem::size_of,
    rc::{Rc, Weak},
    sync::{Arc, atomic::Ordering},
    task::Poll,
};
use surgeist_js_gc::heap::ObjectId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NodeId(usize);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Ordinary {
    Promise,
    Async,
    Generic,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Ordinary(Ordinary),
    Clock(JsInstant),
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum Status {
    Available,
    Claimed,
    Retired,
}

#[derive(Debug)]
enum QueueError {
    Resource(EngineResourceError),
    Fault(JobFault),
    Deadline { now: JsInstant, delay: JsDuration },
}
type QueueResult<T> = Result<T, QueueError>;
impl From<EngineResourceError> for QueueError {
    fn from(error: EngineResourceError) -> Self {
        Self::Resource(error)
    }
}
impl From<JobFault> for QueueError {
    fn from(error: JobFault) -> Self {
        Self::Fault(error)
    }
}
impl QueueError {
    fn into_engine(self) -> EngineError {
        match self {
            Self::Resource(error) => EngineError::Resource(error),
            Self::Fault(error) => crate::error::JobError::Fault(error).into(),
            Self::Deadline { now, delay } => EngineError::JobDeadlineOverflow { now, delay },
        }
    }
}
fn defect(cause: JobFaultCause) -> JobFault {
    JobFault::new(JobFaultOwner::Queue, cause)
}
fn reserve<T>(values: &mut Vec<T>, additional: usize) -> QueueResult<()> {
    values
        .len()
        .checked_add(additional)
        .and_then(|count| count.checked_mul(size_of::<T>()))
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(EngineResourceError::SizeOverflow)?;
    values
        .try_reserve(additional)
        .map_err(|_| EngineResourceError::Allocation)?;
    Ok(())
}
fn deadline(now: JsInstant, delay: JsDuration) -> QueueResult<JsInstant> {
    now.checked_add(delay)
        .ok_or(QueueError::Deadline { now, delay })
}

// Subscriptions have a separate persistent owner; placement is derived from
// this closed sum rather than supplied independently by a caller.
enum Resident {
    Promise(super::PromiseJob),
    Async(super::NativeAsyncJob),
    Generic(GenericJob),
    Timeout(super::TimeoutJob),
    Interval(super::IntervalJob),
}
impl Resident {
    fn from_job(job: Job) -> Result<Self, FinalizationRegistryCleanupJob> {
        match job {
            Job::PromiseJob(job) => Ok(Self::Promise(job)),
            Job::AsyncJob(job) => Ok(Self::Async(job)),
            Job::GenericJob(job) => Ok(Self::Generic(job)),
            Job::TimeoutJob(job) => Ok(Self::Timeout(job)),
            Job::IntervalJob(job) => Ok(Self::Interval(job)),
            Job::FinalizationRegistryCleanupJob(job) => Err(job),
        }
    }
    fn into_job(self) -> Job {
        match self {
            Self::Promise(job) => job.into(),
            Self::Async(job) => job.into(),
            Self::Generic(job) => job.into(),
            Self::Timeout(job) => job.into(),
            Self::Interval(job) => job.into(),
        }
    }
    fn accepts(&self, kind: Kind) -> bool {
        matches!(
            (self, kind),
            (Self::Promise(_), Kind::Ordinary(Ordinary::Promise))
                | (Self::Async(_), Kind::Ordinary(Ordinary::Async))
                | (Self::Generic(_), Kind::Ordinary(Ordinary::Generic))
                | (Self::Timeout(_) | Self::Interval(_), Kind::Clock(_))
        )
    }
    fn observation(&self) -> (Option<JobObserver>, Option<AsyncIdentity>) {
        match self {
            Self::Promise(job) => (Some(job.observer()), None),
            Self::Async(job) => (Some(job.observer()), Some(job.identity())),
            Self::Generic(job) => (Some(job.observer()), None),
            Self::Timeout(job) => (Some(job.observer()), None),
            Self::Interval(_) => (None, None),
        }
    }
    fn cancelled(&self) -> bool {
        match self {
            Self::Timeout(job) => job.cancelled(),
            Self::Interval(job) => job.cancelled(),
            _ => false,
        }
    }
}
struct PayloadRecord {
    command: Option<Resident>,
    next: Option<Box<PayloadRecord>>,
}
enum PayloadState {
    Resident(Box<PayloadRecord>),
    Extracted,
    Retired,
}
fn dispose_records(mut records: Option<Box<PayloadRecord>>, cleanup: &mut UnwindCleanup) {
    while let Some(mut record) = records {
        records = record.next.take();
        let command = record.command.take();
        cleanup.run(|| drop(command));
        cleanup.run(|| drop(record));
    }
}
struct Slot {
    index: NodeId,
    kind: Cell<Kind>,
    previous: Cell<Option<NodeId>>,
    next: Cell<Option<NodeId>>,
    status: Cell<Status>,
    payload: Cell<PayloadState>,
    observer: Option<JobObserver>,
    native: Option<AsyncIdentity>,
    queues: Weak<RuntimeJobQueues>,
}
impl Slot {
    fn owner(&self) -> Option<ContextId> {
        self.native.as_ref().and_then(AsyncIdentity::owner)
    }
    fn mark_fault(&self, fault: JobFault) {
        if let Some(observer) = &self.observer {
            observer.latch_fault(fault);
        }
        if let Some(native) = &self.native {
            native.mark_fault(fault);
        }
    }
    fn notify_fault(&self, fault: JobFault) {
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| {
            if let Some(observer) = &self.observer {
                observer.record_fault(fault);
            }
        });
        cleanup.run(|| {
            if let Some(native) = &self.native {
                native.fault(fault);
            }
        });
        cleanup.finish();
    }
    fn fault(&self, fault: JobFault) {
        if let Some(queues) = self.queues.upgrade() {
            queues.record_fault(fault);
        }
    }
    fn retire(&self, reason: JobRetirement) {
        self.status.set(Status::Retired);
        let records = match self.payload.replace(PayloadState::Retired) {
            PayloadState::Resident(record) => Some(record),
            PayloadState::Extracted | PayloadState::Retired => None,
        };
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| {
            if let Some(observer) = &self.observer {
                observer.retire(reason);
            }
        });
        cleanup.run(|| {
            if let Some(observer) = &self.observer
                && let Some(fault) = observer.fault()
            {
                observer.record_fault(fault);
            }
        });
        cleanup.run(|| {
            if let Some(native) = &self.native {
                native.retire(reason);
            }
        });
        dispose_records(records, &mut cleanup);
        cleanup.finish();
    }
    fn cancelled(&self) -> bool {
        let payload = self.payload.replace(PayloadState::Extracted);
        let cancelled = match &payload {
            PayloadState::Resident(record) => {
                record.command.as_ref().is_some_and(Resident::cancelled)
            }
            PayloadState::Extracted | PayloadState::Retired => false,
        };
        self.payload.set(payload);
        cancelled
    }
}
impl Drop for Slot {
    fn drop(&mut self) {
        let records = match std::mem::replace(self.payload.get_mut(), PayloadState::Retired) {
            PayloadState::Resident(record) => Some(record),
            _ => None,
        };
        let mut cleanup = UnwindCleanup::default();
        dispose_records(records, &mut cleanup);
        cleanup.finish();
    }
}
struct Payload {
    slot: Rc<Slot>,
    record: Option<Box<PayloadRecord>>,
}
impl Payload {
    fn take(slot: &Rc<Slot>) -> Result<Option<Self>, JobFault> {
        if slot.status.get() == Status::Retired {
            return Ok(None);
        }
        if slot.status.get() != Status::Claimed {
            return Err(defect(JobFaultCause::StateInconsistency));
        }
        match slot.payload.replace(PayloadState::Extracted) {
            PayloadState::Resident(record) if record.next.is_none() && record.command.is_some() => {
                Ok(Some(Self {
                    slot: slot.clone(),
                    record: Some(record),
                }))
            }
            other => {
                slot.payload.set(other);
                Err(defect(JobFaultCause::StateInconsistency))
            }
        }
    }
    fn command_mut(&mut self) -> Result<&mut Resident, JobFault> {
        self.record
            .as_mut()
            .and_then(|record| record.command.as_mut())
            .ok_or_else(|| defect(JobFaultCause::StateInconsistency))
    }
    fn command_slot(&mut self) -> Result<&mut Option<Resident>, JobFault> {
        self.record
            .as_mut()
            .map(|record| &mut record.command)
            .ok_or_else(|| defect(JobFaultCause::StateInconsistency))
    }
}
impl Drop for Payload {
    fn drop(&mut self) {
        let Some(mut record) = self.record.take() else {
            return;
        };
        if self.slot.status.get() == Status::Retired {
            let mut cleanup = UnwindCleanup::default();
            dispose_records(Some(record), &mut cleanup);
            cleanup.finish();
            return;
        }
        let previous = self.slot.payload.replace(PayloadState::Extracted);
        let collision = match previous {
            PayloadState::Extracted => false,
            PayloadState::Resident(previous) => {
                record.next = Some(previous);
                true
            }
            PayloadState::Retired => true,
        };
        self.slot.payload.set(PayloadState::Resident(record));
        if collision {
            self.slot.fault(defect(JobFaultCause::StateInconsistency));
        }
    }
}

#[derive(Default)]
struct Bucket {
    head: Option<NodeId>,
    tail: Option<NodeId>,
    reservations: usize,
}
#[derive(Default)]
struct OrdinaryBuckets {
    promise: Bucket,
    asynchronous: Bucket,
    generic: Bucket,
}
impl OrdinaryBuckets {
    fn get(&self, kind: Ordinary) -> &Bucket {
        match kind {
            Ordinary::Promise => &self.promise,
            Ordinary::Async => &self.asynchronous,
            Ordinary::Generic => &self.generic,
        }
    }
    fn get_mut(&mut self, kind: Ordinary) -> &mut Bucket {
        match kind {
            Ordinary::Promise => &mut self.promise,
            Ordinary::Async => &mut self.asynchronous,
            Ordinary::Generic => &mut self.generic,
        }
    }
}
struct ClockWitness {
    retired: Cell<bool>,
}
struct ClockBucket {
    deadline: JsInstant,
    bucket: Bucket,
    witness: Rc<ClockWitness>,
}
enum Entry {
    Free(Option<NodeId>),
    Node(Rc<Slot>),
}
#[derive(Default)]
struct Store {
    entries: Vec<Entry>,
    free: Option<NodeId>,
    ordinary: OrdinaryBuckets,
    clocks: Vec<ClockBucket>,
    len: usize,
}
impl Store {
    fn is_empty(&self) -> Result<bool, JobFault> {
        if self.len > self.entries.len() {
            return Err(defect(JobFaultCause::StateInconsistency));
        }
        // This is the existing idle boundary, outside selected claim work.
        let occupied = self.ordinary.promise.head.is_some()
            || self.ordinary.asynchronous.head.is_some()
            || self.ordinary.generic.head.is_some()
            || self.clocks.iter().any(|clock| clock.bucket.head.is_some());
        if occupied == (self.len == 0) {
            return Err(defect(JobFaultCause::StateInconsistency));
        }
        Ok(!occupied)
    }
    fn node(&self, id: NodeId) -> Result<&Rc<Slot>, JobFault> {
        match self.entries.get(id.0) {
            Some(Entry::Node(slot)) if slot.index == id => Ok(slot),
            Some(_) => Err(defect(JobFaultCause::InvalidSlotIdentity { index: id.0 })),
            None => Err(defect(JobFaultCause::InvalidIndex { index: id.0 })),
        }
    }
    fn clock_index(&self, time: JsInstant) -> Result<usize, usize> {
        self.clocks
            .binary_search_by_key(&time, |clock| clock.deadline)
    }
    fn bucket(&self, kind: Kind) -> Result<&Bucket, JobFault> {
        match kind {
            Kind::Ordinary(kind) => Ok(self.ordinary.get(kind)),
            Kind::Clock(time) => self
                .clock_index(time)
                .ok()
                .and_then(|index| self.clocks.get(index))
                .map(|clock| &clock.bucket)
                .ok_or_else(|| defect(JobFaultCause::MissingClockBucket)),
        }
    }
    fn validate_bucket(&self, kind: Kind) -> Result<(), JobFault> {
        let bucket = self.bucket(kind)?;
        match (bucket.head, bucket.tail) {
            (None, None) => Ok(()),
            (Some(head), Some(tail)) => {
                let first = self.node(head)?;
                let last = self.node(tail)?;
                if first.kind.get() != kind
                    || last.kind.get() != kind
                    || first.previous.get().is_some()
                    || last.next.get().is_some()
                {
                    return Err(defect(JobFaultCause::InvalidQueueLink { index: head.0 }));
                }
                Ok(())
            }
            _ => Err(defect(JobFaultCause::StateInconsistency)),
        }
    }
    fn validate_node(&self, slot: &Rc<Slot>) -> Result<(), JobFault> {
        if !Rc::ptr_eq(self.node(slot.index)?, slot) {
            return Err(defect(JobFaultCause::InvalidSlotIdentity {
                index: slot.index.0,
            }));
        }
        let kind = slot.kind.get();
        self.validate_bucket(kind)?;
        let bucket = self.bucket(kind)?;
        if let Some(previous) = slot.previous.get() {
            let other = self.node(previous)?;
            if previous == slot.index
                || other.kind.get() != kind
                || other.next.get() != Some(slot.index)
            {
                return Err(defect(JobFaultCause::InvalidQueueLink {
                    index: slot.index.0,
                }));
            }
        } else if bucket.head != Some(slot.index) {
            return Err(defect(JobFaultCause::InvalidQueueLink {
                index: slot.index.0,
            }));
        }
        if let Some(next) = slot.next.get() {
            let other = self.node(next)?;
            if next == slot.index
                || other.kind.get() != kind
                || other.previous.get() != Some(slot.index)
            {
                return Err(defect(JobFaultCause::InvalidQueueLink {
                    index: slot.index.0,
                }));
            }
        } else if bucket.tail != Some(slot.index) {
            return Err(defect(JobFaultCause::InvalidQueueLink {
                index: slot.index.0,
            }));
        }
        if slot.previous.get().is_some() && slot.previous.get() == slot.next.get() {
            return Err(defect(JobFaultCause::InvalidQueueLink {
                index: slot.index.0,
            }));
        }
        Ok(())
    }
    fn free_slot(&self) -> Result<(NodeId, Option<NodeId>), JobFault> {
        if self.len > self.entries.len() {
            return Err(defect(JobFaultCause::StateInconsistency));
        }
        match self.free {
            Some(id) => match self.entries.get(id.0) {
                Some(Entry::Free(next))
                    if next.is_none_or(|next| {
                        next != id && matches!(self.entries.get(next.0), Some(Entry::Free(_)))
                    }) =>
                {
                    Ok((id, *next))
                }
                _ => Err(defect(JobFaultCause::InvalidFreeLink { index: id.0 })),
            },
            None if self.len == self.entries.len() => Ok((NodeId(self.entries.len()), None)),
            None => Err(defect(JobFaultCause::StateInconsistency)),
        }
    }
    fn ensure_clock(&mut self, time: JsInstant) -> QueueResult<()> {
        if let Err(index) = self.clock_index(time) {
            reserve(&mut self.clocks, 1)?;
            self.clocks.insert(
                index,
                ClockBucket {
                    deadline: time,
                    bucket: Bucket::default(),
                    witness: Rc::new(ClockWitness {
                        retired: Cell::new(false),
                    }),
                },
            );
        }
        Ok(())
    }
    fn remove_empty_clock(&mut self, kind: Kind) {
        if let Kind::Clock(time) = kind
            && let Ok(index) = self.clock_index(time)
            && self
                .clocks
                .get(index)
                .is_some_and(|clock| clock.bucket.head.is_none() && clock.bucket.reservations == 0)
        {
            self.clocks.remove(index);
        }
    }
    #[expect(
        clippy::result_large_err,
        reason = "retain original job ownership without allocating to box the refused input"
    )]
    fn insert(
        &mut self,
        command: Resident,
        kind: Kind,
        queues: Weak<RuntimeJobQueues>,
    ) -> Result<(), (Resident, QueueError)> {
        let prepare = (|| -> QueueResult<_> {
            if !command.accepts(kind) {
                return Err(defect(JobFaultCause::StateInconsistency).into());
            }
            let (index, next_free) = self.free_slot()?;
            let length = self
                .len
                .checked_add(1)
                .ok_or(EngineResourceError::SizeOverflow)?;
            if index.0 == self.entries.len() {
                reserve(&mut self.entries, 1)?;
            }
            if let Kind::Clock(time) = kind {
                self.ensure_clock(time)?;
            }
            self.validate_bucket(kind)?;
            let previous = self
                .bucket(kind)?
                .tail
                .map(|id| self.node(id).cloned())
                .transpose()?;
            Ok((index, next_free, length, previous))
        })();
        let (index, next_free, length, previous) = match prepare {
            Ok(value) => value,
            Err(error) => {
                self.remove_empty_clock(kind);
                return Err((command, error));
            }
        };
        let appended = index.0 == self.entries.len();
        if appended {
            self.entries.push(Entry::Free(None));
        }
        let bucket = match kind {
            Kind::Ordinary(kind) => Some(self.ordinary.get_mut(kind)),
            Kind::Clock(time) => self
                .clocks
                .binary_search_by_key(&time, |clock| clock.deadline)
                .ok()
                .and_then(|index| self.clocks.get_mut(index))
                .map(|clock| &mut clock.bucket),
        };
        if let (Some(entry), Some(bucket)) = (self.entries.get_mut(index.0), bucket) {
            let (observer, native) = command.observation();
            let slot = Rc::new(Slot {
                index,
                kind: Cell::new(kind),
                previous: Cell::new(previous.as_ref().map(|slot| slot.index)),
                next: Cell::new(None),
                status: Cell::new(Status::Available),
                payload: Cell::new(PayloadState::Resident(Box::new(PayloadRecord {
                    command: Some(command),
                    next: None,
                }))),
                observer,
                native,
                queues,
            });
            // All fallible preparation precedes callback-free linking. Owned
            // neighbor witnesses remain valid throughout this Store loan.
            *entry = Entry::Node(slot);
            if let Some(previous) = previous {
                previous.next.set(Some(index));
            }
            if bucket.head.is_none() {
                bucket.head = Some(index);
            }
            bucket.tail = Some(index);
            self.free = next_free;
            self.len = length;
            Ok(())
        } else {
            if appended {
                self.entries.pop();
            }
            self.remove_empty_clock(kind);
            Err((command, defect(JobFaultCause::StateInconsistency).into()))
        }
    }

    fn unlink(&mut self, slot: &Rc<Slot>) -> Result<Rc<Slot>, JobFault> {
        self.validate_node(slot)?;
        if self.len > self.entries.len() {
            return Err(defect(JobFaultCause::StateInconsistency));
        }
        let length = self
            .len
            .checked_sub(1)
            .ok_or_else(|| defect(JobFaultCause::CounterUnderflow))?;
        if let Some(free) = self.free
            && !matches!(self.entries.get(free.0), Some(Entry::Free(_)))
        {
            return Err(defect(JobFaultCause::InvalidFreeLink { index: free.0 }));
        }
        let previous = slot
            .previous
            .get()
            .map(|id| self.node(id).cloned())
            .transpose()?;
        let next = slot
            .next
            .get()
            .map(|id| self.node(id).cloned())
            .transpose()?;
        let kind = slot.kind.get();
        let bucket = match kind {
            Kind::Ordinary(kind) => Some(self.ordinary.get_mut(kind)),
            Kind::Clock(time) => self
                .clocks
                .binary_search_by_key(&time, |clock| clock.deadline)
                .ok()
                .and_then(|index| self.clocks.get_mut(index))
                .map(|clock| &mut clock.bucket),
        }
        .ok_or_else(|| defect(JobFaultCause::MissingClockBucket))?;
        let entry = self.entries.get_mut(slot.index.0).ok_or_else(|| {
            defect(JobFaultCause::InvalidIndex {
                index: slot.index.0,
            })
        })?;
        // The caller's Rc keeps payload destruction out of this Store loan.
        *entry = Entry::Free(self.free);
        if let Some(previous) = &previous {
            previous.next.set(slot.next.get());
        }
        if let Some(next) = &next {
            next.previous.set(slot.previous.get());
        }
        if bucket.head == Some(slot.index) {
            bucket.head = slot.next.get();
        }
        if bucket.tail == Some(slot.index) {
            bucket.tail = slot.previous.get();
        }
        self.free = Some(slot.index);
        self.len = length;
        self.remove_empty_clock(kind);
        Ok(slot.clone())
    }
    fn move_clock(&mut self, slot: &Rc<Slot>, time: JsInstant) -> Result<(), JobFault> {
        self.validate_node(slot)?;
        let old_kind = slot.kind.get();
        let Kind::Clock(_) = old_kind else {
            return Err(defect(JobFaultCause::StateInconsistency));
        };
        let kind = Kind::Clock(time);
        self.validate_bucket(kind)?;
        if self.bucket(kind)?.reservations == 0 {
            return Err(defect(JobFaultCause::CounterUnderflow));
        }
        if old_kind == kind {
            return Ok(());
        }
        let previous = slot
            .previous
            .get()
            .map(|id| self.node(id).cloned())
            .transpose()?;
        let next = slot
            .next
            .get()
            .map(|id| self.node(id).cloned())
            .transpose()?;
        let tail = self
            .bucket(kind)?
            .tail
            .map(|id| self.node(id).cloned())
            .transpose()?;
        let old_time = match old_kind {
            Kind::Clock(time) => time,
            _ => return Err(defect(JobFaultCause::StateInconsistency)),
        };
        let old_index = self
            .clock_index(old_time)
            .map_err(|_| defect(JobFaultCause::MissingClockBucket))?;
        let new_index = self
            .clock_index(time)
            .map_err(|_| defect(JobFaultCause::MissingClockBucket))?;
        let [old, new] = self
            .clocks
            .get_disjoint_mut([old_index, new_index])
            .map_err(|_| defect(JobFaultCause::StateInconsistency))?;
        if let Some(previous) = previous {
            previous.next.set(slot.next.get());
        }
        if let Some(next) = next {
            next.previous.set(slot.previous.get());
        }
        if old.bucket.head == Some(slot.index) {
            old.bucket.head = slot.next.get();
        }
        if old.bucket.tail == Some(slot.index) {
            old.bucket.tail = slot.previous.get();
        }
        slot.kind.set(kind);
        slot.previous.set(tail.as_ref().map(|slot| slot.index));
        slot.next.set(None);
        if let Some(tail) = tail {
            tail.next.set(Some(slot.index));
        }
        if new.bucket.head.is_none() {
            new.bucket.head = Some(slot.index);
        }
        new.bucket.tail = Some(slot.index);
        self.remove_empty_clock(old_kind);
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ClaimSelection {
    Async,
    Promise,
    Generic,
    ClocksBefore(JsInstant),
}
#[derive(Default)]
struct ClaimScratch {
    slots: Vec<Rc<Slot>>,
    logical_capacity: usize,
}
impl ClaimScratch {
    fn reserve_one(&mut self, context: &mut Context) -> JsResult<()> {
        if self.slots.len() < self.logical_capacity {
            return Ok(());
        }
        let capacity = if self.logical_capacity == 0 {
            1
        } else {
            self.logical_capacity
                .checked_mul(2)
                .ok_or(EngineResourceError::SizeOverflow)?
        };
        let bytes = |count: usize| {
            count
                .checked_mul(size_of::<Rc<Slot>>())
                .filter(|bytes| *bytes <= isize::MAX as usize)
                .ok_or(EngineResourceError::SizeOverflow)
        };
        bytes(capacity)?;
        context.admit_job_claim(JobClaimWork::Capacity(bytes(
            capacity - self.logical_capacity,
        )?))?;
        context.admit_job_claim(JobClaimWork::Relocate(bytes(self.slots.len())?))?;
        self.slots
            .try_reserve_exact(capacity - self.slots.len())
            .map_err(|_| EngineResourceError::Allocation)?;
        self.logical_capacity = capacity;
        Ok(())
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
struct Subscription {
    source: FinalizationRegistryCleanupJob,
    pending: Cell<Option<Box<PayloadRecord>>>,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        let records = self.pending.get_mut().take();
        let mut cleanup = UnwindCleanup::default();
        dispose_records(records, &mut cleanup);
        cleanup.finish();
    }
}
struct ClockReservation {
    queues: Rc<RuntimeJobQueues>,
    time: JsInstant,
    witness: Rc<ClockWitness>,
}
impl Drop for ClockReservation {
    fn drop(&mut self) {
        if self.witness.retired.get() {
            return;
        }
        let result = self.queues.with_store_cleanup(|store| {
            let index = store
                .clock_index(self.time)
                .map_err(|_| defect(JobFaultCause::MissingClockBucket))?;
            let clock = store
                .clocks
                .get_mut(index)
                .ok_or_else(|| defect(JobFaultCause::MissingClockBucket))?;
            if !Rc::ptr_eq(&clock.witness, &self.witness) {
                return Err(defect(JobFaultCause::OwnerMismatch));
            }
            let bucket = &mut clock.bucket;
            let count = bucket
                .reservations
                .checked_sub(1)
                .ok_or_else(|| defect(JobFaultCause::CounterUnderflow))?;
            bucket.reservations = count;
            store.remove_empty_clock(Kind::Clock(self.time));
            Ok(())
        });
        if let Err(error) = result {
            self.queues.record_fault(error);
        }
    }
}
#[derive(Default)]
pub(crate) struct RuntimeJobQueues {
    store: RefCell<Store>,
    finalization_registry_subscriptions: RefCell<VecDeque<Rc<Subscription>>>,
    active_drains: Cell<usize>,
    first_fault: Cell<Option<JobFault>>,
    cleanup_pending: Cell<bool>,
    cleanup_driving: Cell<bool>,
    fault_delivery: Cell<usize>,
    registry: Weak<QueueRegistry>,
}
struct StoreRelease<'a> {
    queues: &'a RuntimeJobQueues,
    registry: Option<Rc<QueueRegistry>>,
    completed: bool,
    unwinding_at_entry: bool,
}
impl<'a> StoreRelease<'a> {
    fn new(queues: &'a RuntimeJobQueues) -> Self {
        Self {
            queues,
            registry: queues.registry.upgrade(),
            completed: true,
            unwinding_at_entry: std::thread::panicking(),
        }
    }
    fn with_store<T>(&mut self, run: impl FnOnce(&mut Store) -> QueueResult<T>) -> QueueResult<T> {
        if let Some(fault) = self.queues.first_fault.get() {
            return Err(fault.into());
        }
        self.unwinding_at_entry = std::thread::panicking();
        self.completed = false;
        let result = (|| {
            let mut store = self.queues.store.try_borrow_mut().map_err(|_| {
                self.queues
                    .latch_fault(defect(JobFaultCause::InaccessibleRegistry))
            })?;
            let result = run(&mut store);
            drop(store);
            if let Err(QueueError::Fault(fault)) = result.as_ref() {
                self.queues.latch_fault(*fault);
            }
            result
        })();
        self.completed = true;
        result
    }
    fn prepare_claim(
        &mut self,
        selection: ClaimSelection,
        context: &mut Context,
    ) -> JsResult<Batch> {
        let mut scratch = ClaimScratch::default();
        match selection {
            ClaimSelection::Async => {
                self.snapshot_bucket(Kind::Ordinary(Ordinary::Async), &mut scratch, context)?
            }
            ClaimSelection::Promise => {
                self.snapshot_bucket(Kind::Ordinary(Ordinary::Promise), &mut scratch, context)?
            }
            ClaimSelection::Generic => {
                self.snapshot_bucket(Kind::Ordinary(Ordinary::Generic), &mut scratch, context)?
            }
            ClaimSelection::ClocksBefore(cutoff) => {
                let mut previous = None;
                loop {
                    let time = self
                        .with_store(|store| {
                            Ok(store
                                .clocks
                                .get(previous.map_or(0, |previous| {
                                    store
                                        .clocks
                                        .partition_point(|clock| clock.deadline <= previous)
                                }))
                                .filter(|clock| clock.deadline < cutoff)
                                .map(|clock| clock.deadline))
                        })
                        .map_err(|error| crate::JsError::from(error.into_engine()))?;
                    let Some(time) = time else { break };
                    self.snapshot_bucket(Kind::Clock(time), &mut scratch, context)?;
                    previous = Some(time);
                }
            }
        }
        context.admit_job_claim(JobClaimWork::Commit(scratch.slots.len()))?;
        let batch = Batch {
            slots: scratch.slots,
        };
        // Admission covers every mark; no fallible step or stop/token read splits it.
        for slot in &batch.slots {
            slot.status.set(Status::Claimed);
        }
        Ok(batch)
    }
    fn snapshot_bucket(
        &mut self,
        kind: Kind,
        scratch: &mut ClaimScratch,
        context: &mut Context,
    ) -> JsResult<()> {
        context.admit_job_claim(JobClaimWork::Bucket)?;
        let (mut cursor, mut remaining) = self
            .with_store(|store| {
                let bucket = store.bucket(kind)?;
                if bucket.head.is_some() != bucket.tail.is_some() {
                    return Err(defect(JobFaultCause::StateInconsistency).into());
                }
                Ok((bucket.head, store.entries.len()))
            })
            .map_err(|error| crate::JsError::from(error.into_engine()))?;
        let mut previous = None;
        while let Some(index) = cursor {
            context.admit_job_claim(JobClaimWork::Node)?;
            let (next, available) = self
                .with_store(|store| {
                    if remaining == 0 {
                        return Err(
                            defect(JobFaultCause::InvalidQueueLink { index: index.0 }).into()
                        );
                    }
                    let slot = store.node(index)?;
                    store.validate_node(slot)?;
                    if slot.kind.get() != kind || slot.previous.get() != previous {
                        return Err(
                            defect(JobFaultCause::InvalidQueueLink { index: index.0 }).into()
                        );
                    }
                    Ok((slot.next.get(), slot.status.get() == Status::Available))
                })
                .map_err(|error| crate::JsError::from(error.into_engine()))?;
            remaining -= 1;
            previous = Some(index);
            cursor = next;
            if available {
                scratch.reserve_one(context)?;
                context.admit_job_claim(JobClaimWork::SnapshotCopy)?;
                let slot = self
                    .with_store(|store| Ok(store.node(index)?.clone()))
                    .map_err(|error| crate::JsError::from(error.into_engine()))?;
                scratch.slots.push(slot);
            }
        }
        Ok(())
    }
}
impl Drop for StoreRelease<'_> {
    fn drop(&mut self) {
        if !self.completed && !self.unwinding_at_entry && std::thread::panicking() {
            self.queues
                .latch_fault(defect(JobFaultCause::StateInconsistency));
        }
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| self.queues.finish_fault_cleanup());
        if let Some(registry) = &self.registry {
            cleanup.run(|| registry.drive());
        }
        cleanup.finish();
    }
}
impl RuntimeJobQueues {
    fn registered(registry: Weak<QueueRegistry>) -> Self {
        Self {
            store: RefCell::new(Store::default()),
            finalization_registry_subscriptions: RefCell::new(VecDeque::new()),
            active_drains: Cell::new(0),
            first_fault: Cell::new(None),
            cleanup_pending: Cell::new(false),
            cleanup_driving: Cell::new(false),
            fault_delivery: Cell::new(0),
            registry,
        }
    }
    fn latch_fault(&self, fault: JobFault) -> JobFault {
        let fault = self.first_fault.get().unwrap_or(fault);
        self.first_fault.set(Some(fault));
        self.cleanup_pending.set(true);
        fault
    }
    fn record_fault(&self, fault: JobFault) -> JobFault {
        let fault = self.latch_fault(fault);
        self.finish_fault_cleanup();
        fault
    }
    fn with_store_cleanup<T>(
        &self,
        run: impl FnOnce(&mut Store) -> Result<T, JobFault>,
    ) -> Result<T, JobFault> {
        let mut release = StoreRelease::new(self);
        release.completed = false;
        let mut store = self
            .store
            .try_borrow_mut()
            .map_err(|_| self.latch_fault(defect(JobFaultCause::InaccessibleRegistry)))?;
        let result = run(&mut store);
        release.completed = true;
        drop(store);
        if let Err(fault) = result.as_ref() {
            self.latch_fault(*fault);
        }
        result
    }
    fn with_store<T>(&self, run: impl FnOnce(&mut Store) -> QueueResult<T>) -> QueueResult<T> {
        if let Some(fault) = self.first_fault.get() {
            return Err(fault.into());
        }
        let mut release = StoreRelease::new(self);
        release.with_store(run)
    }
    fn physical_slot(&self, index: usize) -> Result<Option<Rc<Slot>>, JobFault> {
        let store = self
            .store
            .try_borrow()
            .map_err(|_| self.latch_fault(defect(JobFaultCause::InaccessibleRegistry)))?;
        match store.entries.get(index) {
            Some(Entry::Node(slot)) => Ok(Some(slot.clone())),
            Some(Entry::Free(_)) => Ok(None),
            None => Err(self.latch_fault(defect(JobFaultCause::InvalidIndex { index }))),
        }
    }
    fn finish_fault_cleanup(&self) {
        if let Some(registry) = self.registry.upgrade()
            && registry.first_fault.get().is_some()
        {
            registry.drive();
            return;
        }
        if self.cleanup_pending.get() && self.mark_fault_bank() {
            self.notify_marked_fault_bank();
        }
    }
    // This complete pass contains only closed, atomic owner marks. No external
    // callback may run until every affected registry member passes this gate.
    fn mark_fault_bank(&self) -> bool {
        let Some(fault) = self.first_fault.get() else {
            return true;
        };
        let length = match self.store.try_borrow_mut() {
            Ok(store) => store.entries.len(),
            Err(_) => return false,
        };
        if self
            .finalization_registry_subscriptions
            .try_borrow_mut()
            .is_err()
        {
            return false;
        }
        for index in 0..length {
            match self.physical_slot(index) {
                Ok(Some(slot)) => slot.mark_fault(fault),
                Ok(None) => {}
                Err(_) => return false,
            }
        }
        true
    }
    fn notify_marked_fault_bank(&self) -> bool {
        if self.cleanup_driving.get() {
            return false;
        }
        let Some(fault) = self.first_fault.get() else {
            return true;
        };
        if self.store.try_borrow_mut().is_err()
            || self
                .finalization_registry_subscriptions
                .try_borrow_mut()
                .is_err()
        {
            return false;
        }
        self.cleanup_pending.set(false);
        self.cleanup_driving.set(true);
        let mut cleanup = UnwindCleanup::default();
        let mut complete = true;
        loop {
            let next = {
                let Ok(store) = self.store.try_borrow() else {
                    complete = false;
                    break;
                };
                let index = self.fault_delivery.get();
                // A reentrant clear owns the detached recipients' retirement;
                // inspect the current bank rather than retaining a stale bound.
                if index == store.entries.len() {
                    break;
                }
                let slot = match store.entries.get(index) {
                    Some(Entry::Node(slot)) => Some(slot.clone()),
                    Some(Entry::Free(_)) => None,
                    None => {
                        self.latch_fault(defect(JobFaultCause::InvalidIndex { index }));
                        complete = false;
                        break;
                    }
                };
                let Some(next) = index.checked_add(1) else {
                    self.latch_fault(defect(JobFaultCause::StateInconsistency));
                    complete = false;
                    break;
                };
                self.fault_delivery.set(next);
                slot
            };
            if let Some(slot) = next {
                cleanup.run(move || slot.notify_fault(fault));
            }
        }
        self.cleanup_pending.set(!complete);
        self.cleanup_driving.set(false);
        if let Some(registry) = self.registry.upgrade() {
            cleanup.run(move || registry.drive());
        }
        cleanup.finish();
        complete
    }
    fn present<T>(&self, host: &HostContext<'_>, result: QueueResult<T>) -> JsResult<T> {
        let result = match result {
            Err(QueueError::Fault(fault))
                if matches!(
                    fault.owner(),
                    JobFaultOwner::Queue | JobFaultOwner::ExecutorRegistry
                ) =>
            {
                Err(QueueError::Fault(self.record_fault(fault)))
            }
            other => other,
        };
        host.with(|access| {
            let context = access.into_owner();
            context.check_execution_stop()?;
            result.map_err(|error| error.into_engine().into())
        })
    }
    fn claim(&self, selection: ClaimSelection, host: &HostContext<'_>) -> JsResult<Batch> {
        // The acquired claim owns one delivery pin across all short Store loans.
        // It is released only after both the phase and protected access end.
        let mut release = None;
        let result = host.with(|access| {
            let context = access.into_owner();
            context.check_execution_stop()?;
            let release = release.insert(StoreRelease::new(self));
            let result = context.with_execution_phase(ExecutionPhase::JobClaim, |context| {
                let mut result = release.prepare_claim(selection, context);
                if let Err(error) = &mut result {
                    context.retain_preparation_error(error);
                }
                result
            });
            context.check_execution_stop().and(result)
        });
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| drop(release));
        cleanup.finish_with(result)
    }
    fn finish(&self, slot: &Rc<Slot>) -> Result<(), JobFault> {
        if slot.status.get() == Status::Retired {
            return Ok(());
        }
        let removed = self.with_store_cleanup(|store| store.unlink(slot))?;
        slot.status.set(Status::Retired);
        drop(removed);
        Ok(())
    }
    fn finish_action(
        &self,
        slot: &Rc<Slot>,
        host: &HostContext<'_>,
        result: JsResult<crate::JsValue>,
    ) -> JsResult<()> {
        let finished = self.finish(slot).map_err(|fault| self.record_fault(fault));
        host.with(|access| {
            access.into_owner().check_execution_stop()?;
            match result {
                Err(error) if error.as_engine().is_some() => Err(error),
                result => {
                    finished?;
                    result.map(|_| ())
                }
            }
        })
    }
    fn reserve_clock(self: &Rc<Self>, time: JsInstant) -> QueueResult<ClockReservation> {
        let witness = self.with_store(|store| {
            store.ensure_clock(time)?;
            let index = store
                .clock_index(time)
                .map_err(|_| defect(JobFaultCause::MissingClockBucket))?;
            let clock = store
                .clocks
                .get_mut(index)
                .ok_or_else(|| defect(JobFaultCause::MissingClockBucket))?;
            let bucket = &mut clock.bucket;
            let count = bucket
                .reservations
                .checked_add(1)
                .ok_or(EngineResourceError::SizeOverflow)?;
            bucket.reservations = count;
            Ok(clock.witness.clone())
        })?;
        Ok(ClockReservation {
            queues: self.clone(),
            time,
            witness,
        })
    }
    fn retire_matching(&self, reason: JobRetirement, matches: impl Fn(&Slot) -> bool) {
        let length = match self.store.try_borrow() {
            Ok(store) => store.entries.len(),
            Err(_) => {
                self.record_fault(defect(JobFaultCause::InaccessibleRegistry));
                return;
            }
        };
        let mut cleanup = UnwindCleanup::default();
        for index in 0..length {
            let slot = match self.physical_slot(index) {
                Ok(slot) => slot,
                Err(fault) => {
                    self.record_fault(fault);
                    break;
                }
            };
            if let Some(slot) = slot
                && matches(&slot)
            {
                slot.status.set(Status::Retired);
            }
        }
        for index in 0..length {
            let slot = match self.physical_slot(index) {
                Ok(slot) => slot,
                Err(fault) => {
                    self.record_fault(fault);
                    break;
                }
            };
            if let Some(slot) = slot
                && slot.status.get() == Status::Retired
            {
                let removed = self.with_store_cleanup(|store| store.unlink(&slot));
                if let Err(fault) = removed {
                    self.record_fault(fault);
                }
                cleanup.run(move || slot.retire(reason));
            }
        }
        cleanup.finish();
    }
    pub(crate) fn retire_owner(&self, owner: &ContextId) {
        self.retire_matching(JobRetirement::OwnerDropped, |slot| {
            slot.owner().as_ref() == Some(owner)
        });
    }
    fn prune_cancelled_clocks(&self) {
        self.retire_matching(JobRetirement::Cancelled, Slot::cancelled);
    }
    fn clear(&self) -> Result<(), JobFault> {
        let entries = match self.store.try_borrow_mut() {
            Ok(mut store) => {
                self.fault_delivery.set(0);
                for clock in &store.clocks {
                    clock.witness.retired.set(true);
                }
                std::mem::take(&mut *store).entries
            }
            Err(_) => {
                return Err(self.record_fault(defect(JobFaultCause::InaccessibleRegistry)));
            }
        };
        let mut cleanup = UnwindCleanup::default();
        for entry in &entries {
            if let Entry::Node(slot) = entry {
                slot.status.set(Status::Retired);
            }
        }
        for entry in entries {
            if let Entry::Node(slot) = entry {
                cleanup.run(move || slot.retire(JobRetirement::Cancelled));
            }
        }
        cleanup.finish();
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "retain original job ownership without allocating to box the refused input"
    )]
    fn publish(
        self: &Rc<Self>,
        command: Resident,
        kind: Kind,
    ) -> Result<(), (Resident, QueueError)> {
        if let Some(fault) = self.first_fault.get() {
            return Err((command, fault.into()));
        }
        let mut release = StoreRelease::new(self);
        release.completed = false;
        let mut store = match self.store.try_borrow_mut() {
            Ok(store) => store,
            Err(_) => {
                return Err((
                    command,
                    self.latch_fault(defect(JobFaultCause::InaccessibleRegistry))
                        .into(),
                ));
            }
        };
        let result = store.insert(command, kind, Rc::downgrade(self));
        release.completed = true;
        drop(store);
        if let Err((_, QueueError::Fault(fault))) = &result {
            self.latch_fault(*fault);
        }
        result
    }
    fn publish_subscription(
        &self,
        source: FinalizationRegistryCleanupJob,
    ) -> Result<(), (FinalizationRegistryCleanupJob, QueueError)> {
        if let Some(fault) = self.first_fault.get() {
            return Err((source, fault.into()));
        }
        let mut release = StoreRelease::new(self);
        release.completed = false;
        let mut subscriptions = match self.finalization_registry_subscriptions.try_borrow_mut() {
            Ok(subscriptions) => subscriptions,
            Err(_) => {
                return Err((
                    source,
                    self.latch_fault(defect(JobFaultCause::InaccessibleRegistry))
                        .into(),
                ));
            }
        };
        let valid = subscriptions
            .len()
            .checked_add(1)
            .and_then(|count| count.checked_mul(size_of::<Rc<Subscription>>()))
            .filter(|bytes| *bytes <= isize::MAX as usize);
        if valid.is_none() {
            release.completed = true;
            return Err((source, EngineResourceError::SizeOverflow.into()));
        }
        if subscriptions.try_reserve(1).is_err() {
            release.completed = true;
            return Err((source, EngineResourceError::Allocation.into()));
        }
        subscriptions.push_back(Rc::new(Subscription {
            source,
            pending: Cell::new(None),
        }));
        release.completed = true;
        Ok(())
    }
}
struct DrainGuard(Rc<RuntimeJobQueues>);
impl Drop for DrainGuard {
    fn drop(&mut self) {
        match self.0.active_drains.get().checked_sub(1) {
            Some(count) => self.0.active_drains.set(count),
            None => {
                self.0.record_fault(defect(JobFaultCause::CounterUnderflow));
            }
        }
    }
}

impl RuntimeJobQueues {
    fn shutdown(&self) -> bool {
        let _release = StoreRelease::new(self);
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(|| {
            let _ = self.clear();
        });
        // Clear detaches its entire bank before external callbacks. Even if a
        // callback unwinds, subscriptions remain an independent cleanup set.
        let cleared = self
            .store
            .try_borrow()
            .is_ok_and(|store| store.entries.is_empty());
        let subscriptions = match self.finalization_registry_subscriptions.try_borrow_mut() {
            Ok(mut subscriptions) => Some(std::mem::take(&mut *subscriptions)),
            Err(_) => {
                cleanup.run(|| {
                    self.record_fault(defect(JobFaultCause::InaccessibleRegistry));
                });
                None
            }
        };
        let complete = subscriptions.is_some() && cleared;
        if let Some(subscriptions) = subscriptions {
            for subscription in subscriptions {
                cleanup.run(move || drop(subscription));
            }
        }
        cleanup.finish();
        complete
    }
    fn take_ready_cleanup(self: &Rc<Self>) -> QueueResult<bool> {
        let _release = StoreRelease::new(self);
        if let Some(fault) = self.first_fault.get() {
            return Err(fault.into());
        }
        let length = self
            .finalization_registry_subscriptions
            .try_borrow()
            .map_err(|_| self.record_fault(defect(JobFaultCause::InaccessibleRegistry)))?
            .len();
        for _ in 0..length {
            let subscription = {
                let mut subscriptions = self
                    .finalization_registry_subscriptions
                    .try_borrow_mut()
                    .map_err(|_| {
                    self.latch_fault(defect(JobFaultCause::InaccessibleRegistry))
                })?;
                let Some(subscription) = subscriptions.pop_front() else {
                    drop(subscriptions);
                    return Err(self
                        .record_fault(defect(JobFaultCause::StateInconsistency))
                        .into());
                };
                subscriptions.push_back(subscription.clone());
                subscription
            };
            let record = if let Some(record) = subscription.pending.take() {
                Some(record)
            } else {
                match subscription.source.try_take() {
                    Poll::Pending => None,
                    Poll::Ready(Some(job)) => Some(Box::new(PayloadRecord {
                        command: Some(Resident::Generic(GenericJob(job))),
                        next: None,
                    })),
                    Poll::Ready(None) => {
                        let mut subscriptions = self
                            .finalization_registry_subscriptions
                            .try_borrow_mut()
                            .map_err(|_| {
                                self.record_fault(defect(JobFaultCause::InaccessibleRegistry))
                            })?;
                        subscriptions.retain(|entry| !Rc::ptr_eq(entry, &subscription));
                        None
                    }
                }
            };
            if let Some(mut record) = record {
                let Some(command) = record.command.take() else {
                    subscription.pending.set(Some(record));
                    return Err(self
                        .record_fault(defect(JobFaultCause::StateInconsistency))
                        .into());
                };
                match self.publish(command, Kind::Ordinary(Ordinary::Generic)) {
                    Ok(()) => return Ok(true),
                    Err((command, error)) => {
                        record.command = Some(command);
                        record.next = subscription.pending.take();
                        subscription.pending.set(Some(record));
                        return Err(error);
                    }
                }
            }
        }
        Ok(false)
    }
}
impl Drop for RuntimeJobQueues {
    fn drop(&mut self) {
        for clock in &self.store.get_mut().clocks {
            clock.witness.retired.set(true);
        }
        let entries = std::mem::take(&mut self.store.get_mut().entries);
        let subscriptions = std::mem::take(self.finalization_registry_subscriptions.get_mut());
        let mut cleanup = UnwindCleanup::default();
        for entry in &entries {
            if let Entry::Node(slot) = entry {
                slot.status.set(Status::Retired);
            }
        }
        for entry in entries {
            if let Entry::Node(slot) = entry {
                cleanup.run(move || slot.retire(JobRetirement::Dropped));
            }
        }
        for subscription in subscriptions {
            cleanup.run(move || drop(subscription));
        }
        cleanup.finish();
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum RegistryPhase {
    #[default]
    Healthy,
    Marking,
    Notifying,
    Complete,
}
#[derive(Default)]
struct QueueRegistry {
    queues: RefCell<Vec<(ObjectId, Weak<RuntimeJobQueues>)>>,
    first_fault: Cell<Option<JobFault>>,
    phase: Cell<RegistryPhase>,
    driving: Cell<bool>,
    shutdown_requested: Cell<bool>,
    shutdown_complete: Cell<bool>,
}
struct RegistryRelease {
    registry: Rc<QueueRegistry>,
    completed: bool,
    unwinding_at_entry: bool,
}
impl Drop for RegistryRelease {
    fn drop(&mut self) {
        if !self.completed && !self.unwinding_at_entry && std::thread::panicking() {
            self.registry.latch_fault(JobFault::new(
                JobFaultOwner::ExecutorRegistry,
                JobFaultCause::StateInconsistency,
            ));
        }
        self.registry.drive();
    }
}
impl QueueRegistry {
    fn latch_fault(&self, fault: JobFault) -> JobFault {
        let fault = self.first_fault.get().unwrap_or(fault);
        self.first_fault.set(Some(fault));
        if self.phase.get() == RegistryPhase::Healthy {
            self.phase.set(RegistryPhase::Marking);
        }
        fault
    }
    fn member(&self, index: usize) -> Option<Option<Rc<RuntimeJobQueues>>> {
        let members = self.queues.try_borrow().ok()?;
        members.get(index).map(|(_, queue)| queue.upgrade())
    }
    fn drive(&self) {
        if self.driving.get() {
            return;
        }
        if self.phase.get() != RegistryPhase::Healthy && self.first_fault.get().is_none() {
            self.latch_fault(JobFault::new(
                JobFaultOwner::ExecutorRegistry,
                JobFaultCause::MissingCause,
            ));
            self.phase.set(RegistryPhase::Marking);
        }
        if self.phase.get() == RegistryPhase::Healthy && !self.shutdown_requested.get() {
            return;
        }
        let length = match self.queues.try_borrow_mut() {
            Ok(queues) => queues.len(),
            Err(_) => return,
        };
        self.driving.set(true);
        let mut cleanup = UnwindCleanup::default();
        let mut marked = true;
        if self.phase.get() == RegistryPhase::Marking {
            if let Some(fault) = self.first_fault.get() {
                // First latch every queue, then mark every physical observer
                // bank. A blocked bank leaves ALL notification authority here.
                for index in 0..length {
                    match self.member(index) {
                        Some(Some(queue)) => {
                            queue.latch_fault(fault);
                        }
                        Some(None) => {}
                        None => {
                            marked = false;
                            break;
                        }
                    }
                }
                if marked {
                    for index in 0..length {
                        match self.member(index) {
                            Some(Some(queue)) => {
                                if !queue.mark_fault_bank() {
                                    marked = false;
                                }
                            }
                            Some(None) => {}
                            None => {
                                marked = false;
                            }
                        }
                    }
                }
            } else {
                marked = false;
            }
            if marked {
                self.phase.set(RegistryPhase::Notifying);
            }
        }
        if marked && self.phase.get() == RegistryPhase::Notifying {
            let mut complete = true;
            for index in 0..length {
                match self.member(index) {
                    Some(Some(queue)) => {
                        cleanup.run(|| {
                            if !queue.notify_marked_fault_bank() {
                                complete = false;
                            }
                        });
                        cleanup.run(move || drop(queue));
                    }
                    Some(None) => {}
                    None => {
                        complete = false;
                    }
                }
            }
            if complete {
                self.phase.set(RegistryPhase::Complete);
            }
        }
        if matches!(
            self.phase.get(),
            RegistryPhase::Healthy | RegistryPhase::Complete
        ) && self.shutdown_requested.get()
            && !self.shutdown_complete.get()
        {
            let mut complete = true;
            for index in 0..length {
                match self.member(index) {
                    Some(Some(queue)) => {
                        cleanup.run(|| {
                            if !queue.shutdown() {
                                complete = false;
                            }
                        });
                        cleanup.run(move || drop(queue));
                    }
                    Some(None) => {}
                    None => {
                        complete = false;
                    }
                }
            }
            if complete && let Ok(mut queues) = self.queues.try_borrow_mut() {
                queues.clear();
                self.shutdown_complete.set(true);
            }
        }
        self.driving.set(false);
        cleanup.finish();
    }
    fn request_shutdown(&self) {
        self.shutdown_requested.set(true);
        self.drive();
    }
    fn queues_for(self: &Rc<Self>, context: &Context) -> QueueResult<Rc<RuntimeJobQueues>> {
        if let Some(fault) = self.first_fault.get() {
            return Err(fault.into());
        }
        if self.shutdown_requested.get() {
            return Err(JobFault::new(
                JobFaultOwner::ExecutorRegistry,
                JobFaultCause::OwnerMismatch,
            )
            .into());
        }
        let mut release = RegistryRelease {
            registry: self.clone(),
            completed: false,
            unwinding_at_entry: std::thread::panicking(),
        };
        let identity = context.realm().identity();
        let mut queues = self.queues.try_borrow_mut().map_err(|_| {
            self.latch_fault(JobFault::new(
                JobFaultOwner::ExecutorRegistry,
                JobFaultCause::InaccessibleRegistry,
            ))
        })?;
        if let Some(queue) = queues
            .iter()
            .find(|(key, _)| key.same_heap(identity))
            .and_then(|(_, queue)| queue.upgrade())
        {
            release.completed = true;
            if let Some(fault) = queue.first_fault.get() {
                return Err(fault.into());
            }
            return Ok(queue);
        }
        queues.retain(|(_, queue)| queue.strong_count() != 0);
        let result = reserve(&mut queues, 1);
        release.completed = true;
        result?;
        let queue = Rc::new(RuntimeJobQueues::registered(Rc::downgrade(self)));
        queues.push((identity, Rc::downgrade(&queue)));
        Ok(queue)
    }
}
/// A FIFO executor retaining accepted work independently of temporary drains.
#[derive(Default)]
pub struct SimpleJobExecutor {
    registry: Rc<QueueRegistry>,
    stop: Arc<AtomicBool>,
}
impl std::fmt::Debug for SimpleJobExecutor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SimpleJobExecutor")
            .finish_non_exhaustive()
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
    pub(crate) fn attach_context(
        &self,
        context: &mut Context,
    ) -> Result<Rc<RuntimeJobQueues>, EngineError> {
        if let Some(fault) = self.registry.first_fault.get() {
            return Err(QueueError::Fault(fault).into_engine());
        }
        context.prepare_simple_job_queue()?;
        let queues = self.queues_for(context).map_err(QueueError::into_engine)?;
        context.retain_simple_job_queue(&queues);
        Ok(queues)
    }
    fn queues_for(&self, context: &Context) -> QueueResult<Rc<RuntimeJobQueues>> {
        self.registry.queues_for(context)
    }
}
impl Drop for SimpleJobExecutor {
    fn drop(&mut self) {
        self.registry.request_shutdown();
    }
}

fn invoke_owned<J>(
    host: &HostContext<'_>,
    job: J,
    invoke: impl for<'call> FnOnce(J, ContextAccess<'call>) -> JobCall<J>,
) -> JobCall<J> {
    match host.with_owned(job, invoke) {
        Ok(outcome) => outcome,
        Err((job, error)) => JobCall::Deferred { job, error },
    }
}
impl JobExecutor for SimpleJobExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        access: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        let context = access.into_owner();
        if let Some(fault) = self.registry.first_fault.get() {
            return Err(JobPublicationError::new(
                job,
                JobPublicationCause::from_engine(QueueError::Fault(fault).into_engine(), context),
            ));
        }
        if let Err(cause) = job.check_publication(context.access()) {
            return Err(JobPublicationError::new(job, cause));
        }
        // Deadline arithmetic and all clocks precede insertion and observe the
        // receiving Context's actual epoch, without consuming the original job.
        let kind = match &job {
            Job::TimeoutJob(timer) => {
                deadline(context.clock().now(), timer.timeout()).map(Kind::Clock)
            }
            Job::IntervalJob(timer) => {
                deadline(context.clock().now(), timer.interval()).map(Kind::Clock)
            }
            Job::PromiseJob(_) => Ok(Kind::Ordinary(Ordinary::Promise)),
            Job::AsyncJob(_) => Ok(Kind::Ordinary(Ordinary::Async)),
            Job::GenericJob(_) => Ok(Kind::Ordinary(Ordinary::Generic)),
            Job::FinalizationRegistryCleanupJob(_) => Ok(Kind::Ordinary(Ordinary::Generic)),
        };
        let kind = match kind {
            Ok(kind) => kind,
            Err(error) => {
                return Err(JobPublicationError::new(
                    job,
                    JobPublicationCause::from_engine(error.into_engine(), context),
                ));
            }
        };
        let queues = match self.attach_context(context) {
            Ok(queues) => queues,
            Err(error) => {
                return Err(JobPublicationError::new(
                    job,
                    JobPublicationCause::from_engine(error, context),
                ));
            }
        };
        let result = match Resident::from_job(job) {
            Ok(command) => queues
                .publish(command, kind)
                .map_err(|(command, error)| (command.into_job(), error)),
            Err(source) => queues
                .publish_subscription(source)
                .map_err(|(source, error)| (Job::FinalizationRegistryCleanupJob(source), error)),
        };
        match result {
            Ok(()) => Ok(JobAcceptance::Accepted),
            Err((job, error)) => Err(JobPublicationError::new(
                job,
                JobPublicationCause::from_engine(error.into_engine(), context),
            )),
        }
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
            self.attach_context(context).map_err(Into::into)
        })?;
        let drains = queues
            .active_drains
            .get()
            .checked_add(1)
            .ok_or(EngineResourceError::SizeOverflow)?;
        queues.active_drains.set(drains);
        let _drain = DrainGuard(queues.clone());
        loop {
            if self.stop.load(Ordering::Relaxed) && self.stop.swap(false, Ordering::Relaxed) {
                return queues.present(host, queues.clear().map_err(QueueError::Fault));
            }
            host.with(|access| access.into_owner().check_execution_stop())?;
            let async_batch = queues.claim(ClaimSelection::Async, host)?;
            let mut did_work = !async_batch.slots.is_empty();
            // Prepare every captured request before polling any actual task.
            for slot in &async_batch.slots {
                let Some(mut payload) =
                    queues.present(host, Payload::take(slot).map_err(QueueError::Fault))?
                else {
                    continue;
                };
                let command = match payload.command_mut() {
                    Ok(command) => command,
                    Err(fault) => {
                        return queues.present(host, Err(queues.record_fault(fault).into()));
                    }
                };
                let Resident::Async(job) = command else {
                    return queues.present(
                        host,
                        Err(queues
                            .record_fault(defect(JobFaultCause::StateInconsistency))
                            .into()),
                    );
                };
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
                            queues.finish_action(slot, host, result)?;
                        }
                        JobStep::Finished(status) => {
                            queues.finish_action(
                                slot,
                                host,
                                status
                                    .map(|_| crate::JsValue::undefined())
                                    .map_err(Into::into),
                            )?;
                        }
                        JobStep::Pending => {
                            return queues.present(
                                host,
                                Err(queues
                                    .record_fault(defect(JobFaultCause::StateInconsistency))
                                    .into()),
                            );
                        }
                    }
                } else if let Some(required) = job.owner()
                    && host.with(|access| Ok(access.id()))? != required
                {
                    return Err(EngineError::JobContextRequired { required }.into());
                }
            }
            let clock = host.with(|access| Ok(access.into_owner().clock_handle()))?;
            let now = clock.now();
            queues.prune_cancelled_clocks();
            let clock_batch = queues.claim(ClaimSelection::ClocksBefore(now), host)?;
            did_work |= !clock_batch.slots.is_empty();
            for slot in &clock_batch.slots {
                let Some(mut payload) =
                    queues.present(host, Payload::take(slot).map_err(QueueError::Fault))?
                else {
                    continue;
                };
                let command_slot = match payload.command_slot() {
                    Ok(command) => command,
                    Err(fault) => {
                        return queues.present(host, Err(queues.record_fault(fault).into()));
                    }
                };
                let Some(command) = command_slot.take() else {
                    return queues.present(
                        host,
                        Err(queues
                            .record_fault(defect(JobFaultCause::StateInconsistency))
                            .into()),
                    );
                };
                match command {
                    Resident::Timeout(job) => {
                        if job.cancelled() {
                            queues.present(host, queues.finish(slot).map_err(QueueError::Fault))?;
                            continue;
                        }
                        match invoke_owned(host, job, |job, access| job.call(access)) {
                            JobCall::Deferred { job, error } => {
                                *command_slot = Some(Resident::Timeout(job));
                                return Err(error);
                            }
                            JobCall::Complete(result) => {
                                queues.finish_action(slot, host, result)?;
                            }
                        }
                    }
                    Resident::Interval(job) => {
                        if job.cancelled() {
                            queues.present(host, queues.finish(slot).map_err(QueueError::Fault))?;
                            continue;
                        }
                        // Own the reusable command through every fallible pre-action
                        // operation, including holder access and destination capacity.
                        *command_slot = Some(Resident::Interval(job));
                        let clock = host.with(|access| Ok(access.into_owner().clock_handle()))?;
                        let now = clock.now();
                        let interval = match command_slot.as_ref() {
                            Some(Resident::Interval(job)) => job.interval(),
                            _ => {
                                return queues.present(
                                    host,
                                    Err(queues
                                        .record_fault(defect(JobFaultCause::StateInconsistency))
                                        .into()),
                                );
                            }
                        };
                        let time = queues.present(host, deadline(now, interval))?;
                        let _reservation = queues.present(host, queues.reserve_clock(time))?;
                        let invocation = match command_slot.as_ref() {
                            Some(Resident::Interval(job)) => {
                                match host.with_owned((), |(), access| job.call(access)) {
                                    Ok(invocation) => invocation,
                                    Err(((), error)) => JobInvocation::Deferred(error),
                                }
                            }
                            _ => {
                                return queues.present(
                                    host,
                                    Err(queues
                                        .record_fault(defect(JobFaultCause::StateInconsistency))
                                        .into()),
                                );
                            }
                        };
                        match invocation {
                            JobInvocation::Deferred(error) => return Err(error),
                            JobInvocation::Complete(result) => {
                                if let Err(error) = result {
                                    return queues.finish_action(slot, host, Err(error));
                                }
                                if slot.status.get() != Status::Retired {
                                    let moved = queues.with_store(|store| {
                                        store.move_clock(slot, time).map_err(QueueError::Fault)
                                    });
                                    queues.present(host, moved)?;
                                }
                            }
                        }
                    }
                    other => {
                        *command_slot = Some(other);
                        return queues.present(
                            host,
                            Err(queues
                                .record_fault(defect(JobFaultCause::StateInconsistency))
                                .into()),
                        );
                    }
                }
            }
            // Preserve the existing idle short circuit before promise/generic selection.
            let empty = queues.present(
                host,
                queues.with_store(|store| store.is_empty().map_err(QueueError::Fault)),
            )?;
            if empty {
                if queues.present(host, queues.take_ready_cleanup())? {
                    did_work = true;
                } else {
                    break;
                }
            }
            for slot in &async_batch.slots {
                let Some(mut payload) =
                    queues.present(host, Payload::take(slot).map_err(QueueError::Fault))?
                else {
                    continue;
                };
                let command = match payload.command_mut() {
                    Ok(command) => command,
                    Err(fault) => {
                        return queues.present(host, Err(queues.record_fault(fault).into()));
                    }
                };
                let Resident::Async(job) = command else {
                    return queues.present(
                        host,
                        Err(queues
                            .record_fault(defect(JobFaultCause::StateInconsistency))
                            .into()),
                    );
                };
                let step = future::poll_fn(|cx| {
                    job.register_driver(cx.waker());
                    if job.is_ready() {
                        Poll::Ready(Some(job.run_step(cx, host)))
                    } else {
                        Poll::Ready(None)
                    }
                })
                .await;
                match step {
                    None | Some(JobStep::Pending) => {}
                    Some(JobStep::Deferred(error)) => return Err(error),
                    Some(JobStep::Complete(result)) => {
                        queues.finish_action(slot, host, result)?;
                    }
                    Some(JobStep::Finished(status)) => {
                        queues.finish_action(
                            slot,
                            host,
                            status
                                .map(|_| crate::JsValue::undefined())
                                .map_err(Into::into),
                        )?;
                    }
                    Some(JobStep::Prepared) => {
                        return queues.present(
                            host,
                            Err(queues
                                .record_fault(defect(JobFaultCause::StateInconsistency))
                                .into()),
                        );
                    }
                }
            }
            for selection in [ClaimSelection::Promise, ClaimSelection::Generic] {
                let batch = queues.claim(selection, host)?;
                did_work |= !batch.slots.is_empty();
                for slot in &batch.slots {
                    let Some(mut payload) =
                        queues.present(host, Payload::take(slot).map_err(QueueError::Fault))?
                    else {
                        continue;
                    };
                    let command_slot = match payload.command_slot() {
                        Ok(command) => command,
                        Err(fault) => {
                            return queues.present(host, Err(queues.record_fault(fault).into()));
                        }
                    };
                    let Some(command) = command_slot.take() else {
                        return queues.present(
                            host,
                            Err(queues
                                .record_fault(defect(JobFaultCause::StateInconsistency))
                                .into()),
                        );
                    };
                    let result = match (selection, command) {
                        (ClaimSelection::Promise, Resident::Promise(job)) => {
                            match invoke_owned(host, job, |job, access| job.call(access)) {
                                JobCall::Deferred { job, error } => {
                                    *command_slot = Some(Resident::Promise(job));
                                    return Err(error);
                                }
                                JobCall::Complete(result) => result,
                            }
                        }
                        (ClaimSelection::Generic, Resident::Generic(job)) => {
                            match invoke_owned(host, job, |job, access| job.call(access)) {
                                JobCall::Deferred { job, error } => {
                                    *command_slot = Some(Resident::Generic(job));
                                    return Err(error);
                                }
                                JobCall::Complete(result) => result,
                            }
                        }
                        (_, other) => {
                            *command_slot = Some(other);
                            return queues.present(
                                host,
                                Err(queues
                                    .record_fault(defect(JobFaultCause::StateInconsistency))
                                    .into()),
                            );
                        }
                    };
                    queues.finish_action(slot, host, result)?;
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

#[cfg(test)]
mod fault_tests;

#[cfg(test)]
mod claim_tests;
