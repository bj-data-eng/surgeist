//! Parse-time admission for the pinned Values 5 `attr()` notation.
//!
//! This checks only the function's own syntax. Attribute lookup, syntax matching,
//! fallback substitution, and taint belong to computed-value consumers. The
//! original components stay on the enclosing pending declaration.

use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueRef as Component, CssValueOrigin, CssValueTokenRef as Token,
    parse_component_values,
};

fn trivia(value: &CssComponentValue) -> bool {
    matches!(
        value.view(),
        Component::Comment(_) | Component::Token(Token::Whitespace(_))
    )
}

fn next_significant(items: &[CssComponentValue], start: usize) -> Option<usize> {
    (start..items.len()).find(|index| !trivia(&items[*index]))
}

fn next_non_comment(items: &[CssComponentValue], start: usize) -> Option<usize> {
    (start..items.len()).find(|index| !matches!(items[*index].view(), Component::Comment(_)))
}

fn ident(value: &CssComponentValue) -> Option<&str> {
    match value.view() {
        Component::Token(Token::Ident(name)) => Some(name),
        _ => None,
    }
}

fn delim(value: &CssComponentValue, expected: char) -> bool {
    matches!(value.view(), Component::Token(Token::Delim(actual)) if actual == expected)
}

fn type_name(name: &str) -> bool {
    [
        "angle",
        "color",
        "custom-ident",
        "image",
        "integer",
        "length",
        "length-percentage",
        "number",
        "percentage",
        "resolution",
        "string",
        "time",
        "url",
        "transform-function",
        "transform-list",
    ]
    .iter()
    .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

/// Checks a complete Values 5 `<syntax>` after optional attribute-name tokens.
/// Decoded syntax strings are reparsed iteratively, so a chain of quoted syntax
/// strings cannot consume the Rust call stack. The separate decode count closes
/// a resource gap that component block/function depth alone does not cover.
fn syntax_valid(
    items: &[CssComponentValue],
    decode_limit: usize,
) -> Result<bool, CssComponentValueError> {
    let mut current = items.to_vec();
    let mut decoded = 0;
    let mut outer_string_origin: Option<CssValueOrigin> = None;
    loop {
        let Some(first) = next_significant(&current, 0) else {
            return Ok(false);
        };
        if let Component::Token(Token::String(value)) = current[first].view() {
            if next_significant(&current, first + 1).is_some() {
                return Ok(false);
            }
            let original = outer_string_origin
                .get_or_insert_with(|| current[first].origin().clone())
                .clone();
            if decoded >= decode_limit {
                return Err(CssComponentValueError::new(
                    CssComponentValueErrorKind::NestingLimit,
                    original,
                ));
            }
            decoded += 1;
            current = match parse_component_values(value) {
                Ok(values) => values.items().to_vec(),
                Err(error)
                    if matches!(
                        error.kind(),
                        CssComponentValueErrorKind::NestingLimit
                            | CssComponentValueErrorKind::ComponentLimit
                            | CssComponentValueErrorKind::ByteLimit
                            | CssComponentValueErrorKind::CapacityOverflow
                    ) =>
                {
                    return Err(CssComponentValueError::new(error.kind(), original));
                }
                Err(_) => return Ok(false),
            };
            continue;
        }
        return Ok(syntax_tokens_valid(&current));
    }
}

fn syntax_tokens_valid(items: &[CssComponentValue]) -> bool {
    let Some(mut cursor) = next_significant(items, 0) else {
        return false;
    };
    if delim(&items[cursor], '*') {
        return next_significant(items, cursor + 1).is_none();
    }
    loop {
        let transform_list = if delim(&items[cursor], '<') {
            let Some(name_index) = next_non_comment(items, cursor + 1) else {
                return false;
            };
            let Some(name) = ident(&items[name_index]) else {
                return false;
            };
            if !type_name(name) {
                return false;
            }
            let Some(close) = next_non_comment(items, name_index + 1) else {
                return false;
            };
            if !delim(&items[close], '>') {
                return false;
            }
            cursor = close + 1;
            name.eq_ignore_ascii_case("transform-list")
        } else if ident(&items[cursor]).is_some() {
            cursor += 1;
            false
        } else {
            return false;
        };

        // Comments may intervene, but CSS whitespace before a multiplier is invalid.
        if let Some(next) = next_non_comment(items, cursor)
            && (delim(&items[next], '#') || delim(&items[next], '+'))
        {
            if transform_list {
                return false;
            }
            cursor = next + 1;
        }
        let Some(separator) = next_significant(items, cursor) else {
            return true;
        };
        if !delim(&items[separator], '|') {
            return false;
        }
        let Some(next) = next_significant(items, separator + 1) else {
            return false;
        };
        cursor = next;
    }
}

fn fallback_valid(items: &[CssComponentValue]) -> bool {
    let first = next_significant(items, 0);
    let significant: Vec<_> = items.iter().filter(|value| !trivia(value)).collect();
    if let Some(first) = first
        && let Component::Block(block) = items[first].view()
        && block.kind() == CssBlockKind::CurlyBracket
    {
        if significant.len() != 1 {
            return false;
        }
        return declaration_value_valid(block.values().items(), true);
    }
    declaration_value_valid(items, false)
}

fn declaration_value_valid(items: &[CssComponentValue], wrapped: bool) -> bool {
    if !items
        .iter()
        .any(|value| !matches!(value.view(), Component::Comment(_)))
    {
        return false;
    }
    items.iter().all(|value| match value.view() {
        Component::Token(Token::Semicolon | Token::Delim('!')) => false,
        Component::Token(Token::Comma) if !wrapped => false,
        Component::Block(block) if block.kind() == CssBlockKind::CurlyBracket && !wrapped => false,
        _ => true,
    })
}

/// Whether the function arguments match the selected 2024 `attr()` grammar.
pub(super) fn valid(items: &[CssComponentValue]) -> Result<bool, CssComponentValueError> {
    let Some(mut name_index) = next_significant(items, 0) else {
        return Ok(false);
    };
    if ident(&items[name_index]).is_none() {
        return Ok(false);
    }
    name_index = next_significant(items, name_index + 1).unwrap_or(items.len());
    if name_index < items.len() && delim(&items[name_index], '|') {
        let Some(local) = next_significant(items, name_index + 1) else {
            return Ok(false);
        };
        if ident(&items[local]).is_none() {
            return Ok(false);
        }
        name_index = next_significant(items, local + 1).unwrap_or(items.len());
    }

    let comma = (name_index..items.len())
        .find(|index| matches!(items[*index].view(), Component::Token(Token::Comma)));
    let header_end = comma.unwrap_or(items.len());
    if name_index < header_end
        && !syntax_valid(
            &items[name_index..header_end],
            crate::STRUCTURAL_NESTING_LIMIT as usize,
        )?
    {
        return Ok(false);
    }
    match comma {
        Some(index) => Ok(fallback_valid(&items[index + 1..])),
        None => Ok(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_syntax_decode_budget_is_cumulative_and_reports_outer_string() {
        let first = crate::parse_component_values("foo \"\\\"<length>\\\"\"").unwrap();
        let outer = first.items()[2].origin().clone();
        let error = syntax_valid(&first.items()[2..], 1).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
        assert_eq!(error.origin(), &outer);
        assert!(syntax_valid(&first.items()[2..], 2).unwrap());
    }

    #[test]
    fn deeply_nested_decoded_syntax_reports_original_token() {
        let decoded = format!("{}x{}", "(".repeat(257), ")".repeat(257));
        let arguments = crate::parse_component_values(&format!("foo \"{decoded}\"")).unwrap();
        let origin = arguments.items()[2].origin().clone();
        let error = valid(arguments.items()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
        assert_eq!(error.origin(), &origin);

        let outer = crate::parse_component_values(&format!("\"{decoded}\""))
            .unwrap()
            .items()[0]
            .clone();
        let children = crate::CssComponentValues::try_new(vec![
            CssComponentValue::try_ident("foo").unwrap(),
            CssComponentValue::try_token(" ").unwrap(),
            outer.clone(),
        ])
        .unwrap();
        assert_eq!(
            valid(children.items()).unwrap_err().origin(),
            outer.origin()
        );
        let function = CssComponentValue::try_function("attr", children).unwrap();
        let error = super::super::summarize(&[function], None, true, true)
            .err()
            .expect("nested syntax must exceed the component depth limit");
        assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
        assert_eq!(error.origin(), outer.origin());
    }
}
