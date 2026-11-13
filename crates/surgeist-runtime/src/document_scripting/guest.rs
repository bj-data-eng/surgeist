//! Registered guest callbacks for ordinary authored standalone dispatch.

mod csp_task;
mod element_task;
mod interfaces;
pub(super) use interfaces::{ActiveConstructor, ActiveReactionQueue, PendingConstructor};
mod listeners;
mod reports;
mod source_event;
mod wrapper;

use super::*;
use crate::dom_event::{
    DispatchHost, DomEventCallback, DomEventCallbackError, DomEventComposedPath,
    DomEventConstructionError, DomEventCreationObservation, DomEventDispatch,
    DomEventDispatchError, DomEventDispatchLimits, DomEventDispatchOutcome,
    DomEventDispatchReceipt, DomEventDispatcher, DomEventHostError, DomEventHostInvocation,
    DomEventInit, DomEventListenerOptions, DomEventPhase, DomEventRegistrationId, DomEventTarget,
    DomEventTimestampUnavailable, DomEventType, DomStandaloneEventTarget, NativeDomEvent,
};
pub use csp_task::DocumentCspViolationProgress;
use interfaces::PublicEventBindings;
pub(super) use interfaces::creation_document_registry;
#[cfg(test)]
pub(super) use interfaces::{
    document_registry_identity, missing_pins_refuse_without_native_publication,
};
pub(crate) use reports::QueuedViolation;
use reports::{DirectExceptions, ExceptionRoots};
pub(super) use reports::{GuestExceptionReports, ViolationQueue};
use surgeist_bindings_js::{
    Exception, NativeObjectToken, ObjectPrototype, RetainedCallbackObject, Root, Value,
};
use surgeist_dom::{DomNodeRetirementLimits, DomSourceLimits};

/// Guest adapter limits in addition to the original registry and Hosted limits.
#[derive(Clone, Copy, Debug)]
pub struct DocumentGuestEventLimits {
    /// Maximum retained guest callback, projection and report backing.
    pub storage_bytes: usize,
    /// Cumulative guest adapter work.
    pub work: u64,
    /// Pending listener and constructor exceptions, including slots reserved
    /// by currently active dispatches and constructors.
    pub pending_exception_reports: usize,
    /// Native path, callback, failure and work limits for one advance.
    pub dispatch: DomEventDispatchLimits,
    /// Canonical DOM storage and work for retained ordinary-node watches.
    pub node_retirement: DomNodeRetirementLimits,
    /// Canonical Source limits for a guest-initiated edit.
    pub source_edits: DomSourceLimits,
}

/// Failure before successful guest dispatch completion.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentGuestEventError {
    /// Exact attached host or binding refusal.
    Document(DocumentScriptError),
    /// Event or standalone target construction refused.
    Construction(DomEventConstructionError),
    /// No original native Event timestamp is available.
    Timestamp(crate::DomEventTimestampUnavailable),
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

impl From<DomError> for DocumentGuestEventError {
    fn from(error: DomError) -> Self {
        DocumentScriptNativeError::from(error).into()
    }
}

impl From<DomHostedError> for DocumentGuestEventError {
    fn from(error: DomHostedError) -> Self {
        DocumentScriptNativeError::from(DomHostedRuntimeError::Dom(error)).into()
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

struct GuestTargetProjection {
    native: DomStandaloneEventTarget,
    lifetime: Option<Rc<GuestTargetLifetime>>,
}

impl Drop for GuestTargetProjection {
    fn drop(&mut self) {
        if let Some(lifetime) = &self.lifetime {
            lifetime.retired.set(true);
            lifetime.events.retired_targets.set(true);
        }
    }
}

struct GuestTargetLifetime {
    retired: Cell<bool>,
    events: Rc<GuestFixedBacking>,
    _backing: GuestBacking,
}

struct GuestFixedBacking {
    retired_targets: Cell<bool>,
    _retention: DomHostedRuntimeRetention,
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

// A borrowed Bindings scope cannot extract this private native authority.
pub(crate) struct GuestEventProjection(NativeDomEvent);

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

    /// Live native observations and propagation/cancellation controls.
    pub fn native(&self) -> DocumentGuestEventView<'_> {
        DocumentGuestEventView {
            event: &self.native,
        }
    }
}

/// Borrowed native Event observations and controls. Target-changing
/// operations require the owning script host.
pub struct DocumentGuestEventView<'event> {
    event: &'event NativeDomEvent,
}

