use crate::parser::property_input::{SiteExt, StreamExt};
use std::borrow::Cow;

use super::color::{parse_color, parse_color_interpolation_method};
use super::combinators::{Combinator, Tail, attempt, commit, fill, parse_group};
use super::side_shapes::parse_auto_or;
use super::syntax_types::Stream;
use super::values::{
    parse_length_percentage, parse_nonnegative_length, parse_nonnegative_length_percentage,
};
use crate::diagnostic::CssFeatureId;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::position::{
    next_starts_background_position, parse_background_position_prefix, parse_full_position_bounded,
};
use super::url::parse_url;
use super::values::{
    next_is_comma, parse_hinted_number_calculation, parse_nonnegative_number,
    parse_nonnegative_percentage,
};
use crate::color::CssColorInterpolation;
use crate::numeric::expression::CalculationRoot;
use crate::numeric::{CssLength, CssLengthPercentage, CssNonNegative, CssNumeric};
use crate::values::background::{
    CssBackground, CssBackgroundAttachment, CssBackgroundAttachmentList, CssBackgroundBox,
    CssBackgroundBoxList, CssBackgroundLayer, CssBackgroundLayerBoxes, CssBackgroundRepeat,
    CssBackgroundRepeatList, CssBackgroundRepeatStyle, CssBackgroundSize, CssBackgroundSizeList,
};
use crate::values::border::{
    CssBorderImage, CssBorderImageOutset, CssBorderImageOutsetComponent, CssBorderImageRepeat,
    CssBorderImageRepeatKeyword, CssBorderImageSlice, CssBorderImageSliceComponent,
    CssBorderImageWidth, CssBorderImageWidthComponent,
};
use crate::values::common::CssAutoOr;
use crate::values::effects::{CssFilterImage, CssFilterImageInput, CssFilterImageString};
use crate::values::image::{
    CssAngularColorStop, CssAngularColorStopList, CssAngularColorStopListItem,
    CssAngularColorStopPosition, CssColorStopList, CssColorStopListItem, CssConicGradient,
    CssGradient, CssGradientColorStop, CssHorizontalGradientSide, CssImage,
    CssImageConstructionError, CssImageOrientation, CssImageRendering, CssImageValue,
    CssImageValueList, CssLightDarkImage, CssLinearGradient, CssLinearGradientDirection,
    CssObjectFit, CssRadialEllipseSize, CssRadialExtent, CssRadialGradient, CssRadialShape,
    CssRadialSize, CssSideOrCorner, CssVerticalGradientSide,
};
use crate::values::position::CssPosition;
#[cfg(test)]
use crate::values::position::{CssCartesianPosition, CssHorizontalPosition, CssVerticalPosition};

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

pub(crate) fn parse_image_layer_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssImageValueList, SyntaxError<'i, Error>> {
    let images = input.comma_segments(|input| parse_image_value(input, numeric))?;
    CssImageValueList::try_new(images).ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_background<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBackground, SyntaxError<'i, Error>> {
    // List post-condition: only the final layer may carry the color, so a
    // color is rejected as soon as another layer follows it.
    let mut previous_color = None;
    let layers = input.comma_segments(|input| {
        if let Some(location) = previous_color {
            return Err(unexpected_at(location));
        }
        let (layer, color_location) = parse_background_layer(input, numeric)?;
        previous_color = color_location;
        Ok(layer)
    })?;

    CssBackground::try_new(layers).map_err(|_| unexpected_at(input.site()))
}

fn parse_background_layer<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<
    (CssBackgroundLayer, Option<super::syntax_types::Site<'i>>),
    SyntaxError<'i, Error>,
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
                match input.attempt(|input| parse_image_value(input, numeric)) {
                    Ok(value) => {
                        image = Some(value);
                        Ok(true)
                    }
                    Err(error)
                        if crate::diagnostic::error::property::is_resource_parse_error(&error) =>
                    {
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
                if input.attempt(|input| input.consume_delim('/')).is_ok() {
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
                let location = input.site();
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
            .map_err(|_| unexpected_at(input.site()))?,
        color_location,
    ))
}

pub(super) fn next_starts_background_image<'i>(input: &mut Stream<'i>) -> bool {
    let state = input.mark();
    let starts = match input.consume_after_whitespace() {
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("none"),
        Ok(Token::Url(_)) => true,
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
    input.restore(state);
    starts
}

fn next_starts_background_repeat<'i>(input: &mut Stream<'i>) -> bool {
    let state = input.mark();
    let starts = matches!(
        input.consume_after_whitespace(),
        Ok(Token::Ident(value))
            if matches!(
                value.to_ascii_lowercase().as_str(),
                "repeat-x" | "repeat-y" | "repeat" | "space" | "round" | "no-repeat"
            )
    );
    input.restore(state);
    starts
}

