//! Rendering of source spellings in owned parser diagnostics.

use crate::frontend::{FrontendLimit, FrontendWork};
use surgeist_js_interner::{Interner, JSInternedStrRef, Sym};

type Admission<'a> = dyn FnMut(FrontendWork) -> Result<(), FrontendLimit> + 'a;

/// A spelling can outlive the token borrow without cloning its payload.
pub(crate) enum Spelling {
    Ready(String),
    Symbol(Sym),
    Private(Sym),
    RegExp(Sym, Sym),
}

impl Spelling {
    #[cold]
    pub(crate) fn render(
        self,
        interner: &Interner,
        prefix: &'static str,
        suffix: &'static str,
        control: Option<&mut Admission<'_>>,
    ) -> Result<String, FrontendLimit> {
        let mut writer = Writer {
            text: String::new(),
            control,
        };
        writer.append(prefix)?;
        match self {
            Self::Ready(text) => {
                debug_assert!(prefix.is_empty() && suffix.is_empty());
                // Fixed spellings and opaque numeric formatting have already rendered.
                return Ok(text);
            }
            Self::Symbol(sym) => writer.spelling(interner.resolve_expect(sym))?,
            Self::Private(sym) => {
                writer.append("#")?;
                writer.spelling(interner.resolve_expect(sym))?;
            }
            Self::RegExp(body, flags) => {
                writer.append("/")?;
                writer.spelling(interner.resolve_expect(body))?;
                writer.append("/")?;
                writer.spelling(interner.resolve_expect(flags))?;
            }
        }
        writer.append(suffix)?;
        Ok(writer.text)
    }
}

struct Writer<'borrow, 'control> {
    text: String,
    control: Option<&'borrow mut Admission<'control>>,
}

impl Writer<'_, '_> {
    fn admit(&mut self, work: FrontendWork) -> Result<(), FrontendLimit> {
        self.control
            .as_deref_mut()
            .map_or(Ok(()), |admit| admit(work))
    }

    fn append(&mut self, text: &str) -> Result<(), FrontendLimit> {
        if !text.is_empty() {
            self.admit(FrontendWork::DiagnosticOutput(text.len()))?;
            self.text.push_str(text);
        }
        Ok(())
    }

    fn inspect(&mut self, units: &mut &[u16]) -> Result<Option<u16>, FrontendLimit> {
        let Some((first, rest)) = units.split_first() else {
            return Ok(None);
        };
        self.admit(FrontendWork::DiagnosticInput)?;
        *units = rest;
        Ok(Some(*first))
    }

    fn spelling(&mut self, text: JSInternedStrRef<'_, '_>) -> Result<(), FrontendLimit> {
        if self.control.is_none()
            && let Some(utf8) = text.utf8()
        {
            return self.append(utf8);
        }

        let mut units = text.utf16();
        let mut pending = None;
        loop {
            let first = if let Some(unit) = pending.take() {
                unit
            } else if let Some(unit) = self.inspect(&mut units)? {
                unit
            } else {
                return Ok(());
            };

            let point = match first {
                0xD800..=0xDBFF => match self.inspect(&mut units)? {
                    Some(low @ 0xDC00..=0xDFFF) => {
                        0x10000 + (u32::from(first - 0xD800) << 10) + u32::from(low - 0xDC00)
                    }
                    next => {
                        pending = next;
                        self.surrogate(first)?;
                        continue;
                    }
                },
                0xDC00..=0xDFFF => {
                    self.surrogate(first)?;
                    continue;
                }
                _ => u32::from(first),
            };
            let ch = char::from_u32(point).expect("decoded Unicode scalar");
            self.append(ch.encode_utf8(&mut [0; 4]))?;
        }
    }

    fn surrogate(&mut self, unit: u16) -> Result<(), FrontendLimit> {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let escape = [
            b'\\',
            b'u',
            HEX[usize::from(unit >> 12)],
            HEX[usize::from((unit >> 8) & 0xF)],
            HEX[usize::from((unit >> 4) & 0xF)],
            HEX[usize::from(unit & 0xF)],
        ];
        self.append(std::str::from_utf8(&escape).expect("ASCII escape"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Source,
        lexer::{InputElement, Lexer},
    };

    #[test]
    fn diagnostic_rendering_preserves_regex_private_name_and_raw_template_payloads() {
        for (input, expected, units, bytes) in [
            ("#π", "#π", 1, 3),
            ("/π/g", "/π/g", 2, 5),
            (r"`é\u0061`", r"é\u0061", 7, 8),
        ] {
            let mut interner = Interner::new();
            let mut lexer = Lexer::new(Source::from_bytes(input).reader);
            lexer.set_goal(InputElement::RegExp);
            let token = lexer.next(&mut interner).unwrap().unwrap();
            let mut inspected = 0;
            let mut emitted = 0;
            let mut admit = |work| {
                match work {
                    FrontendWork::DiagnosticInput => inspected += 1,
                    FrontendWork::DiagnosticOutput(bytes) => emitted += bytes,
                    _ => panic!("unexpected rendering work: {work:?}"),
                }
                Ok(())
            };
            let rendered = token
                .kind()
                .diagnostic_spelling()
                .render(&interner, "", "", Some(&mut admit))
                .unwrap();
            assert_eq!(rendered, expected);
            assert_eq!(token.kind().to_string(&interner), expected);
            assert_eq!((inspected, emitted), (units, bytes));
        }
    }
}
