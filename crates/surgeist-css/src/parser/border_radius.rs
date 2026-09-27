//! Checked border-corner radii and physical border-radius shorthand.

use cssparser::{ParseError, Parser, Token};

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
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
    let root_offset = input.position().byte_index();
    let checked = match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid border radius"))?;
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
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "border radius requires a nonnegative length-percentage",
        )
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
                return Err(unsupported_value(
                    input,
                    None,
                    "border-radius accepts at most four radii per axis",
                ));
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
        return Err(unsupported_value(
            input,
            None,
            "border-radius requires a horizontal radius",
        ));
    }
    let vertical = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        let values = parse_radius_list(input, numeric)?;
        if values.is_empty() {
            return Err(unsupported_value(
                input,
                None,
                "border-radius slash requires a vertical radius",
            ));
        }
        Some(values)
    } else {
        None
    };
    CssBorderRadiusShorthand::try_new(horizontal, vertical).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "border-radius requires one to four values per axis",
        )
    })
}
