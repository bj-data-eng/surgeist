use crate::css::support::authored_syntax::property_names;
use surgeist_css::declaration::{CssDeclarationList, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::rule::CssRule;
use surgeist_css::{CssParseReport, CssParser};

fn style_rule_report(
    source: &str,
) -> (
    CssDeclarationList,
    Vec<surgeist_css::diagnostic::CssRecoveryDiagnostic>,
) {
    let wrapped = format!(".x {{ {source} }}");
    let report = CssParser::default().parse_sheet(&wrapped);
    let [CssRule::Style(rule)] = report.syntax().rules() else {
        panic!("expected one retained style rule for `{source}`");
    };
    (rule.declarations().clone(), report.diagnostics().to_vec())
}

#[test]
fn style_attribute_empty_trivia_and_optional_final_semicolon_are_clean() {
    for source in ["", " \t/**/\n", "color: red", "color: red;"] {
        let report: CssParseReport<CssDeclarationList> =
            CssParser::default().parse_block_contents(source);
        assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().is_empty(), !source.contains("color"));
    }
}

#[test]
fn style_attribute_ordinary_custom_global_substitution_and_importance_match_style_blocks() {
    let source = "width: 2px; --Theme: ready; color: inherit; height: var(--h, 8px) !important";
    let attribute = CssParser::default().parse_block_contents(source);
    let (block, block_diagnostics) = style_rule_report(source);

    assert!(attribute.is_clean());
    assert!(block_diagnostics.is_empty());
    assert_eq!(property_names(attribute.syntax()), property_names(&block));
    for (attribute, block) in attribute.syntax().iter().zip(block.iter()) {
        assert_eq!(attribute.body(), block.body());
        assert_eq!(attribute.importance(), block.importance());
        assert_eq!(
            attribute
                .position()
                .expect("parsed declaration position")
                .byte_offset()
                .value()
                + 5,
            block
                .position()
                .expect("parsed declaration position")
                .byte_offset()
                .value()
        );
    }
    assert_eq!(attribute.syntax()[3].importance(), CssImportance::Important);
}

#[test]
fn style_attribute_declaration_error_classes_match_style_blocks_modulo_wrapper_offset() {
    for (unit, code, responsible) in [
        ("mystery: 1;", CssErrorCode::UnknownProperty, 0),
        ("width: nope;", CssErrorCode::InvalidPropertyValue, 7),
        ("width: 2px !oops;", CssErrorCode::InvalidPropertyValue, 11),
        ("color: #ggg;", CssErrorCode::InvalidColorSyntax, 7),
        ("--bad name: 1px;", CssErrorCode::UnexpectedToken, 6),
        ("broken;", CssErrorCode::UnexpectedEnd, 6),
    ] {
        let source = format!("color: red; {unit} height: 3px;");
        let attribute = CssParser::default().parse_block_contents(&source);
        let (block, block_diagnostics) = style_rule_report(&source);

        assert_eq!(property_names(attribute.syntax()), ["color", "height"]);
        assert_eq!(property_names(attribute.syntax()), property_names(&block));
        let [attribute_diagnostic] = attribute.diagnostics() else {
            panic!("expected one attribute diagnostic for `{unit}`");
        };
        let [block_diagnostic] = block_diagnostics.as_slice() else {
            panic!("expected one block diagnostic for `{unit}`");
        };
        let unit_start = source.find(unit).expect("unit");
        assert_eq!(attribute_diagnostic.error().code(), code);
        assert_eq!(block_diagnostic.error().code(), code);
        assert_eq!(
            attribute_diagnostic.action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            block_diagnostic.action(),
            CssRecoveryAction::DropDeclaration
        );
        assert_eq!(
            attribute_diagnostic
                .error()
                .position()
                .byte_offset()
                .value(),
            unit_start + responsible
        );
        assert_eq!(
            block_diagnostic.error().position().byte_offset().value(),
            unit_start + responsible + 5
        );
        assert_eq!(
            attribute_diagnostic.error().position().line(),
            block_diagnostic.error().position().line()
        );
        assert_eq!(
            attribute_diagnostic.error().position().column().value() + 5,
            block_diagnostic.error().position().column().value()
        );
        assert_eq!(
            attribute_diagnostic.span().start().byte_offset().value(),
            unit_start
        );
        assert_eq!(
            attribute_diagnostic.span().end().byte_offset().value(),
            unit_start + unit.len()
        );
        assert_eq!(
            block_diagnostic.span().start().byte_offset().value(),
            unit_start + 5
        );
        assert_eq!(
            block_diagnostic.span().end().byte_offset().value(),
            unit_start + unit.len() + 5
        );
    }
}

