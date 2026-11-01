// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tokenizer/char_ref/mod.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf. Concrete cursor checkpoints,
// checked reference queries and source-qualified emissions replace its sink.

#[cfg(test)]
use super::super::input::ProgressiveInputCheckpoint;
use super::super::{
    input::{InputCheckpoint, InputPoll},
    references::{self, NameLookup, ReferenceValue},
};
use super::*;
use token::DiagnosticCode::*;

enum ReferenceMark {
    Complete(InputCheckpoint),
    #[cfg(test)]
    Progressive(ProgressiveInputCheckpoint),
}
#[derive(Clone, Copy)]
enum ReferencePhase {
    Start,
    Named,
    Decide,
    Follow,
    Ambiguous,
    NumericStart,
    NumericDigits,
    NumericEnd,
}
/// Actual grammar-local owners survive an availability edge without replay.
pub(super) struct ReferenceContinuation {
    attribute: bool,
    amp: HtmlSourceRange,
    initial: ReferenceMark,
    query: OwnedVec<u32>,
    longest: Option<(ReferenceValue, bool, ReferenceMark)>,
    replacement: Option<ReferenceValue>,
    semicolon: bool,
    phase: ReferencePhase,
    hex: bool,
    digits: bool,
    value: u32,
    literal: Option<DraftText>,
    end: usize,
}
impl ReferenceContinuation {
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.query.release(c)?;
        if let Some(literal) = self.literal {
            literal.release(c)?;
        }
        Ok(())
    }
    fn phase(&mut self, phase: ReferencePhase, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<ReferencePhase>(), 1, c)?;
        self.phase = phase;
        Ok(())
    }
}
#[cfg(test)]
impl ReferenceContinuation {
    /// Test hook: a literal scan is under way, but its draft owner is absent.
    pub(super) fn force_ambiguous_without_literal(&mut self) {
        self.phase = ReferencePhase::Ambiguous;
        self.attribute = false;
    }
    /// Test hook: a longest match is being applied, but none was recorded.
    pub(super) fn force_follow_without_replacement(&mut self) {
        self.phase = ReferencePhase::Follow;
        self.attribute = false;
        self.semicolon = true;
        self.replacement = None;
    }
}
impl Tokenizer<'_> {
    fn reference_mark(&self, c: &mut ParseControl) -> Result<ReferenceMark, ParseStop> {
        #[cfg(test)]
        if self.cursor.is_progressive() {
            return Ok(ReferenceMark::Progressive(self.cursor.mark_progressive(c)?));
        }
        Ok(ReferenceMark::Complete(self.cursor.mark(c)?))
    }
    fn reference_rewind(
        &mut self,
        mark: &ReferenceMark,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        match mark {
            ReferenceMark::Complete(mark) => self.cursor.rewind(mark, c)?,
            #[cfg(test)]
            ReferenceMark::Progressive(mark) => self.cursor.rewind_progressive(mark, c)?,
        }
        self.sync_proofs(c)
    }
    fn reference_peek(&self, c: &mut ParseControl) -> Result<InputPoll, ParseStop> {
        if self.cursor.is_progressive() {
            self.cursor.poll_peek(c)
        } else {
            self.cursor.peek(c).map(InputPoll::Ready)
        }
    }
    pub(super) fn character_reference(
        &mut self,
        attribute: bool,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        self.save_proof_range(
            ProofRole::Reference,
            self.current.start(),
            self.current.end(),
            c,
        )?;
        let initial = self.reference_mark(c)?;
        let query = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        admit_record_write(
            const { size_of::<ReferenceContinuation>() + size_of::<Option<ReferenceContinuation>>() },
            2,
            c,
        )?;
        self.reference = Some(ReferenceContinuation {
            attribute,
            amp: self.current.clone(),
            initial,
            query,
            longest: None,
            replacement: None,
            semicolon: false,
            phase: ReferencePhase::Start,
            hex: false,
            digits: false,
            value: 0,
            literal: None,
            end: self.current.end(),
        });
        Ok(())
    }
    pub(super) fn resume_reference(
        &mut self,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<StepProgress, ParseStop> {
        admit_record_write(
            const { size_of::<Option<ReferenceContinuation>>() + size_of::<ReferenceContinuation>() },
            2,
            c,
        )?;
        let mut reference = self.reference.take().expect("retained reference");
        let result = self.reference_steps(&mut reference, c, evidence);
        match result {
            Ok(false) => {
                if let Err(stop) = admit_record_write(
                    const {
                        size_of::<Option<ReferenceContinuation>>()
                            + size_of::<ReferenceContinuation>()
                    },
                    2,
                    c,
                ) {
                    reference.release(c)?;
                    return Err(stop);
                }
                self.reference = Some(reference);
                Ok(StepProgress::NeedInput)
            }
            Ok(true) => {
                reference.release(c)?;
                Ok(StepProgress::Advanced)
            }
            Err(stop) => {
                reference.release(c)?;
                Err(stop)
            }
        }
    }
    fn reference_point(
        &mut self,
        point: u32,
        attribute: bool,
        source: &HtmlSourceRange,
        kind: OriginKind,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), ParseStop> {
        if attribute {
            self.attribute_value(point, kind, source, c)
        } else {
            self.emit_point(point, source, kind, c, evidence)
        }
    }
    fn reference_steps(
        &mut self,
        r: &mut ReferenceContinuation,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<bool, ParseStop> {
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            match r.phase {
                ReferencePhase::Start => {
                    let read = match self.reference_peek(c)? {
                        InputPoll::NeedInput => return Ok(false),
                        InputPoll::Ready(read) => read,
                    };
                    match read {
                        InputRead::Character(ch) if ch.code_point().value() == 0x23 => {
                            self.read(c, evidence)?;
                            r.phase(ReferencePhase::NumericStart, c)?;
                        }
                        InputRead::Character(ch) if alnum(ch.code_point().value()) => {
                            r.phase(ReferencePhase::Named, c)?
                        }
                        _ => {
                            self.reference_point(
                                0x26,
                                r.attribute,
                                &r.amp,
                                OriginKind::Literal,
                                c,
                                evidence,
                            )?;
                            return Ok(true);
                        }
                    }
                }
                ReferencePhase::Named => {
                    let read = match self.reference_peek(c)? {
                        InputPoll::NeedInput => return Ok(false),
                        InputPoll::Ready(read) => read,
                    };
                    let InputRead::Character(ch) = read else {
                        r.phase(ReferencePhase::Decide, c)?;
                        continue;
                    };
                    let p = ch.code_point().value();
                    if !(alnum(p) || p == 0x3B) {
                        r.phase(ReferencePhase::Decide, c)?;
                        continue;
                    }
                    self.read(c, evidence)?;
                    r.query.try_push(c, p)?;
                    match references::lookup_name(r.query.as_slice(), c)? {
                        NameLookup::Complete {
                            value,
                            can_continue,
                        } => {
                            let mark = self.reference_mark(c)?;
                            admit_record_write(
                                size_of::<Option<(ReferenceValue, bool, ReferenceMark)>>(),
                                1,
                                c,
                            )?;
                            r.longest = Some((value, p == 0x3B, mark));
                            if !can_continue {
                                r.phase(ReferencePhase::Decide, c)?;
                            }
                        }
                        NameLookup::Prefix => {}
                        NameLookup::Unknown => r.phase(ReferencePhase::Decide, c)?,
                    }
                }
                ReferencePhase::Decide => {
                    if let Some((value, semicolon, mark)) = &r.longest {
                        self.reference_rewind(mark, c)?;
                        admit_record_write(
                            const { size_of::<Option<ReferenceValue>>() + size_of::<bool>() },
                            2,
                            c,
                        )?;
                        r.replacement = Some(*value);
                        r.semicolon = *semicolon;
                        r.phase(ReferencePhase::Follow, c)?;
                    } else {
                        self.reference_rewind(&r.initial, c)?;
                        let mut literal = DraftText::new(c)?;
                        let added = if r.attribute {
                            self.attribute_value(0x26, OriginKind::Literal, &r.amp, c)
                        } else {
                            literal.push(0x26, &r.amp, OriginKind::Literal, c)
                        };
                        if let Err(stop) = added {
                            literal.release(c)?;
                            return Err(stop);
                        }
                        if let Err(stop) = admit_record_write(size_of::<Option<DraftText>>(), 1, c)
                        {
                            literal.release(c)?;
                            return Err(stop);
                        }
                        r.literal = Some(literal);
                        r.phase(ReferencePhase::Ambiguous, c)?;
                    }
                }
                ReferencePhase::Follow => {
                    if r.attribute && !r.semicolon {
                        let read = match self.reference_peek(c)? {
                            InputPoll::NeedInput => return Ok(false),
                            InputPoll::Ready(read) => read,
                        };
                        if matches!(read,InputRead::Character(ref ch) if alnum(ch.code_point().value())||ch.code_point().value()==0x3D)
                        {
                            self.reference_rewind(&r.initial, c)?;
                            self.reference_point(
                                0x26,
                                true,
                                &r.amp,
                                OriginKind::Literal,
                                c,
                                evidence,
                            )?;
                            return Ok(true);
                        }
                    }
                    let source = self.range(r.amp.start(), self.cursor.position(c)?, c)?;
                    self.set_current(&source, c)?;
                    if !r.semicolon {
                        self.error(MissingSemicolonAfterCharacterReference, c, evidence)?;
                    }
                    let value = r.replacement.expect("actual longest match");
                    if r.attribute {
                        match value {
                            ReferenceValue::One(a) => self.reference_point(
                                u32::from(a),
                                true,
                                &source,
                                OriginKind::CharacterReference,
                                c,
                                evidence,
                            )?,
                            ReferenceValue::Two(a, b) => {
                                self.reference_point(
                                    u32::from(a),
                                    true,
                                    &source,
                                    OriginKind::CharacterReference,
                                    c,
                                    evidence,
                                )?;
                                self.reference_point(
                                    u32::from(b),
                                    true,
                                    &source,
                                    OriginKind::CharacterReference,
                                    c,
                                    evidence,
                                )?;
                            }
                        }
                    } else {
                        let text = DraftText::build(c, |text, c| {
                            match value {
                                ReferenceValue::One(a) => text.push(
                                    u32::from(a),
                                    &source,
                                    OriginKind::CharacterReference,
                                    c,
                                )?,
                                ReferenceValue::Two(a, b) => {
                                    text.push(
                                        u32::from(a),
                                        &source,
                                        OriginKind::CharacterReference,
                                        c,
                                    )?;
                                    text.push(
                                        u32::from(b),
                                        &source,
                                        OriginKind::CharacterReference,
                                        c,
                                    )?;
                                }
                            }
                            Ok(())
                        })?;
                        self.emit(
                            TokenKind::Text(text),
                            source.start(),
                            source.end(),
                            c,
                            evidence,
                        )?;
                    }
                    return Ok(true);
                }
                ReferencePhase::Ambiguous => {
                    let read = match self.reference_peek(c)? {
                        InputPoll::NeedInput => return Ok(false),
                        InputPoll::Ready(read) => read,
                    };
                    if let InputRead::Character(ch) = read {
                        let p = ch.code_point().value();
                        if alnum(p) {
                            self.read(c, evidence)?;
                            admit_record_write(size_of::<usize>(), 1, c)?;
                            r.end = self.current.end();
                            if r.attribute {
                                self.attribute_value(
                                    p,
                                    self.current_kind,
                                    &self.current.clone(),
                                    c,
                                )?;
                            } else {
                                r.literal.as_mut().expect("actual literal").push(
                                    p,
                                    &self.current,
                                    self.current_kind,
                                    c,
                                )?;
                            }
                            continue;
                        }
                        if p == 0x3B {
                            self.set_current(ch.source(), c)?;
                            self.error(UnknownNamedCharacterReference, c, evidence)?;
                        }
                    }
                    if !r.attribute {
                        admit_record_write(size_of::<Option<DraftText>>(), 1, c)?;
                        let text = r.literal.take().expect("actual literal").freeze(c)?;
                        self.emit(TokenKind::Text(text), r.amp.start(), r.end, c, evidence)?;
                    }
                    return Ok(true);
                }
                ReferencePhase::NumericStart => {
                    let read = match self.reference_peek(c)? {
                        InputPoll::NeedInput => return Ok(false),
                        InputPoll::Ready(read) => read,
                    };
                    admit_record_write(size_of::<bool>(), 1, c)?;
                    r.hex = matches!(read,InputRead::Character(ref ch) if matches!(ch.code_point().value(),0x58|0x78));
                    if r.hex {
                        self.read(c, evidence)?;
                    }
                    r.phase(ReferencePhase::NumericDigits, c)?;
                }
                ReferencePhase::NumericDigits => {
                    let read = match self.reference_peek(c)? {
                        InputPoll::NeedInput => return Ok(false),
                        InputPoll::Ready(read) => read,
                    };
                    if let InputRead::Character(ch) = read {
                        let p = ch.code_point().value();
                        let digit = match p {
                            0x30..=0x39 => Some(p - 0x30),
                            0x41..=0x46 if r.hex => Some(p - 0x41 + 10),
                            0x61..=0x66 if r.hex => Some(p - 0x61 + 10),
                            _ => None,
                        };
                        if let Some(digit) = digit {
                            self.read(c, evidence)?;
                            admit_record_write(
                                const { size_of::<bool>() + size_of::<u32>() },
                                2,
                                c,
                            )?;
                            r.digits = true;
                            r.value = r
                                .value
                                .checked_mul(if r.hex { 16 } else { 10 })
                                .and_then(|v| v.checked_add(digit))
                                .filter(|&v| v <= 0x110000)
                                .unwrap_or(0x110000);
                            continue;
                        }
                    }
                    r.phase(ReferencePhase::NumericEnd, c)?;
                }
                ReferencePhase::NumericEnd => {
                    if !r.digits {
                        let source = self.range(r.amp.start(), self.cursor.position(c)?, c)?;
                        self.set_current(&source, c)?;
                        self.error(AbsenceOfDigitsInNumericCharacterReference, c, evidence)?;
                        self.reference_rewind(&r.initial, c)?;
                        self.reference_point(
                            0x26,
                            r.attribute,
                            &r.amp,
                            OriginKind::Literal,
                            c,
                            evidence,
                        )?;
                        return Ok(true);
                    }
                    let read = match self.reference_peek(c)? {
                        InputPoll::NeedInput => return Ok(false),
                        InputPoll::Ready(read) => read,
                    };
                    let semicolon = matches!(read,InputRead::Character(ref ch) if ch.code_point().value()==0x3B);
                    if semicolon {
                        self.read(c, evidence)?;
                    }
                    let source = self.range(r.amp.start(), self.cursor.position(c)?, c)?;
                    self.set_current(&source, c)?;
                    if !semicolon {
                        self.error(MissingSemicolonAfterCharacterReference, c, evidence)?;
                    }
                    let mut value = r.value;
                    match value {
                        0 => {
                            self.error(NullCharacterReference, c, evidence)?;
                            value = 0xFFFD;
                        }
                        0x110000.. => {
                            self.error(CharacterReferenceOutsideUnicodeRange, c, evidence)?;
                            value = 0xFFFD;
                        }
                        0xD800..=0xDFFF => {
                            self.error(SurrogateCharacterReference, c, evidence)?;
                            value = 0xFFFD;
                        }
                        _ => {
                            if (0xFDD0..=0xFDEF).contains(&value) || value & 0xFFFF >= 0xFFFE {
                                self.error(NoncharacterCharacterReference, c, evidence)?;
                            }
                            if value == 0x0D
                                || ((value <= 0x1F || (0x7F..=0x9F).contains(&value))
                                    && !matches!(value, 9 | 10 | 12 | 32))
                            {
                                self.error(ControlCharacterReference, c, evidence)?;
                            }
                            if let Some(replacement) = references::c1_replacement(value, c)? {
                                value = u32::from(replacement);
                            }
                        }
                    }
                    self.reference_point(
                        value,
                        r.attribute,
                        &source,
                        OriginKind::CharacterReference,
                        c,
                        evidence,
                    )?;
                    return Ok(true);
                }
            }
        }
    }
}
