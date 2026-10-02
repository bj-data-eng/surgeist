use super::values::{
    parse_length, parse_length_percentage, parse_nonnegative_length,
    parse_nonnegative_length_percentage,
};
use cssparser::{ParseError, Parser, ToCss, Token, match_ignore_ascii_case};

use super::background::{parse_background_repeat, parse_background_size, parse_image_value};
use super::box_model::parse_drop_shadow;
use super::position::{parse_full_position, parse_physical_position};
use super::url::parse_url;
use super::values::{
    AngleParserContext, next_is_comma, next_is_delim, next_is_ident, parse_angle_or_zero,
    parse_nonnegative_number, parse_nonnegative_percentage, parse_specified_number,
    parse_specified_number_literal, parse_specified_percentage,
};
use crate::error::{CssFeatureId, Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) fn parse_clip<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
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
    input
        .parse_nested_block(|input| parse_clip_rect(input, numeric))
        .map(CssClip::Rect)
}

pub(super) fn parse_outline_offset<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedLength, ParseError<'i, Error>> {
    parse_length(input, numeric, "outline-offset")
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssClipRect, ParseError<'i, Error>> {
    let top = parse_clip_edge(input, numeric)?;
    let comma_separated = input.try_parse(Parser::expect_comma).is_ok();
    let right = parse_clip_edge(input, numeric)?;
    if comma_separated {
        input.expect_comma().map_err(basic)?;
    }
    let bottom = parse_clip_edge(input, numeric)?;
    if comma_separated {
        input.expect_comma().map_err(basic)?;
    }
    let left = parse_clip_edge(input, numeric)?;
    input.expect_exhausted().map_err(basic)?;
    Ok(CssClipRect::new(top, right, bottom, left))
}

fn parse_clip_edge<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssClipEdge, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssClipEdge::Auto);
    }
    let value = parse_length(input, numeric, "clip")?;
    Ok(CssClipEdge::Length(value))
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
    CssFeatureId::new("ext.value.basic-shape.shape"),
];

pub(super) fn parse_transform<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTransform, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssTransform::None);
    }
    let mut functions = Vec::new();
    while !input.is_exhausted() {
        functions.push(parse_transform_function(input, numeric)?);
    }
    CssTransformFunctionList::try_new(functions)
        .map(CssTransform::Functions)
        .ok_or_else(|| unsupported_value(input, None, "transform function list is empty"))
}

