use std::borrow::Cow;

use super::color::{parse_color, parse_color_interpolation_method};
use super::combinators::{Combinator, Tail, attempt, commit, fill, parse_group};
use super::property_input::Parser;
use super::values::{
    parse_length_percentage, parse_nonnegative_length, parse_nonnegative_length_percentage,
};
use crate::error::property::{CssFeatureId, Error, ParseError, basic, unexpected_at};
use cssparser::{Token, match_ignore_ascii_case};

use super::position::{
    next_starts_background_position, parse_background_position_prefix, parse_full_position_bounded,
};
use super::url::parse_url;
use super::values::{
    CalculationRoot, next_is_comma, parse_hinted_number_calculation, parse_nonnegative_number,
    parse_nonnegative_percentage,
};
use crate::syntax::*;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.position"),
    CssFeatureId::new("official.value.background-position"),
    CssFeatureId::new("official.value.background-layer"),
    CssFeatureId::new("official.value.background-image"),
    CssFeatureId::new("official.value.repeat-style"),
    CssFeatureId::new("official.value.background-attachment"),
    CssFeatureId::new("official.value.background-size"),
    CssFeatureId::new("official.value.line-style"),
    CssFeatureId::new("official.value.line-width"),
    CssFeatureId::new("official.value.image"),
    CssFeatureId::new("ext.value.filter-image"),
    CssFeatureId::new("official.value.gradient"),
    CssFeatureId::new("official.value.linear-gradient"),
    CssFeatureId::new("official.value.radial-gradient"),
    CssFeatureId::new("official.value.repeating-linear-gradient"),
    CssFeatureId::new("official.value.repeating-radial-gradient"),
    CssFeatureId::new("ext.value.conic-gradient"),
    CssFeatureId::new("ext.value.gradient-interpolation"),
    CssFeatureId::new("ext.value.gradient-stop-list"),
    CssFeatureId::new("required.value.gradient-position"),
    CssFeatureId::new("official.value.color-stop-list"),
    CssFeatureId::new("official.value.side-or-corner"),
    CssFeatureId::new("official.value.radial-shape"),
    CssFeatureId::new("official.value.radial-size"),
    CssFeatureId::new("official.value.radial-extent"),
    CssFeatureId::new("interop.value.light-dark-image"),
];

pub(super) fn parse_image_layer_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssImageValueList, ParseError<'i, Error>> {
    let images = input.parse_comma_separated(|input| parse_image_value(input, numeric))?;
    CssImageValueList::try_new(images).ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_background<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackground, ParseError<'i, Error>> {
    // List post-condition: only the final layer may carry the color, so a
    // color is rejected as soon as another layer follows it.
    let mut previous_color = None;
    let layers = input.parse_comma_separated(|input| {
        if let Some(location) = previous_color {
            return Err(unexpected_at(location));
        }
        let (layer, color_location) = parse_background_layer(input, numeric)?;
        previous_color = color_location;
        Ok(layer)
    })?;

    CssBackground::try_new(layers).map_err(|_| unexpected_at(input.current_source_location()))
}

fn parse_background_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<
    (CssBackgroundLayer, Option<super::property_input::Site<'i>>),
    ParseError<'i, Error>,
> {
    // `<bg-image> || <bg-position> [ / <bg-size> ]? || <repeat-style> ||
    // <attachment> || <visual-box> || <visual-box> || <'background-color'>`
    // (the color only in the final layer, checked by the list). The list
    // bounds each layer at its comma.
    let (mut image, mut position, mut size, mut repeat, mut attachment) =
        (None, None, None, None, None);
    let (mut origin, mut clip, mut color, mut color_location) = (None, None, None, None);
    parse_group::<7>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => {
                if !next_starts_background_image(input) {
                    return Ok(false);
                }
                // A light-dark() that is not an image may still be the color.
                match input.try_parse(|input| parse_image_value(input, numeric)) {
                    Ok(value) => {
                        image = Some(value);
                        Ok(true)
                    }
                    Err(error) if crate::error::property::is_resource_parse_error(&error) => {
                        Err(error)
                    }
                    Err(_) if next_is_light_dark(input) => Ok(false),
                    Err(error) => Err(error),
                }
            }
            1 => {
                if !commit(
                    &mut position,
                    input,
                    next_starts_background_position,
                    |input| parse_background_position_prefix(input, numeric),
                )? {
                    return Ok(false);
                }
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    size = Some(parse_background_size(input, numeric)?);
                }
                Ok(true)
            }
            2 => commit(
                &mut repeat,
                input,
                next_starts_background_repeat,
                parse_background_repeat,
            ),
            3 => fill(&mut attachment, input, parse_background_attachment),
            4 => fill(&mut origin, input, parse_background_box),
            5 => fill(&mut clip, input, parse_background_box),
            6 => {
                let location = input.current_source_location();
                let matched = fill(&mut color, input, |input| parse_color(input, numeric))?;
                if matched {
                    color_location = Some(location);
                }
                Ok(matched)
            }
            _ => unreachable!("seven background layer operands"),
        },
    )?;

    let boxes = match (origin, clip) {
        (None, _) => None,
        (Some(value), None) => Some(CssBackgroundLayerBoxes::One(value)),
        (Some(origin), Some(clip)) => Some(CssBackgroundLayerBoxes::OriginAndClip { origin, clip }),
    };
    Ok((
        CssBackgroundLayer::try_new(image, position, size, repeat, attachment, boxes, color)
            .map_err(|_| unexpected_at(input.current_source_location()))?,
        color_location,
    ))
}

