use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeContextAuthorState,
    HtmlAttributeContextConformanceError, HtmlAttributeObservation,
    HtmlControlAttributeApplication, HtmlControlGetterOrigin, HtmlControlInitialText,
    HtmlControlInitialTextState, HtmlControlIntegerAuthorState, HtmlControlLengthBoundsReport,
    HtmlControlLengthError, HtmlControlLengthPairState, HtmlControlLengthValue, HtmlInputSizeValue,
    HtmlInputSizeZeroError, HtmlInputType, HtmlNonNegativeInteger, HtmlValueAuthorState,
    HtmlValueInterpreter, HtmlValueRecoveryState, KnownHtmlAttribute as A, KnownHtmlElement as E,
    Namespace, QualifiedName,
};

fn raw(attribute: A, value: &str) -> Attribute {
    Attribute::new(attribute.qualified_name(), value)
}

fn length_report(
    min: Option<&Attribute>,
    max: Option<&Attribute>,
) -> HtmlControlLengthBoundsReport {
    HtmlValueInterpreter::new(256)
        .control_length_bounds(&E::Textarea.qualified_name(), min, max, None, None)
        .unwrap()
}

fn bound(report: &HtmlControlLengthValue) -> Option<&str> {
    match report {
        HtmlControlLengthValue::Bound(value) => Some(value.decimal_digits()),
        HtmlControlLengthValue::NoBound => None,
    }
}

#[test]
fn size_content_has_no_twenty_default_but_getter_does() {
    // HTML #attr-input-size and unsigned-long reflection are distinct algorithms.
    let interpreter = HtmlValueInterpreter::new(64);
    for (value, origin, author) in [
        (
            None,
            HtmlControlGetterOrigin::MissingDefault,
            HtmlControlIntegerAuthorState::Missing,
        ),
        (
            Some(""),
            HtmlControlGetterOrigin::ParseFailureDefault,
            HtmlControlIntegerAuthorState::Nonconforming(surgeist_html::HtmlValueFailure::Syntax(
                surgeist_html::HtmlValueSyntaxError::InvalidNumericSyntax,
            )),
        ),
        (
            Some("oops"),
            HtmlControlGetterOrigin::ParseFailureDefault,
            HtmlControlIntegerAuthorState::Nonconforming(surgeist_html::HtmlValueFailure::Syntax(
                surgeist_html::HtmlValueSyntaxError::InvalidNumericSyntax,
            )),
        ),
        (
            Some("0"),
            HtmlControlGetterOrigin::ZeroDefault,
            HtmlControlIntegerAuthorState::ZeroMustBePositive,
        ),
    ] {
        let attribute = value.map(|value| raw(A::Size, value));
        let report = interpreter
            .input_size_for_input(
                &E::Input.qualified_name(),
                attribute.as_ref(),
                HtmlInputType::Text,
            )
            .unwrap();
        assert_eq!(
            report.content(),
            Some(&HtmlInputSizeValue::NoPositiveRequest)
        );
        assert_eq!(report.reflected_getter(), Some(20));
        assert_eq!(report.getter_origin(), Some(origin));
        assert_eq!(report.author(), author);
        assert_eq!(report.raw_attribute(), attribute.as_ref());
    }
}

#[test]
fn length_missing_and_parse_failure_have_no_bound_and_minus_one_getter() {
    for value in [None, Some(""), Some("bad"), Some("-1"), Some("\u{a0}4")] {
        let minimum = value.map(|value| raw(A::Minlength, value));
        let maximum = value.map(|value| raw(A::Maxlength, value));
        let report = length_report(minimum.as_ref(), maximum.as_ref());
        for side in [report.minimum(), report.maximum()] {
            assert_eq!(side.content(), Some(&HtmlControlLengthValue::NoBound));
            assert_eq!(side.reflected_getter(), Some(-1));
            assert_eq!(
                side.getter_origin(),
                Some(if value.is_none() {
                    HtmlControlGetterOrigin::MissingDefault
                } else {
                    HtmlControlGetterOrigin::ParseFailureDefault
                })
            );
            assert_eq!(
                side.raw().map(DocumentString::code_units),
                value
                    .map(|v| v.encode_utf16().collect::<Vec<_>>())
                    .as_deref()
            );
        }
        assert_eq!(
            report.pair_author(),
            HtmlControlLengthPairState::NotRequired
        );
        assert_eq!(
            report.initial_text_author(),
            HtmlControlInitialTextState::NotRequired
        );
    }
}

