use cssparser::{ParseError, Parser, ParserInput, ToCss, Token, match_ignore_ascii_case};

use super::background::{
    parse_background_repeat, parse_background_size, parse_css_position, parse_css_position_legacy,
    parse_css_position_value, parse_image_layer, parse_url,
};
use super::box_model::{expand_radius_components, parse_drop_shadow, parse_shadow};
use super::values::{
    CalculationRoot, LengthGrammar, checked_percentage_value, next_is_comma, next_is_delim,
    next_is_ident, parse_length_with, parse_length_with_context, parse_length_with_context_legacy,
    parse_number, parse_typed_calculation,
};
use crate::error::{CssFeatureId, Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) fn parse_clip<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssClip, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssClip::Auto);
    }

    let location = input.current_source_location();
    let name = input.expect_function().map_err(basic)?;
    if !name.eq_ignore_ascii_case("rect") {
        return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported clip function `{name}`"),
        ));
    }
    input.parse_nested_block(parse_clip_rect).map(CssClip::Rect)
}

pub(super) fn parse_outline_offset<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOutlineOffset, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_length_with_context(input, LengthGrammar::OutlineOffset, "outline-offset")?;
    CssOutlineOffset::try_new(value)
        .ok_or_else(|| unsupported_value_at(location, None, "outline-offset requires a length"))
}

pub(super) fn parse_transform_box<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformBox, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssBoxEdgeKeyword::from_keyword(ident.as_ref())
        .and_then(CssTransformBox::try_new)
        .ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                unsupported_keyword_reason("transform-box", ident.as_ref()),
            )
        })
}

pub(super) fn parse_blend_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBlendMode, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssBlendMode::from_keyword(ident.as_ref()).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("blend-mode", ident.as_ref()),
        )
    })
}

pub(super) fn parse_blend_mode_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBlendModeList, ParseError<'i, Error>> {
    let mut modes = vec![parse_blend_mode(input)?];
    while !input.is_exhausted() {
        input.expect_comma().map_err(basic)?;
        modes.push(parse_blend_mode(input)?);
    }
    CssBlendModeList::try_new(modes)
        .ok_or_else(|| unsupported_value(input, None, "blend-mode list is empty"))
}

pub(super) fn parse_isolation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssIsolation, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssIsolation::Auto),
        "isolate" => Ok(CssIsolation::Isolate),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("isolation", ident.as_ref()),
        )),
    }
}

fn parse_clip_rect<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssClipRect, ParseError<'i, Error>> {
    let top = parse_clip_edge(input)?;
    let comma_separated = input.try_parse(Parser::expect_comma).is_ok();
    let right = parse_clip_edge(input)?;
    if comma_separated {
        input.expect_comma().map_err(basic)?;
    }
    let bottom = parse_clip_edge(input)?;
    if comma_separated {
        input.expect_comma().map_err(basic)?;
    }
    let left = parse_clip_edge(input)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(CssClipRect::new(top, right, bottom, left))
}

fn parse_clip_edge<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssClipEdge, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssClipEdge::Auto);
    }
    let location = input.current_source_location();
    let value = parse_length_with(input, LengthGrammar::Clip)?;
    CssClipLength::try_new(value)
        .map(CssClipEdge::Length)
        .ok_or_else(|| unsupported_value_at(location, None, "clip edge requires a length or auto"))
}

pub(super) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.blend-mode"),
    CssFeatureId::new("official.value.transform-list"),
    CssFeatureId::new("official.value.transform-function"),
    CssFeatureId::new("official.value.transform.matrix"),
    CssFeatureId::new("official.value.transform.translate"),
    CssFeatureId::new("official.value.transform.translate-x"),
    CssFeatureId::new("official.value.transform.translate-y"),
    CssFeatureId::new("official.value.transform.scale"),
    CssFeatureId::new("official.value.transform.scale-x"),
    CssFeatureId::new("official.value.transform.scale-y"),
    CssFeatureId::new("official.value.transform.rotate"),
    CssFeatureId::new("official.value.transform.skew"),
    CssFeatureId::new("official.value.transform.skew-x"),
    CssFeatureId::new("official.value.transform.skew-y"),
    CssFeatureId::new("official.value.shadow"),
    CssFeatureId::new("ext.value.transform.matrix3d"),
    CssFeatureId::new("ext.value.transform.perspective"),
    CssFeatureId::new("ext.value.transform.rotate3d"),
    CssFeatureId::new("ext.value.transform.rotate-x"),
    CssFeatureId::new("ext.value.transform.rotate-y"),
    CssFeatureId::new("ext.value.transform.rotate-z"),
    CssFeatureId::new("ext.value.transform.scale3d"),
    CssFeatureId::new("ext.value.transform.scale-z"),
    CssFeatureId::new("ext.value.transform.translate3d"),
    CssFeatureId::new("ext.value.transform.translate-z"),
    CssFeatureId::new("ext.value.filter-function-list"),
    CssFeatureId::new("ext.value.filter.blur"),
    CssFeatureId::new("ext.value.filter.brightness"),
    CssFeatureId::new("ext.value.filter.contrast"),
    CssFeatureId::new("ext.value.filter.grayscale"),
    CssFeatureId::new("ext.value.filter.hue-rotate"),
    CssFeatureId::new("ext.value.filter.invert"),
    CssFeatureId::new("ext.value.filter.opacity"),
    CssFeatureId::new("ext.value.filter.saturate"),
    CssFeatureId::new("ext.value.filter.sepia"),
    CssFeatureId::new("ext.value.filter.drop-shadow"),
    CssFeatureId::new("ext.value.basic-shape"),
    CssFeatureId::new("ext.value.basic-shape.inset"),
    CssFeatureId::new("ext.value.basic-shape.circle"),
    CssFeatureId::new("ext.value.basic-shape.ellipse"),
    CssFeatureId::new("ext.value.basic-shape.polygon"),
];

