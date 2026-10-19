#![forbid(unsafe_code)]
//! Numeric token boundaries through the public checked component interface.
//!
//! Independent expectations: CSS Syntax 3 CRD 2021-12-24 §§4.3.3,
//! 4.3.10, 4.3.12 and 4.3.13. Failed fraction/exponent lookahead leaves the
//! following code points pending; exponent signs belong to successful numbers.
//! The public checked token contract preserves spelling, lexical flags and
//! genuine consumed origins. Simple decimal value goldens below are calculated
//! directly from s * (i + f * 10^-d) * 10^(t*e), without host-float oracles.
//! Existing huge-exponent/zero and exact-arithmetic suites remain separate.
//! <https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/>

use std::ops::Range;

use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueRef, CssComponentValues,
    CssNumericTokenKind, CssNumericTokenRef, CssSpecifiedNumber, CssValueOrigin, CssValueTokenRef,
    parse_component_values,
};

use CssNumericTokenKind::{Integer, Number};

#[derive(Debug)]
enum Expected<'a> {
    Number(&'a str, CssNumericTokenKind, bool),
    Percentage(&'a str, CssNumericTokenKind, bool),
    Dimension(&'a str, &'a str, CssNumericTokenKind, bool),
    Delim(char),
    Ident(&'a str),
}

fn assert_numeric(
    actual: CssNumericTokenRef<'_>,
    representation: &str,
    kind: CssNumericTokenKind,
    has_sign: bool,
) {
    assert_eq!(actual.representation(), representation);
    assert_eq!(actual.kind(), kind);
    assert_eq!(actual.has_sign(), has_sign);
}

fn assert_token(component: &CssComponentValue, expected: &Expected<'_>) {
    match (component.view(), expected) {
        (
            CssComponentValueRef::Token(CssValueTokenRef::Number(actual)),
            Expected::Number(representation, kind, sign),
        )
        | (
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(actual)),
            Expected::Percentage(representation, kind, sign),
        ) => assert_numeric(actual, representation, *kind, *sign),
        (
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }),
            Expected::Dimension(representation, expected_unit, kind, sign),
        ) => {
            assert_numeric(number, representation, *kind, *sign);
            assert_eq!(unit, *expected_unit);
        }
        (
            CssComponentValueRef::Token(CssValueTokenRef::Delim(actual)),
            Expected::Delim(expected),
        ) => assert_eq!(actual, *expected),
        (
            CssComponentValueRef::Token(CssValueTokenRef::Ident(actual)),
            Expected::Ident(expected),
        ) => assert_eq!(actual, *expected),
        (actual, expected) => panic!("expected {expected:?}, got {actual:?}"),
    }
}

