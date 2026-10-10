//! Wrapping, whitespace, and line-breaking grammar.
use crate::error::{Error, basic, unexpected_at};
use crate::syntax::*;
use cssparser::{ParseError, Parser, match_ignore_ascii_case};

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

pub(super) fn parse_text_wrap_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextWrapMode, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    text_wrap_mode_keyword(&ident).ok_or_else(|| unexpected_at(input.current_source_location()))
}
pub(super) fn parse_text_wrap_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextWrapStyle, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    text_wrap_style_keyword(&ident).ok_or_else(|| unexpected_at(input.current_source_location()))
}
pub(super) fn parse_white_space_collapse<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWhiteSpaceCollapse, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    white_space_collapse_keyword(&ident)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_white_space_trim<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWhiteSpaceTrim, ParseError<'i, Error>> {
    let mut ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("none") {
        return Ok(CssWhiteSpaceTrim::none());
    }
    let (mut before, mut after, mut inner) = (false, false, false);
    loop {
        let flag = match_ignore_ascii_case! { &ident,
            "discard-before" => &mut before, "discard-after" => &mut after, "discard-inner" => &mut inner,
            _ => return Err(unexpected_at(input.current_source_location())),
        };
        if *flag {
            return Err(unexpected_at(input.current_source_location()));
        }
        *flag = true;
        let state = input.state();
        let Ok(next) = input.try_parse(|input| input.expect_ident_cloned()) else {
            break;
        };
        if next.eq_ignore_ascii_case("discard-before")
            || next.eq_ignore_ascii_case("discard-after")
            || next.eq_ignore_ascii_case("discard-inner")
        {
            ident = next;
        } else {
            input.reset(&state);
            break;
        }
    }
    Ok(CssWhiteSpaceTrim::new(before, after, inner))
}

pub(super) fn parse_text_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextWrap, ParseError<'i, Error>> {
    let (mut mode, mut style) = (None, None);
    while !input.is_exhausted() {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if let Some(value) = text_wrap_mode_keyword(&ident) {
            if mode.replace(value).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
        } else if let Some(value) = text_wrap_style_keyword(&ident) {
            if style.replace(value).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
        } else {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssTextWrap::try_new(mode, style).ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_white_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWhiteSpace, ParseError<'i, Error>> {
    let start = input.state();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let keyword = match_ignore_ascii_case! { &ident,
        "normal" => Some(CssWhiteSpaceKeyword::Normal), "pre" => Some(CssWhiteSpaceKeyword::Pre),
        "pre-wrap" => Some(CssWhiteSpaceKeyword::PreWrap), "pre-line" => Some(CssWhiteSpaceKeyword::PreLine), _ => None,
    };
    if let Some(keyword) = keyword {
        return Ok(CssWhiteSpace::from_keyword(keyword));
    }
    input.reset(&start);
    let (mut collapse, mut mode, mut trim) = (None, None, None);
    while !input.is_exhausted() {
        let state = input.state();
        let ident = input.expect_ident_cloned().map_err(basic)?;
        if let Some(value) = white_space_collapse_keyword(&ident) {
            if collapse.replace(value).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
        } else if let Some(value) = text_wrap_mode_keyword(&ident) {
            if mode.replace(value).is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
        } else if ident.eq_ignore_ascii_case("none")
            || ident.eq_ignore_ascii_case("discard-before")
            || ident.eq_ignore_ascii_case("discard-after")
            || ident.eq_ignore_ascii_case("discard-inner")
        {
            // Quoted property ranges preserve their grouping: the entire trim
            // constituent is contiguous and can occur only once (Values4 §2.2).
            if trim.is_some() {
                return Err(unexpected_at(input.current_source_location()));
            }
            input.reset(&state);
            trim = Some(parse_white_space_trim(input)?);
        } else {
            return Err(unexpected_at(input.current_source_location()));
        }
    }
    CssWhiteSpace::try_new(collapse, mode, trim)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_word_break<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssWordBreak, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssWordBreak::Normal),
        "break-all" => Ok(CssWordBreak::BreakAll),
        "keep-all" => Ok(CssWordBreak::KeepAll),
        "manual" => Ok(CssWordBreak::Manual),
        "auto-phrase" => Ok(CssWordBreak::AutoPhrase),
        "break-word" => Ok(CssWordBreak::BreakWord),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_overflow_wrap<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssOverflowWrap, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "normal" => Ok(CssOverflowWrap::Normal),
        "break-word" => Ok(CssOverflowWrap::BreakWord),
        "anywhere" => Ok(CssOverflowWrap::Anywhere),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}

pub(super) fn parse_text_overflow<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssTextOverflow, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "clip" => Ok(CssTextOverflow::Clip),
        "ellipsis" => Ok(CssTextOverflow::Ellipsis),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}