pub(super) fn parse_transform<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedTransform, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssParsedTransform::new(
            CssTransformValue::None,
            CssTransform::None,
        ));
    }
    let mut current_functions = Vec::new();
    let mut legacy_functions = Vec::new();
    while !input.is_exhausted() {
        let (current, legacy) = parse_transform_function(input)?;
        current_functions.push(current);
        legacy_functions.push(legacy);
    }
    let current = CssTransformFunctionValueList::try_new(current_functions)
        .map(CssTransformValue::Functions)
        .ok_or_else(|| unsupported_value(input, None, "transform function list is empty"))?;
    let legacy = CssTransformFunctionList::try_new(legacy_functions)
        .map(CssTransform::Functions)
        .ok_or_else(|| unsupported_value(input, None, "transform function list is empty"))?;
    Ok(CssParsedTransform::new(current, legacy))
}

fn parse_transform_function<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(CssTransformFunctionValue, CssTransformFunction), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    let kind = parse_transform_function_kind(input, name.as_ref())?;
    let (current, arguments) = input.parse_nested_block(|input| {
        let state = input.state();
        let authored = collect_transform_authored_tokens(input)?;
        input.reset(&state);
        let current = parse_transform_function_value(input, kind)?;
        Ok((
            current,
            CssTransformArguments::new(CssAuthoredFunctionArguments::new(authored)),
        ))
    })?;
    Ok((current, CssTransformFunction::new(kind, arguments)))
}

fn collect_transform_authored_tokens<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<String, ParseError<'i, Error>> {
    let mut value = String::new();
    while !input.is_exhausted() {
        let token = input.next().map_err(basic)?.clone();
        let token_css = match token {
            Token::Function(_) => {
                let mut css = token.to_css_string();
                css.push_str(&input.parse_nested_block(collect_transform_authored_tokens)?);
                css.push(')');
                css
            }
            Token::ParenthesisBlock => {
                let nested = input.parse_nested_block(collect_transform_authored_tokens)?;
                format!("({nested})")
            }
            _ => token.to_css_string(),
        };
        if matches!(token, Token::Comma) {
            if value.ends_with(' ') {
                value.pop();
            }
            value.push_str(", ");
        } else {
            if !value.is_empty() && !value.ends_with(' ') {
                value.push(' ');
            }
            value.push_str(&token_css);
        }
    }
    Ok(value.trim().to_owned())
}

pub(super) fn parse_transform_function_kind<'i, 't>(
    input: &Parser<'i, 't>,
    name: &str,
) -> std::result::Result<CssTransformFunctionKind, ParseError<'i, Error>> {
    match name.to_ascii_lowercase().as_str() {
        "matrix" => Ok(CssTransformFunctionKind::Matrix),
        "matrix3d" => Ok(CssTransformFunctionKind::Matrix3d),
        "perspective" => Ok(CssTransformFunctionKind::Perspective),
        "rotate" => Ok(CssTransformFunctionKind::Rotate),
        "rotate3d" => Ok(CssTransformFunctionKind::Rotate3d),
        "rotatex" => Ok(CssTransformFunctionKind::RotateX),
        "rotatey" => Ok(CssTransformFunctionKind::RotateY),
        "rotatez" => Ok(CssTransformFunctionKind::RotateZ),
        "scale" => Ok(CssTransformFunctionKind::Scale),
        "scale3d" => Ok(CssTransformFunctionKind::Scale3d),
        "scalex" => Ok(CssTransformFunctionKind::ScaleX),
        "scaley" => Ok(CssTransformFunctionKind::ScaleY),
        "scalez" => Ok(CssTransformFunctionKind::ScaleZ),
        "skew" => Ok(CssTransformFunctionKind::Skew),
        "skewx" => Ok(CssTransformFunctionKind::SkewX),
        "skewy" => Ok(CssTransformFunctionKind::SkewY),
        "translate" => Ok(CssTransformFunctionKind::Translate),
        "translate3d" => Ok(CssTransformFunctionKind::Translate3d),
        "translatex" => Ok(CssTransformFunctionKind::TranslateX),
        "translatey" => Ok(CssTransformFunctionKind::TranslateY),
        "translatez" => Ok(CssTransformFunctionKind::TranslateZ),
        _ => Err(unsupported_value(
            input,
            None,
            format!("unsupported transform function `{name}`"),
        )),
    }
}

