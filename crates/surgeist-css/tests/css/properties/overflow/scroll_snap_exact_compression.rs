//! CSSOM WD 2021-08-26 §6.7.2 selects shorter lists before serializing
//! components. Its six-place number rule can give different exact values the
//! same text. CSS Scroll Snap 1 CR 2021-03-11 §§4.2, 5.1 and Appendix A,
//! and Logical 1 WD 2025-12-04 §4.7 supply pair and four-side role semantics.
//! Unitless ordinary length zero compares as px while retaining its authored
//! spelling: frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d,
//! CSSPropertyParserConsumer+LengthDefinitions.h lines 52–69 (blob
//! 2e9084f96aa8efd1b16ce60288cc893542fda9d8), and LengthPercentageDefinitions.h
//! lines 55–65 (blob 58e9b6710f69b36eee1bda429f1ea73bd6c6021d).
//! Explicit units remain distinct; no context-dependent conversion is selected.
//! Logical role tests assert the adopted authored model, not completed expansion.

use crate::css::support::serialization_limits::assert_node_limits;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::numeric::{
    CssCalculation, CssLength, CssLengthPercentage, CssNonNegative, CssNumeric,
};
use surgeist_css::property::{CssKnownPropertyValueRef, CssPropertyGrammar, CssPropertyTarget};
use surgeist_css::serialization::{CssSerializationError, CssSerializationLimits};
use surgeist_css::values::common::{CssAutoOr, CssBoxSideKind, CssFourSides, CssSidePair};

type Limits = CssSerializationLimits;
type Error = CssSerializationError;

