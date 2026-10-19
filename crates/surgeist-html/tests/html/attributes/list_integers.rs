use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeAuthorState,
    HtmlAttributeContextAuthorState, HtmlAttributeObservation, HtmlAttributeRecognition,
    HtmlInteger, HtmlListIntegerAttributeReport, HtmlNonNegativeInteger,
    HtmlNumericAttributeAuthorState, HtmlNumericAttributeConformanceError, HtmlValueAdmissionError,
    HtmlValueAuthorState, HtmlValueFailure, HtmlValueInterpreter, HtmlValueRecoveryState,
    HtmlValueSyntax, HtmlValueSyntaxError, KnownHtmlAttribute, KnownHtmlElement, Namespace,
    QualifiedName,
};

type Operation = fn(
    &HtmlValueInterpreter,
    &QualifiedName,
    Option<&Attribute>,
) -> Result<HtmlListIntegerAttributeReport, HtmlValueAdmissionError>;

#[derive(Clone, Copy)]
struct Case {
    read: Operation,
    element: KnownHtmlElement,
    name: KnownHtmlAttribute,
    row: u16,
    source: &'static str,
    present_context: HtmlAttributeContextAuthorState,
}

const CASES: [Case; 2] = [
    Case {
        read: HtmlValueInterpreter::ol_start,
        element: KnownHtmlElement::Ol,
        name: KnownHtmlAttribute::Start,
        row: 153,
        source: "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-ol-start",
        present_context: HtmlAttributeContextAuthorState::NotRequired,
    },
    Case {
        read: HtmlValueInterpreter::li_value,
        element: KnownHtmlElement::Li,
        name: KnownHtmlAttribute::Value,
        row: 176,
        source: "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-li-value",
        present_context: HtmlAttributeContextAuthorState::Unchecked,
    },
];

fn raw(case: Case, value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(case.name.qualified_name(), value)
}
fn read(case: Case, raw: Option<&Attribute>) -> HtmlListIntegerAttributeReport {
    (case.read)(
        &HtmlValueInterpreter::new(8192),
        &case.element.qualified_name(),
        raw,
    )
    .unwrap()
}
fn invalid_author(report: &HtmlListIntegerAttributeReport) {
    assert_eq!(
        report.author(),
        HtmlNumericAttributeAuthorState::Nonconforming(
            HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(
                HtmlValueSyntaxError::InvalidNumericSyntax,
            ))
        )
    );
    assert_eq!(
        report.common().unwrap().author(),
        &HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::InvalidNumericSyntax)
    );
}
fn recovered(report: &HtmlListIntegerAttributeReport, expected: &str) {
    let integer = report.content_integer().unwrap();
    assert_eq!(integer.as_document_string(), DocumentString::from(expected));
    let HtmlValueRecoveryState::Value(common) = report.common().unwrap().recovery() else {
        panic!("expected exact integer recovery");
    };
    assert_eq!(integer, common);
}

