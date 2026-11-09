//! Consuming public operations over the real request, execution and report owners.
use super::{
    StandaloneDocumentDriver, StandaloneError, StopCause,
    context::{
        ContextFragmentDriver, FailedContextFragment, FragmentExecutionError,
        FrozenContextFragmentParts,
        ownership::{
            FragmentInitialization, OwnedFragmentFailure, SealedFragmentConstruction,
            StoppedFragmentConstruction,
        },
    },
    head::{HeadStep, HtmlParseConstructionBoundary},
    report::HtmlParseReport,
    seal::{OrdinaryInitialization, OwnedOrdinaryFailure, SealedOrdinaryConstruction},
    terminal::{DetachedStandaloneFailure, FailedStandaloneDocument, FrozenStandaloneParts},
};
use crate::construction::{
    SessionClass, SessionError, evidence::HtmlParseDiagnostics, session_class,
};
use crate::parser::control::{HtmlParseStop, HtmlParseUnsupportedState, HtmlParseUsage};
use crate::{HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseLimits, HtmlSource};

/// A terminal reason is separate from recoverable syntax diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseFailureReason {
    CspProtocol(surgeist_html::security::csp::HtmlCspProtocolError),
    CspProcessing,
    Stop(HtmlParseStop),
    UnsupportedDocumentInitialization,
    UnsupportedExecutionProfile,
    UnsupportedConstruction(HtmlParseConstructionBoundary),
    MissingContextFact(crate::HtmlCaptureFact),
    UnsupportedContextReactions,
    Engine(HtmlParseEngineCondition),
}
/// A detected engine condition; it is not an HTML syntax error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseEngineCondition {
    Ownership,
    Tokenizer,
    Tree,
    Completion,
    Cleanup,
}
/// CSP storage this failure agreed to take back (see
/// `HtmlParseFailure::claim_csp_storage`); only a claim can produce one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReturnableCspStorage(crate::meta_csp::Storage);
impl ReturnableCspStorage {
    pub(crate) const NONE: Self = Self(crate::meta_csp::Storage::ZERO);
}
enum FailureStorage {
    OrdinaryInitialization(OrdinaryInitialization),
    Ordinary(OwnedOrdinaryFailure),
    FragmentInitialization(FragmentInitialization),
    Fragment(OwnedFragmentFailure),
    OrdinaryFrozen(SealedOrdinaryConstruction),
    FragmentFrozen(SealedFragmentConstruction),
}
/// Owns the original request and retained terminal evidence. It exposes no partial tree.
pub struct HtmlParseFailure {
    reason: HtmlParseFailureReason,
    storage: FailureStorage,
    cleanup_error: Option<HtmlParseEngineCondition>,
}
impl HtmlParseFailure {
    /// Decides, while this failure is still readable by the caller, whether it
    /// can take back `storage` that a processor's native error carried out of
    /// the parse. Only the original ordinary driver holds the controller the
    /// storage was charged to, and only if that controller admitted it. For any
    /// other owner, or for storage it does not hold, the storage stays
    /// unreleased and `cleanup_error` says why; the original stop and retained
    /// evidence are untouched. The result is what may later be returned.
    pub(crate) fn claim_csp_storage(
        &mut self,
        storage: crate::meta_csp::Storage,
    ) -> ReturnableCspStorage {
        if storage == crate::meta_csp::Storage::ZERO {
            return ReturnableCspStorage(storage);
        }
        let condition = match &mut self.storage {
            FailureStorage::Ordinary(owner) => {
                if owner.csp_storage_fits(storage) {
                    return ReturnableCspStorage(storage);
                }
                HtmlParseEngineCondition::Cleanup
            }
            _ => HtmlParseEngineCondition::Ownership,
        };
        self.cleanup_error.get_or_insert(condition);
        ReturnableCspStorage::NONE
    }
    /// Gives claimed storage back to the controller it was charged to. A
    /// refusal is latched on that controller, which the caller no longer
    /// reaches; a claim always passed the controller's own check.
    pub(crate) fn return_csp_storage(
        &mut self,
        storage: ReturnableCspStorage,
    ) -> Result<(), HtmlParseStop> {
        // A claim for nonzero storage exists only for an ordinary failure, and
        // the owner variant never changes after construction.
        if let FailureStorage::Ordinary(owner) = &mut self.storage
            && storage.0 != crate::meta_csp::Storage::ZERO
        {
            return owner.release_csp_storage(storage.0);
        }
        Ok(())
    }

