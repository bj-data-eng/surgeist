//! Sizing 4 authored contained intrinsic-size declarations.

use cssparser::{ParseError, Parser, Token};

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssContainIntrinsicSize, CssContainIntrinsicSizeFallback, CssContainIntrinsicSizeValue,
    CssLengthCalculation, CssSpecifiedNonNegativeLength,
};

pub(super) fn parse_contain_intrinsic_size_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssContainIntrinsicSizeValue, ParseError<'i, Error>> {
    let auto = input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok();
    let fallback = if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        CssContainIntrinsicSizeFallback::None
    } else {
        CssContainIntrinsicSizeFallback::Length(parse_nonnegative_length(input, numeric)?)
    };
    Ok(if auto {
        CssContainIntrinsicSizeValue::with_auto(fallback)
    } else {
        CssContainIntrinsicSizeValue::new(fallback)
    })
}

pub(super) fn parse_contain_intrinsic_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssContainIntrinsicSize, ParseError<'i, Error>> {
    let width = parse_contain_intrinsic_size_value(input, numeric)?;
    let height = if input.is_exhausted() {
        None
    } else {
        Some(parse_contain_intrinsic_size_value(input, numeric)?)
    };
    Ok(CssContainIntrinsicSize::new(width, height))
}

fn parse_nonnegative_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSpecifiedNonNegativeLength, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid contained intrinsic-size length")
            })?;
            CssSpecifiedNonNegativeLength::try_from_component(component).map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "contained intrinsic size requires a nonnegative length",
                )
            })
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::Length)?;
            CssSpecifiedNonNegativeLength::try_from_calculation(
                CssLengthCalculation::from_expression(expression),
            )
            .map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "invalid contained intrinsic-size length math",
                )
            })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}
