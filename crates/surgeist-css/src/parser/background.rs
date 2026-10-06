use super::color::parse_color;
use super::values::{
    parse_length_percentage, parse_nonnegative_length, parse_nonnegative_length_percentage,
};
use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::position::{
    next_starts_background_position, parse_background_position_prefix, parse_physical_position,
};
use super::url::parse_url;
use super::values::{
    CalculationRoot, next_is_comma, parse_hinted_number_calculation, parse_nonnegative_number,
    parse_nonnegative_percentage,
};
use crate::error::{CssFeatureId, Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

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
    CssFeatureId::new("official.value.gradient"),
    CssFeatureId::new("official.value.linear-gradient"),
    CssFeatureId::new("official.value.radial-gradient"),
    CssFeatureId::new("official.value.repeating-linear-gradient"),
    CssFeatureId::new("official.value.repeating-radial-gradient"),
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
    let mut images = Vec::new();
    loop {
        let image = parse_image_value(input, numeric)?;
        images.push(image);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "image layer list has an empty item",
            ));
        }
    }
    CssImageValueList::try_new(images)
        .ok_or_else(|| unsupported_value(input, None, "image list is empty"))
}

pub(super) fn parse_background<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackground, ParseError<'i, Error>> {
    let mut layers = Vec::new();

    loop {
        let (layer, color_location) = parse_background_layer(input, numeric)?;
        let has_comma = input.try_parse(Parser::expect_comma).is_ok();
        if has_comma && let Some(location) = color_location {
            return Err(unsupported_value_at(
                location,
                None,
                "background color is allowed only in the final layer",
            ));
        }
        layers.push(layer);
        if !has_comma {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "background layer list has an empty item",
            ));
        }
    }

    CssBackground::try_new(layers)
        .map_err(|_| unsupported_value(input, None, "background layer list is empty"))
}

fn parse_background_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<
    (CssBackgroundLayer, Option<cssparser::SourceLocation>),
    ParseError<'i, Error>,
> {
    let mut image = None;
    let mut position = None;
    let mut size = None;
    let mut repeat = None;
    let mut attachment = None;
    let mut boxes = Vec::new();
    let mut color = None;
    let mut color_location = None;

    while !input.is_exhausted() && !next_is_comma(input) {
        if image.is_none() && next_starts_background_image(input) {
            match input.try_parse(|input| parse_image_value(input, numeric)) {
                Ok(value) => {
                    image = Some(value);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) if next_is_light_dark(input) => {}
                Err(error) => return Err(error),
            }
        }
        if position.is_none() && next_starts_background_position(input) {
            position = Some(parse_background_position_prefix(input, numeric)?);
            if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                size = Some(parse_background_size_prefix(input, numeric)?);
            }
            continue;
        }
        if repeat.is_none() && next_starts_background_repeat(input) {
            repeat = Some(parse_background_repeat_prefix(input)?);
            continue;
        }
        if attachment.is_none()
            && let Ok(value) = input.try_parse(parse_background_attachment)
        {
            attachment = Some(value);
            continue;
        }
        if boxes.len() < 2
            && let Ok(value) = input.try_parse(parse_background_box)
        {
            boxes.push(value);
            continue;
        }
        if color.is_none() {
            let location = input.current_source_location();
            match input.try_parse(|input| parse_color(input, numeric)) {
                Ok(parsed) => {
                    color = Some(parsed);
                    color_location = Some(location);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported or duplicate background layer component",
        ));
    }

    if image.is_none()
        && position.is_none()
        && repeat.is_none()
        && attachment.is_none()
        && boxes.is_empty()
        && color.is_none()
    {
        return Err(unsupported_value(input, None, "background layer is empty"));
    }

    let boxes = match boxes.as_slice() {
        [] => None,
        [value] => Some(CssBackgroundLayerBoxes::One(*value)),
        [origin, clip] => Some(CssBackgroundLayerBoxes::OriginAndClip {
            origin: *origin,
            clip: *clip,
        }),
        _ => None,
    };
    Ok((
        CssBackgroundLayer::try_new(image, position, size, repeat, attachment, boxes, color)
            .map_err(|_| unsupported_value(input, None, "background layer is empty"))?,
        color_location,
    ))
}