fn parse_transform_function<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTransformFunction, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    let kind = parse_transform_function_kind(input, name.as_ref())?;
    input.parse_nested_block(|input| parse_transform_function_value(input, numeric, kind))
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
    numeric: &crate::numeric::NumericInputContext<'_>,
    kind: CssTransformFunctionKind,
) -> std::result::Result<CssTransformFunction, ParseError<'i, Error>> {
    let value = match kind {
        CssTransformFunctionKind::Matrix => {
            let components = parse_exact_comma_list(input, 6, |input| {
                parse_specified_number(input, numeric, "transform")
            })?;
            let components = components.try_into().map_err(|_| {
                unsupported_value(input, None, "matrix() requires exactly six numbers")
            })?;
            CssTransformFunction::Matrix(CssTransformMatrix::new(components))
        }
        CssTransformFunctionKind::Matrix3d => {
            let components = parse_exact_comma_list(input, 16, |input| {
                parse_specified_number(input, numeric, "transform")
            })?;
            let components = components.try_into().map_err(|_| {
                unsupported_value(input, None, "matrix3d() requires exactly sixteen numbers")
            })?;
            CssTransformFunction::Matrix3d(Box::new(CssTransformMatrix3d::new(components)))
        }
        CssTransformFunctionKind::Perspective => {
            let perspective = if input
                .try_parse(|input| input.expect_ident_matching("none"))
                .is_ok()
            {
                CssTransformPerspective::None
            } else {
                let length = parse_nonnegative_length(input, numeric, "perspective")?;
                CssTransformPerspective::Length(length)
            };
            input.expect_exhausted().map_err(basic)?;
            CssTransformFunction::Perspective(perspective)
        }
        CssTransformFunctionKind::Rotate => {
            CssTransformFunction::Rotate(parse_one(input, |input| {
                parse_angle_or_zero(input, numeric, AngleParserContext::Transform)
            })?)
        }
        CssTransformFunctionKind::Rotate3d => {
            let x = parse_specified_number(input, numeric, "transform")?;
            input.expect_comma().map_err(basic)?;
            let y = parse_specified_number(input, numeric, "transform")?;
            input.expect_comma().map_err(basic)?;
            let z = parse_specified_number(input, numeric, "transform")?;
            input.expect_comma().map_err(basic)?;
            let angle = parse_angle_or_zero(input, numeric, AngleParserContext::Transform)?;
            input.expect_exhausted().map_err(basic)?;
            CssTransformFunction::Rotate3d(CssTransformRotate3d::new(x, y, z, angle))
        }
        CssTransformFunctionKind::RotateX => {
            CssTransformFunction::RotateX(parse_one(input, |input| {
                parse_angle_or_zero(input, numeric, AngleParserContext::Transform)
            })?)
        }
        CssTransformFunctionKind::RotateY => {
            CssTransformFunction::RotateY(parse_one(input, |input| {
                parse_angle_or_zero(input, numeric, AngleParserContext::Transform)
            })?)
        }
        CssTransformFunctionKind::RotateZ => {
            CssTransformFunction::RotateZ(parse_one(input, |input| {
                parse_angle_or_zero(input, numeric, AngleParserContext::Transform)
            })?)
        }
        CssTransformFunctionKind::Scale => {
            let (x, y) = parse_one_or_two(input, |input| {
                parse_specified_number(input, numeric, "transform")
            })?;
            CssTransformFunction::Scale(CssTransformScale::new(x, y))
        }
        CssTransformFunctionKind::Scale3d => {
            let mut components = parse_exact_comma_list(input, 3, |input| {
                parse_transform_scale_component(input, numeric)
            })?;
            let z = components.pop().ok_or_else(|| {
                unsupported_value(input, None, "scale3d() requires exactly three operands")
            })?;
            let y = components.pop().ok_or_else(|| {
                unsupported_value(input, None, "scale3d() requires exactly three operands")
            })?;
            let x = components.pop().ok_or_else(|| {
                unsupported_value(input, None, "scale3d() requires exactly three operands")
            })?;
            CssTransformFunction::Scale3d(CssTransformScale3d::new(x, y, z))
        }
        CssTransformFunctionKind::ScaleX => {
            CssTransformFunction::ScaleX(parse_one(input, |input| {
                parse_specified_number(input, numeric, "transform")
            })?)
        }
        CssTransformFunctionKind::ScaleY => {
            CssTransformFunction::ScaleY(parse_one(input, |input| {
                parse_specified_number(input, numeric, "transform")
            })?)
        }
        CssTransformFunctionKind::ScaleZ => {
            CssTransformFunction::ScaleZ(parse_one(input, |input| {
                parse_transform_scale_component(input, numeric)
            })?)
        }
        CssTransformFunctionKind::Skew => {
            let (x, y) = parse_one_or_two(input, |input| {
                parse_angle_or_zero(input, numeric, AngleParserContext::Transform)
            })?;
            CssTransformFunction::Skew(CssTransformSkew::new(x, y))
        }
        CssTransformFunctionKind::SkewX => {
            CssTransformFunction::SkewX(parse_one(input, |input| {
                parse_angle_or_zero(input, numeric, AngleParserContext::Transform)
            })?)
        }
        CssTransformFunctionKind::SkewY => {
            CssTransformFunction::SkewY(parse_one(input, |input| {
                parse_angle_or_zero(input, numeric, AngleParserContext::Transform)
            })?)
        }
        CssTransformFunctionKind::Translate => {
            let (x, y) = parse_one_or_two(input, |input| {
                parse_transform_length_percentage(input, numeric)
            })?;
            CssTransformFunction::Translate(CssTransformTranslate::new(x, y))
        }
        CssTransformFunctionKind::Translate3d => {
            let x = parse_transform_length_percentage(input, numeric)?;
            input.expect_comma().map_err(basic)?;
            let y = parse_transform_length_percentage(input, numeric)?;
            input.expect_comma().map_err(basic)?;
            let z = parse_transform_length(input, numeric)?;
            input.expect_exhausted().map_err(basic)?;
            CssTransformFunction::Translate3d(CssTransformTranslate3d::new(x, y, z))
        }
        CssTransformFunctionKind::TranslateX => {
            CssTransformFunction::TranslateX(parse_one(input, |input| {
                parse_transform_length_percentage(input, numeric)
            })?)
        }
        CssTransformFunctionKind::TranslateY => {
            CssTransformFunction::TranslateY(parse_one(input, |input| {
                parse_transform_length_percentage(input, numeric)
            })?)
        }
        CssTransformFunctionKind::TranslateZ => {
            CssTransformFunction::TranslateZ(parse_one(input, |input| {
                parse_transform_length(input, numeric)
            })?)
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

fn parse_transform_scale_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssTransformScaleComponent, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(number) = input.try_parse(|input| parse_specified_number(input, numeric, "transform"))
    {
        return Ok(CssTransformScaleComponent::Number(number));
    }
    input.reset(&state);
    parse_specified_percentage(input, numeric, "transform")
        .map(CssTransformScaleComponent::Percentage)
}

fn parse_transform_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedLengthPercentage, ParseError<'i, Error>> {
    parse_length_percentage(input, numeric, "transform translation")
}

