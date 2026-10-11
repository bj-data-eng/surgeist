// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// State dispatch adapted from html5ever/src/tokenizer/mod.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf. Modern PI and CDATA rules
// reconciled with HTML 877d6146487f15b15bc8dd0e3c625063a37b456a §13.2.5.

use super::*;
use states::{
    Escape::{DoubleEscaped, Escaped},
    Identifier::{Public, System},
    Quote::{Double, Single, Unquoted},
    Raw::{Rawtext, Rcdata, Script, ScriptEscaped},
    State::*,
};
use token::DiagnosticCode::*;

impl Tokenizer<'_> {
    fn emit_syntax(
        &mut self,
        bytes: &[u8],
        source: &HtmlSourceRange,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let text = DraftText::build(c, |text, c| {
            for &byte in bytes {
                c.charge(WorkCategory::InspectByte, 1)?;
                text.push(u32::from(byte), source, OriginKind::RecoverySyntax, c)?;
            }
            Ok(())
        })?;
        self.emit(TokenKind::Text(text), source.start(), source.end(), c)
    }
    fn mark_less_than(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        admit_record_write(size_of::<usize>() + size_of::<HtmlSourceRange>(), 2, c)?;
        self.start = self.current.start();
        self.less_than = self.current.clone();
        Ok(())
    }
    pub(super) fn step(
        &mut self,
        c: &mut ParseControl,
        cdata_allowed: bool,
    ) -> Result<(), ParseStop> {
        // These lookahead states examine the next input without consuming a
        // mismatching character. The indexed cursor charges each repeated read.
        if self.state == MarkupDeclaration {
            if self.consume_ascii(b"--", false, c)? {
                self.create_comment(c)?;
                self.set_state(CommentStart, c)?;
            } else if self.consume_ascii(b"DOCTYPE", true, c)? {
                self.set_state(Doctype, c)?;
            } else if self.consume_ascii(b"[CDATA[", false, c)? {
                if cdata_allowed {
                    self.set_state(Cdata, c)?;
                } else {
                    self.error(CdataInHtmlContent, c)?;
                    self.create_comment(c)?;
                    let source = self.range(self.less_than.end() + 1, self.current.end(), c)?;
                    for &p in b"[CDATA[" {
                        self.comment
                            .as_mut()
                            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                            .push(u32::from(p), &source, OriginKind::RecoverySyntax, c)?;
                    }
                    self.set_state(BogusComment, c)?;
                }
            } else {
                // No input was consumed here: diagnostic covers the opening.
                let source = self.range(self.start, self.cursor.position(c)?, c)?;
                self.set_current(&source, c)?;
                self.error(IncorrectlyOpenedComment, c)?;
                self.create_comment(c)?;
                self.set_state(BogusComment, c)?;
            }
            return Ok(());
        }
        if self.state == AfterDoctypeName {
            if self.consume_ascii(b"PUBLIC", true, c)? {
                self.set_state(AfterDoctypeKeyword(Public), c)?;
                return Ok(());
            }
            if self.consume_ascii(b"SYSTEM", true, c)? {
                self.set_state(AfterDoctypeKeyword(System), c)?;
                return Ok(());
            }
        }
        let Some(p) = self.read(c)? else {
            return self.eof(c);
        };
        match self.state {
            Data => match p {
                0x26 => self.character_reference(false, c)?,
                0x3C => {
                    self.mark_less_than(c)?;
                    self.set_state(TagOpen, c)?;
                }
                0 => {
                    self.error(UnexpectedNullCharacter, c)?;
                    self.character(0, c)?;
                }
                _ => self.character(p, c)?,
            },
            Plaintext => {
                if p == 0 {
                    self.replacement(c)?;
                } else {
                    self.character(p, c)?;
                }
            }
            Raw(kind) => match p {
                0x26 if kind == Rcdata => self.character_reference(false, c)?,
                0x3C => {
                    self.mark_less_than(c)?;
                    if kind == ScriptEscaped(DoubleEscaped) {
                        self.character(p, c)?;
                    }
                    self.set_state(RawLessThan(kind), c)?;
                }
                0x2D if matches!(kind, ScriptEscaped(_)) => {
                    self.character(p, c)?;
                    if let ScriptEscaped(e) = kind {
                        self.set_state(ScriptDash(e), c)?;
                    }
                }
                0 => self.replacement(c)?,
                _ => self.character(p, c)?,
            },
            TagOpen => match p {
                0x21 => self.set_state(MarkupDeclaration, c)?,
                0x2F => {
                    self.remember_slash(c)?;
                    self.set_state(EndTagOpen, c)?;
                }
                0x3F => {
                    self.remember_slash(c)?;
                    self.clear_temporary(c)?;
                    self.set_state(PiOpen, c)?;
                }
                _ if alpha(p) => {
                    self.create_tag(TagKind::Start, c)?;
                    self.tag_name(p, c)?;
                    self.set_state(TagName, c)?;
                }
                _ => {
                    self.error(InvalidFirstCharacterOfTagName, c)?;
                    self.emit_point(0x3C, &self.less_than.clone(), OriginKind::Literal, c)?;
                    self.reconsume(Data, c)?;
                }
            },
            EndTagOpen => match p {
                _ if alpha(p) => {
                    self.create_tag(TagKind::End, c)?;
                    self.tag_name(p, c)?;
                    self.set_state(TagName, c)?;
                }
                0x3E => {
                    self.error(MissingEndTagName, c)?;
                    self.set_state(Data, c)?;
                }
                _ => {
                    self.error(InvalidFirstCharacterOfTagName, c)?;
                    self.create_comment(c)?;
                    self.reconsume(BogusComment, c)?;
                }
            },
            TagName => match p {
                _ if space(p) => self.set_state(BeforeAttributeName, c)?,
                0x2F => self.set_state(SelfClosing, c)?,
                0x3E => self.emit_tag(c)?,
                _ => self.tag_name(p, c)?,
            },
            RawLessThan(ScriptEscaped(DoubleEscaped)) => {
                if p == 0x2F {
                    self.clear_temporary(c)?;
                    self.character(p, c)?;
                    self.set_state(ScriptDoubleEscapeEnd, c)?;
                } else {
                    self.reconsume(Raw(ScriptEscaped(DoubleEscaped)), c)?;
                }
            }
            RawLessThan(ScriptEscaped(Escaped)) => match p {
                0x2F => {
                    self.remember_slash(c)?;
                    self.clear_temporary(c)?;
                    self.set_state(RawEndOpen(ScriptEscaped(Escaped)), c)?;
                }
                _ if alpha(p) => {
                    self.clear_temporary(c)?;
                    self.emit_point(0x3C, &self.less_than.clone(), OriginKind::Literal, c)?;
                    self.reconsume(ScriptDoubleEscapeStart, c)?;
                }
                _ => {
                    self.emit_point(0x3C, &self.less_than.clone(), OriginKind::Literal, c)?;
                    self.reconsume(Raw(ScriptEscaped(Escaped)), c)?;
                }
            },
            RawLessThan(kind) => match p {
                0x2F => {
                    self.remember_slash(c)?;
                    self.clear_temporary(c)?;
                    self.set_state(RawEndOpen(kind), c)?;
                }
                0x21 if kind == Script => {
                    let range = self.range(self.less_than.start(), self.current.end(), c)?;
                    self.emit_syntax(b"<!", &range, c)?;
                    self.set_state(ScriptEscapeStart, c)?;
                }
                _ => {
                    self.emit_point(0x3C, &self.less_than.clone(), OriginKind::Literal, c)?;
                    self.reconsume(Raw(kind), c)?;
                }
            },
            RawEndOpen(kind) => {
                if alpha(p) {
                    self.create_tag(TagKind::End, c)?;
                    self.tag_name(p, c)?;
                    self.temporary
                        .push(p, &self.current, self.current_kind, c)?;
                    self.set_state(RawEndName(kind), c)?;
                } else {
                    self.raw_fallback(c)?;
                    self.reconsume(Raw(kind), c)?;
                }
            }
            RawEndName(kind) => {
                if self.appropriate_end(c)? && (space(p) || matches!(p, 0x2F | 0x3E)) {
                    self.clear_temporary(c)?;
                    if space(p) {
                        self.set_state(BeforeAttributeName, c)?;
                    } else if p == 0x2F {
                        self.set_state(SelfClosing, c)?;
                    } else {
                        self.emit_tag(c)?;
                    }
                } else if alpha(p) {
                    self.tag_name(p, c)?;
                    self.temporary
                        .push(p, &self.current, self.current_kind, c)?;
                } else {
                    self.raw_fallback(c)?;
                    self.reconsume(Raw(kind), c)?;
                }
            }
            ScriptEscapeStart => {
                if p == 0x2D {
                    self.character(p, c)?;
                    self.set_state(ScriptEscapeStartDash, c)?;
                } else {
                    self.reconsume(Raw(Script), c)?;
                }
            }
            ScriptEscapeStartDash => {
                if p == 0x2D {
                    self.character(p, c)?;
                    self.set_state(ScriptDashDash(Escaped), c)?;
                } else {
                    self.reconsume(Raw(Script), c)?;
                }
            }
            ScriptDoubleEscapeStart | ScriptDoubleEscapeEnd => {
                let ending = self.state == ScriptDoubleEscapeEnd;
                if space(p) || matches!(p, 0x2F | 0x3E) {
                    let script = self.temp_matches(b"script", c)?;
                    let escape = if script == ending {
                        Escaped
                    } else {
                        DoubleEscaped
                    };
                    self.character(p, c)?;
                    self.set_state(Raw(ScriptEscaped(escape)), c)?;
                } else if alpha(p) {
                    self.temporary.push(
                        lower(p),
                        &self.current,
                        if lower(p) != p {
                            OriginKind::AsciiLowercase
                        } else {
                            self.current_kind
                        },
                        c,
                    )?;
                    self.character(p, c)?;
                } else {
                    self.reconsume(
                        Raw(ScriptEscaped(if ending { DoubleEscaped } else { Escaped })),
                        c,
                    )?;
                }
            }
            ScriptDash(e) | ScriptDashDash(e) => {
                let double_dash = matches!(self.state, ScriptDashDash(_));
                match p {
                    0x2D => {
                        self.character(p, c)?;
                        self.set_state(ScriptDashDash(e), c)?;
                    }
                    0x3C => {
                        self.mark_less_than(c)?;
                        if e == DoubleEscaped {
                            self.character(p, c)?;
                        }
                        self.set_state(RawLessThan(ScriptEscaped(e)), c)?;
                    }
                    0x3E if double_dash => {
                        self.character(p, c)?;
                        self.set_state(Raw(Script), c)?;
                    }
                    0 => {
                        self.replacement(c)?;
                        self.set_state(Raw(ScriptEscaped(e)), c)?;
                    }
                    _ => {
                        self.character(p, c)?;
                        self.set_state(Raw(ScriptEscaped(e)), c)?;
                    }
                }
            }
            BeforeAttributeName => match p {
                _ if space(p) => {}
                0x2F => self.set_state(SelfClosing, c)?,
                0x3E => self.emit_tag(c)?,
                0x3D => {
                    self.error(UnexpectedEqualsSignBeforeAttributeName, c)?;
                    self.create_attribute(c)?;
                    self.attribute_name(p, c)?;
                    self.set_state(AttributeName, c)?;
                }
                _ => {
                    self.create_attribute(c)?;
                    self.reconsume(AttributeName, c)?;
                }
            },
            AttributeName => match p {
                _ if space(p) => {
                    self.check_attribute_name(c)?;
                    self.set_state(AfterAttributeName, c)?;
                }
                0x2F => {
                    self.check_attribute_name(c)?;
                    self.set_state(SelfClosing, c)?;
                }
                0x3E => {
                    self.check_attribute_name(c)?;
                    self.emit_tag(c)?;
                }
                0x3D => {
                    self.check_attribute_name(c)?;
                    self.attribute_end(self.current.end(), c)?;
                    self.set_state(BeforeAttributeValue, c)?;
                }
                _ => {
                    if matches!(p, 0x22 | 0x27 | 0x3C) {
                        self.error(UnexpectedCharacterInAttributeName, c)?;
                    }
                    self.attribute_name(p, c)?;
                }
            },
            AfterAttributeName => match p {
                _ if space(p) => {}
                0x2F => self.set_state(SelfClosing, c)?,
                0x3D => {
                    self.attribute_end(self.current.end(), c)?;
                    self.set_state(BeforeAttributeValue, c)?;
                }
                0x3E => self.emit_tag(c)?,
                _ => {
                    self.create_attribute(c)?;
                    self.reconsume(AttributeName, c)?;
                }
            },
            BeforeAttributeValue => match p {
                _ if space(p) => {}
                0x22 => {
                    self.attribute_end(self.current.end(), c)?;
                    self.set_state(AttributeValue(Double), c)?;
                }
                0x27 => {
                    self.attribute_end(self.current.end(), c)?;
                    self.set_state(AttributeValue(Single), c)?;
                }
                0x3E => {
                    self.error(MissingAttributeValue, c)?;
                    self.emit_tag(c)?;
                }
                _ => self.reconsume(AttributeValue(Unquoted), c)?,
            },
            AttributeValue(q) => match p {
                0x22 if q == Double => {
                    self.attribute_end(self.current.end(), c)?;
                    self.set_state(AfterAttributeValue, c)?;
                }
                0x27 if q == Single => {
                    self.attribute_end(self.current.end(), c)?;
                    self.set_state(AfterAttributeValue, c)?;
                }
                _ if q == Unquoted && space(p) => self.set_state(BeforeAttributeName, c)?,
                0x3E if q == Unquoted => self.emit_tag(c)?,
                0x26 => self.character_reference(true, c)?,
                0 => {
                    self.error(UnexpectedNullCharacter, c)?;
                    self.attribute_value(
                        0xFFFD,
                        OriginKind::NullReplacement,
                        &self.current.clone(),
                        c,
                    )?;
                }
                _ => {
                    if q == Unquoted && matches!(p, 0x22 | 0x27 | 0x3C | 0x3D | 0x60) {
                        self.error(UnexpectedCharacterInUnquotedAttributeValue, c)?;
                    }
                    self.attribute_value(p, self.current_kind, &self.current.clone(), c)?;
                }
            },
            AfterAttributeValue => match p {
                _ if space(p) => self.set_state(BeforeAttributeName, c)?,
                0x2F => self.set_state(SelfClosing, c)?,
                0x3E => self.emit_tag(c)?,
                _ => {
                    self.error(MissingWhitespaceBetweenAttributes, c)?;
                    self.reconsume(BeforeAttributeName, c)?;
                }
            },
            SelfClosing => {
                if p == 0x3E {
                    self.mark_self_closing(c)?;
                    self.emit_tag(c)?;
                } else {
                    self.error(UnexpectedSolidusInTag, c)?;
                    self.reconsume(BeforeAttributeName, c)?;
                }
            }
            BogusComment => {
                if p == 0x3E {
                    self.emit_comment(c)?;
                } else {
                    self.comment_point(p, c)?;
                }
            }
            CommentStart => match p {
                0x2D => {
                    self.remember_current_syntax(c)?;
                    self.set_state(CommentStartDash, c)?;
                }
                0x3E => {
                    self.error(AbruptClosingOfEmptyComment, c)?;
                    self.emit_comment(c)?;
                }
                _ => self.reconsume(Comment, c)?,
            },
            CommentStartDash => match p {
                0x2D => self.set_state(CommentEnd, c)?,
                0x3E => {
                    self.error(AbruptClosingOfEmptyComment, c)?;
                    self.emit_comment(c)?;
                }
                _ => {
                    self.comment_syntax(b"-", c)?;
                    self.reconsume(Comment, c)?;
                }
            },
            Comment => match p {
                0x3C => {
                    self.comment_point(p, c)?;
                    self.set_state(CommentLessThan, c)?;
                }
                0x2D => {
                    self.remember_current_syntax(c)?;
                    self.set_state(CommentEndDash, c)?;
                }
                _ => self.comment_point(p, c)?,
            },
            CommentLessThan => match p {
                0x21 => {
                    self.comment_point(p, c)?;
                    self.set_state(CommentLessThanBang, c)?;
                }
                0x3C => self.comment_point(p, c)?,
                _ => self.reconsume(Comment, c)?,
            },
            CommentLessThanBang => {
                if p == 0x2D {
                    self.remember_current_syntax(c)?;
                    self.set_state(CommentLessThanBangDash, c)?;
                } else {
                    self.reconsume(Comment, c)?;
                }
            }
            CommentLessThanBangDash => {
                if p == 0x2D {
                    self.set_state(CommentLessThanBangDashDash, c)?;
                } else {
                    self.reconsume(CommentEndDash, c)?;
                }
            }
            CommentLessThanBangDashDash => {
                if p != 0x3E {
                    self.error(NestedComment, c)?;
                }
                self.reconsume(CommentEnd, c)?;
            }
            CommentEndDash => {
                if p == 0x2D {
                    self.set_state(CommentEnd, c)?;
                } else {
                    self.comment_syntax(b"-", c)?;
                    self.reconsume(Comment, c)?;
                }
            }
            CommentEnd => match p {
                0x3E => self.emit_comment(c)?,
                0x21 => self.set_state(CommentEndBang, c)?,
                0x2D => {
                    self.comment_syntax(b"-", c)?;
                    let source = self.range(
                        self.current.start().saturating_sub(1),
                        self.current.end(),
                        c,
                    )?;
                    self.remember_syntax(source, c)?;
                }
                _ => {
                    self.comment_syntax(b"--", c)?;
                    self.reconsume(Comment, c)?;
                }
            },
            CommentEndBang => match p {
                0x2D => {
                    self.comment_syntax(b"--!", c)?;
                    self.remember_current_syntax(c)?;
                    self.set_state(CommentEndDash, c)?;
                }
                0x3E => {
                    self.error(IncorrectlyClosedComment, c)?;
                    self.emit_comment(c)?;
                }
                _ => {
                    self.comment_syntax(b"--!", c)?;
                    self.reconsume(Comment, c)?;
                }
            },
            Doctype => match p {
                _ if space(p) => self.set_state(BeforeDoctypeName, c)?,
                0x3E => self.reconsume(BeforeDoctypeName, c)?,
                _ => {
                    self.error(MissingWhitespaceBeforeDoctypeName, c)?;
                    self.reconsume(BeforeDoctypeName, c)?;
                }
            },
            BeforeDoctypeName => {
                if !space(p) {
                    self.create_doctype(c)?;
                    if p == 0x3E {
                        self.error(MissingDoctypeName, c)?;
                        self.quirks(c)?;
                        self.emit_doctype(c)?;
                    } else {
                        self.doctype_name(p, c)?;
                        self.set_state(DoctypeName, c)?;
                    }
                }
            }
            DoctypeName => match p {
                _ if space(p) => self.set_state(AfterDoctypeName, c)?,
                0x3E => self.emit_doctype(c)?,
                _ => self.doctype_name(p, c)?,
            },
            AfterDoctypeName => match p {
                _ if space(p) => {}
                0x3E => self.emit_doctype(c)?,
                _ => {
                    self.error(InvalidCharacterSequenceAfterDoctypeName, c)?;
                    self.quirks(c)?;
                    self.reconsume(BogusDoctype, c)?;
                }
            },
            AfterDoctypeKeyword(kind) | BeforeDoctypeIdentifier(kind) => {
                let after_keyword = matches!(self.state, AfterDoctypeKeyword(_));
                if space(p) {
                    self.set_state(BeforeDoctypeIdentifier(kind), c)?;
                } else if matches!(p, 0x22 | 0x27) {
                    if after_keyword {
                        self.error(
                            match kind {
                                Public => MissingWhitespaceAfterDoctypePublicKeyword,
                                System => MissingWhitespaceAfterDoctypeSystemKeyword,
                            },
                            c,
                        )?;
                    }
                    self.create_identifier(kind, c)?;
                    self.set_state(
                        DoctypeIdentifier(kind, if p == 0x22 { Double } else { Single }),
                        c,
                    )?;
                } else {
                    self.error(
                        match (kind, p) {
                            (Public, 0x3E) => MissingDoctypePublicIdentifier,
                            (System, 0x3E) => MissingDoctypeSystemIdentifier,
                            (Public, _) => MissingQuoteBeforeDoctypePublicIdentifier,
                            (System, _) => MissingQuoteBeforeDoctypeSystemIdentifier,
                        },
                        c,
                    )?;
                    self.quirks(c)?;
                    if p == 0x3E {
                        self.emit_doctype(c)?;
                    } else {
                        self.reconsume(BogusDoctype, c)?;
                    }
                }
            }
            DoctypeIdentifier(kind, q) => match p {
                0x22 if q == Double => self.set_state(AfterDoctypeIdentifier(kind), c)?,
                0x27 if q == Single => self.set_state(AfterDoctypeIdentifier(kind), c)?,
                0x3E => {
                    self.error(
                        match kind {
                            Public => AbruptDoctypePublicIdentifier,
                            System => AbruptDoctypeSystemIdentifier,
                        },
                        c,
                    )?;
                    self.quirks(c)?;
                    self.emit_doctype(c)?;
                }
                _ => self.identifier_point(kind, p, c)?,
            },
            AfterDoctypeIdentifier(Public) | BetweenDoctypeIdentifiers => match p {
                _ if space(p) => self.set_state(BetweenDoctypeIdentifiers, c)?,
                0x3E => self.emit_doctype(c)?,
                0x22 | 0x27 => {
                    if self.state == AfterDoctypeIdentifier(Public) {
                        self.error(MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers, c)?;
                    }
                    self.create_identifier(System, c)?;
                    self.set_state(
                        DoctypeIdentifier(System, if p == 0x22 { Double } else { Single }),
                        c,
                    )?;
                }
                _ => {
                    self.error(MissingQuoteBeforeDoctypeSystemIdentifier, c)?;
                    self.quirks(c)?;
                    self.reconsume(BogusDoctype, c)?;
                }
            },
            AfterDoctypeIdentifier(System) => match p {
                _ if space(p) => {}
                0x3E => self.emit_doctype(c)?,
                _ => {
                    self.error(UnexpectedCharacterAfterDoctypeSystemIdentifier, c)?;
                    self.reconsume(BogusDoctype, c)?;
                }
            },
            BogusDoctype => match p {
                0x3E => self.emit_doctype(c)?,
                0 => self.error(UnexpectedNullCharacter, c)?,
                _ => {}
            },
            Cdata => {
                if p == 0x5D {
                    self.remember_current_syntax(c)?;
                    self.set_state(CdataBracket, c)?;
                } else {
                    self.character(p, c)?;
                }
            }
            CdataBracket => {
                if p == 0x5D {
                    let source = self.range(self.pending_syntax.start(), self.current.end(), c)?;
                    self.remember_syntax(source, c)?;
                    self.set_state(CdataEnd, c)?;
                } else {
                    self.emit_point(0x5D, &self.pending_syntax.clone(), OriginKind::Literal, c)?;
                    self.reconsume(Cdata, c)?;
                }
            }
            CdataEnd => match p {
                0x5D => {
                    let first = self.range(
                        self.pending_syntax.start(),
                        self.pending_syntax.start() + 1,
                        c,
                    )?;
                    self.emit_point(0x5D, &first, OriginKind::Literal, c)?;
                    let source = self.range(first.end(), self.current.end(), c)?;
                    self.remember_syntax(source, c)?;
                }
                0x3E => self.set_state(Data, c)?,
                _ => {
                    let source = self.pending_syntax.clone();
                    self.emit_syntax(b"]]", &source, c)?;
                    self.reconsume(Cdata, c)?;
                }
            },
            PiOpen => {
                if alpha(p) || p == 0x5F {
                    self.reconsume(PiTarget, c)?;
                } else {
                    self.error(InvalidFirstCharacterOfProcessingInstructionTarget, c)?;
                    self.pi_comment(c)?;
                    self.reconsume(BogusComment, c)?;
                }
            }
            PiTarget => {
                if space(p) || matches!(p, 0x3F | 0x3E) {
                    if ascii_eq(self.temporary.value.units(), b"xml", true, c)?
                        || ascii_eq(self.temporary.value.units(), b"xml-stylesheet", true, c)?
                    {
                        self.error(DisallowedProcessingInstructionTarget, c)?;
                        self.pi_comment(c)?;
                        self.reconsume(BogusComment, c)?;
                    } else {
                        self.finish_pi_target(c)?;
                        self.reconsume(AfterPiTarget, c)?;
                    }
                } else if alnum(p) || matches!(p, 0x2D | 0x5F) {
                    self.temporary
                        .push(p, &self.current, self.current_kind, c)?;
                } else {
                    self.error(InvalidProcessingInstructionTarget, c)?;
                    self.pi_comment(c)?;
                    self.reconsume(BogusComment, c)?;
                }
            }
            AfterPiTarget => {
                if !space(p) {
                    self.reconsume(PiData, c)?;
                }
            }
            PiData => match p {
                0x3F => {
                    self.remember_current_syntax(c)?;
                    self.set_state(PiQuestionable, c)?;
                }
                0x3E => self.emit_pi(c)?,
                _ => self
                    .pi_data
                    .as_mut()
                    .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                    .push(p, &self.current, self.current_kind, c)?,
            },
            PiQuestionable => {
                if p == 0x3E {
                    self.emit_pi(c)?;
                } else {
                    self.pi_data
                        .as_mut()
                        .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                        .push(0x3F, &self.pending_syntax, OriginKind::Literal, c)?;
                    self.reconsume(PiData, c)?;
                }
            }
            MarkupDeclaration => return Err(c.fail_invariant(Invariant::InputCursorState)),
        }
        Ok(())
    }
    fn eof(&mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        match self.state {
            Data | Plaintext | Raw(Rcdata) | Raw(Rawtext) | Raw(Script) => {
                self.emit(TokenKind::Eof, self.current.start(), self.current.end(), c)?;
                admit_record_write(size_of::<bool>(), 1, c)?;
                self.done = true;
            }
            TagOpen | EndTagOpen => {
                self.error(EofBeforeTagName, c)?;
                if self.state == TagOpen {
                    self.emit_point(0x3C, &self.less_than.clone(), OriginKind::Literal, c)?;
                } else {
                    self.raw_fallback(c)?;
                }
                self.reconsume(Data, c)?;
            }
            AttributeName => {
                self.check_attribute_name(c)?;
                self.reconsume(AfterAttributeName, c)?;
            }
            TagName | BeforeAttributeName | AfterAttributeName | BeforeAttributeValue
            | AttributeValue(_) | AfterAttributeValue | SelfClosing => {
                self.error(EofInTag, c)?;
                if self.tag.is_some() {
                    admit_record_write(size_of::<Option<DraftTag>>(), 1, c)?;
                    if let Some(t) = self.tag.take() {
                        t.release(c)?;
                    }
                }
                self.reconsume(Data, c)?;
            }
            Raw(ScriptEscaped(_)) | ScriptDash(_) | ScriptDashDash(_) => {
                self.error(EofInScriptHtmlCommentLikeText, c)?;
                self.reconsume(Data, c)?;
            }
            RawLessThan(ScriptEscaped(DoubleEscaped)) => {
                self.reconsume(Raw(ScriptEscaped(DoubleEscaped)), c)?
            }
            RawLessThan(kind) => {
                self.emit_point(0x3C, &self.less_than.clone(), OriginKind::Literal, c)?;
                self.reconsume(Raw(kind), c)?;
            }
            RawEndOpen(kind) | RawEndName(kind) => {
                self.raw_fallback(c)?;
                self.reconsume(Raw(kind), c)?;
            }
            ScriptEscapeStart | ScriptEscapeStartDash => self.reconsume(Raw(Script), c)?,
            ScriptDoubleEscapeStart => self.reconsume(Raw(ScriptEscaped(Escaped)), c)?,
            ScriptDoubleEscapeEnd => self.reconsume(Raw(ScriptEscaped(DoubleEscaped)), c)?,
            CommentStart | CommentStartDash | Comment | CommentEndDash | CommentEnd
            | CommentEndBang => {
                self.error(EofInComment, c)?;
                self.emit_comment(c)?;
                self.cursor.reconsume(c)?;
            }
            CommentLessThan | CommentLessThanBang => self.reconsume(Comment, c)?,
            CommentLessThanBangDash => self.reconsume(CommentEndDash, c)?,
            CommentLessThanBangDashDash => self.reconsume(CommentEnd, c)?,
            BogusComment => {
                self.emit_comment(c)?;
                self.cursor.reconsume(c)?;
            }
            Doctype | BeforeDoctypeName => {
                self.error(EofInDoctype, c)?;
                self.create_doctype(c)?;
                self.quirks(c)?;
                self.emit_doctype(c)?;
                self.cursor.reconsume(c)?;
            }
            DoctypeName
            | AfterDoctypeName
            | AfterDoctypeKeyword(_)
            | BeforeDoctypeIdentifier(_)
            | DoctypeIdentifier(_, _)
            | AfterDoctypeIdentifier(_)
            | BetweenDoctypeIdentifiers => {
                self.error(EofInDoctype, c)?;
                self.quirks(c)?;
                self.emit_doctype(c)?;
                self.cursor.reconsume(c)?;
            }
            BogusDoctype => {
                self.emit_doctype(c)?;
                self.cursor.reconsume(c)?;
            }
            Cdata => {
                self.error(EofInCdata, c)?;
                self.reconsume(Data, c)?;
            }
            CdataBracket => {
                self.emit_point(0x5D, &self.pending_syntax.clone(), OriginKind::Literal, c)?;
                self.reconsume(Cdata, c)?;
            }
            CdataEnd => {
                let source = self.pending_syntax.clone();
                self.emit_syntax(b"]]", &source, c)?;
                self.reconsume(Cdata, c)?;
            }
            PiOpen | PiTarget | AfterPiTarget | PiData | PiQuestionable => {
                self.error(EofInProcessingInstruction, c)?;
                self.reconsume(Data, c)?;
            }
            MarkupDeclaration => return Err(c.fail_invariant(Invariant::InputCursorState)),
        }
        Ok(())
    }
}
