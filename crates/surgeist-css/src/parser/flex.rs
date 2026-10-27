//! Flexbox 1 authored factor, basis, and shorthand grammar.

use super::combinators::{Combinator, Tail, fill, parse_group};
use super::property_input::Parser;
use crate::diagnostic::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::{Token, match_ignore_ascii_case};

use super::sizing::parse_size_value;
use crate::component::CssComponentValueLimits;
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssNonNegative, CssNumber, CssNumeric};
use crate::values::box_model::CssCalcSize;
use crate::values::layout::{
    CssFlexBasisValue, CssFlexComponents, CssFlexDirection, CssFlexFlow, CssFlexValue, CssFlexWrap,
};

pub(super) fn parse_flex_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexDirection, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "row" => Ok(CssFlexDirection::Row),
        "column" => Ok(CssFlexDirection::Column),
        "row-reverse" => Ok(CssFlexDirection::RowReverse),
        "column-reverse" => Ok(CssFlexDirection::ColumnReverse),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_flex_flow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexFlow, ParseError<'i, Error>> {
    // `<'flex-direction'> || <'flex-wrap'>`.
    let (mut direction, mut wrap) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut direction, input, parse_flex_direction),
            1 => fill(&mut wrap, input, parse_flex_wrap),
            _ => unreachable!("two flex-flow operands"),
        },
    )?;
    Ok(CssFlexFlow::new(
        direction.unwrap_or(CssFlexDirection::Row),
        wrap.unwrap_or(CssFlexWrap::NoWrap),
    ))
}

pub(super) fn parse_flex_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFlexWrap, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "nowrap" => Ok(CssFlexWrap::NoWrap),
        "wrap" => Ok(CssFlexWrap::Wrap),
        "wrap-reverse" => Ok(CssFlexWrap::WrapReverse),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_flex_factor<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssNumeric<CssNumber, CssNonNegative>, ParseError<'i, Error>> {
    super::values::parse_nonnegative_number(input, numeric)
}

pub(super) fn parse_flex_basis<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFlexBasisValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_site = input.position();
    match input.next().map_err(basic)? {
        Token::Ident(value) if value.eq_ignore_ascii_case("content") => {
            Ok(CssFlexBasisValue::content())
        }
        Token::Function(value) if value.eq_ignore_ascii_case("calc-size") => {
            input.reset(&state);
            let component = input
                .collect_numeric_component(numeric)
                .map_err(|_| unexpected_at(location))?;
            CssCalcSize::from_component_with_policy(
                component,
                CssComponentValueLimits::default(),
                numeric.is_source_input(),
            )
            .map(CssFlexBasisValue::from)
            .map_err(|error| {
                unexpected_at(numeric.property_error_location(&error, location, root_site))
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
        .try_parse(|input| parse_flex_factor(input, numeric))
        .ok();
    let (shrink, basis) = if grow.is_some() {
        let shrink = if input.is_exhausted() {
            None
        } else {
            input
                .try_parse(|input| parse_flex_factor(input, numeric))
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
                .try_parse(|input| parse_flex_factor(input, numeric))
                .ok()
        };
        let shrink = if grow.is_some() && !input.is_exhausted() {
            input
                .try_parse(|input| parse_flex_factor(input, numeric))
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
