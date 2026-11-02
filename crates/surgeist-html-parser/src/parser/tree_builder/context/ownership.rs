//! Original-request ownership around actual context-fragment initialization and
//! native completion. This is not complete lexical D1 or public parse success.

use super::*;
use crate::parser::control::ParseUsage;
use crate::request::FragmentRequestParts;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InitializationState {
    Failed,
    Consumed,
}
pub(super) struct FragmentEntry<'source> {
    driver: Option<ContextFragmentDriver<'source>>,
    session: Option<ParserArenaSession>,
    tokenizer: Option<Tokenizer<'source>>,
    before: Option<BeforeHead>,
    reason: Option<FragmentExecutionError>,
    after_cleanup: Option<ParseUsage>,
}
impl<'source> FragmentEntry<'source> {
    pub(super) fn new(
        request: &'source HtmlFragmentParseRequest,
        target: super::super::parse::InitializationTarget,
    ) -> Self {
        // Bootstrap fixed ownership exists before the first source admission.
        let mut driver = ContextFragmentDriver {
            request,
            control: ParseControl::new(*request.limits(), request.cancellation().clone()),
            head: None,
            failed_evidence: None,
            evidence_taken: false,
        };
        let mut session = None;
        let mut before = None;
        let mut tokenizer = None;
        let reason = match Self::prepare(
            &mut driver,
            &mut session,
            &mut before,
            &mut tokenizer,
            target,
        ) {
            Ok(()) => None,
            Err(error) => {
                if error.invariant_fault() {
                    FragmentExecutionError::latch_invariant(&mut driver.control);
                } else {
                    driver
                        .control
                        .stop_unsupported_standalone(StandaloneUnsupported::Provenance);
                }
                Some(error)
            }
        };
        Self {
            driver: Some(driver),
            session,
            tokenizer,
            before,
            reason,
            after_cleanup: None,
        }
    }
    fn prepare(
        driver: &mut ContextFragmentDriver<'source>,
        session: &mut Option<ParserArenaSession>,
        before: &mut Option<BeforeHead>,
        tokenizer: &mut Option<Tokenizer<'source>>,
        target: super::super::parse::InitializationTarget,
    ) -> Result<(), FragmentExecutionError> {
        let request = driver.request;
        let c = &mut driver.control;
        c.admit_source(request.source())?;
        target.admit(c)?;
        // Actual owning slots, error receiver and terminal transfers. No
        // stopped extraction allocates variable storage or copies source text.
        write(
            const {
                size_of::<Self>() + initialization_transfer_layout().0 + 2 * size_of::<ParseUsage>()
            },
            const { 6 + initialization_transfer_layout().1 + 2 },
            c,
        )?;
        c.charge(
            WorkCategory::InspectByte,
            terminal_qualification_bytes() as u64,
        )?;
        let (lexical, mode, template) = initial_modes(request, c)?;
        let document = session.insert(ParserArenaSession::new(
            c,
            RootKind::Document(request.context().document_mode()),
        )?);
        document.initialize_standalone_document(c)?;
        document.initialize_parse_evidence(c, request.source())?;
        let bootstrap = document.initialize_fragment(c, request)?;
        if let Some(inputs) = request.shared_associated_inputs() {
            document.install_associated_inputs(c, inputs)?;
            if let Some(facts) = &inputs.parser_document {
                document.install_document_base_facts(c, facts)?;
            }
        }

        let mut whitespace = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        whitespace.require_owner(c)?;
        let kernel = match LocationKernel::new(session, c) {
            Ok(kernel) => kernel,
            Err(error) => {
                whitespace.release(c)?;
                return Err(error.into());
            }
        };
        let retained = before.insert(BeforeHead {
            kernel,
            input: None,
            whitespace,
        });
        let kernel = &mut retained.kernel;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        kernel.root_target = Some(bootstrap.output);
        kernel.push(&bootstrap.root, c)?;
        *tokenizer = Some(Tokenizer::new(request.source(), State::Data, None, c)?);
        let stage = driver
            .head
            .insert(HeadStage::new(request.source(), before, tokenizer, c)?);
        let document = stage
            .state
            .as_ref()
            .ok_or(head::HeadError::Consumed)?
            .before
            .kernel
            .session
            .root(c)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let images = request.image_support();
        write(size_of::<ConstructionCapability>(), 5, c)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        stage.install_standalone(
            ConstructionCapability {
                source: request.source(),
                owner: c.identity(),
                document,
                profile: ExecutionProfile::Fragment,
                images,
            },
            c,
        )?;
        let state = stage.state.as_mut().ok_or(head::HeadError::Consumed)?;
        state.before.kernel.session.retain_fragment_bootstrap(
            c,
            &state
                .script
                .as_ref()
                .ok_or(head::HeadError::ConstructionCapabilityUnavailable)?
                .capability,
        )?;
        if let Some(form) = request.context().nearest_form() {
            c.charge(WorkCategory::ProviderCall, 1)?;
            write(size_of::<Option<head::FormPointer>>(), 1, c)?;
            state.current_form = Some(head::FormPointer::Captured(form.clone()));
        }
        if template {
            state
                .template_modes
                .try_push(c, InsertionMode::InTemplate)?;
        }
        state.set_mode(mode, c)?;
        state
            .tokenizer
            .as_mut()
            .ok_or(head::HeadError::MissingTokenizer)?
            .set_state(lexical, c)?;
        Ok(())
    }
    pub(super) fn reason(&self) -> Option<&FragmentExecutionError> {
        self.reason.as_ref()
    }
    pub(super) fn take_driver(
        &mut self,
    ) -> Result<ContextFragmentDriver<'source>, InitializationState> {
        if self.reason.is_some() {
            return Err(InitializationState::Failed);
        }
        let mut driver = self.driver.take().ok_or(InitializationState::Consumed)?;
        // The two `ParseUsage` terms are the two snapshot copies a later
        // `into_failed` makes (the stop-time clone and the cleanup clone); the
        // `FailedContextFragment` term is that struct's by-value construction
        // holding them. They are distinct copies, so no usage is charged twice.
        let transfer = driver
            .control
            .require_source(driver.request.source())
            .and_then(|()| {
                write(
                    const {
                        size_of::<ContextFragmentDriver>()
                            + size_of::<Option<ContextFragmentDriver>>()
                            + execution_transfer_layout().0
                            + size_of::<FailedContextFragment>()
                            + 2 * size_of::<ParseUsage>()
                    },
                    const { 2 + execution_transfer_layout().1 + 8 },
                    &mut driver.control,
                )
            });
        let transfer = transfer.and_then(|()| {
            driver.control.charge(
                WorkCategory::InspectByte,
                const { 2 * terminal_qualification_bytes() as u64 },
            )
        });
        if let Err(stop) = transfer {
            self.driver = Some(driver);
            self.reason = Some(stop.into());
            return Err(InitializationState::Failed);
        }
        Ok(driver)
    }
    fn cleanup_failure(&mut self) -> Result<(), FragmentExecutionError> {
        if self.after_cleanup.is_some() {
            return Ok(());
        }
        let driver = self
            .driver
            .as_mut()
            .ok_or(FragmentExecutionError::Consumed)?;
        if driver.control.stop().is_none() {
            return Err(FragmentExecutionError::Consumed);
        }
        driver.evidence_taken = true;
        // A refused partition preserves the actual evidence owner. Snapshot
        // presence is not evidence that that separate move already completed.
        if driver.failed_evidence.is_none() {
            let session = if let Some(stage) = &mut driver.head {
                stage.state.as_mut().map(|s| &mut s.before.kernel.session)
            } else if let Some(before) = &mut self.before {
                Some(&mut before.kernel.session)
            } else {
                self.session.as_mut()
            };
            if let Some(session) = session {
                driver.failed_evidence = session.take_failed_evidence(&mut driver.control)?;
            }
        }
        if let Some(stage) = &mut driver.head {
            stage.release(&mut driver.control)?;
            driver.head = None;
        }
        if let Some(before) = &mut self.before {
            before
                .whitespace
                .require_disposal_owner(&mut driver.control)?;
            before.kernel.release(&mut driver.control)?;
        }
        if let Some(before) = self.before.take() {
            before.whitespace.release(&mut driver.control)?;
        }
        if let Some(session) = &mut self.session {
            session.release(&mut driver.control)?;
            self.session = None;
        }
        if let Some(tokenizer) = self.tokenizer.take() {
            tokenizer.release(&mut driver.control)?;
        }
        self.after_cleanup = Some(driver.control.usage().clone());
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn release_failure(&mut self) -> Result<(), FragmentExecutionError> {
        self.cleanup_failure()?;
        let driver = self.driver.as_mut().expect("retained control");
        if let Some(evidence) = driver.failed_evidence.take() {
            evidence.release(&mut driver.control)?;
        }
        Ok(())
    }
    fn detach(&mut self) -> Result<StoppedFragmentConstruction, FragmentExecutionError> {
        self.cleanup_failure()?;
        let mut driver = self.driver.take().ok_or(FragmentExecutionError::Consumed)?;
        match self.cleaned_observations(&mut driver) {
            Ok(cleaned) => Ok(StoppedFragmentConstruction {
                stop: cleaned.stop,
                at_stop: cleaned.at_stop,
                after_cleanup: cleaned.after_cleanup,
                control: driver.control,
                binding: cleaned.binding,
                evidence: driver.failed_evidence,
                _reason: self.reason.take(),
            }),
            Err(error) => {
                self.driver = Some(driver);
                Err(error)
            }
        }
    }
    /// Moves the stop, both usage observations and the source binding out of
    /// the cleaned entry; a refusal puts back what it had already moved.
    fn cleaned_observations(
        &mut self,
        driver: &mut ContextFragmentDriver<'source>,
    ) -> Result<CleanedObservations, FragmentExecutionError> {
        // Cleanup refuses an owner that has not stopped, and the stop and its
        // snapshot exist together.
        let (stop, at_stop) = driver
            .control
            .stopped()
            .map(|(stop, usage)| (stop, usage.clone()))
            .ok_or(FragmentExecutionError::Consumed)?;
        let Some(after_cleanup) = self.after_cleanup.take() else {
            return Err(FragmentExecutionError::latch_invariant(&mut driver.control));
        };
        match driver.control.take_stopped_optional_source_binding() {
            Ok(binding) => Ok(CleanedObservations {
                stop,
                at_stop,
                after_cleanup,
                binding,
            }),
            Err(error) => {
                self.after_cleanup = Some(after_cleanup);
                Err(error.into())
            }
        }
    }
}
/// What a cleaned stopped construction hands over when it is detached.
struct CleanedObservations {
    stop: ParseStop,
    at_stop: ParseUsage,
    after_cleanup: ParseUsage,
    binding: Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>,
}

