use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeContextAuthorState,
    HtmlAttributeContextConformanceError, HtmlAttributeObservation, HtmlFloatingPointNumber,
    HtmlInputNumericAttributeReport, HtmlInputNumericAttributeValue,
    HtmlInputNumericAuthorState as Author, HtmlInputNumericBoundOrigin as BoundOrigin,
    HtmlInputNumericCommonReport, HtmlInputNumericConstraints, HtmlInputNumericConstraintsError,
    HtmlInputNumericConstraintsOutcome as Outcome, HtmlInputNumericConstraintsReport,
    HtmlInputNumericLimitAuthorState as LimitAuthor, HtmlInputNumericStepBaseOrigin as BaseOrigin,
    HtmlInputNumericStepOrigin as StepOrigin, HtmlInputStep, HtmlInputType as State,
    HtmlNonPositiveInputStepError, HtmlValueAuthorState, HtmlValueFailure, HtmlValueInterpreter,
    HtmlValueRecoveryState, HtmlValueReport, HtmlValueRepresentationError, HtmlValueSyntaxError,
    KnownHtmlAttribute as A, KnownHtmlElement as E, Namespace, QualifiedName,
};

#[test]
fn converted_numeric_selection_preserves_defaults_recovery_and_state_domain() {
    use surgeist_html::{
        HtmlInputNumericContent as Content, HtmlInputNumericStep as Step,
        select_input_numeric_constraints as select,
    };
    let value = |n| HtmlFloatingPointNumber::try_from(n).unwrap();
    let number = select(
        Content::Absent,
        Content::Absent,
        Step::Absent,
        Content::Absent,
        State::Number,
    )
    .unwrap();
    assert_eq!(number.minimum(), None);
    assert_eq!(number.maximum(), None);
    assert_eq!(number.allowed_step().unwrap().value(), 1.0);
    assert_eq!(number.step_base().value(), 0.0);
    assert_eq!(number.step_origin(), StepOrigin::MissingDefault);

    let range = select(
        Content::Failed,
        Content::Absent,
        Step::Value(value(-2.0)),
        Content::Value(value(7.0)),
        State::Range,
    )
    .unwrap();
    assert_eq!(range.minimum().unwrap().value(), 0.0);
    assert_eq!(range.maximum().unwrap().value(), 100.0);
    assert_eq!(range.minimum_origin(), BoundOrigin::ParseFailureFallback);
    assert_eq!(range.allowed_step().unwrap().value(), 1.0);
    assert_eq!(range.step_origin(), StepOrigin::NonPositiveDefault);
    assert_eq!(range.step_base().value(), 7.0);
    assert_eq!(range.step_base_origin(), BaseOrigin::ValueContent);

    let reversed = select(
        Content::Value(value(10.0)),
        Content::Value(value(-5.0)),
        Step::Any,
        Content::Value(value(99.0)),
        State::Range,
    )
    .unwrap();
    assert_eq!(reversed.minimum().unwrap().value(), 10.0);
    assert_eq!(reversed.maximum().unwrap().value(), -5.0);
    assert_eq!(reversed.allowed_step(), None);
    assert_eq!(reversed.step_origin(), StepOrigin::Any);
    assert_eq!(reversed.step_base().value(), 10.0);
    assert_eq!(reversed.limit_author(), LimitAuthor::MinimumExceedsMaximum);
    for state in [State::Text, State::Date, State::Checkbox, State::Image] {
        assert!(
            select(
                Content::Absent,
                Content::Absent,
                Step::Absent,
                Content::Absent,
                state
            )
            .is_none()
        );
    }
}

#[test]
fn step_any_observes_only_the_selected_ascii_keyword_position() {
    use surgeist_html::HtmlInputNumericStep as Step;
    assert_eq!(Step::ANY_KEYWORD, "any");
    for (index, lower, upper) in [(0, 97, 65), (1, 110, 78), (2, 121, 89)] {
        assert!(Step::any_keyword_unit_matches(index, lower));
        assert!(Step::any_keyword_unit_matches(index, upper));
        for invalid in [0, 32, 0xd800, 0x212a] {
            assert!(!Step::any_keyword_unit_matches(index, invalid));
        }
    }
    assert!(!Step::any_keyword_unit_matches(3, 32));
    assert!(!Step::any_keyword_unit_matches(usize::MAX, 121));
}