pub(super) fn next_starts_background_image<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("none"),
        Ok(Token::UnquotedUrl(_)) => true,
        Ok(Token::Function(name)) => {
            name.eq_ignore_ascii_case("url")
                || name.eq_ignore_ascii_case("light-dark")
                || name.eq_ignore_ascii_case("filter")
                || name.eq_ignore_ascii_case("src")
                || matches!(
                    name.to_ascii_lowercase().as_str(),
                    "linear-gradient"
                        | "repeating-linear-gradient"
                        | "radial-gradient"
                        | "repeating-radial-gradient"
                        | "conic-gradient"
                )
        }
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

fn next_starts_background_repeat<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = matches!(
        input.next(),
        Ok(Token::Ident(value))
            if matches!(
                value.to_ascii_lowercase().as_str(),
                "repeat-x" | "repeat-y" | "repeat" | "space" | "round" | "no-repeat"
            )
    );
    input.reset(&state);
    starts
}

/// One `<repeat-style>`. A following component that is not a second repeat
/// keyword is left to the caller's boundary (list comma or shorthand group).
pub(super) fn parse_background_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundRepeat, ParseError<'i, Error>> {
    let first = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &first,
        "repeat-x" => Ok(CssBackgroundRepeat::RepeatX),
        "repeat-y" => Ok(CssBackgroundRepeat::RepeatY),
        _ => {
            let x = parse_background_repeat_style_from_ident(input, first.as_ref())?;
            let y = attempt(input, |input| {
                let second = input.expect_ident_cloned().map_err(basic)?;
                parse_background_repeat_style_from_ident(input, second.as_ref())
            })?
            .unwrap_or(x);
            Ok(CssBackgroundRepeat::Axes { x, y })
        }
    }
}

/// One `<bg-size>`. A following component that is not a second size is left
/// to the caller's boundary (list comma or shorthand group).
pub(super) fn parse_background_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSize, ParseError<'i, Error>> {
    let width = if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "cover" => return Ok(CssBackgroundSize::Cover),
            "contain" => return Ok(CssBackgroundSize::Contain),
            "auto" => CssBackgroundSizeComponent::Auto,
            _ => return Err(unexpected_at(input.current_source_location())),
        }
    } else {
        parse_background_size_component(input, numeric)?
    };
    let height = attempt(input, |input| {
        parse_background_size_component(input, numeric)
    })?;
    Ok(CssBackgroundSize::Explicit { width, height })
}

/// Parses one image-only value through the canonical complete graph boundary.
pub(super) fn parse_image<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssImage, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let start = input.position();
    let value = parse_image_value(input, numeric)?;
    CssImage::try_new(value).map_err(|error| image_construction_error(error, location, start))
}

// All image construction entry points preserve resource failures and the
// original image token origin; semantic None rejection remains grammar-owned.
fn image_construction_error<'i>(
    error: CssImageConstructionError,
    location: super::property_input::Site<'i>,
    start: super::property_input::Site<'i>,
) -> ParseError<'i, Error> {
    let kind = match error {
        CssImageConstructionError::NotImage => {
            return unexpected_at(location);
        }
        CssImageConstructionError::NestingLimit => crate::CssComponentValueErrorKind::NestingLimit,
        CssImageConstructionError::CapacityOverflow => {
            crate::CssComponentValueErrorKind::CapacityOverflow
        }
    };
    crate::error::property::invalid_component_value(
        location,
        crate::CssComponentValueError::new(kind, start.value_origin().expect("image token origin")),
    )
}

