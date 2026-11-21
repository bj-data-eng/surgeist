//! Native AbortController and its original standalone signal.

use super::invocation::SignalAbortFiring;
use super::*;
use crate::internal::dom_lifecycle::{
    AbortEventClock, AdmittedAbortEventClock, DomHostedRuntimeError, EventTargetAssociation,
};
use std::{any::Any, cell::RefMut, fmt};
use surgeist_dom::hosted::DomHostedGlobalId;

/// An explicit native abort value. Object values retain their original shared allocation.
#[derive(Clone)]
pub enum DomAbortValue {
    /// The explicit null value.
    Null,
    /// An opaque native value with its original shared identity.
    ///
    /// The caller owns the value's reference graph. Use weak backreferences
    /// inside a value that points back to its abort signal.
    Native(Rc<dyn Any>),
}

impl fmt::Debug for DomAbortValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("Null"),
            Self::Native(_) => f.debug_tuple("Native").finish(),
        }
    }
}

/// One identity-bearing default AbortError in its signal's relevant Global.
#[derive(Clone)]
pub struct DomAbortDefaultReason(Rc<DefaultReason>);

struct DefaultReason {
    global: DomHostedGlobalId,
    _funding: EventAllocation,
}

impl DomAbortDefaultReason {
    /// The authenticated Global that owns this default reason.
    pub fn global(&self) -> DomHostedGlobalId {
        self.0.global
    }
}

impl PartialEq for DomAbortDefaultReason {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DomAbortDefaultReason {}

impl fmt::Debug for DomAbortDefaultReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomAbortDefaultReason")
            .field("global", &self.global())
            .finish_non_exhaustive()
    }
}

/// The exact reason established at the original abort transition.
#[derive(Clone, Debug)]
pub enum DomAbortReason {
    /// The reason created when the controller supplied no value.
    AbortError(DomAbortDefaultReason),
    /// The exact value supplied by the controller.
    Explicit(DomAbortValue),
}

/// The listener occurrence accepted by a signal-aware registration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSignalListenerRegistration {
    /// A new listener occurrence was inserted and captured by the signal.
    Inserted(DomEventRegistrationId),
    /// The existing occurrence was preserved. The signal owns its detached step.
    Existing(DomEventRegistrationId),
    /// The signal was already aborted and no listener was added.
    SignalAborted,
}

/// Outcome from the original native abort call.
#[derive(Debug)]
pub enum DomAbortProgress {
    /// The original signal transition already occurred.
    AlreadyAborted,
    /// Progress of the original trusted abort event.
    Delivery(DomEventDispatchOutcome),
}

/// Native abort refusal or failure to qualify the original firing clock.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DomAbortError {
    /// The canonical event dispatcher refused the operation.
    Event(DomEventDispatchError),
    /// The original hosted firing authority refused the operation.
    Hosted(DomHostedRuntimeError),
    /// A reentrant operation already has exclusive access to the signal.
    Busy,
    /// This native listener already owes completion of a different abort.
    EnclosingAbortPending,
}

impl From<DomEventDispatchError> for DomAbortError {
    fn from(error: DomEventDispatchError) -> Self {
        Self::Event(error)
    }
}

impl From<DomHostedRuntimeError> for DomAbortError {
    fn from(error: DomHostedRuntimeError) -> Self {
        Self::Hosted(error)
    }
}

impl fmt::Display for DomAbortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Event(_) => f.write_str("native abort delivery was refused"),
            Self::Hosted(_) => f.write_str("the signal's original hosted clock was unavailable"),
            Self::Busy => f.write_str("the native abort signal is borrowed"),
            Self::EnclosingAbortPending => {
                f.write_str("this native listener already owes an abort delivery")
            }
        }
    }
}

impl std::error::Error for DomAbortError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Event(error) => Some(error),
            Self::Hosted(error) => Some(error),
            Self::Busy | Self::EnclosingAbortPending => None,
        }
    }
}

/// A native controller for its one original signal.
pub struct DomAbortController<C> {
    signal: DomAbortSignal<C>,
}

impl<C> fmt::Debug for DomAbortController<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomAbortController")
            .field("signal", &self.signal)
            .finish()
    }
}

