//! Optional script ownership for one actual native-created hosted Document.

use std::cell::{Cell, OnceCell, RefCell};
use std::fmt;
use std::mem::{size_of, size_of_val};
use std::rc::{Rc, Weak};
use std::sync::{Arc, atomic::AtomicBool};

use surgeist_bindings_js::{
    Callback, Completion, EngineKind, Error, HostFailureSlot, Primitive, Process,
    PropertyDescriptor, PropertyKey, Realm, RealmAuthority, RealmId, RecordFailure, Scope, css,
    dom_exception,
};
pub use surgeist_bindings_js::{css::CssNamespaceLimits, dom_exception::DomExceptionLimits};
use surgeist_css::{query::media::CssMediaScripting, serialization::CssSerializationError};
use surgeist_dom::{
    DocumentId, DomDocumentScriptBinding, DomDocumentScriptRegistration, DomDocumentScriptingFacts,
    DomDocumentScriptingObservation, DomError, DomHostedError, DomHostedGlobalId,
    DomHostedRuntimeRetention, DomSnapshot,
};
use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
use surgeist_html_semantics::HtmlScriptingStatus;

use crate::{
    DomEventDom, DomHostedRuntimeError, DomHostedTopLevel, DomSession, DomSourceRuntime,
    HostedRuntime,
};

pub(crate) mod guest;
mod projection;
pub use guest::{
    DocumentGuestEvent, DocumentGuestEventDispatchFailure, DocumentGuestEventError,
    DocumentGuestEventLimits, DocumentGuestEventReport, DocumentGuestEventTarget,
    DocumentGuestEventView, DocumentGuestListenerException, EnableDocumentGuestEventsFailure,
};
mod source;
mod window_event;
mod window_relations;
mod window_timers;
pub use source::{
    DocumentScriptEventError, DocumentScriptEventProgress, DocumentScriptEventReport,
    DocumentScriptGuestResult, SelectedDocumentScriptListener,
};
pub use window_timers::{WindowTimerLimits, WindowTimerProgress};

/// Native adapter errors. These values can be retained as the original typed
/// cause of a bindings host refusal.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentScriptNativeError {
    /// The original native lifecycle refused the operation.
    Hosted(DomHostedRuntimeError),
    /// The canonical DOM refused the operation.
    Dom(DomError),
    /// Requested adapter backing exceeds the configured storage limit.
    StorageLimit,
    /// The original native owner is already loaned.
    Busy,
    /// The permit does not belong to the receiving attachment.
    ForeignPermit,
    /// The original attachment has stopped or expired.
    Revoked,
    /// The selected native Source has an unfinished synchronous obligation.
    SourcePending,
    /// This hook lacks the selected native event's exact Source authority.
    SourceAccess,
    /// The native node's creation Global is unavailable in this attachment.
    CreationGlobalUnavailable,
    /// The exact native Global already has a creation realm.
    CreationRealmAlreadyRegistered,
    /// The guest binding budget refused installation in the creation realm.
    CreationRealmWorkLimit,
    /// A previously admitted projected node lost its managed pin record.
    ProjectionCustody,
    /// The result cannot be represented in the supported service profile.
    RepresentationExhausted,
    /// This attachment did not enable the callable Window timer profile.
    WindowTimersDisabled,
}

impl fmt::Display for DocumentScriptNativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native document scripting refused: {self:?}")
    }
}

impl std::error::Error for DocumentScriptNativeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hosted(error) => Some(error),
            Self::Dom(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DomError> for DocumentScriptNativeError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

impl From<DomHostedRuntimeError> for DocumentScriptNativeError {
    fn from(error: DomHostedRuntimeError) -> Self {
        Self::Hosted(error)
    }
}

/// Distinguishes native entry refusal from the actual bindings result,
/// including a guest exception or a frozen native callback failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentScriptError {
    /// A native entry or owner operation was refused.
    Native(DocumentScriptNativeError),
    /// The actual bindings execution outcome.
    Script(Error),
}

impl fmt::Display for DocumentScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(error) => error.fmt(f),
            Self::Script(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for DocumentScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Native(error) => Some(error),
            Self::Script(error) => Some(error),
        }
    }
}

impl From<Error> for DocumentScriptError {
    fn from(error: Error) -> Self {
        Self::Script(error)
    }
}

impl From<DocumentScriptNativeError> for DocumentScriptError {
    fn from(error: DocumentScriptNativeError) -> Self {
        Self::Native(error)
    }
}

type FailureSlot = HostFailureSlot<DocumentScriptNativeError, DomHostedRuntimeRetention>;
type SourceFailureSlot = HostFailureSlot<crate::DomSourceDriveError, DomHostedRuntimeRetention>;