pub(super) fn parse_image_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssImageValue, ParseError<'i, Error>> {
    let state = input.state();
    let filter =
        matches!(input.next(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case("filter"));
    input.reset(&state);
    if filter {
        input.skip_whitespace();
        let location = input.current_source_location();
        let start = input.position();
        input.next().map_err(basic)?;
        return input.parse_nested_block(|input| {
            let state = input.state();
            let string = matches!(input.next(), Ok(Token::QuotedString(_)));
            input.reset(&state);
            let input_value = if string {
                // Collect the exact original String rather than decoded text re-tokenization.
                input.skip_whitespace();
                let string_location = input.current_source_location();
                let string_start = input.position();
                let component = input.collect_numeric_component(numeric).map_err(|error| {
                    let location =
                        numeric.property_error_location(&error, string_location, string_start);
                    error.component_error().map_or_else(
                        || unexpected_at(location),
                        |detail| {
                            crate::error::property::invalid_component_value(
                                location,
                                detail.clone(),
                            )
                        },
                    )
                })?;
                let string =
                    CssFilterImageString::try_from_component(component).map_err(|error| {
                        crate::error::property::invalid_component_value(string_location, error)
                    })?;
                CssFilterImageInput::from_string(string)
            } else {
                CssFilterImageInput::from_image(parse_image(input, numeric)?)
            };
            input.expect_comma().map_err(basic)?;
            let filters = super::effects::parse_filter_function_list(input, numeric)?;
            CssFilterImage::try_new(input_value, filters)
                .map(|value| CssImageValue::Filter(Box::new(value)))
                .map_err(|error| image_construction_error(error, location, start))
        });
    }
    if next_is_light_dark(input) {
        input.skip_whitespace();
        let location = input.current_source_location();
        let start = input.position();
        input.next().map_err(basic)?;
        return input.parse_nested_block(|input| {
            let light = parse_image_value(input, numeric)?;
            input.expect_comma().map_err(basic)?;
            let dark = parse_image_value(input, numeric)?;
            input.expect_exhausted().map_err(basic)?;
            CssLightDarkImage::try_new(light, dark)
                .map(|value| CssImageValue::LightDark(Box::new(value)))
                .map_err(|error| image_construction_error(error, location, start))
        });
    }
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssImageValue::None);
    }
    if next_is_gradient(input) {
        return parse_gradient(input, numeric).map(CssImageValue::Gradient);
    }
    parse_url(input, numeric).map(CssImageValue::Url)
}

fn next_is_light_dark(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let result = matches!(input.next(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case("light-dark"));
    input.reset(&state);
    result
}

pub(super) fn parse_border_image_source<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssImageValue, ParseError<'i, Error>> {
    parse_image_value(input, numeric)
}

pub(super) fn parse_border_image_slice<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageSlice, ParseError<'i, Error>> {
    let mut values = Vec::new();
    let mut fill = input
        .try_parse(|input| input.expect_ident_matching("fill"))
        .is_ok();

    // The repeated numeric group is one && component; fill cannot divide it.
    while !input.is_exhausted() && values.len() < 4 {
        match input.try_parse(|input| parse_border_image_slice_component(input, numeric)) {
            Ok(value) => values.push(value),
            Err(_) => break,
        }
    }

    if !fill
        && input
            .try_parse(|input| input.expect_ident_matching("fill"))
            .is_ok()
    {
        fill = true;
    }
    CssBorderImageSlice::try_new(values, fill)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_border_image_slice_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageSliceComponent, ParseError<'i, Error>> {
    if let Ok(calculation) =
        input.try_parse(|input| parse_hinted_number_calculation(input, numeric))
    {
        return Ok(CssBorderImageSliceComponent::HintedNumberCalculation(
            calculation,
        ));
    }
    if let Ok(number) = input.try_parse(|input| parse_nonnegative_number(input, numeric)) {
        return Ok(CssBorderImageSliceComponent::Number(number));
    }
    parse_nonnegative_percentage(input, numeric).map(CssBorderImageSliceComponent::Percentage)
}

/// `[ <length-percentage [0,∞]> | <number [0,∞]> | auto ]{1,4}`, leaving a
/// following component to the caller's boundary.
pub(super) fn parse_border_image_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageWidth, ParseError<'i, Error>> {
    let values = repeat_up_to(input, 4, |input| {
        parse_border_image_width_component(input, numeric)
    })?;
    CssBorderImageWidth::try_new(values)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

/// `<item>{1,max}` as optional repetitions under the shared failure policy;
/// the caller's model rejects an empty result.
fn repeat_up_to<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    max: usize,
    mut parse: impl FnMut(&mut Parser<'i, 't>) -> Result<T, ParseError<'i, Error>>,
) -> Result<Vec<T>, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while values.len() < max
        && let Some(value) = attempt(input, &mut parse)?
    {
        values.push(value);
    }
    Ok(values)
}

fn parse_border_image_width_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageWidthComponent, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssBorderImageWidthComponent::Auto);
    }
    if let Ok(calculation) =
        input.try_parse(|input| parse_hinted_number_calculation(input, numeric))
    {
        return Ok(CssBorderImageWidthComponent::HintedNumberCalculation(
            calculation,
        ));
    }
    if let Ok(number) = input.try_parse(|input| parse_nonnegative_number(input, numeric)) {
        return Ok(CssBorderImageWidthComponent::Number(number));
    }
    parse_nonnegative_length_percentage(input, numeric)
        .map(CssBorderImageWidthComponent::LengthPercentage)
}

