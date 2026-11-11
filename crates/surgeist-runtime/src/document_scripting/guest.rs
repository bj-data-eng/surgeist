//! Registered guest callbacks for ordinary authored standalone dispatch.

use super::*;
use crate::dom_event::{
    DispatchHost, DomEventCallback, DomEventCallbackError, DomEventConstructionError,
    DomEventDispatchError, DomEventDispatchLimits, DomEventDispatchOutcome,
    DomEventDispatchReceipt, DomEventDispatcher, DomEventHostError, DomEventInit,
    DomEventListenerOptions, DomEventRegistrationId, DomEventTarget, DomStandaloneEventTarget,
    NativeDomEvent,
};
use surgeist_bindings_js::{Exception, ObjectPrototype, RetainedCallbackObject, Root, Value};

/// Guest adapter limits in addition to the original registry and Hosted limits.
#[derive(Clone, Copy, Debug)]
pub struct DocumentGuestEventLimits {
    /// Maximum retained guest callback, projection and report backing.
    pub storage_bytes: usize,
    /// Cumulative guest adapter work.
    pub work: u64,
    /// Native path, callback, failure and work limits for one advance.
    pub dispatch: DomEventDispatchLimits,
}

/// Failure before successful guest dispatch completion.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentGuestEventError {
    /// Exact attached host or binding refusal.
    Document(DocumentScriptError),
    /// Event or standalone target construction refused.
    Construction(DomEventConstructionError),
    /// Native dispatch or registration refused.
    Dispatch(DomEventDispatchError),
    /// The associated engine Realm has no authority in this attachment.
    CallbackRealmUnavailable,
    /// This host has no installed guest registry, or it already has one.
    RegistryState,
    /// Finite cumulative guest adapter work refused.
    WorkLimit,
    /// The next listener requires a different host route.
    HostRouteUnavailable,
    /// The original Source operation blocked synchronous completion.
    SourcePending(surgeist_dom::DomSourceOperationId),
    /// The original hosted operation blocked synchronous completion.
    HostedPending(surgeist_dom::DomHostedOperationId),
}

impl fmt::Display for DocumentGuestEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "registered guest EventTarget: {self:?}")
    }
}

impl std::error::Error for DocumentGuestEventError {}

impl From<DocumentScriptError> for DocumentGuestEventError {
    fn from(error: DocumentScriptError) -> Self {
        Self::Document(error)
    }
}

impl From<DocumentScriptNativeError> for DocumentGuestEventError {
    fn from(error: DocumentScriptNativeError) -> Self {
        DocumentScriptError::from(error).into()
    }
}

impl From<Error> for DocumentGuestEventError {
    fn from(error: Error) -> Self {
        DocumentScriptError::from(error).into()
    }
}

impl From<DomEventDispatchError> for DocumentGuestEventError {
    fn from(error: DomEventDispatchError) -> Self {
        Self::Dispatch(error)
    }
}

/// An explicitly host-owned guest projection of a native standalone target.
pub struct DocumentGuestEventTarget {
    native: DomStandaloneEventTarget,
    object: RetainedCallbackObject,
}

impl DocumentGuestEventTarget {
    /// Observe the exact projected object in its own process.
    pub fn object<'scope>(&self, scope: &mut Scope<'scope>) -> Result<Value<'scope>, Error> {
        scope.open_callback_object(&self.object)
    }
}

/// An explicitly host-owned guest projection of an authored native Event.
pub struct DocumentGuestEvent {
    native: NativeDomEvent,
    object: RetainedCallbackObject,
}

impl DocumentGuestEvent {
    /// Observe the exact projected object in its own process.
    pub fn object<'scope>(&self, scope: &mut Scope<'scope>) -> Result<Value<'scope>, Error> {
        scope.open_callback_object(&self.object)
    }

    /// The native body's live observations and native Event controls.
    pub fn native(&self) -> &NativeDomEvent {
        &self.native
    }
}

/// The original exception reported by one selected registration.
#[derive(Debug)]
pub struct DocumentGuestListenerException {
    /// Exact native listener registration.
    pub registration: DomEventRegistrationId,
    /// Associated callback Realm, independently qualified by the attachment.
    pub realm: RealmId,
    /// Original JavaScript thrown value.
    pub exception: Exception,
}

/// Completed native dispatch with original ordinary guest exceptions.
pub struct DocumentGuestEventReport {
    receipt: DomEventDispatchReceipt,
    exceptions: Vec<DocumentGuestListenerException>,
    _backing: GuestBacking,
}

impl DocumentGuestEventReport {
    /// Native completion, including the independent cancellation result.
    pub fn receipt(&self) -> &DomEventDispatchReceipt {
        &self.receipt
    }

