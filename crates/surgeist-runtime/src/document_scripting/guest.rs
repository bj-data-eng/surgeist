//! Registered guest callbacks for ordinary authored standalone dispatch.

mod reports;
mod wrapper;

use super::*;
use crate::dom_event::{
    DispatchHost, DomEventCallback, DomEventCallbackError, DomEventConstructionError,
    DomEventDispatch, DomEventDispatchError, DomEventDispatchLimits, DomEventDispatchOutcome,
    DomEventDispatchReceipt, DomEventDispatcher, DomEventHostError, DomEventInit,
    DomEventListenerOptions, DomEventRegistrationId, DomEventTarget, DomStandaloneEventTarget,
    NativeDomEvent,
};
pub(super) use reports::GuestExceptionReports;
use reports::{DirectExceptions, ExceptionRoots};
use surgeist_bindings_js::{Exception, ObjectPrototype, RetainedCallbackObject, Root, Value};

/// Guest adapter limits in addition to the original registry and Hosted limits.
#[derive(Clone, Copy, Debug)]
pub struct DocumentGuestEventLimits {
    /// Maximum retained guest callback, projection and report backing.
    pub storage_bytes: usize,
    /// Cumulative guest adapter work.
    pub work: u64,
    /// Pending ordinary exceptions from guest-initiated dispatches, including
    /// slots reserved by currently active dispatches.
    pub pending_exception_reports: usize,
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
    /// The supplied registry must first claim the exact native DOM.
    RegistryUnbound,
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
    exceptions: Option<Box<DirectExceptions>>,
}

impl DocumentGuestEventReport {
    /// Native completion, including the independent cancellation result.
    pub fn receipt(&self) -> &DomEventDispatchReceipt {
        &self.receipt
    }

    /// Original guest exception observations, in invocation order.
    pub fn exceptions(&self) -> &[DocumentGuestListenerException] {
        self.exceptions
            .as_deref()
            .map_or(&[], DirectExceptions::observations)
    }
}

/// Refused direct dispatch with its original cause and any completed reports.
pub struct DocumentGuestEventDispatchFailure {
    /// Exact failure selected by the attached host or native dispatcher.
    pub error: DocumentGuestEventError,
    exceptions: Option<Box<DirectExceptions>>,
}

impl DocumentGuestEventDispatchFailure {
    /// Ordinary listener exceptions reported before the refusal, in order.
    pub fn exceptions(&self) -> &[DocumentGuestListenerException] {
        self.exceptions
            .as_deref()
            .map_or(&[], DirectExceptions::observations)
    }
}

impl fmt::Debug for DocumentGuestEventDispatchFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentGuestEventDispatchFailure")
            .field("error", &self.error)
            .field("exceptions", &self.exceptions())
            .finish()
    }
}

impl fmt::Display for DocumentGuestEventDispatchFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for DocumentGuestEventDispatchFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl From<DocumentGuestEventError> for DocumentGuestEventDispatchFailure {
    fn from(error: DocumentGuestEventError) -> Self {
        Self {
            error,
            exceptions: None,
        }
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
    registration_window: DomHostedTopLevel,
    _backing: GuestBacking,
}

pub(super) struct GuestEvents {
    registry: RefCell<DomEventDispatcher<()>>,
    limits: DocumentGuestEventLimits,
    used: Rc<Cell<usize>>,
    work: Cell<u64>,
    next_callback: Cell<u64>,
    exception_root_bytes: usize,
    reports: Rc<GuestExceptionReports>,
    refusal: reports::DispatchFailure,
    window_event: Rc<window_event::WindowEvent>,
    target_prototype: OnceCell<RetainedCallbackObject>,
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

    fn project<'scope, T: 'static>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        prototype: ObjectPrototype<'_, 'scope>,
        native: T,
    ) -> Result<RetainedCallbackObject, DocumentGuestEventError> {
        self.step()?;
        scope.project_retained_native_object(prototype, native, |bytes| self.reserve(owner, bytes))
    }
}

/// Refused Window activation with the original unpublished native owner.
#[derive(Debug)]
pub struct EnableDocumentGuestEventsFailure {
    /// Original native preparation, available for a new attempt.
    pub owner: PreparedDocumentScriptHost,
    /// Exact native, registry, or bindings refusal.
    pub error: DocumentGuestEventError,
}

impl PreparedDocumentScriptHost {
    /// Attach an engine and take the already-bound native event registry.
    /// Refusal returns the unpublished owner and restores the registry slot.
    pub fn enable_with_guest_event_registry(
        self,
        target: DomHostedTopLevel,
        policy: HtmlScriptingPolicy,
        hooks: DocumentScriptHooks<'_>,
        registry: &mut Option<DomEventDispatcher<()>>,
        limits: DocumentGuestEventLimits,
    ) -> Result<DocumentScriptHost, EnableDocumentGuestEventsFailure> {
        let Some(input) = registry.take() else {
            return Err(EnableDocumentGuestEventsFailure {
                owner: self,
                error: DocumentGuestEventError::RegistryState,
            });
        };
        match self.enable_guest_engine(target, policy, hooks, &input, limits) {
            Ok((mut engine, admission)) => {
                let reports = admission.install(&mut engine, input);
                Ok(self.into_host(engine, Some(reports)))
            }
            Err(error) => {
                *registry = Some(input);
                Err(EnableDocumentGuestEventsFailure { owner: self, error })
            }
        }
    }