fn parse_transform_function_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    kind: CssTransformFunctionKind,
) -> std::result::Result<CssTransformFunctionValue, ParseError<'i, Error>> {
    let value = match kind {
        CssTransformFunctionKind::Matrix => {
            let components = parse_exact_comma_list(input, 6, parse_transform_number)?;
            let components = components.try_into().map_err(|_| {
                unsupported_value(input, None, "matrix() requires exactly six numbers")
            })?;
            CssTransformFunctionValue::Matrix(CssTransformMatrix::new(components))
        }
        CssTransformFunctionKind::Matrix3d => {
            let components = parse_exact_comma_list(input, 16, parse_transform_number)?;
            let components = components.try_into().map_err(|_| {
                unsupported_value(input, None, "matrix3d() requires exactly sixteen numbers")
            })?;
            CssTransformFunctionValue::Matrix3d(Box::new(CssTransformMatrix3d::new(components)))
        }
        CssTransformFunctionKind::Perspective => {
            let perspective = if input
                .try_parse(|input| input.expect_ident_matching("none"))
                .is_ok()
            {
                CssTransformPerspective::None
            } else {
                let location = input.current_source_location();
                let length = parse_transform_length(input)?;
                let length = CssTransformNonNegativeLength::try_new(length.value().clone())
                    .ok_or_else(|| {
                        unsupported_value_at(
                            location,
                            None,
                            "perspective() requires a non-negative length or none",
                        )
                    })?;
                CssTransformPerspective::Length(length)
            };
            input.expect_exhausted().map_err(basic)?;
            CssTransformFunctionValue::Perspective(perspective)
        }
        CssTransformFunctionKind::Rotate => {
            CssTransformFunctionValue::Rotate(parse_one(input, parse_transform_angle)?)
        }
        CssTransformFunctionKind::Rotate3d => {
            let x = parse_transform_number(input)?;
            input.expect_comma().map_err(basic)?;
            let y = parse_transform_number(input)?;
            input.expect_comma().map_err(basic)?;
            let z = parse_transform_number(input)?;
            input.expect_comma().map_err(basic)?;
            let angle = parse_transform_angle(input)?;
            input.expect_exhausted().map_err(basic)?;
            CssTransformFunctionValue::Rotate3d(CssTransformRotate3d::new(x, y, z, angle))
        }
        CssTransformFunctionKind::RotateX => {
            CssTransformFunctionValue::RotateX(parse_one(input, parse_transform_angle)?)
        }
        CssTransformFunctionKind::RotateY => {
            CssTransformFunctionValue::RotateY(parse_one(input, parse_transform_angle)?)
        }
        CssTransformFunctionKind::RotateZ => {
            CssTransformFunctionValue::RotateZ(parse_one(input, parse_transform_angle)?)
        }
        CssTransformFunctionKind::Scale => {
            let (x, y) = parse_one_or_two(input, parse_transform_number)?;
            CssTransformFunctionValue::Scale(CssTransformScale::new(x, y))
        }
        CssTransformFunctionKind::Scale3d => {
            let mut components = parse_exact_comma_list(input, 3, parse_transform_scale_component)?;
            let z = components.pop().ok_or_else(|| {
                unsupported_value(input, None, "scale3d() requires exactly three operands")
            })?;
            let y = components.pop().ok_or_else(|| {
                unsupported_value(input, None, "scale3d() requires exactly three operands")
            })?;
            let x = components.pop().ok_or_else(|| {
                unsupported_value(input, None, "scale3d() requires exactly three operands")
            })?;
            CssTransformFunctionValue::Scale3d(CssTransformScale3d::new(x, y, z))
        }
        CssTransformFunctionKind::ScaleX => {
            CssTransformFunctionValue::ScaleX(parse_one(input, parse_transform_number)?)
        }
        CssTransformFunctionKind::ScaleY => {
            CssTransformFunctionValue::ScaleY(parse_one(input, parse_transform_number)?)
        }
        CssTransformFunctionKind::ScaleZ => {
            CssTransformFunctionValue::ScaleZ(parse_one(input, parse_transform_scale_component)?)
        }
        CssTransformFunctionKind::Skew => {
            let (x, y) = parse_one_or_two(input, parse_transform_angle)?;
            CssTransformFunctionValue::Skew(CssTransformSkew::new(x, y))
        }
        CssTransformFunctionKind::SkewX => {
            CssTransformFunctionValue::SkewX(parse_one(input, parse_transform_angle)?)
        }
        CssTransformFunctionKind::SkewY => {
            CssTransformFunctionValue::SkewY(parse_one(input, parse_transform_angle)?)
        }
        CssTransformFunctionKind::Translate => {
            let (x, y) = parse_one_or_two(input, parse_transform_length_percentage)?;
            CssTransformFunctionValue::Translate(CssTransformTranslate::new(x, y))
        }
        CssTransformFunctionKind::Translate3d => {
            let x = parse_transform_length_percentage(input)?;
            input.expect_comma().map_err(basic)?;
            let y = parse_transform_length_percentage(input)?;
            input.expect_comma().map_err(basic)?;
            let z = parse_transform_length(input)?;
            input.expect_exhausted().map_err(basic)?;
            CssTransformFunctionValue::Translate3d(CssTransformTranslate3d::new(x, y, z))
        }
        CssTransformFunctionKind::TranslateX => CssTransformFunctionValue::TranslateX(parse_one(
            input,
            parse_transform_length_percentage,
        )?),
        CssTransformFunctionKind::TranslateY => CssTransformFunctionValue::TranslateY(parse_one(
            input,
            parse_transform_length_percentage,
        )?),
        CssTransformFunctionKind::TranslateZ => {
            CssTransformFunctionValue::TranslateZ(parse_one(input, parse_transform_length)?)
        }
    };
    Ok(value)
}

fn parse_exact_comma_list<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    count: usize,
    mut parse: impl FnMut(&mut Parser<'i, 't>) -> std::result::Result<T, ParseError<'i, Error>>,
) -> std::result::Result<Vec<T>, ParseError<'i, Error>> {
    let mut values = Vec::with_capacity(count);
    for index in 0..count {
        if index != 0 {
            input.expect_comma().map_err(basic)?;
        }
        values.push(parse(input)?);
    }
    input.expect_exhausted().map_err(basic)?;
    Ok(values)
}

fn parse_one<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    mut parse: impl FnMut(&mut Parser<'i, 't>) -> std::result::Result<T, ParseError<'i, Error>>,
) -> std::result::Result<T, ParseError<'i, Error>> {
    let value = parse(input)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(value)
}

fn parse_one_or_two<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    mut parse: impl FnMut(&mut Parser<'i, 't>) -> std::result::Result<T, ParseError<'i, Error>>,
) -> std::result::Result<(T, Option<T>), ParseError<'i, Error>> {
    let first = parse(input)?;
    let second = if input.is_exhausted() {
        None
    } else {
        input.expect_comma().map_err(basic)?;
        Some(parse(input)?)
    };
    input.expect_exhausted().map_err(basic)?;
    Ok((first, second))
}

