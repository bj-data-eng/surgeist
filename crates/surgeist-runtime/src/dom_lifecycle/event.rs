use super::*;
use crate::dom_event::{
    DomEventConstructionError, DomEventCreationObservation, DomEventDispatcher, DomEventInit,
    DomEventTimestampUnavailable, DomStandaloneEventTarget, NativeDomEvent,
};
use surgeist_dom::{
    DocumentId, DomError, DomSourceEventClockFailure, DomSourceEventFiring, DomSourceEventTime,
    DomSourceLease, DomSourceProgress,
};

/// Private one-use proof of this constructor's current Global and clock join.
pub(crate) struct EventCreation(DomEventCreationObservation);

impl EventCreation {
    pub(crate) fn from_source(
        observation: Option<Result<DomSourceEventTime, DomSourceEventClockFailure>>,
    ) -> Result<Self, DomEventTimestampUnavailable> {
        let time = observation
            .ok_or(DomEventTimestampUnavailable::NoAuthenticatedClock)?
            .map_err(DomEventTimestampUnavailable::Provider)?;
        let origin = time.origin();
        Ok(Self(DomEventCreationObservation {
            global: time.global(),
            source: origin.source(),
            origin_microseconds: origin.microseconds(),
            occurrence_microseconds: time.occurrence().microseconds(),
        }))
    }

    pub(crate) fn into_observation(self) -> DomEventCreationObservation {
        self.0
    }
}

pub(crate) struct EventTargetAssociation {
    global: DomHostedGlobalId,
    document: DocumentId,
}

impl EventTargetAssociation {
    pub(crate) fn into_parts(self) -> (DomHostedGlobalId, DocumentId) {
        (self.global, self.document)
    }
}

struct EventLookupError(DomHostedRuntimeError);

impl From<DomError> for EventLookupError {
    fn from(error: DomError) -> Self {
        DomHostedError::from(error).into()
    }
}

impl From<DomHostedError> for EventLookupError {
    fn from(error: DomHostedError) -> Self {
        Self(DomHostedRuntimeError::Dom(error))
    }
}

impl HostedState {
    fn event_global(
        &mut self,
        dom: &DomEventDom<'_>,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<EventTargetAssociation, DomHostedRuntimeError> {
        self.qualify(dom)?;
        let document = dom
            .raw()
            .dispatch_global_document_with(global, cancel, &mut || {
                self.step(cancel).map_err(EventLookupError)
            })
            .map_err(|error| error.0)?;
        Ok(EventTargetAssociation { global, document })
    }