/// The original source/capture/options and one preexisting stopped-carrier
/// slot. The carrier owns the request by construction: only a consuming
/// transfer moves it out, so no state of the carrier lacks it.
pub(crate) struct FragmentInitialization {
    request: HtmlFragmentParseRequest,
    stopped: Option<StoppedFragmentConstruction>,
    begun: bool,
}
pub(crate) struct FragmentInitializationAttempt<'owner> {
    entry: FragmentEntry<'owner>,
    stopped: &'owner mut Option<StoppedFragmentConstruction>,
}
impl FragmentInitialization {
    pub(in crate::parser::tree_builder) fn original_request(&self) -> &HtmlFragmentParseRequest {
        &self.request
    }
    pub(crate) fn new(request: HtmlFragmentParseRequest) -> Self {
        Self {
            request: request.share_associated_inputs(),
            stopped: None,
            begun: false,
        }
    }
    #[cfg(test)]
    pub(crate) fn begin(
        &mut self,
    ) -> Result<FragmentInitializationAttempt<'_>, InitializationState> {
        self.begin_for(super::super::parse::InitializationTarget::Native)
    }
    pub(in crate::parser::tree_builder) fn begin_report(
        &mut self,
    ) -> Result<FragmentInitializationAttempt<'_>, InitializationState> {
        self.begin_for(super::super::parse::InitializationTarget::Report)
    }
    fn begin_for(
        &mut self,
        target: super::super::parse::InitializationTarget,
    ) -> Result<FragmentInitializationAttempt<'_>, InitializationState> {
        if self.begun {
            return Err(InitializationState::Consumed);
        }
        self.begun = true;
        Ok(FragmentInitializationAttempt {
            entry: FragmentEntry::new(&self.request, target),
            stopped: &mut self.stopped,
        })
    }
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    pub(crate) fn failure(&self) -> Option<&StoppedFragmentConstruction> {
        self.stopped.as_ref()
    }
    /// The same request-owned terminal slot is filled, never reconstructed. A
    /// refusal hands back the carrier and the stopped construction.
    #[expect(
        clippy::result_large_err,
        reason = "hands back the very owner that a refused step could not move, without allocating after a stop"
    )]
    pub(crate) fn retain_stopped(
        mut self,
        stopped: StoppedFragmentConstruction,
    ) -> Result<Self, (FragmentExecutionError, Self, StoppedFragmentConstruction)> {
        if self.stopped.is_some() || !stopped.qualifies(&self.request) {
            return Err((FragmentExecutionError::Consumed, self, stopped));
        }
        self.stopped = Some(stopped);
        Ok(self)
    }
    /// Joins the retained stopped construction to the original request. A
    /// refusal hands the carrier back with both still in place.
    #[expect(
        clippy::result_large_err,
        reason = "hands back the very owner that a refused step could not move, without allocating after a stop"
    )]
    pub(crate) fn into_failure(
        mut self,
    ) -> Result<OwnedFragmentFailure, (FragmentExecutionError, Self)> {
        let Some(stopped) = self.stopped.take() else {
            return Err((FragmentExecutionError::Consumed, self));
        };
        let Self {
            request,
            stopped: _,
            begun,
        } = self;
        match OwnedFragmentFailure::join(request, stopped) {
            Ok(joined) => Ok(joined),
            Err((error, request, stopped)) => Err((
                error,
                Self {
                    request,
                    stopped: Some(stopped),
                    begun,
                },
            )),
        }
    }
    /// Seals the frozen construction with the original request. A refusal hands
    /// back the carrier and the parts.
    #[expect(
        clippy::result_large_err,
        reason = "hands back the very owner that a refused step could not move, without allocating after a stop"
    )]
    pub(crate) fn into_sealed(
        self,
        parts: FrozenContextFragmentParts,
    ) -> Result<
        SealedFragmentConstruction,
        (FragmentExecutionError, Self, FrozenContextFragmentParts),
    > {
        if self.stopped.is_some() {
            return Err((FragmentExecutionError::Consumed, self, parts));
        }
        let Self {
            request,
            stopped,
            begun,
        } = self;
        match SealedFragmentConstruction::join(request, parts) {
            Ok(sealed) => Ok(sealed),
            Err((error, request, parts)) => Err((
                error,
                Self {
                    request,
                    stopped,
                    begun,
                },
                parts,
            )),
        }
    }
}
/// The slot-style view of a carrier that tests drive: the request leaves it
/// once, after which the carrier is gone.
#[cfg(test)]
pub(crate) struct FragmentInitializationOwner {
    held: Option<FragmentInitialization>,
}
#[cfg(test)]
impl FragmentInitializationOwner {
    pub(crate) fn new(request: HtmlFragmentParseRequest) -> Self {
        Self {
            held: Some(FragmentInitialization::new(request)),
        }
    }
    pub(crate) fn begin(
        &mut self,
    ) -> Result<FragmentInitializationAttempt<'_>, InitializationState> {
        self.held
            .as_mut()
            .ok_or(InitializationState::Consumed)?
            .begin()
    }
    pub(crate) fn source(&self) -> Option<&crate::HtmlSource> {
        self.held.as_ref().map(|held| held.source())
    }
    pub(crate) fn failure(&self) -> Option<&StoppedFragmentConstruction> {
        self.held.as_ref()?.failure()
    }
    pub(crate) fn take_failure(&mut self) -> Result<OwnedFragmentFailure, FragmentExecutionError> {
        let held = self.held.take().ok_or(FragmentExecutionError::Consumed)?;
        match held.into_failure() {
            Ok(failure) => Ok(failure),
            Err((error, held)) => {
                self.held = Some(held);
                Err(error)
            }
        }
    }
    pub(crate) fn retain_execution_failure(
        &mut self,
        stopped: &mut Option<StoppedFragmentConstruction>,
    ) -> Result<(), FragmentExecutionError> {
        let held = self.held.take().ok_or(FragmentExecutionError::Consumed)?;
        let Some(failed) = stopped.take() else {
            self.held = Some(held);
            return Err(FragmentExecutionError::Consumed);
        };
        match held.retain_stopped(failed) {
            Ok(held) => {
                self.held = Some(held);
                Ok(())
            }
            Err((error, held, failed)) => {
                self.held = Some(held);
                *stopped = Some(failed);
                Err(error)
            }
        }
    }
    pub(crate) fn seal(
        &mut self,
        parts: &mut Option<FrozenContextFragmentParts>,
    ) -> Result<SealedFragmentConstruction, FragmentExecutionError> {
        let held = self.held.take().ok_or(FragmentExecutionError::Consumed)?;
        let Some(frozen) = parts.take() else {
            self.held = Some(held);
            return Err(FragmentExecutionError::Consumed);
        };
        match held.into_sealed(frozen) {
            Ok(sealed) => Ok(sealed),
            Err((error, held, frozen)) => {
                self.held = Some(held);
                *parts = Some(frozen);
                Err(error)
            }
        }
    }
}
impl<'owner> FragmentInitializationAttempt<'owner> {
    pub(crate) fn take_driver(
        &mut self,
    ) -> Result<ContextFragmentDriver<'owner>, InitializationState> {
        self.entry.take_driver()
    }
    pub(crate) fn reason(&self) -> Option<&FragmentExecutionError> {
        self.entry.reason()
    }
    #[cfg(test)]
    pub(crate) fn usage(&self) -> Option<&ParseUsage> {
        self.entry.driver.as_ref().map(|d| d.control.usage())
    }
    pub(crate) fn retain_failure(&mut self) -> Result<(), FragmentExecutionError> {
        if self.stopped.is_some() {
            return Err(FragmentExecutionError::Consumed);
        }
        *self.stopped = Some(self.entry.detach()?);
        Ok(())
    }
}