pub(super) fn parse_border_image_outset<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageOutset, ParseError<'i, Error>> {
    // `[ <length [0,∞]> | <number [0,∞]> ]{1,4}`, leaving a following
    // component to the caller's boundary.
    let values = repeat_up_to(input, 4, |input| {
        parse_border_image_outset_component(input, numeric)
    })?;
    CssBorderImageOutset::try_new(values)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_border_image_outset_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageOutsetComponent, ParseError<'i, Error>> {
    if let Ok(number) = input.try_parse(|input| parse_nonnegative_number(input, numeric)) {
        return Ok(CssBorderImageOutsetComponent::Number(number));
    }
    parse_nonnegative_length(input, numeric).map(CssBorderImageOutsetComponent::Length)
}

pub(super) fn parse_border_image_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageRepeat, ParseError<'i, Error>> {
    // `[ stretch | repeat | round | space ]{1,2}`, leaving a following
    // component to the caller's boundary.
    let horizontal = parse_border_image_repeat_keyword(input)?;
    let vertical = attempt(input, parse_border_image_repeat_keyword)?.unwrap_or(horizontal);
    Ok(CssBorderImageRepeat::new(horizontal, vertical))
}

fn parse_border_image_repeat_keyword<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageRepeatKeyword, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "stretch" => Ok(CssBorderImageRepeatKeyword::Stretch),
        "repeat" => Ok(CssBorderImageRepeatKeyword::Repeat),
        "round" => Ok(CssBorderImageRepeatKeyword::Round),
        "space" => Ok(CssBorderImageRepeatKeyword::Space),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn next_starts_border_image_repeat<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = matches!(
        input.next(),
        Ok(Token::Ident(value))
            if matches!(
                value.to_ascii_lowercase().as_str(),
                "stretch" | "repeat" | "round" | "space"
            )
    );
    input.reset(&state);
    starts
}

pub(super) fn next_starts_border_image_slice<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Number { .. } | Token::Percentage { .. }) => true,
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("fill"),
        Ok(Token::Function(name)) => crate::numeric::is_math_function(name),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

pub(super) fn parse_border_image<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImage, ParseError<'i, Error>> {
    // `<'border-image-source'> || <'border-image-slice'> [ /
    // <'border-image-width'> | / <'border-image-width'>? /
    // <'border-image-outset'> ]? || <'border-image-repeat'>`.
    let (mut source, mut slice, mut width, mut outset, mut repeat) = (None, None, None, None, None);
    parse_group::<3>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => commit(&mut source, input, next_starts_background_image, |input| {
                parse_image_value(input, numeric)
            }),
            1 => {
                if !commit(&mut slice, input, next_starts_border_image_slice, |input| {
                    parse_border_image_slice(input, numeric)
                })? {
                    return Ok(false);
                }
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                        outset = Some(parse_border_image_outset(input, numeric)?);
                    } else {
                        width = Some(parse_border_image_width(input, numeric)?);
                        if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                            outset = Some(parse_border_image_outset(input, numeric)?);
                        }
                    }
                }
                Ok(true)
            }
            2 => commit(
                &mut repeat,
                input,
                next_starts_border_image_repeat,
                parse_border_image_repeat,
            ),
            _ => unreachable!("three border-image operands"),
        },
    )?;

    CssBorderImage::try_new(source, slice, width, outset, repeat)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_image_orientation<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssImageOrientation, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("from-image"))
        .is_ok()
    {
        return Ok(CssImageOrientation::FromImage);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssImageOrientation::None);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("flip"))
        .is_ok()
    {
        let angle = if input.is_exhausted() {
            None
        } else {
            Some(super::values::parse_angle_value(input, numeric)?)
        };
        return Ok(CssImageOrientation::Flip(angle));
    }
    let angle = super::values::parse_angle_value(input, numeric)?;
    if input
        .try_parse(|input| input.expect_ident_matching("flip"))
        .is_ok()
    {
        Ok(CssImageOrientation::Flip(Some(angle)))
    } else {
        Ok(CssImageOrientation::Angle(angle))
    }
}

pub(super) fn parse_image_rendering<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssImageRendering, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssImageRendering::Auto),
        "smooth" => Ok(CssImageRendering::Smooth),
        "high-quality" => Ok(CssImageRendering::HighQuality),
        "crisp-edges" => Ok(CssImageRendering::CrispEdges),
        "pixelated" => Ok(CssImageRendering::Pixelated),
        "optimizespeed" => Ok(CssImageRendering::OptimizeSpeed),
        "optimizequality" => Ok(CssImageRendering::OptimizeQuality),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_object_fit<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssObjectFit, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "fill" => Ok(CssObjectFit::Fill),
        "contain" => Ok(CssObjectFit::Contain),
        "cover" => Ok(CssObjectFit::Cover),
        "none" => Ok(CssObjectFit::None),
        "scale-down" => Ok(CssObjectFit::ScaleDown),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

