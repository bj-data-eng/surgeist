//! Literal requirements from HTML #the-progress-element; captured effective
//! quantities have their independent existing tests in semantics_ranges.
use crate::html::support::known_attribute as attr;
use surgeist_html::*;

fn read(value: Option<&str>, maximum: Option<&str>) -> HtmlProgressAttributeReport {
    let value = value.map(|s| attr(KnownHtmlAttribute::Value, s));
    let maximum = maximum.map(|s| attr(KnownHtmlAttribute::Max, s));
    HtmlValueInterpreter::new(4096)
        .progress_attributes(
            &KnownHtmlElement::Progress.qualified_name(),
            value.as_ref(),
            maximum.as_ref(),
        )
        .unwrap()
}
fn recovered(slot: &HtmlProgressNumberReport, expected: f64) {
    assert!(
        matches!(slot.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.value() == expected)
    );
}
fn syntax(slot: &HtmlProgressNumberReport) {
    assert_eq!(
        slot.author(),
        HtmlProgressNumberAuthorState::Nonconforming(HtmlProgressNumberAuthorError::Syntax(
            HtmlValueSyntaxError::InvalidNumericSyntax
        ))
    );
    assert_eq!(
        slot.common().unwrap().author(),
        &HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::InvalidNumericSyntax)
    );
}

#[test]
fn absence_retains_missing_without_materializing_effective_quantities() {
    let report = read(None, None);
    for slot in [report.value(), report.maximum()] {
        assert_eq!(
            slot.attribute().observation(),
            HtmlAttributeObservation::Missing
        );
        assert_eq!(slot.author(), HtmlProgressNumberAuthorState::Missing);
        assert_eq!(
            slot.common().unwrap().author(),
            &HtmlValueAuthorState::Missing
        );
        assert_eq!(
            slot.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Missing
        );
    }
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::NotRequired
    );
    let report = read(None, Some("bad"));
    syntax(report.maximum());
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::NotRequired
    );
}

#[test]
fn present_empty_and_whitespace_fail_grammar_instead_of_becoming_missing() {
    for s in ["", " ", "\t\n"] {
        let report = read(Some(s), Some(s));
        syntax(report.value());
        syntax(report.maximum());
        assert_eq!(
            report.upper_bound(),
            HtmlProgressUpperBoundState::Unavailable(
                HtmlProgressUpperBoundUnavailable::ValueSyntax
            )
        );
        assert_eq!(
            report.value().attribute().raw(),
            Some(&DocumentString::from(s))
        );
        assert_eq!(
            report.value().attribute().observation(),
            if s.is_empty() {
                HtmlAttributeObservation::PresentEmpty
            } else {
                HtmlAttributeObservation::PresentNonempty
            }
        );
    }
}

#[test]
fn value_nonnegativity_and_maximum_positivity_are_independent() {
    for s in ["0", "-0", "0e999", "-0.000"] {
        let report = read(Some(s), Some(s));
        assert_eq!(
            report.value().author(),
            HtmlProgressNumberAuthorState::Conforming
        );
        assert_eq!(
            report.maximum().author(),
            HtmlProgressNumberAuthorState::Nonconforming(
                HtmlProgressNumberAuthorError::NonpositiveMaximum
            )
        );
        assert_eq!(
            report.upper_bound(),
            HtmlProgressUpperBoundState::Conforming
        );
    }
    let report = read(Some("-0.1"), Some("-1"));
    assert_eq!(
        report.value().author(),
        HtmlProgressNumberAuthorState::Nonconforming(HtmlProgressNumberAuthorError::NegativeValue)
    );
    assert_eq!(
        report.maximum().author(),
        HtmlProgressNumberAuthorState::Nonconforming(
            HtmlProgressNumberAuthorError::NonpositiveMaximum
        )
    );
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::ValueAboveMaximum
    );
}

