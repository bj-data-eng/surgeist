//! Authored positioned offsets from Position 3 and Logical Properties 1.

use super::property_input::Parser;
use super::side_shapes::{parse_four_sides, parse_side_pair};
use crate::error::property::{Error, ParseError};

use super::values::parse_length_percentage;
use crate::numeric::NumericInputContext;
use crate::{CssFourSides, CssInsetValue, CssSidePair};

pub(super) fn parse_inset_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInsetValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssInsetValue::Auto);
    }
    parse_length_percentage(input, numeric).map(CssInsetValue::LengthPercentage)
}

pub(super) fn parse_inset_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssInsetValue>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| parse_inset_value(input, numeric))
}

pub(super) fn parse_inset_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssInsetValue>, ParseError<'i, Error>> {
    parse_four_sides(input, |input| parse_inset_value(input, numeric))
}
