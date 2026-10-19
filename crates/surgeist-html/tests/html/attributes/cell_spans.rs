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

const SPANS: [(SpanOperation, KnownHtmlAttribute, u16, &str); 2] = [
    (
        HtmlValueInterpreter::cell_colspan,
        KnownHtmlAttribute::Colspan,
        27,
        "attr-tdth-colspan",
    ),
    (
        HtmlValueInterpreter::cell_rowspan,
        KnownHtmlAttribute::Rowspan,
        132,
        "attr-tdth-rowspan",
    ),
];

fn attr(kind: KnownHtmlAttribute, value: &str) -> Attribute {
    Attribute::new(kind.qualified_name(), value)
}

fn read(kind: KnownHtmlAttribute, value: Option<&str>, element: KnownHtmlElement) -> SpanReport {
    let raw = value.map(|value| attr(kind, value));
    let interpreter = HtmlValueInterpreter::new(4096);
    let element = element.qualified_name();
    match kind {
        KnownHtmlAttribute::Colspan => interpreter.cell_colspan(&element, raw.as_ref()),
        KnownHtmlAttribute::Rowspan => interpreter.cell_rowspan(&element, raw.as_ref()),
        _ => unreachable!("fixture selects only these two source rules"),
    }
    .unwrap()
}

fn quantity(report: &SpanReport, value: u16, origin: HtmlNumericAttributeValueOrigin) {
    assert_eq!(report.intrinsic(), Some(&value));
    assert_eq!(report.origin(), Some(origin));
}

fn recovered(report: &SpanReport, digits: &str) {
    assert!(
        matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value)
        if value.decimal_digits() == digits)
    );
}

fn invalid_author(report: &SpanReport) {
    assert_eq!(
        report.author(),
        HtmlNumericAttributeAuthorState::Nonconforming(
            HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(
                HtmlValueSyntaxError::InvalidNumericSyntax,
            )),
        ),
    );
    assert_eq!(
        report.common().unwrap().author(),
        &HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::InvalidNumericSyntax),
    );
}

#[test]
fn absent_content_defaults_to_one_without_claiming_authored_or_recovered_values() {
    for element in [KnownHtmlElement::Td, KnownHtmlElement::Th] {
        for (_, kind, _, _) in SPANS {
            let report = read(kind, None, element);
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
}

#[test]
fn failed_recovery_defaults_to_one_and_retains_the_rejection_kind() {
    for element in [KnownHtmlElement::Td, KnownHtmlElement::Th] {
        for (_, kind, _, _) in SPANS {
            for input in [
                "",
                "garbage",
                "-1",
                "-9999999999999999999999999999999999999999",
                "+",
                "\u{a0}2",
            ] {
                let report = read(kind, Some(input), element);
                invalid_author(&report);
                let error = if input.starts_with('-') {
                    HtmlValueSyntaxError::NegativeNonnegativeInteger
                } else {
                    HtmlValueSyntaxError::InvalidNumericSyntax
                };
                assert_eq!(
                    report.common().unwrap().recovery(),
                    &HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(error))
                );
                quantity(
                    &report,
                    1,
                    HtmlNumericAttributeValueOrigin::ParseFailureDefault,
                );
                assert_eq!(report.raw(), Some(&DocumentString::from(input)));
            }
        }
    }
}

#[test]
fn authored_zero_defaults_colspan_but_rowspan_retains_remaining_rows() {
    for element in [KnownHtmlElement::Td, KnownHtmlElement::Th] {
        for input in ["0", "0000"] {
            let col = read(KnownHtmlAttribute::Colspan, Some(input), element);
            let row = read(KnownHtmlAttribute::Rowspan, Some(input), element);
            assert_eq!(
                col.author(),
                HtmlNumericAttributeAuthorState::Nonconforming(
                    HtmlNumericAttributeConformanceError::ZeroMustBePositive
                )
            );
            assert_eq!(row.author(), HtmlNumericAttributeAuthorState::Conforming);
            for report in [&col, &row] {
                assert!(
                    matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.is_zero())
                );
                recovered(report, "0");
            }
            quantity(&col, 1, HtmlNumericAttributeValueOrigin::ZeroDefault);
            quantity(&row, 0, HtmlNumericAttributeValueOrigin::Recovered);
        }
    }
}

