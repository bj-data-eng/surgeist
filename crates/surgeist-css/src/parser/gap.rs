//! Exact Box Alignment gap grammars.

use super::property_input::Parser;
use super::side_shapes::parse_side_pair;
use crate::diagnostic::error::property::{Error, ParseError};

use super::values::parse_nonnegative_length_percentage;
use crate::numeric::expression::NumericInputContext;
use crate::values::common::CssSidePair;
use crate::values::layout::CssGapValue;

pub(super) fn parse_gap_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGapValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssGapValue::Normal);
    }
    parse_nonnegative_length_percentage(input, numeric).map(CssGapValue::LengthPercentage)
}

pub(super) fn parse_gap_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssGapValue>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| parse_gap_value(input, numeric))
}
