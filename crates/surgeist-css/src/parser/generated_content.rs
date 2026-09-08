use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::background::parse_url;
use super::values::parse_integer;
use crate::error::{Error, basic, unsupported_value, unsupported_value_at};
use crate::syntax::*;
use crate::validation::unsupported_keyword_reason;

pub(super) fn parse_quotes<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssQuotes, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned)
        && ident.eq_ignore_ascii_case("none")
    {
        return if input.is_exhausted() {
            Ok(CssQuotes::None)
        } else {
            Err(unsupported_value(
                input,
                None,
                "`none` cannot be combined with quotation pairs",
            ))
        };
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
        .ok_or_else(|| unsupported_value(input, None, "quotes requires `none` or string pairs"))
}

pub(super) fn parse_content<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContent, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        match_ignore_ascii_case! { &ident,
            "normal" if input.is_exhausted() => return Ok(CssContent::Normal),
            "normal" => return Err(unsupported_value(input, None, "`normal` cannot be combined with content items")),
            "none" if input.is_exhausted() => return Ok(CssContent::None),
            "none" => return Err(unsupported_value(input, None, "`none` cannot be combined with content items")),
            _ => input.reset(&state),
        };
    }

    let mut items = Vec::new();
    while !input.is_exhausted() {
        items.push(parse_content_item(input)?);
    }
    CssContentList::try_new(items)
        .map(CssContent::Items)
        .ok_or_else(|| unsupported_value(input, None, "content item list is empty"))
}

pub(super) fn parse_list_style_type<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssListStyleType, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssListStyleType::None);
    }
    if let Ok(value) = input.try_parse(parse_content_string) {
        return Ok(CssListStyleType::String(value));
    }
    parse_counter_style(input).map(CssListStyleType::CounterStyle)
}

pub(super) fn parse_list_style_position<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssListStylePosition, ParseError<'i, Error>> {
    let ident = input.expect_ident_cloned().map_err(basic)?;
    match_ignore_ascii_case! { &ident,
        "inside" => Ok(CssListStylePosition::Inside),
        "outside" => Ok(CssListStylePosition::Outside),
        _ => Err(unsupported_value(input, None, unsupported_keyword_reason("list-style-position", ident.as_ref()))),
    }
}

pub(super) fn parse_list_style_image<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssListStyleImage, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(CssListStyleImage::None);
    }
    parse_url(input).map(CssListStyleImage::Url).map_err(|_| {
        unsupported_value(
            input,
            None,
            "list-style-image only supports `none` or url(...)",
        )
    })
}

pub(super) fn parse_list_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssListStyle, ParseError<'i, Error>> {
    let mut style_type = None;
    let mut position = None;
    let mut image = None;
    let mut has_ambiguous_none = false;

    while !input.is_exhausted() {
        let component = parse_list_style_component(input)?;
        match component {
            ListStyleComponent::None => {
                if has_ambiguous_none {
                    return Err(unsupported_value(
                        input,
                        None,
                        "list-style has duplicate `none` components",
                    ));
                }
                has_ambiguous_none = true;
            }
            ListStyleComponent::Type(value) => {
                if style_type.replace(value).is_some() {
                    return Err(unsupported_value(
                        input,
                        None,
                        "list-style has duplicate type components",
                    ));
                }
            }
            ListStyleComponent::Position(value) => {
                if position.replace(value).is_some() {
                    return Err(unsupported_value(
                        input,
                        None,
                        "list-style has duplicate position components",
                    ));
                }
            }
            ListStyleComponent::Image(value) => {
                if image.replace(value).is_some() {
                    return Err(unsupported_value(
                        input,
                        None,
                        "list-style has duplicate image components",
                    ));
                }
            }
        }
    }

    if has_ambiguous_none {
        match (style_type.is_some(), image.is_some()) {
            (false, false) => {
                style_type = Some(CssListStyleType::None);
                image = Some(CssListStyleImage::None);
            }
            (false, true) => {
                style_type = Some(CssListStyleType::None);
            }
            (true, false) => {
                image = Some(CssListStyleImage::None);
            }
            (true, true) => {
                return Err(unsupported_value(
                    input,
                    None,
                    "`none` duplicates a list-style type or image component",
                ));
            }
        }
    }

    CssListStyle::try_new(style_type, position, image)
        .ok_or_else(|| unsupported_value(input, None, "list-style shorthand is empty"))
}

pub(super) fn parse_counter_changes<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCounterChanges, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        if input.is_exhausted() {
            return Ok(CssCounterChanges::None);
        }
        return Err(unsupported_value(
            input,
            None,
            "`none` cannot be combined with counter changes",
        ));
    }

    let mut changes = Vec::new();
    while !input.is_exhausted() {
        let name = parse_counter_name(input)?;
        let value = input
            .try_parse(|input| parse_integer(input, "counter value"))
            .ok();
        changes.push(CssCounterChange::new(name, value));
    }

    CssCounterChanges::try_changes(changes)
        .ok_or_else(|| unsupported_value(input, None, "counter change list is empty"))
}