fn raw(role: A, value: &str) -> Attribute {
    Attribute::new(role.qualified_name(), value)
}
fn observe(
    state: State,
    min: Option<&str>,
    max: Option<&str>,
    step: Option<&str>,
    value: Option<&str>,
) -> HtmlInputNumericConstraintsReport {
    let min = min.map(|s| raw(A::Min, s));
    let max = max.map(|s| raw(A::Max, s));
    let step = step.map(|s| raw(A::Step, s));
    let value = value.map(|s| raw(A::Value, s));
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
fn computed(report: &HtmlInputNumericConstraintsReport) -> HtmlInputNumericConstraints {
    match report.outcome() {
        Outcome::Computed(value) => *value,
        other => panic!("expected computation, got {other:?}"),
    }
}
fn float_common(
    side: &HtmlInputNumericAttributeReport,
) -> &HtmlValueReport<HtmlFloatingPointNumber, HtmlFloatingPointNumber> {
    match side.common().unwrap() {
        HtmlInputNumericCommonReport::FloatingPoint(common) => common,
        other => panic!("expected floating-point evidence, got {other:?}"),
    }
}
fn floating(side: &HtmlInputNumericAttributeReport) -> Option<HtmlFloatingPointNumber> {
    match side.converted()? {
        HtmlInputNumericAttributeValue::FloatingPoint(value) => Some(*value),
        other => panic!("expected floating-point conversion, got {other:?}"),
    }
}
fn value(number: Option<HtmlFloatingPointNumber>) -> Option<f64> {
    number.map(|n| n.value())
}
fn syntax_failure() -> HtmlValueFailure {
    HtmlValueFailure::Syntax(HtmlValueSyntaxError::InvalidNumericSyntax)
}

#[test]
fn number_and_range_have_distinct_literal_limit_defaults() {
    // HTML #number-state-(type=number), #range-state-(type=range), #the-step-attribute.
    for (state, min, max) in [
        (State::Number, None, None),
        (State::Range, Some(0.0), Some(100.0)),
    ] {
        let report = observe(state, None, None, None, None);
        let result = computed(&report);
        assert_eq!(value(result.minimum()), min);
        assert_eq!(value(result.maximum()), max);
        assert_eq!(value(result.allowed_step()), Some(1.0));
        assert_eq!(result.minimum_origin(), BoundOrigin::MissingFallback);
        assert_eq!(result.maximum_origin(), BoundOrigin::MissingFallback);
        assert_eq!(result.step_origin(), StepOrigin::MissingDefault);
        assert_eq!(result.step_base().value(), 0.0);
        assert_eq!(result.step_base_origin(), BaseOrigin::Zero);
        assert_eq!(report.input_code_units(), 0);
        for side in [
            report.minimum(),
            report.maximum(),
            report.step(),
            report.value(),
        ] {
            assert_eq!(side.observation(), HtmlAttributeObservation::Missing);
            assert_eq!(side.author(), Author::Missing);
            assert_eq!(
                float_common(side).recovery(),
                &HtmlValueRecoveryState::Missing
            );
            assert_eq!(side.input_state(), state);
        }
    }
}

#[test]
fn failed_limits_keep_fallback_causes_and_never_clamp_inverse_bounds() {
    for state in [State::Number, State::Range] {
        for (spelling, author) in [
            ("", Author::Nonconforming(syntax_failure())),
            ("bad", Author::Nonconforming(syntax_failure())),
            ("\u{a0}4", Author::Nonconforming(syntax_failure())),
            ("1e9999", Author::Conforming),
        ] {
            let report = observe(state, Some(spelling), Some(spelling), None, None);
            let result = computed(&report);
            assert_eq!(
                value(result.minimum()),
                (state == State::Range).then_some(0.0)
            );
            assert_eq!(
                value(result.maximum()),
                (state == State::Range).then_some(100.0)
            );
            assert_eq!(result.minimum_origin(), BoundOrigin::ParseFailureFallback);
            assert_eq!(result.maximum_origin(), BoundOrigin::ParseFailureFallback);
            assert_eq!(report.minimum().author(), author);
            assert_eq!(report.maximum().author(), author);
        }
        let inverse = observe(state, Some("200"), Some("100"), None, None);
        let result = computed(&inverse);
        assert_eq!(value(result.minimum()), Some(200.0));
        assert_eq!(value(result.maximum()), Some(100.0));
        assert_eq!(result.limit_author(), LimitAuthor::MinimumExceedsMaximum);
        assert_eq!(inverse.minimum().author(), Author::Conforming);
        assert_eq!(inverse.maximum().author(), Author::Conforming);
    }
}

#[test]
fn limit_author_relation_compares_effective_bounds_including_range_fallbacks() {
    // #the-min-and-max-attributes links "maximum"/"minimum" to the concepts
    // defined above, including defaults; its nonperiodic guard has no
    // both-content-attributes-present condition (Architecture selection #1133).
    for (state, min, max, expected_min, expected_max, relation, min_author, max_author) in [
        (
            State::Range,
            None,
            None,
            Some(0.0),
            Some(100.0),
            LimitAuthor::Conforming,
            Author::Missing,
            Author::Missing,
        ),
        (
            State::Range,
            Some("200"),
            None,
            Some(200.0),
            Some(100.0),
            LimitAuthor::MinimumExceedsMaximum,
            Author::Conforming,
            Author::Missing,
        ),
        (
            State::Range,
            None,
            Some("-1"),
            Some(0.0),
            Some(-1.0),
            LimitAuthor::MinimumExceedsMaximum,
            Author::Missing,
            Author::Conforming,
        ),
        (
            State::Range,
            Some("200"),
            Some("bad"),
            Some(200.0),
            Some(100.0),
            LimitAuthor::MinimumExceedsMaximum,
            Author::Conforming,
            Author::Nonconforming(syntax_failure()),
        ),
        (
            State::Range,
            Some("bad"),
            Some("-1"),
            Some(0.0),
            Some(-1.0),
            LimitAuthor::MinimumExceedsMaximum,
            Author::Nonconforming(syntax_failure()),
            Author::Conforming,
        ),
        (
            State::Range,
            Some(" +200junk"),
            None,
            Some(200.0),
            Some(100.0),
            LimitAuthor::MinimumExceedsMaximum,
            Author::Nonconforming(syntax_failure()),
            Author::Missing,
        ),
        (
            State::Range,
            Some("bad"),
            Some("-1tail"),
            Some(0.0),
            Some(-1.0),
            LimitAuthor::MinimumExceedsMaximum,
            Author::Nonconforming(syntax_failure()),
            Author::Nonconforming(syntax_failure()),
        ),
        (
            State::Range,
            Some("100"),
            None,
            Some(100.0),
            Some(100.0),
            LimitAuthor::Conforming,
            Author::Conforming,
            Author::Missing,
        ),
        (
            State::Range,
            None,
            Some("0"),
            Some(0.0),
            Some(0.0),
            LimitAuthor::Conforming,
            Author::Missing,
            Author::Conforming,
        ),
        (
            State::Number,
            Some("200"),
            None,
            Some(200.0),
            None,
            LimitAuthor::NotRequired,
            Author::Conforming,
            Author::Missing,
        ),
        (
            State::Number,
            Some("200"),
            Some("bad"),
            Some(200.0),
            None,
            LimitAuthor::NotRequired,
            Author::Conforming,
            Author::Nonconforming(syntax_failure()),
        ),
        (
            State::Number,
            None,
            Some("-1"),
            None,
            Some(-1.0),
            LimitAuthor::NotRequired,
            Author::Missing,
            Author::Conforming,
        ),
        (
            State::Number,
            Some("bad"),
            Some("-1"),
            None,
            Some(-1.0),
            LimitAuthor::NotRequired,
            Author::Nonconforming(syntax_failure()),
            Author::Conforming,
        ),
        (
            State::Number,
            Some("-1"),
            Some("-1"),
            Some(-1.0),
            Some(-1.0),
            LimitAuthor::Conforming,
            Author::Conforming,
            Author::Conforming,
        ),
    ] {
        let report = observe(state, min, max, None, None);
        let result = computed(&report);
        assert_eq!(
            value(result.minimum()),
            expected_min,
            "{state:?}/{min:?}/{max:?}"
        );
        assert_eq!(
            value(result.maximum()),
            expected_max,
            "{state:?}/{min:?}/{max:?}"
        );
        assert_eq!(result.limit_author(), relation, "{state:?}/{min:?}/{max:?}");
        assert_eq!(report.minimum().author(), min_author);
        assert_eq!(report.maximum().author(), max_author);
    }
}

#[test]
fn prefix_recovery_is_independent_from_authored_grammar() {
    let report = observe(
        State::Number,
        Some(" +2junk"),
        Some("4tail"),
        Some("2junk"),
        Some("7tail"),
    );
    let result = computed(&report);
    assert_eq!(value(result.minimum()), Some(2.0));
    assert_eq!(value(result.maximum()), Some(4.0));
    assert_eq!(value(result.allowed_step()), Some(2.0));
    assert_eq!(result.step_base().value(), 2.0);
    assert_eq!(result.step_base_origin(), BaseOrigin::MinimumContent);
    for side in [
        report.minimum(),
        report.maximum(),
        report.step(),
        report.value(),
    ] {
        assert_eq!(side.author(), Author::Nonconforming(syntax_failure()));
        assert!(matches!(
            float_common(side).author(),
            HtmlValueAuthorState::Invalid(_)
        ));
        assert!(matches!(
            float_common(side).recovery(),
            HtmlValueRecoveryState::Value(_)
        ));
    }
}

#[test]
fn raw_minimum_content_precedes_value_but_default_minimum_does_not() {
    // #concept-input-min-zero: content conversion, not the effective minimum.
    for state in [State::Number, State::Range] {
        for min in [None, Some(""), Some("bad"), Some("1e999")] {
            let report = observe(state, min, None, None, Some("7"));
            assert_eq!(computed(&report).step_base().value(), 7.0);
            assert_eq!(
                computed(&report).step_base_origin(),
                BaseOrigin::ValueContent
            );
        }
        for min in ["0", "-0", " +2tail"] {
            let report = observe(state, Some(min), None, None, Some("7"));
            assert_eq!(
                computed(&report).step_base().value(),
                if min == " +2tail" { 2.0 } else { 0.0 }
            );
            assert_eq!(
                computed(&report).step_base_origin(),
                BaseOrigin::MinimumContent
            );
        }
        let report = observe(state, Some("bad"), None, None, Some("bad"));
        assert_eq!(computed(&report).step_base_origin(), BaseOrigin::Zero);
    }
}

#[test]
fn empty_value_author_exception_belongs_only_to_number() {
    // Original Number clause 36770 says "and not empty"; Range 36933 does not.
    for (state, author) in [
        (State::Number, Author::Conforming),
        (State::Range, Author::Nonconforming(syntax_failure())),
    ] {
        let report = observe(state, None, None, None, Some(""));
        assert_eq!(report.value().author(), author);
        assert_eq!(
            report.value().observation(),
            HtmlAttributeObservation::PresentEmpty
        );
        assert_eq!(
            float_common(report.value()).author(),
            &HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::InvalidNumericSyntax)
        );
        assert_eq!(
            float_common(report.value()).recovery(),
            &HtmlValueRecoveryState::Failed(syntax_failure())
        );
        assert_eq!(computed(&report).step_base_origin(), BaseOrigin::Zero);
    }
}