fn parse_transform_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssSpecifiedLength, ParseError<'i, Error>> {
    parse_length(input, numeric, "transform translation")
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssCircleShape, ParseError<'i, Error>> {
    let radius = if input.is_exhausted() || next_is_ident(input, "at") {
        CssCircleRadius::Default
    } else if let Ok(extent) = input.try_parse(parse_radial_extent) {
        CssCircleRadius::Extent(extent)
    } else {
        CssCircleRadius::LengthPercentage(parse_non_negative_shape_length_percentage(
            input,
            numeric,
            "circle radius",
        )?)
    };
    Ok(CssCircleShape::new(
        radius,
        parse_optional_shape_position(input, numeric)?,
    ))
}

fn parse_ellipse_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssEllipseShape, ParseError<'i, Error>> {
    let radii = if input.is_exhausted() || next_is_ident(input, "at") {
        None
    } else {
        let horizontal = parse_ellipse_radius(input, numeric)?;
        let vertical = parse_ellipse_radius(input, numeric)?;
        Some(CssEllipseRadii::new(horizontal, vertical))
    };
    Ok(CssEllipseShape::new(
        radii,
        parse_optional_shape_position(input, numeric)?,
    ))
}

fn parse_ellipse_radius<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssEllipseRadius, ParseError<'i, Error>> {
    if let Ok(extent) = input.try_parse(parse_radial_extent) {
        Ok(CssEllipseRadius::Extent(extent))
    } else {
        parse_non_negative_shape_length_percentage(input, numeric, "ellipse radius")
            .map(CssEllipseRadius::LengthPercentage)
    }
}

fn parse_optional_shape_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<Option<CssPosition>, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(None);
    }
    input.expect_ident_matching("at")?;
    parse_full_position(input, numeric).map(Some)
}

fn parse_shape_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    context: &str,
) -> Result<CssSpecifiedLengthPercentage, ParseError<'i, Error>> {
    parse_length_percentage(input, numeric, context)
}

fn parse_non_negative_shape_length_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    context: &str,
) -> Result<CssSpecifiedNonNegativeLengthPercentage, ParseError<'i, Error>> {
    parse_nonnegative_length_percentage(input, numeric, context)
}

fn parse_inset_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
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
        offsets.push(parse_shape_length_percentage(
            input,
            numeric,
            "inset shape offset",
        )?);
    }
    let offsets = CssInsetShapeOffsets::try_new(offsets)
        .ok_or_else(|| unsupported_value(input, None, "inset shape is missing an offset"))?;
    let round = if input.is_exhausted() {
        None
    } else {
        input.expect_ident_matching("round")?;
        Some(parse_shape_radii(input, numeric)?)
    };
    Ok(CssInsetShape::new(offsets, round))
}

