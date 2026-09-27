//! Border style grammar shared by longhands, pairs, four-side shorthand and triples.

use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use crate::error::{Error, basic, unsupported_value};
use crate::validation::unsupported_keyword_reason;
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("border-style", ident.as_ref()),
        )),
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
            return Err(unsupported_value(
                input,
                None,
                "border-style accepts at most four values",
            ));
        }
    }
    CssBorderStyleShorthand::try_new(kind, values)
        .ok_or_else(|| unsupported_value(input, None, "border-style requires one to four values"))
}