impl<C> DomAbortController<C> {
    /// The same native signal on each observation.
    pub fn signal(&self) -> DomAbortSignal<C> {
        self.signal.clone()
    }

    /// Establish the reason, remove the selected listeners and drive the trusted event.
    ///
    /// A missing value creates the native default AbortError. After the transition
    /// the signal retains the original delivery until it completes or is abandoned.
    /// If an error arrives after the signal became aborted, resume that delivery
    /// through the signal or the enclosing dispatch.
    pub fn abort(
        &self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        reason: Option<DomAbortValue>,
        limits: DomEventDispatchLimits,
        cancel: &AtomicBool,
    ) -> Result<DomAbortProgress, DomAbortError> {
        let signal = &self.signal.0;
        dispatcher.association.validate(dom.raw())?;
        dispatcher.validate_target(dom.raw(), &signal.target.clone().into(), cancel)?;
        let mut state = signal
            .state
            .try_borrow_mut()
            .map_err(|_| DomAbortError::Busy)?;
        let SignalState::Live(algorithms) = &*state else {
            return Ok(DomAbortProgress::AlreadyAborted);
        };
        let parent = dispatcher
            .active_abort_callback
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
            .filter(|custody| custody.native_callback.get());
        if parent
            .as_ref()
            .is_some_and(|custody| custody.retained_abort().is_some())
        {
            return Err(DomAbortError::EnclosingAbortPending);
        }
        dispatcher.admit_running()?;
        let work = u64::try_from(algorithms.steps.len())
            .ok()
            .and_then(|steps| steps.checked_add(1))
            .filter(|&work| work <= limits.work)
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Work))?;
        let bytes = size_of::<AbortDelivery>();
        let funding = dispatcher.storage.reserve(dom, bytes, bytes, 0, cancel)?;
        let default_funding = if reason.is_none() {
            let bytes = size_of::<DefaultReason>();
            Some(dispatcher.storage.reserve(dom, bytes, bytes, 0, cancel)?)
        } else {
            None
        };
        let firing = dispatcher.admit_signal_abort(dom, cancel)?;
        let clock = signal.clock.admit(dom, cancel)?;
        dispatcher.bind(dom, cancel)?;
        let reason = match (reason, default_funding) {
            (Some(value), _) => DomAbortReason::Explicit(value),
            (None, Some(funding)) => {
                DomAbortReason::AbortError(DomAbortDefaultReason(Rc::new(DefaultReason {
                    global: signal.target.relevant_global(),
                    _funding: funding,
                })))
            }
            (None, None) => return Err(DomEventDispatchError::WrongContinuation.into()),
        };
        let delivery = Rc::new(AbortDelivery {
            target: signal.target.clone(),
            phase: RefCell::new(DeliveryPhase::AwaitingEvent(AbortFiring {
                clock,
                event: Some(firing),
            })),
            receipt: Cell::new(None),
            nested: parent.is_some(),
            _funding: funding,
        });
        if let Some(parent) = parent {
            parent.abort_tail.replace(Some(delivery.clone()));
        }
        let previous = std::mem::replace(
            &mut *state,
            SignalState::Aborted {
                reason,
                delivery: delivery.clone(),
            },
        );
        drop(state);
        let SignalState::Live(algorithms) = previous else {
            return Err(DomEventDispatchError::WrongContinuation.into());
        };
        for algorithm in &algorithms.steps {
            if let Some(registration) = algorithm.registration {
                dispatcher.remove_listener(registration);
            }
        }
        drop(algorithms);
        let mut usage = DispatchUsage {
            work,
            ..DispatchUsage::default()
        };
        let outcome = delivery.advance(dom, dispatcher, context, (limits, &mut usage), cancel)?;
        Ok(DomAbortProgress::Delivery(outcome))
    }
}

/// Stable signal identity associated with the original dispatcher and Global.
pub struct DomAbortSignal<C>(Rc<SignalBody<C>>);

impl<C> Clone for DomAbortSignal<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C> PartialEq for DomAbortSignal<C> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<C> Eq for DomAbortSignal<C> {}

