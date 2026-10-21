//! CSS Overflow 3 terminal controls.

use super::property_input::Parser;
use crate::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::{Token, match_ignore_ascii_case};

use super::values::parse_nonnegative_length;
use crate::numeric::NumericInputContext;
use crate::{CssBoxEdgeKeyword, CssOverflowClipMargin, CssScrollBehavior, CssScrollbarGutter};

pub(super) fn parse_overflow_clip_margin<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOverflowClipMargin, ParseError<'i, Error>> {
    let mut box_edge = None;
    let mut offset = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let state = input.state();
        let location = input.current_source_location();
        match input.next().map_err(basic)? {
            Token::Ident(ident) => {
                let edge = CssBoxEdgeKeyword::from_keyword(ident)
                    .ok_or_else(|| unexpected_at(location))?;
                if box_edge.replace(edge).is_some() {
                    return Err(unexpected_at(location));
                }
            }
            Token::Number { .. }
            | Token::Dimension { .. }
            | Token::Percentage { .. }
            | Token::Function(_) => {
                if offset.is_some() {
                    return Err(unexpected_at(location));
                }
                input.reset(&state);
                offset = Some(parse_nonnegative_length(input, numeric)?);
            }
            token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
        }
    }
    CssOverflowClipMargin::try_new(box_edge, offset)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_scroll_behavior<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollBehavior, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssScrollBehavior::Auto),
        "smooth" => Ok(CssScrollBehavior::Smooth),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_scrollbar_gutter<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssScrollbarGutter, ParseError<'i, Error>> {
    let first = input.expect_ident_cloned().map_err(basic)?;
    if first.eq_ignore_ascii_case("auto") {
        return Ok(CssScrollbarGutter::Auto);
    }
    if first.eq_ignore_ascii_case("stable") {
        if input
            .try_parse(|input| input.expect_ident_matching("both-edges"))
            .is_ok()
        {
            return Ok(CssScrollbarGutter::StableBothEdges);
        }
        return Ok(CssScrollbarGutter::Stable);
    }
    if first.eq_ignore_ascii_case("both-edges") {
        input.expect_ident_matching("stable").map_err(basic)?;
        return Ok(CssScrollbarGutter::StableBothEdges);
    }
    Err(unexpected_at(input.current_source_location()))
}

pub(super) fn parse_overflow_anchor<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<crate::CssOverflowAnchor, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(crate::CssOverflowAnchor::Auto),
        "none" => Ok(crate::CssOverflowAnchor::None),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}
