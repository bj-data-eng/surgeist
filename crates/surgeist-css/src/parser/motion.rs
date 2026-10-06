//! Motion Path authored grammar composed over the shared numeric, position,
//! URL and complete BasicShape owners.
use super::values::{
    AngleParserContext, next_is_delim, next_is_ident, parse_angle_value, parse_length_percentage,
};
use crate::error::{Error, basic, unsupported_value};
use crate::numeric::NumericInputContext;
use crate::*;
use cssparser::{ParseError, Parser, Token};

fn next_keyword<'i, 't>(input: &mut Parser<'i, 't>) -> Option<String> {
    let state = input.state();
    let keyword = match input.next() {
        Ok(Token::Ident(value)) => Some(value.to_ascii_lowercase()),
        _ => None,
    };
    input.reset(&state);
    keyword
}
fn next_path_kind<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let result = match input.next() {
        Ok(Token::UnquotedUrl(_)) => true,
        Ok(Token::Function(name)) => {
            name.eq_ignore_ascii_case("ray")
                || name.eq_ignore_ascii_case("url")
                || name.eq_ignore_ascii_case("src")
                || super::effects::is_basic_shape_function(name)
        }
        _ => false,
    };
    input.reset(&state);
    result
}
fn next_coord_box<'i, 't>(input: &mut Parser<'i, 't>) -> Option<CssCoordBox> {
    next_keyword(input)
        .as_deref()
        .and_then(CssCoordBox::from_keyword)
}
fn next_path<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    next_is_ident(input, "none") || next_coord_box(input).is_some() || next_path_kind(input)
}

pub(super) fn parse_offset_path<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOffsetPath, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssOffsetPath::none());
    }
    let mut coord_box = next_coord_box(input);
    if coord_box.is_some() {
        input.next().map_err(basic)?;
    }
    if let Some(coord_box) = coord_box
        && !next_path_kind(input)
    {
        return Ok(CssOffsetPath::from_coord_box(coord_box));
    }
    let state = input.state();
    let (ray, url) = match input.next() {
        Ok(Token::Function(name)) => (
            name.eq_ignore_ascii_case("ray"),
            name.eq_ignore_ascii_case("url") || name.eq_ignore_ascii_case("src"),
        ),
        Ok(Token::UnquotedUrl(_)) => (false, true),
        _ => (false, false),
    };
    input.reset(&state);
    let path = if ray {
        input.expect_function_matching("ray").map_err(basic)?;
        CssOffsetPathKind::Ray(input.parse_nested_block(|input| parse_ray(input, numeric))?)
    } else if url {
        CssOffsetPathKind::Url(super::url::parse_url(input, numeric)?)
    } else {
        CssOffsetPathKind::BasicShape(super::effects::parse_clip_path_shape(input, numeric)?)
    };
    if coord_box.is_none()
        && let Some(value) = next_coord_box(input)
    {
        input.next().map_err(basic)?;
        coord_box = Some(value);
    }
    Ok(CssOffsetPath::from_path(CssOffsetPathValue::new(
        path, coord_box,
    )))
}

fn parse_ray<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssRay, ParseError<'i, Error>> {
    let (mut angle, mut size, mut contain, mut position) = (None, None, false, None);
    while !input.is_exhausted() {
        if next_is_ident(input, "contain") && !contain {
            input.next().map_err(basic)?;
            contain = true;
        } else if next_is_ident(input, "at") && position.is_none() {
            input.next().map_err(basic)?;
            position = Some(super::position::parse_physical_position_before_angle(
                input, numeric,
            )?);
        } else if size.is_none()
            && let Some(value) = next_keyword(input)
                .as_deref()
                .and_then(CssRaySize::from_keyword)
        {
            input.next().map_err(basic)?;
            size = Some(value);
        } else if angle.is_none() {
            angle = Some(parse_angle_value(
                input,
                numeric,
                AngleParserContext::Motion,
            )?);
        } else {
            return Err(unsupported_value(
                input,
                None,
                "duplicate or unsupported ray argument",
            ));
        }
    }
    let angle = angle.ok_or_else(|| unsupported_value(input, None, "ray requires an angle"))?;
    Ok(CssRay::new(angle, size, contain, position))
}