fn component(text: &str, parsed: bool) -> CssComponentValue {
    if parsed {
        let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let [component] = components.items() else {
            panic!("one component: {text}")
        };
        component.clone()
    } else {
        CssComponentValue::try_token(text).unwrap()
    }
}
fn length(text: &str, parsed: bool) -> CssNumeric<CssLength> {
    CssNumeric::<CssLength>::try_from_component(component(text, parsed)).unwrap()
}
fn padding(text: &str, parsed: bool) -> CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>> {
    if text == "auto" {
        CssAutoOr::Auto
    } else {
        CssAutoOr::Value(
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component(
                text, parsed,
            ))
            .unwrap(),
        )
    }
}
fn raw(component: &CssComponentValue) -> String {
    CssComponentValues::try_new(vec![component.clone()], Default::default())
        .unwrap()
        .serialize_tokens(Default::default())
        .unwrap()
        .as_css()
        .to_owned()
}
fn assert_literal(component: &CssComponentValue, text: &str, parsed: bool) {
    assert_eq!(raw(component), text);
    if parsed {
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("retained parsed origin")
        };
        assert_eq!(origin.source().as_str(), text);
    } else {
        assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    }
}
fn padding_literal(
    value: &CssAutoOr<CssNumeric<CssLengthPercentage, CssNonNegative>>,
) -> &CssComponentValue {
    let CssAutoOr::Value(value) = value else {
        panic!("length or percentage padding")
    };
    value.literal().map(|literal| literal.component()).unwrap()
}
fn pair_padding(texts: [&str; 2], expected: &str) {
    let start = padding(texts[0], true);
    let end = padding(texts[1], false);
    let value = CssSidePair::new(start.clone(), Some(end.clone()));
    let before = value.clone();
    let result = value.serialize_specified(Default::default());
    assert_eq!(value, before);
    assert_eq!(value.first(), &start);
    assert_eq!(value.authored_second(), Some(&end));
    assert_eq!(value.second(), &end);
    assert_literal(padding_literal(value.first()), texts[0], true);
    assert_literal(padding_literal(value.second()), texts[1], false);
    assert_eq!(result.unwrap(), expected);
}
fn pair_margin(texts: [&str; 2], expected: &str) {
    let start = length(texts[0], true);
    let end = length(texts[1], false);
    let value = CssSidePair::new(start.clone(), Some(end.clone()));
    let before = value.clone();
    let result = value.serialize_specified(Default::default());
    assert_eq!(value, before);
    assert_eq!(value.first(), &start);
    assert_eq!(value.authored_second(), Some(&end));
    assert_eq!(value.second(), &end);
    assert_literal(
        value
            .first()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        texts[0],
        true,
    );
    assert_literal(
        value
            .second()
            .literal()
            .map(|literal| literal.component())
            .unwrap(),
        texts[1],
        false,
    );
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn unequal_padding_lengths_keep_both_authored_pair_sides() {
    pair_padding([".12345641px", ".12345642px"], "0.123456px 0.123456px");
}
#[test]
fn unequal_padding_percentages_keep_both_authored_pair_sides() {
    pair_padding([".12345641%", ".12345642%"], "0.123456% 0.123456%");
}
#[test]
fn unequal_signed_margin_lengths_keep_both_authored_pair_sides() {
    pair_margin(["-.12345641px", "-.12345642px"], "-0.123456px -0.123456px");
}

fn role_indices(count: usize) -> [usize; 4] {
    match count {
        1 => [0, 0, 0, 0],
        2 => [0, 1, 0, 1],
        3 => [0, 1, 2, 1],
        4 => [0, 1, 2, 3],
        _ => panic!("one to four authored values"),
    }
}
fn quad_padding(kind: CssBoxSideKind, texts: &[&str], expected: &str) {
    let authored: Vec<_> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| padding(text, i % 2 == 0))
        .collect();
    let value = CssFourSides::try_new(kind, authored.clone()).unwrap();
    let before = value.clone();
    let result = value.serialize_specified(Default::default());
    assert_eq!(value, before);
    assert_eq!(value.kind(), kind);
    assert_eq!(value.authored(), authored);
    for (i, text) in texts.iter().enumerate() {
        assert_literal(padding_literal(&value.authored()[i]), text, i % 2 == 0);
    }
    for (role, source) in role_indices(texts.len()).into_iter().enumerate() {
        assert_eq!(value.assigned().get(role).copied(), Some(&authored[source]));
    }
    assert_eq!(result.unwrap(), expected);
}
fn quad_margin(kind: CssBoxSideKind, texts: &[&str], expected: &str) {
    let authored: Vec<_> = texts
        .iter()
        .enumerate()
        .map(|(i, text)| length(text, i % 2 == 0))
        .collect();
    let value = CssFourSides::try_new(kind, authored.clone()).unwrap();
    let before = value.clone();
    let result = value.serialize_specified(Default::default());
    assert_eq!(value, before);
    assert_eq!(value.kind(), kind);
    assert_eq!(value.authored(), authored);
    for (i, text) in texts.iter().enumerate() {
        assert_literal(
            value.authored()[i]
                .literal()
                .map(|literal| literal.component())
                .unwrap(),
            text,
            i % 2 == 0,
        );
    }
    for (role, source) in role_indices(texts.len()).into_iter().enumerate() {
        assert_eq!(value.assigned().get(role).copied(), Some(&authored[source]));
    }
    assert_eq!(result.unwrap(), expected);
}
const UNEQUAL_LENGTHS: [&str; 4] = [".12345641px", ".12345642px", ".12345643px", ".12345644px"];
const UNEQUAL_PERCENTAGES: [&str; 4] = [".12345641%", ".12345642%", ".12345643%", ".12345644%"];
const UNEQUAL_NEGATIVE_LENGTHS: [&str; 4] = [
    "-.12345641px",
    "-.12345642px",
    "-.12345643px",
    "-.12345644px",
];
macro_rules! quad_test {
    ($name:ident, $helper:ident, $kind:ident, $values:ident, $expected:literal) => {
        #[test]
        fn $name() {
            $helper(CssBoxSideKind::$kind, &$values, $expected);
        }
    };
}
quad_test!(
    physical_padding_lengths_keep_four_exactly_unequal_roles,
    quad_padding,
    Physical,
    UNEQUAL_LENGTHS,
    "0.123456px 0.123456px 0.123456px 0.123456px"
);
quad_test!(
    logical_padding_lengths_keep_four_exactly_unequal_roles,
    quad_padding,
    Logical,
    UNEQUAL_LENGTHS,
    "logical 0.123456px 0.123456px 0.123456px 0.123456px"
);
quad_test!(
    physical_padding_percentages_keep_four_exactly_unequal_roles,
    quad_padding,
    Physical,
    UNEQUAL_PERCENTAGES,
    "0.123456% 0.123456% 0.123456% 0.123456%"
);
quad_test!(
    logical_padding_percentages_keep_four_exactly_unequal_roles,
    quad_padding,
    Logical,
    UNEQUAL_PERCENTAGES,
    "logical 0.123456% 0.123456% 0.123456% 0.123456%"
);
quad_test!(
    physical_signed_margins_keep_four_exactly_unequal_roles,
    quad_margin,
    Physical,
    UNEQUAL_NEGATIVE_LENGTHS,
    "-0.123456px -0.123456px -0.123456px -0.123456px"
);
quad_test!(
    logical_signed_margins_keep_four_exactly_unequal_roles,
    quad_margin,
    Logical,
    UNEQUAL_NEGATIVE_LENGTHS,
    "logical -0.123456px -0.123456px -0.123456px -0.123456px"
);

