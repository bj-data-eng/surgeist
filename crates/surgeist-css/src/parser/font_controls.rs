use super::property_input::Stream;
use crate::diagnostic::error::property::{Error, SyntaxError, basic, unexpected_at};
use surgeist_css_parser::match_ignore_ascii_case;

use crate::values::font::{
    CssFontKerning, CssFontLanguageOverride, CssFontLanguageString, CssFontOpticalSizing,
    CssFontSizeAdjust,
};

pub(super) fn parse_font_kerning<'i>(
    input: &mut Stream<'i>,
) -> Result<CssFontKerning, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontKerning::Auto),
        "normal" => Ok(CssFontKerning::Normal),
        "none" => Ok(CssFontKerning::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_font_size_adjust<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssFontSizeAdjust, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssFontSizeAdjust::None);
    }
    super::values::parse_nonnegative_number(input, numeric).map(CssFontSizeAdjust::Number)
}

pub(super) fn parse_font_language_override<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssFontLanguageOverride, SyntaxError<'i, Error>> {
    if input
        .attempt(|input| input.consume_keyword("normal"))
        .is_ok()
    {
        return Ok(CssFontLanguageOverride::Normal);
    }
    input.discard_whitespace();
    let location = input.site();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|_| unexpected_at(location))?;
    CssFontLanguageString::try_from_component(component)
        .map(CssFontLanguageOverride::String)
        .map_err(|_| unexpected_at(location))
}

pub(super) fn parse_font_optical_sizing<'i>(
    input: &mut Stream<'i>,
) -> Result<CssFontOpticalSizing, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontOpticalSizing::Auto),
        "none" => Ok(CssFontOpticalSizing::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}
