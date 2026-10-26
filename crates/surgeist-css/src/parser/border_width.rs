//! Shared exact line-width and unordered width/style/color grammar for borders and column rules.

use super::combinators::{Combinator, Tail, fill, parse_group};
use super::property_input::Parser;
use super::side_shapes::{parse_four_sides, parse_side_pair};
use crate::error::property::{Error, ParseError, unexpected_at};

use super::border_style::parse_border_style;
use super::parse_color;
use crate::numeric::NumericInputContext;
use crate::{CssFourSides, CssLineStyle, CssLineWidth, CssSidePair};

type ParsedLineTriple = (
    Option<CssLineWidth>,
    Option<CssLineStyle>,
    Option<crate::CssColor>,
);

pub(super) fn parse_exact_border_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssLineWidth, ParseError<'i, Error>> {
    parse_exact_line_width(input, numeric)
}

pub(super) fn parse_exact_line_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssLineWidth, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("thin"))
        .is_ok()
    {
        return Ok(CssLineWidth::Thin);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("medium"))
        .is_ok()
    {
        return Ok(CssLineWidth::Medium);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("thick"))
        .is_ok()
    {
        return Ok(CssLineWidth::Thick);
    }
    super::values::parse_nonnegative_length(input, numeric).map(CssLineWidth::Length)
}

pub(super) fn parse_exact_border_width_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssLineWidth>, ParseError<'i, Error>> {
    parse_side_pair(input, |input| parse_exact_border_width(input, numeric))
}

pub(super) fn parse_exact_border_width_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssLineWidth>, ParseError<'i, Error>> {
    parse_four_sides(input, |input| parse_exact_border_width(input, numeric))
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
