//! Shared exact line-width and unordered width/style/color grammar for borders and column rules.

use super::combinators::{Combinator, Tail, fill, parse_group};
use super::side_shapes::{parse_four_sides, parse_side_pair};
use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, unexpected_at};
use crate::parser::syntax_types::SyntaxError;

use super::border_style::parse_border_style;
use crate::numeric::expression::NumericInputContext;
use crate::parser::color::parse_color;
use crate::values::border::{CssLineStyle, CssLineWidth};
use crate::values::common::{CssFourSides, CssSidePair};

type ParsedLineTriple = (
    Option<CssLineWidth>,
    Option<CssLineStyle>,
    Option<crate::color::CssColor>,
);

pub(super) fn parse_exact_border_width<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssLineWidth, SyntaxError<'i, Error>> {
    parse_exact_line_width(input, numeric)
}

pub(super) fn parse_exact_line_width<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssLineWidth, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("thin")).is_ok() {
        return Ok(CssLineWidth::Thin);
    }
    if input
        .attempt(|input| input.consume_keyword("medium"))
        .is_ok()
    {
        return Ok(CssLineWidth::Medium);
    }
    if input
        .attempt(|input| input.consume_keyword("thick"))
        .is_ok()
    {
        return Ok(CssLineWidth::Thick);
    }
    super::values::parse_nonnegative_length(input, numeric).map(CssLineWidth::Length)
}

pub(super) fn parse_exact_border_width_pair<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSidePair<CssLineWidth>, SyntaxError<'i, Error>> {
    parse_side_pair(input, |input| parse_exact_border_width(input, numeric))
}

pub(super) fn parse_exact_border_width_shorthand<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssFourSides<CssLineWidth>, SyntaxError<'i, Error>> {
    parse_four_sides(input, |input| parse_exact_border_width(input, numeric))
}

pub(super) fn parse_border<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::values::border::CssBorder, SyntaxError<'i, Error>> {
    let (width, style, color) = parse_exact_line_triple(input, numeric)?;
    crate::values::border::CssBorder::try_new(width, style, color)
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_exact_line_triple<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<ParsedLineTriple, SyntaxError<'i, Error>> {
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
