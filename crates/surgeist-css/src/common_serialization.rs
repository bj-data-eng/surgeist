//! CSSOM common serialization idioms over decoded text and serialized items.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn writer(limits: CssSpecifiedValueSerializationLimits) -> Result<SpecifiedRuleWriter> {
    let mut writer = SpecifiedRuleWriter::new(limits);
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)?;
    Ok(writer)
}

/// Escapes arbitrary decoded text using CSSOM identifier serialization.
/// Empty text emits an empty identifier; NUL becomes U+FFFD.
/// This operation does not validate a property's identifier grammar.
pub fn serialize_css_identifier(value: &str) -> Result<String> {
    serialize_css_identifier_with_limits(value, CssSpecifiedValueSerializationLimits::default())
}

/// Escapes decoded identifier text atomically under explicit resource limits.
/// The primitive charges one input and one projection node, and actual output bytes.
pub fn serialize_css_identifier_with_limits(
    value: &str,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String> {
    let mut writer = writer(limits)?;
    writer.append_identifier(value)?;
    Ok(writer.css)
}

/// Quotes arbitrary decoded text using CSSOM double-quoted string serialization.
/// Empty text emits `""`; NUL becomes U+FFFD.
pub fn serialize_css_string(value: &str) -> Result<String> {
    serialize_css_string_with_limits(value, CssSpecifiedValueSerializationLimits::default())
}

/// Quotes decoded string text atomically under explicit resource limits.
/// The primitive charges one input and one projection node, and actual output bytes.
pub fn serialize_css_string_with_limits(
    value: &str,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String> {
    let mut writer = writer(limits)?;
    writer.append_string(value)?;
    Ok(writer.css)
}

/// Joins already serialized items in order with comma followed by one space.
/// Item bytes, including whitespace, empty items and NUL, are retained exactly.
/// An empty list emits empty text; items are not parsed or escaped again.
pub fn serialize_css_comma_separated_list(items: &[&str]) -> Result<String> {
    serialize_css_comma_separated_list_with_limits(
        items,
        CssSpecifiedValueSerializationLimits::default(),
    )
}

/// Joins serialized comma-list items atomically under cumulative resource limits.
/// The list and every item, including an empty item, charge one input and projection node.
pub fn serialize_css_comma_separated_list_with_limits(
    items: &[&str],
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String> {
    serialize_list(items, ", ", limits)
}

/// Joins already serialized items in order with one space.
/// Item bytes, including whitespace, empty items and NUL, are retained exactly.
/// An empty list emits empty text; items are not parsed or escaped again.
pub fn serialize_css_whitespace_separated_list(items: &[&str]) -> Result<String> {
    serialize_css_whitespace_separated_list_with_limits(
        items,
        CssSpecifiedValueSerializationLimits::default(),
    )
}

/// Joins serialized whitespace-list items atomically under cumulative resource limits.
/// The list and every item, including an empty item, charge one input and projection node.
pub fn serialize_css_whitespace_separated_list_with_limits(
    items: &[&str],
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String> {
    serialize_list(items, " ", limits)
}

fn serialize_list(
    items: &[&str],
    separator: &str,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String> {
    let mut writer = writer(limits)?;
    for (index, item) in items.iter().enumerate() {
        writer.context.charge_input(1)?;
        writer.context.charge_projection(1)?;
        if index != 0 {
            writer.append(separator)?;
        }
        writer.append(item)?;
    }
    Ok(writer.css)
}
