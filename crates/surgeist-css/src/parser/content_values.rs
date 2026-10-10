//! Grammar admission for the selected CSS Generated Content 3 `content` value.

use cssparser::{ParseError, Parser, Token};

use super::background::parse_image;
use super::generated_content::parse_content_string;
use super::url::parse_url;
use crate::CssIdent;
use crate::content_values::*;
use crate::error::{Error, basic, unexpected_at};

type Result<'i, T> = std::result::Result<T, ParseError<'i, Error>>;

pub(super) fn parse_content_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssContentValue> {
    let start = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let keyword = if ident.eq_ignore_ascii_case("normal") {
            Some(CssContentValue::Normal)
        } else if ident.eq_ignore_ascii_case("none") {
            Some(CssContentValue::None)
        } else {
            None
        };
        if let Some(keyword) = keyword {
            return if input.is_exhausted() {
                Ok(keyword)
            } else {
                Err(unexpected_at(input.current_source_location()))
            };
        }
    }
    input.reset(&start);

    let mut items = Vec::new();
    while !input.is_exhausted() {
        if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            let alternative = parse_alternative(input, numeric)?;
            input.expect_exhausted().map_err(basic)?;
            return CssGeneratedContent::try_new(items, Some(alternative))
                .map(CssContentValue::Generated)
                .ok_or_else(|| unexpected_at(input.current_source_location()));
        }
        items.push(parse_item(input, numeric)?);
    }
    CssGeneratedContent::try_new(items, None)
        .map(CssContentValue::Generated)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

/// Admits the full shared nonempty list without `content` keywords/slash branches.
pub(super) fn parse_content_list<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssContentList> {
    let mut items = Vec::new();
    while !input.is_exhausted() {
        items.push(parse_item(input, numeric)?);
    }
    CssContentList::try_new(items).ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_alternative<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssContentAlternative> {
    let mut items = Vec::new();
    while !input.is_exhausted() {
        if let Ok(string) = input.try_parse(parse_content_string) {
            items.push(CssContentAlternativeItem::String(string));
            continue;
        }
        let location = input.current_source_location();
        match input.next().map_err(basic)?.clone() {
            Token::Function(name) if name.eq_ignore_ascii_case("counter") => {
                items.push(CssContentAlternativeItem::Counter(
                    input.parse_nested_block(|input| parse_counter(input, numeric))?,
                ))
            }
            Token::Function(name) if name.eq_ignore_ascii_case("counters") => {
                items.push(CssContentAlternativeItem::Counters(
                    input.parse_nested_block(|input| parse_counters(input, numeric))?,
                ))
            }
            token => return Err(location.new_unexpected_token_error::<Error>(token)),
        }
    }
    CssContentAlternative::try_new(items)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_item<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssContentValueItem> {
    if let Ok(string) = input.try_parse(parse_content_string) {
        return Ok(CssContentValueItem::String(string));
    }
    match input.try_parse(|input| parse_image(input, numeric)) {
        Ok(image) => return Ok(CssContentValueItem::Image(image)),
        Err(error) if crate::error::is_resource_parse_error(&error) => return Err(error),
        Err(_) => {}
    }
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match ident.to_ascii_lowercase().as_str() {
            "contents" => Ok(CssContentValueItem::Contents),
            "open-quote" => Ok(CssContentValueItem::OpenQuote),
            "close-quote" => Ok(CssContentValueItem::CloseQuote),
            "no-open-quote" => Ok(CssContentValueItem::NoOpenQuote),
            "no-close-quote" => Ok(CssContentValueItem::NoCloseQuote),
            _ => Err(unexpected_at(input.current_source_location())),
        };
    }
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Function(name) if name.eq_ignore_ascii_case("counter") => input
            .parse_nested_block(|input| parse_counter(input, numeric))
            .map(CssContentValueItem::Counter),
        Token::Function(name) if name.eq_ignore_ascii_case("counters") => input
            .parse_nested_block(|input| parse_counters(input, numeric))
            .map(CssContentValueItem::Counters),
        Token::Function(name) if name.eq_ignore_ascii_case("leader") => input
            .parse_nested_block(parse_leader)
            .map(CssContentValueItem::Leader),
        Token::Function(name) if name.eq_ignore_ascii_case("target-counter") => input
            .parse_nested_block(|input| parse_target_counter(input, numeric))
            .map(CssContentValueItem::TargetCounter),
        Token::Function(name) if name.eq_ignore_ascii_case("target-counters") => input
            .parse_nested_block(|input| parse_target_counters(input, numeric))
            .map(CssContentValueItem::TargetCounters),
        Token::Function(name) if name.eq_ignore_ascii_case("target-text") => input
            .parse_nested_block(|input| parse_target_text(input, numeric))
            .map(CssContentValueItem::TargetText),
        Token::Function(name) if name.eq_ignore_ascii_case("string") => input
            .parse_nested_block(parse_named_string)
            .map(CssContentValueItem::NamedString),
        Token::Function(name) if name.eq_ignore_ascii_case("content") => input
            .parse_nested_block(parse_content_reference)
            .map(CssContentValueItem::Content),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

pub(super) fn parse_generic_name<'i, 't>(input: &mut Parser<'i, 't>) -> Result<'i, CssContentName> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let checked = CssIdent::try_new(ident.to_string())
        .map_err(|_| unexpected_at(input.current_source_location()))?;
    CssContentName::try_new(checked).ok_or_else(|| unexpected_at(input.current_source_location()))
}

pub(super) fn parse_counter_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<'i, CssContentCounterName> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let checked = CssIdent::try_new(ident.to_string())
        .map_err(|_| unexpected_at(input.current_source_location()))?;
    CssContentCounterName::try_new(checked)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_counter<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssContentCounter> {
    let name = parse_counter_name(input)?;
    let style = if input.try_parse(Parser::expect_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssContentCounter::new(name, style))
}

fn parse_counters<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssContentCounters> {
    let name = parse_counter_name(input)?;
    input.expect_comma().map_err(basic)?;
    let separator = parse_content_string(input)?;
    let style = if input.try_parse(Parser::expect_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssContentCounters::new(name, separator, style))
}

pub(super) fn parse_style<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssCounterStyleValue> {
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        let checked = CssIdent::try_new(ident.to_string())
            .map_err(|_| unexpected_at(input.current_source_location()))?;
        return CssCounterStyleValue::try_named(checked)
            .ok_or_else(|| unexpected_at(input.current_source_location()));
    }
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Function(name) if name.eq_ignore_ascii_case("symbols") => input
            .parse_nested_block(|input| parse_symbols(input, numeric))
            .map(CssCounterStyleValue::Symbols),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_symbols<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssSymbolsStyleValue> {
    let start = input.state();
    let system = if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match ident.to_ascii_lowercase().as_str() {
            "cyclic" => Some(CssSymbolsSystem::Cyclic),
            "numeric" => Some(CssSymbolsSystem::Numeric),
            "alphabetic" => Some(CssSymbolsSystem::Alphabetic),
            "symbolic" => Some(CssSymbolsSystem::Symbolic),
            "fixed" => Some(CssSymbolsSystem::Fixed),
            _ => {
                input.reset(&start);
                None
            }
        }
    } else {
        None
    };
    let mut symbols = Vec::new();
    while !input.is_exhausted() {
        if let Ok(string) = input.try_parse(parse_content_string) {
            symbols.push(CssCounterSymbolValue::String(string));
            continue;
        }
        let image = parse_image(input, numeric)?;
        symbols.push(CssCounterSymbolValue::Image(image));
    }
    CssSymbolsStyleValue::try_new(system, symbols)
        .ok_or_else(|| unexpected_at(input.current_source_location()))
}

