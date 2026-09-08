use cssparser::{ParseError, Parser, ParserState, ToCss, Token, match_ignore_ascii_case};

use super::box_model::parse_border_style;
use super::values::{
    CalculationRoot, LengthGrammar, next_is_comma, next_is_delim, parse_color, parse_length_with,
    parse_length_with_context, parse_length_with_context_legacy, parse_typed_calculation,
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
];

pub(super) fn parse_image_layer_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedImageValueList, ParseError<'i, Error>> {
    let mut images = Vec::new();
    let mut i01_layers = Some(Vec::new());
    loop {
        let image = parse_image_value(input)?;
        match (&image, i01_layers.as_mut()) {
            (CssImageValue::None, Some(layers)) => layers.push(CssImageLayer::None),
            (CssImageValue::Url(url), Some(layers)) => {
                layers.push(CssImageLayer::Url(url.clone()));
            }
            (CssImageValue::Gradient(_), _) => i01_layers = None,
            (CssImageValue::None | CssImageValue::Url(_), None) => {}
        }
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
    let current = CssImageValueList::try_new(images)
        .ok_or_else(|| unsupported_value(input, None, "image list is empty"))?;
    let i01_subset = i01_layers.and_then(CssImageLayerList::try_new);
    Ok(CssParsedImageValueList::new(current, i01_subset))
}

pub(super) fn parse_background<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedBackground, ParseError<'i, Error>> {
    let mut layers = Vec::new();
    let mut color_projections = Vec::new();

    loop {
        let (layer, color_projection, color_location) = parse_background_layer(input)?;
        let has_comma = input.try_parse(Parser::expect_comma).is_ok();
        if has_comma && let Some(location) = color_location {
            return Err(unsupported_value_at(
                location,
                None,
                "background color is allowed only in the final layer",
            ));
        }
        layers.push(layer);
        color_projections.push(color_projection);
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

    let i01_subset = match (layers.as_slice(), color_projections.as_slice()) {
        ([layer], [projection]) if layer.has_only_color() => projection.clone(),
        _ => None,
    };
    Ok(CssParsedBackground::new(
        CssBackground::new(layers),
        i01_subset,
    ))
}

fn parse_background_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<
    (
        CssBackgroundLayer,
        Option<CssColor>,
        Option<cssparser::SourceLocation>,
    ),
    ParseError<'i, Error>,
> {
    let mut image = None;
    let mut position = None;
    let mut size = None;
    let mut repeat = None;
    let mut attachment = None;
    let mut boxes = Vec::new();
    let mut color = None;
    let mut color_projection = None;
    let mut color_location = None;

    while !input.is_exhausted() && !next_is_comma(input) {
        if image.is_none() && next_starts_background_image(input) {
            image = Some(parse_image_value(input)?);
            continue;
        }
        if position.is_none() && next_starts_background_position(input) {
            position = Some(parse_background_position_prefix(input)?);
            if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                size = Some(parse_background_size_prefix(input)?);
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
            if let Ok(parsed) = input.try_parse(parse_color) {
                let (current, i01_subset) = parsed.into_parts();
                color = Some(current);
                color_projection = i01_subset;
                color_location = Some(location);
                continue;
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
        CssBackgroundLayer::new(image, position, size, repeat, attachment, boxes, color),
        color_projection,
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

fn next_starts_background_position<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Ident(value)) => matches!(
            value.to_ascii_lowercase().as_str(),
            "left" | "right" | "top" | "bottom" | "center"
        ),
        Ok(Token::Dimension { .. } | Token::Percentage { .. }) => true,
        Ok(Token::Number { value, .. }) => *value == 0.0,
        Ok(Token::Function(name)) => name.eq_ignore_ascii_case("calc"),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

fn parse_background_position_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundPosition, ParseError<'i, Error>> {
    let mut atoms = Vec::new();
    let mut states = Vec::new();
    while atoms.len() < 4 && next_starts_background_position(input) {
        states.push(input.state());
        atoms.push(parse_generic_position_atom(input)?);
    }
    build_background_position(&atoms).ok_or_else(|| {
        invalid_generic_position_atom(input, &states[invalid_background_atom_index(&atoms)])
    })
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

fn parse_background_repeat_prefix<'i, 't>(
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

fn parse_background_size_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundSize, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "cover" => Ok(CssBackgroundSize::Cover),
            "contain" => Ok(CssBackgroundSize::Contain),
            "auto" => {
                let height = input.try_parse(parse_background_size_component).ok();
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

    let width = parse_background_size_component(input)?;
    let height = input.try_parse(parse_background_size_component).ok();
    Ok(CssBackgroundSize::Explicit { width, height })
}

pub(super) fn parse_image_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssImageValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssImageValue::None);
    }
    if next_is_gradient(input) {
        return parse_gradient(input).map(CssImageValue::Gradient);
    }
    parse_url(input).map(CssImageValue::Url)
}

pub(super) fn parse_border_image_source<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssImageValue, ParseError<'i, Error>> {
    parse_image_value(input)
}

pub(super) fn parse_border_image_slice<'i, 't>(
    input: &mut Parser<'i, 't>,
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
        match input.try_parse(parse_border_image_slice_component) {
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
) -> std::result::Result<CssBorderImageSliceComponent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Number { value, .. } => CssNonNegativeNumber::try_new(value)
            .map(CssBorderImageSliceComponent::Number)
            .ok_or_else(|| {
                unsupported_value_at(location, None, "border-image-slice must be non-negative")
            }),
        Token::Percentage { unit_value, .. } => CssNonNegativeNumber::try_new(unit_value * 100.0)
            .map(CssBorderImageSliceComponent::Percentage)
            .ok_or_else(|| {
                unsupported_value_at(location, None, "border-image-slice must be non-negative")
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            if let Ok(expression) = input.try_parse(|input| {
                input.parse_nested_block(|input| {
                    parse_typed_calculation(input, CalculationRoot::Number)
                })
            }) {
                return Ok(CssBorderImageSliceComponent::NumberCalculation(
                    CssNumberCalculation::from_expression(expression),
                ));
            }
            input
                .parse_nested_block(|input| {
                    parse_typed_calculation(input, CalculationRoot::Percentage)
                })
                .map(CssPercentageCalculation::from_expression)
                .map(CssBorderImageSliceComponent::PercentageCalculation)
        }
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "unsupported border-image-slice component `{}`",
                token.to_css_string()
            ),
        )),
    }
}