impl<C> fmt::Debug for DomAbortSignal<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomAbortSignal")
            .field("target", &self.0.target)
            .finish_non_exhaustive()
    }
}

impl<C> DomAbortSignal<C> {
    /// The signal's actual standalone EventTarget.
    pub fn event_target(&self) -> DomStandaloneEventTarget {
        self.0.target.clone()
    }

    /// Observe this signal's original abort state.
    pub fn aborted(&self) -> Result<bool, DomAbortError> {
        let state = self.0.state.try_borrow().map_err(|_| DomAbortError::Busy)?;
        Ok(matches!(*state, SignalState::Aborted { .. }))
    }

    /// Observe the original reason, or absence before abort.
    pub fn reason(&self) -> Result<Option<DomAbortReason>, DomAbortError> {
        let state = self.0.state.try_borrow().map_err(|_| DomAbortError::Busy)?;
        Ok(match &*state {
            SignalState::Live(_) => None,
            SignalState::Aborted { reason, .. } => Some(reason.clone()),
        })
    }

    /// Advance this signal's original, already-accepted standalone delivery.
    ///
    /// A suspended abort started inside a native listener resumes through the
    /// enclosing dispatch. Its completed receipt is available through
    /// [Self::take_abort_receipt].
    pub fn resume_abort(
        &self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        limits: DomEventDispatchLimits,
        cancel: &AtomicBool,
    ) -> Result<DomEventDispatchOutcome, DomAbortError> {
        let delivery = {
            let state = self.0.state.try_borrow().map_err(|_| DomAbortError::Busy)?;
            let SignalState::Aborted { delivery, .. } = &*state else {
                return Err(DomEventDispatchError::WrongContinuation.into());
            };
            delivery.clone()
        };
        if delivery.nested {
            return Err(DomEventDispatchError::WrongContinuation.into());
        }
        delivery.advance(
            dom,
            dispatcher,
            context,
            (limits, &mut DispatchUsage::default()),
            cancel,
        )
    }

    /// Take the receipt of a suspended abort completed by its enclosing dispatch.
    ///
    /// An abort that completed in its original call returns its receipt directly.
    pub fn take_abort_receipt(&self) -> Result<Option<DomEventDispatchReceipt>, DomAbortError> {
        let state = self.0.state.try_borrow().map_err(|_| DomAbortError::Busy)?;
        Ok(match &*state {
            SignalState::Live(_) => None,
            SignalState::Aborted { delivery, .. } => delivery.receipt.take(),
        })
    }

    pub(super) fn listener_algorithms(
        &self,
    ) -> Result<Option<SignalAlgorithms<'_, C>>, DomEventDispatchError> {
        let state = self
            .0
            .state
            .try_borrow_mut()
            .map_err(|_| DomEventDispatchError::WrongContinuation)?;
        let Ok(algorithms) = RefMut::filter_map(state, |state| match state {
            SignalState::Live(algorithms) => Some(algorithms),
            SignalState::Aborted { .. } => None,
        }) else {
            return Ok(None);
        };
        Ok(Some(SignalAlgorithms {
            algorithms,
            limit: self.0.algorithm_limit,
        }))
    }
}

struct SignalBody<C> {
    target: DomStandaloneEventTarget,
    clock: AbortEventClock,
    algorithm_limit: usize,
    state: RefCell<SignalState<C>>,
    funding: Rc<EventAllocation>,
}

enum SignalState<C> {
    Live(Algorithms<C>),
    Aborted {
        reason: DomAbortReason,
        delivery: Rc<AbortDelivery>,
    },
}

struct Algorithms<C> {
    steps: Vec<Algorithm<C>>,
    funding: Option<EventAllocation>,
}

struct Algorithm<C> {
    registration: Option<DomEventRegistrationId>,
    _target: DomEventTarget,
    _type: DomEventType,
    _callback: DomEventCallback<C>,
}

pub(super) struct SignalAlgorithms<'signal, C> {
    algorithms: RefMut<'signal, Algorithms<C>>,
    limit: usize,
}

