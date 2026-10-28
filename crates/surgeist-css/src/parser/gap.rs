//! Exact Box Alignment gap grammars.

use super::side_shapes::parse_side_pair;
use super::syntax_types::Stream;
use crate::diagnostic::error::property::Error;
use crate::parser::syntax_types::SyntaxError;

use super::values::parse_nonnegative_length_percentage;
use crate::numeric::expression::NumericInputContext;
use crate::values::common::CssSidePair;
use crate::values::layout::CssGapValue;

pub(super) fn parse_gap_value<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGapValue, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    if input
        .attempt(|input| input.consume_keyword("normal"))
        .is_ok()
    {
        return Ok(CssGapValue::Normal);
    }
    parse_nonnegative_length_percentage(input, numeric).map(CssGapValue::LengthPercentage)
}

pub(super) fn parse_gap_shorthand<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssGapValue>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_gap_value(input, numeric))
}