pub(super) fn parse_border_image_width<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageWidth, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() && values.len() < 4 {
        values.push(parse_border_image_width_component(input)?);
    }
    CssBorderImageWidth::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "border-image-width is missing a value"))
}

fn parse_border_image_width_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageWidthComponent, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssBorderImageWidthComponent::Auto);
    }
    if let Ok(number) =
        input.try_parse(|input| parse_border_image_non_negative_number(input, "border-image-width"))
    {
        return Ok(match number {
            CssNonNegativeNumberValue::Literal(value) => {
                CssBorderImageWidthComponent::Number(value)
            }
            CssNonNegativeNumberValue::Calculation(value) => {
                CssBorderImageWidthComponent::NumberCalculation(value)
            }
        });
    }
    let location = input.current_source_location();
    let value =
        parse_length_with_context(input, LengthGrammar::BackgroundSize, "border-image-width")?;
    CssBorderImageWidthLengthPercentage::try_new(value)
        .map(CssBorderImageWidthComponent::LengthPercentage)
        .ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                "border-image-width must be a non-negative length-percentage",
            )
        })
}

pub(super) fn parse_border_image_outset<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageOutset, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() && values.len() < 4 {
        values.push(parse_border_image_outset_component(input)?);
    }
    CssBorderImageOutset::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "border-image-outset is missing a value"))
}

fn parse_border_image_outset_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageOutsetComponent, ParseError<'i, Error>> {
    if let Ok(number) = input
        .try_parse(|input| parse_border_image_non_negative_number(input, "border-image-outset"))
    {
        return Ok(match number {
            CssNonNegativeNumberValue::Literal(value) => {
                CssBorderImageOutsetComponent::Number(value)
            }
            CssNonNegativeNumberValue::Calculation(value) => {
                CssBorderImageOutsetComponent::NumberCalculation(value)
            }
        });
    }
    let location = input.current_source_location();
    let value =
        parse_length_with_context(input, LengthGrammar::BorderWidth, "border-image-outset")?;
    CssBorderImageOutsetLength::try_new(value)
        .map(CssBorderImageOutsetComponent::Length)
        .ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                "border-image-outset must be a non-negative length",
            )
        })
}

fn parse_border_image_non_negative_number<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<CssNonNegativeNumberValue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Number { value, .. } => CssNonNegativeNumber::try_new(value)
            .map(CssNonNegativeNumberValue::Literal)
            .ok_or_else(|| {
                unsupported_value_at(location, None, format!("{context} must be non-negative"))
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Number))
            .map(CssNumberCalculation::from_expression)
            .map(CssNonNegativeNumberValue::Calculation),
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "unsupported {context} component `{}`",
                token.to_css_string()
            ),
        )),
    }
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
        Ok(Token::Function(name)) => name.eq_ignore_ascii_case("calc"),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

pub(super) fn parse_border_image<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImage, ParseError<'i, Error>> {
    let mut source = None;
    let mut slice = None;
    let mut width = None;
    let mut outset = None;
    let mut repeat = None;

    while !input.is_exhausted() {
        if source.is_none() && next_starts_background_image(input) {
            source = Some(parse_image_value(input)?);
            continue;
        }
        if slice.is_none() && next_starts_border_image_slice(input) {
            slice = Some(parse_border_image_slice_prefix(input)?);
            if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    outset = Some(parse_border_image_outset_prefix(input)?);
                } else {
                    width = Some(parse_border_image_width_prefix(input)?);
                    if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                        outset = Some(parse_border_image_outset_prefix(input)?);
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

    if source.is_none() && slice.is_none() && repeat.is_none() {
        Err(unsupported_value(
            input,
            None,
            "border-image shorthand is missing a component",
        ))
    } else {
        Ok(CssBorderImage::new(source, slice, width, outset, repeat))
    }
}

fn parse_border_image_slice_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
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
            values.push(parse_border_image_slice_component(input)?);
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
) -> std::result::Result<CssBorderImageWidth, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while values.len() < 4 {
        match input.try_parse(parse_border_image_width_component) {
            Ok(value) => values.push(value),
            Err(_) => break,
        }
    }
    CssBorderImageWidth::try_new(values)
        .ok_or_else(|| unsupported_value(input, None, "border-image-width is missing a value"))
}

fn parse_border_image_outset_prefix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderImageOutset, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while values.len() < 4 {
        match input.try_parse(parse_border_image_outset_component) {
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
) -> std::result::Result<CssImageOrientation, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("from-image"))
        .is_ok()
    {
        return Ok(CssImageOrientation::FromImage);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("flip"))
        .is_ok()
    {
        return Ok(CssImageOrientation::Flip(None));
    }
    let angle = parse_image_orientation_angle(input)?;
    if input
        .try_parse(|input| input.expect_ident_matching("flip"))
        .is_ok()
    {
        Ok(CssImageOrientation::Flip(Some(angle)))
    } else {
        Ok(CssImageOrientation::Angle(angle))
    }
}

