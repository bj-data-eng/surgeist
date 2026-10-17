//! The durable session's exact Source beforetoggle dispatch custody.
//!
//! Canonical Source work, adapter work and dispatcher work are independent
//! caller-selected domains. This owner neither runs the Dialog tail nor owns a
//! second canonical operation, listener registry or task loop.
use crate::dom_event::{
    DomEventDispatchError, DomEventDispatchLimits, DomEventDispatchOutcome,
    DomEventDispatchReceipt, DomEventDispatcher, DomEventDom, DomEventType, NativeDomEvent,
    NativeDomEventOwner,
};
use std::{
    cell::RefCell,
    mem::size_of,
    rc::{Rc, Weak},
    sync::atomic::{AtomicBool, Ordering},
};
use surgeist_dom::{
    Dom, DomError, DomOwnerId, DomSourceEventDispatchPermit, DomSourceEventExecutor,
    DomSourceEventId, DomSourceEventOutcome, DomSourceEventPublication, DomSourceEventRequest,
    DomSourceEventRetention, DomSourceLimits,
};

/// Finite persistent custody and cumulative actual work of this adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceEventRuntimeLimits {
    /// Simultaneously retained original event jobs.
    pub jobs: usize,
    /// Fixed owner payload, linked job bodies and event/receipt backing.
    pub storage_bytes: usize,
    /// Cumulative actual adapter work, including one owner-construction step.
    pub work: u64,
}
/// Independent canonical and dispatcher limits for one servicing attempt.
#[derive(Clone, Copy, Debug)]
pub struct DomSourceEventServiceLimits {
    /// Original canonical operation limits, including all current ancestors.
    pub source: DomSourceLimits,
    /// Independent native dispatcher limits for this call.
    pub dispatch: DomEventDispatchLimits,
}
/// The adapter's exact finite admission domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceEventResource {
    /// Simultaneously retained event jobs.
    Jobs,
    /// Actual retained payload, including simultaneously live job nodes.
    StorageBytes,
    /// Cumulative actual adapter work.
    Work,
}
/// A refusal preserves any already retained original dispatch and receipt.
#[derive(Debug)]
pub enum DomSourceEventRuntimeError {
    /// Canonical Source qualification, phase or admission refused the step.
    Dom(DomError),
    /// The actual native dispatcher refused this entry.
    Dispatch(DomEventDispatchError),
    /// The adapter's independent finite limit was reached.
    Admission(DomSourceEventResource),
    /// A policy-only update attempted to lower this owner's existing limit.
    LimitReduction(DomSourceEventResource),
    /// The durable Session owner has been dropped.
    ExpiredOwner,
    /// The view belongs to another canonical owner.
    ForeignOwner,
    /// An executor was already bound to the input owner.
    PreboundSession,
    /// The original event owner is currently borrowed.
    Busy,
    /// An actual receipt or retained phase does not match its original request.
    Protocol,
}
impl std::fmt::Display for DomSourceEventRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source event refused: {self:?}")
    }
}
impl std::error::Error for DomSourceEventRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}
impl From<DomError> for DomSourceEventRuntimeError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}
/// Servicing accepts only the completed event; ordinary resume runs postchecks.
#[derive(Debug)]
pub enum DomSourceEventServiceProgress {
    /// DOM accepted the actual event outcome; its remaining tail is untouched.
    Serviced {
        /// The exact completed move-only dispatcher receipt.
        receipt: DomEventDispatchReceipt,
    },
    /// The original kernel cursor and event remain in the durable Session.
    Retained {
        /// The actual dispatcher suspension reason.
        dispatch: DomEventDispatchOutcome,
    },
}

