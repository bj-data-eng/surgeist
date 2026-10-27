//! Independent raw-list oracles: CSS Syntax 3 (2026 CRD) §5.5.5 "consume a
//! block's contents", §5.5.3 "consume a qualified rule" and §5.5.6 "consume a
//! declaration", followed by the ordinary property consumer contract
//! (#1353 decision 1, #1396). The 2021 "parse a list of declarations" is
//! removed; a raw list is block contents.
//! <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents>
//!
//! A unit that is not a declaration is reparsed as a qualified rule with `;`
//! as the stop token: it ends at its `{}` block, or before a root `;`, `}` or
//! EOF. A root `}` ends the contents. An at-rule ends after its block or its
//! `;`.
use crate::css::support::authored_syntax::property_names as names;
use surgeist_css::declaration::CssImportance;
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};
use surgeist_css::values::text::CssTextOrientation;
use surgeist_css::{CssParser, CssParserContext, CssParserMode};

/// A unit dropped without reaching a `{}` block is a dropped declaration.
fn dropped(source: &str, expected_names: &[&str], unit: &str, responsible: usize) {
    dropped_by(
        source,
        expected_names,
        unit,
        responsible,
        CssRecoveryAction::DropDeclaration,
    );
}

/// A unit that ended at its `{}` block is a dropped qualified rule.
fn dropped_rule(source: &str, expected_names: &[&str], unit: &str, responsible: usize) {
    dropped_by(
        source,
        expected_names,
        unit,
        responsible,
        CssRecoveryAction::DropQualifiedRule,
    );
}