#[test]
fn step_any_is_exact_ascii_case_insensitive_without_trimming() {
    for spelling in ["any", "AnY", "ANY"] {
        let report = observe(State::Range, None, None, Some(spelling), Some("7"));
        assert!(report.step().any_keyword());
        assert!(report.step().common().is_none());
        assert_eq!(report.step().author(), Author::Conforming);
        assert_eq!(computed(&report).allowed_step(), None);
        assert_eq!(computed(&report).step_origin(), StepOrigin::Any);
        assert_eq!(computed(&report).step_base().value(), 7.0);
    }
    for spelling in [" any", "any ", "anyjunk", "\u{a0}any", ""] {
        let report = observe(State::Number, None, None, Some(spelling), None);
        assert!(!report.step().any_keyword());
        assert_eq!(value(computed(&report).allowed_step()), Some(1.0));
        assert_eq!(
            computed(&report).step_origin(),
            StepOrigin::ParseFailureDefault
        );
        assert_eq!(
            report.step().author(),
            Author::Nonconforming(syntax_failure())
        );
    }
}

#[test]
fn step_positivity_uses_the_existing_rounded_binary64_result() {
    for (spelling, expected, origin, author) in [
        (
            "0",
            1.0,
            StepOrigin::NonPositiveDefault,
            Author::StepMustBePositive,
        ),
        (
            "-0",
            1.0,
            StepOrigin::NonPositiveDefault,
            Author::StepMustBePositive,
        ),
        (
            "-2",
            1.0,
            StepOrigin::NonPositiveDefault,
            Author::StepMustBePositive,
        ),
        (
            "1e-999",
            1.0,
            StepOrigin::NonPositiveDefault,
            Author::StepMustBePositive,
        ),
        (
            "5e-324",
            f64::from_bits(1),
            StepOrigin::Recovered,
            Author::Conforming,
        ),
        ("2.5", 2.5, StepOrigin::Recovered, Author::Conforming),
        (
            "2junk",
            2.0,
            StepOrigin::Recovered,
            Author::Nonconforming(syntax_failure()),
        ),
        (
            "+2",
            2.0,
            StepOrigin::Recovered,
            Author::Nonconforming(syntax_failure()),
        ),
    ] {
        let report = observe(State::Number, None, None, Some(spelling), None);
        assert_eq!(
            value(computed(&report).allowed_step()),
            Some(expected),
            "{spelling}"
        );
        assert_eq!(computed(&report).step_origin(), origin, "{spelling}");
        assert_eq!(report.step().author(), author, "{spelling}");
    }
}

