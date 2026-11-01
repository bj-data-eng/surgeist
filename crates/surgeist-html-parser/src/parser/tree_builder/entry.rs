//! Private ordinary initialization owns every admitted constructor prefix.
//! Source backing remains borrowed from the actual request; public source-owning
//! report assembly and frozen-result transfer are separate consumers.
use super::{StandaloneDocumentDriver, StandaloneError, prologue, write};
use crate::parser::control::{ParseControl, ParseStop, ParseUsage, StandaloneUnsupported};
use crate::{HtmlDocumentParseRequest, HtmlParseResource, HtmlParserScriptingMode, HtmlSource};
use std::mem::size_of;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EntryRejection {
    DocumentInitialization,
    ExecutionProfile,
}
impl EntryRejection {
    pub(super) fn for_request(request: &HtmlDocumentParseRequest) -> Option<Self> {
        if request.capture().is_some() || request.document().is_some() {
            Some(Self::DocumentInitialization)
        } else if request.scripting_mode() != HtmlParserScriptingMode::Disabled
            || request.allow_declarative_shadow_roots()
        {
            Some(Self::ExecutionProfile)
        } else {
            None
        }
    }
    pub(super) fn error(self) -> StandaloneError {
        match self {
            Self::DocumentInitialization => StandaloneError::UnsupportedDocumentInitialization,
            Self::ExecutionProfile => StandaloneError::UnsupportedExecutionProfile,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InitializationFailure {
    Rejected(EntryRejection),
    Stopped(ParseStop),
}
#[derive(Clone, Copy)]
enum Phase {
    Ready,
    Rejected(EntryRejection),
    Failed,
    Transferred,
}

/// A borrowed owning guard, not a parse result. Rejection before execution has
/// no controller/usage to invent. Once execution starts, the actual sole
/// controller and partial driver stay reachable through every initialization
/// error and cleanup retry. No tree access is available from a failed entry.
pub(crate) struct OrdinaryDocumentEntry<'source> {
    source: &'source HtmlSource,
    driver: Option<StandaloneDocumentDriver<'source>>,
    phase: Phase,
    after_cleanup: Option<ParseUsage>,
}
impl<'source> OrdinaryDocumentEntry<'source> {
    #[cfg(test)]
    pub(crate) fn new(request: &'source HtmlDocumentParseRequest) -> Self {
        request.require_inputs_reach_execution();
        Self::create(
            request,
            false,
            super::parse::InitializationTarget::Native,
            false,
        )
    }
    pub(super) fn owned(
        request: &'source HtmlDocumentParseRequest,
        target: super::parse::InitializationTarget,
    ) -> Self {
        Self::create(
            request,
            true,
            target,
            #[cfg(test)]
            false,
        )
    }
    #[cfg(test)]
    pub(super) fn owned_progressive(request: &'source HtmlDocumentParseRequest) -> Self {
        Self::create(
            request,
            true,
            super::parse::InitializationTarget::Native,
            true,
        )
    }
    fn create(
        request: &'source HtmlDocumentParseRequest,
        owned: bool,
        target: super::parse::InitializationTarget,
        #[cfg(test)] progressive: bool,
    ) -> Self {
        if let Some(reason) = EntryRejection::for_request(request) {
            return Self {
                source: request.source(),
                driver: None,
                phase: Phase::Rejected(reason),
                after_cleanup: None,
            };
        }
        // This fixed guard is established before any admission can fail. The
        // selected bootstrap retains the existing control/Arc provider limits;
        // it performs no variable allocation for an error envelope.
        let mut driver = StandaloneDocumentDriver {
            source: request.source(),
            control: ParseControl::new(*request.limits(), request.cancellation().clone()),
            prologue: None,
            tokenizer: None,
            event: None,
            token: None,
            before: None,
            head: None,
            lexical: None,
            failed_evidence: None,
            failure_usage: None,
        };
        let phase = match Self::prepare(
            &mut driver,
            request,
            owned,
            target,
            #[cfg(test)]
            progressive,
        ) {
            Ok(()) => Phase::Ready,
            Err(error) => {
                // Resource/cancellation failure already owns the first stop. An
                // unexpected nonterminal stage error cannot leave a runnable
                // prefix; a lost owner is an invariant, not an unsupported
                // document.
                if driver.control.stop().is_none() {
                    if error.lost_owner() {
                        StandaloneError::owner_lost(&mut driver.control);
                    } else {
                        driver
                            .control
                            .stop_unsupported_standalone(StandaloneUnsupported::InvalidNesting);
                    }
                }
                Phase::Failed
            }
        };
        Self {
            source: request.source(),
            driver: Some(driver),
            phase,
            after_cleanup: None,
        }
    }
    fn prepare(
        driver: &mut StandaloneDocumentDriver<'source>,
        request: &HtmlDocumentParseRequest,
        owned: bool,
        target: super::parse::InitializationTarget,
        #[cfg(test)] progressive: bool,
    ) -> Result<(), StandaloneError> {
        let c = &mut driver.control;
        // Preserve source-admission precedence and use actual compiled layouts.
        c.admit_source(driver.source)?;
        target.admit(c)?;
        if owned {
            let (bytes, writes) = super::seal::initialization_bootstrap_layout();
            write(bytes, writes, c)?;
            c.charge(
                super::super::control::WorkCategory::InspectByte,
                super::seal::initialization_qualification_bytes() as u64,
            )?;
            c.charge(super::super::control::WorkCategory::ProviderCall, 3)?;
            c.charge(super::super::control::WorkCategory::RecordVisit, 6)?;
        }
        write(size_of::<StandaloneDocumentDriver>(), 11, c)?;
        write(
            const {
                size_of::<Option<ParseUsage>>()
                    + 2 * size_of::<Option<crate::construction::evidence::FailureDiagnosticEvidence>>(
                    )
                    + size_of::<super::terminal::FailedStandaloneDocument>()
                    + super::seal::terminal_transfer_layout().0
                    + size_of::<Option<StandaloneDocumentDriver>>()
                    + size_of::<Self>()
            },
            const { 1 + 2 + 6 + super::seal::terminal_transfer_layout().1 + 1 + 4 },
            c,
        )?;
        c.charge(
            super::super::control::WorkCategory::InspectByte,
            super::seal::terminal_qualification_bytes() as u64,
        )?;
        c.charge(super::super::control::WorkCategory::ProviderCall, 2)?;
        c.charge(super::super::control::WorkCategory::RecordVisit, 2)?;
        let mut lexical =
            crate::parser::control::OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        lexical.require_owner(c)?;
        driver.lexical = Some(lexical);
        #[cfg(test)]
        let make = if progressive {
            crate::parser::tokenizer::Tokenizer::progressive
        } else {
            crate::parser::tokenizer::Tokenizer::new
        };
        #[cfg(not(test))]
        let make = crate::parser::tokenizer::Tokenizer::new;
        driver.tokenizer = Some(make(
            driver.source,
            crate::parser::tokenizer::states::State::Data,
            None,
            c,
        )?);
        let prologue = driver
            .prologue
            .insert(prologue::DocumentPrologue::new(driver.source, c)?);
        if let Some(inputs) = request.shared_associated_inputs() {
            prologue
                .kernel()?
                .session
                .install_associated_inputs(c, inputs)
                .map_err(|error| StandaloneError::Head(error.into()))?;
        }
        if let Some(facts) = request.base_facts() {
            prologue
                .kernel()?
                .session
                .install_document_base_facts(c, facts)
                .map_err(|error| StandaloneError::Head(error.into()))?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn source(&self) -> &HtmlSource {
        self.source
    }
    #[cfg(test)]
    pub(crate) fn failure(&self) -> Option<InitializationFailure> {
        match self.phase {
            Phase::Rejected(reason) => Some(InitializationFailure::Rejected(reason)),
            Phase::Failed => Some(InitializationFailure::Stopped(
                self.driver
                    .as_ref()
                    .expect("retained failed entry")
                    .control
                    .stop()
                    .expect("failed execution owns first stop"),
            )),
            Phase::Ready | Phase::Transferred => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn at_stop(&self) -> Option<&ParseUsage> {
        self.driver.as_ref()?.control.first_stop_usage()
    }
    #[cfg(test)]
    pub(crate) fn after_cleanup(&self) -> Option<&ParseUsage> {
        self.after_cleanup.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn retained_usage(&self) -> Option<&ParseUsage> {
        self.driver.as_ref().map(|driver| driver.control.usage())
    }
    #[cfg(test)]
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.driver
            .as_ref()?
            .failed_evidence
            .as_ref()
            .map(|evidence| evidence.diagnostics())
    }
    pub(crate) fn take_driver(
        &mut self,
    ) -> Result<StandaloneDocumentDriver<'source>, StandaloneError> {
        match self.phase {
            Phase::Rejected(EntryRejection::DocumentInitialization) => {
                return Err(StandaloneError::UnsupportedDocumentInitialization);
            }
            Phase::Rejected(EntryRejection::ExecutionProfile) => {
                return Err(StandaloneError::UnsupportedExecutionProfile);
            }
            Phase::Failed => {
                let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
                return Err(match driver.control.stop() {
                    Some(stop) => StandaloneError::Stop(stop),
                    None => StandaloneError::owner_lost(&mut driver.control),
                });
            }
            Phase::Transferred => return Err(StandaloneError::Consumed),
            Phase::Ready => {}
        }
        let mut driver = self.driver.take().ok_or(StandaloneError::Consumed)?;
        let result = driver.control.require_source(self.source).and_then(|()| {
            write(
                const {
                    size_of::<StandaloneDocumentDriver>()
                        + size_of::<Option<StandaloneDocumentDriver>>()
                        + size_of::<Phase>()
                },
                3,
                &mut driver.control,
            )
        });
        if let Err(stop) = result {
            self.driver = Some(driver);
            self.phase = Phase::Failed;
            return Err(stop.into());
        }
        self.phase = Phase::Transferred;
        Ok(driver)
    }
    /// Retains safe committed diagnostics; no post-stop polling, useful work,
    /// allocation, or source reconstruction. Refusal leaves the actual owner.
    pub(crate) fn cleanup_failure(&mut self) -> Result<(), StandaloneError> {
        match self.phase {
            Phase::Rejected(_) => return Ok(()),
            Phase::Failed => {}
            Phase::Ready | Phase::Transferred => {
                return Err(StandaloneError::Consumed);
            }
        }
        if self.after_cleanup.is_some() {
            return Ok(());
        }
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        if driver.failure_usage.is_none() {
            let Some(at_stop) = driver.control.first_stop_usage().cloned() else {
                return Err(StandaloneError::owner_lost(&mut driver.control));
            };
            driver.failure_usage = Some(at_stop);
            if let Some(prologue) = &mut driver.prologue
                && let Some(kernel) = &mut prologue.kernel
            {
                driver.failed_evidence = kernel
                    .session
                    .take_failed_evidence(&mut driver.control)
                    .map_err(super::KernelError::from)
                    .map_err(super::head::HeadError::from)?;
            }
        }
        driver.reclaim_parse_owners()?;
        self.after_cleanup = Some(driver.control.usage().clone());
        Ok(())
    }
    /// Disposal of the retained safe evidence is separate from prefix cleanup.
    /// Snapshots remain return-time observations; work/peaks are not refunded.
    #[cfg(test)]
    pub(crate) fn release_failure(&mut self) -> Result<(), StandaloneError> {
        self.cleanup_failure()?;
        if let Some(driver) = &mut self.driver
            && let Some(evidence) = driver.failed_evidence.take()
        {
            evidence.release(&mut driver.control)?;
        }
        Ok(())
    }
    /// Move a cleaned stopped prefix into the caller's fixed bootstrap slot.
    /// Both slots and the admitted binding survive every qualification/cleanup
    /// refusal. No polling, charging, cloning or allocation follows stop.
    pub(super) fn retain_failure(
        &mut self,
        request: &HtmlDocumentParseRequest,
        output: &mut Option<StoppedInitialization>,
    ) -> Result<(), StandaloneError> {
        if !matches!(self.phase, Phase::Failed) || output.is_some() {
            return Err(StandaloneError::Consumed);
        }
        let driver = self.driver.as_ref().ok_or(StandaloneError::Consumed)?;
        if !driver.control.owns_request(request.cancellation())
            || self.source.identity() != request.source().identity()
            || self.source.coordinates() != request.source().coordinates()
        {
            return Err(StandaloneError::Consumed);
        }
        if let Some((identity, coordinates)) = driver.control.stopped_source_binding()?
            && (identity != request.source().identity()
                || *coordinates != request.source().coordinates())
        {
            return Err(StandaloneError::Consumed);
        }
        self.cleanup_failure()?;
        // Cleanup succeeded, so the observation exists; each move below puts
        // back what it took if the next one is refused.
        let Some(after_cleanup) = self.after_cleanup.take() else {
            return Err(match self.driver.as_mut() {
                Some(driver) => StandaloneError::owner_lost(&mut driver.control),
                None => StandaloneError::Consumed,
            });
        };
        let Some(mut driver) = self.driver.take() else {
            self.after_cleanup = Some(after_cleanup);
            return Err(StandaloneError::Consumed);
        };
        let binding = match driver.control.take_stopped_optional_source_binding() {
            Ok(binding) => binding,
            Err(stop) => {
                self.after_cleanup = Some(after_cleanup);
                self.driver = Some(driver);
                return Err(stop.into());
            }
        };
        *output = Some(StoppedInitialization {
            control: driver.control,
            binding,
            after_cleanup,
            evidence: driver.failed_evidence,
        });
        self.phase = Phase::Transferred;
        Ok(())
    }
}

/// No source borrow or native owner survives this actual cleaned prefix.
/// The original controller remains the sole authority for diagnostic tickets.
pub(super) struct StoppedInitialization {
    control: ParseControl,
    binding: Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>,
    after_cleanup: ParseUsage,
    evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
}
impl StoppedInitialization {
    pub(super) fn stop(&self) -> Option<ParseStop> {
        self.control.stop()
    }
    pub(super) fn source_admitted(&self) -> bool {
        self.binding.is_some()
    }
    pub(super) fn at_stop(&self) -> Option<&ParseUsage> {
        self.control.first_stop_usage()
    }
    pub(super) fn after_cleanup(&self) -> &ParseUsage {
        &self.after_cleanup
    }
    pub(super) fn retained_usage(&self) -> &ParseUsage {
        self.control.usage()
    }
    pub(super) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.evidence
            .as_ref()
            .map(|evidence| evidence.diagnostics())
    }
    #[cfg(test)]
    pub(super) fn release(&mut self) -> Result<(), StandaloneError> {
        if let Some(evidence) = self.evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
}

/// Actual fixed transfer: four stopped fields, three cleared owner/binding/
/// observation slots, one output write and one entry phase write. This layout
/// is included only by the real source-owning bootstrap consumer.
pub(super) const fn initialization_transfer_layout() -> (usize, usize) {
    const {
        (
            size_of::<StoppedInitialization>()
                + size_of::<Option<StandaloneDocumentDriver>>()
                + size_of::<Option<(crate::HtmlSourceId, crate::HtmlSourceCoordinates)>>()
                + size_of::<Option<ParseUsage>>()
                + size_of::<Option<StoppedInitialization>>()
                + size_of::<Phase>(),
            4 + 3 + 1 + 1,
        )
    }
}

#[cfg(test)]
pub(crate) fn initialized_driver(
    request: &HtmlDocumentParseRequest,
) -> StandaloneDocumentDriver<'_> {
    let mut entry = OrdinaryDocumentEntry::new(request);
    match entry.take_driver() {
        Ok(driver) => driver,
        Err(error) => {
            entry
                .release_failure()
                .expect("fixture initialization cleanup");
            panic!("fixture initialization failed: {error:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::control::WorkCategory;
    use crate::{DocumentString, HtmlSourceCoordinates};

    fn limits() -> crate::HtmlParseLimits {
        super::super::tests::limits(100)
    }
    fn released(usage: &ParseUsage) {
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
        ] {
            assert_eq!(usage.resource(resource).current, 0, "{resource:?}");
        }
        assert_eq!(usage.actual_capacity_bytes, 0);
    }
    #[test]
    fn cancelled_entry_preserves_exact_original_utf16_source_and_empty_first_observation() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(DocumentString::from_utf16(vec![
                0xfeff, 0xd800, 0xdc00, 0xdc01,
            ])),
            limits(),
        );
        request.cancellation().cancel();
        let mut entry = OrdinaryDocumentEntry::new(&request);
        assert!(std::ptr::eq(entry.source(), request.source()));
        assert_eq!(
            entry.source().coordinates(),
            HtmlSourceCoordinates::Utf16CodeUnits
        );
        assert_eq!(
            entry.source().as_utf16().unwrap().code_units(),
            &[0xfeff, 0xd800, 0xdc00, 0xdc01]
        );
        assert_eq!(
            entry.failure(),
            Some(InitializationFailure::Stopped(ParseStop::Cancelled))
        );
        let at_stop = entry.at_stop().unwrap().clone();
        assert_eq!(at_stop.resource(HtmlParseResource::SourceUnits).current, 0);
        assert_eq!(at_stop.resource(HtmlParseResource::WorkCredits).current, 0);
        released(&at_stop);
        assert!(entry.after_cleanup().is_none());
        entry.cleanup_failure().unwrap();
        assert_eq!(entry.at_stop(), Some(&at_stop));
        assert_eq!(entry.after_cleanup(), Some(&at_stop));
        assert!(entry.diagnostics().is_none());
        assert!(matches!(
            entry.take_driver(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        entry.release_failure().unwrap();
        assert_eq!(entry.retained_usage(), Some(&at_stop));
    }
    #[test]
    fn source_cap_rejection_precedes_layout_and_keeps_actual_record_visit() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("ab"),
            limits()
                .into_builder()
                .source_units(1)
                .atomic_layout_bytes(0)
                .finish()
                .unwrap(),
        );
        let mut entry = OrdinaryDocumentEntry::new(&request);
        assert_eq!(
            entry.failure(),
            Some(InitializationFailure::Stopped(ParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceUnits,
                limit: 1,
                current: 0,
                requested_delta: 2,
            }))
        );
        let at_stop = entry.at_stop().unwrap().clone();
        assert_eq!(at_stop.work(WorkCategory::RecordVisit), 1);
        assert_eq!(at_stop.resource(HtmlParseResource::SourceUnits).current, 0);
        assert_eq!(at_stop.largest_atomic_layout, 0);
        request.cancellation().cancel();
        entry.release_failure().unwrap();
        assert_eq!(entry.at_stop(), Some(&at_stop));
        assert_eq!(entry.after_cleanup(), Some(&at_stop));
        assert!(matches!(
            entry.take_driver(),
            Err(StandaloneError::Stop(ParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceUnits,
                ..
            }))
        ));
    }
    #[test]
    fn first_driver_copy_exact_and_one_short_stop_at_independently_derived_prefix() {
        // Source admission consumes one RecordVisit. The first actual driver
        // submission then copies its compiled layout and writes eleven fields.
        let bytes = size_of::<StandaloneDocumentDriver>() as u64;
        for (cap, copied, requested) in [(bytes, 0, bytes), (1 + bytes, bytes, 11)] {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8("x"),
                limits().into_builder().work_credits(cap).finish().unwrap(),
            );
            let mut entry = OrdinaryDocumentEntry::new(&request);
            assert_eq!(
                entry.failure(),
                Some(InitializationFailure::Stopped(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: cap,
                    current: 1 + copied,
                    requested_delta: requested,
                }))
            );
            let first = entry.at_stop().unwrap().clone();
            assert_eq!(first.work(WorkCategory::RecordVisit), 1);
            assert_eq!(first.work(WorkCategory::CopyByte), copied);
            assert_eq!(first.work(WorkCategory::RecordWrite), 0);
            assert_eq!(first.resource(HtmlParseResource::SourceUnits).current, 1);
            assert_eq!(first.largest_atomic_layout, bytes);
            released(&first);
            entry.release_failure().unwrap();
            assert_eq!(entry.at_stop(), Some(&first));
            assert_eq!(entry.after_cleanup(), Some(&first));
        }
    }
    #[test]
    fn completion_slot_admission_failure_keeps_pre_cleanup_allocations_in_first_snapshot() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(""),
            limits()
                .into_builder()
                .auxiliary_entries(2)
                .finish()
                .unwrap(),
        );
        let mut entry = OrdinaryDocumentEntry::new(&request);
        assert!(matches!(
            entry.failure(),
            Some(InitializationFailure::Stopped(ParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                limit: 2,
                current: 2,
                requested_delta: 1
            }))
        ));
        let first = entry.at_stop().unwrap().clone();
        // The actual Document admits its child-ticket slot and one append-field
        // index entry before the next metadata slot requests a third. The nested
        // stage constructor already cleaned its Document when new returns; the
        // controller's first latch alone retains its admitted capacity.
        assert_eq!(first.resource(HtmlParseResource::ManagedNodes).current, 1);
        assert!(first.actual_capacity_bytes > 0);
        assert!(first.resource(HtmlParseResource::AuxiliaryEntries).current > 0);
        entry.release_failure().unwrap();
        let after = entry.after_cleanup().unwrap();
        released(after);
        assert_eq!(after.resource(HtmlParseResource::ManagedNodes).current, 1);
        assert_eq!(
            after.resource(HtmlParseResource::WorkCredits),
            first.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(
            after.actual_capacity_high_water,
            first.actual_capacity_high_water
        );
        assert_eq!(entry.at_stop(), Some(&first));
    }
    #[test]
    fn cancellation_before_driver_take_retains_real_empty_evidence_then_disposes_it() {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits());
        let mut entry = OrdinaryDocumentEntry::new(&request);
        assert!(entry.failure().is_none());
        request.cancellation().cancel();
        assert!(matches!(
            entry.take_driver(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        let first = entry.at_stop().unwrap().clone();
        assert_eq!(first.resource(HtmlParseResource::ManagedNodes).current, 1);
        entry.cleanup_failure().unwrap();
        assert_eq!(entry.diagnostics().unwrap().len(), 0);
        assert_eq!(entry.at_stop(), Some(&first));
        let observation = entry.after_cleanup().unwrap().clone();
        entry.cleanup_failure().unwrap();
        assert_eq!(entry.after_cleanup(), Some(&observation));
        entry.release_failure().unwrap();
        assert!(entry.diagnostics().is_none());
        released(entry.retained_usage().unwrap());
        assert_eq!(entry.after_cleanup(), Some(&observation));
    }
    #[test]
    fn initialization_work_cuts_keep_one_stop_and_reclaim_each_real_prefix() {
        for cap in (0..20_000).step_by(127) {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8(""),
                limits().into_builder().work_credits(cap).finish().unwrap(),
            );
            let mut entry = OrdinaryDocumentEntry::new(&request);
            if let Some(first) = entry.failure() {
                let snapshot = entry.at_stop().unwrap().clone();
                assert!(matches!(
                    first,
                    InitializationFailure::Stopped(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                entry.release_failure().unwrap();
                assert_eq!(entry.failure(), Some(first));
                assert_eq!(entry.at_stop(), Some(&snapshot));
                released(entry.retained_usage().unwrap());
            } else {
                match entry.take_driver() {
                    Ok(mut driver) => {
                        if driver.release().is_err() {
                            driver.release().unwrap();
                        }
                        released(driver.control.usage());
                    }
                    Err(_) => {
                        entry.release_failure().unwrap();
                        released(entry.retained_usage().unwrap());
                    }
                }
            }
        }
    }
    #[test]
    #[should_panic(expected = "input banks are shared")]
    fn a_borrowed_test_entry_refuses_a_request_whose_inputs_it_would_skip() {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits())
            .with_encoding_url_facts(Vec::new());
        let _ = OrdinaryDocumentEntry::new(&request);
    }
    #[test]
    fn a_borrowed_test_entry_installs_the_inputs_of_a_request_shared_for_it() {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits())
            .with_encoding_url_facts(Vec::new())
            .share_associated_inputs();
        let mut entry = OrdinaryDocumentEntry::new(&request);
        assert!(entry.failure().is_none());
        let mut driver = entry.take_driver().unwrap();
        driver.release().unwrap();
        released(driver.control.usage());
    }
    #[test]
    fn a_ready_entry_whose_driver_is_gone_reports_consumed_and_the_driver_still_releases() {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits());
        let mut entry = OrdinaryDocumentEntry::new(&request);
        assert!(entry.failure().is_none());
        // Private-state fault injection: the driver is not where a ready entry
        // keeps it, so there is no controller to latch on.
        let lost = entry.driver.take().unwrap();
        assert!(matches!(
            entry.take_driver(),
            Err(StandaloneError::Consumed)
        ));
        entry.driver = Some(lost);
        let mut driver = entry.take_driver().unwrap();
        driver.release().unwrap();
        released(driver.control.usage());
    }
    #[test]
    fn a_failed_entry_whose_driver_is_gone_reports_consumed_and_the_driver_still_cleans() {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("x"), limits());
        request.cancellation().cancel();
        let mut entry = OrdinaryDocumentEntry::new(&request);
        assert!(entry.failure().is_some());
        let lost = entry.driver.take().unwrap();
        assert!(matches!(
            entry.cleanup_failure(),
            Err(StandaloneError::Consumed)
        ));
        assert!(matches!(
            entry.take_driver(),
            Err(StandaloneError::Consumed)
        ));
        entry.driver = Some(lost);
        entry.release_failure().unwrap();
        released(entry.retained_usage().unwrap());
    }
    #[test]
    fn empty_ready_entry_transfers_actual_driver_once_and_reaches_real_completion() {
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(""), limits());
        let mut entry = OrdinaryDocumentEntry::new(&request);
        let mut driver = entry.take_driver().unwrap();
        assert!(entry.retained_usage().is_none());
        assert!(matches!(
            entry.take_driver(),
            Err(StandaloneError::Consumed)
        ));
        for _ in 0..100 {
            if driver.step().unwrap() == super::super::head::HeadStep::Complete {
                driver.release().unwrap();
                released(driver.control.usage());
                return;
            }
        }
        panic!("empty ordinary document did not reach qualified completion");
    }
}