fn next_is_gradient<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_gradient = matches!(
        input.next(),
        Ok(Token::Function(name))
            if matches!(
                name.to_ascii_lowercase().as_str(),
                "linear-gradient"
                    | "repeating-linear-gradient"
                    | "radial-gradient"
                    | "repeating-radial-gradient"
                    | "conic-gradient"
            )
    );
    input.reset(&state);
    is_gradient
}

fn parse_gradient<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGradient, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = input.expect_function().map_err(basic)?.to_ascii_lowercase();
    match name.as_str() {
        "linear-gradient" => input
            .parse_nested_block(|input| parse_linear_gradient(input, numeric))
            .map(CssGradient::Linear),
        "repeating-linear-gradient" => input
            .parse_nested_block(|input| parse_linear_gradient(input, numeric))
            .map(CssGradient::RepeatingLinear),
        "radial-gradient" => input
            .parse_nested_block(|input| parse_radial_gradient(input, numeric))
            .map(CssGradient::Radial),
        "repeating-radial-gradient" => input
            .parse_nested_block(|input| parse_radial_gradient(input, numeric))
            .map(CssGradient::RepeatingRadial),
        "conic-gradient" => input
            .parse_nested_block(|input| parse_conic_gradient(input, numeric))
            .map(CssGradient::Conic),
        _ => Err(unexpected_at(location)),
    }
}

fn parse_linear_gradient<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssLinearGradient, ParseError<'i, Error>> {
    let mut interpolation = parse_optional_gradient_interpolation(input)?;
    let direction = next_starts_linear_gradient_direction(input)
        .then(|| parse_linear_gradient_direction(input, numeric))
        .transpose()?;
    if interpolation.is_none() {
        interpolation = parse_optional_gradient_interpolation(input)?;
    }
    if direction.is_some() || interpolation.is_some() {
        input.expect_comma().map_err(basic)?;
    }
    let stops = parse_color_stop_list(input, numeric)?;
    Ok(CssLinearGradient::new(direction, stops).with_interpolation(interpolation))
}

fn parse_optional_gradient_interpolation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Option<CssColorInterpolation>, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("in"))
        .is_ok()
    {
        parse_color_interpolation_method(input).map(Some)
    } else {
        Ok(None)
    }
}

fn next_starts_linear_gradient_direction<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("to"),
        Ok(Token::Number { .. } | Token::Dimension { .. }) => true,
        Ok(Token::Function(name)) => crate::numeric::is_math_function(name),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

fn parse_linear_gradient_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssLinearGradientDirection, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("to"))
        .is_ok()
    {
        return parse_side_or_corner(input).map(CssLinearGradientDirection::SideOrCorner);
    }
    super::values::parse_angle_or_zero(input, numeric).map(CssLinearGradientDirection::Angle)
}

fn parse_side_or_corner<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssSideOrCorner, ParseError<'i, Error>> {
    let start = input.current_source_location();
    let mut horizontal = None;
    let mut vertical = None;
    for _ in 0..2 {
        if super::values::next_is_ident(input, "in") {
            break;
        }
        let location = input.current_source_location();
        let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) else {
            break;
        };
        match_ignore_ascii_case! { &ident,
            "left" if horizontal.is_none() => {
                horizontal = Some(CssHorizontalGradientSide::Left);
            },
            "right" if horizontal.is_none() => {
                horizontal = Some(CssHorizontalGradientSide::Right);
            },
            "top" if vertical.is_none() => {
                vertical = Some(CssVerticalGradientSide::Top);
            },
            "bottom" if vertical.is_none() => {
                vertical = Some(CssVerticalGradientSide::Bottom);
            },
            _ => return Err(unexpected_at(location)),
        }
    }
    CssSideOrCorner::try_new(horizontal, vertical).ok_or_else(|| unexpected_at(start))
}

fn parse_color_stop_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssColorStopList, ParseError<'i, Error>> {
    let mut items = vec![CssColorStopListItem::Stop(Box::new(
        parse_gradient_color_stop(input, numeric)?,
    ))];
    while input.try_parse(Parser::expect_comma).is_ok() {
        if input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
        if let Ok(hint) =
            input.try_parse(|input| -> std::result::Result<_, ParseError<'i, Error>> {
                let hint = parse_gradient_line_position(input, numeric)?;
                input.expect_comma().map_err(basic)?;
                Ok(hint)
            })
        {
            items.push(CssColorStopListItem::Hint(hint));
        }
        items.push(CssColorStopListItem::Stop(Box::new(
            parse_gradient_color_stop(input, numeric)?,
        )));
    }
    CssColorStopList::try_new(items).ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_gradient_color_stop<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGradientColorStop, ParseError<'i, Error>> {
    let color = parse_color(input, numeric)?;
    let mut positions = Vec::new();
    for _ in 0..2 {
        if let Ok(position) = input.try_parse(|input| parse_gradient_line_position(input, numeric))
        {
            positions.push(position);
        } else {
            break;
        }
    }
    Ok(CssGradientColorStop::try_from_positions(color, positions)
        .expect("at most two parsed positions"))
}

