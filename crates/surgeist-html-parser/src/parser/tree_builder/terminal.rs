//! Terminal parser owners, coherent publication and retryable cleanup.
//! Normative end-of-parsing transitions remain in the sibling EOF owner.

use super::*;
use crate::construction::StandaloneDocumentState;
use head::HeadError;

/// Actual completed construction retains the sole driver, control, arena,
/// tokenizer and all diagnostic/lexical evidence. It is not a public report.
pub(crate) struct CompletedStandaloneDocument<'source> {
    driver: Option<StandaloneDocumentDriver<'source>>,
    frozen: Option<crate::construction::FrozenArena>,
    after_cleanup: Option<super::super::control::ParseUsage>,
    #[cfg(test)]
    cancel_after_freeze: Option<crate::HtmlParseCancellation>,
}
impl<'source> StandaloneDocumentDriver<'source> {
    pub(crate) fn take_completed(
        input: &mut Option<Self>,
    ) -> Result<CompletedStandaloneDocument<'source>, StandaloneError> {
        let driver = input.as_mut().ok_or(StandaloneError::Consumed)?;
        driver.control.require_source(driver.source)?;
        let head = driver.head.as_ref().ok_or(StandaloneError::Consumed)?;
        head.require(&mut driver.control)?;
        head.state
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .qualify_completed(&mut driver.control)?;
        driver.control.charge(WorkCategory::RecordVisit, 6)?;
        if driver.prologue.is_some()
            || driver.before.is_some()
            || driver.tokenizer.is_some()
            || driver.token.is_some()
            || driver.event.is_some()
        {
            return Err(head::HeadError::IncompleteCompletion.into());
        }
        driver
            .lexical
            .as_mut()
            .ok_or(StandaloneError::Consumed)?
            .require_owner(&mut driver.control)?;
        write(
            size_of::<CompletedStandaloneDocument>()
                + size_of::<Self>()
                + size_of::<Option<Self>>(),
            4 + usize::from(cfg!(test)),
            &mut driver.control,
        )?;
        driver.control.require_source(driver.source)?;
        Ok(CompletedStandaloneDocument {
            driver: input.take(),
            frozen: None,
            after_cleanup: None,
            #[cfg(test)]
            cancel_after_freeze: None,
        })
    }
}
impl<'source> CompletedStandaloneDocument<'source> {
    pub(super) fn stop_for_public_failure(&mut self) {
        if let Some(d) = &mut self.driver {
            d.control
                .stop_unsupported_standalone(super::StandaloneUnsupported::Provenance);
        }
    }
    pub(super) fn take_public_failure(
        &mut self,
    ) -> Result<FailedStandaloneDocument<'source>, StandaloneError> {
        self.cleanup_failure()?;
        let driver = self.driver.take().ok_or(StandaloneError::Consumed)?;
        let stop = driver.control.stop().ok_or(StandaloneError::Consumed)?;
        let at_stop = driver
            .control
            .first_stop_usage()
            .ok_or(StandaloneError::Consumed)?
            .clone();
        Ok(FailedStandaloneDocument {
            _source: driver.source,
            after_cleanup: driver.control.usage().clone(),
            control: driver.control,
            stop,
            _at_stop: at_stop,
            evidence: driver.failed_evidence,
        })
    }
    /// Borrows only the proved complete immutable projection. Preparing it is
    /// a separate parse-ledger operation; this lender never resumes that work.
    #[cfg(test)]
    pub(crate) fn associations(
        &self,
    ) -> Result<crate::construction::provenance::AssociationView<'_>, StandaloneError> {
        if let Some(frozen) = &self.frozen {
            return frozen
                .retained_evidence()
                .and_then(|evidence| evidence.associations())
                .map_err(frozen_error);
        }
        self.driver
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .head
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .state
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .before
            .kernel
            .session
            .retained_evidence()
            .and_then(|evidence| evidence.associations())
            .map_err(KernelError::from)
            .map_err(HeadError::from)
            .map_err(StandaloneError::from)
    }
    pub(crate) fn prepare_native_associations(&mut self) -> Result<(), StandaloneError> {
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        driver.control.require_source(driver.source)?;
        let state = driver
            .head
            .as_mut()
            .ok_or(StandaloneError::Consumed)?
            .state
            .as_mut()
            .ok_or(StandaloneError::Consumed)?;
        state.qualify_completed(&mut driver.control)?;
        state
            .before
            .kernel
            .session
            .prepare_native_associations(&mut driver.control, driver.source)
            .map_err(KernelError::from)
            .map_err(HeadError::from)
            .map_err(StandaloneError::from)
    }
    #[cfg(test)]
    pub(crate) fn native_associations(
        &mut self,
    ) -> Result<crate::construction::provenance::NativeAssociations<'_>, StandaloneError> {
        self.prepare_native_associations()?;
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        let state = driver
            .head
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .state
            .as_ref()
            .ok_or(StandaloneError::Consumed)?;
        state
            .before
            .kernel
            .session
            .native_associations(&mut driver.control)
            .map_err(KernelError::from)
            .map_err(HeadError::from)
            .map_err(StandaloneError::from)
    }
    /// Borrows the actual completed native records and the same control. No
    /// session mutation, fixture extraction, freeze or evidence disposal occurs.
    #[cfg(test)]
    pub(crate) fn native_view(
        &mut self,
    ) -> Result<crate::construction::NativeView<'_>, StandaloneError> {
        if self.frozen.is_some() {
            return Err(StandaloneError::Consumed);
        }
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        driver.control.require_source(driver.source)?;
        let head = driver.head.as_ref().ok_or(StandaloneError::Consumed)?;
        head.require(&mut driver.control)?;
        let state = head.state.as_ref().ok_or(StandaloneError::Consumed)?;
        state.qualify_completed(&mut driver.control)?;
        state
            .before
            .kernel
            .session
            .native_view(&mut driver.control)
            .map_err(KernelError::from)
            .map_err(HeadError::from)
            .map_err(StandaloneError::from)
    }
    #[cfg(test)]
    pub(crate) fn take_diagnostics(
        &mut self,
    ) -> Result<crate::construction::evidence::FailureDiagnosticEvidence, StandaloneError> {
        if self.frozen.is_some() {
            return Err(StandaloneError::Consumed);
        }
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        driver.control.require_source(driver.source)?;
        let state = driver
            .head
            .as_mut()
            .ok_or(StandaloneError::Consumed)?
            .state
            .as_mut()
            .ok_or(StandaloneError::Consumed)?;
        state.qualify_completed(&mut driver.control)?;
        state
            .before
            .kernel
            .session
            .take_completed_evidence(&mut driver.control)
            .map_err(KernelError::from)
            .map_err(HeadError::from)
            .map_err(StandaloneError::from)
    }
    /// Fixed borrowed readback of the actual committed store, in emission order.
    /// The completed owner already qualified the sole source/session. This does
    /// not scan, allocate, mutate or resume execution authority.
    #[cfg(test)]
    pub(crate) fn diagnostics(
        &self,
    ) -> Result<crate::construction::evidence::DiagnosticEvidenceView<'_>, StandaloneError> {
        if let Some(evidence) = self
            .driver
            .as_ref()
            .and_then(|driver| driver.failed_evidence.as_ref())
        {
            return Ok(evidence.diagnostics());
        }
        if let Some(frozen) = &self.frozen {
            return frozen
                .retained_evidence()
                .map(|evidence| evidence.diagnostics())
                .map_err(frozen_error);
        }
        self.driver
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .head
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .state
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .before
            .kernel
            .session
            .retained_evidence()
            .map(|evidence| evidence.diagnostics())
            .map_err(KernelError::from)
            .map_err(HeadError::from)
            .map_err(StandaloneError::from)
    }
    #[cfg(test)]
    pub(crate) fn document_state(&mut self) -> Result<StandaloneDocumentState, StandaloneError> {
        if self.frozen.is_some() {
            return Err(StandaloneError::Consumed);
        }
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        driver.control.require_source(driver.source)?;
        let state = driver
            .head
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .state
            .as_ref()
            .ok_or(StandaloneError::Consumed)?;
        state.qualify_completed(&mut driver.control)?;
        Ok(state
            .before
            .kernel
            .session
            .standalone_document_state(&mut driver.control)
            .map_err(KernelError::from)
            .map_err(HeadError::from)?)
    }
    #[cfg(test)]
    pub(crate) fn usage(&self) -> Result<&super::super::control::ParseUsage, StandaloneError> {
        Ok(self
            .driver
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .control
            .usage())
    }
    #[cfg(test)]
    pub(crate) fn at_stop(&self) -> Option<&super::super::control::ParseUsage> {
        self.driver.as_ref()?.control.first_stop_usage()
    }
    #[cfg(test)]
    pub(crate) fn after_cleanup(&self) -> Option<&super::super::control::ParseUsage> {
        self.after_cleanup.as_ref()
    }
    /// Safe committed diagnostics remain owned separately from reclaimed arena
    /// storage. This is the current diagnostics-only provider, not native D1.
    pub(crate) fn cleanup_failure(&mut self) -> Result<(), StandaloneError> {
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        if driver.control.stop().is_none() {
            return Err(StandaloneError::Consumed);
        }
        if self.after_cleanup.is_some() {
            return Ok(());
        }
        if driver.failed_evidence.is_none() {
            if let Some(frozen) = &mut self.frozen {
                driver.failed_evidence = frozen
                    .take_stopped_evidence(&mut driver.control)
                    .map_err(frozen_error)?;
            } else if let Some(head) = &mut driver.head {
                let state = head.state.as_mut().ok_or(StandaloneError::Consumed)?;
                driver.failed_evidence = state
                    .before
                    .kernel
                    .session
                    .take_failed_evidence(&mut driver.control)
                    .map_err(frozen_error)?;
            }
        }
        if let Some(frozen) = &mut self.frozen {
            driver.reclaim_frozen_execution()?;
            frozen.release(&mut driver.control).map_err(frozen_error)?;
            self.frozen = None;
        } else {
            driver.reclaim_parse_owners()?;
        }
        self.after_cleanup = Some(driver.control.usage().clone());
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        // Borrowed release preserves the actual owner on alias/admission failure.
        // A latched stop uses the original zero-useful-work terminal cleanup.
        if driver.control.stop().is_none() {
            write(
                size_of::<Option<StandaloneDocumentDriver>>(),
                1,
                &mut driver.control,
            )?;
        }
        if let Some(frozen) = &mut self.frozen {
            if driver.control.stop().is_some() && driver.failed_evidence.is_none() {
                driver.failed_evidence = frozen
                    .take_stopped_evidence(&mut driver.control)
                    .map_err(frozen_error)?;
            }
            driver.reclaim_frozen_execution()?;
            frozen.release(&mut driver.control).map_err(frozen_error)?;
            self.frozen = None;
            if let Some(evidence) = driver.failed_evidence.take() {
                evidence.release(&mut driver.control)?;
            }
        } else {
            driver.release()?;
        }
        self.driver = None;
        Ok(())
    }
    /// Moves real completed construction into a lifetime-free private owner.
    /// A failure after Arena sharing leaves that owner in this borrowed guard.
    /// No public report, native projection or reverse-map completeness is implied.
    pub(crate) fn freeze_parts(&mut self) -> Result<FrozenStandaloneParts, StandaloneError> {
        if self.frozen.is_some() {
            let driver = self.driver.as_ref().ok_or(StandaloneError::Consumed)?;
            return Err(driver
                .control
                .stop()
                .map_or(StandaloneError::Consumed, StandaloneError::Stop));
        }
        let driver = self.driver.as_mut().ok_or(StandaloneError::Consumed)?;
        let c = &mut driver.control;
        c.require_source(driver.source)?;
        let state = driver
            .head
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .state
            .as_ref()
            .ok_or(StandaloneError::Consumed)?;
        state.qualify_completed(c)?;
        state
            .before
            .kernel
            .session
            .preflight_completed_freeze(c)
            .map_err(frozen_error)?;
        let document_state = state
            .before
            .kernel
            .session
            .standalone_document_state(c)
            .map_err(frozen_error)?;
        // Every fixed output/failure slot is submitted before active storage can
        // move. The fixed usage snapshots contain arrays, never variable clones.
        write(
            size_of::<FrozenStandaloneParts>()
                + size_of::<Option<crate::construction::FrozenArena>>()
                + size_of::<Option<StandaloneDocumentDriver>>()
                + size_of::<ParseControl>()
                + size_of::<FrozenSourceBinding>(),
            13,
            c,
        )?;
        c.charge(WorkCategory::RecordVisit, 3)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let source = FrozenSourceBinding {
            identity: driver.source.identity().clone(),
            coordinates: driver.source.coordinates(),
            length: driver.source.len(),
        };
        let frozen = driver
            .head
            .as_mut()
            .expect("qualified completed head")
            .state
            .as_mut()
            .expect("qualified completed state")
            .before
            .kernel
            .session
            .freeze(c)
            .map_err(frozen_error)?;
        // No fallible gap after the actual Arena move.
        self.frozen = Some(frozen);
        #[cfg(test)]
        if let Some(cancellation) = self.cancel_after_freeze.take() {
            cancellation.cancel();
        }
        driver.reclaim_frozen_execution()?;
        driver.control.checkpoint()?;
        let usage = driver.control.usage().clone();
        let driver = self.driver.take().expect("retained completed control");
        Ok(FrozenStandaloneParts {
            source,
            control: driver.control,
            arena: self.frozen.take(),
            document_state,
            final_usage: usage,
            failed_evidence: driver.failed_evidence,
            after_cleanup: None,
        })
    }
}