fn parse_image_orientation_angle<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssImageOrientationAngle, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Number { value: 0.0, .. } => Ok(CssImageOrientationAngle::Zero),
        Token::Dimension { value, unit, .. } => {
            let unit = match unit.to_ascii_lowercase().as_str() {
                "deg" => CssAngleUnit::Degrees,
                "grad" => CssAngleUnit::Gradians,
                "rad" => CssAngleUnit::Radians,
                "turn" => CssAngleUnit::Turns,
                _ => {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        format!("unsupported image-orientation angle unit `{unit}`"),
                    ));
                }
            };
            CssAngleLiteral::try_new(value, unit)
                .map(CssImageOrientationAngle::Literal)
                .ok_or_else(|| {
                    unsupported_value_at(location, None, "image-orientation angle must be finite")
                })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Angle))
            .map(CssAngleCalculation::from_expression)
            .map(CssImageOrientationAngle::Calculation),
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "unsupported image-orientation angle `{}`",
                token.to_css_string()
            ),
        )),
    }
}

pub(super) fn parse_image_rendering<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssImageRendering, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssImageRendering::Auto),
        "crisp-edges" => Ok(CssImageRendering::CrispEdges),
        "pixelated" => Ok(CssImageRendering::Pixelated),
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
) -> std::result::Result<CssGradient, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = input.expect_function().map_err(basic)?.to_ascii_lowercase();
    match name.as_str() {
        "linear-gradient" => input
            .parse_nested_block(parse_linear_gradient)
            .map(CssGradient::Linear),
        "repeating-linear-gradient" => input
            .parse_nested_block(parse_linear_gradient)
            .map(CssGradient::RepeatingLinear),
        "radial-gradient" => input
            .parse_nested_block(parse_radial_gradient)
            .map(CssGradient::Radial),
        "repeating-radial-gradient" => input
            .parse_nested_block(parse_radial_gradient)
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
) -> std::result::Result<CssLinearGradient, ParseError<'i, Error>> {
    let direction = if next_starts_linear_gradient_direction(input) {
        let direction = parse_linear_gradient_direction(input)?;
        input.expect_comma().map_err(basic)?;
        Some(direction)
    } else {
        None
    };
    let stops = parse_color_stop_list(input)?;
    Ok(CssLinearGradient::new(direction, stops))
}

fn next_starts_linear_gradient_direction<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Ident(value)) => value.eq_ignore_ascii_case("to"),
        Ok(Token::Number { .. } | Token::Dimension { .. }) => true,
        Ok(Token::Function(name)) => name.eq_ignore_ascii_case("calc"),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

fn parse_linear_gradient_direction<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLinearGradientDirection, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("to"))
        .is_ok()
    {
        return parse_side_or_corner(input).map(CssLinearGradientDirection::SideOrCorner);
    }
    parse_gradient_angle(input).map(CssLinearGradientDirection::Angle)
}

fn parse_gradient_angle<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGradientAngle, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } if *value == 0.0 => Ok(CssGradientAngle::Zero),
        Token::Dimension { value, unit, .. } => {
            let unit = match unit.to_ascii_lowercase().as_str() {
                "deg" => CssAngleUnit::Degrees,
                "grad" => CssAngleUnit::Gradians,
                "rad" => CssAngleUnit::Radians,
                "turn" => CssAngleUnit::Turns,
                _ => {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        format!("unsupported gradient angle unit `{unit}`"),
                    ));
                }
            };
            CssAngleLiteral::try_new(*value, unit)
                .map(CssGradientAngle::Literal)
                .ok_or_else(|| {
                    unsupported_value_at(location, None, "gradient angle must be finite")
                })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| {
                let location = input.current_source_location();
                let expression = parse_typed_calculation(input, CalculationRoot::Angle)?;
                if expression.result_type() != CssCalculationType::Angle {
                    return Err(unsupported_value_at(
                        location,
                        None,
                        "gradient angle calculation must have an angle result",
                    ));
                }
                Ok(expression)
            })
            .map(CssAngleCalculation::from_expression)
            .map(CssGradientAngle::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
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
) -> std::result::Result<CssColorStopList, ParseError<'i, Error>> {
    let mut items = vec![CssColorStopListItem::Stop(Box::new(
        parse_gradient_color_stop(input)?,
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
                let hint = parse_gradient_line_position(input)?;
                input.expect_comma().map_err(basic)?;
                Ok(hint)
            })
        {
            items.push(CssColorStopListItem::Hint(hint));
        }
        items.push(CssColorStopListItem::Stop(Box::new(
            parse_gradient_color_stop(input)?,
        )));
    }
    CssColorStopList::try_new(items)
        .ok_or_else(|| unsupported_value(input, None, "gradient requires at least two color stops"))
}

fn parse_gradient_color_stop<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGradientColorStop, ParseError<'i, Error>> {
    let color = parse_color(input)?;
    let position = input.try_parse(parse_gradient_line_position).ok();
    Ok(CssGradientColorStop::new(color, position))
}

fn parse_gradient_line_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssGradientLinePosition, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_length_with_context(input, LengthGrammar::Position, "gradient stop")?;
    CssGradientLinePosition::try_new(value).ok_or_else(|| {
        unsupported_value_at(location, None, "gradient stop requires a length-percentage")
    })
}

#[derive(Clone, Debug)]
enum ParsedRadialSize {
    Extent(CssRadialExtent),
    Explicit {
        values: Vec<CssLength>,
        location: cssparser::SourceLocation,
    },
}

fn parse_radial_gradient<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssRadialGradient, ParseError<'i, Error>> {
    let prelude = if next_starts_radial_prelude(input) {
        let prelude = parse_radial_gradient_prelude(input)?;
        input.expect_comma().map_err(basic)?;
        Some(prelude)
    } else {
        None
    };
    let (shape, size, position) = prelude.unwrap_or((None, None, None));
    let stops = parse_color_stop_list(input)?;
    Ok(CssRadialGradient::new(shape, size, position, stops))
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
        Ok(Token::Function(name)) => name.eq_ignore_ascii_case("calc"),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    starts
}

type RadialPrelude = (
    Option<CssRadialShape>,
    Option<CssRadialSize>,
    Option<CssPositionValue>,
);