impl DocumentGuestEventView<'_> {
    /// Exact native Event type.
    pub fn event_type(&self) -> DomEventType {
        self.event.event_type()
    }

    /// Authenticated creation observation.
    pub fn creation(&self) -> Option<&DomEventCreationObservation> {
        self.event.creation()
    }

    /// Coarse relative time in milliseconds.
    pub fn time_stamp(&self) -> Result<f64, DomEventTimestampUnavailable> {
        self.event.time_stamp()
    }

    /// Current construction trust.
    pub fn is_trusted(&self) -> bool {
        self.event.is_trusted()
    }

    /// Current native Event target.
    pub fn target(&self) -> Option<DomEventTarget> {
        self.event.target()
    }

    /// Current invocation target.
    pub fn current_target(&self) -> Option<DomEventTarget> {
        self.event.current_target()
    }

    /// Legacy alias of the current target.
    pub fn src_element(&self) -> Option<DomEventTarget> {
        self.event.src_element()
    }

    /// Current invocation phase.
    pub fn phase(&self) -> DomEventPhase {
        self.event.phase()
    }

    /// Original composed path for the active invocation.
    pub fn composed_path(&self) -> DomEventComposedPath {
        self.event.composed_path()
    }

    /// Whether the Event bubbles.
    pub fn bubbles(&self) -> bool {
        self.event.bubbles()
    }

    /// Whether the Event is cancelable.
    pub fn cancelable(&self) -> bool {
        self.event.cancelable()
    }

    /// Whether the Event crosses shadow boundaries.
    pub fn composed(&self) -> bool {
        self.event.composed()
    }

    /// Whether the Event is canceled.
    pub fn default_prevented(&self) -> bool {
        self.event.default_prevented()
    }

    /// Legacy propagation flag.
    pub fn cancel_bubble(&self) -> bool {
        self.event.cancel_bubble()
    }

    /// Legacy inverse cancellation flag.
    pub fn return_value(&self) -> bool {
        self.event.return_value()
    }

    /// Request native cancellation.
    pub fn prevent_default(&self) {
        self.event.prevent_default();
    }

    /// Stop native propagation.
    pub fn stop_propagation(&self) {
        self.event.stop_propagation();
    }

    /// Stop remaining listeners in the current invocation.
    pub fn stop_immediate_propagation(&self) {
        self.event.stop_immediate_propagation();
    }

    /// Apply the native legacy propagation rule.
    pub fn set_cancel_bubble(&self, value: bool) {
        self.event.set_cancel_bubble(value);
    }

    /// Apply the native legacy cancellation rule.
    pub fn set_return_value(&self, value: bool) {
        self.event.set_return_value(value);
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

/// An exception raised by synchronous custom element construction.
#[derive(Debug)]
pub struct DocumentGuestConstructorException {
    /// Exact canonical definition selected by the factory.
    pub definition: surgeist_dom::DomRegistryDefinitionId,
    /// The selected constructor object's associated Realm.
    pub realm: RealmId,
    /// Original thrown value or the exception from native result validation.
    pub exception: Exception,
}

/// The custom-element lifecycle callback that threw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentGuestCustomElementCallback {
    /// An observed attribute reaction.
    AttributeChanged,
    /// A connection reaction.
    Connected,
    /// A disconnection reaction.
    Disconnected,
    /// A state-preserving move reaction.
    ConnectedMove,
    /// A change of owner Document.
    Adopted,
}

