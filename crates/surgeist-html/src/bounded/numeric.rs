//! HTML numerical operations with caller-owned admission and scratch.
//!
//! Materializers report every observed allocation capacity on either outcome.
//! Work bounds describe atomic scans; cursors expose fixed steps. These APIs
//! neither allocate a controller nor certify that a caller admitted the work.
//!
//! The range cursors [`HtmlFiniteDecimalIngressCursor`] and [`HtmlBestFiniteNumberCursor`] cost a
//! caller a bounded number of requests per number, one for each source unit read, scratch cell
//! read, write, append or pop, scalar step and output unit. The count grows roughly with the square
//! of the value's binary exponent, and a source digit costs a few requests. Ingress costs 16
//! requests for `"1"`, about 400,000 for `"1e100"`, 3.4 million for `"1.7976931348623157e308"` and
//! 8.4 million for `"1e-320"`. Best costs 12,000 for `1.0`, 1.8 million for `1e300` and 4.9
//! million for `5e-324`. A caller sets its work admission for these cursors on that scale. The
//! measured table is in `docs/reference/html-operations.md`.

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
