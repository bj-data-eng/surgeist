use super::syntax_types::Stream;
use crate::parser::property_input::{SiteExt, StreamExt};
use crate::parser::syntax_types::SyntaxError;
use surgeist_color::{ColorSpace, HueInterpolation};
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::values::parse_numeric_function;
use crate::color::{
    CssColor, CssColorInterpolation, CssColorInterpolationMethod, CssColorMixPercentage,
    CssColorMixWeight, CssColorProfileName,
};
use crate::diagnostic::error::property::{Error, basic, invalid_color, unexpected_at};
use crate::numeric::expression::is_math_function;
use crate::numeric::expression::{CalculationRoot, NumericInputContext};
use crate::numeric::{CssCalculation, CssPercentage};

pub(crate) fn parse_color<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColor, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let root = input.site();
    let component = input.collect_component().map_err(|error| {
        let location = root.source_origin(error.origin()).unwrap_or(root);
        crate::diagnostic::error::property::invalid_component_value(location, error)
    })?;
    crate::color::authored::construct_component(component.as_ref(), numeric).map_err(|failure| {
        let mut path = Vec::with_capacity(failure.path.len() + 1);
        path.push(0);
        path.extend_from_slice(&failure.path);
        let site = root.descendant(&path, failure.closing).unwrap_or(root);
        crate::diagnostic::error::property::error_at(site, failure.kind)
    })
}

pub(super) fn parse_color_interpolation_method<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssColorInterpolation, SyntaxError<'i, Error>> {
    let state = input.mark();
    if let Ok(name) = input.consume_ident()
        && let Some(profile) = CssColorProfileName::try_new(name)
    {
        return Ok(CssColorInterpolation::custom(profile));
    }
    input.restore(state);
    let space = parse_color_interpolation_space(input)?;
    let hue_location = input.site();
    let hue = input.attempt(parse_hue_interpolation).ok();
    let method = CssColorInterpolationMethod::try_new(space, hue)
        .map_err(|_| invalid_color(hue_location, Some("hue interpolation")))?;
    Ok(CssColorInterpolation::from_predefined(method))
}

pub(super) fn next_is_mix_weight(input: &mut Stream<'_>) -> bool {
    let state = input.mark();
    let result = match input.consume_after_whitespace() {
        Ok(Token::Percentage(_)) => true,
        Ok(Token::Function(name)) => is_math_function(name),
        _ => false,
    };
    input.restore(state);
    result
}

pub(super) fn parse_authored_mix_weight<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssColorMixWeight, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let state = input.mark();
    let location = input.site();
    if matches!(input.consume_after_whitespace(), Ok(Token::Function(_))) {
        let expression =
            parse_numeric_function(input, &state, numeric, CalculationRoot::Percentage)?;
        return CssColorMixWeight::try_calculation(
            CssCalculation::<CssPercentage>::from_expression(expression),
        )
        .map_err(|error| {
            crate::diagnostic::error::property::invalid_component_value(location, error)
        });
    }
    input.restore(state);
    parse_authored_color_mix_percentage(input, numeric).map(CssColorMixWeight::literal)
}

fn parse_authored_color_mix_percentage<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColorMixPercentage, SyntaxError<'i, Error>> {
    let location = input.site();
    let component = collect_color_scalar(input, numeric)?;
    CssColorMixPercentage::try_from_component(component)
        .map_err(|_| invalid_color(location, Some("color-mix component percentage")))
}

fn collect_color_scalar<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::component::CssComponentValue, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let offset = input.site();
    let location = input.site();
    input.collect_numeric_component(numeric).map_err(|error| {
        let location = numeric.property_error_location(&error, location, offset);
        if let Some(component) = error.component_error() {
            crate::diagnostic::error::property::invalid_component_value(location, component.clone())
        } else {
            invalid_color(location, Some("component"))
        }
    })
}

fn parse_color_interpolation_space<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<ColorSpace, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    let space =
        if let Some(space) = crate::color::syntax::recognition::predefined_color_space(ident) {
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

fn parse_hue_interpolation<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<HueInterpolation, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    let hue = match_ignore_ascii_case! { &ident,
        "shorter" => HueInterpolation::Shorter,
        "longer" => HueInterpolation::Longer,
        "increasing" => HueInterpolation::Increasing,
        "decreasing" => HueInterpolation::Decreasing,
        _ => return Err(unexpected_at(location)),
    };
    input.consume_keyword("hue").map_err(basic)?;
    Ok(hue)
}