impl<'signal, C> SignalAlgorithms<'signal, C> {
    pub(super) fn admit(
        self,
        dom: &mut DomEventDom<'_>,
        storage: &EventStorage,
        cancel: &AtomicBool,
    ) -> Result<AdmittedSignalAlgorithm<'signal, C>, DomEventDispatchError> {
        let steps = &self.algorithms.steps;
        if steps.len() == self.limit {
            return Err(DomEventDispatchError::Admission(
                DomEventResource::Registrations,
            ));
        }
        storage.step(1, cancel)?;
        if steps.len() < steps.capacity() {
            return Ok(AdmittedSignalAlgorithm {
                algorithms: self.algorithms,
                replacement: None,
            });
        }
        let capacity = steps.len().saturating_mul(2).max(1).min(self.limit);
        let bytes = capacity
            .checked_mul(size_of::<Algorithm<C>>())
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        storage.step(steps.len(), cancel)?;
        let funding = storage.reserve(dom, bytes, bytes, 0, cancel)?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(capacity)
            .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
        if replacement.capacity() != capacity {
            return Err(DomEventDispatchError::Admission(DomEventResource::Storage));
        }
        Ok(AdmittedSignalAlgorithm {
            algorithms: self.algorithms,
            replacement: Some((replacement, funding)),
        })
    }
}

pub(super) struct AdmittedSignalAlgorithm<'signal, C> {
    algorithms: RefMut<'signal, Algorithms<C>>,
    replacement: Option<(Vec<Algorithm<C>>, EventAllocation)>,
}

impl<C> AdmittedSignalAlgorithm<'_, C> {
    pub(super) fn publish(
        mut self,
        registration: Option<DomEventRegistrationId>,
        target: DomEventTarget,
        event_type: DomEventType,
        callback: DomEventCallback<C>,
    ) {
        if let Some((mut steps, funding)) = self.replacement.take() {
            steps.append(&mut self.algorithms.steps);
            drop(std::mem::replace(&mut self.algorithms.steps, steps));
            self.algorithms.funding = Some(funding);
        }
        self.algorithms.steps.push(Algorithm {
            registration,
            _target: target,
            _type: event_type,
            _callback: callback,
        });
    }
}

pub(super) struct AbortDelivery {
    target: DomStandaloneEventTarget,
    phase: RefCell<DeliveryPhase>,
    receipt: Cell<Option<DomEventDispatchReceipt>>,
    nested: bool,
    _funding: EventAllocation,
}

enum DeliveryPhase {
    AwaitingEvent(AbortFiring),
    Prepared(DomEventDispatch),
    Entered,
    Complete,
    Abandoned,
}

struct AbortFiring {
    clock: AdmittedAbortEventClock,
    event: Option<SignalAbortFiring>,
}

impl AbortDelivery {
    pub(super) fn retain_nested_receipt(&self, receipt: DomEventDispatchReceipt) {
        self.receipt.set(Some(receipt));
    }

    pub(super) fn is_pending(&self) -> bool {
        self.phase.try_borrow().map_or(true, |phase| {
            !matches!(*phase, DeliveryPhase::Complete | DeliveryPhase::Abandoned)
        })
    }

    pub(super) fn abandon(&self) {
        if let Ok(mut phase) = self.phase.try_borrow_mut()
            && !matches!(*phase, DeliveryPhase::Complete | DeliveryPhase::Abandoned)
        {
            let old = std::mem::replace(&mut *phase, DeliveryPhase::Abandoned);
            drop(phase);
            drop(old);
        }
    }

