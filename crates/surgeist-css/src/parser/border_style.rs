//! Border style grammar shared by longhands, pairs, four-side shorthand and triples.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::match_ignore_ascii_case;

use super::side_shapes::{parse_four_sides, parse_side_pair};
use crate::values::border::CssLineStyle;
use crate::values::common::{CssFourSides, CssSidePair};

pub(super) fn parse_border_style<'i>(
    input: &mut Stream<'i>,
) -> Result<CssLineStyle, SyntaxError<'i, Error>> {
    // The unadmitted keyword itself is responsible, not the token after it.
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
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

pub(super) fn parse_border_style_pair<'i>(
    input: &mut Stream<'i>,
) -> Result<CssSidePair<CssLineStyle>, SyntaxError<'i, Error>> {
    parse_side_pair(input, parse_border_style)
}

pub(super) fn parse_border_style_shorthand<'i>(
    input: &mut Stream<'i>,
) -> Result<CssFourSides<CssLineStyle>, SyntaxError<'i, Error>> {
    parse_four_sides(input, parse_border_style)
}
