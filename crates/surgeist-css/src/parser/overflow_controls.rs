//! CSS Overflow 3 terminal controls.

use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssBoxEdgeKeyword, CssLengthCalculation, CssOverflowClipMargin, CssScrollBehavior,
    CssScrollbarGutter, CssSpecifiedNonNegativeLength,
};

pub(super) fn parse_overflow_clip_margin<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOverflowClipMargin, ParseError<'i, Error>> {
    let mut box_edge = None;
    let mut offset = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let state = input.state();
        let location = input.current_source_location();
        match input.next().map_err(basic)? {
            Token::Ident(ident) => {
                let edge = CssBoxEdgeKeyword::from_keyword(ident).ok_or_else(|| {
                    unsupported_value_at(
                        location,
                        None,
                        "expected a visual box or nonnegative length",
                    )
                })?;
                if box_edge.replace(edge).is_some() {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        "duplicate clip-margin box edge",
                    ));
                }
            }
            Token::Number { .. }
            | Token::Dimension { .. }
            | Token::Percentage { .. }
            | Token::Function(_) => {
                if offset.is_some() {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        "duplicate clip-margin offset",
                    ));
                }
                input.reset(&state);
                offset = Some(parse_nonnegative_length(input, numeric)?);
            }
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        }
    }
    CssOverflowClipMargin::try_new(box_edge, offset).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "clip margin requires a visual box or nonnegative length",
        )
    })
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
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid clip-margin offset"))?;
            CssSpecifiedNonNegativeLength::try_from_component(component).map_err(|error| {
                unsupported_value_at(
                    numeric.error_location(&error, location, root_offset),
                    None,
                    "clip margin requires a nonnegative length",
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
                    "invalid clip-margin offset math",
                )
            })
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_scroll_behavior<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollBehavior, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssScrollBehavior::Auto),
        "smooth" => Ok(CssScrollBehavior::Smooth),
        _ => Err(unsupported_value(input, None, "expected auto or smooth")),
    }
}

pub(super) fn parse_scrollbar_gutter<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollbarGutter, ParseError<'i, Error>> {
    let first = input.expect_ident_cloned().map_err(basic)?;
    if first.eq_ignore_ascii_case("auto") {
        return Ok(CssScrollbarGutter::Auto);
    }
    if first.eq_ignore_ascii_case("stable") {
        if input
            .try_parse(|input| input.expect_ident_matching("both-edges"))
            .is_ok()
        {
            return Ok(CssScrollbarGutter::StableBothEdges);
        }
        return Ok(CssScrollbarGutter::Stable);
    }
    if first.eq_ignore_ascii_case("both-edges") {
        input.expect_ident_matching("stable").map_err(basic)?;
        return Ok(CssScrollbarGutter::StableBothEdges);
    }
    Err(unsupported_value(
        input,
        None,
        "expected auto or stable with optional both-edges",
    ))
}

pub(super) fn parse_overflow_anchor<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<crate::CssOverflowAnchor, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(crate::CssOverflowAnchor::Auto),
        "none" => Ok(crate::CssOverflowAnchor::None),
        _ => Err(unsupported_value(input, None, "expected auto or none")),
    }
}
