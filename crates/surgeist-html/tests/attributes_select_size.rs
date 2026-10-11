//! Literal source oracles: HTML 877d #attr-select-size/#concept-select-size,
//! #attr-select-multiple, §2.3.2 and §2.3.4.2. Display size is not size IDL.
use surgeist_html::*;

fn attr(name: KnownHtmlAttribute, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(name.qualified_name(), value)
}
fn read(size: Option<&str>, multiple: Option<&str>) -> HtmlSelectSizeReport {
    let size = size.map(|v| attr(KnownHtmlAttribute::Size, v));
    let multiple = multiple.map(|v| attr(KnownHtmlAttribute::Multiple, v));
    HtmlValueInterpreter::new(256)
        .select_size(
            &KnownHtmlElement::Select.qualified_name(),
            size.as_ref(),
            multiple.as_ref(),
        )
        .unwrap()
}
fn known(
    report: &HtmlSelectSizeReport,
    digits: &str,
    expected_origin: HtmlNumericAttributeValueOrigin,
) {
    let HtmlSelectDisplaySize::Value { value, origin } = report.display_size() else {
        panic!("expected known display size")
    };
    assert_eq!(value.decimal_digits(), digits);
    assert_eq!(origin, expected_origin);
}
#[test]
fn absent_size_uses_actual_multiple_presence_and_keeps_missing_common_state() {
    for (multiple, expected) in [
        (None, "1"),
        (Some(""), "4"),
        (Some("multiple"), "4"),
        (Some("false"), "4"),
    ] {
        let report = read(None, multiple);
        assert_eq!(report.observation(), HtmlAttributeObservation::Missing);
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Missing);
        assert_eq!(
            report.common().unwrap().author(),
            &HtmlValueAuthorState::Missing
        );
        assert_eq!(
            report.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Missing
        );
        known(
            &report,
            expected,
            HtmlNumericAttributeValueOrigin::MissingDefault,
        );
    }
    let report = read(None, Some("false"));
    assert_eq!(report.multiple().intrinsic(), Some(&true));
    assert_eq!(
        report.multiple().author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::BooleanSpelling)
    );
    assert_eq!(read(None, None).multiple().intrinsic(), Some(&false));
}
#[test]
fn failed_size_parse_uses_one_or_four_without_erasing_author_failures() {
    for size in ["", " \t\n", "nope", "-1", "-999999999999999999999999999999"] {
        for (multiple, expected) in [(None, "1"), (Some(""), "4")] {
            let report = read(Some(size), multiple);
            assert!(matches!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(_))
                )
            ));
            assert!(matches!(
                report.common().unwrap().author(),
                HtmlValueAuthorState::Invalid(_)
            ));
            assert!(matches!(
                report.common().unwrap().recovery(),
                HtmlValueRecoveryState::Failed(_)
            ));
            assert_eq!(report.raw(), Some(&DocumentString::from(size)));
            known(
                &report,
                expected,
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
            );
        }
    }
}
#[test]
fn positive_author_values_recover_exactly_and_borrow_the_common_integer() {
    for (raw, expected) in [
        ("1", "1"),
        ("2", "2"),
        ("0002", "2"),
        (
            "340282366920938463463374607431768211456",
            "340282366920938463463374607431768211456",
        ),
    ] {
        for multiple in [None, Some(""), Some("false")] {
            let report = read(Some(raw), multiple);
            assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
            known(
                &report,
                expected,
                HtmlNumericAttributeValueOrigin::Recovered,
            );
            let HtmlSelectDisplaySize::Value { value, .. } = report.display_size() else {
                panic!("known")
            };
            let HtmlValueRecoveryState::Value(common) = report.common().unwrap().recovery() else {
                panic!("common recovery")
            };
            assert!(std::ptr::eq(value, common));
            assert_eq!(report.raw(), Some(&DocumentString::from(raw)));
        }
    }
}
#[test]
fn authored_zero_is_invalid_but_successful_recovery_never_selects_a_fallback() {
    for raw in ["0", "0000"] {
        for multiple in [None, Some(""), Some("false")] {
            let report = read(Some(raw), multiple);
            assert_eq!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::ZeroMustBePositive
                )
            );
            assert!(
                matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.is_zero())
            );
            known(&report, "0", HtmlNumericAttributeValueOrigin::Recovered);
        }
    }
}
#[test]
fn negative_zero_and_recovery_only_prefixes_keep_independent_author_evidence() {
    for (raw, expected) in [
        ("-0", "0"),
        ("-000tail", "0"),
        ("+2junk", "2"),
        (" 2 ", "2"),
    ] {
        for multiple in [None, Some("")] {
            let report = read(Some(raw), multiple);
            assert!(matches!(
                report.common().unwrap().author(),
                HtmlValueAuthorState::Invalid(_)
            ));
            assert!(matches!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::CommonValue(_)
                )
            ));
            known(
                &report,
                expected,
                HtmlNumericAttributeValueOrigin::Recovered,
            );
        }
    }
}
#[test]
fn wrong_multiple_role_is_unavailable_only_when_a_fallback_is_needed() {
    let wrong = attr(KnownHtmlAttribute::Required, "required");
    for raw in [None, Some(""), Some("bad"), Some("-1")] {
        let size = raw.map(|raw| attr(KnownHtmlAttribute::Size, raw));
        let report = HtmlValueInterpreter::new(256)
            .select_size(
                &KnownHtmlElement::Select.qualified_name(),
                size.as_ref(),
                Some(&wrong),
            )
            .unwrap();
        assert_eq!(report.multiple().raw_attribute(), Some(&wrong));
        assert_eq!(
            report.multiple().applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(
            report.display_size(),
            HtmlSelectDisplaySize::MultipleRoleUnavailable(
                HtmlAttributeApplicability::DifferentAttribute
            )
        );
        assert!(report.common().is_some());
    }
    for (raw, expected) in [("2", "2"), ("0", "0"), ("-0", "0")] {
        let size = attr(KnownHtmlAttribute::Size, raw);
        let report = HtmlValueInterpreter::new(256)
            .select_size(
                &KnownHtmlElement::Select.qualified_name(),
                Some(&size),
                Some(&wrong),
            )
            .unwrap();
        known(
            &report,
            expected,
            HtmlNumericAttributeValueOrigin::Recovered,
        );
        assert_eq!(report.multiple().intrinsic(), None);
    }
}
#[test]
fn wrong_size_role_never_runs_numeric_grammar_and_keeps_multiple_observation() {
    let size = attr(KnownHtmlAttribute::Cols, "2");
    let multiple = attr(KnownHtmlAttribute::Multiple, "false");
    let report = HtmlValueInterpreter::new(256)
        .select_size(
            &KnownHtmlElement::Select.qualified_name(),
            Some(&size),
            Some(&multiple),
        )
        .unwrap();
    assert_eq!(report.raw_attribute(), Some(&size));
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::DifferentAttribute
    );
    assert!(report.common().is_none());
    assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
    assert_eq!(
        report.display_size(),
        HtmlSelectDisplaySize::Unapplied(HtmlAttributeApplicability::DifferentAttribute)
    );
    assert_eq!(report.multiple().intrinsic(), Some(&true));
    assert_eq!(
        report.multiple().author(),
        HtmlAttributeAuthorState::Nonconforming(HtmlAttributeConformanceError::BooleanSpelling)
    );
}
#[test]
fn exact_expanded_roles_apply_independently_of_prefix_and_preserve_source_metadata() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "select");
    let size = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("s".into()), "size"),
        "0002",
    );
    let multiple = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("m".into()), "multiple"),
        "MuLtIpLe",
    );
    let report = HtmlValueInterpreter::new(256)
        .select_size(&element, Some(&size), Some(&multiple))
        .unwrap();
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&size));
    assert_eq!(report.multiple().raw_attribute(), Some(&multiple));
    assert_eq!(
        report.observation(),
        HtmlAttributeObservation::PresentNonempty
    );
    assert_eq!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.attribute(), KnownHtmlAttribute::Size);
    assert_eq!(report.inventory_rows(), &[143]);
    assert_eq!(report.multiple().inventory_rows(), &[100]);
    assert!(report.source_clause().ends_with("/#attr-select-size"));
    assert!(
        report
            .multiple()
            .source_clause()
            .ends_with("/#attr-select-multiple")
    );
    assert!(
        report
            .processing_source_clause()
            .ends_with("/#concept-select-size")
    );
    assert_eq!(report.input_code_units(), 4);
    assert_eq!(report.multiple().input_code_units(), 8);
    assert_eq!(
        report.multiple().author(),
        HtmlAttributeAuthorState::Conforming
    );
    known(&report, "2", HtmlNumericAttributeValueOrigin::Recovered);
}
#[test]
fn wrong_element_namespace_case_and_attribute_namespace_do_not_guess_defaults() {
    let size = attr(KnownHtmlAttribute::Size, "2");
    for (element, applicability) in [
        (
            KnownHtmlElement::Input.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "SELECT"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            QualifiedName::new(Namespace::svg(), None, "select"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
    ] {
        let report = HtmlValueInterpreter::new(256)
            .select_size(&element, Some(&size), None)
            .unwrap();
        assert_eq!(report.applicability(), applicability);
        assert_eq!(
            report.display_size(),
            HtmlSelectDisplaySize::Unapplied(applicability)
        );
        assert!(report.common().is_none());
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
    }
    for name in [
        QualifiedName::new(Namespace::html(), None, "size"),
        QualifiedName::new(Namespace::none(), None, "SIZE"),
    ] {
        let wrong = Attribute::new(name, "2");
        let report = HtmlValueInterpreter::new(256)
            .select_size(
                &KnownHtmlElement::Select.qualified_name(),
                Some(&wrong),
                None,
            )
            .unwrap();
        assert!(report.common().is_none());
        assert_eq!(
            report.display_size(),
            HtmlSelectDisplaySize::Unapplied(HtmlAttributeApplicability::DifferentAttribute)
        );
    }
}
#[test]
fn utf16_and_surrogates_stay_exact_while_shared_recovery_observes_prefix_only() {
    let size = attr(
        KnownHtmlAttribute::Size,
        DocumentString::from_utf16(vec![50, 0xD800]),
    );
    let multiple = attr(
        KnownHtmlAttribute::Multiple,
        DocumentString::from_utf16(vec![0xD83D, 0xDE00, 0xDC00]),
    );
    let report = HtmlValueInterpreter::new(3)
        .select_size(
            &KnownHtmlElement::Select.qualified_name(),
            Some(&size),
            Some(&multiple),
        )
        .unwrap();
    assert_eq!(report.raw(), Some(size.value()));
    assert_eq!(report.multiple().raw(), Some(multiple.value()));
    assert_eq!(report.input_code_units(), 2);
    assert_eq!(report.multiple().input_code_units(), 3);
    assert!(matches!(
        report.author(),
        HtmlNumericAttributeAuthorState::Nonconforming(_)
    ));
    known(&report, "2", HtmlNumericAttributeValueOrigin::Recovered);
    let missing = HtmlValueInterpreter::new(3)
        .select_size(
            &KnownHtmlElement::Select.qualified_name(),
            None,
            Some(&multiple),
        )
        .unwrap();
    known(
        &missing,
        "4",
        HtmlNumericAttributeValueOrigin::MissingDefault,
    );
}
#[test]
fn each_slot_admits_exact_raw_length_before_roles_or_irrelevant_presence() {
    let element = KnownHtmlElement::Select.qualified_name();
    let size = attr(KnownHtmlAttribute::Size, "0002");
    let multiple = attr(KnownHtmlAttribute::Multiple, "xxxx");
    let report = HtmlValueInterpreter::new(4)
        .select_size(&element, Some(&size), Some(&multiple))
        .unwrap();
    known(&report, "2", HtmlNumericAttributeValueOrigin::Recovered);
    let failure = HtmlValueInterpreter::new(3)
        .select_size(&element, Some(&size), None)
        .unwrap_err();
    assert_eq!((failure.capacity(), failure.required()), (3, 4));
    let short_size = attr(KnownHtmlAttribute::Size, "2");
    let failure = HtmlValueInterpreter::new(3)
        .select_size(&element, Some(&short_size), Some(&multiple))
        .unwrap_err();
    assert_eq!((failure.capacity(), failure.required()), (3, 4));
    let wrong = attr(KnownHtmlAttribute::Cite, "xxxxx");
    for (size, multiple) in [(Some(&wrong), None), (None, Some(&wrong))] {
        let failure = HtmlValueInterpreter::new(4)
            .select_size(&KnownHtmlElement::Div.qualified_name(), size, multiple)
            .unwrap_err();
        assert_eq!((failure.capacity(), failure.required()), (4, 5));
    }
    known(
        &HtmlValueInterpreter::new(0)
            .select_size(&element, None, None)
            .unwrap(),
        "1",
        HtmlNumericAttributeValueOrigin::MissingDefault,
    );
}
#[test]
fn native_constructor_rejects_zero_and_authors_canonical_unbounded_positive_values() {
    assert_eq!(
        HtmlNonNegativeInteger::from(0_u128).into_select_size_attribute(),
        Err(HtmlNumericAttributeConformanceError::ZeroMustBePositive)
    );
    for (digits, canonical) in [
        ("0002", "2"),
        (
            "340282366920938463463374607431768211456",
            "340282366920938463463374607431768211456",
        ),
    ] {
        let value =
            HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(digits)).unwrap();
        let raw = value.into_select_size_attribute().unwrap();
        assert_eq!(raw.name(), &KnownHtmlAttribute::Size.qualified_name());
        assert_eq!(raw.value(), &DocumentString::from(canonical));
        let report = HtmlValueInterpreter::new(256)
            .select_size(&KnownHtmlElement::Select.qualified_name(), Some(&raw), None)
            .unwrap();
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
        known(
            &report,
            canonical,
            HtmlNumericAttributeValueOrigin::Recovered,
        );
    }
}
#[test]
fn old_report_survives_later_inputs_and_shared_and_neighbor_rules_are_unchanged() {
    let old = read(Some("-0"), Some("false"));
    let later = read(None, None);
    known(&old, "0", HtmlNumericAttributeValueOrigin::Recovered);
    known(&later, "1", HtmlNumericAttributeValueOrigin::MissingDefault);
    assert_eq!(old.raw(), Some(&DocumentString::from("-0")));
    assert_eq!(old.multiple().raw(), Some(&DocumentString::from("false")));
    let interpreter = HtmlValueInterpreter::new(256);
    assert_eq!(
        old.common(),
        Some(
            &interpreter
                .nonnegative_integer(Some(&DocumentString::from("-0")))
                .unwrap()
        )
    );
    let zero_rows = attr(KnownHtmlAttribute::Rows, "0");
    let rows = interpreter
        .textarea_rows(
            &KnownHtmlElement::Textarea.qualified_name(),
            Some(&zero_rows),
        )
        .unwrap();
    assert_eq!(rows.intrinsic().unwrap().decimal_digits(), "2");
    assert_eq!(
        rows.origin(),
        Some(HtmlNumericAttributeValueOrigin::ZeroDefault)
    );
    let zero_cols = attr(KnownHtmlAttribute::Cols, "0");
    let cols = interpreter
        .textarea_cols(
            &KnownHtmlElement::Textarea.qualified_name(),
            Some(&zero_cols),
        )
        .unwrap();
    assert_eq!(cols.intrinsic().unwrap().decimal_digits(), "20");
    assert_eq!(
        cols.origin(),
        Some(HtmlNumericAttributeValueOrigin::ZeroDefault)
    );
    let size = attr(KnownHtmlAttribute::Size, "2");
    let input_role = interpreter
        .input_size(&KnownHtmlElement::Select.qualified_name(), Some(&size))
        .unwrap();
    assert_eq!(
        input_role.application(),
        HtmlControlAttributeApplication::Unapplied(HtmlAttributeApplicability::DifferentElement)
    );
}
