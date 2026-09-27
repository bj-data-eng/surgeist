//! Exact Box Alignment gap grammars.

use cssparser::{ParseError, Parser, Token};

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssGapShorthand, CssGapValue, CssLengthPercentageCalculation,
    CssSpecifiedNonNegativeLengthPercentage,
};

pub(super) fn parse_gap_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGapValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssGapValue::Normal);
    }
    let state = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let checked = match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid gap value"))?;
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
    checked.map(CssGapValue::LengthPercentage).map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "gap requires normal or a nonnegative length-percentage",
        )
    })
}

pub(super) fn parse_gap_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssGapShorthand, ParseError<'i, Error>> {
    let row = parse_gap_value(input, numeric)?;
    let column = if input.is_exhausted() {
        None
    } else {
        Some(parse_gap_value(input, numeric)?)
    };
    Ok(CssGapShorthand::new(row, column))
}