#[test]
fn recovered_negative_zero_keeps_invalid_author_spelling_and_distinct_zero_processing() {
    for element in [KnownHtmlElement::Td, KnownHtmlElement::Th] {
        for input in ["-0", "-0000tail", " \t-00 "] {
            let col = read(KnownHtmlAttribute::Colspan, Some(input), element);
            let row = read(KnownHtmlAttribute::Rowspan, Some(input), element);
            for report in [&col, &row] {
                invalid_author(report);
                recovered(report, "0");
            }
            quantity(&col, 1, HtmlNumericAttributeValueOrigin::ZeroDefault);
            quantity(&row, 0, HtmlNumericAttributeValueOrigin::Recovered);
        }
    }
}

#[test]
fn strict_integer_bounds_are_separate_from_processing_caps() {
    // HTML #attr-tdth-colspan/#attr-tdth-rowspan and #forming-a-table.
    // The whole digits grammar succeeds even when an attribute maximum fails.
    for element in [KnownHtmlElement::Td, KnownHtmlElement::Th] {
        for (input, col, row, col_maximum, row_maximum) in [
            ("1", 1, 1, false, false),
            ("0002", 2, 2, false, false),
            ("1000", 1000, 1000, false, false),
            ("1001", 1000, 1001, true, false),
            ("65534", 1000, 65534, true, false),
            ("65535", 1000, 65534, true, true),
        ] {
            for (kind, effective, maximum_failed, maximum) in [
                (KnownHtmlAttribute::Colspan, col, col_maximum, 1000),
                (KnownHtmlAttribute::Rowspan, row, row_maximum, 65534),
            ] {
                let report = read(kind, Some(input), element);
                assert!(matches!(
                    report.common().unwrap().author(),
                    HtmlValueAuthorState::Valid(_)
                ));
                let author = if maximum_failed {
                    HtmlNumericAttributeAuthorState::Nonconforming(
                        HtmlNumericAttributeConformanceError::MaximumExceeded { maximum },
                    )
                } else {
                    HtmlNumericAttributeAuthorState::Conforming
                };
                assert_eq!(report.author(), author);
                quantity(
                    &report,
                    effective,
                    if maximum_failed {
                        HtmlNumericAttributeValueOrigin::Clamped
                    } else {
                        HtmlNumericAttributeValueOrigin::Recovered
                    },
                );
            }
        }
    }
}

#[test]
fn arbitrary_magnitudes_remain_exact_common_values_and_clamp_only_the_processed_quantity() {
    for input in [
        "340282366920938463463374607431768211455",
        "340282366920938463463374607431768211456",
        "999999999999999999999999999999999999999999999999999999999999999999",
    ] {
        for element in [KnownHtmlElement::Td, KnownHtmlElement::Th] {
            for (kind, maximum) in [
                (KnownHtmlAttribute::Colspan, 1000),
                (KnownHtmlAttribute::Rowspan, 65534),
            ] {
                let report = read(kind, Some(input), element);
                assert!(
                    matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value)
                    if value.decimal_digits() == input)
                );
                assert_eq!(
                    report.author(),
                    HtmlNumericAttributeAuthorState::Nonconforming(
                        HtmlNumericAttributeConformanceError::MaximumExceeded { maximum }
                    )
                );
                recovered(&report, input);
                quantity(&report, maximum, HtmlNumericAttributeValueOrigin::Clamped);
                assert_eq!(report.raw(), Some(&DocumentString::from(input)));
            }
        }
    }
}

#[test]
fn recovery_prefixes_do_not_become_conforming_integers_or_float_values() {
    for element in [KnownHtmlElement::Td, KnownHtmlElement::Th] {
        for (_, kind, _, _) in SPANS {
            for (input, expected) in [(" +2tail", 2), ("2 ", 2), ("1.5", 1), ("1e3", 1), ("+0", 0)]
            {
                let report = read(kind, Some(input), element);
                invalid_author(&report);
                recovered(&report, &expected.to_string());
                quantity(
                    &report,
                    if kind == KnownHtmlAttribute::Colspan && expected == 0 {
                        1
                    } else {
                        expected
                    },
                    if kind == KnownHtmlAttribute::Colspan && expected == 0 {
                        HtmlNumericAttributeValueOrigin::ZeroDefault
                    } else {
                        HtmlNumericAttributeValueOrigin::Recovered
                    },
                );
            }
        }
    }
}