fn parse_radial_gradient_prelude<'i, 't>(
    input: &mut Parser<'i, 't>,
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
            position = Some(parse_css_position_value(input)?);
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
        .map(|size| validate_radial_size(shape, size))
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
) -> std::result::Result<ParsedRadialSize, ParseError<'i, Error>> {
    if let Ok(extent) = input.try_parse(parse_radial_extent) {
        return Ok(ParsedRadialSize::Extent(extent));
    }
    let location = input.current_source_location();
    let first = parse_length_with_context(input, LengthGrammar::Position, "radial-gradient size")?;
    let mut values = vec![first];
    if let Ok(second) = input.try_parse(|input| {
        parse_length_with_context(input, LengthGrammar::Position, "radial-gradient size")
    }) {
        values.push(second);
    }
    Ok(ParsedRadialSize::Explicit { values, location })
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
) -> std::result::Result<CssRadialSize, ParseError<'i, Error>> {
    match size {
        ParsedRadialSize::Extent(extent) => Ok(CssRadialSize::Extent(extent)),
        ParsedRadialSize::Explicit { values, location } => match values.as_slice() {
            [radius] if shape != Some(CssRadialShape::Ellipse) => {
                CssRadialCircleSize::try_new(radius.clone())
                    .map(CssRadialSize::Circle)
                    .ok_or_else(|| {
                        unsupported_value_at(
                            location,
                            None,
                            "radial-gradient circle size requires a non-negative length",
                        )
                    })
            }
            [horizontal, vertical] if shape != Some(CssRadialShape::Circle) => {
                CssRadialEllipseSize::try_new(horizontal.clone(), vertical.clone())
                    .map(CssRadialSize::Ellipse)
                    .ok_or_else(|| {
                        unsupported_value_at(
                            location,
                            None,
                            "radial-gradient ellipse size requires two non-negative length-percentages",
                        )
                    })
            }
            [_] => Err(unsupported_value_at(
                location,
                None,
                "ellipse radial-gradient requires two explicit radii",
            )),
            [_, _] => Err(unsupported_value_at(
                location,
                None,
                "circle radial-gradient requires one explicit radius",
            )),
            _ => Err(unsupported_value_at(
                location,
                None,
                "radial-gradient has an invalid explicit size",
            )),
        },
    }
}

pub(super) fn parse_image_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssImageLayer, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssImageLayer::None);
    }
    parse_url(input).map(CssImageLayer::Url)
}

pub(super) fn parse_url<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssUrl, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::UnquotedUrl(value) => CssUrl::try_new(value.to_string())
            .ok_or_else(|| unsupported_value(input, None, "URL is empty")),
        Token::Function(name) if name.eq_ignore_ascii_case("url") => {
            let (value, modifiers) = input.parse_nested_block(|input| {
                let value = input.expect_string_cloned().map_err(basic)?.to_string();
                let mut modifiers = Vec::new();
                while !input.is_exhausted() {
                    let modifier_location = input.current_source_location();
                    match input.next().map_err(basic)?.clone() {
                        Token::Ident(value) => {
                            modifiers.push(CssUrlModifier::Ident(CssIdent::new(value.to_string())));
                        }
                        Token::Function(name) => {
                            let arguments = input.parse_nested_block(|input| {
                                let start = input.position();
                                consume_url_modifier_components(input)?;
                                Ok(CssAuthoredFunctionArguments::new(
                                    input.slice_from(start).to_owned(),
                                ))
                            })?;
                            modifiers.push(CssUrlModifier::Function(CssUrlModifierFunction::new(
                                CssIdent::new(name.to_string()),
                                arguments,
                            )));
                        }
                        token => {
                            return Err(
                                modifier_location.new_unexpected_token_error::<Error>(token)
                            );
                        }
                    }
                }
                Ok((value, modifiers))
            })?;
            if value.is_empty() {
                Err(unsupported_value(input, None, "URL is empty"))
            } else {
                Ok(CssUrl::with_modifiers(value, modifiers))
            }
        }
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn consume_url_modifier_components<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    while !input.is_exhausted() {
        let location = input.current_source_location();
        match input
            .next_including_whitespace_and_comments()
            .map_err(basic)?
            .clone()
        {
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                input.parse_nested_block(consume_url_modifier_components)?;
            }
            token @ (Token::BadString(_)
            | Token::BadUrl(_)
            | Token::CloseParenthesis
            | Token::CloseSquareBracket
            | Token::CloseCurlyBracket) => {
                return Err(location.new_unexpected_token_error::<Error>(token));
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn parse_mask_position_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMaskPositionList, ParseError<'i, Error>> {
    let mut positions = Vec::new();
    loop {
        let (current, legacy) = parse_generic_position(input)?;
        let legacy = (!position_has_typed_calculation(&legacy)).then_some(legacy);
        positions.push(CssMaskPosition::new(current, legacy));
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "mask-position list has an empty item",
            ));
        }
    }
    CssMaskPositionList::try_new(positions)
        .ok_or_else(|| unsupported_value(input, None, "mask-position list is empty"))
}

pub(super) fn parse_background_position_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundPositionList, ParseError<'i, Error>> {
    let mut positions = Vec::new();
    loop {
        positions.push(parse_background_position(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "background-position list has an empty item",
            ));
        }
    }
    CssBackgroundPositionList::try_new(positions)
        .ok_or_else(|| unsupported_value(input, None, "background-position list is empty"))
}

pub(super) fn parse_object_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssObjectPosition, ParseError<'i, Error>> {
    parse_generic_position(input).map(|(position, _)| CssObjectPosition::new(position))
}