#[test]
fn grammar_only_overflow_is_conforming_while_step_requires_successful_parsing() {
    // Number/Range min/max/value require the valid-floating-point grammar.
    // Step additionally requires that its parsing result is greater than zero.
    // Both signs of 1e999 satisfy the grammar but fail binary64 conversion.
    let overflow =
        HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow);
    for state in [State::Number, State::Range] {
        for spelling in ["1e999", "-1e999"] {
            let report = observe(
                state,
                Some(spelling),
                Some(spelling),
                Some(spelling),
                Some(spelling),
            );
            for side in [report.minimum(), report.maximum(), report.value()] {
                assert_eq!(
                    side.author(),
                    Author::Conforming,
                    "{state:?}/{spelling}/{:?}",
                    side.attribute()
                );
            }
            assert_eq!(report.step().author(), Author::Nonconforming(overflow));
            for side in [
                report.minimum(),
                report.maximum(),
                report.step(),
                report.value(),
            ] {
                assert_eq!(
                    float_common(side).author(),
                    &HtmlValueAuthorState::Unrepresentable(
                        HtmlValueRepresentationError::FloatingPointOverflow
                    )
                );
                assert_eq!(
                    float_common(side).recovery(),
                    &HtmlValueRecoveryState::Failed(overflow)
                );
                assert_eq!(side.converted(), None);
            }
            let result = computed(&report);
            assert_eq!(
                value(result.minimum()),
                (state == State::Range).then_some(0.0)
            );
            assert_eq!(
                value(result.maximum()),
                (state == State::Range).then_some(100.0)
            );
            assert_eq!(result.minimum_origin(), BoundOrigin::ParseFailureFallback);
            assert_eq!(result.maximum_origin(), BoundOrigin::ParseFailureFallback);
            assert_eq!(value(result.allowed_step()), Some(1.0));
            assert_eq!(result.step_origin(), StepOrigin::ParseFailureDefault);
            assert_eq!(result.step_base().value(), 0.0);
            assert_eq!(result.step_base_origin(), BaseOrigin::Zero);
        }
    }
}

