//! Checked authored grammars from the selected Scroll Snap 1 publication.

use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssLengthCalculation, CssLengthPercentageCalculation, CssScrollMarginPair,
    CssScrollMarginShorthand, CssScrollPaddingPair, CssScrollPaddingShorthand,
    CssScrollPaddingValue, CssScrollSideKind, CssScrollSnapAlign, CssScrollSnapAlignment,
    CssScrollSnapAxis, CssScrollSnapStop, CssScrollSnapStrictness, CssScrollSnapType,
    CssSpecifiedLength, CssSpecifiedNonNegativeLengthPercentage,
};

pub(super) fn parse_scroll_snap_type<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollSnapType, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let axis = match_ignore_ascii_case! { &ident,
        "none" => return Ok(CssScrollSnapType::None),
        "x" => CssScrollSnapAxis::X,
        "y" => CssScrollSnapAxis::Y,
        "block" => CssScrollSnapAxis::Block,
        "inline" => CssScrollSnapAxis::Inline,
        "both" => CssScrollSnapAxis::Both,
        _ => return Err(unsupported_value(input, None, "scroll-snap-type requires an axis before optional strictness")),
    };
    let strictness = if input.is_exhausted() {
        None
    } else {
        let strictness = input.expect_ident_cloned().map_err(basic)?;
        Some(match_ignore_ascii_case! { &strictness,
            "mandatory" => CssScrollSnapStrictness::Mandatory,
            "proximity" => CssScrollSnapStrictness::Proximity,
            _ => return Err(unsupported_value(input, None, "invalid scroll-snap-type strictness")),
        })
    };
    Ok(CssScrollSnapType::axis(axis, strictness))
}

fn parse_alignment<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollSnapAlignment, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssScrollSnapAlignment::None),
        "start" => Ok(CssScrollSnapAlignment::Start),
        "end" => Ok(CssScrollSnapAlignment::End),
        "center" => Ok(CssScrollSnapAlignment::Center),
        _ => Err(unsupported_value(input, None, "invalid scroll-snap-align keyword")),
    }
}

pub(super) fn parse_scroll_snap_align<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollSnapAlign, ParseError<'i, Error>> {
    let block = parse_alignment(input)?;
    let inline = if input.is_exhausted() {
        None
    } else {
        Some(parse_alignment(input)?)
    };
    Ok(CssScrollSnapAlign::new(block, inline))
}

pub(super) fn parse_scroll_snap_stop<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollSnapStop, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssScrollSnapStop::Normal),
        "always" => Ok(CssScrollSnapStop::Always),
        _ => Err(unsupported_value(input, None, "invalid scroll-snap-stop keyword")),
    }
}

pub(super) fn parse_scroll_margin_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSpecifiedLength, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid scroll-margin length component")
            })?;
            CssSpecifiedLength::try_from_component(component).map_err(|_| {
                unsupported_value_at(location, None, "scroll-margin requires a pure length")
            })
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::Length)?;
            CssSpecifiedLength::try_from_calculation(CssLengthCalculation::from_expression(
                expression,
            ))
            .map_err(|_| unsupported_value_at(location, None, "invalid scroll-margin length math"))
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_scroll_padding_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollPaddingValue, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("auto") => {
            Ok(CssScrollPaddingValue::Auto)
        }
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid scroll-padding component")
            })?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(component)
                .map(CssScrollPaddingValue::LengthPercentage)
                .map_err(|_| {
                    unsupported_value_at(
                        location,
                        None,
                        "scroll-padding requires a nonnegative length-percentage or auto",
                    )
                })
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::LengthPercentage)?;
            CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                CssLengthPercentageCalculation::from_expression(expression),
            )
            .map(CssScrollPaddingValue::LengthPercentage)
            .map_err(|_| unsupported_value_at(location, None, "invalid scroll-padding math"))
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_scroll_margin_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollMarginPair, ParseError<'i, Error>> {
    let start = parse_scroll_margin_length(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_scroll_margin_length(input, numeric)?)
    };
    Ok(CssScrollMarginPair::new(start, end))
}

pub(super) fn parse_scroll_padding_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollPaddingPair, ParseError<'i, Error>> {
    let start = parse_scroll_padding_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_scroll_padding_value(input, numeric)?)
    };
    Ok(CssScrollPaddingPair::new(start, end))
}

fn parse_side_kind<'i, 't>(input: &mut Parser<'i, 't>) -> CssScrollSideKind {
    if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssScrollSideKind::Logical
    } else {
        CssScrollSideKind::Physical
    }
}

pub(super) fn parse_scroll_margin_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollMarginShorthand, ParseError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_scroll_margin_length(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "scroll-margin accepts at most four values",
            ));
        }
    }
    CssScrollMarginShorthand::try_new(kind, authored)
        .ok_or_else(|| unsupported_value(input, None, "scroll-margin requires one to four lengths"))
}

pub(super) fn parse_scroll_padding_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollPaddingShorthand, ParseError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_scroll_padding_value(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "scroll-padding accepts at most four values",
            ));
        }
    }
    CssScrollPaddingShorthand::try_new(kind, authored)
        .ok_or_else(|| unsupported_value(input, None, "scroll-padding requires one to four values"))
}
