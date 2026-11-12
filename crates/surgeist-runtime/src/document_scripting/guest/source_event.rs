//! Attached guest delivery through the original native Source dispatch.

use super::*;
use crate::{DomSourceEventServiceLimits, DomSourceEventServiceProgress};
use surgeist_dom::DomSourceEventRequest;

pub(super) struct ActiveAttachedEvent<'owner>(&'owner NativeOwner);

impl<'owner> ActiveAttachedEvent<'owner> {
    fn enter(
        owner: &'owner NativeOwner,
        request: &DomSourceEventRequest,
    ) -> Result<Self, DocumentGuestEventError> {
        if owner.active_attached_event.borrow().is_some() {
            return Err(DocumentGuestEventError::RegistryState);
        }
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            owner.qualify_event(attachment, &mut session, request)?;
        }
        owner.active_attached_event.replace(Some(request.clone()));
        Ok(Self(owner))
    }
}

impl Drop for ActiveAttachedEvent<'_> {
    fn drop(&mut self) {
        self.0.active_attached_event.replace(None);
    }
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
        let mut usage = crate::dom_event::DispatchUsage::default();
        loop {
            let progress = self.with_native(|dom, _, source| {
                super::super::source::validate_source_event(
                    dom,
                    permit.document,
                    request,
                    &owner.cancellation,
                )?;
                let mut registry = events
                    .registry
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                source
                    .service_attached_event(
                        dom,
                        request,
                        &mut registry,
                        (limits, &mut usage),
                        &owner.cancellation,
                    )
                    .map_err(DocumentScriptEventError::Service)
            })??;
            let DomSourceEventServiceProgress::Retained {
                dispatch: DomEventDispatchOutcome::HostInvocation(mut selected),
            } = progress
            else {
                return Ok(progress);
            };
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
            engine.settle_engine_cleanup(&attachment.realm);
        }
        result.map_err(|error| match error {
            DocumentGuestEventError::Document(error) => DocumentScriptEventError::Access(error),
            error => DocumentScriptEventError::Guest(error),
        })
    }
}

impl GuestEvents {
    fn invoke_native_listener(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'_>,
        request: Option<&DomSourceEventRequest>,
        selected: &DomEventHostInvocation,
    ) -> Result<Result<(), DomEventCallbackError>, DocumentGuestEventError> {
        let mut report = None;
        let mut active = None;
        let values = (|| {
            active = request
                .map(|request| ActiveAttachedEvent::enter(owner, request))
                .transpose()?;
            self.step()?;
            report = Some(self.reports.reserve_count(self, owner, 1)?);
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
            |scope, exception| {
                report
                    .as_mut()
                    .ok_or(DocumentGuestEventError::RegistryState)?
                    .publish(scope, exception)
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
