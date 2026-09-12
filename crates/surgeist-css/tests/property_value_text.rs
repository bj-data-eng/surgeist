#![forbid(unsafe_code)]

//! Supplemental public contracts for source-only property values. Expectations
//! follow the supplied grammar, explicit importance and original source coordinates.
use surgeist_css::{
    CssCustomPropertyName, CssGlobalKeyword, CssImportance, CssKnownProperty,
    CssKnownPropertyValueRef, CssLength, CssPropertyGrammar, CssPropertyNameRef, CssRecoveryAction,
    CssValueOrigin, parse_component_values, parse_declaration, parse_property_value,
    parse_property_value_text, parse_property_value_text_for_grammar,
};

fn width(source: &str) -> surgeist_css::CssParseReport<Option<surgeist_css::CssDeclaration>> {
    parse_property_value_text(
        source,
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        CssImportance::Normal,
    )
}

fn rejected(source: &str) {
    let report = width(source);
    assert!(report.syntax().is_none(), "{source:?}: {report:?}");
    assert_eq!(report.diagnostics().len(), 1, "{report:?}");
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn literal_width_has_independently_expected_typed_value_and_supplied_importance() {
    let report = parse_property_value_text(
        "10px",
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        CssImportance::Important,
    );
    assert!(report.is_clean(), "{report:?}");
    let declaration = report.syntax().as_ref().unwrap();
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(
        declaration.property_name(),
        CssPropertyNameRef::Known(CssKnownProperty::Width)
    );
    let CssKnownPropertyValueRef::Width(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("width")
    };
    assert!(matches!(value.i01_subset(), Some(CssLength::Px(value)) if value.value() == 10.0));
    assert!(declaration.same_occurrence(&declaration.clone()));
}

#[test]
fn raw_value_origin_is_distinct_from_parsed_declaration_and_checked_construction() {
    let source = "/*😀*/\r\n10px";
    let report = width(source);
    assert!(report.is_clean(), "{report:?}");
    let raw = report.syntax().as_ref().unwrap();
    assert!(raw.position().is_none());
    assert!(raw.parsed_name().is_none());
    let value = raw.parsed_value().unwrap();
    assert_eq!(value.source().as_str(), source);
    assert_eq!(value.span().start().byte_offset().value(), 0);
    assert_eq!(value.span().end().byte_offset().value(), source.len());
    let token = raw.value_components().items().last().unwrap();
    let CssValueOrigin::Parsed(token) = token.origin() else {
        panic!("parsed")
    };
    assert!(token.source().same_snapshot(value.source()));
    assert_eq!(token.span().start().byte_offset().value(), 10);
    assert_eq!(token.span().start().line().value(), 1);
    assert_eq!(token.span().start().column().value(), 0);
    let constructed = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        parse_component_values(source).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(constructed.parsed_name().is_none());
    assert!(constructed.parsed_value().is_none());
    assert!(constructed.position().is_none());
    let declaration = parse_declaration("width:10px");
    let declaration = declaration.syntax().as_ref().unwrap();
    assert!(declaration.parsed_name().is_some());
    assert!(declaration.parsed_value().is_some());
    assert!(declaration.position().is_some());
}

#[test]
fn custom_values_preserve_empty_regions_and_nested_punctuation() {
    let name = CssCustomPropertyName::try_new("--Case").unwrap();
    for source in [
        "",
        " ",
        "/**/",
        "{a:b;c:!important} [x] f(;!)",
        "\";!})]\" /*;!})]*/ tail",
    ] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Custom(&name),
            CssImportance::Important,
        );
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        assert_eq!(declaration.custom().unwrap().name().as_str(), "--Case");
        let value = declaration.parsed_value().unwrap();
        assert_eq!(value.span().start().byte_offset().value(), 0);
        assert_eq!(value.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn root_annotations_and_delimiters_reject_even_after_symbolic_substitution() {
    for source in [
        "10px;",
        "10px!important",
        "var(--width)!important",
        "var(--width);",
        "var(--width))",
        "var(--width)]",
        "var(--width)}",
        "",
        " ",
        "red",
        "-50%",
        "inherit 1px",
        "var(foo)",
        "\"bad\nstring",
        "url(a b)",
    ] {
        rejected(source);
    }
    for (source, offset, column) in [("var(--width);", 12, 12), ("/*😀*/10px;", 12, 10)] {
        let report = width(source);
        let position = report.diagnostics()[0].error().position();
        assert_eq!(position.byte_offset().value(), offset);
        assert_eq!(position.column().value(), column);
    }
}

#[test]
fn canonical_and_legacy_grammars_retain_global_and_symbolic_values() {
    let grammar = CssPropertyGrammar::from_name("glyph-orientation-vertical").unwrap();
    for source in ["0deg", "inherit", "var(--orientation)"] {
        let report = parse_property_value_text_for_grammar(source, grammar, CssImportance::Normal);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let known = report.syntax().as_ref().unwrap().known().unwrap();
        assert_eq!(known.property(), CssKnownProperty::TextOrientation);
        assert_eq!(known.grammar(), grammar);
    }
    let report = width("inherit");
    assert_eq!(
        report.syntax().as_ref().unwrap().known().unwrap().global(),
        Some(CssGlobalKeyword::Inherit)
    );
    let report = width("var(--size)");
    assert!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
}

#[test]
fn actual_eof_closures_are_transactional_and_resource_limits_stay_typed() {
    let source = "var(--width";
    let report = width(source);
    assert!(report.syntax().is_some(), "{report:?}");
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::RetainWithImplicitClosure
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.len()
    );
    rejected("unknown(");
    let source = format!("{}x{}", "f(".repeat(300), ")".repeat(300));
    let report = width(&source);
    assert!(report.syntax().is_none());
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::StopAtNestingLimit
    );
}
