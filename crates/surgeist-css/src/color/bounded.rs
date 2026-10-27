//! Public typestate continuation over one canonical CSS color parse.
//!
//! The consumer owns input, all three real storage banks and its controller.
//! Each step yields exactly one typed action whose single completion returns
//! the parse. Nothing allocates, releases capacity, executes abandoned work or
//! publishes partial output here.

use super::CssColorFactIdentity;
use super::operation as core;

/// Encoding of the original immutable input. UTF8 reads are bytes, not chars.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorInputKind {
    /// The input is a sequence of bytes holding UTF-8 text; offsets and lengths count bytes.
    Utf8,
    /// The input is a sequence of UTF-16 code units; offsets and lengths count code units.
    Utf16,
}
impl CssColorInputKind {
    pub(crate) fn native(self) -> crate::tokenization::bounded::InputKind {
        match self {
            Self::Utf8 => crate::tokenization::bounded::InputKind::Utf8,
            Self::Utf16 => crate::tokenization::bounded::InputKind::Utf16,
        }
    }
    pub(crate) fn from_native(kind: crate::tokenization::bounded::InputKind) -> Self {
        match kind {
            crate::tokenization::bounded::InputKind::Utf8 => Self::Utf8,
            crate::tokenization::bounded::InputKind::Utf16 => Self::Utf16,
        }
    }
}
/// Whether the actual parent requires one color or the entire declared input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorGrammarRequest {
    /// One color is parsed from the start of the input; the consumed range may end before the input does.
    PrefixColor,
    /// The entire declared input must be exactly one color.
    WholeColor,
}
impl CssColorGrammarRequest {
    pub(crate) fn native(self) -> core::GrammarRequest {
        match self {
            Self::PrefixColor => core::GrammarRequest::PrefixColor,
            Self::WholeColor => core::GrammarRequest::WholeColor,
        }
    }
}
/// Original unit coordinates retained by the producer, never decoded offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorInputRange {
    /// Offset of the first original input unit in the range.
    pub start: usize,
    /// Offset one past the last original input unit in the range.
    pub end: usize,
}
/// A reached unavailable fact differs from a standard-invalid color.
///
/// The fact names what the consumer's envelope did not supply; it is read through
/// [`CssColorError::unavailable`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorUnavailable {
    /// `currentcolor` was reached without a current color.
    CurrentColor,
    /// A system color was reached without its palette entry.
    Palette(crate::color::CssSystemColor),
    /// `light-dark()` was reached without a used color scheme.
    Scheme,
    /// A custom color profile was reached without its binding.
    Profile,
    /// A profile conversion was reached without its conversion fact.
    ProfileConversion,
    /// `device-cmyk()` was reached without a device fact.
    Device,
}
/// Reached capabilities not representable by the concrete fact carrier.
///
/// Read through [`CssColorError::unsupported`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorUnsupported {
    /// The supplied profile conversion cannot be carried out.
    ProfileConversion,
    /// A conversion back into a custom profile is unsupported.
    ReverseProfile,
    /// Interpolation in a custom color space is unsupported.
    CustomInterpolation,
    /// A calculation needs the context of this contextual unit.
    ContextualUnit(crate::numeric::CssNumericUnit),
}
/// A defect of the supplied fact envelope; it never mints a standard-failure fallback.
///
/// This is a fact about the consumer's input, not an error type of its own; it is
/// read through [`CssColorError::fact_defect`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorFactDefect {
    /// The facts are not the envelope the parse was started with.
    Foreign,
    /// The palette supplies this system color more than once.
    DuplicatePalette(crate::color::CssSystemColor),
    /// Two profile bindings, by ordinal, share one profile name.
    DuplicateProfile {
        /// The ordinal of the first binding.
        first: usize,
        /// The ordinal of the repeating binding.
        second: usize,
    },
    /// Profile metadata is malformed.
    InvalidProfileMetadata,
    /// One profile conversion is supplied more than once.
    DuplicateConversion,
    /// The device facts are malformed or do not match the conversion.
    InvalidDevice,
}
/// A reached semantic failure of one color parse.
///
/// Resource refusal and cancellation stay with the consumer's controller and
/// are not reconstructed as this error. Colour numerics are total, so no
/// numerical failure exists, and protocol misuse is static or a documented
/// panic, so no protocol failure exists either. Exactly one accessor returns
/// a value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorError {
    kind: ColorErrorKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorErrorKind {
    Unavailable(CssColorUnavailable),
    Unsupported(CssColorUnsupported),
    Facts(CssColorFactDefect),
    MalformedInput { offset: usize },
}
impl From<ColorErrorKind> for CssColorError {
    fn from(kind: ColorErrorKind) -> Self {
        Self { kind }
    }
}
impl CssColorError {
    /// Returns the fact the envelope did not supply.
    #[must_use]
    pub const fn unavailable(&self) -> Option<CssColorUnavailable> {
        match self.kind {
            ColorErrorKind::Unavailable(fact) => Some(fact),
            _ => None,
        }
    }
    /// Returns the reached capability the fact carrier cannot represent.
    #[must_use]
    pub const fn unsupported(&self) -> Option<CssColorUnsupported> {
        match self.kind {
            ColorErrorKind::Unsupported(capability) => Some(capability),
            _ => None,
        }
    }
    /// Returns the defect of the supplied fact envelope.
    #[must_use]
    pub const fn fact_defect(&self) -> Option<CssColorFactDefect> {
        match self.kind {
            ColorErrorKind::Facts(defect) => Some(defect),
            _ => None,
        }
    }
    /// Returns the original unit offset that proved the input malformed for its
    /// declared encoding, or the input length when the input ends inside a
    /// multi-unit sequence.
    #[must_use]
    pub const fn malformed_input_offset(&self) -> Option<usize> {
        match self.kind {
            ColorErrorKind::MalformedInput { offset } => Some(offset),
            _ => None,
        }
    }
}
impl std::fmt::Display for CssColorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::color::syntax::serialization::authored_system_name as system;
        match self.kind {
            ColorErrorKind::Unavailable(fact) => {
                f.write_str("the reached color needs ")?;
                match fact {
                    CssColorUnavailable::CurrentColor => f.write_str("the current color"),
                    CssColorUnavailable::Palette(keyword) => {
                        write!(f, "the system color {}", system(keyword))
                    }
                    CssColorUnavailable::Scheme => f.write_str("a used color scheme"),
                    CssColorUnavailable::Profile => f.write_str("a color profile binding"),
                    CssColorUnavailable::ProfileConversion => {
                        f.write_str("a color profile conversion")
                    }
                    CssColorUnavailable::Device => f.write_str("a device color fact"),
                }?;
                f.write_str(", which the facts do not supply")
            }
            ColorErrorKind::Unsupported(capability) => match capability {
                CssColorUnsupported::ProfileConversion => {
                    f.write_str("the supplied color profile conversion is unsupported")
                }
                CssColorUnsupported::ReverseProfile => {
                    f.write_str("conversion into a custom color profile is unsupported")
                }
                CssColorUnsupported::CustomInterpolation => {
                    f.write_str("interpolation in a custom color space is unsupported")
                }
                CssColorUnsupported::ContextualUnit(unit) => write!(
                    f,
                    "a color calculation needs the context of the unit `{}`",
                    crate::numeric::units::suffix(unit)
                ),
            },
            ColorErrorKind::Facts(defect) => {
                f.write_str("invalid color fact envelope: ")?;
                match defect {
                    CssColorFactDefect::Foreign => {
                        f.write_str("the facts are not the envelope the parse started with")
                    }
                    CssColorFactDefect::DuplicatePalette(keyword) => {
                        write!(f, "the system color {} is supplied twice", system(keyword))
                    }
                    CssColorFactDefect::DuplicateProfile { first, second } => {
                        write!(f, "profile bindings {first} and {second} share a name")
                    }
                    CssColorFactDefect::InvalidProfileMetadata => {
                        f.write_str("the profile metadata is malformed")
                    }
                    CssColorFactDefect::DuplicateConversion => {
                        f.write_str("a profile conversion is supplied twice")
                    }
                    CssColorFactDefect::InvalidDevice => {
                        f.write_str("the device facts are malformed")
                    }
                }
            }
            ColorErrorKind::MalformedInput { offset } => write!(
                f,
                "color input is malformed for its declared encoding at unit {offset}"
            ),
        }
    }
}
impl std::error::Error for CssColorError {}
/// The canonical reason and original input range for standard failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorStandardFailureKind {
    /// The input is not a valid `<color>`; no color function or other color syntax matched.
    ColorGrammar,
    /// A color function's component, hue or relative channel is not valid at its position.
    ComponentGrammar,
    /// A calculation inside a color component is invalid, for example through a malformed expression, a wrong argument count or type, or incompatible types.
    NumericGrammar,
    /// A custom color profile name or component reference is invalid, or its conversion rejected the color.
    InvalidProfile,
    /// Resolution found the components invalid, for example a channel outside its grammar or an invalid `color-mix()` weight.
    InvalidComponents,
}
/// The standard-invalid outcome of one color parse: the failure reason and where in the input it was found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorStandardFailure {
    /// The canonical reason the input is standard-invalid.
    pub kind: CssColorStandardFailureKind,
    /// The original input range at which the failure was found.
    pub source: CssColorInputRange,
}

/// The fixed original input metadata. It never proves an HTML/DOM issuer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorSourceDescriptor {
    /// Encoding of the original input.
    pub kind: CssColorInputKind,
    /// Length of the original input, in units of its encoding.
    pub length: usize,
    /// Whether one color or the entire input was requested.
    pub request: CssColorGrammarRequest,
    /// Original input range consumed by the color. It starts at 0 and, for a whole-color request or a standard failure, ends at the input length.
    pub consumed: CssColorInputRange,
    /// Identity and revision of the fact set the parse expected.
    pub facts: CssColorFactIdentity,
}
pub(crate) fn source(context: &core::CheckpointContext<'_>) -> CssColorSourceDescriptor {
    CssColorSourceDescriptor {
        kind: CssColorInputKind::from_native(context.input_kind),
        length: context.input_length,
        request: match context.request {
            core::GrammarRequest::PrefixColor => CssColorGrammarRequest::PrefixColor,
            core::GrammarRequest::WholeColor => CssColorGrammarRequest::WholeColor,
        },
        consumed: CssColorInputRange {
            start: context.consumed.start,
            end: context.consumed.end,
        },
        facts: context.expected,
    }
}
