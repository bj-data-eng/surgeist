#![forbid(unsafe_code)]
//! Counter Styles 3 (2021-07-27) §§3.1.5, 3.5, 3.6, and 3.8 import
//! Values4 (2021-07-15) §11.9 Number-result math into integer slots.
//! Values4 §11.12 and §5.2.1 defer integer rounding (half toward +infinity)
//! and numeric range clamping to computed/used values, preserving specified math.

use surgeist_css::{
    CssComponentValueLimits, CssCounterStyleDescriptorKind as Kind,
    CssCounterStyleDescriptorValue as Value, CssCounterStyleDescriptorValueRef as View,
    CssCounterStyleRule, CssErrorCode, CssRecoveryAction, CssRule,
    CssSpecifiedValueSerializationErrorKind as SerializationError,
    CssSpecifiedValueSerializationLimits as SerializationLimits, parse_component_values,
    parse_counter_style_descriptor_value, parse_sheet, validate_sheet,
};

fn rule(body: &str) -> CssCounterStyleRule {
    let source = format!("@counter-style Math {{ {body} }}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert!(validate_sheet(&source).is_ok());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("one counter-style rule must be retained");
    };
    rule.clone()
}

fn assert_descriptor(body: &str, kind: Kind, expected: &str) {
    let value = rule(body);
    assert_eq!(value.descriptor_specified_css(kind).unwrap(), expected);
    let canonical = value.to_specified_css().unwrap();
    let reparsed = parse_sheet(&canonical);
    assert!(
        reparsed.is_clean(),
        "{canonical}: {:?}",
        reparsed.diagnostics()
    );
    let [CssRule::CounterStyle(reparsed)] = reparsed.syntax().rules() else {
        panic!("canonical serialization must retain the counter-style rule");
    };
    assert_eq!(reparsed.to_specified_css().unwrap(), canonical);
    assert_eq!(reparsed.descriptor_specified_css(kind).unwrap(), expected);
}

#[test]
fn fixed_start_admits_number_result_math_and_retains_calculation_identity() {
    assert_descriptor("system: fixed calc(1 + 1);", Kind::System, "fixed calc(2)");
}

#[test]
fn finite_range_bounds_admit_number_result_math_beside_contextual_infinity() {
    assert_descriptor(
        "range: infinite calc(1 + 1), calc(1 + 1) infinite;",
        Kind::Range,
        "infinite calc(2), calc(2) infinite",
    );
}

#[test]
fn pad_length_admits_number_result_math_in_either_grammar_order() {
    for body in ["pad: calc(1 + 1) '0';", "pad: '0' calc(1 + 1);"] {
        assert_descriptor(body, Kind::Pad, "calc(2) \"0\"");
    }
}

#[test]
fn additive_weights_admit_number_result_math_in_either_tuple_order() {
    assert_descriptor(
        "system: additive; additive-symbols: X calc(1 + 1), calc(1) I, N 0;",
        Kind::AdditiveSymbols,
        "calc(2) X, calc(1) I, 0 N",
    );
}

#[test]
fn fractional_integer_math_keeps_specified_values_before_half_tie_rounding() {
    for (kind, body, expected) in [
        (
            Kind::System,
            "system: fixed calc(-1.5);",
            "fixed calc(-1.5)",
        ),
        (
            Kind::Range,
            "range: calc(-1.5) calc(1.5);",
            "calc(-1.5) calc(1.5)",
        ),
        (Kind::Pad, "pad: calc(1.5) '0';", "calc(1.5) \"0\""),
        (
            Kind::AdditiveSymbols,
            "additive-symbols: calc(2.5) X, calc(1.5) I;",
            "calc(2.5) X, calc(1.5) I",
        ),
    ] {
        assert_descriptor(body, kind, expected);
    }
}

#[test]
fn nonnegative_numeric_ranges_defer_math_clamping_without_clamping_specified_text() {
    assert_descriptor("pad: calc(-1.5) '0';", Kind::Pad, "calc(-1.5) \"0\"");
    assert_descriptor(
        "additive-symbols: calc(-1.5) N;",
        Kind::AdditiveSymbols,
        "calc(-1.5) N",
    );
}

#[test]
fn exact_literals_omitted_fixed_start_and_infinite_bounds_keep_their_contracts() {
    assert_descriptor("system: fixed;", Kind::System, "fixed");
    assert_descriptor(
        "system: fixed +0009007199254740993;",
        Kind::System,
        "fixed 9007199254740993",
    );
    assert_descriptor(
        "range: infinite -2, +0002 infinite;",
        Kind::Range,
        "infinite -2, 2 infinite",
    );
    assert_descriptor("pad: -0 '0';", Kind::Pad, "0 \"0\"");
    assert_descriptor(
        "additive-symbols: 2 X, 1 I, -0 N;",
        Kind::AdditiveSymbols,
        "2 X, 1 I, 0 N",
    );
}

#[test]
fn ordinary_negative_lengths_reversed_ranges_and_nondescending_weights_still_drop_whole_descriptors()
 {
    for (kind, input) in [
        (Kind::Pad, "-1 '0'"),
        (Kind::Range, "0 1, 3 2"),
        (Kind::AdditiveSymbols, "2 X, -1 N"),
        (Kind::AdditiveSymbols, "2 X, 2 I"),
        (Kind::AdditiveSymbols, "1 I, 2 X"),
    ] {
        let source = format!(
            "@counter-style Math {{ {}: {input}; suffix: '.'; }} .after {{}}",
            kind.css_name()
        );
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "one whole-descriptor diagnostic for {source}: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        let [CssRule::CounterStyle(rule), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("descriptor recovery must retain its rule and following sibling");
        };
        assert_eq!(rule.descriptor_specified_css(kind).unwrap(), "");
        assert_eq!(
            rule.descriptor_specified_css(Kind::Suffix).unwrap(),
            "\".\""
        );
    }
}

