//! Attached guest delivery through the original native Source dispatch.

use super::*;
use crate::{
    internal::dom_dialog::DomSourceEventServiceLimits,
    internal::dom_dialog::DomSourceEventServiceProgress,
    internal::dom_event::{DispatchUsage, DomEventDom},
    internal::dom_source::{DomSourceDriveError, DomSourceRuntimeError},
};
use surgeist_dom::{
    event::DomSourceEventRequest, source::attribute::DomSourceAttributeVerificationRequest,
};

pub(super) struct ActiveAttachedEvent<'owner> {
    owner: &'owner NativeOwner,
    previous_event: Option<DomSourceEventRequest>,
    previous_attribute: Option<DomSourceAttributeVerificationRequest>,
}

impl<'owner> ActiveAttachedEvent<'owner> {
    fn enter(
        owner: &'owner NativeOwner,
        request: &DomSourceEventRequest,
    ) -> Result<Self, DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            owner.qualify_dispatched_event(attachment, &mut session, request)?;
        }
        let previous_event = owner.active_attached_event.replace(Some(request.clone()));
        // A child event owns the top Source callback scope. Its parent's
        // attribute verification retains custody until this listener exits.
        let previous_attribute = owner.active_attribute.take();
        Ok(Self {
            owner,
            previous_event,
            previous_attribute,
        })
    }
}

impl Drop for ActiveAttachedEvent<'_> {
    fn drop(&mut self) {
        self.owner
            .active_attached_event
            .replace(self.previous_event.take());
        self.owner
            .active_attribute
            .set(self.previous_attribute.take());
    }
}

#[derive(Clone, Copy)]
enum AttachedSourceEntry {
    External(DocumentId),
    Qualified(DocumentId),
}

impl DocumentScriptHost {
    /// Service the exact pending native event through the transferred listener
    /// registry. The original dispatcher owns the event and its cursor. A
    /// serviced receipt leaves the remaining Source tail for the caller.
    ///
    /// A listener requiring another host route leaves the event retained.
    /// This entry runs no microtask checkpoint.
    pub fn service_attached_source_event(
        &mut self,
        permit: &DocumentScriptPermit,
        request: &DomSourceEventRequest,
        limits: DomSourceEventServiceLimits,
    ) -> Result<DomSourceEventServiceProgress, DocumentScriptEventError> {
        self.check_permit(permit)?;
        self.attachment()?.realm.check_owner_active()?;
        let events = self
            .guest_events()
            .map_err(DocumentScriptEventError::Guest)?;
        let owner = self.native.clone();
        let mut usage = DispatchUsage::default();
        let mut entry = AttachedSourceEntry::External(permit.document);
        loop {
            let progress = self
                .with_native(|dom, _, source| {
                    events.advance_attached_source(
                        &owner,
                        dom,
                        &source,
                        request,
                        (limits, &mut usage),
                        entry,
                    )
                })??
                .map_err(DocumentScriptEventError::Service)?;
            let DomSourceEventServiceProgress::Retained {
                dispatch: DomEventDispatchOutcome::HostInvocation(mut selected),
            } = progress
            else {
                return Ok(progress);
            };
            // A real selected invocation proves native path admission.
            entry = AttachedSourceEntry::Qualified(permit.document);
            self.invoke_attached_listener(&events, Some(request), &mut selected)?;
        }
    }

    pub(super) fn invoke_attached_listener(
        &mut self,
        events: &GuestEvents,
        request: Option<&DomSourceEventRequest>,
        selected: &mut DomEventHostInvocation,
    ) -> Result<(), DocumentScriptEventError> {
        let owner = &self.native;
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let result = match self.engine.as_mut() {
            Some(engine) => engine
                .process
                .execution()
                .in_realm(&engine.realm, |scope| {
                    Ok(events.invoke_native_listener(owner, scope, request, selected))
                })
                .map_err(DocumentGuestEventError::from)
                .and_then(|result| result),
            None => Err(DocumentScriptNativeError::Revoked.into()),
        };
        let result = events.report_selected(selected, result);
        if let Some(engine) = self.engine.as_mut() {
            engine.settle_engine_cleanup(owner, &attachment.realm);
        }
        result.map_err(|error| match error {
            DocumentGuestEventError::Document(error) => DocumentScriptEventError::Access(error),
            error => DocumentScriptEventError::Guest(error),
        })
    }
}

