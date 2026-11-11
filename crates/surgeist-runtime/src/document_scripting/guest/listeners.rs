//! Registry identity and guest-traced listener custody.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct ListenerMatch<'value, 'scope> {
    pub(super) receiver: &'value Value<'scope>,
    pub(super) target: &'value DomStandaloneEventTarget,
    pub(super) event_type: &'value [u16],
    pub(super) callback: &'value Value<'scope>,
    pub(super) capture: bool,
}

impl RegisteredGuestListener {
    pub(super) fn realm(&self) -> &RealmAuthority {
        match &self.object {
            RegisteredListenerObject::Retained(callback) => callback.associated_realm(),
            RegisteredListenerObject::Traced(traced) => &traced.realm,
        }
    }

    pub(super) fn open<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        receiver: &Value<'scope>,
    ) -> Result<Value<'scope>, Error> {
        match &self.object {
            RegisteredListenerObject::Retained(callback) => scope.open_callback_object(callback),
            RegisteredListenerObject::Traced(traced) => scope
                .native_object_reference::<GuestTargetProjection>(receiver, traced.slot)?
                .ok_or(Error::RegistryConflict),
        }
    }

    pub(super) fn clear_receiver<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        receiver: &Value<'scope>,
    ) -> Result<(), Error> {
        if let RegisteredListenerObject::Traced(traced) = &self.object {
            scope.set_native_object_reference::<GuestTargetProjection>(
                receiver,
                traced.slot,
                None,
            )?;
        }
        Ok(())
    }
}

impl GuestEvents {
    pub(super) fn target_projection(
        &self,
        owner: &NativeOwner,
        native: DomStandaloneEventTarget,
    ) -> Result<GuestTargetProjection, DocumentGuestEventError> {
        let lifetime = if self.public.is_some() {
            Some(Rc::new(GuestTargetLifetime {
                retired: Cell::new(false),
                events: self.backing.clone(),
                _backing: self.reserve(owner, size_of::<GuestTargetLifetime>())?,
            }))
        } else {
            None
        };
        Ok(GuestTargetProjection { native, lifetime })
    }

    pub(in crate::document_scripting) fn reap_retired_targets(&self) {
        if self.backing.retired_targets.get()
            && let Ok(mut registry) = self.registry.try_borrow_mut()
        {
            self.backing.retired_targets.set(false);
            registry.reap_guest_target_registrations();
        }
    }

    pub(super) fn find_listener<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        key: ListenerMatch<'_, 'scope>,
    ) -> Result<Option<DomEventRegistrationId>, DocumentGuestEventError> {
        let target = DomEventTarget::from(key.target.clone());
        let mut cursor = 0;
        loop {
            let candidate = self
                .registry
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?
                .next_guest_registration(
                    &target,
                    key.event_type,
                    key.capture,
                    &mut cursor,
                    &owner.cancellation,
                )?;
            let Some((registration, candidate)) = candidate else {
                return Ok(None);
            };
            self.step()?;
            let existing = candidate.open(scope, key.receiver)?;
            if scope.strict_equals(key.callback, &existing)? {
                return Ok(Some(registration));
            }
        }
    }

    pub(super) fn remove_registration(
        &self,
        scope: &mut Scope<'_>,
        id: DomEventRegistrationId,
    ) -> Result<bool, DocumentGuestEventError> {
        self.step()?;
        let (callback, removed) = {
            let mut registry = self
                .registry
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            (
                registry.guest_registration(id),
                registry.remove_listener(id),
            )
        };
        if let Some(callback) = callback
            && let RegisteredListenerObject::Traced(traced) = &callback.object
            && let Some(target) = scope.open_native_object(&traced.target)?
        {
            callback.clear_receiver(scope, &target)?;
        }
        Ok(removed)
    }

    pub(super) fn register_traced<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        projection: &GuestTargetProjection,
        key: ListenerMatch<'_, 'scope>,
        options: DomEventListenerOptions,
        realm: RealmAuthority,
    ) -> Result<(), DocumentGuestEventError> {
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let target = scope
            .native_object_token::<GuestTargetProjection>(key.receiver)?
            .ok_or(Error::ExpectedNativeObject)?;
        let lifetime = projection
            .lifetime
            .as_ref()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        let backing = self.reserve(owner, size_of::<RegisteredGuestListener>())?;
        let code = self
            .next_callback
            .get()
            .checked_add(1)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let slot =
            u32::try_from(code).map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
        let callback = DomEventCallback::guest(Rc::new(RegisteredGuestListener {
            code,
            object: RegisteredListenerObject::Traced(TracedListener {
                realm,
                target,
                slot,
                lifetime: lifetime.clone(),
            }),
            registration_window: attachment.target,
            _backing: backing,
        }));
        let id = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut registry = self
                .registry
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            registry.add_listener(
                &mut session.dom(),
                key.target.clone(),
                key.event_type,
                callback,
                options,
            )?
        };
        if let Err(error) = scope.set_native_object_reference::<GuestTargetProjection>(
            key.receiver,
            slot,
            Some(key.callback),
        ) {
            if let Ok(mut registry) = self.registry.try_borrow_mut() {
                registry.remove_listener(id);
            }
            return Err(error.into());
        }
        self.next_callback.set(code);
        Ok(())
    }
}
