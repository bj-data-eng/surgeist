//! Lists 3 longhands and shorthand, sharing the Content 3 style grammar.

use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use super::background::parse_image_value;
use super::content_values::parse_style;
use super::generated_content::parse_content_string;
use crate::error::{Error, basic, unsupported_value};
use crate::{CssImageValue, CssListStylePosition, CssListStyleTypeValue, CssListStyleValue};

pub(super) fn parse_list_style_type<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssListStyleTypeValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssListStyleTypeValue::None);
    }
    if let Ok(value) = input.try_parse(parse_content_string) {
        return Ok(CssListStyleTypeValue::String(value));
    }
    parse_style(input, numeric).map(CssListStyleTypeValue::CounterStyle)
}

pub(super) fn parse_list_style_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssListStylePosition, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "inside" => Ok(CssListStylePosition::Inside),
        "outside" => Ok(CssListStylePosition::Outside),
        _ => Err(unsupported_value(input, None, "expected inside or outside")),
    }
}

pub(super) fn parse_list_style_image<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssImageValue, ParseError<'i, Error>> {
    parse_image_value(input, numeric)
}

pub(super) fn parse_list_style<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<CssListStyleValue, ParseError<'i, Error>> {
    let mut style_type = None;
    let mut position = None;
    let mut image = None;
    let mut none_count = 0usize;

    while !input.is_exhausted() {
        if input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            none_count += 1;
            continue;
        }
        if position.is_none()
            && let Ok(value) = input.try_parse(parse_list_style_position)
        {
            position = Some(value);
            continue;
        }
        if image.is_none()
            && let Ok(value) = input.try_parse(|input| parse_list_style_image(input, numeric))
        {
            image = Some(value);
            continue;
        }
        if style_type.is_none() {
            style_type = Some(parse_list_style_type(input, numeric)?);
            continue;
        }
        return Err(unsupported_value(
            input,
            None,
            "list-style has more than one component of a kind",
        ));
    }

    let free = usize::from(style_type.is_none()) + usize::from(image.is_none());
    if none_count > free {
        return Err(unsupported_value(
            input,
            None,
            "list-style has too many none components",
        ));
    }
    if none_count != 0 {
        if style_type.is_none() {
            style_type = Some(CssListStyleTypeValue::None);
        }
        if image.is_none() {
            image = Some(CssImageValue::None);
        }
    }
    CssListStyleValue::try_new(style_type, position, image)
        .ok_or_else(|| unsupported_value(input, None, "list-style shorthand is empty"))
}
