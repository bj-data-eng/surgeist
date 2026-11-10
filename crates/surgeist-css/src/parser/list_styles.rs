//! Lists 3 longhands and shorthand, sharing the Content 3 style grammar.

use super::combinators::{Combinator, Tail, commit, fill, keyword, parse_group};
use super::syntax_types::Stream;
use super::values::next_is_ident;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::background::parse_image_value;
use super::content_values::parse_style;
use super::generated_content::parse_content_string;
use crate::values::content::{
    CssListStylePosition, CssListStyleTypeValue, CssListStyleValue, CssMarkerSide,
};
use crate::values::image::CssImageValue;

pub(super) fn parse_marker_side<'i>(
    input: &mut Stream<'i>,
) -> Result<CssMarkerSide, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "match-self" => Ok(CssMarkerSide::MatchSelf),
        "match-parent" => Ok(CssMarkerSide::MatchParent),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_list_style_type<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssListStyleTypeValue, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(CssListStyleTypeValue::None);
    }
    if let Ok(value) = input.attempt(parse_content_string) {
        return Ok(CssListStyleTypeValue::String(value));
    }
    parse_style(input, numeric).map(CssListStyleTypeValue::CounterStyle)
}

pub(super) fn parse_list_style_position<'i>(
    input: &mut Stream<'i>,
) -> Result<CssListStylePosition, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "inside" => Ok(CssListStylePosition::Inside),
        "outside" => Ok(CssListStylePosition::Outside),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_list_style_image<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssImageValue, SyntaxError<'i, Error>> {
    parse_image_value(input, numeric)
}

fn starts_image(input: &mut Stream<'_>) -> bool {
    let state = input.mark();
    let starts = match input.consume_after_whitespace() {
        Ok(Token::Url(_)) => true,
        Ok(Token::Function(name)) => !name.eq_ignore_ascii_case("symbols"),
        _ => false,
    };
    input.restore(state);
    starts
}

pub(super) fn parse_list_style<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<CssListStyleValue, SyntaxError<'i, Error>> {
    // `<'list-style-position'> || <'list-style-image'> || <'list-style-type'>`.
    // Image and type both admit `none`, so up to two `none` operands are
    // collected as ambiguous; any other type candidate commits the type.
    let (mut style_type, mut position, mut image) = (None, None, None);
    let mut nones = [None; 2];
    // The image or type operand that matched last: with both slots claimed
    // by `none`, it is the first component no remaining operand admits.
    let mut last_slot = None;
    let matched = parse_group::<5>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 | 1 => keyword(&mut nones[operand], input, &[("none", ())]),
            2 => fill(&mut position, input, parse_list_style_position),
            _ if next_is_ident(input, "none") => Ok(false),
            3 => {
                let location = input.site();
                // A URL or a function other than symbols() can only be the
                // image, which commits the operand (Values 4 §2.2).
                let matched = if commit(&mut image, input, starts_image, |input| {
                    parse_list_style_image(input, numeric)
                })? {
                    true
                } else {
                    fill(&mut image, input, |input| {
                        parse_list_style_image(input, numeric)
                    })?
                };
                if matched {
                    last_slot = Some(location);
                }
                Ok(matched)
            }
            4 => {
                let location = input.site();
                let matched = commit(
                    &mut style_type,
                    input,
                    |_| true,
                    |input| parse_list_style_type(input, numeric),
                )?;
                if matched {
                    last_slot = Some(location);
                }
                Ok(matched)
            }
            _ => unreachable!("five list-style operands"),
        },
    )?;
    let none_count = usize::from(matched[0]) + usize::from(matched[1]);

    let free = usize::from(style_type.is_none()) + usize::from(image.is_none());
    if none_count > free {
        return Err(unexpected_at(last_slot.unwrap_or_else(|| input.site())));
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
        .ok_or_else(|| unexpected_at(input.site()))
}
