//! Public common-value behavior; expected results follow the pinned clauses.
use surgeist_html::{
    DocumentString, HtmlDimension, HtmlValueAuthorState as Author, HtmlValueFailure as Failure,
    HtmlValueInterpreter, HtmlValueRecoveryState as Recovery,
    HtmlValueRepresentationError as Representation, HtmlValueSyntax,
    HtmlValueSyntaxError as Syntax,
};

#[test]
fn legacy_color_missing_empty_and_transparent_have_distinct_outcomes() {
    let interpreter = HtmlValueInterpreter::new(16);
    let missing = interpreter.legacy_color(None).unwrap();
    assert_eq!(missing.raw(), None);
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.syntax(), HtmlValueSyntax::LegacyColor);
    assert_eq!(
        missing.syntax().source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#colours"
    );

    let raw = DocumentString::from("");
    let empty = interpreter.legacy_color(Some(&raw)).unwrap();
    assert_eq!(empty.author(), &Author::NotDefined);
    assert_eq!(
        empty.recovery(),
        &Recovery::Failed(Failure::Syntax(Syntax::EmptyLegacyColor))
    );
    assert_eq!(empty.raw(), Some(&raw));
    for text in ["transparent", " \tTrAnSpArEnT\r "] {
        let raw = DocumentString::from(text);
        let report = interpreter.legacy_color(Some(&raw)).unwrap();
        assert_eq!(report.author(), &Author::NotDefined);
        assert_eq!(
            report.recovery(),
            &Recovery::Failed(Failure::Syntax(Syntax::TransparentLegacyColor))
        );
        assert_eq!(report.raw(), Some(&raw));
    }
}

#[test]
fn legacy_color_processing_preserves_raw_whitespace_and_surrogates() {
    let interpreter = HtmlValueInterpreter::new(16);
    for (units, rgb) in [
        (vec![0x20, 0x09, 0x0d], [0, 0, 0]),
        (vec![0xd800, 0x66, 0x30], [0, 15, 0]),
        (vec![0xd83d, 0xde00, 0x61], [0, 0, 10]),
    ] {
        let raw = DocumentString::from_utf16(units.clone());
        let report = interpreter.legacy_color(Some(&raw)).unwrap();
        assert_eq!(report.author(), &Author::NotDefined);
        let Recovery::Value(color) = report.recovery() else {
            panic!("expected recovered legacy color")
        };
        assert_eq!(color.channels(), rgb);
        assert_eq!(report.raw().unwrap().code_units(), units);
        assert_eq!(report.input_code_units(), units.len());
    }
}

#[test]
fn legacy_color_admits_original_units_before_trimming_and_truncation() {
    let empty = DocumentString::from("");
    assert!(HtmlValueInterpreter::new(0).legacy_color(None).is_ok());
    assert!(
        HtmlValueInterpreter::new(0)
            .legacy_color(Some(&empty))
            .is_ok()
    );

    for raw in [
        DocumentString::from("   "),
        DocumentString::from("f".repeat(129)),
        DocumentString::from_utf16(vec![0xd83d, 0xde00]),
    ] {
        let report = HtmlValueInterpreter::new(raw.len())
            .legacy_color(Some(&raw))
            .unwrap();
        assert!(matches!(report.recovery(), Recovery::Value(_)));
        assert_eq!(report.raw(), Some(&raw));
        let error = HtmlValueInterpreter::new(raw.len() - 1)
            .legacy_color(Some(&raw))
            .unwrap_err();
        assert_eq!(error.capacity(), raw.len() - 1);
        assert_eq!(error.required(), raw.len());
    }
}

#[test]
fn legacy_color_named_and_hex_results_have_equivalent_canonical_authoring() {
    let interpreter = HtmlValueInterpreter::new(32);
    for (text, rgb, canonical) in [
        (" \tReBeCcApUrPlE ", [102, 51, 153], "#663399"),
        ("#AbC", [170, 187, 204], "#aabbcc"),
        ("#00ff7f", [0, 255, 127], "#00ff7f"),
    ] {
        let raw = DocumentString::from(text);
        let report = interpreter.legacy_color(Some(&raw)).unwrap();
        let Recovery::Value(color) = report.recovery() else {
            panic!("expected recovered legacy color")
        };
        assert_eq!(color.channels(), rgb);
        assert_eq!(color.as_document_string(), DocumentString::from(canonical));
        let canonical = color.as_document_string();
        let roundtrip = interpreter.legacy_color(Some(&canonical)).unwrap();
        assert_eq!(roundtrip.recovery(), report.recovery());
        assert_eq!(report.raw(), Some(&raw));
        assert_eq!(report.author(), &Author::NotDefined);
    }
}