#[test]
fn overflow_keeps_representation_failure_and_source_default() {
    let overflow =
        HtmlValueFailure::Representation(HtmlValueRepresentationError::FloatingPointOverflow);
    let report = observe(
        State::Range,
        Some("1e999"),
        Some("1e999"),
        Some("1e999"),
        Some("1e999"),
    );
    for side in [report.minimum(), report.maximum(), report.value()] {
        assert_eq!(side.author(), Author::Conforming);
    }
    assert_eq!(report.step().author(), Author::Nonconforming(overflow));
    for side in [
        report.minimum(),
        report.maximum(),
        report.step(),
        report.value(),
    ] {
        assert_eq!(
            float_common(side).author(),
            &HtmlValueAuthorState::Unrepresentable(
                HtmlValueRepresentationError::FloatingPointOverflow
            )
        );
        assert_eq!(
            float_common(side).recovery(),
            &HtmlValueRecoveryState::Failed(overflow)
        );
    }
    let result = computed(&report);
    assert_eq!(value(result.minimum()), Some(0.0));
    assert_eq!(value(result.maximum()), Some(100.0));
    assert_eq!(value(result.allowed_step()), Some(1.0));
    assert_eq!(result.step_origin(), StepOrigin::ParseFailureDefault);
    assert_eq!(result.step_base_origin(), BaseOrigin::Zero);
}

