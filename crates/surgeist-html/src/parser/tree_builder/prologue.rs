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
// d7232d746d3112f08926d169591576bd12fe2fdf, Initial and BeforeHtml.
// HTML 877d6146487f15b15bc8dd0e3c625063a37b456a governs doctype and PI drift.

use super::{InsertionLocation, KernelError, LocationKernel, doctype, write};
use crate::construction::parser_session::{
    ParserArenaSession, RootKind, SessionError, release_token_kind, split_leading_ascii_whitespace,
    validate_token_kind,
};
use crate::parser::{
    control::{ControlIdentity, Invariant, OwnedVec, ParseControl, ParseStop, WorkCategory},
    tokenizer::token::{Doctype, Tag, TagKind, Text, Token, TokenKind},
};
use crate::{HtmlDocumentMode, HtmlParseResource, HtmlSource, HtmlSourceRange};
use std::mem::size_of;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PrologueMode {
    Initial,
    BeforeHtml,
    BeforeHead,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PrologueCondition {
    MissingDoctype,
    InvalidDoctype,
    UnexpectedDoctype,
    UnexpectedEndTag,
    UnacknowledgedSelfClosing,
}

pub(super) struct PrologueDiagnostic {
    pub(super) condition: PrologueCondition,
    pub(super) mode: PrologueMode,
    pub(super) source: HtmlSourceRange,
}

#[derive(Debug)]
pub(crate) enum PrologueError {
    Kernel(KernelError),
    AlreadyAtBeforeHead,
    PendingInput,
    MissingInput,
    Consumed,
}
impl From<KernelError> for PrologueError {
    fn from(error: KernelError) -> Self {
        Self::Kernel(error)
    }
}
impl From<ParseStop> for PrologueError {
    fn from(error: ParseStop) -> Self {
        Self::Kernel(error.into())
    }
}
impl From<SessionError> for PrologueError {
    fn from(error: SessionError) -> Self {
        Self::Kernel(error.into())
    }
}

/// Borrowed producer slots own the actual lexical payloads, including after a
/// failed transfer. No synthetic Tag or intermediate tree is constructed.
pub(super) enum Payload {
    Text(Option<Text>),
    Tag(Option<Tag>),
    Comment(Option<Text>),
    Doctype(Option<Doctype>),
    Pi(Option<(Text, Text)>),
    Eof,
}
pub(super) struct PendingToken {
    pub(super) source: HtmlSourceRange,
    pub(super) payload: Payload,
}
impl PendingToken {
    pub(super) fn from_token(token: Token) -> Self {
        let payload = match token.kind {
            TokenKind::Text(text) => Payload::Text(Some(text)),
            TokenKind::Tag(tag) => Payload::Tag(Some(tag)),
            TokenKind::Comment(text) => Payload::Comment(Some(text)),
            TokenKind::Doctype(doctype) => Payload::Doctype(Some(doctype)),
            TokenKind::ProcessingInstruction { target, data } => Payload::Pi(Some((target, data))),
            TokenKind::Eof => Payload::Eof,
        };
        Self {
            source: token.source,
            payload,
        }
    }
    pub(super) fn into_token(self) -> Option<Token> {
        let kind = match self.payload {
            Payload::Text(text) => text.map(TokenKind::Text),
            Payload::Tag(tag) => tag.map(TokenKind::Tag),
            Payload::Comment(text) => text.map(TokenKind::Comment),
            Payload::Doctype(doctype) => doctype.map(TokenKind::Doctype),
            Payload::Pi(pair) => {
                pair.map(|(target, data)| TokenKind::ProcessingInstruction { target, data })
            }
            Payload::Eof => Some(TokenKind::Eof),
        };
        kind.map(|kind| Token {
            kind,
            source: self.source,
        })
    }
    pub(super) fn release(&mut self, c: &mut ParseControl) -> Result<(), PrologueError> {
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
        }
        let present = match &self.payload {
            Payload::Text(value) | Payload::Comment(value) => value.is_some(),
            Payload::Tag(value) => value.is_some(),
            Payload::Doctype(value) => value.is_some(),
            Payload::Pi(value) => value.is_some(),
            Payload::Eof => true,
        };
        if !present {
            return Ok(());
        }
        if c.stop().is_none() {
            write(size_of::<TokenKind>(), 2, c)?;
        }
        let mut kind = match &mut self.payload {
            Payload::Text(text) => text.take().map(TokenKind::Text),
            Payload::Tag(tag) => tag.take().map(TokenKind::Tag),
            Payload::Comment(text) => text.take().map(TokenKind::Comment),
            Payload::Doctype(doctype) => doctype.take().map(TokenKind::Doctype),
            Payload::Pi(pair) => pair
                .take()
                .map(|(target, data)| TokenKind::ProcessingInstruction { target, data }),
            Payload::Eof => Some(TokenKind::Eof),
        };
        let result = release_token_kind(&mut kind, c);
        // This is ownership recovery, including terminal disposal. It never
        // resumes processing after a stop or manufactures useful output.
        if let Some(kind) = kind {
            self.payload = match kind {
                TokenKind::Text(text) => Payload::Text(Some(text)),
                TokenKind::Tag(tag) => Payload::Tag(Some(tag)),
                TokenKind::Comment(text) => Payload::Comment(Some(text)),
                TokenKind::Doctype(doctype) => Payload::Doctype(Some(doctype)),
                TokenKind::ProcessingInstruction { target, data } => {
                    Payload::Pi(Some((target, data)))
                }
                TokenKind::Eof => Payload::Eof,
            };
        }
        result.map_err(PrologueError::from)
    }
}