fn parse_transform_number<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformNumber, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => CssFiniteNumber::try_new(*value)
            .map(CssTransformNumber::Literal)
            .ok_or_else(|| unsupported_value_at(location, None, "transform number must be finite")),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Number))
            .map(CssNumberCalculation::from_expression)
            .map(CssTransformNumber::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_transform_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                "transform percentage must be finite",
            )?;
            CssFiniteNumber::try_new(value)
                .map(CssTransformPercentage::Literal)
                .ok_or_else(|| {
                    unsupported_value_at(location, None, "transform percentage must be finite")
                })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Percentage))
            .map(CssPercentageCalculation::from_expression)
            .map(CssTransformPercentage::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_transform_scale_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformScaleComponent, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(number) = input.try_parse(parse_transform_number) {
        return Ok(CssTransformScaleComponent::Number(number));
    }
    input.reset(&state);
    parse_transform_percentage(input).map(CssTransformScaleComponent::Percentage)
}

fn parse_transform_angle<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformAngle, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } if *value == 0.0 => Ok(CssTransformAngle::Zero),
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
                        format!("unsupported transform angle unit `{unit}`"),
                    ));
                }
            };
            CssAngleLiteral::try_new(*value, unit)
                .map(CssTransformAngle::Literal)
                .ok_or_else(|| {
                    unsupported_value_at(location, None, "transform angle must be finite")
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
                        "transform angle calculation must have an angle result",
                    ));
                }
                Ok(expression)
            })
            .map(CssAngleCalculation::from_expression)
            .map(CssTransformAngle::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_transform_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformLengthPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_length_with_context(input, LengthGrammar::Position, "translate")?;
    CssTransformLengthPercentage::try_new(value).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            "transform translation requires a length-percentage",
        )
    })
}

fn parse_transform_length<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTransformLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_length_with_context(input, LengthGrammar::Position, "translate")?;
    CssTransformLength::try_new(value).ok_or_else(|| {
        unsupported_value_at(location, None, "transform translation requires a length")
    })
}

pub(super) fn validate_authored_function_arguments(
    value: &str,
    mut validate: impl for<'i, 't> FnMut(&mut Parser<'i, 't>) -> bool,
) -> bool {
    let mut input = ParserInput::new(value);
    let mut parser = Parser::new(&mut input);
    validate(&mut parser) && parser.is_exhausted()
}

pub(super) fn collect_authored_tokens<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<String, ParseError<'i, Error>> {
    let mut value = String::new();
    while !input.is_exhausted() {
        let token = input.next().map_err(basic)?;
        let token_css = token.to_css_string();
        if matches!(token, Token::Comma) {
            if value.ends_with(' ') {
                value.pop();
            }
            value.push_str(", ");
        } else {
            if !value.is_empty() && !value.ends_with(' ') {
                value.push(' ');
            }
            value.push_str(&token_css);
        }
    }
    Ok(value.trim().to_owned())
}

pub(super) fn validate_non_negative_length<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    parse_length_with_context_legacy(input, LengthGrammar::BorderWidth, "function length").is_ok()
        && input.is_exhausted()
}

pub(super) fn validate_number_or_percent<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let parsed = match input.next() {
        Ok(Token::Number { .. } | Token::Percentage { .. }) => true,
        Ok(_) => false,
        Err(_) => false,
    };
    parsed && input.is_exhausted()
}

pub(super) fn validate_angle<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    match input.next() {
        Ok(Token::Dimension { unit, .. }) => {
            unit.eq_ignore_ascii_case("deg")
                || unit.eq_ignore_ascii_case("rad")
                || unit.eq_ignore_ascii_case("grad")
                || unit.eq_ignore_ascii_case("turn")
        }
        Ok(Token::Number { value, .. }) => *value == 0.0,
        _ => false,
    }
}

fn parse_radial_extent<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssRadialExtent, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "closest-side" => Ok(CssRadialExtent::ClosestSide),
        "farthest-side" => Ok(CssRadialExtent::FarthestSide),
        "closest-corner" => Ok(CssRadialExtent::ClosestCorner),
        "farthest-corner" => Ok(CssRadialExtent::FarthestCorner),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("shape radius", ident.as_ref()),
        )),
    }
}

fn parse_circle_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCircleShape, ParseError<'i, Error>> {
    let radius = if input.is_exhausted() || next_is_ident(input, "at") {
        CssCircleRadius::Default
    } else if let Ok(extent) = input.try_parse(parse_radial_extent) {
        CssCircleRadius::Extent(extent)
    } else {
        let location = input.current_source_location();
        let value = parse_length_with_context(input, LengthGrammar::Position, "circle radius")?;
        let length = CssShapeLength::try_new(value).ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                "circle radius requires a non-negative length",
            )
        })?;
        CssCircleRadius::Length(length)
    };
    Ok(CssCircleShape::new(
        radius,
        parse_optional_shape_position(input)?,
    ))
}

fn parse_ellipse_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssEllipseShape, ParseError<'i, Error>> {
    let radius = if input.is_exhausted() || next_is_ident(input, "at") {
        CssEllipseRadius::Default
    } else if let Ok(extent) = input.try_parse(parse_radial_extent) {
        CssEllipseRadius::Extent(extent)
    } else {
        let horizontal =
            parse_non_negative_shape_length_percentage(input, "ellipse horizontal radius")?;
        let vertical =
            parse_non_negative_shape_length_percentage(input, "ellipse vertical radius")?;
        CssEllipseRadius::Radii(CssEllipseRadii::new(horizontal, vertical))
    };
    Ok(CssEllipseShape::new(
        radius,
        parse_optional_shape_position(input)?,
    ))
}

