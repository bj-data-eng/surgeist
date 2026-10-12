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
            2,
            &mut driver.control,
        )?;
        driver.control.require_source(driver.source)?;
        Ok(CompletedStandaloneDocument {
            driver: input.take(),
        })
    }
}
impl CompletedStandaloneDocument<'_> {
    pub(crate) fn document_state(&mut self) -> Result<StandaloneDocumentState, StandaloneError> {
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
        driver.release()?;
        self.driver = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::parser_session::ConstructionCompletion;
    use crate::{HtmlDocumentParseRequest, HtmlParseResource, HtmlSource};

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
    fn interactive_precedes_all_pops_and_postload_precedes_complete_continuation() {
        let request = HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><div>A"),
            super::super::tests::limits(100),
        );
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
            // Actual FIFO entry true, empty-queue exit false, then phase write.
            assert_eq!(
                driver.control.usage().work(WorkCategory::RecordWrite) - writes,
                3
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
            let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
            let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
                let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        finish(&mut driver);
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        let lexical = state.lexical.as_slice().as_ptr();
        let diagnostics = state.diagnostics.as_slice().as_ptr();
        let before_diagnostics = state.before.diagnostics.as_slice().as_ptr();
        let lengths = (
            state.lexical.len(),
            state.diagnostics.len(),
            state.before.diagnostics.len(),
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
                retained.diagnostics.len(),
                retained.before.diagnostics.len()
            ),
            lengths
        );
        assert_eq!(retained.lexical.as_slice().as_ptr(), lexical);
        assert_eq!(retained.diagnostics.as_slice().as_ptr(), diagnostics);
        assert_eq!(
            retained.before.diagnostics.as_slice().as_ptr(),
            before_diagnostics
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
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
        let mut driver = StandaloneDocumentDriver::new(&request).unwrap();
        let script = super::super::tests::svg_open_node(&mut driver);
        finish(&mut driver);
        let (c, session, _) = driver.standalone_parts().unwrap();
        assert!(!session.svg_script_state(c, &script).unwrap().processed);
        cleanup(&mut driver);
    }
}
