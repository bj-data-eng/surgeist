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
    DiagnosticAdmission, HtmlParseDiagnosticContext, HtmlParseDiagnosticKind,
    HtmlParseDiagnosticRule, ParseEvidence,
};
use crate::construction::provenance::{
    HtmlParseLexicalDisposition, HtmlParseLexicalKind, HtmlParseLexicalRole, LexicalRef,
};
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
        ControlIdentity, HtmlParseInvariant, HtmlParseStop, HtmlParseWorkCategory, OwnedVec,
        ParseControl, WORK_QUANTUM,
    },
    input::{AsciiComparison, InputCursor, InputRead},
};
use crate::{HtmlSource, HtmlSourceRange};
use states::{HtmlTokenizerIdentifier, HtmlTokenizerState};
use std::mem::size_of;
use surgeist_html::values::DocumentString;
use token::{
    Attribute, Doctype, DraftText, Event, HtmlParseOriginKind, HtmlTokenizerDiagnostic,
    HtmlTokenizerDiagnosticPhase, Tag, TagKind, Token, TokenEvent, TokenKind, freeze_pair,
};

/// Pre-admits explicit persistent record moves/writes, independently of state
/// dispatch. Larger indivisible fixed operations use the controller's atomic
/// layout exception; rejected admission precedes every useful mutation.
fn admit_record_write(
    bytes: usize,
    writes: usize,
    c: &mut ParseControl,
) -> Result<(), HtmlParseStop> {
    c.charge(HtmlParseWorkCategory::CopyByte, bytes as u64)?;
    c.charge(HtmlParseWorkCategory::RecordWrite, writes as u64)?;
    if bytes.saturating_add(writes) > WORK_QUANTUM {
        c.admit_atomic_layout(bytes)?;
    }
    Ok(())
}

/// One node of the attribute-name tree: a name a start tag has kept, or the
/// first bit at which the names below it differ, with the two subtrees that bit
/// separates and one name below it. The names below a branch agree on every bit
/// before the branch's.
#[derive(Clone, Copy)]
enum Node {
    Leaf {
        attribute: usize,
    },
    Branch {
        bit: usize,
        zero: usize,
        one: usize,
        /// The attribute index of one name below this branch.
        below: usize,
    },
}
impl Node {
    /// The child a decided bit leads to; a branch has exactly two.
    fn child(self, side: bool) -> Option<usize> {
        match self {
            Self::Branch { zero, one, .. } => Some(if side { one } else { zero }),
            Self::Leaf { .. } => None,
        }
    }
    fn set_child(&mut self, side: bool, child: usize) -> bool {
        match self {
            Self::Branch { zero, one, .. } => {
                *(if side { one } else { zero }) = child;
                true
            }
            Self::Leaf { .. } => false,
        }
    }
}

/// Bits per UTF-16 unit of a name as the tree reads it: a marker `1` that the
/// unit is present, then its sixteen bits, most significant first. A name ends
/// with a marker `0`, so no name is a prefix of another, and two different
/// names always differ at a bit both have.
const BITS_PER_UNIT: usize = 17;

/// The masks of a unit's sixteen bits, most significant first.
const UNIT_BIT_MASKS: [u16; 16] = [
    0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020, 0x0010,
    0x0008, 0x0004, 0x0002, 0x0001,
];

/// The bit of `units` at `position`, in the layout above. Past the end marker
/// every bit reads zero; two names are never compared beyond the first bit at
/// which they differ, so a bit past the end of one is never the deciding bit.
/// A position inside a unit has a mask, since it is less than seventeen; the
/// absence of one is a defect of the caller.
fn name_bit(units: &[u16], position: usize) -> Option<bool> {
    let (unit, within) = (position / BITS_PER_UNIT, position % BITS_PER_UNIT);
    match units.get(unit) {
        None => Some(false),
        Some(_) if within == 0 => Some(true),
        Some(&value) => UNIT_BIT_MASKS
            .get(within.saturating_sub(1))
            .map(|mask| value & mask != 0),
    }
}

