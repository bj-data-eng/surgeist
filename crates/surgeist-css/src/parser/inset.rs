//! Authored positioned offsets from Position 3 and Logical Properties 1.

use super::side_shapes::{parse_four_sides, parse_side_pair};
use super::syntax_types::Stream;
use crate::diagnostic::error::property::Error;
use crate::parser::syntax_types::SyntaxError;

use super::values::parse_length_percentage;
use crate::numeric::expression::NumericInputContext;
use crate::values::box_model::CssInsetValue;
use crate::values::common::{CssFourSides, CssSidePair};

pub(super) fn parse_inset_value<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInsetValue, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssInsetValue::Auto);
    }
    parse_length_percentage(input, numeric).map(CssInsetValue::LengthPercentage)
}

pub(super) fn parse_inset_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssInsetValue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_inset_value(input, numeric))
}

pub(super) fn parse_inset_shorthand<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssInsetValue>, SyntaxError<'i, Error>> {
    parse_four_sides(input, |input| parse_inset_value(input, numeric))
}