fn parse_optional_shape_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Option<CssPositionValue>, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(None);
    }
    input.expect_ident_matching("at")?;
    parse_css_position_value(input).map(Some)
}

fn parse_shape_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_length_with_context(input, LengthGrammar::Position, context)?;
    CssPositionOffset::try_new(value.clone())
        .map(|_| value)
        .ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                format!("{context} requires a length-percentage"),
            )
        })
}

fn parse_non_negative_shape_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<CssShapeLengthPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_length_with_context(input, LengthGrammar::Position, context)?;
    CssShapeLengthPercentage::try_new(value).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            format!("{context} requires a non-negative length-percentage"),
        )
    })
}

fn parse_inset_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssInsetShape, ParseError<'i, Error>> {
    let mut offsets = Vec::new();
    while !input.is_exhausted() && !next_is_ident(input, "round") {
        if offsets.len() == 4 {
            return Err(unsupported_value(
                input,
                None,
                "inset shape has too many offsets",
            ));
        }
        offsets.push(parse_shape_length_percentage(input, "inset shape offset")?);
    }
    let offsets = CssInsetShapeOffsets::try_new(offsets)
        .ok_or_else(|| unsupported_value(input, None, "inset shape is missing an offset"))?;
    let round = if input.is_exhausted() {
        None
    } else {
        input.expect_ident_matching("round")?;
        Some(parse_shape_radii(input)?)
    };
    Ok(CssInsetShape::new(offsets, round))
}

fn parse_shape_radii<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssBorderRadii, ParseError<'i, Error>> {
    let horizontal = parse_shape_radius_list(input)?;
    let vertical = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        parse_shape_radius_list(input)?
    } else {
        horizontal.clone()
    };
    if !input.is_exhausted() {
        return Err(unsupported_value(input, None, "invalid inset round radii"));
    }
    let (h_top_left, h_top_right, h_bottom_right, h_bottom_left) =
        expand_radius_components(horizontal);
    let (v_top_left, v_top_right, v_bottom_right, v_bottom_left) =
        expand_radius_components(vertical);
    Ok(CssBorderRadii::new(
        CssCornerRadius::new(h_top_left, v_top_left),
        CssCornerRadius::new(h_top_right, v_top_right),
        CssCornerRadius::new(h_bottom_right, v_bottom_right),
        CssCornerRadius::new(h_bottom_left, v_bottom_left),
    ))
}

fn parse_shape_radius_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<Vec<CssLength>, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() && !next_is_delim(input, '/') {
        if values.len() == 4 {
            return Err(unsupported_value(
                input,
                None,
                "inset round has too many radii",
            ));
        }
        values.push(
            parse_non_negative_shape_length_percentage(input, "inset round radius")?
                .value()
                .clone(),
        );
    }
    if values.is_empty() {
        Err(unsupported_value(
            input,
            None,
            "inset round is missing radii",
        ))
    } else {
        Ok(values)
    }
}

fn parse_polygon_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPolygonShape, ParseError<'i, Error>> {
    let mut fill_rule = None;
    let mut round = None;
    loop {
        if fill_rule.is_none()
            && let Ok(value) = input.try_parse(parse_polygon_fill_rule)
        {
            fill_rule = Some(value);
            continue;
        }
        if round.is_none() && next_is_ident(input, "round") {
            input.expect_ident_matching("round")?;
            let location = input.current_source_location();
            let value = parse_length_with_context(input, LengthGrammar::Position, "polygon round")?;
            round = Some(CssShapeLength::try_new(value).ok_or_else(|| {
                unsupported_value_at(
                    location,
                    None,
                    "polygon round requires a non-negative length",
                )
            })?);
            continue;
        }
        break;
    }

    input.expect_comma()?;
    let mut points = Vec::new();
    loop {
        let x = parse_shape_length_percentage(input, "polygon x")?;
        let y = parse_shape_length_percentage(input, "polygon y")?;
        points.push(CssPolygonPoint::new(x, y));
        if input.is_exhausted() {
            break;
        }
        input.expect_comma()?;
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "polygon point list has an empty item",
            ));
        }
    }
    let points = CssPolygonPointList::try_new(points)
        .ok_or_else(|| unsupported_value(input, None, "polygon point list is empty"))?;
    Ok(CssPolygonShape::new(fill_rule, round, points))
}

fn parse_polygon_fill_rule<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPolygonFillRule, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "nonzero" => Ok(CssPolygonFillRule::Nonzero),
        "evenodd" => Ok(CssPolygonFillRule::Evenodd),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("polygon fill rule", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_translate<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTranslate, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssTranslate::None);
    }
    let mut values = Vec::new();
    while !input.is_exhausted() {
        values.push(parse_length_with_context(
            input,
            LengthGrammar::Position,
            "translate",
        )?);
        if values.len() > 3 {
            return Err(unsupported_value(
                input,
                None,
                "translate has too many values",
            ));
        }
    }
    CssTranslateValues::try_new(values)
        .map(CssTranslate::Values)
        .ok_or_else(|| unsupported_value(input, None, "translate is empty"))
}

pub(super) fn parse_rotate<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssRotate, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssRotate::None);
    }
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?;
    let value = match token {
        Token::Dimension { unit, .. }
            if unit.eq_ignore_ascii_case("deg")
                || unit.eq_ignore_ascii_case("rad")
                || unit.eq_ignore_ascii_case("grad")
                || unit.eq_ignore_ascii_case("turn") =>
        {
            token.to_css_string()
        }
        Token::Number { value, .. } if *value == 0.0 => token.to_css_string(),
        _ => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    Ok(CssRotate::Value(value))
}