    fn associated_inputs(&self) -> Option<&crate::construction::associated::Inputs> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(owner) => owner.request().associated_inputs(),
            FailureStorage::Ordinary(owner) => owner
                .original_request()
                .associated_inputs()
                .map(AsRef::as_ref),
            FailureStorage::OrdinaryFrozen(owner) => {
                owner.request.associated_inputs().map(AsRef::as_ref)
            }
            FailureStorage::FragmentInitialization(owner) => {
                owner.original_request().associated_inputs()
            }
            FailureStorage::Fragment(owner) => owner
                .original_request()
                .associated_inputs()
                .map(AsRef::as_ref),
            FailureStorage::FragmentFrozen(owner) => {
                owner.request.associated_inputs().map(AsRef::as_ref)
            }
        }
    }
    /// Original selected contexts, including unused entries, on every failure.
    pub fn encoding_url_contexts(
        &self,
    ) -> Option<&[surgeist_html_semantics::HtmlDocumentEncodingUrlContext]> {
        self.associated_inputs()
            .map(|inputs| inputs.captured.as_slice())
    }
    pub fn encoding_url_context_capacity(&self) -> Option<usize> {
        self.associated_inputs()
            .map(|inputs| inputs.captured.capacity())
    }
    /// Actual original supplied observation bank, including unused entries.
    /// Failure does not replace it with a selected or reconstructed collection.
    pub fn encoding_url_facts(
        &self,
    ) -> Option<&[surgeist_html_semantics::HtmlEncodingUrlParseFacts]> {
        self.associated_inputs()
            .map(|inputs| inputs.encoding_urls.as_slice())
    }
    pub fn encoding_url_fact_capacity(&self) -> Option<usize> {
        self.associated_inputs()
            .map(|inputs| inputs.encoding_urls.capacity())
    }
    pub fn reason(&self) -> HtmlParseFailureReason {
        self.reason
    }
    /// A condition under which the failure could not keep all it learned. When
    /// it is `Cleanup` or `Ownership` the evidence readers (`first_stop`,
    /// `source_admitted`, `at_stop`, `retained_usage`, `after_cleanup` and
    /// `diagnostics`) can read `None` even though execution ran, so a `None`
    /// from them does not mean the request was rejected before execution. The
    /// original request, its source and the reason are always kept, and
    /// `first_stop` still names the stop when the reason does.
    pub fn cleanup_error(&self) -> Option<HtmlParseEngineCondition> {
        self.cleanup_error
    }
    /// The stop the reason names, for a failure that reports a cleanup or an
    /// ownership condition and so holds no stopped prefix.
    fn stop_of_refused_cleanup(&self) -> Option<HtmlParseStop> {
        match (self.cleanup_error, self.reason) {
            (Some(_), HtmlParseFailureReason::Stop(stop)) => Some(stop),
            _ => None,
        }
    }
    /// The original execution stop, including an unsupported construction's
    /// canonical terminal stop. Pre-execution rejection has no invented stop.
    pub fn first_stop(&self) -> Option<HtmlParseStop> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => match o.failure() {
                Some(super::entry::InitializationFailure::Stopped(stop)) => Some(stop),
                // A refused cleanup keeps no stopped prefix, but the reason
                // still names the stop that ended execution.
                _ => self.stop_of_refused_cleanup(),
            },
            FailureStorage::Ordinary(o) => Some(o.stop()),
            FailureStorage::FragmentInitialization(o) => o
                .failure()
                .map(|f| f.stop())
                .or_else(|| self.stop_of_refused_cleanup()),
            FailureStorage::Fragment(o) => Some(o.stop()),
            FailureStorage::OrdinaryFrozen(o) => o.parts.control.stop(),
            FailureStorage::FragmentFrozen(o) => o.parts.control.stop(),
        }
    }
    /// None means rejection before an execution control was established, or a
    /// failure whose `cleanup_error` reports that it kept no stopped prefix.
    /// Some(false) is a stopped source-admission attempt, distinct from an
    /// admitted empty source.
    pub fn source_admitted(&self) -> Option<bool> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.source_admitted(),
            FailureStorage::Ordinary(_) => Some(true),
            FailureStorage::FragmentInitialization(o) => {
                o.failure().map(|f| f.source_binding().is_some())
            }
            FailureStorage::Fragment(_) => Some(true),
            FailureStorage::OrdinaryFrozen(o) => o
                .parts
                .control
                .stopped_source_binding()
                .ok()
                .map(|s| s.is_some()),
            FailureStorage::FragmentFrozen(o) => o
                .parts
                .control
                .stopped_source_binding()
                .ok()
                .map(|s| s.is_some()),
        }
    }
    /// Usage of the actual retained terminal evidence owner. This does not
    /// relabel the first-stop or completed cleanup snapshots.
    pub fn retained_usage(&self) -> Option<HtmlParseUsage> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.retained_usage(),
            FailureStorage::Ordinary(o) => Some(o.retained_usage()),
            FailureStorage::FragmentInitialization(o) => o.failure().map(|f| f.retained_usage()),
            FailureStorage::Fragment(o) => Some(o.retained_usage()),
            FailureStorage::OrdinaryFrozen(o) => Some(o.retained_usage()),
            FailureStorage::FragmentFrozen(o) => Some(o.retained_usage()),
        }
    }
    pub fn source(&self) -> &HtmlSource {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.source(),
            FailureStorage::Ordinary(o) => o.source(),
            FailureStorage::FragmentInitialization(o) => o.source(),
            FailureStorage::Fragment(o) => o.source(),
            FailureStorage::OrdinaryFrozen(o) => o.source(),
            FailureStorage::FragmentFrozen(o) => o.source(),
        }
    }
    pub fn limits(&self) -> &HtmlParseLimits {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.request().limits(),
            FailureStorage::Ordinary(o) => o.original_request().limits(),
            FailureStorage::FragmentInitialization(o) => o.original_request().limits(),
            FailureStorage::Fragment(o) => o.original_request().limits(),
            FailureStorage::OrdinaryFrozen(o) => o.limits(),
            FailureStorage::FragmentFrozen(o) => o.limits(),
        }
    }
    /// The profile of the parse's own operation, if its request selected a
    /// profile mode, as of this call. A parse that ran in a host's operation
    /// has none: the host reads its own operation.
    pub fn profile(
        &self,
    ) -> Result<
        Option<
            surgeist_ledger::Report<
                surgeist_html::accounting::HtmlWork,
                surgeist_html::accounting::HtmlResource,
            >,
        >,
        surgeist_html::accounting::HtmlAccountingFault,
    > {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.request().accounting().profile(),
            FailureStorage::Ordinary(o) => o.original_request().accounting().profile(),
            FailureStorage::FragmentInitialization(o) => {
                o.original_request().accounting().profile()
            }
            FailureStorage::Fragment(o) => o.original_request().accounting().profile(),
            FailureStorage::OrdinaryFrozen(o) => o.accounting().profile(),
            FailureStorage::FragmentFrozen(o) => o.accounting().profile(),
        }
    }
    /// The original captured-document request, absent for offline documents
    /// and fragments. A fragment capture is available through context().
    pub fn capture(&self) -> Option<&crate::HtmlCapture> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.request().capture(),
            FailureStorage::Ordinary(o) => o.original_request().capture(),
            FailureStorage::OrdinaryFrozen(o) => o.request.capture(),
            _ => None,
        }
    }
    /// The actual qualified captured-document target, absent for offline
    /// documents and fragments; this is not an output-tree identity.
    pub fn document(&self) -> Option<&crate::HtmlCaptureDocumentId> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.request().document(),
            FailureStorage::Ordinary(o) => o.original_request().document(),
            FailureStorage::OrdinaryFrozen(o) => o.request.document(),
            _ => None,
        }
    }
    /// The original ordinary request's URL facts, including on pre-source failure.
    pub fn document_base_facts(&self) -> Option<&crate::HtmlParseBaseFacts> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.request().base_facts(),
            FailureStorage::Ordinary(o) => o.original_request().base_facts(),
            FailureStorage::OrdinaryFrozen(o) => o.request.base_facts(),
            _ => None,
        }
    }
    /// The original fragment request's qualified captured facts.
    pub fn captured_base_facts(&self) -> Option<&crate::HtmlCapturedBaseFacts> {
        match &self.storage {
            FailureStorage::FragmentInitialization(o) => o.original_request().base_facts(),
            FailureStorage::Fragment(o) => o.original_request().base_facts(),
            FailureStorage::FragmentFrozen(o) => o.request.base_facts(),
            _ => None,
        }
    }
    /// The original request option, not a claim that execution occurred.
    pub fn requested_scripting_mode(&self) -> surgeist_html::parsing::HtmlParserScriptingMode {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.request().scripting_mode(),
            FailureStorage::Ordinary(_) | FailureStorage::OrdinaryFrozen(_) => {
                surgeist_html::parsing::HtmlParserScriptingMode::Disabled
            }
            FailureStorage::FragmentInitialization(o) => {
                o.original_request().requested_scripting_mode()
            }
            FailureStorage::Fragment(o) => o.original_request().requested_scripting_mode(),
            FailureStorage::FragmentFrozen(o) => o.request.requested_scripting_mode(),
        }
    }
    /// The original permission option, distinct from committed effects.
    pub fn allow_declarative_shadow_roots(&self) -> bool {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => {
                o.request().allow_declarative_shadow_roots()
            }
            FailureStorage::Ordinary(_) | FailureStorage::OrdinaryFrozen(_) => false,
            FailureStorage::FragmentInitialization(o) => {
                o.original_request().allow_declarative_shadow_roots()
            }
            FailureStorage::Fragment(o) => o.original_request().allow_declarative_shadow_roots(),
            FailureStorage::FragmentFrozen(o) => o.request.allow_declarative_shadow_roots(),
        }
    }
    pub fn context(&self) -> Option<&crate::HtmlFragmentContext> {
        match &self.storage {
            FailureStorage::FragmentInitialization(o) => Some(o.original_request().context()),
            FailureStorage::Fragment(o) => Some(o.context()),
            FailureStorage::FragmentFrozen(o) => Some(o.context()),
            _ => None,
        }
    }
    pub fn at_stop(&self) -> Option<&HtmlParseUsage> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.at_stop(),
            FailureStorage::Ordinary(o) => o.at_stop(),
            FailureStorage::FragmentInitialization(o) => o.failure().map(|f| f.at_stop()),
            FailureStorage::Fragment(o) => Some(o.at_stop()),
            FailureStorage::OrdinaryFrozen(o) => o.parts.at_stop(),
            FailureStorage::FragmentFrozen(o) => o.parts.at_stop(),
        }
    }
    pub fn after_cleanup(&self) -> Option<&HtmlParseUsage> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.after_cleanup(),
            FailureStorage::Ordinary(o) => Some(o.after_cleanup()),
            FailureStorage::FragmentInitialization(o) => o.failure().map(|f| f.after_cleanup()),
            FailureStorage::Fragment(o) => Some(o.after_cleanup()),
            FailureStorage::OrdinaryFrozen(o) => o.parts.after_cleanup(),
            FailureStorage::FragmentFrozen(o) => o.parts.after_cleanup(),
        }
    }
    pub fn diagnostics(&self) -> Option<HtmlParseDiagnostics<'_>> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.diagnostics(),
            FailureStorage::Ordinary(o) => o.diagnostics(),
            FailureStorage::FragmentInitialization(o) => o.failure().and_then(|f| f.diagnostics()),
            FailureStorage::Fragment(o) => o.diagnostics(),
            FailureStorage::OrdinaryFrozen(o) => o.parts.diagnostics().ok(),
            FailureStorage::FragmentFrozen(o) => o.parts.diagnostics().ok(),
        }
    }
}
impl std::fmt::Debug for HtmlParseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlParseFailure")
            .field("reason", &self.reason)
            .field("source", self.source().identity())
            .field("cleanup_error", &self.cleanup_error)
            .finish()
    }
}
impl std::fmt::Display for HtmlParseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTML parse failed: {:?}", self.reason)
    }
}
impl std::error::Error for HtmlParseFailure {}