fn dropped_by(
    source: &str,
    expected_names: &[&str],
    unit: &str,
    responsible: usize,
    action: CssRecoveryAction,
) {
    let report = CssParser::default().parse_block_contents(source);
    assert_eq!(
        names(report.syntax()),
        expected_names,
        "{source:?}: {report:?}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid unit: {report:?}")
    };
    let start = source.find(unit).unwrap();
    assert_eq!(diagnostic.action(), action);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        start + responsible
    );
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    // A dropped declaration's span includes the `;` that terminates it, as in
    // every declaration list; a rule that ended at its block has no such stop,
    // and an at-rule consumes its own `;`.
    let stop = usize::from(
        action == CssRecoveryAction::DropDeclaration
            && source[start + unit.len()..].starts_with(';'),
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + unit.len() + stop
    );
    assert!(!report.is_clean());
    assert_eq!(
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn empty_slots_and_optional_final_semicolon_are_clean() {
    for source in ["", " \t/**/\r\n", "; ;/**/;", "width: 2px", "width: 2px;"] {
        let report = CssParser::default().parse_block_contents(source);
        assert!(report.is_clean(), "{source:?}: {report:?}");
        assert_eq!(report.syntax().is_empty(), !source.contains("width"));
        assert_eq!(
            &CssParser::default()
                .parse_block_contents(source)
                .into_validation_result()
                .unwrap(),
            report.syntax()
        );
    }
}

#[test]
fn a_root_close_brace_ends_the_contents_and_discards_the_rest() {
    for (source, expected) in [
        ("} color: red; width: 2px;", &[][..]),
        ("height: 1px; } color: red; width: 2px;", &["height"][..]),
        ("color: red } width: 1px", &["color"][..]),
    ] {
        let report = CssParser::default().parse_block_contents(source);
        assert_eq!(names(report.syntax()), expected, "{source:?}: {report:?}");
        assert!(report.is_clean(), "{source:?}: {report:?}");
    }
}

#[test]
fn a_stray_close_bracket_or_parenthesis_starts_a_dropped_unit() {
    for closer in ["]", ")"] {
        let unit = format!("{closer} color: red");
        let source = format!("{unit}; width: 2px;");
        dropped(&source, &["width"], &unit, 0);
        let raw = CssParser::default().parse_block_contents(&source);
        assert_eq!(
            raw.diagnostics()[0].error().code(),
            CssErrorCode::UnexpectedToken
        );
    }
}

#[test]
fn invalid_started_units_consume_nested_semicolons_comments_and_strings() {
    // The prelude `] --X:` is followed by a `{}` block, which ends the rule.
    let source = "] --X: {a:b;c:d}; width: 2px;";
    dropped_rule(source, &["width"], "] --X: {a:b;c:d}", 0);
    for unit in [
        ") --X: fn([a;b], {c:d;e:f})",
        ", --X: \";})]\" /* ;})] */ data",
        "<!-- color: red",
        "--> color: red",
    ] {
        let source = format!("{unit}; width: 2px;");
        dropped(&source, &["width"], unit, 0);
    }
}

#[test]
fn a_qualified_rule_ends_at_its_block_and_the_following_declaration_is_kept() {
    // §5.5.3: the rule ends at its `{}` block; what follows is a fresh item.
    dropped_rule(
        "a { color: red; } height: 3px; width: 2px;",
        &["height", "width"],
        "a { color: red; }",
        2,
    );
    dropped_rule(
        ".bad { color: red; } color: blue; width: 2px;",
        &["color", "width"],
        ".bad { color: red; }",
        0,
    );
    // The inner declaration of a dropped rule is never admitted.
    let report = CssParser::default().parse_block_contents("a { height: 3px; } width: 2px;");
    assert_eq!(names(report.syntax()), ["width"]);
}

#[test]
fn an_ident_without_a_colon_is_a_rule_that_the_stop_semicolon_ends() {
    dropped(
        "broken color: red; width: 2px;",
        &["width"],
        "broken color: red",
        7,
    );
}

#[test]
fn generic_at_rules_end_after_their_block_or_semicolon() {
    for unit in [
        "@media fn([a;b]) { color: red; .x { height: 3px; } }",
        "@font-face { font-family: Test; src: url(test); }",
        "@unknown \";{}\" /* ;{} */ { --X: {a:b;c:d}; }",
    ] {
        let source = format!("{unit}; width: 2px;");
        dropped_by(&source, &["width"], unit, 0, CssRecoveryAction::DropAtRule);
        assert_eq!(
            CssParser::default()
                .parse_block_contents(&source)
                .diagnostics()[0]
                .error()
                .code(),
            CssErrorCode::UnexpectedToken
        );
    }
    dropped_by(
        "@unknown { color: red; } width: 2px;",
        &["width"],
        "@unknown { color: red; }",
        0,
        CssRecoveryAction::DropAtRule,
    );
    // §5.5.2: an at-rule without a block ends by consuming its `;`, so the
    // dropped unit includes that semicolon.
    dropped_by(
        "@unknown x; width: 2px;",
        &["width"],
        "@unknown x;",
        0,
        CssRecoveryAction::DropAtRule,
    );
}

#[test]
fn duplicate_known_and_case_sensitive_custom_occurrences_remain_in_order() {
    let source = "WIDTH: 2px !IMPORTANT; width: 3px; --Case: A/**/ B; --case: C; --Case: var(--x) !/**/important; color: inherit";
    let report = CssParser::default().parse_block_contents(source);
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
    let report = CssParser::default().parse_block_contents(&source);
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
        ("broken", CssErrorCode::UnexpectedEnd, 6),
    ] {
        let source = format!("{unit}; height: 2px!important;");
        // A failed custom property is consumed whole (§5.5.3 note); any other
        // failed declaration is reconsumed as a rule that ends at the stop
        // token without reaching a block. Both drop a declaration.
        dropped(&source, &["height"], unit, responsible);
        let report = CssParser::default().parse_block_contents(&source);
        assert_eq!(
            report.diagnostics()[0].error().code(),
            code,
            "{unit}: {report:?}"
        );
        assert_eq!(report.syntax()[0].importance(), CssImportance::Important);
    }
}

#[test]
fn a_custom_property_value_ends_at_a_nested_close_brace_which_ends_the_contents() {
    // §5.5.6 step 5: the component values of a nested declaration end at `}`.
    let report = CssParser::default().parse_block_contents("--x: a}; height: 2px!important;");
    assert_eq!(names(report.syntax()), ["--x"]);
    assert!(report.is_clean(), "{report:?}");
}

