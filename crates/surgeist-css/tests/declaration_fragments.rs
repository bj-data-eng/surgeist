#![forbid(unsafe_code)]

//! Singular declaration contracts follow CSS Syntax 3 section 5.3.6 and
//! declaration-value restrictions in section 8.2, with typed property validation.
use surgeist_css::{
    CssImportance, CssKnownProperty, CssPropertyNameRef, CssRecoveryAction, CssRule,
    parse_component_values, parse_declaration, parse_property_value, parse_sheet,
    parse_style_attribute,
};

fn rejected(source: &str) {
    let report = parse_declaration(source);
    assert!(report.syntax().is_none(), "{source:?}: {report:?}");
    assert!(!report.is_clean(), "{source:?}");
    for diagnostic in report.diagnostics() {
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        assert!(diagnostic.error().position().byte_offset().value() <= source.len());
    }
}

#[test]
fn ordinary_values_return_checked_property_coupled_declarations() {
    for (source, property, value) in [
        ("color:red", CssKnownProperty::Color, "red"),
        ("width:2px", CssKnownProperty::Width, "2px"),
    ] {
        let report = parse_declaration(source);
        assert!(report.is_clean(), "{report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        let constructed = parse_property_value(
            CssPropertyNameRef::Known(property),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(declaration.body(), constructed.body());
        assert_eq!(declaration.known().unwrap().property(), property);
        assert_eq!(declaration.importance(), CssImportance::Normal);
        assert!(constructed.parsed_name().is_none());
        assert!(constructed.parsed_value().is_none());
        assert!(declaration.same_occurrence(&declaration.clone()));
        assert!(!declaration.same_occurrence(&constructed));
    }
}

#[test]
fn known_variable_values_remain_symbolic() {
    let report = parse_declaration("width:var(--size)");
    assert!(report.is_clean(), "{report:?}");
    let known = report.syntax().as_ref().unwrap().known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::Width);
    assert!(known.substitution_dependent().is_some());
}

#[test]
fn custom_values_preserve_ordered_nested_data_and_allow_empty_values() {
    for value in [
        "",
        " ",
        "/**/",
        " \t/**/ ",
        "{a:b;c:d} [x y] foo(z)",
        "\";!})]\" /* ;!})] */ tail",
        "initial",
        "var(--other)",
    ] {
        let source = format!("--x:{value}");
        let report = parse_declaration(&source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        assert_eq!(declaration.custom().unwrap().name().as_str(), "--x");
        let origin = declaration.parsed_value().unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), 4);
        assert_eq!(origin.span().end().byte_offset().value(), source.len());
        let actual = declaration.value_components().serialize().unwrap();
        let expected = parse_component_values(value).unwrap().serialize().unwrap();
        assert_eq!(actual.as_css(), expected.as_css());
    }
}

#[test]
fn terminal_important_accepts_case_escapes_and_comments() {
    for annotation in [
        "!important",
        "!IMPORTANT",
        "!/**/important/**/",
        "!\\69mportant",
    ] {
        for value in ["color:red", "--x:data", "--x:"] {
            let source = format!("{value}{annotation}");
            let report = parse_declaration(&source);
            assert!(report.is_clean(), "{source:?}: {report:?}");
            assert_eq!(
                report.syntax().as_ref().unwrap().importance(),
                CssImportance::Important
            );
        }
    }
    for source in [
        "color:red!",
        "color:red!important tail",
        "--x:a!urgent",
        "--x:a!important!important",
        "--x:a! important x",
    ] {
        rejected(source);
    }
}

#[test]
fn escaped_and_unicode_names_keep_original_snapshot_and_crlf_coordinates() {
    for (source, name, value, name_start, name_end, value_start, value_end) in [
        (
            "/*😀*/\r\n\\63 olor:red!important",
            "\\63 olor",
            "red",
            10,
            18,
            19,
            22,
        ),
        ("/*😀*/\r\n--é:blue", "--é", "blue", 10, 14, 15, 19),
    ] {
        let report = parse_declaration(source);
        assert!(report.is_clean(), "{report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        let name_origin = declaration.parsed_name().unwrap();
        let value_origin = declaration.parsed_value().unwrap();
        assert_eq!(name_origin.source().as_str(), source);
        assert!(name_origin.source().same_snapshot(value_origin.source()));
        assert_eq!(name_origin.span().start().byte_offset().value(), name_start);
        assert_eq!(name_origin.span().end().byte_offset().value(), name_end);
        assert_eq!(
            value_origin.span().start().byte_offset().value(),
            value_start
        );
        assert_eq!(value_origin.span().end().byte_offset().value(), value_end);
        assert_eq!(&source[name_start..name_end], name);
        assert_eq!(&source[value_start..value_end], value);
        assert_eq!(declaration.position().unwrap().line().value(), 1);
        assert_eq!(declaration.position().unwrap().column().value(), 0);
    }
    let report = parse_declaration(" /*before*/ color:red /*after*/ ");
    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn invalid_outer_syntax_and_values_reject_the_complete_input() {
    for source in [
        "",
        " \t/**/",
        "color",
        "color:",
        ":red",
        "--:x",
        "unknown:x",
        "@x:y",
        "a{color:red}",
        ";color:red",
        "color:red;",
        "color:red;;",
        "color:red;width:1px",
        "--x:a;--y:b",
        "broken;--x:a",
        "--x:a;broken",
        "--x:a)",
        "--x:a]",
        "--x:a}",
        "--x:)",
        "--x:]",
        "--x:}",
        "--x: ([)]",
        "filter:alpha(opacity",
        "--x:\"bad\nstring",
        "--x:url(a b)",
        "color:var(foo)",
    ] {
        rejected(source);
    }
}

#[test]
fn root_delimiter_after_nested_value_is_the_responsible_token() {
    let source = "--x:f(a);";
    let report = parse_declaration(source);
    assert!(report.syntax().is_none());
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 8);
}

#[test]
fn valid_implicit_closures_are_retained_only_after_complete_declaration_success() {
    for source in ["--x:foo(", "filter:blur("] {
        let report = parse_declaration(source);
        assert!(report.syntax().is_some(), "{source}: {report:?}");
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| d.span().end().byte_offset().value() <= source.len())
        );
    }
    rejected("--x: ([)]");
    rejected("filter:alpha(opacity");
}

#[test]
fn component_depth_exhaustion_preserves_resource_action() {
    let source = format!("--x:{}x{}", "f(".repeat(300), ")".repeat(300));
    let report = parse_declaration(&source);
    assert!(report.syntax().is_none());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::StopAtNestingLimit)
    );
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
}

#[test]
fn declaration_lists_still_recover_ordered_siblings_at_authored_positions() {
    let source = "color:red;unknown:x;width:2px;";
    let report = parse_style_attribute(source);
    let declarations = report.syntax();
    assert_eq!(declarations.len(), 2);
    assert_eq!(
        declarations[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        declarations[1].known().unwrap().property(),
        CssKnownProperty::Width
    );
    assert_eq!(declarations[0].position().unwrap().byte_offset().value(), 0);
    assert_eq!(
        declarations[1].position().unwrap().byte_offset().value(),
        20
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|d| d.action() == CssRecoveryAction::DropDeclaration)
    );
    let report = parse_sheet(&format!("a{{{source}}}"));
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("style rule")
    };
    assert_eq!(rule.declarations().len(), 2);
    assert_eq!(
        rule.declarations()[1]
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        22
    );
}
