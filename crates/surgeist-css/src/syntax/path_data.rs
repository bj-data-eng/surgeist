//! Checked decoded SVG 1.1 path syntax, without geometry or numeric conversion.

use crate::{CssComponentValue, CssComponentValueRef, CssValueOrigin, CssValueTokenRef};
use std::fmt;

/// Intrinsic failure to construct checked decoded SVG path data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPathDataConstructionErrorKind {
    /// The supplied component is not a quoted string.
    ExpectedString,
    /// The quoted string has an implicitly recovered termination.
    RecoveredInput,
    /// The decoded string is empty or contains only SVG whitespace.
    EmptyPath,
    /// The decoded string does not satisfy the complete SVG path grammar.
    InvalidPathData,
}

/// A path admission failure retaining input provenance and decoded-byte location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssPathDataConstructionError {
    kind: CssPathDataConstructionErrorKind,
    origin: CssValueOrigin,
    decoded_byte_offset: Option<usize>,
}

impl CssPathDataConstructionError {
    /// Returns the intrinsic construction failure.
    #[must_use]
    pub const fn kind(&self) -> CssPathDataConstructionErrorKind {
        self.kind
    }
    /// Returns the original component or programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    /// Returns the first invalid decoded byte, or decoded length for incomplete data.
    /// This offset is independent of CSS escaping and is not a CSS source coordinate.
    #[must_use]
    pub const fn decoded_byte_offset(&self) -> Option<usize> {
        self.decoded_byte_offset
    }
}

impl fmt::Display for CssPathDataConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS path data {:?}", self.kind)?;
        if let Some(offset) = self.decoded_byte_offset {
            write!(formatter, " at decoded byte {offset}")?;
        }
        Ok(())
    }
}
impl std::error::Error for CssPathDataConstructionError {}

/// Complete authored SVG path data, preserving decoded spelling and provenance.
///
/// Equality compares decoded bytes independently of CSS token origin. Command case,
/// separators and numeric spellings remain significant; no geometry is computed.
#[derive(Clone, Debug)]
pub struct CssPathData {
    decoded: String,
    origin: CssValueOrigin,
}
impl PartialEq for CssPathData {
    fn eq(&self, other: &Self) -> bool {
        self.decoded == other.decoded
    }
}
impl Eq for CssPathData {}

#[derive(Clone, Copy)]
enum Admission {
    Strict,
    RecoveredSyntax,
}

impl CssPathData {
    /// Validates a complete decoded path with programmatic provenance.
    pub fn try_new(decoded: impl Into<String>) -> Result<Self, CssPathDataConstructionError> {
        Self::validate(decoded.into(), CssValueOrigin::Programmatic)
    }
    /// Admits exactly one complete quoted-string component, retaining its origin.
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssPathDataConstructionError> {
        Self::from_component(component, Admission::Strict)
    }
    /// Returns the unchanged decoded SVG spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.decoded
    }
    /// Returns the original quoted-string token or programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }

    pub(crate) fn from_parser_component(
        component: CssComponentValue,
        context: &crate::numeric::NumericInputContext<'_>,
    ) -> Result<Self, CssPathDataConstructionError> {
        let admission = match context {
            crate::numeric::NumericInputContext::Parsed(_) => Admission::RecoveredSyntax,
            crate::numeric::NumericInputContext::Components(..) => Admission::Strict,
        };
        Self::from_component(component, admission)
    }
    fn from_component(
        component: CssComponentValue,
        admission: Admission,
    ) -> Result<Self, CssPathDataConstructionError> {
        let CssComponentValueRef::Token(CssValueTokenRef::String(decoded)) = component.view()
        else {
            return Err(CssPathDataConstructionError {
                kind: CssPathDataConstructionErrorKind::ExpectedString,
                origin: component.origin().clone(),
                decoded_byte_offset: None,
            });
        };
        if matches!(admission, Admission::Strict)
            && let Some(origin) = component.implicit_termination_origin()
        {
            return Err(CssPathDataConstructionError {
                kind: CssPathDataConstructionErrorKind::RecoveredInput,
                origin: origin.clone(),
                decoded_byte_offset: None,
            });
        }
        Self::validate(decoded.to_owned(), component.origin().clone())
    }
    fn validate(
        decoded: String,
        origin: CssValueOrigin,
    ) -> Result<Self, CssPathDataConstructionError> {
        match Scanner::new(&decoded).validate() {
            Ok(()) => Ok(Self { decoded, origin }),
            Err(offset) => Err(CssPathDataConstructionError {
                kind: if offset.is_none() {
                    CssPathDataConstructionErrorKind::EmptyPath
                } else {
                    CssPathDataConstructionErrorKind::InvalidPathData
                },
                origin,
                decoded_byte_offset: offset,
            }),
        }
    }
}

