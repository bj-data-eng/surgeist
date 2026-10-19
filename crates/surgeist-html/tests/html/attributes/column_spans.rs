use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeObservation,
    HtmlNonNegativeInteger, HtmlNumericAttributeAuthorState, HtmlNumericAttributeConformanceError,
    HtmlNumericAttributeReport, HtmlNumericAttributeValueOrigin, HtmlValueAdmissionError,
    HtmlValueAuthorState, HtmlValueFailure, HtmlValueInterpreter, HtmlValueRecoveryState,
    HtmlValueSyntax, HtmlValueSyntaxError, KnownHtmlAttribute, KnownHtmlElement, Namespace,
    QualifiedName,
};

type SpanReport = HtmlNumericAttributeReport<HtmlNonNegativeInteger, u16>;
type SpanOperation = fn(
    &HtmlValueInterpreter,
    &QualifiedName,
    Option<&Attribute>,
) -> Result<SpanReport, HtmlValueAdmissionError>;

const SPANS: [(SpanOperation, KnownHtmlElement, &str); 2] = [
    (
        HtmlValueInterpreter::col_span,
        KnownHtmlElement::Col,
        "attr-col-span",
    ),
    (
        HtmlValueInterpreter::colgroup_span,
        KnownHtmlElement::Colgroup,
        "attr-colgroup-span",
    ),
];

fn attr(value: &str) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Span.qualified_name(), value)
}

fn quantity(report: &SpanReport, value: u16, origin: HtmlNumericAttributeValueOrigin) {
    assert_eq!(report.intrinsic(), Some(&value));
    assert_eq!(report.origin(), Some(origin));
}

fn invalid_author(report: &SpanReport) {
    assert_eq!(
        report.author(),
        HtmlNumericAttributeAuthorState::Nonconforming(
            HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(
                HtmlValueSyntaxError::InvalidNumericSyntax
            ))
        )
    );
    assert_eq!(
        report.common().unwrap().author(),
        &HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::InvalidNumericSyntax)
    );
}

fn recovered(report: &SpanReport, digits: &str) {
    assert!(
        matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.decimal_digits() == digits)
    );
}

#[test]
fn missing_numeric_content_defaults_to_one_with_missing_common_evidence() {
    for (operation, element, _) in SPANS {
        let report = operation(
            &HtmlValueInterpreter::new(0),
            &element.qualified_name(),
            None,
        )
        .unwrap();
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
        quantity(&report, 1, HtmlNumericAttributeValueOrigin::MissingDefault);
    }
}

#[test]
fn failed_recovery_defaults_to_one_and_preserves_empty_negative_and_unrecognized_input() {
    for (operation, element, _) in SPANS {
        for input in [
            "",
            "bad",
            "-1",
            "-999999999999999999999999999999999999999",
            "+",
            "\u{a0}2",
        ] {
            let raw = attr(input);
            let report = operation(
                &HtmlValueInterpreter::new(256),
                &element.qualified_name(),
                Some(&raw),
            )
            .unwrap();
            invalid_author(&report);
            let failure = if input.starts_with('-') {
                HtmlValueSyntaxError::NegativeNonnegativeInteger
            } else {
                HtmlValueSyntaxError::InvalidNumericSyntax
            };
            assert_eq!(
                report.common().unwrap().recovery(),
                &HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(failure))
            );
            assert_eq!(
                report.observation(),
                if input.is_empty() {
                    HtmlAttributeObservation::PresentEmpty
                } else {
                    HtmlAttributeObservation::PresentNonempty
                }
            );
            assert_eq!(report.raw_attribute(), Some(&raw));
            quantity(
                &report,
                1,
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
            );
        }
    }
}

#[test]
fn strict_zero_is_a_positive_value_violation_with_a_distinct_zero_default() {
    for (operation, element, _) in SPANS {
        for input in ["0", "0000"] {
            let report = operation(
                &HtmlValueInterpreter::new(8),
                &element.qualified_name(),
                Some(&attr(input)),
            )
            .unwrap();
            assert_eq!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::ZeroMustBePositive
                )
            );
            assert!(
                matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.is_zero())
            );
            recovered(&report, "0");
            quantity(&report, 1, HtmlNumericAttributeValueOrigin::ZeroDefault);
        }
    }
}

