//! Qualified fresh ordinary parser-Document readiness and completion facts.
//! HTML 877d: initial no-browsing-context readiness, §13.2.7 and same-state guard.
//! This registry fact does not grant public report or browser lifecycle support.

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConstructionReadiness {
    Interactive,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConstructionCompletion {
    Parsing,
    Complete,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StandaloneDocumentState {
    pub(crate) readiness: ConstructionReadiness,
    pub(crate) ready_for_post_load_tasks: bool,
    pub(crate) completion: ConstructionCompletion,
}
impl SessionStorage {
    fn completion_slot(&self, c: &mut ParseControl) -> Result<usize, SessionError> {
        self.require(c)?;
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::StandaloneDocumentCompletion { node, .. } = payload {
                c.charge(WorkCategory::InspectByte, mem::size_of::<usize>() as u64)?;
                if *node != 0 {
                    return Err(SessionError::CompletionMetadataUnavailable);
                }
                return Ok(slot);
            }
        }
        Err(SessionError::CompletionMetadataUnavailable)
    }
    pub(super) fn validate_completion_facts(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let mut found = false;
        for payload in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::StandaloneDocumentCompletion { node, state } = payload {
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<StandaloneDocumentState>() as u64,
                )?;
                if found || *node != 0 || !matches!(self.records[0].kind, HtmlNodeKind::Document(_))
                {
                    return Err(SessionError::CompletionMetadataUnavailable);
                }
                found = true;
                if state.completion == ConstructionCompletion::Complete
                    && (state.readiness != ConstructionReadiness::Complete
                        || !state.ready_for_post_load_tasks)
                {
                    return Err(SessionError::CompletionIncomplete);
                }
            }
        }
        Ok(())
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
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !matches!(storage.records[0].kind, HtmlNodeKind::Document(_)) {
            return Err(SessionError::NotDocument);
        }
        for payload in &storage.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(
                payload,
                RetainedPayload::StandaloneDocumentCompletion { .. }
            ) {
                return Err(SessionError::CompletionMetadataUnavailable);
            }
        }
        storage.prepare_payload_slot_owned(c)?;
        storage
            .payloads
            .values
            .push(RetainedPayload::StandaloneDocumentCompletion {
                node: 0,
                state: StandaloneDocumentState {
                    readiness: ConstructionReadiness::Complete,
                    ready_for_post_load_tasks: false,
                    completion: ConstructionCompletion::Parsing,
                },
            });
        Ok(())
    }
    pub(crate) fn standalone_document_state(
        &self,
        c: &mut ParseControl,
    ) -> Result<StandaloneDocumentState, SessionError> {
        let storage = self.active()?;
        let slot = storage.completion_slot(c)?;
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<StandaloneDocumentState>() as u64,
        )?;
        let RetainedPayload::StandaloneDocumentCompletion { state, .. } =
            storage.payloads.values[slot]
        else {
            unreachable!("qualified completion slot")
        };
        Ok(state)
    }
    pub(crate) fn update_standalone_readiness(
        &mut self,
        c: &mut ParseControl,
        readiness: ConstructionReadiness,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let slot = storage.completion_slot(c)?;
        let RetainedPayload::StandaloneDocumentCompletion { state, .. } =
            &storage.payloads.values[slot]
        else {
            unreachable!("qualified completion slot")
        };
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<StandaloneDocumentState>() as u64,
        )?;
        if state.completion != ConstructionCompletion::Parsing {
            return Err(SessionError::CompletionIncomplete);
        }
        // Normative same-state return owns a comparison but no field write.
        if state.readiness == readiness {
            return Ok(());
        }
        if readiness == ConstructionReadiness::Complete && !state.ready_for_post_load_tasks {
            return Err(SessionError::CompletionIncomplete);
        }
        payload_write::<ConstructionReadiness>(c, 1)?;
        let RetainedPayload::StandaloneDocumentCompletion { state, .. } =
            &mut storage.payloads.values[slot]
        else {
            unreachable!("qualified completion slot")
        };
        state.readiness = readiness;
        Ok(())
    }
    pub(crate) fn mark_standalone_post_load(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let slot = storage.completion_slot(c)?;
        let RetainedPayload::StandaloneDocumentCompletion { state, .. } =
            storage.payloads.values[slot]
        else {
            unreachable!("qualified completion slot")
        };
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<StandaloneDocumentState>() as u64,
        )?;
        if state.completion != ConstructionCompletion::Parsing
            || state.readiness != ConstructionReadiness::Interactive
            || state.ready_for_post_load_tasks
        {
            return Err(SessionError::CompletionIncomplete);
        }
        payload_write::<bool>(c, 1)?;
        let RetainedPayload::StandaloneDocumentCompletion { state, .. } =
            &mut storage.payloads.values[slot]
        else {
            unreachable!("qualified completion slot")
        };
        state.ready_for_post_load_tasks = true;
        Ok(())
    }
    /// Actual closed constructor/producers, source and owner proof; queued
    /// intrinsic UpdateSelect work is allowed until its two source checkpoints.
    pub(crate) fn qualify_standalone_completion(
        &self,
        c: &mut ParseControl,
        capability: &crate::parser::tree_builder::StandaloneCapability<'_>,
        finished: bool,
    ) -> Result<(), SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active()?;
        storage.require(c)?;
        storage.validate_completion_facts(c)?;
        storage.payloads.check_aliases(&storage.records, c)?;
        let state = self.standalone_document_state(c)?;
        if finished {
            storage.effects.require_freezable(c)?;
            if state.completion != ConstructionCompletion::Complete
                || state.readiness != ConstructionReadiness::Complete
                || !state.ready_for_post_load_tasks
            {
                return Err(SessionError::CompletionIncomplete);
            }
        } else if state.completion != ConstructionCompletion::Parsing {
            return Err(SessionError::CompletionIncomplete);
        }
        Ok(())
    }
    pub(crate) fn require_standalone_effects_finished(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
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
        let slot = storage.completion_slot(c)?;
        let RetainedPayload::StandaloneDocumentCompletion { state, .. } =
            storage.payloads.values[slot]
        else {
            unreachable!("qualified completion slot")
        };
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<StandaloneDocumentState>() as u64,
        )?;
        if state.completion != ConstructionCompletion::Parsing
            || state.readiness != ConstructionReadiness::Complete
            || !state.ready_for_post_load_tasks
        {
            return Err(SessionError::CompletionIncomplete);
        }
        payload_write::<ConstructionCompletion>(c, 1)?;
        let RetainedPayload::StandaloneDocumentCompletion { state, .. } =
            &mut storage.payloads.values[slot]
        else {
            unreachable!("qualified completion slot")
        };
        state.completion = ConstructionCompletion::Complete;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_metadata_consumes_one_actual_retained_slot_and_one_short_reclaims_capacity() {
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
        for allowance in [0, 1] {
            let limits = super::super::tests::limits()
                .into_builder()
                .auxiliary_entries(initial + allowance)
                .finish()
                .unwrap();
            let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
            let mut session =
                ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                    .unwrap();
            let before = session.active().unwrap().payloads.values.len();
            let result = session.initialize_standalone_document(&mut c);
            if allowance == 1 {
                result.unwrap();
                assert_eq!(session.active().unwrap().payloads.values.len(), before + 1);
                assert_eq!(
                    c.usage()
                        .resource(HtmlParseResource::AuxiliaryEntries)
                        .current,
                    initial + 1
                );
                assert_eq!(
                    session
                        .standalone_document_state(&mut c)
                        .unwrap()
                        .completion,
                    ConstructionCompletion::Parsing
                );
            } else {
                assert!(matches!(
                    result,
                    Err(SessionError::Stop(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::AuxiliaryEntries,
                        ..
                    }))
                ));
                assert_eq!(session.active().unwrap().payloads.values.len(), before);
            }
            let stop = c.stop();
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
            assert_eq!(c.stop(), stop);
        }
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
            Err(SessionError::Stop(ParseStop::Cancelled))
        );
        assert_eq!(session.active().unwrap().payloads.values.len(), before);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            work
        );
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.stop(), Some(ParseStop::Cancelled));
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
        let writes = c.usage().work(WorkCategory::RecordWrite);
        assert_eq!(
            session.mark_standalone_post_load(&mut c),
            Err(SessionError::CompletionIncomplete)
        );
        assert_eq!(
            session.mark_standalone_complete(&mut c),
            Err(SessionError::CompletionIncomplete)
        );
        session
            .update_standalone_readiness(&mut c, ConstructionReadiness::Interactive)
            .unwrap();
        let interactive_writes = c.usage().work(WorkCategory::RecordWrite);
        assert_eq!(interactive_writes, writes + 1);
        assert_eq!(
            session.update_standalone_readiness(&mut c, ConstructionReadiness::Complete),
            Err(SessionError::CompletionIncomplete)
        );
        assert_eq!(
            c.usage().work(WorkCategory::RecordWrite),
            interactive_writes
        );
        session.mark_standalone_post_load(&mut c).unwrap();
        session
            .update_standalone_readiness(&mut c, ConstructionReadiness::Complete)
            .unwrap();
        session.mark_standalone_complete(&mut c).unwrap();
        assert_eq!(
            session.update_standalone_readiness(&mut c, ConstructionReadiness::Interactive),
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
}