    pub(super) fn advance<C>(
        &self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &mut DomEventDispatcher<C>,
        context: &mut C,
        budget: (DomEventDispatchLimits, &mut DispatchUsage),
        cancel: &AtomicBool,
    ) -> Result<DomEventDispatchOutcome, DomAbortError> {
        dispatcher.association.validate(dom.raw())?;
        dispatcher.validate_target(dom.raw(), &self.target.clone().into(), cancel)?;
        let mut state = self
            .phase
            .try_borrow_mut()
            .map_err(|_| DomAbortError::Busy)?;
        match &*state {
            DeliveryPhase::Entered => return Err(DomEventDispatchError::AlreadyDispatching.into()),
            DeliveryPhase::Complete => return Err(DomEventDispatchError::AlreadyCompleted.into()),
            DeliveryPhase::Abandoned => return Err(DomEventDispatchError::Abandoned.into()),
            DeliveryPhase::AwaitingEvent(_) | DeliveryPhase::Prepared(_) => {}
        }
        let mut drive = DeliveryDrive {
            stored: &self.phase,
            phase: std::mem::replace(&mut *state, DeliveryPhase::Entered),
        };
        drop(state);
        let result = (|| {
            if let DeliveryPhase::AwaitingEvent(firing) = &mut drive.phase {
                let creation = firing.clock.sample(dom)?;
                let event = firing
                    .event
                    .take()
                    .ok_or(DomEventDispatchError::WrongContinuation)?;
                drive.phase = DeliveryPhase::Prepared(event.prepare(self.target.clone(), creation));
            }
            let DeliveryPhase::Prepared(dispatch) = &mut drive.phase else {
                return Err(DomEventDispatchError::WrongContinuation.into());
            };
            Ok(dispatch.advance_with_host_usage(
                dom,
                dispatcher,
                context,
                budget,
                cancel,
                DispatchHost::Unavailable,
            )?)
        })();
        if matches!(result, Ok(DomEventDispatchOutcome::Complete(_))) {
            drive.phase = DeliveryPhase::Complete;
        }
        if result.is_ok() {
            let stored = self.phase.try_borrow().map_err(|_| DomAbortError::Busy)?;
            if matches!(*stored, DeliveryPhase::Abandoned) {
                return Err(DomEventDispatchError::Abandoned.into());
            }
        }
        result
    }
}

/// Restore the current affine phase even if an internal dispatch edge unwinds.
/// Registry teardown can abandon an entered drive without borrowing its frame.
struct DeliveryDrive<'a> {
    stored: &'a RefCell<DeliveryPhase>,
    phase: DeliveryPhase,
}

impl Drop for DeliveryDrive<'_> {
    fn drop(&mut self) {
        if let Ok(mut stored) = self.stored.try_borrow_mut()
            && matches!(*stored, DeliveryPhase::Entered)
        {
            *stored = std::mem::replace(&mut self.phase, DeliveryPhase::Abandoned);
        }
    }
}

pub(super) struct SignalBook<C> {
    signals: Vec<SignalEntry<C>>,
    funding: Option<EventAllocation>,
}

impl<C> SignalBook<C> {
    pub(super) fn new() -> Self {
        Self {
            signals: Vec::new(),
            funding: None,
        }
    }

    fn admit_slot(
        &mut self,
        dom: &mut DomEventDom<'_>,
        storage: &EventStorage,
        cancel: &AtomicBool,
    ) -> Result<SignalSlot<C>, DomEventDispatchError> {
        for (index, entry) in self.signals.iter().enumerate() {
            storage.step(1, cancel)?;
            if entry.signal.strong_count() == 0 {
                self.signals.swap_remove(index);
                return Ok(SignalSlot::Spare);
            }
        }
        let len = self.signals.len();
        if len < self.signals.capacity() {
            return Ok(SignalSlot::Spare);
        }
        let capacity = len
            .checked_mul(2)
            .map(|capacity| capacity.max(1))
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        let bytes = capacity
            .checked_mul(size_of::<SignalEntry<C>>())
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        storage.step(len, cancel)?;
        let funding = storage.reserve(dom, bytes, bytes, 0, cancel)?;
        let mut signals = Vec::new();
        signals
            .try_reserve_exact(capacity)
            .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
        if signals.capacity() != capacity {
            return Err(DomEventDispatchError::Admission(DomEventResource::Storage));
        }
        Ok(SignalSlot::Grow { signals, funding })
    }

    fn publish(&mut self, slot: SignalSlot<C>, signal: &Rc<SignalBody<C>>) {
        if let SignalSlot::Grow {
            mut signals,
            funding,
        } = slot
        {
            signals.append(&mut self.signals);
            drop(std::mem::replace(&mut self.signals, signals));
            self.funding = Some(funding);
        }
        self.signals.push(SignalEntry {
            signal: Rc::downgrade(signal),
            _funding: signal.funding.clone(),
        });
    }

