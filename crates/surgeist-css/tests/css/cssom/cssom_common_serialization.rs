//! Independent expected output from CSSOM §2.1, including empty/NUL idioms
//! that intentionally do not cross strict property identifier/list grammars.
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#common-serializing-idioms

use crate::css::support::serialization_limits::LimitLadder;
use surgeist_css::serialization::{
    CssSerializationError, CssSerializationLimits, serialize_css_comma_separated_list,
    serialize_css_identifier, serialize_css_string, serialize_css_whitespace_separated_list,
};
use surgeist_css::values::common::CssIdent;

type TextSerializer = fn(&str, CssSerializationLimits) -> Result<String, CssSerializationError>;
type ListSerializer = fn(&[&str], CssSerializationLimits) -> Result<String, CssSerializationError>;

fn assert_text(source: &str, expected: &str, serialize: TextSerializer) {
    let original = source.to_owned();
    assert_eq!(
        serialize(source, CssSerializationLimits::default()).unwrap(),
        expected
    );
    // One node carries the complete text.
    LimitLadder::new(expected, 1, 1).assert_with(
        |limits| serialize(source, limits),
        |_, _, _| {
            assert_eq!(
                serialize(source, CssSerializationLimits::default()).unwrap(),
                expected
            )
        },
    );
    assert_eq!(source, original);
    assert_eq!(
        serialize(source, CssSerializationLimits::default()).unwrap(),
        expected
    );
}

#[test]
fn identifiers_escape_each_cssom_character_branch() {
    for (source, expected) in [
        ("", ""),
        ("\0", "\u{fffd}"),
        ("\0z", "\u{fffd}z"),
        ("\u{1}", "\\1 "),
        ("\t", "\\9 "),
        ("\n", "\\a "),
        ("\r", "\\d "),
        ("\u{1f}", "\\1f "),
        ("\u{7f}", "\\7f "),
        ("0abc", "\\30 abc"),
        ("9", "\\39 "),
        ("-1abc", "-\\31 abc"),
        ("-9", "-\\39 "),
        ("-", "\\-"),
        ("--1", "--1"),
        ("a1", "a1"),
        ("_0", "_0"),
        ("a-Z_09", "a-Z_09"),
        ("\u{80}é😀", "\u{80}é😀"),
        ("a b", "a\\ b"),
        (".", "\\."),
        ("#", "\\#"),
        ("'", "\\'"),
        ("\"", "\\\""),
        ("\\", "\\\\"),
        ("\u{1}a", "\\1 a"),
        ("\0\u{1f}9", "\u{fffd}\\1f 9"),
    ] {
        assert_text(source, expected, serialize_css_identifier);
    }
}

#[test]
fn strings_quote_and_escape_each_cssom_character_branch() {
    for (source, expected) in [
        ("", "\"\""),
        ("\0", "\"\u{fffd}\""),
        ("\u{1}", "\"\\1 \""),
        ("\t", "\"\\9 \""),
        ("\n", "\"\\a \""),
        ("\r", "\"\\d \""),
        ("\u{1f}", "\"\\1f \""),
        ("\u{7f}", "\"\\7f \""),
        ("\"", "\"\\\"\""),
        ("\\", "\"\\\\\""),
        ("'", "\"'\""),
        ("a b-09_", "\"a b-09_\""),
        ("é😀", "\"é😀\""),
        ("\u{80}", "\"\u{80}\""),
        ("\u{1}a", "\"\\1 a\""),
        ("\0\u{1f}9", "\"\u{fffd}\\1f 9\""),
    ] {
        assert_text(source, expected, serialize_css_string);
    }
}

fn assert_list(items: &[&str], expected: &str, serialize: ListSerializer) {
    let originals: Vec<_> = items.iter().map(|item| (*item).to_owned()).collect();
    // One container, then one node for every supplied item, including empty items.
    let nodes = 1 + items.len();
    assert_eq!(
        serialize(items, CssSerializationLimits::default()).unwrap(),
        expected
    );
    LimitLadder::new(expected, nodes, nodes).assert_with(
        |limits| serialize(items, limits),
        |_, _, _| {
            assert_eq!(
                serialize(items, CssSerializationLimits::default()).unwrap(),
                expected
            )
        },
    );
    for (item, original) in items.iter().zip(&originals) {
        assert_eq!(*item, original);
    }
    assert_eq!(
        serialize(items, CssSerializationLimits::default()).unwrap(),
        expected
    );
}

#[test]
fn comma_lists_keep_order_and_join_serialized_item_bytes_exactly() {
    for (items, expected) in [
        (&[][..], ""),
        (&[""][..], ""),
        (&["", ""][..], ", "),
        (&["red"][..], "red"),
        (&["z", "a", "m"][..], "z, a, m"),
        (&["", "a", ""][..], ", a, "),
        (&[" a ", "b"][..], " a , b"),
        (&["a,b", "\"c\"", "\\31 x"][..], "a,b, \"c\", \\31 x"),
        (&["\0", "é😀"][..], "\0, é😀"),
    ] {
        assert_list(items, expected, serialize_css_comma_separated_list);
    }
}

#[test]
fn whitespace_lists_keep_order_and_join_serialized_item_bytes_exactly() {
    for (items, expected) in [
        (&[][..], ""),
        (&[""][..], ""),
        (&["", ""][..], " "),
        (&["red"][..], "red"),
        (&["z", "a", "m"][..], "z a m"),
        (&["", "a", ""][..], " a "),
        (&[" a ", "b"][..], " a  b"),
        (&["a,b", "\"c\"", "\\31 x"][..], "a,b \"c\" \\31 x"),
        (&["\0", "é😀"][..], "\0 é😀"),
    ] {
        assert_list(items, expected, serialize_css_whitespace_separated_list);
    }
}

#[test]
fn empty_results_admit_zero_bytes_and_visit_the_primitive_or_container() {
    let empty_limits = CssSerializationLimits::new(1, 1, 0);
    assert_eq!(serialize_css_identifier("", empty_limits).unwrap(), "");
    assert_eq!(
        serialize_css_comma_separated_list(&[], empty_limits).unwrap(),
        ""
    );
    assert_eq!(
        serialize_css_whitespace_separated_list(&[], empty_limits).unwrap(),
        ""
    );
    assert!(
        serialize_css_string("", empty_limits)
            .unwrap_err()
            .is_byte_limit()
    );
    let zero = CssSerializationLimits::new(0, 0, 0);
    assert!(
        serialize_css_identifier("", zero)
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        serialize_css_comma_separated_list(&[], zero)
            .unwrap_err()
            .is_input_node_limit()
    );
    // An empty retained item is a child visit despite emitting no bytes.
    assert!(
        serialize_css_comma_separated_list(&[""], empty_limits)
            .unwrap_err()
            .is_input_node_limit()
    );
    assert_eq!(
        serialize_css_whitespace_separated_list(&[""], CssSerializationLimits::new(2, 2, 0))
            .unwrap(),
        ""
    );
}

#[test]
fn common_idioms_do_not_weaken_strict_identifier_construction() {
    assert!(
        CssIdent::try_new(String::new())
            .unwrap_err()
            .is_invalid_identifier()
    );
    assert!(
        CssIdent::try_new("\0".to_owned())
            .unwrap_err()
            .is_invalid_identifier()
    );
    assert_eq!(
        serialize_css_identifier("", Default::default()).unwrap(),
        ""
    );
    assert_eq!(
        serialize_css_identifier("\0", Default::default()).unwrap(),
        "\u{fffd}"
    );
}