fn parse_rect_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssRectShape, ParseError<'i, Error>> {
    let top = parse_rect_shape_edge(input, numeric)?;
    let right = parse_rect_shape_edge(input, numeric)?;
    let bottom = parse_rect_shape_edge(input, numeric)?;
    let left = parse_rect_shape_edge(input, numeric)?;
    Ok(CssRectShape::new(
        top,
        right,
        bottom,
        left,
        parse_optional_rectangle_round(input, numeric)?,
    ))
}

fn parse_rect_shape_edge<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssRectShapeEdge, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        Ok(CssRectShapeEdge::Auto)
    } else {
        parse_shape_length_percentage(input, numeric, "rect edge")
            .map(CssRectShapeEdge::LengthPercentage)
    }
}

fn parse_xywh_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssXywhShape, ParseError<'i, Error>> {
    let x = parse_shape_length_percentage(input, numeric, "xywh x")?;
    let y = parse_shape_length_percentage(input, numeric, "xywh y")?;
    let width = parse_non_negative_shape_length_percentage(input, numeric, "xywh width")?;
    let height = parse_non_negative_shape_length_percentage(input, numeric, "xywh height")?;
    Ok(CssXywhShape::new(
        x,
        y,
        width,
        height,
        parse_optional_rectangle_round(input, numeric)?,
    ))
}

fn parse_optional_rectangle_round<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<Option<crate::CssBorderRadiusShorthand>, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(None);
    }
    input.expect_ident_matching("round")?;
    parse_shape_radii(input, numeric).map(Some)
}

fn parse_shape_radii<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<crate::CssBorderRadiusShorthand, ParseError<'i, Error>> {
    let horizontal = parse_shape_radius_list(input, numeric)?;
    let vertical = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_shape_radius_list(input, numeric)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    crate::CssBorderRadiusShorthand::try_new(horizontal, vertical)
        .ok_or_else(|| unsupported_value(input, None, "invalid inset round radii"))
}

fn parse_shape_radius_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<Vec<CssSpecifiedNonNegativeLengthPercentage>, ParseError<'i, Error>> {
    let mut values = Vec::new();
    while !input.is_exhausted() && !next_is_delim(input, '/') {
        if values.len() == 4 {
            return Err(unsupported_value(
                input,
                None,
                "inset round has too many radii",
            ));
        }
        values.push(parse_non_negative_shape_length_percentage(
            input,
            numeric,
            "inset round radius",
        )?);
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssPolygonShape, ParseError<'i, Error>> {
    let fill_rule = input.try_parse(parse_fill_rule).ok();
    let round = if next_is_ident(input, "round") {
        input.expect_ident_matching("round")?;
        Some(parse_length(input, numeric, "polygon round")?)
    } else {
        None
    };

    if fill_rule.is_some() || round.is_some() {
        input.expect_comma()?;
    }
    let mut points = Vec::new();
    loop {
        let x = parse_shape_length_percentage(input, numeric, "polygon x")?;
        let y = parse_shape_length_percentage(input, numeric, "polygon y")?;
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

fn parse_path_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssPathShape, ParseError<'i, Error>> {
    let fill_rule = input.try_parse(parse_fill_rule).ok();
    if fill_rule.is_some() {
        input.expect_comma().map_err(basic)?;
    }
    input.skip_whitespace();
    let location = input.current_source_location();
    let component = context
        .collect(input)
        .map_err(|_| unsupported_value_at(location, None, "path() requires one string"))?;
    let data = CssPathData::from_parser_component(component, context)
        .map_err(|error| unsupported_value_at(location, None, error.to_string()))?;
    Ok(CssPathShape::new(fill_rule, data))
}

fn parse_fill_rule<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssFillRule, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "nonzero" => Ok(CssFillRule::Nonzero),
        "evenodd" => Ok(CssFillRule::Evenodd),
        _ => Err(unsupported_value(
            input,
            None,
            unsupported_keyword_reason("polygon fill rule", ident.as_ref()),
        )),
    }
}

