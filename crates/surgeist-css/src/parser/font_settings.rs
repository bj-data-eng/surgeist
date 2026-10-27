use super::property_input::Parser;
use crate::diagnostic::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::{Token, match_ignore_ascii_case};

use super::values::parse_numeric_function;
use crate::numeric::expression::CalculationRoot;
use crate::numeric::{CssCalculation, CssInteger, CssNonNegative, CssNumeric};
use crate::values::font::{
    CssAuthoredFontFeature, CssAuthoredFontFeatureList, CssAuthoredFontFeatureSettings,
    CssAuthoredFontFeatureValue, CssFontVariation, CssFontVariationList, CssFontVariationSettings,
    CssOpenTypeTag,
};

fn tag<'i, 't>(input: &mut Parser<'i, 't>) -> Result<CssOpenTypeTag, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = input.expect_string_cloned().map_err(basic)?;
    CssOpenTypeTag::try_new(value.to_string()).ok_or_else(|| unexpected_at(location))
}

pub(super) fn parse_font_feature_settings<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssAuthoredFontFeatureSettings, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        if ident.eq_ignore_ascii_case("normal") {
            if input.is_exhausted() {
                return Ok(CssAuthoredFontFeatureSettings::Normal);
            }
            // Fonts 4: `normal` is a whole value, so the token after it is
            // responsible.
            return Err(unexpected_at(input.current_source_location()));
        }
        input.reset(&state);
    }
    let features = input.parse_comma_separated(|input| parse_font_feature(input, numeric))?;
    CssAuthoredFontFeatureList::try_new(features)
        .map(CssAuthoredFontFeatureSettings::Features)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_font_feature<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssAuthoredFontFeature, ParseError<'i, Error>> {
    let tag = tag(input)?;
    // A `#` item is bounded at its comma, so exhaustion ends the item.
    let value = if input.is_exhausted() {
        CssAuthoredFontFeatureValue::Omitted
    } else if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "on" => CssAuthoredFontFeatureValue::On,
            "off" => CssAuthoredFontFeatureValue::Off,
            _ => return Err(unexpected_at(input.current_source_location())),
        }
    } else {
        CssAuthoredFontFeatureValue::Index(parse_feature_index(input, numeric)?)
    };
    Ok(CssAuthoredFontFeature::new(tag, value))
}

fn parse_feature_index<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssNumeric<CssInteger, CssNonNegative>, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let location = input.current_source_location();
    let root_site = input.position();
    let value = match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&start);
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
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    value.map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, root_site))
    })
}

pub(super) fn parse_font_variation_settings<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssFontVariationSettings, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        if ident.eq_ignore_ascii_case("normal") {
            if input.is_exhausted() {
                return Ok(CssFontVariationSettings::Normal);
            }
            // Fonts 4: `normal` is a whole value, so the token after it is
            // responsible.
            return Err(unexpected_at(input.current_source_location()));
        }
        input.reset(&state);
    }
    let variations = input.parse_comma_separated(|input| {
        let tag = tag(input)?;
        let value = super::values::parse_specified_number(input, numeric)?;
        Ok(CssFontVariation::new(tag, value))
    })?;
    CssFontVariationList::try_new(variations)
        .map(CssFontVariationSettings::Variations)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
