//! Flexbox 1 authored factor, basis, and shorthand grammar.

use cssparser::{ParseError, Parser, Token};

use super::sizing::parse_size_value;
use super::values::{CalculationRoot, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssCalcSize, CssComponentValueLimits, CssFlexBasisValue, CssFlexComponents, CssFlexValue,
    CssNumberCalculation, CssSpecifiedNonNegativeNumber,
};

pub(super) fn parse_flex_factor<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    context: &str,
) -> Result<CssSpecifiedNonNegativeNumber, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, format!("invalid {context} number"))
            })?;
            CssSpecifiedNonNegativeNumber::try_from_component(component).map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    format!("{context} requires a nonnegative number"),
                )
            })
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::Number)?;
            CssSpecifiedNonNegativeNumber::try_from_calculation(
                CssNumberCalculation::from_expression(expression),
            )
            .map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    format!("invalid {context} math"),
                )
            })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_flex_basis<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFlexBasisValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Ident(value) if value.eq_ignore_ascii_case("content") => {
            Ok(CssFlexBasisValue::content())
        }
        Token::Function(value) if value.eq_ignore_ascii_case("calc-size") => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid flex-basis calc-size component")
            })?;
            CssCalcSize::from_component_with_policy(
                component,
                CssComponentValueLimits::default(),
                matches!(numeric, NumericInputContext::Parsed(_)),
            )
            .map(CssFlexBasisValue::from)
            .map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "invalid flex-basis calc-size",
                )
            })
        }
        _ => {
            input.reset(&state);
            parse_size_value(input, numeric).map(CssFlexBasisValue::from)
        }
    }
}

pub(super) fn parse_flex<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFlexValue, ParseError<'i, Error>> {
    let start = input.state();
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFlexValue::None);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
        && input.is_exhausted()
    {
        return Ok(CssFlexValue::Auto);
    }
    input.reset(&start);

    let grow = input
        .try_parse(|input| parse_flex_factor(input, numeric, "flex-grow"))
        .ok();
    let (shrink, basis) = if grow.is_some() {
        let shrink = if input.is_exhausted() {
            None
        } else {
            input
                .try_parse(|input| parse_flex_factor(input, numeric, "flex-shrink"))
                .ok()
        };
        let basis = if input.is_exhausted() {
            None
        } else {
            Some(parse_flex_basis(input, numeric)?)
        };
        (shrink, basis)
    } else {
        let basis = Some(parse_flex_basis(input, numeric)?);
        let grow = if input.is_exhausted() {
            None
        } else {
            input
                .try_parse(|input| parse_flex_factor(input, numeric, "flex-grow"))
                .ok()
        };
        let shrink = if grow.is_some() && !input.is_exhausted() {
            input
                .try_parse(|input| parse_flex_factor(input, numeric, "flex-shrink"))
                .ok()
        } else {
            None
        };
        return Ok(CssFlexValue::Components(
            CssFlexComponents::try_new(grow, shrink, basis).expect("basis present"),
        ));
    };
    Ok(CssFlexValue::Components(
        CssFlexComponents::try_new(grow, shrink, basis).expect("grow present"),
    ))
}