    /// Original guest exception observations, in invocation order.
    pub fn exceptions(&self) -> &[DocumentGuestListenerException] {
        &self.exceptions
    }
}

struct GuestBacking {
    used: Rc<Cell<usize>>,
    bytes: usize,
    _canonical: DomHostedRuntimeRetention,
}

impl Drop for GuestBacking {
    fn drop(&mut self) {
        self.used.set(self.used.get() - self.bytes);
    }
}

pub(crate) struct RegisteredGuestListener {
    pub(crate) code: u64,
    object: RetainedCallbackObject,
    settings: DomHostedTopLevel,
    _backing: GuestBacking,
}

pub(super) struct GuestEvents {
    registry: RefCell<DomEventDispatcher<()>>,
    limits: DocumentGuestEventLimits,
    used: Rc<Cell<usize>>,
    work: Cell<u64>,
    next_callback: Cell<u64>,
    exception_root_bytes: usize,
    _backing: DomHostedRuntimeRetention,
}

impl GuestEvents {
    fn step(&self) -> Result<(), DocumentGuestEventError> {
        let work = self
            .work
            .get()
            .checked_add(1)
            .ok_or(DocumentGuestEventError::WorkLimit)?;
        if work > self.limits.work {
            return Err(DocumentGuestEventError::WorkLimit);
        }
        self.work.set(work);
        Ok(())
    }

    fn reserve(
        &self,
        owner: &NativeOwner,
        bytes: usize,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        self.step()?;
        let total = self
            .used
            .get()
            .checked_add(bytes)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        if total > self.limits.storage_bytes {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let canonical = session
            .hosted_execution()
            .retain_script_payload(&mut session.dom(), bytes, &owner.cancellation)
            .map_err(DocumentScriptNativeError::from)?;
        self.used.set(total);
        Ok(GuestBacking {
            used: self.used.clone(),
            bytes,
            _canonical: canonical,
        })
    }

    fn project(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'_>,
    ) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
        self.step()?;
        let object = scope.object(ObjectPrototype::Default)?;
        let observation = scope.callback_object(&object)?;
        scope.retain_callback_object(observation, |bytes| self.reserve(owner, bytes))
    }
}

impl DocumentScriptHost {
    /// Transfer the original native listener registry to this attached host.
    /// A refusal leaves the input registry in its original slot.
    pub fn attach_guest_event_registry(
        &mut self,
        permit: &DocumentScriptPermit,
        registry: &mut Option<DomEventDispatcher<()>>,
        limits: DocumentGuestEventLimits,
    ) -> Result<(), DocumentGuestEventError> {
        self.check_permit(permit)?;
        self.observe()?;
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if engine.guest_events.is_some() {
            return Err(DocumentGuestEventError::RegistryState);
        }
        let input = registry
            .as_ref()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        let fixed = size_of::<GuestEvents>() + size_of::<Cell<usize>>();
        if limits.storage_bytes < fixed
            || engine
                .retained_bytes_with(limits.storage_bytes)
                .is_none_or(|total| total > self.native.storage_limit)
        {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let backing = {
            let mut session = self
                .native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            input.qualify_guest_registry(&session.dom())?;
            session
                .hosted_execution()
                .retain_script_payload(&mut session.dom(), fixed, &self.native.cancellation)
                .map_err(DocumentScriptNativeError::from)?
        };
        let registry = registry
            .take()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        let exception_root_bytes = engine.process.managed_object_storage_bytes();
        engine.guest_events = Some(Rc::new(GuestEvents {
            registry: RefCell::new(registry),
            limits,
            used: Rc::new(Cell::new(fixed)),
            work: Cell::new(0),
            next_callback: Cell::new(0),
            exception_root_bytes,
            _backing: backing,
        }));
        engine.retained_bytes += limits.storage_bytes;
        Ok(())
    }

    fn guest_events(&self) -> Result<Rc<GuestEvents>, DocumentGuestEventError> {
        self.engine
            .as_ref()
            .and_then(|engine| engine.guest_events.clone())
            .ok_or(DocumentGuestEventError::RegistryState)
    }

    /// Create and project an authored standalone target in the attached Window.
    pub fn create_guest_event_target(
        &mut self,
        permit: &DocumentScriptPermit,
    ) -> Result<DocumentGuestEventTarget, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let target = self.attachment()?.target;
        let events = self.guest_events()?;
        let owner = self.native.clone();
        self.in_document(permit, |scope| {
            Ok((|| {
                let object = events.project(&owner, scope)?;
                let native = {
                    let mut session = owner
                        .session
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    let registry = events
                        .registry
                        .try_borrow()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    session
                        .hosted_execution()
                        .create_event_target(
                            &mut session.dom(),
                            &registry,
                            target.global(),
                            &owner.cancellation,
                        )
                        .map_err(DocumentGuestEventError::Construction)?
                };
                Ok(DocumentGuestEventTarget { native, object })
            })())
        })?
    }

