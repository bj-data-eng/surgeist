//! Shared exact line-width and unordered width/style/color grammar for borders and column rules.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, unexpected_at};

use super::border_style::parse_border_style;
use super::parse_color;
use crate::numeric::NumericInputContext;
use crate::{
    CssBorderStyle, CssBorderWidth, CssBorderWidthPair, CssBorderWidthShorthand, CssBoxSideKind,
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
    super::values::parse_nonnegative_length(input, numeric).map(CssBorderWidth::Length)
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
    let mut width = None;
    let mut style: Option<CssBorderStyle> = None;
    let mut color = None;
    while !input.is_exhausted() {
        if let Ok(value) = input.try_parse(|input| parse_exact_line_width(input, numeric)) {
            if width.replace(value).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
            continue;
        }
        if let Ok(value) = input.try_parse(parse_border_style) {
            if style.replace(value).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
            continue;
        }
        if let Ok(value) = input.try_parse(|input| parse_color(input, numeric)) {
            if color.replace(value).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
            continue;
        }
        return Err(unexpected_at(input.current_source_location()));
    }
    if width.is_none() && style.is_none() && color.is_none() {
        return Err(unexpected_at(input.current_source_location()));
    }
    Ok((width, style, color))
}
