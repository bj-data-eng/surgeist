//! Selected Color Adjustment 1 property grammars.

use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use crate::error::{Error, basic, unexpected_at};
use crate::{
    CssColorScheme, CssColorSchemeKeyword, CssColorSchemeName, CssForcedColorAdjust, CssIdent,
    CssPrintColorAdjust,
};

pub(super) fn parse_color_scheme<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssColorScheme, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssColorScheme::normal());
    }
    let mut schemes = Vec::new();
    let mut only = false;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let location = input.current_source_location();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if ident.eq_ignore_ascii_case("only") {
            if only || schemes.is_empty() && input.is_exhausted() {
                return Err(unexpected_at(location));
            }
            only = true;
            if !schemes.is_empty() {
                input.expect_exhausted().map_err(basic)?;
                break;
            }
        } else {
            let scheme = match_ignore_ascii_case! { &ident,
                "light" => CssColorSchemeKeyword::Light,
                "dark" => CssColorSchemeKeyword::Dark,
                _ => {
                    let name = CssColorSchemeName::try_new(CssIdent::new(ident.as_ref()))
                        .map_err(|_| unexpected_at(location))?;
                    CssColorSchemeKeyword::Custom(name)
                },
            };
            schemes.push(scheme);
        }
    }
    CssColorScheme::try_new(schemes, only)
        .map_err(|_| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_forced_color_adjust<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssForcedColorAdjust, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssForcedColorAdjust::Auto),
        "none" => Ok(CssForcedColorAdjust::None),
        "preserve-parent-color" => Ok(CssForcedColorAdjust::PreserveParentColor),
        _ => Err(unexpected_at(location)),
    }
}

pub(super) fn parse_print_color_adjust<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssPrintColorAdjust, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "economy" => Ok(CssPrintColorAdjust::Economy),
        "exact" => Ok(CssPrintColorAdjust::Exact),
        _ => Err(unexpected_at(location)),
    }
}