fn next_starts_background_image<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("none"),
        Ok(Token::UnquotedUrl(_)) => true,
        Ok(Token::Function(name)) => {
            name.eq_ignore_ascii_case("url")
                || name.eq_ignore_ascii_case("light-dark")
                || name.eq_ignore_ascii_case("src")
                || matches!(
                    name.to_ascii_lowercase().as_str(),
                    "linear-gradient"
                        | "repeating-linear-gradient"
                        | "radial-gradient"
                        | "repeating-radial-gradient"
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

pub(super) fn parse_background_repeat_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundRepeat, ParseError<'i, Error>> {
    let first = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &first,
        "repeat-x" => Ok(CssBackgroundRepeat::RepeatX),
        "repeat-y" => Ok(CssBackgroundRepeat::RepeatY),
        _ => {
            let x = parse_background_repeat_style_from_ident(input, first.as_ref())?;
            let y = input
                .try_parse(|input| {
                    let second = input.expect_ident_cloned().map_err(basic)?;
                    parse_background_repeat_style_from_ident(input, second.as_ref())
                })
                .unwrap_or(x);
            Ok(CssBackgroundRepeat::Axes { x, y })
        }
    }
}

pub(super) fn parse_background_size_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSize, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "cover" => Ok(CssBackgroundSize::Cover),
            "contain" => Ok(CssBackgroundSize::Contain),
            "auto" => {
                let height = input.try_parse(|input| parse_background_size_component(input, numeric)).ok();
                Ok(CssBackgroundSize::Explicit {
                    width: CssBackgroundSizeComponent::Auto,
                    height,
                })
            },
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("background-size", ident.as_ref()),
            )),
        };
    }

    let width = parse_background_size_component(input, numeric)?;
    let height = input
        .try_parse(|input| parse_background_size_component(input, numeric))
        .ok();
    Ok(CssBackgroundSize::Explicit { width, height })
}

pub(super) fn parse_image_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssImageValue, ParseError<'i, Error>> {
    if next_is_light_dark(input) {
        let location = input.current_source_location();
        let start = input.position().byte_index();
        input.next().map_err(basic)?;
        return input.parse_nested_block(|input| {
            let light = parse_image_value(input, numeric)?;
            input.expect_comma().map_err(basic)?;
            let dark = parse_image_value(input, numeric)?;
            input.expect_exhausted().map_err(basic)?;
            CssLightDarkImage::try_new(light, dark)
                .map(|value| CssImageValue::LightDark(Box::new(value)))
                .map_err(|error| {
                    let kind = match error {
                        CssImageConstructionError::NestingLimit => {
                            crate::CssComponentValueErrorKind::NestingLimit
                        }
                        CssImageConstructionError::CapacityOverflow => {
                            crate::CssComponentValueErrorKind::CapacityOverflow
                        }
                    };
                    crate::error::invalid_component_value(
                        location,
                        crate::CssComponentValueError::new(
                            kind,
                            numeric.origin_at(start).expect("image function origin"),
                        ),
                    )
                })
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
    let mut fill = false;

    while !input.is_exhausted() && values.len() < 4 {
        if !fill
            && input
                .try_parse(|input| input.expect_ident_matching("fill"))
                .is_ok()
        {
            fill = true;
            continue;
        }
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
        .ok_or_else(|| unsupported_value(input, None, "border-image-slice is missing a value"))
}

fn parse_border_image_slice_component<'i, 't>(
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
    if let Ok(number) =
        input.try_parse(|input| parse_nonnegative_number(input, numeric, "border-image-slice"))
    {
        return Ok(CssBorderImageSliceComponent::Number(number));
    }
    parse_nonnegative_percentage(input, numeric, "border-image-slice")
        .map(CssBorderImageSliceComponent::Percentage)
}

pub(super) fn parse_border_image_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageWidth, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() && values.len() < 4 {
        values.push(parse_border_image_width_component(input, numeric)?);
    }
    CssBorderImageWidth::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "border-image-width is missing a value"))
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
    if let Ok(number) =
        input.try_parse(|input| parse_nonnegative_number(input, numeric, "border-image-width"))
    {
        return Ok(CssBorderImageWidthComponent::Number(number));
    }
    parse_nonnegative_length_percentage(input, numeric, "border-image-width")
        .map(CssBorderImageWidthComponent::LengthPercentage)
}

pub(super) fn parse_border_image_outset<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageOutset, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() && values.len() < 4 {
        values.push(parse_border_image_outset_component(input, numeric)?);
    }
    CssBorderImageOutset::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "border-image-outset is missing a value"))
}