#[test]
fn zero_length_is_a_real_bound_and_zero_getter() {
    let minimum = raw(A::Minlength, "0000");
    let maximum = raw(A::Maxlength, "0");
    let report = length_report(Some(&minimum), Some(&maximum));
    assert_eq!(bound(report.minimum().content().unwrap()), Some("0"));
    assert_eq!(bound(report.maximum().content().unwrap()), Some("0"));
    assert_eq!(report.minimum().reflected_getter(), Some(0));
    assert_eq!(report.maximum().reflected_getter(), Some(0));
    assert_eq!(report.pair_author(), HtmlControlLengthPairState::Conforming);
}

#[test]
fn exact_content_survives_bounded_reflection_fallback() {
    let interpreter = HtmlValueInterpreter::new(256);
    for (value, getter_size, getter_length, origin) in [
        (
            "2147483647",
            2147483647,
            2147483647,
            HtmlControlGetterOrigin::Recovered,
        ),
        (
            "2147483648",
            20,
            -1,
            HtmlControlGetterOrigin::OutOfRangeDefault,
        ),
        (
            "4294967295",
            20,
            -1,
            HtmlControlGetterOrigin::OutOfRangeDefault,
        ),
        (
            "999999999999999999999999999999999999999999999999999999999999999999",
            20,
            -1,
            HtmlControlGetterOrigin::OutOfRangeDefault,
        ),
    ] {
        let size = raw(A::Size, value);
        let maximum = raw(A::Maxlength, value);
        let size_report = interpreter
            .input_size_for_input(
                &E::Input.qualified_name(),
                Some(&size),
                HtmlInputType::Password,
            )
            .unwrap();
        assert_eq!(
            size_report.content(),
            Some(&HtmlInputSizeValue::PositiveCharacters(
                HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(value)).unwrap()
            ))
        );
        assert_eq!(size_report.reflected_getter(), Some(getter_size));
        assert_eq!(size_report.getter_origin(), Some(origin));
        let length = length_report(None, Some(&maximum));
        assert_eq!(bound(length.maximum().content().unwrap()), Some(value));
        assert_eq!(length.maximum().reflected_getter(), Some(getter_length));
        assert_eq!(length.maximum().getter_origin(), Some(origin));
    }
}

#[test]
fn recovery_does_not_certify_whole_value_author_syntax() {
    for (value, recovered, conforming) in [
        ("004", "4", true),
        ("+4", "4", false),
        (" \t4tail", "4", false),
        ("4.9", "4", false),
        ("-0", "0", false),
    ] {
        let minimum = raw(A::Minlength, value);
        let maximum = raw(A::Maxlength, "4");
        let report = length_report(Some(&minimum), Some(&maximum));
        assert_eq!(
            report.minimum().recovered().unwrap().decimal_digits(),
            recovered
        );
        assert_eq!(bound(report.minimum().content().unwrap()), Some(recovered));
        assert_eq!(
            report.minimum().author() == HtmlControlIntegerAuthorState::Conforming,
            conforming
        );
        assert_eq!(report.pair_author(), HtmlControlLengthPairState::Conforming);
    }
}

#[test]
fn isolated_surrogates_are_preserved_through_prefix_recovery_and_failure() {
    for (units, expected) in [
        (vec![b'4' as u16, 0xd800], Some("4")),
        (vec![0xd800, b'4' as u16], None),
    ] {
        let maximum = Attribute::new(
            A::Maxlength.qualified_name(),
            DocumentString::from_utf16(units.clone()),
        );
        let report = length_report(None, Some(&maximum));
        assert_eq!(report.maximum().raw().unwrap().code_units(), units);
        assert_eq!(
            report
                .maximum()
                .recovered()
                .map(HtmlNonNegativeInteger::decimal_digits),
            expected
        );
        assert!(matches!(
            report.maximum().common().unwrap().author(),
            HtmlValueAuthorState::Invalid(_)
        ));
        assert_eq!(
            report.maximum().reflected_getter(),
            Some(if expected.is_some() { 4 } else { -1 })
        );
    }
}