#[test]
fn upper_bound_uses_authored_maximum_or_one_only_for_actual_absence() {
    for (value, maximum, expected) in [
        ("0", None, HtmlProgressUpperBoundState::Conforming),
        ("1", None, HtmlProgressUpperBoundState::Conforming),
        ("2", None, HtmlProgressUpperBoundState::ValueAboveMaximum),
        ("2", Some("3"), HtmlProgressUpperBoundState::Conforming),
        ("3", Some("3"), HtmlProgressUpperBoundState::Conforming),
        (
            "4",
            Some("3"),
            HtmlProgressUpperBoundState::ValueAboveMaximum,
        ),
    ] {
        let report = read(Some(value), maximum);
        assert_eq!(
            report.value().author(),
            HtmlProgressNumberAuthorState::Conforming
        );
        assert_eq!(report.upper_bound(), expected);
    }
}

#[test]
fn invalid_present_maximum_cannot_supply_an_authored_default() {
    for maximum in ["", "bad", "+3", "3tail"] {
        let report = read(Some("2"), Some(maximum));
        syntax(report.maximum());
        assert_eq!(
            report.upper_bound(),
            HtmlProgressUpperBoundState::Unavailable(
                HtmlProgressUpperBoundUnavailable::MaximumSyntax
            )
        );
    }
    recovered(read(Some("2"), Some("3tail")).maximum(), 3.0);
}

#[test]
fn exact_authored_order_survives_binary64_rounding_to_equal_values() {
    let report = read(Some("1.00000000000000001"), Some("1"));
    recovered(report.value(), 1.0);
    recovered(report.maximum(), 1.0);
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::ValueAboveMaximum
    );
    assert_eq!(
        read(Some("10e-1"), Some("1.00000000000000001")).upper_bound(),
        HtmlProgressUpperBoundState::Conforming
    );
}

#[test]
fn underflow_keeps_mathematical_signs_independently_of_common_zero() {
    let report = read(Some("0"), Some("1e-999"));
    assert_eq!(
        report.maximum().author(),
        HtmlProgressNumberAuthorState::Conforming
    );
    recovered(report.maximum(), 0.0);
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::Conforming
    );
    let report = read(Some("-1e-999"), None);
    recovered(report.value(), 0.0);
    assert_eq!(
        report.value().author(),
        HtmlProgressNumberAuthorState::Nonconforming(HtmlProgressNumberAuthorError::NegativeValue)
    );
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::Conforming
    );
}

#[test]
fn grammatical_overflow_still_allows_exact_author_constraints() {
    let report = read(Some("1e999"), Some("1e999"));
    for slot in [report.value(), report.maximum()] {
        assert_eq!(slot.author(), HtmlProgressNumberAuthorState::Conforming);
        assert_eq!(
            slot.common().unwrap().author(),
            &HtmlValueAuthorState::Unrepresentable(
                HtmlValueRepresentationError::FloatingPointOverflow
            )
        );
        assert_eq!(
            slot.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Failed(HtmlValueFailure::Representation(
                HtmlValueRepresentationError::FloatingPointOverflow
            ))
        );
    }
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::Conforming
    );
    assert_eq!(
        read(Some("1e999"), None).upper_bound(),
        HtmlProgressUpperBoundState::ValueAboveMaximum
    );
}

#[test]
fn arbitrary_exponents_compare_without_expansion_or_machine_integer_caps() {
    let e = "99999999999999999999999999999999999999999999999999";
    assert_eq!(
        read(Some(&format!("1e{e}")), Some(&format!("2e{e}"))).upper_bound(),
        HtmlProgressUpperBoundState::Conforming
    );
    let report = read(Some(&format!("2e-{e}")), Some(&format!("1e-{e}")));
    assert_eq!(
        report.upper_bound(),
        HtmlProgressUpperBoundState::ValueAboveMaximum
    );
    assert_eq!(
        report.maximum().author(),
        HtmlProgressNumberAuthorState::Conforming
    );
}

