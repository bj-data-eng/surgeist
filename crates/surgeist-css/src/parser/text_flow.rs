//! Wrapping, whitespace, and line-breaking grammar.
use super::combinators::{Combinator, Tail, commit, fill, parse_group};
use super::syntax_types::Stream;
use super::values::next_is_ident;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use crate::values::text::{
    CssOverflowWrap, CssTextOverflow, CssTextWrap, CssTextWrapMode, CssTextWrapStyle,
    CssWhiteSpace, CssWhiteSpaceCollapse, CssWhiteSpaceKeyword, CssWhiteSpaceTrim, CssWordBreak,
};
use surgeist_css_parser::match_ignore_ascii_case;

fn text_wrap_mode_keyword(ident: &str) -> Option<CssTextWrapMode> {
    match_ignore_ascii_case! { ident,
        "wrap" => Some(CssTextWrapMode::Wrap), "nowrap" => Some(CssTextWrapMode::NoWrap),
        _ => None,
    }
}
fn text_wrap_style_keyword(ident: &str) -> Option<CssTextWrapStyle> {
    match_ignore_ascii_case! { ident,
        "auto" => Some(CssTextWrapStyle::Auto), "balance" => Some(CssTextWrapStyle::Balance),
        "stable" => Some(CssTextWrapStyle::Stable), "pretty" => Some(CssTextWrapStyle::Pretty),
        "avoid-short-last-line" => Some(CssTextWrapStyle::AvoidShortLastLine), _ => None,
    }
}
fn white_space_collapse_keyword(ident: &str) -> Option<CssWhiteSpaceCollapse> {
    match_ignore_ascii_case! { ident,
        "collapse" => Some(CssWhiteSpaceCollapse::Collapse), "discard" => Some(CssWhiteSpaceCollapse::Discard),
        "preserve" => Some(CssWhiteSpaceCollapse::Preserve), "preserve-breaks" => Some(CssWhiteSpaceCollapse::PreserveBreaks),
        "preserve-spaces" => Some(CssWhiteSpaceCollapse::PreserveSpaces), "break-spaces" => Some(CssWhiteSpaceCollapse::BreakSpaces),
        _ => None,
    }
}

pub(crate) fn parse_text_wrap_mode<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextWrapMode, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    text_wrap_mode_keyword(ident).ok_or_else(|| unexpected_at(keyword_location))
}
pub(crate) fn parse_text_wrap_style<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextWrapStyle, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    text_wrap_style_keyword(ident).ok_or_else(|| unexpected_at(keyword_location))
}
pub(crate) fn parse_white_space_collapse<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssWhiteSpaceCollapse, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    white_space_collapse_keyword(ident).ok_or_else(|| unexpected_at(keyword_location))
}

pub(crate) fn parse_white_space_trim<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssWhiteSpaceTrim, SyntaxError<'i, Error>> {
    // Each unknown or repeated keyword is itself responsible.
    let mut keyword_location = input.site();
    let mut ident = input.consume_ident().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        return Ok(CssWhiteSpaceTrim::none());
    }
    let (mut before, mut after, mut inner) = (false, false, false);
    loop {
        let flag = match_ignore_ascii_case! { &ident,
            "discard-before" => &mut before, "discard-after" => &mut after, "discard-inner" => &mut inner,
            _ => return Err(unexpected_at(keyword_location)),
        };
        if *flag {
            return Err(unexpected_at(keyword_location));
        }
        *flag = true;
        let state = input.mark();
        let next_location = input.site();
        let Ok(next) = input.attempt(|input| input.consume_ident()) else {
            break;
        };
        if next.eq_ignore_ascii_case("discard-before")
            || next.eq_ignore_ascii_case("discard-after")
            || next.eq_ignore_ascii_case("discard-inner")
        {
            ident = next;
            keyword_location = next_location;
        } else {
            input.restore(state);
            break;
        }
    }
    Ok(CssWhiteSpaceTrim::new(before, after, inner))
}

fn next_starts_white_space_trim(input: &mut Stream<'_>) -> bool {
    ["none", "discard-before", "discard-after", "discard-inner"]
        .iter()
        .any(|keyword| next_is_ident(input, keyword))
}

pub(crate) fn parse_text_wrap<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextWrap, SyntaxError<'i, Error>> {
    // `<'text-wrap-mode'> || <'text-wrap-style'>`.
    let (mut mode, mut style) = (None, None);
    parse_group::<2>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut mode, input, parse_text_wrap_mode),
            1 => fill(&mut style, input, parse_text_wrap_style),
            _ => unreachable!("two text-wrap operands"),
        },
    )?;
    CssTextWrap::try_new(mode, style).ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_white_space<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssWhiteSpace, SyntaxError<'i, Error>> {
    let start = input.mark();
    let ident = input.consume_ident().map_err(basic)?;
    let keyword = match_ignore_ascii_case! { &ident,
        "normal" => Some(CssWhiteSpaceKeyword::Normal), "pre" => Some(CssWhiteSpaceKeyword::Pre),
        "pre-wrap" => Some(CssWhiteSpaceKeyword::PreWrap), "pre-line" => Some(CssWhiteSpaceKeyword::PreLine), _ => None,
    };
    if let Some(keyword) = keyword {
        return Ok(CssWhiteSpace::from_keyword(keyword));
    }
    input.restore(start);
    // `<'white-space-collapse'> || <'text-wrap-mode'> || <'white-space-trim'>`.
    // The whole trim value is one operand (Values 4 §2.2).
    let (mut collapse, mut mode, mut trim) = (None, None, None);
    parse_group::<3>(
        input,
        Combinator::AnyOf,
        Tail::Reject,
        |_| false,
        |operand, input| match operand {
            0 => fill(&mut collapse, input, parse_white_space_collapse),
            1 => fill(&mut mode, input, parse_text_wrap_mode),
            2 => commit(
                &mut trim,
                input,
                next_starts_white_space_trim,
                parse_white_space_trim,
            ),
            _ => unreachable!("three white-space operands"),
        },
    )?;
    CssWhiteSpace::try_new(collapse, mode, trim).ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_word_break<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssWordBreak, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssWordBreak::Normal),
        "break-all" => Ok(CssWordBreak::BreakAll),
        "keep-all" => Ok(CssWordBreak::KeepAll),
        "manual" => Ok(CssWordBreak::Manual),
        "auto-phrase" => Ok(CssWordBreak::AutoPhrase),
        "break-word" => Ok(CssWordBreak::BreakWord),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_overflow_wrap<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssOverflowWrap, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssOverflowWrap::Normal),
        "break-word" => Ok(CssOverflowWrap::BreakWord),
        "anywhere" => Ok(CssOverflowWrap::Anywhere),
        _ => Err(unexpected_at(keyword_location)),
    }
}

pub(crate) fn parse_text_overflow<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssTextOverflow, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "clip" => Ok(CssTextOverflow::Clip),
        "ellipsis" => Ok(CssTextOverflow::Ellipsis),
        _ => Err(unexpected_at(keyword_location)),
    }
}