    fn enable_guest_engine(
        &self,
        target: DomHostedTopLevel,
        policy: HtmlScriptingPolicy,
        hooks: DocumentScriptHooks<'_>,
        input: &DomEventDispatcher<()>,
        limits: DocumentGuestEventLimits,
    ) -> Result<(ScriptEngine, GuestEventAdmission), DocumentGuestEventError> {
        if !input.is_bound() {
            return Err(DocumentGuestEventError::RegistryUnbound);
        }
        {
            let mut session = self
                .native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            input.qualify_guest_registry(&session.dom())?;
        }
        let script = self.prepare_engine(target, hooks)?;
        let admission = GuestEventAdmission::prepare(
            &self.native,
            input,
            limits,
            script.retained_bytes.checked_add(limits.storage_bytes),
            &script.authority,
        )?;
        let engine = script.publish(&self.native, policy)?;
        Ok((engine, admission))
    }
}

struct GuestEventAdmission {
    limits: DocumentGuestEventLimits,
    required: usize,
    reports: Rc<GuestExceptionReports>,
    refusal: reports::DispatchFailure,
    backing: DomHostedRuntimeRetention,
}

impl GuestEventAdmission {
    fn prepare(
        owner: &NativeOwner,
        input: &DomEventDispatcher<()>,
        limits: DocumentGuestEventLimits,
        total: Option<usize>,
        authority: &RealmAuthority,
    ) -> Result<Self, DocumentGuestEventError> {
        let fixed = size_of::<GuestEvents>() + size_of::<Cell<usize>>();
        let report_bytes = GuestExceptionReports::storage_bytes(limits.pending_exception_reports)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let refusal_bytes = reports::DispatchFailure::storage_bytes();
        let required = fixed
            .checked_add(report_bytes)
            .and_then(|bytes| bytes.checked_add(refusal_bytes))
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        if limits.storage_bytes < required || total.is_none_or(|total| total > owner.storage_limit)
        {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let (backing, report_backing, refusal_backing) = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            input.qualify_guest_registry(&session.dom())?;
            let hosted = session.hosted_execution();
            let mut dom = session.dom();
            let mut retain = |bytes| {
                hosted
                    .retain_script_payload(&mut dom, bytes, &owner.cancellation)
                    .map_err(DocumentScriptNativeError::from)
            };
            (
                retain(fixed)?,
                retain(report_bytes)?,
                retain(refusal_bytes)?,
            )
        };
        let reports = Rc::new(GuestExceptionReports::new(
            limits.pending_exception_reports,
            report_backing,
        )?);
        let refusal = reports::DispatchFailure::new(authority, refusal_backing)?;
        Ok(Self {
            limits,
            required,
            reports,
            refusal,
            backing,
        })
    }

    fn install(
        self,
        engine: &mut ScriptEngine,
        registry: DomEventDispatcher<()>,
    ) -> Rc<GuestExceptionReports> {
        let Self {
            limits,
            required,
            reports,
            refusal,
            backing,
        } = self;
        let exception_root_bytes = engine.process.managed_object_storage_bytes();
        engine.guest_events = Some(Rc::new(GuestEvents {
            registry: RefCell::new(registry),
            limits,
            used: Rc::new(Cell::new(required)),
            work: Cell::new(0),
            next_callback: Cell::new(0),
            exception_root_bytes,
            reports: reports.clone(),
            refusal,
            window_event: engine.window_event.clone(),
            target_prototype: OnceCell::new(),
            _backing: backing,
        }));
        engine.retained_bytes += limits.storage_bytes;
        reports
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
        let authority = &self
            .native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?
            .realm;
        let admission = GuestEventAdmission::prepare(
            &self.native,
            input,
            limits,
            engine.retained_bytes_with(limits.storage_bytes),
            authority,
        )?;
        let registry = registry
            .take()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        self.guest_exception_reports = Some(admission.install(engine, registry));
        Ok(())
    }

