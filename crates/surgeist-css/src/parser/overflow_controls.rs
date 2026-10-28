//! CSS Overflow 3 terminal controls.

use super::combinators::{Combinator, Tail, commit, parse_group};
use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use super::values::parse_nonnegative_length;
use crate::numeric::expression::NumericInputContext;
use crate::values::box_model::{CssOverflowClipMargin, CssScrollBehavior, CssScrollbarGutter};
use crate::values::common::CssBoxEdgeKeyword;

pub(super) fn parse_overflow_clip_margin<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssOverflowClipMargin, SyntaxError<'i, Error>> {
    // `<visual-box> || <length [0,∞]>`: an identifier commits the box and a
    // numeric token or function commits the length.
    fn next_token_is<'i>(input: &mut Stream<'i>, admit: fn(Token<'i>) -> bool) -> bool {
        let state = input.mark();
        let starts = input.consume_after_whitespace().is_ok_and(admit);
        input.restore(state);
        starts
    }
    let (mut box_edge, mut offset) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => commit(
                &mut box_edge,
                input,
                |input| next_token_is(input, |token| matches!(token, Token::Ident(_))),
                |input| {
                    input.discard_whitespace();
                    let location = input.site();
                    let ident = input.consume_ident().map_err(basic)?;
                    CssBoxEdgeKeyword::from_keyword(ident).ok_or_else(|| unexpected_at(location))
                },
            ),
            1 => commit(
                &mut offset,
                input,
                |input| {
                    next_token_is(input, |token| {
                        matches!(
                            token,
                            Token::Number(_)
                                | Token::Dimension { .. }
                                | Token::Percentage(_)
                                | Token::Function(_)
                        )
                    })
                },
                |input| parse_nonnegative_length(input, numeric),
            ),
            _ => unreachable!("two overflow-clip-margin operands"),
        },
    )?;
    CssOverflowClipMargin::try_new(box_edge, offset).ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_scroll_behavior<'i>(
    input: &mut Stream<'i>,
) -> Result<CssScrollBehavior, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(CssScrollBehavior::Auto),
        "smooth" => Ok(CssScrollBehavior::Smooth),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(super) fn parse_scrollbar_gutter<'i>(
    input: &mut Stream<'i>,
) -> Result<CssScrollbarGutter, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let first = input.consume_ident().map_err(basic)?;
    if first.eq_ignore_ascii_case("auto") {
        return Ok(CssScrollbarGutter::Auto);
    }
    if first.eq_ignore_ascii_case("stable") {
        if input
            .attempt(|input| input.consume_keyword("both-edges"))
            .is_ok()
        {
            return Ok(CssScrollbarGutter::StableBothEdges);
        }
        return Ok(CssScrollbarGutter::Stable);
    }
    if first.eq_ignore_ascii_case("both-edges") {
        input.consume_keyword("stable").map_err(basic)?;
        return Ok(CssScrollbarGutter::StableBothEdges);
    }
    // The unknown keyword itself cannot start the value.
    Err(unexpected_at(keyword_location))
}

pub(super) fn parse_overflow_anchor<'i>(
    input: &mut Stream<'i>,
) -> Result<crate::values::box_model::CssOverflowAnchor, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "auto" => Ok(crate::values::box_model::CssOverflowAnchor::Auto),
        "none" => Ok(crate::values::box_model::CssOverflowAnchor::None),
        _ => Err(unexpected_at(keyword_location)),
    }
}