#[test]
fn source_applicable_temporal_states_execute_their_actual_grammars() {
    for state in [
        State::Date,
        State::Month,
        State::Week,
        State::Time,
        State::LocalDateTime,
    ] {
        let report = observe(state, Some("not-numeric"), Some(""), Some("any"), None);
        assert!(matches!(report.outcome(), Outcome::ComputedTemporal(_)));
        for side in [report.minimum(), report.maximum()] {
            assert_eq!(side.applicability(), HtmlAttributeApplicability::Applicable);
            assert!(matches!(side.author(), Author::Nonconforming(_)));
            assert!(side.common().is_some());
            assert!(side.converted().is_none());
        }
        assert_eq!(
            report.minimum().context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(report.step().author(), Author::Conforming);
        assert!(report.step().any_keyword());
        assert!(report.step().common().is_none());
    }
}

#[test]
fn other_states_keep_presence_prohibition_without_parsing_value_as_number() {
    for state in [State::Text, State::Checkbox, State::File] {
        let report = observe(state, Some("1"), None, Some("any"), Some("ordinary text"));
        assert_eq!(
            report.outcome().clone(),
            Outcome::InapplicableInputState(state)
        );
        for side in [report.minimum(), report.step()] {
            assert_eq!(
                side.context_author(),
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::InputStateForbidsAttribute
                )
            );
            assert!(side.common().is_none());
        }
        assert_eq!(
            report.maximum().context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(report.maximum().author(), Author::Missing);
        assert_eq!(
            report.value().context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(report.value().author(), Author::NotChecked);
        assert!(report.value().common().is_none());
    }
    // Actual type recognition/fallback remains owned by the existing report.
    let interpreter = HtmlValueInterpreter::new(32);
    let bad_type = raw(A::Type, "number ");
    let state = *interpreter
        .input_type(&E::Input.qualified_name(), Some(&bad_type))
        .unwrap()
        .intrinsic()
        .unwrap();
    assert_eq!(state, State::Text);
    assert_eq!(
        observe(state, None, None, None, None).outcome().clone(),
        Outcome::InapplicableInputState(State::Text)
    );
}

#[test]
fn each_role_preserves_exact_names_raw_utf16_and_source_metadata() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "input");
    let min = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("m".into()), "min"),
        DocumentString::from_utf16(vec![b'2' as u16, 0xd800]),
    );
    let max = raw(A::Max, "9");
    let step = raw(A::Step, "any");
    let content = raw(A::Value, "7");
    let report = HtmlValueInterpreter::new(8)
        .input_numeric_constraints(
            &element,
            Some(&min),
            Some(&max),
            Some(&step),
            Some(&content),
            State::Number,
        )
        .unwrap();
    assert_eq!(report.minimum().raw_attribute(), Some(&min));
    assert_eq!(report.minimum().element(), &element);
    assert_eq!(report.minimum().raw().unwrap().code_units(), &[50, 0xd800]);
    assert_eq!(floating(report.minimum()).unwrap().value(), 2.0);
    assert_eq!(
        report.minimum().author(),
        Author::Nonconforming(syntax_failure())
    );
    assert_eq!(report.input_code_units(), 7);
    for (side, role, row, anchor) in [
        (report.minimum(), A::Min, 97, "#attr-input-min"),
        (report.maximum(), A::Max, 92, "#attr-input-max"),
        (report.step(), A::Step, 154, "#attr-input-step"),
        (report.value(), A::Value, 175, "#attr-input-value"),
    ] {
        assert_eq!(side.attribute(), role);
        assert_eq!(side.inventory_rows(), &[row]);
        assert!(
            side.source_clause()
                .contains("877d6146487f15b15bc8dd0e3c625063a37b456a")
        );
        assert!(side.source_clause().ends_with(anchor));
        assert!(
            side.input_state_source_clause()
                .ends_with("#number-state-(type=number)")
        );
    }
    assert!(
        report
            .limits_source_clause()
            .ends_with("#the-min-and-max-attributes")
    );
    assert!(report.step_source_clause().ends_with("#the-step-attribute"));
}