enum Phase {
    Ready,
    Dispatching {
        permit: Option<DomSourceEventDispatchPermit>,
    },
    Delivered {
        receipt: DomEventDispatchReceipt,
        permit: Option<DomSourceEventDispatchPermit>,
        publication: Option<DomSourceEventPublication>,
    },
}
struct Job {
    next: Option<Rc<RefCell<Job>>>,
    request: DomSourceEventRequest,
    event: NativeDomEventOwner,
    phase: Phase,
    // Reserved before kernel entry, including callbacks which can add jobs.
    payload_reserved: usize,
    // The canonical custody bank owns the affine scope body. This separate
    // Source reservation contains only the event adapter's own payload.
    source_payload_reserved: usize,
    // Every represented value above drops before its original Source debit.
    retained: DomSourceEventRetention,
}
impl Job {
    fn payload_bytes(&self) -> Result<usize, DomSourceEventRuntimeError> {
        let receipt = match &self.phase {
            Phase::Delivered { receipt, .. } => receipt.retained_backing_bytes(),
            _ => 0,
        };
        self.event
            .retained_payload_bytes()
            .and_then(|bytes| bytes.checked_add(receipt))
            .map(|bytes| bytes.max(self.payload_reserved))
            .ok_or(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::StorageBytes,
            ))
    }
}
struct State {
    owner: DomOwnerId,
    executor: DomSourceEventExecutor,
    limits: DomSourceEventRuntimeLimits,
    jobs: Option<Rc<RefCell<Job>>>,
    work: u64,
}
struct Lookup {
    job: Option<Rc<RefCell<Job>>>,
    previous: Option<Rc<RefCell<Job>>>,
    count: usize,
}
impl State {
    fn next_work(
        &self,
        count: usize,
        cancel: &AtomicBool,
    ) -> Result<u64, DomSourceEventRuntimeError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled.into());
        }
        let count = u64::try_from(count)
            .map_err(|_| DomSourceEventRuntimeError::Admission(DomSourceEventResource::Work))?;
        let next = self
            .work
            .checked_add(count)
            .ok_or(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::Work,
            ))?;
        if next > self.limits.work {
            return Err(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::Work,
            ));
        }
        Ok(next)
    }
    fn step(
        &mut self,
        count: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceEventRuntimeError> {
        self.work = self.next_work(count, cancel)?;
        Ok(())
    }
    fn bytes(&mut self, cancel: &AtomicBool) -> Result<usize, DomSourceEventRuntimeError> {
        let mut bytes = Self::fixed_bytes();
        let mut cursor = self.jobs.clone();
        while let Some(job) = cursor {
            self.step(1, cancel)?;
            let row = job.borrow();
            let payload = row.payload_bytes()?;
            bytes = bytes
                .checked_add(size_of::<RefCell<Job>>())
                .and_then(|bytes| bytes.checked_add(payload))
                .ok_or(DomSourceEventRuntimeError::Admission(
                    DomSourceEventResource::StorageBytes,
                ))?;
            cursor = row.next.clone();
        }
        Ok(bytes)
    }
    fn fixed_bytes() -> usize {
        size_of::<RefCell<Self>>() + DomSourceEventExecutor::binding_payload_bytes()
    }
    fn admit(
        &mut self,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceEventRuntimeError> {
        if self
            .bytes(cancel)?
            .checked_add(additional)
            .is_none_or(|bytes| bytes > self.limits.storage_bytes)
        {
            return Err(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::StorageBytes,
            ));
        }
        Ok(())
    }
    fn find(
        &mut self,
        identity: DomSourceEventId,
        cancel: &AtomicBool,
    ) -> Result<Lookup, DomSourceEventRuntimeError> {
        let mut cursor = self.jobs.clone();
        let mut previous = None;
        let mut count = 0usize;
        while let Some(job) = cursor {
            self.step(1, cancel)?;
            count = count
                .checked_add(1)
                .ok_or(DomSourceEventRuntimeError::Admission(
                    DomSourceEventResource::Jobs,
                ))?;
            if job.borrow().request.identity() == identity {
                return Ok(Lookup {
                    job: Some(job),
                    previous,
                    count,
                });
            }
            cursor = job.borrow().next.clone();
            previous = Some(job);
        }
        Ok(Lookup {
            job: None,
            previous,
            count,
        })
    }
    fn job(
        &mut self,
        dom: &Dom,
        request: &DomSourceEventRequest,
        cancel: &AtomicBool,
    ) -> Result<Rc<RefCell<Job>>, DomSourceEventRuntimeError> {
        self.step(1, cancel)?;
        let lookup = self.find(request.identity(), cancel)?;
        if let Some(job) = lookup.job {
            return Ok(job);
        }
        if lookup.count >= self.limits.jobs {
            return Err(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::Jobs,
            ));
        }
        let payload = NativeDomEventOwner::initial_payload_bytes().ok_or(
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::StorageBytes),
        )?;
        let job_bytes = size_of::<RefCell<Job>>().checked_add(payload).ok_or(
            DomSourceEventRuntimeError::Admission(DomSourceEventResource::StorageBytes),
        )?;
        self.admit(job_bytes, cancel)?;
        let retained = dom.retain_source_event_payload(request, job_bytes, 0, cancel)?;
        // One actual formation/link step admits the new original node before
        // allocation. All old nodes and their Source debits remain live.
        self.step(1, cancel)?;
        let job = Rc::new(RefCell::new(Job {
            next: None,
            request: request.clone(),
            event: NativeDomEvent::dialog_before_toggle(
                request.subject(),
                request.old_state(),
                request.new_state(),
                request.cancelable(),
            ),
            phase: Phase::Ready,
            payload_reserved: payload,
            source_payload_reserved: payload,
            retained,
        }));
        if let Some(tail) = lookup.previous {
            tail.borrow_mut().next = Some(job.clone());
        } else {
            self.jobs = Some(job.clone());
        }
        Ok(job)
    }
}
impl Drop for State {
    fn drop(&mut self) {
        // Terminal owner disposal cannot refuse cleanup or recurse through a
        // caller-sized chain. No job borrow survives a callback boundary.
        while let Some(job) = self.jobs.take() {
            self.jobs = job.borrow_mut().next.take();
        }
    }
}