pub(super) fn parse_scale<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssScale, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssScale::None);
    }
    let mut values = Vec::new();
    while !input.is_exhausted() {
        values.push(parse_number(input)?);
        if values.len() > 3 {
            return Err(unsupported_value(input, None, "scale has too many values"));
        }
    }
    CssScaleValues::try_new(values)
        .map(CssScale::Values)
        .ok_or_else(|| unsupported_value(input, None, "scale is empty"))
}

pub(super) fn parse_filter<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedFilter, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssParsedFilter::new(
            CssFilterValue::None,
            Some(CssFilter::None),
        ));
    }
    let mut current_functions = Vec::new();
    let mut legacy_functions = Vec::new();
    let mut belongs_to_i01 = true;
    while !input.is_exhausted() {
        let (current, legacy) = parse_filter_function(input)?;
        current_functions.push(current);
        match legacy {
            Some(legacy) => legacy_functions.push(legacy),
            None => belongs_to_i01 = false,
        }
    }
    let current = CssFilterFunctionValueList::try_new(current_functions)
        .map(CssFilterValue::Functions)
        .ok_or_else(|| unsupported_value(input, None, "filter function list is empty"))?;
    let legacy = if belongs_to_i01 {
        CssFilterFunctionList::try_new(legacy_functions).map(CssFilter::Functions)
    } else {
        None
    };
    Ok(CssParsedFilter::new(current, legacy))
}

pub(super) fn parse_filter_function<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(CssFilterFunctionValue, Option<CssFilterFunction>), ParseError<'i, Error>>
{
    if let Ok(url) = input.try_parse(parse_url) {
        return Ok((
            CssFilterFunctionValue::Url(url.clone()),
            Some(CssFilterFunction::Url(url)),
        ));
    }
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    input.parse_nested_block(|input| {
        let state = input.state();
        let authored = collect_transform_authored_tokens(input)?;
        input.reset(&state);
        let current = parse_filter_function_value(input, name.as_ref())?;
        let legacy = legacy_filter_function(name.as_ref(), &authored);
        Ok((current, legacy))
    })
}

fn parse_filter_function_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    name: &str,
) -> std::result::Result<CssFilterFunctionValue, ParseError<'i, Error>> {
    match name.to_ascii_lowercase().as_str() {
        "blur" => {
            let length = super::values::parse_shadow_blur_length(input)?;
            input.expect_exhausted().map_err(basic)?;
            CssFilterBlur::try_new(length)
                .map(CssFilterFunctionValue::Blur)
                .ok_or_else(|| {
                    unsupported_value(input, None, "blur() requires a non-negative length")
                })
        }
        "brightness" => parse_filter_amount(input).map(CssFilterFunctionValue::Brightness),
        "contrast" => parse_filter_amount(input).map(CssFilterFunctionValue::Contrast),
        "drop-shadow" => {
            let shadow = parse_drop_shadow(input)?;
            input.expect_exhausted().map_err(basic)?;
            Ok(CssFilterFunctionValue::DropShadow(shadow))
        }
        "grayscale" => parse_filter_amount(input).map(CssFilterFunctionValue::Grayscale),
        "hue-rotate" => parse_filter_angle(input).map(CssFilterFunctionValue::HueRotate),
        "invert" => parse_filter_amount(input).map(CssFilterFunctionValue::Invert),
        "opacity" => parse_filter_amount(input).map(CssFilterFunctionValue::Opacity),
        "saturate" => parse_filter_amount(input).map(CssFilterFunctionValue::Saturate),
        "sepia" => parse_filter_amount(input).map(CssFilterFunctionValue::Sepia),
        _ => Err(unsupported_value(
            input,
            None,
            format!("unsupported filter function `{name}`"),
        )),
    }
}

fn parse_filter_amount<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFilterAmount, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(CssFilterAmount::Default);
    }
    let amount = if let Ok(number) = input.try_parse(parse_filter_number) {
        CssFilterAmount::Number(number)
    } else {
        CssFilterAmount::Percentage(parse_filter_percentage(input)?)
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(amount)
}

fn parse_filter_number<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFilterNumber, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { value, .. } => CssNonNegativeNumber::try_new(*value)
            .map(CssFilterNumber::Literal)
            .ok_or_else(|| {
                unsupported_value_at(
                    location,
                    None,
                    "filter amount must be a finite non-negative number",
                )
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Number))
            .map(CssNumberCalculation::from_expression)
            .map(CssFilterNumber::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_filter_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFilterPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                "filter percentage must be finite",
            )?;
            CssNonNegativeNumber::try_new(value)
                .map(CssFilterPercentage::Literal)
                .ok_or_else(|| {
                    unsupported_value_at(location, None, "filter percentage must be non-negative")
                })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => input
            .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Percentage))
            .map(CssPercentageCalculation::from_expression)
            .map(CssFilterPercentage::Calculation),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_filter_angle<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFilterAngle, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let angle = match input.next().map_err(basic)? {
        Token::Number { value, .. } if *value == 0.0 => CssFilterAngle::Zero,
        Token::Dimension { value, unit, .. } => {
            let unit = match_ignore_ascii_case! { unit,
                "deg" => CssAngleUnit::Degrees,
                "grad" => CssAngleUnit::Gradians,
                "rad" => CssAngleUnit::Radians,
                "turn" => CssAngleUnit::Turns,
                _ => return Err(unsupported_value_at(location, None, "hue-rotate() requires an angle")),
            };
            let value = CssAngleLiteral::try_new(*value, unit).ok_or_else(|| {
                unsupported_value_at(location, None, "filter angle must be finite")
            })?;
            CssFilterAngle::Literal(value)
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            let expression = input.parse_nested_block(|input| {
                parse_typed_calculation(input, CalculationRoot::Angle)
            })?;
            CssFilterAngle::Calculation(CssAngleCalculation::from_expression(expression))
        }
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(angle)
}

