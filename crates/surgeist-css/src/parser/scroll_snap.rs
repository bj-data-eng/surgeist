//! Checked authored grammars from the selected Scroll Snap 1 publication.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::match_ignore_ascii_case;

use super::values::{parse_length, parse_nonnegative_length_percentage};
use crate::numeric::NumericInputContext;
use crate::{
    CssScrollMarginPair, CssScrollMarginShorthand, CssScrollPaddingPair, CssScrollPaddingShorthand,
    CssScrollPaddingValue, CssScrollSideKind, CssScrollSnapAlign, CssScrollSnapAlignment,
    CssScrollSnapAxis, CssScrollSnapStop, CssScrollSnapStrictness, CssScrollSnapType,
    CssSpecifiedLength,
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
        _ => return Err(unexpected_at(input.current_source_location())),
    };
    let strictness = if input.is_exhausted() {
        None
    } else {
        let strictness = input.expect_ident_cloned().map_err(basic)?;
        Some(match_ignore_ascii_case! { &strictness,
            "mandatory" => CssScrollSnapStrictness::Mandatory,
            "proximity" => CssScrollSnapStrictness::Proximity,
            _ => return Err(unexpected_at(input.current_source_location())),
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
        _ => Err(unexpected_at(input.current_source_location())),
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
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_scroll_margin_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssSpecifiedLength, ParseError<'i, Error>> {
    parse_length(input, numeric)
}

pub(super) fn parse_scroll_padding_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssScrollPaddingValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssScrollPaddingValue::Auto);
    }
    parse_nonnegative_length_percentage(input, numeric).map(CssScrollPaddingValue::LengthPercentage)
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
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssScrollMarginShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
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
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssScrollPaddingShorthand::try_new(kind, authored)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
