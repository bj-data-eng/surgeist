#![forbid(unsafe_code)]
//! Independent raw-list oracles: selected CSS Syntax 3 CRD20211224 §§5.3.8,
//! 5.4.2 and 5.4.5, followed by the ordinary property consumer contract.
//! This wholly new front has functional tests alongside implementation;
//! no executable preimplementation RED is claimed.
use surgeist_css::*;

fn names(declarations: &CssDeclarationList) -> Vec<&str> {
    declarations
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            _ => panic!("ordinary test name"),
        })
        .collect()
}

fn dropped(source: &str, expected_names: &[&str], unit: &str, responsible: usize) {
    let report = parse_declaration_list_text(source);
    assert_eq!(
        names(report.syntax()),
        expected_names,
        "{source:?}: {report:?}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid unit: {report:?}")
    };
    let start = source.find(unit).unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        start + responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + unit.len()
    );
    assert!(!report.is_clean());
    assert_eq!(
        validate_declaration_list_text(source)
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn empty_slots_and_optional_final_semicolon_are_clean() {
    for source in ["", " \t/**/\r\n", "; ;/**/;", "width: 2px", "width: 2px;"] {
        let report = parse_declaration_list_text(source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert_eq!(report.syntax().is_empty(), !source.contains("width"));
        assert_eq!(
            &validate_declaration_list_text(source).unwrap(),
            report.syntax()
        );
    }
}

#[test]
fn stray_root_closer_consumes_the_following_declaration_in_its_unit() {
    for closer in ["}", "]", ")"] {
        let unit = format!("{closer} color: red");
        let source = format!("{unit}; width: 2px;");
        dropped(&source, &["width"], &unit, 0);
        let raw = parse_declaration_list_text(&source);
        assert_eq!(
            raw.diagnostics()[0].error().code(),
            CssErrorCode::UnexpectedToken
        );
        // The accepted attribute contract discards only the stray closer.
        let attribute = parse_style_attribute(&source);
        assert_eq!(names(attribute.syntax()), ["color", "width"]);
        assert_eq!(attribute.diagnostics().len(), 1);
        assert_eq!(
            attribute.diagnostics()[0]
                .span()
                .end()
                .byte_offset()
                .value(),
            1
        );
    }
}

#[test]
fn invalid_started_units_consume_nested_semicolons_comments_and_strings() {
    for unit in [
        "] --X: {a:b;c:d}",
        ") --X: fn([a;b], {c:d;e:f})",
        ", --X: \";})]\" /* ;})] */ data",
        ".bad { color: red; } color: blue",
        "<!-- color: red",
        "--> color: red",
    ] {
        let source = format!("{unit}; width: 2px;");
        dropped(&source, &["width"], unit, 0);
    }
}

#[test]
fn ident_started_invalid_units_do_not_admit_nested_rules_or_inner_declarations() {
    dropped(
        "broken color: red; width: 2px;",
        &["width"],
        "broken color: red",
        7,
    );
    dropped(
        "a { color: red; } height: 3px; width: 2px;",
        &["width"],
        "a { color: red; } height: 3px",
        2,
    );
}

#[test]
fn generic_at_rules_end_after_their_block_or_semicolon() {
    for unit in [
        "@unknown x",
        "@media fn([a;b]) { color: red; .x { height: 3px; } }",
        "@font-face { font-family: Test; src: url(test); }",
        "@unknown \";{}\" /* ;{} */ { --X: {a:b;c:d}; }",
    ] {
        let source = format!("{unit}; width: 2px;");
        dropped(&source, &["width"], unit, 0);
        assert_eq!(
            parse_declaration_list_text(&source).diagnostics()[0]
                .error()
                .code(),
            CssErrorCode::UnexpectedToken
        );
    }
    dropped(
        "@unknown { color: red; } width: 2px;",
        &["width"],
        "@unknown { color: red; }",
        0,
    );
    dropped("@unknown x; width: 2px;", &["width"], "@unknown x", 0);
}

#[test]
fn duplicate_known_and_case_sensitive_custom_occurrences_remain_in_order() {
    let source = "WIDTH: 2px !IMPORTANT; width: 3px; --Case: A/**/ B; --case: C; --Case: var(--x) !/**/important; color: inherit";
    let report = parse_declaration_list_text(source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        names(report.syntax()),
        ["width", "width", "--Case", "--case", "--Case", "color"]
    );
    assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
    assert_eq!(report.syntax()[1].importance(), CssImportance::Normal);
    assert_eq!(report.syntax()[4].importance(), CssImportance::Important);
    assert!(!report.syntax()[0].same_occurrence(&report.syntax()[1]));
    assert!(!report.syntax()[2].same_occurrence(&report.syntax()[4]));
    assert_eq!(
        report.syntax()[2]
            .custom()
            .unwrap()
            .value()
            .value()
            .unwrap()
            .as_css(),
        "A/**/ B"
    );
    assert_eq!(
        report.syntax()[4]
            .custom()
            .unwrap()
            .value()
            .value()
            .unwrap()
            .as_css(),
        "var(--x)"
    );
    for declaration in report.syntax().iter() {
        let name = declaration.parsed_name().unwrap();
        let value = declaration.parsed_value().unwrap();
        assert_eq!(name.source().as_str(), source);
        assert!(name.source().same_snapshot(value.source()));
        assert!(
            name.source()
                .same_snapshot(report.syntax()[0].parsed_name().unwrap().source())
        );
    }
    assert_eq!(&source[..5], "WIDTH");
    assert_eq!(
        report.syntax()[0]
            .parsed_name()
            .unwrap()
            .span()
            .end()
            .byte_offset()
            .value(),
        5
    );
}

#[test]
fn valid_custom_components_keep_nested_and_lexical_semicolons_inside_one_value() {
    let value = "{a:b;c:d} fn([e;f]) \";})]\" /* ;})] */ tail";
    let source = format!("--X: {value}; width: var(--size, 2px) !important; --empty:;");
    let report = parse_declaration_list_text(&source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(names(report.syntax()), ["--X", "width", "--empty"]);
    assert_eq!(
        report.syntax()[0]
            .custom()
            .unwrap()
            .value()
            .value()
            .unwrap()
            .as_css(),
        value
    );
    assert_eq!(
        report.syntax()[1]
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        "var(--size, 2px)"
    );
    assert_eq!(report.syntax()[1].importance(), CssImportance::Important);
    assert_eq!(
        report.syntax()[2]
            .custom()
            .unwrap()
            .value()
            .value()
            .unwrap()
            .as_css(),
        ""
    );
}

#[test]
fn ordinary_grammar_drops_invalid_values_annotations_and_residual_tokens() {
    for (unit, code, responsible) in [
        ("width: red", CssErrorCode::InvalidPropertyValue, 7),
        ("mystery: 1", CssErrorCode::UnknownProperty, 0),
        (
            "width: 2px !oops",
            CssErrorCode::InvalidDeclarationAnnotation,
            11,
        ),
        (
            "--x: a!urgent",
            CssErrorCode::InvalidDeclarationAnnotation,
            6,
        ),
        ("--x: a}", CssErrorCode::UnexpectedToken, 6),
        ("broken", CssErrorCode::UnexpectedEnd, 6),
    ] {
        let source = format!("{unit}; height: 2px!important;");
        dropped(&source, &["height"], unit, responsible);
        let report = parse_declaration_list_text(&source);
        assert_eq!(
            report.diagnostics()[0].error().code(),
            code,
            "{unit}: {report:?}"
        );
        assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
    }
}

#[test]
fn mixed_failures_report_one_ordered_diagnostic_per_whole_unit() {
    let source = "width: red; } color: red; @x {} height: 2px; mystery: 1; --ok: yes";
    let report = parse_declaration_list_text(source);
    assert_eq!(names(report.syntax()), ["height", "--ok"]);
    let units = [
        ("width: red", CssErrorCode::InvalidPropertyValue, 7),
        ("} color: red", CssErrorCode::UnexpectedToken, 0),
        ("@x {}", CssErrorCode::UnexpectedToken, 0),
        ("mystery: 1", CssErrorCode::UnknownProperty, 0),
    ];
    assert_eq!(report.diagnostics().len(), units.len());
    for (diagnostic, (unit, code, responsible)) in report.diagnostics().iter().zip(units) {
        let start = source.find(unit).unwrap();
        assert_eq!(diagnostic.error().code(), code);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            start + responsible
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len()
        );
    }
    assert_eq!(
        validate_declaration_list_text(source)
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn invalid_eof_units_are_nonclean_empty_lists_with_actual_source_end() {
    for unit in [
        "} color: red",
        "] --X: fn([a;b",
        ".x {color:red}",
        "@unknown { color: red",
        "@unknown x",
    ] {
        dropped(unit, &[], unit, 0);
    }
    dropped("broken", &[], "broken", 6);
}

#[test]
fn exact_unicode_multiline_spans_and_retained_origins_use_unwrapped_input() {
    let source = "/*😀*/\r\n] --é: {a:b;c:d};\nwidth: 2px;\n--é:blue";
    let report = parse_declaration_list_text(source);
    assert_eq!(names(report.syntax()), ["width", "--é"]);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid unit")
    };
    let position = diagnostic.error().position();
    assert_eq!(
        (
            position.byte_offset().value(),
            position.line().value(),
            position.column().value()
        ),
        (10, 1, 0)
    );
    let end = diagnostic.span().end();
    assert_eq!(
        (
            end.byte_offset().value(),
            end.line().value(),
            end.column().value()
        ),
        (27, 1, 16)
    );
    let width = &report.syntax()[0];
    let position = width.position().unwrap();
    assert_eq!(
        (
            position.byte_offset().value(),
            position.line().value(),
            position.column().value()
        ),
        (29, 2, 0)
    );
    let value = width.parsed_value().unwrap();
    assert_eq!(value.span().start().byte_offset().value(), 35);
    assert_eq!(value.span().end().byte_offset().value(), 39);
    let custom = &report.syntax()[1];
    let name = custom.parsed_name().unwrap();
    assert_eq!(name.span().start().byte_offset().value(), 41);
    assert_eq!(name.span().end().byte_offset().value(), 45);
    assert_eq!(name.source().as_str(), source);
    assert!(name.source().same_snapshot(value.source()));
    assert_eq!(&source[41..45], "--é");
}

#[test]
fn retained_eof_components_report_original_implicit_closures() {
    let source = "width: 2px; --X: fn([x";
    let report = parse_declaration_list_text(source);
    assert_eq!(names(report.syntax()), ["width", "--X"]);
    assert_eq!(report.diagnostics().len(), 2);
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.span().start(), diagnostic.span().end());
    }
    assert_eq!(
        report.syntax()[1].parsed_value().unwrap().source().as_str(),
        source
    );
    assert_eq!(
        validate_declaration_list_text(source)
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn tokenizer_recovery_remains_independent_of_discarded_units() {
    let source = "width: 2px; ] /*open";
    let report = parse_declaration_list_text(source);
    assert_eq!(names(report.syntax()), ["width"]);
    let [unit, comment] = report.diagnostics() else {
        panic!("unit plus lexical recovery")
    };
    assert_eq!(unit.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(unit.error().position().byte_offset().value(), 12);
    assert_eq!(unit.span().start().byte_offset().value(), 12);
    assert_eq!(unit.span().end().byte_offset().value(), source.len());
    assert_eq!(
        comment.action(),
        CssRecoveryAction::IgnoreUnterminatedComment
    );
    assert_eq!(comment.span().start().byte_offset().value(), 14);
    assert_eq!(comment.span().end().byte_offset().value(), source.len());
    assert_eq!(
        comment.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(
        validate_declaration_list_text(source)
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn resource_recovery_keeps_its_own_action_and_allows_later_siblings() {
    for depth in [255_usize, 256] {
        let source = format!(
            "--deep: {}x{}; width: 2px",
            "f(".repeat(depth),
            ")".repeat(depth)
        );
        let report = parse_declaration_list_text(&source);
        assert!(report.is_clean(), "depth {depth}: {report:?}");
        assert_eq!(names(report.syntax()), ["--deep", "width"]);
    }
    for prefix in ["--deep: ", "] --deep: ", "@x "] {
        let unit = format!("{prefix}{}x{}", "f(".repeat(257), ")".repeat(257));
        let source = format!("{unit}; width: 2px");
        let report = parse_declaration_list_text(&source);
        assert_eq!(names(report.syntax()), ["width"]);
        let [diagnostic] = report.diagnostics() else {
            panic!("one resource diagnostic")
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            prefix.len() + 512
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), unit.len());
    }
    let source = format!("--deep: {}x", "f(".repeat(257));
    let report = parse_declaration_list_text(&source);
    assert!(report.syntax().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("resource action owns rejected EOF unit")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 520);
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
}

#[test]
fn contextual_quirks_and_svg_admission_use_the_existing_owning_grammar() {
    let source = "color: ABC; width: 2px";
    let standard = CssParserContext::default().parse_declaration_list_text(source);
    assert_eq!(names(standard.syntax()), ["width"]);
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let report = quirks.parse_declaration_list_text(source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(names(report.syntax()), ["color", "width"]);
    assert_eq!(report.syntax()[0].parser_context(), quirks);
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "color: rgb(170, 187, 204);"
    );
    let svg = quirks.with_svg_glyph_orientation_vertical();
    let report = svg.parse_declaration_list_text("glyph-orientation-vertical: 90; width: 2px");
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[0].property_name(),
        CssPropertyNameRef::SvgGlyphOrientationVertical
    );
    assert_eq!(report.syntax()[0].parser_context(), svg);
    assert_eq!(
        report.syntax()[0].to_specified_css().unwrap(),
        "glyph-orientation-vertical: 90deg;"
    );
    // Writing Modes 3 §5.1.3 admits exact 90 as the legacy Sideways mapping.
    // Explicit SVG selection retains its independent terminal instead.
    let default = CssParserContext::default();
    let report = parse_declaration_list_text("glyph-orientation-vertical: 90; width: 2px");
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(report.syntax()[0].parser_context(), default);
    let known = report.syntax()[0].known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::TextOrientation);
    let CssKnownPropertyValueRef::TextOrientation(value) = known.property_value().unwrap() else {
        panic!("legacy Writing Modes mapping")
    };
    assert_eq!(value.orientation(), &CssTextOrientation::Sideways);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Width
    );
}