pub(super) fn parse_transform_origin<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformOrigin, ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input)?;

    if atoms.len() <= 2
        && let Some((position, legacy)) = build_generic_position(&atoms)
    {
        let legacy = (!position_has_typed_calculation(&legacy)).then_some(legacy);
        return Ok(CssTransformOrigin::new(position, None, legacy));
    }

    if (2..=3).contains(&atoms.len()) {
        let z_index = atoms.len() - 1;
        if let Some((position, _)) = build_generic_position(&atoms[..z_index]) {
            let z = transform_origin_z(&atoms[z_index])
                .ok_or_else(|| invalid_generic_position_atom(input, &states[z_index]))?;
            let legacy = CssPosition::new(contextual_legacy_components(&atoms));
            let legacy = (!position_has_typed_calculation(&legacy)).then_some(legacy);
            return Ok(CssTransformOrigin::new(position, Some(z), legacy));
        }
    }

    let invalid_index = if atoms.len() > 3 {
        3
    } else {
        invalid_atom_index(&atoms)
    };
    Err(invalid_generic_position_atom(input, &states[invalid_index]))
}

pub(super) fn parse_css_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    parse_generic_position(input).map(|(_, legacy)| legacy)
}

pub(super) fn parse_css_position_legacy<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    parse_css_position_legacy_components(input, false)
}

fn parse_css_position_legacy_components<'i, 't>(
    input: &mut Parser<'i, 't>,
    allow_typed_calculation: bool,
) -> std::result::Result<CssPosition, ParseError<'i, Error>> {
    let mut components = Vec::new();
    while !input.is_exhausted() && !next_is_comma(input) && !next_is_delim(input, '/') {
        components.push(parse_legacy_position_component(
            input,
            &components,
            allow_typed_calculation,
        )?);
        if components.len() > 4 {
            return Err(unsupported_value(
                input,
                None,
                "position has too many components",
            ));
        }
    }
    CssPosition::try_new(components)
        .ok_or_else(|| unsupported_value(input, None, "position is empty"))
}

fn parse_legacy_position_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    previous: &[CssPositionComponent],
    allow_typed_calculation: bool,
) -> std::result::Result<CssPositionComponent, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "left" => Ok(CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Left)),
            "right" => Ok(CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Right)),
            "top" => Ok(CssPositionComponent::Vertical(CssVerticalPositionKeyword::Top)),
            "bottom" => Ok(CssPositionComponent::Vertical(CssVerticalPositionKeyword::Bottom)),
            "center" => {
                let has_horizontal = previous.iter().any(|component| matches!(component, CssPositionComponent::Horizontal(_)));
                if has_horizontal {
                    Ok(CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center))
                } else {
                    Ok(CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center))
                }
            },
            _ => Err(unsupported_value(
                input,
                None,
                unsupported_keyword_reason("position", ident.as_ref()),
            )),
        };
    }
    input.reset(&state);
    if allow_typed_calculation {
        parse_length_with(input, LengthGrammar::Position)
    } else {
        parse_length_with_context_legacy(input, LengthGrammar::Position, "position")
    }
    .map(CssPositionComponent::Length)
}

#[derive(Clone, Debug)]
enum GenericPositionAtom {
    Horizontal(CssHorizontalPositionKeyword),
    Vertical(CssVerticalPositionKeyword),
    Center,
    Offset(CssPositionOffset),
}

fn parse_generic_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(CssPositionValue, CssPosition), ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input)?;
    build_generic_position(&atoms)
        .ok_or_else(|| invalid_generic_position_atom(input, &states[invalid_atom_index(&atoms)]))
}

fn parse_position_atoms<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(Vec<GenericPositionAtom>, Vec<ParserState>), ParseError<'i, Error>> {
    let mut atoms = Vec::new();
    let mut states = Vec::new();
    while !input.is_exhausted() && !next_is_comma(input) && !next_is_delim(input, '/') {
        states.push(input.state());
        atoms.push(parse_generic_position_atom(input)?);
        if atoms.len() > 4 {
            return Err(invalid_generic_position_atom(input, &states[4]));
        }
    }
    if atoms.is_empty() {
        return Err(unsupported_value(input, None, "position is empty"));
    }
    Ok((atoms, states))
}

fn parse_background_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundPosition, ParseError<'i, Error>> {
    let (atoms, states) = parse_position_atoms(input)?;
    build_background_position(&atoms).ok_or_else(|| {
        invalid_generic_position_atom(input, &states[invalid_background_atom_index(&atoms)])
    })
}

pub(super) fn parse_css_position_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPositionValue, ParseError<'i, Error>> {
    parse_generic_position(input).map(|(current, _)| current)
}

fn parse_generic_position_atom<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<GenericPositionAtom, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "left" => Ok(GenericPositionAtom::Horizontal(CssHorizontalPositionKeyword::Left)),
            "right" => Ok(GenericPositionAtom::Horizontal(CssHorizontalPositionKeyword::Right)),
            "top" => Ok(GenericPositionAtom::Vertical(CssVerticalPositionKeyword::Top)),
            "bottom" => Ok(GenericPositionAtom::Vertical(CssVerticalPositionKeyword::Bottom)),
            "center" => Ok(GenericPositionAtom::Center),
            _ => {
                input.reset(&state);
                Err(invalid_generic_position_atom(input, &state))
            },
        };
    }
    input.reset(&state);
    let value = parse_length_with(input, LengthGrammar::Position)?;
    let Some(offset) = CssPositionOffset::try_new(value) else {
        return Err(invalid_generic_position_atom(input, &state));
    };
    Ok(GenericPositionAtom::Offset(offset))
}

fn invalid_generic_position_atom<'i, 't>(
    input: &mut Parser<'i, 't>,
    state: &ParserState,
) -> ParseError<'i, Error> {
    input.reset(state);
    let location = input.current_source_location();
    match input.next() {
        Ok(token) => location.new_unexpected_token_error::<Error>(token.clone()),
        Err(error) => error.into(),
    }
}

fn transform_origin_z(atom: &GenericPositionAtom) -> Option<CssTransformOriginZ> {
    let GenericPositionAtom::Offset(offset) = atom else {
        return None;
    };
    CssTransformOriginZ::try_new(offset.value().clone())
}