#[test]
fn style_attribute_non_declaration_units_drop_independently_in_source_order() {
    let source = "@unknown x; color: red; .nested { width: 1px; } opacity: 1; broken; height: 2px; ,; --kept: yes;";
    let report = CssParser::default().parse_block_contents(source);

    // Syntax 3 (2026 CRD) §5.5.5: an at-rule is consumed whole; any other unit that
    // is not a declaration is reconsumed as a qualified rule with `;` as its stop
    // token (§5.5.3), so `.nested { .. }` ends at its block and `opacity: 1` that
    // follows it is a declaration of its own.
    assert_eq!(
        property_names(report.syntax()),
        ["color", "opacity", "height", "--kept"],
        "{:?}",
        report.diagnostics()
    );
    let units = [
        (
            "@unknown x;",
            CssErrorCode::UnexpectedToken,
            CssRecoveryAction::DropAtRule,
            0,
        ),
        (
            ".nested { width: 1px; }",
            CssErrorCode::UnexpectedToken,
            CssRecoveryAction::DropQualifiedRule,
            0,
        ),
        (
            "broken;",
            CssErrorCode::UnexpectedEnd,
            CssRecoveryAction::DropDeclaration,
            "broken".len(),
        ),
        (
            ",;",
            CssErrorCode::UnexpectedToken,
            CssRecoveryAction::DropDeclaration,
            0,
        ),
    ];
    assert_eq!(report.diagnostics().len(), units.len());
    for (diagnostic, (unit, code, action, responsible)) in report.diagnostics().iter().zip(units) {
        let start = source.find(unit).expect("invalid unit");
        assert_eq!(diagnostic.error().code(), code, "{unit}");
        assert_eq!(diagnostic.action(), action, "{unit}");
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            start + responsible,
            "{unit}"
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            start,
            "{unit}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len(),
            "{unit}"
        );
    }
}

#[test]
fn style_attribute_root_closer_ends_the_contents_after_a_dropped_at_rule() {
    // §5.5.5: the root `}` ends the contents, so nothing after it is read.
    let source = "@unknown screen { color: red; } } color: red; top: 1px";
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.syntax().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped at-rule: {:?}", report.diagnostics());
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        "@unknown screen { color: red; }".len()
    );
}

#[test]
fn style_attribute_stray_closers_drop_their_unit_without_hiding_later_values() {
    // A unit that starts with `)` or `]` is not a declaration; it is reconsumed as a
    // qualified rule that ends at the `;` stop token (§5.5.5, §5.5.3).
    let source = "top: 1px; ) width: 2px; left: 1px; ] height: 3px; right: 1px";
    let report = CssParser::default().parse_block_contents(source);

    assert_eq!(property_names(report.syntax()), ["top", "left", "right"]);
    let units = [") width: 2px;", "] height: 3px;"];
    assert_eq!(report.diagnostics().len(), units.len());
    for (diagnostic, unit) in report.diagnostics().iter().zip(units) {
        let start = source.find(unit).expect("invalid unit");
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(diagnostic.error().position().byte_offset().value(), start);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + unit.len()
        );
    }
}

#[test]
fn style_attribute_component_eof_closure_and_nesting_limit_match_shared_boundaries() {
    let implicit = "--value: fn([x";
    let report = CssParser::default().parse_block_contents(implicit);
    assert_eq!(property_names(report.syntax()), ["--value"]);
    assert_eq!(report.diagnostics().len(), 2);
    for diagnostic in report.diagnostics() {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            implicit.len()
        );
        assert_eq!(diagnostic.span().start(), diagnostic.span().end());
    }

    let malformed = "background-image: url(bad url";
    let report = CssParser::default().parse_block_contents(malformed);
    assert!(report.syntax().is_empty());
    let [diagnostic] = report.diagnostics() else {
        panic!("malformed URL must drop one declaration");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        malformed.len()
    );

    for depth in [255_usize, 256] {
        let source = format!("--deep: {}x{}", "f(".repeat(depth), ")".repeat(depth));
        let report = CssParser::default().parse_block_contents(&source);
        assert_eq!(property_names(report.syntax()), ["--deep"], "depth {depth}");
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
    }

    let depth = 257_usize;
    let source = format!("--deep: {}x{}", "f(".repeat(depth), ")".repeat(depth));
    let first_over_limit = source
        .match_indices("f(")
        .nth(256)
        .expect("257th function")
        .0;
    let report = CssParser::default().parse_block_contents(&source);
    // Block contents charges the component limits for the whole input.
    assert!(report.syntax().is_empty());
    crate::css::support::parse_report::assert_contents_refused(&report, &source, "depth 257");
    let [diagnostic] = report.diagnostics() else {
        panic!("over-limit contents must have one diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidComponentValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        first_over_limit
    );
}
