//! Checked border-corner radii and physical border-radius shorthand.

use super::property_input::Parser;
use crate::diagnostic::error::property::{Error, ParseError, unexpected_at};

use super::side_shapes::parse_side_pair;
use super::values::parse_nonnegative_length_percentage;
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::values::border::CssBorderRadiusShorthand;
use crate::values::border::border_radius::CornerRadiusLength;
use crate::values::common::CssSidePair;

pub(super) fn parse_exact_corner_radius<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CornerRadiusLength>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| {
        parse_nonnegative_length_percentage(input, numeric)
    })
}

fn parse_radius_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<Vec<CssNumeric<CssLengthPercentage, CssNonNegative>>, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() {
        let state = input.state();
        if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            input.reset(&state);
            break;
        }
        values.push(parse_nonnegative_length_percentage(input, numeric)?);
        if values.len() == 4 && !input.is_exhausted() {
            let state = input.state();
            let slash_is_next = input.try_parse(|input| input.expect_delim('/')).is_ok();
            input.reset(&state);
            if !slash_is_next {
                return Err(unexpected_at(input.current_source_location()));
            }
        }
    }
    Ok(values)
}

pub(super) fn parse_exact_border_radius<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBorderRadiusShorthand, ParseError<'i, Error>> {
    let horizontal = parse_radius_list(input, numeric)?;
    if horizontal.is_empty() {
        return Err(unexpected_at(input.current_source_location()));
    }
    let vertical = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        let values = parse_radius_list(input, numeric)?;
        if values.is_empty() {
            return Err(unexpected_at(input.current_source_location()));
        }
        Some(values)
    } else {
        None
    };
    CssBorderRadiusShorthand::try_new(horizontal, vertical)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