/// One `<repeat-style>`. A following component that is not a second repeat
/// keyword is left to the caller's boundary (list comma or shorthand group).
pub(super) fn parse_background_repeat<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBackgroundRepeat, SyntaxError<'i, Error>> {
    let first_location = input.site();
    let first = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &first,
        "repeat-x" => Ok(CssBackgroundRepeat::RepeatX),
        "repeat-y" => Ok(CssBackgroundRepeat::RepeatY),
        _ => {
            let x = parse_background_repeat_style_from_ident(first_location, first)?;
            let y = attempt(input, |input| {
                let second_location = input.site();
                let second = input.consume_ident().map_err(basic)?;
                parse_background_repeat_style_from_ident(second_location, second)
            })?
            .unwrap_or(x);
            Ok(CssBackgroundRepeat::Axes { x, y })
        }
    }
}

/// One `<bg-size>`. A following component that is not a second size is left
/// to the caller's boundary (list comma or shorthand group).
pub(super) fn parse_background_size<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSize, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let state = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        match_ignore_ascii_case! { &ident,
            "cover" => return Ok(CssBackgroundSize::Cover),
            "contain" => return Ok(CssBackgroundSize::Contain),
            "auto" => {},
            _ => return Err(unexpected_at(keyword_location)),
        }
        input.restore(state);
    }
    let width = parse_background_size_component(input, numeric)?;
    let height = attempt(input, |input| {
        parse_background_size_component(input, numeric)
    })?;
    Ok(CssBackgroundSize::Explicit { width, height })
}

/// Parses one image-only value through the canonical complete graph boundary.
pub(super) fn parse_image<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssImage, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let start = input.site();
    let value = parse_image_value(input, numeric)?;
    CssImage::try_new(value).map_err(|error| image_construction_error(error, location, start))
}

// All image construction entry points preserve resource failures and the
// original image token origin; semantic None rejection remains grammar-owned.
fn image_construction_error<'i>(
    error: CssImageConstructionError,
    location: super::syntax_types::Site<'i>,
    start: super::syntax_types::Site<'i>,
) -> SyntaxError<'i, Error> {
    // `CssImage::try_new` reports only these three image failures.
    let kind = if error.is_not_image() {
        return unexpected_at(location);
    } else if error.is_nesting_limit() {
        crate::component::values::CssComponentValueErrorKind::NestingLimit
    } else {
        debug_assert!(error.is_capacity_overflow(), "{error:?}");
        crate::component::values::CssComponentValueErrorKind::CapacityOverflow
    };
    crate::diagnostic::error::property::invalid_component_value(
        location,
        crate::component::CssComponentValueError::new(
            kind,
            start.value_origin().expect("image token origin"),
        ),
    )
}

pub(super) fn parse_image_value<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssImageValue, SyntaxError<'i, Error>> {
    let state = input.mark();
    let filter = matches!(input.consume_after_whitespace(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case("filter"));
    input.restore(state);
    if filter {
        input.discard_whitespace();
        let location = input.site();
        let start = input.site();
        input.consume_after_whitespace().map_err(basic)?;
        return input.within_block(|input| {
            let state = input.mark();
            let string = matches!(input.consume_after_whitespace(), Ok(Token::String(_)));
            input.restore(state);
            let input_value = if string {
                // Collect the exact original String rather than decoded text re-tokenization.
                input.discard_whitespace();
                let string_location = input.site();
                let string_start = input.site();
                let component = input.collect_numeric_component(numeric).map_err(|error| {
                    let location =
                        numeric.property_error_location(&error, string_location, string_start);
                    error.component_error().map_or_else(
                        || unexpected_at(location),
                        |detail| {
                            crate::diagnostic::error::property::invalid_component_value(
                                location,
                                detail.clone(),
                            )
                        },
                    )
                })?;
                let string =
                    CssFilterImageString::try_from_component(component).map_err(|error| {
                        crate::diagnostic::error::property::invalid_component_value(
                            string_location,
                            error,
                        )
                    })?;
                CssFilterImageInput::from_string(string)
            } else {
                CssFilterImageInput::from_image(parse_image(input, numeric)?)
            };
            input.consume_comma().map_err(basic)?;
            let filters = super::effects::parse_filter_function_list(input, numeric)?;
            CssFilterImage::try_new(input_value, filters)
                .map(|value| CssImageValue::Filter(Box::new(value)))
                .map_err(|error| image_construction_error(error, location, start))
        });
    }
    if next_is_light_dark(input) {
        input.discard_whitespace();
        let location = input.site();
        let start = input.site();
        input.consume_after_whitespace().map_err(basic)?;
        return input.within_block(|input| {
            let light = parse_image_value(input, numeric)?;
            input.consume_comma().map_err(basic)?;
            let dark = parse_image_value(input, numeric)?;
            input.require_empty().map_err(basic)?;
            CssLightDarkImage::try_new(light, dark)
                .map(|value| CssImageValue::LightDark(Box::new(value)))
                .map_err(|error| image_construction_error(error, location, start))
        });
    }
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssImageValue::None);
    }
    if next_is_gradient(input) {
        return parse_gradient(input, numeric).map(CssImageValue::Gradient);
    }
    parse_url(input, numeric).map(CssImageValue::Url)
}

