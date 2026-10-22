use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::match_ignore_ascii_case;

use crate::{
    CssFontKerning, CssFontLanguageOverride, CssFontLanguageString, CssFontOpticalSizing,
    CssFontSizeAdjust,
};

pub(super) fn parse_font_kerning<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssFontKerning, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontKerning::Auto),
        "normal" => Ok(CssFontKerning::Normal),
        "none" => Ok(CssFontKerning::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_font_size_adjust<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssFontSizeAdjust, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFontSizeAdjust::None);
    }
    super::values::parse_nonnegative_number(input, numeric).map(CssFontSizeAdjust::Number)
}

pub(super) fn parse_font_language_override<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssFontLanguageOverride, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssFontLanguageOverride::Normal);
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|_| unexpected_at(location))?;
    CssFontLanguageString::try_from_component(component)
        .map(CssFontLanguageOverride::String)
        .map_err(|_| unexpected_at(location))
}

pub(super) fn parse_font_optical_sizing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssFontOpticalSizing, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontOpticalSizing::Auto),
        "none" => Ok(CssFontOpticalSizing::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}