#[test]
fn each_authored_cardinality_retains_roles_before_rounded_quad_selection() {
    for count in 1..=4 {
        let expected = match count {
            1 => "0.123456px",
            2 => "0.123456px 0.123456px",
            3 => "0.123456px 0.123456px 0.123456px",
            _ => "0.123456px 0.123456px 0.123456px 0.123456px",
        };
        quad_margin(
            CssBoxSideKind::Physical,
            &UNEQUAL_LENGTHS[..count],
            expected,
        );
        quad_padding(
            CssBoxSideKind::Logical,
            &UNEQUAL_LENGTHS[..count],
            &format!("logical {expected}"),
        );
    }
}

#[test]
fn equivalent_coefficients_and_distinct_origins_select_one_two_and_three() {
    pair_padding(["+1.0PX", "01px"], "1px");
    pair_margin(["1e0px", "1.000px"], "1px");
    for (texts, expected) in [
        (["+1.0px", "1e0px", "01px", "1px"], "1px"),
        (["1px", "2e0px", "01px", "+2.0px"], "1px 2px"),
        (["1px", "2px", "3px", "2e0px"], "1px 2px 3px"),
    ] {
        quad_margin(CssBoxSideKind::Physical, &texts, expected);
        quad_padding(
            CssBoxSideKind::Logical,
            &texts,
            &format!("logical {expected}"),
        );
    }
}

#[test]
fn exact_two_and_three_side_selection_precedes_equal_rounded_text() {
    for (texts, expected) in [
        (
            [
                ".12345641px",
                ".12345642px",
                "12345641e-8px",
                ".123456420px",
            ],
            "0.123456px 0.123456px",
        ),
        (
            [".12345641px", ".12345642px", ".12345643px", ".123456420px"],
            "0.123456px 0.123456px 0.123456px",
        ),
    ] {
        quad_margin(CssBoxSideKind::Physical, &texts, expected);
        quad_padding(
            CssBoxSideKind::Logical,
            &texts,
            &format!("logical {expected}"),
        );
    }
}

#[test]
fn exact_unitless_padding_zero_compares_as_px_and_keeps_first_emission() {
    pair_padding(["-0", "+0px"], "0");
    pair_padding(["-0px", "+0"], "0px");
    quad_padding(CssBoxSideKind::Physical, &["0", "0px", "-0", "-0px"], "0");
    quad_padding(
        CssBoxSideKind::Logical,
        &["0px", "0", "-0px", "-0"],
        "logical 0px",
    );
}
#[test]
fn exact_unitless_margin_zero_compares_as_px_and_keeps_first_emission() {
    pair_margin(["-0", "+0px"], "0");
    pair_margin(["-0px", "+0"], "0px");
    quad_margin(CssBoxSideKind::Physical, &["0", "0px", "-0", "-0px"], "0");
    quad_margin(
        CssBoxSideKind::Logical,
        &["0px", "0", "-0px", "-0"],
        "logical 0px",
    );
}

