#![forbid(unsafe_code)]
//! Functional authored Speech 1 §§8.1–8.2 / 9.1–9.2 model contracts.
//! CSSOM 1 `serialize-a-css-value` supplies omission of an equivalent optional
//! second value; the retained authored choice and cumulative visit budget remain.
//! These tests accompany new APIs; public-boundary preimplementation RED is in
//! speech_pause_rest_lifecycle.rs.

use surgeist_css::*;

fn time(number: &str, unit: CssTimeUnit) -> CssSpeechBreak {
    CssSpeechBreak::try_time(CssTimeValue::from_literal(
        CssTimeLiteral::try_new(number, unit).unwrap(),
    ))
    .unwrap()
}

fn composed(before: CssSpeechBreak, after: Option<CssSpeechBreak>) -> CssSpeechBreakPair {
    CssSpeechBreakPair::try_new(before, after).unwrap()
}

fn parsed(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    report.syntax()[0].clone()
}

fn break_value(source: &CssDeclaration) -> &CssSpeechBreak {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::PauseBefore(value) => value.value(),
        CssKnownPropertyValueRef::PauseAfter(value) => value.value(),
        CssKnownPropertyValueRef::RestBefore(value) => value.value(),
        CssKnownPropertyValueRef::RestAfter(value) => value.value(),
        _ => panic!("pause/rest terminal wrapper"),
    }
}

fn pair(source: &CssDeclaration) -> &CssSpeechBreakPair {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Pause(value) => value.value(),
        CssKnownPropertyValueRef::Rest(value) => value.value(),
        _ => panic!("pause/rest pair wrapper"),
    }
}

#[test]
fn speech_break_models_keep_none_strength_and_explicit_zero_time_distinct() {
    let zero = time("-0", CssTimeUnit::Milliseconds);
    assert_ne!(zero, CssSpeechBreak::None);
    assert_eq!(zero.serialize_specified().unwrap(), "0s");
    assert_eq!(CssSpeechBreak::None.serialize_specified().unwrap(), "none");
    for (strength, css) in [
        (CssSpeechBreakStrength::XWeak, "x-weak"),
        (CssSpeechBreakStrength::Weak, "weak"),
        (CssSpeechBreakStrength::Medium, "medium"),
        (CssSpeechBreakStrength::Strong, "strong"),
        (CssSpeechBreakStrength::XStrong, "x-strong"),
    ] {
        let model = CssSpeechBreak::Strength(strength);
        assert_eq!(model.serialize_specified().unwrap(), css);
        for name in ["pause-before", "pause-after", "rest-before", "rest-after"] {
            assert_eq!(break_value(&parsed(name, css)), &model);
        }
    }
    let zero_none = composed(CssSpeechBreak::None, Some(zero));
    assert_eq!(zero_none.serialize_specified().unwrap(), "none 0s");
}

#[test]
fn checked_speech_time_composition_uses_the_owned_exact_duration_invariant() {
    for number in ["-1", "-1e-400"] {
        let value = CssTimeValue::from_literal(
            CssTimeLiteral::try_new(number, CssTimeUnit::Seconds).unwrap(),
        );
        assert_eq!(
            CssSpeechBreak::try_time(value).unwrap_err().kind(),
            &CssNumericConstructionErrorKind::OutOfRange
        );
    }
    for (number, unit, expected) in [
        ("250", CssTimeUnit::Milliseconds, "0.25s"),
        ("+3", CssTimeUnit::Seconds, "3s"),
        ("1e-400", CssTimeUnit::Seconds, "0s"),
        (
            "9007199254740993",
            CssTimeUnit::Milliseconds,
            "9007199254740.993s",
        ),
    ] {
        let value = time(number, unit);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let CssSpeechBreak::Time(duration) = &value else {
            panic!("time")
        };
        let literal = duration.time().literal().unwrap();
        assert_eq!(literal.numeric().representation(), number);
        assert_eq!(literal.unit(), unit);
        assert!(matches!(literal.origin(), CssValueOrigin::Programmatic));
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn checked_speech_pairs_reject_original_recovered_time_in_either_authored_position() {
    let report = parse_style_attribute("pause-before:calc(1s");
    assert!(!report.is_clean());
    assert!(validate_style_attribute("pause-before:calc(1s").is_err());
    let recovered = break_value(&report.syntax()[0]);
    let CssSpeechBreak::Time(duration) = recovered else {
        panic!("retained recovered calculation")
    };
    assert_eq!(
        CssSpeechBreak::try_time(duration.time().clone())
            .unwrap_err()
            .kind(),
        &CssNumericConstructionErrorKind::RecoveredComponent
    );
    for (before, after) in [
        (recovered.clone(), None),
        (CssSpeechBreak::None, Some(recovered.clone())),
    ] {
        assert_eq!(
            CssSpeechBreakPair::try_new(before, after)
                .unwrap_err()
                .kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
    }
    let parsed_pair = parse_style_attribute("rest:none calc(1s");
    assert!(!parsed_pair.is_clean());
    assert!(pair(&parsed_pair.syntax()[0]).authored_after().is_some());
}

#[test]
fn speech_pairs_retain_authored_second_presence_while_canonically_omitting_equal_values() {
    let weak = CssSpeechBreak::Strength(CssSpeechBreakStrength::Weak);
    let omitted = composed(weak.clone(), None);
    let explicit = composed(weak.clone(), Some(weak.clone()));
    assert_ne!(omitted, explicit);
    assert!(omitted.authored_after().is_none());
    assert_eq!(omitted.before(), omitted.after());
    assert_eq!(explicit.authored_after(), Some(&weak));
    assert_eq!(omitted.serialize_specified().unwrap(), "weak");
    assert_eq!(explicit.serialize_specified().unwrap(), "weak");
    for name in ["pause", "rest"] {
        let source = parsed(name, "WEAK/**/w\\65 ak");
        assert_eq!(pair(&source).before(), &weak);
        assert_eq!(pair(&source).authored_after(), Some(&weak));
        assert_eq!(pair(&source).serialize_specified().unwrap(), "weak");
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            "WEAK/**/w\\65 ak"
        );
    }
    let unequal = composed(weak, Some(time("250", CssTimeUnit::Milliseconds)));
    assert_eq!(unequal.serialize_specified().unwrap(), "weak 0.25s");
}

#[test]
fn speech_pair_time_omission_uses_exact_duration_equivalence_before_rounding() {
    for (before, after, expected) in [
        (
            time("1", CssTimeUnit::Seconds),
            time("1000", CssTimeUnit::Milliseconds),
            "1s",
        ),
        (
            time("1.0", CssTimeUnit::Seconds),
            time("+1e0", CssTimeUnit::Seconds),
            "1s",
        ),
        (
            time("1e-400", CssTimeUnit::Seconds),
            time("1000e-400", CssTimeUnit::Milliseconds),
            "0s",
        ),
        (
            time("1e-400", CssTimeUnit::Seconds),
            time("2e-400", CssTimeUnit::Seconds),
            "0s 0s",
        ),
        (
            time("1.0000001", CssTimeUnit::Seconds),
            time("1.0000002", CssTimeUnit::Seconds),
            "1s 1s",
        ),
        (
            time(
                "1e-999999999999999999999999999999999999999999",
                CssTimeUnit::Seconds,
            ),
            time(
                "1000e-999999999999999999999999999999999999999999",
                CssTimeUnit::Milliseconds,
            ),
            "0s",
        ),
    ] {
        let model = composed(before, Some(after));
        assert_eq!(model.serialize_specified().unwrap(), expected);
        assert!(model.authored_after().is_some());
    }
}

#[test]
fn speech_break_specified_serialization_counts_leaves_and_actual_canonical_bytes_atomically() {
    for (value, css) in [
        (CssSpeechBreak::None, "none"),
        (
            CssSpeechBreak::Strength(CssSpeechBreakStrength::XStrong),
            "x-strong",
        ),
        (time("250", CssTimeUnit::Milliseconds), "0.25s"),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    css.len()
                ))
                .unwrap(),
            css
        );
        for (limits, error) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, css.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, css.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, css.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                error
            );
        }
        assert_eq!(value.serialize_specified().unwrap(), css);
    }
}