struct Attachment {
    target: DomHostedTopLevel,
    canonical: DomDocumentScriptBinding,
    realm: RealmAuthority,
    policy: HtmlScriptingPolicy,
    failures: FailureSlot,
    source_failures: SourceFailureSlot,
}

struct NativeOwner {
    attachment: OnceCell<Attachment>,
    session: RefCell<DomSession>,
    active_source: Cell<Option<source::ActiveSource>>,
    cancellation: Arc<AtomicBool>,
    storage_limit: usize,
    css_limits: Cell<CssNamespaceLimits>,
    dom_exception_limits: Cell<DomExceptionLimits>,
    timer_limits: Cell<Option<WindowTimerLimits>>,
    _backing: DomHostedRuntimeRetention,
}

impl NativeOwner {
    fn recover(mut owner: Rc<Self>) -> Result<DomSession, Rc<Self>> {
        // Do not release the funded target body while an outstanding Weak can
        // keep its allocation alive. The installed hook is the only issuer.
        let Some(native) = Rc::get_mut(&mut owner) else {
            return Err(owner);
        };
        if let Some(attachment) = native.attachment.get() {
            native
                .session
                .get_mut()
                .dom()
                .raw_mut()
                .release_document_script_registration(&attachment.canonical);
        }
        Rc::try_unwrap(owner).map(|native| native.session.into_inner())
    }

    fn observe(
        &self,
        attachment: &Attachment,
        session: &mut DomSession,
    ) -> Result<DomDocumentScriptingObservation, DocumentScriptNativeError> {
        let cancel = &self.cancellation;
        let hosted = session.hosted_execution();
        let mut dom = session.dom();
        if dom.pending_source_edit().is_some()
            || dom.snapshot().incomplete_hosted_operation().is_some()
        {
            return Err(DocumentScriptNativeError::SourcePending);
        }
        Ok(
            hosted.with_script_target(&mut dom, attachment.target, cancel, |source, dom| {
                source.observe_document_scripting(
                    dom.raw(),
                    &attachment.canonical,
                    Some(DomDocumentScriptingFacts {
                        engine: HtmlScriptingEngine::Supported,
                        policy: attachment.policy,
                    }),
                    cancel,
                )
            })?,
        )
    }
}

/// Names of the explicitly delegated host services installed in the realm.
pub struct DocumentScriptHooks<'a> {
    /// Reads the attached Document's current direct child count.
    pub child_count: &'a [u16],
    /// During a selected Source event, takes a condition and the exact projected
    /// Comment. After authenticating the Comment, converts the condition to
    /// Number and detaches it for a nonzero, non-NaN result.
    pub detach_source_node_if: &'a [u16],
}

/// Owned native preparation. No script process or attachment exists yet.
pub struct PreparedDocumentScriptHost {
    native: Rc<NativeOwner>,
}

impl fmt::Debug for PreparedDocumentScriptHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedDocumentScriptHost")
            .finish_non_exhaustive()
    }
}

/// A failed enable returns the same unpublished native owner.
#[derive(Debug)]
pub struct EnableDocumentScriptFailure {
    /// Original native owner, available for native use or a new attempt.
    pub owner: PreparedDocumentScriptHost,
    /// Exact failed phase's error.
    pub error: DocumentScriptError,
}

/// A refused preparation returns the exact original Session.
pub struct PrepareDocumentScriptFailure {
    /// The original native owner.
    pub session: DomSession,
    /// The native phase's refusal.
    pub error: DocumentScriptNativeError,
}

impl fmt::Debug for PrepareDocumentScriptFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrepareDocumentScriptFailure")
            .field("error", &self.error)
            .finish_non_exhaustive()
    }
}

