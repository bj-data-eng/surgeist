//! Text 4 authored alignment grammar and source-preserving string admission.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::property_input::StreamExt;
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use crate::component::{CssComponentValueRef, CssValueTokenRef};
use crate::numeric::expression::NumericInputContext;
use crate::values::text::{
    CssCharacterAlignment, CssTextAlign, CssTextAlignAllValue, CssTextAlignLastValue,
    CssTextAlignPosition, CssTextAlignValue,
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

fn character<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
    fallback: Option<CssTextAlignPosition>,
) -> Result<CssCharacterAlignment, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let offset = input.site();
    let component = input.collect_numeric_component(numeric).map_err(|error| {
        unexpected_at(numeric.property_error_location(&error, location, offset))
    })?;
    if !matches!(
        component.view(),
        CssComponentValueRef::Token(CssValueTokenRef::String(_))
    ) {
        return Err(unexpected_at(location));
    }
    CssCharacterAlignment::try_from_component(component, fallback)
        .map_err(|_| unexpected_at(location))
}

fn alignment<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssTextAlignAllValue, SyntaxError<'i, Error>> {
    let start = input.mark();
    // A token that starts no alignment is itself responsible.
    let token_location = input.site();
    let token = input.consume_after_whitespace().map_err(basic)?;
    match token {
        Token::String(_) => {
            input.restore(start);
            let component = character(input, numeric, None)?;
            if input.only_whitespace_remains() {
                return Ok(CssTextAlignAllValue::Character(component));
            }
            let keyword_location = input.site();
            let ident = input.consume_ident().map_err(basic)?;
            let fallback = position(ident).ok_or_else(|| unexpected_at(keyword_location))?;
            let component = CssCharacterAlignment::try_from_component(
                component.component().clone(),
                Some(fallback),
            )
            .expect("previously checked character component");
            Ok(CssTextAlignAllValue::Character(component))
        }
        Token::Ident(ident) => {
            let value = keyword(ident).ok_or_else(|| unexpected_at(token_location))?;
            if let Some(fallback) = position(ident)
                && !input.only_whitespace_remains()
            {
                return Ok(CssTextAlignAllValue::Character(character(
                    input,
                    numeric,
                    Some(fallback),
                )?));
            }
            Ok(CssTextAlignAllValue::Keyword(value))
        }
        _ => Err(unexpected_at(token_location)),
    }
}

pub(super) fn parse_text_align<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssTextAlignValue, SyntaxError<'i, Error>> {
    if input
        .attempt(|input| input.consume_keyword("justify-all"))
        .is_ok()
    {
        return Ok(CssTextAlignValue::JustifyAll);
    }
    alignment(input, numeric).map(CssTextAlignValue::Alignment)
}

pub(super) fn parse_text_align_all<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssTextAlignAllValue, SyntaxError<'i, Error>> {
    alignment(input, numeric)
}

pub(super) fn parse_text_align_last<'i>(
    input: &mut Stream<'i>,
) -> Result<CssTextAlignLastValue, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    if ident.eq_ignore_ascii_case("auto") {
        return Ok(CssTextAlignLastValue::Auto);
    }
    keyword(ident)
        .map(CssTextAlignLastValue::Keyword)
        .ok_or_else(|| unexpected_at(keyword_location))
}
