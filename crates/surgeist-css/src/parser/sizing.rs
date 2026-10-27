//! Sizing 3 and Sizing 4 authored property grammars.

use super::property_input::Parser;
use crate::diagnostic::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::Token;

use super::values::parse_nonnegative_length_percentage;
use crate::component::CssComponentValueLimits;
use crate::numeric::expression::NumericInputContext;
use crate::values::box_model::{
    CssBoxCalcSize, CssBoxSize, CssCalcSize, CssMaxSizeValue, CssSizeValue,
};

pub(crate) fn parse_size_value<'i, 't>(
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

pub(crate) fn parse_max_size_value<'i, 't>(
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
    let root_site = input.position();
    let value = parse_box_size(input, numeric)?;
    CssMaxSizeValue::try_box_size(value).map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, root_site))
    })
}

fn parse_box_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBoxSize, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_site = input.position();
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
                let value = parse_nonnegative_length_percentage(input, numeric.ordinary())?;
                input.expect_exhausted().map_err(basic)?;
                Ok(value)
            })?;
            Ok(CssBoxSize::FitContentFunction(value))
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc-size") => {
            input.reset(&state);
            let component = input
                .collect_numeric_component(numeric)
                .map_err(|_| unexpected_at(location))?;
            let recovered = numeric.is_source_input();
            let value = CssCalcSize::from_component_with_policy(
                component,
                CssComponentValueLimits::default(),
                recovered,
            )
            .map_err(|error| {
                unexpected_at(numeric.property_error_location(&error, location, root_site))
            })?;
            CssBoxCalcSize::try_from(value)
                .map(CssBoxSize::CalcSize)
                .map_err(|error| {
                    unexpected_at(numeric.property_error_location(&error, location, root_site))
                })
        }
        _ => {
            input.reset(&state);
            parse_nonnegative_length_percentage(input, numeric).map(CssBoxSize::LengthPercentage)
        }
    }
}
