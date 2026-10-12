// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tree_builder/rules.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf: BeforeHead, ordinary InHead,
// Disabled InHeadNoscript, Text and constructor-proved Disabled ScriptData. HTML
// 877d6146487f15b15bc8dd0e3c625063a37b456a governs behavior and PI.

use super::{
    KernelError, LocationKernel, copy_node,
    formatting::FormattingEntry,
    prologue::{BeforeHead, Payload, PendingToken, PrologueError},
    tag_sets, write,
};
use crate::{
    HtmlParseResource, HtmlSource, HtmlSourceRange,
    construction::parser_session::{
        BodyTextRuns, ParserNode, SessionError, TextRun, split_leading_ascii_whitespace,
        validate_token_kind,
    },
    parser::{
        control::{ControlIdentity, Invariant, OwnedVec, ParseControl, ParseStop, WorkCategory},
        tokenizer::{
            Tokenizer, TokenizerHandoffError,
            states::{Raw, State},
            token::{Event, TagKind, Text, Token},
        },
    },
};
use std::mem::size_of;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InsertionMode {
    BeforeHead,
    InHead,
    InHeadNoscript,
    Text,
    AfterHead,
    InBody,
    AfterBody,
    AfterAfterBody,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InFrameset,
    AfterFrameset,
    AfterAfterFrameset,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HeadCondition {
    UnexpectedDoctype,
    UnexpectedEndTag,
    UnexpectedHead,
    UnexpectedHtml,
    UnexpectedInNoscript,
    UnacknowledgedSelfClosing,
    EofInText,
    UnexpectedBodyToken,
    NullInBody,
    UnexpectedOpenElement,
    UnexpectedFormatting,
    UnexpectedTableToken,
    NonWhitespaceInTable,
    NullInTable,
    UnexpectedFramesetToken,
    NullInFrameset,
    UnexpectedForeignToken,
    ForeignNamespaceMismatch,
    NullInForeign,
}
pub(super) struct HeadDiagnostic {
    pub(super) condition: HeadCondition,
    pub(super) mode: InsertionMode,
    pub(super) source: HtmlSourceRange,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadBoundary {
    AfterHead,
    Script,
    Template,
    Form,
    Foreign,
    ConstructionEof,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadStep {
    Consumed,
    Boundary(HeadBoundary),
}
pub(crate) enum HeadError {
    Kernel(KernelError),
    Prologue(PrologueError),
    Handoff(TokenizerHandoffError),
    MissingInput,
    MissingTokenizer,
    Consumed,
    AlreadyAtBoundary,
    NotAtBoundary,
    TokenizerExhausted,
    InvalidTextToken,
    MissingReturnMode,
    StandaloneCapabilityUnavailable,
    UnsupportedScriptPause,
    UnsupportedPendingScript,
    InvalidScriptNesting,
    InvalidFormattingState,
}
impl std::fmt::Debug for HeadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kernel(error) => f.debug_tuple("Kernel").field(error).finish(),
            Self::Prologue(error) => f.debug_tuple("Prologue").field(error).finish(),
            Self::Handoff(error) => f.debug_tuple("Handoff").field(error).finish(),
            Self::MissingInput => f.write_str("MissingInput"),
            Self::MissingTokenizer => f.write_str("MissingTokenizer"),
            Self::Consumed => f.write_str("Consumed"),
            Self::AlreadyAtBoundary => f.write_str("AlreadyAtBoundary"),
            Self::NotAtBoundary => f.write_str("NotAtBoundary"),
            Self::TokenizerExhausted => f.write_str("TokenizerExhausted"),
            Self::InvalidTextToken => f.write_str("InvalidTextToken"),
            Self::MissingReturnMode => f.write_str("MissingReturnMode"),
            Self::StandaloneCapabilityUnavailable => f.write_str("StandaloneCapabilityUnavailable"),
            Self::UnsupportedScriptPause => f.write_str("UnsupportedScriptPause"),
            Self::UnsupportedPendingScript => f.write_str("UnsupportedPendingScript"),
            Self::InvalidScriptNesting => f.write_str("InvalidScriptNesting"),
            Self::InvalidFormattingState => f.write_str("InvalidFormattingState"),
        }
    }
}
impl From<KernelError> for HeadError {
    fn from(value: KernelError) -> Self {
        Self::Kernel(value)
    }
}
impl From<PrologueError> for HeadError {
    fn from(value: PrologueError) -> Self {
        Self::Prologue(value)
    }
}
impl From<ParseStop> for HeadError {
    fn from(value: ParseStop) -> Self {
        Self::Kernel(value.into())
    }
}
impl From<SessionError> for HeadError {
    fn from(value: SessionError) -> Self {
        Self::Kernel(value.into())
    }
}
impl From<TokenizerHandoffError> for HeadError {
    fn from(value: TokenizerHandoffError) -> Self {
        Self::Handoff(value)
    }
}

#[derive(Clone, Copy)]
enum Name {
    Html,
    Head,
    Body,
    Br,
    Base,
    Basefont,
    Bgsound,
    Link,
    Meta,
    Title,
    Noscript,
    Noframes,
    Style,
    Script,
    Template,
    Other,
}
#[derive(Clone, Copy)]
enum Class {
    Text,
    Comment,
    Pi,
    Doctype,
    Start(Name, bool),
    End(Name),
    Eof,
}