#[test]
fn negative_zero_recovers_successfully_without_becoming_valid_author_syntax() {
    for (operation, element, _) in SPANS {
        for input in ["-0", "-0000tail", " \t-00 "] {
            let report = operation(
                &HtmlValueInterpreter::new(16),
                &element.qualified_name(),
                Some(&attr(input)),
            )
            .unwrap();
            invalid_author(&report);
            recovered(&report, "0");
            quantity(&report, 1, HtmlNumericAttributeValueOrigin::ZeroDefault);
        }
    }
}

#[test]
fn literal_bounds_keep_common_validity_separate_from_attribute_range_and_clamping() {
    // HTML877d #attr-col-span/#attr-colgroup-span author 1..1000;
    // #forming-a-table defaults/clamps, not the WebIDL reflection algorithm.
    for (operation, element, _) in SPANS {
        for (input, digits, effective, above_maximum) in [
            ("1", "1", 1, false),
            ("0002", "2", 2, false),
            ("1000", "1000", 1000, false),
            ("1001", "1001", 1000, true),
            ("65535", "65535", 1000, true),
        ] {
            let raw = attr(input);
            let report = operation(
                &HtmlValueInterpreter::new(32),
                &element.qualified_name(),
                Some(&raw),
            )
            .unwrap();
            assert!(
                matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.decimal_digits() == digits)
            );
            assert_eq!(
                report.author(),
                if above_maximum {
                    HtmlNumericAttributeAuthorState::Nonconforming(
                        HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 },
                    )
                } else {
                    HtmlNumericAttributeAuthorState::Conforming
                }
            );
            recovered(&report, digits);
            quantity(
                &report,
                effective,
                if above_maximum {
                    HtmlNumericAttributeValueOrigin::Clamped
                } else {
                    HtmlNumericAttributeValueOrigin::Recovered
                },
            );
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
    }
}

#[test]
fn huge_exact_integers_clamp_only_processing_and_do_not_become_machine_range_syntax_errors() {
    for (operation, element, _) in SPANS {
        for input in [
            "340282366920938463463374607431768211455",
            "340282366920938463463374607431768211456",
            "999999999999999999999999999999999999999999999999999999999999999999",
        ] {
            let report = operation(
                &HtmlValueInterpreter::new(128),
                &element.qualified_name(),
                Some(&attr(input)),
            )
            .unwrap();
            assert!(
                matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.decimal_digits() == input)
            );
            assert_eq!(
                report.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }
                )
            );
            recovered(&report, input);
            quantity(&report, 1000, HtmlNumericAttributeValueOrigin::Clamped);
            assert_eq!(report.raw(), Some(&DocumentString::from(input)));
        }
    }
}

#[test]
fn prefix_recovery_preserves_lexical_failures_including_junk_on_a_clamped_quantity() {
    for (operation, element, _) in SPANS {
        for (input, digits, effective, origin) in [
            (
                " +2tail",
                "2",
                2,
                HtmlNumericAttributeValueOrigin::Recovered,
            ),
            ("2 ", "2", 2, HtmlNumericAttributeValueOrigin::Recovered),
            ("1.5", "1", 1, HtmlNumericAttributeValueOrigin::Recovered),
            ("1e3", "1", 1, HtmlNumericAttributeValueOrigin::Recovered),
            ("+0", "0", 1, HtmlNumericAttributeValueOrigin::ZeroDefault),
            (
                "1001tail",
                "1001",
                1000,
                HtmlNumericAttributeValueOrigin::Clamped,
            ),
            (
                "9999999999999999999999999999999999999999999999999999tail",
                "9999999999999999999999999999999999999999999999999999",
                1000,
                HtmlNumericAttributeValueOrigin::Clamped,
            ),
        ] {
            let report = operation(
                &HtmlValueInterpreter::new(256),
                &element.qualified_name(),
                Some(&attr(input)),
            )
            .unwrap();
            invalid_author(&report);
            recovered(&report, digits);
            quantity(&report, effective, origin);
        }
    }
}

