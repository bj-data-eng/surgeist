//! Authored margin and padding from Box 3 and Logical Properties 1.

use super::property_input::Parser;
use super::side_shapes::{parse_four_sides, parse_side_pair};
use crate::error::property::{Error, ParseError};

use super::values::{parse_length_percentage, parse_nonnegative_length_percentage};
use crate::numeric::NumericInputContext;
use crate::{CssFourSides, CssMarginValue, CssPaddingValue, CssSidePair};

pub(super) fn parse_box_margin_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMarginValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssMarginValue::Auto);
    }
    parse_length_percentage(input, numeric).map(CssMarginValue::LengthPercentage)
}

pub(super) fn parse_box_padding_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingValue, ParseError<'i, Error>> {
    parse_nonnegative_length_percentage(input, numeric).map(CssPaddingValue::new)
}

pub(super) fn parse_box_margin_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssMarginValue>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| parse_box_margin_value(input, numeric))
}

pub(super) fn parse_box_padding_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssPaddingValue>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| parse_box_padding_value(input, numeric))
}

pub(super) fn parse_box_margin_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssMarginValue>, ParseError<'i, Error>> {
    parse_four_sides(input, |input| parse_box_margin_value(input, numeric))
}

pub(super) fn parse_box_padding_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssPaddingValue>, ParseError<'i, Error>> {
    parse_four_sides(input, |input| parse_box_padding_value(input, numeric))
}
