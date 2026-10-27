//! Images, gradients and one-dimensional stripes.
//!
//! # Backgrounds, border images, and gradients
//!
//! Background and image values preserve authored layer, image, gradient, stop,
//! border-image, and object-sizing structure. Selected Images 4 additions admit
//! conic gradients, explicit shared [`CssColorInterpolation`](crate::color::CssColorInterpolation), singleton lists,
//! and two authored positions per stop. [`CssGradientColorStop::positions`](crate::values::image::CssGradientColorStop::positions)
//! retains linear positions; [`CssAngularColorStopPosition`] retains conic angle,
//! percentage, exact zero, and [`CssCalculation<CssAnglePercentage>`](crate::numeric::CssCalculation) branches.
//! Radial and conic `at` fields use the full symbolic [`CssPosition`](crate::values::position::CssPosition) owner.
//! They do not resolve URLs, load or
//! decode images, compute geometry, or paint.
//!
//! ```
//! use surgeist_css::values::image::{CssGradient, CssImageValue};
//! use surgeist_css::property::CssKnownPropertyValueRef;
//! use surgeist_css::conformance::{CssSupportStatus, feature_metadata};
//! use surgeist_css::CssParser;
//!
//! let report = CssParser::default().parse_block_contents(concat!(
//!     "background-image: linear-gradient(to right, red 0%, 40%, blue); ",
//!     "border-image: url(frame.png) 10 fill / 2 / 1 round",
//! ));
//! assert!(report.is_clean(), "{:?}", report.diagnostics());
//!
//! let CssKnownPropertyValueRef::BackgroundImage(images) = report.syntax()[0]
//!     .known().expect("known background image")
//!     .property_value().expect("ordinary background image")
//! else { panic!("expected background-image") };
//! assert!(matches!(
//!     images.value().images(),
//!     [CssImageValue::Gradient(CssGradient::Linear(_))]
//! ));
//!
//! let CssKnownPropertyValueRef::BorderImage(border) = report.syntax()[1]
//!     .known().expect("known border image")
//!     .property_value().expect("ordinary border image")
//! else { panic!("expected border-image") };
//! assert!(border.value().slice().expect("slice").fill());
//! assert_eq!(
//!     border.value().serialize_specified(Default::default()).unwrap(),
//!     "url(\"frame.png\") 10 fill / 2 / 1 round",
//! );
//!
//! let gradient = feature_metadata("official.value.linear-gradient")
//!     .expect("linear-gradient metadata");
//! assert_eq!(gradient.source().id().as_str(), "O-IMAGES3");
//! assert_eq!(gradient.status(), CssSupportStatus::Complete);
//! ```
//!
//! The Images and Backgrounds/Borders property and value records, including
//! `background-position`, `object-position`, and `box-shadow`, are Complete
//! catalog entries.

pub(crate) mod gradients;
pub(crate) mod image_1d;
pub(crate) mod image_orientation_serialization;
pub(crate) mod image_serialization;
pub(crate) mod images;
pub(crate) mod syntax;

#[doc(inline)]
pub use image_1d::{
    CssImage1D, CssStripe, CssStripeLengthPercentage, CssStripeThickness, CssStripes,
};
#[doc(inline)]
pub use images::{CssImage, CssImageConstructionError, CssLightDarkImage};
#[doc(inline)]
pub use syntax::{
    CssAngularColorStop, CssAngularColorStopList, CssAngularColorStopListItem,
    CssAngularColorStopPosition, CssColorStopList, CssColorStopListItem, CssConicGradient,
    CssGradient, CssGradientColorStop, CssHorizontalGradientSide, CssImageOrientation,
    CssImageRendering, CssImageValue, CssImageValueList, CssLinearGradient,
    CssLinearGradientDirection, CssObjectFit, CssRadialEllipseSize, CssRadialExtent,
    CssRadialGradient, CssRadialShape, CssRadialSize, CssSideOrCorner, CssVerticalGradientSide,
};