#[test]
fn exact_names_and_source_rows_are_distinct_for_col_and_colgroup_content() {
    for (operation, element, anchor) in SPANS {
        let element = QualifiedName::new(Namespace::html(), Some("h".into()), element.local_name());
        let raw = Attribute::new(
            QualifiedName::new(Namespace::none(), Some("a".into()), "span"),
            "0002",
        );
        let report = operation(&HtmlValueInterpreter::new(4), &element, Some(&raw)).unwrap();
        assert_eq!(report.element(), &element);
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(
            report.observation(),
            HtmlAttributeObservation::PresentNonempty
        );
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.attribute(), KnownHtmlAttribute::Span);
        assert_eq!(report.inventory_rows(), &[147]);
        assert!(report.source_clause().ends_with(&format!("/#{anchor}")));
        assert!(
            report
                .processing_source_clause()
                .ends_with("/#forming-a-table")
        );
        assert_eq!(report.input_code_units(), 4);
        assert_eq!(
            report.common().unwrap().syntax(),
            HtmlValueSyntax::NonnegativeInteger
        );
        quantity(&report, 2, HtmlNumericAttributeValueOrigin::Recovered);
    }
}

fn unapplied(report: &SpanReport) {
    assert_ne!(
        report.applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
    assert!(report.common().is_none());
    assert!(report.intrinsic().is_none());
    assert!(report.origin().is_none());
}

#[test]
fn each_operation_requires_its_exact_html_element_without_treating_raw_as_absence() {
    for (operation, element, _) in SPANS {
        let raw = attr("bad");
        for wrong in [
            KnownHtmlElement::Div.qualified_name(),
            if element == KnownHtmlElement::Col {
                KnownHtmlElement::Colgroup.qualified_name()
            } else {
                KnownHtmlElement::Col.qualified_name()
            },
            QualifiedName::new(Namespace::html(), None, element.local_name().to_uppercase()),
            QualifiedName::new(Namespace::svg(), None, element.local_name()),
            QualifiedName::new(Namespace::none(), None, element.local_name()),
        ] {
            let report = operation(&HtmlValueInterpreter::new(8), &wrong, Some(&raw)).unwrap();
            unapplied(&report);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                report.observation(),
                HtmlAttributeObservation::PresentNonempty
            );
            unapplied(&operation(&HtmlValueInterpreter::new(8), &wrong, None).unwrap());
        }
    }
}

#[test]
fn wrong_attribute_expanded_names_are_unapplied_before_numeric_grammar() {
    for (operation, element, _) in SPANS {
        for raw in [
            Attribute::new(KnownHtmlAttribute::Colspan.qualified_name(), "bad"),
            Attribute::new(QualifiedName::new(Namespace::none(), None, "SPAN"), "bad"),
            Attribute::new(QualifiedName::new(Namespace::svg(), None, "span"), "bad"),
        ] {
            let report = operation(
                &HtmlValueInterpreter::new(8),
                &element.qualified_name(),
                Some(&raw),
            )
            .unwrap();
            unapplied(&report);
            assert_eq!(report.raw_attribute(), Some(&raw));
        }
    }
}

#[test]
fn utf16_is_preserved_across_prefix_recovery_and_leading_surrogate_failure() {
    for (operation, element, _) in SPANS {
        for (units, recovered_digits, effective, origin) in [
            (
                vec![0xD800, 0x32],
                None,
                1,
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
            ),
            (
                vec![0xD83D, 0xDE00, 0x32],
                None,
                1,
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
            ),
            (
                vec![0x32, 0xD800],
                Some("2"),
                2,
                HtmlNumericAttributeValueOrigin::Recovered,
            ),
            (
                vec![0x32, 0xD83D, 0xDE00],
                Some("2"),
                2,
                HtmlNumericAttributeValueOrigin::Recovered,
            ),
        ] {
            let raw = Attribute::new(
                KnownHtmlAttribute::Span.qualified_name(),
                DocumentString::from_utf16(units.clone()),
            );
            let report = operation(
                &HtmlValueInterpreter::new(4),
                &element.qualified_name(),
                Some(&raw),
            )
            .unwrap();
            invalid_author(&report);
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(report.input_code_units(), units.len());
            quantity(&report, effective, origin);
            if let Some(digits) = recovered_digits {
                recovered(&report, digits);
            } else {
                assert!(matches!(
                    report.common().unwrap().recovery(),
                    HtmlValueRecoveryState::Failed(_)
                ));
            }
        }
    }
}

