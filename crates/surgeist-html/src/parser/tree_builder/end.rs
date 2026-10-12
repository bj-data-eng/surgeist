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
    /// Borrows the actual completed native records and the same control. No
    /// session mutation, fixture extraction, freeze or evidence disposal occurs.
    pub(crate) fn native_view(
        &mut self,
    ) -> Result<crate::construction::parser_session::NativeView<'_>, StandaloneError> {
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
    ) -> Result<crate::construction::parser_session::evidence::ParseEvidence, StandaloneError> {
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
    ) -> Result<&crate::construction::parser_session::evidence::ParseEvidence, StandaloneError>
    {
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
            .map_err(KernelError::from)
            .map_err(HeadError::from)
            .map_err(StandaloneError::from)
    }
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

/// Private assembler input. No tree, native references, effects or uncommitted
/// token drafts survive successful extraction. Source ownership stays with the
/// borrowed request; the later consuming public entry must move that handle.
pub(crate) struct FailedStandaloneDocument<'source> {
    source: &'source crate::HtmlSource,
    control: ParseControl,
    stop: ParseStop,
    at_stop: super::super::control::ParseUsage,
    after_cleanup: super::super::control::ParseUsage,
    evidence: Option<crate::construction::parser_session::evidence::ParseEvidence>,
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
            driver.failure_usage = Some(driver.control.usage().clone());
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
    ) -> Option<&crate::construction::parser_session::evidence::ParseEvidence> {
        self.evidence.as_ref()
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

    fn units(view: &mut NativeView<'_>, text: NativeString<'_>) -> Vec<u16> {
        (0..text.len())
            .map(|i| view.unit(&text, i).unwrap().unwrap())
            .collect()
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
