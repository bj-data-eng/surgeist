// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tokenizer/mod.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf. Safe indexed Unicode code-point
// dispatch replaces Tendril, atoms, SIMD, streaming and TokenSink machinery.
// Behavior is governed by HTML 877d6146487f15b15bc8dd0e3c625063a37b456a.

use crate::HtmlParseResource;
mod char_ref;
mod state;
pub(crate) mod states;
#[cfg(test)]
mod tests;
pub(crate) mod token;

use super::{
    buffer::Utf16Buffer,
    control::{
        ControlIdentity, Invariant, OwnedVec, ParseControl, ParseStop, WORK_QUANTUM, WorkCategory,
    },
    input::{AsciiComparison, InputCursor, InputRead},
};
use crate::{DocumentString, HtmlSource, HtmlSourceRange};
use states::{Identifier, State};
use std::mem::size_of;
use token::{
    Attribute, Diagnostic, DiagnosticCode, DiagnosticPhase, Doctype, DraftText, Event, OriginKind,
    Tag, TagKind, Token, TokenEvent, TokenKind, freeze_pair,
};

/// Pre-admits explicit persistent record moves/writes, independently of state
/// dispatch. Larger indivisible fixed operations use the controller's atomic
/// layout exception; rejected admission precedes every useful mutation.
fn admit_record_write(bytes: usize, writes: usize, c: &mut ParseControl) -> Result<(), ParseStop> {
    c.charge(WorkCategory::CopyByte, bytes as u64)?;
    c.charge(WorkCategory::RecordWrite, writes as u64)?;
    if bytes.saturating_add(writes) > WORK_QUANTUM {
        c.admit_atomic_layout(bytes)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Branch {
    next: [Option<usize>; 2],
    terminal: bool,
}
/// Fixed-bit radix lookup: exactly sixteen admitted decisions per UTF-16 unit,
/// independent of previous attribute count and adversarial shared prefixes.
struct Names {
    nodes: OwnedVec<Branch>,
}
impl Names {
    fn new() -> Result<Self, ParseStop> {
        Ok(Self {
            nodes: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
        })
    }
    fn insert(&mut self, units: &[u16], c: &mut ParseControl) -> Result<bool, ParseStop> {
        if self.nodes.len() == 0 {
            self.nodes.try_push(
                c,
                Branch {
                    next: [None; 2],
                    terminal: false,
                },
            )?;
        }
        let mut index = 0;
        for unit in units {
            c.charge(WorkCategory::InspectByte, 2)?;
            let unit = *unit;
            for shift in (0..16).rev() {
                c.charge(WorkCategory::StateStep, 1)?;
                let bit = usize::from((unit >> shift) & 1);
                c.charge(WorkCategory::InspectByte, size_of::<Option<usize>>() as u64)?;
                let next = self
                    .nodes
                    .get_mut(index, c)?
                    .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                    .next[bit];
                index = if let Some(next) = next {
                    next
                } else {
                    let next = self.nodes.len();
                    self.nodes.try_push(
                        c,
                        Branch {
                            next: [None; 2],
                            terminal: false,
                        },
                    )?;
                    c.charge(WorkCategory::CopyByte, size_of::<Option<usize>>() as u64)?;
                    c.charge(WorkCategory::RecordWrite, 1)?;
                    self.nodes
                        .get_mut(index, c)?
                        .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                        .next[bit] = Some(next);
                    next
                };
            }
        }
        c.charge(WorkCategory::InspectByte, 1)?;
        let duplicate = self
            .nodes
            .get_mut(index, c)?
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .terminal;
        admit_record_write(size_of::<bool>(), 1, c)?;
        self.nodes
            .get_mut(index, c)?
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .terminal = true;
        Ok(duplicate)
    }
}
struct DraftAttribute {
    name: DraftText,
    value: DraftText,
    start: usize,
    end: usize,
    name_checked: bool,
    duplicate: bool,
}
struct DraftTag {
    kind: TagKind,
    name: DraftText,
    self_closing: bool,
    attributes: OwnedVec<Attribute>,
    attribute: Option<DraftAttribute>,
    names: Names,
    had_duplicates: bool,
}
struct DraftDoctype {
    name: Option<DraftText>,
    public: Option<DraftText>,
    system: Option<DraftText>,
    force_quirks: bool,
}
impl DraftDoctype {
    fn empty() -> Self {
        Self {
            name: None,
            public: None,
            system: None,
            force_quirks: false,
        }
    }
    fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        for text in [self.name, self.public, self.system].into_iter().flatten() {
            text.release(c)?;
        }
        Ok(())
    }
}
impl DraftTag {
    fn release(mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.name.release(c)?;
        if let Some(a) = self.attribute {
            a.name.release(c)?;
            a.value.release(c)?;
        }
        release_attributes(&mut self.attributes, c)?;
        self.attributes.release(c)?;
        self.names.nodes.release(c)
    }
}
fn release_attributes(
    attrs: &mut OwnedVec<Attribute>,
    c: &mut ParseControl,
) -> Result<(), ParseStop> {
    loop {
        let next = if c.stop().is_some() {
            attrs.cleanup_pop(c)?
        } else {
            match attrs.try_pop(c) {
                Ok(a) => a,
                Err(_) => attrs.cleanup_pop(c)?,
            }
        };
        let Some(a) = next else {
            return Ok(());
        };
        a.release(c)?;
    }
}

/// One concrete pull tokenizer. A consumer must release every returned event
/// and finally the tokenizer through the same controller, including after stop.
/// This is an internal seam, not a public token stream or parsing report.
pub(crate) struct Tokenizer<'source> {
    owner: ControlIdentity,
    cursor: InputCursor<'source>,
    state: State,
    current: HtmlSourceRange,
    current_kind: OriginKind,
    start: usize,
    less_than: HtmlSourceRange,
    slash: HtmlSourceRange,
    pending_syntax: HtmlSourceRange,
    temporary: DraftText,
    last_start: Utf16Buffer,
    tag: Option<DraftTag>,
    comment: Option<DraftText>,
    doctype: Option<DraftDoctype>,
    pi_target: Option<DraftText>,
    pi_data: Option<DraftText>,
    // A single state step has at most preprocessing + three diagnostics + token.
    // Fixed stack storage is not an unbounded emission queue.
    pending: [Option<Event>; 8],
    pending_len: usize,
    done: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenizerHandoffError {
    Stop(ParseStop),
    ForeignSource,
    NotData,
    PendingEvents,
    ActiveDraft,
}
impl From<ParseStop> for TokenizerHandoffError {
    fn from(stop: ParseStop) -> Self {
        Self::Stop(stop)
    }
}
impl<'source> Tokenizer<'source> {
    /// Checks the actual paused document-prologue boundary without changing
    /// lexical state, pulling an event, or consuming any backing owner. EOF is
    /// admissible: the driver can retain its already-returned EOF token.
    pub(crate) fn validate_head_handoff(
        &self,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<(), TokenizerHandoffError> {
        c.require_identity(&self.owner)?;
        c.require_source(source)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        if self.current.source() != source.identity() {
            return Err(TokenizerHandoffError::ForeignSource);
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.state != State::Data {
            return Err(TokenizerHandoffError::NotData);
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.pending_len != 0 {
            return Err(TokenizerHandoffError::PendingEvents);
        }
        c.charge(WorkCategory::RecordVisit, 5)?;
        if self.tag.is_some()
            || self.comment.is_some()
            || self.doctype.is_some()
            || self.pi_target.is_some()
            || self.pi_data.is_some()
        {
            return Err(TokenizerHandoffError::ActiveDraft);
        }
        Ok(())
    }
    /// Borrows the actual preprocessed/reconsume-aware next input item without
    /// consuming it or changing lexical state. A script end hands off in Data
    /// with no queued events/drafts; a raw token endpoint is not substituted.
    pub(crate) fn checked_next_input_position(
        &self,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<HtmlSourceRange, TokenizerHandoffError> {
        self.validate_head_handoff(source, c)?;
        c.admit_atomic_layout(size_of::<HtmlSourceRange>())?;
        c.charge(WorkCategory::CopyByte, size_of::<HtmlSourceRange>() as u64)?;
        c.charge(WorkCategory::RecordWrite, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        match self.cursor.peek(c)? {
            super::input::InputRead::Character(character) => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(WorkCategory::InspectByte, size_of::<usize>() as u64)?;
                let before = character.source().start();
                Ok(self.cursor.checked_range(before, before, c)?)
            }
            super::input::InputRead::Eof(range) => Ok(range),
        }
    }
    pub(crate) fn new(
        source: &'source HtmlSource,
        state: State,
        last_start: Option<&DocumentString>,
        c: &mut ParseControl,
    ) -> Result<Self, ParseStop> {
        c.require_source(source)?;
        let cursor = InputCursor::new(source, c)?;
        let temporary = DraftText::new(c)?;
        let mut name = Utf16Buffer::new(c)?;
        if let Some(last) = last_start
            && let Err(stop) = name.extend_units(last.code_units(), c)
        {
            name.release(c)?;
            temporary.release(c)?;
            return Err(stop);
        }
        let range = match cursor.checked_range(0, 0, c) {
            Ok(range) => range,
            Err(stop) => {
                name.release(c)?;
                temporary.release(c)?;
                return Err(stop);
            }
        };
        if let Err(stop) = admit_record_write(size_of::<Self>(), 1, c) {
            name.release(c)?;
            temporary.release(c)?;
            return Err(stop);
        }
        Ok(Self {
            owner: c.identity(),
            cursor,
            state,
            current: range.clone(),
            current_kind: OriginKind::Literal,
            start: 0,
            less_than: range.clone(),
            slash: range.clone(),
            pending_syntax: range,
            temporary,
            last_start: name,
            tag: None,
            comment: None,
            doctype: None,
            pi_target: None,
            pi_data: None,
            pending: std::array::from_fn(|_| None),
            pending_len: 0,
            done: false,
        })
    }
    /// The tree owner supplies the actual adjusted-current-node namespace fact
    /// at each pull. Only markup declaration dispatch consumes this fact.
    pub(crate) fn next(
        &mut self,
        c: &mut ParseControl,
        cdata_allowed: bool,
    ) -> Result<Option<Event>, ParseStop> {
        c.require_identity(&self.owner)?;
        while self.pending_len == 0 && !self.done {
            c.charge(WorkCategory::StateStep, 1)?;
            self.step(c, cdata_allowed)?;
        }
        c.checkpoint()?;
        if self.pending_len == 0 {
            return Ok(None);
        }
        // Each taken source slot is written to None, each subsequent event
        // replaces a destination slot, and the persistent length is updated.
        // This fixed bulk move is admitted before any slot is taken.
        admit_record_write(
            size_of::<Option<Event>>() * self.pending_len + size_of::<usize>(),
            2 * self.pending_len,
            c,
        )?;
        let first = self.pending[0].take();
        for i in 1..self.pending_len {
            self.pending[i - 1] = self.pending[i].take();
        }
        self.pending_len -= 1;
        Ok(first)
    }
    /// Applied only between pulls by the concrete tree algorithm.
    pub(crate) fn set_state(
        &mut self,
        state: State,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        c.require_identity(&self.owner)?;
        admit_record_write(size_of::<State>(), 1, c)?;
        self.state = state;
        Ok(())
    }
    pub(crate) fn release(mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.temporary.release(c)?;
        self.last_start.release(c)?;
        if let Some(t) = self.tag {
            t.release(c)?;
        }
        if let Some(t) = self.comment {
            t.release(c)?;
        }
        if let Some(t) = self.doctype {
            t.release(c)?;
        }
        if let Some(t) = self.pi_target {
            t.release(c)?;
        }
        if let Some(t) = self.pi_data {
            t.release(c)?;
        }
        for event in self.pending.iter_mut().filter_map(Option::take) {
            event.release(c)?;
        }
        Ok(())
    }
    fn range(
        &self,
        start: usize,
        end: usize,
        c: &mut ParseControl,
    ) -> Result<HtmlSourceRange, ParseStop> {
        self.cursor.checked_range(start, end, c)
    }
    fn read(&mut self, c: &mut ParseControl) -> Result<Option<u32>, ParseStop> {
        match self.cursor.consume(c)? {
            InputRead::Character(ch) => {
                if ch.source() != &self.current {
                    admit_record_write(size_of::<OriginKind>(), 1, c)?;
                    self.current_kind = OriginKind::Literal;
                }
                admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
                self.current = ch.source().clone();
                if let Some(record) = ch.preprocessing() {
                    if let super::input::PreprocessingRecord::Diagnostic(code) = record {
                        let code = match code {
                            super::input::InputDiagnostic::Surrogate => {
                                DiagnosticCode::SurrogateInInputStream
                            }
                            super::input::InputDiagnostic::Noncharacter => {
                                DiagnosticCode::NoncharacterInInputStream
                            }
                            super::input::InputDiagnostic::ControlCharacter => {
                                DiagnosticCode::ControlCharacterInInputStream
                            }
                        };
                        self.error_at(code, DiagnosticPhase::InputStream, c)?;
                    } else {
                        admit_record_write(size_of::<OriginKind>(), 1, c)?;
                        self.current_kind = OriginKind::Newline;
                        let mut retention = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
                        if let Err(stop) = retention.try_push(c, self.current.clone()) {
                            retention.release(c)?;
                            return Err(stop);
                        }
                        self.queue(
                            Event::Preprocessing {
                                record,
                                source: self.current.clone(),
                                retention,
                            },
                            c,
                        )?;
                    }
                }
                Ok(Some(ch.code_point().value()))
            }
            InputRead::Eof(range) => {
                admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
                self.current = range;
                Ok(None)
            }
        }
    }
    fn reconsume(&mut self, state: State, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.cursor.reconsume(c)?;
        self.set_state(state, c)
    }
    fn queue(&mut self, event: Event, c: &mut ParseControl) -> Result<(), ParseStop> {
        if self.pending_len == self.pending.len() {
            event.release(c)?;
            return Err(c.fail_invariant(Invariant::InputCursorState));
        }
        if let Err(stop) = admit_record_write(size_of::<Option<Event>>() + size_of::<usize>(), 2, c)
        {
            event.release(c)?;
            return Err(stop);
        }
        self.pending[self.pending_len] = Some(event);
        self.pending_len += 1;
        Ok(())
    }
    fn set_current(
        &mut self,
        source: &HtmlSourceRange,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.current = source.clone();
        Ok(())
    }
    fn remember_slash(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.slash = self.current.clone();
        Ok(())
    }
    fn remember_current_syntax(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.pending_syntax = self.current.clone();
        Ok(())
    }
    fn remember_syntax(
        &mut self,
        source: HtmlSourceRange,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.pending_syntax = source;
        Ok(())
    }
    fn error(&mut self, code: DiagnosticCode, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.error_at(code, DiagnosticPhase::Tokenizer, c)
    }
    fn error_at(
        &mut self,
        code: DiagnosticCode,
        phase: DiagnosticPhase,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let mut diagnostics = OwnedVec::new(HtmlParseResource::Diagnostics)?;
        if let Err(stop) = diagnostics.try_push(
            c,
            Diagnostic {
                code,
                phase,
                state: self.state,
                source: self.current.clone(),
            },
        ) {
            diagnostics.release(c)?;
            return Err(stop);
        }
        self.queue(Event::Diagnostic(diagnostics), c)
    }
    fn emit(
        &mut self,
        kind: TokenKind,
        start: usize,
        end: usize,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let source = match self.range(start, end, c) {
            Ok(source) => source,
            Err(stop) => {
                kind.release(c)?;
                return Err(stop);
            }
        };
        self.queue(Event::Token(TokenEvent::new(Token { kind, source }, c)?), c)
    }
    fn emit_point(
        &mut self,
        point: u32,
        source: &HtmlSourceRange,
        kind: OriginKind,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let text = DraftText::build(c, |text, c| text.push(point, source, kind, c))?;
        self.emit(TokenKind::Text(text), source.start(), source.end(), c)
    }
    fn character(&mut self, point: u32, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.emit_point(point, &self.current.clone(), self.current_kind, c)
    }
    fn replacement(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.error(DiagnosticCode::UnexpectedNullCharacter, c)?;
        self.emit_point(
            0xFFFD,
            &self.current.clone(),
            OriginKind::NullReplacement,
            c,
        )
    }
    fn clear_temporary(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.temporary.value.clear(c)?;
        self.temporary.origins.clear(c)
    }
    fn temp_matches(&self, expected: &[u8], c: &mut ParseControl) -> Result<bool, ParseStop> {
        ascii_eq(self.temporary.value.units(), expected, false, c)
    }
    fn appropriate_end(&self, c: &mut ParseControl) -> Result<bool, ParseStop> {
        let tag = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        let a = tag.name.value.units();
        let b = self.last_start.units();
        if a.len() != b.len() {
            c.charge(WorkCategory::StateStep, 1)?;
            return Ok(false);
        }
        for (a, b) in a.iter().zip(b) {
            c.charge(WorkCategory::InspectByte, 4)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn create_tag(&mut self, kind: TagKind, c: &mut ParseControl) -> Result<(), ParseStop> {
        if self.tag.is_some() {
            admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
            if let Some(old) = self.tag.take() {
                old.release(c)?;
            }
        }
        admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
        self.tag = Some(DraftTag {
            kind,
            name: DraftText::new(c)?,
            self_closing: false,
            attributes: OwnedVec::new(HtmlParseResource::TokenAttributes)?,
            attribute: None,
            names: Names::new()?,
            had_duplicates: false,
        });
        Ok(())
    }
    fn tag_name(&mut self, point: u32, c: &mut ParseControl) -> Result<(), ParseStop> {
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c)?;
            OriginKind::NullReplacement
        } else if lower(point) != point {
            OriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .name
            .push(
                if point == 0 { 0xFFFD } else { lower(point) },
                &self.current,
                kind,
                c,
            )
    }
    fn create_attribute(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.finish_attribute(c)?;
        admit_record_write(size_of::<Option<DraftAttribute>>(), 1, c)?;
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .attribute = Some(DraftAttribute {
            name: DraftText::new(c)?,
            value: DraftText::new(c)?,
            start: self.current.start(),
            end: self.current.end(),
            name_checked: false,
            duplicate: false,
        });
        Ok(())
    }
    fn attribute_name(&mut self, point: u32, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.attribute_end(self.current.end(), c)?;
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c)?;
            OriginKind::NullReplacement
        } else if lower(point) != point {
            OriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        let tag = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        tag.attribute
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .name
            .push(
                if point == 0 { 0xFFFD } else { lower(point) },
                &self.current,
                kind,
                c,
            )
    }
    fn check_attribute_name(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        if let Some(a) = &mut t.attribute
            && !a.name_checked
        {
            let duplicate = t.names.insert(a.name.value.units(), c)?;
            admit_record_write(2 * size_of::<bool>(), 2, c)?;
            a.duplicate = duplicate;
            a.name_checked = true;
            if a.duplicate {
                admit_record_write(size_of::<bool>(), 1, c)?;
                t.had_duplicates = true;
                self.error(DiagnosticCode::DuplicateAttribute, c)?;
            }
        }
        Ok(())
    }
    fn attribute_value(
        &mut self,
        point: u32,
        kind: OriginKind,
        source: &HtmlSourceRange,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        self.attribute_end(source.end(), c)?;
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        t.attribute
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .value
            .push(point, source, kind, c)
    }
    fn finish_attribute(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.check_attribute_name(c)?;
        let t = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        let Some(a) = &t.attribute else {
            return Ok(());
        };
        // Range validation may reject work admission. Retain the whole draft
        // in its owner until that checked provider call and removal admission
        // have succeeded, so terminal release can still dispose both buffers.
        let source = if a.duplicate {
            None
        } else {
            Some(self.range(a.start, a.end, c)?)
        };
        admit_record_write(size_of::<Option<DraftAttribute>>(), 1, c)?;
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        if let Some(a) = t.attribute.take() {
            if a.duplicate {
                a.name.release(c)?;
                a.value.release(c)?;
            } else {
                let source = source.ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
                let (name, value) = freeze_pair(a.name, a.value, c)?;
                if let Err((stop, a)) = t.attributes.try_push_recover(
                    c,
                    Attribute {
                        name,
                        value,
                        source,
                    },
                ) {
                    a.release(c)?;
                    return Err(stop);
                }
            }
        }
        Ok(())
    }
    fn attribute_end(&mut self, end: usize, c: &mut ParseControl) -> Result<(), ParseStop> {
        if let Some(t) = &mut self.tag
            && let Some(a) = &mut t.attribute
        {
            admit_record_write(size_of::<usize>(), 1, c)?;
            a.end = end;
        }
        Ok(())
    }
    fn emit_tag(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.finish_attribute(c)?;
        if self.tag.as_ref().is_some_and(|t| t.kind == TagKind::End) {
            if self.tag.as_ref().is_some_and(|t| t.attributes.len() != 0) {
                self.error(DiagnosticCode::EndTagWithAttributes, c)?;
            }
            if self.tag.as_ref().is_some_and(|t| t.self_closing) {
                self.error(DiagnosticCode::EndTagWithTrailingSolidus, c)?;
            }
        }
        let t = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        if t.kind == TagKind::Start {
            self.last_start.clear(c)?;
            self.last_start.extend_units(t.name.value.units(), c)?;
        }
        self.set_state(State::Data, c)?;
        admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
        let mut t = self
            .tag
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        t.names.nodes.release(c)?;
        let name = match t.name.freeze(c) {
            Ok(name) => name,
            Err(stop) => {
                release_attributes(&mut t.attributes, c)?;
                t.attributes.release(c)?;
                return Err(stop);
            }
        };
        let tag = Tag {
            kind: t.kind,
            name,
            self_closing: t.self_closing,
            attributes: t.attributes,
            had_duplicate_attributes: t.had_duplicates,
        };
        self.emit(TokenKind::Tag(tag), self.start, self.current.end(), c)
    }
    fn create_comment(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        self.comment = Some(DraftText::new(c)?);
        Ok(())
    }
    fn comment_point(&mut self, point: u32, c: &mut ParseControl) -> Result<(), ParseStop> {
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c)?;
            OriginKind::NullReplacement
        } else {
            self.current_kind
        };
        self.comment
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .push(
                if point == 0 { 0xFFFD } else { point },
                &self.current,
                kind,
                c,
            )
    }
    fn comment_syntax(&mut self, syntax: &[u8], c: &mut ParseControl) -> Result<(), ParseStop> {
        let source = self.range(self.pending_syntax.start(), self.current.start(), c)?;
        for &p in syntax {
            c.charge(WorkCategory::InspectByte, 1)?;
            self.comment
                .as_mut()
                .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                .push(u32::from(p), &source, OriginKind::RecoverySyntax, c)?;
        }
        Ok(())
    }
    fn emit_comment(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.set_state(State::Data, c)?;
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        let text = self
            .comment
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .freeze(c)?;
        self.emit(TokenKind::Comment(text), self.start, self.current.end(), c)
    }
    fn doctype_name(&mut self, point: u32, c: &mut ParseControl) -> Result<(), ParseStop> {
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c)?;
            OriginKind::NullReplacement
        } else if lower(point) != point {
            OriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        if d.name.is_none() {
            admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
            d.name = Some(DraftText::new(c)?);
        }
        d.name
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .push(
                if point == 0 { 0xFFFD } else { lower(point) },
                &self.current,
                kind,
                c,
            )
    }
    fn create_identifier(
        &mut self,
        kind: Identifier,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        *match kind {
            Identifier::Public => &mut d.public,
            Identifier::System => &mut d.system,
        } = Some(DraftText::new(c)?);
        Ok(())
    }
    fn identifier_point(
        &mut self,
        kind: Identifier,
        point: u32,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let origin = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c)?;
            OriginKind::NullReplacement
        } else {
            self.current_kind
        };
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        match kind {
            Identifier::Public => &mut d.public,
            Identifier::System => &mut d.system,
        }
        .as_mut()
        .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
        .push(
            if point == 0 { 0xFFFD } else { point },
            &self.current,
            origin,
            c,
        )
    }
    fn quirks(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<bool>(), 1, c)?;
        self.doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .force_quirks = true;
        Ok(())
    }
    fn mark_self_closing(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<bool>(), 1, c)?;
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .self_closing = true;
        Ok(())
    }
    fn emit_doctype(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.set_state(State::Data, c)?;
        admit_record_write(size_of::<Option<DraftDoctype>>(), 1, c)?;
        let d = self
            .doctype
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        let mut drafts = [d.name, d.public, d.system];
        let mut frozen = std::array::from_fn::<_, 3, _>(|_| None);
        for i in 0..3 {
            if let Some(draft) = drafts[i].take() {
                match draft.freeze(c) {
                    Ok(text) => frozen[i] = Some(text),
                    Err(stop) => {
                        for draft in drafts.into_iter().flatten() {
                            draft.release(c)?;
                        }
                        for text in frozen.into_iter().flatten() {
                            text.release(c)?;
                        }
                        return Err(stop);
                    }
                }
            }
        }
        let [name, public_id, system_id] = frozen;
        self.emit(
            TokenKind::Doctype(Doctype {
                name,
                public_id,
                system_id,
                force_quirks: d.force_quirks,
            }),
            self.start,
            self.current.end(),
            c,
        )
    }
    fn consume_ascii(
        &mut self,
        bytes: &[u8],
        insensitive: bool,
        c: &mut ParseControl,
    ) -> Result<bool, ParseStop> {
        if !self.cursor.matches_ascii(
            bytes,
            if insensitive {
                AsciiComparison::Insensitive
            } else {
                AsciiComparison::Sensitive
            },
            c,
        )? {
            return Ok(false);
        }
        for _ in bytes {
            c.charge(WorkCategory::StateStep, 1)?;
            self.read(c)?;
        }
        Ok(true)
    }
    fn raw_fallback(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        if self.tag.is_some() {
            admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
            if let Some(tag) = self.tag.take() {
                tag.release(c)?;
            }
        }
        let text = DraftText::build(c, |text, c| {
            text.push(0x3C, &self.less_than, OriginKind::Literal, c)?;
            text.push(0x2F, &self.slash, OriginKind::Literal, c)?;
            for i in 0..self.temporary.value.len() {
                c.charge(WorkCategory::InspectByte, 2)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                let origin = &self.temporary.origins.as_slice()[i];
                text.push(
                    u32::from(self.temporary.value.units()[i]),
                    &origin.source,
                    origin.kind,
                    c,
                )?;
            }
            Ok(())
        })?;
        let end = self
            .temporary
            .origins
            .as_slice()
            .last()
            .map_or(self.slash.end(), |o| o.source.end());
        if let Err(stop) = self.clear_temporary(c) {
            text.release(c)?;
            return Err(stop);
        }
        self.emit(TokenKind::Text(text), self.less_than.start(), end, c)
    }
    fn pi_comment(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.create_comment(c)?;
        self.comment
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            .push(0x3F, &self.slash, OriginKind::Literal, c)?;
        for i in 0..self.temporary.value.len() {
            c.charge(WorkCategory::InspectByte, 2)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let origin = &self.temporary.origins.as_slice()[i];
            self.comment
                .as_mut()
                .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                .push(
                    u32::from(self.temporary.value.units()[i]),
                    &origin.source,
                    origin.kind,
                    c,
                )?;
        }
        self.clear_temporary(c)
    }
    fn emit_pi(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.set_state(State::Data, c)?;
        // Admit both removals before taking either nested owner.
        admit_record_write(2 * size_of::<Option<DraftText>>(), 2, c)?;
        let target = self
            .pi_target
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        let data = self
            .pi_data
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        let (target, data) = freeze_pair(target, data, c)?;
        self.emit(
            TokenKind::ProcessingInstruction { target, data },
            self.start,
            self.current.end(),
            c,
        )
    }
    fn create_doctype(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<Option<DraftDoctype>>(), 1, c)?;
        self.doctype = Some(DraftDoctype::empty());
        Ok(())
    }
    fn finish_pi_target(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let replacement = DraftText::new(c)?;
        let data = DraftText::new(c)?;
        // Temporary removal, target insertion, and data insertion are a single
        // bounded transfer with no fallible operation after admission.
        if let Err(stop) = admit_record_write(
            size_of::<DraftText>() + 2 * size_of::<Option<DraftText>>(),
            3,
            c,
        ) {
            replacement.release(c)?;
            data.release(c)?;
            return Err(stop);
        }
        self.pi_target = Some(std::mem::replace(&mut self.temporary, replacement));
        self.pi_data = Some(data);
        Ok(())
    }
}
fn lower(p: u32) -> u32 {
    if (0x41..=0x5A).contains(&p) {
        p + 0x20
    } else {
        p
    }
}
fn alpha(p: u32) -> bool {
    (0x41..=0x5A).contains(&p) || (0x61..=0x7A).contains(&p)
}
fn alnum(p: u32) -> bool {
    alpha(p) || (0x30..=0x39).contains(&p)
}
fn space(p: u32) -> bool {
    matches!(p, 9 | 10 | 12 | 32)
}
fn ascii_eq(
    units: &[u16],
    bytes: &[u8],
    insensitive: bool,
    c: &mut ParseControl,
) -> Result<bool, ParseStop> {
    c.charge(WorkCategory::StateStep, 1)?;
    if units.len() != bytes.len() {
        return Ok(false);
    }
    for (unit, byte) in units.iter().zip(bytes) {
        c.charge(WorkCategory::InspectByte, 3)?;
        if (if insensitive {
            lower(u32::from(*unit))
        } else {
            u32::from(*unit)
        }) != u32::from(*byte)
        {
            return Ok(false);
        }
    }
    Ok(true)
}
