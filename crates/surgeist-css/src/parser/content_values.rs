//! Grammar admission for the selected CSS Generated Content 3 `content` value.

use super::syntax_types::Stream;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::parser::syntax_types::SyntaxError;
use surgeist_css_parser::Token;

use super::background::parse_image;
use super::generated_content::parse_content_string;
use super::url::parse_url;
use crate::values::common::CssIdent;
use crate::values::content::content_values::*;

type Result<'i, T> = std::result::Result<T, SyntaxError<'i, Error>>;

pub(super) fn parse_content_value<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssContentValue> {
    let start = input.mark();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        let keyword = if ident.eq_ignore_ascii_case("normal") {
            Some(CssContentValue::Normal)
        } else if ident.eq_ignore_ascii_case("none") {
            Some(CssContentValue::None)
        } else {
            None
        };
        if let Some(keyword) = keyword {
            return if input.only_whitespace_remains() {
                Ok(keyword)
            } else {
                Err(unexpected_at(input.site()))
            };
        }
    }
    input.restore(start);

    let mut items = Vec::new();
    while !input.only_whitespace_remains() {
        if input.attempt(|input| input.consume_delim('/')).is_ok() {
            let alternative = parse_alternative(input, numeric)?;
            input.require_empty().map_err(basic)?;
            return CssGeneratedContent::try_new(items, Some(alternative))
                .map(CssContentValue::Generated)
                .ok_or_else(|| unexpected_at(input.site()));
        }
        items.push(parse_item(input, numeric)?);
    }
    CssGeneratedContent::try_new(items, None)
        .map(CssContentValue::Generated)
        .ok_or_else(|| unexpected_at(input.site()))
}

/// Admits the full shared nonempty list without `content` keywords/slash branches.
pub(super) fn parse_content_list<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssContentList> {
    let mut items = Vec::new();
    while !input.only_whitespace_remains() {
        items.push(parse_item(input, numeric)?);
    }
    CssContentList::try_new(items).ok_or_else(|| unexpected_at(input.site()))
}

fn parse_alternative<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssContentAlternative> {
    let mut items = Vec::new();
    while !input.only_whitespace_remains() {
        if let Ok(string) = input.attempt(parse_content_string) {
            items.push(CssContentAlternativeItem::String(string));
            continue;
        }
        let location = input.site();
        match input.consume_after_whitespace().map_err(basic)? {
            Token::Function(name) if name.eq_ignore_ascii_case("counter") => {
                items.push(CssContentAlternativeItem::Counter(
                    input.within_block(|input| parse_counter(input, numeric))?,
                ))
            }
            Token::Function(name) if name.eq_ignore_ascii_case("counters") => {
                items.push(CssContentAlternativeItem::Counters(
                    input.within_block(|input| parse_counters(input, numeric))?,
                ))
            }
            token => return Err(location.unexpected::<Error>(token)),
        }
    }
    CssContentAlternative::try_new(items).ok_or_else(|| unexpected_at(input.site()))
}

fn parse_item<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssContentValueItem> {
    if let Ok(string) = input.attempt(parse_content_string) {
        return Ok(CssContentValueItem::String(string));
    }
    // A URL commits its image (Values 5 request URL modifiers report inside
    // the function).
    if super::ui::starts_url(input) {
        return parse_image(input, numeric).map(CssContentValueItem::Image);
    }
    match input.attempt(|input| parse_image(input, numeric)) {
        Ok(image) => return Ok(CssContentValueItem::Image(image)),
        Err(error) if crate::diagnostic::error::property::is_resource_parse_error(&error) => {
            return Err(error);
        }
        Err(_) => {}
    }
    input.discard_whitespace();
    let ident_location = input.site();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        // An unadmitted keyword is itself responsible (Content 3).
        return match ident.to_ascii_lowercase().as_str() {
            "contents" => Ok(CssContentValueItem::Contents),
            "open-quote" => Ok(CssContentValueItem::OpenQuote),
            "close-quote" => Ok(CssContentValueItem::CloseQuote),
            "no-open-quote" => Ok(CssContentValueItem::NoOpenQuote),
            "no-close-quote" => Ok(CssContentValueItem::NoCloseQuote),
            _ => Err(unexpected_at(ident_location)),
        };
    }
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Function(name) if name.eq_ignore_ascii_case("counter") => input
            .within_block(|input| parse_counter(input, numeric))
            .map(CssContentValueItem::Counter),
        Token::Function(name) if name.eq_ignore_ascii_case("counters") => input
            .within_block(|input| parse_counters(input, numeric))
            .map(CssContentValueItem::Counters),
        Token::Function(name) if name.eq_ignore_ascii_case("leader") => input
            .within_block(parse_leader)
            .map(CssContentValueItem::Leader),
        Token::Function(name) if name.eq_ignore_ascii_case("target-counter") => input
            .within_block(|input| parse_target_counter(input, numeric))
            .map(CssContentValueItem::TargetCounter),
        Token::Function(name) if name.eq_ignore_ascii_case("target-counters") => input
            .within_block(|input| parse_target_counters(input, numeric))
            .map(CssContentValueItem::TargetCounters),
        Token::Function(name) if name.eq_ignore_ascii_case("target-text") => input
            .within_block(|input| parse_target_text(input, numeric))
            .map(CssContentValueItem::TargetText),
        Token::Function(name) if name.eq_ignore_ascii_case("string") => input
            .within_block(parse_named_string)
            .map(CssContentValueItem::NamedString),
        Token::Function(name) if name.eq_ignore_ascii_case("content") => input
            .within_block(parse_content_reference)
            .map(CssContentValueItem::Content),
        token => Err(location.unexpected::<Error>(token)),
    }
}

