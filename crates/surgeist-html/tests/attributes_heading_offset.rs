use surgeist_html::*;

type OffsetReport = HtmlNumericAttributeReport<HtmlNonNegativeInteger, HtmlNonNegativeInteger>;
fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Headingoffset.qualified_name(), value)
}
fn read(value: Option<&str>) -> OffsetReport {
    let attribute = value.map(raw);
    HtmlValueInterpreter::new(4096)
        .heading_offset(&KnownHtmlElement::Div.qualified_name(), attribute.as_ref())
        .unwrap()
}
fn invalid_author(report: &OffsetReport) {
    assert_eq!(
        report.author(),
        HtmlNumericAttributeAuthorState::Nonconforming(
            HtmlNumericAttributeConformanceError::CommonValue(HtmlValueFailure::Syntax(
                HtmlValueSyntaxError::InvalidNumericSyntax,
            )),
        )
    );
    assert_eq!(
        report.common().unwrap().author(),
        &HtmlValueAuthorState::Invalid(HtmlValueSyntaxError::InvalidNumericSyntax)
    );
}
fn quantity(report: &OffsetReport, digits: &str, origin: HtmlNumericAttributeValueOrigin) {
    assert_eq!(report.intrinsic().unwrap().decimal_digits(), digits);
    assert_eq!(report.origin(), Some(origin));
}

#[test]
fn every_authored_offset_zero_through_eight_and_leading_zeros_conforms() {
    for (spelling, canonical) in [
        ("0", "0"),
        ("1", "1"),
        ("2", "2"),
        ("3", "3"),
        ("4", "4"),
        ("5", "5"),
        ("6", "6"),
        ("7", "7"),
        ("8", "8"),
        ("000", "0"),
        ("0008", "8"),
    ] {
        let report = read(Some(spelling));
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert!(
            matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.decimal_digits()==canonical)
        );
        assert!(
            matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.decimal_digits()==canonical)
        );
        quantity(
            &report,
            canonical,
            HtmlNumericAttributeValueOrigin::Recovered,
        );
        assert_eq!(report.raw(), Some(&DocumentString::from(spelling)));
    }
}

#[test]
fn author_maximum_does_not_clamp_exact_recovery_or_call_huge_values_bad_syntax() {
    for spelling in [
        "9".to_owned(),
        "123".to_owned(),
        "340282366920938463463374607431768211456".to_owned(),
        "9".repeat(2048),
    ] {
        let report = read(Some(&spelling));
        assert_eq!(
            report.author(),
            HtmlNumericAttributeAuthorState::Nonconforming(
                HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 8 }
            )
        );
        assert!(
            matches!(report.common().unwrap().author(), HtmlValueAuthorState::Valid(value) if value.decimal_digits()==spelling)
        );
        assert!(
            matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.decimal_digits()==spelling)
        );
        quantity(
            &report,
            &spelling,
            HtmlNumericAttributeValueOrigin::Recovered,
        );
        let native =
            HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(spelling)).unwrap();
        assert_eq!(
            native.into_heading_offset_attribute(),
            Err(HtmlNumericAttributeConformanceError::MaximumExceeded { maximum: 8 })
        );
    }
}

#[test]
fn invalid_whole_spelling_keeps_prefix_recovery_and_lexical_error_before_bounds() {
    for (spelling, recovered) in [
        ("+8", "8"),
        ("  +9tail", "9"),
        ("8.0", "8"),
        ("8garbage", "8"),
        ("-0", "0"),
        ("-000suffix", "0"),
        ("\t8\n", "8"),
        ("9bad", "9"),
    ] {
        let report = read(Some(spelling));
        invalid_author(&report);
        assert!(
            matches!(report.common().unwrap().recovery(), HtmlValueRecoveryState::Value(value) if value.decimal_digits()==recovered)
        );
        quantity(
            &report,
            recovered,
            HtmlNumericAttributeValueOrigin::Recovered,
        );
        assert_eq!(report.raw(), Some(&DocumentString::from(spelling)));
    }
}