#[test]
fn strict_failures_preserve_common_prefix_recovery_and_original_utf16() {
    for text in ["+2tail", " 2", "2e+", "2 "] {
        let report = read(Some(text), Some("3"));
        syntax(report.value());
        recovered(report.value(), 2.0);
        assert_eq!(
            report.upper_bound(),
            HtmlProgressUpperBoundState::Unavailable(
                HtmlProgressUpperBoundUnavailable::ValueSyntax
            )
        );
        assert_eq!(
            report.value().attribute().raw(),
            Some(&DocumentString::from(text))
        );
    }
    for text in ["NaN", "Infinity", "\u{a0}2", "١"] {
        syntax(read(Some(text), None).value());
    }
    let raw = attr(
        KnownHtmlAttribute::Value,
        DocumentString::from_utf16(vec![u16::from(b'2'), 0xD800]),
    );
    let report = HtmlValueInterpreter::new(2)
        .progress_attributes(
            &KnownHtmlElement::Progress.qualified_name(),
            Some(&raw),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    syntax(report.value());
    recovered(report.value(), 2.0);
    assert_eq!(report.value().attribute().raw_attribute(), Some(&raw));
}

#[test]
fn expanded_names_preserve_prefixes_and_do_not_run_unapplied_grammars() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "progress");
    let value = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "value"),
        "2",
    );
    let maximum = attr(KnownHtmlAttribute::Max, "3");
    let interpreter = HtmlValueInterpreter::new(100);
    let report = interpreter
        .progress_attributes(&element, Some(&value), Some(&maximum))
        .unwrap();
    assert_eq!(report.value().attribute().element(), &element);
    assert_eq!(report.value().attribute().raw_attribute(), Some(&value));
    assert_eq!(report.value().attribute().inventory_rows(), &[177]);
    assert_eq!(report.maximum().attribute().inventory_rows(), &[93]);
    assert!(
        report
            .value()
            .attribute()
            .source_clause()
            .ends_with("#attr-progress-value")
    );
    assert!(
        report
            .maximum()
            .attribute()
            .source_clause()
            .ends_with("#attr-progress-max")
    );
    for element in [
        KnownHtmlElement::Meter.qualified_name(),
        QualifiedName::new(Namespace::none(), None, "progress"),
        QualifiedName::new(Namespace::html(), None, "PROGRESS"),
    ] {
        let report = interpreter
            .progress_attributes(&element, Some(&value), Some(&maximum))
            .unwrap();
        for slot in [report.value(), report.maximum()] {
            assert!(slot.common().is_none());
            assert_eq!(slot.author(), HtmlProgressNumberAuthorState::Unapplied);
        }
        assert_eq!(report.upper_bound(), HtmlProgressUpperBoundState::Unapplied);
    }
}

#[test]
fn wrong_maximum_roles_are_not_absence_or_fallback_bounds() {
    let value = attr(KnownHtmlAttribute::Value, "2");
    for name in [
        KnownHtmlAttribute::Value.qualified_name(),
        QualifiedName::new(Namespace::html(), None, "max"),
        QualifiedName::new(Namespace::none(), None, "MAX"),
    ] {
        let maximum = Attribute::new(name, "3");
        let report = HtmlValueInterpreter::new(10)
            .progress_attributes(
                &KnownHtmlElement::Progress.qualified_name(),
                Some(&value),
                Some(&maximum),
            )
            .unwrap();
        assert_eq!(
            report.maximum().attribute().applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.maximum().attribute().raw_attribute(), Some(&maximum));
        assert!(report.maximum().common().is_none());
        assert_eq!(report.upper_bound(), HtmlProgressUpperBoundState::Unapplied);
    }
}