fn parse_border_image_outset_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageOutsetComponent, ParseError<'i, Error>> {
    if let Ok(number) =
        input.try_parse(|input| parse_nonnegative_number(input, numeric, "border-image-outset"))
    {
        return Ok(CssBorderImageOutsetComponent::Number(number));
    }
    parse_nonnegative_length(input, numeric, "border-image-outset")
        .map(CssBorderImageOutsetComponent::Length)
}

pub(super) fn parse_border_image_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageRepeat, ParseError<'i, Error>> {
    let horizontal = parse_border_image_repeat_keyword(input)?;
    let vertical = if input.is_exhausted() {
        horizontal
    } else {
        parse_border_image_repeat_keyword(input)?
    };
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("border-image-repeat", ident.as_ref()),
        )),
    }
}

fn next_starts_border_image_repeat<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
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

fn next_starts_border_image_slice<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
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
    let mut source = None;
    let mut slice = None;
    let mut width = None;
    let mut outset = None;
    let mut repeat = None;

    while !input.is_exhausted() {
        if source.is_none() && next_starts_background_image(input) {
            source = Some(parse_image_value(input, numeric)?);
            continue;
        }
        if slice.is_none() && next_starts_border_image_slice(input) {
            slice = Some(parse_border_image_slice_prefix(input, numeric)?);
            if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    outset = Some(parse_border_image_outset_prefix(input, numeric)?);
                } else {
                    width = Some(parse_border_image_width_prefix(input, numeric)?);
                    if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                        outset = Some(parse_border_image_outset_prefix(input, numeric)?);
                    }
                }
            }
            continue;
        }
        if repeat.is_none() && next_starts_border_image_repeat(input) {
            repeat = Some(parse_border_image_repeat_prefix(input)?);
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported or duplicate border-image component",
        ));
    }

    CssBorderImage::try_new(source, slice, width, outset, repeat).ok_or_else(|| {
        unsupported_value(input, None, "border-image shorthand is missing a component")
    })
}

fn parse_border_image_slice_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageSlice, ParseError<'i, Error>> {
    let mut values = Vec::new();
    let mut fill = false;
    while values.len() < 4 && next_starts_border_image_slice(input) {
        if !fill
            && input
                .try_parse(|input| input.expect_ident_matching("fill"))
                .is_ok()
        {
            fill = true;
        } else {
            values.push(parse_border_image_slice_component(input, numeric)?);
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
        .ok_or_else(|| unsupported_value(input, None, "border-image-slice is missing a value"))
}

fn parse_border_image_width_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageWidth, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while values.len() < 4 {
        match input.try_parse(|input| parse_border_image_width_component(input, numeric)) {
            Ok(value) => values.push(value),
            Err(_) => break,
        }
    }
    CssBorderImageWidth::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "border-image-width is missing a value"))
}

fn parse_border_image_outset_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBorderImageOutset, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while values.len() < 4 {
        match input.try_parse(|input| parse_border_image_outset_component(input, numeric)) {
            Ok(value) => values.push(value),
            Err(_) => break,
        }
    }
    CssBorderImageOutset::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "border-image-outset is missing a value"))
}

fn parse_border_image_repeat_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageRepeat, ParseError<'i, Error>> {
    let horizontal = parse_border_image_repeat_keyword(input)?;
    let vertical = input
        .try_parse(parse_border_image_repeat_keyword)
        .unwrap_or(horizontal);
    Ok(CssBorderImageRepeat::new(horizontal, vertical))
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
            Some(super::values::parse_angle_value(
                input,
                numeric,
                super::values::AngleParserContext::ImageOrientation,
            )?)
        };
        return Ok(CssImageOrientation::Flip(angle));
    }
    let angle = super::values::parse_angle_value(
        input,
        numeric,
        super::values::AngleParserContext::ImageOrientation,
    )?;
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("image-rendering", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("object-fit", ident.as_ref()),
        )),
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
        _ => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported image function `{name}`"),
        )),
    }
}

fn parse_linear_gradient<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssLinearGradient, ParseError<'i, Error>> {
    let direction = if next_starts_linear_gradient_direction(input) {
        let direction = parse_linear_gradient_direction(input, numeric)?;
        input.expect_comma().map_err(basic)?;
        Some(direction)
    } else {
        None
    };
    let stops = parse_color_stop_list(input, numeric)?;
    Ok(CssLinearGradient::new(direction, stops))
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
    super::values::parse_angle_or_zero(input, numeric, super::values::AngleParserContext::Gradient)
        .map(CssLinearGradientDirection::Angle)
}