    pub(super) fn retire(&mut self) {
        for entry in self.signals.drain(..) {
            let Some(signal) = entry.signal.upgrade() else {
                continue;
            };
            let Ok(mut state) = signal.state.try_borrow_mut() else {
                continue;
            };
            match &mut *state {
                SignalState::Live(algorithms) => {
                    let steps = std::mem::take(&mut algorithms.steps);
                    let funding = algorithms.funding.take();
                    drop(state);
                    drop(steps);
                    drop(funding);
                }
                SignalState::Aborted { delivery, .. } => {
                    let delivery = delivery.clone();
                    drop(state);
                    delivery.abandon();
                }
            }
        }
    }
}

enum SignalSlot<C> {
    Spare,
    Grow {
        signals: Vec<SignalEntry<C>>,
        funding: EventAllocation,
    },
}

/// Track the original signals without extending their public lifetimes.
/// The weak allocation remains funded until the corresponding entry retires.
struct SignalEntry<C> {
    signal: Weak<SignalBody<C>>,
    _funding: Rc<EventAllocation>,
}

impl<C> DomEventDispatcher<C> {
    pub(crate) fn create_abort_controller(
        &mut self,
        dom: &mut DomEventDom<'_>,
        association: EventTargetAssociation,
        clock: AbortEventClock,
        algorithm_limit: usize,
        cancel: &AtomicBool,
    ) -> Result<DomAbortController<C>, DomEventDispatchError> {
        let slot = self.abort_signals.admit_slot(dom, &self.storage, cancel)?;
        let bytes = size_of::<SignalBody<C>>() + size_of::<EventAllocation>();
        let funding = self.storage.reserve(dom, bytes, bytes, 0, cancel)?;
        let target = self.create_standalone_target(dom, association, cancel)?;
        let signal = Rc::new(SignalBody {
            target,
            clock,
            algorithm_limit,
            state: RefCell::new(SignalState::Live(Algorithms {
                steps: Vec::new(),
                funding: None,
            })),
            funding: Rc::new(funding),
        });
        self.abort_signals.publish(slot, &signal);
        Ok(DomAbortController {
            signal: DomAbortSignal(signal),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::dom_event::tests::hosted_style_parent_tests::hosted_session;

    #[test]
    fn refused_signal_creation_releases_provisional_index_and_body_backing() {
        let cancel = AtomicBool::new(false);
        let mut session = hosted_session();
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        let global = hosted
            .create_top_level(&mut dom, host, &cancel)
            .unwrap()
            .global();
        let registry = |storage_bytes| {
            DomEventDispatcher::<()>::new(DomEventRegistryLimits {
                registrations: 0,
                pending_events: 0,
                nested_dispatches: 0,
                storage_bytes,
                work: 100,
            })
            .unwrap()
        };

        let probe = registry(16_000_000);
        let target = hosted
            .create_event_target(&mut dom, &probe, global, &cancel)
            .unwrap();
        let target_bytes = target.inspect_allocations().0;
        drop(target);
        let header = probe.storage.used_bytes();
        let required = target_bytes
            + size_of::<SignalBody<()>>()
            + size_of::<EventAllocation>()
            + size_of::<SignalEntry<()>>();
        let mut short = registry(header + required - 1);
        assert!(matches!(
            hosted.create_abort_controller(&mut dom, &mut short, global, 0, &cancel),
            Err(DomEventConstructionError::Event(
                DomEventDispatchError::Admission(DomEventResource::Storage)
            ))
        ));
        assert_eq!(short.storage.used_bytes(), header);

        let mut exact = registry(header + required);
        let controller = hosted
            .create_abort_controller(&mut dom, &mut exact, global, 0, &cancel)
            .unwrap();
        assert_eq!(exact.storage.used_bytes(), header + required);
        drop(controller);
        let replacement = hosted
            .create_abort_controller(&mut dom, &mut exact, global, 0, &cancel)
            .unwrap();
        assert_eq!(exact.storage.used_bytes(), header + required);
        let storage = exact.storage.clone();
        drop(exact);
        assert_eq!(
            storage.used_bytes(),
            header + required - size_of::<SignalEntry<()>>()
        );
        drop(replacement);
        assert_eq!(storage.used_bytes(), header);
    }
}
