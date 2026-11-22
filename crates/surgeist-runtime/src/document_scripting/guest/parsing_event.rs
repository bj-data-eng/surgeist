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
    listener: Option<ParsingListener>,
    dispatch: Option<DomEventDispatch>,
}

struct ParsingListener {
    selection: DomEventHostInvocation,
    result: Result<(), DomEventCallbackError>,
    checkpoint: Option<Root>,
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
            listener: None,
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
            if let Some(listener) = &mut firing.listener {
                if let Some(event) = &listener.checkpoint {
                    // An outermost listener's script cleanup performs the
                    // checkpoint before returning to DOM's invocation steps.
                    // Keep the native selection open and Window.event current
                    // while jobs may synchronously reenter event dispatch.
                    let _current = events.window_event.enter(event.clone());
                    self.checkpoint()
                        .map_err(DocumentScriptEventError::Access)?;
                    listener.checkpoint.take();
                }
                self.finish_attached_listener(
                    &events,
                    &mut listener.selection,
                    Ok(listener.result),
                )?;
                firing.listener.take();
            }
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
                    let mut event = None;
                    let result =
                        self.call_attached_listener(&events, None, &selected, Some(&mut event));
                    match (result, event) {
                        (Ok(result), Some(event)) => {
                            firing.listener = Some(ParsingListener {
                                selection: selected,
                                result,
                                checkpoint: Some(event),
                            });
                        }
                        (Ok(_), None) => self.finish_attached_listener(
                            &events,
                            &mut selected,
                            Err(DocumentGuestEventError::RegistryState),
                        )?,
                        (Err(error), _) => {
                            self.finish_attached_listener(&events, &mut selected, Err(error))?;
                        }
                    }
                }
                settled => return Ok(settled),
            }
        }
    }
}
