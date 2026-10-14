//! HTML 877d §13.2.7, reduced by the closed fresh parser Document operation.
//! Readiness, canonical pops and both unconditional spin checkpoints are real.
//! External work is eliminated from the admitted producer inventory, not from
//! a Disabled label or an empty task queue. No public parse report is produced.

use super::*;
use crate::construction::parser_session::ConstructionReadiness;
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
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
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
        // Both admitted constructors own a fresh parser Document without a
        // browsing context. Fragment target/context documents remain qualified
        // immutable dependencies; they do not supply a host or execution domain
        // for that parser Document. Actual intrinsic tasks still run through
        // the same checkpoint owner. New loader/host/realm/task producers must
        // revise this closed boundary.
        let script = self.script.as_ref().expect("qualified script state");
        script
            .capability
            .validate_session(c, &self.before.kernel.session)?;
        Ok(())
    }
    pub(super) fn qualify_completed(&self, c: &mut ParseControl) -> Result<(), HeadError> {
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