/// The reason a session error ends a parse with. It is derived from the same
/// class as the stop (`session_class`), so the two cannot disagree: an outcome
/// that has a reason variant of its own keeps it, any other outcome is the
/// stop it ends the parse as, and a fault of the parser's own is an engine
/// condition.
fn session_reason(e: SessionError) -> HtmlParseFailureReason {
    match session_class(&e) {
        SessionClass::Fault(_) => HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Tree),
        SessionClass::Outcome(stop) => match e {
            SessionError::CspService => HtmlParseFailureReason::CspProcessing,
            SessionError::CspProtocol(p) => HtmlParseFailureReason::CspProtocol(p),
            SessionError::ContextFactUnavailable(f) => {
                HtmlParseFailureReason::MissingContextFact(f)
            }
            SessionError::ContextReactionUnavailable => {
                HtmlParseFailureReason::UnsupportedContextReactions
            }
            _ => stop_reason(stop),
        },
    }
}
/// The reason for a stop: the two Meta CSP stops have reasons of their own, so
/// one stop is one reason whether the service raised it or the controller
/// latched it.
fn stop_reason(stop: HtmlParseStop) -> HtmlParseFailureReason {
    match stop {
        HtmlParseStop::CspProcessing => HtmlParseFailureReason::CspProcessing,
        HtmlParseStop::CspProtocol(p) => HtmlParseFailureReason::CspProtocol(p),
        other => HtmlParseFailureReason::Stop(other),
    }
}
/// The reason for a cause: derived from the same `stop_cause` that the stop is
/// latched from, so the two cannot disagree. A lost owner is the engine
/// condition `lost` that the error's level reports for it; a session error is
/// its own reason; a state the environment does not continue from is the
/// unsupported state the stop names.
fn cause_reason(cause: StopCause, lost: HtmlParseEngineCondition) -> HtmlParseFailureReason {
    match cause {
        StopCause::LostOwner => HtmlParseFailureReason::Engine(lost),
        StopCause::Session(error) => session_reason(error),
        StopCause::Unsupported(state) => {
            HtmlParseFailureReason::Stop(HtmlParseStop::UnsupportedStandaloneState(state))
        }
    }
}
fn kernel_reason(e: &super::KernelError) -> HtmlParseFailureReason {
    cause_reason(e.stop_cause(), HtmlParseEngineCondition::Tree)
}
fn head_reason(e: &super::head::HeadError) -> HtmlParseFailureReason {
    match e {
        super::head::HeadError::Kernel(k) => kernel_reason(k),
        super::head::HeadError::Prologue(p) => prologue_reason(p),
        _ => cause_reason(e.stop_cause(), HtmlParseEngineCondition::Tree),
    }
}
fn prologue_reason(e: &super::prologue::PrologueError) -> HtmlParseFailureReason {
    match e {
        super::prologue::PrologueError::Kernel(k) => kernel_reason(k),
        _ => cause_reason(e.stop_cause(), HtmlParseEngineCondition::Tokenizer),
    }
}
fn ordinary_reason(e: &StandaloneError) -> HtmlParseFailureReason {
    match e {
        StandaloneError::Head(h) => head_reason(h),
        StandaloneError::Prologue(p) => prologue_reason(p),
        // These two name the unsupported state in a reason of their own; the
        // stop is the provenance boundary of the same cause.
        StandaloneError::UnsupportedDocumentInitialization => {
            HtmlParseFailureReason::UnsupportedDocumentInitialization
        }
        StandaloneError::UnsupportedExecutionProfile => {
            HtmlParseFailureReason::UnsupportedExecutionProfile
        }
        StandaloneError::Stop(_)
        | StandaloneError::Consumed
        | StandaloneError::InputBankRefused
        | StandaloneError::TokenizerExhausted => {
            cause_reason(e.stop_cause(), HtmlParseEngineCondition::Ownership)
        }
    }
}
fn fragment_reason(e: &FragmentExecutionError) -> HtmlParseFailureReason {
    match e {
        FragmentExecutionError::Kernel(k) => kernel_reason(k),
        FragmentExecutionError::Head(h) => head_reason(h),
        FragmentExecutionError::Stop(_) | FragmentExecutionError::Consumed => {
            cause_reason(e.stop_cause(), HtmlParseEngineCondition::Ownership)
        }
    }
}
#[expect(
    clippy::large_enum_variant,
    reason = "the variants own the very owners a stop must hand over, without allocating after it"
)]
enum OrdinaryAttempt {
    Ready(FrozenStandaloneParts),
    Failed(HtmlParseFailureReason, DetachedStandaloneFailure),
    Initialization(HtmlParseFailureReason, Option<HtmlParseEngineCondition>),
}
/// Ends an execution that stopped: the driver is cleaned and its failure
/// detached, or, if either step is refused, the request-owning carrier reports
/// the failure with a cleanup error. The driver cannot move into the carrier,
/// since it borrows the request the carrier owns.
fn abandon_ordinary(
    mut driver: StandaloneDocumentDriver<'_>,
    reason: HtmlParseFailureReason,
    cause: Option<StopCause>,
) -> OrdinaryAttempt {
    if let Some(cause) = cause {
        cause.latch(&mut driver.control);
    }
    let Ok(failed) = driver.into_failed() else {
        return OrdinaryAttempt::Initialization(reason, Some(HtmlParseEngineCondition::Cleanup));
    };
    detach_ordinary(failed, reason)
}
fn detach_ordinary(
    failed: FailedStandaloneDocument<'_>,
    reason: HtmlParseFailureReason,
) -> OrdinaryAttempt {
    match failed.into_detached() {
        Ok(failure) => OrdinaryAttempt::Failed(reason, failure),
        // The cleaned failure could not be detached from its source binding.
        Err(_) => {
            OrdinaryAttempt::Initialization(reason, Some(HtmlParseEngineCondition::Ownership))
        }
    }
}
fn ordinary_attempt(
    owner: &mut OrdinaryInitialization,
    mut service: Option<&mut dyn crate::meta_csp::Service>,
    fixed: usize,
    initial_base: bool,
) -> OrdinaryAttempt {
    let mut attempt = match owner.begin_report() {
        Ok(a) => a,
        Err(e) => return OrdinaryAttempt::Initialization(ordinary_reason(&e), None),
    };
    let mut driver = match attempt.take_driver() {
        Ok(d) => d,
        Err(e) => {
            let reason = ordinary_reason(&e);
            let refused = attempt
                .retain_failure()
                .is_err()
                .then_some(HtmlParseEngineCondition::Cleanup);
            return OrdinaryAttempt::Initialization(reason, refused);
        }
    };
    // What ends the execution, for the stop it reports if nothing latched one
    // first; a failure that latched its own stop leaves none to name.
    let mut cause = None;
    // Actual output receivers are admitted on this public-only path before
    // its first token pull; no heap indirection is introduced.
    let mut reason = super::write(
        const { std::mem::size_of::<OrdinaryAttempt>() },
        3,
        &mut driver.control,
    )
    .err()
    .map(HtmlParseFailureReason::Stop);
    if service.is_some() && reason.is_none() {
        if let Err(e) = super::write(fixed, 2, &mut driver.control) {
            reason = Some(HtmlParseFailureReason::Stop(e));
        }
        if reason.is_none()
            && let Err(e) = driver.enable_meta_csp(initial_base)
        {
            reason = Some(ordinary_reason(&e));
            cause = Some(e.stop_cause());
        }
    }
    while reason.is_none() {
        match driver.step_with_meta_csp(&mut service) {
            Ok(HeadStep::Consumed) => {}
            Ok(HeadStep::Complete) => break,
            Ok(HeadStep::Boundary(b)) => {
                reason = Some(HtmlParseFailureReason::UnsupportedConstruction(b));
                cause = Some(StopCause::Unsupported(
                    HtmlParseUnsupportedState::Provenance,
                ));
                break;
            }
            Err(e) => {
                reason = Some(ordinary_reason(&e));
                cause = Some(e.stop_cause());
                break;
            }
        }
    }
    if let Some(reason) = reason {
        return abandon_ordinary(driver, reason, cause);
    }
    // A refused completion has already latched a lost owner as an invariant.
    let mut completed = match driver.into_completed() {
        Ok(completed) => completed,
        Err((error, driver)) => {
            return abandon_ordinary(driver, ordinary_reason(&error), Some(error.stop_cause()));
        }
    };
    let completion = completed
        .prepare_native_associations()
        .and_then(|()| completed.freeze_parts());
    match completion {
        Ok(parts) => OrdinaryAttempt::Ready(parts),
        Err(e) => {
            let reason = ordinary_reason(&e);
            completed.stop_for_public_failure(e.stop_cause());
            match completed.take_public_failure() {
                Ok(failed) => detach_ordinary(failed, reason),
                Err(_) => {
                    OrdinaryAttempt::Initialization(reason, Some(HtmlParseEngineCondition::Cleanup))
                }
            }
        }
    }
}
#[expect(
    clippy::result_large_err,
    reason = "returns the actual original request, stopped control and cleanup owner without allocating after a stop"
)]
pub(crate) fn document(
    request: HtmlDocumentParseRequest,
    service: Option<&mut dyn crate::meta_csp::Service>,
    fixed: usize,
) -> Result<HtmlParseReport, HtmlParseFailure> {
    let initial_base = request.initial_policy_base();
    let mut owner = OrdinaryInitialization::new(request);
    let parts = match ordinary_attempt(&mut owner, service, fixed, initial_base) {
        OrdinaryAttempt::Ready(parts) => parts,
        OrdinaryAttempt::Initialization(reason, refused) => {
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::OrdinaryInitialization(owner),
                cleanup_error: refused,
            });
        }
        OrdinaryAttempt::Failed(reason, failure) => {
            return Err(ordinary_failure(owner, reason, failure));
        }
    };
    let sealed = match owner.into_sealed(parts) {
        Ok(sealed) => sealed,
        Err((error, owner, mut parts)) => {
            let reason = ordinary_reason(&error);
            error.stop_cause().latch(&mut parts.control);
            return Err(detach_frozen_ordinary(owner, reason, parts));
        }
    };
    match HtmlParseReport::publish_document(sealed) {
        Ok(report) => Ok(report),
        Err((e, mut sealed)) => {
            let reason = session_reason(e);
            StopCause::Session(e).latch(&mut sealed.parts.control);
            let refused = sealed.parts.cleanup_failure().is_err();
            Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::OrdinaryFrozen(sealed),
                cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
            })
        }
    }
}
/// Ends a frozen ordinary execution that could not be sealed: the stopped
/// parts are cleaned and detached into the failure. If either step is refused
/// the request-owning carrier reports it, a cleanup fault for the cleaning and
/// an ownership fault for the detaching, as the stopped-driver path does.
fn detach_frozen_ordinary(
    owner: OrdinaryInitialization,
    reason: HtmlParseFailureReason,
    mut parts: FrozenStandaloneParts,
) -> HtmlParseFailure {
    let carrier = |owner, fault| HtmlParseFailure {
        reason,
        storage: FailureStorage::OrdinaryInitialization(owner),
        cleanup_error: Some(fault),
    };
    // The detach below finds the cleaning done and goes on to the binding.
    if parts.cleanup_failure().is_err() {
        return carrier(owner, HtmlParseEngineCondition::Cleanup);
    }
    match parts.into_detached_failure() {
        Ok(failure) => ordinary_failure(owner, reason, failure),
        Err(_) => carrier(owner, HtmlParseEngineCondition::Ownership),
    }
}
/// Joins a cleaned failure to the original request. If the join is refused the
/// carrier keeps the request and reports why the failure was not kept.
fn ordinary_failure(
    owner: OrdinaryInitialization,
    reason: HtmlParseFailureReason,
    failure: DetachedStandaloneFailure,
) -> HtmlParseFailure {
    match owner.into_failure(failure) {
        Ok(failure) => HtmlParseFailure {
            reason,
            storage: FailureStorage::Ordinary(failure),
            cleanup_error: None,
        },
        Err((_, owner, _)) => HtmlParseFailure {
            reason,
            storage: FailureStorage::OrdinaryInitialization(owner),
            cleanup_error: Some(HtmlParseEngineCondition::Ownership),
        },
    }
}
#[expect(
    clippy::large_enum_variant,
    reason = "the variants own the very owners a stop must hand over, without allocating after it"
)]
enum FragmentAttempt {
    Ready(FrozenContextFragmentParts),
    Failed(HtmlParseFailureReason, StoppedFragmentConstruction),
    Initialization(HtmlParseFailureReason, Option<HtmlParseEngineCondition>),
}
/// Ends a fragment execution that stopped; a refused step is reported by the
/// request-owning carrier, as for the ordinary driver.
fn abandon_fragment(
    driver: ContextFragmentDriver<'_>,
    reason: HtmlParseFailureReason,
) -> FragmentAttempt {
    match driver.into_failed() {
        Ok(failed) => stop_fragment(failed, reason),
        Err(_) => FragmentAttempt::Initialization(reason, Some(HtmlParseEngineCondition::Cleanup)),
    }
}
fn stop_fragment(
    failed: FailedContextFragment<'_>,
    reason: HtmlParseFailureReason,
) -> FragmentAttempt {
    match StoppedFragmentConstruction::from_failed(failed) {
        Ok(stopped) => FragmentAttempt::Failed(reason, stopped),
        // The failure did not qualify against its request and binding.
        Err(_) => {
            FragmentAttempt::Initialization(reason, Some(HtmlParseEngineCondition::Ownership))
        }
    }
}
fn fragment_attempt(owner: &mut FragmentInitialization) -> FragmentAttempt {
    let mut attempt = match owner.begin_report() {
        Ok(attempt) => attempt,
        Err(_) => {
            return FragmentAttempt::Initialization(
                HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Ownership),
                None,
            );
        }
    };
    let mut driver =
        match attempt.take_driver() {
            Ok(d) => d,
            Err(_) => {
                let why = attempt.reason().map(fragment_reason).unwrap_or(
                    HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Ownership),
                );
                let refused = attempt
                    .retain_failure()
                    .is_err()
                    .then_some(HtmlParseEngineCondition::Cleanup);
                return FragmentAttempt::Initialization(why, refused);
            }
        };
    let mut cause = None;
    let why = loop {
        match driver.step() {
            Ok(HeadStep::Consumed) => {}
            Ok(HeadStep::Complete) => break None,
            Ok(HeadStep::Boundary(b)) => {
                cause = Some(StopCause::Unsupported(
                    HtmlParseUnsupportedState::Provenance,
                ));
                break Some(HtmlParseFailureReason::UnsupportedConstruction(b));
            }
            Err(e) => {
                cause = Some(e.stop_cause());
                break Some(fragment_reason(&e));
            }
        }
    };
    if let Some(why) = why {
        if let Some(cause) = cause {
            driver.stop_for_public_failure(cause);
        }
        return abandon_fragment(driver, why);
    }
    // A refused completion has already latched a lost owner as an invariant.
    let mut completed = match driver.into_completed() {
        Ok(completed) => completed,
        Err((error, mut driver)) => {
            driver.stop_for_public_failure(error.stop_cause());
            return abandon_fragment(driver, fragment_reason(&error));
        }
    };
    match completed
        .prepare_native_associations()
        .and_then(|()| completed.freeze_parts())
    {
        Ok(parts) => FragmentAttempt::Ready(parts),
        Err(e) => {
            let why = fragment_reason(&e);
            completed.stop_for_public_failure(e.stop_cause());
            match completed.take_failed() {
                Ok(failed) => stop_fragment(failed, why),
                Err(_) => {
                    FragmentAttempt::Initialization(why, Some(HtmlParseEngineCondition::Cleanup))
                }
            }
        }
    }
}
#[expect(
    clippy::result_large_err,
    reason = "returns the actual original request, stopped control and cleanup owner without allocating after a stop"
)]
pub(crate) fn fragment(
    request: HtmlFragmentParseRequest,
) -> Result<HtmlParseReport, HtmlParseFailure> {
    let mut owner = FragmentInitialization::new(request);
    let parts = match fragment_attempt(&mut owner) {
        FragmentAttempt::Ready(parts) => parts,
        FragmentAttempt::Initialization(reason, refused) => {
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::FragmentInitialization(owner),
                cleanup_error: refused,
            });
        }
        FragmentAttempt::Failed(reason, stopped) => {
            return Err(fragment_failure(owner, reason, stopped));
        }
    };
    let sealed = match owner.into_sealed(parts) {
        Ok(sealed) => sealed,
        Err((error, owner, mut parts)) => {
            let reason = fragment_reason(&error);
            error.stop_cause().latch(&mut parts.control);
            return Err(detach_frozen_fragment(owner, reason, parts));
        }
    };
    match HtmlParseReport::publish_fragment(sealed) {
        Ok(report) => Ok(report),
        Err((e, mut sealed)) => {
            let reason = session_reason(e);
            StopCause::Session(e).latch(&mut sealed.parts.control);
            let refused = sealed.parts.cleanup_failure().is_err();
            Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::FragmentFrozen(sealed),
                cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
            })
        }
    }
}
/// Ends a frozen fragment execution that could not be sealed, as
/// [`detach_frozen_ordinary`] does for an ordinary one.
pub(super) fn detach_frozen_fragment(
    owner: FragmentInitialization,
    reason: HtmlParseFailureReason,
    mut parts: FrozenContextFragmentParts,
) -> HtmlParseFailure {
    let carrier = |owner, fault| HtmlParseFailure {
        reason,
        storage: FailureStorage::FragmentInitialization(owner),
        cleanup_error: Some(fault),
    };
    // The detach below finds the cleaning done and goes on to the binding.
    if parts.cleanup_failure().is_err() {
        return carrier(owner, HtmlParseEngineCondition::Cleanup);
    }
    match StoppedFragmentConstruction::from_frozen(parts) {
        Ok(stopped) => fragment_failure(owner, reason, stopped),
        Err(_) => carrier(owner, HtmlParseEngineCondition::Ownership),
    }
}
/// Joins a stopped construction to the original request. If either step is
/// refused the carrier keeps the request and the stopped construction, and
/// reports why they were not joined.
fn fragment_failure(
    owner: FragmentInitialization,
    reason: HtmlParseFailureReason,
    stopped: StoppedFragmentConstruction,
) -> HtmlParseFailure {
    let carrier = |owner| HtmlParseFailure {
        reason,
        storage: FailureStorage::FragmentInitialization(owner),
        cleanup_error: Some(HtmlParseEngineCondition::Ownership),
    };
    match owner.retain_stopped(stopped) {
        Ok(owner) => match owner.into_failure() {
            Ok(failure) => HtmlParseFailure {
                reason,
                storage: FailureStorage::Fragment(failure),
                cleanup_error: None,
            },
            Err((_, owner)) => carrier(owner),
        },
        Err((_, owner, _)) => carrier(owner),
    }
}
/// Exact added physical terminal envelope, admitted before first token pull.
pub(super) const fn terminal_layout() -> (usize, usize) {
    const {
        use std::mem::size_of;
        (
            size_of::<HtmlParseFailure>()
                + size_of::<FailureStorage>()
                + size_of::<HtmlParseFailureReason>()
                + size_of::<Option<HtmlParseEngineCondition>>(),
            4,
        )
    }
}

