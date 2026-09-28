//! Sizing 3 and Sizing 4 authored property grammars.

use cssparser::{ParseError, Parser, Token};

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssBoxCalcSize, CssBoxSize, CssCalcSize, CssComponentValueLimits,
    CssLengthPercentageCalculation, CssMaxSizeValue, CssSizeValue,
    CssSpecifiedNonNegativeLengthPercentage,
};

pub(super) fn parse_size_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSizeValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssSizeValue::Auto);
    }
    parse_box_size(input, numeric).map(CssSizeValue::BoxSize)
}

pub(super) fn parse_max_size_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMaxSizeValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssMaxSizeValue::NONE);
    }
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let value = parse_box_size(input, numeric)?;
    CssMaxSizeValue::try_box_size(value).map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "maximum sizing excludes auto",
        )
    })
}

fn parse_box_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBoxSize, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("stretch") => Ok(CssBoxSize::Stretch),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("contain") => Ok(CssBoxSize::Contain),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("min-content") => {
            Ok(CssBoxSize::MinContent)
        }
        Token::Ident(ident) if ident.eq_ignore_ascii_case("max-content") => {
            Ok(CssBoxSize::MaxContent)
        }
        Token::Ident(ident) if ident.eq_ignore_ascii_case("fit-content") => {
            Ok(CssBoxSize::FitContent)
        }
        Token::Function(name) if name.eq_ignore_ascii_case("fit-content") => {
            let value = input.parse_nested_block(|input| {
                let value = parse_nonnegative_length_percentage(input, numeric)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(value)
            })?;
            Ok(CssBoxSize::FitContentFunction(value))
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc-size") => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid calc-size component"))?;
            let recovered = matches!(numeric, NumericInputContext::Parsed(_));
            let value = CssCalcSize::from_component_with_policy(
                component,
                CssComponentValueLimits::default(),
                recovered,
            )
            .map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "invalid calc-size basis or calculation",
                )
            })?;
            CssBoxCalcSize::try_from(value)
                .map(CssBoxSize::CalcSize)
                .map_err(|error| {
                    unsupported_value_at(
                        numeric.error_location(&error, location, root_offset),
                        None,
                        "calc-size basis is invalid for box sizing",
                    )
                })
        }
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid sizing component"))?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
                .map(CssBoxSize::LengthPercentage)
                .map_err(|error| {
                    unsupported_value_at(
                        numeric.error_location(&error, location, root_offset),
                        None,
                        "sizing requires a nonnegative length-percentage",
                    )
                })
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
            .map(CssBoxSize::LengthPercentage)
            .map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "invalid sizing math",
                )
            })
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
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid fit-content argument")
            })?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component).map_err(
                |error| {
                    unsupported_value_at(
                        numeric.error_location(&error, location, root_offset),
                        None,
                        "fit-content requires a nonnegative length-percentage",
                    )
                },
            )
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
            .map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "invalid fit-content math",
                )
            })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}
