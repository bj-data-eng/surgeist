//! Consuming public operations over the real request, execution and report owners.
use super::{
    StandaloneDocumentDriver, StandaloneError,
    context::{
        ContextFragmentDriver, FailedContextFragment, FragmentExecutionError,
        FrozenContextFragmentParts,
        ownership::{
            FragmentInitialization, OwnedFragmentFailure, SealedFragmentConstruction,
            StoppedFragmentConstruction,
        },
    },
    head::{HeadBoundary, HeadStep},
    report::ImmutableParseReport,
    seal::{OrdinaryInitialization, OwnedOrdinaryFailure, SealedOrdinaryConstruction},
    terminal::{DetachedStandaloneFailure, FailedStandaloneDocument, FrozenStandaloneParts},
};
use crate::construction::{SessionError, evidence::DiagnosticEvidenceView};
use crate::parser::control::{ParseStop, ParseUsage, StandaloneUnsupported};
use crate::{HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseLimits, HtmlSource};

/// A terminal reason is separate from recoverable syntax diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseFailureReason {
    CspProtocol(surgeist_html::HtmlCspProtocolError),
    CspProcessing,
    Stop(ParseStop),
    UnsupportedDocumentInitialization,
    UnsupportedExecutionProfile,
    UnsupportedConstruction(HeadBoundary),
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
        let condition = match &self.storage {
            FailureStorage::Ordinary(owner) if owner.csp_storage_fits(storage) => {
                return ReturnableCspStorage(storage);
            }
            FailureStorage::Ordinary(_) => HtmlParseEngineCondition::Cleanup,
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
    ) -> Result<(), ParseStop> {
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
    pub fn cleanup_error(&self) -> Option<HtmlParseEngineCondition> {
        self.cleanup_error
    }
    /// The original execution stop, including an unsupported construction's
    /// canonical terminal stop. Pre-execution rejection has no invented stop.
    pub fn first_stop(&self) -> Option<ParseStop> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => match o.failure() {
                Some(super::entry::InitializationFailure::Stopped(stop)) => Some(stop),
                _ => None,
            },
            FailureStorage::Ordinary(o) => Some(o.stop()),
            FailureStorage::FragmentInitialization(o) => o.failure().map(|f| f.stop()),
            FailureStorage::Fragment(o) => Some(o.stop()),
            FailureStorage::OrdinaryFrozen(o) => o.parts.control.stop(),
            FailureStorage::FragmentFrozen(o) => o.parts.control.stop(),
        }
    }
    /// None means rejection before an execution control was established.
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
    pub fn retained_usage(&self) -> Option<&ParseUsage> {
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
    pub fn requested_scripting_mode(&self) -> crate::HtmlParserScriptingMode {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.request().scripting_mode(),
            FailureStorage::Ordinary(_) | FailureStorage::OrdinaryFrozen(_) => {
                crate::HtmlParserScriptingMode::Disabled
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
    pub fn at_stop(&self) -> Option<&ParseUsage> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.at_stop(),
            FailureStorage::Ordinary(o) => o.at_stop(),
            FailureStorage::FragmentInitialization(o) => o.failure().map(|f| f.at_stop()),
            FailureStorage::Fragment(o) => Some(o.at_stop()),
            FailureStorage::OrdinaryFrozen(o) => o.parts.at_stop(),
            FailureStorage::FragmentFrozen(o) => o.parts.at_stop(),
        }
    }
    pub fn after_cleanup(&self) -> Option<&ParseUsage> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.after_cleanup(),
            FailureStorage::Ordinary(o) => Some(o.after_cleanup()),
            FailureStorage::FragmentInitialization(o) => o.failure().map(|f| f.after_cleanup()),
            FailureStorage::Fragment(o) => Some(o.after_cleanup()),
            FailureStorage::OrdinaryFrozen(o) => o.parts.after_cleanup(),
            FailureStorage::FragmentFrozen(o) => o.parts.after_cleanup(),
        }
    }
    pub fn diagnostics(&self) -> Option<DiagnosticEvidenceView<'_>> {
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

