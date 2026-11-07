//! Boa's lexing for ECMAScript template literals.

use crate::{
    frontend::FrontendLimit,
    lexer::{Cursor, Error, Token, TokenKind, Tokenizer, string::UTF16CodeUnitsBuffer},
    source::ReadChar,
};
use surgeist_js_ast::PositionGroup;
use surgeist_js_interner::{Interner, Sym};
use std::io::{self, ErrorKind};

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
            let units = Self::raw_units(interner.resolve_expect(raw).utf16());
            cursor.intern(interner, units.as_slice())?
        };
        let cooked_units = Self::cooked_units(interner.resolve_expect(raw).utf16());
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

    /// Converts the raw template string into a mutable string slice.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-templatestrings
    fn raw_units(string: &[u16]) -> Vec<u16> {
        let mut iter = string.iter().peekable();
        let mut buf: Vec<u16> = Vec::new();
        loop {
            match iter.next() {
                Some(0x5C /* \ */) => {
                    buf.push_code_point(0x5C);
                    match iter.next() {
                        Some(0x0D /* <CR> */) => {
                            buf.push_code_point(0x0A);
                        }
                        Some(ch) => {
                            buf.push_code_point(u32::from(*ch));
                        }
                        None => break,
                    }
                }
                Some(0x0D /* <CR> */) => {
                    buf.push_code_point(0x0A);
                }
                Some(ch) => {
                    buf.push_code_point(u32::from(*ch));
                }
                None => break,
            }
        }
        buf
    }

    /// Cooks the template spelling, returning `None` for invalid escapes.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-static-semantics-templatestrings
    fn cooked_units(string: &[u16]) -> Option<Vec<u16>> {
        let mut iter = string.iter().peekable();
        let mut buf: Vec<u16> = Vec::new();

        loop {
            match iter.next() {
                Some(0x5C /* \ */) => {
                    let escape_value = match iter.next() {
                        Some(0x62 /* b */) => 0x08 /* <BS> */,
                        Some(0x74 /* t */) => 0x09 /* <HT> */,
                        Some(0x6E /* n */) => 0x0A /* <LF> */,
                        Some(0x76 /* v */) => 0x0B /* <VT> */,
                        Some(0x66 /* f */) => 0x0C /* <FF> */,
                        Some(0x72 /* r */) => 0x0D /* <CR> */,
                        Some(0x22 /* " */) => 0x22 /* " */,
                        Some(0x27 /* ' */) => 0x27 /* ' */,
                        Some(0x5C /* \ */) => 0x5C /* \ */,
                        Some(0x30 /* 0 */) if iter
                            .peek().as_ref().is_none_or(|ch| !(0x30..=0x39 /* 0..=9 */).contains(**ch)) => 0x00 /* NULL */,
                        // Hex Escape
                        Some(0x078 /* x */) => {
                            let mut s = String::with_capacity(2);
                            s.push(char::from_u32(u32::from(*iter.next()?))?);
                            s.push(char::from_u32(u32::from(*iter.next()?))?);
                            u16::from_str_radix(&s, 16).ok()?.into()
                        }
                        // Unicode Escape
                        Some(0x75 /* u */) => {
                            let next = *iter.next()?;
                            if next ==  0x7B /* { */ {
                                let mut buffer = String::with_capacity(6);
                                loop {
                                    let next = *iter.next()?;
                                    if next == 0x7D /* } */ {
                                        break;
                                    }
                                    buffer.push(char::from_u32(u32::from(next))?);
                                }
                                let cp = u32::from_str_radix(&buffer, 16).ok()?;
                                if cp > 0x10_FFFF {
                                    return None;
                                }
                                cp
                            } else {
                                let mut s = String::with_capacity(4);
                                s.push(char::from_u32(u32::from(next))?);
                                s.push(char::from_u32(u32::from(*iter.next()?))?);
                                s.push(char::from_u32(u32::from(*iter.next()?))?);
                                s.push(char::from_u32(u32::from(*iter.next()?))?);
                                u16::from_str_radix(&s, 16).ok()?.into()
                            }
                        }
                        // NonOctalDecimalEscapeSequence
                        Some(0x38 /* 8 */ | 0x39 /* 9 */) => {
                            return None;
                        }
                        // LegacyOctalEscapeSequence
                        Some(ch) if (0x30..=0x37 /* '0'..='7' */).contains(ch) => {
                            return None;
                        }
                        // Line Terminator
                        Some(0x0A /* <LF> */ | 0x0D /* <CR> */ | 0x2028 /* <LS> */ | 0x2029 /* <PS> */) => {
                            continue;
                        }
                        Some(ch) => {
                            u32::from(*ch)
                        }
                        None => return None,
                    };
                    buf.push_code_point(escape_value);
                }
                Some(0x0D /* <CR> */) => {
                    buf.push_code_point(0x0A);
                }
                Some(ch) => {
                    buf.push_code_point(u32::from(*ch));
                }
                None => break,
            }
        }

        Some(buf)
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
