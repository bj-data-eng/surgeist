// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Replaces html5ever/src/tokenizer/interface.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf: concrete exact tokens, no sink.

use super::super::{
    buffer::{FrozenUtf16, Utf16Buffer},
    control::{OwnedVec, ParseControl, ParseStop, WorkCategory},
    input::CodePoint,
};
use crate::HtmlParseResource;
use crate::HtmlSourceRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OriginKind {
    Literal,
    Newline,
    AsciiLowercase,
    NullReplacement,
    CharacterReference,
    RecoverySyntax,
}
pub struct TextOrigin {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) source: HtmlSourceRange,
    pub(crate) kind: OriginKind,
}
pub(crate) struct Text {
    pub(crate) value: FrozenUtf16,
    pub(crate) origins: OwnedVec<TextOrigin>,
}
impl Text {
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.value.release(c)?;
        self.origins.release(c)
    }
}
pub(super) struct DraftText {
    pub(super) value: Utf16Buffer,
    pub(super) origins: OwnedVec<TextOrigin>,
}
impl DraftText {
    pub(super) fn new(c: &mut ParseControl) -> Result<Self, ParseStop> {
        Ok(Self {
            value: Utf16Buffer::new(c)?,
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
        })
    }
    pub(super) fn push(
        &mut self,
        point: u32,
        source: &HtmlSourceRange,
        kind: OriginKind,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let start = self.value.len();
        let point = CodePoint::new(point)
            .ok_or_else(|| c.fail_invariant(super::super::control::Invariant::InputCursorState))?;
        self.value.push_code_point(point, c)?;
        c.charge(
            WorkCategory::CopyByte,
            std::mem::size_of::<HtmlSourceRange>() as u64,
        )?;
        self.origins.reserve_geometric(c, 1)?;
        self.origins.try_push(
            c,
            TextOrigin {
                start,
                end: self.value.len(),
                source: source.clone(),
                kind,
            },
        )
    }
    pub(super) fn freeze(self, c: &mut ParseControl) -> Result<Text, ParseStop> {
        let Self { value, origins } = self;
        match value.freeze(c) {
            Ok(value) => Ok(Text { value, origins }),
            Err(stop) => {
                origins.release(c)?;
                Err(stop)
            }
        }
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.value.release(c)?;
        self.origins.release(c)
    }
    /// Keeps temporary ownership until the last fallible write succeeds.
    pub(super) fn build(
        c: &mut ParseControl,
        write: impl FnOnce(&mut Self, &mut ParseControl) -> Result<(), ParseStop>,
    ) -> Result<Text, ParseStop> {
        let mut draft = Self::new(c)?;
        if let Err(stop) = write(&mut draft, c) {
            draft.release(c)?;
            return Err(stop);
        }
        draft.freeze(c)
    }
}
pub(super) fn freeze_pair(
    a: DraftText,
    b: DraftText,
    c: &mut ParseControl,
) -> Result<(Text, Text), ParseStop> {
    let a = match a.freeze(c) {
        Ok(a) => a,
        Err(stop) => {
            b.release(c)?;
            return Err(stop);
        }
    };
    match b.freeze(c) {
        Ok(b) => Ok((a, b)),
        Err(stop) => {
            a.release(c)?;
            Err(stop)
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TagKind {
    Start,
    End,
}
pub(crate) struct Attribute {
    pub(crate) lexical: Option<crate::construction::parser_session::provenance::LexicalRef>,
    pub(crate) name: Text,
    pub(crate) value: Text,
    pub(crate) source: HtmlSourceRange,
}
impl Attribute {
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.name.release(c)?;
        self.value.release(c)
    }
}
pub(crate) struct Tag {
    pub(crate) kind: TagKind,
    pub(crate) name: Text,
    pub(crate) self_closing: bool,
    pub(crate) attributes: OwnedVec<Attribute>,
    pub(crate) had_duplicate_attributes: bool,
}
pub(crate) struct Doctype {
    pub(crate) name: Option<Text>,
    pub(crate) public_id: Option<Text>,
    pub(crate) system_id: Option<Text>,
    pub(crate) force_quirks: bool,
}
pub(crate) enum TokenKind {
    Text(Text),
    Tag(Tag),
    Comment(Text),
    Doctype(Doctype),
    ProcessingInstruction { target: Text, data: Text },
    Eof,
}
pub(crate) struct Token {
    pub(crate) lexical: Option<crate::construction::parser_session::provenance::LexicalRef>,
    pub(crate) kind: TokenKind,
    pub(crate) source: HtmlSourceRange,
}
impl Token {
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.kind.release(c)
    }
}
impl TokenKind {
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        match self {
            TokenKind::Text(t) | TokenKind::Comment(t) => t.release(c),
            TokenKind::Tag(mut t) => {
                t.name.release(c)?;
                super::release_attributes(&mut t.attributes, c)?;
                t.attributes.release(c)
            }
            TokenKind::Doctype(d) => {
                for t in [d.name, d.public_id, d.system_id].into_iter().flatten() {
                    t.release(c)?;
                }
                Ok(())
            }
            TokenKind::ProcessingInstruction { target, data } => {
                target.release(c)?;
                data.release(c)
            }
            TokenKind::Eof => Ok(()),
        }
    }
}
/// Named normative tokenizer conditions, never inferred from inherited prose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    SurrogateInInputStream,
    NoncharacterInInputStream,
    ControlCharacterInInputStream,
    UnexpectedNullCharacter,
    EofBeforeTagName,
    InvalidFirstCharacterOfTagName,
    MissingEndTagName,
    EofInTag,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedCharacterInAttributeName,
    DuplicateAttribute,
    MissingAttributeValue,
    UnexpectedCharacterInUnquotedAttributeValue,
    MissingWhitespaceBetweenAttributes,
    UnexpectedSolidusInTag,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    IncorrectlyOpenedComment,
    CdataInHtmlContent,
    AbruptClosingOfEmptyComment,
    EofInComment,
    NestedComment,
    IncorrectlyClosedComment,
    MissingWhitespaceBeforeDoctypeName,
    MissingDoctypeName,
    EofInDoctype,
    InvalidCharacterSequenceAfterDoctypeName,
    MissingWhitespaceAfterDoctypePublicKeyword,
    MissingWhitespaceAfterDoctypeSystemKeyword,
    MissingDoctypePublicIdentifier,
    MissingDoctypeSystemIdentifier,
    MissingQuoteBeforeDoctypePublicIdentifier,
    MissingQuoteBeforeDoctypeSystemIdentifier,
    AbruptDoctypePublicIdentifier,
    AbruptDoctypeSystemIdentifier,
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
    UnexpectedCharacterAfterDoctypeSystemIdentifier,
    EofInCdata,
    EofInScriptHtmlCommentLikeText,
    EofInProcessingInstruction,
    InvalidFirstCharacterOfProcessingInstructionTarget,
    InvalidProcessingInstructionTarget,
    DisallowedProcessingInstructionTarget,
    MissingSemicolonAfterCharacterReference,
    UnknownNamedCharacterReference,
    AbsenceOfDigitsInNumericCharacterReference,
    NullCharacterReference,
    CharacterReferenceOutsideUnicodeRange,
    SurrogateCharacterReference,
    NoncharacterCharacterReference,
    ControlCharacterReference,
}
impl DiagnosticCode {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::SurrogateInInputStream => "surrogate-in-input-stream",
            Self::NoncharacterInInputStream => "noncharacter-in-input-stream",
            Self::ControlCharacterInInputStream => "control-character-in-input-stream",
            Self::UnexpectedNullCharacter => "unexpected-null-character",
            Self::EofBeforeTagName => "eof-before-tag-name",
            Self::InvalidFirstCharacterOfTagName => "invalid-first-character-of-tag-name",
            Self::MissingEndTagName => "missing-end-tag-name",
            Self::EofInTag => "eof-in-tag",
            Self::UnexpectedEqualsSignBeforeAttributeName => {
                "unexpected-equals-sign-before-attribute-name"
            }
            Self::UnexpectedCharacterInAttributeName => "unexpected-character-in-attribute-name",
            Self::DuplicateAttribute => "duplicate-attribute",
            Self::MissingAttributeValue => "missing-attribute-value",
            Self::UnexpectedCharacterInUnquotedAttributeValue => {
                "unexpected-character-in-unquoted-attribute-value"
            }
            Self::MissingWhitespaceBetweenAttributes => "missing-whitespace-between-attributes",
            Self::UnexpectedSolidusInTag => "unexpected-solidus-in-tag",
            Self::EndTagWithAttributes => "end-tag-with-attributes",
            Self::EndTagWithTrailingSolidus => "end-tag-with-trailing-solidus",
            Self::IncorrectlyOpenedComment => "incorrectly-opened-comment",
            Self::CdataInHtmlContent => "cdata-in-html-content",
            Self::AbruptClosingOfEmptyComment => "abrupt-closing-of-empty-comment",
            Self::EofInComment => "eof-in-comment",
            Self::NestedComment => "nested-comment",
            Self::IncorrectlyClosedComment => "incorrectly-closed-comment",
            Self::MissingWhitespaceBeforeDoctypeName => "missing-whitespace-before-doctype-name",
            Self::MissingDoctypeName => "missing-doctype-name",
            Self::EofInDoctype => "eof-in-doctype",
            Self::InvalidCharacterSequenceAfterDoctypeName => {
                "invalid-character-sequence-after-doctype-name"
            }
            Self::MissingWhitespaceAfterDoctypePublicKeyword => {
                "missing-whitespace-after-doctype-public-keyword"
            }
            Self::MissingWhitespaceAfterDoctypeSystemKeyword => {
                "missing-whitespace-after-doctype-system-keyword"
            }
            Self::MissingDoctypePublicIdentifier => "missing-doctype-public-identifier",
            Self::MissingDoctypeSystemIdentifier => "missing-doctype-system-identifier",
            Self::MissingQuoteBeforeDoctypePublicIdentifier => {
                "missing-quote-before-doctype-public-identifier"
            }
            Self::MissingQuoteBeforeDoctypeSystemIdentifier => {
                "missing-quote-before-doctype-system-identifier"
            }
            Self::AbruptDoctypePublicIdentifier => "abrupt-doctype-public-identifier",
            Self::AbruptDoctypeSystemIdentifier => "abrupt-doctype-system-identifier",
            Self::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers => {
                "missing-whitespace-between-doctype-public-and-system-identifiers"
            }
            Self::UnexpectedCharacterAfterDoctypeSystemIdentifier => {
                "unexpected-character-after-doctype-system-identifier"
            }
            Self::EofInCdata => "eof-in-cdata",
            Self::EofInScriptHtmlCommentLikeText => "eof-in-script-html-comment-like-text",
            Self::EofInProcessingInstruction => "eof-in-processing-instruction",
            Self::InvalidFirstCharacterOfProcessingInstructionTarget => {
                "invalid-first-character-of-processing-instruction-target"
            }
            Self::InvalidProcessingInstructionTarget => "invalid-processing-instruction-target",
            Self::DisallowedProcessingInstructionTarget => {
                "disallowed-processing-instruction-target"
            }
            Self::MissingSemicolonAfterCharacterReference => {
                "missing-semicolon-after-character-reference"
            }
            Self::UnknownNamedCharacterReference => "unknown-named-character-reference",
            Self::AbsenceOfDigitsInNumericCharacterReference => {
                "absence-of-digits-in-numeric-character-reference"
            }
            Self::NullCharacterReference => "null-character-reference",
            Self::CharacterReferenceOutsideUnicodeRange => {
                "character-reference-outside-unicode-range"
            }
            Self::SurrogateCharacterReference => "surrogate-character-reference",
            Self::NoncharacterCharacterReference => "noncharacter-character-reference",
            Self::ControlCharacterReference => "control-character-reference",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticPhase {
    InputStream,
    Tokenizer,
}
pub(crate) enum Event {
    Token(TokenEvent),
    Diagnostic(crate::construction::parser_session::evidence::DiagnosticId),
    /// First committed cursor traversal only; no replay/lookahead duplication.
    Preprocessing {
        _record: super::super::input::PreprocessingRecord,
        _source: HtmlSourceRange,
        retention: OwnedVec<HtmlSourceRange>,
    },
}
impl Event {
    pub(crate) fn token(&self) -> Option<&Token> {
        match self {
            Self::Token(t) => Some(t.token()),
            _ => None,
        }
    }
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        match self {
            Self::Token(t) => t.release(c),
            Self::Diagnostic(_) => Ok(()),
            Self::Preprocessing { retention, .. } => retention.release(c),
        }
    }
}
/// Exactly one token in a fallibly allocated, execution-owned record slot.
/// Its nested text tickets remain owned until release or consumer transfer.
pub(crate) struct TokenEvent {
    record: OwnedVec<Token>,
}
impl TokenEvent {
    pub(super) fn new(token: Token, c: &mut ParseControl) -> Result<Self, ParseStop> {
        let mut record = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        if let Err((stop, token)) = record.try_push_recover(c, token) {
            token.release(c)?;
            record.release(c)?;
            return Err(stop);
        }
        Ok(Self { record })
    }
    pub(crate) fn token(&self) -> &Token {
        &self.record.as_slice()[0]
    }
    pub(crate) fn into_token(mut self, c: &mut ParseControl) -> Result<Token, ParseStop> {
        match self.record.try_pop(c) {
            Ok(Some(token)) => {
                self.record.release(c)?;
                Ok(token)
            }
            Ok(None) => {
                let stop = c.fail_invariant(super::super::control::Invariant::InputCursorState);
                self.record.release(c)?;
                Err(stop)
            }
            Err(stop) => {
                self.release(c)?;
                Err(stop)
            }
        }
    }
    pub(crate) fn release(mut self, c: &mut ParseControl) -> Result<(), ParseStop> {
        loop {
            let token = if c.stop().is_some() {
                self.record.cleanup_pop(c)?
            } else {
                match self.record.try_pop(c) {
                    Ok(token) => token,
                    Err(_) => self.record.cleanup_pop(c)?,
                }
            };
            let Some(token) = token else {
                break;
            };
            token.release(c)?;
        }
        self.record.release(c)
    }
}