fn invalid_atom_index(atoms: &[GenericPositionAtom]) -> usize {
    match atoms.len() {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => {
            if build_generic_position(&atoms[..2]).is_some() {
                2
            } else if !matches!(
                atoms[0],
                GenericPositionAtom::Horizontal(
                    CssHorizontalPositionKeyword::Left | CssHorizontalPositionKeyword::Right
                ) | GenericPositionAtom::Vertical(
                    CssVerticalPositionKeyword::Top | CssVerticalPositionKeyword::Bottom
                )
            ) {
                0
            } else if !matches!(atoms[1], GenericPositionAtom::Offset(_)) {
                1
            } else if !matches!(
                atoms[2],
                GenericPositionAtom::Horizontal(
                    CssHorizontalPositionKeyword::Left | CssHorizontalPositionKeyword::Right
                ) | GenericPositionAtom::Vertical(
                    CssVerticalPositionKeyword::Top | CssVerticalPositionKeyword::Bottom
                )
            ) {
                2
            } else if !matches!(atoms[3], GenericPositionAtom::Offset(_)) {
                3
            } else {
                2
            }
        }
        _ => 0,
    }
}

fn build_generic_position(
    atoms: &[GenericPositionAtom],
) -> Option<(CssPositionValue, CssPosition)> {
    use GenericPositionAtom::{Center, Horizontal, Offset, Vertical};

    let (horizontal, vertical, components) = match atoms {
        [Horizontal(keyword)] => (
            horizontal_keyword(*keyword),
            CssVerticalPosition::Center,
            vec![CssPositionComponent::Horizontal(*keyword)],
        ),
        [Vertical(keyword)] => (
            CssHorizontalPosition::Center,
            vertical_keyword(*keyword),
            vec![CssPositionComponent::Vertical(*keyword)],
        ),
        [Center] => (
            CssHorizontalPosition::Center,
            CssVerticalPosition::Center,
            vec![CssPositionComponent::Horizontal(
                CssHorizontalPositionKeyword::Center,
            )],
        ),
        [Offset(offset)] => (
            CssHorizontalPosition::Offset(offset.clone()),
            CssVerticalPosition::Center,
            vec![CssPositionComponent::Length(offset.value().clone())],
        ),
        [Horizontal(horizontal), Vertical(vertical)] => (
            horizontal_keyword(*horizontal),
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Horizontal(*horizontal),
                CssPositionComponent::Vertical(*vertical),
            ],
        ),
        [Vertical(vertical), Horizontal(horizontal)] => (
            horizontal_keyword(*horizontal),
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Vertical(*vertical),
                CssPositionComponent::Horizontal(*horizontal),
            ],
        ),
        [Horizontal(horizontal), Center] => (
            horizontal_keyword(*horizontal),
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Horizontal(*horizontal),
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
            ],
        ),
        [Center, Horizontal(horizontal)] => (
            horizontal_keyword(*horizontal),
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
                CssPositionComponent::Horizontal(*horizontal),
            ],
        ),
        [Vertical(vertical), Center] => (
            CssHorizontalPosition::Center,
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Vertical(*vertical),
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
            ],
        ),
        [Center, Vertical(vertical)] => (
            CssHorizontalPosition::Center,
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
                CssPositionComponent::Vertical(*vertical),
            ],
        ),
        [Center, Center] => (
            CssHorizontalPosition::Center,
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
            ],
        ),
        [Horizontal(horizontal), Offset(offset)] => (
            horizontal_keyword(*horizontal),
            CssVerticalPosition::Offset(offset.clone()),
            vec![
                CssPositionComponent::Horizontal(*horizontal),
                CssPositionComponent::Length(offset.value().clone()),
            ],
        ),
        [Center, Offset(offset)] => (
            CssHorizontalPosition::Center,
            CssVerticalPosition::Offset(offset.clone()),
            vec![
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center),
                CssPositionComponent::Length(offset.value().clone()),
            ],
        ),
        [Offset(offset), Vertical(vertical)] => (
            CssHorizontalPosition::Offset(offset.clone()),
            vertical_keyword(*vertical),
            vec![
                CssPositionComponent::Length(offset.value().clone()),
                CssPositionComponent::Vertical(*vertical),
            ],
        ),
        [Offset(offset), Center] => (
            CssHorizontalPosition::Offset(offset.clone()),
            CssVerticalPosition::Center,
            vec![
                CssPositionComponent::Length(offset.value().clone()),
                CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center),
            ],
        ),
        [Offset(horizontal), Offset(vertical)] => (
            CssHorizontalPosition::Offset(horizontal.clone()),
            CssVerticalPosition::Offset(vertical.clone()),
            vec![
                CssPositionComponent::Length(horizontal.value().clone()),
                CssPositionComponent::Length(vertical.value().clone()),
            ],
        ),
        [
            Horizontal(horizontal),
            Offset(horizontal_offset),
            Vertical(vertical),
            Offset(vertical_offset),
        ] if is_horizontal_edge(*horizontal) && is_vertical_edge(*vertical) => (
            horizontal_edge_offset(*horizontal, horizontal_offset.clone()),
            vertical_edge_offset(*vertical, vertical_offset.clone()),
            legacy_components(atoms),
        ),
        [
            Vertical(vertical),
            Offset(vertical_offset),
            Horizontal(horizontal),
            Offset(horizontal_offset),
        ] if is_vertical_edge(*vertical) && is_horizontal_edge(*horizontal) => (
            horizontal_edge_offset(*horizontal, horizontal_offset.clone()),
            vertical_edge_offset(*vertical, vertical_offset.clone()),
            legacy_components(atoms),
        ),
        _ => return None,
    };

    Some((
        CssPositionValue::new(horizontal, vertical),
        CssPosition::new(components),
    ))
}