#[test]
fn missing_and_failed_content_do_not_invent_a_start_or_ordinal() {
    for case in CASES {
        let missing = read(case, None);
        assert_eq!(
            missing.attribute().observation(),
            HtmlAttributeObservation::Missing
        );
        assert_eq!(
            missing.attribute().applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(missing.author(), HtmlNumericAttributeAuthorState::Missing);
        assert_eq!(
            missing.common().unwrap().author(),
            &HtmlValueAuthorState::Missing
        );
        assert_eq!(
            missing.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Missing
        );
        assert_eq!(
            missing.attribute().context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
        assert_eq!(missing.attribute().raw_attribute(), None);
        assert_eq!(missing.content_integer(), None);

        let empty = raw(case, "");
        let report = read(case, Some(&empty));
        assert_eq!(
            report.attribute().observation(),
            HtmlAttributeObservation::PresentEmpty
        );
        assert_eq!(report.attribute().raw_attribute(), Some(&empty));
        assert_eq!(report.attribute().context_author(), case.present_context);
        invalid_author(&report);
        assert_eq!(
            report.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(
                HtmlValueSyntaxError::InvalidNumericSyntax
            ))
        );
        assert_eq!(report.content_integer(), None);
    }
}

#[test]
fn whole_signed_integer_author_forms_keep_raw_sign_and_leading_zeroes() {
    // The defining clauses select valid integer, not nonnegative/positive integer.
    for case in CASES {
        for (text, expected) in [
            ("0", "0"),
            ("-0", "0"),
            ("-0000", "0"),
            ("00042", "42"),
            ("42", "42"),
            ("-42", "-42"),
        ] {
            let attribute = raw(case, text);
            let report = read(case, Some(&attribute));
            assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
            let HtmlValueAuthorState::Valid(author) = report.common().unwrap().author() else {
                panic!("expected whole signed author syntax");
            };
            assert_eq!(author.as_document_string(), DocumentString::from(expected));
            recovered(&report, expected);
            assert_eq!(report.attribute().raw(), Some(&DocumentString::from(text)));
            assert_eq!(
                report.common().unwrap().raw(),
                Some(&DocumentString::from(text))
            );
            assert_eq!(report.attribute().context_author(), case.present_context);
            assert_eq!(
                report.attribute().recognition(),
                HtmlAttributeRecognition::Unapplied
            );
            assert_eq!(
                report.attribute().author(),
                HtmlAttributeAuthorState::Unapplied
            );
            assert_eq!(report.attribute().intrinsic(), None);
        }
    }
}

#[test]
fn nonconforming_prefix_recovery_never_becomes_local_author_success() {
    for case in CASES {
        for (text, expected) in [
            ("+2", "2"),
            (" \t+2tail", "2"),
            ("1.5", "1"),
            ("-12junk", "-12"),
            ("12 ", "12"),
            ("-0junk", "0"),
            ("1e999", "1"),
            ("2\0", "2"),
        ] {
            let attribute = raw(case, text);
            let report = read(case, Some(&attribute));
            invalid_author(&report);
            recovered(&report, expected);
            assert_eq!(report.attribute().raw_attribute(), Some(&attribute));
            assert_eq!(report.attribute().context_author(), case.present_context);
        }
    }
}

#[test]
fn rejected_prefixes_have_typed_failure_and_no_local_default() {
    for case in CASES {
        for text in [
            "+",
            "-",
            " \t\n\r\u{c}",
            "x",
            "１２",
            "\u{a0}2",
            "--1",
            "+-1",
        ] {
            let attribute = raw(case, text);
            let report = read(case, Some(&attribute));
            invalid_author(&report);
            assert_eq!(report.content_integer(), None);
            assert_eq!(
                report.common().unwrap().recovery(),
                &HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(
                    HtmlValueSyntaxError::InvalidNumericSyntax
                ))
            );
            assert_eq!(report.attribute().raw_attribute(), Some(&attribute));
        }
    }
}

#[test]
fn arbitrary_signed_magnitudes_are_exact_without_machine_projection_or_clamping() {
    const HUGE: &str = "123456789012345678901234567890123456789012345678901234567890";
    const NEGATIVE: &str = "-123456789012345678901234567890123456789012345678901234567890";
    for case in CASES {
        for text in [HUGE, NEGATIVE] {
            let attribute = raw(case, text);
            let report = read(case, Some(&attribute));
            assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
            recovered(&report, text);
            assert!(report.content_integer().unwrap().to_i128().is_err());
            assert_eq!(report.attribute().raw_attribute(), Some(&attribute));
        }
        let attribute = raw(case, format!(" \t+{HUGE}junk"));
        let report = read(case, Some(&attribute));
        invalid_author(&report);
        recovered(&report, HUGE);
        let long = "9".repeat(4096);
        let attribute = raw(case, long.as_str());
        let report = read(case, Some(&attribute));
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
        recovered(&report, &long);
        assert_eq!(report.attribute().input_code_units(), 4096);
        let error = (case.read)(
            &HtmlValueInterpreter::new(4095),
            &case.element.qualified_name(),
            Some(&attribute),
        )
        .unwrap_err();
        assert_eq!(error.capacity(), 4095);
        assert_eq!(error.required(), 4096);
    }
}

#[test]
fn source_and_prefix_records_follow_exact_expanded_roles() {
    for case in CASES {
        let element = QualifiedName::new(
            Namespace::html(),
            Some("h".into()),
            case.element.local_name(),
        );
        let attribute = Attribute::new(
            QualifiedName::new(
                Namespace::none(),
                Some("raw-prefix".into()),
                case.name.local_name(),
            ),
            "-7",
        );
        let report =
            (case.read)(&HtmlValueInterpreter::new(32), &element, Some(&attribute)).unwrap();
        recovered(&report, "-7");
        assert_eq!(report.attribute().element(), &element);
        assert_eq!(report.attribute().raw_attribute(), Some(&attribute));
        assert_eq!(report.attribute().attribute(), case.name);
        assert_eq!(report.attribute().inventory_rows(), &[case.row]);
        assert_eq!(report.attribute().source_clause(), case.source);
        assert_eq!(report.attribute().vocabulary_source_clause(), None);
        assert_eq!(report.common().unwrap().syntax(), HtmlValueSyntax::Integer);
        assert_eq!(report.attribute().context_author(), case.present_context);
    }
}