    /// Take the earliest original listener exception from guest-initiated
    /// dispatch. This observation can be taken after terminal script refusal.
    pub fn take_guest_listener_exception(
        &self,
        permit: &DocumentScriptPermit,
    ) -> Result<Option<DocumentGuestListenerException>, DocumentGuestEventError> {
        let attachment = self
            .native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if permit.realm != attachment.realm.identity()
            || permit.document != attachment.target.document()
        {
            return Err(DocumentScriptNativeError::ForeignPermit.into());
        }
        let reports = self
            .guest_exception_reports
            .as_ref()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        Ok(reports.take())
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
                let prototype = events.target_prototype(&owner, scope)?;
                let object = events.project(
                    &owner,
                    scope,
                    ObjectPrototype::Object(&prototype),
                    native.clone(),
                )?;
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
                let object =
                    events.project(&owner, scope, ObjectPrototype::Default, native.clone())?;
                Ok(DocumentGuestEvent { native, object })
            })())
        })?
    }

    /// Register the exact guest object under the attached Window's authority.
    /// This Rust-hosted entry supplies the native attachment explicitly.
    pub fn add_guest_event_listener(
        &mut self,
        permit: &DocumentScriptPermit,
        target: &DocumentGuestEventTarget,
        event_type: &[u16],
        callback: &Root,
        options: DomEventListenerOptions,
    ) -> Result<DomEventRegistrationId, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let registration_window = self.attachment()?.target;
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
                    registration_window,
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
    ) -> Result<DocumentGuestEventReport, DocumentGuestEventDispatchFailure> {
        self.check_permit(permit)
            .map_err(DocumentGuestEventError::from)?;
        let owner = self.native.clone();
        let events = self.guest_events()?;
        let mut exceptions = None;
        let result = self
            .in_document(permit, |scope| {
                Ok(events.dispatch(&owner, scope, target, event, &mut exceptions))
            })
            .map_err(DocumentGuestEventError::from)
            .and_then(std::convert::identity);
        let exceptions = exceptions.filter(|bank| !bank.observations().is_empty());
        match result {
            Ok(receipt) => Ok(DocumentGuestEventReport {
                receipt,
                exceptions,
            }),
            Err(error) => Err(DocumentGuestEventDispatchFailure { error, exceptions }),
        }
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
        exceptions: &mut Option<Box<DirectExceptions>>,
    ) -> Result<DomEventDispatchReceipt, DocumentGuestEventError> {
        let max = self.limits.dispatch.failure_records;
        let bank = DirectExceptions::reserve(self, owner, max)?;
        let mut roots = ExceptionRoots::reserve(self, owner, max)?;
        // Store observations outside the engine entry, whose terminal failure
        // can override the result returned by its action.
        let exceptions = exceptions.insert(bank);
        let receiver = target.object(scope)?;
        let argument = event.object(scope)?;
        let invocation = self.prepare(owner, &target.native, &event.native)?;
        self.run(
            owner,
            scope,
            invocation,
            &receiver,
            &argument,
            |scope, exception| exceptions.publish(scope, &mut roots, exception),
        )
    }

    fn prepare(
        &self,
        owner: &NativeOwner,
        target: &DomStandaloneEventTarget,
        event: &NativeDomEvent,
    ) -> Result<DomEventDispatch, DocumentGuestEventError> {
        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let mut registry = self
            .registry
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        Ok(registry.prepare_dispatch(
            &mut session.dom(),
            target.clone(),
            event,
            &owner.cancellation,
        )?)
    }

    fn run<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        mut invocation: DomEventDispatch,
        receiver: &Value<'scope>,
        argument: &Value<'scope>,
        mut report: impl FnMut(
            &mut Scope<'scope>,
            DocumentGuestListenerException,
        ) -> Result<(), DocumentGuestEventError>,
    ) -> Result<DomEventDispatchReceipt, DocumentGuestEventError> {
        let cancel = &owner.cancellation;
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
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
                DomEventDispatchOutcome::Complete(receipt) => return Ok(receipt),
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
            let mut current_event = None;
            let result = (|| {
                let callback = selected
                    .selected_guest()
                    .ok_or(DocumentGuestEventError::RegistryState)?;
                let authority = callback.object.associated_realm();
                authority.check_owner_active()?;
                if callback.registration_window != attachment.target
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
                let object = scope.open_callback_object(&callback.object)?;
                // A standalone path item is outside any shadow tree.
                current_event = Some(self.window_event.enter(scope.retain(argument)));
                let arguments = std::slice::from_ref(argument);
                let called = if scope.is_callable(&object)? {
                    scope.call(&object, receiver, arguments)
                } else {
                    self.step()?;
                    scope.call_object_operation(&object, &b"handleEvent".map(u16::from), arguments)
                };
                match called {
                    Ok(_) => Ok(None),
                    Err(Error::Exception(exception)) => Ok(Some(DocumentGuestListenerException {
                        registration: selected.registration(),
                        realm: authority.identity(),
                        exception,
                    })),
                    Err(error) => Err(error.into()),
                }
            })()
            .and_then(|exception| match exception {
                Some(exception) => {
                    report(scope, exception)?;
                    Ok(Err(DomEventCallbackError { code: key }))
                }
                None => Ok(Ok(())),
            });
            let registry = self
                .registry
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            match result {
                Ok(result) => selected.complete(&registry, result)?,
                Err(error) => {
                    selected.refuse(&registry, DomEventHostError { code: key })?;
                    return Err(error);
                }
            }
            drop(current_event);
        }
    }
}