#[test]
fn absent_empty_and_admission_are_separate_from_syntax() {
    let interpreter = HtmlValueInterpreter::new(0);
    let missing = interpreter.integer(None).unwrap();
    assert_eq!(missing.raw(), None);
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.input_code_units(), 0);

    let empty = DocumentString::from("");
    let report = interpreter.integer(Some(&empty)).unwrap();
    assert_eq!(report.raw(), Some(&empty));
    assert_eq!(
        report.author(),
        &Author::Invalid(Syntax::InvalidNumericSyntax)
    );
    assert_eq!(
        report.recovery(),
        &Recovery::Failed(Failure::Syntax(Syntax::InvalidNumericSyntax))
    );

    let raw = DocumentString::from("x");
    let error = interpreter.integer(Some(&raw)).unwrap_err();
    assert_eq!(error.capacity(), 0);
    assert_eq!(error.required(), 1);
    let admitted = HtmlValueInterpreter::new(1).integer(Some(&raw)).unwrap();
    assert_eq!(
        admitted.author(),
        &Author::Invalid(Syntax::InvalidNumericSyntax)
    );
    assert_eq!(admitted.raw(), Some(&raw));
}

#[test]
fn integer_author_grammar_and_recovery_have_distinct_results() {
    let interpreter = HtmlValueInterpreter::new(128);
    for (text, value) in [(" +00012tail", 12), ("\t-42 suffix", -42), ("12\n", 12)] {
        let raw = DocumentString::from(text);
        let report = interpreter.integer(Some(&raw)).unwrap();
        assert_eq!(
            report.author(),
            &Author::Invalid(Syntax::InvalidNumericSyntax)
        );
        let Recovery::Value(recovered) = report.recovery() else {
            panic!("expected recovered integer")
        };
        assert_eq!(recovered.to_i128().unwrap(), value);
        assert_eq!(report.raw(), Some(&raw));
    }
    let raw = DocumentString::from("-000");
    let report = interpreter.integer(Some(&raw)).unwrap();
    let Author::Valid(value) = report.author() else {
        panic!("expected valid integer")
    };
    assert_eq!(value.to_i128().unwrap(), 0);
    assert!(!value.is_negative());

    let raw = DocumentString::from("99999999999999999999999999999999999999999999999999");
    let report = interpreter.integer(Some(&raw)).unwrap();
    let Author::Valid(value) = report.author() else {
        panic!("expected exact integer")
    };
    assert_eq!(
        value.magnitude().decimal_digits(),
        "99999999999999999999999999999999999999999999999999"
    );
    assert!(value.to_i128().is_err());
}

#[test]
fn nonnegative_recovery_rejects_negative_but_recovers_negative_zero() {
    let interpreter = HtmlValueInterpreter::new(8);
    let negative_zero = DocumentString::from("-0");
    let report = interpreter
        .nonnegative_integer(Some(&negative_zero))
        .unwrap();
    assert!(matches!(report.author(), Author::Invalid(_)));
    let Recovery::Value(value) = report.recovery() else {
        panic!("expected recovered zero")
    };
    assert_eq!(value.to_u128().unwrap(), 0);

    let raw = DocumentString::from("-1");
    let report = interpreter.nonnegative_integer(Some(&raw)).unwrap();
    assert_eq!(
        report.recovery(),
        &Recovery::Failed(Failure::Syntax(Syntax::NegativeNonnegativeInteger))
    );
}

#[test]
fn floating_point_overflow_is_not_lexical_invalidity() {
    let interpreter = HtmlValueInterpreter::new(32);
    let raw = DocumentString::from("1e9999");
    let report = interpreter.floating_point_number(Some(&raw)).unwrap();
    assert_eq!(
        report.author(),
        &Author::Unrepresentable(Representation::FloatingPointOverflow)
    );
    assert_eq!(
        report.recovery(),
        &Recovery::Failed(Failure::Representation(
            Representation::FloatingPointOverflow
        ))
    );

    for (text, number) in [
        ("+1", 1.0),
        (" 1e+tail", 1.0),
        ("1.", 1.0),
        ("1.5tail", 1.5),
    ] {
        let raw = DocumentString::from(text);
        let report = interpreter.floating_point_number(Some(&raw)).unwrap();
        assert_eq!(
            report.author(),
            &Author::Invalid(Syntax::InvalidNumericSyntax)
        );
        let Recovery::Value(value) = report.recovery() else {
            panic!("expected recovered float")
        };
        assert_eq!(value.value(), number);
        assert_eq!(report.raw(), Some(&raw));
    }
}

