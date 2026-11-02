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
use crate::construction::evidence::{
    DiagnosticContext, DiagnosticKind, DiagnosticRule, ParseEvidence,
};
use crate::construction::provenance::{LexicalDisposition, LexicalKind, LexicalRef, LexicalRole};
mod char_ref;
mod lexical;
mod state;
pub(crate) mod states;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) mod token;

use super::{
    buffer::Utf16Buffer,
    control::{
        ControlIdentity, Invariant, OwnedVec, ParseControl, ParseStop, WORK_QUANTUM, WorkCategory,
    },
    input::{AsciiComparison, AsciiMatch, InputCursor, InputRead},
};
use crate::{DocumentString, HtmlSource, HtmlSourceRange};
use states::{Identifier, State};
use std::mem::size_of;
use token::{
    Attribute, DiagnosticCode, DiagnosticPhase, Doctype, DraftText, Event, OriginKind, Tag,
    TagKind, Token, TokenEvent, TokenKind, freeze_pair,
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
impl Branch {
    /// The child for one decided bit; a bit has exactly two children.
    fn child(&self, bit: bool) -> Option<usize> {
        let [zero, one] = self.next;
        if bit { one } else { zero }
    }
    fn child_mut(&mut self, bit: bool) -> &mut Option<usize> {
        let [zero, one] = &mut self.next;
        if bit { one } else { zero }
    }
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
            self.nodes.reserve_geometric(c, 1)?;
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
                let bit = (unit >> shift) & 1 == 1;
                c.charge(WorkCategory::InspectByte, size_of::<Option<usize>>() as u64)?;
                let next = self
                    .nodes
                    .get_mut(index, c)?
                    .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
                    .child(bit);
                index = if let Some(next) = next {
                    next
                } else {
                    let next = self.nodes.len();
                    self.nodes.reserve_geometric(c, 1)?;
                    self.nodes.try_push(
                        c,
                        Branch {
                            next: [None; 2],
                            terminal: false,
                        },
                    )?;
                    c.charge(WorkCategory::CopyByte, size_of::<Option<usize>>() as u64)?;
                    c.charge(WorkCategory::RecordWrite, 1)?;
                    *self
                        .nodes
                        .get_mut(index, c)?
                        .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
                        .child_mut(bit) = Some(next);
                    next
                };
            }
        }
        c.charge(WorkCategory::InspectByte, 1)?;
        let duplicate = self
            .nodes
            .get_mut(index, c)?
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .terminal;
        admit_record_write(size_of::<bool>(), 1, c)?;
        self.nodes
            .get_mut(index, c)?
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
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
    lexical: LexicalRef,
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
pub(crate) enum TokenizerPull {
    Event(Event),
    NeedInput,
    Exhausted,
}
#[derive(Clone, Copy, PartialEq)]
enum StepProgress {
    Advanced,
    NeedInput,
}
#[derive(Clone, Copy)]
enum ProofRole {
    LessThan,
    Slash,
    Attribute,
    Reference,
}
/// Anchors that retain one boundary proof, not a start/end pair.
#[derive(Clone, Copy)]
enum SingleProof {
    Start,
    Declaration,
    AttributeEnd,
}
/// Only finite grammar-owned anchors are retained, never a source-boundary map.
struct TokenizerProofs {
    current: (
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
    ),
    previous: (
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
    ),
    start: crate::source::ConsumedSourceBoundary,
    less_than: (
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
    ),
    slash: (
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
    ),
    pending: (
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
    ),
    declaration: crate::source::ConsumedSourceBoundary,
    attribute: (
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
    ),
    reference: (
        crate::source::ConsumedSourceBoundary,
        crate::source::ConsumedSourceBoundary,
    ),
}
impl TokenizerProofs {
    fn find(
        &self,
        offset: usize,
        c: &mut ParseControl,
    ) -> Result<crate::source::ConsumedSourceBoundary, ParseStop> {
        for proof in [
            &self.current.0,
            &self.current.1,
            &self.previous.0,
            &self.previous.1,
            &self.start,
            &self.less_than.0,
            &self.less_than.1,
            &self.slash.0,
            &self.slash.1,
            &self.pending.0,
            &self.pending.1,
            &self.pending.2,
            &self.declaration,
            &self.attribute.0,
            &self.attribute.1,
            &self.reference.0,
            &self.reference.1,
        ] {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, size_of::<usize>() as u64)?;
            if proof.offset() == offset {
                c.charge(
                    WorkCategory::CopyByte,
                    size_of::<crate::source::ConsumedSourceBoundary>() as u64,
                )?;
                return Ok(proof.clone());
            }
        }
        Err(c.fail_invariant(Invariant::TokenizerState))
    }
}
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
    lexical: Option<LexicalRef>,
    proofs: Option<TokenizerProofs>,
    reference: Option<char_ref::ReferenceContinuation>,
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
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.reference.is_some() {
            return Err(TokenizerHandoffError::ActiveDraft);
        }
        Ok(())
    }
    /// Returns the qualified next position without decoding a next item. The
    /// saved current start is used for reconsume, otherwise the committed cursor
    /// endpoint. Script handoff requires Data with no queued events/drafts.
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
        Ok(self.cursor.next_anchor(c)?)
    }

    pub(crate) fn new(
        source: &'source HtmlSource,
        state: State,
        last_start: Option<&DocumentString>,
        c: &mut ParseControl,
    ) -> Result<Self, ParseStop> {
        Self::create(
            source,
            state,
            last_start,
            #[cfg(test)]
            false,
            c,
        )
    }
    #[cfg(test)]
    pub(crate) fn progressive(
        source: &'source HtmlSource,
        state: State,
        last_start: Option<&DocumentString>,
        c: &mut ParseControl,
    ) -> Result<Self, ParseStop> {
        Self::create(source, state, last_start, true, c)
    }
    fn create(
        source: &'source HtmlSource,
        state: State,
        last_start: Option<&DocumentString>,
        #[cfg(test)] progressive: bool,
        c: &mut ParseControl,
    ) -> Result<Self, ParseStop> {
        c.require_source(source)?;
        #[cfg(test)]
        let cursor = if progressive {
            InputCursor::progressive(source, c)?
        } else {
            InputCursor::new(source, c)?
        };
        #[cfg(not(test))]
        let cursor = InputCursor::new(source, c)?;
        #[cfg(test)]
        let proofs = if progressive {
            let pair = cursor.consumed_span(c)?.expect("actual progressive span");
            admit_record_write(size_of::<TokenizerProofs>(), 10, c)?;
            Some(TokenizerProofs {
                current: pair.clone(),
                previous: pair.clone(),
                start: pair.0.clone(),
                less_than: pair.clone(),
                slash: pair.clone(),
                pending: (pair.0.clone(), pair.0.clone(), pair.1.clone()),
                declaration: pair.0.clone(),
                attribute: pair.clone(),
                reference: pair,
            })
        } else {
            None
        };
        #[cfg(not(test))]
        let proofs = None;
        let temporary = DraftText::new(c)?;
        let mut name = Utf16Buffer::new(c)?;
        if let Some(last) = last_start
            && let Err(stop) = name.extend_units(last.code_units(), c)
        {
            name.release(c)?;
            temporary.release(c)?;
            return Err(stop);
        }
        #[cfg(test)]
        let range = if progressive {
            cursor.next_anchor(c)
        } else {
            cursor.checked_range(0, 0, c)
        };
        #[cfg(not(test))]
        let range = cursor.checked_range(0, 0, c);
        let range = match range {
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
            lexical: None,
            proofs,
            reference: None,
        })
    }
    /// The tree owner supplies the actual adjusted-current-node namespace fact
    /// at each pull. Only markup declaration dispatch consumes this fact.
    pub(crate) fn next(
        &mut self,
        c: &mut ParseControl,
        cdata_allowed: bool,
        evidence: &mut ParseEvidence,
    ) -> Result<TokenizerPull, ParseStop> {
        c.require_identity(&self.owner)?;
        while self.pending_len == 0 && !self.done {
            c.charge(WorkCategory::StateStep, 1)?;
            let progress = if self.reference.is_some() {
                self.resume_reference(c, evidence)?
            } else {
                self.step(c, cdata_allowed, evidence)?
            };
            if progress == StepProgress::NeedInput {
                c.checkpoint()?;
                if self.pending_len == 0 {
                    return Ok(TokenizerPull::NeedInput);
                }
                break;
            }
        }
        c.checkpoint()?;
        let Some(remaining) = self.pending_len.checked_sub(1) else {
            return Ok(TokenizerPull::Exhausted);
        };
        // Each taken source slot is written to None, each subsequent event
        // replaces a destination slot, and the persistent length is updated.
        // This fixed bulk move is admitted before any slot is taken.
        let (Some(bytes), Some(writes)) = (
            size_of::<Option<Event>>()
                .checked_mul(self.pending_len)
                .and_then(|bytes| bytes.checked_add(size_of::<usize>())),
            self.pending_len.checked_mul(2),
        ) else {
            return c.representation(HtmlParseResource::AtomicLayoutBytes);
        };
        admit_record_write(bytes, writes, c)?;
        let Some(live) = self.pending.get_mut(..self.pending_len) else {
            return Err(c.fail_invariant(Invariant::TokenizerState));
        };
        let Some(front) = live.first_mut() else {
            return Err(c.fail_invariant(Invariant::TokenizerState));
        };
        let first = front.take();
        // The emptied front moves behind the remaining events, which each
        // advance one slot; the vacated slot is then no longer live.
        live.rotate_left(1);
        self.pending_len = remaining;
        match first {
            Some(event) => Ok(TokenizerPull::Event(event)),
            None => Err(c.fail_invariant(Invariant::TokenizerState)),
        }
    }
    #[cfg(test)]
    pub(crate) fn make_available(
        &mut self,
        through: usize,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        self.cursor.make_available(through, c)
    }
    #[cfg(test)]
    pub(crate) fn seal_input(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.cursor.seal_input(c)
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
        if let Some(reference) = self.reference {
            reference.release(c)?;
        }
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
        if self.proofs.is_some() {
            let start = self.boundary_proof(start, c)?;
            let end = self.boundary_proof(end, c)?;
            self.cursor.range_between(&start, &end, c)
        } else {
            self.cursor.checked_range(start, end, c)
        }
    }
    fn boundary_proof(
        &self,
        offset: usize,
        c: &mut ParseControl,
    ) -> Result<crate::source::ConsumedSourceBoundary, ParseStop> {
        self.proofs
            .as_ref()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .find(offset, c)
    }
    fn sync_proofs(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        let Some(p) = self.proofs.as_mut() else {
            return Ok(());
        };
        if let Some(pair) = self.cursor.consumed_span(c)? {
            admit_record_write(
                const { 4 * size_of::<crate::source::ConsumedSourceBoundary>() },
                2,
                c,
            )?;
            p.previous = p.current.clone();
            p.current = pair;
        }
        Ok(())
    }
    fn save_proof_range(
        &mut self,
        role: ProofRole,
        start: usize,
        end: usize,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let Some(p) = self.proofs.as_mut() else {
            return Ok(());
        };
        let a = p.find(start, c)?;
        let b = p.find(end, c)?;
        admit_record_write(
            const { 2 * size_of::<crate::source::ConsumedSourceBoundary>() },
            1,
            c,
        )?;
        match role {
            ProofRole::LessThan => p.less_than = (a, b),
            ProofRole::Slash => p.slash = (a, b),
            ProofRole::Attribute => p.attribute = (a, b),
            ProofRole::Reference => p.reference = (a, b),
        }
        Ok(())
    }
    /// Retains the pending syntax range together with the current start.
    fn save_syntax_proof(
        &mut self,
        start: usize,
        end: usize,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let Some(p) = self.proofs.as_mut() else {
            return Ok(());
        };
        let a = p.find(start, c)?;
        let b = p.find(end, c)?;
        let middle = p.find(self.current.start(), c)?;
        admit_record_write(
            const { 3 * size_of::<crate::source::ConsumedSourceBoundary>() },
            1,
            c,
        )?;
        p.pending = (a, middle, b);
        Ok(())
    }
    /// Retains one proved boundary of a progressive tokenizer; a complete
    /// tokenizer keeps no proofs.
    fn save_single_proof(
        &mut self,
        role: SingleProof,
        offset: usize,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        if !self.cursor.is_progressive() {
            return Ok(());
        }
        let p = self
            .proofs
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        let proof = p.find(offset, c)?;
        admit_record_write(size_of::<crate::source::ConsumedSourceBoundary>(), 1, c)?;
        match role {
            SingleProof::Start => p.start = proof,
            SingleProof::Declaration => p.declaration = proof,
            SingleProof::AttributeEnd => p.attribute.1 = proof,
        }
        Ok(())
    }
    fn read(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<Option<u32>, ParseStop> {
        let read = if self.cursor.is_progressive() {
            match self.cursor.poll_consume(c)? {
                super::input::InputPoll::Ready(read) => read,
                super::input::InputPoll::NeedInput => {
                    return Err(c.fail_invariant(Invariant::InputCursorState));
                }
            }
        } else {
            self.cursor.consume(c)?
        };
        self.sync_proofs(c)?;
        match read {
            InputRead::Character(ch) => {
                if ch.source() != &self.current {
                    admit_record_write(size_of::<OriginKind>(), 1, c)?;
                    self.current_kind = OriginKind::Literal;
                }
                admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
                self.current = ch.source().clone();
                if let Some(record) = ch.preprocessing() {
                    if let super::input::PreprocessingRecord::Diagnostic(code) = record {
                        self.error_at(
                            Self::input_stream_code(code),
                            DiagnosticPhase::InputStream,
                            c,
                            evidence,
                        )?;
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
                                _record: record,
                                _source: self.current.clone(),
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
        let Some(slot) = self.pending.get_mut(self.pending_len) else {
            event.release(c)?;
            return Err(c.fail_invariant(Invariant::TokenizerState));
        };
        if let Err(stop) = admit_record_write(
            const { size_of::<Option<Event>>() + size_of::<usize>() },
            2,
            c,
        ) {
            event.release(c)?;
            return Err(stop);
        }
        *slot = Some(event);
        // The slot exists, so its index is below the fixed slot count.
        self.pending_len = self.pending_len.saturating_add(1);
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
        self.save_proof_range(
            ProofRole::Slash,
            self.current.start(),
            self.current.end(),
            c,
        )?;
        self.slash = self.current.clone();
        Ok(())
    }
    fn remember_current_syntax(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.save_syntax_proof(self.current.start(), self.current.end(), c)?;
        self.pending_syntax = self.current.clone();
        Ok(())
    }
    fn remember_syntax(
        &mut self,
        source: HtmlSourceRange,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.save_syntax_proof(source.start(), source.end(), c)?;
        self.pending_syntax = source;
        Ok(())
    }
    fn error(
        &mut self,
        code: DiagnosticCode,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.error_at(code, DiagnosticPhase::Tokenizer, c, evidence)
    }
    fn input_stream_code(diagnostic: super::input::InputDiagnostic) -> DiagnosticCode {
        match diagnostic {
            super::input::InputDiagnostic::Surrogate => DiagnosticCode::SurrogateInInputStream,
            super::input::InputDiagnostic::Noncharacter => {
                DiagnosticCode::NoncharacterInInputStream
            }
            super::input::InputDiagnostic::ControlCharacter => {
                DiagnosticCode::ControlCharacterInInputStream
            }
        }
    }
    fn error_at(
        &mut self,
        code: DiagnosticCode,
        phase: DiagnosticPhase,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let source = self.current.clone();
        self.error_at_source(code, phase, &source, c, evidence)
    }
    /// Reports at an explicit input frontier: the character a lookahead
    /// examined is ahead of the last consumed one.
    fn error_at_source(
        &mut self,
        code: DiagnosticCode,
        phase: DiagnosticPhase,
        source: &HtmlSourceRange,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let id = evidence.append(
            c,
            DiagnosticKind::Tokenizer(code),
            DiagnosticContext::Tokenizer {
                phase,
                state: self.state,
            },
            DiagnosticRule {
                anchor: code.rule_anchor(),
                branch: code.name(),
            },
            crate::construction::evidence::DiagnosticLocationInput {
                source,
                role: crate::construction::evidence::DiagnosticLocationRole::InputFrontier,
                token: None,
            },
        )?;
        self.queue(Event::Diagnostic(id), c)
    }

    fn emit(
        &mut self,
        kind: TokenKind,
        start: usize,
        end: usize,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let source = match self.range(start, end, c) {
            Ok(source) => source,
            Err(stop) => {
                kind.release(c)?;
                return Err(stop);
            }
        };
        let lexical = match self.retain_emitted_lexical(&kind, &source, c, evidence) {
            Ok(reference) => reference,
            Err(stop) => {
                kind.release(c)?;
                return Err(stop);
            }
        };
        self.queue(
            Event::Token(TokenEvent::new(
                Token {
                    kind,
                    source,
                    lexical,
                },
                c,
            )?),
            c,
        )
    }
    fn emit_point(
        &mut self,
        point: u32,
        source: &HtmlSourceRange,
        kind: OriginKind,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let text = DraftText::build(c, |text, c| text.push(point, source, kind, c))?;
        self.emit(
            TokenKind::Text(text),
            source.start(),
            source.end(),
            c,
            evidence,
        )
    }
    fn character(
        &mut self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.emit_current_point(point, self.current_kind, c, evidence)
    }
    fn emit_current_point(
        &mut self,
        point: u32,
        kind: OriginKind,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let text = DraftText::build(c, |text, c| text.push(point, &self.current, kind, c))?;
        self.emit(
            TokenKind::Text(text),
            self.current.start(),
            self.current.end(),
            c,
            evidence,
        )
    }
    fn replacement(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.error(DiagnosticCode::UnexpectedNullCharacter, c, evidence)?;
        self.emit_current_point(0xFFFD, OriginKind::NullReplacement, c, evidence)
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
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
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
    fn tag_name(
        &mut self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c, evidence)?;
            OriginKind::NullReplacement
        } else if lower(point) != point {
            OriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .name
            .push(
                if point == 0 { 0xFFFD } else { lower(point) },
                &self.current,
                kind,
                c,
            )
    }
    fn create_attribute(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.finish_attribute(c, evidence)?;
        self.save_proof_range(
            ProofRole::Attribute,
            self.current.start(),
            self.current.end(),
            c,
        )?;
        let lexical = evidence.lexical_begin(
            c,
            &self.current,
            LexicalKind::Attribute,
            self.lexical.as_ref(),
        )?;
        admit_record_write(size_of::<Option<DraftAttribute>>(), 1, c)?;
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .attribute = Some(DraftAttribute {
            name: DraftText::new(c)?,
            value: DraftText::new(c)?,
            start: self.current.start(),
            end: self.current.end(),
            name_checked: false,
            duplicate: false,
            lexical,
        });
        Ok(())
    }
    fn attribute_name(
        &mut self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.attribute_end(self.current.end(), c)?;
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c, evidence)?;
            OriginKind::NullReplacement
        } else if lower(point) != point {
            OriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        let tag = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        tag.attribute
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .name
            .push(
                if point == 0 { 0xFFFD } else { lower(point) },
                &self.current,
                kind,
                c,
            )
    }
    fn check_attribute_name(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        if let Some(a) = &mut t.attribute
            && !a.name_checked
        {
            let duplicate = t.names.insert(a.name.value.units(), c)?;
            admit_record_write(const { 2 * size_of::<bool>() }, 2, c)?;
            a.duplicate = duplicate;
            a.name_checked = true;
            if a.duplicate {
                admit_record_write(size_of::<bool>(), 1, c)?;
                t.had_duplicates = true;
                self.error(DiagnosticCode::DuplicateAttribute, c, evidence)?;
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
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        t.attribute
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .value
            .push(point, source, kind, c)
    }
    fn finish_attribute(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.check_attribute_name(c, evidence)?;
        let t = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        let Some(a) = &t.attribute else {
            return Ok(());
        };
        // Range validation may reject work admission. Retain the whole draft
        // in its owner until that checked provider call and removal admission
        // have succeeded, so terminal release can still dispose both buffers.
        let source = self.range(a.start, a.end, c)?;
        Self::retain_lexical_text(
            evidence,
            c,
            &a.lexical,
            LexicalRole::Name,
            a.name.origins.as_slice(),
        )?;
        Self::retain_lexical_text(
            evidence,
            c,
            &a.lexical,
            LexicalRole::Value,
            a.value.origins.as_slice(),
        )?;
        if a.value.value.len() == 0 {
            let anchor = self.range(a.end, a.end, c)?;
            evidence.lexical_part(c, &a.lexical, LexicalRole::Value, &anchor, 0..0, None)?;
        }
        evidence.lexical_finish(
            c,
            &a.lexical,
            &source,
            LexicalKind::Attribute,
            if a.duplicate {
                LexicalDisposition::DuplicateAttribute
            } else if t.kind == TagKind::End {
                LexicalDisposition::EndTagAttribute
            } else {
                LexicalDisposition::StoredAttribute
            },
        )?;
        if !a.duplicate {
            // Reserve while both draft buffers remain in their original owner.
            // A denied physical layout must not consume their cleanup tickets.
            self.tag
                .as_mut()
                .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
                .attributes
                .reserve_geometric(c, 1)?;
        }
        admit_record_write(size_of::<Option<DraftAttribute>>(), 1, c)?;
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        if let Some(a) = t.attribute.take() {
            if a.duplicate {
                a.name.release(c)?;
                a.value.release(c)?;
            } else {
                let (name, value) = freeze_pair(a.name, a.value, c)?;
                if let Err((stop, a)) = t.attributes.try_push_recover(
                    c,
                    Attribute {
                        lexical: Some(a.lexical),
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
        self.save_single_proof(SingleProof::AttributeEnd, end, c)?;
        if let Some(t) = &mut self.tag
            && let Some(a) = &mut t.attribute
        {
            admit_record_write(size_of::<usize>(), 1, c)?;
            a.end = end;
        }
        Ok(())
    }
    fn emit_tag(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.finish_attribute(c, evidence)?;
        if self.tag.as_ref().is_some_and(|t| t.kind == TagKind::End) {
            if self.tag.as_ref().is_some_and(|t| t.attributes.len() != 0) {
                self.error(DiagnosticCode::EndTagWithAttributes, c, evidence)?;
            }
            if self.tag.as_ref().is_some_and(|t| t.self_closing) {
                self.error(DiagnosticCode::EndTagWithTrailingSolidus, c, evidence)?;
            }
        }
        let t = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        if t.kind == TagKind::Start {
            self.last_start.clear(c)?;
            self.last_start.extend_units(t.name.value.units(), c)?;
        }
        self.set_state(State::Data, c)?;
        admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
        let mut t = self
            .tag
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
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
        self.emit(
            TokenKind::Tag(tag),
            self.start,
            self.current.end(),
            c,
            evidence,
        )
    }
    fn create_comment(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        self.comment = Some(DraftText::new(c)?);
        Ok(())
    }
    fn comment_point(
        &mut self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c, evidence)?;
            OriginKind::NullReplacement
        } else {
            self.current_kind
        };
        self.comment
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
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
                .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
                .push(u32::from(p), &source, OriginKind::RecoverySyntax, c)?;
        }
        Ok(())
    }
    fn emit_comment(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.set_state(State::Data, c)?;
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        let text = self
            .comment
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .freeze(c)?;
        self.emit(
            TokenKind::Comment(text),
            self.start,
            self.current.end(),
            c,
            evidence,
        )
    }
    fn doctype_name(
        &mut self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let kind = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c, evidence)?;
            OriginKind::NullReplacement
        } else if lower(point) != point {
            OriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        if d.name.is_none() {
            admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
            d.name = Some(DraftText::new(c)?);
        }
        d.name
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
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
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
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
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        let origin = if point == 0 {
            self.error(DiagnosticCode::UnexpectedNullCharacter, c, evidence)?;
            OriginKind::NullReplacement
        } else {
            self.current_kind
        };
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        match kind {
            Identifier::Public => &mut d.public,
            Identifier::System => &mut d.system,
        }
        .as_mut()
        .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
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
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .force_quirks = true;
        Ok(())
    }
    fn mark_self_closing(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<bool>(), 1, c)?;
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .self_closing = true;
        Ok(())
    }
    fn emit_doctype(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.set_state(State::Data, c)?;
        admit_record_write(size_of::<Option<DraftDoctype>>(), 1, c)?;
        let d = self
            .doctype
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        let mut frozen = std::array::from_fn::<_, 3, _>(|_| None);
        let mut slots = [d.name, d.public, d.system].into_iter().zip(&mut frozen);
        while let Some((draft, slot)) = slots.next() {
            if let Some(draft) = draft {
                match draft.freeze(c) {
                    Ok(text) => *slot = Some(text),
                    Err(stop) => {
                        for (draft, _) in slots {
                            if let Some(draft) = draft {
                                draft.release(c)?;
                            }
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
            evidence,
        )
    }
    fn consume_ascii(
        &mut self,
        bytes: &[u8],
        insensitive: bool,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<AsciiMatch, ParseStop> {
        let comparison = if insensitive {
            AsciiComparison::Insensitive
        } else {
            AsciiComparison::Sensitive
        };
        let lookahead = if self.cursor.is_progressive() {
            self.cursor.poll_matches_ascii(bytes, comparison, c)?
        } else {
            let lookahead = self.cursor.matches_ascii(bytes, comparison, c)?;
            super::input::Lookahead {
                result: if lookahead.result {
                    AsciiMatch::Matched
                } else {
                    AsciiMatch::Mismatched
                },
                examined: lookahead.examined,
            }
        };
        // The state decides on the next input character, so that character's
        // stream error precedes whatever the mismatch reports.
        if let Some(examined) = lookahead.examined {
            self.error_at_source(
                Self::input_stream_code(examined.diagnostic),
                DiagnosticPhase::InputStream,
                &examined.source,
                c,
                evidence,
            )?;
        }
        let matched = lookahead.result;
        if matched != AsciiMatch::Matched {
            return Ok(matched);
        }
        for _ in bytes {
            c.charge(WorkCategory::StateStep, 1)?;
            self.read(c, evidence)?;
            if let Some(occurrence) = self.lexical.as_ref() {
                evidence.lexical_part(
                    c,
                    occurrence,
                    LexicalRole::Syntax,
                    &self.current,
                    0..0,
                    None,
                )?;
            }
        }
        Ok(AsciiMatch::Matched)
    }
    fn raw_fallback(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        if let Some(tag) = self.tag.as_ref() {
            if let Some(occurrence) = self.lexical.as_ref() {
                Self::retain_lexical_text(
                    evidence,
                    c,
                    occurrence,
                    LexicalRole::Name,
                    tag.name.origins.as_slice(),
                )?;
            }
            admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
            if let Some(tag) = self.tag.take() {
                tag.release(c)?;
            }
        }
        let text = DraftText::build(c, |text, c| {
            text.push(0x3C, &self.less_than, OriginKind::Literal, c)?;
            text.push(0x2F, &self.slash, OriginKind::Literal, c)?;
            self.temporary.require_paired(c)?;
            for (&unit, origin) in self
                .temporary
                .value
                .units()
                .iter()
                .zip(self.temporary.origins.as_slice())
            {
                c.charge(WorkCategory::InspectByte, 2)?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                text.push(u32::from(unit), &origin.source, origin.kind, c)?;
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
        self.emit(
            TokenKind::Text(text),
            self.less_than.start(),
            end,
            c,
            evidence,
        )
    }
    fn pi_comment(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        if let Some(occurrence) = self.lexical.as_ref() {
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                LexicalRole::Target,
                self.temporary.origins.as_slice(),
            )?;
        }
        self.create_comment(c)?;
        self.comment
            .as_mut()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
            .push(0x3F, &self.slash, OriginKind::Literal, c)?;
        self.temporary.require_paired(c)?;
        for (&unit, origin) in self
            .temporary
            .value
            .units()
            .iter()
            .zip(self.temporary.origins.as_slice())
        {
            c.charge(WorkCategory::InspectByte, 2)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            self.comment
                .as_mut()
                .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?
                .push(u32::from(unit), &origin.source, origin.kind, c)?;
        }
        self.clear_temporary(c)
    }
    fn emit_pi(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        self.set_state(State::Data, c)?;
        // Admit both removals before taking either nested owner.
        admit_record_write(const { 2 * size_of::<Option<DraftText>>() }, 2, c)?;
        let target = self
            .pi_target
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        let data = self
            .pi_data
            .take()
            .ok_or_else(|| c.fail_invariant(Invariant::TokenizerState))?;
        let (target, data) = freeze_pair(target, data, c)?;
        self.emit(
            TokenKind::ProcessingInstruction { target, data },
            self.start,
            self.current.end(),
            c,
            evidence,
        )
    }
    /// Drops the target and data drafts of an instruction that ends without a
    /// token, once its lexical occurrence has retained their parts.
    fn discard_pi(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        // Admit both removals before taking either nested owner.
        admit_record_write(const { 2 * size_of::<Option<DraftText>>() }, 2, c)?;
        let target = self.pi_target.take();
        let data = self.pi_data.take();
        if let Some(target) = target {
            target.release(c)?;
        }
        if let Some(data) = data {
            data.release(c)?;
        }
        Ok(())
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
            const { size_of::<DraftText>() + 2 * size_of::<Option<DraftText>>() },
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
        // Upper-case ASCII letters have bit 5 clear; setting it adds 0x20.
        p | 0x20
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