#[test]
fn missing_and_failed_zero_contributions_remain_distinct_from_recovered_zero() {
    let missing = read(None);
    assert_eq!(missing.author(), HtmlNumericAttributeAuthorState::Missing);
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(
        missing.common().unwrap().author(),
        &HtmlValueAuthorState::Missing
    );
    assert_eq!(
        missing.common().unwrap().recovery(),
        &HtmlValueRecoveryState::Missing
    );
    quantity(
        &missing,
        "0",
        HtmlNumericAttributeValueOrigin::MissingDefault,
    );
    for spelling in [
        "",
        " \t\r\n\u{c}",
        "bad",
        "-1",
        "-9999999999999999999999999999999999999999",
        "+",
        "\u{a0}8",
    ] {
        let report = read(Some(spelling));
        invalid_author(&report);
        assert_eq!(
            report.common().unwrap().recovery(),
            &HtmlValueRecoveryState::Failed(HtmlValueFailure::Syntax(
                if spelling.starts_with('-') {
                    HtmlValueSyntaxError::NegativeNonnegativeInteger
                } else {
                    HtmlValueSyntaxError::InvalidNumericSyntax
                }
            ))
        );
        quantity(
            &report,
            "0",
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        );
        assert_eq!(
            report.observation(),
            if spelling.is_empty() {
                HtmlAttributeObservation::PresentEmpty
            } else {
                HtmlAttributeObservation::PresentNonempty
            }
        );
    }
    quantity(
        &read(Some("0")),
        "0",
        HtmlNumericAttributeValueOrigin::Recovered,
    );
}

#[test]
fn native_construction_accepts_exact_range_and_emits_canonical_standard_attributes() {
    for value in 0_u8..=8 {
        let native = HtmlNonNegativeInteger::from(u128::from(value));
        let attribute = native.into_heading_offset_attribute().unwrap();
        assert_eq!(
            attribute.name(),
            &KnownHtmlAttribute::Headingoffset.qualified_name()
        );
        assert_eq!(attribute.name().namespace(), &Namespace::none());
        assert_eq!(attribute.value(), &DocumentString::from(value.to_string()));
        let report = HtmlValueInterpreter::new(1)
            .heading_offset(
                &KnownHtmlElement::Section.qualified_name(),
                Some(&attribute),
            )
            .unwrap();
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
        quantity(
            &report,
            &value.to_string(),
            HtmlNumericAttributeValueOrigin::Recovered,
        );
    }
    let native =
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from("0008")).unwrap();
    assert_eq!(
        native.into_heading_offset_attribute().unwrap().value(),
        &DocumentString::from("8")
    );
}

#[test]
fn global_html_applicability_is_not_restricted_to_heading_names() {
    let attribute = raw("8");
    for local in ["div", "section", "h1", "h6", "custom-control", "H1", "DIV"] {
        let element = QualifiedName::new(Namespace::html(), Some("h".into()), local);
        let report = HtmlValueInterpreter::new(1)
            .heading_offset(&element, Some(&attribute))
            .unwrap();
        assert_eq!(report.element(), &element);
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Conforming);
        quantity(&report, "8", HtmlNumericAttributeValueOrigin::Recovered);
    }
}

