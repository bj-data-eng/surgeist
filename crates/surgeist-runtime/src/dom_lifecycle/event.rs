use super::*;
use crate::dom_event::{
    DomEventConstructionError, DomEventCreationObservation, DomEventDispatcher, DomEventInit,
    DomStandaloneEventTarget, NativeDomEvent,
};
use surgeist_dom::DocumentId;

/// Private one-use proof of this constructor's current Global and clock join.
pub(crate) struct EventCreation(DomEventCreationObservation);

impl EventCreation {
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
            .dispatch_global_document(global, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
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
        self.step(cancel)?;
        self.source
            .admit_runtime_work(dom.raw(), 1, cancel)
            .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
        let occurrence = self.clock.sample().map_err(clock_error)?;
        Ok(EventCreation(creation_observation(
            association.global,
            origin,
            occurrence,
        )?))
    }
}

impl HostedRuntime {
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
                .dispatch_relevant_global(document.node(), cancel)
                .map_err(DomHostedRuntimeError::Dom)?
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