#[derive(Clone, Copy)]
pub(super) enum InitializationTarget {
    #[cfg(test)]
    Native,
    Report,
}
impl InitializationTarget {
    pub(super) fn admit(
        self,
        c: &mut crate::parser::control::ParseControl,
    ) -> Result<(), HtmlParseStop> {
        match self {
            #[cfg(test)]
            Self::Native => Ok(()),
            Self::Report => {
                let (bytes, writes) = terminal_layout();
                super::write(bytes, writes, c)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HtmlParseResource;

    /// Frozen parts of a finished ordinary parse and the carrier that owns its
    /// request, stopped as a refused seal would stop them.
    fn frozen_ordinary_parts() -> (OrdinaryInitialization, FrozenStandaloneParts) {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        );
        let initial_base = request.initial_policy_base();
        let mut owner = OrdinaryInitialization::new(request);
        let OrdinaryAttempt::Ready(mut parts) = ordinary_attempt(&mut owner, None, 0, initial_base)
        else {
            panic!("the fixture parses");
        };
        parts
            .control
            .stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance);
        (owner, parts)
    }

    #[test]
    fn a_frozen_failure_whose_source_binding_is_gone_is_reported_as_an_ownership_fault() {
        let (owner, mut parts) = frozen_ordinary_parts();
        // Cleanup succeeds; only moving the source binding out is refused.
        parts.control.take_stopped_source_binding().unwrap();
        let failure = detach_frozen_ordinary(
            owner,
            HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled),
            parts,
        );
        assert_eq!(
            failure.cleanup_error(),
            Some(HtmlParseEngineCondition::Ownership)
        );
    }

