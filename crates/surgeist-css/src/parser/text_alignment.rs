//! Text 4 authored alignment grammar and source-preserving string admission.

use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::numeric::NumericInputContext;
use crate::{
    CssCharacterAlignment, CssComponentValueRef, CssTextAlign, CssTextAlignAllValue,
    CssTextAlignLastValue, CssTextAlignPosition, CssTextAlignValue, CssValueTokenRef,
};

fn keyword(ident: &str) -> Option<CssTextAlign> {
    match_ignore_ascii_case! { ident,
        "start" => Some(CssTextAlign::Start),
        "end" => Some(CssTextAlign::End),
        "left" => Some(CssTextAlign::Left),
        "right" => Some(CssTextAlign::Right),
        "center" => Some(CssTextAlign::Center),
        "justify" => Some(CssTextAlign::Justify),
        "match-parent" => Some(CssTextAlign::MatchParent),
        _ => None,
    }
}

fn position(ident: &str) -> Option<CssTextAlignPosition> {
    match_ignore_ascii_case! { ident,
        "start" => Some(CssTextAlignPosition::Start),
        "end" => Some(CssTextAlignPosition::End),
        "left" => Some(CssTextAlignPosition::Left),
        "right" => Some(CssTextAlignPosition::Right),
        "center" => Some(CssTextAlignPosition::Center),
        _ => None,
    }
}

fn character<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    fallback: Option<CssTextAlignPosition>,
) -> Result<CssCharacterAlignment, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    let component = numeric.collect(input).map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, offset),
            None,
            "invalid text alignment string component",
        )
    })?;
    if !matches!(
        component.view(),
        CssComponentValueRef::Token(CssValueTokenRef::String(_))
    ) {
        return Err(unsupported_value_at(
            location,
            None,
            "text alignment requires a string component",
        ));
    }
    CssCharacterAlignment::try_from_component(component, fallback).map_err(|_| {
        unsupported_value_at(
            location,
            None,
            "text alignment string must contain exactly one extended grapheme cluster",
        )
    })
}

fn alignment<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    shorthand: bool,
) -> Result<CssTextAlignAllValue, ParseError<'i, Error>> {
    let start = input.state();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::QuotedString(_) => {
            input.reset(&start);
            let component = character(input, numeric, None)?;
            if input.is_exhausted() {
                return Ok(CssTextAlignAllValue::Character(component));
            }
            let ident = input.expect_ident_cloned().map_err(basic)?;
            let fallback = position(ident.as_ref()).ok_or_else(|| {
                unsupported_value(
                    input,
                    None,
                    "only a positional keyword can follow a character",
                )
            })?;
            let component = CssCharacterAlignment::try_from_component(
                component.component().clone(),
                Some(fallback),
            )
            .expect("previously checked character component");
            Ok(CssTextAlignAllValue::Character(component))
        }
        Token::Ident(ident) => {
            let value = keyword(ident.as_ref())
                .ok_or_else(|| unsupported_value(input, None, "unknown text alignment keyword"))?;
            if let Some(fallback) = position(ident.as_ref())
                && !input.is_exhausted()
            {
                return Ok(CssTextAlignAllValue::Character(character(
                    input,
                    numeric,
                    Some(fallback),
                )?));
            }
            Ok(CssTextAlignAllValue::Keyword(value))
        }
        _ => Err(unsupported_value(
            input,
            None,
            if shorthand {
                "text-align requires a keyword or character alignment"
            } else {
                "text-align-all requires a keyword or character alignment"
            },
        )),
    }
}

pub(super) fn parse_text_align<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssTextAlignValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("justify-all"))
        .is_ok()
    {
        return Ok(CssTextAlignValue::JustifyAll);
    }
    alignment(input, numeric, true).map(CssTextAlignValue::Alignment)
}

pub(super) fn parse_text_align_all<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssTextAlignAllValue, ParseError<'i, Error>> {
    alignment(input, numeric, false)
}

pub(super) fn parse_text_align_last<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<CssTextAlignLastValue, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if ident.eq_ignore_ascii_case("auto") {
        return Ok(CssTextAlignLastValue::Auto);
    }
    keyword(ident.as_ref())
        .map(CssTextAlignLastValue::Keyword)
        .ok_or_else(|| unsupported_value(input, None, "unknown text-align-last keyword"))
}