/// The names of the attributes a start tag has kept, by index into its
/// attribute list, as a lookup needs them.
pub(crate) trait KeptNames {
    fn name(&self, index: usize) -> Option<&[u16]>;
}
impl KeptNames for [Attribute] {
    fn name(&self, index: usize) -> Option<&[u16]> {
        self.get(index)
            .map(|attribute| attribute.name.value.text().code_units())
    }
}

/// Compressed binary radix tree over the bits of the attribute names a start
/// tag has kept (the names are stored once, in the attribute list): one leaf
/// for each name and one branch for each pair of subtrees, so two nodes for a
/// name however long it is.
///
/// A lookup follows the probe's bits down from the root and stops at the first
/// branch whose bit lies past the probe's end marker. The names below such a
/// branch agree on every bit before it, the marker's among them, and the probe
/// differs from them at or before that marker, so any one of them gives the
/// first bit at which the probe differs from the kept names; the branch holds
/// one. The branches read are therefore at distinct bit positions up to the end
/// marker, at most `17 * units + 1` of them, and the same bound holds for the
/// walk that places a new branch, so a lookup takes at most
/// `2 * (17 * units + 2)` steps and compares the probe once with one kept name.
/// The work of looking up a tag's names is linear in their total length,
/// whatever the names before them are.
pub(crate) struct Names {
    nodes: OwnedVec<Node>,
    root: usize,
}
impl Names {
    pub(crate) fn new() -> Result<Self, HtmlParseStop> {
        Ok(Self {
            nodes: OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            root: 0,
        })
    }
    /// The first bit at which two names differ, or `None` for equal names.
    fn first_difference(
        stored: &[u16],
        units: &[u16],
        c: &mut ParseControl,
    ) -> Result<Option<usize>, HtmlParseStop> {
        for (at, (left, right)) in stored.iter().zip(units).enumerate() {
            // Two stored UTF-16 units are inspected.
            c.charge(HtmlParseWorkCategory::InspectByte, 4)?;
            if left != right {
                let bit = at
                    .checked_mul(BITS_PER_UNIT)
                    .and_then(|bits| bits.checked_add(1))
                    .and_then(|bits| bits.checked_add((left ^ right).leading_zeros() as usize));
                return bit
                    .map(Some)
                    .ok_or_else(|| c.stop_representation(HtmlParseResource::AuxiliaryEntries));
            }
        }
        if stored.len() == units.len() {
            return Ok(None);
        }
        // One name ends where the other continues: the end marker decides.
        stored
            .len()
            .min(units.len())
            .checked_mul(BITS_PER_UNIT)
            .map(Some)
            .ok_or_else(|| c.stop_representation(HtmlParseResource::AuxiliaryEntries))
    }
    /// The bit of `units` at `position`.
    fn side(units: &[u16], position: usize, c: &mut ParseControl) -> Result<bool, HtmlParseStop> {
        name_bit(units, position)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))
    }
    /// The child of `branch` that the name's bit at `bit` leads to.
    fn descend(
        branch: Node,
        units: &[u16],
        bit: usize,
        c: &mut ParseControl,
    ) -> Result<usize, HtmlParseStop> {
        let side = Self::side(units, bit, c)?;
        branch
            .child(side)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))
    }
    fn node(&mut self, at: usize, c: &mut ParseControl) -> Result<Node, HtmlParseStop> {
        self.nodes
            .get_mut(at, c)?
            .copied()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))
    }
    fn push(&mut self, node: Node, c: &mut ParseControl) -> Result<usize, HtmlParseStop> {
        let at = self.nodes.len();
        self.nodes.reserve_geometric(c, 1)?;
        self.nodes.try_push(c, node)?;
        Ok(at)
    }
    /// Records a name and reports whether an equal one was recorded before. The
    /// name will be the attribute at index `kept.len()` if it is not a
    /// duplicate, and `kept` lists the names recorded so far.
    pub(crate) fn insert(
        &mut self,
        units: &[u16],
        kept: &(impl KeptNames + ?Sized),
        index: usize,
        c: &mut ParseControl,
    ) -> Result<bool, HtmlParseStop> {
        let leaf = Node::Leaf { attribute: index };
        if self.nodes.len() == 0 {
            self.root = self.push(leaf, c)?;
            return Ok(false);
        }
        // Follow the name's bits to the leaf whose name shares the most with it, or to the
        // first branch past its end marker, which holds a name that shares as much.
        let end = units
            .len()
            .checked_mul(BITS_PER_UNIT)
            .ok_or_else(|| c.stop_representation(HtmlParseResource::AuxiliaryEntries))?;
        let mut at = self.root;
        let stored = loop {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            match self.node(at, c)? {
                Node::Leaf { attribute } => break attribute,
                Node::Branch { bit, below, .. } if bit > end => break below,
                branch @ Node::Branch { bit, .. } => {
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        size_of::<usize>() as u64,
                    )?;
                    at = Self::descend(branch, units, bit, c)?;
                }
            }
        };
        let stored = kept
            .name(stored)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        let Some(differ) = Self::first_difference(stored, units, c)? else {
            return Ok(true);
        };
        // The new branch goes above the first node whose own bit is not before
        // the one at which the names differ.
        let mut parent: Option<(usize, bool)> = None;
        let mut at = self.root;
        loop {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            match self.node(at, c)? {
                branch @ Node::Branch { bit, .. } if bit < differ => {
                    let side = Self::side(units, bit, c)?;
                    parent = Some((at, side));
                    at = Self::descend(branch, units, bit, c)?;
                }
                _ => break,
            }
        }
        let new_leaf = self.push(leaf, c)?;
        let side = Self::side(units, differ, c)?;
        let (zero, one) = if side { (at, new_leaf) } else { (new_leaf, at) };
        let branch = self.push(
            Node::Branch {
                bit: differ,
                zero,
                one,
                below: index,
            },
            c,
        )?;
        c.charge(HtmlParseWorkCategory::CopyByte, size_of::<usize>() as u64)?;
        c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        match parent {
            None => self.root = branch,
            Some((above, side)) => {
                let linked = self
                    .nodes
                    .get_mut(above, c)?
                    .is_some_and(|node| node.set_child(side, branch));
                if !linked {
                    return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
                }
            }
        }
        Ok(false)
    }
}
impl Names {
    /// Credits the storage of the recorded names.
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        self.nodes.release(c)
    }
}
struct DraftAttribute {
    name: DraftText,
    value: DraftText,
    start: usize,
    end: usize,
    name_checked: bool,
    duplicate: bool,
    lexical: Option<LexicalRef>,
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
    fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        for text in [self.name, self.public, self.system].into_iter().flatten() {
            text.release(c)?;
        }
        Ok(())
    }
}
impl DraftTag {
    fn release(mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
) -> Result<(), HtmlParseStop> {
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
    Exhausted,
}
pub(crate) struct Tokenizer<'source> {
    owner: ControlIdentity,
    cursor: InputCursor<'source>,
    state: HtmlTokenizerState,
    current: HtmlSourceRange,
    current_kind: HtmlParseOriginKind,
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
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenizerHandoffError {
    Stop(HtmlParseStop),
    ForeignSource,
    NotData,
    PendingEvents,
    ActiveDraft,
}
impl From<HtmlParseStop> for TokenizerHandoffError {
    fn from(stop: HtmlParseStop) -> Self {
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        if self.current.source() != source.identity() {
            return Err(TokenizerHandoffError::ForeignSource);
        }
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.state != HtmlTokenizerState::Data {
            return Err(TokenizerHandoffError::NotData);
        }
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.pending_len != 0 {
            return Err(TokenizerHandoffError::PendingEvents);
        }
        c.charge(HtmlParseWorkCategory::RecordVisit, 5)?;
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
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            size_of::<HtmlSourceRange>() as u64,
        )?;
        c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(self.cursor.next_anchor(c)?)
    }