fn assert_origin(component: &CssComponentValue, source: &str, range: Range<usize>) {
    let CssValueOrigin::Parsed(origin) = component.origin() else {
        panic!("source token must keep its consumed source origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), range.start);
    assert_eq!(origin.span().end().byte_offset().value(), range.end);
}

// Test each selected boundary both at real EOF and before an unambiguous later
// token. The semicolon prevents the sentinel from extending a dimension unit.
fn assert_sequence(source: &str, expected: &[(Expected<'_>, Range<usize>)]) {
    for with_sentinel in [false, true] {
        let input = if with_sentinel {
            format!("{source};sentinel")
        } else {
            source.to_owned()
        };
        let values = parse_component_values(&input).expect("valid token sequence");
        assert_eq!(
            values.items().len(),
            expected.len() + if with_sentinel { 2 } else { 0 },
            "{input:?}"
        );
        assert_eq!(values.component_count(), values.items().len());
        for (component, (token, range)) in values.items().iter().zip(expected) {
            assert_token(component, token);
            assert_origin(component, &input, range.clone());
        }
        if with_sentinel {
            let delimiter = &values.items()[expected.len()];
            assert!(matches!(
                delimiter.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Semicolon)
            ));
            assert_origin(delimiter, &input, source.len()..source.len() + 1);
            let sentinel = &values.items()[expected.len() + 1];
            assert_token(sentinel, &Expected::Ident("sentinel"));
            assert_origin(sentinel, &input, source.len() + 1..input.len());
        }
        let CssValueOrigin::Parsed(first) = values.items()[0].origin() else {
            panic!("original source snapshot")
        };
        for component in values.items() {
            let CssValueOrigin::Parsed(origin) = component.origin() else {
                panic!("each token keeps the shared source snapshot")
            };
            assert!(origin.source().same_snapshot(first.source()));
        }
    }
}

#[test]
fn number_start_lookahead_preserves_flags_and_false_branch_code_points() {
    for (source, kind, sign) in [
        (".5", Number, false),
        ("+.5", Number, true),
        ("-.5", Number, true),
        ("5", Integer, false),
        ("+5", Integer, true),
        ("-5", Integer, true),
    ] {
        assert_sequence(
            source,
            &[(Expected::Number(source, kind, sign), 0..source.len())],
        );
    }
    for sign in ['+', '-'] {
        let source = format!("{sign}.");
        assert_sequence(
            &source,
            &[(Expected::Delim(sign), 0..1), (Expected::Delim('.'), 1..2)],
        );
    }
    assert_sequence(
        ".a",
        &[(Expected::Delim('.'), 0..1), (Expected::Ident("a"), 1..2)],
    );
    // Non-ASCII digits are identifier code points, not CSS's ASCII digits.
    assert_sequence("²", &[(Expected::Ident("²"), 0..2)]);
    for delimiter in ['+', '-', '.'] {
        let source = delimiter.to_string();
        assert_sequence(&source, &[(Expected::Delim(delimiter), 0..1)]);
    }
    assert_sequence(
        "+a",
        &[(Expected::Delim('+'), 0..1), (Expected::Ident("a"), 1..2)],
    );
    assert_sequence("-a", &[(Expected::Ident("-a"), 0..2)]);
}

#[test]
fn failed_fraction_guard_retains_dots_and_a_later_fractional_number() {
    for (source, prefix, sign) in [("1.", "1", false), ("+1.", "+1", true), ("-1.", "-1", true)] {
        assert_sequence(
            source,
            &[
                (Expected::Number(prefix, Integer, sign), 0..prefix.len()),
                (Expected::Delim('.'), prefix.len()..source.len()),
            ],
        );
    }
    assert_sequence(
        "1..5",
        &[
            (Expected::Number("1", Integer, false), 0..1),
            (Expected::Delim('.'), 1..2),
            (Expected::Number(".5", Number, false), 2..4),
        ],
    );
    assert_sequence(
        "1.2.3",
        &[
            (Expected::Number("1.2", Number, false), 0..3),
            (Expected::Number(".3", Number, false), 3..5),
        ],
    );
}

#[test]
fn failed_exponent_guard_leaves_identifier_units_and_signed_tail_tokens() {
    for (source, unit) in [("1e", "e"), ("1e-", "e-")] {
        assert_sequence(
            source,
            &[(
                Expected::Dimension("1", unit, Integer, false),
                0..source.len(),
            )],
        );
    }
    for (source, unit) in [("1e+", "e"), ("1E+", "E")] {
        assert_sequence(
            source,
            &[
                (Expected::Dimension("1", unit, Integer, false), 0..2),
                (Expected::Delim('+'), 2..3),
            ],
        );
    }
    assert_sequence(
        "1e+x",
        &[
            (Expected::Dimension("1", "e", Integer, false), 0..2),
            (Expected::Delim('+'), 2..3),
            (Expected::Ident("x"), 3..4),
        ],
    );
    assert_sequence(
        "1e-+2",
        &[
            (Expected::Dimension("1", "e-", Integer, false), 0..3),
            (Expected::Number("+2", Integer, true), 3..5),
        ],
    );
}

#[test]
fn successful_exponents_remain_whole_numeric_prefixes_with_number_flags() {
    for (source, sign) in [
        ("1e0", false),
        ("1E+2", false),
        ("1e-2", false),
        ("+.5e+2", true),
    ] {
        assert_sequence(
            source,
            &[(Expected::Number(source, Number, sign), 0..source.len())],
        );
    }
    assert_sequence(
        "1e-2PX",
        &[(Expected::Dimension("1e-2", "PX", Number, false), 0..6)],
    );
    assert_sequence(
        "-1.5E+2%",
        &[(Expected::Percentage("-1.5E+2", Number, true), 0..8)],
    );
    assert_sequence("-0", &[(Expected::Number("-0", Integer, true), 0..2)]);
    assert_sequence("-0.0", &[(Expected::Number("-0.0", Number, true), 0..4)]);
}

#[test]
fn numeric_category_dispatch_preserves_case_escapes_and_invalid_unit_tails() {
    assert_sequence(
        "1-",
        &[
            (Expected::Number("1", Integer, false), 0..1),
            (Expected::Delim('-'), 1..2),
        ],
    );
    for (source, unit) in [("1--", "--"), ("1-a", "-a"), ("12px", "px")] {
        let prefix = if source == "12px" { "12" } else { "1" };
        assert_sequence(
            source,
            &[(
                Expected::Dimension(prefix, unit, Integer, false),
                0..source.len(),
            )],
        );
    }
    assert_sequence(
        "+1.25PX",
        &[(Expected::Dimension("+1.25", "PX", Number, true), 0..7)],
    );
    assert_sequence(
        r"1P\58 ",
        &[(Expected::Dimension("1", "PX", Integer, false), 0..6)],
    );
    assert_sequence("25%", &[(Expected::Percentage("25", Integer, false), 0..3)]);
    assert_sequence(
        "1px%",
        &[
            (Expected::Dimension("1", "px", Integer, false), 0..3),
            (Expected::Delim('%'), 3..4),
        ],
    );
}

#[test]
fn validated_numeric_conversion_uses_sign_fraction_and_signed_decimal_exponent() {
    for (source, kind, sign, expected_value) in [
        ("+001", Integer, true, "1"),
        ("-.5", Number, true, "-0.5"),
        ("-12.5e-1", Number, true, "-1.25"),
        ("+.125E+2", Number, true, "12.5"),
        ("125e-2", Number, false, "1.25"),
        ("1", Integer, false, "1"),
        ("1.0", Number, false, "1"),
        ("1e0", Number, false, "1"),
    ] {
        let parsed = parse_component_values(source).unwrap();
        let [original] = parsed.items() else {
            panic!("one complete numeric token")
        };
        for component in [
            original.clone(),
            CssComponentValue::try_number(source).unwrap(),
        ] {
            assert_token(&component, &Expected::Number(source, kind, sign));
            let value = CssSpecifiedNumber::try_from_component(component.clone()).unwrap();
            let before = value.clone();
            assert_eq!(
                value.serialize_specified().unwrap(),
                expected_value,
                "{source}"
            );
            assert_eq!(value, before);
            assert_eq!(value.literal_component(), Some(&component));
            assert_eq!(value.origin(), component.origin());
            // Mathematical conversion does not erase the authored spelling.
            assert_eq!(
                CssComponentValues::try_new(vec![component])
                    .unwrap()
                    .serialize()
                    .unwrap()
                    .as_css(),
                source
            );
        }
        assert_origin(original, source, 0..source.len());
    }
}

#[test]
fn whole_number_construction_rejects_invalid_spelling_and_admits_a_valid_retry() {
    for spelling in [
        "", "+", "-", ".", "+.", "-.", "1.", "1e", "1e+", "1e-", "1 2", "1px", "1%", "NaN",
        "Infinity",
    ] {
        let error = CssComponentValue::try_number(spelling)
            .expect_err("a prefix/category is not a whole CSS number");
        assert_eq!(
            error.kind(),
            CssComponentValueErrorKind::InvalidNumber,
            "{spelling:?}"
        );
        assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
        let retry = CssComponentValue::try_number("-.5e+2").unwrap();
        assert_token(&retry, &Expected::Number("-.5e+2", Number, true));
        assert_eq!(retry.origin(), &CssValueOrigin::Programmatic);
        assert_eq!(
            CssSpecifiedNumber::try_from_component(retry)
                .unwrap()
                .serialize_specified()
                .unwrap(),
            "-50"
        );
    }
}

#[test]
fn dimension_construction_validates_number_text_before_an_exponent_like_unit() {
    for spelling in ["", "+", ".", "1.", "1e+", "1 2", "1px", "1%"] {
        let error = CssComponentValue::try_dimension(spelling, "e2")
            .expect_err("dimension number must be checked whole text");
        assert_eq!(
            error.kind(),
            CssComponentValueErrorKind::InvalidDimension,
            "{spelling:?}"
        );
        assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
        let retry = CssComponentValue::try_dimension("1", "e2").unwrap();
        assert_token(&retry, &Expected::Dimension("1", "e2", Integer, false));
        assert_eq!(retry.origin(), &CssValueOrigin::Programmatic);
        let css = CssComponentValues::try_new(vec![retry])
            .unwrap()
            .serialize()
            .unwrap();
        let reparsed = parse_component_values(css.as_css()).unwrap();
        let [component] = reparsed.items() else {
            panic!("one dimension, not numeric exponent")
        };
        assert_token(component, &Expected::Dimension("1", "e2", Integer, false));
    }
}
