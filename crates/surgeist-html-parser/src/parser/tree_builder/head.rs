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
    construction::{
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
pub enum InsertionMode {
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
    InTemplate,
}
impl InsertionMode {
    pub(super) fn rule_anchor(self) -> &'static str {
        match self {
            Self::BeforeHead => "the-before-head-insertion-mode",
            Self::InHead => "parsing-main-inhead",
            Self::InHeadNoscript => "parsing-main-inheadnoscript",
            Self::Text => "parsing-main-incdata",
            Self::AfterHead => "the-after-head-insertion-mode",
            Self::InBody => "parsing-main-inbody",
            Self::AfterBody => "parsing-main-afterbody",
            Self::AfterAfterBody => "the-after-after-body-insertion-mode",
            Self::InTable => "parsing-main-intable",
            Self::InTableText => "parsing-main-intabletext",
            Self::InCaption => "parsing-main-incaption",
            Self::InColumnGroup => "parsing-main-incolgroup",
            Self::InTableBody => "parsing-main-intbody",
            Self::InRow => "parsing-main-intr",
            Self::InCell => "parsing-main-intd",
            Self::InFrameset => "parsing-main-inframeset",
            Self::AfterFrameset => "parsing-main-afterframeset",
            Self::AfterAfterFrameset => "the-after-after-frameset-insertion-mode",
            Self::InTemplate => "parsing-main-intemplate",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadCondition {
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
#[cfg(test)]
pub(super) struct HeadDiagnostic {
    pub(super) condition: HeadCondition,
    pub(super) mode: InsertionMode,
    pub(super) source: HtmlSourceRange,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadBoundary {
    AfterHead,
    Script,
    Template,
    Form,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadStep {
    NeedInput,
    Consumed,
    Complete,
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
    #[cfg(test)]
    NotAtBoundary,
    TokenizerExhausted,
    InvalidTextToken,
    MissingReturnMode,
    ConstructionCapabilityUnavailable,
    UnsupportedScriptPause,
    UnsupportedPendingScript,
    InvalidScriptNesting,
    InvalidFormattingState,
    IncompleteCompletion,
}
impl HeadError {
    /// A stage, kernel or prologue owner its driver keeps is gone, or the
    /// formatting list holds a state its own sequencing cannot produce.
    pub(super) fn lost_owner(&self) -> bool {
        match self {
            Self::Consumed | Self::MissingInput | Self::MissingTokenizer => true,
            Self::InvalidFormattingState => true,
            Self::Kernel(KernelError::Consumed) => true,
            Self::Prologue(error) => error.lost_owner(),
            _ => false,
        }
    }
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
            #[cfg(test)]
            Self::NotAtBoundary => f.write_str("NotAtBoundary"),
            Self::TokenizerExhausted => f.write_str("TokenizerExhausted"),
            Self::InvalidTextToken => f.write_str("InvalidTextToken"),
            Self::MissingReturnMode => f.write_str("MissingReturnMode"),
            Self::ConstructionCapabilityUnavailable => {
                f.write_str("ConstructionCapabilityUnavailable")
            }
            Self::UnsupportedScriptPause => f.write_str("UnsupportedScriptPause"),
            Self::UnsupportedPendingScript => f.write_str("UnsupportedPendingScript"),
            Self::InvalidScriptNesting => f.write_str("InvalidScriptNesting"),
            Self::IncompleteCompletion => f.write_str("IncompleteCompletion"),
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
    pub(super) capability: super::ConstructionCapability<'source>,
    nesting: u32,
    pause: bool,
    insertion_point: Option<HtmlSourceRange>,
    saved_insertion_point: Option<Option<HtmlSourceRange>>,
    pending_blocking: Option<ParserNode>,
}

/// All state belongs to this one driver and the existing flat arena. A proofless
/// script or template start suspends without insertion effects. The sealed fresh
/// driver retains actual script parser state and metadata in the same arena.
/// One source-defined form pointer; captured forms are qualified evidence,
/// never native nodes in the construction arena.
pub(super) enum FormPointer {
    Owned(ParserNode),
    Captured(crate::HtmlCaptureElementId),
}
impl FormPointer {
    pub(super) fn owned(&self) -> Option<&ParserNode> {
        match self {
            Self::Owned(node) => Some(node),
            Self::Captured(_) => None,
        }
    }
}

pub(super) struct HeadState<'source> {
    pub(super) before: BeforeHead,
    pub(super) tokenizer: Option<Tokenizer<'source>>,
    pub(super) pending: Option<PendingToken>,
    pub(super) mode: InsertionMode,
    pub(super) head: Option<ParserNode>,
    // Initial ordinary document pointer; no captured/fragment form is guessed.
    pub(super) current_form: Option<FormPointer>,
    pub(super) return_mode: Option<InsertionMode>,
    pub(super) boundary: Option<HeadBoundary>,
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
    pub(super) template_modes: OwnedVec<InsertionMode>,
    pub(super) script: Option<StandaloneScriptState<'source>>,
    pub(super) svg_pending: Option<ParserNode>,
    pub(super) end: Option<super::end::EndPhase>,
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
#[cfg(test)]
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
        input.whitespace.require_owner(c)?;
        if let Some(token) = &input.input {
            validate_source(source, token, c)?;
        }
        tokenizer
            .as_ref()
            .ok_or(HeadError::MissingTokenizer)?
            .validate_head_handoff(source, c)?;
        let mut lexical = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut formatting = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        lexical.require_owner(c)?;
        formatting.require_owner(c)?;
        let table = super::tables::TableCharacters::new(c)?;
        let mut template_modes = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        template_modes.require_owner(c)?;
        // Preserve the existing enclosing ownership-transfer admission and add
        // the actual end-continuation field, removing the former diagnostic vector. The table constructor separately
        // admits its four field writes. All admissions precede the first take.
        if let Err(stop) = write(const { size_of::<Self>() + size_of::<HeadState>() }, 29, c) {
            lexical.release(c)?;
            formatting.release(c)?;
            table.entries.release(c)?;
            template_modes.release(c)?;
            return Err(stop.into());
        }
        if let Err(stop) = c.charge(WorkCategory::ProviderCall, 1) {
            lexical.release(c)?;
            formatting.release(c)?;
            table.entries.release(c)?;
            template_modes.release(c)?;
            return Err(stop.into());
        }
        let Some(mut before) = before.take() else {
            lexical.release(c)?;
            formatting.release(c)?;
            table.entries.release(c)?;
            template_modes.release(c)?;
            return Err(HeadError::MissingInput);
        };
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

                lexical,
                lexical_pending: None,
                whitespace_pending: None,
                script: None,
                svg_pending: None,
                end: None,
                frameset_ok: true,
                ignore_first_lf: false,
                temporary_head: None,
                body_runs: None,
                body_run: None,
                formatting,
                formatting_pending: None,
                formatting_removed: None,
                table,
                template_modes,
            }),
        })
    }
    pub(super) fn install_standalone(
        &mut self,
        capability: super::ConstructionCapability<'source>,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        self.require(c)?;
        let state = self.state.as_mut().ok_or(HeadError::Consumed)?;
        capability.validate_session(c, &state.before.kernel.session)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        if state.mode != InsertionMode::BeforeHead || state.script.is_some() {
            return Err(HeadError::ConstructionCapabilityUnavailable);
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
        &mut crate::construction::ParserArenaSession,
        &super::ConstructionCapability<'source>,
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
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
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
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?
            .capability
            .validate_session(c, &state.before.kernel.session)?;
        if state.svg_pending.is_some() {
            return Err(HeadError::UnsupportedScriptPause);
        }
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
        if let Some(templates) = state.before.kernel.templates.as_mut() {
            templates.clear(c)?;
            release_fixture_vector(templates, HtmlParseResource::AuxiliaryEntries, c)?;
        }
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
            &mut state.template_modes,
            HtmlParseResource::AuxiliaryEntries,
            c,
        )?;
        release_fixture_vector(
            &mut state.before.whitespace,
            HtmlParseResource::AuxiliaryEntries,
            c,
        )?;

        release_fixture_vector(&mut state.lexical, HtmlParseResource::AuxiliaryEntries, c)?;
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn into_standalone_fixture(
        self,
    ) -> (
        crate::construction::ParserArenaSession,
        super::ConstructionCapability<'source>,
    ) {
        let state = self.state.expect("prepared real standalone fixture");
        (
            state.before.kernel.session,
            state.script.expect("actual issued capability").capability,
        )
    }
    pub(super) fn require(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        c.require_identity(&self.owner)?;
        c.require_source(self.source)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.state.as_ref().ok_or(HeadError::Consumed)?;
        Ok(())
    }
    /// Pulls only the actual tokenizer, after completing any pending reprocess.
    /// Lexical/preprocessing evidence remains owned; nothing is silently dropped.
    pub(super) fn step(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        self.step_with_meta_csp(c, &mut None)
    }
    pub(super) fn step_with_meta_csp(
        &mut self,
        c: &mut ParseControl,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<HeadStep, HeadError> {
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
        if state.svg_pending.is_some() {
            return Err(HeadError::UnsupportedScriptPause);
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        if state.end.is_some() {
            return state.end_step(c);
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
            let evidence = state
                .before
                .kernel
                .session
                .evidence(c)
                .map_err(KernelError::from)?;
            let pulled = state
                .tokenizer
                .as_mut()
                .ok_or(HeadError::Consumed)?
                .next(c, cdata, evidence)?;
            match pulled {
                super::super::tokenizer::TokenizerPull::Event(event) => {
                    state.lexical_pending = Some(event)
                }
                super::super::tokenizer::TokenizerPull::NeedInput => {
                    return Ok(HeadStep::NeedInput);
                }
                super::super::tokenizer::TokenizerPull::Exhausted => {
                    return Err(HeadError::TokenizerExhausted);
                }
            }
            let event = state
                .lexical_pending
                .as_ref()
                .ok_or(HeadError::TokenizerExhausted)?;
            if let Event::Diagnostic(id) = event {
                state
                    .before
                    .kernel
                    .session
                    .evidence(c)
                    .map_err(KernelError::from)?
                    .get(c, id)?;
            }
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(token) = event.token(c)? {
                validate_source(self.source, token, c)?;
                write(
                    const { size_of::<Option<Event>>() + size_of::<PendingToken>() },
                    4,
                    c,
                )?;
                let event = super::take_token_event(&mut state.lexical_pending, c)?;
                state.pending = Some(PendingToken::from_token(event.into_token(c)?));
                // HTML877d pre/listing/textarea: the initial-LF rule applies to the
                // token immediately after the start tag, whichever mode handles it.
                if state.ignore_first_lf && state.first_lf(c)? {
                    return state.finish(c);
                }
            } else {
                state.lexical.reserve_geometric(c, 1)?;
                write(size_of::<Option<Event>>(), 1, c)?;
                let event = state
                    .lexical_pending
                    .take()
                    .ok_or(HeadError::TokenizerExhausted)?;
                if let Err((stop, event)) = state.lexical.try_push_recover(c, event) {
                    // Terminal recovery preserves ownership, without useful work.
                    state.lexical_pending = Some(event);
                    return Err(stop.into());
                }
            }
        }
        state.advance(self.source, c, service)
    }
    #[cfg(test)]
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
        self.release_owners(c, true)
    }
    pub(super) fn release_frozen_execution(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        self.release_owners(c, false)
    }
    fn release_owners(
        &mut self,
        c: &mut ParseControl,
        active_session: bool,
    ) -> Result<(), HeadError> {
        if c.stop().is_some() {
            c.cleanup_release(&self.owner, HtmlParseResource::AuxiliaryEntries, 0)?;
        } else {
            c.require_identity(&self.owner)?;
        }
        if c.stop().is_none() {
            // Normal owner transfer is admitted before mutation; terminal
            // disposal only deallocates/account-releases retained owners.
            write(
                const {
                    size_of::<Option<HeadState>>()
                        + size_of::<Option<Tokenizer>>()
                        + size_of::<Option<Text>>()
                        + size_of::<Option<Event>>()
                },
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
        if c.stop().is_none() {
            write(size_of::<Option<ParserNode>>(), 1, c)?;
        }
        state.svg_pending = None;
        if active_session {
            state.before.kernel.release(c)?;
        } else {
            state.before.kernel.release_frozen_stack(c)?;
        }
        if let Some(tokenizer) = state.tokenizer.take() {
            tokenizer.release(c)?;
        }
        // These empty vectors still retain admitted capacities. Move and release
        // the whole state only after alias-sensitive arena release succeeded.
        let state = self.state.take().ok_or(HeadError::Consumed)?;
        state.before.whitespace.release(c)?;
        state.lexical.release(c)?;
        state.formatting.release(c)?;
        state.table.entries.release(c)?;
        state.template_modes.release(c)?;
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
    pub(super) fn record_trigger(
        &mut self,
        rule: crate::construction::provenance::ParserRule,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        let pending = self.pending.as_ref().ok_or(HeadError::MissingInput)?;
        self.before.kernel.session.record_parser_trigger(
            c,
            crate::construction::provenance::ParserTrigger {
                source: &pending.source,
                rule,
                family: pending.family,
            },
        )?;
        Ok(())
    }
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
        rule: crate::construction::evidence::DiagnosticRule,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        c.charge(
            WorkCategory::InspectByte,
            size_of::<crate::construction::evidence::DiagnosticToken>() as u64,
        )?;
        let mode = self.mode(c)?;
        let pending = self.pending.as_ref().ok_or(HeadError::MissingInput)?;
        let source = &pending.source;
        let family = pending.family;
        self.before.kernel.session.evidence(c)?.append(
            c,
            crate::construction::evidence::DiagnosticKind::Tree(condition),
            crate::construction::evidence::DiagnosticContext::Tree(mode),
            rule,
            crate::construction::evidence::DiagnosticLocationInput {
                source,
                role: crate::construction::evidence::DiagnosticLocationRole::TokenEnvelope,
                token: Some(family),
            },
        )?;
        Ok(())
    }

    pub(super) fn finish(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        if let Some(pending) = &self.pending {
            self.before
                .kernel
                .session
                .evidence(c)?
                .lexical_disposition(c, pending.lexical.as_ref(), false)?;
        }
        write(size_of::<Option<PendingToken>>(), 1, c)?;
        self.pending = None;
        Ok(HeadStep::Consumed)
    }
    pub(super) fn discard(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        if let Some(pending) = &self.pending {
            let evidence = self.before.kernel.session.evidence(c)?;
            evidence.lexical_disposition(c, pending.lexical.as_ref(), true)?;
            if let Payload::Text(Some(text)) = &pending.payload {
                evidence.retain_omission(
                    c,
                    text.origins.as_slice(),
                    crate::construction::provenance::OmissionKind::IgnoredText,
                    crate::construction::provenance::ParserRule::Tree(self.mode),
                )?;
            }
        }
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
                    const { size_of::<TagKind>() + size_of::<bool>() } as u64,
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
                    .ok_or(HeadError::MissingInput)?;
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
            Payload::Comment(input) => {
                self.before
                    .kernel
                    .session
                    .allocate_comment(c, input, Some(&location.parent))?
            }
            Payload::Pi(input) => self.before.kernel.session.allocate_processing_instruction(
                c,
                input,
                Some(&location.parent),
            )?,
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
        self.insert_tag_with_meta_csp(c, &mut None)
    }
    pub(super) fn insert_tag_with_meta_csp(
        &mut self,
        c: &mut ParseControl,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<ParserNode, HeadError> {
        let place = self.before.kernel.appropriate_place(None, c)?;
        let template = self.before.kernel.last_template(c)?;
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
            &place.parent,
            self.current_form.as_ref().and_then(FormPointer::owned),
            template.as_ref().map_or(
                crate::construction::CreationContext::Ordinary,
                crate::construction::CreationContext::Template,
            ),
        )?;
        let location = self.before.kernel.adjusted_location(None, c)?;
        self.before.kernel.insert_element_at_with_meta_csp(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
            service,
        )?;
        self.before.kernel.push(&node, c)?;
        Ok(node)
    }
    pub(super) fn insert_input(&mut self, c: &mut ParseControl) -> Result<ParserNode, HeadError> {
        let place = self.before.kernel.appropriate_place(None, c)?;
        let template = self.before.kernel.last_template(c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let capability = &self
            .script
            .as_ref()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?
            .capability;
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let Payload::Tag(input) = &mut pending.payload else {
            return Err(HeadError::MissingInput);
        };
        let node = self.before.kernel.session.allocate_input_for_creation(
            c,
            input,
            crate::construction::CreationLocation {
                intended_parent: &place.parent,
                current_form: self.current_form.as_ref().and_then(FormPointer::owned),
                context: template.as_ref().map_or(
                    crate::construction::CreationContext::Ordinary,
                    crate::construction::CreationContext::Template,
                ),
            },
            &pending.source,
            capability,
        )?;
        let location = self.before.kernel.adjusted_location(None, c)?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        Ok(node)
    }
    pub(super) fn insert_details(&mut self, c: &mut ParseControl) -> Result<ParserNode, HeadError> {
        let place = self.before.kernel.appropriate_place(None, c)?;
        let template = self.before.kernel.last_template(c)?;
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
        let node = self.before.kernel.session.allocate_details_for_creation(
            c,
            input,
            crate::construction::CreationLocation {
                intended_parent: &place.parent,
                current_form: self.current_form.as_ref().and_then(FormPointer::owned),
                context: template.as_ref().map_or(
                    crate::construction::CreationContext::Ordinary,
                    crate::construction::CreationContext::Template,
                ),
            },
            &pending.source,
            capability,
        )?;
        let location = self.before.kernel.adjusted_location(None, c)?;
        self.before
            .kernel
            .insert_element_at(&node, &location, c, Some(capability))?;
        self.before.kernel.push(&node, c)?;
        Ok(node)
    }
    pub(super) fn insert_select_family(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<ParserNode, HeadError> {
        let place = self.before.kernel.appropriate_place(None, c)?;
        let template = self.before.kernel.last_template(c)?;
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
                crate::construction::CreationLocation {
                    intended_parent: &place.parent,
                    current_form: self.current_form.as_ref().and_then(FormPointer::owned),
                    context: template.as_ref().map_or(
                        crate::construction::CreationContext::Ordinary,
                        crate::construction::CreationContext::Template,
                    ),
                },
                &pending.source,
                capability,
            )?;
        let location = self.before.kernel.adjusted_location(None, c)?;
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
        let place = self.kernel().appropriate_place(None, c)?;
        let node = self
            .kernel()
            .session
            .allocate_implicit_head(c, Some(&place.parent))?;
        let location = self.kernel().adjusted_location(None, c)?;
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
            self.emit(
                HeadCondition::UnacknowledgedSelfClosing,
                crate::construction::evidence::DiagnosticRule {
                    anchor: "parsing-main-inhead",
                    branch: "raw-text-start/self-closing",
                },
                c,
            )?;
        }
        self.finish(c)
    }
    pub(super) fn qualify_end_script(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.validate_standalone_capability(c)?;
        let script = self
            .script
            .as_ref()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
        c.charge(WorkCategory::RecordVisit, 7)?;
        c.charge(
            WorkCategory::InspectByte,
            size_of::<StandaloneScriptState>() as u64,
        )?;
        // §13.4 seeds a template mode for a template context without pushing
        // the captured template. That original seed is legitimate at EOF.
        let context_template_seed = if self.template_modes.len() == 1
            && let Some(context) = self.before.kernel.session.fragment_context(c)?
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let node = context
                .capture()
                .node(context.context_element().subject())
                .map_err(|_| SessionError::ContextUnavailable)?;
            let crate::HtmlCapturedNodeKind::Element(element) = node.kind() else {
                return Err(KernelError::NotElement.into());
            };
            super::tag_sets::html_named(element.name(), "template", c)?
        } else {
            false
        };
        if script.nesting != 0
            || script.pause
            || script.pending_blocking.is_some()
            || script.saved_insertion_point.is_some()
            || self.svg_pending.is_some()
            || self.return_mode.is_some()
            || (self.template_modes.len() != 0 && !context_template_seed)
        {
            return Err(HeadError::IncompleteCompletion);
        }
        Ok(())
    }
    pub(super) fn end_insertion_point(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.qualify_end_script(c)?;
        write(size_of::<Option<HtmlSourceRange>>(), 1, c)?;
        self.script
            .as_mut()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?
            .insertion_point = None;
        Ok(())
    }
    pub(super) fn validate_standalone_capability(
        &self,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.script
            .as_ref()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?
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
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
        script
            .capability
            .validate_session(c, &self.before.kernel.session)?;
        let place = self.before.kernel.appropriate_place(None, c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Payload::Tag(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        let template = self.before.kernel.last_template(c)?;
        let context = template.as_ref().map_or(
            crate::construction::CreationContext::Ordinary,
            crate::construction::CreationContext::Template,
        );
        let node = self.before.kernel.session.allocate_disabled_parser_script(
            c,
            input,
            Some(&place.parent),
            context,
            &script.capability,
        )?;
        let location = self.before.kernel.adjusted_location(None, c)?;
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
            self.emit(
                HeadCondition::UnacknowledgedSelfClosing,
                crate::construction::evidence::DiagnosticRule {
                    anchor: "parsing-main-inhead",
                    branch: "script-start/self-closing",
                },
                c,
            )?;
        }
        self.finish(c)
    }
    /// HTML foreign SVG end ordering; no HTML Text-mode or microtask checkpoint.
    pub(super) fn svg_end(
        &mut self,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.svg_pending.is_some() {
            return Err(HeadError::UnsupportedScriptPause);
        }
        let capability = &self
            .script
            .as_ref()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?
            .capability;
        capability.validate_session(c, &self.before.kernel.session)?;
        // Retain before the fallible canonical pop and its intrinsic consequences.
        let retained = super::copy_node(node, c)?;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        self.svg_pending = Some(retained);
        self.before
            .kernel
            .pop(c, self.script.as_ref().map(|s| &s.capability))?
            .ok_or(KernelError::EmptyStack)?;
        let node = self
            .svg_pending
            .as_ref()
            .ok_or(HeadError::InvalidScriptNesting)?;
        let script = self
            .script
            .as_mut()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
        self.before
            .kernel
            .session
            .begin_svg_wrapper(c, node, &script.capability)?;
        let next = self
            .tokenizer
            .as_ref()
            .ok_or(HeadError::Consumed)?
            .checked_next_input_position(script.capability.source, c)?;
        write(
            const { size_of::<Option<Option<HtmlSourceRange>>>() + size_of::<Option<HtmlSourceRange>>() },
            2,
            c,
        )?;
        script.saved_insertion_point = Some(script.insertion_point.take());
        script.insertion_point = Some(next);
        c.charge(WorkCategory::InspectByte, size_of::<u32>() as u64)?;
        let nesting = script
            .nesting
            .checked_add(1)
            .ok_or(HeadError::InvalidScriptNesting)?;
        write(size_of::<u32>(), 1, c)?;
        script.nesting = nesting;
        write(size_of::<bool>(), 1, c)?;
        script.pause = true;
        self.before
            .kernel
            .session
            .process_svg_wrapper(c, node, &script.capability)?;
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
            const { size_of::<Option<Option<HtmlSourceRange>>>() + size_of::<Option<HtmlSourceRange>>() },
            2,
            c,
        )?;
        script.insertion_point = script
            .saved_insertion_point
            .take()
            .ok_or(HeadError::InvalidScriptNesting)?;
        self.before.kernel.session.finish_svg_wrapper(c, node)?;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        self.svg_pending = None;
        self.discard(c)
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
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
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
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let next = self
            .tokenizer
            .as_ref()
            .ok_or(HeadError::Consumed)?
            .checked_next_input_position(script.capability.source, c)?;
        write(
            const { size_of::<Option<Option<HtmlSourceRange>>>() + size_of::<Option<HtmlSourceRange>>() },
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
            const { size_of::<Option<Option<HtmlSourceRange>>>() + size_of::<Option<HtmlSourceRange>>() },
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
        self.record_trigger(
            crate::construction::provenance::ParserRule::Tree(InsertionMode::InBody),
            c,
        )?;
        self.emit(
            HeadCondition::UnexpectedHtml,
            crate::construction::evidence::DiagnosticRule {
                anchor: "parsing-main-inbody",
                branch: "html-start/delegate-in-body",
            },
            c,
        )?;
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
            self.emit(
                HeadCondition::UnacknowledgedSelfClosing,
                crate::construction::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "html-start/self-closing",
                },
                c,
            )?;
        }
        self.finish(c)
    }
    fn advance(
        &mut self,
        source: &HtmlSource,
        c: &mut ParseControl,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<HeadStep, HeadError> {
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let mode = self.mode(c)?;
            self.record_trigger(crate::construction::provenance::ParserRule::Tree(mode), c)?;
            if self.foreign_dispatch(c)?
                && let Some(step) = self.foreign_rules(c)?
            {
                return Ok(step);
            }
            if mode == InsertionMode::InTemplate {
                if let Some(step) = self.template_rules(c)? {
                    return Ok(step);
                }
                continue;
            }
            if matches!(
                mode,
                InsertionMode::AfterHead
                    | InsertionMode::InBody
                    | InsertionMode::AfterBody
                    | InsertionMode::AfterAfterBody
            ) {
                if let Some(step) = self.body_rules_with_meta_csp(mode, source, c, service)? {
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
                            return Err(HeadError::InvalidTextToken);
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
                            self.emit(
                                HeadCondition::EofInText,
                                crate::construction::evidence::DiagnosticRule {
                                    anchor: mode.rule_anchor(),
                                    branch: "text/eof",
                                },
                                c,
                            )?;
                            if is_script {
                                c.charge(WorkCategory::RecordVisit, 1)?;
                                let script = self
                                    .script
                                    .as_ref()
                                    .ok_or(HeadError::ConstructionCapabilityUnavailable)?;
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
                self.emit(
                    HeadCondition::UnexpectedDoctype,
                    crate::construction::evidence::DiagnosticRule {
                        anchor: mode.rule_anchor(),
                        branch: "doctype/ignore",
                    },
                    c,
                )?;
                return self.discard(c);
            }
            if mode == InsertionMode::BeforeHead {
                match class {
                    Class::Comment | Class::Pi => return self.ordinary_node(c),
                    Class::Start(Name::Head, close) => {
                        let node = self.insert_tag(c)?;
                        self.set_head(&node, c)?;
                        if close {
                            self.emit(
                                HeadCondition::UnacknowledgedSelfClosing,
                                crate::construction::evidence::DiagnosticRule {
                                    anchor: mode.rule_anchor(),
                                    branch: "head-start/self-closing",
                                },
                                c,
                            )?;
                        }
                        return self.finish(c);
                    }
                    Class::End(Name::Head | Name::Body | Name::Html | Name::Br) => {}
                    Class::End(_) => {
                        self.emit(
                            HeadCondition::UnexpectedEndTag,
                            crate::construction::evidence::DiagnosticRule {
                                anchor: mode.rule_anchor(),
                                branch: "before-head/disallowed-end-tag",
                            },
                            c,
                        )?;
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
                        self.emit(
                            HeadCondition::UnexpectedInNoscript,
                            crate::construction::evidence::DiagnosticRule {
                                anchor: mode.rule_anchor(),
                                branch: "noscript/head-or-noscript-start",
                            },
                            c,
                        )?;
                        return self.discard(c);
                    }
                    Class::End(name) if !matches!(name, Name::Br) => {
                        self.emit(
                            HeadCondition::UnexpectedInNoscript,
                            crate::construction::evidence::DiagnosticRule {
                                anchor: mode.rule_anchor(),
                                branch: "noscript/disallowed-end-tag",
                            },
                            c,
                        )?;
                        return self.discard(c);
                    }
                    _ => {
                        self.emit(
                            HeadCondition::UnexpectedInNoscript,
                            crate::construction::evidence::DiagnosticRule {
                                anchor: mode.rule_anchor(),
                                branch: "noscript/anything-else",
                            },
                            c,
                        )?;
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
                    self.insert_tag_with_meta_csp(c, service)?;
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
                        self.emit(
                            HeadCondition::UnacknowledgedSelfClosing,
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-inhead",
                                branch: "noscript-start/self-closing",
                            },
                            c,
                        )?;
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
                Class::Start(Name::Template, _) => return self.template_start(c),
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
                    self.emit(
                        HeadCondition::UnexpectedHead,
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-inhead",
                            branch: "head-start/ignore",
                        },
                        c,
                    )?;
                    return self.discard(c);
                }
                Class::End(Name::Template) => {
                    return self.end_template(c);
                }
                Class::End(Name::Body | Name::Html | Name::Br) => {}
                Class::End(_) => {
                    self.emit(
                        HeadCondition::UnexpectedEndTag,
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-inhead",
                            branch: "head/disallowed-end-tag",
                        },
                        c,
                    )?;
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

impl HeadState<'_> {
    #[cfg(test)]
    pub(super) fn diagnostics(&self) -> Vec<HeadDiagnostic> {
        let e = self.before.kernel.session.observed_evidence();
        (0..e.len())
            .filter_map(|index| {
                let (record, source) = e.retained_record(index)?;
                match (record.kind, record.context) {
                    (
                        crate::construction::evidence::DiagnosticKind::Tree(condition),
                        crate::construction::evidence::DiagnosticContext::Tree(mode),
                    ) => Some(HeadDiagnostic {
                        condition,
                        mode,
                        source: source.source.clone(),
                    }),
                    _ => None,
                }
            })
            .collect()
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
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
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

#[cfg(test)]
mod event_retention_tests {
    use super::super::{
        entry,
        tests::{assert_released, limits},
    };
    use super::*;
    use crate::parser::{
        control::{Invariant, ParseUsage},
        input::PreprocessingRecord,
    };
    use crate::{HtmlDocumentParseRequest, HtmlParseCancellation, HtmlParseResource, HtmlSource};

    fn in_head(driver: &mut super::super::StandaloneDocumentDriver<'_>) {
        for _ in 0..32 {
            if driver
                .head
                .as_ref()
                .and_then(|head| head.state.as_ref())
                .is_some_and(|state| state.mode == InsertionMode::InHead && state.pending.is_none())
            {
                return;
            }
            assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
        }
        panic!("actual head-start prefix was not consumed");
    }

    fn preprocessing(event: &Event, source: &HtmlSource, start: usize) {
        let Event::Preprocessing {
            _record: record,
            _source: range,
            retention,
        } = event
        else {
            panic!("real CRLF preprocessing Event");
        };
        assert_eq!(*record, PreprocessingRecord::Newline { was_crlf: true });
        assert_eq!(range.source(), source.identity());
        assert_eq!((range.start(), range.end()), (start, start + 2));
        assert_eq!((retention.len(), retention.capacity()), (1, 1));
        assert_eq!(retention.as_slice()[0], *range);
    }

    fn first_comment_event_footprint(source: &HtmlSource) -> ParseUsage {
        let request = HtmlDocumentParseRequest::offline(source.clone(), limits(100));
        let mut driver = entry::initialized_driver(&request);
        assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
        assert_eq!(driver.lexical.as_ref().unwrap().len(), 1);
        preprocessing(&driver.lexical.as_ref().unwrap().as_slice()[0], source, 4);
        let baseline = driver.control.usage().clone();
        driver.release().unwrap();
        assert_released(&driver.control);
        baseline
    }

    #[test]
    fn fresh_event_capacity_clips_and_one_short_keeps_the_pending_preprocessing_ticket() {
        let source = HtmlSource::utf8("<!--\r\n\r\n\r\n-->");
        let footprint = first_comment_event_footprint(&source);
        // CommentStart queues the first preprocessing Event and reconsumes
        // its newline in Comment on the next pull; no data unit/origin has
        // been appended at this first observable prefix checkpoint.
        let base = footprint.actual_capacity_bytes
            - size_of::<Event>() as u64
            - size_of::<crate::HtmlSourceRange>() as u64;
        let event = size_of::<Event>() as u64;
        let origin = size_of::<crate::parser::tokenizer::token::TextOrigin>() as u64;
        let range = size_of::<crate::HtmlSourceRange>() as u64;
        // Three source-defined CRLFs produce three one-range Event tickets,
        // three LF units and a four-slot geometric DraftText origin capacity.
        // Existing two Event slots are replaced by exactly three when four
        // cannot fit. Peak overlaps only the old/new vector being replaced.
        let final_bytes = base + 6 + 4 * origin + 3 * range + 3 * event;
        let peak = (base + 6 + 6 * origin + 3 * range + 2 * event)
            .max(base + 6 + 4 * origin + 3 * range + 5 * event)
            .max(
                footprint
                    .resource(HtmlParseResource::PeakTransientBytes)
                    .high_water,
            );
        for short in [false, true] {
            let caps = limits(100)
                .into_builder()
                .managed_capacity_bytes(final_bytes - u64::from(short))
                .peak_transient_bytes(peak)
                .finish()
                .unwrap();
            let request = HtmlDocumentParseRequest::offline(source.clone(), caps);
            let mut driver = entry::initialized_driver(&request);
            for _ in 0..2 {
                assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
            }
            assert_eq!(
                (
                    driver.lexical.as_ref().unwrap().len(),
                    driver.lexical.as_ref().unwrap().capacity()
                ),
                (2, 2)
            );
            let result = driver.step();
            if short {
                assert!(matches!(
                    result,
                    Err(super::super::StandaloneError::Stop(
                        ParseStop::LimitExceeded {
                            resource: HtmlParseResource::ManagedCapacityBytes,
                            ..
                        }
                    ))
                ));
                assert_eq!(
                    (
                        driver.lexical.as_ref().unwrap().len(),
                        driver.lexical.as_ref().unwrap().capacity()
                    ),
                    (2, 2)
                );
                preprocessing(driver.event.as_ref().unwrap(), &source, 8);
                assert_eq!(
                    driver.control.usage().actual_capacity_bytes,
                    final_bytes - event
                );
                let before = driver.control.usage().clone();
                let stop = driver.control.stop();
                assert!(driver.step().is_err());
                assert_eq!(driver.control.usage(), &before);
                assert_eq!(driver.control.stop(), stop);
                preprocessing(driver.event.as_ref().unwrap(), &source, 8);
            } else {
                assert_eq!(result.unwrap(), HeadStep::Consumed);
                assert_eq!(
                    (
                        driver.lexical.as_ref().unwrap().len(),
                        driver.lexical.as_ref().unwrap().capacity()
                    ),
                    (3, 3)
                );
                assert!(driver.event.is_none());
                assert_eq!(driver.control.usage().actual_capacity_bytes, final_bytes);
                assert_eq!(
                    driver
                        .control
                        .usage()
                        .resource(HtmlParseResource::PeakTransientBytes)
                        .high_water,
                    peak
                );
            }
            for (i, value) in driver
                .lexical
                .as_ref()
                .unwrap()
                .as_slice()
                .iter()
                .enumerate()
            {
                preprocessing(value, &source, 4 + 2 * i);
            }
            let at_stop = driver.control.first_stop_usage().cloned();
            let before = driver.control.usage().clone();
            driver.release().unwrap();
            assert_released(&driver.control);
            assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
            assert_eq!(driver.control.first_stop_usage(), at_stop.as_ref());
            assert_eq!(
                driver.control.usage().actual_capacity_high_water,
                before.actual_capacity_high_water
            );
            if at_stop.is_some() {
                assert_eq!(
                    driver
                        .control
                        .usage()
                        .resource(HtmlParseResource::WorkCredits),
                    before.resource(HtmlParseResource::WorkCredits)
                );
            }
        }
    }

    #[test]
    fn fresh_event_logical_denial_retains_geometric_capacity_and_exact_pending_owner() {
        let source = HtmlSource::utf8("<!--\r\n\r\n\r\n-->");
        let baseline = first_comment_event_footprint(&source);
        // Inside one comment there is no intervening text-token slot. The
        // observed first Event footprint already includes one real Event;
        // allowing one additional slot admits exactly two retained Events.
        let caps = limits(100)
            .into_builder()
            .auxiliary_entries(
                baseline
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current
                    + 1,
            )
            .finish()
            .unwrap();
        let request = HtmlDocumentParseRequest::offline(source.clone(), caps);
        let mut driver = entry::initialized_driver(&request);
        for _ in 0..2 {
            assert_eq!(driver.step().unwrap(), HeadStep::Consumed);
        }
        assert!(matches!(
            driver.step(),
            Err(super::super::StandaloneError::Stop(
                ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    ..
                }
            ))
        ));
        assert_eq!(
            (
                driver.lexical.as_ref().unwrap().len(),
                driver.lexical.as_ref().unwrap().capacity()
            ),
            (2, 4)
        );
        preprocessing(driver.event.as_ref().unwrap(), &source, 8);
        for (i, event) in driver
            .lexical
            .as_ref()
            .unwrap()
            .as_slice()
            .iter()
            .enumerate()
        {
            preprocessing(event, &source, 4 + 2 * i);
        }
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            baseline
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current
                + 1
        );
        let at_stop = driver.control.first_stop_usage().unwrap().clone();
        let before = driver.control.usage().clone();
        assert!(driver.step().is_err());
        assert_eq!(driver.control.usage(), &before);
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.first_stop_usage(), Some(&at_stop));
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
    }

    #[test]
    fn head_event_logical_denial_keeps_real_preprocessing_ticket_and_zero_cleanup() {
        let source = HtmlSource::utf8(format!(
            "<!doctype html><html><head><!--{}--></head><body></body></html>",
            "\r\n".repeat(32)
        ));
        let baseline = {
            let request = HtmlDocumentParseRequest::offline(source.clone(), limits(100));
            let mut driver = entry::initialized_driver(&request);
            in_head(&mut driver);
            assert_eq!(
                driver
                    .head
                    .as_ref()
                    .unwrap()
                    .state
                    .as_ref()
                    .unwrap()
                    .lexical
                    .len(),
                0
            );
            let baseline = driver.control.usage().clone();
            driver.release().unwrap();
            assert_released(&driver.control);
            baseline
        };
        let allowed = (baseline
            .resource(HtmlParseResource::AuxiliaryEntries)
            .high_water
            - baseline
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current) as usize;
        assert!(allowed < 32);
        let caps = limits(100)
            .into_builder()
            .auxiliary_entries(
                baseline
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .high_water,
            )
            .finish()
            .unwrap();
        let request = HtmlDocumentParseRequest::offline(source.clone(), caps);
        let mut driver = entry::initialized_driver(&request);
        in_head(&mut driver);
        assert!(matches!(
            driver.step(),
            Err(super::super::StandaloneError::Head(HeadError::Kernel(
                KernelError::Stop(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::AuxiliaryEntries,
                    ..
                })
            )))
        ));
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert_eq!(
            (state.lexical.len(), state.lexical.capacity()),
            (allowed, (allowed + 1).next_power_of_two())
        );
        for (i, event) in state.lexical.as_slice().iter().enumerate() {
            preprocessing(event, &source, 31 + 2 * i);
        }
        preprocessing(
            state.lexical_pending.as_ref().unwrap(),
            &source,
            31 + 2 * allowed,
        );
        let at_stop = driver.control.first_stop_usage().unwrap().clone();
        let before = driver.control.usage().clone();
        assert!(driver.step().is_err());
        assert_eq!(driver.control.usage(), &before);
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.first_stop_usage(), Some(&at_stop));
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
    }

    #[test]
    fn head_event_growth_preserves_source_eof_and_foreign_cancelled_owner_refusals() {
        use crate::construction::evidence::{DiagnosticContext, DiagnosticKind};
        use crate::parser::tokenizer::token::{DiagnosticCode, DiagnosticPhase};
        let source = HtmlSource::utf8(format!(
            "<!doctype html><html><head><!--{}--></head><body></body></html>",
            "\0".repeat(32)
        ));
        let request = HtmlDocumentParseRequest::offline(source.clone(), limits(100));
        let mut driver = entry::initialized_driver(&request);
        in_head(&mut driver);
        let before = driver.control.usage().clone();
        let mut foreign = ParseControl::new(limits(100), HtmlParseCancellation::new());
        assert!(matches!(
            driver.head.as_mut().unwrap().step(&mut foreign),
            Err(HeadError::Kernel(KernelError::Stop(
                ParseStop::InternalInvariant(Invariant::ForeignControl)
            )))
        ));
        assert_eq!(driver.control.usage(), &before);
        assert!(
            driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .lexical_pending
                .is_none()
        );
        assert_eq!(foreign.first_stop_usage().unwrap(), foreign.usage());
        for _ in 0..128 {
            match driver.step().unwrap() {
                HeadStep::Consumed => {}
                HeadStep::Complete => break,
                other => panic!("unexpected ordinary boundary: {other:?}"),
            }
        }
        let state = driver.head.as_ref().unwrap().state.as_ref().unwrap();
        assert!(state.end.is_some());
        assert_eq!((state.lexical.len(), state.lexical.capacity()), (32, 32));
        let evidence = state.before.kernel.session.observed_evidence();
        for (i, event) in state.lexical.as_slice().iter().enumerate() {
            let Event::Diagnostic(id) = event else {
                panic!("NULL diagnostic Event");
            };
            let (record, location) = evidence.get(&mut driver.control, id).unwrap();
            assert_eq!(
                record.kind,
                DiagnosticKind::Tokenizer(DiagnosticCode::UnexpectedNullCharacter)
            );
            assert_eq!(
                record.context,
                DiagnosticContext::Tokenizer {
                    phase: DiagnosticPhase::Tokenizer,
                    state: State::Comment
                }
            );
            assert_eq!(location.source.source(), source.identity());
            assert_eq!(
                (location.source.start(), location.source.end()),
                (31 + i, 32 + i)
            );
        }
        let mut input = Some(driver);
        let mut complete =
            super::super::StandaloneDocumentDriver::take_completed(&mut input).unwrap();
        assert!(input.is_none());
        assert_eq!(complete.diagnostics().unwrap().len(), 32);
        {
            use crate::construction::native_view::NativeKind;
            let mut view = complete.native_view().unwrap();
            let root = view.root().unwrap();
            let html = view.child_at(&root, 1).unwrap().unwrap();
            let head = view.child_at(&html, 0).unwrap().unwrap();
            assert_eq!(view.child_count(&head).unwrap(), 1);
            let comment = view.child_at(&head, 0).unwrap().unwrap();
            let NativeKind::Comment(value) = view.kind(&comment).unwrap() else {
                panic!("real head comment");
            };
            assert_eq!(value.len(), 32);
            for i in 0..32 {
                assert_eq!(view.unit(&value, i).unwrap(), Some(0xfffd));
            }
        }
        let mut parts = complete.freeze_parts().unwrap();
        parts.release().unwrap();
        assert_eq!(parts.retained_usage().actual_capacity_bytes, 0);
        assert!(parts.at_stop().is_none());

        // Cancellation is a real request signal at a reached source-qualified
        // head phase, not a callback injected into the reserve operation.
        let request = HtmlDocumentParseRequest::offline(source, limits(100));
        let mut driver = entry::initialized_driver(&request);
        in_head(&mut driver);
        request.cancellation().cancel();
        assert!(driver.step().is_err());
        let snapshot = driver.control.first_stop_usage().unwrap().clone();
        let before = driver.control.usage().clone();
        assert!(
            driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .lexical_pending
                .is_none()
        );
        assert_eq!(
            driver
                .head
                .as_ref()
                .unwrap()
                .state
                .as_ref()
                .unwrap()
                .lexical
                .len(),
            0
        );
        driver.release().unwrap();
        assert_released(&driver.control);
        assert_eq!(driver.control.usage().actual_capacity_bytes, 0);
        assert_eq!(driver.control.first_stop_usage(), Some(&snapshot));
        assert_eq!(
            driver
                .control
                .usage()
                .resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
    }
}

#[cfg(test)]
mod stage_custody_tests {
    use super::super::{
        entry,
        tests::{assert_released, limits},
    };
    use super::*;
    use crate::parser::tokenizer::{Tokenizer, states::State};
    use crate::{HtmlDocumentParseRequest, HtmlParseCancellation, HtmlSource};

    #[test]
    fn a_head_stage_without_the_before_head_owner_reports_missing_input_and_keeps_the_tokenizer() {
        let source = HtmlSource::utf8("<p>x");
        let mut c = ParseControl::new(limits(100), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut tokenizer = Some(Tokenizer::new(&source, State::Data, None, &mut c).unwrap());
        let mut before = None;
        let error = HeadStage::new(&source, &mut before, &mut tokenizer, &mut c)
            .err()
            .unwrap();
        assert!(matches!(error, HeadError::MissingInput), "{error:?}");
        assert!(before.is_none());
        // The tokenizer was neither taken nor charged for, so it still releases.
        tokenizer.take().unwrap().release(&mut c).unwrap();
        assert_released(&c);
    }

    #[test]
    fn releasing_a_head_stage_without_its_state_reports_consumed_and_the_state_releases_later() {
        let request =
            HtmlDocumentParseRequest::offline(HtmlSource::utf8("<html><p>x"), limits(100));
        let mut driver = entry::initialized_driver(&request);
        for _ in 0..100 {
            if driver.head.is_some() {
                break;
            }
            driver.step().unwrap();
        }
        let head = driver.head.as_mut().unwrap();
        // Private-state fault injection: the stage's state is not where release
        // expects it.
        let state = head.state.take().unwrap();
        let error = head.release(&mut driver.control).unwrap_err();
        assert!(matches!(error, HeadError::Consumed), "{error:?}");
        head.state = Some(state);
        driver.release().unwrap();
        assert_released(&driver.control);
    }
}
