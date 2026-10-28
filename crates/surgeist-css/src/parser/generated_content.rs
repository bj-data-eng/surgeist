use super::property_input::Stream;
use crate::diagnostic::error::property::{Error, SyntaxError, basic, unexpected_at};
use cssparser::Token;
use surgeist_css_parser::match_ignore_ascii_case;

use crate::values::content::{CssContentString, CssQuotePair, CssQuotePairList, CssQuotes};

pub(crate) fn parse_quotes<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssQuotes, SyntaxError<'i, Error>> {
    let state = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        let keyword = match_ignore_ascii_case! { &ident,
            "auto" => Some(CssQuotes::Auto),
            "none" => Some(CssQuotes::None),
            "match-parent" => Some(CssQuotes::MatchParent),
            _ => None,
        };
        if let Some(keyword) = keyword {
            return if input.only_whitespace_remains() {
                Ok(keyword)
            } else {
                Err(unexpected_at(input.site()))
            };
        }
    }
    input.restore(state);

    let mut pairs = Vec::new();
    while !input.only_whitespace_remains() {
        let open = parse_content_string(input)?;
        let close = parse_content_string(input)?;
        pairs.push(CssQuotePair::new(open, close));
    }
    CssQuotePairList::try_new(pairs)
        .map(CssQuotes::Pairs)
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(crate) fn parse_content<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssContentValue, SyntaxError<'i, Error>> {
    super::content_values::parse_content_value(input, numeric)
}

pub(crate) fn parse_counter_changes<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssCounterChangesValue, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        if input.only_whitespace_remains() {
            return Ok(crate::values::content::CssCounterChangesValue::none());
        }
        return Err(unexpected_at(input.site()));
    }

    let mut changes = Vec::new();
    while !input.only_whitespace_remains() {
        let name = super::content_values::parse_counter_name(input)?;
        // An integer or math function commits the optional value, so a
        // malformed one is responsible itself.
        let value = if starts_integer(input) {
            Some(super::values::parse_integer_value(input, numeric)?)
        } else {
            None
        };
        changes.push(crate::values::content::CssCounterChangeValue::new(
            name, value,
        ));
    }

    crate::values::content::CssCounterChangesValue::try_changes(changes)
        .ok_or_else(|| unexpected_at(input.site()))
}

pub(super) fn parse_content_string<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<CssContentString, SyntaxError<'i, Error>> {
    let location = input.site();
    let value = input.consume_string().map_err(basic)?;
    CssContentString::try_new(value.to_string()).ok_or_else(|| unexpected_at(location))
}

pub(crate) fn parse_string_set<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<crate::values::content::CssStringSet, SyntaxError<'i, Error>> {
    let state = input.mark();
    if input.attempt(|input| input.consume_keyword("none")).is_ok()
        && input.only_whitespace_remains()
    {
        return Ok(crate::values::content::CssStringSet::none());
    }
    // `none "named"` is a generic custom-ident assignment, not the whole keyword.
    input.restore(state);
    let mut entries = Vec::new();
    loop {
        let name = super::content_values::parse_generic_name(input)?;
        let mut strings = vec![parse_content_string(input)?];
        while let Ok(string) = input.attempt(parse_content_string) {
            strings.push(string);
        }
        entries.push(
            crate::values::content::CssStringSetEntry::try_new(name, strings)
                .expect("nonempty strings"),
        );
        if input.only_whitespace_remains() {
            break;
        }
        input.consume_comma().map_err(basic)?;
    }
    Ok(crate::values::content::CssStringSet::try_entries(entries).expect("nonempty assignments"))
}

pub(crate) fn parse_bookmark_level<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssBookmarkLevel, SyntaxError<'i, Error>> {
    if input.attempt(|input| input.consume_keyword("none")).is_ok() {
        return Ok(crate::values::content::CssBookmarkLevel::none());
    }
    let level = super::values::parse_positive_integer_value(input, numeric)?;
    Ok(crate::values::content::CssBookmarkLevel::new(level))
}

pub(crate) fn parse_bookmark_label<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssContentList, SyntaxError<'i, Error>> {
    super::content_values::parse_content_list(input, numeric)
}

pub(crate) fn parse_bookmark_state<'i>(
    input: &mut Stream<'i>,
) -> std::result::Result<crate::values::content::CssBookmarkState, SyntaxError<'i, Error>> {
    let keyword_location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "open" => Ok(crate::values::content::CssBookmarkState::Open),
        "closed" => Ok(crate::values::content::CssBookmarkState::Closed),
        _ => Err(unexpected_at(keyword_location)),
    }
}

fn starts_integer(input: &mut Stream<'_>) -> bool {
    let state = input.mark();
    let starts = match input.consume_after_whitespace() {
        Ok(Token::Number { .. }) => true,
        Ok(Token::Function(name)) => crate::numeric::expression::is_math_function(name),
        _ => false,
    };
    input.restore(state);
    starts
}
