use cssparser::{ParseError, Parser, match_ignore_ascii_case};

use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
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
                Err(unsupported_value(
                    input,
                    None,
                    "a `quotes` keyword cannot be combined with quotation pairs",
                ))
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
        .ok_or_else(|| unsupported_value(input, None, "quotes requires a keyword or string pairs"))
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
        return Err(unsupported_value(
            input,
            None,
            "`none` cannot be combined with counter changes",
        ));
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
        .ok_or_else(|| unsupported_value(input, None, "counter change list is empty"))
}

pub(super) fn parse_content_string<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContentString, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = input.expect_string_cloned().map_err(basic)?;
    CssContentString::try_new(value.to_string())
        .ok_or_else(|| unsupported_value_at(location, None, "content string contains null"))
}
