//! Source-defined temporal content grammar through input_numeric_constraints.
//! HTML 877d6146487f: Date/Month/Week/Time/LocalDateTime min/max/value grammar.

use surgeist_html::{
    Attribute, HtmlAttributeApplicability, HtmlAttributeObservation, HtmlInputNumericAuthorState,
    HtmlInputType, HtmlValueInterpreter, KnownHtmlAttribute as A, KnownHtmlElement as E,
};

fn temporal_content_authorship(state: HtmlInputType, spelling: &str) {
    let minimum = Attribute::new(A::Min.qualified_name(), spelling);
    let maximum = Attribute::new(A::Max.qualified_name(), spelling);
    let value = Attribute::new(A::Value.qualified_name(), spelling);
    let report = HtmlValueInterpreter::new(64)
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            Some(&minimum),
            Some(&maximum),
            None,
            Some(&value),
            state,
        )
        .unwrap();
    for (role, observed, raw) in [
        (A::Min, report.minimum(), &minimum),
        (A::Max, report.maximum(), &maximum),
        (A::Value, report.value(), &value),
    ] {
        assert_eq!(observed.attribute(), role);
        assert_eq!(observed.input_state(), state);
        assert_eq!(observed.raw_attribute(), Some(raw));
        assert_eq!(
            observed.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(
            observed.observation(),
            HtmlAttributeObservation::PresentNonempty
        );
        assert_eq!(
            observed.author(),
            HtmlInputNumericAuthorState::Conforming,
            "{state:?}/{role:?}/{spelling} must execute its source author grammar"
        );
    }
}

#[test]
fn date_content_author_grammar_is_applied() {
    // #date-state-(type=date):attr-input-min, valid Gregorian date.
    temporal_content_authorship(HtmlInputType::Date, "1970-01-01");
}

#[test]
fn month_content_author_grammar_is_applied() {
    // #month-state-(type=month):attr-input-min, valid month.
    temporal_content_authorship(HtmlInputType::Month, "1970-01");
}

#[test]
fn week_content_author_grammar_is_applied() {
    // #week-state-(type=week):attr-input-min, valid ISO week.
    temporal_content_authorship(HtmlInputType::Week, "1970-W01");
}

#[test]
fn time_content_author_grammar_is_applied() {
    // #time-state-(type=time):attr-input-min, valid clock time.
    temporal_content_authorship(HtmlInputType::Time, "00:00");
}

#[test]
fn local_date_time_content_author_grammar_is_applied() {
    // #local-date-and-time-state-(type=datetime-local):attr-input-min.
    temporal_content_authorship(HtmlInputType::LocalDateTime, "1970-01-01T00:00");
}

use surgeist_html::{
    DocumentString, HtmlAttributeContextAuthorState, HtmlInputNumericAttributeValue,
    HtmlInputNumericBoundOrigin as BoundOrigin, HtmlInputNumericCommonReport,
    HtmlInputNumericConstraintsError, HtmlInputNumericConstraintsOutcome as Outcome,
    HtmlInputNumericConstraintsReport, HtmlInputNumericLimitAuthorState as LimitAuthor,
    HtmlInputNumericStepBaseOrigin as BaseOrigin, HtmlInputNumericStepOrigin as StepOrigin,
    HtmlInputTemporalConstraints, HtmlInputTemporalCoordinate, HtmlInputTemporalCoordinateUnit,
    HtmlValueAuthorState, HtmlValueFailure, HtmlValueRecoveryState, HtmlValueRepresentationError,
    HtmlValueSyntaxError, Namespace, QualifiedName,
};

fn observe(
    state: HtmlInputType,
    min: Option<&str>,
    max: Option<&str>,
    step: Option<&str>,
    value: Option<&str>,
) -> HtmlInputNumericConstraintsReport {
    let min = min.map(|s| Attribute::new(A::Min.qualified_name(), s));
    let max = max.map(|s| Attribute::new(A::Max.qualified_name(), s));
    let step = step.map(|s| Attribute::new(A::Step.qualified_name(), s));
    let value = value.map(|s| Attribute::new(A::Value.qualified_name(), s));
    HtmlValueInterpreter::new(4096)
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            min.as_ref(),
            max.as_ref(),
            step.as_ref(),
            value.as_ref(),
            state,
        )
        .unwrap()
}
fn temporal(report: &HtmlInputNumericConstraintsReport) -> &HtmlInputTemporalConstraints {
    match report.outcome() {
        Outcome::ComputedTemporal(result) => result,
        other => panic!("expected exact temporal result: {other:?}"),
    }
}
fn coordinate(value: Option<&HtmlInputTemporalCoordinate>) -> Option<DocumentString> {
    value.map(|value| value.value().as_document_string())
}

