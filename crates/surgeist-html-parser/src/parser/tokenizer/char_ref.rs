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

use super::super::{
    input::InputCheckpoint,
    references::{self, MAX_NAME_POINTS, NameLookup, ReferenceValue},
};
use super::*;
use token::HtmlTokenizerDiagnostic::*;

impl Tokenizer<'_> {
    fn reference_point(
        &mut self,
        point: u32,
        attribute: bool,
        source: &HtmlSourceRange,
        kind: HtmlParseOriginKind,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        if attribute {
            self.attribute_value(point, kind, source, c)
        } else {
            self.emit_point(point, source, kind, c, evidence)
        }
    }
    /// Decodes the character reference whose ampersand was just consumed. The
    /// whole source is available, so the decision is made here in one pass:
    /// the name being read lives in fixed storage on this frame, the longest
    /// table name needs no more than the table bounds, and only the text a
    /// failed reference leaves behind is retained, as the token's own text.
    pub(super) fn character_reference(
        &mut self,
        attribute: bool,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        let amp = self.current.clone();
        let initial = self.cursor.mark(c)?;
        c.charge(HtmlParseWorkCategory::StateStep, 1)?;
        match self.cursor.peek(c)? {
            InputRead::Character(ch) if ch.code_point().value() == 0x23 => {
                self.read(c, evidence)?;
                self.numeric_reference(attribute, &amp, &initial, c, evidence)
            }
            InputRead::Character(ch) if alnum(ch.code_point().value()) => {
                self.named_reference(attribute, &amp, &initial, c, evidence)
            }
            _ => self.reference_point(
                0x26,
                attribute,
                &amp,
                HtmlParseOriginKind::Literal,
                c,
                evidence,
            ),
        }
    }
    fn named_reference(
        &mut self,
        attribute: bool,
        amp: &HtmlSourceRange,
        initial: &InputCheckpoint,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        let mut query = [0_u32; MAX_NAME_POINTS];
        let mut length = 0;
        let mut longest: Option<(ReferenceValue, bool, InputCheckpoint)> = None;
        loop {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            let InputRead::Character(ch) = self.cursor.peek(c)? else {
                break;
            };
            let p = ch.code_point().value();
            if !(alnum(p) || p == 0x3B) {
                break;
            }
            self.read(c, evidence)?;
            // A query as long as the longest table name is complete or
            // unknown, never a prefix, so the loop ends before it overflows.
            let Some(slot) = query.get_mut(length) else {
                return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
            };
            *slot = p;
            length = length.saturating_add(1);
            let Some(name) = query.get(..length) else {
                return Err(c.fail_invariant(HtmlParseInvariant::TokenizerState));
            };
            match references::lookup_name(name, c)? {
                NameLookup::Complete {
                    value,
                    can_continue,
                } => {
                    let mark = self.cursor.mark(c)?;
                    longest = Some((value, p == 0x3B, mark));
                    if !can_continue {
                        break;
                    }
                }
                NameLookup::Prefix => {}
                NameLookup::Unknown => break,
            }
        }
        c.charge(HtmlParseWorkCategory::StateStep, 1)?;
        let Some((value, semicolon, mark)) = longest else {
            self.cursor.rewind(initial, c)?;
            return self.unknown_reference(attribute, amp, c, evidence);
        };
        self.cursor.rewind(&mark, c)?;
        c.charge(HtmlParseWorkCategory::StateStep, 1)?;
        if attribute && !semicolon {
            let follows = matches!(
                self.cursor.peek(c)?,
                InputRead::Character(ref ch)
                    if alnum(ch.code_point().value()) || ch.code_point().value() == 0x3D
            );
            if follows {
                self.cursor.rewind(initial, c)?;
                return self.reference_point(
                    0x26,
                    true,
                    amp,
                    HtmlParseOriginKind::Literal,
                    c,
                    evidence,
                );
            }
        }
        let source = self.range(amp.start(), self.cursor.position(c)?, c)?;
        self.set_current(&source, c)?;
        if !semicolon {
            self.error(MissingSemicolonAfterCharacterReference, c, evidence)?;
        }
        if attribute {
            match value {
                ReferenceValue::One(a) => self.reference_point(
                    u32::from(a),
                    true,
                    &source,
                    HtmlParseOriginKind::CharacterReference,
                    c,
                    evidence,
                ),
                ReferenceValue::Two(a, b) => {
                    for point in [a, b] {
                        self.reference_point(
                            u32::from(point),
                            true,
                            &source,
                            HtmlParseOriginKind::CharacterReference,
                            c,
                            evidence,
                        )?;
                    }
                    Ok(())
                }
            }
        } else {
            let text = DraftText::build(c, |text, c| {
                match value {
                    ReferenceValue::One(a) => text.push(
                        u32::from(a),
                        &source,
                        HtmlParseOriginKind::CharacterReference,
                        c,
                    )?,
                    ReferenceValue::Two(a, b) => {
                        for point in [a, b] {
                            text.push(
                                u32::from(point),
                                &source,
                                HtmlParseOriginKind::CharacterReference,
                                c,
                            )?;
                        }
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
            )
        }
    }
    /// An ampersand and alphanumerics that name no table entry stay literal,
    /// and a semicolon that ends them reports the unknown reference.
    fn unknown_reference(
        &mut self,
        attribute: bool,
        amp: &HtmlSourceRange,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        // An attribute value takes its characters directly; a text reference
        // retains them as the text of the token it emits.
        let mut literal = if attribute {
            None
        } else {
            Some(DraftText::new(c)?)
        };
        match self.unknown_reference_steps(amp, &mut literal, c, evidence) {
            Ok(end) => {
                let Some(literal) = literal else {
                    return Ok(());
                };
                let text = literal.freeze(c)?;
                self.emit(TokenKind::Text(text), amp.start(), end, c, evidence)
            }
            Err(stop) => {
                if let Some(literal) = literal {
                    literal.release(c)?;
                }
                Err(stop)
            }
        }
    }
    fn unknown_reference_steps(
        &mut self,
        amp: &HtmlSourceRange,
        literal: &mut Option<DraftText>,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<usize, HtmlParseStop> {
        match literal {
            Some(literal) => literal.push(0x26, amp, HtmlParseOriginKind::Literal, c)?,
            None => self.attribute_value(0x26, HtmlParseOriginKind::Literal, amp, c)?,
        }
        let mut end = amp.end();
        loop {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            let InputRead::Character(ch) = self.cursor.peek(c)? else {
                return Ok(end);
            };
            let p = ch.code_point().value();
            if alnum(p) {
                self.read(c, evidence)?;
                end = self.current.end();
                match literal {
                    Some(literal) => {
                        literal.push(p, &self.current, self.current_kind, c)?;
                    }
                    None => {
                        self.attribute_value(p, self.current_kind, &self.current.clone(), c)?;
                    }
                }
                continue;
            }
            if p == 0x3B {
                self.set_current(ch.source(), c)?;
                self.error(UnknownNamedCharacterReference, c, evidence)?;
            }
            return Ok(end);
        }
    }
    fn numeric_reference(
        &mut self,
        attribute: bool,
        amp: &HtmlSourceRange,
        initial: &InputCheckpoint,
        c: &mut ParseControl,
        evidence: &mut ParseEvidence,
    ) -> Result<(), HtmlParseStop> {
        c.charge(HtmlParseWorkCategory::StateStep, 1)?;
        let hex = matches!(
            self.cursor.peek(c)?,
            InputRead::Character(ref ch) if matches!(ch.code_point().value(), 0x58 | 0x78)
        );
        if hex {
            self.read(c, evidence)?;
        }
        let radix = if hex { 16 } else { 10 };
        let mut value = 0_u32;
        let mut digits = false;
        loop {
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            let InputRead::Character(ch) = self.cursor.peek(c)? else {
                break;
            };
            let Some(digit) = references::numeric_digit(ch.code_point().value(), radix) else {
                break;
            };
            self.read(c, evidence)?;
            digits = true;
            value = references::accumulate_digit(value, radix, digit);
        }
        c.charge(HtmlParseWorkCategory::StateStep, 1)?;
        if !digits {
            let source = self.range(amp.start(), self.cursor.position(c)?, c)?;
            self.set_current(&source, c)?;
            self.error(AbsenceOfDigitsInNumericCharacterReference, c, evidence)?;
            self.cursor.rewind(initial, c)?;
            return self.reference_point(
                0x26,
                attribute,
                amp,
                HtmlParseOriginKind::Literal,
                c,
                evidence,
            );
        }
        let semicolon = matches!(
            self.cursor.peek(c)?,
            InputRead::Character(ref ch) if ch.code_point().value() == 0x3B
        );
        if semicolon {
            self.read(c, evidence)?;
        }
        let source = self.range(amp.start(), self.cursor.position(c)?, c)?;
        self.set_current(&source, c)?;
        if !semicolon {
            self.error(MissingSemicolonAfterCharacterReference, c, evidence)?;
        }
        match value {
            0 => self.error(NullCharacterReference, c, evidence)?,
            0x110000.. => {
                self.error(CharacterReferenceOutsideUnicodeRange, c, evidence)?;
            }
            0xD800..=0xDFFF => self.error(SurrogateCharacterReference, c, evidence)?,
            value => {
                if (0xFDD0..=0xFDEF).contains(&value) || value & 0xFFFF >= 0xFFFE {
                    self.error(NoncharacterCharacterReference, c, evidence)?;
                }
                if value == 0x0D
                    || ((value <= 0x1F || (0x7F..=0x9F).contains(&value))
                        && !matches!(value, 9 | 10 | 12 | 32))
                {
                    self.error(ControlCharacterReference, c, evidence)?;
                }
            }
        }
        let scalar = u32::from(references::numeric_scalar(value, c)?);
        self.reference_point(
            scalar,
            attribute,
            &source,
            HtmlParseOriginKind::CharacterReference,
            c,
            evidence,
        )
    }
}
