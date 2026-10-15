//! Authored positioned offsets from Position 3 and Logical Properties 1.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::Token;

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::numeric::NumericInputContext;
use crate::{
    CssBoxSideKind, CssInsetPair, CssInsetShorthand, CssInsetValue, CssLengthPercentageCalculation,
    CssSpecifiedLengthPercentage,
};

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
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_site = input.position();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = input
                .collect_numeric_component(numeric)
                .map_err(|_| unexpected_at(location))?;
            CssSpecifiedLengthPercentage::from_property_component(component, numeric)
                .map(CssInsetValue::LengthPercentage)
                .map_err(|error| {
                    unexpected_at(numeric.property_error_location(&error, location, root_site))
                })
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
            .map(CssInsetValue::LengthPercentage)
            .map_err(|error| {
                unexpected_at(numeric.property_error_location(&error, location, root_site))
            })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_inset_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInsetPair, ParseError<'i, Error>> {
    let start = parse_inset_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_inset_value(input, numeric)?)
    };
    Ok(CssInsetPair::new(start, end))
}

pub(super) fn parse_inset_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssInsetShorthand, ParseError<'i, Error>> {
    let kind = if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    };
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_inset_value(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssInsetShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