    #[test]
    fn a_frozen_failure_whose_cleanup_is_refused_is_reported_as_a_cleanup_fault() {
        let (owner, mut parts) = frozen_ordinary_parts();
        parts.control.refuse_next_construction_retire();
        let failure = detach_frozen_ordinary(
            owner,
            HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled),
            parts,
        );
        assert_eq!(
            failure.cleanup_error(),
            Some(HtmlParseEngineCondition::Cleanup)
        );
    }

    #[test]
    fn csp_storage_returned_to_a_failure_without_the_ordinary_driver_is_reported() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        );
        request.cancellation().cancel();
        let mut failed = request.parse().unwrap_err();
        assert!(!matches!(failed.storage, FailureStorage::Ordinary(_)));
        assert_eq!(failed.cleanup_error(), None);
        let retained = failed.retained_usage();
        let claimed = failed.claim_csp_storage(crate::meta_csp::Storage {
            bytes: 1,
            units: 1,
            entries: 1,
        });
        assert_eq!(claimed, ReturnableCspStorage::NONE);
        failed.return_csp_storage(claimed).unwrap();
        assert_eq!(
            failed.cleanup_error(),
            Some(HtmlParseEngineCondition::Ownership)
        );
        assert_eq!(
            failed.reason(),
            HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled)
        );
        assert_eq!(failed.retained_usage(), retained);
    }

    /// An ordinary failure whose controller was holding processor storage when
    /// a cancellation stopped the parse, built through the real driver chain.
    fn ordinary_failure_holding(storage: crate::meta_csp::Storage) -> HtmlParseFailure {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        );
        let mut driver = Some(super::super::entry::initialized_driver(&request));
        storage
            .acquire(&mut driver.as_mut().unwrap().control)
            .unwrap();
        request.cancellation().cancel();
        assert!(driver.as_mut().unwrap().step().is_err());
        let mut failed = Some(StandaloneDocumentDriver::take_failed(&mut driver).unwrap());
        let detached = FailedStandaloneDocument::detach(&mut failed).unwrap();
        drop(failed);
        drop(driver);
        let owned = OwnedOrdinaryFailure::take(&mut Some(request), &mut Some(detached)).unwrap();
        HtmlParseFailure {
            reason: HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled),
            storage: FailureStorage::Ordinary(owned),
            cleanup_error: None,
        }
    }

    #[test]
    fn csp_storage_the_ordinary_controller_admitted_is_claimed_and_returned_in_full() {
        let storage = crate::meta_csp::Storage {
            bytes: 8,
            units: 2,
            entries: 1,
        };
        let mut failed = ordinary_failure_holding(storage);
        assert!(matches!(failed.storage, FailureStorage::Ordinary(_)));
        let held = failed.retained_usage().unwrap();
        let claimed = failed.claim_csp_storage(storage);
        assert_eq!(claimed, ReturnableCspStorage(storage));
        assert_eq!(failed.cleanup_error(), None);
        // Claiming decides; it changes no accounting.
        assert_eq!(failed.retained_usage().unwrap(), held);
        failed.return_csp_storage(claimed).unwrap();
        let returned = failed.retained_usage().unwrap();
        for (resource, amount) in [
            (HtmlParseResource::ManagedCapacityBytes, 8),
            (HtmlParseResource::PeakTransientBytes, 8),
            (HtmlParseResource::ManagedUtf16Units, 2),
            (HtmlParseResource::AuxiliaryEntries, 1),
        ] {
            assert_eq!(
                returned.resource(resource).current,
                held.resource(resource).current - amount,
                "{resource:?}"
            );
        }
        assert_eq!(failed.cleanup_error(), None);
        assert_eq!(
            failed.reason(),
            HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled)
        );
    }

    #[test]
    fn a_csp_failure_envelope_keeps_the_claim_it_returns_when_dropped() {
        let storage = crate::meta_csp::Storage {
            bytes: 8,
            units: 2,
            entries: 1,
        };
        let failure = crate::meta_csp::HtmlParseCspFailure::<()>::new(
            ordinary_failure_holding(storage),
            None,
            crate::meta_csp::HtmlParseCspFailureKind::Cancelled,
            storage,
        );
        // The drop returns exactly this claim, and the caller can see that
        // nothing was left unreleased.
        assert_ne!(failure.storage, ReturnableCspStorage::NONE);
        assert_eq!(failure.parse_failure().cleanup_error(), None);
        drop(failure);
    }

    #[test]
    fn a_failure_whose_cleanup_was_refused_still_reports_its_first_stop() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        );
        let failed = HtmlParseFailure {
            reason: HtmlParseFailureReason::Stop(HtmlParseStop::Cancelled),
            storage: FailureStorage::OrdinaryInitialization(OrdinaryInitialization::new(request)),
            cleanup_error: Some(HtmlParseEngineCondition::Cleanup),
        };
        // The carrier holds no stopped prefix, but execution ran and was
        // stopped: the stop the reason names is the first one.
        assert_eq!(failed.first_stop(), Some(HtmlParseStop::Cancelled));
        assert_eq!(failed.source_admitted(), None);
        assert_eq!(
            failed.cleanup_error(),
            Some(HtmlParseEngineCondition::Cleanup)
        );
        // Without a cleanup condition, the same carrier names no stop.
        let rejected = HtmlParseFailure {
            reason: HtmlParseFailureReason::UnsupportedDocumentInitialization,
            storage: FailureStorage::OrdinaryInitialization(OrdinaryInitialization::new(
                HtmlDocumentParseRequest::offline(
                    HtmlSource::utf8("x"),
                    super::super::tests::limits(100),
                ),
            )),
            cleanup_error: None,
        };
        assert_eq!(rejected.first_stop(), None);
    }

    #[test]
    fn csp_storage_the_ordinary_controller_never_admitted_is_reported_not_claimed() {
        let tight = super::super::tests::limits(100)
            .into_builder()
            .managed_nodes(4)
            .finish()
            .unwrap();
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>x</p>"), tight);
        let mut failed = request.parse().unwrap_err();
        assert!(matches!(failed.storage, FailureStorage::Ordinary(_)));
        assert_eq!(failed.cleanup_error(), None);
        let retained = failed.retained_usage();
        // The controller admitted no provider storage, so a return of this
        // storage could only underflow it.
        let claimed = failed.claim_csp_storage(crate::meta_csp::Storage {
            bytes: 8,
            units: 2,
            entries: 1,
        });
        assert_eq!(claimed, ReturnableCspStorage::NONE);
        assert_eq!(
            failed.cleanup_error(),
            Some(HtmlParseEngineCondition::Cleanup)
        );
        assert_eq!(failed.retained_usage(), retained);
    }

    #[test]
    fn public_terminal_receiver_exact_and_one_short_preserve_source_first_stop_and_cleanup() {
        // Actual new terminal receiver plus its moved storage/reason/optional
        // cleanup receiver. No tree or token backing is synthesized here.
        let bytes = std::mem::size_of::<HtmlParseFailure>()
            + std::mem::size_of::<FailureStorage>()
            + std::mem::size_of::<HtmlParseFailureReason>()
            + std::mem::size_of::<Option<HtmlParseEngineCondition>>();

        let literal = "<!doctype html><p>x";
        let limits = super::super::tests::limits(100);
        let failed = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(literal),
            limits
                .into_builder()
                .atomic_layout_bytes((bytes - 1) as u64)
                .finish()
                .unwrap(),
        )
        .parse()
        .unwrap_err();
        assert!(
            matches!(failed.reason(),HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded{resource:HtmlParseResource::AtomicLayoutBytes,requested_delta,..}) if requested_delta==bytes as u64)
        );
        assert_eq!(failed.source_admitted(), Some(true));
        assert_eq!(failed.source().as_utf8(), Some(literal));
        assert_eq!(failed.cleanup_error(), None);
        assert_eq!(
            failed
                .after_cleanup()
                .unwrap()
                .resource(HtmlParseResource::ManagedNodes)
                .current(),
            0
        );
        // Exactly admitting this receiver continues to the larger existing
        // original-request initialization envelope, still before token pull.
        let next = super::super::seal::initialization_bootstrap_layout().0;
        assert!(next > bytes);
        let exact = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(literal),
            limits
                .into_builder()
                .atomic_layout_bytes(bytes as u64)
                .finish()
                .unwrap(),
        )
        .parse()
        .unwrap_err();
        assert!(
            matches!(exact.reason(),HtmlParseFailureReason::Stop(HtmlParseStop::LimitExceeded{resource:HtmlParseResource::AtomicLayoutBytes,requested_delta,..}) if requested_delta==next as u64)
        );
        assert_eq!(
            exact
                .at_stop()
                .unwrap()
                .resource(HtmlParseResource::AtomicLayoutBytes)
                .high_water(),
            bytes as u64
        );
        assert_eq!(
            exact
                .after_cleanup()
                .unwrap()
                .resource(HtmlParseResource::ManagedNodes)
                .current(),
            0
        );
        assert_eq!(exact.cleanup_error(), None);
    }

    /// The reason a stage error reports for an outcome is the one the first
    /// stop agrees with: the outcome's own, or a stop that names it.
    #[test]
    fn a_stage_error_holding_an_outcome_reports_the_outcome_reason() {
        use super::super::{KernelError, head::HeadError, prologue::PrologueError};
        for (error, reason, _) in super::super::outcome_tests::outcomes() {
            assert_eq!(session_reason(error), reason, "{error:?}");
            let session = || KernelError::Session(error);
            let ordinary = [
                StandaloneError::Head(HeadError::Kernel(session())),
                StandaloneError::Prologue(PrologueError::Kernel(session())),
                StandaloneError::Head(HeadError::Prologue(PrologueError::Kernel(session()))),
            ];
            for stage in &ordinary {
                assert_eq!(ordinary_reason(stage), reason, "{error:?}");
            }
            let fragment = [
                FragmentExecutionError::Kernel(session()),
                FragmentExecutionError::Head(HeadError::Kernel(session())),
                FragmentExecutionError::Head(HeadError::Prologue(PrologueError::Kernel(session()))),
            ];
            for stage in &fragment {
                assert_eq!(fragment_reason(stage), reason, "{error:?}");
            }
        }
        // The completion stage's own error is the same completion state.
        let completion = HtmlParseFailureReason::Stop(HtmlParseStop::UnsupportedStandaloneState(
            HtmlParseUnsupportedState::Completion,
        ));
        assert_eq!(
            ordinary_reason(&StandaloneError::Head(HeadError::IncompleteCompletion)),
            completion
        );
        assert_eq!(
            fragment_reason(&FragmentExecutionError::Head(
                HeadError::IncompleteCompletion
            )),
            completion
        );
    }

    /// A parser fault that reaches a stage is an engine condition.
    #[test]
    fn a_stage_error_holding_a_parser_fault_reports_an_engine_condition() {
        use super::super::{KernelError, head::HeadError};
        for (error, _) in super::super::outcome_tests::faults() {
            let engine = HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Tree);
            assert_eq!(session_reason(error), engine, "{error:?}");
            let stage = StandaloneError::Head(HeadError::Kernel(KernelError::Session(error)));
            assert_eq!(ordinary_reason(&stage), engine, "{error:?}");
        }
    }

    /// Every session error is either an outcome, whose reason is not an engine
    /// condition and which the table of outcomes covers, or a fault of the
    /// parser's own, whose reason is an engine condition. A stop already
    /// latched is its own reason.
    #[test]
    fn every_session_error_is_an_outcome_with_its_reason_or_an_engine_condition() {
        use crate::construction::{SessionClass, session_class};
        let engine = HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Tree);
        let table = super::super::outcome_tests::outcomes();
        for error in super::super::outcome_tests::every_session_error() {
            match session_class(&error) {
                SessionClass::Fault(_) => assert_eq!(session_reason(error), engine, "{error:?}"),
                SessionClass::Outcome(stop) => {
                    assert_ne!(session_reason(error), engine, "{error:?}");
                    if let SessionError::Stop(_) = error {
                        assert_eq!(session_reason(error), HtmlParseFailureReason::Stop(stop));
                    } else {
                        assert!(
                            table.iter().any(|(listed, reason, listed_stop)| {
                                std::mem::discriminant(listed) == std::mem::discriminant(&error)
                                    && *reason == session_reason(error)
                                    && *listed_stop == stop
                            }),
                            "{error:?} has no entry in the table of outcomes"
                        );
                    }
                }
            }
        }
    }

    /// A stage error that stands for a state the closed environment does not
    /// support reports that state as its reason, in both drivers, which is the
    /// state the first stop names.
    #[test]
    fn a_stage_state_the_environment_does_not_support_reports_it_as_its_reason() {
        for (error, state) in super::super::outcome_tests::stage_states() {
            let reason =
                HtmlParseFailureReason::Stop(HtmlParseStop::UnsupportedStandaloneState(state));
            assert_eq!(
                ordinary_reason(&StandaloneError::Head(error())),
                reason,
                "{state:?}"
            );
            assert_eq!(
                fragment_reason(&FragmentExecutionError::Head(error())),
                reason,
                "{state:?}"
            );
        }
    }

    /// The reason for an error of the tree builder's own structures that input
    /// may reach is the unsupported state its stop names, in both drivers.
    #[test]
    fn a_structure_error_reports_the_unsupported_nesting_state_as_its_reason() {
        let reason = HtmlParseFailureReason::Stop(HtmlParseStop::UnsupportedStandaloneState(
            HtmlParseUnsupportedState::InvalidNesting,
        ));
        for (name, error) in super::super::outcome_tests::structure_errors() {
            assert_eq!(
                ordinary_reason(&StandaloneError::Head(error())),
                reason,
                "{name}"
            );
            assert_eq!(
                fragment_reason(&FragmentExecutionError::Head(error())),
                reason,
                "{name}"
            );
        }
    }

    /// A document initialization or execution profile the standalone
    /// environment was given no proof of keeps its own reason in both stages.
    #[test]
    fn an_unsupported_request_shape_keeps_its_own_reason() {
        assert_eq!(
            ordinary_reason(&StandaloneError::UnsupportedDocumentInitialization),
            HtmlParseFailureReason::UnsupportedDocumentInitialization
        );
        assert_eq!(
            ordinary_reason(&StandaloneError::UnsupportedExecutionProfile),
            HtmlParseFailureReason::UnsupportedExecutionProfile
        );
    }

    /// One Meta CSP stop is one reason, whether the service raised it as a
    /// session error or the controller latched it as a stop.
    #[test]
    fn a_meta_csp_stop_has_the_same_reason_as_the_error_that_names_it() {
        let protocol = crate::construction::csp_protocol_error();
        assert_eq!(
            session_reason(SessionError::Stop(HtmlParseStop::CspProcessing)),
            session_reason(SessionError::CspService)
        );
        assert_eq!(
            session_reason(SessionError::Stop(HtmlParseStop::CspProtocol(protocol))),
            session_reason(SessionError::CspProtocol(protocol))
        );
    }
}
