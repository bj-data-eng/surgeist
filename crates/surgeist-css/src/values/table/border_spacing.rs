//! Bounded specified serialization for checked CSS2 table border spacing.

use crate::numeric::{CssLength, CssNonNegative, CssNumeric};
use crate::specified_inverse::InverseValueEquality as _;
use crate::values::common::common_values::{side_element, side_pair_entry};

/// One `border-spacing` length; the schema names the generic pair through this private name.
pub(crate) type BorderSpacingLength = CssNumeric<CssLength, CssNonNegative>;

// A single authored length leaves the second value unauthored (it is the first); two equal
// authored lengths are written as one, the shorter form with the same meaning.
side_element!(
    BorderSpacingLength,
    same = |a, b, context| a.inverse_equal(b, context)
);
side_pair_entry!(BorderSpacingLength);
