//! HTML 877d §13.2.7, reduced by the closed fresh ordinary operation.
//! Readiness, canonical pops and both unconditional spin checkpoints are real.
//! External work is eliminated from the admitted producer inventory, not from
//! a Disabled label or an empty task queue. No public parse report is produced.

use super::*;
use crate::construction::parser_session::{ConstructionReadiness, StandaloneDocumentState};
use head::{HeadError, HeadState, HeadStep};
use prologue::Payload;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EndPhase {
    InsertionPoint,
    Interactive,
    PopOpen,
    DeferredScriptsAbsent,
    DomContentLoadedDisposition,
    ScriptsCheckpoint,
    LoadBlockersCheckpoint,
    RetainCompleteContinuation,
    PrintDisposition,
    PostLoadMilestone,
    CompleteReadiness,
    NullBrowsingContextTail,
    VerifyComplete,
    Done,
}
impl HeadState<'_> {
    fn set_end(&mut self, phase: EndPhase, c: &mut ParseControl) -> Result<(), HeadError> {
        write(size_of::<Option<EndPhase>>(), 1, c)?;
        self.end = Some(phase);
        Ok(())
    }
    pub(super) fn begin_end(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        self.qualify_end_script(c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        if self.end.is_some()
            || !matches!(
                self.pending.as_ref().map(|p| &p.payload),
                Some(Payload::Eof)
            )
        {
            return Err(HeadError::IncompleteCompletion);
        }
        let script = self
            .script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?;
        self.before
            .kernel
            .session
            .qualify_standalone_completion(c, &script.capability, false)?;
        self.set_end(EndPhase::InsertionPoint, c)?;
        Ok(HeadStep::Consumed)
    }
    /// One fixed phase or one canonical pop per step. Long traversals and tasks
    /// retain their own actual bounded control admissions; no bulk stack clear.
    pub(super) fn end_step(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        self.qualify_end_script(c)?;
        c.charge(WorkCategory::StateStep, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            size_of::<Option<EndPhase>>() as u64,
        )?;
        let phase = self.end.ok_or(HeadError::IncompleteCompletion)?;
        match phase {
            EndPhase::InsertionPoint => {
                self.end_insertion_point(c)?;
                self.set_end(EndPhase::Interactive, c)?;
            }
            EndPhase::Interactive => {
                self.before
                    .kernel
                    .session
                    .update_standalone_readiness(c, ConstructionReadiness::Interactive)?;
                self.set_end(EndPhase::PopOpen, c)?;
            }
            EndPhase::PopOpen => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.before.kernel.stack()?.len() == 0 {
                    self.set_end(EndPhase::DeferredScriptsAbsent, c)?;
                } else {
                    let script = self.script.as_ref().expect("qualified script state");
                    self.before
                        .kernel
                        .pop(c, Some(&script.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                }
            }
            EndPhase::DeferredScriptsAbsent => {
                // HTML preparation stops at step18, SVG at its separately
                // selected no-loader/execution/recipient boundary. These actual
                // constructors cannot enqueue an executable deferred script.
                self.closed_end_producers(c)?;
                self.set_end(EndPhase::DomContentLoadedDisposition, c)?;
            }
            EndPhase::DomContentLoadedDisposition => {
                // Step6 has no admitted global/settings/clock/recipient/client/
                // BiDi domain or retained external consequence. Do not enqueue
                // a pretend DOMContentLoaded task or claim an event was fired.
                self.closed_end_producers(c)?;
                self.set_end(EndPhase::ScriptsCheckpoint, c)?;
            }
            EndPhase::ScriptsCheckpoint => {
                let script = self.script.as_ref().expect("qualified script state");
                script
                    .capability
                    .checkpoint(c, &mut self.before.kernel.session)?;
                self.closed_end_producers(c)?;
                self.set_end(EndPhase::LoadBlockersCheckpoint, c)?;
            }
            EndPhase::LoadBlockersCheckpoint => {
                // Spin performs its checkpoint even with a satisfied wait goal.
                // No actual loader/resource/browsing-context producer is admitted.
                let script = self.script.as_ref().expect("qualified script state");
                script
                    .capability
                    .checkpoint(c, &mut self.before.kernel.session)?;
                self.closed_end_producers(c)?;
                self.set_end(EndPhase::RetainCompleteContinuation, c)?;
            }
            EndPhase::RetainCompleteContinuation => {
                // The fixed phase below retains only step9's intrinsic readiness
                // continuation. There is no global queue or lifecycle journal.
                self.set_end(EndPhase::PrintDisposition, c)?;
            }
            EndPhase::PrintDisposition => {
                self.closed_end_producers(c)?;
                self.set_end(EndPhase::PostLoadMilestone, c)?;
            }
            EndPhase::PostLoadMilestone => {
                self.before.kernel.session.mark_standalone_post_load(c)?;
                self.set_end(EndPhase::CompleteReadiness, c)?;
            }
            EndPhase::CompleteReadiness => {
                self.before
                    .kernel
                    .session
                    .update_standalone_readiness(c, ConstructionReadiness::Complete)?;
                self.set_end(EndPhase::NullBrowsingContextTail, c)?;
            }
            EndPhase::NullBrowsingContextTail => {
                self.closed_end_producers(c)?;
                self.set_end(EndPhase::VerifyComplete, c)?;
            }
            EndPhase::VerifyComplete => {
                self.closed_end_producers(c)?;
                c.charge(WorkCategory::RecordVisit, 2)?;
                if self.before.kernel.stack()?.len() != 0
                    || !matches!(
                        self.pending.as_ref().map(|p| &p.payload),
                        Some(Payload::Eof)
                    )
                {
                    return Err(HeadError::IncompleteCompletion);
                }
                self.before
                    .kernel
                    .session
                    .require_standalone_effects_finished(c)?;
                // Admit final cursor/token writes before marking completion. No
                // fallible gap follows that stamp or can orphan the EOF anchor.
                write(
                    size_of::<Option<EndPhase>>() + size_of::<Option<prologue::PendingToken>>(),
                    2,
                    c,
                )?;
                self.before.kernel.session.mark_standalone_complete(c)?;
                self.pending = None;
                self.end = Some(EndPhase::Done);
                return Ok(HeadStep::Complete);
            }
            EndPhase::Done => {
                self.qualify_completed(c)?;
                return Ok(HeadStep::Complete);
            }
        }
        Ok(HeadStep::Consumed)
    }
    fn closed_end_producers(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.qualify_end_script(c)?;
        // This proof is issued only from the actual constructor that rejects
        // captured documents, Enabled, declarative shadow and imported backing.
        // New loader/host/realm/task producers must revise this closed boundary.
        let script = self.script.as_ref().expect("qualified script state");
        script
            .capability
            .validate_session(c, &self.before.kernel.session)?;
        Ok(())
    }
    fn qualify_completed(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.qualify_end_script(c)?;
        c.charge(WorkCategory::RecordVisit, 3)?;
        if self.end != Some(EndPhase::Done)
            || self.pending.is_some()
            || self.before.kernel.stack()?.len() != 0
        {
            return Err(HeadError::IncompleteCompletion);
        }
        let script = self.script.as_ref().expect("qualified script state");
        self.before
            .kernel
            .session
            .qualify_standalone_completion(c, &script.capability, true)?;
        Ok(())
    }
}

/// Actual completed construction retains the sole driver, control, arena,
/// tokenizer and all diagnostic/lexical evidence. It is not a public report.
pub(crate) struct CompletedStandaloneDocument<'source> {
    driver: Option<StandaloneDocumentDriver<'source>>,
    frozen: Option<crate::construction::parser_session::FrozenArena>,
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
impl CompletedStandaloneDocument<'_> {
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
    pub(crate) fn native_associations(
        &mut self,
    ) -> Result<
        crate::construction::parser_session::provenance::NativeAssociations<'_>,
        StandaloneError,
    > {
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
    pub(crate) fn native_view(
        &mut self,
    ) -> Result<crate::construction::parser_session::NativeView<'_>, StandaloneError> {
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
    pub(crate) fn take_diagnostics(
        &mut self,
    ) -> Result<
        crate::construction::parser_session::evidence::FailureDiagnosticEvidence,
        StandaloneError,
    > {
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
    pub(crate) fn diagnostics(
        &self,
    ) -> Result<
        crate::construction::parser_session::evidence::DiagnosticEvidenceView<'_>,
        StandaloneError,
    > {
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
    pub(crate) fn usage(&self) -> Result<&super::super::control::ParseUsage, StandaloneError> {
        Ok(self
            .driver
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
            .control
            .usage())
    }
    pub(crate) fn at_stop(&self) -> Option<&super::super::control::ParseUsage> {
        self.driver.as_ref()?.control.first_stop_usage()
    }
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
                + size_of::<Option<crate::construction::parser_session::FrozenArena>>()
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

fn frozen_error(error: crate::construction::parser_session::SessionError) -> StandaloneError {
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
    control: ParseControl,
    arena: Option<crate::construction::parser_session::FrozenArena>,
    document_state: StandaloneDocumentState,
    final_usage: super::super::control::ParseUsage,
    failed_evidence:
        Option<crate::construction::parser_session::evidence::FailureDiagnosticEvidence>,
    after_cleanup: Option<super::super::control::ParseUsage>,
}
impl FrozenStandaloneParts {
    pub(crate) fn source_binding(
        &self,
    ) -> (&crate::HtmlSourceId, crate::HtmlSourceCoordinates, usize) {
        (
            &self.source.identity,
            self.source.coordinates,
            self.source.length,
        )
    }
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
    ) -> Result<
        crate::construction::parser_session::evidence::DiagnosticEvidenceView<'_>,
        StandaloneError,
    > {
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
    source: &'source crate::HtmlSource,
    control: ParseControl,
    stop: ParseStop,
    at_stop: super::super::control::ParseUsage,
    after_cleanup: super::super::control::ParseUsage,
    evidence: Option<crate::construction::parser_session::evidence::FailureDiagnosticEvidence>,
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
            source: driver.source,
            after_cleanup: driver.control.usage().clone(),
            control: driver.control,
            stop,
            at_stop: driver.failure_usage.expect("prepared fixed snapshot"),
            evidence: driver.failed_evidence,
        })
    }
}
impl FailedStandaloneDocument<'_> {
    pub(crate) fn source(&self) -> &crate::HtmlSource {
        self.source
    }
    pub(crate) fn stop(&self) -> ParseStop {
        self.stop
    }
    pub(crate) fn at_stop(&self) -> &super::super::control::ParseUsage {
        &self.at_stop
    }
    pub(crate) fn after_cleanup(&self) -> &super::super::control::ParseUsage {
        &self.after_cleanup
    }
    pub(crate) fn retained_usage(&self) -> &super::super::control::ParseUsage {
        self.control.usage()
    }
    pub(crate) fn diagnostics(
        &self,
    ) -> Option<crate::construction::parser_session::evidence::DiagnosticEvidenceView<'_>> {
        self.evidence
            .as_ref()
            .map(|evidence| evidence.diagnostics())
    }
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        if let Some(evidence) = self.evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::parser_session::ConstructionCompletion;
    use crate::construction::parser_session::native_view::{
        NativeFragmentOwner, NativeKind, NativeString, NativeView,
    };
    use crate::{HtmlDocumentParseRequest, HtmlParseResource, HtmlSource};

    fn frozen_completed(request: &HtmlDocumentParseRequest) -> CompletedStandaloneDocument<'_> {
        let mut entry = super::super::entry::OrdinaryDocumentEntry::new(request);
        let mut driver = Some(entry.take_driver().unwrap());
        for _ in 0..4096 {
            if driver.as_mut().unwrap().step().unwrap() == HeadStep::Complete {
                return StandaloneDocumentDriver::take_completed(&mut driver).unwrap();
            }
        }
        panic!("real EOF was not reached");
    }
    fn native_child<'a>(
        document: &'a crate::HtmlDocument,
        parent: crate::HtmlNode<'a>,
        local: &str,
    ) -> crate::HtmlNode<'a> {
        for id in parent.children() {
            let node = document.node(&id).unwrap();
            if let crate::HtmlNodeKind::Element(element) = node.kind()
                && element.name().local_name().code_units()
                    == local.encode_utf16().collect::<Vec<_>>()
            {
                return node;
            }
        }
        panic!("missing literal child {local}");
    }
    #[test]
    fn frozen_parts_end_borrow_and_retain_same_source_native_identity_and_completion() {
        fn produce() -> (FrozenStandaloneParts, crate::HtmlSourceId) {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8("<!doctype html><p>x"),
                super::super::tests::limits(100),
            );
            let source = request.source().identity().clone();
            let mut completed = frozen_completed(&request);
            let parts = completed.freeze_parts().unwrap();
            assert!(completed.driver.is_none());
            (parts, source)
        }
        let (mut parts, source) = produce();
        assert_eq!(
            parts.source_binding(),
            (&source, crate::HtmlSourceCoordinates::Utf8Bytes, 19)
        );
        assert_eq!(
            parts.document_state().completion,
            ConstructionCompletion::Complete
        );
        assert!(parts.final_usage().actual_capacity_bytes > 0);
        assert!(
            parts
                .retained_usage()
                .resource(HtmlParseResource::ManagedUtf16Units)
                .current
                > 0
        );
        let document = parts.document().unwrap();
        let html = native_child(document, document.root(), "html");
        let body = native_child(document, html, "body");
        let p = native_child(document, body, "p");
        let text = document.node(&p.children().next().unwrap()).unwrap();
        assert!(
            matches!(text.kind(), crate::HtmlNodeKind::Text(value) if value.code_units()==[120])
        );
        parts.release().unwrap();
        assert!(parts.after_cleanup().is_some());
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_parts_exact_utf16_recovery_and_committed_diagnostic_owner_survive_move() {
        let mut source = "<!doctype html><p>".encode_utf16().collect::<Vec<_>>();
        source.extend([0xd83d, 0xde00, 0xd800, 13, 10, 0, 38, 97, 109, 112, 59]);
        let length = source.len();
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(crate::DocumentString::from_utf16(source)),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let pointer = completed.diagnostics().unwrap().records_pointer();
        let count = completed.diagnostics().unwrap().len();
        let mut parts = completed.freeze_parts().unwrap();
        assert_eq!(
            parts.source_binding(),
            (
                request.source().identity(),
                crate::HtmlSourceCoordinates::Utf16CodeUnits,
                length
            )
        );
        assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
        assert_eq!(parts.diagnostics().unwrap().len(), count);
        assert!(count > 0);
        let document = parts.document().unwrap();
        let html = native_child(document, document.root(), "html");
        let body = native_child(document, html, "body");
        let p = native_child(document, body, "p");
        let text = document.node(&p.children().next().unwrap()).unwrap();
        // InBody ignores NULL; CRLF preprocesses to LF and &amp; contributes &.
        assert!(
            matches!(text.kind(),crate::HtmlNodeKind::Text(value) if value.code_units()==[0xd83d,0xde00,0xd800,10,38])
        );
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_parts_real_arena_alias_retains_failure_diagnostics_and_retries_cleanup() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>\0"),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let mut parts = completed.freeze_parts().unwrap();
        let pointer = parts.diagnostics().unwrap().records_pointer();
        let count = parts.diagnostics().unwrap().len();
        let alias = parts.document().unwrap().clone();
        assert!(parts.release().is_err());
        assert!(parts.arena.is_some());
        let first = parts.at_stop().unwrap().clone();
        assert!(parts.document().is_err());
        assert!(parts.cleanup_failure().is_err());
        assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
        assert_eq!(parts.diagnostics().unwrap().len(), count);
        drop(alias);
        parts.cleanup_failure().unwrap();
        assert!(parts.arena.is_none());
        assert_eq!(parts.at_stop(), Some(&first));
        let after = parts.after_cleanup().unwrap().clone();
        assert_eq!(
            after.resource(HtmlParseResource::Diagnostics).current,
            count as u64
        );
        parts.release().unwrap();
        assert_eq!(parts.after_cleanup(), Some(&after));
        assert_eq!(parts.at_stop(), Some(&first));
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_native_projection_cancellation_partitions_only_committed_diagnostics() {
        let source = HtmlSource::utf8("<p>A🙂\r\n&NotEqualTilde;\0");
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut completed = frozen_completed(&request);
        assert!(completed.native_associations().unwrap().len() > 0);
        let pointer = completed.diagnostics().unwrap().records_pointer();
        let count = completed.diagnostics().unwrap().len();
        assert_eq!(count, 3); // missing doctype, tokenizer null, and InBody null
        let mut parts = completed.freeze_parts().unwrap();
        assert!(
            parts
                .retained_usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current
                > count as u64
        );
        let before = parts.retained_usage().clone();
        {
            let diagnostics = parts.diagnostics().unwrap();
            assert_eq!(diagnostics.records_pointer(), pointer);
            let (record, location) = diagnostics.retained_record(2).unwrap();
            assert_eq!(
                record.kind,
                crate::construction::parser_session::evidence::DiagnosticKind::Tree(
                    super::super::head::HeadCondition::NullInBody
                )
            );
            assert_eq!(location.source, source.range(25, 26).unwrap());
        }
        assert_eq!(parts.retained_usage(), &before); // fixed lender performs no admitted work
        request.cancellation().cancel();
        assert!(matches!(
            parts.document(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        let first = parts.at_stop().unwrap().clone();
        let work = parts
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits);
        let polls = parts.control.owned_span_observation().polls;
        parts.cleanup_failure().unwrap();
        assert!(parts.arena.is_none());
        assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
        assert_eq!(
            parts
                .retained_usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            count as u64
        );
        assert_eq!(
            parts
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(parts.control.owned_span_observation().polls, polls);
        assert_eq!(parts.at_stop(), Some(&first));
        parts.release().unwrap();
        assert_eq!(parts.at_stop(), Some(&first));
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_native_projection_alias_refusal_keeps_partition_and_reclaims_it_once() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(crate::DocumentString::from_utf16([60, 112, 62, 65, 0])),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        assert!(completed.native_associations().unwrap().len() > 0);
        let mut parts = completed.freeze_parts().unwrap();
        let pointer = parts.diagnostics().unwrap().records_pointer();
        let count = parts.diagnostics().unwrap().len();
        assert_eq!(count, 3);
        let alias = parts.document().unwrap().clone();
        assert!(parts.release().is_err());
        let first = parts.at_stop().unwrap().clone();
        let work = parts
            .retained_usage()
            .resource(HtmlParseResource::WorkCredits);
        assert!(parts.cleanup_failure().is_err());
        assert!(parts.arena.is_some());
        assert!(parts.failed_evidence.is_some());
        assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
        assert_eq!(
            parts
                .retained_usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            count as u64
        );
        let retained = parts.retained_usage().clone();
        assert!(parts.cleanup_failure().is_err());
        assert_eq!(parts.retained_usage(), &retained);
        drop(alias);
        parts.cleanup_failure().unwrap();
        assert_eq!(
            parts
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits),
            work
        );
        assert_eq!(parts.at_stop(), Some(&first));
        assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
        let diagnostics = parts.diagnostics().unwrap();
        let (record, location) = diagnostics.retained_record(2).unwrap();
        assert_eq!(
            record.kind,
            crate::construction::parser_session::evidence::DiagnosticKind::Tree(
                super::super::head::HeadCondition::NullInBody
            )
        );
        assert_eq!(location.source, request.source().range(4, 5).unwrap());
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_parts_active_payload_alias_rejects_before_freeze_and_preserves_completed_owner() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>x"),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let html = {
            let mut view = completed.native_view().unwrap();
            let root = view.root().unwrap();
            child_named(&mut view, &root, "html")
        };
        let driver = completed.driver.as_mut().unwrap();
        let alias = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .session
            .element_name(&mut driver.control, &html)
            .unwrap()
            .unwrap()
            .local_name()
            .clone();
        assert!(matches!(
            completed.freeze_parts(),
            Err(StandaloneError::Head(HeadError::Kernel(
                KernelError::Session(SessionError::AliasedPayload)
            )))
        ));
        assert!(completed.frozen.is_none());
        assert!(completed.driver.as_ref().unwrap().control.stop().is_none());
        assert!(matches!(
            completed.native_view(),
            Err(StandaloneError::Head(HeadError::Kernel(
                KernelError::Session(SessionError::AliasedPayload)
            )))
        ));
        drop(alias);
        completed.native_view().unwrap();
        let mut parts = completed.freeze_parts().unwrap();
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_parts_missing_transferred_evidence_cannot_invent_an_empty_store() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>\0"),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let evidence = completed.take_diagnostics().unwrap();
        let pointer = evidence.records_pointer();
        let count = evidence.len();
        assert!(count > 0);
        assert!(matches!(
            completed.freeze_parts(),
            Err(StandaloneError::Head(HeadError::Kernel(
                KernelError::Session(SessionError::WrongToken)
            )))
        ));
        assert!(completed.frozen.is_none());
        assert!(completed.driver.as_ref().unwrap().control.stop().is_none());
        completed.native_view().unwrap();
        let driver = completed.driver.as_mut().unwrap();
        assert_eq!(evidence.records_pointer(), pointer);
        assert_eq!(evidence.len(), count);
        evidence.release(&mut driver.control).unwrap();
        completed.release().unwrap();
        assert!(completed.driver.is_none());
    }
    #[test]
    fn frozen_parts_cancellation_after_transfer_keeps_safe_evidence_and_first_snapshot() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>\0"),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let mut parts = completed.freeze_parts().unwrap();
        request.cancellation().cancel();
        assert!(matches!(
            parts.document(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        let first = parts.at_stop().unwrap().clone();
        let pointer = parts.diagnostics().unwrap().records_pointer();
        let polls = parts.control.owned_span_observation().polls;
        parts.cleanup_failure().unwrap();
        assert_eq!(parts.control.owned_span_observation().polls, polls);
        assert_eq!(parts.at_stop(), Some(&first));
        assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_parts_actual_after_share_cancellation_keeps_both_owners_for_terminal_release() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>\0"),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let pointer = completed.diagnostics().unwrap().records_pointer();
        completed.cancel_after_freeze = Some(request.cancellation().clone());
        assert!(matches!(
            completed.freeze_parts(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        assert!(completed.frozen.is_some());
        assert!(completed.driver.is_some());
        assert_eq!(completed.diagnostics().unwrap().records_pointer(), pointer);
        assert!(completed.native_view().is_err());
        assert!(completed.freeze_parts().is_err());
        let first = completed
            .driver
            .as_ref()
            .unwrap()
            .control
            .first_stop_usage()
            .unwrap()
            .clone();
        let polls = completed
            .driver
            .as_ref()
            .unwrap()
            .control
            .owned_span_observation()
            .polls;
        completed.cleanup_failure().unwrap();
        assert_eq!(completed.at_stop(), Some(&first));
        assert!(
            completed.after_cleanup().unwrap().actual_capacity_bytes < first.actual_capacity_bytes
        );
        assert_eq!(
            completed
                .driver
                .as_ref()
                .unwrap()
                .control
                .owned_span_observation()
                .polls,
            polls
        );
        assert_eq!(completed.diagnostics().unwrap().records_pointer(), pointer);
        assert!(completed.frozen.is_none());
        completed.release().unwrap();
        assert!(completed.driver.is_none());
        assert!(completed.frozen.is_none());
        assert!(first.actual_capacity_bytes > 0);
    }
    #[test]
    fn frozen_parts_exact_work_and_one_short_plus_bounded_cuts_preserve_actual_owners() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>\0"),
            super::super::tests::limits(100),
        );
        let mut measured = frozen_completed(&request);
        let before = measured
            .usage()
            .unwrap()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut result = measured.freeze_parts().unwrap();
        let cost = result
            .final_usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        result.release().unwrap();
        assert!(cost > 0);
        let mut active_stops = 0;
        let mut frozen_stops = 0;
        for allowance in (0..cost).step_by(97).chain([cost - 1, cost]) {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8("<p>\0"),
                super::super::tests::limits(100),
            );
            let mut completed = frozen_completed(&request);
            let pointer = completed.diagnostics().unwrap().records_pointer();
            let count = completed.diagnostics().unwrap().len();
            let driver = completed.driver.as_mut().unwrap();
            let used = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            driver
                .control
                .charge(
                    WorkCategory::StateStep,
                    request.limits().work_credits() - used - allowance,
                )
                .unwrap();
            match completed.freeze_parts() {
                Ok(mut parts) => {
                    assert_eq!(allowance, cost, "only exact complete allowance succeeds");
                    assert_eq!(
                        parts
                            .final_usage()
                            .resource(HtmlParseResource::WorkCredits)
                            .current,
                        request.limits().work_credits()
                    );
                    if parts.release().is_err() {
                        parts.release().unwrap();
                    }
                    super::super::tests::assert_released(&parts.control);
                }
                Err(error) => {
                    assert!(allowance < cost);
                    let driver = completed.driver.as_ref().unwrap();
                    assert!(
                        matches!(
                            driver.control.stop(),
                            Some(ParseStop::LimitExceeded {
                                resource: HtmlParseResource::WorkCredits,
                                ..
                            })
                        ),
                        "{error:?}"
                    );
                    let stop = driver.control.stop();
                    let first = driver.control.first_stop_usage().unwrap().clone();
                    let polls = driver.control.owned_span_observation().polls;
                    if completed.frozen.is_some() {
                        frozen_stops += 1;
                    } else {
                        active_stops += 1;
                    }
                    assert!(completed.freeze_parts().is_err());
                    completed.cleanup_failure().unwrap();
                    assert_eq!(completed.diagnostics().unwrap().records_pointer(), pointer);
                    assert_eq!(completed.diagnostics().unwrap().len(), count);
                    assert_eq!(completed.driver.as_ref().unwrap().control.stop(), stop);
                    assert_eq!(completed.at_stop(), Some(&first));
                    assert_eq!(
                        completed
                            .driver
                            .as_ref()
                            .unwrap()
                            .control
                            .owned_span_observation()
                            .polls,
                        polls
                    );
                    assert!(
                        completed.after_cleanup().unwrap().actual_capacity_bytes
                            <= first.actual_capacity_bytes
                    );
                    completed.release().unwrap();
                    assert!(completed.driver.is_none());
                }
            }
        }
        assert!(active_stops > 0);
        assert!(frozen_stops > 0);
    }
    #[test]
    fn frozen_parts_retains_template_control_and_script_metadata_in_same_arena() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><template><p>T</template><script>x</script><textarea>\nX</textarea><input type=hidden value=y><select><option>A</select>",
            ),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let script = {
            let mut view = completed.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let head = child_named(&mut view, &html, "head");
            child_named(&mut view, &head, "script")
        };
        let mut parts = completed.freeze_parts().unwrap();
        let document = parts.document().unwrap();
        let html = native_child(document, document.root(), "html");
        let head = native_child(document, html, "head");
        let template = native_child(document, head, "template");
        let contents = document
            .fragment(&template.template_contents().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(contents.root().children().len(), 1);
        let body = native_child(document, html, "body");
        let textarea = native_child(document, body, "textarea").id();
        let hidden = native_child(document, body, "input").id();
        drop(contents);
        let arena = parts.arena.as_ref().unwrap();
        assert!(
            arena
                .script_metadata(&mut parts.control, &script)
                .unwrap()
                .already_started
        );
        assert_eq!(
            arena
                .textarea_raw_value(&mut parts.control, &textarea)
                .unwrap()
                .code_units(),
            [88]
        );
        arena
            .hidden_input_state(&mut parts.control, &hidden)
            .unwrap();
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }
    #[test]
    fn frozen_parts_stopped_evidence_foreign_owner_refusal_preserves_actual_store() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>\0"),
            super::super::tests::limits(100),
        );
        let mut completed = frozen_completed(&request);
        let mut parts = completed.freeze_parts().unwrap();
        let pointer = parts.diagnostics().unwrap().records_pointer();
        let mut foreign = ParseControl::new(
            super::super::tests::limits(100),
            crate::HtmlParseCancellation::new(),
        );
        foreign.admit_source(request.source()).unwrap();
        foreign.stop_unsupported_standalone(
            super::super::super::control::StandaloneUnsupported::Completion,
        );
        let before = foreign.usage().clone();
        assert!(
            parts
                .arena
                .as_mut()
                .unwrap()
                .take_stopped_evidence(&mut foreign)
                .is_err()
        );
        assert_eq!(foreign.usage(), &before);
        assert_eq!(parts.diagnostics().unwrap().records_pointer(), pointer);
        assert!(parts.control.stop().is_none());
        parts.release().unwrap();
        super::super::tests::assert_released(&parts.control);
    }

    #[test]
    fn completed_native_field_projection_retains_atomic_merged_origins_and_later_trigger() {
        use crate::construction::parser_session::provenance::ConstructionOperation;
        use crate::parser::tokenizer::token::OriginKind;
        let source = HtmlSource::utf8("<p>A🙂\r\n&NotEqualTilde;");
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut driver = super::super::entry::OrdinaryDocumentEntry::new(&request)
            .take_driver()
            .unwrap();
        finish(&mut driver);
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        let (text, root) = {
            let mut view = completed.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let p = child_named(&mut view, &body, "p");
            let text = view.child_at(&p, 0).unwrap().unwrap();
            let value = match view.kind(&text).unwrap() {
                NativeKind::Text(value) => value,
                _ => panic!("native Text"),
            };
            assert_eq!(
                units(&mut view, value),
                [65, 0xd83d, 0xde42, 10, 0x2242, 0x338]
            );
            (text, root)
        };
        let mut rows = completed.native_associations().unwrap();
        let mut observed = Vec::new();
        let mut roots = 0;
        for index in 0..rows.len() {
            if rows.node_for(index, &root).unwrap() {
                let (_, cause) = rows.row(index).unwrap().unwrap();
                assert_eq!(cause.operation, ConstructionOperation::Request);
                assert!(cause.source.is_none());
                assert!(cause.captured.is_none());
                assert!(cause.original_node.is_none());
                roots += 1;
            }
            if rows.text_for(index, &text).unwrap() {
                let reference = rows.cause_reference(index).unwrap().unwrap();
                let cause = rows.cause(&reference).unwrap();
                assert_eq!(cause.operation, ConstructionOperation::Merge);
                assert!(cause.original.is_some());
                assert!(cause.incoming.is_some());
                assert!(cause.trigger.is_some());
                assert!(cause.source.is_none());
                assert!(cause.rule.is_none());
                assert!(cause.family.is_none());
                let original = cause.original.unwrap();
                let trigger = cause.trigger.unwrap();
                let (group, origins) = rows.group(original).unwrap().unwrap();
                assert!(!origins.is_empty());
                let retirement = group.retirement.expect("actual replaced field retained");
                let retired = rows.cause_at(retirement).unwrap().unwrap();
                assert_eq!(retired.operation, ConstructionOperation::Retirement);
                assert_eq!(retired.original, Some(original));
                assert!(matches!(
                    retired.retired_destination,
                    Some(crate::construction::parser_session::StringLocation::Text(_))
                ));
                assert!(retired.source.is_none());
                assert_eq!(retired.trigger, Some(trigger));
                let trigger = rows.cause_at(trigger).unwrap().unwrap();
                assert_eq!(trigger.operation, ConstructionOperation::Token);
                assert_eq!(trigger.source, Some(source.range(10, 25).unwrap()));
                let (row, _) = rows.row(index).unwrap().unwrap();
                observed.push((row.start, row.end, row.source.clone(), row.kind));
            }
        }
        assert_eq!(
            observed,
            [
                (
                    0,
                    1,
                    Some(source.range(3, 4).unwrap()),
                    Some(OriginKind::Literal)
                ),
                (
                    1,
                    3,
                    Some(source.range(4, 8).unwrap()),
                    Some(OriginKind::Literal)
                ),
                (
                    3,
                    4,
                    Some(source.range(8, 10).unwrap()),
                    Some(OriginKind::Newline)
                ),
                (
                    4,
                    5,
                    Some(source.range(10, 25).unwrap()),
                    Some(OriginKind::CharacterReference)
                ),
                (
                    5,
                    6,
                    Some(source.range(10, 25).unwrap()),
                    Some(OriginKind::CharacterReference)
                ),
            ]
        );
        assert_eq!(roots, 1);
        let diagnostics = completed.take_diagnostics().unwrap();
        completed.prepare_native_associations().unwrap();
        diagnostics
            .release(&mut completed.driver.as_mut().unwrap().control)
            .unwrap();
        drop(text);
        drop(root);
        completed.release().unwrap();
    }

    fn units(view: &mut NativeView<'_>, text: NativeString<'_>) -> Vec<u16> {
        (0..text.len())
            .map(|i| view.unit(&text, i).unwrap().unwrap())
            .collect()
    }
    #[test]
    fn completed_intrinsic_raw_fields_keep_utf16_child_and_hidden_sanitization_relations() {
        use crate::construction::parser_session::provenance::{
            ConstructionOperation, RawFieldKind,
        };
        use crate::parser::tokenizer::token::OriginKind;
        for (input, local, kind, expected, expected_maps) in [
            (
                "<textarea>A🙂\r\n&NotEqualTilde;</textarea>",
                "textarea",
                RawFieldKind::Textarea,
                vec![65, 0xd83d, 0xde42, 10, 0x2242, 0x338],
                vec![
                    (0, 1, 10, 11, OriginKind::Literal),
                    (1, 3, 11, 13, OriginKind::Literal),
                    (3, 4, 13, 15, OriginKind::Newline),
                    (4, 5, 15, 30, OriginKind::CharacterReference),
                    (5, 6, 15, 30, OriginKind::CharacterReference),
                ],
            ),
            (
                "<input value='A&#13;B&#10;C' type=hidden>",
                "input",
                RawFieldKind::HiddenInput,
                vec![65, 66, 67],
                vec![
                    (0, 1, 14, 15, OriginKind::Literal),
                    (1, 2, 20, 21, OriginKind::Literal),
                    (2, 3, 26, 27, OriginKind::Literal),
                ],
            ),
        ] {
            let source = HtmlSource::utf16(crate::DocumentString::from_utf16(
                input.encode_utf16().collect::<Vec<_>>(),
            ));
            let request =
                HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
            let mut owner = completed(&request);
            let node = {
                let mut view = owner.native_view().unwrap();
                let root = view.root().unwrap();
                let html = child_named(&mut view, &root, "html");
                let body = child_named(&mut view, &html, "body");
                child_named(&mut view, &body, local)
            };
            {
                let driver = owner.driver.as_mut().unwrap();
                let session = &driver
                    .head
                    .as_ref()
                    .unwrap()
                    .state
                    .as_ref()
                    .unwrap()
                    .before
                    .kernel
                    .session;
                match kind {
                    RawFieldKind::Textarea => assert_eq!(
                        session
                            .textarea_raw_value(&mut driver.control, &node)
                            .unwrap()
                            .code_units(),
                        expected
                    ),
                    RawFieldKind::HiddenInput => {
                        let mut raw =
                            crate::parser::buffer::Utf16Buffer::new(&mut driver.control).unwrap();
                        session
                            .append_hidden_input_internal_value(
                                &mut driver.control,
                                &node,
                                &mut raw,
                            )
                            .unwrap();
                        assert_eq!(raw.units(), expected);
                        raw.release(&mut driver.control).unwrap();
                    }
                }
            }
            let mut rows = owner.native_associations().unwrap();
            let mut actual = Vec::new();
            for index in 0..rows.len() {
                if rows.raw_for(index, &node, kind).unwrap() {
                    let (row, cause) = rows.row(index).unwrap().unwrap();
                    assert_eq!(cause.operation, ConstructionOperation::Replacement);
                    assert!(cause.original.is_some());
                    assert!(cause.trigger.is_some());
                    actual.push((
                        row.start,
                        row.end,
                        row.source.as_ref().unwrap().start(),
                        row.source.as_ref().unwrap().end(),
                        row.kind.unwrap(),
                    ));
                }
            }
            assert_eq!(actual, expected_maps);
            drop(node);
            owner.release().unwrap();
        }
    }
    #[test]
    fn completed_formatting_reconstruction_distinguishes_original_creation_and_later_trigger() {
        use crate::construction::parser_session::provenance::ConstructionOperation;
        let source = HtmlSource::utf8("<p><b>A</p>B");
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut owner = completed(&request);
        let (replay, body) = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let b = view.child_at(&body, 1).unwrap().unwrap();
            assert!(matches!(view.kind(&b).unwrap(), NativeKind::Element(_)));
            let text = view.child_at(&b, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                panic!("reconstructed B")
            };
            assert_eq!(units(&mut view, value), [66]);
            (b, body)
        };
        let mut rows = owner.native_associations().unwrap();
        let mut intended = None;
        for index in 0..rows.len() {
            if rows.node_for(index, &body).unwrap() {
                let (row, _) = rows.row(index).unwrap().unwrap();
                let crate::construction::parser_session::provenance::NativeDestination::Node(node) =
                    row.destination
                else {
                    unreachable!()
                };
                intended = Some(node);
            }
        }
        assert!(intended.is_some());
        let mut seen = 0;
        for index in 0..rows.len() {
            if rows.node_for(index, &replay).unwrap() {
                let (_, cause) = rows.row(index).unwrap().unwrap();
                assert_eq!(cause.operation, ConstructionOperation::Replay);
                assert_eq!(cause.intended_parent, intended);
                let (original, trigger) = (cause.original_cause.unwrap(), cause.trigger.unwrap());
                assert_eq!(
                    rows.cause_at(original).unwrap().unwrap().source,
                    Some(source.range(3, 6).unwrap())
                );
                assert_eq!(
                    rows.cause_at(trigger).unwrap().unwrap().source,
                    Some(source.range(11, 12).unwrap())
                );
                seen += 1;
            }
        }
        assert_eq!(seen, 1);
        drop((replay, body));
        owner.release().unwrap();
    }
    #[test]
    fn completed_adoption_replay_keeps_furthest_block_parent_and_original_moved_text() {
        use crate::construction::parser_session::provenance::{
            ConstructionOperation, NativeDestination,
        };
        use crate::parser::tokenizer::token::OriginKind;
        let source = HtmlSource::utf8("<b>1<p>2</b>3</p>");
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut owner = completed(&request);
        let (original, replay, parent, moved) = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            assert_eq!(view.child_count(&body).unwrap(), 2);
            let original = view.child_at(&body, 0).unwrap().unwrap();
            let parent = view.child_at(&body, 1).unwrap().unwrap();
            let replay = view.child_at(&parent, 0).unwrap().unwrap();
            let moved = view.child_at(&replay, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&moved).unwrap() else {
                panic!("actual adopted text")
            };
            assert_eq!(units(&mut view, value), [50]);
            let tail = view.child_at(&parent, 1).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&tail).unwrap() else {
                panic!("actual tail")
            };
            assert_eq!(units(&mut view, value), [51]);
            (original, replay, parent, moved)
        };
        let mut rows = owner.native_associations().unwrap();
        let mut intended = None;
        let mut original_index = None;
        for index in 0..rows.len() {
            if rows.node_for(index, &parent).unwrap() {
                let (row, _) = rows.row(index).unwrap().unwrap();
                let NativeDestination::Node(node) = row.destination else {
                    unreachable!()
                };
                intended = Some(node);
            }
            if rows.node_for(index, &original).unwrap() {
                let (row, _) = rows.row(index).unwrap().unwrap();
                let NativeDestination::Node(node) = row.destination else {
                    unreachable!()
                };
                original_index = Some(node);
            }
        }
        assert!(intended.is_some());
        let mut seen = 0;
        let mut moved_maps = Vec::new();
        for index in 0..rows.len() {
            if rows.node_for(index, &replay).unwrap() {
                let (row, cause) = rows.row(index).unwrap().unwrap();
                let NativeDestination::Node(node) = row.destination else {
                    unreachable!()
                };
                assert_ne!(Some(node), original_index);
                assert_eq!(cause.operation, ConstructionOperation::Replay);
                assert_eq!(cause.intended_parent, intended);
                let (creation, trigger) = (cause.original_cause.unwrap(), cause.trigger.unwrap());
                assert_eq!(
                    rows.cause_at(creation).unwrap().unwrap().source,
                    Some(source.range(0, 3).unwrap())
                );
                assert_eq!(
                    rows.cause_at(trigger).unwrap().unwrap().source,
                    Some(source.range(8, 12).unwrap())
                );
                seen += 1;
            }
            if rows.text_for(index, &moved).unwrap() {
                let (row, _) = rows.row(index).unwrap().unwrap();
                moved_maps.push((row.start, row.end, row.source.clone(), row.kind));
            }
        }
        assert_eq!(seen, 1);
        assert_eq!(
            moved_maps,
            [(
                0,
                1,
                Some(source.range(7, 8).unwrap()),
                Some(OriginKind::Literal)
            )]
        );
        drop((original, replay, parent, moved));
        owner.release().unwrap();
    }
    #[test]
    fn native_projection_exact_and_one_short_admission_retains_only_safe_failed_diagnostics() {
        let source = HtmlSource::utf8("<p>A🙂\r\n&NotEqualTilde;");
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut successful = completed(&request);
        let before = successful.usage().unwrap().clone();
        successful.prepare_native_associations().unwrap();
        let after = successful.usage().unwrap().clone();
        assert!(
            after
                .resource(HtmlParseResource::SourceMapEntries)
                .high_water
                > before
                    .resource(HtmlParseResource::SourceMapEntries)
                    .high_water
        );
        let cost = after.resource(HtmlParseResource::WorkCredits).current;
        let map_peak = after
            .resource(HtmlParseResource::SourceMapEntries)
            .high_water;
        successful.release().unwrap();
        for resource in [
            HtmlParseResource::WorkCredits,
            HtmlParseResource::SourceMapEntries,
        ] {
            for exact in [false, true] {
                let cap = if resource == HtmlParseResource::WorkCredits {
                    cost
                } else {
                    map_peak
                };
                let limits = super::super::tests::limits(100).into_builder();
                let limits = if resource == HtmlParseResource::WorkCredits {
                    limits.work_credits(cap - u64::from(!exact))
                } else {
                    limits.source_map_entries(cap - u64::from(!exact))
                }
                .finish()
                .unwrap();
                let request = HtmlDocumentParseRequest::offline(source.clone(), limits);
                let mut owner = completed(&request);
                let pointer = owner.diagnostics().unwrap().records_pointer();
                if exact {
                    owner.prepare_native_associations().unwrap();
                    assert_eq!(
                        owner
                            .usage()
                            .unwrap()
                            .resource(HtmlParseResource::SourceMapEntries)
                            .current,
                        after.resource(HtmlParseResource::SourceMapEntries).current
                    );
                    if owner.release().is_err() {
                        owner.release().unwrap();
                    }
                } else {
                    assert!(owner.prepare_native_associations().is_err());
                    let stop = owner.driver.as_ref().unwrap().control.stop().unwrap();
                    assert!(
                        matches!(stop, ParseStop::LimitExceeded { resource: stopped, .. } if stopped == resource)
                    );
                    assert!(owner.native_associations().is_err());
                    assert_eq!(owner.driver.as_ref().unwrap().control.stop(), Some(stop));
                    let mut input = owner.driver.take();
                    let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
                    assert_eq!(failed.stop(), stop);
                    assert_eq!(failed.diagnostics().unwrap().records_pointer(), pointer);
                    // ManagedNodes is a permanent cumulative admission count,
                    // not retained storage and is deliberately never refunded.
                    assert_eq!(
                        failed
                            .after_cleanup()
                            .resource(HtmlParseResource::ManagedNodes)
                            .current,
                        failed
                            .at_stop()
                            .resource(HtmlParseResource::ManagedNodes)
                            .current
                    );
                    assert_eq!(
                        failed
                            .after_cleanup()
                            .resource(HtmlParseResource::BufferedUtf16Units)
                            .current,
                        0
                    );
                    assert_eq!(
                        failed
                            .after_cleanup()
                            .resource(HtmlParseResource::WorkCredits)
                            .current,
                        failed
                            .at_stop()
                            .resource(HtmlParseResource::WorkCredits)
                            .current
                    );
                    assert!(
                        failed
                            .after_cleanup()
                            .resource(HtmlParseResource::SourceMapEntries)
                            .current
                            < failed
                                .at_stop()
                                .resource(HtmlParseResource::SourceMapEntries)
                                .current
                    );
                    assert!(failed.retained_usage().actual_capacity_bytes > 0);
                    failed.release().unwrap();
                    super::super::tests::assert_released(&failed.control);
                }
            }
        }
    }
    #[test]
    fn native_projection_cancel_and_alias_refusal_preserve_the_whole_owner_and_first_stop() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>A"),
            super::super::tests::limits(100),
        );
        let mut owner = completed(&request);
        let html = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            child_named(&mut view, &root, "html")
        };
        let alias = {
            let driver = owner.driver.as_mut().unwrap();
            let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
            state
                .before
                .kernel
                .session
                .element_name(&mut driver.control, &html)
                .unwrap()
                .unwrap()
                .local_name()
                .clone()
        };
        assert!(owner.prepare_native_associations().is_err());
        assert!(owner.driver.as_ref().unwrap().control.stop().is_none());
        drop(alias);
        drop(html);
        owner.prepare_native_associations().unwrap();
        request.cancellation().cancel();
        assert!(owner.native_associations().is_err());
        let stop = owner.driver.as_ref().unwrap().control.stop().unwrap();
        assert_eq!(stop, ParseStop::Cancelled);
        assert!(owner.native_view().is_err());
        assert_eq!(owner.driver.as_ref().unwrap().control.stop(), Some(stop));
        let mut input = owner.driver.take();
        let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
        assert_eq!(failed.stop(), stop);
        failed.release().unwrap();
        super::super::tests::assert_released(&failed.control);
    }
    #[test]
    fn completed_selectedcontent_clone_retains_original_native_identity_and_scheduling_trigger() {
        use crate::construction::parser_session::provenance::{
            ConstructionOperation, NativeDestination,
        };
        let input = "<select><button><selectedcontent></selectedcontent></button><option selected>A</option></select>";
        let source = HtmlSource::utf8(input);
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut owner = completed(&request);
        let (original, copy) = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let select = child_named(&mut view, &body, "select");
            let button = child_named(&mut view, &select, "button");
            let selected = child_named(&mut view, &button, "selectedcontent");
            let option = child_named(&mut view, &select, "option");
            let original = view.child_at(&option, 0).unwrap().unwrap();
            let copy = view.child_at(&selected, 0).unwrap().unwrap();
            for node in [&original, &copy] {
                let NativeKind::Text(value) = view.kind(node).unwrap() else {
                    panic!("actual cloned A")
                };
                assert_eq!(units(&mut view, value), [65]);
            }
            (original, copy)
        };
        let mut rows = owner.native_associations().unwrap();
        let mut original_identity = None;
        for index in 0..rows.len() {
            if rows.node_for(index, &original).unwrap() {
                let (row, _) = rows.row(index).unwrap().unwrap();
                let NativeDestination::Node(node) = row.destination else {
                    panic!("qualified original node")
                };
                original_identity = Some(node);
            }
        }
        let mut nodes = 0;
        let mut fields = 0;
        for index in 0..rows.len() {
            if rows.node_for(index, &copy).unwrap() {
                let (_, cause) = rows.row(index).unwrap().unwrap();
                assert_eq!(cause.operation, ConstructionOperation::Copy);
                assert_eq!(cause.original_node, original_identity);
                let trigger = cause.trigger.unwrap();
                let trigger = rows.cause_at(trigger).unwrap().unwrap();
                let start = input.find("</option>").unwrap();
                assert_eq!(
                    trigger.source,
                    Some(source.range(start, start + 9).unwrap())
                );
                nodes += 1;
            }
            if rows.text_for(index, &copy).unwrap() {
                let (row, cause) = rows.row(index).unwrap().unwrap();
                assert_eq!(cause.operation, ConstructionOperation::Copy);
                assert!(cause.original.is_some());
                let start = input.find(">A<").unwrap() + 1;
                assert_eq!(row.source, Some(source.range(start, start + 1).unwrap()));
                assert_eq!((row.start, row.end), (0, 1));
                fields += 1;
            }
        }
        assert_eq!((nodes, fields), (1, 1));
        drop((original, copy));
        owner.release().unwrap();
    }
    #[test]
    fn completed_template_foreign_and_rewritten_fields_keep_exact_native_bindings() {
        use crate::construction::parser_session::{
            StringLocation,
            provenance::{ConstructionOperation, NativeDestination},
        };
        use crate::parser::tokenizer::token::OriginKind;
        let input =
            "<template><svg><lineargradient></lineargradient></svg></template><image><br></br>";
        let source = HtmlSource::utf8(input);
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut owner = completed(&request);
        let nodes = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let head = child_named(&mut view, &html, "head");
            let template = child_named(&mut view, &head, "template");
            let contents = view.template_contents(&template).unwrap().unwrap();
            assert!(view.parent(&contents).unwrap().is_none());
            let svg = child_named(&mut view, &contents, "svg");
            let adjusted = child_named(&mut view, &svg, "linearGradient");
            let NativeKind::Element(name) = view.kind(&adjusted).unwrap() else {
                panic!("adjusted SVG")
            };
            assert_eq!(
                units(&mut view, name.local),
                "linearGradient".encode_utf16().collect::<Vec<_>>()
            );
            let body = child_named(&mut view, &html, "body");
            let image = child_named(&mut view, &body, "img");
            let end_br = view.child_at(&body, 2).unwrap().unwrap();
            let NativeKind::Element(name) = view.kind(&end_br).unwrap() else {
                panic!("recovered end br")
            };
            assert_eq!(units(&mut view, name.local), [98, 114]);
            [adjusted, image, end_br]
        };
        let mut rows = owner.native_associations().unwrap();
        let mut identities = [None; 3];
        for index in 0..rows.len() {
            for (position, node) in nodes.iter().enumerate() {
                if rows.node_for(index, node).unwrap() {
                    let (row, _) = rows.row(index).unwrap().unwrap();
                    let NativeDestination::Node(id) = row.destination else {
                        unreachable!()
                    };
                    identities[position] = Some(id);
                }
            }
        }
        assert!(identities.iter().all(Option::is_some));
        let mut names = [Vec::new(), Vec::new(), Vec::new()];
        let mut source_less_svg_namespace = false;
        for index in 0..rows.len() {
            let (row, cause) = rows.row(index).unwrap().unwrap();
            match row.destination {
                NativeDestination::Field(StringLocation::ElementName(node)) => {
                    for (position, id) in identities.iter().enumerate() {
                        if Some(node) == *id {
                            if position == 0 {
                                assert_eq!(cause.operation, ConstructionOperation::Replacement);
                                assert!(cause.original.is_some());
                            }
                            names[position].push((
                                row.start,
                                row.end,
                                row.source.clone(),
                                row.kind,
                            ));
                        }
                    }
                }
                NativeDestination::Field(StringLocation::Namespace(node))
                    if Some(node) == identities[0] =>
                {
                    assert!(row.source.is_none());
                    assert_eq!((row.start, row.end), (0, 26));
                    source_less_svg_namespace = true;
                }
                _ => {}
            }
        }
        let expected = (0..14)
            .map(|offset| {
                (
                    offset,
                    offset + 1,
                    Some(source.range(16 + offset, 17 + offset).unwrap()),
                    Some(OriginKind::RecoverySyntax),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(names[0], expected);
        assert_eq!(
            names[1],
            [(
                0,
                3,
                Some(source.range(66, 71).unwrap()),
                Some(OriginKind::RecoverySyntax)
            )]
        );
        assert_eq!(
            names[2],
            [
                (
                    0,
                    1,
                    Some(source.range(78, 79).unwrap()),
                    Some(OriginKind::Literal)
                ),
                (
                    1,
                    2,
                    Some(source.range(79, 80).unwrap()),
                    Some(OriginKind::Literal)
                )
            ]
        );
        assert!(source_less_svg_namespace);
        drop(nodes);
        owner.release().unwrap();
    }
    #[test]
    fn completed_fostered_table_text_keeps_original_processing_and_later_flush_trigger() {
        use crate::construction::parser_session::provenance::{ConstructionOperation, ParserRule};
        use crate::parser::tokenizer::token::OriginKind;
        use crate::parser::tree_builder::head::InsertionMode;
        let source = HtmlSource::utf8("<table>AB<tr><td>Z</td></tr></table>");
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut owner = completed(&request);
        let text = {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let text = view.child_at(&body, 0).unwrap().unwrap();
            let NativeKind::Text(value) = view.kind(&text).unwrap() else {
                panic!("actual fostered Text before table")
            };
            assert_eq!(units(&mut view, value), [65, 66]);
            let table = view.child_at(&body, 1).unwrap().unwrap();
            assert!(matches!(view.kind(&table).unwrap(), NativeKind::Element(_)));
            text
        };
        let mut rows = owner.native_associations().unwrap();
        let mut observed = Vec::new();
        for index in 0..rows.len() {
            if !rows.text_for(index, &text).unwrap() {
                continue;
            }
            let (row, cause) = rows.row(index).unwrap().unwrap();
            assert_eq!(cause.operation, ConstructionOperation::Merge);
            let groups = [cause.original.unwrap(), cause.incoming.unwrap()];
            observed.push((row.start, row.end, row.source.clone(), row.kind));
            for (group, expected) in groups.into_iter().zip([(7, 8), (8, 9)]) {
                let (group, _) = rows.group(group).unwrap().unwrap();
                let replay = group.cause;
                let replay = rows.cause_at(replay).unwrap().unwrap();
                assert_eq!(replay.operation, ConstructionOperation::Replay);
                let (original, trigger) = (replay.original_cause.unwrap(), replay.trigger.unwrap());
                let original = rows.cause_at(original).unwrap().unwrap();
                assert_eq!(original.operation, ConstructionOperation::Token);
                assert_eq!(
                    original.source,
                    Some(source.range(expected.0, expected.1).unwrap())
                );
                assert_eq!(
                    original.rule,
                    Some(ParserRule::Tree(InsertionMode::InTableText))
                );
                let trigger = rows.cause_at(trigger).unwrap().unwrap();
                assert_eq!(trigger.source, Some(source.range(9, 13).unwrap()));
            }
        }
        assert_eq!(
            observed,
            [
                (
                    0,
                    1,
                    Some(source.range(7, 8).unwrap()),
                    Some(OriginKind::Literal)
                ),
                (
                    1,
                    2,
                    Some(source.range(8, 9).unwrap()),
                    Some(OriginKind::Literal)
                ),
            ]
        );
        drop(text);
        owner.release().unwrap();
    }
    #[test]
    fn completed_pi_pseudo_attribute_derivation_keeps_literal_and_xml_reference_groups() {
        use crate::construction::parser_session::{
            StringLocation,
            provenance::{ConstructionOperation, NativeDestination},
        };
        use crate::parser::tokenizer::token::OriginKind;
        let source = HtmlSource::utf8("<body><?marker name='x&amp;y'?>");
        let request =
            HtmlDocumentParseRequest::offline(source.clone(), super::super::tests::limits(100));
        let mut owner = completed(&request);
        {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let pi = view.child_at(&body, 0).unwrap().unwrap();
            let NativeKind::ProcessingInstruction { target, data } = view.kind(&pi).unwrap() else {
                panic!("actual PI")
            };
            assert_eq!(
                units(&mut view, target),
                "marker".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                units(&mut view, data),
                "name='x&amp;y'".encode_utf16().collect::<Vec<_>>()
            );
        }
        let mut rows = owner.native_associations().unwrap();
        let mut values = Vec::new();
        let mut names = 0;
        for index in 0..rows.len() {
            let (row, cause) = rows.row(index).unwrap().unwrap();
            match row.destination {
                NativeDestination::Field(StringLocation::PiName(_, 0)) => {
                    assert_eq!(cause.operation, ConstructionOperation::Replacement);
                    assert!(cause.original.is_some());
                    names += 1;
                }
                NativeDestination::Field(StringLocation::PiValue(_, 0)) => {
                    assert_eq!(cause.operation, ConstructionOperation::Replacement);
                    assert!(cause.original.is_some());
                    assert!(cause.trigger.is_some());
                    values.push((row.start, row.end, row.source.clone(), row.kind));
                }
                _ => {}
            }
        }
        assert_eq!(names, 4);
        assert_eq!(
            values,
            [
                (
                    0,
                    1,
                    Some(source.range(21, 22).unwrap()),
                    Some(OriginKind::Literal)
                ),
                (
                    1,
                    2,
                    Some(source.range(22, 27).unwrap()),
                    Some(OriginKind::RecoverySyntax)
                ),
                (
                    2,
                    3,
                    Some(source.range(27, 28).unwrap()),
                    Some(OriginKind::Literal)
                ),
            ]
        );
        owner.release().unwrap();
    }
    fn child_named(view: &mut NativeView<'_>, parent: &ParserNode, local: &str) -> ParserNode {
        for i in 0..view.child_count(parent).unwrap() {
            let child = view.child_at(parent, i).unwrap().unwrap();
            if let NativeKind::Element(name) = view.kind(&child).unwrap()
                && units(view, name.local) == local.encode_utf16().collect::<Vec<_>>()
            {
                return child;
            }
        }
        panic!("literal native child {local} missing");
    }
    fn completed<'a>(request: &'a HtmlDocumentParseRequest) -> CompletedStandaloneDocument<'a> {
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(request);
        finish(&mut driver);
        StandaloneDocumentDriver::take_completed(&mut Some(driver)).unwrap()
    }

    #[test]
    fn completed_native_view_retains_exact_payloads_and_template_backlinks() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!DOCTYPE html SYSTEM 'about:legacy-compat'><?probe data><html lang='en'><head><template id='t'><svg><a xlink:href='x'>A&#x1F600;</a><!--C--></svg></template></head><body><p>B</p></body></html>",
            ),
            super::super::tests::limits(100),
        );
        let mut owner = completed(&request);
        {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            assert!(matches!(
                view.kind(&root).unwrap(),
                NativeKind::Document(crate::HtmlDocumentMode::NoQuirks)
            ));
            assert!(view.parent(&root).unwrap().is_none());
            assert_eq!(view.child_count(&root).unwrap(), 3);
            let doctype = view.child_at(&root, 0).unwrap().unwrap();
            let NativeKind::Doctype {
                name,
                public_id,
                system_id,
            } = view.kind(&doctype).unwrap()
            else {
                panic!("doctype")
            };
            assert_eq!(
                units(&mut view, name),
                "html".encode_utf16().collect::<Vec<_>>()
            );
            assert!(units(&mut view, public_id.unwrap()).is_empty());
            assert_eq!(
                units(&mut view, system_id.unwrap()),
                "about:legacy-compat".encode_utf16().collect::<Vec<_>>()
            );
            let pi = view.child_at(&root, 1).unwrap().unwrap();
            let NativeKind::ProcessingInstruction { target, data } = view.kind(&pi).unwrap() else {
                panic!("PI")
            };
            assert_eq!(
                units(&mut view, target),
                "probe".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                units(&mut view, data),
                "data".encode_utf16().collect::<Vec<_>>()
            );
            let html = child_named(&mut view, &root, "html");
            let attr = view.attribute_at(&html, 0).unwrap().unwrap();
            assert_eq!(view.attribute_count(&html).unwrap(), 1);
            assert!(attr.name.namespace.is_none());
            assert!(attr.name.prefix.is_none());
            assert_eq!(
                units(&mut view, attr.name.local),
                "lang".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(units(&mut view, attr.value), vec![101, 110]);
            assert!(view.attribute_at(&html, 1).unwrap().is_none());
            let parent = view.parent(&html).unwrap().unwrap();
            assert!(view.same_node(&parent, &root).unwrap());
            assert!(view.shadow(&html).unwrap().is_none());
            let head = child_named(&mut view, &html, "head");
            let template = child_named(&mut view, &head, "template");
            assert_eq!(view.child_count(&template).unwrap(), 0);
            let contents = view.template_contents(&template).unwrap().unwrap();
            assert!(matches!(
                view.kind(&contents).unwrap(),
                NativeKind::Fragment
            ));
            assert!(view.parent(&contents).unwrap().is_none());
            let Some(NativeFragmentOwner::Template(backlink)) =
                view.fragment_owner(&contents).unwrap()
            else {
                panic!("template owner")
            };
            assert!(view.same_node(&backlink, &template).unwrap());
            let svg = child_named(&mut view, &contents, "svg");
            let a = child_named(&mut view, &svg, "a");
            let NativeKind::Element(name) = view.kind(&a).unwrap() else {
                panic!("element")
            };
            assert_eq!(
                units(&mut view, name.namespace.unwrap()),
                "http://www.w3.org/2000/svg"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
            let href = view.attribute_at(&a, 0).unwrap().unwrap();
            assert_eq!(
                units(&mut view, href.name.namespace.unwrap()),
                "http://www.w3.org/1999/xlink"
                    .encode_utf16()
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                units(&mut view, href.name.prefix.unwrap()),
                "xlink".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                units(&mut view, href.name.local),
                "href".encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(units(&mut view, href.value), vec![120]);
            let text_node = view.child_at(&a, 0).unwrap().unwrap();
            let NativeKind::Text(text) = view.kind(&text_node).unwrap() else {
                panic!("text")
            };
            assert_eq!(units(&mut view, text), vec![65, 0xd83d, 0xde00]);
            let comment_node = view.child_at(&svg, 1).unwrap().unwrap();
            let NativeKind::Comment(comment) = view.kind(&comment_node).unwrap() else {
                panic!("comment")
            };
            assert_eq!(units(&mut view, comment), vec![67]);
            assert!(view.child_at(&svg, 2).unwrap().is_none());
            let body = child_named(&mut view, &html, "body");
            let p = child_named(&mut view, &body, "p");
            assert!(view.template_contents(&p).unwrap().is_none());
            assert!(view.fragment_owner(&p).unwrap().is_none());
            assert!(matches!(
                view.attribute_count(&root),
                Err(SessionError::WrongToken)
            ));
        }
        assert_eq!(
            owner.document_state().unwrap().completion,
            crate::construction::parser_session::ConstructionCompletion::Complete
        );
        owner.release().unwrap();
        assert!(matches!(
            owner.native_view(),
            Err(StandaloneError::Consumed)
        ));
    }

    #[test]
    fn completed_native_units_are_exact_and_admitted_before_each_read() {
        let mut input = "<!doctype html><p>".encode_utf16().collect::<Vec<_>>();
        input.extend([0xd800, 0, 0xdc00, 13, 10]);
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf16(crate::DocumentString::from_utf16(input)),
            super::super::tests::limits(100),
        );
        let mut owner = completed(&request);
        {
            let mut view = owner.native_view().unwrap();
            let root = view.root().unwrap();
            let html = child_named(&mut view, &root, "html");
            let body = child_named(&mut view, &html, "body");
            let p = child_named(&mut view, &body, "p");
            let text_node = view.child_at(&p, 0).unwrap().unwrap();
            let NativeKind::Text(text) = view.kind(&text_node).unwrap() else {
                panic!("text")
            };
            assert_eq!(units(&mut view, text), vec![0xd800, 0xdc00, 10]);
            let visits = view.usage().work(WorkCategory::RecordVisit);
            let reads = view.usage().work(WorkCategory::InspectByte);
            assert_eq!(view.unit(&text, 0).unwrap(), Some(0xd800));
            assert_eq!(view.usage().work(WorkCategory::RecordVisit) - visits, 7);
            assert_eq!(view.usage().work(WorkCategory::InspectByte) - reads, 2);
            let reads = view.usage().work(WorkCategory::InspectByte);
            assert_eq!(view.unit(&text, text.len()).unwrap(), None);
            assert_eq!(view.usage().work(WorkCategory::InspectByte), reads);
            let used = view
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            view.control()
                .charge(
                    WorkCategory::StateStep,
                    request.limits().work_credits() - used - 8,
                )
                .unwrap();
            let reads = view.usage().work(WorkCategory::InspectByte);
            let Err(SessionError::Stop(first)) = view.unit(&text, 0) else {
                panic!("one short")
            };
            assert!(matches!(
                first,
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    ..
                }
            ));
            assert_eq!(view.usage().work(WorkCategory::InspectByte), reads);
            assert!(matches!(view.kind(&root), Err(SessionError::Stop(stop)) if stop == first));
        }
        owner.release().unwrap();
        assert!(owner.driver.is_none());
    }

    #[test]
    fn completed_native_view_rejects_foreign_nodes_and_cancel_without_mutation() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A"),
            super::super::tests::limits(100),
        );
        let mut first = completed(&request);
        let other_request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A"),
            super::super::tests::limits(100),
        );
        let mut second = completed(&other_request);
        let foreign = second.native_view().unwrap().root().unwrap();
        {
            let mut view = first.native_view().unwrap();
            assert!(matches!(
                view.kind(&foreign),
                Err(SessionError::ForeignNode)
            ));
            let root = view.root().unwrap();
            let work = view
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            request.cancellation().cancel();
            assert!(matches!(
                view.child_count(&root),
                Err(SessionError::Stop(ParseStop::Cancelled))
            ));
            assert_eq!(
                view.usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                work
            );
        }
        first.release().unwrap();
        second.release().unwrap();
    }

    #[test]
    fn completed_native_view_preserves_backing_alias_rejection_and_incomplete_take() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, EndPhase::InsertionPoint);
        let node = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .current(&mut driver.control)
            .unwrap();
        let mut input = Some(driver);
        assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
        let mut driver = input.take().unwrap();
        finish(&mut driver);
        let mut owner = StandaloneDocumentDriver::take_completed(&mut Some(driver)).unwrap();
        let alias = {
            let driver = owner.driver.as_mut().unwrap();
            let (c, session, _) = driver.standalone_parts().unwrap();
            session
                .element_name(c, &node)
                .unwrap()
                .unwrap()
                .local_name()
                .clone()
        };
        assert!(owner.native_view().is_err());
        assert!(owner.release().is_err());
        assert!(owner.driver.is_some());
        drop(alias);
        {
            let mut view = owner.native_view().unwrap();
            let NativeKind::Element(name) = view.kind(&node).unwrap() else {
                panic!("element")
            };
            assert_eq!(units(&mut view, name.local), vec![112]);
        }
        owner.release().unwrap();
    }

    #[test]
    fn completed_native_view_rejects_a_string_from_the_other_control() {
        let first_request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>A"),
            super::super::tests::limits(100),
        );
        let second_request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p>B"),
            super::super::tests::limits(100),
        );
        let mut first = completed(&first_request);
        let mut second = completed(&second_request);
        {
            let mut other = second.native_view().unwrap();
            let root = other.root().unwrap();
            let html = child_named(&mut other, &root, "html");
            let NativeKind::Element(name) = other.kind(&html).unwrap() else {
                panic!("element")
            };
            let mut view = first.native_view().unwrap();
            let reads = view.usage().work(WorkCategory::InspectByte);
            let Err(SessionError::Stop(first_stop)) = view.unit(&name.local, 0) else {
                panic!("foreign control")
            };
            assert!(matches!(
                first_stop,
                ParseStop::InternalInvariant(crate::parser::control::Invariant::ForeignControl)
            ));
            assert_eq!(view.usage().work(WorkCategory::InspectByte), reads);
            assert!(matches!(view.root(), Err(SessionError::Stop(stop)) if stop == first_stop));
            assert_eq!(other.unit(&name.local, 0).unwrap(), Some(104));
        }
        first.release().unwrap();
        second.release().unwrap();
    }

    #[test]
    fn completed_native_lender_one_short_retains_whole_owner_for_cleanup() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><p>A"),
            super::super::tests::limits(100),
        );
        let mut first = completed(&request);
        let used = first
            .usage()
            .unwrap()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let cost = {
            let view = first.native_view().unwrap();
            view.usage()
                .resource(HtmlParseResource::WorkCredits)
                .current
                - used
        };
        assert!(cost > 0);
        first.release().unwrap();
        let mut owner = completed(&request);
        let driver = owner.driver.as_mut().unwrap();
        let used = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        driver
            .control
            .charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - cost + 1,
            )
            .unwrap();
        assert!(owner.native_view().is_err());
        let driver = owner.driver.as_ref().unwrap();
        assert!(driver.head.is_some());
        let stop = driver.control.stop().unwrap();
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        assert!(owner.native_view().is_err());
        assert_eq!(owner.driver.as_ref().unwrap().control.stop(), Some(stop));
        owner.release().unwrap();
        assert!(owner.driver.is_none());
    }

    fn phase(driver: &StandaloneDocumentDriver<'_>) -> Option<EndPhase> {
        driver
            .head
            .as_ref()
            .and_then(|h| h.state.as_ref())
            .and_then(|s| s.end)
    }
    fn to_phase(driver: &mut StandaloneDocumentDriver<'_>, expected: EndPhase) {
        for _ in 0..1_000 {
            if phase(driver) == Some(expected) {
                return;
            }
            assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
        }
        panic!("actual end phase {expected:?} was not reached");
    }
    fn state(driver: &mut StandaloneDocumentDriver<'_>) -> StandaloneDocumentState {
        let (c, session, _) = driver.standalone_parts().unwrap();
        session.standalone_document_state(c).unwrap()
    }
    fn finish(driver: &mut StandaloneDocumentDriver<'_>) {
        for _ in 0..1_000 {
            match driver.step().unwrap() {
                HeadStep::Consumed => {}
                HeadStep::Complete => return,
                other => panic!("unexpected ordinary boundary {other:?}"),
            }
        }
        panic!("finite ordinary source did not finish");
    }
    fn cleanup(driver: &mut StandaloneDocumentDriver<'_>) {
        if driver.release().is_err() {
            driver.release().unwrap();
        }
        super::super::tests::assert_released(&driver.control);
    }

    #[test]
    fn ordered_evidence_keeps_actual_delayed_table_and_lexer_emission_order() {
        use crate::construction::parser_session::evidence::{
            DiagnosticContext as C, DiagnosticKind as K, DiagnosticSeverity,
        };
        use crate::parser::tokenizer::token::{DiagnosticCode as L, DiagnosticPhase};
        use head::{HeadCondition as T, InsertionMode as M};
        use prologue::{PrologueCondition as P, PrologueMode};
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<table>A<!--\0-->\0</table>"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        finish(&mut driver);
        let pointer = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .session
            .observed_evidence()
            .records_pointer();
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        let evidence = completed.diagnostics().unwrap();
        assert_eq!(evidence.records_pointer(), pointer);
        // HTML Initial; comment-state NULL; delayed InTableText list error,
        // InTable anything-else; Data NULL; InTableText NULL. Source offsets
        // intentionally go backwards after the lexical comment error.
        let expected = [
            (
                K::Prologue(P::MissingDoctype),
                C::Prologue(PrologueMode::Initial),
                0,
                7,
                "the-initial-insertion-mode",
            ),
            (
                K::Tokenizer(L::UnexpectedNullCharacter),
                C::Tokenizer {
                    phase: DiagnosticPhase::Tokenizer,
                    state: crate::parser::tokenizer::states::State::Comment,
                },
                12,
                13,
                "parse-error-unexpected-null-character",
            ),
            (
                K::Tree(T::NonWhitespaceInTable),
                C::Tree(M::InTableText),
                7,
                8,
                "parsing-main-intabletext",
            ),
            (
                K::Tree(T::UnexpectedTableToken),
                C::Tree(M::InTableText),
                7,
                8,
                "parsing-main-intable",
            ),
            (
                K::Tokenizer(L::UnexpectedNullCharacter),
                C::Tokenizer {
                    phase: DiagnosticPhase::Tokenizer,
                    state: crate::parser::tokenizer::states::State::Data,
                },
                16,
                17,
                "parse-error-unexpected-null-character",
            ),
            (
                K::Tree(T::NullInTable),
                C::Tree(M::InTableText),
                16,
                17,
                "parsing-main-intabletext",
            ),
        ];
        assert_eq!(evidence.len(), expected.len());
        for (index, (kind, context, start, end, anchor)) in expected.into_iter().enumerate() {
            let (record, source) = evidence.retained_record(index).unwrap();
            assert_eq!(
                (record.kind, record.context, record.severity),
                (kind, context, DiagnosticSeverity::Recoverable)
            );
            assert_eq!((source.start(), source.end()), (start, end));
            assert_eq!(source.source(), request.source().identity());
            assert_eq!(record.rule.anchor, anchor);
            use crate::construction::parser_session::evidence::{
                DiagnosticLocationRole as R, DiagnosticToken,
            };
            assert_eq!(
                source.role,
                match index {
                    0 => R::TokenEnvelope,
                    1 | 4 => R::InputFrontier,
                    2 | 3 => R::BufferedTableCharacter,
                    _ => R::TextOrigin,
                }
            );
            assert_eq!(
                source.token,
                match index {
                    0 => Some(DiagnosticToken::Tag),
                    1 | 4 => None,
                    _ => Some(DiagnosticToken::Text),
                }
            );
            assert!(!record.rule.branch.is_empty());
        }
        assert!(evidence.retained_record(expected.len()).is_none());
        completed.release().unwrap();
    }

    #[test]
    fn completed_evidence_move_preserves_both_buffers_and_requires_same_completion_owner() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<p/>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        finish(&mut driver);
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        let before = completed.diagnostics().unwrap().records_pointer();
        let evidence = completed.take_diagnostics().unwrap();
        assert_eq!(evidence.records_pointer(), before);
        assert!(completed.diagnostics().is_err());
        assert_eq!(evidence.len(), 2); // missing doctype; nonvoid self-closing p
        let c = &mut completed.driver.as_mut().unwrap().control;
        evidence.release(c).unwrap();
        completed.release().unwrap();
    }

    #[test]
    fn failed_extraction_cleanup_refusal_keeps_actual_evidence_for_retry_or_disposal() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, EndPhase::InsertionPoint);
        let node = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .current(&mut driver.control)
            .unwrap();
        let (c, session, _) = driver.standalone_parts().unwrap();
        let alias = session
            .element_name(c, &node)
            .unwrap()
            .unwrap()
            .local_name()
            .clone();
        request.cancellation().cancel();
        assert!(driver.step().is_err());
        let stop = driver.control.stop();
        let pointer = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .session
            .observed_evidence()
            .records_pointer();
        let mut input = Some(driver);
        assert!(StandaloneDocumentDriver::take_failed(&mut input).is_err());
        assert!(input.is_some());
        assert_eq!(
            input
                .as_ref()
                .unwrap()
                .failed_evidence
                .as_ref()
                .unwrap()
                .records_pointer(),
            pointer
        );
        assert_eq!(input.as_ref().unwrap().control.stop(), stop);
        drop(alias);
        let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(failed.diagnostics().unwrap().records_pointer(), pointer);
        assert_eq!(Some(failed.stop()), stop);
        failed.release().unwrap();
        super::super::tests::assert_released(&failed.control);
    }

    #[test]
    fn failed_evidence_retains_only_committed_prefix_and_actual_storage_tickets() {
        let limits = super::super::tests::limits(100)
            .into_builder()
            .diagnostics(1)
            .finish()
            .unwrap();
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<table>\0"), limits);
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        loop {
            if driver.step().is_err() {
                break;
            }
        }
        let stop = driver.control.stop().unwrap();
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                ..
            }
        ));
        let before = driver.control.usage().clone();
        let polls = driver.control.owned_span_observation().polls;
        let mut input = Some(driver);
        let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(failed.stop(), stop);
        assert_eq!(failed.source().identity(), request.source().identity());
        assert_eq!(failed.at_stop(), &before);
        // ManagedNodes is the nonrefundable cumulative allocation count;
        // native reclamation must not rewrite this execution history.
        assert_eq!(
            failed
                .after_cleanup()
                .resource(HtmlParseResource::ManagedNodes),
            before.resource(HtmlParseResource::ManagedNodes)
        );
        assert_eq!(
            failed
                .after_cleanup()
                .resource(HtmlParseResource::ManagedEdges)
                .current,
            0
        );
        assert_eq!(
            failed
                .after_cleanup()
                .resource(HtmlParseResource::OpenElements)
                .current,
            0
        );
        assert_eq!(
            failed
                .after_cleanup()
                .resource(HtmlParseResource::Diagnostics)
                .current,
            1
        );
        assert_eq!(
            failed
                .after_cleanup()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            1
        );
        assert!(failed.after_cleanup().actual_capacity_bytes > 0);
        assert_eq!(
            failed
                .after_cleanup()
                .resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(failed.control.owned_span_observation().polls, polls);
        let evidence = failed.diagnostics().unwrap();
        assert_eq!(evidence.len(), 1);
        let (record, source) = evidence.retained_record(0).unwrap();
        assert_eq!(
            record.kind,
            crate::construction::parser_session::evidence::DiagnosticKind::Prologue(
                prologue::PrologueCondition::MissingDoctype
            )
        );
        assert_eq!((source.start(), source.end()), (0, 7));
        assert_eq!(failed.retained_usage(), failed.after_cleanup());
        failed.release().unwrap();
        assert!(failed.diagnostics().is_none());
        super::super::tests::assert_released(&failed.control);
        assert_eq!(failed.control.owned_span_observation().polls, polls);
        assert_eq!(failed.stop(), stop);
        assert_eq!(
            failed
                .retained_usage()
                .resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
    }

    #[test]
    fn cancellation_at_final_completion_retains_prior_diagnostics_without_useful_cleanup() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, EndPhase::VerifyComplete);
        request.cancellation().cancel();
        assert!(driver.step().is_err());
        let mut input = Some(driver);
        assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
        let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
        assert_eq!(failed.stop(), ParseStop::Cancelled);
        assert_eq!(
            failed.at_stop().resource(HtmlParseResource::WorkCredits),
            failed
                .after_cleanup()
                .resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(failed.diagnostics().unwrap().len(), 2); // missing doctype; open div at EOF
        failed.release().unwrap();
        super::super::tests::assert_released(&failed.control);
    }

    #[test]
    fn interactive_precedes_all_pops_and_postload_precedes_complete_continuation() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, EndPhase::InsertionPoint);
        assert_eq!(
            state(&mut driver),
            StandaloneDocumentState {
                readiness: ConstructionReadiness::Complete,
                ready_for_post_load_tasks: false,
                completion: ConstructionCompletion::Parsing
            }
        );
        let open = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .stack()
            .unwrap()
            .len();
        to_phase(&mut driver, EndPhase::PopOpen);
        assert_eq!(
            state(&mut driver).readiness,
            ConstructionReadiness::Interactive
        );
        assert_eq!(
            driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .stack()
                .unwrap()
                .len(),
            open
        );
        assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
        assert_eq!(
            driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .before
                .kernel
                .stack()
                .unwrap()
                .len(),
            open - 1
        );
        to_phase(&mut driver, EndPhase::PostLoadMilestone);
        assert_eq!(
            state(&mut driver).readiness,
            ConstructionReadiness::Interactive
        );
        assert!(!state(&mut driver).ready_for_post_load_tasks);
        driver.step().unwrap();
        assert_eq!(phase(&driver), Some(EndPhase::CompleteReadiness));
        assert_eq!(
            state(&mut driver),
            StandaloneDocumentState {
                readiness: ConstructionReadiness::Interactive,
                ready_for_post_load_tasks: true,
                completion: ConstructionCompletion::Parsing
            }
        );
        driver.step().unwrap();
        assert_eq!(
            state(&mut driver).readiness,
            ConstructionReadiness::Complete
        );
        assert_eq!(
            state(&mut driver).completion,
            ConstructionCompletion::Parsing
        );
        finish(&mut driver);
        assert_eq!(
            state(&mut driver).completion,
            ConstructionCompletion::Complete
        );
        let input_reads = driver.control.usage().work(WorkCategory::InputUnitRead);
        let writes = driver.control.usage().work(WorkCategory::RecordWrite);
        assert_eq!(driver.step().unwrap(), HeadStep::Complete);
        assert_eq!(
            driver.control.usage().work(WorkCategory::InputUnitRead),
            input_reads
        );
        assert_eq!(
            driver.control.usage().work(WorkCategory::RecordWrite),
            writes
        );
        cleanup(&mut driver);
    }

    #[test]
    fn both_spin_checkpoints_run_when_empty_and_do_not_write_readiness_early() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(""),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, EndPhase::DomContentLoadedDisposition);
        let writes = driver.control.usage().work(WorkCategory::RecordWrite);
        driver.step().unwrap();
        assert_eq!(phase(&driver), Some(EndPhase::ScriptsCheckpoint));
        // Step6's eliminated external task has only its actual cursor write.
        assert_eq!(
            driver.control.usage().work(WorkCategory::RecordWrite) - writes,
            1
        );
        for next in [
            EndPhase::LoadBlockersCheckpoint,
            EndPhase::RetainCompleteContinuation,
        ] {
            let writes = driver.control.usage().work(WorkCategory::RecordWrite);
            driver.step().unwrap();
            assert_eq!(phase(&driver), Some(next));
            // Actual FIFO entry/exit plus retained triggering-cause reference,
            // restored cause and final phase write: five fixed owned writes.
            assert_eq!(
                driver.control.usage().work(WorkCategory::RecordWrite) - writes,
                5
            );
            assert_eq!(
                state(&mut driver).readiness,
                ConstructionReadiness::Interactive
            );
            assert!(!state(&mut driver).ready_for_post_load_tasks);
        }
        finish(&mut driver);
        cleanup(&mut driver);
    }

    #[test]
    fn eof_drains_real_option_removal_task_and_clears_selectedcontent() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A</option></select>",
            ),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let option = loop {
            driver.step().unwrap();
            let Some(state) = driver.head.as_mut().and_then(|h| h.state.as_mut()) else {
                continue;
            };
            let Some(last) = state.before.kernel.stack().unwrap().len().checked_sub(1) else {
                continue;
            };
            let node = state.before.kernel.at(last, &mut driver.control).unwrap();
            if state
                .before
                .kernel
                .named(&node, "option", &mut driver.control)
                .unwrap()
            {
                break node;
            }
        };
        to_phase(&mut driver, EndPhase::InsertionPoint);
        let (c, session, capability) = driver.standalone_parts().unwrap();
        session.remove(c, &option, Some(capability)).unwrap();
        assert_eq!(
            session.freeze(c).err(),
            Some(SessionError::PendingIntrinsicWork)
        );
        to_phase(&mut driver, EndPhase::ScriptsCheckpoint);
        driver.step().unwrap();
        assert_eq!(phase(&driver), Some(EndPhase::LoadBlockersCheckpoint));
        let (c, session, _) = driver.standalone_parts().unwrap();
        session.require_standalone_effects_finished(c).unwrap();
        finish(&mut driver);
        let (mut c, mut frozen) = super::super::tests::body_document(driver);
        let actual = super::super::tests::outline(frozen.document().unwrap());
        frozen.release(&mut c).unwrap();
        super::super::tests::assert_released(&c);
        assert_eq!(
            actual,
            super::super::tests::expected_outline(&[
                (0, "html"),
                (1, "head"),
                (1, "body"),
                (2, "select"),
                (3, "button"),
                (4, "selectedcontent")
            ])
        );
    }

    const PHASES: [EndPhase; 13] = [
        EndPhase::InsertionPoint,
        EndPhase::Interactive,
        EndPhase::PopOpen,
        EndPhase::DeferredScriptsAbsent,
        EndPhase::DomContentLoadedDisposition,
        EndPhase::ScriptsCheckpoint,
        EndPhase::LoadBlockersCheckpoint,
        EndPhase::RetainCompleteContinuation,
        EndPhase::PrintDisposition,
        EndPhase::PostLoadMilestone,
        EndPhase::CompleteReadiness,
        EndPhase::NullBrowsingContextTail,
        EndPhase::VerifyComplete,
    ];

    #[test]
    fn every_end_phase_cancellation_preserves_state_and_has_zero_useful_cleanup() {
        for target in PHASES {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8(
                    "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A",
                ),
                super::super::tests::limits(100),
            );
            let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
            to_phase(&mut driver, target);
            let work = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            request.cancellation().cancel();
            assert!(matches!(
                driver.step(),
                Err(StandaloneError::Stop(ParseStop::Cancelled))
            ));
            assert_eq!(phase(&driver), Some(target));
            let mut input = Some(driver);
            assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
            let driver = input.as_mut().unwrap();
            assert_eq!(phase(driver), Some(target));
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                work
            );
            cleanup(driver);
            assert_eq!(driver.control.stop(), Some(ParseStop::Cancelled));
            assert_eq!(
                driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                work
            );
        }
    }

    #[test]
    fn exact_one_short_end_phase_work_keeps_first_stop_and_reclaims_all_real_owners() {
        for target in PHASES {
            let request = HtmlDocumentParseRequest::offline(
                HtmlSource::utf8(
                    "<!doctype html><select><button><selectedcontent></selectedcontent></button><option selected>A",
                ),
                super::super::tests::limits(100),
            );
            let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
            to_phase(&mut driver, target);
            let before = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
            driver.step().unwrap();
            let cost = driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current
                - before;
            assert!(cost > 0);
            cleanup(&mut driver);
            for allowance in [0, cost - 1] {
                let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
                to_phase(&mut driver, target);
                let used = driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current;
                driver
                    .control
                    .charge(
                        WorkCategory::StateStep,
                        request.limits().work_credits() - used - allowance,
                    )
                    .unwrap();
                assert!(driver.step().is_err(), "{target:?}/{allowance}");
                let first = driver.control.stop().unwrap();
                assert!(matches!(
                    first,
                    ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                let reads = driver.control.usage().work(WorkCategory::InputUnitRead);
                assert!(matches!(driver.step(), Err(StandaloneError::Stop(stop)) if stop == first));
                assert_eq!(
                    driver.control.usage().work(WorkCategory::InputUnitRead),
                    reads
                );
                let work = driver
                    .control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current;
                cleanup(&mut driver);
                assert_eq!(driver.control.stop(), Some(first));
                assert_eq!(
                    driver
                        .control
                        .usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    work
                );
            }
        }
    }

    #[test]
    fn completed_owner_retains_actual_evidence_and_same_control_usage_without_clones() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype bad><html><head><meta/><title>A&amp;B</title></head><body>\0<div/>X",
            ),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        finish(&mut driver);
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        let lexical = state.lexical.as_slice().as_ptr();
        let diagnostics = state
            .before
            .kernel
            .session
            .observed_evidence()
            .records_pointer();

        let lengths = (
            state.lexical.len(),
            state.before.kernel.session.observed_evidence().len(),
            state.before.diagnostics().len(),
        );
        assert!(lengths.0 > 0 && lengths.1 > 0 && lengths.2 > 0);
        let usage = driver.control.usage().clone();
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        assert!(input.is_none());
        assert!(
            completed
                .usage()
                .unwrap()
                .resource(HtmlParseResource::WorkCredits)
                .current
                >= usage.resource(HtmlParseResource::WorkCredits).current
        );
        let actual = completed.document_state().unwrap();
        assert_eq!(actual.readiness, ConstructionReadiness::Complete);
        assert!(actual.ready_for_post_load_tasks);
        assert_eq!(actual.completion, ConstructionCompletion::Complete);
        let retained = completed
            .driver
            .as_ref()
            .unwrap()
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap();
        assert_eq!(
            (
                retained.lexical.len(),
                retained.before.kernel.session.observed_evidence().len(),
                retained.before.diagnostics().len()
            ),
            lengths
        );
        assert_eq!(retained.lexical.as_slice().as_ptr(), lexical);
        assert_eq!(
            retained
                .before
                .kernel
                .session
                .observed_evidence()
                .records_pointer(),
            diagnostics
        );

        completed.release().unwrap();
        assert!(completed.driver.is_none());
        assert!(matches!(
            completed.document_state(),
            Err(StandaloneError::Consumed)
        ));
    }

    #[test]
    fn completed_take_incomplete_and_aliased_rejection_retains_exact_input_owner() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, EndPhase::InsertionPoint);
        let node = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .current(&mut driver.control)
            .unwrap();
        let mut input = Some(driver);
        assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
        assert!(input.is_some());
        finish(input.as_mut().unwrap());
        let driver = input.as_mut().unwrap();
        let (c, session, _) = driver.standalone_parts().unwrap();
        let alias = session
            .element_name(c, &node)
            .unwrap()
            .unwrap()
            .local_name()
            .clone();
        assert!(matches!(
            StandaloneDocumentDriver::take_completed(&mut input),
            Err(StandaloneError::Head(HeadError::Kernel(
                KernelError::Session(SessionError::AliasedPayload)
            )))
        ));
        assert!(input.is_some());
        drop(alias);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        completed.release().unwrap();
    }

    #[test]
    fn completed_take_exact_one_short_preflight_retains_terminal_owner() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        finish(&mut driver);
        let before = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        let cost = completed
            .usage()
            .unwrap()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        completed.release().unwrap();
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        finish(&mut driver);
        let used = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        driver
            .control
            .charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - cost + 1,
            )
            .unwrap();
        let mut input = Some(driver);
        assert!(StandaloneDocumentDriver::take_completed(&mut input).is_err());
        let driver = input.as_mut().unwrap();
        let first = driver.control.stop().unwrap();
        assert!(matches!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        cleanup(driver);
        assert_eq!(driver.control.stop(), Some(first));
    }

    #[test]
    fn completed_release_alias_and_cancel_keep_real_borrowed_owner_until_cleanup() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        to_phase(&mut driver, EndPhase::InsertionPoint);
        let node = driver
            .head
            .as_ref()
            .unwrap()
            .state
            .as_ref()
            .unwrap()
            .before
            .kernel
            .current(&mut driver.control)
            .unwrap();
        finish(&mut driver);
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        let driver = completed.driver.as_mut().unwrap();
        let (c, session, _) = driver.standalone_parts().unwrap();
        let alias = session
            .element_name(c, &node)
            .unwrap()
            .unwrap()
            .local_name()
            .clone();
        assert!(completed.release().is_err());
        assert!(completed.driver.is_some());
        drop(alias);
        request.cancellation().cancel();
        assert!(matches!(
            completed.document_state(),
            Err(StandaloneError::Stop(ParseStop::Cancelled))
        ));
        assert_eq!(
            completed.driver.as_ref().unwrap().control.stop(),
            Some(ParseStop::Cancelled)
        );
        completed.release().unwrap();
        assert!(completed.driver.is_none());
        assert!(matches!(completed.usage(), Err(StandaloneError::Consumed)));
    }

    #[test]
    fn completed_release_one_short_preflight_retains_owner_then_terminal_cleanup_is_free() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        finish(&mut driver);
        let mut input = Some(driver);
        let mut completed = StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        let driver = completed.driver.as_mut().unwrap();
        let used = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        // Its first actual admission is Option<driver> bytes + one write.
        let first_cost = size_of::<Option<StandaloneDocumentDriver>>() as u64 + 1;
        driver
            .control
            .charge(
                WorkCategory::StateStep,
                request.limits().work_credits() - used - first_cost + 1,
            )
            .unwrap();
        assert!(completed.release().is_err());
        let driver = completed.driver.as_ref().unwrap();
        let first = driver.control.stop().unwrap();
        assert!(matches!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        assert!(driver.head.is_some());
        let work = driver
            .control
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        completed.release().unwrap();
        assert!(completed.driver.is_none());
        // State/usage getters are intentionally consumed along with that owner.
        assert!(matches!(completed.usage(), Err(StandaloneError::Consumed)));
        assert!(work > 0);
    }

    #[test]
    fn unclosed_svg_script_stays_unprocessed_during_eof_pops() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><svg><script>A"),
            super::super::tests::limits(100),
        );
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let script = super::super::tests::svg_open_node(&mut driver);
        finish(&mut driver);
        let (c, session, _) = driver.standalone_parts().unwrap();
        assert!(!session.svg_script_state(c, &script).unwrap().processed);
        cleanup(&mut driver);
    }

    #[test]
    fn failed_transfer_preserves_first_stop_usage_before_reference_scratch_cleanup() {
        // Missing doctype is the one retained diagnostic. The actual unknown
        // named reference then reaches the next diagnostic cap while its
        // literal text and reference-query scratch are still owned.
        let limits = super::super::tests::limits(100)
            .into_builder()
            .diagnostics(1)
            .finish()
            .unwrap();
        let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p>&bogus;"), limits);
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        let mut stopped = false;
        for _ in 0..128 {
            if driver.step().is_err() {
                stopped = true;
                break;
            }
        }
        assert!(stopped, "literal reference reaches the second diagnostic");
        let first = driver.control.stop().unwrap();
        assert!(matches!(
            first,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::Diagnostics,
                limit: 1,
                current: 1,
                requested_delta: 1,
            }
        ));
        let latched = driver.control.first_stop_usage().unwrap().clone();
        let after_unwind = driver.control.usage().clone();
        // The literal ampersand plus five letters were retained at the latch.
        // Normal reference unwinding disposes them before returning the error.
        assert!(
            latched
                .resource(HtmlParseResource::BufferedUtf16Units)
                .current
                >= after_unwind
                    .resource(HtmlParseResource::BufferedUtf16Units)
                    .current
                    + 6
        );
        assert!(
            latched
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                > after_unwind
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current
        );
        assert_eq!(
            latched.resource(HtmlParseResource::WorkCredits),
            after_unwind.resource(HtmlParseResource::WorkCredits)
        );
        let mut input = Some(driver);
        let mut failed = StandaloneDocumentDriver::take_failed(&mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(failed.stop(), first);
        assert_eq!(failed.source().identity(), request.source().identity());
        let evidence = failed.diagnostics().unwrap();
        assert_eq!(evidence.len(), 1);
        let (record, source) = evidence.retained_record(0).unwrap();
        assert_eq!(
            record.kind,
            crate::construction::parser_session::evidence::DiagnosticKind::Prologue(
                prologue::PrologueCondition::MissingDoctype
            )
        );
        assert_eq!((source.start(), source.end()), (0, 3));
        let exported_at_stop = failed.at_stop().clone();
        let after_cleanup = failed.after_cleanup().clone();
        assert_eq!(
            after_cleanup.resource(HtmlParseResource::WorkCredits),
            latched.resource(HtmlParseResource::WorkCredits)
        );
        failed.release().unwrap();
        super::super::tests::assert_released(&failed.control);
        assert_eq!(failed.stop(), first);
        assert_eq!(failed.at_stop(), &exported_at_stop);
        // This is the published failure observation contract, not a demand for
        // a particular staging layout: at_stop is the canonical latch snapshot.
        assert_eq!(exported_at_stop, latched);
    }
}