/// Original request fields (7), one owning Option clear and joined fields
/// (2). This compiled fixed envelope precedes every terminal request join.
const fn request_join_layout() -> (usize, usize) {
    const {
        (
            size_of::<Option<StoppedFragmentConstruction>>()
                + size_of::<FragmentRequestParts>()
                + size_of::<OwnedFragmentFailure>(),
            1 + 7 + 2,
        )
    }
}
/// Initialization clears actual binding/driver/reason/cleanup snapshot slots
/// (4), writes seven detached fields, then performs the request join.
const fn initialization_transfer_layout() -> (usize, usize) {
    const {
        let (join_bytes, join_writes) = request_join_layout();
        (
            size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
                + size_of::<Option<ContextFragmentDriver>>()
                + size_of::<Option<FragmentExecutionError>>()
                + size_of::<Option<ParseUsage>>()
                + size_of::<StoppedFragmentConstruction>()
                + join_bytes,
            4 + 7 + join_writes,
        )
    }
}
/// Ready execution also retains the original six-field failure receiver and
/// two actual usage snapshots before the first token pull.
/// Active failure clears its actual binding slot (1), writes seven carrier
/// fields, then joins the original request. The cleaned failure moves into the
/// carrier by value, so no slot is cleared for it or for an error.
const fn execution_transfer_layout() -> (usize, usize) {
    const {
        let (join_bytes, join_writes) = request_join_layout();
        (
            size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
                + size_of::<StoppedFragmentConstruction>()
                + 2 * size_of::<Option<StoppedFragmentConstruction>>()
                + join_bytes,
            1 + 7 + 2 + join_writes,
        )
    }
}
/// A frozen failure is cleaned and detached into the carrier by value, as an
/// active failed execution is. This envelope is admitted before freezing that
/// owner and clears only the controller's source binding slot.
pub(super) const fn frozen_failure_layout() -> (usize, usize) {
    const {
        let (join_bytes, join_writes) = request_join_layout();
        (
            size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
                + size_of::<StoppedFragmentConstruction>()
                + 2 * size_of::<Option<StoppedFragmentConstruction>>()
                + join_bytes,
            1 + 7 + 2 + join_writes,
        )
    }
}
const fn terminal_qualification_bytes() -> usize {
    const {
        2 * size_of::<crate::HtmlParseCancellation>()
            + 2 * size_of::<crate::HtmlSourceId>()
            + 2 * size_of::<crate::HtmlSourceCoordinates>()
    }
}
pub(crate) struct StoppedFragmentConstruction {
    control: ParseControl,
    binding: Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>,
    stop: ParseStop,
    at_stop: ParseUsage,
    after_cleanup: ParseUsage,
    evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
    _reason: Option<FragmentExecutionError>,
}
impl StoppedFragmentConstruction {
    fn qualifies(&self, request: &HtmlFragmentParseRequest) -> bool {
        self.control.owns_request(request.cancellation())
            && self.binding.as_ref().is_none_or(|(id, basis)| {
                id == request.source().identity() && *basis == request.source().coordinates()
            })
    }
    /// The slot form that tests drive; a refusal leaves both slots as they were.
    #[cfg(test)]
    pub(crate) fn take(
        input: &mut Option<FailedContextFragment<'_>>,
        reason: &mut Option<FragmentExecutionError>,
    ) -> Result<Self, FragmentExecutionError> {
        let failed = input.take().ok_or(FragmentExecutionError::Consumed)?;
        match Self::from_failed(failed) {
            Ok(mut stopped) => {
                stopped._reason = reason.take();
                Ok(stopped)
            }
            Err((error, failed)) => {
                *input = Some(failed);
                Err(error)
            }
        }
    }
    /// Detaches the cleaned failure of an execution from its borrow of the
    /// request. A refusal hands the failure back with its binding intact.
    #[expect(
        clippy::result_large_err,
        reason = "hands back the very owner that a refused step could not move, without allocating after a stop"
    )]
    pub(crate) fn from_failed<'a>(
        mut failed: FailedContextFragment<'a>,
    ) -> Result<Self, (FragmentExecutionError, FailedContextFragment<'a>)> {
        match Self::qualified_binding(&mut failed) {
            Ok(binding) => Ok(Self {
                control: failed.control,
                binding,
                stop: failed.stop,
                at_stop: failed.at_stop,
                after_cleanup: failed.after_cleanup,
                evidence: failed.evidence,
                _reason: None,
            }),
            Err(error) => Err((error, failed)),
        }
    }
    fn qualified_binding(
        failed: &mut FailedContextFragment<'_>,
    ) -> Result<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>, FragmentExecutionError>
    {
        let actual = failed.control.stopped_source_binding()?;
        if !failed.control.owns_request(failed.request.cancellation())
            || actual.is_some_and(|(id, basis)| {
                id != failed.request.source().identity()
                    || *basis != failed.request.source().coordinates()
            })
        {
            return Err(FragmentExecutionError::Consumed);
        }
        Ok(failed.control.take_stopped_optional_source_binding()?)
    }

    /// The slot form that tests drive; a refusal leaves both slots as they were.
    #[cfg(test)]
    pub(crate) fn take_frozen(
        input: &mut Option<FrozenContextFragmentParts>,
        reason: &mut Option<FragmentExecutionError>,
    ) -> Result<Self, FragmentExecutionError> {
        let parts = input.take().ok_or(FragmentExecutionError::Consumed)?;
        match Self::from_frozen(parts) {
            Ok(mut stopped) => {
                stopped._reason = reason.take();
                Ok(stopped)
            }
            Err((error, parts)) => {
                *input = Some(parts);
                Err(error)
            }
        }
    }
    /// Cleans a stopped frozen construction and detaches its failure. A
    /// refusal hands the construction back with what it had already moved.
    #[expect(
        clippy::result_large_err,
        reason = "hands back the very owner that a refused step could not move, without allocating after a stop"
    )]
    pub(crate) fn from_frozen(
        mut parts: FrozenContextFragmentParts,
    ) -> Result<Self, (FragmentExecutionError, FrozenContextFragmentParts)> {
        match Self::cleaned_frozen(&mut parts) {
            Ok(cleaned) => Ok(Self {
                control: parts.control,
                binding: cleaned.binding,
                stop: cleaned.stop,
                at_stop: cleaned.at_stop,
                after_cleanup: cleaned.after_cleanup,
                evidence: parts.failed_evidence,
                _reason: None,
            }),
            Err(error) => Err((error, parts)),
        }
    }
    /// Cleans the stopped immutable owner, then moves its observations and
    /// source binding out; a refusal puts back what it had already moved.
    fn cleaned_frozen(
        parts: &mut FrozenContextFragmentParts,
    ) -> Result<CleanedObservations, FragmentExecutionError> {
        parts.cleanup_failure()?;
        let actual = parts.control.stopped_source_binding()?;
        if actual.is_some_and(|(id, basis)| id != &parts.source.0 || *basis != parts.source.1) {
            return Err(FragmentExecutionError::Consumed);
        }
        let Some((stop, at_stop)) = parts
            .control
            .stopped()
            .map(|(stop, usage)| (stop, usage.clone()))
        else {
            return Err(FragmentExecutionError::latch_invariant(&mut parts.control));
        };
        let Some(after_cleanup) = parts.after_cleanup.take() else {
            return Err(FragmentExecutionError::latch_invariant(&mut parts.control));
        };
        match parts.control.take_stopped_optional_source_binding() {
            Ok(binding) => Ok(CleanedObservations {
                stop,
                at_stop,
                after_cleanup,
                binding,
            }),
            Err(error) => {
                parts.after_cleanup = Some(after_cleanup);
                Err(error.into())
            }
        }
    }
    pub(crate) fn stop(&self) -> ParseStop {
        self.stop
    }
    #[cfg(test)]
    pub(crate) fn reason(&self) -> Option<&FragmentExecutionError> {
        self._reason.as_ref()
    }
    pub(crate) fn source_binding(
        &self,
    ) -> Option<(&crate::HtmlSourceId, crate::HtmlSourceCoordinates)> {
        self.binding.as_ref().map(|(id, basis)| (id, *basis))
    }
    pub(crate) fn at_stop(&self) -> &ParseUsage {
        &self.at_stop
    }
    pub(crate) fn after_cleanup(&self) -> &ParseUsage {
        &self.after_cleanup
    }
    pub(crate) fn retained_usage(&self) -> &ParseUsage {
        self.control.usage()
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.evidence.as_ref().map(|e| e.diagnostics())
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), FragmentExecutionError> {
        if let Some(evidence) = self.evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
}
pub(crate) struct OwnedFragmentFailure {
    request: FragmentRequestParts,
    failure: StoppedFragmentConstruction,
}
impl OwnedFragmentFailure {
    pub(in crate::parser::tree_builder) fn original_request(&self) -> &FragmentRequestParts {
        &self.request
    }
    /// A refusal hands both owners back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "hands back the very owner that a refused step could not move, without allocating after a stop"
    )]
    fn join(
        request: HtmlFragmentParseRequest,
        failure: StoppedFragmentConstruction,
    ) -> Result<
        Self,
        (
            FragmentExecutionError,
            HtmlFragmentParseRequest,
            StoppedFragmentConstruction,
        ),
    > {
        if !failure.control.owns_request(request.cancellation())
            || failure.binding.as_ref().is_some_and(|(id, basis)| {
                id != request.source().identity() || *basis != request.source().coordinates()
            })
        {
            return Err((FragmentExecutionError::Consumed, request, failure));
        }
        Ok(Self {
            request: request.into_parts(),
            failure,
        })
    }
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    pub(crate) fn context(&self) -> &crate::HtmlFragmentContext {
        self.request.context()
    }
    pub(crate) fn stop(&self) -> ParseStop {
        self.failure.stop()
    }
    #[cfg(test)]
    pub(crate) fn reason(&self) -> Option<&FragmentExecutionError> {
        self.failure.reason()
    }
    pub(crate) fn at_stop(&self) -> &ParseUsage {
        self.failure.at_stop()
    }
    pub(crate) fn after_cleanup(&self) -> &ParseUsage {
        self.failure.after_cleanup()
    }
    pub(crate) fn retained_usage(&self) -> &ParseUsage {
        self.failure.retained_usage()
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.failure.diagnostics()
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), FragmentExecutionError> {
        self.failure.release()
    }
}