fn frozen_error(error: crate::construction::SessionError) -> StandaloneError {
    StandaloneError::Head(HeadError::Kernel(KernelError::Session(error)))
}

/// Original source metadata after all borrowed execution owners are gone. The
/// actual source backing stays with the outer consuming entry, not a copied text.
struct FrozenSourceBinding {
    identity: crate::HtmlSourceId,
    coordinates: crate::HtmlSourceCoordinates,
    length: usize,
}
pub(crate) struct FrozenStandaloneParts {
    source: FrozenSourceBinding,
    pub(super) control: ParseControl,
    pub(super) arena: Option<crate::construction::FrozenArena>,
    document_state: StandaloneDocumentState,
    final_usage: super::super::control::ParseUsage,
    failed_evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
    after_cleanup: Option<super::super::control::ParseUsage>,
}
impl FrozenStandaloneParts {
    pub(super) fn prepare_report_request(
        &mut self,
        request: &crate::request::DocumentRequestParts,
    ) -> Result<(), crate::construction::SessionError> {
        use crate::construction::SessionError;
        self.control.require_source(request.source())?;
        self.control.charge(WorkCategory::ProviderCall, 1)?;
        self.control.charge(WorkCategory::RecordVisit, 7)?;
        self.control.charge(
            WorkCategory::InspectByte,
            (2 * size_of::<crate::HtmlSourceId>()
                + 2 * size_of::<crate::HtmlSourceCoordinates>()
                + 2 * size_of::<usize>()
                + 2 * size_of::<crate::HtmlParseCancellation>()
                + size_of::<StandaloneDocumentState>()) as u64,
        )?;
        if !request.is_offline()
            || !self.control.owns_request(request.cancellation())
            || self.source_binding()
                != (
                    request.source().identity(),
                    request.source().coordinates(),
                    request.source().len(),
                )
            || self.failed_evidence.is_some()
            || self.after_cleanup.is_some()
            || self.arena.as_ref().and_then(|a| a.document()).is_none()
            || self.document_state.completion
                != crate::construction::ConstructionCompletion::Complete
        {
            return Err(SessionError::WrongToken);
        }
        Ok(())
    }
    /// Consumes only a successfully reclaimed stopped frozen owner. A cleanup
    /// refusal preserves the input slot, actual binding and safe evidence.
    pub(crate) fn detach_failure(
        input: &mut Option<Self>,
    ) -> Result<DetachedStandaloneFailure, StandaloneError> {
        let parts = input.as_mut().ok_or(StandaloneError::Consumed)?;
        parts.cleanup_failure()?;
        let stop = parts.control.stop().ok_or(StandaloneError::Consumed)?;
        if parts.control.first_stop_usage().is_none() {
            return Err(StandaloneError::Consumed);
        }
        let binding = parts.control.take_stopped_source_binding()?;
        let parts = input.take().expect("reclaimed stopped frozen owner");
        Ok(DetachedStandaloneFailure {
            binding,
            control: parts.control,
            stop,
            after_cleanup: parts.after_cleanup.expect("real frozen cleanup snapshot"),
            evidence: parts.failed_evidence,
        })
    }
    /// Admits the private source-owning immutable construction transfer. The
    /// complete lexical/D1 proof is a separate required public-assembly input.
    pub(super) fn prepare_seal(
        &mut self,
        request: &crate::HtmlDocumentParseRequest,
    ) -> Result<(), StandaloneError> {
        if request.capture().is_some() || request.document().is_some() {
            return Err(StandaloneError::UnsupportedDocumentInitialization);
        }
        self.control.charge(WorkCategory::ProviderCall, 1)?;
        if !self.control.owns_request(request.cancellation()) {
            return Err(StandaloneError::Consumed);
        }
        self.control.require_source(request.source())?;
        self.control.charge(WorkCategory::RecordVisit, 5)?;
        if self.source_binding()
            != (
                request.source().identity(),
                request.source().coordinates(),
                request.source().len(),
            )
            || self.failed_evidence.is_some()
            || self.after_cleanup.is_some()
            || self
                .arena
                .as_ref()
                .and_then(|arena| arena.document())
                .is_none()
        {
            return Err(StandaloneError::Consumed);
        }
        write(
            size_of::<super::seal::SealedOrdinaryConstruction>()
                + size_of::<Option<Self>>()
                + size_of::<Option<crate::HtmlDocumentParseRequest>>()
                + size_of::<crate::request::DocumentRequestParts>()
                + size_of::<super::super::control::ParseUsage>(),
            9,
            &mut self.control,
        )?;
        self.control.checkpoint()?;
        self.final_usage = self.control.usage().clone();
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn immutable_document(&self) -> Option<&crate::HtmlDocument> {
        self.arena.as_ref()?.document()
    }
    #[cfg(test)]
    pub(super) fn immutable_evidence(
        &self,
    ) -> Result<&crate::construction::evidence::ParseEvidence, StandaloneError> {
        self.arena
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .retained_evidence()
            .map_err(frozen_error)
    }
    /// Disposal of an already sealed success never partitions its evidence into
    /// a failed parse. Refusal preserves the same arena/evidence/ticket owner.
    #[cfg(test)]
    pub(super) fn release_sealed(&mut self) -> Result<(), StandaloneError> {
        let Some(arena) = &mut self.arena else {
            return Ok(());
        };
        arena.release(&mut self.control).map_err(frozen_error)?;
        self.arena = None;
        self.after_cleanup = Some(self.control.usage().clone());
        Ok(())
    }
    pub(crate) fn source_binding(
        &self,
    ) -> (&crate::HtmlSourceId, crate::HtmlSourceCoordinates, usize) {
        (
            &self.source.identity,
            self.source.coordinates,
            self.source.length,
        )
    }
    #[cfg(test)]
    pub(crate) fn document(&mut self) -> Result<&crate::HtmlDocument, StandaloneError> {
        self.control.checkpoint()?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        self.arena
            .as_ref()
            .and_then(|arena| arena.document())
            .ok_or(StandaloneError::Consumed)
    }
    pub(crate) fn document_state(&self) -> StandaloneDocumentState {
        self.document_state
    }
    #[cfg(test)]
    pub(crate) fn final_usage(&self) -> &super::super::control::ParseUsage {
        &self.final_usage
    }
    pub(crate) fn retained_usage(&self) -> &super::super::control::ParseUsage {
        self.control.usage()
    }
    pub(crate) fn at_stop(&self) -> Option<&super::super::control::ParseUsage> {
        self.control.first_stop_usage()
    }
    pub(crate) fn after_cleanup(&self) -> Option<&super::super::control::ParseUsage> {
        self.after_cleanup.as_ref()
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Result<crate::construction::evidence::DiagnosticEvidenceView<'_>, StandaloneError> {
        if let Some(evidence) = &self.failed_evidence {
            return Ok(evidence.diagnostics());
        }
        self.arena
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .retained_evidence()
            .map(|evidence| evidence.diagnostics())
            .map_err(frozen_error)
    }
    /// Stops already exist; retain committed diagnostics while actual frozen
    /// arena/payload storage is reclaimed. A refused alias keeps all real owners.
    pub(crate) fn cleanup_failure(&mut self) -> Result<(), StandaloneError> {
        if self.control.stop().is_none() {
            return Err(StandaloneError::Consumed);
        }
        if self.after_cleanup.is_some() {
            return Ok(());
        }
        if let Some(arena) = &mut self.arena {
            if self.failed_evidence.is_none() {
                self.failed_evidence = arena
                    .take_stopped_evidence(&mut self.control)
                    .map_err(frozen_error)?;
            }
            arena.release(&mut self.control).map_err(frozen_error)?;
            self.arena = None;
        }
        self.after_cleanup = Some(self.control.usage().clone());
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        if self.control.stop().is_some() {
            self.cleanup_failure()?;
        } else if let Some(arena) = &mut self.arena {
            arena.release(&mut self.control).map_err(frozen_error)?;
            self.arena = None;
            self.after_cleanup = Some(self.control.usage().clone());
        }
        if let Some(evidence) = self.failed_evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
}

/// Private assembler input. No tree, native references, effects or uncommitted
/// token drafts survive successful extraction. Source ownership stays with the
/// borrowed request; the later consuming public entry must move that handle.
pub(crate) struct FailedStandaloneDocument<'source> {
    _source: &'source crate::HtmlSource,
    control: ParseControl,
    stop: ParseStop,
    _at_stop: super::super::control::ParseUsage,
    after_cleanup: super::super::control::ParseUsage,
    evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
}
/// Lifetime-free stopped owner after all borrowed parse owners were reclaimed.
/// Its binding is moved from the actual controller, never cloned after stop.
pub(crate) struct DetachedStandaloneFailure {
    pub(super) binding: (crate::HtmlSourceId, crate::HtmlSourceCoordinates),
    control: ParseControl,
    stop: ParseStop,
    after_cleanup: super::super::control::ParseUsage,
    evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
}
impl DetachedStandaloneFailure {
    pub(crate) fn release_csp_storage(&mut self, storage: crate::meta_csp::Storage) {
        storage.release(&mut self.control);
    }
    pub(super) fn owns_request(&self, cancellation: &crate::HtmlParseCancellation) -> bool {
        self.control.owns_request(cancellation)
    }
    pub(super) fn stop(&self) -> ParseStop {
        self.stop
    }
    pub(super) fn at_stop(&self) -> &super::super::control::ParseUsage {
        self.control
            .first_stop_usage()
            .expect("qualified canonical first stop")
    }
    pub(super) fn after_cleanup(&self) -> &super::super::control::ParseUsage {
        &self.after_cleanup
    }
    pub(super) fn retained_usage(&self) -> &super::super::control::ParseUsage {
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
impl<'source> StandaloneDocumentDriver<'source> {
    pub(crate) fn take_failed(
        input: &mut Option<Self>,
    ) -> Result<FailedStandaloneDocument<'source>, StandaloneError> {
        let driver = input.as_mut().ok_or(StandaloneError::Consumed)?;
        let stop = driver.control.stop().ok_or(StandaloneError::Consumed)?;
        // Fixed snapshots and moves use the envelope admitted before first pull.
        // No checkpoint, allocation, variable copy or error append follows stop.
        if driver.failure_usage.is_none() {
            driver.failure_usage = Some(
                driver
                    .control
                    .first_stop_usage()
                    .expect("canonical first-stop snapshot")
                    .clone(),
            );
            let session = if let Some(head) = &mut driver.head {
                &mut head
                    .state
                    .as_mut()
                    .ok_or(StandaloneError::Consumed)?
                    .before
                    .kernel
                    .session
            } else if let Some(before) = &mut driver.before {
                &mut before.kernel.session
            } else {
                &mut driver
                    .prologue
                    .as_mut()
                    .ok_or(StandaloneError::Consumed)?
                    .kernel
                    .as_mut()
                    .ok_or(StandaloneError::Consumed)?
                    .session
            };
            driver.failed_evidence = session
                .take_failed_evidence(&mut driver.control)
                .map_err(KernelError::from)
                .map_err(HeadError::from)?;
        }
        // A cleanup refusal keeps the driver and detached committed evidence in
        // the caller's original owning slot. A retry cannot lose or re-copy it.
        driver.reclaim_parse_owners()?;
        let driver = input.take().expect("stopped actual owner reclaimed");
        Ok(FailedStandaloneDocument {
            _source: driver.source,
            after_cleanup: driver.control.usage().clone(),
            control: driver.control,
            stop,
            _at_stop: driver.failure_usage.expect("prepared fixed snapshot"),
            evidence: driver.failed_evidence,
        })
    }
}
impl FailedStandaloneDocument<'_> {
    /// The constructor of this owner already completed cleanup before moving
    /// out the driver. All storage left is committed diagnostic retention.
    pub(crate) fn detach(
        input: &mut Option<Self>,
    ) -> Result<DetachedStandaloneFailure, StandaloneError> {
        let failed = input.as_mut().ok_or(StandaloneError::Consumed)?;
        let binding = failed.control.take_stopped_source_binding()?;
        let failed = input.take().expect("cleaned stopped original owner");
        Ok(DetachedStandaloneFailure {
            binding,
            control: failed.control,
            stop: failed.stop,
            after_cleanup: failed.after_cleanup,
            evidence: failed.evidence,
        })
    }
    #[cfg(test)]
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self._source
    }
    #[cfg(test)]
    pub(crate) fn stop(&self) -> ParseStop {
        self.stop
    }
    #[cfg(test)]
    pub(crate) fn at_stop(&self) -> &super::super::control::ParseUsage {
        &self._at_stop
    }
    #[cfg(test)]
    pub(crate) fn after_cleanup(&self) -> &super::super::control::ParseUsage {
        &self.after_cleanup
    }
    #[cfg(test)]
    pub(crate) fn retained_usage(&self) -> &super::super::control::ParseUsage {
        self.control.usage()
    }
    #[cfg(test)]
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::evidence::DiagnosticEvidenceView<'_>> {
        self.evidence
            .as_ref()
            .map(|evidence| evidence.diagnostics())
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        if let Some(evidence) = self.evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