#[test]
fn a_huge_recovered_prefix_clamps_but_does_not_replace_lexical_nonconformance_with_range_error() {
    let digits = "9999999999999999999999999999999999999999999999999999999999999999";
    let input = format!("{digits}tail");
    for (kind, cap) in [
        (KnownHtmlAttribute::Colspan, 1000),
        (KnownHtmlAttribute::Rowspan, 65534),
    ] {
        let report = read(kind, Some(&input), KnownHtmlElement::Td);
        invalid_author(&report);
        recovered(&report, digits);
        quantity(&report, cap, HtmlNumericAttributeValueOrigin::Clamped);
    }
}

#[test]
fn exact_expanded_names_preserve_prefixes_and_source_qualified_raw_records() {
    let interpreter = HtmlValueInterpreter::new(8);
    for element in ["td", "th"] {
        let element = QualifiedName::new(Namespace::html(), Some("h".into()), element);
        for (operation, kind, row, anchor) in SPANS {
            let raw = Attribute::new(
                QualifiedName::new(Namespace::none(), Some("a".into()), kind.local_name()),
                "0002",
            );
            let report = operation(&interpreter, &element, Some(&raw)).unwrap();
            assert_eq!(report.element(), &element);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(report.raw(), Some(raw.value()));
            assert_eq!(
                report.observation(),
                HtmlAttributeObservation::PresentNonempty
            );
            assert_eq!(
                report.applicability(),
                HtmlAttributeApplicability::Applicable
            );
            assert_eq!(report.attribute(), kind);
            assert_eq!(report.inventory_rows(), &[row]);
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
fn foreign_or_different_element_roles_retain_raw_without_running_numeric_grammar() {
    let interpreter = HtmlValueInterpreter::new(8);
    for (operation, kind, _, _) in SPANS {
        let raw = attr(kind, "bad");
        for element in [
            KnownHtmlElement::Div.qualified_name(),
            QualifiedName::new(Namespace::html(), None, "TD"),
            QualifiedName::new(Namespace::svg(), None, "td"),
            QualifiedName::new(Namespace::none(), None, "th"),
        ] {
            let report = operation(&interpreter, &element, Some(&raw)).unwrap();
            unapplied(&report);
            assert_eq!(report.raw_attribute(), Some(&raw));
            let absent = operation(&interpreter, &element, None).unwrap();
            unapplied(&absent);
        }
    }
}

#[test]
fn wrong_attribute_roles_are_not_absence_or_failed_conversion_defaults() {
    let interpreter = HtmlValueInterpreter::new(8);
    for (operation, kind, _, _) in SPANS {
        for raw in [
            attr(KnownHtmlAttribute::Size, "2"),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, kind.local_name().to_uppercase()),
                "2",
            ),
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, kind.local_name()),
                "2",
            ),
        ] {
            let report = operation(
                &interpreter,
                &KnownHtmlElement::Td.qualified_name(),
                Some(&raw),
            )
            .unwrap();
            unapplied(&report);
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                report.observation(),
                HtmlAttributeObservation::PresentNonempty
            );
        }
    }
}