pub(super) fn parse_translate<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssTranslate, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssTranslate::None);
    }
    let x = parse_length_percentage(input, numeric, "translate x")?;
    let y = if input.is_exhausted() {
        None
    } else {
        Some(parse_length_percentage(input, numeric, "translate y")?)
    };
    let z = if input.is_exhausted() {
        None
    } else {
        Some(parse_length(input, numeric, "translate z")?)
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssTranslate::Values(
        CssTranslateValues::try_new(x, y, z).expect("parser retained consecutive translate axes"),
    ))
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssScale, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssScale::None);
    }
    let mut values = Vec::new();
    while !input.is_exhausted() {
        values.push(parse_specified_number_literal(input, numeric, "scale")?);
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFilter, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssFilter::None);
    }
    let mut functions = Vec::new();
    while !input.is_exhausted() {
        functions.push(parse_filter_function(input, numeric)?);
    }
    CssFilterFunctionList::try_new(functions)
        .map(CssFilter::Functions)
        .ok_or_else(|| unsupported_value(input, None, "filter function list is empty"))
}

pub(super) fn parse_filter_function<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFilterFunction, ParseError<'i, Error>> {
    if let Ok(url) = input.try_parse(|input| parse_url(input, numeric)) {
        return Ok(CssFilterFunction::Url(url));
    }
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    input.parse_nested_block(|input| parse_filter_function_value(input, numeric, name.as_ref()))
}

fn parse_filter_function_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
    name: &str,
) -> std::result::Result<CssFilterFunction, ParseError<'i, Error>> {
    match name.to_ascii_lowercase().as_str() {
        "blur" => {
            let blur = if input.is_exhausted() {
                CssFilterBlur::omitted()
            } else {
                CssFilterBlur::new(parse_nonnegative_length(input, numeric, "filter blur")?)
            };
            input.expect_exhausted().map_err(basic)?;
            Ok(CssFilterFunction::Blur(blur))
        }
        "brightness" => parse_filter_amount(input, numeric).map(CssFilterFunction::Brightness),
        "contrast" => parse_filter_amount(input, numeric).map(CssFilterFunction::Contrast),
        "drop-shadow" => {
            let shadow = parse_drop_shadow(input, numeric)?;
            input.expect_exhausted().map_err(basic)?;
            Ok(CssFilterFunction::DropShadow(shadow))
        }
        "grayscale" => parse_filter_amount(input, numeric).map(CssFilterFunction::Grayscale),
        "hue-rotate" => {
            let angle = if input.is_exhausted() {
                CssFilterHueRotate::omitted()
            } else {
                CssFilterHueRotate::new(parse_angle_or_zero(
                    input,
                    numeric,
                    AngleParserContext::Filter,
                )?)
            };
            input.expect_exhausted().map_err(basic)?;
            Ok(CssFilterFunction::HueRotate(angle))
        }
        "invert" => parse_filter_amount(input, numeric).map(CssFilterFunction::Invert),
        "opacity" => parse_filter_amount(input, numeric).map(CssFilterFunction::Opacity),
        "saturate" => parse_filter_amount(input, numeric).map(CssFilterFunction::Saturate),
        "sepia" => parse_filter_amount(input, numeric).map(CssFilterFunction::Sepia),
        _ => Err(unsupported_value(
            input,
            None,
            format!("unsupported filter function `{name}`"),
        )),
    }
}

fn parse_filter_amount<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssFilterAmount, ParseError<'i, Error>> {
    if input.is_exhausted() {
        return Ok(CssFilterAmount::Default);
    }
    let amount = if let Ok(number) =
        input.try_parse(|input| parse_nonnegative_number(input, numeric, "filter amount"))
    {
        CssFilterAmount::Number(number)
    } else {
        CssFilterAmount::Percentage(parse_nonnegative_percentage(
            input,
            numeric,
            "filter amount",
        )?)
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(amount)
}

pub(super) fn parse_clip_path<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssClipPath, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssClipPath::None);
    }
    if let Ok(url) = input.try_parse(|input| parse_url(input, numeric)) {
        return Ok(CssClipPath::Url(url));
    }
    let mut reference_box = input.try_parse(parse_geometry_box).ok();
    if input.is_exhausted() {
        return reference_box
            .map(CssClipPath::GeometryBox)
            .ok_or_else(|| unsupported_value(input, None, "missing clip-path shape or box"));
    }
    let shape = parse_clip_path_shape(input, numeric)?;
    if reference_box.is_none() && !input.is_exhausted() {
        reference_box = Some(parse_geometry_box(input)?);
    }
    input.expect_exhausted().map_err(basic)?;
    Ok(CssClipPath::BasicShape(CssClipPathShape::new(
        shape,
        reference_box,
    )))
}