impl PreparedDocumentScriptHost {
    /// Admit the owner backing before moving the exact Session into it.
    #[expect(
        clippy::result_large_err,
        reason = "Refusal returns the original DomSession and live obligations without allocating after admission failure"
    )]
    pub fn prepare(
        mut session: DomSession,
        storage_limit: usize,
        cancellation: Arc<AtomicBool>,
    ) -> Result<Self, PrepareDocumentScriptFailure> {
        let bytes = size_of::<NativeOwner>();
        if bytes > storage_limit {
            return Err(PrepareDocumentScriptFailure {
                session,
                error: DocumentScriptNativeError::StorageLimit,
            });
        }
        let backing = match session.hosted_execution().retain_script_payload(
            &mut session.dom(),
            bytes,
            &cancellation,
        ) {
            Ok(backing) => backing,
            Err(error) => {
                return Err(PrepareDocumentScriptFailure {
                    session,
                    error: error.into(),
                });
            }
        };
        Ok(Self {
            native: Rc::new(NativeOwner {
                attachment: OnceCell::new(),
                session: RefCell::new(session),
                active_source: Cell::new(None),
                cancellation,
                storage_limit,
                css_limits: Cell::new(CssNamespaceLimits::default()),
                dom_exception_limits: Cell::new(DomExceptionLimits::default()),
                timer_limits: Cell::new(None),
                _backing: backing,
            }),
        })
    }

    /// Select the Window CSS conversion and native identifier limits before attachment.
    #[must_use]
    pub fn with_css_namespace_limits(self, limits: CssNamespaceLimits) -> Self {
        self.native.css_limits.set(limits);
        self
    }

    /// Select the DOMException constructor conversion limits before attachment.
    #[must_use]
    pub fn with_dom_exception_limits(self, limits: DomExceptionLimits) -> Self {
        self.native.dom_exception_limits.set(limits);
        self
    }

    /// Enable the callable Window timer branch for this attachment.
    ///
    /// Timer progress requires explicit native task service. String handlers
    /// and callbacks from other Window attachments are outside this profile.
    #[must_use]
    pub fn with_window_timer_limits(self, limits: WindowTimerLimits) -> Self {
        self.native.timer_limits.set(Some(limits));
        self
    }

    /// Recover the original unpublished Session.
    pub fn into_session(self) -> Result<DomSession, Self> {
        NativeOwner::recover(self.native).map_err(|native| Self { native })
    }

    /// Attach an actual engine to the selected native top-level Document.
    /// Installs the Window relations, CSS namespace, DOMException, and named host services.
    pub fn enable(
        self,
        target: DomHostedTopLevel,
        policy: HtmlScriptingPolicy,
        hooks: DocumentScriptHooks<'_>,
    ) -> Result<DocumentScriptHost, EnableDocumentScriptFailure> {
        let result = self
            .prepare_engine(target, hooks)
            .and_then(|engine| engine.publish(&self.native, policy));
        match result {
            Ok(engine) => Ok(self.into_host(engine, None)),
            Err(error) => Err(EnableDocumentScriptFailure { owner: self, error }),
        }
    }

    fn into_host(
        self,
        engine: ScriptEngine,
        guest_exception_reports: Option<Rc<guest::GuestExceptionReports>>,
    ) -> DocumentScriptHost {
        DocumentScriptHost {
            engine: Some(engine),
            native: self.native,
            retirement: None,
            guest_exception_reports,
        }
    }

    fn prepare_engine(
        &self,
        target: DomHostedTopLevel,
        hooks: DocumentScriptHooks<'_>,
    ) -> Result<UnpublishedScriptEngine, DocumentScriptError> {
        let owner = &self.native;
        let cancel = &owner.cancellation;
        let hosted = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let hosted = session.hosted_execution();
            hosted
                .with_script_target(&mut session.dom(), target, cancel, |_, _| Ok(()))
                .map_err(DocumentScriptNativeError::from)?;
            hosted
        };

        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let authority = process
            .execution()
            .in_realm(&realm, |scope| scope.realm_authority())?;
        let permit = DocumentScriptPermit {
            realm: authority.identity(),
            document: target.document(),
        };
        let read = child_count_callback(Rc::downgrade(owner), permit.clone());
        let detach = source::detach_callback(Rc::downgrade(owner), permit.clone());
        let hook_bytes = size_of_val(&read)
            .checked_add(size_of_val(&detach))
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let failure_bytes = FailureSlot::storage_bytes();
        let source_failure_bytes = SourceFailureSlot::storage_bytes();
        let liveness_bytes = DomDocumentScriptRegistration::storage_bytes();
        let css_bytes = css::callback_storage_bytes::<DomHostedRuntimeRetention>();
        let css_failure_bytes =
            HostFailureSlot::<CssSerializationError, DomHostedRuntimeRetention>::storage_bytes();
        let dom_exception_bytes =
            dom_exception::constructor_storage_bytes::<DomHostedRuntimeRetention>();
        let window_event_bytes = window_event::WindowEvent::storage_bytes();
        let timer_limits = owner.timer_limits.get();
        let timer_bytes = match timer_limits {
            Some(limits) => limits
                .reserved_storage()
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?,
            None => 0,
        };
        let bytes = size_of::<NativeOwner>()
            .checked_add(hook_bytes)
            .and_then(|bytes| bytes.checked_add(failure_bytes))
            .and_then(|bytes| bytes.checked_add(source_failure_bytes))
            .and_then(|bytes| bytes.checked_add(liveness_bytes))
            .and_then(|bytes| bytes.checked_add(css_bytes))
            .and_then(|bytes| bytes.checked_add(css_failure_bytes))
            .and_then(|bytes| bytes.checked_add(dom_exception_bytes))
            .and_then(|bytes| bytes.checked_add(window_relations::storage_bytes()))
            .and_then(|bytes| bytes.checked_add(window_event_bytes))
            .and_then(|bytes| bytes.checked_add(timer_bytes))
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        if bytes > owner.storage_limit {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let (
            hook_backing,
            failure_backing,
            source_failure_backing,
            liveness_backing,
            css_backing,
            css_failure_backing,
            dom_exception_backing,
            window_relations_backing,
            window_event_backing,
            timer_backing,
        ) = {
            let mut session = owner
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let mut dom = session.dom();
            let hook = hosted
                .retain_script_payload(&mut dom, hook_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let failure = hosted
                .retain_script_payload(&mut dom, failure_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let source_failure = hosted
                .retain_script_payload(&mut dom, source_failure_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let liveness = hosted
                .retain_script_payload(&mut dom, liveness_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let css = hosted
                .retain_script_payload(&mut dom, css_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let css_failure = hosted
                .retain_script_payload(&mut dom, css_failure_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let dom_exception = hosted
                .retain_script_payload(&mut dom, dom_exception_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let window_relations = hosted
                .retain_script_payload(&mut dom, window_relations::storage_bytes(), cancel)
                .map_err(DocumentScriptNativeError::from)?;
            hosted
                .admit_script_work(&dom, window_relations::installation_work(), cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let window_event = hosted
                .retain_script_payload(&mut dom, window_event_bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?;
            let timer = timer_limits
                .map(|limits| {
                    hosted.admit_script_work(&dom, 2, cancel)?;
                    hosted
                        .retain_script_payload(
                            &mut dom,
                            window_timers::WindowTimers::fixed_bytes(),
                            cancel,
                        )
                        .map(|backing| (limits, backing))
                })
                .transpose()
                .map_err(DocumentScriptNativeError::from)?;
            (
                hook,
                failure,
                source_failure,
                liveness,
                css,
                css_failure,
                dom_exception,
                window_relations,
                window_event,
                timer,
            )
        };
        let failures = FailureSlot::new(&authority, failure_backing)?;
        let source_failures = SourceFailureSlot::new(&authority, source_failure_backing)?;
        let window_event = window_event::WindowEvent::new(window_event_backing);
        let window_timers = timer_backing
            .map(|(limits, backing)| window_timers::WindowTimers::new(limits, backing));

        // Drop Process before releasing the installed hook's debit on failure.
        let mut process = process;
        for (name, callback) in [
            (hooks.child_count, Callback::new(read)),
            (hooks.detach_source_node_if, Callback::new(detach)),
        ] {
            {
                let mut session = owner
                    .session
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                hosted
                    .admit_script_work(&session.dom(), name.len(), cancel)
                    .map_err(DocumentScriptNativeError::from)?;
            }
            process.execution().in_realm(&realm, |scope| {
                let global = scope.global()?;
                let hook = scope.function(&callback)?;
                scope.define_new_property(
                    &global,
                    PropertyKey::String(name),
                    PropertyDescriptor::Data {
                        value: &hook,
                        writable: false,
                        enumerable: false,
                        configurable: false,
                    },
                )
            })?;
        }
        process.execution().in_realm(&realm, |scope| {
            window_relations::install(
                scope,
                Rc::downgrade(owner),
                permit,
                window_relations_backing,
            )?;
            css::install_window_css(
                scope,
                owner.css_limits.get(),
                css_backing,
                css_failure_backing,
            )?;
            dom_exception::install(
                scope,
                owner.dom_exception_limits.get(),
                dom_exception_backing,
            )?;
            window_event.install(scope)?;
            if let Some(timers) = &window_timers {
                timers.install(owner, scope)?;
            }
            Ok(())
        })?;

        Ok(UnpublishedScriptEngine {
            process,
            realm,
            target,
            authority,
            failures,
            source_failures,
            window_event,
            window_timers,
            liveness_backing,
            retained_bytes: bytes,
            _hook_backing: hook_backing,
        })
    }
}

struct UnpublishedScriptEngine {
    process: Process,
    realm: Realm,
    target: DomHostedTopLevel,
    authority: RealmAuthority,
    failures: FailureSlot,
    source_failures: SourceFailureSlot,
    window_event: Rc<window_event::WindowEvent>,
    window_timers: Option<Rc<window_timers::WindowTimers>>,
    liveness_backing: DomHostedRuntimeRetention,
    retained_bytes: usize,
    _hook_backing: DomHostedRuntimeRetention,
}

impl UnpublishedScriptEngine {
    fn publish(
        self,
        owner: &NativeOwner,
        policy: HtmlScriptingPolicy,
    ) -> Result<ScriptEngine, DocumentScriptError> {
        let cancel = &owner.cancellation;
        let Self {
            process,
            realm,
            target,
            authority,
            failures,
            source_failures,
            window_event,
            window_timers,
            liveness_backing,
            retained_bytes,
            _hook_backing,
        } = self;
        // Keep the hook debit alive through engine disposal on refusal.
        let process = process;
        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let hosted = session.hosted_execution();
        let (canonical, registration) = hosted
            .with_script_target(&mut session.dom(), target, cancel, |source, dom| {
                source.attach_document_script_host(
                    dom.raw_mut(),
                    target.global(),
                    target.document(),
                    DomDocumentScriptingFacts {
                        engine: HtmlScriptingEngine::Supported,
                        policy,
                    },
                    liveness_backing,
                    cancel,
                )
            })
            .map_err(DocumentScriptNativeError::from)?;
        // Only this unpublished owner can initialize the cell; it is consumed
        // on success, and no guest operation can run during publication.
        owner.attachment.get_or_init(|| Attachment {
            target,
            canonical,
            realm: authority,
            policy,
            failures,
            source_failures,
        });
        Ok(ScriptEngine {
            registration,
            guest_events: None,
            window_event,
            window_timers,
            process,
            realm,
            comments: projection::ProjectedComments::default(),
            creations: CreationRealms::default(),
            retained_bytes,
            _hook_backing,
        })
    }
}

/// Explicit delegation to one native Document. It retains neither the native
/// owner nor the script process, and conveys no caller/incumbent settings.
#[derive(Clone)]
pub struct DocumentScriptPermit {
    realm: RealmId,
    document: DocumentId,
}

impl DocumentScriptPermit {
    /// The original native identity covered by this permit.
    pub const fn document(&self) -> DocumentId {
        self.document
    }
}

#[derive(Default)]
struct CreationRealms(Option<Box<CreationRealm>>);

impl CreationRealms {
    fn find<E>(
        &self,
        global: DomHostedGlobalId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<&Realm>, E> {
        let mut entry = self.0.as_deref();
        while let Some(record) = entry {
            step()?;
            if record.global == global {
                return Ok(Some(&record.realm));
            }
            entry = record.next.as_deref();
        }
        Ok(None)
    }
}

impl Drop for CreationRealms {
    fn drop(&mut self) {
        while let Some(mut entry) = self.0.take() {
            self.0 = entry.next.take();
        }
    }
}

struct CreationRealm {
    next: Option<Box<CreationRealm>>,
    global: DomHostedGlobalId,
    realm: Realm,
    _backing: DomHostedRuntimeRetention,
}

struct ScriptEngine {
    // Close the canonical engine fact before releasing SDK handles/captures.
    registration: DomDocumentScriptRegistration,
    guest_events: Option<Rc<guest::GuestEvents>>,
    window_event: Rc<window_event::WindowEvent>,
    window_timers: Option<Rc<window_timers::WindowTimers>>,
    process: Process,
    realm: Realm,
    comments: projection::ProjectedComments,
    creations: CreationRealms,
    retained_bytes: usize,
    _hook_backing: DomHostedRuntimeRetention,
}

impl ScriptEngine {
    fn retained_bytes_with(&self, additional: usize) -> Option<usize> {
        self.comments
            .len()
            .checked_mul(projection::ProjectedComments::pin_bytes(&self.process)?)?
            .checked_add(self.retained_bytes)?
            .checked_add(self.comments.slot_bytes()?)?
            .checked_add(additional)
    }

    fn settle_engine_cleanup(&mut self, realm: &RealmAuthority) {
        self.comments.drain_retirements(&mut self.process);
        if let Some(events) = &self.guest_events {
            events.reap_retired_targets(&mut self.process);
        }
        if realm.check_owner_active().is_err() {
            self.comments.release_all(&mut self.process);
            if let Some(timers) = &self.window_timers {
                timers.discard(&mut self.process);
            }
        }
    }
}

enum ScriptRetirement {
    NativeInactive,
    BindingsFailure(Error),
}

/// An attached original native owner and the separately owned script process.
pub struct DocumentScriptHost {
    engine: Option<ScriptEngine>,
    native: Rc<NativeOwner>,
    retirement: Option<ScriptRetirement>,
    guest_exception_reports: Option<Rc<guest::GuestExceptionReports>>,
}

struct NativeLoanExit<'a> {
    host: &'a mut DocumentScriptHost,
    acquired: bool,
}

impl Drop for NativeLoanExit<'_> {
    fn drop(&mut self) {
        if !self.acquired {
            return;
        }
        self.host.dispose_retired_engine();
        if let Some(engine) = self.host.engine.as_mut()
            && let Some(attachment) = self.host.native.attachment.get()
        {
            engine.settle_engine_cleanup(&attachment.realm);
        }
    }
}

impl fmt::Debug for DocumentScriptHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentScriptHost").finish_non_exhaustive()
    }
}

impl DocumentScriptHost {
    /// Explicitly delegate access to this attachment's original Document.
    /// Every use independently qualifies the current native association.
    pub fn permit(&self) -> Result<DocumentScriptPermit, DocumentScriptNativeError> {
        let attachment = self.attachment()?;
        Ok(DocumentScriptPermit {
            realm: attachment.realm.identity(),
            document: attachment.target.document(),
        })
    }

    fn attachment(&self) -> Result<&Attachment, DocumentScriptNativeError> {
        if self
            .engine
            .as_ref()
            .is_none_or(|engine| !engine.registration.is_active())
        {
            return Err(DocumentScriptNativeError::Revoked);
        }
        let attachment = self
            .native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        Ok(attachment)
    }

    fn dispose_retired_engine(&mut self) {
        if self
            .engine
            .as_ref()
            .is_some_and(|engine| !engine.registration.is_active())
            && let Some(attachment) = self.native.attachment.get()
        {
            // Preserve the original terminal bindings result before its weak
            // owner expires. Dropping the Process abandons jobs without entry.
            self.retirement = Some(match attachment.realm.check_owner_active() {
                Ok(()) => ScriptRetirement::NativeInactive,
                Err(error) => ScriptRetirement::BindingsFailure(error),
            });
            drop(self.engine.take());
        }
    }

    fn check_permit(&self, permit: &DocumentScriptPermit) -> Result<(), DocumentScriptNativeError> {
        let attachment = self.attachment()?;
        if permit.realm != attachment.realm.identity()
            || permit.document != attachment.target.document()
        {
            return Err(DocumentScriptNativeError::ForeignPermit);
        }
        Ok(())
    }

    fn observe(&mut self) -> Result<DomDocumentScriptingObservation, DocumentScriptError> {
        let attachment = self.attachment()?;
        attachment.realm.check_owner_active()?;
        let result = {
            let mut session = self
                .native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            self.native.observe(attachment, &mut session)
        };
        match result {
            Ok(observation) if observation.status() == HtmlScriptingStatus::Enabled => {
                Ok(observation)
            }
            Ok(_) => {
                self.engine.take();
                Err(
                    DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Dom(
                        DomHostedError::ScriptingDisabled,
                    ))
                    .into(),
                )
            }
            Err(error) => {
                if lost_attachment(&error) {
                    self.engine.take();
                }
                Err(error.into())
            }
        }
    }

    /// Admit a creation realm for nodes originating at a live
    /// top level in this attachment's native Session. The original attachment
    /// remains the sole script execution entry. The creation realm stays
    /// available for adopted nodes after their original top level retires.
    pub fn register_node_creation_realm(
        &mut self,
        permit: &DocumentScriptPermit,
        origin: DomHostedTopLevel,
    ) -> Result<(), DocumentScriptError> {
        self.check_permit(permit)?;
        self.observe()?;
        let attachment = self.attachment()?;
        let receiving_global = attachment.target.global();
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let bytes = size_of::<CreationRealm>()
            .checked_add(engine.process.realm_storage_bytes())
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let retained_bytes = engine
            .retained_bytes
            .checked_add(bytes)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let total = engine
            .retained_bytes_with(bytes)
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let cancel = &self.native.cancellation;
        let backing = {
            let mut session = self
                .native
                .session
                .try_borrow_mut()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            let hosted = session.hosted_execution();
            let mut dom = session.dom();
            hosted
                .with_script_target(&mut dom, origin, cancel, |_, _| Ok(()))
                .map_err(DocumentScriptNativeError::from)?;
            let registered = origin.global() == receiving_global
                || engine
                    .creations
                    .find(origin.global(), || {
                        hosted.admit_script_work(&dom, 1, cancel)
                    })
                    .map_err(DocumentScriptNativeError::from)?
                    .is_some();
            if registered {
                return Err(DocumentScriptNativeError::CreationRealmAlreadyRegistered.into());
            }
            if total > self.native.storage_limit {
                return Err(DocumentScriptNativeError::StorageLimit.into());
            }
            hosted
                .retain_script_payload(&mut dom, bytes, cancel)
                .map_err(DocumentScriptNativeError::from)?
        };
        let realm = engine.process.execution().create_realm()?;
        if let Some(events) = &engine.guest_events {
            events.install_origin(&mut engine.process, &realm, &self.native, origin.global())?;
        }
        engine.creations.0 = Some(Box::new(CreationRealm {
            next: engine.creations.0.take(),
            global: origin.global(),
            realm,
            _backing: backing,
        }));
        engine.retained_bytes = retained_bytes;
        Ok(())
    }

    /// Sample the current scripting fact for the exact native snapshot.
    pub fn observe_scripting(
        &mut self,
        permit: &DocumentScriptPermit,
    ) -> Result<DomDocumentScriptingObservation, DocumentScriptError> {
        self.check_permit(permit)?;
        self.observe()
    }

    /// Sample and submit the exact pending Script's relevant settings. A
    /// request for a different Global remains pending and returns None.
    /// Resuming the original Source exposes its next requirement.
    pub fn service_script_settings(
        &mut self,
        permit: &DocumentScriptPermit,
        request: surgeist_dom::DomSourceScriptRequest,
        limits: surgeist_dom::DomSourceLimits,
    ) -> Result<Option<surgeist_dom::DomSourceScriptSettings>, DocumentScriptError> {
        self.check_permit(permit)?;
        let attachment = self.attachment()?;
        attachment.realm.check_owner_active()?;
        let cancel = &self.native.cancellation;
        let mut session = self
            .native
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let hosted = session.hosted_execution();
        let mut dom = session.dom();
        let mut lease = dom
            .resume_source_edit(request.operation(), limits, cancel)
            .map_err(DocumentScriptNativeError::from)?;
        lease
            .source_script_step(request, cancel)
            .map_err(DocumentScriptNativeError::from)?;
        if request.relevant_global() != Some(attachment.target.global()) {
            return Ok(None);
        }
        let settings = hosted
            .with_script_source_settings(
                &mut lease,
                attachment.target,
                request,
                cancel,
                |source, lease| {
                    let settings = lease
                        .observe_script_settings(
                            request,
                            source,
                            &attachment.canonical,
                            DomDocumentScriptingFacts {
                                engine: HtmlScriptingEngine::Supported,
                                policy: attachment.policy,
                            },
                            cancel,
                        )
                        .map_err(DomHostedRuntimeError::Dom)?;
                    lease
                        .accept_script_settings(settings, cancel)
                        .map_err(|error| DomHostedRuntimeError::Dom(error.into()))?;
                    Ok(settings)
                },
            )
            .map_err(DocumentScriptNativeError::from)?;
        Ok(Some(settings))
    }

    /// Run guest work in the actual attached realm without a native Session
    /// loan. This entry performs no implicit microtask checkpoint.
    pub fn in_document<T>(
        &mut self,
        permit: &DocumentScriptPermit,
        action: impl for<'scope> FnOnce(&mut Scope<'scope>) -> Result<T, Error>,
    ) -> Result<T, DocumentScriptError> {
        self.check_permit(permit)?;
        self.observe()?;
        let attachment = self
            .native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let result = engine.process.execution().in_realm(&engine.realm, action);
        engine.settle_engine_cleanup(&attachment.realm);
        Ok(result?)
    }

    /// Run the shared guest queue only after current native qualification and
    /// after synchronous native obligations settle. The engine does not
    /// surface rejected promises through this checkpoint.
    pub fn checkpoint(&mut self) -> Result<(), DocumentScriptError> {
        self.dispose_retired_engine();
        if let Some(retirement) = &self.retirement {
            return Err(match retirement {
                ScriptRetirement::BindingsFailure(error) => error.clone().into(),
                ScriptRetirement::NativeInactive => {
                    DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Inactive).into()
                }
            });
        }
        self.observe()?;
        let attachment = self
            .native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let result = engine.process.execution().checkpoint();
        engine.settle_engine_cleanup(&attachment.realm);
        Ok(result?)
    }

    /// Historical native snapshot; it carries no live script authority.
    pub fn snapshot(&self) -> Result<DomSnapshot, DocumentScriptNativeError> {
        let session = self
            .native
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        Ok(session.snapshot())
    }

    /// Lend the original canonical view and its matching lifecycle and Source
    /// permissions to the Rust host. On exit, native retirement disposes of
    /// the script process without a checkpoint and preserves the action result.
    pub fn with_native<T>(
        &mut self,
        action: impl FnOnce(&mut DomEventDom<'_>, HostedRuntime, DomSourceRuntime) -> T,
    ) -> Result<T, DocumentScriptNativeError> {
        // The Session loan and DOM view must end before Process disposal,
        // including if caller code unwinds after canonical retirement.
        let mut exit = NativeLoanExit {
            host: self,
            acquired: false,
        };
        let mut session = exit
            .host
            .native
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        exit.acquired = true;
        let hosted = session.hosted_execution();
        let source = session.source_execution();
        Ok(action(&mut session.dom(), hosted, source))
    }

    /// Revoke the attachment and dispose of its queue without a checkpoint.
    pub fn stop(self) -> StoppedDocumentScriptHost {
        drop(self.engine);
        StoppedDocumentScriptHost {
            native: self.native,
        }
    }
}

fn lost_attachment(error: &DocumentScriptNativeError) -> bool {
    matches!(
        error,
        DocumentScriptNativeError::Hosted(
            DomHostedRuntimeError::Inactive
                | DomHostedRuntimeError::TargetNotFound
                | DomHostedRuntimeError::OwnerDisposed
                | DomHostedRuntimeError::Dom(
                    DomHostedError::StaleIdentity
                        | DomHostedError::SourceExpired
                        | DomHostedError::SourcePoisoned
                )
        )
    )
}

/// Native custody after the script process has been disposed.
#[derive(Debug)]
pub struct StoppedDocumentScriptHost {
    native: Rc<NativeOwner>,
}

impl fmt::Debug for NativeOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeOwner").finish_non_exhaustive()
    }
}

impl StoppedDocumentScriptHost {
    /// Recover the same native Session. Its historical scripting facts remain
    /// usable with their exact historical snapshots.
    pub fn into_session(self) -> Result<DomSession, Self> {
        NativeOwner::recover(self.native).map_err(|native| Self { native })
    }
}

/// Map the supplied document status to the corresponding CSS media value.
/// The caller selects the status from its capture or live observation.
/// Unavailable status yields no CSS observation.
pub const fn media_scripting_for_status(status: HtmlScriptingStatus) -> Option<CssMediaScripting> {
    match status {
        HtmlScriptingStatus::Disabled => Some(CssMediaScripting::None),
        HtmlScriptingStatus::Enabled => Some(CssMediaScripting::Enabled),
        HtmlScriptingStatus::Unavailable => None,
    }
}

fn child_count_callback(
    native: Weak<NativeOwner>,
    permit: DocumentScriptPermit,
) -> impl for<'scope> Fn(
    &mut Scope<'scope>,
    surgeist_bindings_js::Arguments<'scope>,
) -> Result<Completion<'scope>, Error> {
    move |scope, _| read_child_count(&native, &permit, scope)
}