fn invalid_background_atom_index(atoms: &[GenericPositionAtom]) -> usize {
    if atoms.len() == 4 && build_background_position(&atoms[..3]).is_some() {
        3
    } else {
        invalid_atom_index(atoms)
    }
}

fn build_background_position(atoms: &[GenericPositionAtom]) -> Option<CssBackgroundPosition> {
    use GenericPositionAtom::{Center, Horizontal, Offset, Vertical};

    let (horizontal, vertical) = match atoms {
        [Horizontal(horizontal), Offset(offset), Vertical(vertical)]
            if is_horizontal_edge(*horizontal) && is_vertical_edge(*vertical) =>
        {
            (
                horizontal_edge_offset(*horizontal, offset.clone()),
                vertical_keyword(*vertical),
            )
        }
        [Horizontal(horizontal), Offset(offset), Center] if is_horizontal_edge(*horizontal) => (
            horizontal_edge_offset(*horizontal, offset.clone()),
            CssVerticalPosition::Center,
        ),
        [Vertical(vertical), Offset(offset), Horizontal(horizontal)]
            if is_vertical_edge(*vertical) && is_horizontal_edge(*horizontal) =>
        {
            (
                horizontal_keyword(*horizontal),
                vertical_edge_offset(*vertical, offset.clone()),
            )
        }
        [Vertical(vertical), Offset(offset), Center] if is_vertical_edge(*vertical) => (
            CssHorizontalPosition::Center,
            vertical_edge_offset(*vertical, offset.clone()),
        ),
        [Horizontal(horizontal), Vertical(vertical), Offset(offset)]
            if is_horizontal_edge(*horizontal) && is_vertical_edge(*vertical) =>
        {
            (
                horizontal_keyword(*horizontal),
                vertical_edge_offset(*vertical, offset.clone()),
            )
        }
        [Center, Vertical(vertical), Offset(offset)] if is_vertical_edge(*vertical) => (
            CssHorizontalPosition::Center,
            vertical_edge_offset(*vertical, offset.clone()),
        ),
        [Vertical(vertical), Horizontal(horizontal), Offset(offset)]
            if is_vertical_edge(*vertical) && is_horizontal_edge(*horizontal) =>
        {
            (
                horizontal_edge_offset(*horizontal, offset.clone()),
                vertical_keyword(*vertical),
            )
        }
        [Center, Horizontal(horizontal), Offset(offset)] if is_horizontal_edge(*horizontal) => (
            horizontal_edge_offset(*horizontal, offset.clone()),
            CssVerticalPosition::Center,
        ),
        _ => {
            let (position, legacy) = build_generic_position(atoms)?;
            let legacy = (!position_has_typed_calculation(&legacy)).then_some(legacy);
            return Some(CssBackgroundPosition::new(
                position.horizontal().clone(),
                position.vertical().clone(),
                legacy,
            ));
        }
    };

    let legacy = CssPosition::new(contextual_legacy_components(atoms));
    let legacy = (!position_has_typed_calculation(&legacy)).then_some(legacy);
    Some(CssBackgroundPosition::new(horizontal, vertical, legacy))
}

fn contextual_legacy_components(atoms: &[GenericPositionAtom]) -> Vec<CssPositionComponent> {
    let mut components = Vec::with_capacity(atoms.len());
    for atom in atoms {
        let component = match atom {
            GenericPositionAtom::Horizontal(keyword) => CssPositionComponent::Horizontal(*keyword),
            GenericPositionAtom::Vertical(keyword) => CssPositionComponent::Vertical(*keyword),
            GenericPositionAtom::Center => {
                if components
                    .iter()
                    .any(|component| matches!(component, CssPositionComponent::Horizontal(_)))
                {
                    CssPositionComponent::Vertical(CssVerticalPositionKeyword::Center)
                } else {
                    CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center)
                }
            }
            GenericPositionAtom::Offset(offset) => {
                CssPositionComponent::Length(offset.value().clone())
            }
        };
        components.push(component);
    }
    components
}

fn position_has_typed_calculation(position: &CssPosition) -> bool {
    position.components().iter().any(|component| {
        matches!(
            component,
            CssPositionComponent::Length(CssLength::Calc(CssCalcLength::Typed(_)))
        )
    })
}

fn legacy_components(atoms: &[GenericPositionAtom]) -> Vec<CssPositionComponent> {
    atoms
        .iter()
        .map(|atom| match atom {
            GenericPositionAtom::Horizontal(keyword) => CssPositionComponent::Horizontal(*keyword),
            GenericPositionAtom::Vertical(keyword) => CssPositionComponent::Vertical(*keyword),
            GenericPositionAtom::Center => {
                CssPositionComponent::Horizontal(CssHorizontalPositionKeyword::Center)
            }
            GenericPositionAtom::Offset(offset) => {
                CssPositionComponent::Length(offset.value().clone())
            }
        })
        .collect()
}

const fn horizontal_keyword(keyword: CssHorizontalPositionKeyword) -> CssHorizontalPosition {
    match keyword {
        CssHorizontalPositionKeyword::Left => CssHorizontalPosition::Left,
        CssHorizontalPositionKeyword::Center => CssHorizontalPosition::Center,
        CssHorizontalPositionKeyword::Right => CssHorizontalPosition::Right,
    }
}

const fn vertical_keyword(keyword: CssVerticalPositionKeyword) -> CssVerticalPosition {
    match keyword {
        CssVerticalPositionKeyword::Top => CssVerticalPosition::Top,
        CssVerticalPositionKeyword::Center => CssVerticalPosition::Center,
        CssVerticalPositionKeyword::Bottom => CssVerticalPosition::Bottom,
    }
}

const fn is_horizontal_edge(keyword: CssHorizontalPositionKeyword) -> bool {
    matches!(
        keyword,
        CssHorizontalPositionKeyword::Left | CssHorizontalPositionKeyword::Right
    )
}

