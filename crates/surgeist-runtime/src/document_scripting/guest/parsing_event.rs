//! Hosted delivery of the native events fired by HTML stop parsing.

use super::*;
use crate::internal::{
    dom_event::{DispatchUsage, ParsingEvent},
    dom_lifecycle::{EventCreation, NativeTaskRunning},
};

/// One original event occurrence held by the lifecycle task until delivery
/// settles. Admission can suspend either creation or the listener cursor.
pub(in crate::document_scripting) struct ParsingEventFiring {
    event: ParsingEvent,
    global: DomHostedGlobalId,
    document: DocumentId,
    creation: Option<Result<EventCreation, DomEventTimestampUnavailable>>,
    dispatch: Option<DomEventDispatch>,
}

impl ParsingEventFiring {
    pub(in crate::document_scripting) fn new(
        event: ParsingEvent,
        global: DomHostedGlobalId,
        document: DocumentId,
    ) -> Self {
        Self {
            event,
            global,
            document,
            creation: None,
            dispatch: None,
        }
    }
}

impl DocumentScriptHost {
    pub(in crate::document_scripting) fn advance_parsing_event(
        &mut self,
        firing: &mut ParsingEventFiring,
        running: &NativeTaskRunning,
        limits: DomEventDispatchLimits,
    ) -> Result<DomEventDispatchOutcome, DocumentScriptEventError> {
        let attachment = self.attachment()?;
        if attachment.target.global() != firing.global
            || attachment.target.document() != firing.document
            || running.global() != firing.global
        {
            return Err(DocumentScriptNativeError::ForeignPermit.into());
        }
        attachment.realm.check_owner_active()?;
        let events = self
            .guest_events()
            .map_err(DocumentScriptEventError::Guest)?;
        let owner = self.native.clone();
        let cancel = &owner.cancellation;
        let mut usage = DispatchUsage::default();
        loop {
            let progress = self
                .with_native(|dom, _, _| {
                    let mut registry = events
                        .registry
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    if firing.dispatch.is_none() {
                        if firing.creation.is_none() {
                            firing.creation = Some(
                                running
                                    .sample_event_creation(dom, cancel)
                                    .map_err(DocumentScriptNativeError::from)?,
                            );
                        }
                        firing.dispatch = Some(registry.prepare_parsing_event(
                            dom,
                            firing.event,
                            (firing.global, firing.document),
                            &mut firing.creation,
                            cancel,
                        )?);
                    }
                    let dispatch = firing
                        .dispatch
                        .as_mut()
                        .ok_or(DocumentGuestEventError::RegistryState)?;
                    Ok::<_, DocumentGuestEventError>(dispatch.advance_with_host_usage(
                        dom,
                        &mut registry,
                        &mut (),
                        (limits, &mut usage),
                        cancel,
                        DispatchHost::AttachedNative,
                    )?)
                })?
                .map_err(DocumentScriptEventError::Guest)?;
            match progress {
                DomEventDispatchOutcome::HostInvocation(mut selected) => {
                    self.invoke_attached_listener(&events, None, &mut selected)?;
                }
                settled => return Ok(settled),
            }
        }
    }
}