/// The original exception from a custom-element lifecycle callback.
#[derive(Debug)]
pub struct DocumentGuestCustomElementCallbackException {
    /// Exact definition associated with the callback.
    pub definition: surgeist_dom::DomRegistryDefinitionId,
    /// Original element receiver.
    pub receiver: surgeist_dom::NodeId,
    /// The callback that threw.
    pub callback: DocumentGuestCustomElementCallback,
    /// The callback object’s associated realm.
    pub realm: RealmId,
    /// Original thrown value.
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

pub(crate) struct GuestBacking {
    used: Rc<Cell<usize>>,
    bytes: usize,
    _canonical: DomHostedRuntimeRetention,
}

/// A retained verification decision for the original native attribute write.
pub(super) struct PendingAttribute {
    realm: RealmId,
    request: surgeist_dom::DomSourceAttributeVerificationRequest,
    reply: AttributeReply,
    parent: Option<surgeist_dom::DomSourceAttributeVerificationRequest>,
    interrupted_parent: Option<Box<Self>>,
    _header: GuestBacking,
    reaction_failure: Option<HostFailureSlot<DocumentGuestEventError, GuestBacking>>,
    shadow_reactions: Option<surgeist_dom::DomSynchronousReactionQueue>,
}

enum AttributeReply {
    Abandon,
    OriginalValue,
    Mismatch(crate::trusted_types_csp::AttributeMismatchProgress),
    Converted {
        result: surgeist_bindings_js::conversion::Converted<
            Option<surgeist_bindings_js::TrustedTypeData>,
        >,
        // Keeps the original setup debit alive if the engine retires first.
        _account: Rc<interfaces::Callbacks>,
    },
    Abrupt(Error),
    Accepted {
        abrupt: Option<Error>,
    },
}

impl Drop for PendingAttribute {
    fn drop(&mut self) {
        let mut parent = self.interrupted_parent.take();
        while let Some(mut frame) = parent {
            parent = frame.interrupted_parent.take();
        }
    }
}

impl NativeOwner {
    pub(super) fn reconcile_pending_attribute(&self) -> Result<(), DocumentScriptNativeError> {
        let mut pending = self
            .pending_attribute
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        if pending.is_none() {
            return Ok(());
        }
        let current = self
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .dom()
            .pending_source_edit();
        while let Some(mut frame) = pending.take() {
            if let Some(status) =
                current.filter(|status| status.operation() == frame.request.operation())
            {
                if status.cancellation_requested()
                    && !matches!(frame.reply, AttributeReply::Accepted { .. })
                {
                    frame.reply = AttributeReply::Abandon;
                }
                *pending = Some(frame);
                break;
            }
            *pending = frame.interrupted_parent.take();
        }
        Ok(())
    }

    fn retain_interrupted_attribute(&self, parent: Box<PendingAttribute>) {
        let Ok(mut pending) = self.pending_attribute.try_borrow_mut() else {
            return;
        };
        let Some(mut child) = pending.as_deref_mut() else {
            return;
        };
        while let Some(ref mut next) = child.interrupted_parent {
            child = next;
        }
        if child.parent == Some(parent.request) {
            child.interrupted_parent = Some(parent);
        }
    }
}

impl GuestBacking {
    fn split_off(&mut self, bytes: usize) -> Result<Self, DocumentScriptNativeError> {
        let remaining = self
            .bytes
            .checked_sub(bytes)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let canonical = self._canonical.split_off(bytes)?;
        self.bytes = remaining;
        Ok(Self {
            used: self.used.clone(),
            bytes,
            _canonical: canonical,
        })
    }
}

impl Drop for GuestBacking {
    fn drop(&mut self) {
        self.used.set(self.used.get() - self.bytes);
    }
}

struct TracedListener {
    realm: RealmAuthority,
    target: NativeObjectToken<GuestTargetProjection>,
    slot: u32,
    lifetime: Rc<GuestTargetLifetime>,
}

enum RegisteredListenerObject {
    Retained {
        callback: RefCell<Option<RetainedCallbackObject>>,
        lifetime: Option<Rc<interfaces::NodeLifetime>>,
    },
    Traced(TracedListener),
}

pub(crate) struct RegisteredGuestListener {
    pub(crate) code: u64,
    object: RegisteredListenerObject,
    registration_window: DomHostedTopLevel,
    _backing: GuestBacking,
}

impl RegisteredGuestListener {
    pub(crate) fn target_retired(&self) -> bool {
        match &self.object {
            RegisteredListenerObject::Traced(traced) => traced.lifetime.retired.get(),
            RegisteredListenerObject::Retained { lifetime, .. } => {
                lifetime.as_ref().is_some_and(|target| target.is_retired())
            }
        }
    }
}

pub(super) struct GuestEvents {
    registry: RefCell<DomEventDispatcher<()>>,
    limits: DocumentGuestEventLimits,
    budget: GuestBudget,
    next_callback: Cell<u64>,
    exception_root_bytes: usize,
    reports: Rc<GuestExceptionReports>,
    refusal: reports::DispatchFailure,
    window_event: Rc<window_event::WindowEvent>,
    target_prototype: OnceCell<RetainedCallbackObject>,
    public: Option<PublicEventBindings>,
    backing: Rc<GuestFixedBacking>,
}

struct GuestBudget {
    used: Rc<Cell<usize>>,
    work: Cell<u64>,
}

pub(super) struct OriginAdmission {
    backing: RefCell<GuestBacking>,
    requests: Cell<u64>,
}

impl OriginAdmission {
    fn take(
        &self,
        events: &GuestEvents,
        bytes: usize,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        let remaining = self
            .requests
            .get()
            .checked_sub(1)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        events.step()?;
        let backing = self
            .backing
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .split_off(bytes)?;
        self.requests.set(remaining);
        Ok(backing)
    }