fn next_is_light_dark(input: &mut Stream<'_>) -> bool {
    let state = input.mark();
    let result = matches!(input.consume_after_whitespace(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case("light-dark"));
    input.restore(state);
    result
}

pub(crate) fn parse_border_image_source<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssImageValue, SyntaxError<'i, Error>> {
    parse_image_value(input, numeric)
}

pub(crate) fn parse_border_image_slice<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageSlice, SyntaxError<'i, Error>> {
    let mut values = Vec::new();
    let mut fill = input.attempt(|input| input.consume_keyword("fill")).is_ok();

    // The repeated numeric group is one && component; fill cannot divide it.
    while !input.only_whitespace_remains() && values.len() < 4 {
        match input.attempt(|input| parse_border_image_slice_component(input, numeric)) {
            Ok(value) => values.push(value),
            Err(_) => break,
        }
    }

    if !fill && input.attempt(|input| input.consume_keyword("fill")).is_ok() {
        fill = true;
    }
    CssBorderImageSlice::try_new(values, fill).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_border_image_slice_component<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageSliceComponent, SyntaxError<'i, Error>> {
    if let Ok(calculation) = input.attempt(|input| parse_hinted_number_calculation(input, numeric))
    {
        return Ok(CssBorderImageSliceComponent::HintedNumberCalculation(
            calculation,
        ));
    }
    if let Ok(number) = input.attempt(|input| parse_nonnegative_number(input, numeric)) {
        return Ok(CssBorderImageSliceComponent::Number(number));
    }
    parse_nonnegative_percentage(input, numeric).map(CssBorderImageSliceComponent::Percentage)
}

/// `[ <length-percentage [0,∞]> | <number [0,∞]> | auto ]{1,4}`, leaving a
/// following component to the caller's boundary.
pub(crate) fn parse_border_image_width<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageWidth, SyntaxError<'i, Error>> {
    let values = repeat_up_to(input, 4, |input| {
        parse_border_image_width_component(input, numeric)
    })?;
    CssBorderImageWidth::try_new(values).ok_or_else(|| unexpected_at(input.site()))
}

/// `<item>{1,max}` as optional repetitions under the shared failure policy;
/// the caller's model rejects an empty result.
fn repeat_up_to<'i, T>(
    input: &mut Stream<'i>,
    max: usize,
    mut parse: impl FnMut(&mut Stream<'i>) -> Result<T, SyntaxError<'i, Error>>,
) -> Result<Vec<T>, SyntaxError<'i, Error>> {
    let mut values = Vec::new();
    while values.len() < max
        && let Some(value) = attempt(input, &mut parse)?
    {
        values.push(value);
    }
    Ok(values)
}

fn parse_border_image_width_component<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageWidthComponent, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("auto")).is_ok() {
        return Ok(CssBorderImageWidthComponent::Auto);
    }
    if let Ok(calculation) = input.attempt(|input| parse_hinted_number_calculation(input, numeric))
    {
        return Ok(CssBorderImageWidthComponent::HintedNumberCalculation(
            calculation,
        ));
    }
    if let Ok(number) = input.attempt(|input| parse_nonnegative_number(input, numeric)) {
        return Ok(CssBorderImageWidthComponent::Number(number));
    }
    parse_nonnegative_length_percentage(input, numeric)
        .map(CssBorderImageWidthComponent::LengthPercentage)
}

