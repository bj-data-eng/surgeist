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

/// The shortest member count for a `<T>{1,4}` box-side value in
/// top/right/bottom/left order (or its logical equivalent).
///
/// Box 3 and Backgrounds 3 copy an omitted left from right, an omitted
/// bottom from top and an omitted right from top; CSSOM "serialize a CSS
/// value" step 2 then omits every member the copy reproduces. The comparisons
/// run in that dependency order, left/right first, and stop at the first
/// difference, so a fallible or charged `equal` sees one fixed sequence.
pub(crate) fn shortest_four_side_count<E>(
    mut equal: impl FnMut(usize, usize) -> std::result::Result<bool, E>,
) -> std::result::Result<usize, E> {
    Ok(if !equal(1, 3)? {
        4
    } else if !equal(0, 2)? {
        3
    } else if !equal(0, 1)? {
        2
    } else {
        1
    })
}

#[cfg(test)]
mod four_side_tests {
    use super::shortest_four_side_count;

    fn count(values: [&str; 4]) -> (usize, Vec<(usize, usize)>) {
        let mut calls = Vec::new();
        let count = shortest_four_side_count(|left, right| {
            calls.push((left, right));
            Ok::<_, ()>(values[left] == values[right])
        })
        .unwrap();
        (count, calls)
    }

    #[test]
    fn omits_each_member_its_copy_rule_reproduces() {
        // Expected counts follow the Box 3 margin copy rules directly.
        assert_eq!(count(["a", "a", "a", "a"]).0, 1);
        assert_eq!(count(["a", "b", "a", "b"]).0, 2);
        assert_eq!(count(["a", "b", "c", "b"]).0, 3);
        assert_eq!(count(["a", "b", "a", "c"]).0, 4);
        // Top equal to right and bottom does not shorten past a distinct left.
        assert_eq!(count(["a", "a", "a", "b"]).0, 4);
        assert_eq!(count(["a", "a", "b", "a"]).0, 3);
    }

    #[test]
    fn comparisons_run_left_right_first_and_stop_at_the_first_difference() {
        assert_eq!(count(["a", "b", "a", "c"]).1, [(1, 3)]);
        assert_eq!(count(["a", "b", "c", "b"]).1, [(1, 3), (0, 2)]);
        assert_eq!(count(["a", "a", "a", "a"]).1, [(1, 3), (0, 2), (0, 1)]);
        assert_eq!(
            shortest_four_side_count(|_, _| Err::<bool, _>("first")),
            Err("first")
        );
    }
}
