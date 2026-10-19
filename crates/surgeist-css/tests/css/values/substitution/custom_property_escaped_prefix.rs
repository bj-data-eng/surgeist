#![forbid(unsafe_code)]

//! Public custom-name boundary tests. Variables1
//! CR20220616 §2 defines decoded dashed-ident identity and reserved `--`;
//! Syntax3 CRD20211224 §§4.3.7/4.3.9/4.3.11 define the escape decoding.
//! CssCustomPropertyName::try_new publicly accepts one authored identifier
//! token, whereas as_str returns its decoded identity.

use std::collections::HashSet;
use surgeist_css::*;

fn name_region(declaration: &CssDeclaration, source: &str, start: usize, end: usize) {
    let origin = declaration.parsed_name().unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
    assert_eq!(declaration.position().unwrap(), origin.span().start());
}

#[test]
fn escaped_leading_dashes_are_one_authored_token_with_plain_decoded_identity() {
    let plain = CssCustomPropertyName::try_new("--Theme").unwrap();
    let mut equivalent_names = HashSet::new();
    for authored in [
        "--Theme",
        r"\2d \2d Theme",
        r"\2D \2D Theme",
        r"\-\-Theme",
        r"-\2d Theme",
        r"\2d -Theme",
        r"\00002d\00002dTheme",
    ] {
        let name = CssCustomPropertyName::try_new(authored)
            .unwrap_or_else(|| panic!("one authored custom identifier: {authored:?}"));
        assert_eq!(name.as_str(), "--Theme");
        assert_eq!(name, plain);
        equivalent_names.insert(name);
    }
    assert_eq!(equivalent_names.len(), 1);

    // The constructor accepts CSS spelling, not an arbitrary decoded String.
    let escaped_space = CssCustomPropertyName::try_new(r"\2d \2d bad\ name").unwrap();
    assert_eq!(escaped_space.as_str(), "--bad name");
    assert!(CssCustomPropertyName::try_new("--bad name").is_none());
}

#[test]
fn reserved_decoded_name_and_non_identifier_or_residual_source_remain_rejected() {
    for authored in [
        "--",
        r"\2d \2d",
        r"\-\-",
        r"\00002d\00002d",
        r"\2d Theme", // Decodes to only one leading dash.
        r"\2d \2d Theme extra",
        r"\2d \2d Theme;",
        r"\2d \2d Theme()", // A function token, not an identifier token.
        r"\2d \2d Theme/**/",
        r" \2d \2d Theme",
        "\\2d \\2d Theme\n",
    ] {
        assert!(
            CssCustomPropertyName::try_new(authored).is_none(),
            "whole-token negative: {authored:?}"
        );
    }
}

#[test]
fn attribute_and_singular_fronts_decode_prefix_escapes_and_retain_original_name_tokens() {
    const SOURCE: &str = r"\2d \2d Theme:RED!important;--Theme:BLUE;-\2d Theme:MiXeD";
    let report = parse_style_attribute(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [first, second, third] = report.syntax().as_slice() else {
        panic!("three authored custom occurrences");
    };
    let plain = CssCustomPropertyName::try_new("--Theme").unwrap();
    for (declaration, value, importance) in [
        (first, "RED", CssImportance::Important),
        (second, "BLUE", CssImportance::Normal),
        (third, "MiXeD", CssImportance::Normal),
    ] {
        assert_eq!(declaration.custom().unwrap().name(), &plain);
        assert!(declaration.known().is_none());
        assert!(declaration.is_name_case_sensitive());
        assert_eq!(declaration.importance(), importance);
        assert_eq!(
            declaration
                .custom()
                .unwrap()
                .value()
                .value()
                .unwrap()
                .as_css(),
            value
        );
    }
    name_region(first, SOURCE, 0, 13);
    name_region(second, SOURCE, 28, 35);
    name_region(third, SOURCE, 41, 51);
    assert!(
        first
            .parsed_name()
            .unwrap()
            .source()
            .same_snapshot(third.parsed_name().unwrap().source())
    );
    assert_eq!(
        validate_style_attribute(SOURCE).unwrap(),
        report.syntax().clone()
    );

    const SINGLE: &str = r"\2d \2d Theme:RED";
    let report = parse_declaration(SINGLE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let declaration = report.syntax().as_ref().unwrap();
    assert_eq!(declaration.custom().unwrap().name(), &plain);
    name_region(declaration, SINGLE, 0, 13);
    assert_eq!(declaration.to_specified_css().unwrap(), "--Theme: RED;");
}

#[test]
fn checked_custom_admission_emits_decoded_name_and_preserves_supplied_value_origin() {
    let name = CssCustomPropertyName::try_new(r"\2d \2d Theme").unwrap();
    let components = parse_component_values("MiXeD").unwrap();
    let before_components = components.clone();
    let declaration = parse_property_value(
        CssPropertyNameRef::Custom(&name),
        components,
        CssImportance::Important,
    )
    .unwrap();
    let before = declaration.clone();
    assert_eq!(declaration.custom().unwrap().name().as_str(), "--Theme");
    assert!(declaration.known().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert_eq!(declaration.value_components(), &before_components);
    assert_eq!(declaration.importance(), CssImportance::Important);
    const EXPECTED: &str = "--Theme: MiXeD !important;";
    assert_eq!(declaration.to_specified_css().unwrap(), EXPECTED);
    assert_eq!(declaration, before);
    assert!(declaration.same_occurrence(&before));
    let (CssValueOrigin::Parsed(original), CssValueOrigin::Parsed(retained)) = (
        before_components.items()[0].origin(),
        declaration.value_components().items()[0].origin(),
    ) else {
        panic!("supplied value token preserves its actual parsed origin");
    };
    assert!(original.source().same_snapshot(retained.source()));
    let reparsed = validate_style_attribute(EXPECTED).unwrap();
    let [reparsed] = reparsed.as_slice() else {
        panic!("one canonical custom declaration");
    };
    assert_eq!(reparsed.custom().unwrap().name(), &name);
    assert_eq!(reparsed.to_specified_css().unwrap(), EXPECTED);
}

#[test]
fn escaped_prefix_decoding_preserves_case_and_unicode_codepoint_distinctions() {
    let upper = CssCustomPropertyName::try_new(r"\2d \2d Theme").unwrap();
    let lower = CssCustomPropertyName::try_new(r"\2d \2d theme").unwrap();
    assert_eq!(upper.as_str(), "--Theme");
    assert_eq!(lower.as_str(), "--theme");
    assert_ne!(upper, lower);

    // Variables1's own example contrasts U+00F3 with U+006F U+0301.
    let composed = CssCustomPropertyName::try_new(r"\2d \2d fo\f3").unwrap();
    let decomposed = CssCustomPropertyName::try_new(r"\2d \2d foo\301").unwrap();
    assert_eq!(composed.as_str(), "--foó");
    assert_eq!(decomposed.as_str(), "--foo\u{301}");
    assert_eq!(composed, CssCustomPropertyName::try_new("--foó").unwrap());
    assert_eq!(
        decomposed,
        CssCustomPropertyName::try_new("--foo\u{301}").unwrap()
    );
    assert_ne!(composed, decomposed);
    let names = HashSet::from([upper, lower, composed, decomposed]);
    assert_eq!(names.len(), 4);
}