#[test]
fn mixed_failures_report_one_ordered_diagnostic_per_whole_unit() {
    let source = "width: red; @x {} height: 2px; mystery: 1; .y {} --ok: yes; } color: red";
    let report = CssParser::default().parse_block_contents(source);
    assert_eq!(names(report.syntax()), ["height", "--ok"]);
    let units = [
        (
            "width: red",
            CssErrorCode::InvalidPropertyValue,
            7,
            CssRecoveryAction::DropDeclaration,
        ),
        (
            "@x {}",
            CssErrorCode::UnexpectedToken,
            0,
            CssRecoveryAction::DropAtRule,
        ),
        (
            "mystery: 1",
            CssErrorCode::UnknownProperty,
            0,
            CssRecoveryAction::DropDeclaration,
        ),
        (
            ".y {}",
            CssErrorCode::UnexpectedToken,
            0,
            CssRecoveryAction::DropQualifiedRule,
        ),
    ];
    assert_eq!(report.diagnostics().len(), units.len());
    for (diagnostic, (unit, code, responsible, action)) in report.diagnostics().iter().zip(units) {
        let start = source.find(unit).unwrap();
        assert_eq!(diagnostic.error().code(), code);
        assert_eq!(diagnostic.action(), action);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            start + responsible
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        let stop = usize::from(
            action == CssRecoveryAction::DropDeclaration
                && source[start + unit.len()..].starts_with(';'),
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len() + stop
        );
    }
    assert_eq!(
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn invalid_eof_units_are_nonclean_empty_lists_with_actual_source_end() {
    dropped("] --X: fn([a;b", &[], "] --X: fn([a;b", 0);
    dropped_rule(".x {color:red}", &[], ".x {color:red}", 0);
    for unit in ["@unknown { color: red", "@unknown x"] {
        dropped_by(unit, &[], unit, 0, CssRecoveryAction::DropAtRule);
    }
    dropped("broken", &[], "broken", 6);
    let report = CssParser::default().parse_block_contents("} color: red");
    assert!(
        report.syntax().is_empty() && report.is_clean(),
        "{report:?}"
    );
}

#[test]
fn exact_unicode_multiline_spans_and_retained_origins_use_unwrapped_input() {
    let source = "/*😀*/\r\n] --é: {a:b;c:d};\nwidth: 2px;\n--é:blue";
    let report = CssParser::default().parse_block_contents(source);
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
    let report = CssParser::default().parse_block_contents(source);
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
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn tokenizer_recovery_remains_independent_of_discarded_units() {
    let source = "width: 2px; ] /*open";
    let report = CssParser::default().parse_block_contents(source);
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
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn nesting_beyond_the_ceiling_refuses_the_whole_contents_with_one_typed_diagnostic() {
    // The 256-level ceiling is an implementation limit, not Syntax 3. Block
    // contents charges every original component once against the component
    // limits, so a refusal covers the whole input and nothing is retained.
    for depth in [255_usize, 256] {
        let source = format!(
            "--deep: {}x{}; width: 2px",
            "f(".repeat(depth),
            ")".repeat(depth)
        );
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "depth {depth}: {report:?}");
        assert_eq!(names(report.syntax()), ["--deep", "width"]);
    }
    for source in [
        format!(
            "--deep: {}x{}; width: 2px",
            "f(".repeat(257),
            ")".repeat(257)
        ),
        format!(
            "] --deep: {}x{}; width: 2px",
            "f(".repeat(257),
            ")".repeat(257)
        ),
        format!("@x {}x{}; width: 2px", "f(".repeat(257), ")".repeat(257)),
        format!("--deep: {}x", "f(".repeat(257)),
    ] {
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.syntax().is_empty(), "{report:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one resource diagnostic: {report:?}")
        };
        assert!(diagnostic.error().is_resource_refusal(), "{diagnostic:?}");
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn contextual_quirks_and_svg_admission_use_the_owning_grammar() {
    let source = "color: ABC; width: 2px";
    let standard = CssParser::default().parse_block_contents(source);
    assert_eq!(names(standard.syntax()), ["width"]);
    let quirks = CssParserContext::new(CssParserMode::Quirks);
    let report = CssParser::new(quirks, Default::default()).parse_block_contents(source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(names(report.syntax()), ["color", "width"]);
    assert_eq!(report.syntax()[0].parser_context(), quirks);
    assert_eq!(
        report.syntax()[0]
            .serialize_specified(Default::default())
            .unwrap(),
        "color: rgb(170, 187, 204);"
    );
    let svg = quirks.with_svg_glyph_orientation_vertical();
    let report = CssParser::new(svg, Default::default())
        .parse_block_contents("glyph-orientation-vertical: 90; width: 2px");
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(
        report.syntax()[0].property_name(),
        CssPropertyNameRef::SvgGlyphOrientationVertical
    );
    assert_eq!(report.syntax()[0].parser_context(), svg);
    assert_eq!(
        report.syntax()[0]
            .serialize_specified(Default::default())
            .unwrap(),
        "glyph-orientation-vertical: 90deg;"
    );
    // Writing Modes 3 §5.1.3 admits exact 90 as the legacy Sideways mapping.
    // Explicit SVG selection retains its independent terminal instead.
    let default = CssParserContext::default();
    let report =
        CssParser::default().parse_block_contents("glyph-orientation-vertical: 90; width: 2px");
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().len(), 2);
    assert_eq!(report.syntax()[0].parser_context(), default);
    let known = report.syntax()[0].known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::TextOrientation);
    let CssKnownPropertyValueRef::TextOrientation(value) = known.property_value().unwrap() else {
        panic!("legacy Writing Modes mapping")
    };
    assert_eq!(value.value(), &CssTextOrientation::Sideways);
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Width
    );
}
