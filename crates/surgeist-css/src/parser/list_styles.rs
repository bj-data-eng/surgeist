//! Lists 3 longhands and shorthand, sharing the Content 3 style grammar.

use super::combinators::{Combinator, Tail, commit, fill, keyword, parse_group};
use super::property_input::Parser;
use super::values::next_is_ident;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::match_ignore_ascii_case;

use super::background::parse_image_value;
use super::content_values::parse_style;
use super::generated_content::parse_content_string;
use crate::{
    CssImageValue, CssListStylePosition, CssListStyleTypeValue, CssListStyleValue, CssMarkerSide,
};

pub(super) fn parse_marker_side<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssMarkerSide, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "match-self" => Ok(CssMarkerSide::MatchSelf),
        "match-parent" => Ok(CssMarkerSide::MatchParent),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

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
        _ => Err(unexpected_at(input.current_source_location())),
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
    // `<'list-style-position'> || <'list-style-image'> || <'list-style-type'>`.
    // Image and type both admit `none`, so up to two `none` operands are
    // collected as ambiguous; any other type candidate commits the type.
    let (mut style_type, mut position, mut image) = (None, None, None);
    let mut nones = [None; 2];
    let matched = parse_group::<5>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 | 1 => keyword(&mut nones[operand], input, &[("none", ())]),
            2 => fill(&mut position, input, parse_list_style_position),
            _ if next_is_ident(input, "none") => Ok(false),
            3 => fill(&mut image, input, |input| {
                parse_list_style_image(input, numeric)
            }),
            4 => commit(
                &mut style_type,
                input,
                |_| true,
                |input| parse_list_style_type(input, numeric),
            ),
            _ => unreachable!("five list-style operands"),
        },
    )?;
    let none_count = usize::from(matched[0]) + usize::from(matched[1]);

    let free = usize::from(style_type.is_none()) + usize::from(image.is_none());
    if none_count > free {
        return Err(unexpected_at(input.current_source_location()));
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
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}