#[test]
fn floating_lists_preserve_recovery_substitution_and_complete_author_check() {
    let interpreter = HtmlValueInterpreter::new(64);
    let raw = DocumentString::from("1.5; nope 1e9999,-2");
    let report = interpreter.floating_point_list(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Invalid(_)));
    let Recovery::Value(values) = report.recovery() else {
        panic!("expected list")
    };
    assert_eq!(
        values
            .values()
            .iter()
            .map(|v| v.value())
            .collect::<Vec<_>>(),
        vec![1.5, 0.0, 0.0, -2.0]
    );

    let raw = DocumentString::from("1e9999,nope");
    let report = interpreter.floating_point_list(Some(&raw)).unwrap();
    assert_eq!(
        report.author(),
        &Author::Invalid(Syntax::InvalidNumericSyntax)
    );
    let raw = DocumentString::from("1e9999,2");
    let report = interpreter.floating_point_list(Some(&raw)).unwrap();
    assert_eq!(
        report.author(),
        &Author::Unrepresentable(Representation::FloatingPointOverflow)
    );
}

#[test]
fn dimensions_report_processing_without_inventing_author_grammar() {
    let interpreter = HtmlValueInterpreter::new(64);
    let raw = DocumentString::from(" 0012.50%tail");
    let report = interpreter.dimension(Some(&raw)).unwrap();
    assert_eq!(report.author(), &Author::NotDefined);
    let Recovery::Value(HtmlDimension::Percentage(value)) = report.recovery() else {
        panic!("expected percentage")
    };
    assert_eq!(value.integer_part().decimal_digits(), "12");
    assert_eq!(value.fractional_digits(), Some("5"));
    assert_eq!(report.raw(), Some(&raw));

    let zero = DocumentString::from("0%");
    let report = interpreter.nonzero_dimension(Some(&zero)).unwrap();
    assert_eq!(report.author(), &Author::NotDefined);
    assert_eq!(
        report.recovery(),
        &Recovery::Failed(Failure::Syntax(Syntax::ZeroNonzeroDimension))
    );

    let raw = DocumentString::from("2.5*, 10%, ,3");
    let report = interpreter.dimension_list(Some(&raw)).unwrap();
    assert_eq!(report.author(), &Author::NotDefined);
    let Recovery::Value(list) = report.recovery() else {
        panic!("expected dimension list")
    };
    assert_eq!(
        list.values()
            .iter()
            .map(|v| v.as_document_string().to_utf8().unwrap())
            .collect::<Vec<_>>(),
        vec!["2.5*", "10%", "0*", "3"]
    );
}

#[test]
fn isolated_surrogates_remain_raw_and_recovery_observes_only_numeric_prefix() {
    let raw = DocumentString::from_utf16(vec![b'1' as u16, 0xd800, 0xdc00, 0xdfff]);
    let report = HtmlValueInterpreter::new(4).integer(Some(&raw)).unwrap();
    assert_eq!(report.input_code_units(), 4);
    assert_eq!(
        report.raw().unwrap().code_units(),
        &[0x31, 0xd800, 0xdc00, 0xdfff]
    );
    assert!(matches!(report.author(), Author::Invalid(_)));
    let Recovery::Value(value) = report.recovery() else {
        panic!("expected prefix recovery")
    };
    assert_eq!(value.to_i128().unwrap(), 1);
    let error = HtmlValueInterpreter::new(3)
        .integer(Some(&raw))
        .unwrap_err();
    assert_eq!(error.required(), 4);
}

#[test]
fn month_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2024-02");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.month(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.month(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("2024-00");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .month(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .month(Some(&raw))
            .is_err()
    );
}

#[test]
fn date_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2024-02-29");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.date(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.date(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("2023-02-29");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .date(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .date(Some(&raw))
            .is_err()
    );
}

#[test]
fn yearless_date_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("--02-29");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.yearless_date(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.yearless_date(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("02-30");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .yearless_date(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .yearless_date(Some(&raw))
            .is_err()
    );
}

#[test]
fn time_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("23:59:59.125");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.time(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.time(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("23:59:59.1234");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .time(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .time(Some(&raw))
            .is_err()
    );
}

#[test]
fn local_date_time_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2024-02-29 23:59");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.local_date_time(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.local_date_time(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("2024-02-29_23:59");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .local_date_time(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .local_date_time(Some(&raw))
            .is_err()
    );
}

#[test]
fn normalized_local_date_time_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2024-02-29T23:59");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.normalized_local_date_time(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.normalized_local_date_time(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("2024-02-29T23:59:00");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .normalized_local_date_time(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .normalized_local_date_time(Some(&raw))
            .is_err()
    );
}

#[test]
fn time_zone_offset_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("+23:59");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.time_zone_offset(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.time_zone_offset(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("-00:00");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .time_zone_offset(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .time_zone_offset(Some(&raw))
            .is_err()
    );
}

#[test]
fn global_date_time_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2024-02-29T23:59Z");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.global_date_time(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.global_date_time(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("2024-02-29T23:59");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .global_date_time(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .global_date_time(Some(&raw))
            .is_err()
    );
}