// SVG 1.1 §8.3.9 BNF with maximal numeric consumption. The adopted Appendix
// F.6.2 interpretation admits signed/zero radii, without applying correction.
// Iterative, linear in decoded bytes and constant in auxiliary memory.
struct Scanner<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Scanner<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            bytes: data.as_bytes(),
            offset: 0,
        }
    }
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.offset).copied()
    }
    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.offset += 1;
        }
    }
    fn separator(&mut self, required: bool) -> Result<(), usize> {
        let start = self.offset;
        self.whitespace();
        if self.peek() == Some(b',') {
            self.offset += 1;
            self.whitespace();
        }
        if required && self.offset == start {
            return Err(self.offset);
        }
        Ok(())
    }
    fn digits(&mut self) -> bool {
        let start = self.offset;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.offset += 1;
        }
        self.offset != start
    }
    fn number(&mut self) -> Result<(), usize> {
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.offset += 1;
        }
        let integer = self.digits();
        if self.peek() == Some(b'.') {
            self.offset += 1;
            if !self.digits() && !integer {
                return Err(self.offset);
            }
        } else if !integer {
            return Err(self.offset);
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.offset += 1;
            }
            if !self.digits() {
                return Err(self.offset);
            }
        }
        Ok(())
    }
    fn group(&mut self, command: u8, arity: usize) -> Result<(), usize> {
        for index in 0..arity {
            if index != 0 {
                self.separator(command == b'A' && index == 3)?;
            }
            if command == b'A' && matches!(index, 3 | 4) {
                if !matches!(self.peek(), Some(b'0' | b'1')) {
                    return Err(self.offset);
                }
                self.offset += 1;
            } else {
                self.number()?;
            }
        }
        Ok(())
    }
    fn validate(mut self) -> Result<(), Option<usize>> {
        self.whitespace();
        if self.peek().is_none() {
            return Err(None);
        }
        if !matches!(self.peek(), Some(b'M' | b'm')) {
            return Err(Some(self.offset));
        }
        while let Some(command) = self.peek() {
            let command = command.to_ascii_uppercase();
            let arity = match command {
                b'M' | b'L' | b'T' => 2,
                b'H' | b'V' => 1,
                b'C' => 6,
                b'S' | b'Q' => 4,
                b'A' => 7,
                b'Z' => 0,
                _ => return Err(Some(self.offset)),
            };
            self.offset += 1;
            self.whitespace();
            if arity != 0 {
                loop {
                    self.group(command, arity).map_err(Some)?;
                    self.whitespace();
                    if self.peek().is_none()
                        || self.peek().is_some_and(|byte| byte.is_ascii_alphabetic())
                    {
                        break;
                    }
                    if self.peek() == Some(b',') {
                        let comma = self.offset;
                        self.offset += 1;
                        self.whitespace();
                        if self.peek().is_none()
                            || self.peek().is_some_and(|byte| byte.is_ascii_alphabetic())
                        {
                            return Err(Some(comma));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
