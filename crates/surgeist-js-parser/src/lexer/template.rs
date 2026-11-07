//! Boa's lexing for ECMAScript template literals.

use crate::{
    frontend::{FrontendLimit, FrontendWork},
    lexer::{Cursor, Error, Token, TokenKind, Tokenizer, string::UTF16CodeUnitsBuffer},
    source::ReadChar,
};
use std::io::{self, ErrorKind};
use surgeist_js_ast::PositionGroup;
use surgeist_js_interner::{Interner, Sym};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateString {
    /// The raw template string.
    raw: Sym,

    /// The cooked template string.
    cooked: Option<Sym>,
}

impl TemplateString {
    fn try_new<R: ReadChar>(
        cursor: &mut Cursor<'_, R>,
        interner: &mut Interner,
        spelling: &[u16],
    ) -> Result<Self, FrontendLimit> {
        let raw = cursor.intern(interner, spelling)?;
        let normalized_raw = {
            let units = interner.resolve_expect(raw).utf16();
            let normalized = if let Some(admit) = cursor.frontend_control() {
                SpellingPass::new(units, admit).raw()?
            } else {
                SpellingPass::new(units, |_| Ok(())).raw()?
            };
            cursor.intern(interner, normalized.as_slice())?
        };

        let units = interner.resolve_expect(raw).utf16();
        let cooked_units = if let Some(admit) = cursor.frontend_control() {
            SpellingPass::new(units, admit).cooked()?
        } else {
            SpellingPass::new(units, |_| Ok(())).cooked()?
        };
        let cooked = cooked_units
            .map(|units| cursor.intern(interner, units.as_slice()))
            .transpose()?;
        Ok(Self {
            raw: normalized_raw,
            cooked,
        })
    }

    /// Returns the raw template string.
    pub fn raw(self) -> Sym {
        self.raw
    }

    /// Returns the cooked template string if it exists.
    pub fn cooked(self) -> Option<Sym> {
        self.cooked
    }
}

struct SpellingPass<'a, A> {
    remaining: &'a [u16],
    peeked: Option<u16>,
    output: Vec<u16>,
    admit: A,
}

impl<'a, A: FnMut(FrontendWork) -> Result<(), FrontendLimit>> SpellingPass<'a, A> {
    fn new(input: &'a [u16], admit: A) -> Self {
        Self {
            remaining: input,
            peeked: None,
            output: Vec::new(),
            admit,
        }
    }

    fn pull(&mut self) -> Result<Option<u16>, FrontendLimit> {
        let Some((unit, remaining)) = self.remaining.split_first() else {
            return Ok(None);
        };
        (self.admit)(FrontendWork::TemplateInput)?;
        self.remaining = remaining;
        Ok(Some(*unit))
    }

    fn next(&mut self) -> Result<Option<u16>, FrontendLimit> {
        if let Some(unit) = self.peeked.take() {
            Ok(Some(unit))
        } else {
            self.pull()
        }
    }

    fn peek(&mut self) -> Result<Option<u16>, FrontendLimit> {
        if self.peeked.is_none() {
            self.peeked = self.pull()?;
        }
        Ok(self.peeked)
    }

    fn append(&mut self, point: u32) -> Result<(), FrontendLimit> {
        let width = if point > 0xFFFF { 2 } else { 1 };
        (self.admit)(FrontendWork::TemplateOutput(width))?;
        self.output.push_code_point(point);
        Ok(())
    }

    /// Normalizes the raw line endings without interpreting escapes.
    fn raw(mut self) -> Result<Vec<u16>, FrontendLimit> {
        while let Some(unit) = self.next()? {
            let normalized = if unit == 0x0D { 0x0A } else { unit };
            self.append(u32::from(normalized))?;
        }
        Ok(self.output)
    }