#[test]
fn wrong_expanded_names_precede_state_capability_and_keep_all_observations() {
    let interpreter = HtmlValueInterpreter::new(32);
    let wrong_min = Attribute::new(QualifiedName::new(Namespace::svg(), None, "min"), "3");
    let wrong_max = raw(A::Min, "4");
    let good_value = raw(A::Value, "7");
    let report = interpreter
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            Some(&wrong_min),
            Some(&wrong_max),
            None::<&surgeist_html::Attribute>,
            Some(&good_value),
            State::Time,
        )
        .unwrap();
    assert_eq!(
        report.outcome().clone(),
        Outcome::Unapplied {
            attribute: A::Min,
            reason: HtmlAttributeApplicability::DifferentAttribute
        }
    );
    assert_eq!(report.minimum().raw_attribute(), Some(&wrong_min));
    assert_eq!(report.maximum().raw_attribute(), Some(&wrong_max));
    assert_eq!(report.value().raw_attribute(), Some(&good_value));
    assert_eq!(report.minimum().author(), Author::Unapplied);
    assert!(matches!(report.value().author(), Author::Nonconforming(_)));
    for (element, reason) in [
        (
            QualifiedName::new(Namespace::svg(), None, "input"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            E::Div.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "Input"),
            HtmlAttributeApplicability::DifferentElement,
        ),
    ] {
        let report = interpreter
            .input_numeric_constraints(
                &element,
                None::<&surgeist_html::Attribute>,
                None::<&surgeist_html::Attribute>,
                None::<&surgeist_html::Attribute>,
                None::<&surgeist_html::Attribute>,
                State::Number,
            )
            .unwrap();
        assert_eq!(
            report.outcome().clone(),
            Outcome::Unapplied {
                attribute: A::Min,
                reason
            }
        );
    }
    let upper = Attribute::new(QualifiedName::new(Namespace::none(), None, "MAX"), "4");
    let report = interpreter
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
            Some(&upper),
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
            State::Range,
        )
        .unwrap();
    assert_eq!(
        report.outcome().clone(),
        Outcome::Unapplied {
            attribute: A::Max,
            reason: HtmlAttributeApplicability::DifferentAttribute
        }
    );
}

#[test]
fn every_supplied_value_is_admitted_before_wrong_names_or_state_processing() {
    let interpreter = HtmlValueInterpreter::new(4);
    let long = raw(A::Title, "12345");
    for slot in 0..4 {
        let mut inputs = [None; 4];
        inputs[slot] = Some(&long);
        for state in [State::Number, State::Time, State::Text] {
            let error = interpreter
                .input_numeric_constraints(
                    &E::Div.qualified_name(),
                    inputs[0],
                    inputs[1],
                    inputs[2],
                    inputs[3],
                    state,
                )
                .unwrap_err();
            let HtmlInputNumericConstraintsError::Admission(error) = error else {
                panic!("expected admission")
            };
            assert_eq!(error.capacity(), 4);
            assert_eq!(error.required(), 5);
        }
    }
    // Four admitted values need not fit a fabricated aggregate cap.
    let min = raw(A::Min, "1234");
    let max = raw(A::Max, "1234");
    let step = raw(A::Step, "1234");
    let content = raw(A::Value, "1234");
    let report = interpreter
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            Some(&min),
            Some(&max),
            Some(&step),
            Some(&content),
            State::Number,
        )
        .unwrap();
    assert_eq!(report.input_code_units(), 16);
    assert!(matches!(report.outcome().clone(), Outcome::Computed(_)));
}

#[test]
fn native_step_and_limits_use_existing_checked_finite_values() {
    for n in [0.0, -0.0, -1.0] {
        assert_eq!(
            HtmlInputStep::positive(HtmlFloatingPointNumber::try_from(n).unwrap()),
            Err(HtmlNonPositiveInputStepError)
        );
    }
    let any = HtmlInputStep::any();
    assert_eq!(any.number(), None);
    assert_eq!(any.as_document_string(), DocumentString::from("any"));
    assert_eq!(any.into_step_attribute(), raw(A::Step, "any"));
    let positive =
        HtmlInputStep::positive(HtmlFloatingPointNumber::try_from(0.25).unwrap()).unwrap();
    assert_eq!(positive.number().unwrap().value(), 0.25);
    assert_eq!(positive.into_step_attribute(), raw(A::Step, "0.25"));
    let min = HtmlFloatingPointNumber::try_from(-2.5)
        .unwrap()
        .into_input_min_attribute();
    let max = HtmlFloatingPointNumber::try_from(4.0)
        .unwrap()
        .into_input_max_attribute();
    let step = positive.into_step_attribute();
    let report = HtmlValueInterpreter::new(32)
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            Some(&min),
            Some(&max),
            Some(&step),
            None::<&surgeist_html::Attribute>,
            State::Number,
        )
        .unwrap();
    for side in [report.minimum(), report.maximum(), report.step()] {
        assert_eq!(side.author(), Author::Conforming);
    }
    assert_eq!(value(computed(&report).minimum()), Some(-2.5));
    assert_eq!(value(computed(&report).maximum()), Some(4.0));
}