fn parse_side_or_corner<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssSideOrCorner, ParseError<'i, Error>> {
    let start = input.current_source_location();
    let mut horizontal = None;
    let mut vertical = None;
    for _ in 0..2 {
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
            _ => return Err(unsupported_value_at(
                location,
                None,
                format!("invalid gradient side or corner `{ident}`"),
            )),
        }
    }
    CssSideOrCorner::try_new(horizontal, vertical)
        .ok_or_else(|| unsupported_value_at(start, None, "gradient direction is empty"))
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
            return Err(unsupported_value(
                input,
                None,
                "color-stop list has an empty item",
            ));
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
    CssColorStopList::try_new(items)
        .ok_or_else(|| unsupported_value(input, None, "gradient requires at least two color stops"))
}

fn parse_gradient_color_stop<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssGradientColorStop, ParseError<'i, Error>> {
    let color = parse_color(input, numeric)?;
    let position = input
        .try_parse(|input| parse_gradient_line_position(input, numeric))
        .ok();
    Ok(CssGradientColorStop::from_color(color, position))
}

fn parse_gradient_line_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedLengthPercentage, ParseError<'i, Error>> {
    parse_length_percentage(input, numeric, "gradient stop")
}

#[derive(Clone, Debug)]
enum ParsedRadialSize {
    Extent(CssRadialExtent),
    Explicit {
        values: Vec<(crate::CssComponentValue, cssparser::SourceLocation)>,
        location: cssparser::SourceLocation,
    },
}

fn parse_radial_gradient<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssRadialGradient, ParseError<'i, Error>> {
    let prelude = if next_starts_radial_prelude(input) {
        let prelude = parse_radial_gradient_prelude(input, numeric)?;
        input.expect_comma().map_err(basic)?;
        Some(prelude)
    } else {
        None
    };
    let (shape, size, position) = prelude.unwrap_or((None, None, None));
    let stops = parse_color_stop_list(input, numeric)?;
    CssRadialGradient::try_new(shape, size, position, stops).ok_or_else(|| {
        unsupported_value(
            input,
            None,
            "radial-gradient shape and size are incompatible",
        )
    })
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
    Option<CssPhysicalPosition>,
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

    while !input.is_exhausted() && !next_is_comma(input) {
        if position.is_none()
            && input
                .try_parse(|input| input.expect_ident_matching("at"))
                .is_ok()
        {
            position = Some(parse_physical_position(input, numeric)?);
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
            && let Ok(parsed_size) =
                input.try_parse(|input| parse_radial_size_input(input, numeric))
        {
            size = Some(parsed_size);
            consumed = true;
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported radial-gradient prelude component",
        ));
    }
    if !consumed {
        return Err(unsupported_value_at(
            start,
            None,
            "radial-gradient prelude is empty",
        ));
    }

    let size = size
        .map(|size| validate_radial_size(shape, size, numeric))
        .transpose()?;
    Ok((shape, size, position))
}

fn parse_radial_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssRadialShape, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "circle" => Ok(CssRadialShape::Circle),
        "ellipse" => Ok(CssRadialShape::Ellipse),
        _ => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported radial-gradient shape `{ident}`"),
        )),
    }
}

fn parse_radial_size_input<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<ParsedRadialSize, ParseError<'i, Error>> {
    if let Ok(extent) = input.try_parse(parse_radial_extent) {
        return Ok(ParsedRadialSize::Extent(extent));
    }
    let location = input.current_source_location();
    let first = parse_radial_size_component(input, numeric)?;
    let mut values = vec![first];
    if let Ok(second) = input.try_parse(|input| parse_radial_size_component(input, numeric)) {
        values.push(second);
    }
    Ok(ParsedRadialSize::Explicit { values, location })
}

fn parse_radial_size_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<(crate::CssComponentValue, cssparser::SourceLocation), ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {}
        Token::Function(name) if crate::numeric::is_math_function(name) => {}
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
    input.reset(&state);
    numeric
        .collect(input)
        .map(|component| (component, location))
        .map_err(|_| unsupported_value_at(location, None, "invalid radial-gradient size"))
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
        _ => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported radial-gradient extent `{ident}`"),
        )),
    }
}