    fn finish(self) -> Result<(), DocumentScriptNativeError> {
        if self.requests.get() != 0 || self.backing.into_inner().bytes != 0 {
            return Err(DocumentScriptNativeError::RepresentationExhausted);
        }
        Ok(())
    }
}

impl GuestBudget {
    fn charge_work(&self, limit: u64, units: u64) -> Result<(), DocumentGuestEventError> {
        let work = self
            .work
            .get()
            .checked_add(units)
            .ok_or(DocumentGuestEventError::WorkLimit)?;
        if work > limit {
            return Err(DocumentGuestEventError::WorkLimit);
        }
        self.work.set(work);
        Ok(())
    }

    fn reserve(
        &self,
        owner: &NativeOwner,
        limits: DocumentGuestEventLimits,
        bytes: usize,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        self.reserve_with(limits, bytes, || {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            session
                .hosted_execution()
                .retain_script_payload(&mut session.dom(), bytes, &owner.cancellation)
                .map_err(DocumentScriptNativeError::from)
        })
    }

    fn reserve_with(
        &self,
        limits: DocumentGuestEventLimits,
        bytes: usize,
        retain: impl FnOnce() -> Result<DomHostedRuntimeRetention, DocumentScriptNativeError>,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        self.charge_work(limits.work, 1)?;
        let total = self
            .used
            .get()
            .checked_add(bytes)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        if total > limits.storage_bytes {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let canonical = retain()?;
        self.used.set(total);
        Ok(GuestBacking {
            used: self.used.clone(),
            bytes,
            _canonical: canonical,
        })
    }
}

impl GuestEvents {
    pub(super) fn installs_public_bindings(&self) -> bool {
        self.public.is_some()
    }

    pub(super) fn prepare_origin(
        &self,
        owner: &NativeOwner,
    ) -> Result<Option<OriginAdmission>, DocumentScriptError> {
        let Some(public) = &self.public else {
            return Ok(None);
        };
        let (bytes, requests) = public.origin_installation_cost();
        // Reserve canonical backing before V8 can retain a callback capture.
        // The private install has no guest entry and spends its charged work
        // as each extent is transferred to its final owner.
        if self
            .budget
            .work
            .get()
            .checked_add(requests)
            .and_then(|work| work.checked_add(1))
            .is_none_or(|work| work > self.limits.work)
        {
            return Err(DocumentScriptNativeError::CreationRealmWorkLimit.into());
        }
        let backing = self.reserve(owner, bytes).map_err(creation_realm_error)?;
        Ok(Some(OriginAdmission {
            backing: RefCell::new(backing),
            requests: Cell::new(requests),
        }))
    }

    pub(super) fn install_origin(
        &self,
        process: &mut Process,
        realm: &Realm,
        dom_exception_limits: DomExceptionLimits,
        origin: (DomHostedGlobalId, DocumentId, surgeist_dom::DomRegistryId),
        admission: OriginAdmission,
    ) -> Result<(), DocumentScriptError> {
        let public = self.public.as_ref().ok_or(Error::RegistryConflict)?;
        process
            .execution()
            .in_realm(realm, |scope| {
                Ok(public.install_origin(scope, self, dom_exception_limits, origin, admission))
            })?
            .map_err(creation_realm_error)
    }

    fn step(&self) -> Result<(), DocumentGuestEventError> {
        self.charge_work(1)
    }

    fn charge_work(&self, units: u64) -> Result<(), DocumentGuestEventError> {
        self.budget.charge_work(self.limits.work, units)
    }