    /// Create and project an authored Event using the attached Window's clock.
    pub fn create_guest_event(
        &mut self,
        permit: &DocumentScriptPermit,
        event_type: &[u16],
        init: DomEventInit,
    ) -> Result<DocumentGuestEvent, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let target = self.attachment()?.target;
        let events = self.guest_events()?;
        let owner = self.native.clone();
        self.in_document(permit, |scope| {
            Ok((|| {
                let object = events.project(&owner, scope)?;
                let native = {
                    let mut session = owner
                        .session
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    session
                        .hosted_execution()
                        .create_event(
                            &mut session.dom(),
                            target.global(),
                            event_type,
                            init,
                            &owner.cancellation,
                        )
                        .map_err(DocumentGuestEventError::Construction)?
                };
                Ok(DocumentGuestEvent { native, object })
            })())
        })?
    }

    /// Register the exact guest function under the attached Window's authority.
    pub fn add_guest_event_listener(
        &mut self,
        permit: &DocumentScriptPermit,
        target: &DocumentGuestEventTarget,
        event_type: &[u16],
        callback: &Root,
        options: DomEventListenerOptions,
    ) -> Result<DomEventRegistrationId, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let settings = self.attachment()?.target;
        let owner = self.native.clone();
        let events = self.guest_events()?;
        self.in_document(permit, |scope| {
            Ok((|| {
                events.step()?;
                let value = scope.import(callback)?;
                let observed = scope.callback_object(&value)?;
                if observed.associated_realm().identity() != permit.realm {
                    return Err(DocumentGuestEventError::CallbackRealmUnavailable);
                }
                if !scope.is_callable(&value)? {
                    return Err(Error::ExpectedFunction.into());
                }
                let target = DomEventTarget::from(target.native.clone());
                let mut cursor = 0;
                loop {
                    let candidate = events
                        .registry
                        .try_borrow()
                        .map_err(|_| DocumentScriptNativeError::Busy)?
                        .next_guest_registration(
                            &target,
                            event_type,
                            options.capture,
                            &mut cursor,
                            &owner.cancellation,
                        )?;
                    let Some((registration, candidate)) = candidate else {
                        break;
                    };
                    events.step()?;
                    let existing = scope.open_callback_object(&candidate.object)?;
                    if scope.strict_equals(&value, &existing)? {
                        return Ok(registration);
                    }
                }
                let backing = events.reserve(&owner, size_of::<RegisteredGuestListener>())?;
                let object = scope
                    .retain_callback_object(observed, |bytes| events.reserve(&owner, bytes))?;
                let code = events
                    .next_callback
                    .get()
                    .checked_add(1)
                    .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
                let callback = DomEventCallback::guest(Rc::new(RegisteredGuestListener {
                    code,
                    object,
                    settings,
                    _backing: backing,
                }));
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let mut registry = events
                    .registry
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let registration = registry.add_listener(
                    &mut session.dom(),
                    target,
                    event_type,
                    callback,
                    options,
                )?;
                events.next_callback.set(code);
                Ok(registration)
            })())
        })?
    }

    /// Remove the exact registered generation from this guest registry.
    pub fn remove_guest_event_listener(
        &mut self,
        permit: &DocumentScriptPermit,
        registration: DomEventRegistrationId,
    ) -> Result<bool, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let events = self.guest_events()?;
        self.in_document(permit, |_| {
            Ok((|| {
                events.step()?;
                let mut registry = events
                    .registry
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                Ok(registry.remove_listener(registration))
            })())
        })?
    }

    /// Run the exact native invocation through the attached guest adapter.
    pub fn dispatch_guest_event(
        &mut self,
        permit: &DocumentScriptPermit,
        target: &DocumentGuestEventTarget,
        event: &DocumentGuestEvent,
    ) -> Result<DocumentGuestEventReport, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let owner = self.native.clone();
        let events = self.guest_events()?;
        self.in_document(permit, |scope| {
            Ok(events.dispatch(&owner, scope, target, event))
        })?
    }
}

#[cfg(test)]
pub(in crate::document_scripting) mod tests;

