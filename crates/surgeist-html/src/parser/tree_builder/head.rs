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
// Disabled InHeadNoscript and non-script Text. HTML
// 877d6146487f15b15bc8dd0e3c625063a37b456a governs behavior and PI.

use super::{
    KernelError, LocationKernel, StackPush, copy_node,
    prologue::{BeforeHead, Payload, PendingToken, PrologueError},
    tag_sets, write,
};
use crate::{
    HtmlParseResource, HtmlSource, HtmlSourceRange,
    construction::parser_session::{
        ParserNode, SessionError, split_leading_ascii_whitespace, validate_token_kind,
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
pub(super) enum HeadMode {
    BeforeHead,
    InHead,
    InHeadNoscript,
    Text,
    AfterHead,
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
}
pub(super) struct HeadDiagnostic {
    pub(super) condition: HeadCondition,
    pub(super) mode: HeadMode,
    pub(super) source: HtmlSourceRange,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HeadBoundary {
    AfterHead,
    Script,
    Template,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HeadStep {
    Consumed,
    Boundary(HeadBoundary),
}
pub(super) enum HeadError {
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

/// All state belongs to this one driver and the existing flat arena. A suspended
/// script/template start has no marker, script metadata or insertion effects.
pub(super) struct HeadState<'source> {
    pub(super) before: BeforeHead,
    pub(super) tokenizer: Option<Tokenizer<'source>>,
    pub(super) pending: Option<PendingToken>,
    pub(super) mode: HeadMode,
    pub(super) head: Option<ParserNode>,
    return_mode: Option<HeadMode>,
    boundary: Option<HeadBoundary>,
    pub(super) diagnostics: OwnedVec<HeadDiagnostic>,
    pub(super) lexical: OwnedVec<Event>,
    lexical_pending: Option<Event>,
    whitespace_pending: Option<Text>,
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
        diagnostics.require_owner(c)?;
        lexical.require_owner(c)?;
        // Three driver fields + eleven state fields + caller BeforeHead and
        // Tokenizer slots + the retained BeforeHead input slot are written.
        // Admit all seventeen field writes before the first owning take.
        if let Err(stop) = write(size_of::<Self>() + size_of::<HeadState>(), 17, c) {
            diagnostics.release(c)?;
            lexical.release(c)?;
            return Err(stop.into());
        }
        if let Err(stop) = c.charge(WorkCategory::ProviderCall, 1) {
            diagnostics.release(c)?;
            lexical.release(c)?;
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
                mode: HeadMode::BeforeHead,
                head: None,
                return_mode: None,
                boundary: None,
                diagnostics,
                lexical,
                lexical_pending: None,
                whitespace_pending: None,
            }),
        })
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
            // This stage admits only the fresh ordinary HTML stack. CDATA is
            // disallowed by its actual exclusively-HTML element creation paths.
            state.lexical_pending = state
                .tokenizer
                .as_mut()
                .ok_or(HeadError::Consumed)?
                .next(c, false)?;
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
        state.advance(c)
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
        Ok(())
    }
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
    fn kernel(&mut self) -> &mut LocationKernel {
        &mut self.before.kernel
    }
    fn mode(&self, c: &mut ParseControl) -> Result<HeadMode, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, size_of::<HeadMode>() as u64)?;
        Ok(self.mode)
    }
    fn set_mode(&mut self, mode: HeadMode, c: &mut ParseControl) -> Result<(), HeadError> {
        write(size_of::<HeadMode>(), 1, c)?;
        self.mode = mode;
        Ok(())
    }
    fn emit(&mut self, condition: HeadCondition, c: &mut ParseControl) -> Result<(), HeadError> {
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
    fn finish(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        write(size_of::<Option<PendingToken>>(), 1, c)?;
        self.pending = None;
        Ok(HeadStep::Consumed)
    }
    fn discard(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        self.pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .release(c)?;
        self.finish(c)
    }
    fn stop_at(
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
    fn whitespace(&mut self, c: &mut ParseControl) -> Result<bool, HeadError> {
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
        if self.whitespace_pending.is_some() {
            if mode == HeadMode::BeforeHead {
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
                self.before
                    .kernel
                    .insert_text(&mut self.whitespace_pending, None, c)?;
            }
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(input.is_none())
    }
    fn ordinary_node(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
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
        self.before
            .kernel
            .session
            .insert(c, &location.parent, &node, location.before.as_ref())?;
        self.finish(c)
    }
    fn insert_tag(&mut self, c: &mut ParseControl) -> Result<ParserNode, HeadError> {
        let Payload::Tag(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        Ok(self
            .before
            .kernel
            .insert_html_tag(input, StackPush::Push, None, c)?
            .node)
    }
    fn set_head(&mut self, node: &ParserNode, c: &mut ParseControl) -> Result<(), HeadError> {
        let node = copy_node(node, c)?;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        self.head = Some(node);
        self.set_mode(HeadMode::InHead, c)
    }
    fn implied_head(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        let location = self.kernel().adjusted_location(None, c)?;
        let node = self.kernel().session.allocate_implicit_head(c)?;
        self.kernel().insert_element_at(&node, &location, c)?;
        self.kernel().push(&node, c)?;
        self.set_head(&node, c)
    }
    fn raw_text(
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
        write(size_of::<Option<HeadMode>>(), 1, c)?;
        let mode = self.mode(c)?;
        self.return_mode = Some(mode);
        self.set_mode(HeadMode::Text, c)?;
        if self_closing {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        self.finish(c)
    }
    fn html(&mut self, self_closing: bool, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
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
    fn advance(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let mode = self.mode(c)?;
            if mode != HeadMode::Text && self.whitespace(c)? {
                return self.finish(c);
            }
            let class = self.classify(c)?;
            if mode == HeadMode::Text {
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
                        self.before.kernel.insert_text(input, None, c)?;
                        return self.finish(c);
                    }
                    Class::End(_) | Class::Eof => {
                        if matches!(class, Class::Eof) {
                            self.emit(HeadCondition::EofInText, c)?;
                        }
                        self.kernel().pop(c)?.ok_or(KernelError::EmptyStack)?;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        c.charge(
                            WorkCategory::InspectByte,
                            size_of::<Option<HeadMode>>() as u64,
                        )?;
                        let mode = self.return_mode.ok_or(HeadError::MissingReturnMode)?;
                        self.set_mode(mode, c)?;
                        write(size_of::<Option<HeadMode>>(), 1, c)?;
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
            if mode == HeadMode::BeforeHead {
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
            if mode == HeadMode::InHeadNoscript {
                match class {
                    Class::End(Name::Noscript) => {
                        self.kernel().pop(c)?.ok_or(KernelError::EmptyStack)?;
                        self.set_mode(HeadMode::InHead, c)?;
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
                        self.kernel().pop(c)?.ok_or(KernelError::EmptyStack)?;
                        self.set_mode(HeadMode::InHead, c)?;
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
                    self.kernel().pop(c)?.ok_or(KernelError::EmptyStack)?;
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
                    self.set_mode(HeadMode::InHeadNoscript, c)?;
                    if close {
                        self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
                    }
                    return self.finish(c);
                }
                Class::Start(Name::Script, _) => return self.stop_at(HeadBoundary::Script, c),
                Class::Start(Name::Template, _) => return self.stop_at(HeadBoundary::Template, c),
                Class::End(Name::Head) => {
                    self.kernel().pop(c)?.ok_or(KernelError::EmptyStack)?;
                    self.set_mode(HeadMode::AfterHead, c)?;
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
            self.kernel().pop(c)?.ok_or(KernelError::EmptyStack)?;
            self.set_mode(HeadMode::AfterHead, c)?;
            return self.stop_at(HeadBoundary::AfterHead, c);
        }
    }
}