#[test]
fn inherited_common_grammar_keeps_partial_exponents_and_decimal_recovery() {
    // Common floating-point recovery stops at these prefixes; author grammar
    // requires the whole value. No additional attribute-specific trimming occurs.
    for (spelling, recovered) in [
        ("1.", 1.0),
        ("2e", 2.0),
        ("3e+", 3.0),
        ("4e-", 4.0),
        (".5tail", 0.5),
    ] {
        let report = observe(State::Number, Some(spelling), None, Some(spelling), None);
        assert_eq!(
            report.minimum().author(),
            Author::Nonconforming(syntax_failure())
        );
        assert_eq!(
            report.step().author(),
            Author::Nonconforming(syntax_failure())
        );
        assert_eq!(value(computed(&report).minimum()), Some(recovered));
        assert_eq!(value(computed(&report).allowed_step()), Some(recovered));
    }
}

#[test]
fn leading_isolated_surrogates_remain_raw_and_fail_the_conversion() {
    let units = vec![0xd800, b'7' as u16];
    let raw = Attribute::new(
        A::Min.qualified_name(),
        DocumentString::from_utf16(units.clone()),
    );
    let report = HtmlValueInterpreter::new(2)
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            Some(&raw),
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
            State::Range,
        )
        .unwrap();
    assert_eq!(report.minimum().raw().unwrap().code_units(), units);
    assert_eq!(
        report.minimum().author(),
        Author::Nonconforming(syntax_failure())
    );
    assert_eq!(value(computed(&report).minimum()), Some(0.0));
    assert_eq!(
        computed(&report).minimum_origin(),
        BoundOrigin::ParseFailureFallback
    );
    assert_eq!(computed(&report).step_base_origin(), BaseOrigin::Zero);
    assert_eq!(report.input_code_units(), 2);
    let error = HtmlValueInterpreter::new(1)
        .input_numeric_constraints(
            &E::Input.qualified_name(),
            Some(&raw),
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
            State::Range,
        )
        .unwrap_err();
    let HtmlInputNumericConstraintsError::Admission(error) = error else {
        panic!("expected admission")
    };
    assert_eq!(error.required(), 2);
}

#[test]
fn every_wrong_role_blocks_compound_output_without_erasing_other_members() {
    let min = raw(A::Min, "1");
    let max = raw(A::Max, "9");
    let step = raw(A::Step, "2");
    let content = raw(A::Value, "7");
    let wrong = raw(A::Title, "3");
    for (slot, role) in [A::Min, A::Max, A::Step, A::Value].into_iter().enumerate() {
        let mut raws = [Some(&min), Some(&max), Some(&step), Some(&content)];
        raws[slot] = Some(&wrong);
        let report = HtmlValueInterpreter::new(32)
            .input_numeric_constraints(
                &E::Input.qualified_name(),
                raws[0],
                raws[1],
                raws[2],
                raws[3],
                State::Number,
            )
            .unwrap();
        assert_eq!(
            report.outcome().clone(),
            Outcome::Unapplied {
                attribute: role,
                reason: HtmlAttributeApplicability::DifferentAttribute
            }
        );
        let members = [
            report.minimum(),
            report.maximum(),
            report.step(),
            report.value(),
        ];
        assert_eq!(members[slot].raw_attribute(), Some(&wrong));
        assert_eq!(members[slot].author(), Author::Unapplied);
        assert!(members[slot].common().is_none());
        for (other, member) in members.into_iter().enumerate() {
            if other != slot {
                assert_eq!(member.raw_attribute(), raws[other]);
                assert_eq!(member.author(), Author::Conforming);
                assert!(member.converted().is_some());
            }
        }
    }
}
