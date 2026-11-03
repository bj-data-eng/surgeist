//! Intrinsic numeric values from the pinned HTML common microsyntaxes.

pub use crate::internal::microsyntax::numeric::natural_ratio::{
    HtmlNonNegativeRational, HtmlRationalError,
};
pub use crate::internal::microsyntax::numeric::{
    HtmlDecimalDigitsError, HtmlDimension, HtmlDimensionList, HtmlFloatingPointList,
    HtmlFloatingPointNumber, HtmlInteger, HtmlNegativeIntegerError, HtmlNonFiniteNumberError,
    HtmlNonNegativeDecimal, HtmlNonNegativeInteger, HtmlNumericRangeError, HtmlSignedDecimal,
};
