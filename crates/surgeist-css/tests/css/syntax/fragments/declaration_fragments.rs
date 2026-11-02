//! Singular declaration contracts follow CSS Syntax 3 section 5.4.7 and
//! declaration-value restrictions in section 8.2, with typed property validation.
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTarget};
use surgeist_css::rule::CssRule;

fn rejected(source: &str) {
    let report = CssDeclaration::parse(&CssParser::default(), source);
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
        let report = CssDeclaration::parse(&CssParser::default(), source);
        assert!(report.is_clean(), "{report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        let constructed = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(property)),
            CssComponentValues::parse(&CssParser::default(), value).unwrap(),
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
    let report = CssDeclaration::parse(&CssParser::default(), "width:var(--size)");
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
        let report = CssDeclaration::parse(&CssParser::default(), &source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        assert_eq!(declaration.custom().unwrap().name().as_str(), "--x");
        let origin = declaration.parsed_value().unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), 4);
        assert_eq!(origin.span().end().byte_offset().value(), source.len());
        let actual = declaration
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap();
        let expected = CssComponentValues::parse(&CssParser::default(), value)
            .unwrap()
            .serialize_tokens(Default::default())
            .unwrap();
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
            let report = CssDeclaration::parse(&CssParser::default(), &source);
            assert!(report.is_clean(), "{source:?}: {report:?}");
            assert_eq!(
                report.syntax().as_ref().unwrap().importance(),
                CssImportance::Important
            );
        }
    }
    for source in ["color:red!", "color:red!important tail"] {
        rejected(source);
    }
    // The flag is the last two tokens of the value only (Syntax 3 section 5.5.6
    // step 6); anywhere else the `!` is part of a custom property's value, as a
    // declaration in a block has it.
    for (source, value, importance) in [
        ("--x:a!urgent", "a!urgent", CssImportance::Normal),
        (
            "--x:a! important x",
            "a! important x",
            CssImportance::Normal,
        ),
        ("--x:1!important x", "1!important x", CssImportance::Normal),
        (
            "--x:a!important!important",
            "a!important",
            CssImportance::Important,
        ),
    ] {
        let report = CssDeclaration::parse(&CssParser::default(), source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        let declaration = report.syntax().as_ref().unwrap();
        assert_eq!(declaration.importance(), importance, "{source:?}");
        let span = declaration.parsed_value().unwrap().span();
        assert_eq!(
            &source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            value,
            "{source:?}"
        );
        let in_block = CssParser::default().parse_block_contents(source);
        assert!(in_block.is_clean(), "{source:?}: {in_block:?}");
        assert_eq!(in_block.syntax()[0].importance(), importance, "{source:?}");
    }
}

#[test]
fn a_backslash_before_a_newline_is_no_escape_in_a_declaration_text() {
    // Syntax 3 sections 4.3.1 and 4.3.8: `\` followed by a newline is not a valid
    // escape, so the `-` before it does not start an ident and is a delimiter.
    let report = CssDeclaration::parse(&CssParser::default(), "inia -\\\n btial");
    assert!(report.syntax().is_none(), "{report:?}");
    let error = report.diagnostics()[0].error();
    assert_eq!(
        error.code(),
        surgeist_css::diagnostic::CssErrorCode::UnexpectedToken
    );
    assert_eq!(error.encountered().unwrap().authored(), "-");
    assert_eq!(error.position().byte_offset().value(), 5);
}

#[test]
fn a_block_as_part_of_a_larger_value_rejects_the_declaration() {
    // A top-level `{}` block may be the whole value of a property and nothing
    // else (Syntax 3 section 5.5.6 step 5); a custom property's value takes any
    // components.
    for source in [
        "width:var(--a){}",
        "color:red{x}",
        "width:var(--a)/**/{a:b;",
    ] {
        let report = CssDeclaration::parse(&CssParser::default(), source);
        assert!(report.syntax().is_none(), "{source:?}: {report:?}");
        assert_eq!(
            report.diagnostics()[0].error().code(),
            surgeist_css::diagnostic::CssErrorCode::UnexpectedToken,
            "{source:?}"
        );
    }
    for source in ["--x:{a}", "--x:{a} b", "--x:a{b}"] {
        let report = CssDeclaration::parse(&CssParser::default(), source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
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
        let report = CssDeclaration::parse(&CssParser::default(), source);
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
    let report = CssDeclaration::parse(&CssParser::default(), " /*before*/ color:red /*after*/ ");
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
    let report = CssDeclaration::parse(&CssParser::default(), source);
    assert!(report.syntax().is_none());
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 8);
}

#[test]
fn valid_implicit_closures_are_retained_only_after_complete_declaration_success() {
    for source in ["--x:foo(", "filter:blur("] {
        let report = CssDeclaration::parse(&CssParser::default(), source);
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
    let report = CssDeclaration::parse(&CssParser::default(), &source);
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
fn declaration_lists_recover_ordered_siblings_at_authored_positions() {
    let source = "color:red;unknown:x;width:2px;";
    let report = CssParser::default().parse_block_contents(source);
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
    let report = CssParser::default().parse_sheet(format!("a{{{source}}}"));
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