    /// Cooks the template spelling, returning no cooked string for an invalid escape.
    fn cooked(mut self) -> Result<Option<Vec<u16>>, FrontendLimit> {
        while let Some(unit) = self.next()? {
            let point = match unit {
                0x5C => match self.next()? {
                    Some(0x62) => 0x08, // b
                    Some(0x74) => 0x09, // t
                    Some(0x6E) => 0x0A, // n
                    Some(0x76) => 0x0B, // v
                    Some(0x66) => 0x0C, // f
                    Some(0x72) => 0x0D, // r
                    Some(0x30)
                        if self
                            .peek()?
                            .is_none_or(|unit| !(0x30..=0x39).contains(&unit)) =>
                    {
                        0
                    }
                    Some(0x78) => {
                        // x
                        let Some(point) = self.fixed_hex(2)? else {
                            return Ok(None);
                        };
                        point
                    }
                    Some(0x75) => {
                        // u
                        let point = if self.peek()? == Some(0x7B) {
                            self.next()?;
                            self.braced_hex()?
                        } else {
                            self.fixed_hex(4)?
                        };
                        let Some(point) = point else { return Ok(None) };
                        point
                    }
                    Some(0x30..=0x39) | None => return Ok(None),
                    Some(0x0A | 0x0D | 0x2028 | 0x2029) => continue,
                    Some(unit) => u32::from(unit),
                },
                0x0D => 0x0A,
                unit => u32::from(unit),
            };
            self.append(point)?;
        }
        Ok(Some(self.output))
    }

    fn fixed_hex(&mut self, width: usize) -> Result<Option<u32>, FrontendLimit> {
        let mut digits = HexDigits::Start;
        for _ in 0..width {
            let Some(unit) = self.next()? else {
                return Ok(None);
            };
            if !digits.push(unit) {
                return Ok(None);
            }
        }
        Ok(digits.finish())
    }

    fn braced_hex(&mut self) -> Result<Option<u32>, FrontendLimit> {
        let mut digits = HexDigits::Start;
        while let Some(unit) = self.next()? {
            if unit == 0x7D {
                return Ok(digits.finish().filter(|point| *point <= 0x10_FFFF));
            }
            if !digits.push(unit) {
                return Ok(None);
            }
        }
        Ok(None)
    }
}

/// Folds an escape without materializing a digit buffer or rescanning its spelling.
enum HexDigits {
    Start,
    Sign,
    Value(u32),
    Invalid,
}

impl HexDigits {
    fn push(&mut self, unit: u16) -> bool {
        // Surrogates failed before the previous radix conversion could start.
        let Some(ch) = char::from_u32(u32::from(unit)) else {
            return false;
        };
        let value = match *self {
            Self::Start if ch == '+' => {
                *self = Self::Sign;
                return true;
            }
            Self::Start | Self::Sign => 0,
            Self::Value(value) => value,
            Self::Invalid => return true,
        };
        // Retain failure until the whole escape has been inspected.
        *self = value
            .checked_mul(16)
            .and_then(|value| value.checked_add(ch.to_digit(16)?))
            .map_or(Self::Invalid, Self::Value);
        true
    }

    const fn finish(self) -> Option<u32> {
        match self {
            Self::Value(value) => Some(value),
            Self::Start | Self::Sign | Self::Invalid => None,
        }
    }
}

/// Template literal lexing.
///
/// Expects: Initial `` ` `` to already be consumed by cursor.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-template-literals
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
#[derive(Debug, Clone, Copy)]
pub(super) struct TemplateLiteral;

impl<R> Tokenizer<R> for TemplateLiteral {
    fn lex(
        &mut self,
        cursor: &mut Cursor<'_, R>,
        start_pos: PositionGroup,
        interner: &mut Interner,
    ) -> Result<Token, Error>
    where
        R: ReadChar,
    {
        let mut buf = Vec::new();
        let no_substitution = loop {
            let ch = cursor.next_char()?.ok_or_else(|| {
                Error::from(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "unterminated template literal",
                ))
            })?;

            match ch {
                // `
                0x0060 => break true,
                // $
                0x0024 if cursor.next_if(0x7B /* { */)? => break false,
                // \
                0x005C => {
                    let escape_ch = cursor.peek_char()?.ok_or_else(|| {
                        Error::from(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            "unterminated escape sequence in literal",
                        ))
                    })?;

                    buf.push(u16::from(b'\\'));
                    let escape_ch = match escape_ch {
                        // `
                        0x0060 => Some(0x0060),
                        // $
                        0x0024 => Some(0x0024),
                        // \
                        0x005C => Some(0x005C),
                        _ => None,
                    };
                    if let Some(ch) = escape_ch {
                        let _ = cursor.next_char()?.expect("already checked next character");
                        buf.push(ch);
                    }
                }
                ch => {
                    buf.push_code_point(ch);
                }
            }
        };

        let template_string = TemplateString::try_new(cursor, interner, buf.as_slice())?;
        let token_kind = if no_substitution {
            TokenKind::template_no_substitution(template_string)
        } else {
            TokenKind::template_middle(template_string)
        };
        Ok(Token::new_by_position_group(
            token_kind,
            start_pos,
            cursor.pos_group(),
        ))
    }
}