fn parse_geometry_box<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssBoxEdgeKeyword, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    CssBoxEdgeKeyword::from_keyword(ident.as_ref()).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            unsupported_keyword_reason("clip-path geometry box", ident.as_ref()),
        )
    })
}

fn parse_clip_path_shape<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssBasicShape, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = match input.next().map_err(basic)? {
        Token::Function(name) => name.clone(),
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    };
    let normalized_name = name.to_ascii_lowercase();
    if !matches!(
        normalized_name.as_str(),
        "inset" | "circle" | "ellipse" | "polygon" | "rect" | "xywh" | "path" | "shape"
    ) {
        return Err(unsupported_value(
            input,
            None,
            format!("unsupported clip-path function `{name}`"),
        ));
    }
    input.parse_nested_block(|input| {
        let shape = match normalized_name.as_str() {
            "shape" => super::shapes::parse_shape(input, numeric).map(CssBasicShape::Shape),
            "inset" => parse_inset_shape(input, numeric)
                .map(Box::new)
                .map(CssBasicShape::Inset),
            "circle" => parse_circle_shape(input, numeric).map(CssBasicShape::Circle),
            "ellipse" => parse_ellipse_shape(input, numeric).map(CssBasicShape::Ellipse),
            "polygon" => parse_polygon_shape(input, numeric).map(CssBasicShape::Polygon),
            "path" => parse_path_shape(input, numeric).map(CssBasicShape::Path),
            "rect" => parse_rect_shape(input, numeric).map(CssBasicShape::Rect),
            "xywh" => parse_xywh_shape(input, numeric).map(CssBasicShape::Xywh),
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
        Ok(shape)
    })
}

pub(super) fn parse_mask_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssMaskList, ParseError<'i, Error>> {
    let mut layers = Vec::new();
    loop {
        layers.push(parse_mask_layer(input, numeric)?);
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
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<CssMaskLayer, ParseError<'i, Error>> {
    let mut image = None;
    let mut position = None;
    let mut size = None;
    let mut repeat = None;

    while !input.is_exhausted() && !next_is_comma(input) {
        if image.is_none() {
            match input.try_parse(|input| parse_image_value(input, numeric)) {
                Ok(value) => {
                    image = Some(value);
                    continue;
                }
                Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
                Err(_) => {}
            }
        }
        if repeat.is_none()
            && let Ok(parsed_repeat) = input.try_parse(parse_background_repeat)
        {
            repeat = Some(parsed_repeat);
            continue;
        }
        if position.is_none()
            && let Ok(parsed_position) =
                input.try_parse(|input| parse_physical_position(input, numeric))
        {
            position = Some(parsed_position);
            if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                size = Some(parse_background_size(input, numeric)?);
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

    fn parse(value: &str) -> Result<CssTransform, ParseError<'_, Error>> {
        let mut input = ParserInput::new(value);
        let snapshot = crate::CssSourceSnapshot::new(value);
        let numeric = crate::numeric::NumericInputContext::parsed(&snapshot);
        Parser::new(&mut input).parse_entirely(|input| parse_transform(input, &numeric))
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
        let CssTransform::Functions(functions) = parsed else {
            panic!("expected transform functions");
        };
        assert!(matches!(
            &functions.functions()[0],
            CssTransformFunction::Scale3d(scale)
                if matches!(scale.y(), CssTransformScaleComponent::Percentage(_))
        ));
        assert!(matches!(
            &functions.functions()[1],
            CssTransformFunction::Translate3d(translation)
                if matches!(translation.z().literal_component().unwrap().view(), crate::CssComponentValueRef::Token(crate::CssValueTokenRef::Dimension { number, unit }) if number.representation() == "3" && unit == "px")
        ));
        assert_eq!(
            functions.functions()[0].kind(),
            CssTransformFunctionKind::Scale3d
        );
    }
}