/// A source-established fresh ordinary document operation. This constructor
/// cannot accept an imported/captured document or inferred srcdoc/mode flags.
/// The selected pin initializes parser-cannot-change-mode to false (§13.2.6.4.1).
pub(super) struct DocumentPrologue<'a> {
    source: &'a HtmlSource,
    owner: ControlIdentity,
    kernel: Option<LocationKernel>,
    mode: PrologueMode,
    pending: Option<PendingToken>,
    diagnostics: Option<OwnedVec<PrologueDiagnostic>>,
    whitespace: Option<OwnedVec<Text>>,
    whitespace_pending: Option<Text>,
}

/// Real construction and exact reprocessed lexical input awaiting BeforeHead.
/// This value is not EOF acceptance, an immutable parse result or a report.
pub(super) struct BeforeHead {
    pub(super) kernel: LocationKernel,
    pub(super) input: Option<Token>,
    pub(super) diagnostics: OwnedVec<PrologueDiagnostic>,
    pub(super) whitespace: OwnedVec<Text>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PrologueStep {
    Consumed,
    BeforeHead,
}

impl<'a> DocumentPrologue<'a> {
    pub(super) fn new(source: &'a HtmlSource, c: &mut ParseControl) -> Result<Self, PrologueError> {
        c.require_source(source)?;
        write(size_of::<Self>(), 8, c)?;
        let mut diagnostics = OwnedVec::new(HtmlParseResource::Diagnostics)?;
        diagnostics.require_owner(c)?;
        let mut whitespace = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        whitespace.require_owner(c)?;
        let mut session =
            match ParserArenaSession::new(c, RootKind::Document(HtmlDocumentMode::NoQuirks)) {
                Ok(session) => Some(session),
                Err(error) => {
                    diagnostics.release(c)?;
                    whitespace.release(c)?;
                    return Err(error.into());
                }
            };
        let mut kernel = match LocationKernel::new(&mut session, c) {
            Ok(kernel) => kernel,
            Err(error) => {
                if let Some(mut session) = session {
                    session.release(c)?;
                }
                diagnostics.release(c)?;
                whitespace.release(c)?;
                return Err(error.into());
            }
        };
        if let Err(stop) = c.charge(WorkCategory::ProviderCall, 1) {
            kernel.release(c)?;
            diagnostics.release(c)?;
            whitespace.release(c)?;
            return Err(stop.into());
        }
        Ok(Self {
            source,
            owner: c.identity(),
            kernel: Some(kernel),
            mode: PrologueMode::Initial,
            pending: None,
            diagnostics: Some(diagnostics),
            whitespace: Some(whitespace),
            whitespace_pending: None,
        })
    }
    fn require(&self, c: &mut ParseControl) -> Result<(), PrologueError> {
        c.require_identity(&self.owner)?;
        c.require_source(self.source)?;
        if self.kernel.is_none() {
            return Err(PrologueError::Consumed);
        }
        Ok(())
    }
    fn kernel(&mut self) -> Result<&mut LocationKernel, PrologueError> {
        self.kernel.as_mut().ok_or(PrologueError::Consumed)
    }
    fn set_mode(&mut self, mode: PrologueMode, c: &mut ParseControl) -> Result<(), PrologueError> {
        write(size_of::<PrologueMode>(), 1, c)?;
        self.mode = mode;
        Ok(())
    }
    fn emit(
        &mut self,
        condition: PrologueCondition,
        c: &mut ParseControl,
    ) -> Result<(), PrologueError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let source = &self
            .pending
            .as_ref()
            .ok_or(PrologueError::MissingInput)?
            .source;
        write(size_of::<HtmlSourceRange>(), 1, c)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let diagnostic = PrologueDiagnostic {
            condition,
            mode: self.mode,
            source: source.clone(),
        };
        self.diagnostics
            .as_mut()
            .ok_or(PrologueError::Consumed)?
            .try_push(c, diagnostic)?;
        Ok(())
    }
    fn finish_pending(&mut self, c: &mut ParseControl) -> Result<(), PrologueError> {
        write(size_of::<Option<PendingToken>>(), 1, c)?;
        self.pending = None;
        Ok(())
    }
    fn discard(&mut self, c: &mut ParseControl) -> Result<(), PrologueError> {
        self.pending
            .as_mut()
            .ok_or(PrologueError::MissingInput)?
            .release(c)?;
        self.finish_pending(c)
    }
    fn whitespace_prefix(&mut self, c: &mut ParseControl) -> Result<bool, PrologueError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Payload::Text(input) = &mut self
            .pending
            .as_mut()
            .ok_or(PrologueError::MissingInput)?
            .payload
        else {
            return Ok(false);
        };
        write(size_of::<Option<Text>>(), 1, c)?;
        self.whitespace_pending = split_leading_ascii_whitespace(c, input)?;
        if self.whitespace_pending.is_some() {
            write(size_of::<Option<Text>>(), 1, c)?;
            let prefix = self.whitespace_pending.take().expect("retained prefix");
            if let Err((stop, prefix)) = self
                .whitespace
                .as_mut()
                .ok_or(PrologueError::Consumed)?
                .try_push_recover(c, prefix)
            {
                self.whitespace_pending = Some(prefix);
                return Err(stop.into());
            }
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        Ok(input.is_none())
    }
    pub(super) fn process(
        &mut self,
        input: &mut Option<Token>,
        c: &mut ParseControl,
    ) -> Result<PrologueStep, PrologueError> {
        self.require(c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        if self.mode == PrologueMode::BeforeHead {
            return Err(PrologueError::AlreadyAtBeforeHead);
        }
        if self.pending.is_some() {
            return Err(PrologueError::PendingInput);
        }
        let token = input.as_ref().ok_or(PrologueError::MissingInput)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        if token.source.source() != self.source.identity() {
            return Err(c.fail_invariant(Invariant::ForeignSource).into());
        }
        // A shared source identity does not reassign lexical backing tickets.
        // Verify every actual payload owner before taking it or emitting a
        // diagnostic/changing mode under this execution's controller.
        validate_token_kind(&token.kind, c)?;
        write(size_of::<Token>(), 1, c)?;
        write(size_of::<PendingToken>(), 2, c)?;
        self.pending = Some(PendingToken::from_token(
            input.take().expect("validated input"),
        ));
        self.advance(c)
    }
    fn advance(&mut self, c: &mut ParseControl) -> Result<PrologueStep, PrologueError> {
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            c.charge(WorkCategory::RecordVisit, 2)?;
            if self.whitespace_prefix(c)? {
                self.finish_pending(c)?;
                return Ok(PrologueStep::Consumed);
            }
            let payload = &mut self
                .pending
                .as_mut()
                .ok_or(PrologueError::MissingInput)?
                .payload;
            match payload {
                Payload::Comment(input) => {
                    let kernel = self.kernel.as_mut().ok_or(PrologueError::Consumed)?;
                    let node = kernel.session.allocate_comment(c, input)?;
                    let root = kernel.session.root(c)?;
                    kernel.session.insert(c, &root, &node, None, None)?;
                    self.finish_pending(c)?;
                    return Ok(PrologueStep::Consumed);
                }
                Payload::Pi(input) => {
                    let kernel = self.kernel.as_mut().ok_or(PrologueError::Consumed)?;
                    let node = kernel.session.allocate_processing_instruction(c, input)?;
                    let root = kernel.session.root(c)?;
                    kernel.session.insert(c, &root, &node, None, None)?;
                    self.finish_pending(c)?;
                    return Ok(PrologueStep::Consumed);
                }
                Payload::Doctype(input) if self.mode == PrologueMode::Initial => {
                    let decision =
                        doctype::classify(input.as_ref().ok_or(PrologueError::MissingInput)?, c)?;
                    if decision.error {
                        self.emit(PrologueCondition::InvalidDoctype, c)?;
                    }
                    let Payload::Doctype(input) = &mut self
                        .pending
                        .as_mut()
                        .ok_or(PrologueError::MissingInput)?
                        .payload
                    else {
                        unreachable!("doctype payload retained");
                    };
                    let kernel = self.kernel.as_mut().ok_or(PrologueError::Consumed)?;
                    let node = kernel.session.allocate_doctype(c, input)?;
                    if kernel.session.document_accepts_doctype(c)? {
                        let root = kernel.session.root(c)?;
                        kernel.session.insert(c, &root, &node, None, None)?;
                    }
                    if decision.mode != HtmlDocumentMode::NoQuirks {
                        kernel.session.set_document_mode(c, decision.mode)?;
                    }
                    self.set_mode(PrologueMode::BeforeHtml, c)?;
                    self.finish_pending(c)?;
                    return Ok(PrologueStep::Consumed);
                }
                _ if self.mode == PrologueMode::Initial => {
                    self.emit(PrologueCondition::MissingDoctype, c)?;
                    self.kernel()?
                        .session
                        .set_document_mode(c, HtmlDocumentMode::Quirks)?;
                    self.set_mode(PrologueMode::BeforeHtml, c)?;
                    // The same exact token, including its original source, is
                    // reprocessed; no cursor read, clone or fabricated emission.
                    continue;
                }
                Payload::Doctype(_) => {
                    self.emit(PrologueCondition::UnexpectedDoctype, c)?;
                    self.discard(c)?;
                    return Ok(PrologueStep::Consumed);
                }
                Payload::Tag(input) => {
                    let tag = input.as_ref().ok_or(PrologueError::MissingInput)?;
                    c.require_document_string(tag.name.value.allocation(), tag.name.value.text())?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let html = super::tag_sets::ascii_eq(tag.name.value.text(), "html", c)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let start = tag.kind == TagKind::Start;
                    if start && html {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        c.charge(WorkCategory::InspectByte, 1)?;
                        let self_closing = tag.self_closing;
                        let kernel = self.kernel.as_mut().ok_or(PrologueError::Consumed)?;
                        let root = kernel.session.root(c)?;
                        write(size_of::<InsertionLocation>(), 2, c)?;
                        let location = InsertionLocation {
                            parent: root,
                            before: None,
                        };
                        let node = kernel.session.allocate_html_tag(c, input)?;
                        kernel.insert_element_at(&node, &location, c, None)?;
                        kernel.push(&node, c)?;
                        if self_closing {
                            self.emit(PrologueCondition::UnacknowledgedSelfClosing, c)?;
                        }
                        self.set_mode(PrologueMode::BeforeHead, c)?;
                        self.finish_pending(c)?;
                        return Ok(PrologueStep::BeforeHead);
                    }
                    if !start {
                        let permitted = html
                            || super::tag_sets::ascii_eq(tag.name.value.text(), "head", c)?
                            || super::tag_sets::ascii_eq(tag.name.value.text(), "body", c)?
                            || super::tag_sets::ascii_eq(tag.name.value.text(), "br", c)?;
                        if !permitted {
                            self.emit(PrologueCondition::UnexpectedEndTag, c)?;
                            self.discard(c)?;
                            return Ok(PrologueStep::Consumed);
                        }
                    }
                }
                _ => {}
            }
            let kernel = self.kernel()?;
            let node = kernel.session.allocate_implicit_html(c)?;
            let root = kernel.session.root(c)?;
            kernel.session.insert(c, &root, &node, None, None)?;
            kernel.push(&node, c)?;
            self.set_mode(PrologueMode::BeforeHead, c)?;
            return Ok(PrologueStep::BeforeHead);
        }
    }
    pub(super) fn take_before_head(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<BeforeHead, PrologueError> {
        self.require(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.mode != PrologueMode::BeforeHead {
            return Err(PrologueError::PendingInput);
        }
        write(size_of::<BeforeHead>(), 4, c)?;
        write(size_of::<Option<LocationKernel>>(), 1, c)?;
        write(size_of::<Option<PendingToken>>(), 1, c)?;
        write(size_of::<Option<OwnedVec<PrologueDiagnostic>>>(), 1, c)?;
        write(size_of::<Option<OwnedVec<Text>>>(), 1, c)?;
        Ok(BeforeHead {
            kernel: self.kernel.take().expect("active kernel"),
            input: self.pending.take().and_then(PendingToken::into_token),
            diagnostics: self.diagnostics.take().expect("active diagnostics"),
            whitespace: self.whitespace.take().expect("active whitespace"),
        })
    }
    pub(super) fn release(&mut self, c: &mut ParseControl) -> Result<(), PrologueError> {
        if c.stop().is_some() {
            c.cleanup_release(&self.owner, HtmlParseResource::AuxiliaryEntries, 0)?;
        } else {
            c.require_identity(&self.owner)?;
        }
        if self.kernel.is_none() {
            return Err(PrologueError::Consumed);
        }
        if let Some(pending) = &mut self.pending {
            pending.release(c)?;
        }
        if let Some(prefix) = self.whitespace_pending.take() {
            prefix.release(c)?;
        }
        if let Some(whitespace) = &mut self.whitespace {
            loop {
                let text = if c.stop().is_some() {
                    whitespace.cleanup_pop(c)?
                } else {
                    whitespace.try_pop(c)?
                };
                let Some(text) = text else { break };
                text.release(c)?;
            }
        }
        self.kernel.as_mut().expect("active kernel").release(c)?;
        if let Some(diagnostics) = self.diagnostics.take() {
            diagnostics.release(c)?;
        }
        if let Some(whitespace) = self.whitespace.take() {
            whitespace.release(c)?;
        }
        self.kernel = None;
        self.pending = None;
        Ok(())
    }
}
