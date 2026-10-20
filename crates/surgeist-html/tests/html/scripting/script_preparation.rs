//! HTML 877d #prepare-the-script-element literal state/effect oracles.
//! This adapter applies actual public script flags and records each real slot
//! write. Qualified tree/document acquisition remains the consuming owner's job.
use surgeist_html::bounded::*;
use surgeist_html::{
    DocumentString, HtmlScriptFlags, HtmlScriptKind, HtmlScriptReadySteps, HtmlScriptResult,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Document {
    Parser,
    Adopted,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct State {
    flags: HtmlScriptFlags,
    parser: Option<Document>,
    preparation: Option<Document>,
    kind: Option<HtmlScriptKind>,
    result: HtmlScriptResult,
    ready_steps: Option<HtmlScriptReadySteps>,
}
struct Fixture {
    cursor: HtmlScriptPreparationCursor,
    state: State,
    saved: Option<Document>,
    node_document: Document,
    async_present: bool,
    text_empty: bool,
    src_present: bool,
    connected: bool,
    explicit_type: Option<DocumentString>,
    language: Option<DocumentString>,
    scripting: HtmlScriptPreparationReply,
    writes: Vec<(HtmlScriptPreparationAction, State)>,
}
impl Fixture {
    fn parser() -> Self {
        let mut flags = HtmlScriptFlags::fresh();
        flags.mark_parser_inserted();
        Self {
            cursor: HtmlScriptPreparationCursor::new(),
            state: State {
                flags,
                parser: Some(Document::Parser),
                preparation: None,
                kind: None,
                result: HtmlScriptResult::Uninitialized,
                ready_steps: None,
            },
            saved: None,
            node_document: Document::Parser,
            async_present: false,
            text_empty: false,
            src_present: false,
            connected: true,
            explicit_type: None,
            language: None,
            scripting: HtmlScriptPreparationReply::Disabled,
            writes: vec![],
        }
    }
    fn step(&mut self) -> Result<(), HtmlScriptPreparationError> {
        use HtmlScriptPreparationAction as A;
        use HtmlScriptPreparationReply as R;
        let action = self.cursor.action();
        let reply = match action {
            A::ReadAlreadyStarted => R::AlreadyStarted(self.state.flags.already_started()),
            A::SaveAndClearParserDocument => {
                self.saved = self.state.parser.take();
                R::ParserDocumentSaved(self.saved.is_some())
            }
            A::ReadAsyncPresence => R::AsyncPresent(self.async_present),
            A::SetForceAsync => {
                self.state.flags.parser_document_cleared_without_async();
                R::Applied
            }
            A::ReadChildTextEmpty => R::ChildTextEmpty(self.text_empty),
            A::ReadSrcPresence => R::SrcPresent(self.src_present),
            A::ReadConnected => R::Connected(self.connected),
            A::ClassifyType => {
                let mut kind = HtmlScriptTypeCursor::preparation(
                    self.explicit_type.as_ref(),
                    self.language.as_ref(),
                );
                while kind.required_work().is_some() {
                    kind.advance();
                }
                R::Classified(kind.result().expect("complete admitted classification"))
            }
            A::StoreType(kind) => {
                self.state.kind = Some(kind);
                R::Applied
            }
            A::RestoreParserDocument => {
                self.state.parser = self.saved;
                self.state.flags.parser_document_restored();
                R::Applied
            }
            A::MarkAlreadyStarted => {
                self.state.flags.mark_already_started();
                R::Applied
            }
            A::StorePreparationDocument => {
                self.state.preparation = Some(self.node_document);
                R::Applied
            }
            A::CompareDocuments => R::DocumentsEqual(self.state.parser == self.state.preparation),
            A::RequireDisabledScripting => self.scripting,
            A::Complete(_) => return Err(HtmlScriptPreparationError::ReplyMismatch),
        };
        if matches!(
            action,
            A::SaveAndClearParserDocument
                | A::SetForceAsync
                | A::StoreType(_)
                | A::RestoreParserDocument
                | A::MarkAlreadyStarted
                | A::StorePreparationDocument
        ) {
            self.writes.push((action, self.state.clone()));
        }
        self.cursor.reply(reply)
    }
    fn run(&mut self) -> Result<HtmlScriptPreparationResult, HtmlScriptPreparationError> {
        for _ in 0..32 {
            if let HtmlScriptPreparationAction::Complete(result) = self.cursor.action() {
                return Ok(result);
            }
            self.step()?;
        }
        panic!("finite script preparation prefix did not complete")
    }
    fn until(&mut self, action: HtmlScriptPreparationAction) {
        for _ in 0..32 {
            if self.cursor.action() == action {
                return;
            }
            self.step().unwrap();
        }
        panic!("required action not reached: {action:?}")
    }
}

#[test]
fn already_started_preserves_every_existing_slot_without_clearing_parser_document() {
    let mut fixture = Fixture::parser();
    fixture.state.flags.mark_already_started();
    fixture.state.preparation = Some(Document::Adopted);
    fixture.state.kind = Some(HtmlScriptKind::Module);
    let expected = fixture.state.clone();
    assert_eq!(
        fixture.run(),
        Ok(HtmlScriptPreparationResult::AlreadyStarted)
    );
    assert_eq!(fixture.state, expected);
    assert!(fixture.writes.is_empty());
}

#[test]
fn empty_disconnected_and_unrecognized_returns_keep_only_the_reached_prefix_writes() {
    for (text_empty, src_present, connected, kind, expected) in [
        (
            true,
            false,
            false,
            Some("module"),
            HtmlScriptPreparationResult::Empty,
        ),
        (
            true,
            true,
            false,
            None,
            HtmlScriptPreparationResult::Disconnected,
        ),
        (
            false,
            false,
            true,
            Some(" module "),
            HtmlScriptPreparationResult::UnrecognizedType,
        ),
    ] {
        for async_present in [false, true] {
            let mut fixture = Fixture::parser();
            fixture.text_empty = text_empty;
            fixture.src_present = src_present;
            fixture.connected = connected;
            fixture.explicit_type = kind.map(DocumentString::from);
            fixture.async_present = async_present;
            assert_eq!(fixture.run(), Ok(expected));
            assert_eq!(fixture.state.parser, None);
            assert_eq!(fixture.state.preparation, None);
            assert_eq!(fixture.state.kind, None);
            assert_eq!(fixture.state.flags.force_async(), !async_present);
            assert!(!fixture.state.flags.already_started());
            assert_eq!(fixture.state.result, HtmlScriptResult::Uninitialized);
            assert_eq!(fixture.state.ready_steps, None);
        }
    }
}

#[test]
fn recognized_types_store_restore_mark_and_preparation_document_in_distinct_source_steps() {
    for (kind, expected) in [
        (None, HtmlScriptKind::Classic),
        (Some("module"), HtmlScriptKind::Module),
        (Some("importmap"), HtmlScriptKind::ImportMap),
        (Some("speculationrules"), HtmlScriptKind::SpeculationRules),
    ] {
        let mut fixture = Fixture::parser();
        fixture.explicit_type = kind.map(DocumentString::from);
        fixture.until(HtmlScriptPreparationAction::RestoreParserDocument);
        assert_eq!(fixture.state.kind, Some(expected));
        assert_eq!(fixture.state.parser, None);
        assert!(fixture.state.flags.force_async());
        assert!(!fixture.state.flags.already_started());
        fixture.step().unwrap(); // Step14 is acknowledged independently.
        assert_eq!(fixture.state.parser, Some(Document::Parser));
        assert!(!fixture.state.flags.force_async());
        assert!(!fixture.state.flags.already_started());
        assert_eq!(fixture.state.preparation, None);
        fixture.step().unwrap(); // Step15, before preparation-time document.
        assert!(fixture.state.flags.already_started());
        assert_eq!(fixture.state.preparation, None);
        fixture.step().unwrap(); // Step16, before document comparison.
        assert_eq!(fixture.state.preparation, Some(Document::Parser));
        assert_eq!(
            fixture.run(),
            Ok(HtmlScriptPreparationResult::Disabled(expected))
        );
        assert_eq!(
            fixture.state.flags,
            HtmlScriptFlags::from_retained(false, true, false, false, false)
        );
        assert_eq!(fixture.state.result, HtmlScriptResult::Uninitialized);
        assert_eq!(fixture.state.ready_steps, None);
    }
}

#[test]
fn document_mismatch_is_checked_after_mark_and_preparation_write_before_scripting_evidence() {
    let mut fixture = Fixture::parser();
    fixture.node_document = Document::Adopted;
    fixture.scripting = HtmlScriptPreparationReply::Unavailable;
    assert_eq!(
        fixture.run(),
        Ok(HtmlScriptPreparationResult::ParserDocumentMismatch)
    );
    assert_eq!(fixture.state.parser, Some(Document::Parser));
    assert_eq!(fixture.state.preparation, Some(Document::Adopted));
    assert_eq!(fixture.state.kind, Some(HtmlScriptKind::Classic));
    assert!(fixture.state.flags.already_started());
    assert!(!fixture.state.flags.force_async());
}

#[test]
fn absent_parser_document_skips_restoration_and_preserves_native_attribute_effects() {
    for async_present in [false, true] {
        let mut fixture = Fixture::parser();
        fixture.state.parser = None;
        fixture.state.flags = HtmlScriptFlags::fresh();
        fixture.async_present = async_present;
        if async_present {
            fixture.state.flags.async_attribute_added();
        }
        assert_eq!(
            fixture.run(),
            Ok(HtmlScriptPreparationResult::Disabled(
                HtmlScriptKind::Classic
            ))
        );
        assert_eq!(fixture.state.parser, None);
        assert_eq!(fixture.state.preparation, Some(Document::Parser));
        assert_eq!(fixture.state.flags.force_async(), !async_present);
        assert!(fixture.state.flags.already_started());
        assert!(
            fixture
                .writes
                .iter()
                .all(|(action, _)| *action != HtmlScriptPreparationAction::RestoreParserDocument)
        );
    }
}

#[test]
fn present_empty_src_passes_empty_text_gate_without_loading_or_result_effects() {
    let mut fixture = Fixture::parser();
    fixture.text_empty = true;
    fixture.src_present = true; // The actual value can be the empty string.
    assert_eq!(
        fixture.run(),
        Ok(HtmlScriptPreparationResult::Disabled(
            HtmlScriptKind::Classic
        ))
    );
    assert_eq!(
        fixture.state.flags,
        HtmlScriptFlags::from_retained(false, true, false, false, false)
    );
    assert_eq!(fixture.state.result, HtmlScriptResult::Uninitialized);
    assert_eq!(fixture.state.ready_steps, None);
}

#[test]
fn missing_or_enabled_scripting_never_produces_a_disabled_success_or_loses_pending_action() {
    for (observation, expected) in [
        (
            HtmlScriptPreparationReply::Unavailable,
            HtmlScriptPreparationError::EvidenceUnavailable,
        ),
        (
            HtmlScriptPreparationReply::Enabled,
            HtmlScriptPreparationError::ScriptingEnabled,
        ),
    ] {
        let mut fixture = Fixture::parser();
        fixture.scripting = observation;
        fixture.until(HtmlScriptPreparationAction::RequireDisabledScripting);
        let state = fixture.state.clone();
        assert_eq!(fixture.step(), Err(expected));
        assert_eq!(
            fixture.cursor.action(),
            HtmlScriptPreparationAction::RequireDisabledScripting
        );
        assert_eq!(fixture.state, state);
    }
}

#[test]
fn missing_connectedness_and_unfinished_classification_never_become_false_or_unrecognized() {
    for action in [
        HtmlScriptPreparationAction::ReadConnected,
        HtmlScriptPreparationAction::ClassifyType,
    ] {
        let mut fixture = Fixture::parser();
        fixture.until(action);
        let state = fixture.state.clone();
        assert_eq!(
            fixture
                .cursor
                .reply(HtmlScriptPreparationReply::Unavailable),
            Err(HtmlScriptPreparationError::EvidenceUnavailable)
        );
        assert_eq!(fixture.cursor.action(), action);
        assert_eq!(fixture.state, state);
        assert!(!fixture.state.flags.already_started());
        assert_eq!(fixture.state.preparation, None);
    }
}

#[test]
fn wrong_acknowledgement_and_reply_after_completion_preserve_the_pending_state() {
    let mut cursor = HtmlScriptPreparationCursor::new();
    assert_eq!(
        cursor.reply(HtmlScriptPreparationReply::Applied),
        Err(HtmlScriptPreparationError::ReplyMismatch)
    );
    assert_eq!(
        cursor.action(),
        HtmlScriptPreparationAction::ReadAlreadyStarted
    );
    cursor
        .reply(HtmlScriptPreparationReply::AlreadyStarted(true))
        .unwrap();
    let complete = cursor.action();
    assert_eq!(
        cursor.reply(HtmlScriptPreparationReply::Applied),
        Err(HtmlScriptPreparationError::ReplyMismatch)
    );
    assert_eq!(cursor.action(), complete);
}