    fn reserve(
        &self,
        owner: &NativeOwner,
        bytes: usize,
    ) -> Result<GuestBacking, DocumentGuestEventError> {
        self.budget.reserve(owner, self.limits, bytes)
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

fn creation_realm_error(error: DocumentGuestEventError) -> DocumentScriptError {
    match error {
        DocumentGuestEventError::Document(error) => error,
        DocumentGuestEventError::WorkLimit => {
            DocumentScriptNativeError::CreationRealmWorkLimit.into()
        }
        _ => Error::BackendUnavailable.into(),
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
    /// Attach an engine, take the bound native registry, and install Event
    /// and EventTarget in the private Window before publishing it.
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
        let mut script = self.prepare_engine(target, hooks)?;
        let mut admission = GuestEventAdmission::prepare(
            &self.native,
            input,
            limits,
            script.retained_bytes.checked_add(limits.storage_bytes),
            &script.authority,
        )?;
        admission.public = Some(PublicEventBindings::install(
            &mut script,
            &self.native,
            &admission,
        )?);
        let engine = script.publish(&self.native, policy)?;
        engine.registration.observe_custom_element_reactions()?;
        Ok((engine, admission))
    }
}

struct GuestEventAdmission {
    limits: DocumentGuestEventLimits,
    budget: GuestBudget,
    reports: Rc<GuestExceptionReports>,
    refusal: reports::DispatchFailure,
    backing: DomHostedRuntimeRetention,
    public: Option<PublicEventBindings>,
}

impl GuestEventAdmission {
    fn prepare(
        owner: &NativeOwner,
        input: &DomEventDispatcher<()>,
        limits: DocumentGuestEventLimits,
        total: Option<usize>,
        authority: &RealmAuthority,
    ) -> Result<Self, DocumentGuestEventError> {
        let fixed =
            size_of::<GuestEvents>() + size_of::<Cell<usize>>() + size_of::<GuestFixedBacking>();
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
            budget: GuestBudget {
                used: Rc::new(Cell::new(required)),
                work: Cell::new(0),
            },
            reports,
            refusal,
            backing,
            public: None,
        })
    }

    fn install(
        self,
        engine: &mut ScriptEngine,
        registry: DomEventDispatcher<()>,
    ) -> Rc<GuestExceptionReports> {
        let Self {
            limits,
            budget,
            reports,
            refusal,
            backing,
            public,
        } = self;
        let exception_root_bytes = engine.process.managed_object_storage_bytes();
        let events = Rc::new(GuestEvents {
            registry: RefCell::new(registry),
            limits,
            budget,
            next_callback: Cell::new(0),
            exception_root_bytes,
            reports: reports.clone(),
            refusal,
            window_event: engine.window_event.clone(),
            target_prototype: OnceCell::new(),
            public,
            backing: Rc::new(GuestFixedBacking {
                retired_targets: Cell::new(false),
                _retention: backing,
            }),
        });
        if let Some(public) = &events.public {
            public.activate(&events);
        }
        engine.guest_events = Some(events);
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

    /// Take the earliest original listener exception from an attached dispatch. This observation can be taken after terminal script refusal.
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

    /// Take the next reported custom element constructor exception.
    pub fn take_guest_constructor_exception(
        &self,
        permit: &DocumentScriptPermit,
    ) -> Result<Option<DocumentGuestConstructorException>, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let reports = self
            .guest_exception_reports
            .as_ref()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        Ok(reports.take_constructor())
    }

