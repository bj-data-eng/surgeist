//! HTML Email Input type mismatch over the exact current value.

use super::input::HtmlInputEmailMode;
use crate::internal::microsyntax::tokens::is_ascii_whitespace;

// HTML877d's label production uses the RFC 1034 label-length limit.
const MAX_LABEL_LENGTH: u8 = 63;

#[derive(Clone, Copy)]
enum Address {
    Start,
    LocalPart,
    Label {
        length: u8,
        ends_with_alphanumeric: bool,
    },
    TrailingWhitespace,
}

impl Address {
    const fn start_label() -> Self {
        Self::Label {
            length: 0,
            ends_with_alphanumeric: false,
        }
    }

    const fn is_complete(self) -> bool {
        matches!(
            self,
            Self::Label {
                ends_with_alphanumeric: true,
                ..
            } | Self::TrailingWhitespace
        )
    }

    fn advance(self, unit: u16, multiple: bool) -> Option<Self> {
        let ascii = u8::try_from(unit).ok()?;
        if multiple {
            if ascii == b',' {
                return self.is_complete().then_some(Self::Start);
            }
            if is_ascii_whitespace(unit) {
                return match self {
                    Self::Start => Some(Self::Start),
                    complete if complete.is_complete() => Some(Self::TrailingWhitespace),
                    _ => None,
                };
            }
        }
        match self {
            Self::Start | Self::LocalPart if is_local_part_character(ascii) => {
                Some(Self::LocalPart)
            }
            Self::LocalPart if ascii == b'@' => Some(Self::start_label()),
            Self::Label { length, .. }
                if ascii.is_ascii_alphanumeric() && length < MAX_LABEL_LENGTH =>
            {
                Some(Self::Label {
                    length: length + 1,
                    ends_with_alphanumeric: true,
                })
            }
            Self::Label { length, .. }
                if ascii == b'-' && (1..MAX_LABEL_LENGTH).contains(&length) =>
            {
                Some(Self::Label {
                    length: length + 1,
                    ends_with_alphanumeric: false,
                })
            }
            Self::Label {
                ends_with_alphanumeric: true,
                ..
            } if ascii == b'.' => Some(Self::start_label()),
            _ => None,
        }
    }
}

fn is_local_part_character(ascii: u8) -> bool {
    ascii.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_\x60{|}~".contains(&ascii)
}

/// Evaluate the intrinsic Email type-mismatch flag for the current UTF-16 value.
///
/// The empty string has no mismatch. In multiple mode each comma separates two
/// valid addresses, with optional surrounding ASCII whitespace for each address.
/// The last delimiter still requires a following address. The exact current
/// value may differ from the string originally supplied to its sanitizer.
///
/// Calls `step` before each inspected code unit. The first refusal is returned
/// unchanged. A decisive mismatch needs no further input reads.
pub fn input_email_type_mismatch_with<E>(
    value: &[u16],
    mode: HtmlInputEmailMode,
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    if value.is_empty() {
        return Ok(false);
    }
    let multiple = mode == HtmlInputEmailMode::Multiple;
    let mut address = Address::Start;
    for unit in value {
        step()?;
        let Some(next) = address.advance(*unit, multiple) else {
            return Ok(true);
        };
        address = next;
    }
    Ok(!address.is_complete())
}