#[test]
fn wrong_expanded_roles_admit_and_retain_raw_without_running_numeric_grammar() {
    let attribute = raw("bad");
    for (element, attribute, expected) in [
        (
            QualifiedName::new(Namespace::svg(), None, "h1"),
            attribute.clone(),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::none(), None, "div"),
            attribute.clone(),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            KnownHtmlElement::Div.qualified_name(),
            Attribute::new(KnownHtmlAttribute::Id.qualified_name(), "bad"),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            KnownHtmlElement::Div.qualified_name(),
            Attribute::new(
                QualifiedName::new(Namespace::none(), None, "HEADINGOFFSET"),
                "bad",
            ),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
        (
            KnownHtmlElement::Div.qualified_name(),
            Attribute::new(
                QualifiedName::new(Namespace::svg(), None, "headingoffset"),
                "bad",
            ),
            HtmlAttributeApplicability::DifferentAttribute,
        ),
    ] {
        let report = HtmlValueInterpreter::new(3)
            .heading_offset(&element, Some(&attribute))
            .unwrap();
        assert_eq!(report.element(), &element);
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(report.applicability(), expected);
        assert_eq!(report.author(), HtmlNumericAttributeAuthorState::Unapplied);
        assert_eq!(report.common(), None);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.origin(), None);
    }
}

#[test]
fn exact_utf16_is_retained_while_surrogate_suffix_and_prefix_recover_differently() {
    for (units, recovered, origin) in [
        (
            vec![56, 0xD800],
            "8",
            HtmlNumericAttributeValueOrigin::Recovered,
        ),
        (
            vec![56, 0xDC00],
            "8",
            HtmlNumericAttributeValueOrigin::Recovered,
        ),
        (
            vec![56, 0xD83D, 0xDE00],
            "8",
            HtmlNumericAttributeValueOrigin::Recovered,
        ),
        (
            vec![0xD800, 56],
            "0",
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        ),
        (
            vec![0xD83D, 0xDE00, 56],
            "0",
            HtmlNumericAttributeValueOrigin::ParseFailureDefault,
        ),
    ] {
        let attribute = raw(DocumentString::from_utf16(units.clone()));
        let report = HtmlValueInterpreter::new(units.len())
            .heading_offset(&KnownHtmlElement::Div.qualified_name(), Some(&attribute))
            .unwrap();
        assert_eq!(report.raw().unwrap().code_units(), units);
        invalid_author(&report);
        quantity(&report, recovered, origin);
        let error = HtmlValueInterpreter::new(units.len() - 1)
            .heading_offset(&KnownHtmlElement::Div.qualified_name(), Some(&attribute))
            .unwrap_err();
        assert_eq!(error.required(), units.len());
        assert_eq!(error.capacity(), units.len() - 1);
    }
}

#[test]
fn author_and_processing_sources_are_specific_and_raw_cap_precedes_applicability() {
    let element = QualifiedName::new(Namespace::html(), Some("h".into()), "div");
    let attribute = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("a".into()), "headingoffset"),
        "8",
    );
    let report = HtmlValueInterpreter::new(1)
        .heading_offset(&element, Some(&attribute))
        .unwrap();
    assert_eq!(report.element(), &element);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    assert_eq!(report.attribute(), KnownHtmlAttribute::Headingoffset);
    assert_eq!(report.inventory_rows(), &[62]);
    assert_eq!(
        report.source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#attr-headingoffset"
    );
    assert_eq!(
        report.processing_source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#get-an-element's-computed-heading-offset"
    );
    assert_eq!(
        report.common().unwrap().syntax(),
        HtmlValueSyntax::NonnegativeInteger
    );
    assert_eq!(report.input_code_units(), 1);
    let wide = raw(format!("8{}", "x".repeat(2048)));
    for element in [element, QualifiedName::new(Namespace::svg(), None, "h1")] {
        let error = HtmlValueInterpreter::new(wide.value().len() - 1)
            .heading_offset(&element, Some(&wide))
            .unwrap_err();
        assert_eq!(error.capacity(), wide.value().len() - 1);
        assert_eq!(error.required(), wide.value().len());
    }
    let exact = HtmlValueInterpreter::new(wide.value().len())
        .heading_offset(&KnownHtmlElement::Div.qualified_name(), Some(&wide))
        .unwrap();
    quantity(&exact, "8", HtmlNumericAttributeValueOrigin::Recovered);
    invalid_author(&exact);
}