#[test]
fn explicit_units_auto_and_nonzero_physical_units_remain_distinct() {
    pair_padding(["0px", "0%"], "0px 0%");
    pair_padding(["0px", "0em"], "0px 0em");
    pair_margin(["0px", "0cm"], "0px 0cm");
    pair_margin(["96px", "1in"], "96px 1in");
    let pair = CssSidePair::new(padding("auto", false), Some(padding("auto", false)));
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "auto"
    );
    let pair = CssSidePair::new(padding("auto", false), Some(padding("0", false)));
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "auto 0"
    );
    let single = CssSidePair::new(padding("auto", false), None);
    assert!(single.authored_second().is_none());
    assert_eq!(single.second(), single.first());
    assert_eq!(
        single.serialize_specified(Limits::new(2, 2, 4)).unwrap(),
        "auto"
    );
}

#[test]
fn tiny_positive_padding_is_distinct_from_actual_px_zero() {
    pair_padding([".0000001px", "0px"], "0px 0px");
    pair_padding([".0000001%", "0%"], "0% 0%");
    quad_padding(
        CssBoxSideKind::Physical,
        &["0px", ".0000001px", "0px", "-0px"],
        "0px 0px 0px 0px",
    );
}
#[test]
fn tiny_negative_margin_is_distinct_from_actual_px_zero() {
    pair_margin(["-.0000001px", "0px"], "0px 0px");
    quad_margin(
        CssBoxSideKind::Logical,
        &["0px", "-.0000001px", "0px", "-0px"],
        "logical 0px 0px 0px 0px",
    );
}
#[test]
fn negative_padding_is_rejected_before_any_lossy_rounding() {
    for text in [
        "-.0000001px",
        "-.0000001%",
        "-1e-170141183460469231731687303715884105728px",
    ] {
        assert!(
            CssNumeric::<CssLengthPercentage, CssNonNegative>::try_from_component(component(
                text, false
            ))
            .is_err()
        );
        let grammar = CssPropertyGrammar::from_name("scroll-padding-block").unwrap();
        assert!(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(grammar),
                CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                CssImportance::Normal
            )
            .is_err()
        );
    }
}

#[test]
fn exponents_outside_i128_keep_exact_inequality_in_pairs_and_quads() {
    let values = [
        "1e-170141183460469231731687303715884105728px",
        "1e-170141183460469231731687303715884105729px",
        "1e-170141183460469231731687303715884105730px",
        "1e-170141183460469231731687303715884105731px",
    ];
    pair_padding([values[0], values[1]], "0px 0px");
    pair_margin([values[0], values[1]], "0px 0px");
    quad_margin(CssBoxSideKind::Physical, &values, "0px 0px 0px 0px");
    quad_padding(CssBoxSideKind::Logical, &values, "logical 0px 0px 0px 0px");
}
#[test]
fn huge_exponent_compensation_and_signed_actual_zero_remain_equal() {
    let values = [
        "1e-170141183460469231731687303715884105728px",
        "10e-170141183460469231731687303715884105729px",
        "100e-170141183460469231731687303715884105730px",
        "1000e-170141183460469231731687303715884105731px",
    ];
    pair_margin([values[0], values[1]], "0px");
    pair_padding([values[0], values[1]], "0px");
    quad_margin(CssBoxSideKind::Physical, &values, "0px");
    quad_padding(CssBoxSideKind::Logical, &values, "logical 0px");
    pair_margin(
        ["-0e-170141183460469231731687303715884105728px", "+0px"],
        "0px",
    );
}