fn parse_gradient_line_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedLengthPercentage, ParseError<'i, Error>> {
    parse_length_percentage(input, numeric)
}

#[derive(Debug)]
enum ParsedRadialSize<'i> {
    Extent(CssRadialExtent),
    Explicit {
        values: Vec<(
            Cow<'i, crate::CssComponentValue>,
            super::property_input::Site<'i>,
        )>,
        location: super::property_input::Site<'i>,
    },
}

fn parse_radial_gradient<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssRadialGradient, ParseError<'i, Error>> {
    let mut interpolation = parse_optional_gradient_interpolation(input)?;
    let prelude = if next_starts_radial_prelude(input) {
        let prelude = parse_radial_gradient_prelude(input, numeric)?;
        Some(prelude)
    } else {
        None
    };
    if interpolation.is_none() {
        interpolation = parse_optional_gradient_interpolation(input)?;
    }
    if prelude.is_some() || interpolation.is_some() {
        input.expect_comma().map_err(basic)?;
    }
    let (shape, size, position) = prelude.unwrap_or((None, None, None));
    let stops = parse_color_stop_list(input, numeric)?;
    CssRadialGradient::try_new(shape, size, position, stops)
        .map(|gradient| gradient.with_interpolation(interpolation))
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn next_starts_radial_prelude<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Ident(ident)) => matches!(
            ident.to_ascii_lowercase().as_str(),
            "circle"
                | "ellipse"
                | "closest-side"
                | "farthest-side"
                | "closest-corner"
                | "farthest-corner"
                | "at"
        ),
        Ok(Token::Dimension { .. } | Token::Percentage { .. }) => true,
        Ok(Token::Number { value, .. }) => *value == 0.0,
        Ok(Token::Function(name)) => crate::numeric::is_math_function(name),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

type RadialPrelude = (
    Option<CssRadialShape>,
    Option<CssRadialSize>,
    Option<CssPosition>,
);

fn parse_radial_gradient_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<RadialPrelude, ParseError<'i, Error>> {
    let start = input.current_source_location();
    let mut shape = None;
    let mut size = None;
    let mut position = None;
    let mut consumed = false;

    while !input.is_exhausted()
        && !next_is_comma(input)
        && !super::values::next_is_ident(input, "in")
    {
        if position.is_none()
            && input
                .try_parse(|input| input.expect_ident_matching("at"))
                .is_ok()
        {
            position = Some(parse_full_position_bounded(input, numeric, &["in"])?);
            consumed = true;
            break;
        }
        if shape.is_none()
            && let Ok(parsed_shape) = input.try_parse(parse_radial_shape)
        {
            shape = Some(parsed_shape);
            consumed = true;
            continue;
        }
        if size.is_none()
            && let Ok(parsed_size) = input.try_parse(parse_radial_size_input)
        {
            size = Some(parsed_size);
            consumed = true;
            continue;
        }
        return Err(unexpected_at(input.current_source_location()));
    }
    if !consumed {
        return Err(unexpected_at(start));
    }

    let size = size
        .map(|size| validate_radial_size(shape, size, numeric))
        .transpose()?;
    Ok((shape, size, position))
}

fn parse_conic_gradient<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssConicGradient, ParseError<'i, Error>> {
    let mut interpolation = parse_optional_gradient_interpolation(input)?;
    let from = if input
        .try_parse(|input| input.expect_ident_matching("from"))
        .is_ok()
    {
        Some(super::values::parse_angle_or_zero(input, numeric)?)
    } else {
        None
    };
    let position = if input
        .try_parse(|input| input.expect_ident_matching("at"))
        .is_ok()
    {
        Some(parse_full_position_bounded(input, numeric, &["in"])?)
    } else {
        None
    };
    if interpolation.is_none() {
        interpolation = parse_optional_gradient_interpolation(input)?;
    }
    if from.is_some() || position.is_some() || interpolation.is_some() {
        input.expect_comma().map_err(basic)?;
    }
    let mut items = vec![CssAngularColorStopListItem::Stop(Box::new(
        parse_angular_color_stop(input, numeric)?,
    ))];
    while input.try_parse(Parser::expect_comma).is_ok() {
        if input.is_exhausted() {
            return Err(unexpected_at(input.current_source_location()));
        }
        if let Ok(hint) = input.try_parse(|input| -> Result<_, ParseError<'i, Error>> {
            let hint = parse_angular_position(input, numeric)?;
            input.expect_comma().map_err(basic)?;
            Ok(hint)
        }) {
            items.push(CssAngularColorStopListItem::Hint(hint));
        }
        items.push(CssAngularColorStopListItem::Stop(Box::new(
            parse_angular_color_stop(input, numeric)?,
        )));
    }
    let stops = CssAngularColorStopList::try_new(items)
        .expect("parsed nonempty interleaved angular stop list");
    Ok(CssConicGradient::new(from, position, interpolation, stops))
}