/// Immutable native/source ownership only; no complete lexical/D1 certificate.
pub(crate) struct SealedFragmentConstruction {
    pub(in crate::parser::tree_builder) request: FragmentRequestParts,
    pub(in crate::parser::tree_builder) parts: FrozenContextFragmentParts,
}
impl SealedFragmentConstruction {
    /// A refusal hands both owners back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "hands back the very owner that a refused step could not move, without allocating after a stop"
    )]
    fn join(
        request: HtmlFragmentParseRequest,
        mut parts: FrozenContextFragmentParts,
    ) -> Result<
        Self,
        (
            FragmentExecutionError,
            HtmlFragmentParseRequest,
            FrozenContextFragmentParts,
        ),
    > {
        if let Err(error) = parts.prepare_seal(&request) {
            return Err((error, request, parts));
        }
        Ok(Self {
            request: request.into_parts(),
            parts,
        })
    }
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.request.source()
    }
    pub(crate) fn limits(&self) -> &crate::HtmlParseLimits {
        self.request.limits()
    }
    pub(crate) fn context(&self) -> &crate::HtmlFragmentContext {
        self.request.context()
    }
    #[cfg(test)]
    pub(crate) fn requested_scripting_mode(&self) -> HtmlParserScriptingMode {
        self.request.requested_scripting_mode()
    }
    #[cfg(test)]
    pub(crate) fn effective_scripting_mode(&self) -> HtmlParserScriptingMode {
        self.request.effective_scripting_mode()
    }
    #[cfg(test)]
    pub(crate) fn allow_declarative_shadow_roots(&self) -> bool {
        self.request.allow_declarative_shadow_roots()
    }
    #[cfg(test)]
    pub(crate) fn fragment(&self) -> Option<&crate::HtmlFragment> {
        self.parts.arena.as_ref()?.fragment()
    }
    #[cfg(test)]
    pub(crate) fn evidence(
        &self,
    ) -> Result<&crate::construction::evidence::ParseEvidence, FragmentExecutionError> {
        Ok(self
            .parts
            .arena
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .retained_evidence()?)
    }
    #[cfg(test)]
    pub(crate) fn effects(
        &self,
    ) -> Result<crate::construction::context::FrozenFragmentEffects<'_>, FragmentExecutionError>
    {
        Ok(self
            .parts
            .arena
            .as_ref()
            .ok_or(FragmentExecutionError::Consumed)?
            .frozen_fragment_effects()?)
    }
    #[cfg(test)]
    pub(crate) fn final_usage(&self) -> &ParseUsage {
        &self.parts.final_usage
    }
    pub(crate) fn retained_usage(&self) -> &ParseUsage {
        self.parts.control.usage()
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), FragmentExecutionError> {
        if let Some(arena) = &mut self.parts.arena {
            arena.release(&mut self.parts.control)?;
            self.parts.arena = None;
            self.parts.after_cleanup = Some(self.parts.control.usage().clone());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HtmlParseLimits, HtmlParseResource, HtmlScriptingStatus, HtmlSource};

    fn request(source: &str) -> HtmlFragmentParseRequest {
        super::super::tests::request(source, "div", HtmlScriptingStatus::Disabled)
    }
    fn limits(source: u64, work: u64) -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(source)
            .buffered_utf16_units(100_000)
            .token_attributes(10_000)
            .auxiliary_entries(1_000_000)
            .managed_nodes(100_000)
            .managed_edges(100_000)
            .managed_attributes(100_000)
            .managed_utf16_units(100_000)
            .source_map_entries(100_000)
            .source_map_payload_units(100_000)
            .diagnostics(100_000)
            .diagnostic_payload_units(100_000)
            .context_effects(100_000)
            .open_elements(100)
            .tree_depth(100_000)
            .managed_capacity_bytes(100_000_000)
            .peak_transient_bytes(100_000_000)
            .atomic_layout_bytes(100_000_000)
            .work_credits(work)
            .finish()
            .unwrap()
    }
    fn with_limits(source: &str, limits: HtmlParseLimits) -> HtmlFragmentParseRequest {
        let original = request(source);
        HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(source),
            original.context().clone(),
            limits,
            original.requested_scripting_mode(),
            original.allow_declarative_shadow_roots(),
        )
        .unwrap()
    }
    fn freeze(attempt: &mut FragmentInitializationAttempt<'_>) -> FrozenContextFragmentParts {
        let mut driver = Some(attempt.take_driver().unwrap());
        super::super::tests::complete(driver.as_mut().unwrap());
        let mut completed = ContextFragmentDriver::take_completed(&mut driver).unwrap();
        completed.prepare_native_associations().unwrap();
        completed.freeze_parts().unwrap()
    }
    fn assert_reclaimed(usage: &ParseUsage) {
        for resource in [
            HtmlParseResource::BufferedUtf16Units,
            HtmlParseResource::TokenAttributes,
            HtmlParseResource::AuxiliaryEntries,
            HtmlParseResource::ManagedEdges,
            HtmlParseResource::ManagedAttributes,
            HtmlParseResource::ManagedUtf16Units,
            HtmlParseResource::SourceMapEntries,
            HtmlParseResource::SourceMapPayloadUnits,
            HtmlParseResource::Diagnostics,
            HtmlParseResource::DiagnosticPayloadUnits,
            HtmlParseResource::ContextEffects,
            HtmlParseResource::OpenElements,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::PeakTransientBytes,
        ] {
            assert_eq!(usage.resource(resource).current, 0, "{resource:?}");
        }
    }
    #[test]
    fn native_source_seal_keeps_initialized_evidence_without_certifying_native_projection() {
        use crate::construction::context::FrozenFragmentEffect;
        let literal = "<!doctype html><template>A</template><p>Z</p>";
        let original = super::super::tests::template_request(literal, true);
        let source = original.source().identity().clone();
        let capture = original.context().capture().identity().clone();
        let cancel = original.cancellation().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            let mut driver = Some(attempt.take_driver().unwrap());
            super::super::tests::complete(driver.as_mut().unwrap());
            let mut completed = ContextFragmentDriver::take_completed(&mut driver).unwrap();
            {
                let actual = completed.driver.as_mut().unwrap();
                let state = actual.head.as_mut().unwrap().state.as_mut().unwrap();
                assert!(matches!(
                    state
                        .before
                        .kernel
                        .session
                        .native_associations(&mut actual.control),
                    Err(SessionError::WrongToken)
                ));
                assert!(actual.control.stop().is_none());
            }
            // No prepare_native_associations: the completed native/source seal
            // qualifies retained initialized evidence, not a prepared projection.
            Some(completed.freeze_parts().unwrap())
        };
        let mut sealed = owner.seal(&mut parts).unwrap();
        assert!(parts.is_none());
        assert!(owner.source().is_none());
        let before = sealed.retained_usage().clone();
        cancel.cancel();
        assert_eq!(sealed.source().identity(), &source);
        assert_eq!(sealed.source().as_utf8(), Some(literal));
        assert_eq!(sealed.context().capture().identity(), &capture);
        assert!(!sealed.evidence().unwrap().diagnostics().is_empty());
        {
            let fragment = sealed.fragment().unwrap();
            let children: Vec<_> = fragment.root().children().collect();
            assert_eq!(children.len(), 2);
            assert_ne!(children[0], children[1]);
            let template = fragment.node(&children[0]).unwrap();
            let crate::HtmlNodeKind::Element(element) = template.kind() else {
                panic!("actual template")
            };
            assert_eq!(
                element.name().local_name(),
                &crate::DocumentString::from("template")
            );
            assert_eq!(template.children().count(), 0);
            let contents_id = template.template_contents().unwrap();
            let contents = fragment.node(&contents_id).unwrap();
            let text_id = contents.children().next().unwrap();
            assert!(matches!(
                fragment.node(&text_id).unwrap().kind(),
                crate::HtmlNodeKind::Text(text) if text == &crate::DocumentString::from("A")
            ));
            let paragraph = fragment.node(&children[1]).unwrap();
            let text_id = paragraph.children().next().unwrap();
            assert!(matches!(
                fragment.node(&text_id).unwrap().kind(),
                crate::HtmlNodeKind::Text(text) if text == &crate::DocumentString::from("Z")
            ));
            let effects = sealed.effects().unwrap();
            assert_eq!(effects.len(), 1);
            let Some(FrozenFragmentEffect::TemplateCache { document, inert }) =
                effects.get(0).unwrap()
            else {
                panic!("actual inert cache effect")
            };
            assert_eq!(&document.id(), sealed.context().target_document().subject());
            assert!(matches!(inert.kind(), crate::HtmlNodeKind::Document(_)));
        }
        assert_eq!(sealed.retained_usage(), &before);
        sealed.release().unwrap();
        assert_reclaimed(sealed.retained_usage());
    }
    #[test]
    fn original_fragment_source_capture_and_options_move_into_cancel_independent_native_seal() {
        let original = request("<p>A&amp;B</p><p>Z</p>");
        let source_id = original.source().identity().clone();
        let capture_id = original.context().capture().identity().clone();
        let cancel = original.cancellation().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        let mut sealed = owner.seal(&mut parts).unwrap();
        assert!(parts.is_none());
        assert!(owner.source().is_none());
        assert_eq!(sealed.limits(), &super::super::super::tests::limits(100));
        assert!(
            sealed
                .parts
                .control
                .owns_request(sealed.request.cancellation())
        );
        assert_eq!(sealed.source().identity(), &source_id);
        assert_eq!(sealed.context().capture().identity(), &capture_id);
        assert_eq!(
            sealed.requested_scripting_mode(),
            HtmlParserScriptingMode::Inert
        );
        assert_eq!(
            sealed.effective_scripting_mode(),
            HtmlParserScriptingMode::Disabled
        );
        assert!(!sealed.allow_declarative_shadow_roots());
        let before = sealed
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        cancel.cancel();
        let fragment = sealed.fragment().unwrap();
        let children: Vec<_> = fragment.root().children().collect();
        assert_eq!(children.len(), 2);
        assert_ne!(children[0], children[1]);
        for (id, expected) in children.iter().zip(["A&B", "Z"]) {
            let node = fragment.node(id).unwrap();
            let crate::HtmlNodeKind::Element(element) = node.kind() else {
                panic!("real p")
            };
            assert_eq!(
                element.name().local_name(),
                &crate::DocumentString::from("p")
            );
            let text = node.children().next().unwrap();
            let crate::HtmlNodeKind::Text(text) = fragment.node(&text).unwrap().kind() else {
                panic!("real child text")
            };
            assert_eq!(text, &crate::DocumentString::from(expected));
        }
        drop(children);
        assert!(sealed.evidence().is_ok());
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before
        );
        assert_eq!(
            sealed
                .final_usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            sealed.source().len() as u64
        );
        sealed.release().unwrap();
        assert_reclaimed(sealed.retained_usage());
    }
    #[test]
    fn pre_source_zero_work_zero_and_cancel_keep_original_error_and_honest_absent_binding() {
        for (source_cap, work, cancelled) in [
            (0, 100_000_000, false),
            (100, 0, false),
            (100, 100_000_000, true),
        ] {
            let original = with_limits("<p>A", limits(source_cap, work));
            if cancelled {
                original.cancellation().cancel();
            }
            let id = original.source().identity().clone();
            let mut owner = FragmentInitializationOwner::new(original);
            {
                let mut attempt = owner.begin().unwrap();
                assert_eq!(
                    attempt.take_driver().err(),
                    Some(InitializationState::Failed)
                );
                assert!(attempt.reason().is_some());
                assert!(attempt.usage().is_some());
                let first = attempt
                    .entry
                    .driver
                    .as_ref()
                    .unwrap()
                    .control
                    .stop()
                    .unwrap();
                attempt.retain_failure().unwrap();
                assert_eq!(attempt.stopped.as_ref().unwrap().stop(), first);
                assert!(attempt.stopped.as_ref().unwrap().source_binding().is_none());
            }
            assert!(owner.failure().is_some());
            let mut failure = owner.take_failure().unwrap();
            assert_eq!(failure.source().identity(), &id);
            assert!(failure.reason().is_some());
            assert!(failure.diagnostics().is_none());
            assert_eq!(
                failure
                    .at_stop()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                0
            );
            assert_reclaimed(failure.after_cleanup());
            failure.release().unwrap();
            assert_reclaimed(failure.retained_usage());
            assert!(owner.take_failure().is_err());
        }
    }
    #[test]
    fn initialized_capability_error_keeps_source_binding_and_original_qualified_reason() {
        // This real managed-node cap refuses the actual parser Document before
        // any F/context allocation. Source admission has already committed.
        let restricted = with_limits(
            "<p>A",
            limits(100, 100_000_000)
                .into_builder()
                .managed_nodes(0)
                .finish()
                .unwrap(),
        );
        let capture = restricted.context().capture().identity().clone();
        let mut owner = FragmentInitializationOwner::new(restricted);
        {
            let mut attempt = owner.begin().unwrap();
            assert_eq!(
                attempt.take_driver().err(),
                Some(InitializationState::Failed)
            );
            attempt.retain_failure().unwrap();
        }
        let mut failure = owner.take_failure().unwrap();
        assert!(matches!(
            failure.stop(),
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedNodes,
                ..
            }
        ));
        assert!(failure.reason().is_some());
        assert!(failure.failure.source_binding().is_some());
        assert_eq!(failure.context().capture().identity(), &capture);
        failure.release().unwrap();
        assert_reclaimed(failure.retained_usage());
    }
    #[test]
    fn active_cancel_failure_detaches_before_original_request_move_and_keeps_committed_diagnostics()
    {
        let mut owner = FragmentInitializationOwner::new(request("<!doctype html><p>A</p>"));
        let mut stopped = {
            let mut attempt = owner.begin().unwrap();
            let mut driver = Some(attempt.take_driver().unwrap());
            super::super::tests::complete(driver.as_mut().unwrap());
            driver.as_ref().unwrap().request.cancellation().cancel();
            let mut reason = ContextFragmentDriver::take_completed(&mut driver).err();
            let mut failed = Some(ContextFragmentDriver::take_failed(&mut driver).unwrap());
            let detached = StoppedFragmentConstruction::take(&mut failed, &mut reason).unwrap();
            assert!(failed.is_none());
            Some(detached)
        };
        owner.retain_execution_failure(&mut stopped).unwrap();
        assert!(stopped.is_none());
        let mut failure = owner.take_failure().unwrap();
        assert_eq!(failure.stop(), ParseStop::Cancelled);
        assert!(!failure.diagnostics().unwrap().is_empty());
        assert!(failure.reason().is_some());
        failure.release().unwrap();
        assert_reclaimed(failure.retained_usage());
    }
    #[test]
    fn foreign_original_request_refuses_seal_without_moving_frozen_or_original_slots() {
        let mut first = FragmentInitializationOwner::new(request("<p>A</p>"));
        let mut parts = {
            let mut attempt = first.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        let mut foreign = FragmentInitializationOwner::new(request("<p>A</p>"));
        assert!(matches!(
            foreign.seal(&mut parts),
            Err(FragmentExecutionError::Consumed)
        ));
        assert!(foreign.source().is_some());
        assert!(parts.is_some());
        let mut sealed = first.seal(&mut parts).unwrap();
        sealed.release().unwrap();
        assert_reclaimed(sealed.retained_usage());
    }
    #[test]
    fn cancelled_seal_retains_both_slots_then_moves_only_diagnostics_to_original_failure() {
        let original = request("<!doctype html><p>A</p>");
        let cancel = original.cancellation().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        cancel.cancel();
        let mut reason = owner.seal(&mut parts).err();
        assert!(owner.source().is_some());
        assert!(parts.is_some());
        let first = parts.as_ref().unwrap().control.stop().unwrap();
        let mut stopped =
            Some(StoppedFragmentConstruction::take_frozen(&mut parts, &mut reason).unwrap());
        assert!(parts.is_none());
        owner.retain_execution_failure(&mut stopped).unwrap();
        let mut failure = owner.take_failure().unwrap();
        assert_eq!(failure.stop(), first);
        assert_eq!(first, ParseStop::Cancelled);
        assert!(!failure.diagnostics().unwrap().is_empty());
        failure.release().unwrap();
        assert_reclaimed(failure.retained_usage());
    }
    #[test]
    fn frozen_failure_alias_refusal_retains_owner_and_first_stop_for_real_retry() {
        let original = request("<!doctype html><p>A</p>");
        let cancel = original.cancellation().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        let alias = parts
            .as_ref()
            .unwrap()
            .arena
            .as_ref()
            .unwrap()
            .fragment()
            .unwrap()
            .clone();
        cancel.cancel();
        let mut reason = owner.seal(&mut parts).err();
        let first = parts.as_ref().unwrap().control.stop().unwrap();
        assert!(StoppedFragmentConstruction::take_frozen(&mut parts, &mut reason).is_err());
        assert!(parts.as_ref().unwrap().arena.is_some());
        assert!(owner.source().is_some());
        assert!(reason.is_some());
        assert_eq!(parts.as_ref().unwrap().control.stop(), Some(first));
        drop(alias);
        let mut stopped =
            Some(StoppedFragmentConstruction::take_frozen(&mut parts, &mut reason).unwrap());
        owner.retain_execution_failure(&mut stopped).unwrap();
        let mut failure = owner.take_failure().unwrap();
        assert_eq!(failure.stop(), first);
        assert_eq!(failure.diagnostics().unwrap().len(), 1);
        failure.release().unwrap();
        assert_reclaimed(failure.retained_usage());
    }
    #[test]
    fn sealed_native_alias_refusal_preserves_original_and_actual_arena_until_disposal_retry() {
        let mut owner = FragmentInitializationOwner::new(request("<p>A</p>"));
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        let mut sealed = owner.seal(&mut parts).unwrap();
        let alias = sealed.fragment().unwrap().clone();
        assert!(sealed.release().is_err());
        assert!(sealed.fragment().is_some());
        assert_eq!(sealed.source().len(), 8);
        drop(alias);
        sealed.release().unwrap();
        assert!(sealed.fragment().is_none());
        assert_reclaimed(sealed.retained_usage());
    }
    #[test]
    fn source_qualified_seal_exact_and_one_short_fixed_transfer_preserve_both_real_slots() {
        let literal = "<!doctype html><p>A</p>";
        let mut measured = FragmentInitializationOwner::new(request(literal));
        let mut measured_parts = {
            let mut attempt = measured.begin().unwrap();
            freeze(&mut attempt)
        };
        let before = measured_parts
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let effects = measured_parts
            .arena
            .as_ref()
            .unwrap()
            .frozen_fragment_effects()
            .unwrap();
        assert_eq!(effects.len(), 0);
        let slots = effects.registry_slots();
        let bindings: Vec<_> = (0..slots)
            .filter_map(|index| effects.registry_binding(index).unwrap())
            .collect();
        // The internal html and literal p have the two actual registry rows.
        let names: Vec<_> = bindings
            .iter()
            .map(|binding| {
                let crate::HtmlNodeKind::Element(element) = binding.node.kind() else {
                    panic!("actual E")
                };
                element.name().local_name().clone()
            })
            .collect();
        assert_eq!(
            names,
            [
                crate::DocumentString::from("html"),
                crate::DocumentString::from("p")
            ]
        );
        let effect_qualification =
            slots as u64 + 2 * (1 + crate::construction::context::registry_binding_layout() as u64);
        drop(bindings);
        drop(names);
        measured_parts.release().unwrap();
        // One request-authority call, eleven comparisons, their actual fixed
        // envelopes, then seven request fields, two joined fields and one
        // final usage snapshot. Source qualification only polls. The parts
        // move into the join by value, so no option slot is cleared.
        let reads = 2 * size_of::<crate::HtmlFragmentContext>()
            + 2 * size_of::<crate::HtmlSourceId>()
            + 2 * size_of::<crate::HtmlSourceCoordinates>()
            + 2 * size_of::<usize>()
            + 4 * size_of::<HtmlParserScriptingMode>()
            + 2 * size_of::<bool>();
        let moves = size_of::<SealedFragmentConstruction>()
            + size_of::<FragmentRequestParts>()
            + size_of::<ParseUsage>();
        let tail = 1 + 11 + reads as u64 + effect_qualification + moves as u64 + 10;
        for missing in [0, 1] {
            let mut owner = FragmentInitializationOwner::new(with_limits(
                literal,
                limits(100, before + tail - missing),
            ));
            let mut parts = {
                let mut attempt = owner.begin().unwrap();
                Some(freeze(&mut attempt))
            };
            let result = owner.seal(&mut parts);
            if missing == 0 {
                let mut sealed = result.unwrap();
                assert!(parts.is_none());
                assert_eq!(
                    sealed
                        .final_usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    before + tail
                );
                // There are no active credits after exact completion: cleanup
                // may latch a stop and must preserve/reclaim its actual owner.
                if sealed.release().is_err() {
                    sealed.release().unwrap();
                }
                assert_reclaimed(sealed.retained_usage());
            } else {
                assert!(matches!(
                    result,
                    Err(FragmentExecutionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }))
                ));
                assert!(owner.source().is_some());
                assert!(parts.as_ref().unwrap().arena.is_some());
                let first = parts.as_ref().unwrap().control.stop().unwrap();
                let mut reason = Some(FragmentExecutionError::Stop(first));
                let mut stopped = Some(
                    StoppedFragmentConstruction::take_frozen(&mut parts, &mut reason).unwrap(),
                );
                owner.retain_execution_failure(&mut stopped).unwrap();
                let mut failure = owner.take_failure().unwrap();
                assert_eq!(failure.stop(), first);
                assert_eq!(failure.diagnostics().unwrap().len(), 1);
                failure.release().unwrap();
                assert_reclaimed(failure.retained_usage());
            }
        }
    }
    #[test]
    fn initialization_fixed_transfer_one_short_layout_retains_source_and_bootstrap_carrier() {
        let bytes = size_of::<FragmentEntry>()
            + initialization_transfer_layout().0
            + 2 * size_of::<ParseUsage>();
        let original = with_limits(
            "<p>A",
            limits(100, 100_000_000)
                .into_builder()
                .atomic_layout_bytes(bytes as u64 - 1)
                .finish()
                .unwrap(),
        );
        let mut owner = FragmentInitializationOwner::new(original);
        {
            let mut attempt = owner.begin().unwrap();
            assert_eq!(
                attempt.take_driver().err(),
                Some(InitializationState::Failed)
            );
            assert!(matches!(
                attempt.entry.driver.as_ref().unwrap().control.stop(),
                Some(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AtomicLayoutBytes,
                    ..
                })
            ));
            assert_eq!(
                attempt
                    .usage()
                    .unwrap()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                0
            );
            attempt.retain_failure().unwrap();
        }
        assert!(owner.failure().unwrap().source_binding().is_some());
        let mut failure = owner.take_failure().unwrap();
        assert!(failure.diagnostics().is_none());
        failure.release().unwrap();
        assert_reclaimed(failure.retained_usage());
    }
    #[test]
    fn every_new_initial_fixed_admission_cut_keeps_actual_request_stop_and_zero_cleanup() {
        let bytes = size_of::<FragmentEntry>()
            + initialization_transfer_layout().0
            + 2 * size_of::<ParseUsage>();
        let writes = 6 + initialization_transfer_layout().1 + 2;
        // admit_source RecordVisit1, actual fixed CopyByte, actual receiver
        // writes, then qualified terminal identity inspections. Each cut is
        // chosen at one new boundary before namespace/context/root effects.
        for budget in [
            0,
            1,
            1 + bytes as u64 - 1,
            1 + bytes as u64,
            1 + bytes as u64 + writes as u64 - 1,
            1 + bytes as u64 + writes as u64,
            1 + bytes as u64 + writes as u64 + terminal_qualification_bytes() as u64 - 1,
        ] {
            let mut owner =
                FragmentInitializationOwner::new(with_limits("<p>A", limits(100, budget)));
            {
                let mut attempt = owner.begin().unwrap();
                assert_eq!(
                    attempt.take_driver().err(),
                    Some(InitializationState::Failed)
                );
                let control = &attempt.entry.driver.as_ref().unwrap().control;
                assert!(matches!(
                    control.stop(),
                    Some(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                assert_eq!(
                    control
                        .usage()
                        .resource(HtmlParseResource::ManagedNodes)
                        .current,
                    0
                );
                assert!(attempt.reason().is_some());
                attempt.retain_failure().unwrap();
            }
            let stopped = owner.failure().unwrap();
            assert_eq!(stopped.source_binding().is_some(), budget > 0);
            let first = stopped.stop();
            let mut failure = owner.take_failure().unwrap();
            assert_eq!(failure.stop(), first);
            assert!(failure.diagnostics().is_none());
            assert!(owner.begin().is_err());
            failure.release().unwrap();
            assert_reclaimed(failure.retained_usage());
            assert_eq!(
                failure
                    .retained_usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                failure
                    .at_stop()
                    .resource(HtmlParseResource::WorkCredits)
                    .current
            );
        }
    }
    #[test]
    fn initialized_empty_diagnostics_prefix_is_distinct_from_pre_evidence_absence() {
        for cap in [0, 1, 2] {
            let original = with_limits(
                "<p>A",
                limits(100, 100_000_000)
                    .into_builder()
                    .managed_nodes(cap)
                    .finish()
                    .unwrap(),
            );
            let mut owner = FragmentInitializationOwner::new(original);
            {
                let mut attempt = owner.begin().unwrap();
                assert_eq!(
                    attempt.take_driver().err(),
                    Some(InitializationState::Failed)
                );
                attempt.retain_failure().unwrap();
            }
            let mut failure = owner.take_failure().unwrap();
            assert!(matches!(
                failure.stop(),
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedNodes,
                    ..
                }
            ));
            assert_eq!(
                failure
                    .at_stop()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                cap
            );
            assert_eq!(failure.diagnostics().is_some(), cap > 0);
            if let Some(diagnostics) = failure.diagnostics() {
                assert_eq!(diagnostics.len(), 0);
            }
            failure.release().unwrap();
            assert_reclaimed(failure.retained_usage());
            assert_eq!(
                failure
                    .retained_usage()
                    .resource(HtmlParseResource::ManagedNodes)
                    .current,
                cap
            );
        }
    }
    #[test]
    fn cancellation_between_real_initialization_and_driver_transfer_retains_ready_owners_for_failure_cleanup()
     {
        let original = request("<p>A");
        let cancel = original.cancellation().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        {
            let mut attempt = owner.begin().unwrap();
            assert!(attempt.entry.driver.as_ref().unwrap().head.is_some());
            cancel.cancel();
            assert_eq!(
                attempt.take_driver().err(),
                Some(InitializationState::Failed)
            );
            assert!(matches!(
                attempt.reason(),
                Some(FragmentExecutionError::Stop(ParseStop::Cancelled))
            ));
            attempt.retain_failure().unwrap();
        }
        let mut failure = owner.take_failure().unwrap();
        assert_eq!(failure.stop(), ParseStop::Cancelled);
        assert_eq!(failure.diagnostics().unwrap().len(), 0);
        failure.release().unwrap();
        assert_reclaimed(failure.retained_usage());
    }
    /// The request join of a fragment failure, listed term by term from what
    /// production moves rather than read from `request_join_layout`: the
    /// carrier's stopped slot (cleared), the request parts it hands over and
    /// the owned failure that joins them, with one write per cleared slot,
    /// seven request fields and two joined fields.
    fn listed_request_join() -> (usize, usize) {
        (
            size_of::<Option<StoppedFragmentConstruction>>()
                + size_of::<FragmentRequestParts>()
                + size_of::<OwnedFragmentFailure>(),
            1 + 7 + 2,
        )
    }
    /// The transfer of a cleaned fragment failure into its carrier, listed
    /// term by term and not read from `execution_transfer_layout`: the
    /// controller's source binding slot (cleared), the stopped construction
    /// built by value, the carrier's two own failure options, and the request
    /// join. The cleaned failure and the error move by value, so neither has
    /// a slot term.
    fn listed_execution_transfer() -> (usize, usize) {
        let (join_bytes, join_writes) = listed_request_join();
        (
            size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
                + size_of::<StoppedFragmentConstruction>()
                + 2 * size_of::<Option<StoppedFragmentConstruction>>()
                + join_bytes,
            1 + 7 + 2 + join_writes,
        )
    }
    #[test]
    fn the_lowered_fragment_failure_charges_are_the_listed_production_terms() {
        // A frozen failure is cleaned and detached into the carrier by value
        // exactly as an active one is, so both layouts are the same listed
        // transfer. Neither carries a slot for the failed owner or the error.
        assert_eq!(execution_transfer_layout(), listed_execution_transfer());
        assert_eq!(frozen_failure_layout(), listed_execution_transfer());
        // The layouts changed when their stale option slots were removed, so
        // the listed sum is also spelled without the shared helper: three
        // option slots and nine moved fields of the carrier, then the join.
        let (bytes, writes) = listed_execution_transfer();
        assert_eq!(
            bytes,
            size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
                + size_of::<StoppedFragmentConstruction>()
                + 3 * size_of::<Option<StoppedFragmentConstruction>>()
                + size_of::<FragmentRequestParts>()
                + size_of::<OwnedFragmentFailure>()
        );
        assert_eq!(writes, 20);
    }
    #[test]
    fn ready_driver_transfer_exact_and_one_short_retains_real_stage_without_token_pull() {
        let measured_request = request("<p>A");
        let mut measured = FragmentEntry::new(
            &measured_request,
            super::super::parse::InitializationTarget::Native,
        );
        let before = measured
            .driver
            .as_ref()
            .unwrap()
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        measured_request.cancellation().cancel();
        assert!(measured.take_driver().is_err());
        measured.release_failure().unwrap();
        // Actual driver + owning Option movement, separate active terminal
        // failure/join envelope, six failure fields/two usage snapshots,
        // followed by the two additional qualified terminal joins.
        let (execution_bytes, execution_writes) = listed_execution_transfer();
        let bytes = size_of::<ContextFragmentDriver>()
            + size_of::<Option<ContextFragmentDriver>>()
            + execution_bytes
            + size_of::<FailedContextFragment>()
            + 2 * size_of::<ParseUsage>();
        let writes = 2 + execution_writes + 8;
        let tail = bytes as u64 + writes as u64 + 2 * terminal_qualification_bytes() as u64;
        for missing in [0, 1] {
            let mut owner = FragmentInitializationOwner::new(with_limits(
                "<p>A",
                limits(100, before + tail - missing),
            ));
            {
                let mut attempt = owner.begin().unwrap();
                if missing == 0 {
                    let mut driver = attempt.take_driver().unwrap();
                    assert_eq!(driver.control.usage().work(WorkCategory::InputUnitRead), 0);
                    assert_eq!(
                        driver
                            .control
                            .usage()
                            .resource(HtmlParseResource::ManagedNodes)
                            .current,
                        3
                    );
                    driver.request.cancellation().cancel();
                    assert!(driver.step().is_err());
                    driver.release().unwrap();
                    super::super::super::tests::assert_released(&driver.control);
                } else {
                    assert_eq!(
                        attempt.take_driver().err(),
                        Some(InitializationState::Failed)
                    );
                    assert!(attempt.entry.driver.as_ref().unwrap().head.is_some());
                    assert_eq!(
                        attempt.usage().unwrap().work(WorkCategory::InputUnitRead),
                        0
                    );
                    attempt.retain_failure().unwrap();
                }
            }
            if missing == 1 {
                let mut failure = owner.take_failure().unwrap();
                assert!(matches!(
                    failure.stop(),
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                assert_eq!(failure.diagnostics().unwrap().len(), 0);
                failure.release().unwrap();
                assert_reclaimed(failure.retained_usage());
            }
        }
    }
    #[test]
    fn missing_captured_shadow_fact_preserves_actual_error_context_and_canonical_stop() {
        let original = super::super::tests::shadow_request_with_presence(
            "<template shadowrootmode=open><p>A</p></template>",
            true,
            false,
        );
        let target = original.context().target().clone();
        let capture = original.context().capture().identity().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut stopped = {
            let mut attempt = owner.begin().unwrap();
            let mut driver = Some(attempt.take_driver().unwrap());
            let mut reason = Some(loop {
                match driver.as_mut().unwrap().step() {
                    Err(error) => break error,
                    Ok(HeadStep::Consumed) => {}
                    Ok(other) => panic!("unexpected pre-refusal step {other:?}"),
                }
            });
            assert!(matches!(
                reason,
                Some(FragmentExecutionError::Head(head::HeadError::Kernel(
                    KernelError::Session(SessionError::ContextFactUnavailable(
                        crate::HtmlCaptureFact::ShadowRoot
                    ))
                )))
            ));
            let mut failed = Some(ContextFragmentDriver::take_failed(&mut driver).unwrap());
            Some(StoppedFragmentConstruction::take(&mut failed, &mut reason).unwrap())
        };
        owner.retain_execution_failure(&mut stopped).unwrap();
        let mut failure = owner.take_failure().unwrap();
        assert_eq!(failure.context().capture().identity(), &capture);
        assert_eq!(failure.context().target(), &target);
        assert!(matches!(
            failure.reason(),
            Some(FragmentExecutionError::Head(head::HeadError::Kernel(
                KernelError::Session(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::ShadowRoot
                ))
            )))
        ));
        assert_eq!(
            failure.stop(),
            ParseStop::UnsupportedStandaloneState(StandaloneUnsupported::Provenance)
        );
        failure.release().unwrap();
        assert_reclaimed(failure.retained_usage());
    }
    #[test]
    fn sealed_captured_shadow_effects_keep_literal_order_properties_nodes_and_source_after_cancel()
    {
        use crate::construction::context::{FrozenFragmentEffect, FrozenFragmentHost};
        let literal = "<template shadowrootmode=closed shadowrootdelegatesfocus shadowrootserializable shadowrootclonable shadowrootslotassignment=manual shadowrootcustomelementregistry><p>A</p></template><template shadowrootmode=open><p>B</p></template><p>Z</p>";
        let original = super::super::tests::shadow_request(literal, true);
        let cancel = original.cancellation().clone();
        let host = original.context().context_element().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        let mut sealed = owner.seal(&mut parts).unwrap();
        let before = sealed
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        cancel.cancel();
        {
            let effects = sealed.effects().unwrap();
            assert_eq!(effects.len(), 2); // attachment, then target inert-cache for fallback template
            let Some(FrozenFragmentEffect::AttachShadow {
                host: FrozenFragmentHost::Captured(actual),
                template,
                contents,
                properties,
                keep_registry_null,
                declarative,
                available_to_internals,
                source,
            }) = effects.get(0).unwrap()
            else {
                panic!("actual attach")
            };
            assert_eq!(actual, &host);
            assert_ne!(template.id(), contents.id());
            assert!(matches!(contents.kind(), crate::HtmlNodeKind::Fragment));
            let crate::HtmlNodeKind::Element(element) = template.kind() else {
                panic!("template")
            };
            assert_eq!(
                element.name().local_name(),
                &crate::DocumentString::from("template")
            );
            assert_eq!(properties.mode(), crate::HtmlShadowRootMode::Closed);
            assert_eq!(
                properties.slot_assignment(),
                crate::HtmlSlotAssignment::Manual
            );
            assert!(
                properties.delegates_focus() && properties.clonable() && properties.serializable()
            );
            assert!(keep_registry_null && declarative && available_to_internals);
            assert_eq!(source.source(), sealed.source().identity());
            assert_eq!(source.start(), 0);
            assert_eq!(source.end(), literal.find('>').unwrap() + 1);
            let id = contents.children().next().unwrap();
            let native = sealed.parts.arena.as_ref().unwrap().fragment().unwrap();
            let p = native.node(&id).unwrap();
            let text = p.children().next().unwrap();
            assert!(
                matches!(native.node(&text).unwrap().kind(), crate::HtmlNodeKind::Text(value)
                if value == &crate::DocumentString::from("A"))
            );
            let Some(FrozenFragmentEffect::TemplateCache { document, inert }) =
                effects.get(1).unwrap()
            else {
                panic!("actual fallback target cache")
            };
            assert_eq!(
                &document.node_document(),
                sealed.context().target_document()
            );
            assert!(matches!(
                inert.kind(),
                crate::HtmlNodeKind::Document(crate::HtmlDocumentMode::NoQuirks)
            ));
            assert!(effects.get(2).unwrap().is_none());
        }
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before
        );
        sealed.release().unwrap();
        assert_reclaimed(sealed.retained_usage());
    }
    #[test]
    fn frozen_effect_registry_rows_are_actual_borrowed_capture_qualifications_after_cancel() {
        use crate::construction::context::FrozenFragmentRegistry;
        let mut builder = crate::HtmlCaptureBuilder::new(
            crate::HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
            crate::HtmlHostFactsRevision::fresh(),
        );
        let registry = builder.add_registry(Some(Vec::new())).unwrap();
        builder
            .establish_registry_scope(&registry, crate::HtmlRegistryScope::Global)
            .unwrap();
        let doc = builder
            .add_document(
                crate::HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                crate::HtmlCaptureRegistryInput::Registry(registry.clone()),
            )
            .unwrap();
        let context = builder
            .add_element(
                &doc,
                crate::QualifiedName::new(crate::Namespace::html(), None, "div"),
                crate::OrderedAttributes::default(),
                crate::HtmlCapturedIsValue::Absent,
                crate::HtmlCaptureRegistryInput::Registry(registry.clone()),
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let id = capture.registry_id(&registry).unwrap();
        let context =
            crate::HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap())
                .unwrap();
        let original = HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<p>A</p>"),
            context,
            limits(100, 100_000_000),
        )
        .unwrap();
        let cancel = original.cancellation().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        let mut sealed = owner.seal(&mut parts).unwrap();
        cancel.cancel();
        let before = sealed
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut names = Vec::new();
        {
            let effects = sealed.effects().unwrap();
            for slot in 0..effects.registry_slots() {
                let Some(binding) = effects.registry_binding(slot).unwrap() else {
                    continue;
                };
                let FrozenFragmentRegistry::Captured(registry) = binding.registry else {
                    panic!("actual registry")
                };
                assert!(registry.is_registry(&id));
                let crate::HtmlNodeKind::Element(element) = binding.node.kind() else {
                    panic!("actual E")
                };
                names.push(element.name().local_name().clone());
            }
            assert!(
                effects
                    .registry_binding(effects.registry_slots())
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(
            names,
            [
                crate::DocumentString::from("html"),
                crate::DocumentString::from("p")
            ]
        );
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before
        );
        drop(names);
        sealed.release().unwrap();
        assert_reclaimed(sealed.retained_usage());
    }
    #[test]
    fn sealed_owned_shadow_host_effect_uses_same_native_arena_without_capture_impersonation() {
        use crate::construction::context::{FrozenFragmentEffect, FrozenFragmentHost};
        let original = super::super::tests::shadow_request(
            "<div><template shadowrootmode=open><p>A</p></template><span>L</span></div>",
            true,
        );
        let cancel = original.cancellation().clone();
        let mut owner = FragmentInitializationOwner::new(original);
        let mut parts = {
            let mut attempt = owner.begin().unwrap();
            Some(freeze(&mut attempt))
        };
        let mut sealed = owner.seal(&mut parts).unwrap();
        let before = sealed
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        cancel.cancel();
        {
            let effects = sealed.effects().unwrap();
            let Some(FrozenFragmentEffect::AttachShadow {
                host: FrozenFragmentHost::Owned(host),
                contents,
                template,
                ..
            }) = effects.get(1).unwrap()
            else {
                panic!("actual local host")
            };
            let fragment = sealed.fragment().unwrap();
            let first = fragment.root().children().next().unwrap();
            assert_eq!(host.id(), first);
            assert_ne!(contents.id(), first);
            assert_ne!(template.id(), first);
            assert_eq!(
                host.declarative_shadow_root().unwrap().contents(),
                &contents.id()
            );
            let Some(FrozenFragmentEffect::TemplateCache { document, .. }) =
                effects.get(0).unwrap()
            else {
                panic!("creation first requires target inert cache")
            };
            assert_eq!(
                &document.node_document(),
                sealed.context().target_document()
            );
            assert!(effects.get(2).unwrap().is_none());
        }
        assert_eq!(
            sealed
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            before
        );
        sealed.release().unwrap();
        assert_reclaimed(sealed.retained_usage());
    }
}