#[test]
fn actual_six_input_states_apply_and_all_other_states_ignore_content() {
    // Expected set comes from each defining input-state paragraph, not the producer.
    let allowed = [
        HtmlInputType::Text,
        HtmlInputType::Search,
        HtmlInputType::Telephone,
        HtmlInputType::Url,
        HtmlInputType::Email,
        HtmlInputType::Password,
    ];
    let ignored = [
        HtmlInputType::Hidden,
        HtmlInputType::Date,
        HtmlInputType::Month,
        HtmlInputType::Week,
        HtmlInputType::Time,
        HtmlInputType::LocalDateTime,
        HtmlInputType::Number,
        HtmlInputType::Range,
        HtmlInputType::Color,
        HtmlInputType::Checkbox,
        HtmlInputType::Radio,
        HtmlInputType::File,
        HtmlInputType::Submit,
        HtmlInputType::Image,
        HtmlInputType::Reset,
        HtmlInputType::Button,
    ];
    let interpreter = HtmlValueInterpreter::new(64);
    let size = raw(A::Size, "4");
    let minimum = raw(A::Minlength, "1");
    let maximum = raw(A::Maxlength, "4");
    for (states, applies) in [(&allowed[..], true), (&ignored[..], false)] {
        for &state in states {
            let report = interpreter
                .control_length_bounds(
                    &E::Input.qualified_name(),
                    Some(&minimum),
                    Some(&maximum),
                    Some(state),
                    None,
                )
                .unwrap();
            let size_report = interpreter
                .input_size_for_input(&E::Input.qualified_name(), Some(&size), state)
                .unwrap();
            for side in [report.minimum(), report.maximum()] {
                assert_eq!(
                    side.application(),
                    if applies {
                        HtmlControlAttributeApplication::Applicable
                    } else {
                        HtmlControlAttributeApplication::IgnoredInputState
                    }
                );
                assert_eq!(side.content().is_some(), applies);
                assert_eq!(side.input_state(), Some(state));
                assert_eq!(
                    side.context_author(),
                    if applies {
                        HtmlAttributeContextAuthorState::Conforming
                    } else {
                        HtmlAttributeContextAuthorState::Nonconforming(
                            HtmlAttributeContextConformanceError::InputStateForbidsAttribute,
                        )
                    }
                );
            }
            assert_eq!(report.maximum().reflected_getter(), Some(4));
            assert_eq!(size_report.reflected_getter(), Some(4));
            assert_eq!(size_report.content().is_some(), applies);
            assert_eq!(size_report.raw_attribute(), Some(&size));
        }
    }
}

#[test]
fn absent_input_state_is_required_without_guessing_text_or_erasing_getters() {
    let interpreter = HtmlValueInterpreter::new(64);
    let minimum = raw(A::Minlength, "5");
    let maximum = raw(A::Maxlength, "4");
    let report = interpreter
        .control_length_bounds(
            &E::Input.qualified_name(),
            Some(&minimum),
            Some(&maximum),
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        report.minimum().application(),
        HtmlControlAttributeApplication::InputStateRequired
    );
    assert_eq!(report.minimum().content(), None);
    assert_eq!(report.minimum().reflected_getter(), Some(5));
    assert_eq!(
        report.minimum().context_author(),
        HtmlAttributeContextAuthorState::Unchecked
    );
    assert_eq!(report.pair_author(), HtmlControlLengthPairState::Unchecked);
    assert_eq!(
        report.initial_text_author(),
        HtmlControlInitialTextState::InputStateRequired
    );
    let size = raw(A::Size, "4");
    let size = interpreter
        .input_size(&E::Input.qualified_name(), Some(&size))
        .unwrap();
    assert_eq!(
        size.application(),
        HtmlControlAttributeApplication::InputStateRequired
    );
    assert_eq!(size.reflected_getter(), Some(4));
    assert_eq!(size.content(), None);
}