    fn event_settings(
        &mut self,
        dom: &DomEventDom<'_>,
        association: &EventTargetAssociation,
        cancel: &AtomicBool,
    ) -> Result<&records::Settings, DomHostedRuntimeError> {
        let route = self.find_global(association.global, cancel, |source| {
            source
                .admit_runtime_work(dom.raw(), 1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
        })?;
        self.settings_at(route, association.document)
    }

    fn sample_event_creation(
        &mut self,
        dom: &DomEventDom<'_>,
        association: &EventTargetAssociation,
        cancel: &AtomicBool,
    ) -> Result<EventCreation, DomHostedRuntimeError> {
        let origin = self
            .event_settings(dom, association, cancel)?
            .navigation_start;
        self.sample_event_creation_at_origin(dom, association.global, origin, cancel)
    }

    fn sample_event_creation_at_origin(
        &mut self,
        dom: &DomEventDom<'_>,
        global: DomHostedGlobalId,
        origin: clock::CoarsenedSample,
        cancel: &AtomicBool,
    ) -> Result<EventCreation, DomHostedRuntimeError> {
        self.step(cancel)?;
        self.source
            .admit_runtime_work(dom.raw(), 1, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        let occurrence = self.clock.sample().map_err(clock_error)?;
        Ok(EventCreation(creation_observation(
            global, origin, occurrence,
        )?))
    }
}

/// Non-owning permission for an admitted native producer to sample the
/// original session clock without retaining its Event storage or host.
#[derive(Clone)]
pub(crate) struct HostedEventClock(Weak<RefCell<HostedState>>);

impl HostedOwner {
    pub(crate) fn event_clock(&self) -> HostedEventClock {
        HostedEventClock(Rc::downgrade(&self.state))
    }
}

impl HostedEventClock {
    pub(crate) fn sample(
        &self,
        dom: &DomEventDom<'_>,
        global: DomHostedGlobalId,
        origin: surgeist_dom::DomHostedMoment,
        cancel: &AtomicBool,
    ) -> Result<Result<EventCreation, DomEventTimestampUnavailable>, DomHostedRuntimeError> {
        let owner = self
            .0
            .upgrade()
            .ok_or(DomHostedRuntimeError::OwnerDisposed)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        let source = state.source.id();
        if origin.source() != source || global.source() != source || state.clock.source() != source
        {
            return Err(DomHostedRuntimeError::Protocol);
        }
        state.step(cancel)?;
        if dom.raw().source_dispatch_blocker().is_none() {
            state
                .source
                .admit_runtime_work(dom.raw(), 1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        }
        let occurrence = match state.clock.sample().and_then(|sample| sample.coarsen()) {
            Ok(occurrence) => occurrence,
            Err(error) => {
                return Ok(Err(DomEventTimestampUnavailable::Provider(
                    event_clock_failure(error),
                )));
            }
        };
        let origin = clock::CoarsenedSample {
            source,
            microseconds: origin.microseconds(),
        };
        creation_from_coarsened(global, origin, occurrence)
            .map(|observation| Ok(EventCreation(observation)))
    }
}

fn event_clock_failure(error: clock::Error) -> DomSourceEventClockFailure {
    match error {
        clock::Error::Unavailable => DomSourceEventClockFailure::Unavailable,
        clock::Error::Regressed => DomSourceEventClockFailure::Regressed,
        clock::Error::Overflow => DomSourceEventClockFailure::Overflow,
    }
}

impl NativeTaskRunning {
    pub(crate) fn sample_event_creation(
        &self,
        dom: &DomEventDom<'_>,
        cancel: &AtomicBool,
    ) -> Result<EventCreation, DomHostedRuntimeError> {
        let owner = self
            .state
            .upgrade()
            .ok_or(DomHostedRuntimeError::OwnerDisposed)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        state.qualify(dom)?;
        state.sample_event_creation_at_origin(dom, self.global, self.time_origin, cancel)
    }
}

impl HostedRuntime {
    pub(crate) fn finish_source_event_firing(
        &self,
        lease: &mut DomSourceLease<'_>,
        firing: DomSourceEventFiring,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomHostedRuntimeError> {
        let owner = self.state()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomHostedRuntimeError::Busy)?;
        let HostedState {
            source,
            clock,
            work,
            limits,
            ..
        } = &mut *state;
        if clock.source() != source.id() {
            return Err(DomHostedRuntimeError::Protocol);
        }
        spend(work, limits.work, cancel)?;
        let loan = lease
            .begin_event_firing(firing, source, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        // A provider failure is the outcome of this one actual attempt. Seal
        // it with the selected body so mandatory Source cleanup can continue.
        let occurrence = clock
            .sample()
            .and_then(|sample| sample.coarsen())
            .map(|sample| sample.microseconds)
            .map_err(event_clock_failure);
        loan.finish(occurrence).map_err(DomHostedRuntimeError::Dom)
    }

    /// Construct an initialized untrusted Event from this session's actual
    /// current Global, original time origin and shared native clock.
    pub fn create_event(
        &mut self,
        dom: &mut DomEventDom<'_>,
        global: DomHostedGlobalId,
        event_type: &[u16],
        init: DomEventInit,
        cancel: &AtomicBool,
    ) -> Result<NativeDomEvent, DomEventConstructionError> {
        let creation = {
            let owner = self.state()?;
            let mut state = owner
                .try_borrow_mut()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            let association = state.event_global(dom, global, cancel)?;
            state.sample_event_creation(dom, &association, cancel)?
        };
        NativeDomEvent::authored(
            dom,
            self.event_storage.clone(),
            creation,
            event_type,
            init,
            cancel,
        )
        .map_err(Into::into)
    }

    /// Construct an untrusted, uninitialized Event using the receiving
    /// Document's actual relevant Global and original time origin.
    ///
    /// This base Event constructor accepts Event, Events, HTMLEvents and
    /// SVGEvents with ASCII-insensitive matching. Dispatch requires a
    /// successful `NativeDomEvent::init_event`.
    pub fn create_legacy_event(
        &mut self,
        dom: &mut DomEventDom<'_>,
        document: DocumentId,
        interface: &[u16],
        cancel: &AtomicBool,
    ) -> Result<NativeDomEvent, DomEventConstructionError> {
        if !base_event_interface(interface) {
            return Err(DomEventConstructionError::UnsupportedInterface);
        }
        let creation = {
            let owner = self.state()?;
            let mut state = owner
                .try_borrow_mut()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            state.qualify(dom)?;
            let global = dom
                .raw()
                .dispatch_document_relevant_global_with(document, cancel, &mut || {
                    state.step(cancel).map_err(EventLookupError)
                })
                .map_err(|error| error.0)?
                .ok_or(DomEventConstructionError::MissingRelevantGlobal)?;
            let association = EventTargetAssociation { global, document };
            state.sample_event_creation(dom, &association, cancel)?
        };
        NativeDomEvent::legacy(dom, self.event_storage.clone(), creation, cancel)
            .map_err(Into::into)
    }

    /// Construct a standalone EventTarget with this session's actual relevant
    /// Global. Its listener list belongs to the supplied dispatcher.
    pub fn create_event_target<C>(
        &mut self,
        dom: &mut DomEventDom<'_>,
        dispatcher: &DomEventDispatcher<C>,
        global: DomHostedGlobalId,
        cancel: &AtomicBool,
    ) -> Result<DomStandaloneEventTarget, DomEventConstructionError> {
        let association = {
            let owner = self.state()?;
            let mut state = owner
                .try_borrow_mut()
                .map_err(|_| DomHostedRuntimeError::Busy)?;
            let association = state.event_global(dom, global, cancel)?;
            state.event_settings(dom, &association, cancel)?;
            association
        };
        dispatcher
            .create_standalone_target(dom, association, cancel)
            .map_err(Into::into)
    }
}

fn base_event_interface(interface: &[u16]) -> bool {
    const ALIASES: [&[u8]; 4] = [b"event", b"events", b"htmlevents", b"svgevents"];
    ALIASES.iter().any(|alias| {
        interface.len() == alias.len()
            && interface.iter().zip(alias.iter()).all(|(&unit, &ascii)| {
                unit == u16::from(ascii) || unit == u16::from(ascii.to_ascii_uppercase())
            })
    })
}

pub(super) fn creation_observation(
    global: DomHostedGlobalId,
    origin: clock::CoarsenedSample,
    occurrence: clock::Sample,
) -> Result<DomEventCreationObservation, DomHostedRuntimeError> {
    let occurrence = occurrence.coarsen().map_err(clock_error)?;
    creation_from_coarsened(global, origin, occurrence)
}

fn creation_from_coarsened(
    global: DomHostedGlobalId,
    origin: clock::CoarsenedSample,
    occurrence: clock::CoarsenedSample,
) -> Result<DomEventCreationObservation, DomHostedRuntimeError> {
    if occurrence.source != origin.source {
        return Err(DomHostedRuntimeError::Protocol);
    }
    Ok(DomEventCreationObservation {
        global,
        source: origin.source,
        origin_microseconds: origin.microseconds,
        occurrence_microseconds: occurrence.microseconds,
    })
}