fn session_reason(e: SessionError) -> HtmlParseFailureReason {
    match e {
        SessionError::CspService => HtmlParseFailureReason::CspProcessing,
        SessionError::CspProtocol(p) => HtmlParseFailureReason::CspProtocol(p),
        SessionError::Stop(s) => HtmlParseFailureReason::Stop(s),
        SessionError::AssociatedFacts => {
            HtmlParseFailureReason::Stop(ParseStop::InvalidAssociatedFacts)
        }
        SessionError::ContextFactUnavailable(f) => HtmlParseFailureReason::MissingContextFact(f),
        SessionError::ContextReactionUnavailable => {
            HtmlParseFailureReason::UnsupportedContextReactions
        }
        _ => HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Tree),
    }
}
fn kernel_reason(e: &super::KernelError) -> HtmlParseFailureReason {
    match e {
        super::KernelError::Stop(s) => HtmlParseFailureReason::Stop(*s),
        super::KernelError::Session(s) => session_reason(*s),
        _ => HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Tree),
    }
}
fn head_reason(e: &super::head::HeadError) -> HtmlParseFailureReason {
    match e {
        super::head::HeadError::Kernel(k) => kernel_reason(k),
        super::head::HeadError::Prologue(p) => prologue_reason(p),
        super::head::HeadError::Handoff(crate::parser::tokenizer::TokenizerHandoffError::Stop(
            stop,
        )) => HtmlParseFailureReason::Stop(*stop),
        _ => HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Tree),
    }
}
fn prologue_reason(e: &super::prologue::PrologueError) -> HtmlParseFailureReason {
    match e {
        super::prologue::PrologueError::Kernel(k) => kernel_reason(k),
        _ => HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Tokenizer),
    }
}
fn ordinary_reason(e: &StandaloneError) -> HtmlParseFailureReason {
    match e {
        StandaloneError::Stop(s) => HtmlParseFailureReason::Stop(*s),
        StandaloneError::Head(h) => head_reason(h),
        StandaloneError::Prologue(p) => prologue_reason(p),
        StandaloneError::UnsupportedDocumentInitialization => {
            HtmlParseFailureReason::UnsupportedDocumentInitialization
        }
        StandaloneError::UnsupportedExecutionProfile => {
            HtmlParseFailureReason::UnsupportedExecutionProfile
        }
        _ => HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Ownership),
    }
}
fn fragment_reason(e: &FragmentExecutionError) -> HtmlParseFailureReason {
    match e {
        FragmentExecutionError::Stop(s) => HtmlParseFailureReason::Stop(*s),
        FragmentExecutionError::Kernel(k) => kernel_reason(k),
        FragmentExecutionError::Head(h) => head_reason(h),
        _ => HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Ownership),
    }
}
#[expect(
    clippy::large_enum_variant,
    reason = "the variants own the very owners a stop must hand over, without allocating after it"
)]
enum OrdinaryAttempt {
    Ready(FrozenStandaloneParts),
    Failed(HtmlParseFailureReason, DetachedStandaloneFailure),
    Initialization(HtmlParseFailureReason, bool),
}
/// Ends an execution that stopped: the driver is cleaned and its failure
/// detached, or, if either step is refused, the request-owning carrier reports
/// the failure with a cleanup error. The driver cannot move into the carrier,
/// since it borrows the request the carrier owns.
fn abandon_ordinary(
    mut driver: StandaloneDocumentDriver<'_>,
    reason: HtmlParseFailureReason,
    lost_owner: bool,
) -> OrdinaryAttempt {
    if lost_owner {
        StandaloneError::owner_lost(&mut driver.control);
    } else {
        driver
            .control
            .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
    }
    let Ok(failed) = driver.into_failed() else {
        return OrdinaryAttempt::Initialization(reason, true);
    };
    detach_ordinary(failed, reason)
}
fn detach_ordinary(
    failed: FailedStandaloneDocument<'_>,
    reason: HtmlParseFailureReason,
) -> OrdinaryAttempt {
    match failed.into_detached() {
        Ok(failure) => OrdinaryAttempt::Failed(reason, failure),
        Err(_) => OrdinaryAttempt::Initialization(reason, true),
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
        Err(e) => return OrdinaryAttempt::Initialization(ordinary_reason(&e), false),
    };
    let mut driver = match attempt.take_driver() {
        Ok(d) => d,
        Err(e) => {
            let reason = ordinary_reason(&e);
            let cleanup_failed = attempt.retain_failure().is_err();
            return OrdinaryAttempt::Initialization(reason, cleanup_failed);
        }
    };
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
        }
    }
    while reason.is_none() {
        match driver.step_with_meta_csp(&mut service) {
            Ok(HeadStep::Consumed) => {}
            Ok(HeadStep::NeedInput) => {
                reason = Some(HtmlParseFailureReason::Engine(
                    HtmlParseEngineCondition::Tokenizer,
                ));
                break;
            }
            Ok(HeadStep::Complete) => break,
            Ok(HeadStep::Boundary(b)) => {
                reason = Some(HtmlParseFailureReason::UnsupportedConstruction(b));
                break;
            }
            Err(e) => {
                reason = Some(ordinary_reason(&e));
                break;
            }
        }
    }
    if let Some(reason) = reason {
        return abandon_ordinary(driver, reason, false);
    }
    // A refused completion has already latched a lost owner as an invariant.
    let mut completed = match driver.into_completed() {
        Ok(completed) => completed,
        Err((error, driver)) => {
            return abandon_ordinary(driver, ordinary_reason(&error), error.lost_owner());
        }
    };
    let completion = completed
        .prepare_native_associations()
        .and_then(|()| completed.freeze_parts());
    match completion {
        Ok(parts) => OrdinaryAttempt::Ready(parts),
        Err(e) => {
            let reason = ordinary_reason(&e);
            completed.stop_for_public_failure(e.lost_owner());
            match completed.take_public_failure() {
                Ok(failed) => detach_ordinary(failed, reason),
                Err(_) => OrdinaryAttempt::Initialization(reason, true),
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
) -> Result<ImmutableParseReport, HtmlParseFailure> {
    let initial_base = request.initial_policy_base();
    let mut owner = OrdinaryInitialization::new(request);
    let parts = match ordinary_attempt(&mut owner, service, fixed, initial_base) {
        OrdinaryAttempt::Ready(parts) => parts,
        OrdinaryAttempt::Initialization(reason, refused) => {
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::OrdinaryInitialization(owner),
                cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
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
            if error.lost_owner() {
                StandaloneError::owner_lost(&mut parts.control);
            } else {
                parts
                    .control
                    .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            }
            return Err(match parts.into_detached_failure() {
                Ok(failure) => ordinary_failure(owner, reason, failure),
                Err(_) => HtmlParseFailure {
                    reason,
                    storage: FailureStorage::OrdinaryInitialization(owner),
                    cleanup_error: Some(HtmlParseEngineCondition::Cleanup),
                },
            });
        }
    };
    match ImmutableParseReport::publish_document(sealed) {
        Ok(report) => Ok(report),
        Err((e, mut sealed)) => {
            let reason = session_reason(e);
            sealed
                .parts
                .control
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            let refused = sealed.parts.cleanup_failure().is_err();
            Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::OrdinaryFrozen(sealed),
                cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
            })
        }
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
    Initialization(HtmlParseFailureReason, bool),
}
/// Ends a fragment execution that stopped; a refused step is reported by the
/// request-owning carrier, as for the ordinary driver.
fn abandon_fragment(
    driver: ContextFragmentDriver<'_>,
    reason: HtmlParseFailureReason,
) -> FragmentAttempt {
    match driver.into_failed() {
        Ok(failed) => stop_fragment(failed, reason),
        Err(_) => FragmentAttempt::Initialization(reason, true),
    }
}
fn stop_fragment(
    failed: FailedContextFragment<'_>,
    reason: HtmlParseFailureReason,
) -> FragmentAttempt {
    match StoppedFragmentConstruction::take(&mut Some(failed), &mut None) {
        Ok(stopped) => FragmentAttempt::Failed(reason, stopped),
        Err(_) => FragmentAttempt::Initialization(reason, true),
    }
}
fn fragment_attempt(owner: &mut FragmentInitialization) -> FragmentAttempt {
    let mut attempt = match owner.begin_report() {
        Ok(attempt) => attempt,
        Err(_) => {
            return FragmentAttempt::Initialization(
                HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Ownership),
                false,
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
                let refused = attempt.retain_failure().is_err();
                return FragmentAttempt::Initialization(why, refused);
            }
        };
    let why = loop {
        match driver.step() {
            Ok(HeadStep::Consumed) => {}
            Ok(HeadStep::NeedInput) => {
                break Some(HtmlParseFailureReason::Engine(
                    HtmlParseEngineCondition::Tokenizer,
                ));
            }
            Ok(HeadStep::Complete) => break None,
            Ok(HeadStep::Boundary(b)) => {
                break Some(HtmlParseFailureReason::UnsupportedConstruction(b));
            }
            Err(e) => break Some(fragment_reason(&e)),
        }
    };
    if let Some(why) = why {
        driver.stop_for_public_failure(false);
        return abandon_fragment(driver, why);
    }
    // A refused completion has already latched a lost owner as an invariant.
    let mut completed = match driver.into_completed() {
        Ok(completed) => completed,
        Err((error, mut driver)) => {
            driver.stop_for_public_failure(error.lost_owner());
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
            completed.stop_for_public_failure(e.lost_owner());
            match completed.take_failed() {
                Ok(failed) => stop_fragment(failed, why),
                Err(_) => FragmentAttempt::Initialization(why, true),
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
) -> Result<ImmutableParseReport, HtmlParseFailure> {
    let mut owner = FragmentInitialization::new(request);
    let parts = match fragment_attempt(&mut owner) {
        FragmentAttempt::Ready(parts) => parts,
        FragmentAttempt::Initialization(reason, refused) => {
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::FragmentInitialization(owner),
                cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
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
            if error.lost_owner() {
                FragmentExecutionError::owner_lost(&mut parts.control);
            } else {
                parts
                    .control
                    .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            }
            return Err(
                match StoppedFragmentConstruction::take_frozen(&mut Some(parts), &mut None) {
                    Ok(stopped) => fragment_failure(owner, reason, stopped),
                    Err(_) => HtmlParseFailure {
                        reason,
                        storage: FailureStorage::FragmentInitialization(owner),
                        cleanup_error: Some(HtmlParseEngineCondition::Cleanup),
                    },
                },
            );
        }
    };
    match ImmutableParseReport::publish_fragment(sealed) {
        Ok(report) => Ok(report),
        Err((e, mut sealed)) => {
            let reason = session_reason(e);
            sealed
                .parts
                .control
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            let refused = sealed.parts.cleanup_failure().is_err();
            Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::FragmentFrozen(sealed),
                cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
            })
        }
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
    ) -> Result<(), ParseStop> {
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
        let retained = failed.retained_usage().cloned();
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
            HtmlParseFailureReason::Stop(ParseStop::Cancelled)
        );
        assert_eq!(failed.retained_usage().cloned(), retained);
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
            reason: HtmlParseFailureReason::Stop(ParseStop::Cancelled),
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
        let held = failed.retained_usage().cloned().unwrap();
        let claimed = failed.claim_csp_storage(storage);
        assert_eq!(claimed, ReturnableCspStorage(storage));
        assert_eq!(failed.cleanup_error(), None);
        // Claiming decides; it changes no accounting.
        assert_eq!(failed.retained_usage().cloned().unwrap(), held);
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
            HtmlParseFailureReason::Stop(ParseStop::Cancelled)
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
        let retained = failed.retained_usage().cloned();
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
        assert_eq!(failed.retained_usage().cloned(), retained);
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
            matches!(failed.reason(),HtmlParseFailureReason::Stop(ParseStop::LimitExceeded{resource:HtmlParseResource::AtomicLayoutBytes,requested_delta,..}) if requested_delta==bytes as u64)
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
            matches!(exact.reason(),HtmlParseFailureReason::Stop(ParseStop::LimitExceeded{resource:HtmlParseResource::AtomicLayoutBytes,requested_delta,..}) if requested_delta==next as u64)
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
}
