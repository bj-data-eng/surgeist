//! HTML numerical operations with caller-owned admission and scratch.
//!
//! Materializers report every observed allocation capacity on either outcome.
//! Work bounds describe atomic scans; cursors expose fixed steps. These APIs
//! neither allocate a controller nor certify that a caller admitted the work.

pub use crate::numeric::natural_ratio::{
    NaturalArithmeticError, NaturalComparison, NaturalDifference, NaturalDigits, NaturalProduct,
    RationalIntegerMaterializationFailure,
};
pub use crate::numeric::range::best::BestFiniteNumberCursor;
pub use crate::numeric::range::ingress::FiniteDecimalIngressCursor;
pub use crate::numeric::range::lattice::{RangeLatticeCursor, RangeMidpointCursor};
pub use crate::numeric::range::{
    RangeBufferHeader, RangeCell, RangeDecimalParts, RangeNumericError, RangeNumericHeaders,
    RangeNumericLayout, RangeNumericReply, RangeNumericRequest, RangeScratchRole,
};
pub use crate::numeric::{
    FloatingPointAllocationError, FloatingPointGrammarCursor, FloatingPointParseMode,
    FloatingPointParseResult, IntegerPrefixScanner, IntegerRecoveryError, IntegerUnitStep,
    IntegerUnitStepError, IntegerUnitStepResult, NumericMaterializationError,
    NumericMaterializationFailure, NumericMaterialized, NumericParseError, ScannedIntegerPrefix,
    integer_long_work_bound, parse_integer_long, parse_nonnegative_integer_capped,
    try_floating_point_number, try_parse_integer,
};