fn read_child_count<'scope>(
    native: &Weak<NativeOwner>,
    permit: &DocumentScriptPermit,
    scope: &mut Scope<'scope>,
) -> Result<Completion<'scope>, Error> {
    let native = native.upgrade().ok_or(Error::ExpiredIdentity)?;
    let attachment = native.attachment.get().ok_or(Error::ExpiredIdentity)?;
    if scope.realm_authority()?.identity() != permit.realm {
        return refuse(
            scope,
            &attachment.failures,
            DocumentScriptNativeError::ForeignPermit,
        );
    }
    let count = (|| {
        let mut session = native
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let status = match native.active_source() {
            Some(active) => native.qualify_event(attachment, &mut session, active.request())?,
            None => native.observe(attachment, &mut session)?.status(),
        };
        if status != HtmlScriptingStatus::Enabled {
            return Err(DocumentScriptNativeError::Hosted(
                DomHostedRuntimeError::Dom(DomHostedError::ScriptingDisabled),
            ));
        }
        let snapshot = session.snapshot();
        let document = snapshot
            .document(permit.document)
            .map_err(DocumentScriptNativeError::Dom)?;
        let count = document
            .node(permit.document.node())
            .map_err(DocumentScriptNativeError::Dom)?
            .children()
            .len();
        u32::try_from(count).map_err(|_| DocumentScriptNativeError::RepresentationExhausted)
    })();
    match count {
        Ok(count) => Ok(Completion::Return(
            scope.primitive(Primitive::Number(f64::from(count)))?,
        )),
        Err(error) => refuse(scope, &attachment.failures, error),
    }
}

fn refuse<T, E: std::error::Error + 'static>(
    scope: &mut Scope<'_>,
    slot: &HostFailureSlot<E, DomHostedRuntimeRetention>,
    error: E,
) -> Result<T, Error> {
    match slot.record(error) {
        Ok(failure) | Err(RecordFailure::Occupied { first: failure, .. }) => {
            Err(scope.refuse_host(&failure))
        }
        Err(RecordFailure::Inactive { error, .. }) => Err(error),
    }
}

#[cfg(test)]
mod cross_realm_tests;