fn validate_radial_size<'i>(
    shape: Option<CssRadialShape>,
    size: ParsedRadialSize,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssRadialSize, ParseError<'i, Error>> {
    match size {
        ParsedRadialSize::Extent(extent) => Ok(CssRadialSize::Extent(extent)),
        ParsedRadialSize::Explicit { values, location } => {
            let value = match values.as_slice() {
                [(radius, radius_location)] => {
                    let radius =
                        if matches!(radius.view(), crate::CssComponentValueRef::Function(_)) {
                            let components =
                                crate::CssComponentValues::try_new(vec![radius.clone()])
                                    .expect("one radial operand");
                            let expression = numeric
                                .admit(components, CalculationRoot::Length)
                                .map_err(|_| {
                                    unsupported_value_at(
                                        *radius_location,
                                        None,
                                        "invalid radial circle calculation",
                                    )
                                })?;
                            CssSpecifiedNonNegativeLength::try_from_calculation(
                                crate::CssLengthCalculation::from_expression(expression),
                            )
                        } else {
                            CssSpecifiedNonNegativeLength::try_from_component(radius.clone())
                        };
                    CssRadialSize::Circle(radius.map_err(|_| {
                        unsupported_value_at(
                            *radius_location,
                            None,
                            "radial circle requires a nonnegative length",
                        )
                    })?)
                }
                [horizontal, vertical] => {
                    let admit = |(component, component_location): &(
                        crate::CssComponentValue,
                        cssparser::SourceLocation,
                    )| {
                        let value =
                            if matches!(component.view(), crate::CssComponentValueRef::Function(_))
                            {
                                let components =
                                    crate::CssComponentValues::try_new(vec![component.clone()])
                                        .expect("one radial operand");
                                let expression = numeric
                                    .admit(components, CalculationRoot::LengthPercentage)
                                    .map_err(|_| {
                                        unsupported_value_at(
                                            *component_location,
                                            None,
                                            "invalid radial ellipse calculation",
                                        )
                                    })?;
                                CssSpecifiedNonNegativeLengthPercentage::try_from_calculation(
                                    crate::CssLengthPercentageCalculation::from_expression(
                                        expression,
                                    ),
                                )
                            } else {
                                CssSpecifiedNonNegativeLengthPercentage::try_from_component(
                                    component.clone(),
                                )
                            };
                        value.map_err(|_| {
                            unsupported_value_at(
                                *component_location,
                                None,
                                "radial ellipse requires nonnegative length-percentages",
                            )
                        })
                    };
                    CssRadialSize::Ellipse(CssRadialEllipseSize::new(
                        admit(horizontal)?,
                        admit(vertical)?,
                    ))
                }
                _ => {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        "invalid radial-gradient size arity",
                    ));
                }
            };
            CssRadialGradient::allows_shape_size(shape, Some(&value))
                .then_some(value)
                .ok_or_else(|| {
                    unsupported_value_at(location, None, "radial shape and explicit size disagree")
                })
        }
    }
}

pub(super) fn parse_background_size_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSizeList, ParseError<'i, Error>> {
    let mut sizes = Vec::new();
    loop {
        sizes.push(parse_background_size(input, numeric)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "background-size list has an empty item",
            ));
        }
    }
    CssBackgroundSizeList::try_new(sizes)
        .ok_or_else(|| unsupported_value(input, None, "background-size list is empty"))
}

pub(super) fn parse_background_size<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssBackgroundSize, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "cover" => Ok(CssBackgroundSize::Cover),
            "contain" => Ok(CssBackgroundSize::Contain),
            "auto" => {
                let height = if !input.is_exhausted() && !next_is_comma(input) {
                    Some(parse_background_size_component(input, numeric)?)
                } else {
                    None
                };
                Ok(CssBackgroundSize::Explicit {
                    width: CssBackgroundSizeComponent::Auto,
                    height,
                })
            },
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("background-size", ident.as_ref()),
            )),
        };
    }

    let width = parse_background_size_component(input, numeric)?;
    let height = if !input.is_exhausted() && !next_is_comma(input) {
        Some(parse_background_size_component(input, numeric)?)
    } else {
        None
    };
    Ok(CssBackgroundSize::Explicit { width, height })
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
        parse_nonnegative_length_percentage(input, numeric, "background-size")
            .map(CssBackgroundSizeComponent::Length)
    }
}