fn exact_limits(
    expected: &str,
    nodes: usize,
    bytes: usize,
    serialize: impl Fn(Limits) -> Result<String, Error>,
) {
    assert_eq!(expected.len(), bytes);
    assert_node_limits(expected, nodes, serialize);
}
#[test]
fn pair_limits_charge_only_authored_children_and_both_rounded_tokens() {
    let value = CssSidePair::new(
        length(UNEQUAL_LENGTHS[0], false),
        Some(length(UNEQUAL_LENGTHS[1], true)),
    );
    let before = value.clone();
    // Two 10-byte lengths and one space, each child one input/projection, plus the aggregate.
    exact_limits("0.123456px 0.123456px", 3, 21, |limits| {
        let result = value.serialize_specified(limits);
        assert_eq!(value, before);
        result
    });
}
#[test]
fn quad_limits_charge_authored_children_and_logical_marker_before_final_text() {
    let value = CssFourSides::try_new(
        CssBoxSideKind::Logical,
        UNEQUAL_LENGTHS
            .iter()
            .map(|text| padding(text, false))
            .collect(),
    )
    .unwrap();
    let before = value.clone();
    // Four 10-byte lengths + three spaces + the eight-byte logical marker.
    // Four authored nodes and one marker; expanded roles add no visits.
    exact_limits(
        "logical 0.123456px 0.123456px 0.123456px 0.123456px",
        5,
        51,
        |limits| {
            let result = value.serialize_specified(limits);
            assert_eq!(value, before);
            result
        },
    );
}
#[test]
fn compressed_authored_counts_scratch_limits_and_failure_order_stay_exact() {
    for count in 1..=4 {
        let value = CssFourSides::try_new(
            CssBoxSideKind::Physical,
            vec![length(".12345641px", false); count],
        )
        .unwrap();
        // One aggregate node covers the physical form as it covers `logical`.
        exact_limits("0.123456px", count + 1, 10, |limits| {
            value.serialize_specified(limits)
        });
        let value = CssFourSides::try_new(
            CssBoxSideKind::Logical,
            vec![padding(".12345641px", false); count],
        )
        .unwrap();
        exact_limits("logical 0.123456px", count + 1, 18, |limits| {
            value.serialize_specified(limits)
        });
    }
    let pair = CssSidePair::new(
        length(".12345641px", false),
        Some(length(".123456410px", false)),
    );
    exact_limits("0.123456px", 3, 10, |limits| {
        pair.serialize_specified(limits)
    });
    assert!(
        pair.serialize_specified(Limits::new(0, 0, 0))
            .unwrap_err()
            .is_input_node_limit()
    );
    assert!(
        pair.serialize_specified(Limits::new(3, 0, 0))
            .unwrap_err()
            .is_projection_node_limit()
    );
    assert!(
        pair.serialize_specified(Limits::new(3, 3, 9))
            .unwrap_err()
            .is_byte_limit()
    );
    let pair = CssSidePair::new(length("1px", false), Some(length("1px", false)));
    exact_limits("1px", 3, 3, |limits| pair.serialize_specified(limits));
    let single = CssSidePair::new(length("1px", false), None);
    exact_limits("1px", 2, 3, |limits| single.serialize_specified(limits));
}

#[test]
fn canonical_math_pair_and_quad_compression_keeps_actual_calculation_policy() {
    let calc = |text| {
        CssNumeric::<CssLength>::try_from_calculation(
            CssCalculation::<CssLength>::try_from_components(
                CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap()
    };
    let pair = CssSidePair::new(calc("calc(1px + 2px)"), Some(calc("calc(3px)")));
    let before = pair.clone();
    assert_eq!(
        pair.serialize_specified(Default::default()).unwrap(),
        "calc(3px)"
    );
    assert_eq!(pair, before);
    let quad = CssFourSides::try_new(
        CssBoxSideKind::Logical,
        vec![
            calc("calc(1em + 2px)"),
            calc("calc(2px + 1em)"),
            calc("calc(1.0em + 2.0px)"),
            calc("calc(2px + 1em)"),
        ],
    )
    .unwrap();
    let before = quad.clone();
    assert_eq!(
        quad.serialize_specified(Default::default()).unwrap(),
        "logical calc(1em + 2px)"
    );
    assert_eq!(quad, before);
}

#[test]
fn parsed_property_pair_preserves_declaration_and_source_after_rounding() {
    let source = "scroll-padding-inline:.12345641px .12345642px!important";
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    let before = declaration.clone();
    let CssKnownPropertyValueRef::ScrollPaddingInline(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed scroll-padding-inline")
    };
    let pair = value.value();
    let start = padding_literal(pair.first());
    let end = padding_literal(pair.authored_second().unwrap());
    assert_eq!(raw(start), ".12345641px");
    assert_eq!(raw(end), ".12345642px");
    for component in [start, end] {
        let CssValueOrigin::Parsed(origin) = component.origin() else {
            panic!("parsed declaration source")
        };
        assert_eq!(origin.source().as_str(), source);
    }
    let result = pair.serialize_specified(Default::default());
    assert_eq!(declaration, &before);
    assert_eq!(result.unwrap(), "0.123456px 0.123456px");
}
