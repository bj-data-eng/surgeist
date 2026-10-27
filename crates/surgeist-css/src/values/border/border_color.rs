//! Authored border colors, shared by the physical and flow-relative border-color families.

use crate::color::CssColor;
use crate::values::common::common_values::{four_sides_entry, side_element, side_pair_entry};

side_element!(
    CssColor,
    same = |a, b, context| a.specified_inverse_eq(b, context)
);
four_sides_entry!(CssColor);
side_pair_entry!(CssColor);