pub(super) fn parse_background_repeat_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundRepeatList, ParseError<'i, Error>> {
    let mut repeats = Vec::new();
    loop {
        repeats.push(parse_background_repeat(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "background-repeat list has an empty item",
            ));
        }
    }
    CssBackgroundRepeatList::try_new(repeats)
        .ok_or_else(|| unsupported_value(input, None, "background-repeat list is empty"))
}

pub(super) fn parse_background_repeat<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundRepeat, ParseError<'i, Error>> {
    let first = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &first,
        "repeat-x" => Ok(CssBackgroundRepeat::RepeatX),
        "repeat-y" => Ok(CssBackgroundRepeat::RepeatY),
        _ => {
            let x = parse_background_repeat_style_from_ident(input, first.as_ref())?;
            let y = if input.is_exhausted() || next_is_comma(input) {
                x
            } else {
                let second = input.expect_ident_cloned().map_err(basic)?;
                parse_background_repeat_style_from_ident(input, second.as_ref())?
            };
            Ok(CssBackgroundRepeat::Axes { x, y })
        }
    }
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("background-repeat", ident),
        )),
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
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("background box", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_background_box_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundBoxList, ParseError<'i, Error>> {
    let mut boxes = Vec::new();
    loop {
        boxes.push(parse_background_box(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "background box list has an empty item",
            ));
        }
    }
    CssBackgroundBoxList::try_new(boxes)
        .ok_or_else(|| unsupported_value(input, None, "background box list is empty"))
}

pub(super) fn parse_background_attachment_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundAttachmentList, ParseError<'i, Error>> {
    let mut attachments = Vec::new();
    loop {
        attachments.push(parse_background_attachment(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "background-attachment list has an empty item",
            ));
        }
    }
    CssBackgroundAttachmentList::try_new(attachments)
        .ok_or_else(|| unsupported_value(input, None, "background-attachment list is empty"))
}

pub(super) fn parse_background_attachment<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundAttachment, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "scroll" => Ok(CssBackgroundAttachment::Scroll),
        "fixed" => Ok(CssBackgroundAttachment::Fixed),
        "local" => Ok(CssBackgroundAttachment::Local),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("background-attachment", ident.as_ref()),
        )),
    }
}

use crate::cursor_values::{
    CssCursor, CssCursorImage, CssCursorImageSource, CssCursorImages, CssCursorUrlSet,
    CssCursorUrlSetDescriptor, CssCursorUrlSetOption, CssCursorUrlSetReference,
};

pub(super) fn parse_cursor<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssCursor, ParseError<'i, Error>> {
    let mut images = Vec::new();
    while let Ok(source) = input.try_parse(|input| parse_cursor_image_source(input, numeric)) {
        let hotspot = if let Ok(x) = input.try_parse(|input| {
            super::values::parse_specified_number(input, numeric, "cursor hotspot")
        }) {
            let y = super::values::parse_specified_number(input, numeric, "cursor hotspot")?;
            Some([x, y])
        } else {
            None
        };
        images.push(CssCursorImage::new(source, hotspot));
        input.expect_comma().map_err(basic)?;
    }
    let fallback = parse_cursor_keyword(input)?;
    if images.is_empty() {
        Ok(CssCursor::Keyword(fallback))
    } else {
        let images = CssCursorImages::try_new(images, fallback)
            .expect("at least one cursor image was parsed");
        Ok(CssCursor::Images(images))
    }
}

fn parse_cursor_image_source<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssCursorImageSource, ParseError<'i, Error>> {
    if let Ok(url) = input.try_parse(|input| parse_url(input, numeric)) {
        return Ok(CssCursorImageSource::Url(url));
    }
    let name = input.expect_function().map_err(basic)?.clone();
    if !name.eq_ignore_ascii_case("image-set") && !name.eq_ignore_ascii_case("-webkit-image-set") {
        return Err(unsupported_value(
            input,
            None,
            "cursor requires a URL or URL image set",
        ));
    }
    let options = input.parse_nested_block(|input| {
        let mut options = Vec::new();
        loop {
            let reference = if let Ok(url) = input.try_parse(|input| parse_url(input, numeric)) {
                CssCursorUrlSetReference::Url(url)
            } else {
                let string = input.expect_string_cloned().map_err(basic)?;
                let string = CssContentString::try_new(string.to_string())
                    .ok_or_else(|| unsupported_value(input, None, "invalid cursor URL string"))?;
                CssCursorUrlSetReference::String(string)
            };
            let mut descriptors = Vec::new();
            // Only two descriptor kinds exist. A third token is rejected by the
            // required comma/end boundary; duplicate kinds cross the same checked constructor.
            for _ in 0..2 {
                if let Ok(value) = input.try_parse(|input| {
                    super::values::parse_ordinary_resolution(
                        input,
                        numeric,
                        "cursor image resolution",
                    )
                }) {
                    descriptors.push(CssCursorUrlSetDescriptor::Resolution(value));
                } else if let Ok(value) = input.try_parse(parse_cursor_image_type) {
                    descriptors.push(CssCursorUrlSetDescriptor::Type(value));
                } else {
                    break;
                }
            }
            let option =
                CssCursorUrlSetOption::try_new(reference, descriptors).ok_or_else(|| {
                    unsupported_value(input, None, "duplicate cursor image-set descriptor")
                })?;
            options.push(option);
            if input.is_exhausted() {
                break;
            }
            input.expect_comma().map_err(basic)?;
        }
        Ok(options)
    })?;
    let set =
        CssCursorUrlSet::try_new(options).expect("at least one cursor image-set option was parsed");
    Ok(CssCursorImageSource::UrlSet(set))
}