// Values 4 §4.2: an excluded <custom-ident> is itself responsible, not the
// token after it.
pub(super) fn parse_generic_name<'i>(input: &mut Stream<'i>) -> Result<'i, CssContentName> {
    input.discard_whitespace();
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    let checked = CssIdent::try_new(ident.to_string()).map_err(|_| unexpected_at(location))?;
    CssContentName::try_new(checked).ok_or_else(|| unexpected_at(location))
}

pub(super) fn parse_counter_name<'i>(input: &mut Stream<'i>) -> Result<'i, CssContentCounterName> {
    input.discard_whitespace();
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    let checked = CssIdent::try_new(ident.to_string()).map_err(|_| unexpected_at(location))?;
    CssContentCounterName::try_new(checked).ok_or_else(|| unexpected_at(location))
}

fn parse_counter<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssContentCounter> {
    let name = parse_counter_name(input)?;
    let style = if input.attempt(Stream::consume_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.require_empty().map_err(basic)?;
    Ok(CssContentCounter::new(name, style))
}

fn parse_counters<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssContentCounters> {
    let name = parse_counter_name(input)?;
    input.consume_comma().map_err(basic)?;
    let separator = parse_content_string(input)?;
    let style = if input.attempt(Stream::consume_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.require_empty().map_err(basic)?;
    Ok(CssContentCounters::new(name, separator, style))
}

pub(super) fn parse_style<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssCounterStyleValue> {
    let keyword_location = input.site();
    if let Ok(ident) = input.attempt(Stream::consume_ident) {
        let checked =
            CssIdent::try_new(ident.to_string()).map_err(|_| unexpected_at(keyword_location))?;
        return CssCounterStyleValue::try_named(checked)
            .ok_or_else(|| unexpected_at(keyword_location));
    }
    let location = input.site();
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Function(name) if name.eq_ignore_ascii_case("symbols") => input
            .within_block(|input| parse_symbols(input, numeric))
            .map(CssCounterStyleValue::Symbols),
        token => Err(location.unexpected::<Error>(token)),
    }
}

fn parse_symbols<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssSymbolsStyleValue> {
    let start = input.mark();
    let system = if let Ok(ident) = input.attempt(Stream::consume_ident) {
        match ident.to_ascii_lowercase().as_str() {
            "cyclic" => Some(CssSymbolsSystem::Cyclic),
            "numeric" => Some(CssSymbolsSystem::Numeric),
            "alphabetic" => Some(CssSymbolsSystem::Alphabetic),
            "symbolic" => Some(CssSymbolsSystem::Symbolic),
            "fixed" => Some(CssSymbolsSystem::Fixed),
            _ => {
                input.restore(start);
                None
            }
        }
    } else {
        None
    };
    let mut symbols = Vec::new();
    while !input.only_whitespace_remains() {
        if let Ok(string) = input.attempt(parse_content_string) {
            symbols.push(CssCounterSymbolValue::String(string));
            continue;
        }
        let image = parse_image(input, numeric)?;
        symbols.push(CssCounterSymbolValue::Image(image));
    }
    CssSymbolsStyleValue::try_new(system, symbols).ok_or_else(|| unexpected_at(input.site()))
}

fn parse_target<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssContentTarget> {
    if let Ok(string) = input.attempt(parse_content_string) {
        return Ok(CssContentTarget::String(string));
    }
    parse_url(input, numeric).map(CssContentTarget::Url)
}

fn parse_target_counter<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssTargetCounter> {
    let target = parse_target(input, numeric)?;
    input.consume_comma().map_err(basic)?;
    let name = parse_generic_name(input)?;
    let style = if input.attempt(Stream::consume_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.require_empty().map_err(basic)?;
    Ok(CssTargetCounter::new(target, name, style))
}

fn parse_target_counters<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssTargetCounters> {
    let target = parse_target(input, numeric)?;
    input.consume_comma().map_err(basic)?;
    let name = parse_generic_name(input)?;
    input.consume_comma().map_err(basic)?;
    let separator = parse_content_string(input)?;
    let style = if input.attempt(Stream::consume_comma).is_ok() {
        Some(parse_style(input, numeric)?)
    } else {
        None
    };
    input.require_empty().map_err(basic)?;
    Ok(CssTargetCounters::new(target, name, separator, style))
}

fn parse_target_text<'i>(
    input: &mut Stream<'i>,
    numeric: &crate::numeric::expression::NumericInputContext<'_>,
) -> Result<'i, CssTargetText> {
    let target = parse_target(input, numeric)?;
    let mode = if input.attempt(Stream::consume_comma).is_ok() {
        let location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        Some(match ident.to_ascii_lowercase().as_str() {
            "content" => CssTargetTextMode::Content,
            "before" => CssTargetTextMode::Before,
            "after" => CssTargetTextMode::After,
            "first-letter" => CssTargetTextMode::FirstLetter,
            // The unknown keyword itself cannot continue the function.
            _ => return Err(unexpected_at(location)),
        })
    } else {
        None
    };
    input.require_empty().map_err(basic)?;
    Ok(CssTargetText::new(target, mode))
}

