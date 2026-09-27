use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{CalculationRoot, next_is_comma, parse_numeric_function};
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::validation::unsupported_keyword_reason;
use crate::{
    CssAuthoredFontFeature, CssAuthoredFontFeatureList, CssAuthoredFontFeatureSettings,
    CssAuthoredFontFeatureValue, CssFontFeatureIndex, CssFontVariation, CssFontVariationList,
    CssFontVariationSettings, CssIntegerCalculation, CssNumberCalculation, CssOpenTypeTag,
    CssSpecifiedNumber,
};

fn tag<'i, 't>(input: &mut Parser<'i, 't>) -> Result<CssOpenTypeTag, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = input.expect_string_cloned().map_err(basic)?;
    CssOpenTypeTag::try_new(value.to_string()).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            "OpenType tag requires four printable ASCII characters",
        )
    })
}

pub(super) fn parse_font_feature_settings<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssAuthoredFontFeatureSettings, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        if ident.eq_ignore_ascii_case("normal") && input.is_exhausted() {
            return Ok(CssAuthoredFontFeatureSettings::Normal);
        }
        input.reset(&state);
    }
    let mut features = Vec::new();
    loop {
        features.push(parse_font_feature(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font-feature-settings list has an empty item",
            ));
        }
    }
    CssAuthoredFontFeatureList::try_new(features)
        .map(CssAuthoredFontFeatureSettings::Features)
        .ok_or_else(|| unsupported_value(input, None, "font-feature-settings list is empty"))
}

fn parse_font_feature<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssAuthoredFontFeature, ParseError<'i, Error>> {
    let tag = tag(input)?;
    let value = if input.is_exhausted() || next_is_comma(input) {
        CssAuthoredFontFeatureValue::Omitted
    } else if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "on" => CssAuthoredFontFeatureValue::On,
            "off" => CssAuthoredFontFeatureValue::Off,
            _ => return Err(unsupported_value(input, None, unsupported_keyword_reason("font feature value", ident.as_ref()))),
        }
    } else {
        CssAuthoredFontFeatureValue::Index(parse_feature_index(input, numeric)?)
    };
    Ok(CssAuthoredFontFeature::new(tag, value))
}

fn parse_feature_index<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssFontFeatureIndex, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let value = match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&start);
            let component = numeric.collect(input).map_err(|_| {
                unsupported_value_at(location, None, "invalid font feature integer")
            })?;
            CssFontFeatureIndex::try_from_component(component)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &start, numeric, CalculationRoot::Integer)?;
            CssFontFeatureIndex::try_from_calculation(CssIntegerCalculation::from_expression(
                expression,
            ))
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    value.map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "font feature index requires a nonnegative integer",
        )
    })
}

pub(super) fn parse_font_variation_settings<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssFontVariationSettings, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        if ident.eq_ignore_ascii_case("normal") && input.is_exhausted() {
            return Ok(CssFontVariationSettings::Normal);
        }
        input.reset(&state);
    }
    let mut variations = Vec::new();
    loop {
        let tag = tag(input)?;
        let value = parse_variation_number(input, numeric)?;
        variations.push(CssFontVariation::new(tag, value));
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "font-variation-settings list has an empty item",
            ));
        }
    }
    CssFontVariationList::try_new(variations)
        .map(CssFontVariationSettings::Variations)
        .ok_or_else(|| unsupported_value(input, None, "font-variation-settings list is empty"))
}

fn parse_variation_number<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedNumber, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let location = input.current_source_location();
    let root_offset = input.position().byte_index();
    let value = match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&start);
            let component = numeric
                .collect(input)
                .map_err(|_| unsupported_value_at(location, None, "invalid variation number"))?;
            CssSpecifiedNumber::try_from_component(component)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            let expression =
                parse_numeric_function(input, &start, numeric, CalculationRoot::Number)?;
            CssSpecifiedNumber::try_from_calculation(CssNumberCalculation::from_expression(
                expression,
            ))
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    value.map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, root_offset),
            None,
            "variation axis requires a number",
        )
    })
}