#[test]
fn wrong_role_case_namespace_and_custom_elements_run_no_integer_grammar() {
    for case in CASES {
        let element = case.element.qualified_name();
        let correct = raw(case, "not an integer");
        let other_name = if case.name == KnownHtmlAttribute::Start {
            KnownHtmlAttribute::Value
        } else {
            KnownHtmlAttribute::Start
        };
        let wrong = Attribute::new(other_name.qualified_name(), "not an integer");
        let uppercase = Attribute::new(
            QualifiedName::new(
                Namespace::none(),
                None,
                case.name.local_name().to_ascii_uppercase(),
            ),
            "",
        );
        let namespaced = Attribute::new(
            QualifiedName::new(Namespace::svg(), Some("s".into()), case.name.local_name()),
            "-7",
        );
        let other_element = if case.element == KnownHtmlElement::Ol {
            KnownHtmlElement::Li
        } else {
            KnownHtmlElement::Ol
        };
        for (element, attribute, expected) in [
            (
                other_element.qualified_name(),
                &correct,
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(
                    Namespace::html(),
                    None,
                    case.element.local_name().to_ascii_uppercase(),
                ),
                &correct,
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(Namespace::html(), None, "x-list"),
                &correct,
                HtmlAttributeApplicability::DifferentElement,
            ),
            (
                QualifiedName::new(Namespace::svg(), None, case.element.local_name()),
                &correct,
                HtmlAttributeApplicability::ForeignElementNamespace,
            ),
            (
                QualifiedName::new(Namespace::none(), None, case.element.local_name()),
                &correct,
                HtmlAttributeApplicability::ForeignElementNamespace,
            ),
            (
                element.clone(),
                &wrong,
                HtmlAttributeApplicability::DifferentAttribute,
            ),
            (
                element.clone(),
                &uppercase,
                HtmlAttributeApplicability::DifferentAttribute,
            ),
            (
                element.clone(),
                &namespaced,
                HtmlAttributeApplicability::DifferentAttribute,
            ),
        ] {
            let report =
                (case.read)(&HtmlValueInterpreter::new(32), &element, Some(attribute)).unwrap();
            assert_eq!(report.attribute().applicability(), expected);
            assert_eq!(report.attribute().raw_attribute(), Some(attribute));
            assert_eq!(report.attribute().element(), &element);
            assert_eq!(
                report.attribute().context_author(),
                HtmlAttributeContextAuthorState::Unapplied
            );
            assert_eq!(
                report.attribute().recognition(),
                HtmlAttributeRecognition::Unapplied
            );
            assert_eq!(
                report.attribute().author(),
                HtmlAttributeAuthorState::Unapplied
            );
            assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
            assert_eq!(report.common(), None);
            assert_eq!(report.content_integer(), None);
        }
        let report = (case.read)(
            &HtmlValueInterpreter::new(0),
            &other_element.qualified_name(),
            None,
        )
        .unwrap();
        assert_eq!(
            report.attribute().observation(),
            HtmlAttributeObservation::Missing
        );
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
        assert_eq!(report.common(), None);
        assert_eq!(report.content_integer(), None);
    }
}