/// Actual source-initial state of the sealed ordinary Disabled execution owner.
/// No pending script producer is reached by the Disabled preparation prefix.
pub(super) struct StandaloneScriptState<'source> {
    pub(super) capability: super::StandaloneCapability<'source>,
    nesting: u32,
    pause: bool,
    insertion_point: Option<HtmlSourceRange>,
    saved_insertion_point: Option<Option<HtmlSourceRange>>,
    pending_blocking: Option<ParserNode>,
}

/// All state belongs to this one driver and the existing flat arena. A proofless
/// script or template start suspends without insertion effects. The sealed fresh
/// driver retains actual script parser state and metadata in the same arena.
pub(super) struct HeadState<'source> {
    pub(super) before: BeforeHead,
    pub(super) tokenizer: Option<Tokenizer<'source>>,
    pub(super) pending: Option<PendingToken>,
    pub(super) mode: InsertionMode,
    pub(super) head: Option<ParserNode>,
    // Initial ordinary document pointer; no captured/fragment form is guessed.
    pub(super) current_form: Option<ParserNode>,
    pub(super) return_mode: Option<InsertionMode>,
    pub(super) boundary: Option<HeadBoundary>,
    pub(super) diagnostics: OwnedVec<HeadDiagnostic>,
    pub(super) lexical: OwnedVec<Event>,
    lexical_pending: Option<Event>,
    pub(super) whitespace_pending: Option<Text>,
    pub(super) frameset_ok: bool,
    pub(super) ignore_first_lf: bool,
    pub(super) temporary_head: Option<ParserNode>,
    pub(super) body_runs: Option<BodyTextRuns>,
    pub(super) body_run: Option<TextRun>,
    pub(super) formatting: OwnedVec<FormattingEntry>,
    pub(super) formatting_pending: Option<FormattingEntry>,
    pub(super) formatting_removed: Option<FormattingEntry>,
    pub(super) table: super::tables::TableCharacters,
    pub(super) script: Option<StandaloneScriptState<'source>>,
}
/// Source-established ordinary offline/Disabled head construction. No imported
/// host, srcdoc, mode-fixed state or execution mode can be supplied here.
pub(super) struct HeadStage<'source> {
    source: &'source HtmlSource,
    owner: ControlIdentity,
    pub(super) state: Option<HeadState<'source>>,
}
/// Unfinished construction, tokenizer and source evidence for actual subsequent
/// dispatch. This is not EOF acceptance, a parse result or a complete report.
pub(super) struct HeadContinuation<'source> {
    pub(super) boundary: HeadBoundary,
    pub(super) driver: HeadStage<'source>,
}
impl<'source> HeadStage<'source> {
    /// Preflight both real borrowed owners before any take. The tokenizer owner
    /// verifies a paused Data-state handoff without resetting or pulling it.
    pub(super) fn new(
        source: &'source HtmlSource,
        before: &mut Option<BeforeHead>,
        tokenizer: &mut Option<Tokenizer<'source>>,
        c: &mut ParseControl,
    ) -> Result<Self, HeadError> {
        c.require_source(source)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let input = before.as_mut().ok_or(HeadError::MissingInput)?;
        input.kernel.require(c)?;
        input.diagnostics.require_owner(c)?;
        input.whitespace.require_owner(c)?;
        if let Some(token) = &input.input {
            validate_source(source, token, c)?;
        }
        tokenizer
            .as_ref()
            .ok_or(HeadError::MissingTokenizer)?
            .validate_head_handoff(source, c)?;
        let mut diagnostics = OwnedVec::new(HtmlParseResource::Diagnostics)?;
        let mut lexical = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut formatting = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        diagnostics.require_owner(c)?;
        lexical.require_owner(c)?;
        formatting.require_owner(c)?;
        let table = super::tables::TableCharacters::new(c)?;
        // Three driver fields + twenty-two state fields + caller BeforeHead and
        // Tokenizer slots + the retained BeforeHead input slot are written.
        // The table constructor separately admits its four actual field writes.
        // Admit all twenty-eight enclosing field writes before the first owning take.
        if let Err(stop) = write(size_of::<Self>() + size_of::<HeadState>(), 28, c) {
            diagnostics.release(c)?;
            lexical.release(c)?;
            formatting.release(c)?;
            table.entries.release(c)?;
            return Err(stop.into());
        }
        if let Err(stop) = c.charge(WorkCategory::ProviderCall, 1) {
            diagnostics.release(c)?;
            lexical.release(c)?;
            formatting.release(c)?;
            table.entries.release(c)?;
            return Err(stop.into());
        }
        let mut before = before.take().expect("verified prologue continuation");
        let pending = before.input.take().map(PendingToken::from_token);
        Ok(Self {
            source,
            owner: c.identity(),
            state: Some(HeadState {
                before,
                tokenizer: tokenizer.take(),
                pending,
                mode: InsertionMode::BeforeHead,
                head: None,
                current_form: None,
                return_mode: None,
                boundary: None,
                diagnostics,
                lexical,
                lexical_pending: None,
                whitespace_pending: None,
                script: None,
                frameset_ok: true,
                ignore_first_lf: false,
                temporary_head: None,
                body_runs: None,
                body_run: None,
                formatting,
                formatting_pending: None,
                formatting_removed: None,
                table,
            }),
        })
    }
    pub(super) fn install_standalone(
        &mut self,
        capability: super::StandaloneCapability<'source>,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        self.require(c)?;
        let state = self.state.as_mut().ok_or(HeadError::Consumed)?;
        capability.validate_session(c, &state.before.kernel.session)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        if state.mode != InsertionMode::BeforeHead || state.script.is_some() {
            return Err(HeadError::StandaloneCapabilityUnavailable);
        }
        write(size_of::<Option<StandaloneScriptState>>(), 7, c)?;
        state.script = Some(StandaloneScriptState {
            capability,
            nesting: 0,
            pause: false,
            insertion_point: None,
            saved_insertion_point: None,
            pending_blocking: None,
        });
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn standalone_parts(
        &mut self,
    ) -> Option<(
        &mut crate::construction::parser_session::ParserArenaSession,
        &super::StandaloneCapability<'source>,
    )> {
        let state = self.state.as_mut()?;
        Some((
            &mut state.before.kernel.session,
            &state.script.as_ref()?.capability,
        ))
    }
    #[cfg(test)]
    pub(super) fn script_observation(
        &self,
        c: &mut ParseControl,
    ) -> Result<(u32, bool, bool, bool, bool), HeadError> {
        self.require(c)?;
        let script = self
            .state
            .as_ref()
            .ok_or(HeadError::Consumed)?
            .script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?;
        c.charge(WorkCategory::RecordVisit, 5)?;
        c.charge(
            WorkCategory::InspectByte,
            (size_of::<u32>()
                + size_of::<bool>()
                + size_of::<Option<HtmlSourceRange>>()
                + size_of::<Option<Option<HtmlSourceRange>>>()
                + size_of::<Option<ParserNode>>()) as u64,
        )?;
        Ok((
            script.nesting,
            script.pause,
            script.insertion_point.is_some(),
            script.saved_insertion_point.is_some(),
            script.pending_blocking.is_some(),
        ))
    }
    /// Test-only extraction releases the actual lexer/stack/evidence first.
    /// On failure the borrowed owner remains available for terminal disposal.
    #[cfg(test)]
    pub(super) fn prepare_standalone_fixture(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        self.require(c)?;
        let state = self.state.as_mut().ok_or(HeadError::Consumed)?;
        state
            .script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?
            .capability
            .validate_session(c, &state.before.kernel.session)?;
        state.release_table(c)?;
        state.release_body(c)?;
        state.release_formatting(c)?;
        if let Some(pending) = &mut state.pending {
            pending.release(c)?;
        }
        super::dispose_token(&mut state.before.input, c).map_err(|error| match error {
            super::StandaloneError::Head(error) => error,
            super::StandaloneError::Stop(stop) => stop.into(),
            _ => HeadError::InvalidTextToken,
        })?;
        write(
            size_of::<Option<Text>>() + size_of::<Option<Event>>() + size_of::<Option<Tokenizer>>(),
            3,
            c,
        )?;
        if let Some(text) = state.whitespace_pending.take() {
            text.release(c)?;
        }
        if let Some(event) = state.lexical_pending.take() {
            event.release(c)?;
        }
        while let Some(event) = state.lexical.try_pop(c)? {
            event.release(c)?;
        }
        while let Some(text) = state.before.whitespace.try_pop(c)? {
            text.release(c)?;
        }
        state
            .before
            .kernel
            .open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .clear(c)?;
        if let Some(tokenizer) = state.tokenizer.take() {
            tokenizer.release(c)?;
        }
        release_fixture_vector(
            state
                .before
                .kernel
                .open
                .as_mut()
                .ok_or(KernelError::Consumed)?,
            HtmlParseResource::OpenElements,
            c,
        )?;
        release_fixture_vector(
            &mut state.table.entries,
            HtmlParseResource::AuxiliaryEntries,
            c,
        )?;
        release_fixture_vector(
            &mut state.formatting,
            HtmlParseResource::AuxiliaryEntries,
            c,
        )?;
        release_fixture_vector(
            &mut state.before.whitespace,
            HtmlParseResource::AuxiliaryEntries,
            c,
        )?;
        release_fixture_vector(
            &mut state.before.diagnostics,
            HtmlParseResource::Diagnostics,
            c,
        )?;
        release_fixture_vector(&mut state.diagnostics, HtmlParseResource::Diagnostics, c)?;
        release_fixture_vector(&mut state.lexical, HtmlParseResource::AuxiliaryEntries, c)?;
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn into_standalone_fixture(
        self,
    ) -> (
        crate::construction::parser_session::ParserArenaSession,
        super::StandaloneCapability<'source>,
    ) {
        let state = self.state.expect("prepared real standalone fixture");
        (
            state.before.kernel.session,
            state.script.expect("actual issued capability").capability,
        )
    }
    fn require(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        c.require_identity(&self.owner)?;
        c.require_source(self.source)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.state.as_ref().ok_or(HeadError::Consumed)?;
        Ok(())
    }
    /// Pulls only the actual tokenizer, after completing any pending reprocess.
    /// Lexical/preprocessing evidence remains owned; nothing is silently dropped.
    pub(super) fn step(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        self.require(c)?;
        let state = self.state.as_mut().ok_or(HeadError::Consumed)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(script) = &state.script {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, size_of::<bool>() as u64)?;
            if script.pause {
                return Err(HeadError::UnsupportedScriptPause);
            }
        }
        if state.boundary.is_some() {
            return Err(HeadError::AlreadyAtBoundary);
        }
        loop {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if state.pending.is_some() {
                break;
            }
            c.charge(WorkCategory::StateStep, 1)?;
            write(size_of::<Option<Event>>(), 1, c)?;
            // Namespace, including a foreign HTML integration point, controls
            // CDATA permission. Ordinary parsing has no fragment substitution.
            let cdata = state.foreign_namespace(c)?.is_some();
            state.lexical_pending = state
                .tokenizer
                .as_mut()
                .ok_or(HeadError::Consumed)?
                .next(c, cdata)?;
            let event = state
                .lexical_pending
                .as_ref()
                .ok_or(HeadError::TokenizerExhausted)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(token) = event.token() {
                validate_source(self.source, token, c)?;
                write(size_of::<Option<Event>>() + size_of::<PendingToken>(), 3, c)?;
                let Event::Token(event) = state.lexical_pending.take().expect("retained event")
                else {
                    unreachable!("actual token variant inspected")
                };
                state.pending = Some(PendingToken::from_token(event.into_token(c)?));
            } else {
                write(size_of::<Option<Event>>(), 1, c)?;
                let event = state
                    .lexical_pending
                    .take()
                    .expect("retained lexical event");
                if let Err((stop, event)) = state.lexical.try_push_recover(c, event) {
                    // Terminal recovery preserves ownership, without useful work.
                    state.lexical_pending = Some(event);
                    return Err(stop.into());
                }
            }
        }
        state.advance(self.source, c)
    }
    pub(super) fn take_continuation(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<HeadContinuation<'source>, HeadError> {
        self.require(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            size_of::<Option<HeadBoundary>>() as u64,
        )?;
        let boundary = self
            .state
            .as_ref()
            .ok_or(HeadError::Consumed)?
            .boundary
            .ok_or(HeadError::NotAtBoundary)?;
        write(
            size_of::<HeadContinuation>() + size_of::<Option<HeadState>>(),
            5,
            c,
        )?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        Ok(HeadContinuation {
            boundary,
            driver: Self {
                source: self.source,
                owner: c.identity(),
                state: self.state.take(),
            },
        })
    }
    pub(super) fn release(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        if c.stop().is_some() {
            c.cleanup_release(&self.owner, HtmlParseResource::AuxiliaryEntries, 0)?;
        } else {
            c.require_identity(&self.owner)?;
        }
        if c.stop().is_none() {
            // Normal owner transfer is admitted before mutation; terminal
            // disposal only deallocates/account-releases retained owners.
            write(
                size_of::<Option<HeadState>>()
                    + size_of::<Option<Tokenizer>>()
                    + size_of::<Option<Text>>()
                    + size_of::<Option<Event>>(),
                4,
                c,
            )?;
        }
        let state = self.state.as_mut().ok_or(HeadError::Consumed)?;
        if let Some(pending) = &mut state.pending {
            pending.release(c)?;
        }
        if let Some(prefix) = state.whitespace_pending.take() {
            prefix.release(c)?;
        }
        if let Some(event) = state.lexical_pending.take() {
            event.release(c)?;
        }
        loop {
            let event = if c.stop().is_some() {
                state.lexical.cleanup_pop(c)?
            } else {
                state.lexical.try_pop(c)?
            };
            let Some(event) = event else { break };
            event.release(c)?;
        }
        loop {
            let text = if c.stop().is_some() {
                state.before.whitespace.cleanup_pop(c)?
            } else {
                state.before.whitespace.try_pop(c)?
            };
            let Some(text) = text else { break };
            text.release(c)?;
        }
        state.release_table(c)?;
        state.release_body(c)?;
        state.release_formatting(c)?;
        state.before.kernel.release(c)?;
        if let Some(tokenizer) = state.tokenizer.take() {
            tokenizer.release(c)?;
        }
        // These empty vectors still retain admitted capacities. Move and release
        // the whole state only after alias-sensitive arena release succeeded.
        let state = self.state.take().expect("retained active state");
        state.before.diagnostics.release(c)?;
        state.before.whitespace.release(c)?;
        state.diagnostics.release(c)?;
        state.lexical.release(c)?;
        state.formatting.release(c)?;
        state.table.entries.release(c)?;
        Ok(())
    }
}
#[cfg(test)]
fn release_fixture_vector<T>(
    vector: &mut OwnedVec<T>,
    resource: HtmlParseResource,
    c: &mut ParseControl,
) -> Result<(), ParseStop> {
    vector.require_owner(c)?;
    write(size_of::<OwnedVec<T>>() * 2, 2, c)?;
    let empty = OwnedVec::new(resource)?;
    std::mem::replace(vector, empty).release(c)
}
fn validate_source(
    source: &HtmlSource,
    token: &Token,
    c: &mut ParseControl,
) -> Result<(), HeadError> {
    c.charge(WorkCategory::ProviderCall, 1)?;
    c.charge(WorkCategory::RecordVisit, 1)?;
    if token.source.source() != source.identity() {
        return Err(c.fail_invariant(Invariant::ForeignSource).into());
    }
    validate_token_kind(&token.kind, c)?;
    Ok(())
}
impl HeadState<'_> {
    pub(super) fn kernel(&mut self) -> &mut LocationKernel {
        &mut self.before.kernel
    }
    pub(super) fn mode(&self, c: &mut ParseControl) -> Result<InsertionMode, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, size_of::<InsertionMode>() as u64)?;
        Ok(self.mode)
    }
    pub(super) fn set_mode(
        &mut self,
        mode: InsertionMode,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        write(size_of::<InsertionMode>(), 1, c)?;
        self.mode = mode;
        Ok(())
    }
    pub(super) fn emit(
        &mut self,
        condition: HeadCondition,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let source = &self.pending.as_ref().ok_or(HeadError::MissingInput)?.source;
        write(size_of::<HeadDiagnostic>(), 3, c)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let mode = self.mode(c)?;
        self.diagnostics.try_push(
            c,
            HeadDiagnostic {
                condition,
                mode,
                source: source.clone(),
            },
        )?;
        Ok(())
    }
    pub(super) fn finish(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        write(size_of::<Option<PendingToken>>(), 1, c)?;
        self.pending = None;
        Ok(HeadStep::Consumed)
    }
    pub(super) fn discard(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        self.pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .release(c)?;
        self.finish(c)
    }
    pub(super) fn stop_at(
        &mut self,
        boundary: HeadBoundary,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        write(size_of::<Option<HeadBoundary>>(), 1, c)?;
        self.boundary = Some(boundary);
        Ok(HeadStep::Boundary(boundary))
    }
    fn classify(&self, c: &mut ParseControl) -> Result<Class, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let payload = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload;
        let class = match payload {
            Payload::Text(_) => Class::Text,
            Payload::Comment(_) => Class::Comment,
            Payload::Pi(_) => Class::Pi,
            Payload::Doctype(_) => Class::Doctype,
            Payload::Eof => Class::Eof,
            Payload::Tag(input) => {
                let tag = input.as_ref().ok_or(HeadError::MissingInput)?;
                c.require_document_string(tag.name.value.allocation(), tag.name.value.text())?;
                c.charge(WorkCategory::RecordVisit, 2)?;
                c.charge(
                    WorkCategory::InspectByte,
                    (size_of::<TagKind>() + size_of::<bool>()) as u64,
                )?;
                let kind = tag.kind;
                let self_closing = tag.self_closing;
                let mut selected = Name::Other;
                for (name, value) in [
                    ("html", Name::Html),
                    ("head", Name::Head),
                    ("body", Name::Body),
                    ("br", Name::Br),
                    ("base", Name::Base),
                    ("basefont", Name::Basefont),
                    ("bgsound", Name::Bgsound),
                    ("link", Name::Link),
                    ("meta", Name::Meta),
                    ("title", Name::Title),
                    ("noscript", Name::Noscript),
                    ("noframes", Name::Noframes),
                    ("style", Name::Style),
                    ("script", Name::Script),
                    ("template", Name::Template),
                ] {
                    c.charge(WorkCategory::StateStep, 1)?;
                    if tag_sets::ascii_eq(tag.name.value.text(), name, c)? {
                        selected = value;
                        break;
                    }
                }
                if kind == TagKind::Start {
                    Class::Start(selected, self_closing)
                } else {
                    Class::End(selected)
                }
            }
        };
        Ok(class)
    }
    pub(super) fn whitespace(&mut self, c: &mut ParseControl) -> Result<bool, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let mode = self.mode(c)?;
        let Payload::Text(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Ok(false);
        };
        write(size_of::<Option<Text>>(), 1, c)?;
        self.whitespace_pending = split_leading_ascii_whitespace(c, input)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        write(size_of::<bool>(), 1, c)?;
        let empty = input.is_none();
        if self.whitespace_pending.is_some() {
            if mode == InsertionMode::BeforeHead {
                write(size_of::<Option<Text>>(), 1, c)?;
                let text = self
                    .whitespace_pending
                    .take()
                    .expect("retained whitespace prefix");
                if let Err((stop, text)) = self.before.whitespace.try_push_recover(c, text) {
                    self.whitespace_pending = Some(text);
                    return Err(stop.into());
                }
            } else {
                // These trailing rows explicitly process whitespace using the
                // InBody rules while retaining their actual current mode.
                if matches!(
                    mode,
                    InsertionMode::AfterBody | InsertionMode::AfterAfterBody
                ) {
                    self.reconstruct_formatting(c)?;
                }
                self.before.kernel.insert_text(
                    &mut self.whitespace_pending,
                    None,
                    c,
                    self.script.as_ref().map(|script| &script.capability),
                )?;
            }
        }
        Ok(empty)
    }
    pub(super) fn ordinary_node(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        let location = self.before.kernel.adjusted_location(None, c)?;
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let node = match &mut pending.payload {
            Payload::Comment(input) => self.before.kernel.session.allocate_comment(c, input)?,
            Payload::Pi(input) => self
                .before
                .kernel
                .session
                .allocate_processing_instruction(c, input)?,
            _ => return Err(HeadError::MissingInput),
        };
        self.before.kernel.session.insert(
            c,
            &location.parent,
            &node,
            location.before.as_ref(),
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.finish(c)
    }
    pub(super) fn insert_tag(&mut self, c: &mut ParseControl) -> Result<ParserNode, HeadError> {
        let location = self.before.kernel.adjusted_location(None, c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Payload::Tag(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        let node = self.before.kernel.session.allocate_html_tag_for_creation(
            c,
            input,
            &location.parent,
            self.current_form.as_ref(),
        )?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        Ok(node)
    }
    pub(super) fn insert_hidden_input(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<ParserNode, HeadError> {
        let location = self.before.kernel.adjusted_location(None, c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let capability = &self
            .script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?
            .capability;
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let Payload::Tag(input) = &mut pending.payload else {
            return Err(HeadError::MissingInput);
        };
        let node = self
            .before
            .kernel
            .session
            .allocate_hidden_input_for_creation(
                c,
                input,
                &location.parent,
                self.current_form.as_ref(),
                &pending.source,
                capability,
            )?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        Ok(node)
    }
    pub(super) fn insert_select_family(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<ParserNode, HeadError> {
        let location = self.before.kernel.adjusted_location(None, c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let capability = &self
            .script
            .as_ref()
            .ok_or_else(|| {
                c.stop_unsupported_standalone(
                    crate::parser::control::StandaloneUnsupported::Provenance,
                )
            })?
            .capability;
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let Payload::Tag(input) = &mut pending.payload else {
            return Err(HeadError::MissingInput);
        };
        let node = self
            .before
            .kernel
            .session
            .allocate_select_family_for_creation(
                c,
                input,
                &location.parent,
                self.current_form.as_ref(),
                &pending.source,
                capability,
            )?;
        self.before
            .kernel
            .insert_element_at(&node, &location, c, Some(capability))?;
        self.before.kernel.push(&node, c)?;
        Ok(node)
    }
    fn set_head(&mut self, node: &ParserNode, c: &mut ParseControl) -> Result<(), HeadError> {
        let node = copy_node(node, c)?;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        self.head = Some(node);
        self.set_mode(InsertionMode::InHead, c)
    }
    fn implied_head(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        let location = self.kernel().adjusted_location(None, c)?;
        let node = self.kernel().session.allocate_implicit_head(c)?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.kernel().push(&node, c)?;
        self.set_head(&node, c)
    }
    pub(super) fn raw_text(
        &mut self,
        raw: Raw,
        self_closing: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.insert_tag(c)?;
        self.tokenizer
            .as_mut()
            .ok_or(HeadError::Consumed)?
            .set_state(State::Raw(raw), c)?;
        write(size_of::<Option<InsertionMode>>(), 1, c)?;
        let mode = self.mode(c)?;
        self.return_mode = Some(mode);
        self.set_mode(InsertionMode::Text, c)?;
        if self_closing {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        self.finish(c)
    }
    pub(super) fn validate_standalone_capability(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?
            .capability
            .validate_session(c, &self.before.kernel.session)?;
        Ok(())
    }
    pub(super) fn script_start(
        &mut self,
        self_closing: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let script = self
            .script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?;
        script
            .capability
            .validate_session(c, &self.before.kernel.session)?;
        let location = self.before.kernel.adjusted_location(None, c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Payload::Tag(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            unreachable!("classified actual script tag")
        };
        let node = self.before.kernel.session.allocate_disabled_parser_script(
            c,
            input,
            &script.capability,
        )?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.tokenizer
            .as_mut()
            .ok_or(HeadError::Consumed)?
            .set_state(State::Raw(Raw::Script), c)?;
        write(size_of::<Option<InsertionMode>>(), 1, c)?;
        let mode = self.mode(c)?;
        self.return_mode = Some(mode);
        self.set_mode(InsertionMode::Text, c)?;
        if self_closing {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        self.finish(c)
    }
    fn script_end(
        &mut self,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let script = self
            .script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, size_of::<u32>() as u64)?;
        // This closed owner cannot reenter. Nesting is not the standard's
        // checkpoint predicate: its speculative parser is absent and its JS
        // execution stack is absent/empty, so the checkpoint is mandatory.
        if script.nesting != 0 {
            return Err(HeadError::InvalidScriptNesting);
        }
        script
            .capability
            .checkpoint(c, &mut self.before.kernel.session)?;
        self.before
            .kernel
            .pop(c, self.script.as_ref().map(|script| &script.capability))?
            .ok_or(KernelError::EmptyStack)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            size_of::<Option<InsertionMode>>() as u64,
        )?;
        let mode = self.return_mode.ok_or(HeadError::MissingReturnMode)?;
        self.set_mode(mode, c)?;
        write(size_of::<Option<InsertionMode>>(), 1, c)?;
        self.return_mode = None;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let script = self
            .script
            .as_mut()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let next = self
            .tokenizer
            .as_ref()
            .ok_or(HeadError::Consumed)?
            .checked_next_input_position(script.capability.source, c)?;
        write(
            size_of::<Option<Option<HtmlSourceRange>>>() + size_of::<Option<HtmlSourceRange>>(),
            2,
            c,
        )?;
        script.saved_insertion_point = Some(script.insertion_point.take());
        script.insertion_point = Some(next);
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, size_of::<u32>() as u64)?;
        let nesting = script
            .nesting
            .checked_add(1)
            .ok_or(HeadError::InvalidScriptNesting)?;
        write(size_of::<u32>(), 1, c)?;
        script.nesting = nesting;
        self.before
            .kernel
            .session
            .prepare_disabled_script(c, node, &script.capability)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, size_of::<u32>() as u64)?;
        let nesting = script
            .nesting
            .checked_sub(1)
            .ok_or(HeadError::InvalidScriptNesting)?;
        write(size_of::<u32>(), 1, c)?;
        script.nesting = nesting;
        if nesting == 0 {
            write(size_of::<bool>(), 1, c)?;
            script.pause = false;
        }
        write(
            size_of::<Option<Option<HtmlSourceRange>>>() + size_of::<Option<HtmlSourceRange>>(),
            2,
            c,
        )?;
        script.insertion_point = script
            .saved_insertion_point
            .take()
            .ok_or(HeadError::InvalidScriptNesting)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if script.pending_blocking.is_some() {
            return Err(HeadError::UnsupportedPendingScript);
        }
        self.discard(c)
    }
    pub(super) fn html(
        &mut self,
        self_closing: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.emit(HeadCondition::UnexpectedHtml, c)?;
        if self.before.kernel.contains_html_template(c)? {
            return self.discard(c);
        }
        let top = self.before.kernel.first(c)?;
        let Payload::Tag(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        self.before
            .kernel
            .session
            .add_attributes_if_missing(c, &top, input)?;
        if self_closing {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        self.finish(c)
    }
    fn advance(
        &mut self,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let mode = self.mode(c)?;
            if self.foreign_dispatch(c)?
                && let Some(step) = self.foreign_rules(c)?
            {
                return Ok(step);
            }
            if matches!(
                mode,
                InsertionMode::AfterHead
                    | InsertionMode::InBody
                    | InsertionMode::AfterBody
                    | InsertionMode::AfterAfterBody
            ) {
                if let Some(step) = self.body_rules(mode, source, c)? {
                    return Ok(step);
                }
                continue;
            }
            if matches!(
                mode,
                InsertionMode::InTable
                    | InsertionMode::InTableText
                    | InsertionMode::InCaption
                    | InsertionMode::InColumnGroup
                    | InsertionMode::InTableBody
                    | InsertionMode::InRow
                    | InsertionMode::InCell
            ) {
                if let Some(step) = self.table_rules(mode, source, c)? {
                    return Ok(step);
                }
                continue;
            }
            if matches!(
                mode,
                InsertionMode::InFrameset
                    | InsertionMode::AfterFrameset
                    | InsertionMode::AfterAfterFrameset
            ) {
                return self.frameset_rules(mode, c);
            }
            if mode == InsertionMode::Text && self.first_lf(c)? {
                return self.finish(c);
            }
            if mode != InsertionMode::Text && self.whitespace(c)? {
                return self.finish(c);
            }
            let class = self.classify(c)?;
            if mode == InsertionMode::Text {
                match class {
                    Class::Text => {
                        let Payload::Text(input) = &mut self
                            .pending
                            .as_mut()
                            .ok_or(HeadError::MissingInput)?
                            .payload
                        else {
                            unreachable!("classified actual text")
                        };
                        self.before.kernel.insert_text(
                            input,
                            None,
                            c,
                            self.script.as_ref().map(|script| &script.capability),
                        )?;
                        return self.finish(c);
                    }
                    Class::End(_) | Class::Eof => {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        let top = self.before.kernel.at(
                            self.before
                                .kernel
                                .stack()?
                                .len()
                                .checked_sub(1)
                                .ok_or(KernelError::EmptyStack)?,
                            c,
                        )?;
                        let is_script = self.before.kernel.named(&top, "script", c)?;
                        if is_script && matches!(class, Class::End(Name::Script)) {
                            return self.script_end(&top, c);
                        }
                        if is_script && !matches!(class, Class::Eof) {
                            return Err(HeadError::InvalidTextToken);
                        }
                        if matches!(class, Class::Eof) {
                            self.emit(HeadCondition::EofInText, c)?;
                            if is_script {
                                c.charge(WorkCategory::RecordVisit, 1)?;
                                let script = self
                                    .script
                                    .as_ref()
                                    .ok_or(HeadError::StandaloneCapabilityUnavailable)?;
                                self.before.kernel.session.mark_script_eof(
                                    c,
                                    &top,
                                    &script.capability,
                                )?;
                            }
                        }
                        self.before
                            .kernel
                            .pop(c, self.script.as_ref().map(|script| &script.capability))?
                            .ok_or(KernelError::EmptyStack)?;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        c.charge(
                            WorkCategory::InspectByte,
                            size_of::<Option<InsertionMode>>() as u64,
                        )?;
                        let mode = self.return_mode.ok_or(HeadError::MissingReturnMode)?;
                        self.set_mode(mode, c)?;
                        write(size_of::<Option<InsertionMode>>(), 1, c)?;
                        self.return_mode = None;
                        if matches!(class, Class::Eof) {
                            continue;
                        }
                        return self.discard(c);
                    }
                    _ => return Err(HeadError::InvalidTextToken),
                }
            }
            if let Class::Start(Name::Html, close) = class {
                return self.html(close, c);
            }
            if matches!(class, Class::Doctype) {
                self.emit(HeadCondition::UnexpectedDoctype, c)?;
                return self.discard(c);
            }
            if mode == InsertionMode::BeforeHead {
                match class {
                    Class::Comment | Class::Pi => return self.ordinary_node(c),
                    Class::Start(Name::Head, close) => {
                        let node = self.insert_tag(c)?;
                        self.set_head(&node, c)?;
                        if close {
                            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
                        }
                        return self.finish(c);
                    }
                    Class::End(Name::Head | Name::Body | Name::Html | Name::Br) => {}
                    Class::End(_) => {
                        self.emit(HeadCondition::UnexpectedEndTag, c)?;
                        return self.discard(c);
                    }
                    _ => {}
                }
                self.implied_head(c)?;
                continue;
            }
            if mode == InsertionMode::InHeadNoscript {
                match class {
                    Class::End(Name::Noscript) => {
                        self.before
                            .kernel
                            .pop(c, self.script.as_ref().map(|script| &script.capability))?
                            .ok_or(KernelError::EmptyStack)?;
                        self.set_mode(InsertionMode::InHead, c)?;
                        return self.discard(c);
                    }
                    Class::Comment
                    | Class::Pi
                    | Class::Start(
                        Name::Basefont
                        | Name::Bgsound
                        | Name::Link
                        | Name::Meta
                        | Name::Noframes
                        | Name::Style,
                        _,
                    ) => {}
                    Class::Start(Name::Head | Name::Noscript, _) => {
                        self.emit(HeadCondition::UnexpectedInNoscript, c)?;
                        return self.discard(c);
                    }
                    Class::End(name) if !matches!(name, Name::Br) => {
                        self.emit(HeadCondition::UnexpectedInNoscript, c)?;
                        return self.discard(c);
                    }
                    _ => {
                        self.emit(HeadCondition::UnexpectedInNoscript, c)?;
                        self.before
                            .kernel
                            .pop(c, self.script.as_ref().map(|script| &script.capability))?
                            .ok_or(KernelError::EmptyStack)?;
                        self.set_mode(InsertionMode::InHead, c)?;
                        continue;
                    }
                }
            }
            match class {
                Class::Comment | Class::Pi => return self.ordinary_node(c),
                Class::Start(
                    Name::Base | Name::Basefont | Name::Bgsound | Name::Link | Name::Meta,
                    _,
                ) => {
                    self.insert_tag(c)?;
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                    // Unicode input gives irrelevant encoding confidence (§13.2.3,
                    // syntax line 791); no tentative meta encoding transition.
                    return self.finish(c);
                }
                Class::Start(Name::Title, close) => return self.raw_text(Raw::Rcdata, close, c),
                Class::Start(Name::Noframes | Name::Style, close) => {
                    return self.raw_text(Raw::Rawtext, close, c);
                }
                Class::Start(Name::Noscript, close) => {
                    self.insert_tag(c)?;
                    self.set_mode(InsertionMode::InHeadNoscript, c)?;
                    if close {
                        self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
                    }
                    return self.finish(c);
                }
                Class::Start(Name::Script, close) => {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if self.script.is_some() {
                        return self.script_start(close, c);
                    }
                    return self.stop_at(HeadBoundary::Script, c);
                }
                Class::Start(Name::Template, _) => return self.stop_at(HeadBoundary::Template, c),
                Class::End(Name::Head) => {
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                    self.set_mode(InsertionMode::AfterHead, c)?;
                    self.discard(c)?;
                    return self.stop_at(HeadBoundary::AfterHead, c);
                }
                Class::Start(Name::Head, _) => {
                    self.emit(HeadCondition::UnexpectedHead, c)?;
                    return self.discard(c);
                }
                Class::End(Name::Template) => {
                    // No template start is consumed by this stage; the actual
                    // stack scan still establishes the absent-template branch.
                    if self.before.kernel.contains_html_template(c)? {
                        return self.stop_at(HeadBoundary::Template, c);
                    }
                    self.emit(HeadCondition::UnexpectedEndTag, c)?;
                    return self.discard(c);
                }
                Class::End(Name::Body | Name::Html | Name::Br) => {}
                Class::End(_) => {
                    self.emit(HeadCondition::UnexpectedEndTag, c)?;
                    return self.discard(c);
                }
                _ => {}
            }
            self.before
                .kernel
                .pop(c, self.script.as_ref().map(|script| &script.capability))?
                .ok_or(KernelError::EmptyStack)?;
            self.set_mode(InsertionMode::AfterHead, c)?;
            return self.stop_at(HeadBoundary::AfterHead, c);
        }
    }
}

#[cfg(test)]
mod script_state_tests {
    use super::*;

    #[test]
    fn actual_standalone_owner_rejects_corrupted_nonzero_nesting_before_pop_and_preparation() {
        let request = crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<script>x</script>"),
            super::super::tests::limits(100),
        );
        let mut driver = super::super::StandaloneDocumentDriver::new(&request).unwrap();
        for _ in 0..100 {
            driver.step().unwrap();
            if driver
                .head
                .as_ref()
                .is_some_and(|head| head.state.as_ref().unwrap().mode == InsertionMode::Text)
            {
                break;
            }
        }
        driver.step().unwrap(); // actual script character token
        let state = driver.head.as_mut().unwrap().state.as_mut().unwrap();
        assert_eq!(state.mode, InsertionMode::Text);
        assert_eq!(state.script.as_ref().unwrap().nesting, 0);
        // Private-state fault injection tests the defense, without issuing a
        // capability, simulating script execution, or adding an input producer.
        state.script.as_mut().unwrap().nesting = 1;
        assert!(matches!(
            driver.step(),
            Err(super::super::StandaloneError::Head(
                HeadError::InvalidScriptNesting
            ))
        ));
        assert_eq!(
            driver.control.stop(),
            Some(ParseStop::UnsupportedStandaloneState(
                crate::parser::control::StandaloneUnsupported::InvalidNesting
            ))
        );
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(state.mode, InsertionMode::Text);
        assert_eq!(state.before.kernel.stack().unwrap().len(), 3);
        assert!(matches!(
            state.pending.as_ref().unwrap().payload,
            Payload::Tag(Some(_))
        ));
        driver.release().unwrap();
        super::super::tests::assert_released(&driver.control);
    }
}
