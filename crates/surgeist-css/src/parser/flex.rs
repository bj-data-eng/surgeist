//! Flexbox 1 authored factor, basis, and shorthand grammar.

use super::combinators::{Combinator, Tail, fill, parse_group};
use super::property_input::Stream;
use crate::diagnostic::error::property::{Error, SyntaxError, basic, unexpected_at};
use cssparser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::sizing::parse_size_value;
use crate::component::CssComponentValueLimits;
use crate::numeric::expression::NumericInputContext;
use crate::numeric::{CssNonNegative, CssNumber, CssNumeric};
use crate::values::box_model::CssCalcSize;
use crate::values::layout::{
    CssFlexBasisValue, CssFlexComponents, CssFlexDirection, CssFlexFlow, CssFlexValue, CssFlexWrap,
};

pub(super) fn parse_flex_direction<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFlexDirection, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "row" => Ok(CssFlexDirection::Row),
        "column" => Ok(CssFlexDirection::Column),
        "row-reverse" => Ok(CssFlexDirection::RowReverse),
        "column-reverse" => Ok(CssFlexDirection::ColumnReverse),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_flex_flow<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFlexFlow, SyntaxError<'i, Error>> {
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

pub(super) fn parse_flex_wrap<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssFlexWrap, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "nowrap" => Ok(CssFlexWrap::NoWrap),
        "wrap" => Ok(CssFlexWrap::Wrap),
        "wrap-reverse" => Ok(CssFlexWrap::WrapReverse),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_flex_factor<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssNumeric<CssNumber, CssNonNegative>, SyntaxError<'i, Error>> {
    super::values::parse_nonnegative_number(input, numeric)
}

pub(super) fn parse_flex_basis<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFlexBasisValue, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let state = input.mark();
    let location = input.site();
    let root_site = input.site();
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Ident(value) if value.eq_ignore_ascii_case("content") => {
            Ok(CssFlexBasisValue::content())
        }
        Token::Function(value) if value.eq_ignore_ascii_case("calc-size") => {
            input.restore(state);
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
            input.restore(state);
            parse_size_value(input, numeric).map(CssFlexBasisValue::from)
        }
    }
}

pub(super) fn parse_flex<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFlexValue, SyntaxError<'i, Error>> {
    let start = input.mark();
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssFlexValue::None);
    }
    if input.attempt(|input| input.consume_keyword("auto")).is_ok()
        && input.only_whitespace_remains()
    {
        return Ok(CssFlexValue::Auto);
    }
    input.restore(start);

    let grow = input
        .attempt(|input| parse_flex_factor(input, numeric))
        .ok();
    let (shrink, basis) = if grow.is_some() {
        let shrink = if input.only_whitespace_remains() {
            None
        } else {
            input
                .attempt(|input| parse_flex_factor(input, numeric))
                .ok()
        };
        let basis = if input.only_whitespace_remains() {
            None
        } else {
            Some(parse_flex_basis(input, numeric)?)
        };
        (shrink, basis)
    } else {
        let basis = Some(parse_flex_basis(input, numeric)?);
        let grow = if input.only_whitespace_remains() {
            None
        } else {
            input
                .attempt(|input| parse_flex_factor(input, numeric))
                .ok()
        };
        let shrink = if grow.is_some() && !input.only_whitespace_remains() {
            input
                .attempt(|input| parse_flex_factor(input, numeric))
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
