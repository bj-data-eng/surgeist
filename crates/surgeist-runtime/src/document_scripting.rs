//! Optional script ownership for one actual native-created hosted Document.

use std::cell::{Cell, OnceCell, RefCell};
use std::fmt;
use std::mem::{size_of, size_of_val};
use std::rc::{Rc, Weak};
use std::sync::{Arc, atomic::AtomicBool};

use surgeist_bindings_js::{
    Callback, Completion, EngineKind, Error, HostFailureSlot, Primitive, Process,
    PropertyDescriptor, PropertyKey, Realm, RealmAuthority, RealmId, RecordFailure, Scope,
};
use surgeist_css::query::media::CssMediaScripting;
use surgeist_dom::{
    DocumentId, DomDocumentScriptBinding, DomDocumentScriptingFacts,
    DomDocumentScriptingObservation, DomError, DomHostedError, DomHostedRuntimeRetention,
    DomSnapshot,
};
use surgeist_html::scripting::{HtmlScriptingEngine, HtmlScriptingPolicy};
use surgeist_html_semantics::HtmlScriptingStatus;

use crate::{
    DomEventDom, DomHostedRuntimeError, DomHostedTopLevel, DomSession, DomSourceRuntime,
    HostedRuntime,
};

mod source;
pub use source::{
    DocumentScriptEventError, DocumentScriptEventProgress, DocumentScriptEventReport,
    DocumentScriptGuestResult, SelectedDocumentScriptListener,
};

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
    /// The result cannot be represented in the supported service profile.
    RepresentationExhausted,
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
    _backing: DomHostedRuntimeRetention,
}

impl NativeOwner {
    fn recover(mut owner: Rc<Self>) -> Result<DomSession, Rc<Self>> {
        // Do not release the funded target body while an outstanding Weak can
        // keep its allocation alive. The installed hook is the only issuer.
        if Rc::get_mut(&mut owner).is_none() {
            return Err(owner);
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
    /// During a selected Source event, converts its argument to Number and
    /// detaches the delegated direct Document comment for a nonzero, non-NaN result.
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
                _backing: backing,
            }),
        })
    }

    /// Recover the original unpublished Session.
    pub fn into_session(self) -> Result<DomSession, Self> {
        NativeOwner::recover(self.native).map_err(|native| Self { native })
    }

    /// Attach an actual engine to the selected native top-level Document.
    /// Installs the explicitly named host services in its realm.
    pub fn enable(
        self,
        target: DomHostedTopLevel,
        policy: HtmlScriptingPolicy,
        hooks: DocumentScriptHooks<'_>,
    ) -> Result<DocumentScriptHost, EnableDocumentScriptFailure> {
        match self.enable_inner(target, policy, hooks) {
            Ok(engine) => Ok(DocumentScriptHost {
                engine: Some(engine),
                native: self.native,
            }),
            Err(error) => Err(EnableDocumentScriptFailure { owner: self, error }),
        }
    }

    fn enable_inner(
        &self,
        target: DomHostedTopLevel,
        policy: HtmlScriptingPolicy,
        hooks: DocumentScriptHooks<'_>,
    ) -> Result<ScriptEngine, DocumentScriptError> {
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
        let detach = source::detach_callback(Rc::downgrade(owner), permit);
        let hook_bytes = size_of_val(&read)
            .checked_add(size_of_val(&detach))
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let failure_bytes = FailureSlot::storage_bytes();
        let source_failure_bytes = SourceFailureSlot::storage_bytes();
        let bytes = size_of::<NativeOwner>()
            .checked_add(hook_bytes)
            .and_then(|bytes| bytes.checked_add(failure_bytes))
            .and_then(|bytes| bytes.checked_add(source_failure_bytes))
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        if bytes > owner.storage_limit {
            return Err(DocumentScriptNativeError::StorageLimit.into());
        }
        let (hook_backing, failure_backing, source_failure_backing) = {
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
            (hook, failure, source_failure)
        };
        let failures = FailureSlot::new(&authority, failure_backing)?;
        let source_failures = SourceFailureSlot::new(&authority, source_failure_backing)?;

        // Drop Process before releasing the installed hook's debit on failure.
        let mut process = process;
        process.execution().in_realm(&realm, |scope| {
            let global = scope.global()?;
            for (name, callback) in [
                (hooks.child_count, Callback::new(read)),
                (hooks.detach_source_node_if, Callback::new(detach)),
            ] {
                let hook = scope.function(&callback)?;
                scope.define_property(
                    &global,
                    PropertyKey::String(name),
                    PropertyDescriptor::Data {
                        value: &hook,
                        writable: false,
                        enumerable: false,
                        configurable: false,
                    },
                )?;
            }
            Ok(())
        })?;

        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let canonical = hosted
            .with_script_target(&mut session.dom(), target, cancel, |source, dom| {
                source.attach_document_script_host(
                    dom.raw_mut(),
                    target.host(),
                    target.global(),
                    target.document(),
                    DomDocumentScriptingFacts {
                        engine: HtmlScriptingEngine::Supported,
                        policy,
                    },
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
            process,
            realm,
            retained_bytes: bytes,
            _hook_backing: hook_backing,
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

struct ScriptEngine {
    process: Process,
    realm: Realm,
    retained_bytes: usize,
    _hook_backing: DomHostedRuntimeRetention,
}

/// An attached original native owner and the separately owned script process.
pub struct DocumentScriptHost {
    engine: Option<ScriptEngine>,
    native: Rc<NativeOwner>,
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
        if self.engine.is_none() {
            return Err(DocumentScriptNativeError::Revoked);
        }
        self.native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)
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

    /// Sample the current scripting fact for the exact native snapshot.
    pub fn observe_scripting(
        &mut self,
        permit: &DocumentScriptPermit,
    ) -> Result<DomDocumentScriptingObservation, DocumentScriptError> {
        self.check_permit(permit)?;
        self.observe()
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
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        Ok(engine.process.execution().in_realm(&engine.realm, action)?)
    }

    /// Run the shared guest queue only after current native qualification and
    /// after synchronous native obligations settle. The engine does not
    /// surface rejected promises through this checkpoint.
    pub fn checkpoint(&mut self) -> Result<(), DocumentScriptError> {
        self.observe()?;
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        Ok(engine.process.execution().checkpoint()?)
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
    /// permissions to the Rust host. The script process stays with this owner.
    pub fn with_native<T>(
        &mut self,
        action: impl FnOnce(&mut DomEventDom<'_>, HostedRuntime, DomSourceRuntime) -> T,
    ) -> Result<T, DocumentScriptNativeError> {
        let mut session = self
            .native
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
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
            Some(active) => native.qualify_event(attachment, &mut session, &active)?,
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