fn legacy_filter_function(name: &str, authored: &str) -> Option<CssFilterFunction> {
    if authored.is_empty() {
        return None;
    }
    let is_i01 = validate_authored_function_arguments(authored, |input| {
        match name.to_ascii_lowercase().as_str() {
            "blur" => validate_non_negative_length(input),
            "brightness" | "contrast" | "grayscale" | "invert" | "opacity" | "saturate"
            | "sepia" => validate_number_or_percent(input),
            "hue-rotate" => validate_angle(input) && input.is_exhausted(),
            "drop-shadow" => {
                input
                    .try_parse(parse_shadow)
                    .is_ok_and(|shadow| shadow.has_exact_i01_projection())
                    && input.is_exhausted()
            }
            _ => false,
        }
    });
    if !is_i01 {
        return None;
    }
    let arguments = CssFilterArguments::new(CssAuthoredFunctionArguments::new(authored));
    match name.to_ascii_lowercase().as_str() {
        "blur" => Some(CssFilterFunction::Blur(arguments)),
        "brightness" => Some(CssFilterFunction::Brightness(arguments)),
        "contrast" => Some(CssFilterFunction::Contrast(arguments)),
        "drop-shadow" => Some(CssFilterFunction::DropShadow(arguments)),
        "grayscale" => Some(CssFilterFunction::Grayscale(arguments)),
        "hue-rotate" => Some(CssFilterFunction::HueRotate(arguments)),
        "invert" => Some(CssFilterFunction::Invert(arguments)),
        "opacity" => Some(CssFilterFunction::Opacity(arguments)),
        "saturate" => Some(CssFilterFunction::Saturate(arguments)),
        "sepia" => Some(CssFilterFunction::Sepia(arguments)),
        _ => None,
    }
}

pub(super) fn parse_clip_path<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedClipPath, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssParsedClipPath::new(
            Some(CssClipPathValue::None),
            Some(CssClipPath::None),
        ));
    }
    if let Ok(url) = input.try_parse(parse_url) {
        return Ok(CssParsedClipPath::new(
            Some(CssClipPathValue::Url(url.clone())),
            Some(CssClipPath::Url(url)),
        ));
    }
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    let normalized_name = name.to_ascii_lowercase();
    if !matches!(
        normalized_name.as_str(),
        "inset" | "circle" | "ellipse" | "polygon"
    ) {
        return Err(unsupported_value(
            input,
            None,
            format!("unsupported clip-path function `{name}`"),
        ));
    }
    let (authored, current) = input.parse_nested_block(|input| {
        let state = input.state();
        let authored = collect_authored_tokens(input)?;
        input.reset(&state);
        if normalized_name == "circle"
            && input.try_parse(parse_frozen_circle_percentage).is_ok()
            && input.is_exhausted()
        {
            return Ok((authored, None));
        }
        input.reset(&state);
        let current = match normalized_name.as_str() {
            "inset" => parse_inset_shape(input)
                .map(Box::new)
                .map(CssBasicShapeValue::Inset),
            "circle" => parse_circle_shape(input).map(CssBasicShapeValue::Circle),
            "ellipse" => parse_ellipse_shape(input).map(CssBasicShapeValue::Ellipse),
            "polygon" => parse_polygon_shape(input).map(CssBasicShapeValue::Polygon),
            _ => Err(unsupported_value(
                input,
                None,
                "unsupported basic-shape function",
            )),
        }?;
        if !input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "basic-shape function has trailing arguments",
            ));
        }
        Ok((authored, Some(CssClipPathValue::BasicShape(current))))
    })?;
    let legacy = legacy_basic_shape(&normalized_name, &authored).map(CssClipPath::BasicShape);
    Ok(CssParsedClipPath::new(current, legacy))
}

fn parse_frozen_circle_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = match input.next().map_err(basic)? {
        Token::Percentage { unit_value, .. } => checked_percentage_value(
            location,
            *unit_value,
            "circle percentage radius must be finite",
        )?,
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    if value != 50.0 {
        return Err(unsupported_value_at(
            location,
            None,
            "circle percentage radius is not in the frozen compatibility case",
        ));
    }
    input.expect_ident_matching("at")?;
    input.expect_ident_matching("center")?;
    Ok(())
}

fn legacy_basic_shape(name: &str, authored: &str) -> Option<CssBasicShape> {
    if authored.is_empty() {
        return None;
    }
    let valid = validate_authored_function_arguments(authored, |input| match name {
        "circle" => validate_legacy_circle_shape(input),
        "ellipse" => validate_legacy_ellipse_shape(input),
        "inset" => validate_legacy_inset_shape(input),
        "polygon" => validate_legacy_polygon_shape(input),
        _ => false,
    });
    if !valid {
        return None;
    }
    let arguments = CssBasicShapeArguments::new(CssAuthoredFunctionArguments::new(authored));
    match name {
        "inset" => Some(CssBasicShape::Inset(arguments)),
        "circle" => Some(CssBasicShape::Circle(arguments)),
        "ellipse" => Some(CssBasicShape::Ellipse(arguments)),
        "polygon" => Some(CssBasicShape::Polygon(arguments)),
        _ => None,
    }
}

fn validate_legacy_shape_radius<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    input.try_parse(parse_radial_extent).is_ok()
        || parse_length_with_context_legacy(input, LengthGrammar::BackgroundSize, "shape radius")
            .is_ok()
}

