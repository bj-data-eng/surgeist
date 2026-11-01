//! Consuming public operations over the real request, execution and report owners.
use super::{
    StandaloneDocumentDriver, StandaloneError,
    context::{
        ContextFragmentDriver, FragmentExecutionError,
        ownership::{
            FragmentInitializationOwner, OwnedFragmentFailure, SealedFragmentConstruction,
            StoppedFragmentConstruction,
        },
    },
    head::{HeadBoundary, HeadStep},
    report::ImmutableParseReport,
    seal::{OrdinaryInitializationOwner, OwnedOrdinaryFailure, SealedOrdinaryConstruction},
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
enum FailureStorage {
    OrdinaryInitialization(OrdinaryInitializationOwner),
    Ordinary(OwnedOrdinaryFailure),
    FragmentInitialization(FragmentInitializationOwner),
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
    pub(crate) fn release_csp_storage(&mut self, storage: crate::meta_csp::Storage) {
        match &mut self.storage {
            FailureStorage::Ordinary(owner) => owner.release_csp_storage(storage),
            // A native error is issued only by the original ordinary driver, so
            // no other owner holds the controller its storage was charged to.
            // The storage stays unreleased and the cleanup records why; the
            // original stop and retained evidence are untouched.
            _ => {
                self.cleanup_error
                    .get_or_insert(HtmlParseEngineCondition::Ownership);
            }
        }
    }

    fn associated_inputs(&self) -> Option<&crate::construction::associated::Inputs> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(owner) => {
                owner.request()?.associated_inputs().map(AsRef::as_ref)
            }
            FailureStorage::Ordinary(owner) => owner
                .original_request()
                .associated_inputs()
                .map(AsRef::as_ref),
            FailureStorage::OrdinaryFrozen(owner) => {
                owner.request.associated_inputs().map(AsRef::as_ref)
            }
            FailureStorage::FragmentInitialization(owner) => owner
                .original_request()?
                .associated_inputs()
                .map(AsRef::as_ref),
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
            FailureStorage::OrdinaryInitialization(o) => {
                o.source().expect("original request retained")
            }
            FailureStorage::Ordinary(o) => o.source(),
            FailureStorage::FragmentInitialization(o) => {
                o.source().expect("original request retained")
            }
            FailureStorage::Fragment(o) => o.source(),
            FailureStorage::OrdinaryFrozen(o) => o.source(),
            FailureStorage::FragmentFrozen(o) => o.source(),
        }
    }
    pub fn limits(&self) -> &HtmlParseLimits {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => {
                o.request().expect("original request").limits()
            }
            FailureStorage::Ordinary(o) => o.original_request().limits(),
            FailureStorage::FragmentInitialization(o) => {
                o.original_request().expect("original request").limits()
            }
            FailureStorage::Fragment(o) => o.original_request().limits(),
            FailureStorage::OrdinaryFrozen(o) => o.limits(),
            FailureStorage::FragmentFrozen(o) => o.limits(),
        }
    }
    /// The original captured-document request, absent for offline documents
    /// and fragments. A fragment capture is available through context().
    pub fn capture(&self) -> Option<&crate::HtmlCapture> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => {
                o.request().expect("original request").capture()
            }
            FailureStorage::Ordinary(o) => o.original_request().capture(),
            FailureStorage::OrdinaryFrozen(o) => o.request.capture(),
            _ => None,
        }
    }
    /// The actual qualified captured-document target, absent for offline
    /// documents and fragments; this is not an output-tree identity.
    pub fn document(&self) -> Option<&crate::HtmlCaptureDocumentId> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => {
                o.request().expect("original request").document()
            }
            FailureStorage::Ordinary(o) => o.original_request().document(),
            FailureStorage::OrdinaryFrozen(o) => o.request.document(),
            _ => None,
        }
    }
    /// The original ordinary request's URL facts, including on pre-source failure.
    pub fn document_base_facts(&self) -> Option<&crate::HtmlParseBaseFacts> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => {
                o.request().expect("original request").base_facts()
            }
            FailureStorage::Ordinary(o) => o.original_request().base_facts(),
            FailureStorage::OrdinaryFrozen(o) => o.request.base_facts(),
            _ => None,
        }
    }
    /// The original fragment request's qualified captured facts.
    pub fn captured_base_facts(&self) -> Option<&crate::HtmlCapturedBaseFacts> {
        match &self.storage {
            FailureStorage::FragmentInitialization(o) => {
                o.original_request().expect("original request").base_facts()
            }
            FailureStorage::Fragment(o) => o.original_request().base_facts(),
            FailureStorage::FragmentFrozen(o) => o.request.base_facts(),
            _ => None,
        }
    }
    /// The original request option, not a claim that execution occurred.
    pub fn requested_scripting_mode(&self) -> crate::HtmlParserScriptingMode {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => {
                o.request().expect("original request").scripting_mode()
            }
            FailureStorage::Ordinary(_) | FailureStorage::OrdinaryFrozen(_) => {
                crate::HtmlParserScriptingMode::Disabled
            }
            FailureStorage::FragmentInitialization(o) => o
                .original_request()
                .expect("original request")
                .requested_scripting_mode(),
            FailureStorage::Fragment(o) => o.original_request().requested_scripting_mode(),
            FailureStorage::FragmentFrozen(o) => o.request.requested_scripting_mode(),
        }
    }
    /// The original permission option, distinct from committed effects.
    pub fn allow_declarative_shadow_roots(&self) -> bool {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o
                .request()
                .expect("original request")
                .allow_declarative_shadow_roots(),
            FailureStorage::Ordinary(_) | FailureStorage::OrdinaryFrozen(_) => false,
            FailureStorage::FragmentInitialization(o) => o
                .original_request()
                .expect("original request")
                .allow_declarative_shadow_roots(),
            FailureStorage::Fragment(o) => o.original_request().allow_declarative_shadow_roots(),
            FailureStorage::FragmentFrozen(o) => o.request.allow_declarative_shadow_roots(),
        }
    }
    pub fn context(&self) -> Option<&crate::HtmlFragmentContext> {
        match &self.storage {
            FailureStorage::FragmentInitialization(o) => {
                Some(o.original_request().expect("original request").context())
            }
            FailureStorage::Fragment(o) => Some(o.context()),
            FailureStorage::FragmentFrozen(o) => Some(o.context()),
            _ => None,
        }
    }
    pub fn at_stop(&self) -> Option<&ParseUsage> {
        match &self.storage {
            FailureStorage::OrdinaryInitialization(o) => o.at_stop(),
            FailureStorage::Ordinary(o) => Some(o.at_stop()),
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
enum OrdinaryAttempt {
    Ready,
    Failed(HtmlParseFailureReason),
    Initialization(HtmlParseFailureReason, bool),
}
fn ordinary_attempt(
    owner: &mut OrdinaryInitializationOwner,
    ready: &mut Option<FrozenStandaloneParts>,
    failed_output: &mut Option<DetachedStandaloneFailure>,
    mut service: Option<&mut dyn crate::meta_csp::Service>,
    fixed: usize,
    initial_base: bool,
) -> OrdinaryAttempt {
    let mut attempt = match owner.begin_report() {
        Ok(a) => a,
        Err(e) => return OrdinaryAttempt::Initialization(ordinary_reason(&e), false),
    };
    let mut driver = match attempt.take_driver() {
        Ok(d) => Some(d),
        Err(e) => {
            let reason = ordinary_reason(&e);
            let cleanup_failed = attempt.retain_failure().is_err();
            return OrdinaryAttempt::Initialization(reason, cleanup_failed);
        }
    };
    // Actual output receivers are admitted on this public-only path before
    // its first token pull; no heap indirection is introduced.
    let mut reason = super::write(
        std::mem::size_of::<Option<FrozenStandaloneParts>>()
            + std::mem::size_of::<Option<DetachedStandaloneFailure>>()
            + std::mem::size_of::<OrdinaryAttempt>(),
        3,
        &mut driver.as_mut().expect("actual driver").control,
    )
    .err()
    .map(HtmlParseFailureReason::Stop);
    if service.is_some() && reason.is_none() {
        if let Err(e) = super::write(
            fixed,
            2,
            &mut driver.as_mut().expect("actual driver").control,
        ) {
            reason = Some(HtmlParseFailureReason::Stop(e));
        }
        if reason.is_none()
            && let Err(e) = driver
                .as_mut()
                .expect("actual driver")
                .enable_meta_csp(initial_base)
        {
            reason = Some(ordinary_reason(&e));
        }
    }
    while reason.is_none() {
        match driver
            .as_mut()
            .expect("actual driver")
            .step_with_meta_csp(&mut service)
        {
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
        driver
            .as_mut()
            .expect("actual driver")
            .control
            .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
        let mut failed = Some(
            StandaloneDocumentDriver::take_failed(&mut driver).expect("closed execution cleanup"),
        );
        *failed_output = Some(
            FailedStandaloneDocument::detach(&mut failed).expect("same request terminal transfer"),
        );
        return OrdinaryAttempt::Failed(reason);
    }
    let mut completed = match StandaloneDocumentDriver::take_completed(&mut driver) {
        Ok(completed) => completed,
        Err(error) => {
            let reason = ordinary_reason(&error);
            driver
                .as_mut()
                .expect("completion refusal retains driver")
                .control
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            let mut failed = Some(
                StandaloneDocumentDriver::take_failed(&mut driver)
                    .expect("closed completion-refusal cleanup"),
            );
            *failed_output = Some(
                FailedStandaloneDocument::detach(&mut failed)
                    .expect("same stopped request authority"),
            );
            return OrdinaryAttempt::Failed(reason);
        }
    };
    let completion = completed
        .prepare_native_associations()
        .and_then(|()| completed.freeze_parts());
    match completion {
        Ok(parts) => {
            *ready = Some(parts);
            OrdinaryAttempt::Ready
        }
        Err(e) => {
            let reason = ordinary_reason(&e);
            completed.stop_for_public_failure();
            let mut failed = Some(
                completed
                    .take_public_failure()
                    .expect("closed completed cleanup"),
            );
            *failed_output =
                Some(FailedStandaloneDocument::detach(&mut failed).expect("same stopped binding"));
            OrdinaryAttempt::Failed(reason)
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
    let mut owner = OrdinaryInitializationOwner::new(request);
    let mut parts = None;
    let mut failed = None;
    match ordinary_attempt(
        &mut owner,
        &mut parts,
        &mut failed,
        service,
        fixed,
        initial_base,
    ) {
        OrdinaryAttempt::Ready => {}
        OrdinaryAttempt::Initialization(reason, refused) => {
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::OrdinaryInitialization(owner),
                cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
            });
        }
        OrdinaryAttempt::Failed(reason) => {
            let failure = OwnedOrdinaryFailure::take(
                &mut Some(owner.take_request().expect("transferred real request")),
                &mut failed,
            )
            .expect("original authority");
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::Ordinary(failure),
                cleanup_error: None,
            });
        }
    };
    let mut request = Some(owner.take_request().expect("complete actual request"));
    let sealed = match SealedOrdinaryConstruction::take(&mut request, &mut parts) {
        Ok(sealed) => sealed,
        Err(e) => {
            let reason = ordinary_reason(&e);
            parts
                .as_mut()
                .expect("retained freeze")
                .control
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            let failure = super::terminal::FrozenStandaloneParts::detach_failure(&mut parts)
                .expect("closed frozen cleanup");
            let failure = OwnedOrdinaryFailure::take(&mut request, &mut Some(failure))
                .expect("actual original request");
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::Ordinary(failure),
                cleanup_error: None,
            });
        }
    };
    let mut input = Some(sealed);
    match ImmutableParseReport::take_document(&mut input) {
        Ok(report) => Ok(report),
        Err(e) => {
            let reason = session_reason(e);
            let mut sealed = input.take().expect("publication refusal retains owner");
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
#[expect(
    clippy::result_large_err,
    reason = "returns the actual original request, stopped control and cleanup owner without allocating after a stop"
)]
pub(crate) fn fragment(
    request: HtmlFragmentParseRequest,
) -> Result<ImmutableParseReport, HtmlParseFailure> {
    let mut owner = FragmentInitializationOwner::new(request);
    let mut parts;
    let mut stopped = None;
    let reason;
    {
        let mut attempt = owner.begin_report().expect("fresh request owner");
        let mut driver = match attempt.take_driver() {
            Ok(d) => Some(d),
            Err(_) => {
                let why = attempt.reason().map(fragment_reason).unwrap_or(
                    HtmlParseFailureReason::Engine(HtmlParseEngineCondition::Ownership),
                );
                let refused = attempt.retain_failure().is_err();
                drop(attempt);
                return Err(HtmlParseFailure {
                    reason: why,
                    storage: FailureStorage::FragmentInitialization(owner),
                    cleanup_error: refused.then_some(HtmlParseEngineCondition::Cleanup),
                });
            }
        };
        let mut why = None;
        loop {
            match driver.as_mut().expect("actual fragment driver").step() {
                Ok(HeadStep::Consumed) => {}
                Ok(HeadStep::NeedInput) => {
                    why = Some(HtmlParseFailureReason::Engine(
                        HtmlParseEngineCondition::Tokenizer,
                    ));
                    break;
                }
                Ok(HeadStep::Complete) => break,
                Ok(HeadStep::Boundary(b)) => {
                    why = Some(HtmlParseFailureReason::UnsupportedConstruction(b));
                    break;
                }
                Err(e) => {
                    why = Some(fragment_reason(&e));
                    break;
                }
            }
        }
        if let Some(why) = why {
            driver
                .as_mut()
                .expect("actual driver")
                .stop_for_public_failure();
            let mut failed = Some(
                ContextFragmentDriver::take_failed(&mut driver).expect("closed fragment cleanup"),
            );
            stopped = Some(
                StoppedFragmentConstruction::take(&mut failed, &mut None)
                    .expect("actual stopped authority"),
            );
            parts = None;
            reason = Some(why);
        } else {
            match ContextFragmentDriver::take_completed(&mut driver) {
                Err(error) => {
                    let why = fragment_reason(&error);
                    driver
                        .as_mut()
                        .expect("completion refusal retains fragment driver")
                        .stop_for_public_failure();
                    let mut failed = Some(
                        ContextFragmentDriver::take_failed(&mut driver)
                            .expect("closed fragment completion-refusal cleanup"),
                    );
                    stopped = Some(
                        StoppedFragmentConstruction::take(&mut failed, &mut None)
                            .expect("same stopped fragment authority"),
                    );
                    parts = None;
                    reason = Some(why);
                }
                Ok(mut completed) => match completed
                    .prepare_native_associations()
                    .and_then(|()| completed.freeze_parts())
                {
                    Ok(p) => {
                        parts = Some(p);
                        reason = None;
                    }
                    Err(e) => {
                        let why = fragment_reason(&e);
                        completed.stop_for_public_failure();
                        let mut failed = Some(
                            completed
                                .take_failed()
                                .expect("closed completed fragment cleanup"),
                        );
                        stopped = Some(
                            StoppedFragmentConstruction::take(&mut failed, &mut None)
                                .expect("stopped original source"),
                        );
                        parts = None;
                        reason = Some(why);
                    }
                },
            }
        }
    }
    if let Some(reason) = reason {
        owner
            .retain_execution_failure(&mut stopped)
            .expect("original authority");
        return Err(HtmlParseFailure {
            reason,
            storage: FailureStorage::Fragment(
                owner.take_failure().expect("original owned failure"),
            ),
            cleanup_error: None,
        });
    }
    let sealed = match owner.seal(&mut parts) {
        Ok(sealed) => sealed,
        Err(e) => {
            let reason = fragment_reason(&e);
            parts
                .as_mut()
                .expect("retained actual parts")
                .control
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
            stopped = Some(
                StoppedFragmentConstruction::take_frozen(&mut parts, &mut None)
                    .expect("closed frozen cleanup"),
            );
            owner
                .retain_execution_failure(&mut stopped)
                .expect("qualified original authority");
            return Err(HtmlParseFailure {
                reason,
                storage: FailureStorage::Fragment(owner.take_failure().expect("same request")),
                cleanup_error: None,
            });
        }
    };
    let mut input = Some(sealed);
    match ImmutableParseReport::take_fragment(&mut input) {
        Ok(report) => Ok(report),
        Err(e) => {
            let reason = session_reason(e);
            let mut sealed = input.take().expect("same refused report slot");
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
/// Exact added physical terminal envelope, admitted before first token pull.
pub(super) fn terminal_layout() -> (usize, usize) {
    use std::mem::size_of;
    (
        size_of::<HtmlParseFailure>()
            + size_of::<FailureStorage>()
            + size_of::<HtmlParseFailureReason>()
            + size_of::<Option<HtmlParseEngineCondition>>(),
        4,
    )
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
        failed.release_csp_storage(crate::meta_csp::Storage {
            bytes: 1,
            units: 1,
            entries: 1,
        });
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
