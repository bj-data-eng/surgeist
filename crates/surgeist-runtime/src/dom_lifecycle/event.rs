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
    ) -> Result<(EventTargetAssociation, clock::CoarsenedSample), DomHostedRuntimeError> {
        self.qualify(dom)?;
        let document = dom
            .raw()
            .dispatch_global_document(global, cancel)
            .map_err(DomHostedRuntimeError::Dom)?;
        let route = self.find_global(global, cancel, |source| {
            source
                .admit_runtime_work(dom.raw(), 1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))
        })?;
        let origin = self.settings_at(route, document)?.navigation_start;
        Ok((EventTargetAssociation { global, document }, origin))
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
            let (_, origin) = state.event_global(dom, global, cancel)?;
            state.step(cancel)?;
            state
                .source
                .admit_runtime_work(dom.raw(), 1, cancel)
                .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
            let occurrence = state.clock.sample().map_err(clock_error)?;
            EventCreation(creation_observation(global, origin, occurrence)?)
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
            state.event_global(dom, global, cancel)?.0
        };
        dispatcher
            .create_standalone_target(dom, association, cancel)
            .map_err(Into::into)
    }
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
