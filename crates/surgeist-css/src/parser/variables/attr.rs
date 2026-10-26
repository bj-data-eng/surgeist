//! Parse-time admission for the pinned Values 5 `attr()` notation.
//!
//! This checks only the function's own syntax. Attribute lookup, syntax matching,
//! fallback substitution, and taint belong to computed-value consumers. The
//! original components stay on the enclosing pending declaration.

use crate::utf16_value::Control;
use crate::{
    CssBlockKind, CssComponentValue, CssComponentValueError, CssComponentValueErrorKind,
    CssComponentValueRef as Component, CssValueOrigin, CssValueTokenRef as Token,
};

fn trivia(value: &CssComponentValue) -> bool {
    matches!(
        value.view(),
        Component::Comment(_) | Component::Token(Token::Whitespace(_))
    )
}

fn next_significant(
    items: &[CssComponentValue],
    start: usize,
    control: Option<&Control<'_>>,
) -> Option<usize> {
    for (index, item) in items.iter().enumerate().skip(start) {
        if super::step(control).is_err() {
            return None;
        }
        if !trivia(item) {
            return Some(index);
        }
    }
    None
}

fn next_non_comment(
    items: &[CssComponentValue],
    start: usize,
    control: Option<&Control<'_>>,
) -> Option<usize> {
    for (index, item) in items.iter().enumerate().skip(start) {
        if super::step(control).is_err() {
            return None;
        }
        if !matches!(item.view(), Component::Comment(_)) {
            return Some(index);
        }
    }
    None
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
    syntax_valid_with_control(items, decode_limit, None)
}
struct Decoded<'a> {
    source: Option<crate::CssSourceSnapshot>,
    values: Option<crate::CssComponentValues>,
    control: Option<&'a Control<'a>>,
    bytes: usize,
}
impl Drop for Decoded<'_> {
    fn drop(&mut self) {
        drop(self.values.take());
        drop(self.source.take());
        if let Some(control) = self.control {
            control.released(self.bytes);
        }
    }
}
fn syntax_valid_with_control(
    items: &[CssComponentValue],
    decode_limit: usize,
    control: Option<&Control<'_>>,
) -> Result<bool, CssComponentValueError> {
    let mut current: Option<Decoded<'_>> = None;
    let mut decoded = 0;
    let mut outer_string_origin: Option<CssValueOrigin> = None;
    loop {
        super::step(control)?;
        let values = current
            .as_ref()
            .and_then(|current| current.values.as_ref())
            .map_or(items, crate::CssComponentValues::items);
        let Some(first) = next_significant(values, 0, control) else {
            return Ok(false);
        };
        if let Component::Token(Token::String(value)) = values[first].view() {
            if next_significant(values, first + 1, control).is_some() {
                return Ok(false);
            }
            let original = outer_string_origin
                .get_or_insert_with(|| values[first].origin().clone())
                .clone();
            if decoded >= decode_limit {
                return Err(CssComponentValueError::new(
                    CssComponentValueErrorKind::NestingLimit,
                    original,
                ));
            }
            decoded += 1;
            let (source, parsed) = if let Some(control) = control {
                let source = crate::CssSourceSnapshot::try_from_decoded_utf8(value, control)
                    .map_err(|_| {
                        CssComponentValueError::new(
                            CssComponentValueErrorKind::CapacityOverflow,
                            original.clone(),
                        )
                    })?;
                let parsed = crate::component_values::parse_controlled_snapshot(
                    &source,
                    crate::CssComponentValueLimits::default(),
                    control,
                );
                (Some(source), parsed)
            } else {
                (None, crate::component_values::parse_component_values(value))
            };
            let values = match parsed {
                Ok(values) => values,
                Err(error) => {
                    let kind = error.kind;
                    let bytes = source
                        .as_ref()
                        .and_then(|source| source.payload_bytes(true));
                    drop(error);
                    drop(source);
                    if let Some(control) = control {
                        let Some(bytes) = bytes else {
                            unreachable!("admitted decoded source payload")
                        };
                        control.released(bytes);
                        control
                            .component_resource(kind)
                            .map_err(|_| CssComponentValueError::new(kind, original.clone()))?;
                    }
                    if matches!(
                        kind,
                        CssComponentValueErrorKind::NestingLimit
                            | CssComponentValueErrorKind::ComponentLimit
                            | CssComponentValueErrorKind::ByteLimit
                            | CssComponentValueErrorKind::CapacityOverflow
                    ) {
                        return Err(CssComponentValueError::new(kind, original));
                    }
                    return Ok(false);
                }
            };
            let bytes = if let Some(control) = control {
                let backing =
                    crate::component_values::retained_backing_bytes_with(&values, &mut || {
                        control.step()
                    })
                    .map_err(|_| {
                        CssComponentValueError::new(
                            CssComponentValueErrorKind::CapacityOverflow,
                            original.clone(),
                        )
                    })?;
                let source_bytes = source
                    .as_ref()
                    .and_then(|source| source.payload_bytes(true))
                    .ok_or_else(|| {
                        CssComponentValueError::new(
                            CssComponentValueErrorKind::CapacityOverflow,
                            original.clone(),
                        )
                    })?;
                backing.checked_add(source_bytes).ok_or_else(|| {
                    CssComponentValueError::new(
                        CssComponentValueErrorKind::CapacityOverflow,
                        original.clone(),
                    )
                })?
            } else {
                0
            };
            let next = Decoded {
                source,
                values: Some(values),
                control,
                bytes,
            };
            drop(current.replace(next));
            continue;
        }
        return Ok(syntax_tokens_valid(values, control));
    }
}