#[test]
fn exact_coordinates_and_real_typed_common_reports_are_composed() {
    for (state, min, max, min_number, max_number) in [
        (
            HtmlInputType::Date,
            "1969-12-31",
            "1970-01-01",
            "-86400000",
            "0",
        ),
        (HtmlInputType::Month, "1969-12", "1970-01", "-1", "0"),
        (
            HtmlInputType::Week,
            "1970-W01",
            "1970-W02",
            "-259200000",
            "345600000",
        ),
        (
            HtmlInputType::Time,
            "00:00:00.000001",
            "00:00:01",
            "0.001",
            "1000",
        ),
        (
            HtmlInputType::LocalDateTime,
            "1969-12-31T23:59:59.999999999",
            "1970-01-01T00:00",
            "-0.000001",
            "0",
        ),
    ] {
        let report = observe(state, Some(min), Some(max), None, Some(max));
        let result = temporal(&report);
        assert_eq!(result.state(), state);
        assert_eq!(coordinate(result.minimum()), Some(min_number.into()));
        assert_eq!(coordinate(result.maximum()), Some(max_number.into()));
        assert_eq!(
            result.step_base().value().as_document_string(),
            DocumentString::from(min_number)
        );
        assert_eq!(result.step_base_origin(), BaseOrigin::MinimumContent);
        assert_eq!(result.minimum_origin(), BoundOrigin::Recovered);
        assert_eq!(result.maximum_origin(), BoundOrigin::Recovered);
        assert_eq!(result.limit_author(), LimitAuthor::Conforming);
        assert_eq!(
            result.unit(),
            if state == HtmlInputType::Month {
                HtmlInputTemporalCoordinateUnit::Months
            } else {
                HtmlInputTemporalCoordinateUnit::Milliseconds
            }
        );
        match (state, report.minimum().common().unwrap()) {
            (HtmlInputType::Date, HtmlInputNumericCommonReport::Date(_))
            | (HtmlInputType::Month, HtmlInputNumericCommonReport::Month(_))
            | (HtmlInputType::Week, HtmlInputNumericCommonReport::Week(_))
            | (HtmlInputType::Time, HtmlInputNumericCommonReport::Time(_))
            | (HtmlInputType::LocalDateTime, HtmlInputNumericCommonReport::LocalDateTime(_)) => (),
            other => panic!("incorrect grammar dispatch {other:?}"),
        }
        assert!(
            matches!(report.minimum().converted(), Some(HtmlInputNumericAttributeValue::Temporal(value)) if value.state() == state)
        );
        assert_eq!(
            report.minimum().context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
}

#[test]
fn temporal_defaults_are_exact_without_any_successful_content() {
    for (state, step, base, origin) in [
        (HtmlInputType::Date, "86400000", "0", BaseOrigin::Zero),
        (HtmlInputType::Month, "1", "0", BaseOrigin::Zero),
        (
            HtmlInputType::Week,
            "604800000",
            "-259200000",
            BaseOrigin::StateDefault,
        ),
        (HtmlInputType::Time, "60000", "0", BaseOrigin::Zero),
        (HtmlInputType::LocalDateTime, "60000", "0", BaseOrigin::Zero),
    ] {
        let report = observe(state, None, None, None, None);
        let result = temporal(&report);
        assert_eq!(result.minimum(), None);
        assert_eq!(result.maximum(), None);
        assert_eq!(result.minimum_origin(), BoundOrigin::MissingFallback);
        assert_eq!(result.maximum_origin(), BoundOrigin::MissingFallback);
        assert_eq!(
            result.allowed_step().unwrap().as_document_string(),
            DocumentString::from(step)
        );
        assert_eq!(result.step_origin(), StepOrigin::MissingDefault);
        assert_eq!(
            result.step_base().value().as_document_string(),
            DocumentString::from(base)
        );
        assert_eq!(result.step_base_origin(), origin);
        assert_eq!(result.limit_author(), LimitAuthor::NotRequired);
        assert!(!result.has_reversed_range());
    }
}

#[test]
fn min_value_and_week_default_base_precedence_use_actual_conversions() {
    for (state, min, content, min_number, value_number) in [
        (
            HtmlInputType::Date,
            "1969-12-31",
            "1970-01-02",
            "-86400000",
            "86400000",
        ),
        (HtmlInputType::Month, "1969-12", "1970-02", "-1", "1"),
        (
            HtmlInputType::Week,
            "1970-W01",
            "1970-W02",
            "-259200000",
            "345600000",
        ),
        (HtmlInputType::Time, "00:00", "00:01", "0", "60000"),
        (
            HtmlInputType::LocalDateTime,
            "1969-12-31T23:59",
            "1970-01-01T00:01",
            "-60000",
            "60000",
        ),
    ] {
        let report = observe(state, Some(min), None, Some("AnY"), Some(content));
        assert_eq!(
            temporal(&report).step_base().value().as_document_string(),
            DocumentString::from(min_number)
        );
        assert_eq!(
            temporal(&report).step_base_origin(),
            BaseOrigin::MinimumContent
        );
        for min in [None, Some("bad"), Some("")] {
            let report = observe(state, min, None, Some("any"), Some(content));
            assert_eq!(temporal(&report).minimum(), None);
            assert_eq!(
                temporal(&report).step_base().value().as_document_string(),
                DocumentString::from(value_number)
            );
            assert_eq!(
                temporal(&report).step_base_origin(),
                BaseOrigin::ValueContent
            );
            assert_eq!(temporal(&report).allowed_step(), None);
        }
    }
    let report = observe(
        HtmlInputType::Week,
        Some("bad"),
        None,
        Some("any"),
        Some("bad"),
    );
    assert_eq!(
        temporal(&report).step_base_origin(),
        BaseOrigin::StateDefault
    );
    assert_eq!(
        temporal(&report).step_base().value().as_document_string(),
        DocumentString::from("-259200000")
    );
}

#[test]
fn empty_value_author_exception_does_not_supply_a_base_or_a_bound() {
    for state in [
        HtmlInputType::Date,
        HtmlInputType::Month,
        HtmlInputType::Week,
        HtmlInputType::Time,
        HtmlInputType::LocalDateTime,
    ] {
        let report = observe(state, Some(""), Some(""), None, Some(""));
        assert!(matches!(
            report.minimum().author(),
            HtmlInputNumericAuthorState::Nonconforming(_)
        ));
        assert!(matches!(
            report.maximum().author(),
            HtmlInputNumericAuthorState::Nonconforming(_)
        ));
        assert_eq!(
            report.value().author(),
            HtmlInputNumericAuthorState::Conforming
        );
        assert_eq!(report.value().converted(), None);
        assert_eq!(temporal(&report).minimum(), None);
        assert_eq!(temporal(&report).maximum(), None);
        assert_eq!(
            temporal(&report).minimum_origin(),
            BoundOrigin::ParseFailureFallback
        );
        assert_eq!(
            temporal(&report).step_base_origin(),
            if state == HtmlInputType::Week {
                BaseOrigin::StateDefault
            } else {
                BaseOrigin::Zero
            }
        );
    }
}

#[test]
fn step_scaling_preserves_binary64_bits_and_prefix_recovery_evidence() {
    for state in [HtmlInputType::Time, HtmlInputType::LocalDateTime] {
        for (spelling, expected, author) in [
            ("0.125", "125", HtmlInputNumericAuthorState::Conforming),
            (
                "0.1",
                "100.0000000000000055511151231257827021181583404541015625",
                HtmlInputNumericAuthorState::Conforming,
            ),
            (
                "0.125tail",
                "125",
                HtmlInputNumericAuthorState::Nonconforming(HtmlValueFailure::Syntax(
                    HtmlValueSyntaxError::InvalidNumericSyntax,
                )),
            ),
        ] {
            let report = observe(state, None, None, Some(spelling), None);
            assert_eq!(
                temporal(&report)
                    .allowed_step()
                    .unwrap()
                    .as_document_string(),
                DocumentString::from(expected)
            );
            assert_eq!(temporal(&report).step_origin(), StepOrigin::Recovered);
            assert_eq!(report.step().author(), author);
            assert!(matches!(
                report.step().common(),
                Some(HtmlInputNumericCommonReport::FloatingPoint(_))
            ));
        }
    }
    for (state, expected) in [
        (HtmlInputType::Date, "10800000"),
        (HtmlInputType::Month, "0.125"),
        (HtmlInputType::Week, "75600000"),
    ] {
        let report = observe(state, None, None, Some("0.125"), None);
        assert_eq!(
            temporal(&report)
                .allowed_step()
                .unwrap()
                .as_document_string(),
            DocumentString::from(expected)
        );
    }
}

#[test]
fn finite_step_extremes_survive_actual_raw_parsing_and_exact_scaling() {
    let tiny = observe(HtmlInputType::Time, None, None, Some("5e-324"), None);
    let Some(HtmlInputNumericCommonReport::FloatingPoint(common)) = tiny.step().common() else {
        panic!("floating step evidence")
    };
    assert!(
        matches!(common.recovery(), HtmlValueRecoveryState::Value(value) if value.value() == f64::from_bits(1))
    );
    assert_eq!(temporal(&tiny).step_origin(), StepOrigin::Recovered);
    let scaled = temporal(&tiny).allowed_step().unwrap();
    assert_eq!(scaled.integer_part().decimal_digits(), "0");
    // 2^-1074 * 1000 = 5^1074 / 10^1071; its numerator is odd and ends in 5.
    assert_eq!(scaled.fractional_digits().unwrap().len(), 1071);
    assert!(scaled.fractional_digits().unwrap().ends_with('5'));
    let large = observe(
        HtmlInputType::Time,
        None,
        None,
        Some("1.7976931348623157e308"),
        None,
    );
    assert_eq!(
        large.step().author(),
        HtmlInputNumericAuthorState::Conforming
    );
    assert_eq!(temporal(&large).step_origin(), StepOrigin::Recovered);
    // Exact IEEE binary64 MAX = (2^53 - 1) * 2^971, multiplied by 1000.
    assert_eq!(
        temporal(&large)
            .allowed_step()
            .unwrap()
            .as_document_string(),
        DocumentString::from(
            "179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368000"
        )
    );
}

#[test]
fn step_rejections_select_scaled_defaults_and_preserve_author_failure_causes() {
    let overflow =
        HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow);
    for (state, default) in [
        (HtmlInputType::Date, "86400000"),
        (HtmlInputType::Month, "1"),
        (HtmlInputType::Week, "604800000"),
        (HtmlInputType::Time, "60000"),
        (HtmlInputType::LocalDateTime, "60000"),
    ] {
        for (spelling, origin, author) in [
            (
                "0",
                StepOrigin::NonPositiveDefault,
                HtmlInputNumericAuthorState::StepMustBePositive,
            ),
            (
                "-0",
                StepOrigin::NonPositiveDefault,
                HtmlInputNumericAuthorState::StepMustBePositive,
            ),
            (
                "-2",
                StepOrigin::NonPositiveDefault,
                HtmlInputNumericAuthorState::StepMustBePositive,
            ),
            (
                "1e-999",
                StepOrigin::NonPositiveDefault,
                HtmlInputNumericAuthorState::StepMustBePositive,
            ),
            (
                "1e999",
                StepOrigin::ParseFailureDefault,
                HtmlInputNumericAuthorState::Nonconforming(overflow),
            ),
            (
                " any ",
                StepOrigin::ParseFailureDefault,
                HtmlInputNumericAuthorState::Nonconforming(HtmlValueFailure::Syntax(
                    HtmlValueSyntaxError::InvalidNumericSyntax,
                )),
            ),
        ] {
            let report = observe(state, None, None, Some(spelling), None);
            assert_eq!(
                temporal(&report)
                    .allowed_step()
                    .unwrap()
                    .as_document_string(),
                DocumentString::from(default)
            );
            assert_eq!(temporal(&report).step_origin(), origin);
            assert_eq!(report.step().author(), author);
            if spelling == "1e999" {
                let Some(HtmlInputNumericCommonReport::FloatingPoint(common)) =
                    report.step().common()
                else {
                    panic!("floating step evidence")
                };
                assert_eq!(
                    common.author(),
                    &HtmlValueAuthorState::Unrepresentable(
                        HtmlValueRepresentationError::FloatingPointOverflow
                    )
                );
                assert_eq!(common.recovery(), &HtmlValueRecoveryState::Failed(overflow));
            }
        }
    }
}

#[test]
fn time_reversed_ranges_are_periodic_and_other_inverse_ranges_are_author_invalid() {
    let report = observe(
        HtmlInputType::Time,
        Some("23:00"),
        Some("01:00"),
        None,
        None,
    );
    let result = temporal(&report);
    assert!(result.has_periodic_domain());
    assert!(result.has_reversed_range());
    assert_eq!(result.limit_author(), LimitAuthor::Conforming);
    assert_eq!(coordinate(result.minimum()), Some("82800000".into()));
    assert_eq!(coordinate(result.maximum()), Some("3600000".into()));
    for (state, min, max) in [
        (HtmlInputType::Date, "1970-01-02", "1970-01-01"),
        (HtmlInputType::Month, "1970-02", "1970-01"),
        (HtmlInputType::Week, "1970-W02", "1970-W01"),
        (
            HtmlInputType::LocalDateTime,
            "1970-01-01T00:00:00.000000002",
            "1970-01-01T00:00:00.000000001",
        ),
    ] {
        let report = observe(state, Some(min), Some(max), None, None);
        assert_eq!(
            temporal(&report).limit_author(),
            LimitAuthor::MinimumExceedsMaximum
        );
        assert!(!temporal(&report).has_reversed_range());
        assert!(!temporal(&report).has_periodic_domain());
        let equal = observe(state, Some(min), Some(min), None, None);
        assert_eq!(temporal(&equal).limit_author(), LimitAuthor::Conforming);
    }
}

#[test]
fn large_years_long_fractions_and_local_space_are_not_projected_or_normalized() {
    let report = observe(
        HtmlInputType::Date,
        Some("340282366920938463463374607431768211456-01-01"),
        Some("340282366920938463463374607431768211455-01-01"),
        None,
        None,
    );
    assert_eq!(
        report.minimum().author(),
        HtmlInputNumericAuthorState::Conforming
    );
    assert_eq!(
        temporal(&report).limit_author(),
        LimitAuthor::MinimumExceedsMaximum
    );
    assert!(
        temporal(&report)
            .minimum()
            .unwrap()
            .value()
            .magnitude()
            .integer_part()
            .decimal_digits()
            .len()
            > 40
    );
    let report = observe(
        HtmlInputType::LocalDateTime,
        Some("1969-12-31 23:59:59.999999999"),
        None,
        None,
        None,
    );
    assert!(matches!(
        report.minimum().author(),
        HtmlInputNumericAuthorState::Nonconforming(HtmlValueFailure::Syntax(
            HtmlValueSyntaxError::Temporal(_)
        ))
    ));
    assert_eq!(
        coordinate(temporal(&report).minimum()),
        Some("-0.000001".into())
    );
    assert_eq!(
        report.minimum().raw().unwrap(),
        &DocumentString::from("1969-12-31 23:59:59.999999999")
    );
}

#[test]
fn exact_names_surrogates_and_admission_remain_owned_by_the_shared_front_door() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    let min = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "min"),
        DocumentString::from_utf16(vec![0xd800, 50]),
    );
    let report = HtmlValueInterpreter::new(2)
        .input_numeric_constraints(&element, Some(&min), None, None, None, HtmlInputType::Time)
        .unwrap();
    assert_eq!(report.minimum().element(), &element);
    assert_eq!(report.minimum().raw_attribute(), Some(&min));
    assert!(matches!(
        report.minimum().author(),
        HtmlInputNumericAuthorState::Nonconforming(_)
    ));
    assert_eq!(temporal(&report).minimum(), None);
    let wrong = Attribute::new(QualifiedName::new(Namespace::svg(), None, "min"), "bad");
    let error = HtmlValueInterpreter::new(2)
        .input_numeric_constraints(
            &element,
            Some(&wrong),
            None,
            None,
            None,
            HtmlInputType::Time,
        )
        .unwrap_err();
    assert!(
        matches!(error, HtmlInputNumericConstraintsError::Admission(error) if error.required() == 3)
    );
    let report = HtmlValueInterpreter::new(3)
        .input_numeric_constraints(
            &element,
            Some(&wrong),
            None,
            None,
            None,
            HtmlInputType::Time,
        )
        .unwrap();
    assert_eq!(
        report.outcome(),
        &Outcome::Unapplied {
            attribute: A::Min,
            reason: HtmlAttributeApplicability::DifferentAttribute
        }
    );
    assert_eq!(report.minimum().raw_attribute(), Some(&wrong));
}
