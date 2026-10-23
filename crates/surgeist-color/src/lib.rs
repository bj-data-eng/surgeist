//! Dependency-free finite numerical color values and controlled operations.
//!
//! Every numerical operation is total over proof-typed operands. [`Finite`]
//! and [`UnitInterval`] reject NaN and both infinities when they are built;
//! a result beyond binary64 is the signed closest finite value.
//!
//! ```
//! use surgeist_color::{Color, ColorSpace, Finite, PredefinedColorSpace, UnitInterval};
//! let srgb = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
//! let red = Color::new(srgb, [Finite::new(f64::MAX), Finite::new(0.0), None], UnitInterval::new(1.0));
//! let xyz = red.converted(ColorSpace::Predefined(PredefinedColorSpace::XyzD65));
//! assert_eq!(xyz.channels()[0], Some(f64::MAX));
//! ```
//!
//! A proof has no public constructor other than its checked `new`:
//!
//! ```compile_fail,E0423
//! let finite = surgeist_color::Finite(1.0);
//! ```
//!
//! ```compile_fail,E0423
//! let unit = surgeist_color::UnitInterval(2.0);
//! ```
//!
//! A color takes proven channels, never raw `f64` values:
//!
//! ```compile_fail,E0308
//! use surgeist_color::{Color, ColorSpace, UnitInterval};
//! let color = Color::new(ColorSpace::Lab, [Some(50.0), None, None], UnitInterval::new(1.0));
//! ```
//!
//! The fallible raw constructor no longer exists:
//!
//! ```compile_fail,E0599
//! use surgeist_color::{Color, ColorSpace};
//! let color = Color::try_new(ColorSpace::Lab, [Some(50.0), None, None], Some(1.0));
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
mod finite;
pub use finite::{Finite, UnitInterval};
mod spaces;
pub use spaces::*;
mod value;
pub use value::{Color, SerializationFamily, Transformation};
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
pub use rectangular_color_conversion::{RectangularCoordinates, RectangularSpace};
mod hsl_color_conversion;
pub use hsl_color_conversion::{
    HslCoordinates, HwbCoordinates, convert_hsl_to_srgb, convert_hwb_to_srgb, convert_srgb_to_hsl,
    convert_srgb_to_hwb,
};
mod lab_color_conversion;
pub use lab_color_conversion::{
    PolarCoordinates, convert_lab_to_lch, convert_lch_to_lab, convert_oklab_to_oklch,
    convert_oklch_to_oklab,
};
mod naive_color_conversion;
pub use naive_color_conversion::{naively_convert_cmyk_to_srgba, naively_convert_srgba_to_cmyk};