    pub(crate) fn new(
        source: &'source HtmlSource,
        state: HtmlTokenizerState,
        last_start: Option<&DocumentString>,
        c: &mut ParseControl,
    ) -> Result<Self, HtmlParseStop> {
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
            current_kind: HtmlParseOriginKind::Literal,
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
        })
    }
    /// The tree owner supplies the actual adjusted-current-node namespace fact
    /// at each pull. Only markup declaration dispatch consumes this fact.
    pub(crate) fn next(
        &mut self,
        c: &mut ParseControl,
        cdata_allowed: bool,
        evidence: &mut ParseEvidence,
    ) -> Result<TokenizerPull, HtmlParseStop> {
        c.require_identity(&self.owner)?;
        while self.pending_len == 0 && !self.done {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            self.step(c, cdata_allowed, evidence)?;
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
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
        };
        let Some(front) = live.first_mut() else {
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
        };
        let first = front.take();
        // The emptied front moves behind the remaining events, which each
        // advance one slot; the vacated slot is then no longer live.
        live.rotate_left(1);
        self.pending_len = remaining;
        match first {
            Some(event) => Ok(TokenizerPull::Event(event)),
            None => Err(c.fail_invariant(HtmlParseInvariant::TokenizerState)),
        }
    }
    /// Applied only between pulls by the concrete tree algorithm.
    pub(crate) fn set_state(
        &mut self,
        state: HtmlTokenizerState,
        c: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        c.require_identity(&self.owner)?;
        admit_record_write(size_of::<HtmlTokenizerState>(), 1, c)?;
        self.state = state;
        Ok(())
    }
    pub(crate) fn release(mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
    ) -> Result<HtmlSourceRange, HtmlParseStop> {
        self.cursor.checked_range(start, end, c)
    }
    fn read(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<Option<u32>, HtmlParseStop> {
        let read = self.cursor.consume(c)?;
        match read {
            InputRead::Character(ch) => {
                if ch.source() != &self.current {
                    admit_record_write(size_of::<HtmlParseOriginKind>(), 1, c)?;
                    self.current_kind = HtmlParseOriginKind::Literal;
                }
                admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
                self.current = ch.source().clone();
                if let Some(record) = ch.preprocessing() {
                    if let super::input::PreprocessingRecord::Diagnostic(code) = record {
                        self.error_at(
                            Self::input_stream_code(code),
                            HtmlTokenizerDiagnosticPhase::InputStream,
                            c,
                            evidence,
                        )?;
                    } else {
                        admit_record_write(size_of::<HtmlParseOriginKind>(), 1, c)?;
                        self.current_kind = HtmlParseOriginKind::Newline;
                        // The record is retained only as evidence of the
                        // preprocessing, which a parse without the graph or
                        // the diagnostics does not keep.
                        if !evidence.records_graph() {
                            return Ok(Some(ch.code_point().value()));
                        }
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
    fn reconsume(
        &mut self,
        state: HtmlTokenizerState,
        c: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        self.cursor.reconsume(c)?;
        self.set_state(state, c)
    }
    fn queue(&mut self, event: Event, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        let Some(slot) = self.pending.get_mut(self.pending_len) else {
            event.release(c)?;
            return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
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
    ) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.current = source.clone();
        Ok(())
    }
    fn remember_slash(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.slash = self.current.clone();
        Ok(())
    }
    fn remember_current_syntax(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.pending_syntax = self.current.clone();
        Ok(())
    }
    fn remember_syntax(
        &mut self,
        source: HtmlSourceRange,
        c: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<HtmlSourceRange>(), 1, c)?;
        self.pending_syntax = source;
        Ok(())
    }
    fn error(
        &mut self,
        code: HtmlTokenizerDiagnostic,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        self.error_at(code, HtmlTokenizerDiagnosticPhase::Tokenizer, c, evidence)
    }
    fn input_stream_code(diagnostic: super::input::InputDiagnostic) -> HtmlTokenizerDiagnostic {
        match diagnostic {
            super::input::InputDiagnostic::Surrogate => {
                HtmlTokenizerDiagnostic::SurrogateInInputStream
            }
            super::input::InputDiagnostic::Noncharacter => {
                HtmlTokenizerDiagnostic::NoncharacterInInputStream
            }
            super::input::InputDiagnostic::ControlCharacter => {
                HtmlTokenizerDiagnostic::ControlCharacterInInputStream
            }
        }
    }
    fn error_at(
        &mut self,
        code: HtmlTokenizerDiagnostic,
        phase: HtmlTokenizerDiagnosticPhase,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        if !evidence.records_diagnostics() {
            return Ok(());
        }
        let source = self.current.clone();
        self.error_at_source(code, phase, &source, c, evidence)
    }
    /// Reports at an explicit input frontier: the character a lookahead
    /// examined is ahead of the last consumed one.
    fn error_at_source(
        &mut self,
        code: HtmlTokenizerDiagnostic,
        phase: HtmlTokenizerDiagnosticPhase,
        source: &HtmlSourceRange,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        if !evidence.records_diagnostics() {
            return Ok(());
        }
        let admission = evidence.append(
            c,
            HtmlParseDiagnosticKind::Tokenizer(code),
            HtmlParseDiagnosticContext::Tokenizer {
                phase,
                state: self.state,
            },
            HtmlParseDiagnosticRule {
                anchor: code.rule_anchor(),
                branch: code.name(),
            },
            crate::construction::evidence::DiagnosticLocationInput {
                source,
                role: crate::construction::evidence::HtmlParseDiagnosticLocationRole::InputFrontier,
                token: None,
            },
        )?;
        // A dropped diagnostic names no record, so the tree builder has none to
        // validate and no event is queued.
        match admission {
            DiagnosticAdmission::Retained(id) => self.queue(Event::Diagnostic(id), c),
            DiagnosticAdmission::Dropped | DiagnosticAdmission::Unselected => Ok(()),
        }
    }

    fn emit(
        &mut self,
        kind: TokenKind,
        start: usize,
        end: usize,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
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
        kind: HtmlParseOriginKind,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
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
    ) -> Result<(), HtmlParseStop> {
        self.emit_current_point(point, self.current_kind, c, evidence)
    }
    fn emit_current_point(
        &mut self,
        point: u32,
        kind: HtmlParseOriginKind,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
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
    ) -> Result<(), HtmlParseStop> {
        self.error(
            HtmlTokenizerDiagnostic::UnexpectedNullCharacter,
            c,
            evidence,
        )?;
        self.emit_current_point(0xFFFD, HtmlParseOriginKind::NullReplacement, c, evidence)
    }
    fn clear_temporary(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        self.temporary.value.clear(c)?;
        self.temporary.last_end = None;
        self.temporary.origins.clear(c)
    }
    fn temp_matches(&self, expected: &[u8], c: &mut ParseControl) -> Result<bool, HtmlParseStop> {
        ascii_eq(self.temporary.value.units(), expected, false, c)
    }
    fn appropriate_end(&self, c: &mut ParseControl) -> Result<bool, HtmlParseStop> {
        let tag = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        let a = tag.name.value.units();
        let b = self.last_start.units();
        if a.len() != b.len() {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            return Ok(false);
        }
        for (a, b) in a.iter().zip(b) {
            c.charge(HtmlParseWorkCategory::InspectByte, 4)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn create_tag(&mut self, kind: TagKind, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
    ) -> Result<(), HtmlParseStop> {
        let kind = if point == 0 {
            self.error(
                HtmlTokenizerDiagnostic::UnexpectedNullCharacter,
                c,
                evidence,
            )?;
            HtmlParseOriginKind::NullReplacement
        } else if lower(point) != point {
            HtmlParseOriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
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
    ) -> Result<(), HtmlParseStop> {
        self.finish_attribute(c, evidence)?;
        let lexical = evidence.lexical_begin(
            c,
            &self.current,
            HtmlParseLexicalKind::Attribute,
            self.lexical.as_ref(),
        )?;
        admit_record_write(size_of::<Option<DraftAttribute>>(), 1, c)?;
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
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
    ) -> Result<(), HtmlParseStop> {
        self.attribute_end(self.current.end(), c)?;
        let kind = if point == 0 {
            self.error(
                HtmlTokenizerDiagnostic::UnexpectedNullCharacter,
                c,
                evidence,
            )?;
            HtmlParseOriginKind::NullReplacement
        } else if lower(point) != point {
            HtmlParseOriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        let tag = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        tag.attribute
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
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
    ) -> Result<(), HtmlParseStop> {
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        if let Some(a) = &mut t.attribute
            && !a.name_checked
        {
            let duplicate = t.names.insert(
                a.name.value.units(),
                t.attributes.as_slice(),
                t.attributes.len(),
                c,
            )?;
            admit_record_write(const { 2 * size_of::<bool>() }, 2, c)?;
            a.duplicate = duplicate;
            a.name_checked = true;
            if a.duplicate {
                admit_record_write(size_of::<bool>(), 1, c)?;
                t.had_duplicates = true;
                self.error(HtmlTokenizerDiagnostic::DuplicateAttribute, c, evidence)?;
            }
        }
        Ok(())
    }
    fn attribute_value(
        &mut self,
        point: u32,
        kind: HtmlParseOriginKind,
        source: &HtmlSourceRange,
        c: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        self.attribute_end(source.end(), c)?;
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        t.attribute
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
            .value
            .push(point, source, kind, c)
    }
    fn finish_attribute(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        self.check_attribute_name(c, evidence)?;
        let t = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        let Some(a) = &t.attribute else {
            return Ok(());
        };
        // Range validation may reject work admission. Retain the whole draft
        // in its owner until that checked provider call and removal admission
        // have succeeded, so terminal release can still dispose both buffers.
        let source = self.range(a.start, a.end, c)?;
        if let Some(occurrence) = &a.lexical {
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                HtmlParseLexicalRole::Name,
                a.name.origins.as_slice(),
            )?;
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                HtmlParseLexicalRole::Value,
                a.value.origins.as_slice(),
            )?;
            if a.value.value.len() == 0 {
                let anchor = self.range(a.end, a.end, c)?;
                evidence.lexical_part(
                    c,
                    occurrence,
                    HtmlParseLexicalRole::Value,
                    &anchor,
                    0..0,
                    None,
                )?;
            }
            evidence.lexical_finish(
                c,
                occurrence,
                &source,
                HtmlParseLexicalKind::Attribute,
                if a.duplicate {
                    HtmlParseLexicalDisposition::DuplicateAttribute
                } else if t.kind == TagKind::End {
                    HtmlParseLexicalDisposition::EndTagAttribute
                } else {
                    HtmlParseLexicalDisposition::StoredAttribute
                },
            )?;
        }
        if !a.duplicate {
            // Reserve while both draft buffers remain in their original owner.
            // A denied physical layout must not consume their cleanup tickets.
            self.tag
                .as_mut()
                .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
                .attributes
                .reserve_geometric(c, 1)?;
        }
        admit_record_write(size_of::<Option<DraftAttribute>>(), 1, c)?;
        let t = self
            .tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        if let Some(a) = t.attribute.take() {
            if a.duplicate {
                a.name.release(c)?;
                a.value.release(c)?;
            } else {
                let (name, value) = freeze_pair(a.name, a.value, c)?;
                if let Err((stop, a)) = t.attributes.try_push_recover(
                    c,
                    Attribute {
                        lexical: a.lexical,
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
    fn attribute_end(&mut self, end: usize, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
    ) -> Result<(), HtmlParseStop> {
        self.finish_attribute(c, evidence)?;
        if self.tag.as_ref().is_some_and(|t| t.kind == TagKind::End) {
            if self.tag.as_ref().is_some_and(|t| t.attributes.len() != 0) {
                self.error(HtmlTokenizerDiagnostic::EndTagWithAttributes, c, evidence)?;
            }
            if self.tag.as_ref().is_some_and(|t| t.self_closing) {
                self.error(
                    HtmlTokenizerDiagnostic::EndTagWithTrailingSolidus,
                    c,
                    evidence,
                )?;
            }
        }
        let t = self
            .tag
            .as_ref()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        if t.kind == TagKind::Start {
            self.last_start.clear(c)?;
            self.last_start.extend_units(t.name.value.units(), c)?;
        }
        self.set_state(HtmlTokenizerState::Data, c)?;
        admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
        let mut t = self
            .tag
            .take()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
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
    fn create_comment(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        self.comment = Some(DraftText::new(c)?);
        Ok(())
    }
    fn comment_point(
        &mut self,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        let kind = if point == 0 {
            self.error(
                HtmlTokenizerDiagnostic::UnexpectedNullCharacter,
                c,
                evidence,
            )?;
            HtmlParseOriginKind::NullReplacement
        } else {
            self.current_kind
        };
        self.comment
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
            .push(
                if point == 0 { 0xFFFD } else { point },
                &self.current,
                kind,
                c,
            )
    }
    fn comment_syntax(&mut self, syntax: &[u8], c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        let source = self.range(self.pending_syntax.start(), self.current.start(), c)?;
        for &p in syntax {
            c.charge(HtmlParseWorkCategory::InspectByte, 1)?;
            self.comment
                .as_mut()
                .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
                .push(
                    u32::from(p),
                    &source,
                    HtmlParseOriginKind::RecoverySyntax,
                    c,
                )?;
        }
        Ok(())
    }
    fn emit_comment(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        self.set_state(HtmlTokenizerState::Data, c)?;
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        let text = self
            .comment
            .take()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
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
    ) -> Result<(), HtmlParseStop> {
        let kind = if point == 0 {
            self.error(
                HtmlTokenizerDiagnostic::UnexpectedNullCharacter,
                c,
                evidence,
            )?;
            HtmlParseOriginKind::NullReplacement
        } else if lower(point) != point {
            HtmlParseOriginKind::AsciiLowercase
        } else {
            self.current_kind
        };
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        if d.name.is_none() {
            admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
            d.name = Some(DraftText::new(c)?);
        }
        d.name
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
            .push(
                if point == 0 { 0xFFFD } else { lower(point) },
                &self.current,
                kind,
                c,
            )
    }
    fn create_identifier(
        &mut self,
        kind: HtmlTokenizerIdentifier,
        c: &mut ParseControl,
    ) -> Result<(), HtmlParseStop> {
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
        *match kind {
            HtmlTokenizerIdentifier::Public => &mut d.public,
            HtmlTokenizerIdentifier::System => &mut d.system,
        } = Some(DraftText::new(c)?);
        Ok(())
    }
    fn identifier_point(
        &mut self,
        kind: HtmlTokenizerIdentifier,
        point: u32,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        let origin = if point == 0 {
            self.error(
                HtmlTokenizerDiagnostic::UnexpectedNullCharacter,
                c,
                evidence,
            )?;
            HtmlParseOriginKind::NullReplacement
        } else {
            self.current_kind
        };
        let d = self
            .doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        match kind {
            HtmlTokenizerIdentifier::Public => &mut d.public,
            HtmlTokenizerIdentifier::System => &mut d.system,
        }
        .as_mut()
        .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
        .push(
            if point == 0 { 0xFFFD } else { point },
            &self.current,
            origin,
            c,
        )
    }
    fn quirks(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<bool>(), 1, c)?;
        self.doctype
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
            .force_quirks = true;
        Ok(())
    }
    fn mark_self_closing(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<bool>(), 1, c)?;
        self.tag
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
            .self_closing = true;
        Ok(())
    }
    fn emit_doctype(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        self.set_state(HtmlTokenizerState::Data, c)?;
        admit_record_write(size_of::<Option<DraftDoctype>>(), 1, c)?;
        let d = self
            .doctype
            .take()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
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
    ) -> Result<bool, HtmlParseStop> {
        let comparison = if insensitive {
            AsciiComparison::Insensitive
        } else {
            AsciiComparison::Sensitive
        };
        let lookahead = self.cursor.matches_ascii(bytes, comparison, c)?;
        // Report the next input character's stream error before the mismatch
        // is acted on. The pinned standard sets no relative order between
        // input-stream and tokenizer errors; this is the smallest rule
        // consistent with html5lib test3.test case 80 and test4.test cases
        // 60-62 (see the tokenizer ordering test).
        if let Some(examined) = lookahead.examined {
            self.error_at_source(
                Self::input_stream_code(examined.diagnostic),
                HtmlTokenizerDiagnosticPhase::InputStream,
                &examined.source,
                c,
                evidence,
            )?;
        }
        if !lookahead.result {
            return Ok(false);
        }
        for _ in bytes {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            self.read(c, evidence)?;
            if let Some(occurrence) = self.lexical.as_ref() {
                evidence.lexical_part(
                    c,
                    occurrence,
                    HtmlParseLexicalRole::Syntax,
                    &self.current,
                    0..0,
                    None,
                )?;
            }
        }
        Ok(true)
    }
    fn raw_fallback(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        if let Some(tag) = self.tag.as_ref() {
            if let Some(occurrence) = self.lexical.as_ref() {
                Self::retain_lexical_text(
                    evidence,
                    c,
                    occurrence,
                    HtmlParseLexicalRole::Name,
                    tag.name.origins.as_slice(),
                )?;
            }
            admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
            if let Some(tag) = self.tag.take() {
                tag.release(c)?;
            }
        }
        let text = DraftText::build(c, |text, c| {
            text.push(0x3C, &self.less_than, HtmlParseOriginKind::Literal, c)?;
            text.push(0x2F, &self.slash, HtmlParseOriginKind::Literal, c)?;
            self.temporary.replay_into(text, &self.current, c)
        })?;
        let end = self.temporary.last_end().unwrap_or(self.slash.end());
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
    ) -> Result<(), HtmlParseStop> {
        if let Some(occurrence) = self.lexical.as_ref() {
            Self::retain_lexical_text(
                evidence,
                c,
                occurrence,
                HtmlParseLexicalRole::Target,
                self.temporary.origins.as_slice(),
            )?;
        }
        self.create_comment(c)?;
        self.comment
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?
            .push(0x3F, &self.slash, HtmlParseOriginKind::Literal, c)?;
        let comment = self
            .comment
            .as_mut()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        self.temporary.replay_into(comment, &self.current, c)?;
        self.clear_temporary(c)
    }
    fn emit_pi(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        self.set_state(HtmlTokenizerState::Data, c)?;
        // Admit both removals before taking either nested owner.
        admit_record_write(const { 2 * size_of::<Option<DraftText>>() }, 2, c)?;
        let target = self
            .pi_target
            .take()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
        let data = self
            .pi_data
            .take()
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TokenizerState))?;
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
    fn discard_pi(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
    fn create_doctype(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        admit_record_write(size_of::<Option<DraftDoctype>>(), 1, c)?;
        self.doctype = Some(DraftDoctype::empty());
        Ok(())
    }
    fn finish_pi_target(&mut self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
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
) -> Result<bool, HtmlParseStop> {
    c.charge(HtmlParseWorkCategory::StateStep, 1)?;
    if units.len() != bytes.len() {
        return Ok(false);
    }
    for (unit, byte) in units.iter().zip(bytes) {
        c.charge_unit_literal_comparison()?;
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
