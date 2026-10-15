//! Checked border-corner radii and physical border-radius shorthand.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::Token;

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::numeric::NumericInputContext;
use crate::{
    CssBorderRadiusShorthand, CssCornerRadiusValue, CssLengthPercentageCalculation,
    CssSpecifiedNonNegativeLengthPercentage,
};

fn parse_radius_scalar<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSpecifiedNonNegativeLengthPercentage, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_site = input.position();
    let checked = match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = input
                .collect_numeric_component(numeric)
                .map_err(|_| unexpected_at(location))?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    checked.map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, root_site))
    })
}

pub(super) fn parse_exact_corner_radius<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssCornerRadiusValue, ParseError<'i, Error>> {
    let horizontal = parse_radius_scalar(input, numeric)?;
    let vertical = if input.is_exhausted() {
        None
    } else {
        Some(parse_radius_scalar(input, numeric)?)
    };
    Ok(CssCornerRadiusValue::new(horizontal, vertical))
}

fn parse_radius_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<Vec<CssSpecifiedNonNegativeLengthPercentage>, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() {
        let state = input.state();
        if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            input.reset(&state);
            break;
        }
        values.push(parse_radius_scalar(input, numeric)?);
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
