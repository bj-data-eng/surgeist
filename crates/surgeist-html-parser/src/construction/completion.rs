//! Qualified fresh ordinary parser-Document readiness and completion facts.
//! HTML 877d: initial no-browsing-context readiness, §13.2.7 and same-state guard.
//! This completion fact is held by the parse, not by the registry, and does not grant public
//! report or browser lifecycle support.

use super::*;
use surgeist_html::construction::ConstructionNodeKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseReadiness {
    Interactive,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseCompletion {
    Parsing,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseDocumentState {
    pub(crate) readiness: HtmlParseReadiness,
    pub(crate) ready_for_post_load_tasks: bool,
    pub(crate) completion: HtmlParseCompletion,
}
impl HtmlParseDocumentState {
    pub fn readiness(&self) -> HtmlParseReadiness {
        self.readiness
    }
    pub fn ready_for_post_load_tasks(&self) -> bool {
        self.ready_for_post_load_tasks
    }
    pub fn completion(&self) -> HtmlParseCompletion {
        self.completion
    }
}
/// The completion of the standalone parser Document. Every state it can hold is a legal one:
/// a parse that is complete is interactive-complete with its post-load tasks ready, so the
/// combinations the public [`HtmlParseDocumentState`] could spell but never reaches have no
/// value here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Completion {
    Parsing {
        readiness: HtmlParseReadiness,
        ready_for_post_load_tasks: bool,
    },
    Complete,
}
impl Completion {
    /// The state of a fresh standalone parser Document.
    pub(super) const fn fresh() -> Self {
        Self::Parsing {
            readiness: HtmlParseReadiness::Complete,
            ready_for_post_load_tasks: false,
        }
    }
    pub(super) const fn state(self) -> HtmlParseDocumentState {
        match self {
            Self::Parsing {
                readiness,
                ready_for_post_load_tasks,
            } => HtmlParseDocumentState {
                readiness,
                ready_for_post_load_tasks,
                completion: HtmlParseCompletion::Parsing,
            },
            Self::Complete => HtmlParseDocumentState {
                readiness: HtmlParseReadiness::Complete,
                ready_for_post_load_tasks: true,
                completion: HtmlParseCompletion::Complete,
            },
        }
    }
    /// Whether the parse is complete.
    pub(super) const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}
impl SessionStorage {
    /// The completion of the standalone parser Document. A session that has not initialized
    /// one has no completion metadata.
    pub(super) fn completion(&self, c: &mut ParseControl) -> Result<Completion, SessionError> {
        self.require(c)?;
        self.completion_during_effects(c)
    }
    /// The original nested source-effect owner has already admitted its frame;
    /// its activity observation must not require that same frame to be idle.
    pub(super) fn completion_during_effects(
        &self,
        c: &mut ParseControl,
    ) -> Result<Completion, SessionError> {
        c.require_identity(&self.control)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.payloads
            .facts
            .completion
            .ok_or(SessionError::CompletionMetadataUnavailable)
    }
    /// Checks that a completion, when there is one, belongs to the Document at record 0.
    pub(super) fn validate_completion_facts(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.payloads.facts.completion.is_some() {
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<Completion>() as u64,
            )?;
            if !matches!(
                self.construction
                    .node_at(0)
                    .ok_or_else(|| latch_registry_row(c))?
                    .kind(),
                ConstructionNodeKind::Document(_)
            ) {
                return Err(SessionError::CompletionMetadataUnavailable);
            }
        }
        Ok(())
    }
    /// Writes the next completion, after the caller has charged the write.
    fn write_completion(&mut self, next: Completion) {
        self.payloads.facts.completion = Some(next);
    }
}
impl ParserArenaSession {
    /// Only the source-qualified fresh prologue calls this. Generic native and
    /// inert Document constructors retain their separate document facts.
    pub(crate) fn initialize_standalone_document(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        storage.require(c)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if !matches!(
            storage
                .construction
                .node_at(0)
                .ok_or_else(|| latch_registry_row(c))?
                .kind(),
            ConstructionNodeKind::Document(_)
        ) {
            return Err(SessionError::NotDocument);
        }
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if storage.payloads.facts.completion.is_some() {
            return Err(SessionError::CompletionMetadataUnavailable);
        }
        payload_write::<Completion>(c, 1)?;
        storage.write_completion(Completion::fresh());
        Ok(())
    }
    pub(crate) fn standalone_document_state(
        &self,
        c: &mut ParseControl,
    ) -> Result<HtmlParseDocumentState, SessionError> {
        let completion = self.active()?.completion(c)?;
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<HtmlParseDocumentState>() as u64,
        )?;
        Ok(completion.state())
    }
    pub(crate) fn update_standalone_readiness(
        &mut self,
        c: &mut ParseControl,
        readiness: HtmlParseReadiness,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let current = storage.completion(c)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<Completion>() as u64,
        )?;
        let Completion::Parsing {
            readiness: held,
            ready_for_post_load_tasks,
        } = current
        else {
            return Err(SessionError::CompletionIncomplete);
        };
        // Normative same-state return owns a comparison but no field write.
        if held == readiness {
            return Ok(());
        }
        if readiness == HtmlParseReadiness::Complete && !ready_for_post_load_tasks {
            return Err(SessionError::CompletionIncomplete);
        }
        payload_write::<HtmlParseReadiness>(c, 1)?;
        storage.write_completion(Completion::Parsing {
            readiness,
            ready_for_post_load_tasks,
        });
        Ok(())
    }
    pub(crate) fn mark_standalone_post_load(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let current = storage.completion(c)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<Completion>() as u64,
        )?;
        let Completion::Parsing {
            readiness: HtmlParseReadiness::Interactive,
            ready_for_post_load_tasks: false,
        } = current
        else {
            return Err(SessionError::CompletionIncomplete);
        };
        payload_write::<bool>(c, 1)?;
        storage.write_completion(Completion::Parsing {
            readiness: HtmlParseReadiness::Interactive,
            ready_for_post_load_tasks: true,
        });
        Ok(())
    }
    /// Actual closed constructor/producers, source and owner proof; queued
    /// intrinsic UpdateSelect work is allowed until its two source checkpoints.
    pub(crate) fn qualify_standalone_completion(
        &self,
        c: &mut ParseControl,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
        finished: bool,
    ) -> Result<(), SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active()?;
        storage.require(c)?;
        storage.validate_completion_facts(c)?;
        let completion = storage.completion(c)?;
        if finished {
            storage.effects.require_freezable(c)?;
            if !completion.is_complete() {
                return Err(SessionError::CompletionIncomplete);
            }
        } else if completion.is_complete() {
            return Err(SessionError::CompletionIncomplete);
        }
        Ok(())
    }
    pub(crate) fn require_standalone_effects_finished(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        if self.active()?.csp_pending(c)? {
            return Err(SessionError::PendingIntrinsicWork);
        }
        let storage = self.active()?;
        storage.require(c)?;
        storage.effects.require_freezable(c)
    }
    pub(crate) fn mark_standalone_complete(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        self.require_standalone_effects_finished(c)?;
        let storage = self.active_mut()?;
        let current = storage.completion(c)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<Completion>() as u64,
        )?;
        let Completion::Parsing {
            readiness: HtmlParseReadiness::Complete,
            ready_for_post_load_tasks: true,
        } = current
        else {
            return Err(SessionError::CompletionIncomplete);
        };
        payload_write::<HtmlParseCompletion>(c, 1)?;
        storage.write_completion(Completion::Complete);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html::construction::HtmlDocumentMode;

    /// The completion of the parser Document is one value of the session, held inline: starting
    /// it admits no auxiliary entry and adds no registry row, so it cannot be refused for a
    /// limit that its parse has already reached.
    #[test]
    fn completion_is_held_by_the_session_and_admits_no_auxiliary_entry() {
        let mut c = ParseControl::new(
            super::super::tests::limits(),
            crate::HtmlParseCancellation::new(),
        );
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let initial = c
            .usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current;
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        // A parse whose auxiliary entries are exactly those of the empty session can still
        // start its completion.
        let limits = super::super::tests::limits()
            .into_builder()
            .auxiliary_entries(initial)
            .active_formatting_entries(initial)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let before = session.active().unwrap().payloads.values.len();
        session.initialize_standalone_document(&mut c).unwrap();
        assert_eq!(session.active().unwrap().payloads.values.len(), before);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            initial
        );
        assert_eq!(
            session
                .standalone_document_state(&mut c)
                .unwrap()
                .completion,
            HtmlParseCompletion::Parsing
        );
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.stop(), None);
    }

    #[test]
    fn cancelled_metadata_initialization_preserves_registry_and_first_stop() {
        let cancellation = crate::HtmlParseCancellation::new();
        let mut c = ParseControl::new(super::super::tests::limits(), cancellation.clone());
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        let before = session.active().unwrap().payloads.values.len();
        let work = c.usage().resource(HtmlParseResource::WorkCredits).current;
        cancellation.cancel();
        assert_eq!(
            session.initialize_standalone_document(&mut c),
            Err(SessionError::Stop(HtmlParseStop::Cancelled))
        );
        assert_eq!(session.active().unwrap().payloads.values.len(), before);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.stop(), Some(HtmlParseStop::Cancelled));
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
    }

    #[test]
    fn completion_requires_actual_postload_and_rejects_duplicate_or_fragment_metadata() {
        let mut c = ParseControl::new(
            super::super::tests::limits(),
            crate::HtmlParseCancellation::new(),
        );
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        assert_eq!(
            session.initialize_standalone_document(&mut c),
            Err(SessionError::CompletionMetadataUnavailable)
        );
        let writes = c.usage().work(HtmlParseWorkCategory::RecordWrite);
        assert_eq!(
            session.mark_standalone_post_load(&mut c),
            Err(SessionError::CompletionIncomplete)
        );
        assert_eq!(
            session.mark_standalone_complete(&mut c),
            Err(SessionError::CompletionIncomplete)
        );
        session
            .update_standalone_readiness(&mut c, HtmlParseReadiness::Interactive)
            .unwrap();
        let interactive_writes = c.usage().work(HtmlParseWorkCategory::RecordWrite);
        assert_eq!(interactive_writes, writes + 1);
        assert_eq!(
            session.update_standalone_readiness(&mut c, HtmlParseReadiness::Complete),
            Err(SessionError::CompletionIncomplete)
        );
        assert_eq!(
            c.usage().work(HtmlParseWorkCategory::RecordWrite),
            interactive_writes
        );
        session.mark_standalone_post_load(&mut c).unwrap();
        session
            .update_standalone_readiness(&mut c, HtmlParseReadiness::Complete)
            .unwrap();
        session.mark_standalone_complete(&mut c).unwrap();
        assert_eq!(
            session.update_standalone_readiness(&mut c, HtmlParseReadiness::Interactive),
            Err(SessionError::CompletionIncomplete)
        );
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        let mut fragment = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        assert_eq!(
            fragment.initialize_standalone_document(&mut c),
            Err(SessionError::NotDocument)
        );
        fragment.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_session_without_a_started_completion_reports_the_missing_metadata() {
        let mut c = ParseControl::new(
            super::super::tests::limits(),
            crate::HtmlParseCancellation::new(),
        );
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        assert_eq!(
            session.standalone_document_state(&mut c),
            Err(SessionError::CompletionMetadataUnavailable)
        );
        assert_eq!(
            session.mark_standalone_complete(&mut c),
            Err(SessionError::CompletionMetadataUnavailable)
        );
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn every_completion_value_converts_to_a_state_of_the_public_vocabulary() {
        for readiness in [
            HtmlParseReadiness::Interactive,
            HtmlParseReadiness::Complete,
        ] {
            for ready in [false, true] {
                let state = Completion::Parsing {
                    readiness,
                    ready_for_post_load_tasks: ready,
                }
                .state();
                assert_eq!(state.readiness(), readiness);
                assert_eq!(state.ready_for_post_load_tasks(), ready);
                assert_eq!(state.completion(), HtmlParseCompletion::Parsing);
            }
        }
        let done = Completion::Complete.state();
        assert_eq!(done.readiness(), HtmlParseReadiness::Complete);
        assert!(done.ready_for_post_load_tasks());
        assert_eq!(done.completion(), HtmlParseCompletion::Complete);
    }
}
