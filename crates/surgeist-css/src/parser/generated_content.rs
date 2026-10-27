use super::property_input::Parser;
use crate::diagnostic::error::property::{Error, ParseError, basic, unexpected_at};
use cssparser::{Token, match_ignore_ascii_case};

use crate::values::content::{CssContentString, CssQuotePair, CssQuotePairList, CssQuotes};

pub(crate) fn parse_quotes<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssQuotes, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let keyword = match_ignore_ascii_case! { &ident,
            "auto" => Some(CssQuotes::Auto),
            "none" => Some(CssQuotes::None),
            "match-parent" => Some(CssQuotes::MatchParent),
            _ => None,
        };
        if let Some(keyword) = keyword {
            return if input.is_exhausted() {
                Ok(keyword)
            } else {
                Err(unexpected_at(input.current_source_location()))
            };
        }
    }
    input.reset(&state);

    let mut pairs = Vec::new();
    while !input.is_exhausted() {
        let open = parse_content_string(input)?;
        let close = parse_content_string(input)?;
        pairs.push(CssQuotePair::new(open, close));
    }
    CssQuotePairList::try_new(pairs)
        .map(CssQuotes::Pairs)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(crate) fn parse_content<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssContentValue, ParseError<'i, Error>> {
    super::content_values::parse_content_value(input, numeric)
}

pub(crate) fn parse_counter_changes<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssCounterChangesValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        if input.is_exhausted() {
            return Ok(crate::values::content::CssCounterChangesValue::none());
        }
        return Err(unexpected_at(input.current_source_location()));
    }

    let mut changes = Vec::new();
    while !input.is_exhausted() {
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
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_content_string<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContentString, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = input.expect_string_cloned().map_err(basic)?;
    CssContentString::try_new(value.to_string()).ok_or_else(|| unexpected_at(location))
}

pub(crate) fn parse_string_set<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<crate::values::content::CssStringSet, ParseError<'i, Error>> {
    let state = input.state();
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
        && input.is_exhausted()
    {
        return Ok(crate::values::content::CssStringSet::none());
    }
    // `none "named"` is a generic custom-ident assignment, not the whole keyword.
    input.reset(&state);
    let mut entries = Vec::new();
    loop {
        let name = super::content_values::parse_generic_name(input)?;
        let mut strings = vec![parse_content_string(input)?];
        while let Ok(string) = input.try_parse(parse_content_string) {
            strings.push(string);
        }
        entries.push(
            crate::values::content::CssStringSetEntry::try_new(name, strings)
                .expect("nonempty strings"),
        );
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
    }
    Ok(crate::values::content::CssStringSet::try_entries(entries).expect("nonempty assignments"))
}

pub(crate) fn parse_bookmark_level<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssBookmarkLevel, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(crate::values::content::CssBookmarkLevel::none());
    }
    let level = super::values::parse_positive_integer_value(input, numeric)?;
    Ok(crate::values::content::CssBookmarkLevel::new(level))
}

pub(crate) fn parse_bookmark_label<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> std::result::Result<crate::values::content::CssContentList, ParseError<'i, Error>> {
    super::content_values::parse_content_list(input, numeric)
}

pub(crate) fn parse_bookmark_state<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<crate::values::content::CssBookmarkState, ParseError<'i, Error>> {
    let keyword_location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "open" => Ok(crate::values::content::CssBookmarkState::Open),
        "closed" => Ok(crate::values::content::CssBookmarkState::Closed),
        _ => Err(unexpected_at(keyword_location)),
    }
}

fn starts_integer(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let starts = match input.next() {
        Ok(Token::Number { .. }) => true,
        Ok(Token::Function(name)) => crate::numeric::expression::is_math_function(name),
        _ => false,
    };
    input.reset(&state);
    starts
}