fn parse_angular_color_stop<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssAngularColorStop, ParseError<'i, Error>> {
    let color = parse_color(input, numeric)?;
    let mut positions = Vec::new();
    for _ in 0..2 {
        if let Ok(position) = input.try_parse(|input| parse_angular_position(input, numeric)) {
            positions.push(position);
        } else {
            break;
        }
    }
    Ok(CssAngularColorStop::try_new(color, positions)
        .expect("at most two parsed angular positions"))
}

fn parse_angular_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssAngularColorStopPosition, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|error| super::values::angle_error(numeric, &error, location, offset))?;
    CssAngularColorStopPosition::from_parser_component(component, numeric)
        .map_err(|error| super::values::angle_error(numeric, &error, location, offset))
}

fn parse_radial_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssRadialShape, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "circle" => Ok(CssRadialShape::Circle),
        "ellipse" => Ok(CssRadialShape::Ellipse),
        _ => Err(unexpected_at(location)),
    }
}

fn parse_radial_size_input<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<ParsedRadialSize<'i>, ParseError<'i, Error>> {
    if let Ok(extent) = input.try_parse(parse_radial_extent) {
        return Ok(ParsedRadialSize::Extent(extent));
    }
    let location = input.current_source_location();
    let first = parse_radial_size_component(input)?;
    let mut values = vec![first];
    if let Ok(second) = input.try_parse(parse_radial_size_component) {
        values.push(second);
    }
    Ok(ParsedRadialSize::Explicit { values, location })
}

fn parse_radial_size_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<
    (
        Cow<'i, crate::CssComponentValue>,
        super::property_input::Site<'i>,
    ),
    ParseError<'i, Error>,
> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {}
        Token::Function(name) if crate::numeric::is_math_function(name) => {}
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
    input.reset(&state);
    input
        .collect_component()
        .map(|component| (component, location))
        .map_err(|_| unexpected_at(location))
}

fn parse_radial_extent<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssRadialExtent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "closest-side" => Ok(CssRadialExtent::ClosestSide),
        "farthest-side" => Ok(CssRadialExtent::FarthestSide),
        "closest-corner" => Ok(CssRadialExtent::ClosestCorner),
        "farthest-corner" => Ok(CssRadialExtent::FarthestCorner),
        _ => Err(unexpected_at(location)),
    }
}

fn validate_radial_size<'i>(
    shape: Option<CssRadialShape>,
    size: ParsedRadialSize<'i>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssRadialSize, ParseError<'i, Error>> {
    match size {
        ParsedRadialSize::Extent(extent) => Ok(CssRadialSize::Extent(extent)),
        ParsedRadialSize::Explicit { values, location } => {
            let mut values = values.into_iter();
            let value = match (values.next(), values.next(), values.next()) {
                (Some((radius, radius_location)), None, None) => {
                    let radius =
                        if matches!(radius.view(), crate::CssComponentValueRef::Function(_)) {
                            let components = crate::numeric::CalculationComponents::single(radius)
                                .expect("one radial operand");
                            let expression = numeric
                                .admit(components, CalculationRoot::Length)
                                .map_err(|_| unexpected_at(radius_location))?;
                            CssSpecifiedNonNegativeLength::try_from_calculation(
                                crate::CssLengthCalculation::from_expression(expression),
                            )
                        } else {
                            CssSpecifiedNonNegativeLength::try_from_component(radius.into_owned())
                        };
                    CssRadialSize::Circle(radius.map_err(|_| unexpected_at(radius_location))?)
                }
                (Some(horizontal), Some(vertical), None) => {
                    let admit = |(component, component_location): (
                        Cow<'i, crate::CssComponentValue>,
                        super::property_input::Site<'i>,
                    )| {
                        let value =
                            if matches!(component.view(), crate::CssComponentValueRef::Function(_))
                            {
                                let components =
                                    crate::numeric::CalculationComponents::single(component)
                                        .expect("one radial operand");
                                let expression = numeric
                                    .admit(components, CalculationRoot::LengthPercentage)
                                    .map_err(|_| unexpected_at(component_location))?;
                                CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                                    crate::CssLengthPercentageCalculation::from_expression(
                                        expression,
                                    ),
                                )
                            } else {
                                CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                                    component.into_owned(),
                                )
                            };
                        value.map_err(|_| unexpected_at(component_location))
                    };
                    CssRadialSize::Ellipse(CssRadialEllipseSize::new(
                        admit(horizontal)?,
                        admit(vertical)?,
                    ))
                }
                _ => {
                    return Err(unexpected_at(location));
                }
            };
            CssRadialGradient::allows_shape_size(shape, Some(&value))
                .then_some(value)
                .ok_or_else(|| unexpected_at(location))
        }
    }
}