pub(crate) struct SourceEventOwner {
    state: Rc<RefCell<State>>,
}
#[derive(Clone)]
pub(crate) struct SourceEventRuntime {
    state: Weak<RefCell<State>>,
}
impl SourceEventOwner {
    pub(crate) fn admit_initial(
        limits: DomSourceEventRuntimeLimits,
    ) -> Result<(), DomSourceEventRuntimeError> {
        if limits.work == 0 {
            return Err(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::Work,
            ));
        }
        if State::fixed_bytes() > limits.storage_bytes {
            return Err(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::StorageBytes,
            ));
        }
        Ok(())
    }
    pub(crate) fn new(
        owner: DomOwnerId,
        executor: DomSourceEventExecutor,
        limits: DomSourceEventRuntimeLimits,
    ) -> Self {
        Self {
            state: Rc::new(RefCell::new(State {
                owner,
                executor,
                limits,
                jobs: None,
                work: 1,
            })),
        }
    }
    pub(crate) fn execution(&self) -> SourceEventRuntime {
        SourceEventRuntime {
            state: Rc::downgrade(&self.state),
        }
    }
    pub(crate) fn pending(&self) -> Result<Option<DomSourceEventId>, DomSourceEventRuntimeError> {
        let state = self
            .state
            .try_borrow()
            .map_err(|_| DomSourceEventRuntimeError::Busy)?;
        Ok(state
            .jobs
            .as_ref()
            .map(|job| job.borrow().request.identity()))
    }
}
impl SourceEventRuntime {
    pub(crate) fn raise_event_runtime_limits(
        &self,
        dom: &DomEventDom<'_>,
        limits: DomSourceEventRuntimeLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceEventRuntimeError> {
        let original = self
            .state
            .upgrade()
            .ok_or(DomSourceEventRuntimeError::ExpiredOwner)?;
        let mut state = original
            .try_borrow_mut()
            .map_err(|_| DomSourceEventRuntimeError::Busy)?;
        if state.owner != dom.raw().owner() {
            return Err(DomSourceEventRuntimeError::ForeignOwner);
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled.into());
        }
        // Constant policy validation uses the proposed cap so an exhausted
        // original owner can recover. Failed attempts retain their actual work.
        let next = state
            .work
            .checked_add(1)
            .ok_or(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::Work,
            ))?;
        if next > limits.work {
            return Err(DomSourceEventRuntimeError::Admission(
                DomSourceEventResource::Work,
            ));
        }
        state.work = next;
        for (reduced, resource) in [
            (
                limits.jobs < state.limits.jobs,
                DomSourceEventResource::Jobs,
            ),
            (
                limits.storage_bytes < state.limits.storage_bytes,
                DomSourceEventResource::StorageBytes,
            ),
            (
                limits.work < state.limits.work,
                DomSourceEventResource::Work,
            ),
        ] {
            if reduced {
                return Err(DomSourceEventRuntimeError::LimitReduction(resource));
            }
        }
        // Monotonic admission cannot invalidate any already admitted node or
        // callback reservation; no recount or execution is needed here.
        state.limits = limits;
        Ok(())
    }
    pub(crate) fn raise_event_limits(
        &self,
        dom: &mut DomEventDom<'_>,
        request: &DomSourceEventRequest,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomSourceEventRuntimeError> {
        let original = self
            .state
            .upgrade()
            .ok_or(DomSourceEventRuntimeError::ExpiredOwner)?;
        {
            let state = original
                .try_borrow()
                .map_err(|_| DomSourceEventRuntimeError::Busy)?;
            if state.owner != dom.raw().owner() {
                return Err(DomSourceEventRuntimeError::ForeignOwner);
            }
        }
        dom.raw_mut()
            .raise_source_event_limits(request, limits, cancel)?;
        Ok(())
    }
    pub(crate) fn service_event<C>(
        &self,
        dom: &mut DomEventDom<'_>,
        request: &DomSourceEventRequest,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomSourceEventServiceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceEventServiceProgress, DomSourceEventRuntimeError> {
        let original = self
            .state
            .upgrade()
            .ok_or(DomSourceEventRuntimeError::ExpiredOwner)?;
        if original.borrow().owner != dom.raw().owner() {
            return Err(DomSourceEventRuntimeError::ForeignOwner);
        }
        // Refresh the exact original top operation's caller limits without
        // advancing its cursor or entering the unfulfilled Dialog tail.
        drop(
            dom.raw_mut()
                .resume_source_edit(request.operation(), limits.source, cancel)?,
        );
        let job = original.borrow_mut().job(dom.raw(), request, cancel)?;
        let delivered = matches!(job.borrow().phase, Phase::Delivered { .. });
        if !delivered {
            let (event, scope) = {
                let mut state = original.borrow_mut();
                state.step(1, cancel)?;
                let maximum = NativeDomEventOwner::maximum_payload_bytes(limits.dispatch).ok_or(
                    DomSourceEventRuntimeError::Admission(DomSourceEventResource::StorageBytes),
                )?;
                let source_maximum =
                    NativeDomEventOwner::maximum_source_payload_bytes(limits.dispatch)
                        .ok_or(DomSourceEventRuntimeError::Admission(
                            DomSourceEventResource::StorageBytes,
                        ))?
                        .max(job.borrow().event.retained_source_payload_bytes().ok_or(
                            DomSourceEventRuntimeError::Admission(
                                DomSourceEventResource::StorageBytes,
                            ),
                        )?);
                let previous = job.borrow().payload_reserved;
                if maximum > previous {
                    state.admit(maximum - previous, cancel)?;
                }
                let source_previous = job.borrow().source_payload_reserved;
                if source_maximum > source_previous {
                    let retained = dom.raw().retain_source_event_payload(
                        request,
                        source_maximum - source_previous,
                        0,
                        cancel,
                    )?;
                    let mut row = job.borrow_mut();
                    row.retained.absorb(retained)?;
                    row.source_payload_reserved = source_maximum;
                }
                job.borrow_mut().payload_reserved = maximum.max(previous);
                if matches!(job.borrow().phase, Phase::Ready) {
                    let permit = state.executor.begin_dispatch(dom.raw(), request, cancel)?;
                    job.borrow_mut().phase = Phase::Dispatching {
                        permit: Some(permit),
                    };
                }
                let row = job.borrow();
                let Phase::Dispatching {
                    permit: Some(permit),
                } = &row.phase
                else {
                    return Err(DomSourceEventRuntimeError::Protocol);
                };
                let scope = state.executor.enter_dispatch(dom.raw(), permit, cancel)?;
                (std::ops::Deref::deref(&row.event).clone(), scope)
            };
            let outcome = {
                let _scope = scope;
                if event.has_started() {
                    dispatcher.resume(dom.raw_mut(), context, &event, limits.dispatch)
                } else {
                    dispatcher.dispatch(dom.raw_mut(), context, &event, limits.dispatch)
                }
            }
            .map_err(DomSourceEventRuntimeError::Dispatch)?;
            match outcome {
                DomEventDispatchOutcome::Complete(receipt) => {
                    // No fallible lookup, context access or admission separates
                    // the actual receipt from its original durable job.
                    let mut row = job.borrow_mut();
                    let Phase::Dispatching { permit } =
                        std::mem::replace(&mut row.phase, Phase::Ready)
                    else {
                        unreachable!("actual retained dispatch");
                    };
                    row.phase = Phase::Delivered {
                        receipt,
                        permit,
                        publication: None,
                    };
                }
                dispatch => return Ok(DomSourceEventServiceProgress::Retained { dispatch }),
            }
        }
        let mut state = original.borrow_mut();
        let selected = state.find(request.identity(), cancel)?;
        assert!(
            selected
                .job
                .as_ref()
                .is_some_and(|actual| Rc::ptr_eq(actual, &job)),
            "original durable job"
        );
        let cleanup_work = state.next_work(1, cancel)?;
        {
            let mut row = job.borrow_mut();
            let Phase::Delivered {
                receipt,
                permit,
                publication,
            } = &mut row.phase
            else {
                return Err(DomSourceEventRuntimeError::Protocol);
            };
            if receipt.original_target() != request.subject()
                || receipt.event_type() != DomEventType::BeforeToggle
            {
                return Err(DomSourceEventRuntimeError::Protocol);
            }
            if publication.is_none() {
                let outcome = if receipt.default_prevented() {
                    DomSourceEventOutcome::Canceled
                } else {
                    DomSourceEventOutcome::NotCanceled
                };
                *publication = Some(state.executor.seal_completed(
                    dom.raw(),
                    permit,
                    outcome,
                    cancel,
                )?);
            }
        }
        // Admission refusal retains both the actual receipt and its sealed
        // publication; retry only acceptance, never listeners or the Show tail.
        {
            let mut row = job.borrow_mut();
            let Phase::Delivered { publication, .. } = &mut row.phase else {
                unreachable!();
            };
            let mut lease =
                dom.raw_mut()
                    .resume_source_edit(request.operation(), limits.source, cancel)?;
            lease.accept_source_event(request, publication, cancel)?;
        }
        // Acceptance runs no callback. Its admitted pointer moves cannot lose
        // the receipt to a subsequent cancellation or adapter-work refusal.
        state.work = cleanup_work;
        let next = job.borrow_mut().next.take();
        if let Some(previous) = selected.previous {
            previous.borrow_mut().next = next;
        } else {
            state.jobs = next;
        }
        let phase = std::mem::replace(&mut job.borrow_mut().phase, Phase::Ready);
        let Phase::Delivered { receipt, .. } = phase else {
            unreachable!();
        };
        Ok(DomSourceEventServiceProgress::Serviced { receipt })
    }
}

#[cfg(test)]
mod payload_tests {
    use super::*;
    use crate::{
        DomEventCallback, DomEventCallbackError, DomEventListenerFailure, DomEventListenerOptions,
        DomEventRegistryLimits, DomEventResource,
    };
    use surgeist_dom::{
        DomBatch, DomDispatchCustodyLimits, DomLimits, DomSourceEdit, DomSourcePhase,
        DomSourceProgress, DomSourceRequest, Namespace,
    };

    #[test]
    fn retained_path_and_growing_failures_refuse_before_callbacks_until_owner_storage_fits() {
        const LISTENERS: usize = 64;
        let cancel = AtomicBool::new(false);
        let source = DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 4);
        let first = DomSourceEventServiceLimits {
            source,
            dispatch: DomEventDispatchLimits {
                path_items: 3,
                snapshot_listeners: LISTENERS,
                failure_records: 1,
                callbacks: 1,
                work: 100_000,
            },
        };
        let retry = DomSourceEventServiceLimits {
            source,
            dispatch: DomEventDispatchLimits {
                path_items: 0,
                failure_records: LISTENERS,
                callbacks: LISTENERS,
                ..first.dispatch
            },
        };
        let fixture = |storage_bytes| {
            let mut dom = Dom::new(
                DomLimits::new(128, 100_000, 256, 128, 128, 32, 4_000_000)
                    .unwrap()
                    .with_dispatch_custody(
                        DomDispatchCustodyLimits::new(64, 4096, 16_000_000, 16_000_000, 4_000_000)
                            .unwrap(),
                    ),
            )
            .unwrap();
            let mut batch = DomBatch::new();
            let document = batch.create_document();
            let parent = batch.create_element_ns(&document, Namespace::html(), "div");
            let dialog = batch.create_element_ns(&document, Namespace::html(), "dialog");
            batch.insert_before(&parent, &document, None);
            batch.insert_before(&dialog, &parent, None);
            let commit = dom.apply(batch, &cancel).unwrap();
            let subject = commit.created(&dialog).unwrap().unwrap();
            let executor = dom.bind_source_event_executor(&cancel).unwrap();
            let policy = DomSourceEventRuntimeLimits {
                jobs: 1,
                storage_bytes,
                work: 1_000_000,
            };
            SourceEventOwner::admit_initial(policy).unwrap();
            let owner = SourceEventOwner::new(dom.owner(), executor, policy);
            let runtime = owner.execution();
            let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
                registrations: LISTENERS,
                pending_events: 1,
                nested_dispatches: 1,
            })
            .unwrap();
            for index in 0..LISTENERS {
                dispatcher
                    .add_listener(
                        &DomEventDom::borrow(&mut dom),
                        subject,
                        DomEventType::BeforeToggle,
                        DomEventCallback::new(move |_, _, calls: &mut Vec<usize>, event| {
                            calls.push(index);
                            event.prevent_default();
                            Err(DomEventCallbackError { code: 91 })
                        }),
                        DomEventListenerOptions {
                            capture: false,
                            once: false,
                        },
                    )
                    .unwrap();
            }
            let request = {
                let mut lease = dom
                    .begin_source_edit(DomSourceEdit::show_dialog(subject), source, &cancel)
                    .unwrap();
                let DomSourceProgress::Request(DomSourceRequest::Event(request)) =
                    lease.advance(&cancel).unwrap()
                else {
                    panic!("the actual Show beforetoggle request is required");
                };
                request
            };
            (dom, owner, runtime, dispatcher, request)
        };

        // Measure actual allocations from a real retained event, not the owner's
        // reservation counter or the kernel's maximum-payload estimate. The
        // first failure limit is one, so successful callback 1 and retention
        // establish exactly one allocated failure slot. Resume preserves its
        // frozen path, scope and listener snapshot while failures grow to 64.
        let required = {
            let (mut dom, owner, runtime, mut dispatcher, request) = fixture(16_000_000);
            let mut calls = Vec::new();
            assert!(matches!(
                runtime
                    .service_event(
                        &mut DomEventDom::borrow(&mut dom),
                        &request,
                        &mut dispatcher,
                        &mut calls,
                        first,
                        &cancel,
                    )
                    .unwrap(),
                DomSourceEventServiceProgress::Retained {
                    dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
                }
            ));
            assert_eq!(calls, [0]);
            let job = owner.state.borrow().jobs.clone().unwrap();
            let actual = job.borrow().event.retained_payload_bytes().unwrap();
            size_of::<RefCell<State>>()
                + DomSourceEventExecutor::binding_payload_bytes()
                + size_of::<RefCell<Job>>()
                + actual
                + (LISTENERS - 1) * size_of::<DomEventListenerFailure>()
        };
        let tight = required - 1;
        let (mut dom, owner, runtime, mut dispatcher, request) = fixture(tight);
        let mut calls = Vec::new();
        assert!(matches!(
            runtime
                .service_event(
                    &mut DomEventDom::borrow(&mut dom),
                    &request,
                    &mut dispatcher,
                    &mut calls,
                    first,
                    &cancel,
                )
                .unwrap(),
            DomSourceEventServiceProgress::Retained {
                dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks)
            }
        ));
        assert_eq!(calls, [0]);
        let original_job = owner.state.borrow().jobs.clone().unwrap();
        let retained_before = original_job
            .borrow()
            .event
            .retained_payload_bytes()
            .unwrap();
        let stamp = dom.snapshot().stamp();
        let attempt = runtime.service_event(
            &mut DomEventDom::borrow(&mut dom),
            &request,
            &mut dispatcher,
            &mut calls,
            retry,
            &cancel,
        );
        assert!(
            matches!(
                attempt,
                Err(DomSourceEventRuntimeError::Admission(
                    DomSourceEventResource::StorageBytes
                ))
            ),
            "the frozen path/scope and 64 failure slots need {required} bytes, \
             exceeding owner storage {tight}; retry must refuse before callback 2, \
             but returned {attempt:?} after {} callbacks",
            calls.len()
        );
        assert_eq!(calls, [0]);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert!(Rc::ptr_eq(
            owner.state.borrow().jobs.as_ref().unwrap(),
            &original_job
        ));
        assert_eq!(
            original_job
                .borrow()
                .event
                .retained_payload_bytes()
                .unwrap(),
            retained_before
        );
        assert!(matches!(
            original_job.borrow().phase,
            Phase::Dispatching { .. }
        ));

        // Allow both the retained failure bank and a full replacement bank at
        // once. This independently sufficient peak also fits frozen custody.
        runtime
            .raise_event_runtime_limits(
                &DomEventDom::borrow(&mut dom),
                DomSourceEventRuntimeLimits {
                    jobs: 1,
                    storage_bytes: required + LISTENERS * size_of::<DomEventListenerFailure>(),
                    work: 1_000_000,
                },
                &cancel,
            )
            .unwrap();
        let DomSourceEventServiceProgress::Serviced { receipt } = runtime
            .service_event(
                &mut DomEventDom::borrow(&mut dom),
                &request,
                &mut dispatcher,
                &mut calls,
                retry,
                &cancel,
            )
            .unwrap()
        else {
            panic!("the original retained event must finish after honest admission");
        };
        assert_eq!(calls, (0..LISTENERS).collect::<Vec<_>>());
        assert_eq!(receipt.callbacks(), LISTENERS);
        assert_eq!(receipt.failures().len(), LISTENERS);
        assert_eq!(receipt.original_target(), request.subject());
        assert!(receipt.default_prevented());
        assert!(owner.state.borrow().jobs.is_none());
        let mut lease = dom
            .resume_source_edit(request.operation(), source, &cancel)
            .unwrap();
        assert_eq!(lease.status().phase(), DomSourcePhase::EventReceiptAccepted);
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            DomSourceProgress::Complete(_)
        ));
        assert_eq!(calls.len(), LISTENERS);
    }
}
