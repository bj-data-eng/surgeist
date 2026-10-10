use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use crate::error::{Error, basic, unexpected_at};
use crate::syntax::*;

pub(super) fn parse_quotes<'i, 't>(
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

pub(super) fn parse_content<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<crate::CssContentValue, ParseError<'i, Error>> {
    super::content_values::parse_content_value(input, numeric)
}

pub(super) fn parse_counter_changes<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<crate::CssCounterChangesValue, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        if input.is_exhausted() {
            return Ok(crate::CssCounterChangesValue::none());
        }
        return Err(unexpected_at(input.current_source_location()));
    }

    let mut changes = Vec::new();
    while !input.is_exhausted() {
        let name = super::content_values::parse_counter_name(input)?;
        let value = input
            .try_parse(|input| super::values::parse_integer_value(input, numeric))
            .ok();
        changes.push(crate::CssCounterChangeValue::new(name, value));
    }

    crate::CssCounterChangesValue::try_changes(changes)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_content_string<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContentString, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = input.expect_string_cloned().map_err(basic)?;
    CssContentString::try_new(value.to_string()).ok_or_else(|| unexpected_at(location))
}

pub(super) fn parse_string_set<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<crate::CssStringSet, ParseError<'i, Error>> {
    let state = input.state();
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
        && input.is_exhausted()
    {
        return Ok(crate::CssStringSet::none());
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
        entries.push(crate::CssStringSetEntry::try_new(name, strings).expect("nonempty strings"));
        if input.is_exhausted() {
            break;
        }
        input.expect_comma().map_err(basic)?;
    }
    Ok(crate::CssStringSet::try_entries(entries).expect("nonempty assignments"))
}

pub(super) fn parse_bookmark_level<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<crate::CssBookmarkLevel, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(crate::CssBookmarkLevel::none());
    }
    let level = super::values::parse_positive_integer_value(input, numeric)?;
    Ok(crate::CssBookmarkLevel::try_new(level).expect("checked positive integer root"))
}

pub(super) fn parse_bookmark_label<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> std::result::Result<crate::CssContentList, ParseError<'i, Error>> {
    super::content_values::parse_content_list(input, numeric)
}

pub(super) fn parse_bookmark_state<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<crate::CssBookmarkState, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "open" => Ok(crate::CssBookmarkState::Open),
        "closed" => Ok(crate::CssBookmarkState::Closed),
        _ => Err(unexpected_at(input.current_source_location())),
    }
}