#[test]
fn li_parent_author_evidence_remains_unchecked_independently_of_numeric_validity() {
    for text in ["5", "-5", "0", "+5tail", "bad", ""] {
        let attribute = raw(CASES[1], text);
        let report = read(CASES[1], Some(&attribute));
        assert_eq!(
            report.attribute().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(report.attribute().raw_attribute(), Some(&attribute));
    }
    let missing = read(CASES[1], None);
    assert_eq!(
        missing.attribute().context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    // Neither local reader receives reversed/parent/owner facts. It cannot infer
    // ascending default 1, reversed count or an item's ordinal from absence.
    assert_eq!(read(CASES[0], None).content_integer(), None);
    assert_eq!(missing.content_integer(), None);
}

#[test]
fn surrogate_units_remain_exact_while_only_ascii_digit_prefixes_recover() {
    for case in CASES {
        for (units, expected) in [
            (vec![0xD800, 0x32], None),
            (vec![0xD83D, 0xDE00, 0x32], None),
            (vec![0x32, 0xD800], Some("2")),
            (vec![0x2D, 0x32, 0xDC00], Some("-2")),
            (vec![0x32, 0xFF13], Some("2")),
        ] {
            let attribute = raw(case, DocumentString::from_utf16(units.clone()));
            let report = read(case, Some(&attribute));
            invalid_author(&report);
            assert_eq!(report.attribute().raw().unwrap().code_units(), units);
            assert_eq!(report.common().unwrap().raw().unwrap().code_units(), units);
            match expected {
                Some(expected) => recovered(&report, expected),
                None => assert_eq!(report.content_integer(), None),
            }
        }
    }
}

#[test]
fn original_raw_caps_precede_wrong_role_and_whole_syntax_validation() {
    for case in CASES {
        for units in [vec![0xD800], vec![0xD83D, 0xDE00], vec![0x32, 0xD800, 0x20]] {
            let value = DocumentString::from_utf16(units.clone());
            for (element, name) in [
                (case.element.qualified_name(), case.name.qualified_name()),
                (
                    KnownHtmlElement::Div.qualified_name(),
                    case.name.qualified_name(),
                ),
                (
                    case.element.qualified_name(),
                    KnownHtmlAttribute::Id.qualified_name(),
                ),
                (
                    QualifiedName::new(Namespace::svg(), None, case.element.local_name()),
                    case.name.qualified_name(),
                ),
            ] {
                let attribute = Attribute::new(name, value.clone());
                let error = (case.read)(
                    &HtmlValueInterpreter::new(units.len() - 1),
                    &element,
                    Some(&attribute),
                )
                .unwrap_err();
                assert_eq!(error.capacity(), units.len() - 1);
                assert_eq!(error.required(), units.len());
                let report = (case.read)(
                    &HtmlValueInterpreter::new(units.len()),
                    &element,
                    Some(&attribute),
                )
                .unwrap();
                assert_eq!(report.attribute().raw_attribute(), Some(&attribute));
                assert_eq!(report.attribute().raw().unwrap().code_units(), units);
            }
        }
        let empty = raw(case, "");
        invalid_author(
            &(case.read)(
                &HtmlValueInterpreter::new(0),
                &case.element.qualified_name(),
                Some(&empty),
            )
            .unwrap(),
        );
        assert_eq!(
            (case.read)(
                &HtmlValueInterpreter::new(0),
                &case.element.qualified_name(),
                None
            )
            .unwrap()
            .author(),
            HtmlNumericAttributeAuthorState::Missing
        );
        let zero = raw(case, "0");
        let error = (case.read)(
            &HtmlValueInterpreter::new(0),
            &case.element.qualified_name(),
            Some(&zero),
        )
        .unwrap_err();
        assert_eq!(error.capacity(), 0);
        assert_eq!(error.required(), 1);
    }
}

#[test]
fn existing_native_integer_factories_author_both_roles_without_new_wrappers() {
    const HUGE: &str = "123456789012345678901234567890123456789012345678901234567890";
    for (digits, negative, expected) in [
        ("0000", true, "0"),
        ("00042", false, "42"),
        ("00042", true, "-42"),
        (HUGE, false, HUGE),
        (
            HUGE,
            true,
            "-123456789012345678901234567890123456789012345678901234567890",
        ),
    ] {
        let magnitude =
            HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(digits)).unwrap();
        let integer = HtmlInteger::from_magnitude(magnitude, negative);
        for case in CASES {
            let attribute =
                Attribute::new(case.name.qualified_name(), integer.as_document_string());
            assert_eq!(attribute.value(), &DocumentString::from(expected));
            assert_eq!(attribute.name(), &case.name.qualified_name());
            let report = read(case, Some(&attribute));
            assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
            recovered(&report, expected);
            assert_eq!(report.content_integer(), Some(&integer));
        }
    }
}

#[test]
fn owned_reports_preserve_old_content_after_raw_and_later_reports_are_dropped() {
    for case in CASES {
        let original = raw(case, " \t-0007tail");
        let report = read(case, Some(&original));
        let retained = report.clone();
        drop(original);
        let later = raw(case, "99");
        recovered(&read(case, Some(&later)), "99");
        drop(later);
        assert_eq!(report, retained);
        invalid_author(&report);
        recovered(&report, "-7");
        assert_eq!(
            report.attribute().raw(),
            Some(&DocumentString::from(" \t-0007tail"))
        );
        assert_eq!(
            report.common().unwrap().raw(),
            Some(&DocumentString::from(" \t-0007tail"))
        );
    }
}
