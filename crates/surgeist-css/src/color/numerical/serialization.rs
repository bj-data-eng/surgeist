//! Fixed resolved serialization plan and finite output continuation.
//!
//! This numerical cursor has no CSS source, fact, owner or resolver record.
//! Its adapter qualifies the caller bank before lending one actual digit.

use super::super::alpha::byte_alpha_for_serialization;
use super::value::{CssUsedColor, CssUsedColorError};
use crate::finite_format::RoundingMode;
use crate::finite_format::bounded::{FiniteFormatCursor, FormatDigit, FormatReply, FormatRequest};
use crate::{CssColorInterpolationSpace as Space, CssPredefinedColorSpace as Predefined};

/// Selects the standard used serialization, including the conditional HTML form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorSerializationRequest {
    Css,
    HtmlCompatible,
}

#[derive(Clone, Copy)]
enum Piece {
    Literal(&'static [u8]),
    Number(f64),
    None,
    Hex(u8),
    End,
}

struct Plan {
    pieces: [Piece; 19],
}
impl Plan {
    fn new(
        value: CssUsedColor,
        request: CssColorSerializationRequest,
    ) -> Result<Self, CssUsedColorError> {
        let mut pieces = [Piece::End; 19];
        if request == CssColorSerializationRequest::HtmlCompatible
            && value.alpha() == Some(1.0)
            && let Some([r, g, b]) = value.srgb_bytes()
        {
            pieces[..4].copy_from_slice(&[
                Piece::Literal(b"#"),
                Piece::Hex(r),
                Piece::Hex(g),
                Piece::Hex(b),
            ]);
            return Ok(Self { pieces });
        }
        let missing = value.channels().contains(&None) || value.alpha().is_none();
        let value = if matches!(value.space(), Space::Hsl | Space::Hwb) && !missing {
            let converted = value.convert(Space::Predefined(Predefined::Srgb))?;
            if value.uses_color_function() {
                converted.with_color_function()
            } else {
                converted.with_legacy_srgb()
            }
        } else {
            value
        };
        let space = value.space();
        let legacy = space == Space::Predefined(Predefined::Srgb)
            && !value.uses_color_function()
            && !missing;
        let alpha = value.alpha() != Some(1.0);
        let (prefix, percentages): (&'static [u8], bool) = match space {
            Space::Predefined(Predefined::Srgb) if legacy => {
                (if alpha { b"rgba(" } else { b"rgb(" }, false)
            }
            Space::Predefined(Predefined::Srgb) => (b"color(srgb ", false),
            Space::Predefined(Predefined::SrgbLinear) => (b"color(srgb-linear ", false),
            Space::Predefined(Predefined::DisplayP3) => (b"color(display-p3 ", false),
            Space::Predefined(Predefined::DisplayP3Linear) => (b"color(display-p3-linear ", false),
            Space::Predefined(Predefined::A98Rgb) => (b"color(a98-rgb ", false),
            Space::Predefined(Predefined::ProphotoRgb) => (b"color(prophoto-rgb ", false),
            Space::Predefined(Predefined::Rec2020) => (b"color(rec2020 ", false),
            Space::Predefined(Predefined::XyzD50) => (b"color(xyz-d50 ", false),
            Space::Predefined(Predefined::XyzD65) => (b"color(xyz-d65 ", false),
            Space::Hsl => (b"hsl(", true),
            Space::Hwb => (b"hwb(", true),
            Space::Lab => (b"lab(", false),
            Space::Lch => (b"lch(", false),
            Space::Oklab => (b"oklab(", false),
            Space::Oklch => (b"oklch(", false),
        };
        let separator: &'static [u8] = if legacy { b", " } else { b" " };
        let channels = value.channels();
        pieces[0] = Piece::Literal(prefix);
        let mut next = 1;
        // Exactly three fixed channels, never a variable component/profile scan.
        for (index, channel) in channels.into_iter().enumerate() {
            pieces[next] = match channel {
                Some(number) => Piece::Number(if legacy {
                    number.clamp(0.0, 1.0) * 255.0
                } else {
                    number
                }),
                None => Piece::None,
            };
            next += 1;
            if percentages && index != 0 && channel.is_some() {
                pieces[next] = Piece::Literal(b"%");
                next += 1;
            }
            if index != 2 {
                pieces[next] = Piece::Literal(separator);
                next += 1;
            }
        }
        if alpha {
            pieces[next] = Piece::Literal(if legacy { b", " } else { b" / " });
            next += 1;
            pieces[next] = match value.alpha() {
                None => Piece::None,
                Some(number) => Piece::Number(if legacy {
                    value
                        .alpha_byte()
                        .map_or(number, byte_alpha_for_serialization)
                } else {
                    number
                }),
            };
            next += 1;
        }
        pieces[next] = Piece::Literal(b")");
        Ok(Self { pieces })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SerializationAction {
    Plan,
    Compute,
    ReadDigit { index: usize },
    AppendDigit { index: usize },
    ReplaceDigit { index: usize },
    Emit { unit: u8 },
    Done,
    Stopped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SerializationProtocolError {
    WrongAction,
    WrongIndex,
    Overflow,
    Terminal,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SerializationKernel {
    Plan,
    Format,
    Transition,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SerializationError {
    Protocol(SerializationProtocolError),
    Numerical(CssUsedColorError),
}
enum Stage {
    Plan,
    Pieces,
    Format(FiniteFormatCursor),
    Done,
    Stopped,
}

/// Fixed state only. The caller owns all digits and every final output byte.
pub(crate) struct SerializationCursor {
    value: CssUsedColor,
    request: CssColorSerializationRequest,
    plan: Option<Plan>,
    stage: Stage,
    piece: usize,
    offset: usize,
    digit_base: usize,
    digits: usize,
    emitted: usize,
}
impl SerializationCursor {
    /// The caller admits the fixed continuation before construction. Planning
    /// and numerical conversion remain pending until the Plan action is serviced.
    pub(crate) fn new(value: CssUsedColor, request: CssColorSerializationRequest) -> Self {
        Self {
            value,
            request,
            plan: None,
            stage: Stage::Plan,
            piece: 0,
            offset: 0,
            digit_base: 0,
            digits: 0,
            emitted: 0,
        }
    }
    pub(crate) fn action(&self) -> SerializationAction {
        match &self.stage {
            Stage::Plan => SerializationAction::Plan,
            Stage::Pieces => match self.plan.as_ref().expect("planned pieces").pieces[self.piece] {
                Piece::Literal(bytes) => bytes
                    .get(self.offset)
                    .map_or(SerializationAction::Compute, |unit| {
                        SerializationAction::Emit { unit: *unit }
                    }),
                Piece::None => b"none"
                    .get(self.offset)
                    .map_or(SerializationAction::Compute, |unit| {
                        SerializationAction::Emit { unit: *unit }
                    }),
                Piece::Hex(byte) if self.offset < 2 => SerializationAction::Emit {
                    unit: b"0123456789abcdef"[usize::from(if self.offset == 0 {
                        byte >> 4
                    } else {
                        byte & 15
                    })],
                },
                Piece::Hex(_) | Piece::Number(_) | Piece::End => SerializationAction::Compute,
            },
            Stage::Format(cursor) => match cursor.next_request() {
                FormatRequest::Compute(_) | FormatRequest::Done(_) => SerializationAction::Compute,
                FormatRequest::ReadDigit { index } => SerializationAction::ReadDigit {
                    index: self.digit_base + index,
                },
                FormatRequest::AppendDigit { index, .. } => SerializationAction::AppendDigit {
                    index: self.digit_base + index,
                },
                FormatRequest::ReplaceDigit { index, .. } => SerializationAction::ReplaceDigit {
                    index: self.digit_base + index,
                },
                FormatRequest::EmitByte { unit } => SerializationAction::Emit { unit },
            },
            Stage::Done => SerializationAction::Done,
            Stage::Stopped => SerializationAction::Stopped,
        }
    }
    pub(crate) fn kernel(&self) -> SerializationKernel {
        match &self.stage {
            Stage::Plan => SerializationKernel::Plan,
            Stage::Format(cursor) if matches!(cursor.next_request(), FormatRequest::Compute(_)) => {
                SerializationKernel::Format
            }
            _ => SerializationKernel::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), SerializationError> {
        match &mut self.stage {
            Stage::Plan => {
                let plan = match Plan::new(self.value, self.request) {
                    Ok(plan) => plan,
                    Err(error) => {
                        self.stage = Stage::Stopped;
                        return Err(SerializationError::Numerical(error));
                    }
                };
                self.plan = Some(plan);
                self.stage = Stage::Pieces;
            }
            Stage::Pieces => match self.plan.as_ref().expect("planned pieces").pieces[self.piece] {
                Piece::Number(value) => {
                    self.digit_base = self.digits;
                    self.stage = Stage::Format(
                        FiniteFormatCursor::new(value, RoundingMode::TowardPositiveInfinity)
                            .map_err(|_| {
                                SerializationError::Numerical(CssUsedColorError::Unrepresentable)
                            })?,
                    );
                }
                Piece::End => self.stage = Stage::Done,
                _ if self.action() == SerializationAction::Compute => {
                    self.piece += 1;
                    self.offset = 0;
                }
                _ => {
                    return Err(SerializationError::Protocol(
                        SerializationProtocolError::WrongAction,
                    ));
                }
            },
            Stage::Format(cursor) => match cursor.next_request() {
                FormatRequest::Compute(kernel) => cursor
                    .complete(FormatReply::Computed(kernel))
                    .map_err(|_| {
                        SerializationError::Protocol(SerializationProtocolError::WrongAction)
                    })?,
                FormatRequest::Done(_) => {
                    self.stage = Stage::Pieces;
                    self.piece += 1;
                    self.offset = 0;
                }
                _ => {
                    return Err(SerializationError::Protocol(
                        SerializationProtocolError::WrongAction,
                    ));
                }
            },
            Stage::Done | Stage::Stopped => {
                return Err(SerializationError::Protocol(
                    SerializationProtocolError::Terminal,
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn read_digit(
        &mut self,
        index: usize,
        digit: FormatDigit,
    ) -> Result<(), SerializationProtocolError> {
        if self.action() != (SerializationAction::ReadDigit { index }) {
            return Err(SerializationProtocolError::WrongAction);
        }
        let Stage::Format(cursor) = &mut self.stage else {
            return Err(SerializationProtocolError::WrongAction);
        };
        cursor
            .complete(FormatReply::Digit {
                index: index - self.digit_base,
                digit,
            })
            .map_err(|_| SerializationProtocolError::WrongAction)
    }
    pub(crate) fn digit_record(
        &self,
        index: usize,
    ) -> Result<FormatDigit, SerializationProtocolError> {
        let Stage::Format(cursor) = &self.stage else {
            return Err(SerializationProtocolError::WrongAction);
        };
        let digit = match cursor.next_request() {
            FormatRequest::AppendDigit {
                index: expected,
                digit,
            }
            | FormatRequest::ReplaceDigit {
                index: expected,
                digit,
            } if self.digit_base + expected == index => digit,
            _ => return Err(SerializationProtocolError::WrongIndex),
        };
        if matches!(self.action(), SerializationAction::AppendDigit { .. }) {
            self.digits
                .checked_add(1)
                .ok_or(SerializationProtocolError::Overflow)?;
        }
        Ok(digit)
    }
    /// Called only after the actual append/replacement succeeds. Refused storage
    /// leaves the same request available, including the exact old digit.
    pub(crate) fn digit_written(
        &mut self,
        index: usize,
        digit: FormatDigit,
    ) -> Result<(), SerializationProtocolError> {
        let action = self.action();
        let reply = match action {
            SerializationAction::AppendDigit { index: expected }
                if index == expected && index == self.digits =>
            {
                FormatReply::Appended {
                    index: index - self.digit_base,
                    digit,
                }
            }
            SerializationAction::ReplaceDigit { index: expected } if index == expected => {
                FormatReply::Replaced {
                    index: index - self.digit_base,
                    digit,
                }
            }
            _ => return Err(SerializationProtocolError::WrongIndex),
        };
        let next = if matches!(action, SerializationAction::AppendDigit { .. }) {
            self.digits
                .checked_add(1)
                .ok_or(SerializationProtocolError::Overflow)?
        } else {
            self.digits
        };
        let Stage::Format(cursor) = &mut self.stage else {
            return Err(SerializationProtocolError::WrongAction);
        };
        cursor
            .complete(reply)
            .map_err(|_| SerializationProtocolError::WrongAction)?;
        self.digits = next;
        Ok(())
    }
    pub(crate) fn emitted(&mut self, unit: u8) -> Result<(), SerializationProtocolError> {
        if self.action() != (SerializationAction::Emit { unit }) {
            return Err(SerializationProtocolError::WrongAction);
        }
        let next = self
            .emitted
            .checked_add(1)
            .ok_or(SerializationProtocolError::Overflow)?;
        match &mut self.stage {
            Stage::Format(cursor) => cursor
                .complete(FormatReply::Emitted { unit })
                .map_err(|_| SerializationProtocolError::WrongAction)?,
            Stage::Pieces => self.offset += 1,
            _ => return Err(SerializationProtocolError::WrongAction),
        }
        self.emitted = next;
        Ok(())
    }
    pub(crate) fn abandon(&mut self) {
        self.stage = Stage::Stopped;
    }
    pub(crate) fn completed_length(&self) -> Result<usize, SerializationProtocolError> {
        if matches!(self.stage, Stage::Done) {
            Ok(self.emitted)
        } else {
            Err(SerializationProtocolError::Terminal)
        }
    }
}
