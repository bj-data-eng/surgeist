#![forbid(unsafe_code)]
//! Fonts 4 2026-09-07 §12.2 requires one or more literal families.
//! Pinned WebKit 73aa6c89 CSSFontFeatureValuesRule.cpp setter requires complete
//! native nongeneric prelude consumption; its getter uses comma-space joining.
//! Native CSS Syntax EOF recovery is distinct from a clean-report policy.
use surgeist_css::*;

fn list(source: &str) -> CssFontFeatureValuesFamilyList {
    let report = parse_font_feature_values_family_list(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().clone().expect("complete literal list")
}
fn component_error(
    report: &CssParseReport<Option<CssFontFeatureValuesFamilyList>>,
) -> &CssComponentValueError {
    assert!(report.syntax().is_none());
    report
        .diagnostics()
        .iter()
        .find_map(|diagnostic| match diagnostic.error().kind() {
            ErrorKind::InvalidComponentValue(error) => Some(error.as_ref()),
            _ => None,
        })
        .expect("typed component resource failure")
}

#[test]
fn native_literal_names_decode_escapes_and_serialize_in_order() {
    let value = list(r#"'serif', A B, A\ B, \31 Face, 日本語, 'a"b', ''"#);
    let names: Vec<_> = value
        .families()
        .iter()
        .map(CssFontFaceFamily::as_str)
        .collect();
    assert_eq!(
        names,
        ["serif", "A B", "A B", "1Face", "日本語", "a\"b", ""]
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        r#""serif", A B, A B, "1Face", 日本語, "a\"b", """#
    );
    let reparsed = list(&value.serialize_specified().unwrap());
    assert_eq!(reparsed.families(), value.families());
}

#[test]
fn reserved_generics_and_incomplete_lists_reject_the_whole_input() {
    for source in [
        "",
        "/*empty*/",
        "serif",
        "SeRiF",
        "sans-serif",
        "ui-serif",
        "initial",
        "A default",
        r"\73 erif",
        "A,",
        ",A",
        "A,,B",
        "A;B",
        "A{}",
        "A !important",
        "A, 12",
        "'bad\nname'",
    ] {
        let report = parse_font_feature_values_family_list(source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.diagnostics().is_empty(), "{source}");
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|d| matches!(d.error().kind(), ErrorKind::InvalidComponentValue(_))),
            "syntax rejection is not resource exhaustion: {source}"
        );
    }
    for source in ["'serif'", "'initial'", "'default'", "A,B"] {
        assert!(!list(source).families().is_empty());
    }
}

#[test]
fn original_member_occurrences_survive_input_drop_and_programmatic_lists_claim_no_source() {
    let source = String::from("/*😀*/\n'A', 日本語 /*tail*/");
    let value = list(&source);
    let expected = source.clone();
    drop(source);
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed original input")
    };
    assert_eq!(origin.source().as_str(), expected);
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), expected.len());
    let first = value.family_origin(0).unwrap();
    let second = value.family_origin(1).unwrap();
    assert_eq!(first.source().as_str(), expected);
    assert_eq!(
        first.span().start().byte_offset().value(),
        expected.find("'A'").unwrap()
    );
    assert_eq!(
        first.span().end().byte_offset().value(),
        expected.find(',').unwrap()
    );
    assert_eq!(
        second.span().start().byte_offset().value(),
        expected.find("日本語").unwrap()
    );
    assert_eq!(
        second.span().end().byte_offset().value(),
        expected.find(" /*tail*/").unwrap()
    );
    assert!(value.family_origin(2).is_none());
    let constructed = CssFontFeatureValuesFamilyList::try_new(value.families().to_vec()).unwrap();
    assert_eq!(constructed.origin(), &CssValueOrigin::Programmatic);
    assert!(constructed.family_origin(0).is_none());
    assert_eq!(constructed.serialize_specified().unwrap(), "A, 日本語");
    assert_eq!(
        CssFontFeatureValuesFamilyList::try_new(vec![])
            .unwrap_err()
            .kind(),
        CssFontFeatureValuesErrorKind::EmptyFamilies
    );
    let quirks = CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
    assert_eq!(
        quirks
            .parse_font_feature_values_family_list(&expected)
            .syntax()
            .as_ref()
            .unwrap()
            .families(),
        value.families()
    );
}

