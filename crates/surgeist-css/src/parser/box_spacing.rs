//! Authored margin and padding from Box 3 and Logical Properties 1.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, unexpected_at};

use super::values::{parse_length_percentage, parse_nonnegative_length_percentage};
use crate::numeric::NumericInputContext;
use crate::{
    CssBoxSideKind, CssMarginPair, CssMarginShorthand, CssMarginValue, CssPaddingPair,
    CssPaddingShorthand, CssPaddingValue,
};

pub(super) fn parse_box_margin_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMarginValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssMarginValue::Auto);
    }
    parse_length_percentage(input, numeric).map(CssMarginValue::LengthPercentage)
}

pub(super) fn parse_box_padding_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingValue, ParseError<'i, Error>> {
    parse_nonnegative_length_percentage(input, numeric).map(CssPaddingValue::new)
}

pub(super) fn parse_box_margin_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMarginPair, ParseError<'i, Error>> {
    let start = parse_box_margin_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_box_margin_value(input, numeric)?)
    };
    Ok(CssMarginPair::new(start, end))
}

pub(super) fn parse_box_padding_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingPair, ParseError<'i, Error>> {
    let start = parse_box_padding_value(input, numeric)?;
    let end = if input.is_exhausted() {
        None
    } else {
        Some(parse_box_padding_value(input, numeric)?)
    };
    Ok(CssPaddingPair::new(start, end))
}

fn parse_side_kind<'i, 't>(input: &mut Parser<'i, 't>) -> CssBoxSideKind {
    if input
        .try_parse(|input| input.expect_ident_matching("logical"))
        .is_ok()
    {
        CssBoxSideKind::Logical
    } else {
        CssBoxSideKind::Physical
    }
}

pub(super) fn parse_box_margin_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssMarginShorthand, ParseError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_box_margin_value(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssMarginShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_box_padding_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssPaddingShorthand, ParseError<'i, Error>> {
    let kind = parse_side_kind(input);
    let mut authored = Vec::new();
    while !input.is_exhausted() {
        authored.push(parse_box_padding_value(input, numeric)?);
        if authored.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssPaddingShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