impl GuestEvents {
    fn dispatch(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'_>,
        target: &DocumentGuestEventTarget,
        event: &DocumentGuestEvent,
    ) -> Result<DocumentGuestEventReport, DocumentGuestEventError> {
        let cancel = &owner.cancellation;
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let max = self.limits.dispatch.failure_records;
        let bytes = size_of::<DocumentGuestListenerException>()
            .checked_add(self.exception_root_bytes)
            .and_then(|bytes| bytes.checked_mul(max))
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let backing = self.reserve(owner, bytes)?;
        let mut exceptions = Vec::new();
        exceptions
            .try_reserve_exact(max)
            .map_err(|_| Error::Allocation)?;
        if exceptions.capacity() != max {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let receiver = target.object(scope)?;
        let argument = event.object(scope)?;
        let mut invocation = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut registry = self
                .registry
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            registry.prepare_dispatch(
                &mut session.dom(),
                target.native.clone(),
                &event.native,
                cancel,
            )?
        };
        loop {
            self.step()?;
            let outcome = {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                let mut registry = self
                    .registry
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                invocation.advance_with_host(
                    &mut session.dom(),
                    &mut registry,
                    &mut (),
                    self.limits.dispatch,
                    cancel,
                    DispatchHost::Guest,
                )?
            };
            let mut selected = match outcome {
                DomEventDispatchOutcome::Complete(receipt) => {
                    return Ok(DocumentGuestEventReport {
                        receipt,
                        exceptions,
                        _backing: backing,
                    });
                }
                DomEventDispatchOutcome::HostInvocation(selected) => selected,
                DomEventDispatchOutcome::Retained(resource) => {
                    return Err(DomEventDispatchError::Admission(resource).into());
                }
                DomEventDispatchOutcome::RetainedHostUnavailable => {
                    return Err(DocumentGuestEventError::HostRouteUnavailable);
                }
                DomEventDispatchOutcome::RetainedSourceOperation(operation) => {
                    return Err(DocumentGuestEventError::SourcePending(operation));
                }
                DomEventDispatchOutcome::RetainedHostedOperation(operation) => {
                    return Err(DocumentGuestEventError::HostedPending(operation));
                }
                DomEventDispatchOutcome::RetainedParsedSynchronous { requirement } => {
                    return Err(
                        DomEventDispatchError::ParsedSynchronousPending { requirement }.into(),
                    );
                }
                DomEventDispatchOutcome::RetainedParsedAdmission(error) => {
                    return Err(DomEventDispatchError::Dom(error).into());
                }
                DomEventDispatchOutcome::RetainedHostedAuthority(error) => {
                    return Err(DomEventDispatchError::Hosted(error).into());
                }
                DomEventDispatchOutcome::RetainedForeignOwner => {
                    return Err(DomEventDispatchError::Dom(DomError::ForeignOwner).into());
                }
                DomEventDispatchOutcome::RetainedForeignRegistry => {
                    return Err(DomEventDispatchError::WrongContinuation.into());
                }
                DomEventDispatchOutcome::RetainedAfterUnwind => {
                    return Err(DomEventDispatchError::Abandoned.into());
                }
            };
            let key = selected.key();
            let result = (|| {
                let callback = selected
                    .selected_guest()
                    .ok_or(DocumentGuestEventError::RegistryState)?;
                let authority = callback.object.associated_realm();
                authority.check_owner_active()?;
                if callback.settings != attachment.target
                    || authority.identity() != attachment.realm.identity()
                    || scope.realm_authority()?.identity() != authority.identity()
                {
                    return Err(DocumentGuestEventError::CallbackRealmUnavailable);
                }
                {
                    let mut session = owner
                        .session
                        .try_borrow_mut()
                        .map_err(|_| DocumentScriptNativeError::Busy)?;
                    if owner.observe(attachment, &mut session)?.status()
                        != HtmlScriptingStatus::Enabled
                    {
                        return Err(DocumentScriptNativeError::Revoked.into());
                    }
                }
                self.step()?;
                let function = scope.open_callback_object(&callback.object)?;
                match scope.call(&function, &receiver, std::slice::from_ref(&argument)) {
                    Ok(_) => Ok(None),
                    Err(Error::Exception(exception)) => Ok(Some(DocumentGuestListenerException {
                        registration: selected.registration(),
                        realm: authority.identity(),
                        exception,
                    })),
                    Err(error) => Err(error.into()),
                }
            })();
            let registry = self
                .registry
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            match result {
                Ok(Some(exception)) => {
                    if exceptions.len() == max {
                        return Err(DocumentScriptNativeError::StorageLimit.into());
                    }
                    exceptions.push(exception);
                    selected.complete(&registry, Err(DomEventCallbackError { code: key }))?;
                }
                Ok(None) => selected.complete(&registry, Ok(()))?,
                Err(error) => {
                    selected.refuse(&registry, DomEventHostError { code: key })?;
                    return Err(error);
                }
            }
        }
    }
}