const fn is_vertical_edge(keyword: CssVerticalPositionKeyword) -> bool {
    matches!(
        keyword,
        CssVerticalPositionKeyword::Top | CssVerticalPositionKeyword::Bottom
    )
}

fn horizontal_edge_offset(
    keyword: CssHorizontalPositionKeyword,
    offset: CssPositionOffset,
) -> CssHorizontalPosition {
    match keyword {
        CssHorizontalPositionKeyword::Left => CssHorizontalPosition::LeftOffset(offset),
        CssHorizontalPositionKeyword::Right => CssHorizontalPosition::RightOffset(offset),
        CssHorizontalPositionKeyword::Center => CssHorizontalPosition::Center,
    }
}

fn vertical_edge_offset(
    keyword: CssVerticalPositionKeyword,
    offset: CssPositionOffset,
) -> CssVerticalPosition {
    match keyword {
        CssVerticalPositionKeyword::Top => CssVerticalPosition::TopOffset(offset),
        CssVerticalPositionKeyword::Bottom => CssVerticalPosition::BottomOffset(offset),
        CssVerticalPositionKeyword::Center => CssVerticalPosition::Center,
    }
}

pub(super) fn parse_background_size_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundSizeList, ParseError<'i, Error>> {
    let mut sizes = Vec::new();
    loop {
        sizes.push(parse_background_size(input)?);
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
) -> std::result::Result<CssBackgroundSize, ParseError<'i, Error>> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "cover" => Ok(CssBackgroundSize::Cover),
            "contain" => Ok(CssBackgroundSize::Contain),
            "auto" => {
                let height = if !input.is_exhausted() && !next_is_comma(input) {
                    Some(parse_background_size_component(input)?)
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

    let width = parse_background_size_component(input)?;
    let height = if !input.is_exhausted() && !next_is_comma(input) {
        Some(parse_background_size_component(input)?)
    } else {
        None
    };
    Ok(CssBackgroundSize::Explicit { width, height })
}

pub(super) fn parse_background_size_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBackgroundSizeComponent, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssBackgroundSizeComponent::Auto)
    } else {
        parse_length_with(input, LengthGrammar::BackgroundSize)
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

pub(super) fn parse_cursor<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCursor, ParseError<'i, Error>> {
    let mut urls = Vec::new();
    while let Ok(url) = input.try_parse(parse_url) {
        urls.push(url);
        input.expect_comma().map_err(basic)?;
    }
    let fallback = parse_cursor_keyword(input)?;
    if urls.is_empty() {
        Ok(CssCursor::Keyword(fallback))
    } else {
        Ok(CssCursor::urls(urls, fallback))
    }
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
) -> std::result::Result<CssOutline, ParseError<'i, Error>> {
    let mut width = None;
    let mut style = None;
    let mut color = None;
    while !input.is_exhausted() {
        if width.is_none()
            && let Ok(parsed_width) = input.try_parse(parse_outline_width)
        {
            width = Some(parsed_width);
            continue;
        }
        if style.is_none()
            && let Ok(parsed_style) = input.try_parse(parse_outline_style)
        {
            style = Some(parsed_style);
            continue;
        }
        if color.is_none()
            && let Ok(parsed_color) = input.try_parse(parse_color)
        {
            color = Some(parsed_color);
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "unsupported outline component",
        ));
    }
    if width.is_none() && style.is_none() && color.is_none() {
        None
    } else {
        Some(CssOutline::new_current(width, style, color))
    }
    .ok_or_else(|| unsupported_value(input, None, "outline shorthand is empty"))
}

pub(super) fn parse_outline_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOutlineStyle, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssOutlineStyle::Auto)
    } else {
        parse_border_style(input).map(CssOutlineStyle::Border)
    }
}

pub(super) fn parse_outline_width<'i, 't>(
    input: &mut Parser<'i, 't>,
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
    parse_length_with_context(input, LengthGrammar::BorderWidth, "outline-width")
        .map(CssOutlineWidth::Length)
}

#[cfg(test)]
mod tests {
    use cssparser::{Parser, ParserInput};

    use super::*;

    fn parse_current(source: &str) -> CssPositionValue {
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        parser
            .parse_entirely(parse_css_position_value)
            .expect("valid generic position")
    }

    #[test]
    fn generic_position_model_distinguishes_omitted_and_free_offset_axes() {
        let top = parse_current("top");
        assert!(matches!(top.horizontal(), CssHorizontalPosition::Center));
        assert!(matches!(top.vertical(), CssVerticalPosition::Top));

        let free = parse_current("25% 10px");
        assert!(matches!(
            free.horizontal(),
            CssHorizontalPosition::Offset(offset)
                if matches!(offset.value(), CssLength::Percent(value) if value.value() == 25.0)
        ));
        assert!(matches!(
            free.vertical(),
            CssVerticalPosition::Offset(offset)
                if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)
        ));
    }

    #[test]
    fn generic_position_model_retains_each_edge_offset_origin_and_pair_order() {
        for source in ["left 10px bottom 20%", "bottom 20% left 10px"] {
            let position = parse_current(source);
            assert!(matches!(
                position.horizontal(),
                CssHorizontalPosition::LeftOffset(offset)
                    if matches!(offset.value(), CssLength::Px(value) if value.value() == 10.0)
            ));
            assert!(matches!(
                position.vertical(),
                CssVerticalPosition::BottomOffset(offset)
                    if matches!(offset.value(), CssLength::Percent(value) if value.value() == 20.0)
            ));
        }

        let opposite = parse_current("right calc(1px * 2) top calc(10% + 2px)");
        assert!(matches!(
            opposite.horizontal(),
            CssHorizontalPosition::RightOffset(offset)
                if matches!(offset.value(), CssLength::Calc(CssCalcLength::Typed(_)))
        ));
        assert!(matches!(
            opposite.vertical(),
            CssVerticalPosition::TopOffset(offset)
                if matches!(offset.value(), CssLength::Calc(_))
        ));
    }
}