enum ListStyleComponent {
    None,
    Type(CssListStyleType),
    Position(CssListStylePosition),
    Image(CssListStyleImage),
}

fn parse_list_style_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<ListStyleComponent, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("none"))
        .is_ok()
    {
        return Ok(ListStyleComponent::None);
    }

    if let Ok(position) = input.try_parse(parse_list_style_position) {
        return Ok(ListStyleComponent::Position(position));
    }

    if let Ok(image) = input.try_parse(parse_list_style_image) {
        return Ok(ListStyleComponent::Image(image));
    }

    parse_list_style_type(input).map(ListStyleComponent::Type)
}

fn parse_content_item<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContentItem, ParseError<'i, Error>> {
    if let Ok(value) = input.try_parse(parse_content_string) {
        return Ok(CssContentItem::String(value));
    }
    if let Ok(url) = input.try_parse(parse_url) {
        return Ok(CssContentItem::Url(url));
    }
    if let Ok(ident) = input.try_parse(Parser::expect_ident_cloned) {
        return match_ignore_ascii_case! { &ident,
            "open-quote" => Ok(CssContentItem::OpenQuote),
            "close-quote" => Ok(CssContentItem::CloseQuote),
            "no-open-quote" => Ok(CssContentItem::NoOpenQuote),
            "no-close-quote" => Ok(CssContentItem::NoCloseQuote),
            _ => Err(unsupported_value(input, None, unsupported_keyword_reason("content", ident.as_ref()))),
        };
    }

    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Function(name) if name.eq_ignore_ascii_case("counter") => input
            .parse_nested_block(parse_counter_function)
            .map(CssContentItem::Counter),
        Token::Function(name) if name.eq_ignore_ascii_case("counters") => input
            .parse_nested_block(parse_counters_function)
            .map(CssContentItem::Counters),
        Token::Function(name) if name.eq_ignore_ascii_case("attr") => input
            .parse_nested_block(parse_attr_function)
            .map(CssContentItem::Attr),
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported content function `{name}()`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn parse_counter_function<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCounterFunction, ParseError<'i, Error>> {
    let name = parse_counter_name(input)?;
    let style = if input.try_parse(Parser::expect_comma).is_ok() {
        Some(parse_counter_style(input)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssCounterFunction::new(name, style))
}

fn parse_counters_function<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCountersFunction, ParseError<'i, Error>> {
    let name = parse_counter_name(input)?;
    input.expect_comma().map_err(basic)?;
    let separator = parse_content_string(input)?;
    let style = if input.try_parse(Parser::expect_comma).is_ok() {
        Some(parse_counter_style(input)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssCountersFunction::new(name, separator, style))
}

fn parse_attr_function<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAttributeName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = input.expect_ident_cloned().map_err(basic)?;
    input.expect_exhausted().map_err(basic)?;
    CssAttributeName::try_new(name.to_string()).ok_or_else(|| {
        unsupported_value_at(
            location,
            None,
            format!("unsupported attr() attribute name `{name}`"),
        )
    })
}

fn parse_content_string<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssContentString, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = input.expect_string_cloned().map_err(basic)?;
    CssContentString::try_new(value.to_string())
        .ok_or_else(|| unsupported_value_at(location, None, "content string contains null"))
}

fn parse_counter_name<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCounterName, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let name = input
        .expect_ident_cloned()
        .map_err(|_| unsupported_value(input, None, "expected counter name"))?;
    CssCounterName::try_new(name.to_string()).ok_or_else(|| {
        unsupported_value_at(location, None, format!("unsupported counter name `{name}`"))
    })
}

fn parse_counter_style<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssCounterStyle, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    if let Some(style) = parse_builtin_counter_style(ident.as_ref()) {
        return Ok(CssCounterStyle::BuiltIn(style));
    }
    CssCounterStyleName::try_new(ident.to_string())
        .map(CssCounterStyle::Named)
        .ok_or_else(|| {
            unsupported_value_at(
                location,
                None,
                format!("unsupported counter style `{ident}`"),
            )
        })
}

fn parse_builtin_counter_style(value: &str) -> Option<CssBuiltInCounterStyle> {
    match value.to_ascii_lowercase().as_str() {
        "disc" => Some(CssBuiltInCounterStyle::Disc),
        "circle" => Some(CssBuiltInCounterStyle::Circle),
        "square" => Some(CssBuiltInCounterStyle::Square),
        "decimal" => Some(CssBuiltInCounterStyle::Decimal),
        "decimal-leading-zero" => Some(CssBuiltInCounterStyle::DecimalLeadingZero),
        "lower-alpha" => Some(CssBuiltInCounterStyle::LowerAlpha),
        "upper-alpha" => Some(CssBuiltInCounterStyle::UpperAlpha),
        "lower-latin" => Some(CssBuiltInCounterStyle::LowerLatin),
        "upper-latin" => Some(CssBuiltInCounterStyle::UpperLatin),
        "lower-roman" => Some(CssBuiltInCounterStyle::LowerRoman),
        "upper-roman" => Some(CssBuiltInCounterStyle::UpperRoman),
        _ => None,
    }
}