pub(crate) fn parse_border_image_outset<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageOutset, SyntaxError<'i, Error>> {
    // `[ <length [0,∞]> | <number [0,∞]> ]{1,4}`, leaving a following
    // component to the caller's boundary.
    let values = repeat_up_to(input, 4, |input| {
        parse_border_image_outset_component(input, numeric)
    })?;
    CssBorderImageOutset::try_new(values).ok_or_else(|| unexpected_at(input.site()))
}

fn parse_border_image_outset_component<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageOutsetComponent, SyntaxError<'i, Error>> {
    if let Ok(number) = input.attempt(|input| parse_nonnegative_number(input, numeric)) {
        return Ok(CssBorderImageOutsetComponent::Number(number));
    }
    parse_nonnegative_length(input, numeric).map(CssBorderImageOutsetComponent::Length)
}

pub(crate) fn parse_border_image_repeat<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBorderImageRepeat, SyntaxError<'i, Error>> {
    // `[ stretch | repeat | round | space ]{1,2}`, leaving a following
    // component to the caller's boundary.
    let horizontal = parse_border_image_repeat_keyword(input)?;
    let vertical = attempt(input, parse_border_image_repeat_keyword)?.unwrap_or(horizontal);
    Ok(CssBorderImageRepeat::new(horizontal, vertical))
}

fn parse_border_image_repeat_keyword<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBorderImageRepeatKeyword, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "stretch" => Ok(CssBorderImageRepeatKeyword::Stretch),
        "repeat" => Ok(CssBorderImageRepeatKeyword::Repeat),
        "round" => Ok(CssBorderImageRepeatKeyword::Round),
        "space" => Ok(CssBorderImageRepeatKeyword::Space),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn next_starts_border_image_repeat<'i>(input: &mut Stream<'i>) -> bool {
    let state = input.mark();
    let starts = matches!(
        input.consume_after_whitespace(),
        Ok(Token::Ident(value))
            if matches!(
                value.to_ascii_lowercase().as_str(),
                "stretch" | "repeat" | "round" | "space"
            )
    );
    input.restore(state);
    starts
}

pub(super) fn next_starts_border_image_slice<'i>(input: &mut Stream<'i>) -> bool {
    let state = input.mark();
    let starts = match input.consume_after_whitespace() {
        Ok(Token::Number(_) | Token::Percentage(_)) => true,
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("fill"),
        Ok(Token::Function(name)) => crate::numeric::expression::is_math_function(name),
        Ok(_) | Err(_) => false,
    };
    input.restore(state);
    starts
}

pub(crate) fn parse_border_image<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImage, SyntaxError<'i, Error>> {
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
                if input.attempt(|input| input.consume_delim('/')).is_ok() {
                    if input.attempt(|input| input.consume_delim('/')).is_ok() {
                        outset = Some(parse_border_image_outset(input, numeric)?);
                    } else {
                        width = Some(parse_border_image_width(input, numeric)?);
                        if input.attempt(|input| input.consume_delim('/')).is_ok() {
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
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_image_orientation<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssImageOrientation, SyntaxError<'i, Error>> {
    if input
        .attempt(|input| input.consume_keyword("from-image"))
        .is_ok()
    {
        return Ok(CssImageOrientation::FromImage);
    }
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssImageOrientation::None);
    }
    if input.attempt(|input| input.consume_keyword("flip")).is_ok() {
        let angle = if input.only_whitespace_remains() {
            None
        } else {
            Some(super::values::parse_angle_value(input, numeric)?)
        };
        return Ok(CssImageOrientation::Flip(angle));
    }
    let angle = super::values::parse_angle_value(input, numeric)?;
    if input.attempt(|input| input.consume_keyword("flip")).is_ok() {
        Ok(CssImageOrientation::Flip(Some(angle)))
    } else {
        Ok(CssImageOrientation::Angle(angle))
    }
}

pub(crate) fn parse_image_rendering<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssImageRendering, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssImageRendering::Auto),
        "smooth" => Ok(CssImageRendering::Smooth),
        "high-quality" => Ok(CssImageRendering::HighQuality),
        "crisp-edges" => Ok(CssImageRendering::CrispEdges),
        "pixelated" => Ok(CssImageRendering::Pixelated),
        "optimizespeed" => Ok(CssImageRendering::OptimizeSpeed),
        "optimizequality" => Ok(CssImageRendering::OptimizeQuality),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_object_fit<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssObjectFit, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "fill" => Ok(CssObjectFit::Fill),
        "contain" => Ok(CssObjectFit::Contain),
        "cover" => Ok(CssObjectFit::Cover),
        "none" => Ok(CssObjectFit::None),
        "scale-down" => Ok(CssObjectFit::ScaleDown),
        _ => Err(unexpected_at(keyword_location)),
    }
}