#[test]
fn malformed_and_wrong_domain_math_drop_only_the_descriptor_and_preserve_last_valid_math() {
    for (kind, accepted, expected, rejected) in [
        (
            Kind::System,
            "fixed calc(1 + 1)",
            "fixed calc(2)",
            "fixed calc(1px)",
        ),
        (
            Kind::Range,
            "calc(1 + 1) infinite",
            "calc(2) infinite",
            "0 1, calc(1%) infinite",
        ),
        (
            Kind::Pad,
            "calc(1 + 1) '0'",
            "calc(2) \"0\"",
            "calc(1 + ) '0'",
        ),
        (
            Kind::AdditiveSymbols,
            "calc(1 + 1) X, 1 I",
            "calc(2) X, 1 I",
            "2 X, calc(1s) I",
        ),
    ] {
        let source = format!(
            ".before {{}} @counter-style Math {{ {}: {accepted}; {}: {rejected}; suffix: '.'; }} .after {{}}",
            kind.css_name(),
            kind.css_name()
        );
        let report = parse_sheet(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!(
                "only the invalid member should recover for {source}: {:?}",
                report.diagnostics()
            );
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
        let [
            CssRule::Style(_),
            CssRule::CounterStyle(rule),
            CssRule::Style(_),
        ] = report.syntax().rules()
        else {
            panic!("both sibling rules must survive descriptor recovery");
        };
        assert_eq!(rule.descriptor_specified_css(kind).unwrap(), expected);
        assert_eq!(rule.descriptors().occurrences().count(), 2);
    }
}

#[test]
fn detached_checked_and_pending_reentry_admit_math_without_replacing_original_components() {
    for (kind, input) in [
        (Kind::System, "fixed calc(+0001 + 1)"),
        (Kind::Range, "calc(+0001 + 1) infinite"),
        (Kind::Pad, "calc(+0001 + 1) '0'"),
        (Kind::AdditiveSymbols, "calc(+0001 + 1) X, 1 I"),
    ] {
        let supplied = parse_component_values(input).unwrap();
        let detached = parse_counter_style_descriptor_value(input, kind);
        assert!(
            detached.is_clean(),
            "{kind:?}: {:?}",
            detached.diagnostics()
        );
        let detached = detached
            .syntax()
            .as_ref()
            .expect("ordinary descriptor value");
        assert_eq!(detached.components(), &supplied);
        assert_eq!(detached.origin().unwrap().source().as_str(), input);
        let checked = Value::try_from_components(kind, supplied.clone()).unwrap();
        assert_eq!(checked.kind(), kind);
        assert_eq!(checked.components(), &supplied);
        assert!(checked.origin().is_none());
        assert!(!matches!(checked.view(), View::Pending(_)));
        let pending =
            Value::try_from_components(kind, parse_component_values("env(choice)").unwrap())
                .unwrap();
        let completed = pending
            .reparse_after_substitution(supplied.clone())
            .unwrap();
        assert_eq!(completed.components(), &supplied);
        assert_eq!(completed.kind(), kind);
        assert!(completed.origin().is_none());
        assert!(matches!(pending.view(), View::Pending(_)));
        assert_eq!(checked.view(), completed.view());
    }
}

#[test]
fn strict_checked_math_rejects_recovered_closures_and_residual_substitution() {
    let pending =
        Value::try_from_components(Kind::System, parse_component_values("env(choice)").unwrap())
            .unwrap();
    for input in [
        "fixed calc(1 + 1",
        "fixed calc(env(choice) + 1)",
        "fixed calc(1px)",
    ] {
        assert!(
            pending
                .reparse_after_substitution(parse_component_values(input).unwrap())
                .is_err(),
            "{input}"
        );
    }
    assert!(
        Value::try_from_components(
            Kind::System,
            parse_component_values("fixed calc(1 + 1").unwrap()
        )
        .is_err()
    );
}

#[test]
fn math_projection_byte_failures_are_atomic_and_checked_graph_limits_are_cumulative() {
    let value = rule(
        "system: fixed calc(1 + 1); range: calc(1) infinite; pad: calc(2) '0'; additive-symbols: calc(2) X, 1 I;",
    );
    let retained = value.clone();
    let expected = value.to_specified_css().unwrap();
    assert_eq!(
        value
            .to_specified_css_with_limits(SerializationLimits::new(65536, 262144, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        value
            .to_specified_css_with_limits(SerializationLimits::new(
                65536,
                262144,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        SerializationError::ByteLimit
    );
    assert_eq!(value, retained);
    let input = parse_component_values("fixed calc(1 + 1)").unwrap();
    assert!(
        Value::try_from_components_with_limits(
            Kind::System,
            input.clone(),
            CssComponentValueLimits::try_new(256, 2, 65536).unwrap()
        )
        .is_err()
    );
    let pending =
        Value::try_from_components(Kind::System, parse_component_values("env(choice)").unwrap())
            .unwrap();
    assert!(
        pending
            .reparse_after_substitution_with_limits(
                input,
                CssComponentValueLimits::try_new(256, 65536, 1).unwrap()
            )
            .is_err()
    );
    assert!(matches!(pending.view(), View::Pending(_)));
}
