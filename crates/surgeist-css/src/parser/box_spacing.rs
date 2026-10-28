//! Authored margin and padding from Box 3 and Logical Properties 1.

use super::property_input::Stream;
use super::side_shapes::{parse_four_sides, parse_side_pair};
use crate::diagnostic::error::property::{Error, SyntaxError};

use super::values::{parse_length_percentage, parse_nonnegative_length_percentage};
use crate::numeric::expression::NumericInputContext;
use crate::values::box_model::{CssMarginValue, CssPaddingValue};
use crate::values::common::{CssFourSides, CssSidePair};

pub(super) fn parse_box_margin_value<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMarginValue, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssMarginValue::Auto);
    }
    parse_length_percentage(input, numeric).map(CssMarginValue::LengthPercentage)
}

pub(super) fn parse_box_padding_value<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingValue, SyntaxError<'i, Error>> {
    parse_nonnegative_length_percentage(input, numeric).map(CssPaddingValue::new)
}

pub(super) fn parse_box_margin_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssMarginValue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_box_margin_value(input, numeric))
}

pub(super) fn parse_box_padding_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssPaddingValue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_box_padding_value(input, numeric))
}

pub(super) fn parse_box_margin_shorthand<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssMarginValue>, SyntaxError<'i, Error>> {
    parse_four_sides(input, |input| parse_box_margin_value(input, numeric))
}

pub(super) fn parse_box_padding_shorthand<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssPaddingValue>, SyntaxError<'i, Error>> {
    parse_four_sides(input, |input| parse_box_padding_value(input, numeric))
}
