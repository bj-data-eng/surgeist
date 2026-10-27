//! Images, gradients and one-dimensional stripes.

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
