//! Border style grammar shared by longhands, pairs, four-side shorthand and triples.

use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use crate::error::{Error, basic, unexpected_at};
use crate::{CssBorderStyle, CssBorderStylePair, CssBorderStyleShorthand, CssBoxSideKind};

pub(super) fn parse_border_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssBorderStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssBorderStyle::None),
        "hidden" => Ok(CssBorderStyle::Hidden),
        "dotted" => Ok(CssBorderStyle::Dotted),
        "dashed" => Ok(CssBorderStyle::Dashed),
        "solid" => Ok(CssBorderStyle::Solid),
        "double" => Ok(CssBorderStyle::Double),
        "groove" => Ok(CssBorderStyle::Groove),
        "ridge" => Ok(CssBorderStyle::Ridge),
        "inset" => Ok(CssBorderStyle::Inset),
        "outset" => Ok(CssBorderStyle::Outset),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_border_style_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssBorderStylePair, ParseError<'i, Error>> {
    let start = parse_border_style(input)?;
    let authored_end = if input.is_exhausted() {
        None
    } else {
        Some(parse_border_style(input)?)
    };
    Ok(CssBorderStylePair::new(start, authored_end))
}

pub(super) fn parse_border_style_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssBorderStyleShorthand, ParseError<'i, Error>> {
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
        values.push(parse_border_style(input)?);
        if values.len() == 4 && !input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssBorderStyleShorthand::try_new(kind, values)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