#[test]
fn forbidden_input_authorship_keeps_lexical_error_and_raw() {
    let maximum = raw(A::Maxlength, "+4suffix");
    let report = HtmlValueInterpreter::new(64)
        .control_length_bounds(
            &E::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
            Some(&maximum),
            Some(HtmlInputType::Number),
            None,
        )
        .unwrap();
    assert!(matches!(
        report.maximum().author(),
        HtmlControlIntegerAuthorState::Nonconforming(_)
    ));
    assert_eq!(
        report.maximum().context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::InputStateForbidsAttribute
        )
    );
    assert_eq!(report.maximum().recovered().unwrap().decimal_digits(), "4");
    assert_eq!(report.maximum().raw_attribute(), Some(&maximum));
}

#[test]
fn absent_attributes_select_no_presence_author_violation_even_on_ignored_state() {
    let report = HtmlValueInterpreter::new(0)
        .control_length_bounds(
            &E::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
            None::<&surgeist_html::Attribute>,
            Some(HtmlInputType::Checkbox),
            None,
        )
        .unwrap();
    assert_eq!(
        report.minimum().context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(
        report.maximum().context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(report.minimum().reflected_getter(), Some(-1));
    assert_eq!(report.input_code_units(), 0);
}

#[test]
fn exact_pair_comparison_does_not_project_or_saturate_huge_bounds() {
    for (min, max, relation) in [
        ("4", "4", HtmlControlLengthPairState::Conforming),
        ("5", "4", HtmlControlLengthPairState::MinimumExceedsMaximum),
        (
            "999999999999999999999999999999999999999999999999999999999998",
            "999999999999999999999999999999999999999999999999999999999999",
            HtmlControlLengthPairState::Conforming,
        ),
        (
            "1000000000000000000000000000000000000000000000000000000000000",
            "999999999999999999999999999999999999999999999999999999999999",
            HtmlControlLengthPairState::MinimumExceedsMaximum,
        ),
        (
            " 5tail",
            "4",
            HtmlControlLengthPairState::MinimumExceedsMaximum,
        ),
    ] {
        let minimum = raw(A::Minlength, min);
        let maximum = raw(A::Maxlength, max);
        let report = length_report(Some(&minimum), Some(&maximum));
        assert_eq!(report.pair_author(), relation);
        assert!(report.minimum().recovered().is_some());
        assert!(report.maximum().recovered().is_some());
    }
}

#[test]
fn wrong_attribute_namespace_is_independent_of_the_other_valid_bound() {
    let minimum = Attribute::new(
        QualifiedName::new(Namespace::html(), None, "minlength"),
        "9",
    );
    let maximum = raw(A::Maxlength, "4");
    let report = length_report(Some(&minimum), Some(&maximum));
    assert_eq!(
        report.minimum().applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    assert_eq!(report.minimum().common(), None);
    assert_eq!(report.minimum().reflected_getter(), None);
    assert_eq!(report.minimum().raw_attribute(), Some(&minimum));
    assert_eq!(bound(report.maximum().content().unwrap()), Some("4"));
    assert_eq!(report.maximum().reflected_getter(), Some(4));
    assert_eq!(report.pair_author(), HtmlControlLengthPairState::Unapplied);
    let wrong_max = raw(A::Size, "2");
    let valid_min = raw(A::Minlength, "1");
    let report = length_report(Some(&valid_min), Some(&wrong_max));
    assert_eq!(bound(report.minimum().content().unwrap()), Some("1"));
    assert_eq!(report.maximum().content(), None);
    assert_eq!(report.maximum().raw_attribute(), Some(&wrong_max));
}

#[test]
fn expanded_identity_preserves_prefixes_but_local_case_and_namespace_are_exact() {
    let interpreter = HtmlValueInterpreter::new(64);
    let element = QualifiedName::new(Namespace::html(), Some(DocumentString::from("p")), "input");
    let size = Attribute::new(
        QualifiedName::new(Namespace::none(), Some(DocumentString::from("q")), "size"),
        "4",
    );
    let report = interpreter
        .input_size_for_input(&element, Some(&size), HtmlInputType::Text)
        .unwrap();
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&size));
    assert_eq!(report.reflected_getter(), Some(4));
    for (element, expected) in [
        (
            QualifiedName::new(Namespace::svg(), None, "input"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "Input"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            E::Select.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
    ] {
        let report = interpreter
            .input_size_for_input(&element, Some(&size), HtmlInputType::Text)
            .unwrap();
        assert_eq!(report.applicability(), expected);
        assert_eq!(report.reflected_getter(), None);
        assert_eq!(report.raw_attribute(), Some(&size));
    }
    let upper = Attribute::new(QualifiedName::new(Namespace::none(), None, "SIZE"), "4");
    assert_eq!(
        interpreter
            .input_size(&E::Input.qualified_name(), Some(&upper))
            .unwrap()
            .applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
}

#[test]
fn textarea_does_not_accept_an_input_state_fact_family() {
    let maximum = raw(A::Maxlength, "4");
    let report = HtmlValueInterpreter::new(64)
        .control_length_bounds(
            &E::Textarea.qualified_name(),
            None::<&surgeist_html::Attribute>,
            Some(&maximum),
            Some(HtmlInputType::Text),
            None,
        )
        .unwrap();
    assert_eq!(
        report.maximum().applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    assert_eq!(report.maximum().input_state(), Some(HtmlInputType::Text));
    assert_eq!(report.maximum().raw_attribute(), Some(&maximum));
    assert_eq!(report.maximum().content(), None);
}

#[test]
fn input_value_checks_exact_utf16_without_textarea_newline_normalization() {
    let interpreter = HtmlValueInterpreter::new(64);
    let maximum = raw(A::Maxlength, "4");
    for (units, expected) in [
        (
            vec![0xd83d, 0xde00, 0xd800, b'x' as u16],
            HtmlControlInitialTextState::Conforming { code_units: 4 },
        ),
        (
            vec![0xd83d, 0xde00, 0xd800, b'x' as u16, b'y' as u16],
            HtmlControlInitialTextState::ExceedsMaximum { code_units: 5 },
        ),
        (
            vec![0x0d, 0x0a, 0x0d, 0x0a, 0x0d],
            HtmlControlInitialTextState::ExceedsMaximum { code_units: 5 },
        ),
    ] {
        let initial = HtmlControlInitialText::InputValue(Some(Attribute::new(
            A::Value.qualified_name(),
            DocumentString::from_utf16(units),
        )));
        let report = interpreter
            .control_length_bounds(
                &E::Input.qualified_name(),
                None::<&surgeist_html::Attribute>,
                Some(&maximum),
                Some(HtmlInputType::Text),
                Some(&initial),
            )
            .unwrap();
        assert_eq!(report.initial_text_author(), expected);
        assert_eq!(report.initial_text(), Some(&initial));
    }
}

#[test]
fn textarea_descendant_text_counts_normalized_utf16_without_wrapping() {
    let interpreter = HtmlValueInterpreter::new(64);
    let maximum = raw(A::Maxlength, "4");
    for (units, expected) in [
        (
            vec![0x0d, 0x0a, 0x0d, 0xd83d, 0xde00],
            HtmlControlInitialTextState::Conforming { code_units: 4 },
        ),
        (
            vec![0x0d, 0x0a, 0x0d, 0xd83d, 0xde00, 0xd800],
            HtmlControlInitialTextState::ExceedsMaximum { code_units: 5 },
        ),
        (
            vec![0x0d, 0x0d, 0x0a, b'x' as u16, b'y' as u16],
            HtmlControlInitialTextState::Conforming { code_units: 4 },
        ),
        (
            vec![b'a' as u16, b'b' as u16, b'c' as u16, b'd' as u16],
            HtmlControlInitialTextState::Conforming { code_units: 4 },
        ),
    ] {
        let initial =
            HtmlControlInitialText::TextareaDescendantText(DocumentString::from_utf16(units));
        let report = interpreter
            .control_length_bounds(
                &E::Textarea.qualified_name(),
                None::<&surgeist_html::Attribute>,
                Some(&maximum),
                None,
                Some(&initial),
            )
            .unwrap();
        assert_eq!(report.initial_text_author(), expected);
        assert_eq!(report.initial_text(), Some(&initial));
    }
}

#[test]
fn initial_text_fact_absence_is_not_empty_current_value_or_full_conformance() {
    let interpreter = HtmlValueInterpreter::new(64);
    let minimum = raw(A::Minlength, "2");
    let maximum = raw(A::Maxlength, "4");
    let absent = HtmlControlInitialText::InputValue(None);
    let empty = HtmlControlInitialText::InputValue(Some(raw(A::Value, "")));
    for (initial, expected) in [
        (None, HtmlControlInitialTextState::Unchecked),
        (Some(&absent), HtmlControlInitialTextState::NotRequired),
        (
            Some(&empty),
            HtmlControlInitialTextState::Conforming { code_units: 0 },
        ),
    ] {
        let report = interpreter
            .control_length_bounds(
                &E::Input.qualified_name(),
                Some(&minimum),
                Some(&maximum),
                Some(HtmlInputType::Text),
                initial,
            )
            .unwrap();
        assert_eq!(report.initial_text_author(), expected);
        assert_eq!(report.pair_author(), HtmlControlLengthPairState::Conforming);
    }
}

#[test]
fn wrong_initial_text_fact_family_and_value_identity_remain_explicit() {
    let interpreter = HtmlValueInterpreter::new(64);
    let maximum = raw(A::Maxlength, "4");
    let textarea = HtmlControlInitialText::TextareaDescendantText(DocumentString::from("x"));
    let input = HtmlControlInitialText::InputValue(Some(raw(A::Value, "x")));
    let wrong_value = HtmlControlInitialText::InputValue(Some(Attribute::new(
        QualifiedName::new(Namespace::svg(), None, "value"),
        "xxxxx",
    )));
    let wrong_name = HtmlControlInitialText::InputValue(Some(raw(A::Size, "xxxxx")));
    for (element, state, facts, expected) in [
        (
            E::Input,
            Some(HtmlInputType::Text),
            &textarea,
            HtmlControlInitialTextState::DifferentContext,
        ),
        (
            E::Textarea,
            None,
            &input,
            HtmlControlInitialTextState::DifferentContext,
        ),
        (
            E::Input,
            Some(HtmlInputType::Text),
            &wrong_value,
            HtmlControlInitialTextState::DifferentValueAttribute,
        ),
        (
            E::Input,
            Some(HtmlInputType::Text),
            &wrong_name,
            HtmlControlInitialTextState::DifferentValueAttribute,
        ),
    ] {
        let report = interpreter
            .control_length_bounds(
                &element.qualified_name(),
                None::<&surgeist_html::Attribute>,
                Some(&maximum),
                state,
                Some(facts),
            )
            .unwrap();
        assert_eq!(report.initial_text_author(), expected);
        assert_eq!(report.initial_text(), Some(facts));
        assert_eq!(bound(report.maximum().content().unwrap()), Some("4"));
    }
}

#[test]
fn every_supplied_value_is_admitted_before_wrong_names_roles_or_ignored_states() {
    let interpreter = HtmlValueInterpreter::new(4);
    let short_min = raw(A::Minlength, "1");
    let long_min = raw(A::Minlength, "12345");
    let long_wrong_max = raw(A::Size, "12345");
    let long_wrong_fact = HtmlControlInitialText::InputValue(Some(raw(A::Size, "12345")));
    for (min, max, facts) in [
        (Some(&long_min), None, None),
        (Some(&short_min), Some(&long_wrong_max), None),
        (Some(&short_min), None, Some(&long_wrong_fact)),
    ] {
        let error = interpreter
            .control_length_bounds(
                &E::Div.qualified_name(),
                min,
                max,
                Some(HtmlInputType::Number),
                facts,
            )
            .unwrap_err();
        let HtmlControlLengthError::Admission(error) = error else {
            panic!("raw admission expected")
        };
        assert_eq!(error.capacity(), 4);
        assert_eq!(error.required(), 5);
    }
    let size = raw(A::Size, "12345");
    assert_eq!(
        interpreter
            .input_size_for_input(
                &E::Input.qualified_name(),
                Some(&size),
                HtmlInputType::Checkbox
            )
            .unwrap_err()
            .required(),
        5
    );
}

#[test]
fn per_value_capacity_is_not_an_aggregate_limit_and_total_is_retained() {
    let minimum = raw(A::Minlength, "0001");
    let maximum = raw(A::Maxlength, "0004");
    let initial = HtmlControlInitialText::TextareaDescendantText(DocumentString::from("abcd"));
    let report = HtmlValueInterpreter::new(4)
        .control_length_bounds(
            &E::Textarea.qualified_name(),
            Some(&minimum),
            Some(&maximum),
            None,
            Some(&initial),
        )
        .unwrap();
    assert_eq!(report.input_code_units(), 12);
    assert_eq!(
        report.initial_text_author(),
        HtmlControlInitialTextState::Conforming { code_units: 4 }
    );
    let report = HtmlValueInterpreter::new(0)
        .control_length_bounds(
            &E::Textarea.qualified_name(),
            Some(&raw(A::Minlength, "")),
            Some(&raw(A::Maxlength, "")),
            None,
            Some(&HtmlControlInitialText::TextareaDescendantText(
                DocumentString::from(""),
            )),
        )
        .unwrap();
    assert_eq!(report.input_code_units(), 0);
    assert_eq!(
        report.minimum().observation(),
        HtmlAttributeObservation::PresentEmpty
    );
    assert_eq!(
        report.maximum().content(),
        Some(&HtmlControlLengthValue::NoBound)
    );
}

#[test]
fn source_rows_and_role_definitions_are_selected_without_reinventing_inventory() {
    let interpreter = HtmlValueInterpreter::new(64);
    for (element, state, prefix) in [
        (E::Input, Some(HtmlInputType::Text), "input"),
        (E::Textarea, None, "textarea"),
    ] {
        let report = interpreter
            .control_length_bounds(
                &element.qualified_name(),
                None::<&surgeist_html::Attribute>,
                None::<&surgeist_html::Attribute>,
                state,
                None,
            )
            .unwrap();
        assert_eq!(report.minimum().inventory_rows(), &[99]);
        assert_eq!(report.maximum().inventory_rows(), &[94]);
        assert!(
            report
                .minimum()
                .source_clause()
                .ends_with(&format!("#attr-{prefix}-minlength"))
        );
        assert!(
            report
                .maximum()
                .source_clause()
                .ends_with(&format!("#attr-{prefix}-maxlength"))
        );
        assert!(
            report
                .minimum()
                .processing_source_clause()
                .ends_with("#attr-fe-minlength")
        );
        assert!(
            report
                .maximum()
                .processing_source_clause()
                .ends_with("#attr-fe-maxlength")
        );
        assert!(
            report
                .maximum()
                .getter_source_clause()
                .ends_with("#reflecting-content-attributes-in-idl-attributes")
        );
    }
    let size = interpreter
        .input_size(
            &E::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(size.inventory_rows(), &[143]);
    assert!(size.source_clause().ends_with("#attr-input-size"));
}

#[test]
fn native_authoring_uses_exact_canonical_digits_and_separate_positive_size_rule() {
    assert_eq!(
        HtmlNonNegativeInteger::from(0).into_input_size_attribute(),
        Err(HtmlInputSizeZeroError)
    );
    assert_eq!(
        HtmlNonNegativeInteger::from(0).into_minlength_attribute(),
        raw(A::Minlength, "0")
    );
    assert_eq!(
        HtmlNonNegativeInteger::from(0).into_maxlength_attribute(),
        raw(A::Maxlength, "0")
    );
    let huge = "999999999999999999999999999999999999999999999999999999999999";
    let value =
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(format!("000{huge}")))
            .unwrap();
    assert_eq!(
        value.clone().into_input_size_attribute().unwrap(),
        raw(A::Size, huge)
    );
    assert_eq!(
        value.clone().into_minlength_attribute(),
        raw(A::Minlength, huge)
    );
    assert_eq!(value.into_maxlength_attribute(), raw(A::Maxlength, huge));
}

#[test]
fn failed_bound_recovery_is_retained_separately_from_missing_bound() {
    let maximum = raw(A::Maxlength, "-1");
    let report = length_report(None, Some(&maximum));
    assert!(matches!(
        report.minimum().common().unwrap().recovery(),
        HtmlValueRecoveryState::Missing
    ));
    assert!(matches!(
        report.maximum().common().unwrap().recovery(),
        HtmlValueRecoveryState::Failed(_)
    ));
    assert_eq!(
        report.maximum().observation(),
        HtmlAttributeObservation::PresentNonempty
    );
}

#[test]
fn absent_or_failed_maximum_and_absent_input_value_need_no_text_state_fact() {
    let interpreter = HtmlValueInterpreter::new(64);
    let invalid = raw(A::Maxlength, "invalid");
    let valid = raw(A::Maxlength, "4");
    let absent_value = HtmlControlInitialText::InputValue(None);
    for (maximum, initial) in [
        (None, None),
        (Some(&invalid), None),
        (Some(&valid), Some(&absent_value)),
    ] {
        let report = interpreter
            .control_length_bounds(
                &E::Input.qualified_name(),
                None::<&surgeist_html::Attribute>,
                maximum,
                None,
                initial,
            )
            .unwrap();
        assert_eq!(
            report.initial_text_author(),
            HtmlControlInitialTextState::NotRequired
        );
        assert_eq!(
            report.maximum().application(),
            HtmlControlAttributeApplication::InputStateRequired
        );
    }
}

#[test]
fn input_state_context_is_retained_when_initial_text_check_is_ignored() {
    let maximum = raw(A::Maxlength, "0");
    let initial = HtmlControlInitialText::InputValue(Some(raw(A::Value, "x")));
    let report = HtmlValueInterpreter::new(64)
        .control_length_bounds(
            &E::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
            Some(&maximum),
            Some(HtmlInputType::Number),
            Some(&initial),
        )
        .unwrap();
    assert_eq!(report.maximum().recovered().unwrap().decimal_digits(), "0");
    assert_eq!(report.maximum().reflected_getter(), Some(0));
    assert_eq!(
        report.initial_text_author(),
        HtmlControlInitialTextState::IgnoredInputState
    );
    assert_eq!(report.initial_text(), Some(&initial));
}

#[test]
fn invalid_maximum_identity_does_not_check_or_discard_initial_text() {
    let minimum = raw(A::Minlength, "1");
    let maximum = raw(A::Size, "0");
    let initial = HtmlControlInitialText::TextareaDescendantText(DocumentString::from("abc"));
    let report = HtmlValueInterpreter::new(64)
        .control_length_bounds(
            &E::Textarea.qualified_name(),
            Some(&minimum),
            Some(&maximum),
            None,
            Some(&initial),
        )
        .unwrap();
    assert_eq!(
        report.initial_text_author(),
        HtmlControlInitialTextState::Unapplied
    );
    assert_eq!(report.initial_text(), Some(&initial));
    assert_eq!(bound(report.minimum().content().unwrap()), Some("1"));
}

#[test]
fn prefixed_input_value_has_unnamespaced_expanded_identity() {
    let maximum = raw(A::Maxlength, "2");
    let initial = HtmlControlInitialText::InputValue(Some(Attribute::new(
        QualifiedName::new(Namespace::none(), Some(DocumentString::from("p")), "value"),
        "xy",
    )));
    let report = HtmlValueInterpreter::new(64)
        .control_length_bounds(
            &E::Input.qualified_name(),
            None::<&surgeist_html::Attribute>,
            Some(&maximum),
            Some(HtmlInputType::Email),
            Some(&initial),
        )
        .unwrap();
    assert_eq!(
        report.initial_text_author(),
        HtmlControlInitialTextState::Conforming { code_units: 2 }
    );
    assert_eq!(report.initial_text(), Some(&initial));
}
