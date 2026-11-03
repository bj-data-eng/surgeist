//! HTML numerical operations with caller-owned admission and scratch.
//!
//! Materializers report every observed allocation capacity on either outcome.
//! Work bounds describe atomic scans; cursors expose fixed steps. These APIs
//! neither allocate a controller nor certify that a caller admitted the work.

pub use crate::internal::microsyntax::numeric::natural_ratio::{
    HtmlNaturalArithmeticError, HtmlNaturalComparison, HtmlNaturalDifference, HtmlNaturalDigits,
    HtmlNaturalProduct, HtmlRationalIntegerMaterializationFailure,
};
pub use crate::internal::microsyntax::numeric::range::best::HtmlBestFiniteNumberCursor;
pub use crate::internal::microsyntax::numeric::range::ingress::HtmlFiniteDecimalIngressCursor;
pub use crate::internal::microsyntax::numeric::range::lattice::{
    HtmlRangeLatticeCursor, HtmlRangeMidpointCursor,
};
pub use crate::internal::microsyntax::numeric::range::{
    HtmlRangeBufferHeader, HtmlRangeCell, HtmlRangeDecimalParts, HtmlRangeNumericError,
    HtmlRangeNumericHeaders, HtmlRangeNumericLayout, HtmlRangeNumericReply,
    HtmlRangeNumericRequest, HtmlRangeScratchRole,
};
pub use crate::internal::microsyntax::numeric::{
    HtmlFloatingPointAllocationError, HtmlFloatingPointGrammarCursor, HtmlFloatingPointParseMode,
    HtmlFloatingPointParseResult, HtmlIntegerPrefixScanner, HtmlIntegerRecoveryError,
    HtmlIntegerUnitStep, HtmlIntegerUnitStepError, HtmlIntegerUnitStepResult,
    HtmlNumericMaterializationError, HtmlNumericMaterializationFailure, HtmlNumericMaterialized,
    HtmlNumericParseError, HtmlScannedIntegerPrefix, integer_long_work_bound, parse_integer_long,
    parse_nonnegative_integer_capped, try_floating_point_number, try_parse_integer,
};