fn next_is_gradient<'i>(input: &mut Stream<'i>) -> bool {
    let state = input.mark();
    let is_gradient = matches!(
        input.consume_after_whitespace(),
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
    input.restore(state);
    is_gradient
}

fn parse_gradient<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssGradient, SyntaxError<'i, Error>> {
    let location = input.site();
    let name = input
        .consume_function_name()
        .map_err(basic)?
        .to_ascii_lowercase();
    match name.as_str() {
        "linear-gradient" => input
            .within_block(|input| parse_linear_gradient(input, numeric))
            .map(CssGradient::Linear),
        "repeating-linear-gradient" => input
            .within_block(|input| parse_linear_gradient(input, numeric))
            .map(CssGradient::RepeatingLinear),
        "radial-gradient" => input
            .within_block(|input| parse_radial_gradient(input, numeric))
            .map(CssGradient::Radial),
        "repeating-radial-gradient" => input
            .within_block(|input| parse_radial_gradient(input, numeric))
            .map(CssGradient::RepeatingRadial),
        "conic-gradient" => input
            .within_block(|input| parse_conic_gradient(input, numeric))
            .map(CssGradient::Conic),
        _ => Err(unexpected_at(location)),
    }
}

fn parse_linear_gradient<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssLinearGradient, SyntaxError<'i, Error>> {
    let mut interpolation = parse_optional_gradient_interpolation(input)?;
    let direction = next_starts_linear_gradient_direction(input)
        .then(|| parse_linear_gradient_direction(input, numeric))
        .transpose()?;
    if interpolation.is_none() {
        interpolation = parse_optional_gradient_interpolation(input)?;
    }
    if direction.is_some() || interpolation.is_some() {
        input.consume_comma().map_err(basic)?;
    }
    let stops = parse_color_stop_list(input, numeric)?;
    Ok(CssLinearGradient::new(direction, stops).with_interpolation(interpolation))
}

fn parse_optional_gradient_interpolation<'i>(
    input: &mut Stream<'i>,
) -> Result<Option<CssColorInterpolation>, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("in")).is_ok() {
        parse_color_interpolation_method(input).map(Some)
    } else {
        Ok(None)
    }
}

fn next_starts_linear_gradient_direction<'i>(input: &mut Stream<'i>) -> bool {
    let state = input.mark();
    let starts = match input.consume_after_whitespace() {
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("to"),
        Ok(Token::Number(_) | Token::Dimension { .. }) => true,
        Ok(Token::Function(name)) => crate::numeric::expression::is_math_function(name),
        Ok(_) | Err(_) => false,
    };
    input.restore(state);
    starts
}

fn parse_linear_gradient_direction<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssLinearGradientDirection, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("to")).is_ok() {
        return parse_side_or_corner(input).map(CssLinearGradientDirection::SideOrCorner);
    }
    super::values::parse_angle_or_zero(input, numeric).map(CssLinearGradientDirection::Angle)
}

fn parse_side_or_corner<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssSideOrCorner, SyntaxError<'i, Error>> {
    let start = input.site();
    let mut horizontal = None;
    let mut vertical = None;
    for _ in 0..2 {
        if super::values::next_is_ident(input, "in") {
            break;
        }
        let location = input.site();
        let Ok(ident) = input.attempt(Stream::consume_ident) else {
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

fn parse_color_stop_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssColorStopList, SyntaxError<'i, Error>> {
    let mut items = vec![CssColorStopListItem::Stop(Box::new(
        parse_gradient_color_stop(input, numeric)?,
    ))];
    while input.attempt(Stream::consume_comma).is_ok() {
        if input.only_whitespace_remains() {
            return Err(unexpected_at(input.site()));
        }
        if let Ok(hint) = input.attempt(|input| -> std::result::Result<_, SyntaxError<'i, Error>> {
            let hint = parse_gradient_line_position(input, numeric)?;
            input.consume_comma().map_err(basic)?;
            Ok(hint)
        }) {
            items.push(CssColorStopListItem::Hint(hint));
        }
        items.push(CssColorStopListItem::Stop(Box::new(
            parse_gradient_color_stop(input, numeric)?,
        )));
    }
    CssColorStopList::try_new(items).ok_or_else(|| unexpected_at(input.site()))
}