fn parse_named_string<'i>(input: &mut Stream<'i>) -> Result<'i, CssNamedString> {
    let name = parse_generic_name(input)?;
    let mode = if input.attempt(Stream::consume_comma).is_ok() {
        let location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        Some(match ident.to_ascii_lowercase().as_str() {
            "first" => CssNamedStringMode::First,
            "start" => CssNamedStringMode::Start,
            "last" => CssNamedStringMode::Last,
            "first-except" => CssNamedStringMode::FirstExcept,
            // The unknown keyword itself cannot continue the function.
            _ => return Err(unexpected_at(location)),
        })
    } else {
        None
    };
    input.require_empty().map_err(basic)?;
    Ok(CssNamedString::new(name, mode))
}

fn parse_content_reference<'i>(
    input: &mut Stream<'i>,
) -> Result<'i, Option<CssContentReferenceMode>> {
    let mode = if input.only_whitespace_remains() {
        None
    } else {
        let location = input.site();
        let ident = input.consume_ident().map_err(basic)?;
        Some(match ident.to_ascii_lowercase().as_str() {
            "text" => CssContentReferenceMode::Text,
            "before" => CssContentReferenceMode::Before,
            "after" => CssContentReferenceMode::After,
            "first-letter" => CssContentReferenceMode::FirstLetter,
            "marker" => CssContentReferenceMode::Marker,
            // The unknown keyword itself cannot continue the function.
            _ => return Err(unexpected_at(location)),
        })
    };
    input.require_empty().map_err(basic)?;
    Ok(mode)
}

fn parse_leader<'i>(input: &mut Stream<'i>) -> Result<'i, CssLeaderValue> {
    if let Ok(string) = input.attempt(parse_content_string) {
        input.require_empty().map_err(basic)?;
        return Ok(CssLeaderValue::String(string));
    }
    let location = input.site();
    let ident = input.consume_ident().map_err(basic)?;
    let mode = match ident.to_ascii_lowercase().as_str() {
        "dotted" => CssLeaderValue::Dotted,
        "solid" => CssLeaderValue::Solid,
        "space" => CssLeaderValue::Space,
        _ => return Err(unexpected_at(location)),
    };
    input.require_empty().map_err(basic)?;
    Ok(mode)
}
