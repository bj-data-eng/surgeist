//! Authored margin and padding from Box 3 and Logical Properties 1.

use cssparser::{ParseError, Parser, Token};

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::error::{Error, basic, unexpected_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssBoxSideKind, CssLengthPercentageCalculation, CssMarginPair, CssMarginShorthand,
    CssMarginValue, CssPaddingPair, CssPaddingShorthand, CssPaddingValue,
    CssSpecifiedLengthPercentage, CssSpecifiedNonNegativeLengthPercentage,
};

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
    parse_signed_length_percentage(input, numeric).map(CssMarginValue::LengthPercentage)
}

pub(super) fn parse_box_padding_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingValue, ParseError<'i, Error>> {
    parse_nonnegative_length_percentage(input, numeric).map(CssPaddingValue::new)
}

fn parse_signed_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSpecifiedLengthPercentage, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unexpected_at(location))?;
            CssSpecifiedLengthPercentage::from_property_component(component, numeric).map_err(
                |error| unexpected_at(numeric.error_location(&error, location, root_offset)),
            )
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
            .map_err(|error| unexpected_at(numeric.error_location(&error, location, root_offset)))
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_nonnegative_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSpecifiedNonNegativeLengthPercentage, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unexpected_at(location))?;
            CssSpecifiedNonNegativeLengthPercentage::from_property_component(component, numeric)
                .map_err(|error| {
                    unexpected_at(numeric.error_location(&error, location, root_offset))
                })
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
            .map_err(|error| unexpected_at(numeric.error_location(&error, location, root_offset)))
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_box_margin_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMarginPair, ParseError<'i, Error>> {
    let start = parse_box_margin_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_box_margin_value(input, numeric)?)
    };
    Ok(CssMarginPair::new(start, end))
}

pub(super) fn parse_box_padding_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingPair, ParseError<'i, Error>> {
    let start = parse_box_padding_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_box_padding_value(input, numeric)?)
    };
    Ok(CssPaddingPair::new(start, end))
}

fn parse_side_kind<'i, 't>(input: &mut Parser<'i, 't>) -> CssBoxSideKind {
    if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    }
}

pub(super) fn parse_box_margin_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMarginShorthand, ParseError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_box_margin_value(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssMarginShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_box_padding_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingShorthand, ParseError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_box_padding_value(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssPaddingShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