#[test]
fn standalone_raw_cap_is_checked_before_even_wrong_element_or_attribute_roles() {
    for (operation, element, _) in SPANS {
        let interpreter = HtmlValueInterpreter::new(2);
        quantity(
            &operation(&interpreter, &element.qualified_name(), Some(&attr("12"))).unwrap(),
            12,
            HtmlNumericAttributeValueOrigin::Recovered,
        );
        let raw = attr("123");
        for subject in [
            element.qualified_name(),
            KnownHtmlElement::Div.qualified_name(),
        ] {
            let failure = operation(&interpreter, &subject, Some(&raw)).unwrap_err();
            assert_eq!((failure.required(), failure.capacity()), (3, 2));
        }
        let wrong = Attribute::new(KnownHtmlAttribute::Size.qualified_name(), "123");
        let failure = operation(&interpreter, &element.qualified_name(), Some(&wrong)).unwrap_err();
        assert_eq!((failure.required(), failure.capacity()), (3, 2));
        quantity(
            &operation(
                &HtmlValueInterpreter::new(0),
                &element.qualified_name(),
                None,
            )
            .unwrap(),
            1,
            HtmlNumericAttributeValueOrigin::MissingDefault,
        );
        quantity(
            &operation(
                &HtmlValueInterpreter::new(0),
                &element.qualified_name(),
                Some(&attr("")),
            )
            .unwrap(),
            1,
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        );
    }
}

#[test]
fn one_native_constructor_emits_both_roles_intrinsic_numeric_syntax_and_rejects_exact_out_of_range()
{
    for (input, expected) in [
        (
            "0",
            Err(HtmlNumericAttributeConformanceError::ZeroMustBePositive),
        ),
        ("0001", Ok("1")),
        ("0002", Ok("2")),
        ("1000", Ok("1000")),
        (
            "1001",
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
        ),
        (
            "340282366920938463463374607431768211456",
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
        ),
    ] {
        let value = HtmlNonNegativeInteger::from_decimal_digits(&input.into()).unwrap();
        let actual = value.into_column_span_attribute();
        match expected {
            Ok(digits) => {
                let raw = actual.unwrap();
                assert_eq!(raw.name(), &KnownHtmlAttribute::Span.qualified_name());
                assert_eq!(raw.value(), &DocumentString::from(digits));
                for (operation, element, _) in SPANS {
                    let report = operation(
                        &HtmlValueInterpreter::new(4),
                        &element.qualified_name(),
                        Some(&raw),
                    )
                    .unwrap();
                    assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
                    quantity(
                        &report,
                        digits.parse().unwrap(),
                        HtmlNumericAttributeValueOrigin::Recovered,
                    );
                }
            }
            Err(error) => assert_eq!(actual, Err(error)),
        }
    }
}

#[test]
fn shared_bounded_span_extension_preserves_cells_and_unbounded_neighbor_content() {
    let interpreter = HtmlValueInterpreter::new(128);
    let row = Attribute::new(KnownHtmlAttribute::Rowspan.qualified_name(), "0");
    let col = Attribute::new(KnownHtmlAttribute::Colspan.qualified_name(), "0");
    let row = interpreter
        .cell_rowspan(&KnownHtmlElement::Td.qualified_name(), Some(&row))
        .unwrap();
    assert_eq!(row.author(), HtmlNumericAttributeAuthorState::Conforming);
    quantity(&row, 0, HtmlNumericAttributeValueOrigin::Recovered);
    quantity(
        &interpreter
            .cell_colspan(&KnownHtmlElement::Th.qualified_name(), Some(&col))
            .unwrap(),
        1,
        HtmlNumericAttributeValueOrigin::ZeroDefault,
    );
    let huge = "340282366920938463463374607431768211456";
    let raw = Attribute::new(KnownHtmlAttribute::Cols.qualified_name(), huge);
    let textarea = interpreter
        .textarea_cols(&KnownHtmlElement::Textarea.qualified_name(), Some(&raw))
        .unwrap();
    assert_eq!(
        textarea.author(),
        HtmlNumericAttributeAuthorState::Conforming
    );
    assert_eq!(textarea.intrinsic().unwrap().decimal_digits(), huge);
    assert_eq!(
        textarea.origin(),
        Some(HtmlNumericAttributeValueOrigin::Recovered)
    );
    let native = HtmlNonNegativeInteger::from_decimal_digits(&huge.into()).unwrap();
    assert_eq!(
        native
            .clone()
            .into_textarea_cols_attribute()
            .unwrap()
            .value(),
        &DocumentString::from(huge)
    );
    assert_eq!(
        native.into_select_size_attribute().unwrap().value(),
        &DocumentString::from(huge)
    );
    let report = interpreter
        .colgroup_span(
            &KnownHtmlElement::Colgroup.qualified_name(),
            Some(&attr("2")),
        )
        .unwrap();
    quantity(&report, 2, HtmlNumericAttributeValueOrigin::Recovered);
}