#[test]
fn isolated_and_supplementary_utf16_survive_failure_or_digit_prefix_recovery() {
    for (operation, kind, _, _) in SPANS {
        for (units, effective, origin, recovery) in [
            (
                vec![0xD800, 0x32],
                1,
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
                None,
            ),
            (
                vec![0xD83D, 0xDE00, 0x32],
                1,
                HtmlNumericAttributeValueOrigin::ParseFailureDefault,
                None,
            ),
            (
                vec![0x32, 0xD800],
                2,
                HtmlNumericAttributeValueOrigin::Recovered,
                Some("2"),
            ),
            (
                vec![0x32, 0xD83D, 0xDE00],
                2,
                HtmlNumericAttributeValueOrigin::Recovered,
                Some("2"),
            ),
        ] {
            let raw = Attribute::new(
                kind.qualified_name(),
                DocumentString::from_utf16(units.clone()),
            );
            let report = operation(
                &HtmlValueInterpreter::new(8),
                &KnownHtmlElement::Th.qualified_name(),
                Some(&raw),
            )
            .unwrap();
            invalid_author(&report);
            assert_eq!(report.raw().unwrap().code_units(), units);
            assert_eq!(report.input_code_units(), units.len());
            quantity(&report, effective, origin);
            if let Some(digits) = recovery {
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
fn per_value_admission_happens_before_wrong_element_and_attribute_roles() {
    for (operation, kind, _, _) in SPANS {
        let interpreter = HtmlValueInterpreter::new(2);
        let element = KnownHtmlElement::Td.qualified_name();
        let exact = attr(kind, "12");
        quantity(
            &operation(&interpreter, &element, Some(&exact)).unwrap(),
            12,
            HtmlNumericAttributeValueOrigin::Recovered,
        );
        let oversized = attr(kind, "123");
        for element in [&element, &KnownHtmlElement::Div.qualified_name()] {
            let failure = operation(&interpreter, element, Some(&oversized)).unwrap_err();
            assert_eq!((failure.required(), failure.capacity()), (3, 2));
        }
        let wrong = attr(KnownHtmlAttribute::Size, "123");
        let failure = operation(&interpreter, &element, Some(&wrong)).unwrap_err();
        assert_eq!((failure.required(), failure.capacity()), (3, 2));
        let missing = operation(&HtmlValueInterpreter::new(0), &element, None).unwrap();
        quantity(&missing, 1, HtmlNumericAttributeValueOrigin::MissingDefault);
        let empty = attr(kind, "");
        quantity(
            &operation(&HtmlValueInterpreter::new(0), &element, Some(&empty)).unwrap(),
            1,
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        );
    }
}

#[test]
fn checked_native_constructors_reject_out_of_range_integers_and_emit_canonical_attributes() {
    for (digits, col_result, row_result) in [
        (
            "0",
            Err(HtmlNumericAttributeConformanceError::ZeroMustBePositive),
            Ok("0"),
        ),
        ("0001", Ok("1"), Ok("1")),
        ("0001000", Ok("1000"), Ok("1000")),
        (
            "1001",
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
            Ok("1001"),
        ),
        (
            "65534",
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
            Ok("65534"),
        ),
        (
            "65535",
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 65534 }),
        ),
        (
            "340282366920938463463374607431768211456",
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 1000 }),
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 65534 }),
        ),
    ] {
        let value = HtmlNonNegativeInteger::from_decimal_digits(&digits.into()).unwrap();
        for (kind, actual, expected) in [
            (
                KnownHtmlAttribute::Colspan,
                value.clone().into_cell_colspan_attribute(),
                col_result,
            ),
            (
                KnownHtmlAttribute::Rowspan,
                value.into_cell_rowspan_attribute(),
                row_result,
            ),
        ] {
            match expected {
                Ok(digits) => {
                    let actual = actual.unwrap();
                    assert_eq!(actual.name(), &kind.qualified_name());
                    assert_eq!(actual.value(), &DocumentString::from(digits));
                    let report = read(kind, Some(digits), KnownHtmlElement::Td);
                    assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
                    quantity(
                        &report,
                        digits.parse().unwrap(),
                        HtmlNumericAttributeValueOrigin::Recovered,
                    );
                }
                Err(error) => assert_eq!(actual, Err(error)),
            }
        }
    }
}

#[test]
fn cell_span_views_and_native_constructors_do_not_change_neighbor_numeric_rules() {
    let interpreter = HtmlValueInterpreter::new(128);
    let huge = "340282366920938463463374607431768211456";
    let raw = attr(KnownHtmlAttribute::Cols, huge);
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
    let canvas = interpreter
        .canvas_width(
            &KnownHtmlElement::Canvas.qualified_name(),
            Some(&attr(KnownHtmlAttribute::Width, "0")),
        )
        .unwrap();
    assert_eq!(canvas.author(), HtmlNumericAttributeAuthorState::Conforming);
    assert_eq!(canvas.intrinsic().unwrap().decimal_digits(), "0");
    assert_eq!(
        canvas.origin(),
        Some(HtmlNumericAttributeValueOrigin::Recovered)
    );
    let textarea = interpreter
        .textarea_rows(
            &KnownHtmlElement::Textarea.qualified_name(),
            Some(&attr(KnownHtmlAttribute::Rows, "0")),
        )
        .unwrap();
    assert_eq!(
        textarea.origin(),
        Some(HtmlNumericAttributeValueOrigin::ZeroDefault)
    );
    assert_eq!(textarea.intrinsic().unwrap().decimal_digits(), "2");
    let cell = read(
        KnownHtmlAttribute::Colspan,
        Some(huge),
        KnownHtmlElement::Td,
    );
    quantity(&cell, 1000, HtmlNumericAttributeValueOrigin::Clamped);
    assert_eq!(cell.raw(), Some(&DocumentString::from(huge)));
    recovered(&cell, huge);
}