fn validate_legacy_circle_shape<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    if input
        .try_parse(|input| input.expect_ident_matching("at"))
        .is_ok()
    {
        return parse_css_position_legacy(input).is_ok() && input.is_exhausted();
    }
    validate_legacy_shape_radius(input)
        && (input.is_exhausted()
            || input
                .try_parse(|input| input.expect_ident_matching("at"))
                .is_ok()
                && parse_css_position_legacy(input).is_ok()
                && input.is_exhausted())
}

fn validate_legacy_ellipse_shape<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    if input
        .try_parse(|input| input.expect_ident_matching("at"))
        .is_ok()
    {
        return parse_css_position_legacy(input).is_ok() && input.is_exhausted();
    }
    if !validate_legacy_shape_radius(input) {
        return false;
    }
    if !input.is_exhausted() && !next_is_ident(input, "at") && !validate_legacy_shape_radius(input)
    {
        return false;
    }
    input.is_exhausted()
        || input
            .try_parse(|input| input.expect_ident_matching("at"))
            .is_ok()
            && parse_css_position_legacy(input).is_ok()
            && input.is_exhausted()
}

fn validate_legacy_inset_shape<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let mut count = 0;
    while !input.is_exhausted() && !next_is_ident(input, "round") {
        if count == 4
            || parse_length_with_context_legacy(input, LengthGrammar::BackgroundSize, "inset shape")
                .is_err()
        {
            return false;
        }
        count += 1;
    }
    count > 0
        && (input.is_exhausted()
            || input
                .try_parse(|input| input.expect_ident_matching("round"))
                .is_ok()
                && validate_legacy_length_sequence(input, 1, 4))
}

fn validate_legacy_length_sequence<'i, 't>(
    input: &mut Parser<'i, 't>,
    min: usize,
    max: usize,
) -> bool {
    let mut count = 0;
    while !input.is_exhausted() {
        if count == max
            || parse_length_with_context_legacy(input, LengthGrammar::Position, "function length")
                .is_err()
        {
            return false;
        }
        count += 1;
        if !input.is_exhausted() {
            let _ = input.try_parse(Parser::expect_comma);
        }
    }
    count >= min
}

fn validate_legacy_polygon_shape<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let mut points = 0;
    loop {
        if parse_length_with_context_legacy(input, LengthGrammar::Position, "polygon x").is_err()
            || parse_length_with_context_legacy(input, LengthGrammar::Position, "polygon y")
                .is_err()
        {
            return false;
        }
        points += 1;
        if input.is_exhausted() {
            return points >= 1;
        }
        if input.try_parse(Parser::expect_comma).is_err() || input.is_exhausted() {
            return false;
        }
    }
}

pub(super) fn parse_mask_list<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMaskList, ParseError<'i, Error>> {
    let mut layers = Vec::new();
    loop {
        layers.push(parse_mask_layer(input)?);
        if input.try_parse(Parser::expect_comma).is_err() {
            break;
        }
        if input.is_exhausted() {
            return Err(unsupported_value(
                input,
                None,
                "mask list has an empty item",
            ));
        }
    }
    if layers.is_empty() {
        Err(unsupported_value(input, None, "mask list is empty"))
    } else {
        Ok(CssMaskList::new(layers))
    }
}

pub(super) fn parse_mask_layer<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssMaskLayer, ParseError<'i, Error>> {
    let mut image = None;
    let mut position = None;
    let mut size = None;
    let mut repeat = None;

    while !input.is_exhausted() && !next_is_comma(input) {
        if image.is_none()
            && let Ok(parsed_image) = input.try_parse(parse_image_layer)
        {
            image = Some(parsed_image);
            continue;
        }
        if repeat.is_none()
            && let Ok(parsed_repeat) = input.try_parse(parse_background_repeat)
        {
            repeat = Some(parsed_repeat);
            continue;
        }
        if position.is_none()
            && let Ok(parsed_position) = input.try_parse(parse_css_position)
        {
            position = Some(parsed_position);
            if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                size = Some(parse_background_size(input)?);
            }
            continue;
        }
        return Err(unsupported_value(input, None, "unsupported mask component"));
    }
    CssMaskLayer::try_new(image, position, size, repeat)
        .ok_or_else(|| unsupported_value(input, None, "mask layer is empty"))
}

#[cfg(test)]
mod transform_tests {
    use cssparser::{Parser, ParserInput};

    use super::*;

    fn parse(value: &str) -> Result<CssParsedTransform, ParseError<'_, Error>> {
        let mut input = ParserInput::new(value);
        Parser::new(&mut input).parse_entirely(parse_transform)
    }

    #[test]
    fn transform_parser_requires_commas_at_the_function_boundary() {
        assert!(parse("matrix(1 0 0 1 0 0)").is_err());
        assert!(parse("rotate3d(1 0 0 45deg)").is_err());
        assert!(parse("translate3d(1px 2px 3px)").is_err());
    }

    #[test]
    fn transform_parser_builds_checked_percentage_scale_and_length_only_z_values() {
        let parsed =
            parse("scale3d(1, 50%, 2) translate3d(10%, 20%, 3px)").expect("valid typed transforms");
        let (current, legacy) = parsed.into_parts();
        let CssTransformValue::Functions(functions) = current else {
            panic!("expected current transform functions");
        };
        assert!(matches!(
            &functions.functions()[0],
            CssTransformFunctionValue::Scale3d(scale)
                if matches!(scale.y(), CssTransformScaleComponent::Percentage(_))
        ));
        assert!(matches!(
            &functions.functions()[1],
            CssTransformFunctionValue::Translate3d(translation)
                if matches!(translation.z().value(), CssLength::Px(value) if value.value() == 3.0)
        ));
        let CssTransform::Functions(legacy) = legacy else {
            panic!("expected legacy transform projection");
        };
        assert_eq!(legacy.functions()[0].arguments().as_css(), "1, 50%, 2");
    }
}