fn parse_gradient_color_stop<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssGradientColorStop, SyntaxError<'i, Error>> {
    let color = parse_color(input, numeric)?;
    let mut positions = Vec::new();
    for _ in 0..2 {
        if let Ok(position) = input.attempt(|input| parse_gradient_line_position(input, numeric)) {
            positions.push(position);
        } else {
            break;
        }
    }
    Ok(CssGradientColorStop::try_from_positions(color, positions)
        .expect("at most two parsed positions"))
}

fn parse_gradient_line_position<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssNumeric<CssLengthPercentage>, SyntaxError<'i, Error>> {
    parse_length_percentage(input, numeric)
}

#[derive(Debug)]
enum ParsedRadialSize<'i> {
    Extent(CssRadialExtent),
    Explicit {
        values: Vec<(
            Cow<'i, crate::component::CssComponentValue>,
            super::syntax_types::Site<'i>,
        )>,
        location: super::syntax_types::Site<'i>,
    },
}

fn parse_radial_gradient<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssRadialGradient, SyntaxError<'i, Error>> {
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
        input.consume_comma().map_err(basic)?;
    }
    let (shape, size, position) = prelude.unwrap_or((None, None, None));
    let stops = parse_color_stop_list(input, numeric)?;
    CssRadialGradient::try_new(shape, size, position, stops)
        .map(|gradient| gradient.with_interpolation(interpolation))
        .ok_or_else(|| unexpected_at(input.site()))
}

fn next_starts_radial_prelude<'i>(input: &mut Stream<'i>) -> bool {
    let state = input.mark();
    let starts = match input.consume_after_whitespace() {
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
        Ok(Token::Dimension { .. } | Token::Percentage(_)) => true,
        Ok(Token::Number(number)) => number.to_f64() == 0.0,
        Ok(Token::Function(name)) => crate::numeric::expression::is_math_function(name),
        Ok(_) | Err(_) => false,
    };
    input.restore(state);
    starts
}

type RadialPrelude = (
    Option<CssRadialShape>,
    Option<CssRadialSize>,
    Option<CssPosition>,
);

fn parse_radial_gradient_prelude<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<RadialPrelude, SyntaxError<'i, Error>> {
    let start = input.site();
    let mut shape = None;
    let mut size = None;
    let mut position = None;
    let mut consumed = false;

    while !input.only_whitespace_remains()
        && !next_is_comma(input)
        && !super::values::next_is_ident(input, "in")
    {
        if position.is_none() && input.attempt(|input| input.consume_keyword("at")).is_ok() {
            position = Some(parse_full_position_bounded(input, numeric, &["in"])?);
            consumed = true;
            break;
        }
        if shape.is_none()
            && let Ok(parsed_shape) = input.attempt(parse_radial_shape)
        {
            shape = Some(parsed_shape);
            consumed = true;
            continue;
        }
        if size.is_none()
            && let Ok(parsed_size) = input.attempt(parse_radial_size_input)
        {
            size = Some(parsed_size);
            consumed = true;
            continue;
        }
        return Err(unexpected_at(input.site()));
    }
    if !consumed {
        return Err(unexpected_at(start));
    }

    let size = size
        .map(|size| validate_radial_size(shape, size, numeric))
        .transpose()?;
    Ok((shape, size, position))
}

fn parse_conic_gradient<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssConicGradient, SyntaxError<'i, Error>> {
    let mut interpolation = parse_optional_gradient_interpolation(input)?;
    let from = if input.attempt(|input| input.consume_keyword("from")).is_ok() {
        Some(super::values::parse_angle_or_zero(input, numeric)?)
    } else {
        None
    };
    let position = if input.attempt(|input| input.consume_keyword("at")).is_ok() {
        Some(parse_full_position_bounded(input, numeric, &["in"])?)
    } else {
        None
    };
    if interpolation.is_none() {
        interpolation = parse_optional_gradient_interpolation(input)?;
    }
    if from.is_some() || position.is_some() || interpolation.is_some() {
        input.consume_comma().map_err(basic)?;
    }
    let mut items = vec![CssAngularColorStopListItem::Stop(Box::new(
        parse_angular_color_stop(input, numeric)?,
    ))];
    while input.attempt(Stream::consume_comma).is_ok() {
        if input.only_whitespace_remains() {
            return Err(unexpected_at(input.site()));
        }
        if let Ok(hint) = input.attempt(|input| -> Result<_, SyntaxError<'i, Error>> {
            let hint = parse_angular_position(input, numeric)?;
            input.consume_comma().map_err(basic)?;
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

fn parse_angular_color_stop<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssAngularColorStop, SyntaxError<'i, Error>> {
    let color = parse_color(input, numeric)?;
    let mut positions = Vec::new();
    for _ in 0..2 {
        if let Ok(position) = input.attempt(|input| parse_angular_position(input, numeric)) {
            positions.push(position);
        } else {
            break;
        }
    }
    Ok(CssAngularColorStop::try_new(color, positions)
        .expect("at most two parsed angular positions"))
}

fn parse_angular_position<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssAngularColorStopPosition, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let offset = input.site();
    let component = input
        .collect_numeric_component(numeric)
        .map_err(|error| super::values::angle_error(numeric, &error, location, offset))?;
    CssAngularColorStopPosition::from_parser_component(component, numeric)
        .map_err(|error| super::values::angle_error(numeric, &error, location, offset))
}

