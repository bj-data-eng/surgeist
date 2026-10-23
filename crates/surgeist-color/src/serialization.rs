//! Fixed resolved serialization plan and finite output continuation.
//!
//! This numerical cursor has no CSS source, fact, owner or resolver record.
//! Its adapter qualifies the caller bank before lending one actual digit.

use super::value::Color;
use crate::alpha::byte_alpha_for_serialization;
use crate::finite::{Finite, closest_finite};
use crate::format::RoundingMode;
use crate::format::bounded::{FiniteFormatCursor, FormatDigit, FormatReply, FormatRequest};
use crate::{ColorSpace as Space, PredefinedColorSpace as Predefined};

/// Selects the standard used serialization, including the conditional HTML form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationRequest {
    Css,
    HtmlCompatible,
}

#[derive(Clone, Copy)]
enum Piece {
    Literal(&'static [u8]),
    Number(Finite),
    None,
    Hex(u8),
    End,
}

struct Plan {
    pieces: [Piece; 19],
}
impl Plan {
    fn new(value: Color, request: SerializationRequest) -> Self {
        let mut pieces = [Piece::End; 19];
        if request == SerializationRequest::HtmlCompatible
            && value.alpha() == Some(1.0)
            && let Some([r, g, b]) = value.srgb_bytes()
        {
            pieces[..4].copy_from_slice(&[
                Piece::Literal(b"#"),
                Piece::Hex(r),
                Piece::Hex(g),
                Piece::Hex(b),
            ]);
            return Self { pieces };
        }
        let missing = value.channels().contains(&None) || value.alpha().is_none();
        let value = if matches!(value.space(), Space::Hsl | Space::Hwb) && !missing {
            let converted = value.converted(Space::Predefined(Predefined::Srgb));
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
                Some(number) => Piece::Number(closest_finite(if legacy {
                    number.clamp(0.0, 1.0) * 255.0
                } else {
                    number
                })),
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
                Some(number) => Piece::Number(closest_finite(if legacy {
                    value
                        .alpha_byte()
                        .map_or(number, byte_alpha_for_serialization)
                } else {
                    number
                })),
            };
            next += 1;
        }
        pieces[next] = Piece::Literal(b")");
        Self { pieces }
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
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SerializationKernel {
    Plan,
    Format,
    Transition,
}
enum Stage {
    Plan,
    Pieces,
    Format(FiniteFormatCursor),
    Done,
}

/// Fixed state only. The caller owns all digits and every final output byte.
///
/// Each completion method requires its own pending action. The typed actions
/// in `controlled` call only the method of the action they were issued for,
/// so a mismatch is a defect in this crate and panics.
pub(crate) struct SerializationCursor {
    value: Color,
    request: SerializationRequest,
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
    pub(crate) fn new(value: Color, request: SerializationRequest) -> Self {
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
    /// The serialized value, unchanged by planning.
    pub(crate) const fn value(&self) -> Color {
        self.value
    }
    /// Count of acknowledged output bytes.
    pub(crate) const fn emitted(&self) -> usize {
        self.emitted
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
    pub(crate) fn format_kernel(&self) -> Option<crate::format::bounded::FormatKernel> {
        match &self.stage {
            Stage::Format(c) => match c.next_request() {
                FormatRequest::Compute(k) => Some(k),
                _ => None,
            },
            _ => None,
        }
    }
    pub(crate) fn compute(&mut self) {
        let pending = self.action();
        match &mut self.stage {
            Stage::Plan => {
                self.plan = Some(Plan::new(self.value, self.request));
                self.stage = Stage::Pieces;
            }
            Stage::Pieces => match self.plan.as_ref().expect("planned pieces").pieces[self.piece] {
                Piece::Number(value) => {
                    self.digit_base = self.digits;
                    self.stage = Stage::Format(FiniteFormatCursor::new(
                        value,
                        RoundingMode::TowardPositiveInfinity,
                    ));
                }
                Piece::End => self.stage = Stage::Done,
                _ => {
                    assert_eq!(
                        pending,
                        SerializationAction::Compute,
                        "a piece transition is computed only after its last byte"
                    );
                    self.piece += 1;
                    self.offset = 0;
                }
            },
            Stage::Format(cursor) => match cursor.next_request() {
                FormatRequest::Compute(kernel) => cursor
                    .complete(FormatReply::Computed(kernel))
                    .expect("a pending format kernel accepts its own completion"),
                FormatRequest::Done(_) => {
                    self.stage = Stage::Pieces;
                    self.piece += 1;
                    self.offset = 0;
                }
                request => panic!("serialization compute issued for pending {request:?}"),
            },
            Stage::Done => panic!("a complete serialization has no pending computation"),
        }
    }
    pub(crate) fn read_digit(&mut self, index: usize, digit: FormatDigit) {
        assert_eq!(
            self.action(),
            SerializationAction::ReadDigit { index },
            "a digit read is issued only for the pending read"
        );
        let Stage::Format(cursor) = &mut self.stage else {
            panic!("a pending digit read belongs to a number")
        };
        cursor
            .complete(FormatReply::Digit {
                index: index - self.digit_base,
                digit,
            })
            .expect("a pending digit read accepts its own record");
    }
    /// The digit the pending append or replacement at `index` writes.
    pub(crate) fn pending_digit(&self, index: usize) -> FormatDigit {
        let Stage::Format(cursor) = &self.stage else {
            panic!("a pending digit write belongs to a number")
        };
        match cursor.next_request() {
            FormatRequest::AppendDigit {
                index: expected,
                digit,
            }
            | FormatRequest::ReplaceDigit {
                index: expected,
                digit,
            } if self.digit_base + expected == index => digit,
            request => panic!("digit write at index {index} issued for pending {request:?}"),
        }
    }
    /// Called only after the actual append/replacement succeeds. Refused storage
    /// leaves the same request available, including the exact old digit.
    pub(crate) fn digit_written(&mut self, index: usize, digit: FormatDigit) {
        let action = self.action();
        let (reply, next) = match action {
            SerializationAction::AppendDigit { index: expected }
                if index == expected && index == self.digits =>
            {
                (
                    FormatReply::Appended {
                        index: index - self.digit_base,
                        digit,
                    },
                    self.digits
                        .checked_add(1)
                        .expect("at most 309 digits per number fit in usize"),
                )
            }
            SerializationAction::ReplaceDigit { index: expected } if index == expected => (
                FormatReply::Replaced {
                    index: index - self.digit_base,
                    digit,
                },
                self.digits,
            ),
            _ => panic!("digit write at index {index} issued for pending {action:?}"),
        };
        let Stage::Format(cursor) = &mut self.stage else {
            panic!("a pending digit write belongs to a number")
        };
        cursor
            .complete(reply)
            .expect("a pending digit write accepts its own record");
        self.digits = next;
    }
    pub(crate) fn emitted_unit(&mut self, unit: u8) {
        assert_eq!(
            self.action(),
            SerializationAction::Emit { unit },
            "an emission is acknowledged only for its own pending byte"
        );
        let next = self
            .emitted
            .checked_add(1)
            .expect("serialized output length fits in usize");
        match &mut self.stage {
            Stage::Format(cursor) => cursor
                .complete(FormatReply::Emitted { unit })
                .expect("a pending emission accepts its own byte"),
            Stage::Pieces => self.offset += 1,
            _ => panic!("an emission belongs to a piece or a number"),
        }
        self.emitted = next;
    }
}