#[test]
fn each_original_slot_is_admitted_before_wrong_role_or_element_processing() {
    let value = attr(KnownHtmlAttribute::Value, "0");
    let wrong_maximum = attr(KnownHtmlAttribute::Title, "12345");
    let error = HtmlValueInterpreter::new(4)
        .progress_attributes(
            &KnownHtmlElement::Div.qualified_name(),
            Some(&value),
            Some(&wrong_maximum),
        )
        .unwrap_err();
    assert_eq!((error.capacity(), error.required()), (4, 5));
    let raw = attr(
        KnownHtmlAttribute::Value,
        DocumentString::from_utf16(vec![0xD83D, 0xDE00, 0xD800]),
    );
    let error = HtmlValueInterpreter::new(2)
        .progress_attributes(
            &KnownHtmlElement::Div.qualified_name(),
            Some(&raw),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap_err();
    assert_eq!((error.capacity(), error.required()), (2, 3));
    assert!(
        HtmlValueInterpreter::new(3)
            .progress_attributes(
                &KnownHtmlElement::Div.qualified_name(),
                Some(&raw),
                None::<&surgeist_html::Attribute>
            )
            .is_ok()
    );
    assert!(
        HtmlValueInterpreter::new(1)
            .progress_attributes(
                &KnownHtmlElement::Progress.qualified_name(),
                Some(&value),
                Some(&attr(KnownHtmlAttribute::Max, "1"))
            )
            .is_ok()
    );
}

fn number(value: f64) -> HtmlFloatingPointNumber {
    HtmlFloatingPointNumber::try_from(value).unwrap()
}

#[test]
fn native_pairs_check_all_emitted_constraints_and_preserve_absence() {
    for (value, maximum, expected) in [
        (
            Some(-0.1),
            Some(3.0),
            HtmlProgressNativeError::NegativeValue,
        ),
        (None, Some(0.0), HtmlProgressNativeError::NonpositiveMaximum),
        (
            Some(0.0),
            Some(-0.0),
            HtmlProgressNativeError::NonpositiveMaximum,
        ),
        (
            None,
            Some(-1.0),
            HtmlProgressNativeError::NonpositiveMaximum,
        ),
        (Some(2.0), None, HtmlProgressNativeError::ValueAboveMaximum),
        (
            Some(4.0),
            Some(3.0),
            HtmlProgressNativeError::ValueAboveMaximum,
        ),
    ] {
        assert_eq!(
            HtmlProgressAttributes::new(value.map(number), maximum.map(number)),
            Err(expected)
        );
    }
    let absent = HtmlProgressAttributes::new(None, None).unwrap();
    assert_eq!(absent.value(), None);
    assert_eq!(absent.maximum(), None);
    assert!(absent.into_attributes().is_empty());
    assert_eq!(
        HtmlProgressAttributes::new(None, Some(number(3.0)))
            .unwrap()
            .into_attributes(),
        vec![attr(KnownHtmlAttribute::Max, "3")]
    );
    assert!(HtmlFloatingPointNumber::try_from(f64::INFINITY).is_err());
    assert!(HtmlFloatingPointNumber::try_from(f64::NAN).is_err());
}

#[test]
fn native_canonical_pairs_read_back_with_all_author_requirements_satisfied() {
    for (value, maximum) in [
        (0.5, None),
        (-0.0, None),
        (2.0, Some(3.0)),
        (f64::MIN_POSITIVE, Some(f64::MAX)),
        (f64::MAX, Some(f64::MAX)),
        (f64::from_bits(1), Some(f64::from_bits(1))),
    ] {
        let native = HtmlProgressAttributes::new(Some(number(value)), maximum.map(number)).unwrap();
        assert_eq!(native.value(), Some(number(value)));
        assert_eq!(native.maximum(), maximum.map(number));
        let attributes = native.into_attributes();
        assert_eq!(
            attributes[0].name(),
            &KnownHtmlAttribute::Value.qualified_name()
        );
        if maximum.is_some() {
            assert_eq!(
                attributes[1].name(),
                &KnownHtmlAttribute::Max.qualified_name()
            );
        }
        let report = HtmlValueInterpreter::new(4096)
            .progress_attributes(
                &KnownHtmlElement::Progress.qualified_name(),
                attributes.first(),
                attributes.get(1),
            )
            .unwrap();
        assert_eq!(
            report.value().author(),
            HtmlProgressNumberAuthorState::Conforming
        );
        assert_eq!(
            report.maximum().author(),
            if maximum.is_some() {
                HtmlProgressNumberAuthorState::Conforming
            } else {
                HtmlProgressNumberAuthorState::Missing
            }
        );
        assert_eq!(
            report.upper_bound(),
            HtmlProgressUpperBoundState::Conforming
        );
    }
    assert_eq!(
        HtmlProgressAttributes::new(Some(number(-0.0)), None)
            .unwrap()
            .into_attributes(),
        vec![attr(KnownHtmlAttribute::Value, "0")]
    );
}
