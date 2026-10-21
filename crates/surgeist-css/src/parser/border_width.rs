//! Shared exact line-width and unordered width/style/color grammar for borders and column rules.

use super::combinators::{Combinator, Tail, fill, parse_group};
use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::Token;

use super::border_style::parse_border_style;
use super::parse_color;
use super::values::{CalculationRoot, is_math_function, parse_numeric_function};
use crate::numeric::NumericInputContext;
use crate::{
    CssBorderStyle, CssBorderWidth, CssBorderWidthPair, CssBorderWidthShorthand, CssBoxSideKind,
    CssLengthCalculation, CssSpecifiedNonNegativeLength,
};

type ParsedLineTriple = (
    Option<CssBorderWidth>,
    Option<CssBorderStyle>,
    Option<crate::CssColor>,
);

pub(super) fn parse_exact_border_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBorderWidth, ParseError<'i, Error>> {
    parse_exact_line_width(input, numeric)
}

pub(super) fn parse_exact_line_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBorderWidth, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("thin"))
        .is_ok()
    {
        return Ok(CssBorderWidth::Thin);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("medium"))
        .is_ok()
    {
        return Ok(CssBorderWidth::Medium);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("thick"))
        .is_ok()
    {
        return Ok(CssBorderWidth::Thick);
    }
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    let root_site = input.position();
    let length = match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
            input.reset(&state);
            let component = input
                .collect_numeric_component(numeric)
                .map_err(|_| unexpected_at(location))?;
            CssSpecifiedNonNegativeLength::from_property_component(component, numeric)
        }
        Token::Function(name) if is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &state, numeric, CalculationRoot::Length)?;
            CssSpecifiedNonNegativeLength::try_from_calculation(
                CssLengthCalculation::from_expression(expression),
            )
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    length.map(CssBorderWidth::Length).map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, root_site))
    })
}

pub(super) fn parse_exact_border_width_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBorderWidthPair, ParseError<'i, Error>> {
    let start = parse_exact_border_width(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_exact_border_width(input, numeric)?)
    };
    Ok(CssBorderWidthPair::new(start, end))
}

pub(super) fn parse_exact_border_width_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssBorderWidthShorthand, ParseError<'i, Error>> {
    let kind = if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    };
    let mut values = Vec::new();
    while !input.is_exhausted() {
        values.push(parse_exact_border_width(input, numeric)?);
        if values.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssBorderWidthShorthand::try_new(kind, values)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_border<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::CssBorder, ParseError<'i, Error>> {
    let (width, style, color) = parse_exact_line_triple(input, numeric)?;
    crate::CssBorder::try_new(width, style, color)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_exact_line_triple<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<ParsedLineTriple, ParseError<'i, Error>> {
    // `<line-width> || <line-style> || <color>`.
    let (mut width, mut style, mut color) = (None, None, None);
    parse_group::<3>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut width, input, |input| {
                parse_exact_line_width(input, numeric)
            }),
            1 => fill(&mut style, input, parse_border_style),
            2 => fill(&mut color, input, |input| parse_color(input, numeric)),
            _ => unreachable!("three line operands"),
        },
    )?;
    Ok((width, style, color))
}
