//! Dependency-free checked numerical color values and controlled operations.
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
mod spaces;
pub use spaces::*;
mod value;
pub use value::{Color, ColorError, SerializationFamily, Transformation};
mod interpolation;
mod serialization;
pub use serialization::SerializationRequest;
mod protocol;
pub use protocol::*;
mod controlled;
pub use controlled::*;
mod alpha;
pub mod format;
pub use alpha::byte_alpha_for_serialization;
mod rectangular_color_conversion;
pub use rectangular_color_conversion::{
    RectangularConversionError, RectangularCoordinates, RectangularSpace,
};
mod hsl_color_conversion;
pub use hsl_color_conversion::{
    HslCoordinates, HslHwbConversionError, HwbCoordinates, convert_hsl_to_srgb,
    convert_hwb_to_srgb, convert_srgb_to_hsl, convert_srgb_to_hwb,
};
mod lab_color_conversion;
pub use lab_color_conversion::{
    PolarConversionError, PolarCoordinates, convert_lab_to_lch, convert_lch_to_lab,
    convert_oklab_to_oklch, convert_oklch_to_oklab,
};
mod naive_color_conversion;
pub use naive_color_conversion::{
    NaiveConversionError, naively_convert_cmyk_to_srgba, naively_convert_srgba_to_cmyk,
};
