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
    references::{self, NameLookup, ReferenceValue},
};
use super::*;
use token::DiagnosticCode::*;

impl Tokenizer<'_> {
    pub(super) fn character_reference(
        &mut self,
        attribute: bool,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let amp = self.current.clone();
        let mark = self.cursor.mark(c)?;
        let mut query = OwnedVec::new(Resource::AuxiliaryEntries)?;
        let result = self.reference_inner(attribute, &amp, &mark, &mut query, c);
        query.release(c)?;
        result
    }
    fn reference_point(
        &mut self,
        point: u32,
        attribute: bool,
        source: &HtmlSourceRange,
        kind: OriginKind,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        if attribute {
            self.attribute_value(point, kind, source, c)
        } else {
            self.emit_point(point, source, kind, c)
        }
    }
    fn reference_inner(
        &mut self,
        attribute: bool,
        amp: &HtmlSourceRange,
        mark: &InputCheckpoint,
        query: &mut OwnedVec<u32>,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        if let InputRead::Character(ch) = self.cursor.peek(c)?
            && ch.code_point().value() == 0x23
        {
            self.read(c)?;
            return self.numeric_reference(attribute, amp, mark, c);
        }
        if !matches!(self.cursor.peek(c)?,InputRead::Character(ref ch) if alnum(ch.code_point().value()))
        {
            return self.reference_point(0x26, attribute, amp, OriginKind::Literal, c);
        }
        let mut longest: Option<(ReferenceValue, bool, InputCheckpoint)> = None;
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            let InputRead::Character(ch) = self.cursor.peek(c)? else {
                break;
            };
            let p = ch.code_point().value();
            if !(alnum(p) || p == 0x3B) {
                break;
            }
            self.read(c)?;
            query.try_push(c, p)?;
            match references::lookup_name(query.as_slice(), c)? {
                NameLookup::Complete {
                    value,
                    can_continue,
                } => {
                    longest = Some((value, p == 0x3B, self.cursor.mark(c)?));
                    if !can_continue {
                        break;
                    }
                }
                NameLookup::Prefix => {}
                NameLookup::Unknown => break,
            }
        }
        if let Some((value, semicolon, end)) = longest {
            self.cursor.rewind(&end, c)?;
            if attribute
                && !semicolon
                && let InputRead::Character(ch) = self.cursor.peek(c)?
                && (alnum(ch.code_point().value()) || ch.code_point().value() == 0x3D)
            {
                self.cursor.rewind(mark, c)?;
                return self.reference_point(0x26, attribute, amp, OriginKind::Literal, c);
            }
            let source = self.range(amp.start(), self.cursor.position(c)?, c)?;
            self.set_current(&source, c)?;
            if !semicolon {
                self.error(MissingSemicolonAfterCharacterReference, c)?;
            }
            // Emit the two scalar replacement as one text record with one source
            // association per scalar, preserving the table's combining sequence.
            if attribute {
                match value {
                    ReferenceValue::One(a) => self.reference_point(
                        u32::from(a),
                        true,
                        &source,
                        OriginKind::CharacterReference,
                        c,
                    )?,
                    ReferenceValue::Two(a, b) => {
                        self.reference_point(
                            u32::from(a),
                            true,
                            &source,
                            OriginKind::CharacterReference,
                            c,
                        )?;
                        self.reference_point(
                            u32::from(b),
                            true,
                            &source,
                            OriginKind::CharacterReference,
                            c,
                        )?;
                    }
                }
            } else {
                let text = DraftText::build(c, |text, c| {
                    match value {
                        ReferenceValue::One(a) => {
                            text.push(u32::from(a), &source, OriginKind::CharacterReference, c)?
                        }
                        ReferenceValue::Two(a, b) => {
                            text.push(u32::from(a), &source, OriginKind::CharacterReference, c)?;
                            text.push(u32::from(b), &source, OriginKind::CharacterReference, c)?;
                        }
                    }
                    Ok(())
                })?;
                self.emit(TokenKind::Text(text), source.start(), source.end(), c)?;
            }
            return Ok(());
        }
        self.cursor.rewind(mark, c)?;
        // Ambiguous ampersand: no table match. Consume alphanumeric spelling
        // literally; its semicolon diagnoses but is reconsumed by the return state.
        let mut literal = DraftText::new(c)?;
        let result = (|| -> Result<usize, ParseStop> {
            if attribute {
                self.attribute_value(0x26, OriginKind::Literal, amp, c)?;
            } else {
                literal.push(0x26, amp, OriginKind::Literal, c)?;
            }
            let mut end = amp.end();
            loop {
                c.charge(WorkCategory::StateStep, 1)?;
                let InputRead::Character(ch) = self.cursor.peek(c)? else {
                    break;
                };
                let p = ch.code_point().value();
                if !alnum(p) {
                    if p == 0x3B {
                        self.set_current(ch.source(), c)?;
                        self.error(UnknownNamedCharacterReference, c)?;
                    }
                    break;
                }
                self.read(c)?;
                end = self.current.end();
                if attribute {
                    self.attribute_value(p, self.current_kind, &self.current.clone(), c)?;
                } else {
                    literal.push(p, &self.current, self.current_kind, c)?;
                }
            }
            Ok(end)
        })();
        let end = match result {
            Ok(end) => end,
            Err(stop) => {
                literal.release(c)?;
                return Err(stop);
            }
        };
        if attribute {
            literal.release(c)?;
        } else {
            self.emit(TokenKind::Text(literal.freeze(c)?), amp.start(), end, c)?;
        }
        Ok(())
    }
    fn numeric_reference(
        &mut self,
        attribute: bool,
        amp: &HtmlSourceRange,
        mark: &InputCheckpoint,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let hex = matches!(self.cursor.peek(c)?,InputRead::Character(ref ch) if matches!(ch.code_point().value(),0x58|0x78));
        if hex {
            self.read(c)?;
        }
        let mut value = 0_u32;
        let mut digits = false;
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            let InputRead::Character(ch) = self.cursor.peek(c)? else {
                break;
            };
            let p = ch.code_point().value();
            let digit = match p {
                0x30..=0x39 => Some(p - 0x30),
                0x41..=0x46 if hex => Some(p - 0x41 + 10),
                0x61..=0x66 if hex => Some(p - 0x61 + 10),
                _ => None,
            };
            let Some(digit) = digit else {
                break;
            };
            self.read(c)?;
            digits = true;
            // Once above Unicode, further digits cannot restore validity. This
            // bounded sentinel avoids arithmetic overflow for arbitrary input.
            value = value
                .checked_mul(if hex { 16 } else { 10 })
                .and_then(|v| v.checked_add(digit))
                .filter(|&v| v <= 0x110000)
                .unwrap_or(0x110000);
        }
        if !digits {
            let source = self.range(amp.start(), self.cursor.position(c)?, c)?;
            self.set_current(&source, c)?;
            self.error(AbsenceOfDigitsInNumericCharacterReference, c)?;
            self.cursor.rewind(mark, c)?;
            return self.reference_point(0x26, attribute, amp, OriginKind::Literal, c);
        }
        let semicolon = matches!(self.cursor.peek(c)?,InputRead::Character(ref ch) if ch.code_point().value()==0x3B);
        if semicolon {
            self.read(c)?;
        }
        let source = self.range(amp.start(), self.cursor.position(c)?, c)?;
        self.set_current(&source, c)?;
        if !semicolon {
            self.error(MissingSemicolonAfterCharacterReference, c)?;
        }
        match value {
            0 => {
                self.error(NullCharacterReference, c)?;
                value = 0xFFFD;
            }
            0x110000.. => {
                self.error(CharacterReferenceOutsideUnicodeRange, c)?;
                value = 0xFFFD;
            }
            0xD800..=0xDFFF => {
                self.error(SurrogateCharacterReference, c)?;
                value = 0xFFFD;
            }
            _ => {
                if (0xFDD0..=0xFDEF).contains(&value) || value & 0xFFFF >= 0xFFFE {
                    self.error(NoncharacterCharacterReference, c)?;
                }
                if value == 0x0D
                    || ((value <= 0x1F || (0x7F..=0x9F).contains(&value))
                        && !matches!(value, 9 | 10 | 12 | 32))
                {
                    self.error(ControlCharacterReference, c)?;
                }
                if let Some(replacement) = references::c1_replacement(value, c)? {
                    value = u32::from(replacement);
                }
            }
        }
        self.reference_point(value, attribute, &source, OriginKind::CharacterReference, c)
    }
}