fn parse_radial_shape<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssRadialShape, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "circle" => Ok(CssRadialShape::Circle),
        "ellipse" => Ok(CssRadialShape::Ellipse),
        _ => Err(unexpected_at(location)),
    }
}

fn parse_radial_size_input<'i>(
    input: &mut Stream<'i>,
) -> Result<ParsedRadialSize<'i>, SyntaxError<'i, Error>> {
    if let Ok(extent) = input.attempt(parse_radial_extent) {
        return Ok(ParsedRadialSize::Extent(extent));
    }
    let location = input.site();
    let first = parse_radial_size_component(input)?;
    let mut values = vec![first];
    if let Ok(second) = input.attempt(parse_radial_size_component) {
        values.push(second);
    }
    Ok(ParsedRadialSize::Explicit { values, location })
}

fn parse_radial_size_component<'i>(
    input: &mut Stream<'i>,
) -> Result<
    (
        Cow<'i, crate::component::CssComponentValue>,
        super::syntax_types::Site<'i>,
    ),
    SyntaxError<'i, Error>,
> {
    input.discard_whitespace();
    let state = input.mark();
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Number(_) | Token::Dimension { .. } | Token::Percentage(_) => {}
        Token::Function(name) if crate::numeric::expression::is_math_function(name) => {}
        token => return Err(location.unexpected::<Error>(token)),
    }
    input.restore(state);
    input
        .collect_component()
        .map(|component| (component, location))
        .map_err(|_| unexpected_at(location))
}

