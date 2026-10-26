use super::property_input::Parser;
use crate::error::property::ParseError;
use cssparser::{Token, match_ignore_ascii_case};
use surgeist_color::{ColorSpace, HueInterpolation};

use super::values::parse_numeric_function;
use crate::error::property::{Error, basic, invalid_color, unexpected_at};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::syntax::*;

pub(super) fn parse_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    input.skip_whitespace();
    let root = input.position();
    let component = input.collect_component().map_err(|error| {
        let location = root.source_origin(error.origin()).unwrap_or(root);
        crate::error::property::invalid_component_value(location, error)
    })?;
    crate::color::authored::construct_component(component.as_ref(), numeric).map_err(|failure| {
        let mut path = Vec::with_capacity(failure.path.len() + 1);
        path.push(0);
        path.extend_from_slice(&failure.path);
        let site = root.descendant(&path, failure.closing).unwrap_or(root);
        crate::error::property::error_at(site, failure.kind)
    })
}

pub(super) fn parse_color_interpolation_method<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolation, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(name) = input.expect_ident_cloned()
        && let Some(profile) = CssColorProfileName::try_new(name.as_ref())
    {
        return Ok(CssColorInterpolation::custom(profile));
    }
    input.reset(&state);
    let space = parse_color_interpolation_space(input)?;
    let hue_location = input.current_source_location();
    let hue = input.try_parse(parse_hue_interpolation).ok();
    let method = CssColorInterpolationMethod::try_new(space, hue)
        .map_err(|_| invalid_color(hue_location, Some("hue interpolation")))?;
    Ok(CssColorInterpolation::from_predefined(method))
}

pub(super) fn next_is_mix_weight(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let result = match input.next() {
        Ok(Token::Percentage { .. }) => true,
        Ok(Token::Function(name)) => is_math_function(name),
        _ => false,
    };
    input.reset(&state);
    result
}

pub(super) fn parse_authored_mix_weight<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssColorMixWeight, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    if matches!(input.next(), Ok(Token::Function(_))) {
        let expression =
            parse_numeric_function(input, &state, numeric, CalculationRoot::Percentage)?;
        return CssColorMixWeight::try_calculation(
            CssCalculation::<CssPercentage>::from_expression(expression),
        )
        .map_err(|error| crate::error::property::invalid_component_value(location, error));
    }
    input.reset(&state);
    parse_authored_color_mix_percentage(input, numeric).map(CssColorMixWeight::literal)
}

fn parse_authored_color_mix_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColorMixPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let component = collect_color_scalar(input, numeric)?;
    CssColorMixPercentage::try_from_component(component)
        .map_err(|_| invalid_color(location, Some("color-mix component percentage")))
}

fn collect_color_scalar<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::CssComponentValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    let offset = input.position();
    let location = input.current_source_location();
    input.collect_numeric_component(numeric).map_err(|error| {
        let location = numeric.property_error_location(&error, location, offset);
        if let Some(component) = error.component_error() {
            crate::error::property::invalid_component_value(location, component.clone())
        } else {
            invalid_color(location, Some("component"))
        }
    })
}

fn parse_color_interpolation_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<ColorSpace, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let space = if let Some(space) = crate::syntax::color::predefined_color_space(&ident) {
        ColorSpace::Predefined(space)
    } else {
        match_ignore_ascii_case! { &ident,
            "hsl" => ColorSpace::Hsl,
            "hwb" => ColorSpace::Hwb,
            "lab" => ColorSpace::Lab,
            "lch" => ColorSpace::Lch,
            "oklab" => ColorSpace::Oklab,
            "oklch" => ColorSpace::Oklch,
            _ => return Err(unexpected_at(location)),
        }
    };
    Ok(space)
}

fn parse_hue_interpolation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<HueInterpolation, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let hue = match_ignore_ascii_case! { &ident,
        "shorter" => HueInterpolation::Shorter,
        "longer" => HueInterpolation::Longer,
        "increasing" => HueInterpolation::Increasing,
        "decreasing" => HueInterpolation::Decreasing,
        _ => return Err(unexpected_at(location)),
    };
    input.expect_ident_matching("hue").map_err(basic)?;
    Ok(hue)
}