fn syntax_tokens_valid(items: &[CssComponentValue], control: Option<&Control<'_>>) -> bool {
    let Some(mut cursor) = next_significant(items, 0, control) else {
        return false;
    };
    if delim(&items[cursor], '*') {
        return next_significant(items, cursor + 1, control).is_none();
    }
    loop {
        let transform_list = if delim(&items[cursor], '<') {
            let Some(name_index) = next_non_comment(items, cursor + 1, control) else {
                return false;
            };
            let Some(name) = ident(&items[name_index]) else {
                return false;
            };
            if !type_name(name) {
                return false;
            }
            let Some(close) = next_non_comment(items, name_index + 1, control) else {
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
        if let Some(next) = next_non_comment(items, cursor, control)
            && (delim(&items[next], '#') || delim(&items[next], '+'))
        {
            if transform_list {
                return false;
            }
            cursor = next + 1;
        }
        let Some(separator) = next_significant(items, cursor, control) else {
            return true;
        };
        if !delim(&items[separator], '|') {
            return false;
        }
        let Some(next) = next_significant(items, separator + 1, control) else {
            return false;
        };
        cursor = next;
    }
}

fn fallback_valid(items: &[CssComponentValue], control: Option<&Control<'_>>) -> bool {
    let first = next_significant(items, 0, control);
    let mut significant = 0usize;
    for value in items {
        if super::step(control).is_err() {
            return false;
        }
        if !trivia(value) {
            significant += 1;
        }
    }
    if let Some(first) = first
        && let Component::Block(block) = items[first].view()
        && block.kind() == CssBlockKind::CurlyBracket
    {
        if significant != 1 {
            return false;
        }
        return declaration_value_valid(block.values().items(), true, control);
    }
    declaration_value_valid(items, false, control)
}

fn declaration_value_valid(
    items: &[CssComponentValue],
    wrapped: bool,
    control: Option<&Control<'_>>,
) -> bool {
    let mut non_comment = false;
    for value in items {
        if super::step(control).is_err() {
            return false;
        }
        non_comment |= !matches!(value.view(), Component::Comment(_));
        match value.view() {
            Component::Token(Token::Semicolon | Token::Delim('!')) => return false,
            Component::Token(Token::Comma) if !wrapped => return false,
            Component::Block(block) if block.kind() == CssBlockKind::CurlyBracket && !wrapped => {
                return false;
            }
            _ => {}
        }
    }
    non_comment
}

/// Whether the function arguments match the selected 2024 `attr()` grammar.
pub(super) fn valid(items: &[CssComponentValue]) -> Result<bool, CssComponentValueError> {
    valid_with_control(items, None)
}
pub(super) fn valid_with_control(
    items: &[CssComponentValue],
    control: Option<&Control<'_>>,
) -> Result<bool, CssComponentValueError> {
    let Some(mut name_index) = next_significant(items, 0, control) else {
        return Ok(false);
    };
    if ident(&items[name_index]).is_none() {
        return Ok(false);
    }
    name_index = next_significant(items, name_index + 1, control).unwrap_or(items.len());
    if name_index < items.len() && delim(&items[name_index], '|') {
        let Some(local) = next_significant(items, name_index + 1, control) else {
            return Ok(false);
        };
        if ident(&items[local]).is_none() {
            return Ok(false);
        }
        name_index = next_significant(items, local + 1, control).unwrap_or(items.len());
    }

    let mut comma = None;
    for (index, item) in items.iter().enumerate().skip(name_index) {
        super::step(control)?;
        if matches!(item.view(), Component::Token(Token::Comma)) {
            comma = Some(index);
            break;
        }
    }
    let header_end = comma.unwrap_or(items.len());
    if name_index < header_end
        && !match control {
            Some(_) => syntax_valid_with_control(
                &items[name_index..header_end],
                crate::STRUCTURAL_NESTING_LIMIT as usize,
                control,
            )?,
            None => syntax_valid(
                &items[name_index..header_end],
                crate::STRUCTURAL_NESTING_LIMIT as usize,
            )?,
        }
    {
        return Ok(false);
    }
    match comma {
        Some(index) => Ok(fallback_valid(&items[index + 1..], control)),
        None => Ok(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_syntax_decode_budget_is_cumulative_and_reports_outer_string() {
        let first =
            crate::component_values::parse_component_values("foo \"\\\"<length>\\\"\"").unwrap();
        let outer = first.items()[2].origin().clone();
        let error = syntax_valid(&first.items()[2..], 1).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
        assert_eq!(error.origin(), &outer);
        assert!(syntax_valid(&first.items()[2..], 2).unwrap());
    }

    #[test]
    fn deeply_nested_decoded_syntax_reports_original_token() {
        let decoded = format!("{}x{}", "(".repeat(257), ")".repeat(257));
        let arguments =
            crate::component_values::parse_component_values(&format!("foo \"{decoded}\"")).unwrap();
        let origin = arguments.items()[2].origin().clone();
        let error = valid(arguments.items()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::NestingLimit);
        assert_eq!(error.origin(), &origin);

        let outer = crate::component_values::parse_component_values(&format!("\"{decoded}\""))
            .unwrap()
            .items()[0]
            .clone();
        let children = crate::CssComponentValues::try_new(
            vec![
                CssComponentValue::try_ident("foo").unwrap(),
                CssComponentValue::try_token(" ").unwrap(),
                outer.clone(),
            ],
            Default::default(),
        )
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