impl DiagnosticCode {
    pub(crate) fn rule_anchor(self) -> &'static str {
        match self {
            Self::SurrogateInInputStream => "parse-error-surrogate-in-input-stream",
            Self::NoncharacterInInputStream => "parse-error-noncharacter-in-input-stream",
            Self::ControlCharacterInInputStream => "parse-error-control-character-in-input-stream",
            Self::UnexpectedNullCharacter => "parse-error-unexpected-null-character",
            Self::EofBeforeTagName => "parse-error-eof-before-tag-name",
            Self::InvalidFirstCharacterOfTagName => {
                "parse-error-invalid-first-character-of-tag-name"
            }
            Self::MissingEndTagName => "parse-error-missing-end-tag-name",
            Self::EofInTag => "parse-error-eof-in-tag",
            Self::UnexpectedEqualsSignBeforeAttributeName => {
                "parse-error-unexpected-equals-sign-before-attribute-name"
            }
            Self::UnexpectedCharacterInAttributeName => {
                "parse-error-unexpected-character-in-attribute-name"
            }
            Self::DuplicateAttribute => "parse-error-duplicate-attribute",
            Self::MissingAttributeValue => "parse-error-missing-attribute-value",
            Self::UnexpectedCharacterInUnquotedAttributeValue => {
                "parse-error-unexpected-character-in-unquoted-attribute-value"
            }
            Self::MissingWhitespaceBetweenAttributes => {
                "parse-error-missing-whitespace-between-attributes"
            }
            Self::UnexpectedSolidusInTag => "parse-error-unexpected-solidus-in-tag",
            Self::EndTagWithAttributes => "parse-error-end-tag-with-attributes",
            Self::EndTagWithTrailingSolidus => "parse-error-end-tag-with-trailing-solidus",
            Self::IncorrectlyOpenedComment => "parse-error-incorrectly-opened-comment",
            Self::CdataInHtmlContent => "parse-error-cdata-in-html-content",
            Self::AbruptClosingOfEmptyComment => "parse-error-abrupt-closing-of-empty-comment",
            Self::EofInComment => "parse-error-eof-in-comment",
            Self::NestedComment => "parse-error-nested-comment",
            Self::IncorrectlyClosedComment => "parse-error-incorrectly-closed-comment",
            Self::MissingWhitespaceBeforeDoctypeName => {
                "parse-error-missing-whitespace-before-doctype-name"
            }
            Self::MissingDoctypeName => "parse-error-missing-doctype-name",
            Self::EofInDoctype => "parse-error-eof-in-doctype",
            Self::InvalidCharacterSequenceAfterDoctypeName => {
                "parse-error-invalid-character-sequence-after-doctype-name"
            }
            Self::MissingWhitespaceAfterDoctypePublicKeyword => {
                "parse-error-missing-whitespace-after-doctype-public-keyword"
            }
            Self::MissingWhitespaceAfterDoctypeSystemKeyword => {
                "parse-error-missing-whitespace-after-doctype-system-keyword"
            }
            Self::MissingDoctypePublicIdentifier => "parse-error-missing-doctype-public-identifier",
            Self::MissingDoctypeSystemIdentifier => "parse-error-missing-doctype-system-identifier",
            Self::MissingQuoteBeforeDoctypePublicIdentifier => {
                "parse-error-missing-quote-before-doctype-public-identifier"
            }
            Self::MissingQuoteBeforeDoctypeSystemIdentifier => {
                "parse-error-missing-quote-before-doctype-system-identifier"
            }
            Self::AbruptDoctypePublicIdentifier => "parse-error-abrupt-doctype-public-identifier",
            Self::AbruptDoctypeSystemIdentifier => "parse-error-abrupt-doctype-system-identifier",
            Self::MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers => {
                "parse-error-missing-whitespace-between-doctype-public-and-system-identifiers"
            }
            Self::UnexpectedCharacterAfterDoctypeSystemIdentifier => {
                "parse-error-unexpected-character-after-doctype-system-identifier"
            }
            Self::EofInCdata => "parse-error-eof-in-cdata",
            Self::EofInScriptHtmlCommentLikeText => {
                "parse-error-eof-in-script-html-comment-like-text"
            }
            Self::EofInProcessingInstruction => "parse-error-eof-in-processing-instruction",
            Self::InvalidFirstCharacterOfProcessingInstructionTarget => {
                "parse-error-invalid-first-character-of-processing-instruction-target"
            }
            Self::InvalidProcessingInstructionTarget => {
                "parse-error-invalid-processing-instruction-target"
            }
            Self::DisallowedProcessingInstructionTarget => {
                "parse-error-disallowed-processing-instruction-target"
            }
            Self::MissingSemicolonAfterCharacterReference => {
                "parse-error-missing-semicolon-after-character-reference"
            }
            Self::UnknownNamedCharacterReference => "parse-error-unknown-named-character-reference",
            Self::AbsenceOfDigitsInNumericCharacterReference => {
                "parse-error-absence-of-digits-in-numeric-character-reference"
            }
            Self::NullCharacterReference => "parse-error-null-character-reference",
            Self::CharacterReferenceOutsideUnicodeRange => {
                "parse-error-character-reference-outside-unicode-range"
            }
            Self::SurrogateCharacterReference => "parse-error-surrogate-character-reference",
            Self::NoncharacterCharacterReference => "parse-error-noncharacter-character-reference",
            Self::ControlCharacterReference => "parse-error-control-character-reference",
        }
    }
}

impl TextOrigin {
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn source(&self) -> &HtmlSourceRange {
        &self.source
    }
    pub fn kind(&self) -> OriginKind {
        self.kind
    }
}