    /// Take the next reported custom-element lifecycle callback exception.
    pub fn take_guest_custom_element_callback_exception(
        &self,
        permit: &DocumentScriptPermit,
    ) -> Result<Option<DocumentGuestCustomElementCallbackException>, DocumentGuestEventError> {
        self.check_permit(permit)?;
        let reports = self
            .guest_exception_reports
            .as_ref()
            .ok_or(DocumentGuestEventError::RegistryState)?;
        Ok(reports.take_custom_element_callback())
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
                    events.target_projection(&owner, native.clone())?,
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
                let object = match &events.public {
                    Some(public) => {
                        let prototype = public.event_prototype(scope)?;
                        let trusted = public.trusted_getter(scope)?;
                        let object = events.project(
                            &owner,
                            scope,
                            ObjectPrototype::Object(&prototype),
                            GuestEventProjection(native.clone()),
                        )?;
                        let value = scope.open_callback_object(&object)?;
                        interfaces::expose_trust(scope, &value, &trusted)?;
                        object
                    }
                    None => events.project(
                        &owner,
                        scope,
                        ObjectPrototype::Default,
                        GuestEventProjection(native.clone()),
                    )?,
                };
                Ok(DocumentGuestEvent { native, object })
            })())
        })?
    }

    /// Initialize an idle guest Event through its original attached owner.
    /// The native target and its traced guest projection clear together.
    pub fn initialize_guest_event(
        &mut self,
        permit: &DocumentScriptPermit,
        event: &DocumentGuestEvent,
        event_type: &[u16],
        bubbles: bool,
        cancelable: bool,
    ) -> Result<(), DocumentGuestEventError> {
        self.check_permit(permit)?;
        let owner = self.native.clone();
        let events = self.guest_events()?;
        self.in_document(permit, |scope| {
            Ok((|| {
                let receiver = event.object(scope)?;
                events.initialize_projection(
                    &owner,
                    scope,
                    &receiver,
                    &event.native,
                    event_type,
                    [bubbles, cancelable],
                )
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
                let receiver = target.object(scope)?;
                let native_target = DomEventTarget::from(target.native.clone());
                let key = listeners::ListenerMatch {
                    receiver: &receiver,
                    target: &native_target,
                    event_type,
                    callback: &value,
                    capture: options.capture,
                };
                if let Some(registration) = events.find_listener(&owner, scope, key)? {
                    return Ok(registration);
                }
                events.register_retained(&owner, scope, key, options, observed, None)
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
        self.in_document(permit, |scope| {
            Ok(events.remove_registration(scope, registration))
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
        let invocation = self.prepare(owner, target.native.clone().into(), &event.native)?;
        self.run(
            owner,
            scope,
            invocation,
            &receiver,
            &argument,
            |scope, exception| exceptions.publish(scope, &mut roots, exception),
        )
    }

    fn initialize_projection<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        receiver: &Value<'scope>,
        event: &NativeDomEvent,
        event_type: &[u16],
        [bubbles, cancelable]: [bool; 2],
    ) -> Result<(), DocumentGuestEventError> {
        scope.check_active()?;
        if event.is_dispatching() {
            return Ok(());
        }
        self.step()?;
        let attachment = owner
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        if scope.realm_authority()?.identity() != attachment.realm.identity() {
            return Err(DocumentScriptNativeError::ForeignPermit.into());
        }
        {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            if owner.observe(attachment, &mut session)?.status() != HtmlScriptingStatus::Enabled {
                return Err(DocumentScriptNativeError::Revoked.into());
            }
            event.init_event(
                &mut session.dom(),
                event_type,
                bubbles,
                cancelable,
                &owner.cancellation,
            )?;
        }
        if let Err(error) = scope.set_native_object_reference::<GuestEventProjection>(
            receiver,
            interfaces::EVENT_TARGET_REFERENCE,
            None,
        ) {
            return refuse(scope, &self.refusal, DocumentGuestEventError::from(error))
                .map_err(Into::into);
        }
        Ok(())
    }

    fn prepare(
        &self,
        owner: &NativeOwner,
        target: DomEventTarget,
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
        Ok(registry.prepare_dispatch(&mut session.dom(), target, event, &owner.cancellation)?)
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
        let mut projection = if self.public.is_some() {
            self.step()?;
            Some((
                scope
                    .native_object::<GuestEventProjection>(argument)?
                    .ok_or(Error::ExpectedNativeObject)?,
                wrapper::event_receiver(owner, self, scope, receiver)?
                    .ok_or(Error::ExpectedNativeObject)?
                    .target(),
            ))
        } else {
            None
        };
        let dispatch_target = projection.as_ref().map(|(_, target)| target.clone());
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
            let projected = if let Some((event, target)) = &projection
                && event.payload().0.target().as_ref() == Some(target)
            {
                projection = None;
                scope.set_native_object_reference::<GuestEventProjection>(
                    argument,
                    interfaces::EVENT_TARGET_REFERENCE,
                    Some(receiver),
                )
            } else {
                Ok(())
            };
            let projection_failed = projected.is_err();
            let mut selected = match outcome {
                DomEventDispatchOutcome::Complete(receipt) => {
                    if let Err(error) = projected {
                        return refuse(scope, &self.refusal, DocumentGuestEventError::from(error))
                            .map_err(Into::into);
                    }
                    return Ok(receipt);
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
            let projection = projected
                .map_err(DocumentGuestEventError::from)
                .and_then(|()| {
                    let Some(dispatch_target) = &dispatch_target else {
                        return Ok(None);
                    };
                    let current = selected
                        .event()
                        .current_target()
                        .ok_or(Error::RegistryConflict)?;
                    if &current == dispatch_target {
                        return Ok(None);
                    }
                    wrapper::event_target_value(owner, self, scope, argument, &current).map(Some)
                });
            let (current, recipient_error) = match projection {
                Ok(current) => (current, None),
                Err(error) => (None, Some(error)),
            };
            let values = match recipient_error {
                Some(error) => Err(error),
                None => Ok((argument, current.as_ref().unwrap_or(receiver))),
            };
            let result = self.invoke_selected(owner, scope, &selected, values, None, &mut report);
            if let Err(error) = self.report_selected(&mut selected, result) {
                if projection_failed {
                    return refuse(scope, &self.refusal, error).map_err(Into::into);
                }
                return Err(error);
            }
        }
    }

    fn invoke_selected<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        selected: &DomEventHostInvocation,
        values: Result<
            (
                impl std::borrow::Borrow<Value<'scope>>,
                impl std::borrow::Borrow<Value<'scope>>,
            ),
            DocumentGuestEventError,
        >,
        source: Option<&source_event::ActiveAttachedEvent<'_>>,
        report: impl FnOnce(
            &mut Scope<'scope>,
            DocumentGuestListenerException,
        ) -> Result<(), DocumentGuestEventError>,
    ) -> Result<Result<(), DomEventCallbackError>, DocumentGuestEventError> {
        let callback = selected.selected_guest();
        let prepared = (|| {
            let (argument, receiver) = values?;
            let attachment = owner
                .attachment
                .get()
                .ok_or(DocumentScriptNativeError::Revoked)?;
            let callback = callback.ok_or(DocumentGuestEventError::RegistryState)?;
            let authority = callback.realm()?;
            authority.check_owner_active()?;
            if callback.registration_window != attachment.target
                || authority.identity() != attachment.realm.identity()
                || scope.realm_authority()?.identity() != authority.identity()
            {
                return Err(DocumentGuestEventError::CallbackRealmUnavailable);
            }
            if source.is_none() {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                owner.qualify_guest_dispatch(attachment, &mut session)?;
            }
            self.step()?;
            let object = callback.open(scope, std::borrow::Borrow::borrow(&receiver))?;
            Ok((argument, receiver, object, authority.identity()))
        })();
        // Native selection removes the once registration before guest entry.
        // Keep its local callable on success, and clear the edge on refusal too.
        let cleared = match callback {
            Some(callback) if selected.once_removed() => callback.release(scope),
            _ => Ok(()),
        };
        let (argument, receiver, object, realm) = prepared?;
        cleared?;
        let argument = std::borrow::Borrow::borrow(&argument);
        let receiver = std::borrow::Borrow::borrow(&receiver);
        let _current_event =
            (!selected.in_shadow_tree()).then(|| self.window_event.enter(scope.retain(argument)));
        let arguments = std::slice::from_ref(argument);
        let called = if scope.is_callable(&object)? {
            scope.call(&object, receiver, arguments)
        } else {
            self.step()?;
            scope.call_object_operation(&object, &b"handleEvent".map(u16::from), arguments)
        };
        match called {
            Ok(_) => Ok(Ok(())),
            Err(Error::Exception(exception)) => {
                report(
                    scope,
                    DocumentGuestListenerException {
                        registration: selected.registration(),
                        realm,
                        exception,
                    },
                )?;
                Ok(Err(DomEventCallbackError {
                    code: selected.key(),
                }))
            }
            Err(error) => Err(error.into()),
        }
    }

    fn report_selected(
        &self,
        selected: &mut DomEventHostInvocation,
        result: Result<Result<(), DomEventCallbackError>, DocumentGuestEventError>,
    ) -> Result<(), DocumentGuestEventError> {
        let registry = self
            .registry
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        match result {
            Ok(result) => Ok(selected.complete(&registry, result)?),
            Err(error) => {
                selected.refuse(
                    &registry,
                    DomEventHostError {
                        code: selected.key(),
                    },
                )?;
                Err(error)
            }
        }
    }
}