fn parse_cursor_image_type<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssContentString, ParseError<'i, Error>> {
    let name = input.expect_function().map_err(basic)?.clone();
    if !name.eq_ignore_ascii_case("type") {
        return Err(unsupported_value(
            input,
            None,
            "cursor image-set type requires type()",
        ));
    }
    input.parse_nested_block(|input| {
        let value = input.expect_string_cloned().map_err(basic)?;
        let value = CssContentString::try_new(value.to_string())
            .ok_or_else(|| unsupported_value(input, None, "invalid cursor image type string"))?;
        input.expect_exhausted().map_err(basic)?;
        Ok(value)
    })
}

pub(super) fn parse_cursor_keyword<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCursorKeyword, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssCursorKeyword::Auto),
        "default" => Ok(CssCursorKeyword::Default),
        "none" => Ok(CssCursorKeyword::None),
        "context-menu" => Ok(CssCursorKeyword::ContextMenu),
        "help" => Ok(CssCursorKeyword::Help),
        "pointer" => Ok(CssCursorKeyword::Pointer),
        "progress" => Ok(CssCursorKeyword::Progress),
        "wait" => Ok(CssCursorKeyword::Wait),
        "cell" => Ok(CssCursorKeyword::Cell),
        "crosshair" => Ok(CssCursorKeyword::Crosshair),
        "text" => Ok(CssCursorKeyword::Text),
        "vertical-text" => Ok(CssCursorKeyword::VerticalText),
        "alias" => Ok(CssCursorKeyword::Alias),
        "copy" => Ok(CssCursorKeyword::Copy),
        "move" => Ok(CssCursorKeyword::Move),
        "no-drop" => Ok(CssCursorKeyword::NoDrop),
        "not-allowed" => Ok(CssCursorKeyword::NotAllowed),
        "grab" => Ok(CssCursorKeyword::Grab),
        "grabbing" => Ok(CssCursorKeyword::Grabbing),
        "all-scroll" => Ok(CssCursorKeyword::AllScroll),
        "col-resize" => Ok(CssCursorKeyword::ColResize),
        "row-resize" => Ok(CssCursorKeyword::RowResize),
        "n-resize" => Ok(CssCursorKeyword::NResize),
        "e-resize" => Ok(CssCursorKeyword::EResize),
        "s-resize" => Ok(CssCursorKeyword::SResize),
        "w-resize" => Ok(CssCursorKeyword::WResize),
        "ne-resize" => Ok(CssCursorKeyword::NeResize),
        "nw-resize" => Ok(CssCursorKeyword::NwResize),
        "se-resize" => Ok(CssCursorKeyword::SeResize),
        "sw-resize" => Ok(CssCursorKeyword::SwResize),
        "ew-resize" => Ok(CssCursorKeyword::EwResize),
        "ns-resize" => Ok(CssCursorKeyword::NsResize),
        "nesw-resize" => Ok(CssCursorKeyword::NeswResize),
        "nwse-resize" => Ok(CssCursorKeyword::NwseResize),
        "zoom-in" => Ok(CssCursorKeyword::ZoomIn),
        "zoom-out" => Ok(CssCursorKeyword::ZoomOut),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("cursor", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_pointer_events<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPointerEvents, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssPointerEvents::Auto),
        "none" => Ok(CssPointerEvents::None),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("pointer-events", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_user_select<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUserSelect, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssUserSelect::Auto),
        "text" => Ok(CssUserSelect::Text),
        "none" => Ok(CssUserSelect::None),
        "all" => Ok(CssUserSelect::All),
        "contain" => Ok(CssUserSelect::Contain),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("user-select", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_outline<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOutline, ParseError<'i, Error>> {
    let mut width = None;
    let mut style = None;
    let mut color = None;
    let mut autos = 0;
    while !input.is_exhausted() {
        let location = input.current_source_location();
        if input
            .try_parse(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            autos += 1;
            if autos > 2 {
                return Err(crate::error::unsupported_value_at(
                    location,
                    None,
                    "outline has too many auto components",
                ));
            }
            continue;
        }
        if width.is_none() {
            match input.try_parse(|input| parse_outline_width(input, numeric)) {
                Ok(parsed_width) => {
                    width = Some(parsed_width);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        if style.is_none()
            && let Ok(parsed_style) = input.try_parse(parse_outline_style)
        {
            style = Some(parsed_style);
            continue;
        }
        if color.is_none() {
            match input.try_parse(|input| parse_outline_color(input, numeric)) {
                Ok(parsed_color) => {
                    color = Some(parsed_color);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported outline component",
        ));
    }
    // Resolve auto only after the unordered explicit slots are known. One lone
    // auto (with optional width) sets both semantic slots under selected UI4 §3.1.
    match autos {
        0 => {}
        1 if style.is_none() || color.is_none() => {
            if style.is_none() {
                style = Some(CssOutlineStyle::Auto);
            }
            if color.is_none() {
                color = Some(CssOutlineColor::Auto);
            }
        }
        2 if style.is_none() && color.is_none() => {
            style = Some(CssOutlineStyle::Auto);
            color = Some(CssOutlineColor::Auto);
        }
        _ => {
            return Err(unsupported_value(
                input,
                None,
                "outline auto duplicates an explicit component",
            ));
        }
    }
    if width.is_none() && style.is_none() && color.is_none() {
        None
    } else {
        Some(CssOutline::try_new(width, style, color).expect("nonempty parsed outline"))
    }
    .ok_or_else(|| unsupported_value(input, None, "outline shorthand is empty"))
}

pub(super) fn parse_outline_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOutlineStyle, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssOutlineStyle::Auto),
        "none" => Ok(CssOutlineStyle::None),
        "dotted" => Ok(CssOutlineStyle::Dotted),
        "dashed" => Ok(CssOutlineStyle::Dashed),
        "solid" => Ok(CssOutlineStyle::Solid),
        "double" => Ok(CssOutlineStyle::Double),
        "groove" => Ok(CssOutlineStyle::Groove),
        "ridge" => Ok(CssOutlineStyle::Ridge),
        "inset" => Ok(CssOutlineStyle::Inset),
        "outset" => Ok(CssOutlineStyle::Outset),
        _ => Err(crate::error::unsupported_value_at(location, None, unsupported_keyword_reason("outline-style", ident.as_ref()))),
    }
}

pub(super) fn parse_outline_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOutlineColor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssOutlineColor::Auto);
    }
    let start = input.state();
    let stripes =
        matches!(input.next(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case("stripes"));
    input.reset(&start);
    if stripes {
        return super::image_1d::parse_image_1d(input, numeric)
            .map(|image| CssOutlineColor::Image1D(Box::new(image)));
    }
    parse_color(input, numeric).map(|color| CssOutlineColor::Color(Box::new(color)))
}

pub(super) fn parse_outline_width<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssOutlineWidth, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "thin" => Ok(CssOutlineWidth::Thin),
            "medium" => Ok(CssOutlineWidth::Medium),
            "thick" => Ok(CssOutlineWidth::Thick),
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("outline-width", ident.as_ref()),
            )),
        };
    }
    parse_nonnegative_length(input, numeric, "outline-width").map(CssOutlineWidth::Length)
}

#[cfg(test)]
mod tests {
    use cssparser::{Parser, ParserInput};

    use super::*;

    fn parse_position(source: &str) -> CssPhysicalPosition {
        let snapshot = crate::CssSourceSnapshot::new(source);
        let numeric = crate::numeric::NumericInputContext::parsed(&snapshot);
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        parser
            .parse_entirely(|input| parse_physical_position(input, &numeric))
            .expect("valid generic position")
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