#[test]
fn valid_eof_recovery_retains_literal_names_and_original_source() {
    for (source, name) in [("'Demo", "Demo"), ("Demo/*unterminated", "Demo")] {
        let report = parse_font_feature_values_family_list(source);
        assert!(!report.is_clean(), "{source}: {report:?}");
        let value = report
            .syntax()
            .as_ref()
            .expect("valid EOF closure is retained");
        assert_eq!(value.families()[0].as_str(), name);
        let CssValueOrigin::Parsed(origin) = value.origin() else {
            panic!("parsed")
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(value.serialize_specified().unwrap(), "Demo");
    }
    assert!(
        parse_font_feature_values_family_list("A, 'bad\n")
            .syntax()
            .is_none()
    );
}

#[test]
fn complete_input_limits_include_later_members_and_retry_has_no_partial_state() {
    let source = "A,日"; // Three components and six UTF-8 bytes.
    let exact = CssComponentValueLimits::try_new(0, 3, source.len()).unwrap();
    let report = parse_font_feature_values_family_list_with_limits(source, exact);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().as_ref().unwrap().families().len(), 2);
    let short = CssComponentValueLimits::try_new(0, 2, source.len()).unwrap();
    let failed = parse_font_feature_values_family_list_with_limits(source, short);
    let error = component_error(&failed);
    assert_eq!(error.kind(), CssComponentValueErrorKind::ComponentLimit);
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("later token origin")
    };
    assert_eq!(origin.span().start().byte_offset().value(), 2);
    assert_eq!(origin.source().as_str(), source);
    let failed = parse_font_feature_values_family_list_with_limits(
        source,
        CssComponentValueLimits::try_new(0, 3, source.len() - 1).unwrap(),
    );
    assert_eq!(
        component_error(&failed).kind(),
        CssComponentValueErrorKind::ByteLimit
    );
    assert!(
        matches!(component_error(&failed).origin(), CssValueOrigin::UnretainedInput { byte_length } if *byte_length == source.len())
    );
    let failed = parse_font_feature_values_family_list_with_limits(
        "A, f(B)",
        CssComponentValueLimits::try_new(0, 99, 99).unwrap(),
    );
    assert_eq!(
        component_error(&failed).kind(),
        CssComponentValueErrorKind::NestingLimit
    );
    // Resource admission covers even trailing syntax that grammar would reject.
    let failed = parse_font_feature_values_family_list_with_limits(
        "A;B",
        CssComponentValueLimits::try_new(0, 2, 99).unwrap(),
    );
    assert_eq!(
        component_error(&failed).kind(),
        CssComponentValueErrorKind::ComponentLimit
    );
    assert_eq!(
        CssParserContext::default()
            .parse_font_feature_values_family_list_with_limits(source, exact)
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        "A, 日"
    );
}

#[test]
fn literal_slice_writer_charges_all_members_and_separators_atomically() {
    let families = vec![
        CssFontFaceFamily::try_new("A").unwrap(),
        CssFontFaceFamily::try_new("serif").unwrap(),
        CssFontFaceFamily::try_new("日").unwrap(),
    ];
    let before = families.clone();
    let expected = "A, \"serif\", 日";
    let exact = CssSpecifiedValueSerializationLimits::new(3, 3, expected.len());
    assert_eq!(
        serialize_font_face_family_list_with_limits(&families, exact).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 2),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialize_font_face_family_list_with_limits(&families, limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(families, before);
    }
    assert_eq!(
        serialize_font_face_family_list_with_limits(&families, exact).unwrap(),
        expected
    );
    let checked = CssFontFeatureValuesFamilyList::try_new(families).unwrap();
    assert_eq!(
        checked.serialize_specified_with_limits(exact).unwrap(),
        expected
    );
    assert_eq!(
        serialize_font_face_family_list_with_limits(
            &[],
            CssSpecifiedValueSerializationLimits::new(0, 0, 0)
        )
        .unwrap(),
        ""
    );
}

#[test]
fn trivia_and_original_line_endings_are_charged_without_rewriting_occurrences() {
    let source = "/*😀*/\r\nA,\r\n日";
    let value = list(source);
    for (index, spelling) in [(0, "A"), (1, "日")] {
        let origin = value.family_origin(index).unwrap();
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find(spelling).unwrap()
        );
        assert_eq!(
            origin.span().end().byte_offset().value(),
            source.find(spelling).unwrap() + spelling.len()
        );
    }
    let source = "A/*x*/,B"; // Ident, comment, comma, ident: four components.
    let limited = parse_font_feature_values_family_list_with_limits(
        source,
        CssComponentValueLimits::try_new(0, 3, source.len()).unwrap(),
    );
    assert_eq!(
        component_error(&limited).kind(),
        CssComponentValueErrorKind::ComponentLimit
    );
    let CssValueOrigin::Parsed(origin) = component_error(&limited).origin() else {
        panic!("original member")
    };
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find('B').unwrap()
    );
    assert!(
        parse_font_feature_values_family_list_with_limits(
            source,
            CssComponentValueLimits::try_new(0, 4, source.len()).unwrap()
        )
        .is_clean()
    );
    let source = "/*😀*/\r\nA, 12";
    let rejected = parse_font_feature_values_family_list(source);
    assert!(rejected.syntax().is_none());
    assert_eq!(
        rejected.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        source.find("12").unwrap()
    );
}