fn parse_radial_extent<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssRadialExtent, SyntaxError<'i, Error>> {
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
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
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssRadialSize, SyntaxError<'i, Error>> {
    match size {
        ParsedRadialSize::Extent(extent) => Ok(CssRadialSize::Extent(extent)),
        ParsedRadialSize::Explicit { values, location } => {
            let mut values = values.into_iter();
            let value = match (values.next(), values.next(), values.next()) {
                (Some((radius, radius_location)), None, None) => {
                    let radius = if matches!(
                        radius.view(),
                        crate::component::CssComponentValueRef::Function(_)
                    ) {
                        let components =
                            crate::numeric::expression::CalculationComponents::single(radius)
                                .expect("one radial operand");
                        let expression = numeric
                            .admit(components, CalculationRoot::Length)
                            .map_err(|_| unexpected_at(radius_location))?;
                        CssNumeric::<CssLength, CssNonNegative>::from_parser_calculation(
                                crate::numeric::CssCalculation::<crate::numeric::CssLength>::from_expression(
                                    expression,
                                ),
                            )
                    } else {
                        CssNumeric::<CssLength, CssNonNegative>::try_from_component(
                            radius.into_owned(),
                        )
                    };
                    CssRadialSize::Circle(radius.map_err(|_| unexpected_at(radius_location))?)
                }
                (Some(horizontal), Some(vertical), None) => {
                    let admit = |(component, component_location): (
                        Cow<'i, crate::component::CssComponentValue>,
                        super::syntax_types::Site<'i>,
                    )| {
                        let value = if matches!(
                            component.view(),
                            crate::component::CssComponentValueRef::Function(_)
                        ) {
                            let components =
                                crate::numeric::expression::CalculationComponents::single(
                                    component,
                                )
                                .expect("one radial operand");
                            let expression = numeric
                                .admit(components, CalculationRoot::LengthPercentage)
                                .map_err(|_| unexpected_at(component_location))?;
                            CssNumeric::<CssLengthPercentage, CssNonNegative>::from_parser_calculation(
                                    crate::numeric::CssCalculation::<crate::numeric::CssLengthPercentage>::from_expression(
                                        expression,
                                    ),
                                )
                        } else {
                            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(
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

pub(crate) fn parse_background_size_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSizeList, SyntaxError<'i, Error>> {
    let sizes = input.comma_segments(|input| parse_background_size(input, numeric))?;
    CssBackgroundSizeList::try_new(sizes).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_background_size_component<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<
    CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>,
    SyntaxError<'i, Error>,
> {
    parse_auto_or(input, |input| {
        parse_nonnegative_length_percentage(input, numeric)
    })
}

pub(crate) fn parse_background_repeat_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBackgroundRepeatList, SyntaxError<'i, Error>> {
    let repeats = input.comma_segments(parse_background_repeat)?;
    CssBackgroundRepeatList::try_new(repeats).ok_or_else(|| unexpected_at(input.site()))
}

/// The repeat style `ident`, whose own `location` is responsible when it is
/// unknown.
pub(super) fn parse_background_repeat_style_from_ident<'i>(
    location: super::syntax_types::Site<'i>,
    ident: &str,
) -> std::result::Result<CssBackgroundRepeatStyle, SyntaxError<'i, Error>> {
    match ident.to_ascii_lowercase().as_str() {
        "repeat" => Ok(CssBackgroundRepeatStyle::Repeat),
        "space" => Ok(CssBackgroundRepeatStyle::Space),
        "round" => Ok(CssBackgroundRepeatStyle::Round),
        "no-repeat" => Ok(CssBackgroundRepeatStyle::NoRepeat),
        _ => Err(unexpected_at(location)),
    }
}

pub(super) fn parse_background_box<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBackgroundBox, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "border-box" => Ok(CssBackgroundBox::BorderBox),
        "padding-box" => Ok(CssBackgroundBox::PaddingBox),
        "content-box" => Ok(CssBackgroundBox::ContentBox),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_background_box_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBackgroundBoxList, SyntaxError<'i, Error>> {
    let boxes = input.comma_segments(|input| parse_background_box(input))?;
    CssBackgroundBoxList::try_new(boxes).ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_background_attachment_list<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBackgroundAttachmentList, SyntaxError<'i, Error>> {
    let attachments = input.comma_segments(|input| parse_background_attachment(input))?;
    CssBackgroundAttachmentList::try_new(attachments).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_background_attachment<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssBackgroundAttachment, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "scroll" => Ok(CssBackgroundAttachment::Scroll),
        "fixed" => Ok(CssBackgroundAttachment::Fixed),
        "local" => Ok(CssBackgroundAttachment::Local),
        _ => Err(unexpected_at(keyword_location)),
    }
}

#[cfg(test)]
mod tests {

    use super::super::position::parse_position_prefix;
    use super::*;

    fn parse_position(source: &str) -> CssCartesianPosition {
        let snapshot = crate::source::CssSourceSnapshot::new(source);
        let numeric = crate::numeric::expression::NumericInputContext::parsed(&snapshot);
        let document = crate::syntax_consumption::source_document(source, &snapshot, 0)
            .expect("position source document");
        let mut parser = document.root();
        let value = parse_position_prefix(&mut parser, &numeric).expect("valid generic position");
        parser.require_empty().expect("complete generic position");
        let crate::values::position::CssPositionRef::Cartesian(cartesian) = value.view() else {
            panic!("physical keywords form a Cartesian position");
        };
        cartesian.clone()
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
                if matches!(offset.literal_component().unwrap().view(), crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Percentage(number)) if number.representation() == "25")
        ));
        assert!(matches!(
            free.vertical(),
            CssVerticalPosition::Offset(offset)
                if matches!(offset.literal_component().unwrap().view(), crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "10" && unit == "px")
        ));
    }

    #[test]
    fn generic_position_model_retains_each_edge_offset_origin_and_pair_order() {
        for source in ["left 10px bottom 20%", "bottom 20% left 10px"] {
            let position = parse_position(source);
            assert!(matches!(
                position.horizontal(),
                CssHorizontalPosition::LeftOffset(offset)
                    if matches!(offset.literal_component().unwrap().view(), crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "10" && unit == "px")
            ));
            assert!(matches!(
                position.vertical(),
                CssVerticalPosition::BottomOffset(offset)
                    if matches!(offset.literal_component().unwrap().view(), crate::component::CssComponentValueRef::Token(crate::component::CssValueTokenRef::Percentage(number)) if number.representation() == "20")
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
