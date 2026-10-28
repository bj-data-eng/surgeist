use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::property_input::StreamExt;
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::values::parse_numeric_function;
use crate::numeric::expression::CalculationRoot;
use crate::numeric::{CssCalculation, CssInteger, CssNonNegative, CssNumeric};
use crate::values::font::{
    CssAuthoredFontFeature, CssAuthoredFontFeatureList, CssAuthoredFontFeatureSettings,
    CssAuthoredFontFeatureValue, CssFontVariation, CssFontVariationList, CssFontVariationSettings,
    CssOpenTypeTag,
};

fn tag<'i>(input: &mut Stream<'i>) -> Result<CssOpenTypeTag, SyntaxError<'i, Error>> {
    let location = input.site();
    let value = input.consume_string().map_err(basic)?;
    CssOpenTypeTag::try_new(value.to_string()).ok_or_else(|| unexpected_at(location))
}

pub(super) fn parse_font_feature_settings<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssAuthoredFontFeatureSettings, SyntaxError<'i, Error>> {
    let state = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        if ident.eq_ignore_ascii_case("normal") {
            if input.only_whitespace_remains() {
                return Ok(CssAuthoredFontFeatureSettings::Normal);
            }
            // Fonts 4: `normal` is a whole value, so the token after it is
            // responsible.
            return Err(unexpected_at(input.site()));
        }
        input.restore(state);
    }
    let features = input.comma_segments(|input| parse_font_feature(input, numeric))?;
    CssAuthoredFontFeatureList::try_new(features)
        .map(CssAuthoredFontFeatureSettings::Features)
        .ok_or_else(|| unexpected_at(input.site()))
}

fn parse_font_feature<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssAuthoredFontFeature, SyntaxError<'i, Error>> {
    let tag = tag(input)?;
    // A `#` item is bounded at its comma, so exhaustion ends the item.
    let value = if input.only_whitespace_remains() {
        CssAuthoredFontFeatureValue::Omitted
    } else if let Ok(ident) = input.attempt(Stream::consume_ident) {
        match_ignore_ascii_case! { &ident,
            "on" => CssAuthoredFontFeatureValue::On,
            "off" => CssAuthoredFontFeatureValue::Off,
            _ => return Err(unexpected_at(input.site())),
        }
    } else {
        CssAuthoredFontFeatureValue::Index(parse_feature_index(input, numeric)?)
    };
    Ok(CssAuthoredFontFeature::new(tag, value))
}

fn parse_feature_index<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssNumeric<CssInteger, CssNonNegative>, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let start = input.mark();
    let location = input.site();
    let root_site = input.site();
    let value = match input.consume_after_whitespace().map_err(basic)? {
        Token::Number(_) => {
            input.restore(start);
            let component = input
                .collect_numeric_component(numeric)
                .map_err(|_| unexpected_at(location))?;
            CssNumeric::<CssInteger, CssNonNegative>::try_from_component(component)
        }
        Token::Function(name) if crate::numeric::expression::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &start, numeric, CalculationRoot::Integer)?;
            CssNumeric::<CssInteger, CssNonNegative>::from_parser_calculation(CssCalculation::<
                CssInteger,
            >::from_expression(
                expression
            ))
        }
        token => return Err(location.unexpected::<Error>(token)),
    };
    value.map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, root_site))
    })
}

pub(super) fn parse_font_variation_settings<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssFontVariationSettings, SyntaxError<'i, Error>> {
    let state = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        if ident.eq_ignore_ascii_case("normal") {
            if input.only_whitespace_remains() {
                return Ok(CssFontVariationSettings::Normal);
            }
            // Fonts 4: `normal` is a whole value, so the token after it is
            // responsible.
            return Err(unexpected_at(input.site()));
        }
        input.restore(state);
    }
    let variations = input.comma_segments(|input| {
        let tag = tag(input)?;
        let value = super::values::parse_specified_number(input, numeric)?;
        Ok(CssFontVariation::new(tag, value))
    })?;
    CssFontVariationList::try_new(variations)
        .map(CssFontVariationSettings::Variations)
        .ok_or_else(|| unexpected_at(input.site()))
}
