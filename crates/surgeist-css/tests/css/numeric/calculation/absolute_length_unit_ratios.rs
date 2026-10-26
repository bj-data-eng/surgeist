//! Values 4 WD 2024-03-12 §6.2 fixes the absolute length units relative to
//! each other: 1in = 2.54cm = 96px, 1mm = 1/10th of 1cm, 1Q = 1/40th of 1cm,
//! 1pc = 1/6th of 1in and 1pt = 1/72nd of 1in. Equal absolute lengths in
//! different units are therefore mathematically equal inside math functions,
//! so their ratio is exactly 1 and their difference is exactly zero.

use surgeist_css::{
    CssCalculation, CssComponentValues, CssLength, CssNumber, CssNumeric, CssParser,
};

/// Pairs of equal absolute lengths, each spelled in two different units.
const EQUAL_LENGTHS: [(&str, &str); 12] = [
    ("1cm", "10mm"),
    ("1cm", "40Q"),
    ("1mm", "4Q"),
    ("1in", "2.54cm"),
    ("1in", "25.4mm"),
    ("1in", "101.6Q"),
    ("1in", "96px"),
    ("1in", "72pt"),
    ("1in", "6pc"),
    ("1pc", "12pt"),
    ("3cm", "30mm"),
    ("7mm", "28Q"),
];

fn number(source: &str) -> String {
    CssNumeric::<CssNumber>::try_from_calculation(
        CssCalculation::<CssNumber>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
    .serialize_specified(Default::default())
    .unwrap()
}

fn length(source: &str) -> String {
    CssNumeric::<CssLength>::try_from_calculation(
        CssCalculation::<CssLength>::try_from_components(
            CssComponentValues::parse(&CssParser::default(), source).unwrap(),
            Default::default(),
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}")),
    )
    .unwrap()
    .serialize_specified(Default::default())
    .unwrap()
}

#[test]
fn ratio_of_equal_absolute_lengths_rounds_down_to_exactly_one() {
    // Division multiplies by a binary64 reciprocal, so even `3cm / 3cm` is
    // not exactly one; that arithmetic is independent of the unit ratios and
    // this test keeps to pairs whose canonical value has an exact reciprocal product.
    for (left, right) in EQUAL_LENGTHS.into_iter().filter(|(left, _)| *left != "3cm") {
        for (a, b) in [(left, right), (right, left)] {
            assert_eq!(
                number(&format!("calc(round(down, {a} / {b}, 1))")),
                "calc(1)",
                "{a} / {b}"
            );
            assert_eq!(
                number(&format!("calc(round(up, {a} / {b}, 1))")),
                "calc(1)",
                "{a} / {b}"
            );
        }
    }
}

#[test]
fn sign_of_the_difference_of_equal_absolute_lengths_is_zero() {
    for (left, right) in EQUAL_LENGTHS {
        for (a, b) in [(left, right), (right, left)] {
            assert_eq!(
                number(&format!("calc(sign({a} - {b}))")),
                "calc(0)",
                "{a} - {b}"
            );
        }
    }
}

#[test]
fn difference_and_rounding_of_equal_absolute_lengths_are_exact() {
    for (left, right) in EQUAL_LENGTHS {
        for (a, b) in [(left, right), (right, left)] {
            assert_eq!(
                length(&format!("calc({a} - {b})")),
                "calc(0px)",
                "{a} - {b}"
            );
            // A length rounded to a step equal to itself is unchanged: the
            // canonical px value of `a`, which equals `b`.
            assert_eq!(
                length(&format!("calc(round(down, {a}, {b}) - {b})")),
                "calc(0px)",
                "round(down, {a}, {b})"
            );
        }
    }
}

#[test]
fn modulus_and_remainder_of_equal_absolute_lengths_are_zero() {
    for (left, right) in EQUAL_LENGTHS {
        for (a, b) in [(left, right), (right, left)] {
            for function in ["mod", "rem"] {
                assert_eq!(
                    length(&format!("calc({function}({a}, {b}))")),
                    "calc(0px)",
                    "{function}({a}, {b})"
                );
                assert_eq!(
                    number(&format!("calc(sign({function}({a}, {b})))")),
                    "calc(0)",
                    "sign({function}({a}, {b}))"
                );
            }
        }
    }
}

#[test]
fn issue_reproductions_project_the_exact_mathematical_result() {
    assert_eq!(number("calc(round(down, 1cm / 10mm, 1))"), "calc(1)");
    assert_eq!(number("calc(sign(10mm - 1cm))"), "calc(0)");
    assert_eq!(number("calc(sign(1in - 96px))"), "calc(0)");
    assert_eq!(length("calc(1cm - 10mm)"), "calc(0px)");
}

/// Known inexact class, pinned rather than endorsed: Values 4 makes
/// `rem(10mm, 1mm)` exactly `0px`. Each leaf projects to the correctly rounded
/// canonical px value (`10mm` to `fl(4800/127)`, `1mm` to `fl(480/127)`), but
/// those binary64 values are not exact multiples of each other, so the binary64
/// remainder is one divisor short of zero. Evaluating same-unit operands in
/// their source unit, or routing `mod()`/`rem()`/`round()` through the exact
/// rational path, is tracked by #1433; these expectations become
/// `calc(0px)` when it lands.
#[test]
fn known_inexact_same_unit_remainders_follow_binary64_projection() {
    for source in [
        "calc(rem(10mm, 1mm))",
        "calc(rem(3mm, 1mm))",
        "calc(rem(1cm, 1mm))",
        "calc(mod(10mm, 1mm))",
    ] {
        assert_eq!(length(source), "calc(3.779528px)", "{source}");
    }
}