impl GuestEvents {
    fn advance_attached_source(
        &self,
        owner: &NativeOwner,
        dom: &mut DomEventDom<'_>,
        source: &DomSourceRuntime,
        request: &DomSourceEventRequest,
        limits: (DomSourceEventServiceLimits, &mut DispatchUsage),
        entry: AttachedSourceEntry,
    ) -> Result<
        Result<DomSourceEventServiceProgress, DomSourceRuntimeError>,
        DocumentScriptNativeError,
    > {
        let (document, departed) = match entry {
            AttachedSourceEntry::External(document) => {
                super::super::source::validate_current_source_event(
                    dom,
                    request,
                    &owner.cancellation,
                )?;
                let snapshot = dom.snapshot();
                let error = match snapshot.document(document)?.node(request.subject()) {
                    Ok(_) => None,
                    Err(error @ (DomError::WrongDocument | DomError::StaleNode)) => Some(error),
                    Err(error) => return Err(error.into()),
                };
                (document, error)
            }
            AttachedSourceEntry::Qualified(document) => (document, None),
        };
        let mut registry = self
            .registry
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        if let Some(error) = departed {
            match source.has_started_attached_event(
                request,
                document,
                &registry,
                &owner.cancellation,
            ) {
                Ok(true) => {}
                Ok(false) => return Err(error.into()),
                Err(error) => return Ok(Err(error)),
            }
        }
        Ok(source.service_attached_event(
            dom,
            request,
            document,
            &mut registry,
            limits,
            &owner.cancellation,
        ))
    }

    pub(super) fn service_guest_source_event(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'_>,
        request: &DomSourceEventRequest,
    ) -> Result<(), DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let limits = DomSourceEventServiceLimits {
            source: self.limits.source_edits,
            dispatch: self.limits.dispatch,
        };
        let mut usage = DispatchUsage::default();
        loop {
            scope.check_active()?;
            let progress = {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let source = session.source_execution();
                self.advance_attached_source(
                    owner,
                    &mut session.dom(),
                    &source,
                    request,
                    (limits, &mut usage),
                    // The exact request was just issued to the authenticated
                    // guest edit. Native dispatch still qualifies its path.
                    AttachedSourceEntry::Qualified(attachment.target.document()),
                )?
            };
            let mut selected = match progress {
                Ok(DomSourceEventServiceProgress::Serviced { .. }) => return Ok(()),
                Ok(DomSourceEventServiceProgress::Retained { dispatch }) => {
                    selected_dispatch(dispatch)?
                }
                Err(error) => {
                    return refuse(
                        scope,
                        &attachment.source_failures,
                        DomSourceDriveError::from(error),
                    )
                    .map_err(Into::into);
                }
            };
            let invoked = self.invoke_native_listener(owner, scope, Some(request), &selected);
            let reported = self.report_selected(&mut selected, invoked);
            scope.check_active()?;
            reported?;
        }
    }

    fn invoke_native_listener(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'_>,
        request: Option<&DomSourceEventRequest>,
        selected: &DomEventHostInvocation,
    ) -> Result<Result<(), DomEventCallbackError>, DocumentGuestEventError> {
        owner.schedule_media_removals(scope)?;
        let mut report = None;
        let mut active = None;
        let values = (|| {
            active = request
                .map(|request| ActiveAttachedEvent::enter(owner, request))
                .transpose()?;
            self.step()?;
            report = Some(self.reports.reserve_count(1)?);
            let current = selected
                .event()
                .current_target()
                .ok_or(Error::RegistryConflict)?;
            let receiver = wrapper::project_native_event_target(owner, self, scope, &current)?;
            let event = self.project_native_event(owner, scope, selected.event())?;
            Ok((event, receiver))
        })();
        match self.invoke_selected(
            owner,
            scope,
            selected,
            values,
            active.as_ref(),
            |_scope, exception| {
                report
                    .as_mut()
                    .ok_or(DocumentGuestEventError::RegistryState)?
                    .publish(exception)
            },
        ) {
            Ok(result) => Ok(result),
            Err(error) => match scope.check_active() {
                Err(first) => Err(first.into()),
                Ok(()) => refuse(scope, &self.refusal, error).map_err(Into::into),
            },
        }
    }

    fn project_native_event<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        native: &NativeDomEvent,
    ) -> Result<Value<'scope>, DocumentGuestEventError> {
        let mut identity = native
            .attached_projection()
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        if let Some(token) = identity.as_ref()
            && let Some(event) = scope.open_native_object(token)?
        {
            return Ok(event);
        }
        let public = self
            .public
            .as_ref()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        let prototype = public.native_event_prototype(scope, native)?;
        let trusted = public.trusted_getter(scope)?;
        let target = native.target().ok_or(Error::RegistryConflict)?;
        let target = wrapper::project_native_event_target(owner, self, scope, &target)?;
        let event = scope.project_native_object(
            ObjectPrototype::Object(&prototype),
            GuestEventProjection(native.clone()),
            |bytes| self.reserve(owner, bytes),
        )?;
        interfaces::expose_trust(scope, &event, &trusted)?;
        scope.set_native_object_reference::<GuestEventProjection>(
            &event,
            interfaces::EVENT_TARGET_REFERENCE,
            Some(&target),
        )?;
        *identity = Some(
            scope
                .native_object_token(&event)?
                .ok_or(Error::ExpectedNativeObject)?,
        );
        Ok(event)
    }
}