pub(super) fn parse_background_size_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSizeList, ParseError<'i, Error>> {
    let sizes = input.parse_comma_separated(|input| parse_background_size(input, numeric))?;
    CssBackgroundSizeList::try_new(sizes)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_background_size_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSizeComponent, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssBackgroundSizeComponent::Auto)
    } else {
        parse_nonnegative_length_percentage(input, numeric).map(CssBackgroundSizeComponent::Length)
    }
}

pub(super) fn parse_background_repeat_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundRepeatList, ParseError<'i, Error>> {
    let repeats = input.parse_comma_separated(parse_background_repeat)?;
    CssBackgroundRepeatList::try_new(repeats)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_background_repeat_style_from_ident<'i, 't>(
    input: &Parser<'i, 't>,
    ident: &str,
) -> std::result::Result<CssBackgroundRepeatStyle, ParseError<'i, Error>> {
    match ident.to_ascii_lowercase().as_str() {
        "repeat" => Ok(CssBackgroundRepeatStyle::Repeat),
        "space" => Ok(CssBackgroundRepeatStyle::Space),
        "round" => Ok(CssBackgroundRepeatStyle::Round),
        "no-repeat" => Ok(CssBackgroundRepeatStyle::NoRepeat),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_background_box<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundBox, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "border-box" => Ok(CssBackgroundBox::BorderBox),
        "padding-box" => Ok(CssBackgroundBox::PaddingBox),
        "content-box" => Ok(CssBackgroundBox::ContentBox),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_background_box_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundBoxList, ParseError<'i, Error>> {
    let boxes = input.parse_comma_separated(|input| parse_background_box(input))?;
    CssBackgroundBoxList::try_new(boxes)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_background_attachment_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundAttachmentList, ParseError<'i, Error>> {
    let attachments = input.parse_comma_separated(|input| parse_background_attachment(input))?;
    CssBackgroundAttachmentList::try_new(attachments)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_background_attachment<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundAttachment, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "scroll" => Ok(CssBackgroundAttachment::Scroll),
        "fixed" => Ok(CssBackgroundAttachment::Fixed),
        "local" => Ok(CssBackgroundAttachment::Local),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

#[cfg(test)]
mod tests {

    use super::super::position::parse_physical_position;
    use super::*;

    fn parse_position(source: &str) -> CssPhysicalPosition {
        let snapshot = crate::CssSourceSnapshot::new(source);
        let numeric = crate::numeric::NumericInputContext::parsed(&snapshot);
        let document = crate::syntax_consumption::source_document(source, &snapshot, 0)
            .expect("position source document");
        let mut parser = Parser::new(&document);
        let value = parse_physical_position(&mut parser, &numeric).expect("valid generic position");
        parser
            .expect_exhausted()
            .expect("complete generic position");
        value
    }

    #[test]
    fn generic_position_model_distinguishes_omitted_and_free_offset_axes() {
        let top = parse_position("top");
        assert!(matches!(top.horizontal(), CssHorizontalPosition::Center));
        assert!(matches!(top.vertical(), CssVerticalPosition::Top));

        let free = parse_position("25% 10px");
        assert!(matches!(
            free.horizontal(),
            CssHorizontalPosition::Offset(offset)
                if matches!(offset.literal_component().unwrap().view(), crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(number)) if number.representation() == "25")
        ));
        assert!(matches!(
            free.vertical(),
            CssVerticalPosition::Offset(offset)
                if matches!(offset.literal_component().unwrap().view(), crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "10" && unit == "px")
        ));
    }

    #[test]
    fn generic_position_model_retains_each_edge_offset_origin_and_pair_order() {
        for source in ["left 10px bottom 20%", "bottom 20% left 10px"] {
            let position = parse_position(source);
            assert!(matches!(
                position.horizontal(),
                CssHorizontalPosition::LeftOffset(offset)
                    if matches!(offset.literal_component().unwrap().view(), crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "10" && unit == "px")
            ));
            assert!(matches!(
                position.vertical(),
                CssVerticalPosition::BottomOffset(offset)
                    if matches!(offset.literal_component().unwrap().view(), crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Percentage(number)) if number.representation() == "20")
            ));
        }

        let opposite = parse_position("right calc(1px * 2) top calc(10% + 2px)");
        assert!(matches!(
            opposite.horizontal(),
            CssHorizontalPosition::RightOffset(offset)
                if offset.calculation().is_some()
        ));
        assert!(matches!(
            opposite.vertical(),
            CssVerticalPosition::TopOffset(offset)
                if offset.calculation().is_some()
        ));
    }
}