#[test]
fn week_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2020-W53");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.week(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.week(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("2021-W53");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .week(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .week(Some(&raw))
            .is_err()
    );
}

#[test]
fn duration_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("PT1.125S");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.duration(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.duration(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("PT1.1234S");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .duration(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .duration(Some(&raw))
            .is_err()
    );
}

#[test]
fn date_or_time_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2024-02-29");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.date_or_time(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.date_or_time(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("2024-02-30");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .date_or_time(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .date_or_time(Some(&raw))
            .is_err()
    );
}

#[test]
fn date_with_optional_time_checks_authored_strings_without_claiming_recovery() {
    let raw = DocumentString::from("2024-02-29T23:59+01:00");
    let interpreter = HtmlValueInterpreter::new(raw.len());
    let report = interpreter.date_with_optional_time(Some(&raw)).unwrap();
    assert!(matches!(report.author(), Author::Valid(_)));
    assert_eq!(report.recovery(), &Recovery::Unsupported);
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.input_code_units(), raw.len());
    let missing = interpreter.date_with_optional_time(None).unwrap();
    assert_eq!(missing.author(), &Author::Missing);
    assert_eq!(missing.recovery(), &Recovery::Missing);
    assert_eq!(missing.raw(), None);
    let invalid = DocumentString::from("23:59");
    let invalid_report = HtmlValueInterpreter::new(invalid.len())
        .date_with_optional_time(Some(&invalid))
        .unwrap();
    assert!(matches!(
        invalid_report.author(),
        Author::Invalid(Syntax::Temporal(_))
    ));
    assert_eq!(invalid_report.recovery(), &Recovery::Unsupported);
    assert_eq!(invalid_report.raw(), Some(&invalid));
    assert!(
        HtmlValueInterpreter::new(raw.len() - 1)
            .date_with_optional_time(Some(&raw))
            .is_err()
    );
}

#[test]
fn token_splitting_preserves_raw_unknown_tokens_and_ascii_boundaries() {
    let interpreter = HtmlValueInterpreter::new(64);
    let raw = DocumentString::from_utf16(vec![0x20, 0x41, 0x09, 0x41, 0xa0, 0x42, 0x20, 0xd800]);
    let report = interpreter.space_separated_tokens(Some(&raw)).unwrap();
    assert_eq!(report.author(), &Author::NotDefined);
    let Recovery::Value(tokens) = report.recovery() else {
        panic!("expected tokens")
    };
    let units: Vec<&[u16]> = tokens
        .tokens()
        .iter()
        .map(|token| token.value().code_units())
        .collect();
    assert_eq!(
        units,
        vec![&[0x41][..], &[0x41, 0xa0, 0x42][..], &[0xd800][..]]
    );
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.syntax(), HtmlValueSyntax::SpaceSeparatedTokens);

    let raw = DocumentString::from(" A, A, ,");
    let report = interpreter.comma_separated_tokens(Some(&raw)).unwrap();
    assert_eq!(report.author(), &Author::NotDefined);
    let Recovery::Value(tokens) = report.recovery() else {
        panic!("expected tokens")
    };
    assert_eq!(
        tokens
            .tokens()
            .iter()
            .map(|token| token.value().to_utf8().unwrap())
            .collect::<Vec<_>>(),
        vec!["A", "A", ""]
    );
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!(report.syntax(), HtmlValueSyntax::CommaSeparatedTokens);

    let missing = interpreter.space_separated_tokens(None).unwrap();
    assert_eq!(missing.recovery(), &Recovery::Missing);
    let raw = DocumentString::from("");
    let empty = interpreter.space_separated_tokens(Some(&raw)).unwrap();
    let Recovery::Value(tokens) = empty.recovery() else {
        panic!("expected empty token list")
    };
    assert!(tokens.tokens().is_empty());
    assert_eq!(empty.raw(), Some(&raw));
    assert!(
        HtmlValueInterpreter::new(0)
            .comma_separated_tokens(Some(&DocumentString::from(",")))
            .is_err()
    );
}

#[test]
fn reports_retain_old_raw_values_and_source_qualification() {
    let raw = DocumentString::from("0012");
    let interpreter = HtmlValueInterpreter::new(4);
    let old = interpreter.integer(Some(&raw)).unwrap();
    let next = interpreter
        .integer(Some(&DocumentString::from("bad")))
        .unwrap();
    assert!(matches!(old.author(), Author::Valid(_)));
    assert!(matches!(next.author(), Author::Invalid(_)));
    assert_eq!(old.raw(), Some(&raw));
    assert_eq!(
        old.syntax().source_clause(),
        "https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#signed-integers"
    );
}
