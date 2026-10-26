//! Border style grammar shared by longhands, pairs, four-side shorthand and triples.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::match_ignore_ascii_case;

use super::side_shapes::{parse_four_sides, parse_side_pair};
use crate::{CssFourSides, CssLineStyle, CssSidePair};

pub(super) fn parse_border_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssLineStyle, ParseError<'i, Error>> {
    // The unadmitted keyword itself is responsible, not the token after it.
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "none" => Ok(CssLineStyle::None),
        "hidden" => Ok(CssLineStyle::Hidden),
        "dotted" => Ok(CssLineStyle::Dotted),
        "dashed" => Ok(CssLineStyle::Dashed),
        "solid" => Ok(CssLineStyle::Solid),
        "double" => Ok(CssLineStyle::Double),
        "groove" => Ok(CssLineStyle::Groove),
        "ridge" => Ok(CssLineStyle::Ridge),
        "inset" => Ok(CssLineStyle::Inset),
        "outset" => Ok(CssLineStyle::Outset),
        _ => Err(unexpected_at(location)),
    }
}

pub(super) fn parse_border_style_pair<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssSidePair<CssLineStyle>, ParseError<'i, Error>> {
    parse_side_pair(input, parse_border_style)
}

pub(super) fn parse_border_style_shorthand<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssFourSides<CssLineStyle>, ParseError<'i, Error>> {
    parse_four_sides(input, parse_border_style)
}
