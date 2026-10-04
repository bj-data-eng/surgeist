#![forbid(unsafe_code)]
//! Independent expected output from CSSOM §2.1, including empty/NUL idioms
//! that intentionally do not cross strict property identifier/list grammars.
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#common-serializing-idioms

use surgeist_css::*;

type TextSerializer = fn(&str) -> Result<String, CssSpecifiedValueSerializationError>;
type BoundedTextSerializer = fn(
    &str,
    CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError>;
type ListSerializer = fn(&[&str]) -> Result<String, CssSpecifiedValueSerializationError>;
type BoundedListSerializer = fn(
    &[&str],
    CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError>;

fn assert_text(
    source: &str,
    expected: &str,
    serialize: TextSerializer,
    bounded: BoundedTextSerializer,
) {
    let original = source.to_owned();
    assert_eq!(serialize(source).unwrap(), expected);
    assert_eq!(
        bounded(
            source,
            CssSpecifiedValueSerializationLimits::new(1, 1, expected.len()),
        )
        .unwrap(),
        expected,
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(bounded(source, limits).unwrap_err().kind(), kind);
        assert_eq!(source, original);
        assert_eq!(serialize(source).unwrap(), expected);
    }
    if !expected.is_empty() {
        assert_eq!(
            bounded(
                source,
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
            )
            .unwrap_err()
            .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        );
    }
    assert_eq!(source, original);
    assert_eq!(serialize(source).unwrap(), expected);
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
        assert_text(
            source,
            expected,
            serialize_css_identifier,
            serialize_css_identifier_with_limits,
        );
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
        assert_text(
            source,
            expected,
            serialize_css_string,
            serialize_css_string_with_limits,
        );
    }
}

fn assert_list(
    items: &[&str],
    expected: &str,
    serialize: ListSerializer,
    bounded: BoundedListSerializer,
) {
    let originals: Vec<_> = items.iter().map(|item| (*item).to_owned()).collect();
    // One container, then one node for every supplied item, including empty items.
    let nodes = 1 + items.len();
    assert_eq!(serialize(items).unwrap(), expected);
    assert_eq!(
        bounded(
            items,
            CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len()),
        )
        .unwrap(),
        expected,
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(nodes - 1, nodes, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(nodes, nodes - 1, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(bounded(items, limits).unwrap_err().kind(), kind);
        assert_eq!(serialize(items).unwrap(), expected);
    }
    if !expected.is_empty() {
        assert_eq!(
            bounded(
                items,
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len() - 1),
            )
            .unwrap_err()
            .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        );
    }
    for (item, original) in items.iter().zip(&originals) {
        assert_eq!(*item, original);
    }
    assert_eq!(serialize(items).unwrap(), expected);
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
        assert_list(
            items,
            expected,
            serialize_css_comma_separated_list,
            serialize_css_comma_separated_list_with_limits,
        );
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
        assert_list(
            items,
            expected,
            serialize_css_whitespace_separated_list,
            serialize_css_whitespace_separated_list_with_limits,
        );
    }
}

#[test]
fn empty_results_admit_zero_bytes_but_still_visit_the_primitive_or_container() {
    let empty_limits = CssSpecifiedValueSerializationLimits::new(1, 1, 0);
    assert_eq!(
        serialize_css_identifier_with_limits("", empty_limits).unwrap(),
        ""
    );
    assert_eq!(
        serialize_css_comma_separated_list_with_limits(&[], empty_limits).unwrap(),
        ""
    );
    assert_eq!(
        serialize_css_whitespace_separated_list_with_limits(&[], empty_limits).unwrap(),
        ""
    );
    assert_eq!(
        serialize_css_string_with_limits("", empty_limits)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    let zero = CssSpecifiedValueSerializationLimits::new(0, 0, 0);
    assert_eq!(
        serialize_css_identifier_with_limits("", zero)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        serialize_css_comma_separated_list_with_limits(&[], zero)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    // An empty retained item is a child visit despite emitting no bytes.
    assert_eq!(
        serialize_css_comma_separated_list_with_limits(&[""], empty_limits)
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
    assert_eq!(
        serialize_css_whitespace_separated_list_with_limits(
            &[""],
            CssSpecifiedValueSerializationLimits::new(2, 2, 0)
        )
        .unwrap(),
        ""
    );
}

#[test]
fn common_idioms_do_not_weaken_strict_identifier_construction() {
    assert!(CssIdent::try_new(String::new()).is_err());
    assert!(CssIdent::try_new("\0".to_owned()).is_err());
    assert_eq!(serialize_css_identifier("").unwrap(), "");
    assert_eq!(serialize_css_identifier("\0").unwrap(), "\u{fffd}");
}
