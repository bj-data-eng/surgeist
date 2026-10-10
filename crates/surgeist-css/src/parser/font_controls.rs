use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{CalculationRoot, parse_numeric_function};
use crate::error::{Error, basic, unexpected_at};
use crate::{
    CssFontKerning, CssFontLanguageOverride, CssFontLanguageString, CssFontOpticalSizing,
    CssFontSizeAdjust, CssNumberCalculation, CssSpecifiedNonNegativeNumber,
};

pub(super) fn parse_font_kerning<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssFontKerning, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontKerning::Auto),
        "normal" => Ok(CssFontKerning::Normal),
        "none" => Ok(CssFontKerning::None),
        _ => Err(unexpected_at(input.current_source_location())),
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
    input.skip_whitespace();
    let start = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let value = match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&start);
            let component = numeric
                .collect(input)
                .map_err(|_| unexpected_at(location))?;
            CssSpecifiedNonNegativeNumber::try_from_component(component)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &start, numeric, CalculationRoot::Number)?;
            CssSpecifiedNonNegativeNumber::try_from_calculation(
                CssNumberCalculation::from_expression(expression),
            )
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    value
        .map(CssFontSizeAdjust::Number)
        .map_err(|error| unexpected_at(numeric.error_location(&error, location, root_offset)))
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
    let component = numeric
        .collect(input)
        .map_err(|_| unexpected_at(location))?;
    CssFontLanguageString::try_from_component(component)
        .map(CssFontLanguageOverride::String)
        .map_err(|_| unexpected_at(location))
}

pub(super) fn parse_font_optical_sizing<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssFontOpticalSizing, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssFontOpticalSizing::Auto),
        "none" => Ok(CssFontOpticalSizing::None),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}
