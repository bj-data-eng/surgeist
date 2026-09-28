//! Flexbox 1 authored factor, basis, and shorthand grammar.

use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::sizing::parse_size_value;
use super::values::{CalculationRoot, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::validation::unsupported_keyword_reason;
use crate::{
    CssCalcSize, CssComponentValueLimits, CssFlexBasisValue, CssFlexComponents, CssFlexDirection,
    CssFlexFlow, CssFlexValue, CssFlexWrap, CssNumberCalculation, CssSpecifiedNonNegativeNumber,
};

pub(super) fn parse_flex_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexDirection, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "row" => Ok(CssFlexDirection::Row),
        "column" => Ok(CssFlexDirection::Column),
        "row-reverse" => Ok(CssFlexDirection::RowReverse),
        "column-reverse" => Ok(CssFlexDirection::ColumnReverse),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("flex-direction", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_flex_flow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexFlow, ParseError<'i, Error>> {
    let mut direction = None;
    let mut wrap = None;

    while !input.is_exhausted() {
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        match_ignore_ascii_case! { &ident,
            "row" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::Row,
                location,
                ident.as_ref(),
            )?,
            "column" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::Column,
                location,
                ident.as_ref(),
            )?,
            "row-reverse" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::RowReverse,
                location,
                ident.as_ref(),
            )?,
            "column-reverse" => set_flex_flow_direction(
                &mut direction,
                CssFlexDirection::ColumnReverse,
                location,
                ident.as_ref(),
            )?,
            "nowrap" => set_flex_flow_wrap(
                &mut wrap,
                CssFlexWrap::NoWrap,
                location,
                ident.as_ref(),
            )?,
            "wrap" => set_flex_flow_wrap(
                &mut wrap,
                CssFlexWrap::Wrap,
                location,
                ident.as_ref(),
            )?,
            "wrap-reverse" => set_flex_flow_wrap(
                &mut wrap,
                CssFlexWrap::WrapReverse,
                location,
                ident.as_ref(),
            )?,
            _ => return Err(unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("flex-flow", ident.as_ref()),
            )),
        }
    }

    if direction.is_none() && wrap.is_none() {
        return Err(unsupported_value(
            input,
            None,
            "flex-flow requires a direction or wrapping component",
        ));
    }

    Ok(CssFlexFlow::new(
        direction.unwrap_or(CssFlexDirection::Row),
        wrap.unwrap_or(CssFlexWrap::NoWrap),
    ))
}

fn set_flex_flow_direction<'i>(
    slot: &mut Option<CssFlexDirection>,
    value: CssFlexDirection,
    location: cssparser::SourceLocation,
    ident: &str,
) -> std::result::Result<(), ParseError<'i, Error>> {
    if slot.replace(value).is_some() {
        return Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("flex-flow direction", ident),
        ));
    }
    Ok(())
}

fn set_flex_flow_wrap<'i>(
    slot: &mut Option<CssFlexWrap>,
    value: CssFlexWrap,
    location: cssparser::SourceLocation,
    ident: &str,
) -> std::result::Result<(), ParseError<'i, Error>> {
    if slot.replace(value).is_some() {
        return Err(unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("flex-flow wrap", ident),
        ));
    }
    Ok(())
}

pub(super) fn parse_flex_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexWrap, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "nowrap" => Ok(CssFlexWrap::NoWrap),
        "wrap" => Ok(CssFlexWrap::Wrap),
        "wrap-reverse" => Ok(CssFlexWrap::WrapReverse),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("flex-wrap", ident.as_ref()),
        )),
    }
}

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