#[test]
fn speech_pairs_share_one_budget_and_charge_suppressed_authored_after_children() {
    for (model, nodes, css) in [
        (composed(CssSpeechBreak::None, None), 2, "none"),
        (
            composed(CssSpeechBreak::None, Some(CssSpeechBreak::None)),
            3,
            "none",
        ),
        (
            composed(
                time("1", CssTimeUnit::Seconds),
                Some(time("1000", CssTimeUnit::Milliseconds)),
            ),
            3,
            "1s",
        ),
        (
            composed(
                CssSpeechBreak::Strength(CssSpeechBreakStrength::Strong),
                Some(time("250", CssTimeUnit::Milliseconds)),
            ),
            3,
            "strong 0.25s",
        ),
    ] {
        assert_eq!(
            model
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    nodes,
                    nodes,
                    css.len()
                ))
                .unwrap(),
            css
        );
        for (limits, error) in [
            (
                CssSpecifiedValueSerializationLimits::new(nodes - 1, nodes, css.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes - 1, css.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, css.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                model
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                error
            );
        }
        assert_eq!(model.serialize_specified().unwrap(), css);
        assert_eq!(model.authored_after().is_some(), nodes == 3);
    }
}

#[test]
fn speech_time_calculations_keep_math_phase_and_use_the_shared_time_projection_budget() {
    for name in ["pause", "rest"] {
        let source = parsed(name, "calc(-1s) calc(-1s)");
        let model = pair(&source);
        let CssSpeechBreak::Time(duration) = model.before() else {
            panic!("calculated time")
        };
        assert!(duration.time().literal().is_none());
        assert!(duration.time().calculation().is_some());
        assert_eq!(model.serialize_specified().unwrap(), "calc(-1s)");
        // Input traversal includes both retained calculation children even
        // though only one is emitted; default failure leaves the tree intact.
        assert_eq!(
            model
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    2, 100, 100
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            "calc(-1s) calc(-1s)"
        );
        assert_eq!(model.serialize_specified().unwrap(), "calc(-1s)");
    }
}

#[test]
fn speech_expanded_typed_payloads_serialize_in_before_after_order_with_original_occurrence() {
    for name in ["pause", "rest"] {
        let source = parsed(name, "250MS X-STRONG");
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("completed pair")
        };
        let [before, after] = values.items() else {
            panic!("ordered pair")
        };
        for (item, expected) in [(before, "0.25s"), (after, "x-strong")] {
            let value = match item.ordinary_value().unwrap().view() {
                CssLonghandValueRef::PauseBefore(value)
                | CssLonghandValueRef::PauseAfter(value)
                | CssLonghandValueRef::RestBefore(value)
                | CssLonghandValueRef::RestAfter(value) => value,
                _ => panic!("coupled speech terminal"),
            };
            assert_eq!(value.serialize_specified().unwrap(), expected);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
    }
}
