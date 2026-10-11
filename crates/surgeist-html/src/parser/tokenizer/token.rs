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
    control::{OwnedVec, ParseControl, ParseStop, Resource, WorkCategory},
    input::CodePoint,
};
use crate::HtmlSourceRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OriginKind {
    Literal,
    Newline,
    AsciiLowercase,
    NullReplacement,
    CharacterReference,
    RecoverySyntax,
}
pub(crate) struct TextOrigin {
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
            origins: OwnedVec::new(Resource::SourceMapEntries)?,
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
pub(crate) enum DiagnosticCode {
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
pub(crate) struct Diagnostic {
    pub(crate) code: DiagnosticCode,
    pub(crate) phase: DiagnosticPhase,
    pub(crate) state: super::states::State,
    pub(crate) source: HtmlSourceRange,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticPhase {
    InputStream,
    Tokenizer,
}
pub(crate) enum Event {
    Token(TokenEvent),
    Diagnostic(OwnedVec<Diagnostic>),
    /// First committed cursor traversal only; no replay/lookahead duplication.
    Preprocessing {
        record: super::super::input::PreprocessingRecord,
        source: HtmlSourceRange,
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
            Self::Diagnostic(d) => d.release(c),
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
        let mut record = OwnedVec::new(Resource::AuxiliaryEntries)?;
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
