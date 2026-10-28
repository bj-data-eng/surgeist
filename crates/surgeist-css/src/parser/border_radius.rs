//! Checked border-corner radii and physical border-radius shorthand.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, unexpected_at};
use crate::parser::syntax_types::SyntaxError;

use super::side_shapes::parse_side_pair;
use super::values::parse_nonnegative_length_percentage;
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::values::border::CssBorderRadiusShorthand;
use crate::values::border::border_radius::CornerRadiusLength;
use crate::values::common::CssSidePair;

pub(super) fn parse_exact_corner_radius<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CornerRadiusLength>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| {
        parse_nonnegative_length_percentage(input, numeric)
    })
}

fn parse_radius_list<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<Vec<CssNumeric<CssLengthPercentage, CssNonNegative>>, SyntaxError<'i, Error>> {
    let mut values = Vec::new();
    while !input.only_whitespace_remains() {
        let state = input.mark();
        if input.attempt(|input| input.consume_delim('/')).is_ok() {
            input.restore(state);
            break;
        }
        values.push(parse_nonnegative_length_percentage(input, numeric)?);
        if values.len() == 4 && !input.only_whitespace_remains() {
            let state = input.mark();
            let slash_is_next = input.attempt(|input| input.consume_delim('/')).is_ok();
            input.restore(state);
            if !slash_is_next {
                return Err(unexpected_at(input.site()));
            }
        }
    }
    Ok(values)
}

pub(super) fn parse_exact_border_radius<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBorderRadiusShorthand, SyntaxError<'i, Error>> {
    let horizontal = parse_radius_list(input, numeric)?;
    if horizontal.is_empty() {
        return Err(unexpected_at(input.site()));
    }
    let vertical = if input.attempt(|input| input.consume_delim('/')).is_ok() {
        let values = parse_radius_list(input, numeric)?;
        if values.is_empty() {
            return Err(unexpected_at(input.site()));
        }
        Some(values)
    } else {
        None
    };
    CssBorderRadiusShorthand::try_new(horizontal, vertical)
        .ok_or_else(|| unexpected_at(input.site()))
}