fn parse_target<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssContentTarget> {
    if let Ok(string) = input.try_parse(parse_content_string) {
        return Ok(CssContentTarget::String(string));
    }
    parse_url(input, numeric).map(CssContentTarget::Url)
}

fn parse_target_counter<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssTargetCounter> {
    let target = parse_target(input, numeric)?;
    input.expect_comma().map_err(basic)?;
    let name = parse_generic_name(input)?;
    let style = if input.try_parse(Parser::expect_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssTargetCounter::new(target, name, style))
}

fn parse_target_counters<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssTargetCounters> {
    let target = parse_target(input, numeric)?;
    input.expect_comma().map_err(basic)?;
    let name = parse_generic_name(input)?;
    input.expect_comma().map_err(basic)?;
    let separator = parse_content_string(input)?;
    let style = if input.try_parse(Parser::expect_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssTargetCounters::new(target, name, separator, style))
}

fn parse_target_text<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &crate::numeric::NumericInputContext<'_>,
) -> Result<'i, CssTargetText> {
    let target = parse_target(input, numeric)?;
    let mode = if input.try_parse(Parser::expect_comma).is_ok() {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        Some(match ident.to_ascii_lowercase().as_str() {
            "content" => CssTargetTextMode::Content,
            "before" => CssTargetTextMode::Before,
            "after" => CssTargetTextMode::After,
            "first-letter" => CssTargetTextMode::FirstLetter,
            _ => {
                return Err(unexpected_at(input.current_source_location()));
            }
        })
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssTargetText::new(target, mode))
}

fn parse_named_string<'i, 't>(input: &mut Parser<'i, 't>) -> Result<'i, CssNamedString> {
    let name = parse_generic_name(input)?;
    let mode = if input.try_parse(Parser::expect_comma).is_ok() {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        Some(match ident.to_ascii_lowercase().as_str() {
            "first" => CssNamedStringMode::First,
            "start" => CssNamedStringMode::Start,
            "last" => CssNamedStringMode::Last,
            "first-except" => CssNamedStringMode::FirstExcept,
            _ => {
                return Err(unexpected_at(input.current_source_location()));
            }
        })
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssNamedString::new(name, mode))
}

fn parse_content_reference<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<'i, Option<CssContentReferenceMode>> {
    let mode = if input.is_exhausted() {
        None
    } else {
        let ident = input.expect_ident_cloned().map_err(basic)?;
        Some(match ident.to_ascii_lowercase().as_str() {
            "text" => CssContentReferenceMode::Text,
            "before" => CssContentReferenceMode::Before,
            "after" => CssContentReferenceMode::After,
            "first-letter" => CssContentReferenceMode::FirstLetter,
            "marker" => CssContentReferenceMode::Marker,
            _ => {
                return Err(unexpected_at(input.current_source_location()));
            }
        })
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(mode)
}

fn parse_leader<'i, 't>(input: &mut Parser<'i, 't>) -> Result<'i, CssLeaderValue> {
    if let Ok(string) = input.try_parse(parse_content_string) {
        input.expect_exhausted().map_err(basic)?;
        return Ok(CssLeaderValue::String(string));
    }
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let mode = match ident.to_ascii_lowercase().as_str() {
        "dotted" => CssLeaderValue::Dotted,
        "solid" => CssLeaderValue::Solid,
        "space" => CssLeaderValue::Space,
        _ => return Err(unexpected_at(input.current_source_location())),
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(mode)
}