pub(super) fn parse_offset_position<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOffsetPosition, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        return Ok(CssOffsetPosition::Normal);
    }
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssOffsetPosition::Auto);
    }
    super::position::parse_physical_position(input, numeric).map(CssOffsetPosition::Position)
}
pub(super) fn parse_offset_anchor<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOffsetAnchor, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("auto"))
        .is_ok()
    {
        return Ok(CssOffsetAnchor::Auto);
    }
    super::position::parse_physical_position(input, numeric).map(CssOffsetAnchor::Position)
}
fn take_modifier<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<Option<CssOffsetRotateModifier>, ParseError<'i, Error>> {
    let modifier = match next_keyword(input).as_deref() {
        Some("auto") => Some(CssOffsetRotateModifier::Auto),
        Some("reverse") => Some(CssOffsetRotateModifier::Reverse),
        _ => None,
    };
    if modifier.is_some() {
        input.next().map_err(basic)?;
    }
    Ok(modifier)
}
pub(super) fn parse_offset_rotate<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOffsetRotate, ParseError<'i, Error>> {
    let mut modifier = take_modifier(input)?;
    // This same owner parses one whole quoted rotation constituent in offset.
    // A modifier alone is complete, so a following distance or slash belongs
    // to the enclosing grammar rather than requiring an angle here.
    let angle = if modifier.is_some() {
        match input.try_parse(|input| parse_angle_value(input, numeric, AngleParserContext::Motion))
        {
            Ok(value) => Some(value),
            Err(error) if crate::error::is_resource_parse_error(&error) => {
                return Err(error);
            }
            Err(_) => None,
        }
    } else {
        Some(parse_angle_value(
            input,
            numeric,
            AngleParserContext::Motion,
        )?)
    };
    if modifier.is_none() {
        modifier = take_modifier(input)?;
    }
    CssOffsetRotate::try_new(modifier, angle)
        .map_err(|_| unsupported_value(input, None, "offset-rotate is empty"))
}

pub(super) fn parse_offset<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOffset, ParseError<'i, Error>> {
    let position = if next_is_ident(input, "normal") {
        input.next().map_err(basic)?;
        Some(CssOffsetPosition::Normal)
    } else if next_is_ident(input, "auto") {
        input.next().map_err(basic)?;
        Some(CssOffsetPosition::Auto)
    } else if !next_path(input) && !next_is_delim(input, '/') {
        Some(CssOffsetPosition::Position(
            super::position::parse_physical_position_before_angle(input, numeric)?,
        ))
    } else {
        None
    };
    let path = if next_path(input) {
        Some(parse_offset_path(input, numeric)?)
    } else {
        None
    };
    let (mut distance, mut rotate) = (None, None);
    if path.is_some() {
        while !input.is_exhausted() && !next_is_delim(input, '/') {
            // Values 3 combinators are not associative: distance can precede
            // or follow the whole offset-rotate, but cannot split its children.
            if rotate.is_none() {
                match input.try_parse(|input| parse_offset_rotate(input, numeric)) {
                    Ok(value) => {
                        rotate = Some(value);
                        continue;
                    }
                    Err(error) if crate::error::is_resource_parse_error(&error) => {
                        return Err(error);
                    }
                    Err(_) => {}
                }
            }
            if distance.is_none() {
                distance = Some(parse_length_percentage(input, numeric, "offset-distance")?);
            } else {
                return Err(unsupported_value(
                    input,
                    None,
                    "duplicate or unsupported offset constituent",
                ));
            }
        }
    }
    let anchor = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_offset_anchor(input, numeric)?)
    } else {
        None
    };
    CssOffset::try_new(position, path, distance, rotate, anchor)
        .map_err(|_| unsupported_value(input, None, "offset requires a position or path"))
}
