//! Block partition follows the selected immutable Syntax 3 consume-block-contents
//! and consume-qualified-rule algorithms (including the custom-property exception):
//! https://github.com/w3c/csswg-drafts/blob/f971255463f01fb740e2a3a7ecfe83e319cddab9/css-syntax-3/Overview.bs
//! Terminal runs are retained under the catalog's narrow terminal-retention
//! reconciliation with CSS Nesting 1 sections 3.3 and 5:
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/
//! Rejected complete qualified rules and at-keywords partition nonempty runs;
//! no-rule outcomes do not. An empty run never creates a placeholder.

use surgeist_css::{
    CssDeclaration, CssKnownProperty, CssNormalizedItem, CssParseReport, CssRecoveryAction,
    CssRule, CssRuleContextKindRef, CssScopedRule, CssSheet, normalize_sheet, parse_sheet,
};

fn declaration(value: &CssDeclaration, property: CssKnownProperty, css: &str) {
    assert_eq!(value.known().unwrap().property(), property);
    assert_eq!(
        value
            .value_components()
            .serialize()
            .unwrap()
            .as_css()
            .trim(),
        css
    );
}

fn color(value: &CssDeclaration, css: &str) {
    declaration(value, CssKnownProperty::Color, css);
}

fn recovery(
    source: &str,
    report: &CssParseReport<CssSheet>,
    failed: Option<(&str, CssRecoveryAction)>,
    closures: usize,
) {
    let dropped: Vec<_> = report
        .diagnostics()
        .iter()
        .filter(|d| d.action() != CssRecoveryAction::RetainWithImplicitClosure)
        .collect();
    if let Some((unit, action)) = failed {
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].action(), action);
        let start = source.find(unit).unwrap();
        assert_eq!(dropped[0].span().start().byte_offset().value(), start);
        assert_eq!(
            dropped[0].span().end().byte_offset().value(),
            start + unit.len()
        );
    } else {
        assert!(dropped.is_empty());
    }
    let retained: Vec<_> = report
        .diagnostics()
        .iter()
        .filter(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .collect();
    assert_eq!(retained.len(), closures);
    for diagnostic in retained {
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

fn split(source: &str, failed: &str, action: CssRecoveryAction, closures: usize) {
    let report = parse_sheet(source);
    recovery(source, &report, Some((failed, action)), closures);
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    assert_eq!(parent.declarations().len(), 1);
    color(&parent.declarations()[0], "red");
    let [CssRule::NestedDeclarations(run)] = parent.rules() else {
        panic!("separate trailing run")
    };
    assert_eq!(run.declarations().len(), 1);
    color(&run.declarations()[0], "green");
    assert_eq!(
        run.position().byte_offset().value(),
        source.find("color:green").unwrap()
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 2);
    assert!(
        values[0]
            .source()
            .same_occurrence(&parent.declarations()[0])
    );
    assert!(values[1].source().same_occurrence(&run.declarations()[0]));
    assert_eq!((values[0].order(), values[1].order()), (0, 1));
    assert!(matches!(
        values[0].rule_context().kind(),
        CssRuleContextKindRef::Style(_)
    ));
    assert!(matches!(
        values[1].rule_context().kind(),
        CssRuleContextKindRef::NestedDeclarations(_)
    ));
    assert!(
        values[1]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(values[0].rule_context())
    );
    assert!(
        values[0]
            .selector_context()
            .same_context(values[1].selector_context())
    );
}

#[test]
fn rejected_complete_qualified_rule_partitions_before_brace_and_eof() {
    split(
        ".p{color:red;.bad,{}color:green}",
        ".bad,{}",
        CssRecoveryAction::DropQualifiedRule,
        0,
    );
    split(
        ".p{color:red;.bad,{}color:green",
        ".bad,{}",
        CssRecoveryAction::DropQualifiedRule,
        1,
    );
}

#[test]
fn rejected_at_rule_partitions_for_statement_and_block() {
    split(
        ".p{color:red;@unknown; color:green}",
        "@unknown;",
        CssRecoveryAction::DropAtRule,
        0,
    );
    split(
        ".p{color:red;@unknown{}color:green}",
        "@unknown{}",
        CssRecoveryAction::DropAtRule,
        0,
    );
}

#[test]
fn rejected_first_rule_without_pending_declarations_leaves_leading_slot_available() {
    for source in [".p{.bad,{}color:green}", ".p{@unknown; color:green}"] {
        let report = parse_sheet(source);
        assert!(!report.is_clean());
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("parent")
        };
        assert_eq!(parent.declarations().len(), 1);
        color(&parent.declarations()[0], "green");
        assert!(parent.rules().is_empty());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let values: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(values.len(), 1);
        assert!(matches!(
            values[0].rule_context().kind(),
            CssRuleContextKindRef::Style(_)
        ));
    }
}

#[test]
fn no_rule_semicolon_and_custom_property_lookalike_preserve_one_run() {
    // Malformed var() rejects the custom declaration. Its fallback prelude starts
    // with a decoded custom ident and colon; bad-declaration remnants through the
    // semicolon return nothing, even with blocks in those remnants.
    for source in [
        ".p{color:red;.bad; color:green}",
        ".p{color:red;.bad; color:green",
        ".p{color:red;--bad:var() {opacity:0} .leak{};color:green}",
    ] {
        let report = parse_sheet(source);
        assert!(!report.is_clean());
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("parent")
        };
        assert_eq!(parent.declarations().len(), 2, "{source}");
        color(&parent.declarations()[0], "red");
        color(&parent.declarations()[1], "green");
        assert!(parent.rules().is_empty(), "{source}");
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let values: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(values.len(), 2);
        assert!(
            values[0]
                .rule_context()
                .same_context(values[1].rule_context())
        );
        assert!(
            values[0]
                .selector_context()
                .same_context(values[1].selector_context())
        );
    }
}

#[test]
fn rejected_rule_preserves_distinct_adjacent_declaration_runs() {
    let source = ".p{.ok{}color:red;.bad,{}color:green}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    assert!(parent.declarations().is_empty());
    let [
        CssRule::Style(child),
        CssRule::NestedDeclarations(red),
        CssRule::NestedDeclarations(green),
    ] = parent.rules()
    else {
        panic!("child and two runs")
    };
    assert!(child.declarations().is_empty());
    assert_eq!(
        (red.declarations().len(), green.declarations().len()),
        (1, 1)
    );
    color(&red.declarations()[0], "red");
    color(&green.declarations()[0], "green");
    assert_eq!(
        red.position().byte_offset().value(),
        source.find("color:red").unwrap()
    );
    assert_eq!(
        green.position().byte_offset().value(),
        source.find("color:green").unwrap()
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 2);
    assert!(
        !values[0]
            .rule_context()
            .same_context(values[1].rule_context())
    );
    assert!(
        values[0]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(values[1].rule_context().parent().unwrap())
    );
    assert!(
        values[0]
            .selector_context()
            .same_context(values[1].selector_context())
    );
}

#[test]
fn malformed_neighbor_keeps_original_declarations_in_distinct_normalized_contexts() {
    // Exact existing structured_rules input; corrected partition, no deleted coverage.
    let source = ".card { color: red; .bad, { color: blue; } color: green; .child { opacity: 1; } color: black; }";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    assert_eq!(parent.declarations().len(), 1);
    let [
        CssRule::NestedDeclarations(green),
        CssRule::Style(child),
        CssRule::NestedDeclarations(black),
    ] = parent.rules()
    else {
        panic!("green, child, black")
    };
    assert_eq!(
        (
            green.declarations().len(),
            child.declarations().len(),
            black.declarations().len()
        ),
        (1, 1, 1)
    );
    color(&parent.declarations()[0], "red");
    color(&green.declarations()[0], "green");
    declaration(&child.declarations()[0], CssKnownProperty::Opacity, "1");
    color(&black.declarations()[0], "black");
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (index, (value, original)) in values
        .iter()
        .zip([
            &parent.declarations()[0],
            &green.declarations()[0],
            &child.declarations()[0],
            &black.declarations()[0],
        ])
        .enumerate()
    {
        assert_eq!(value.order(), index);
        assert!(value.source().same_occurrence(original));
    }
    for index in [1, 3] {
        assert!(matches!(
            values[index].rule_context().kind(),
            CssRuleContextKindRef::NestedDeclarations(_)
        ));
        assert!(
            values[index]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(values[0].rule_context())
        );
        assert!(
            values[index]
                .selector_context()
                .same_context(values[0].selector_context())
        );
    }
    assert!(
        !values[1]
            .rule_context()
            .same_context(values[3].rule_context())
    );
    assert!(matches!(
        values[2].rule_context().kind(),
        CssRuleContextKindRef::Style(_)
    ));
    assert!(
        values[2]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(values[0].rule_context())
    );
}

#[test]
fn nested_media_partitions_rejected_children_and_retains_terminal_run() {
    for source in [
        ".p{@media all{color:red;.bad,{}color:green}}",
        ".p{@media all{color:red;.bad,{}color:green",
    ] {
        let report = parse_sheet(source);
        recovery(
            source,
            &report,
            Some((".bad,{}", CssRecoveryAction::DropQualifiedRule)),
            if source.ends_with('}') { 0 } else { 2 },
        );
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("parent")
        };
        assert!(parent.declarations().is_empty());
        let [CssRule::Media(media)] = parent.rules() else {
            panic!("media")
        };
        let [
            CssRule::NestedDeclarations(red),
            CssRule::NestedDeclarations(green),
        ] = media.rules()
        else {
            panic!("two media runs")
        };
        assert_eq!(
            (red.declarations().len(), green.declarations().len()),
            (1, 1)
        );
        color(&red.declarations()[0], "red");
        color(&green.declarations()[0], "green");
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let values: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(values.len(), 2);
        assert!(
            !values[0]
                .rule_context()
                .same_context(values[1].rule_context())
        );
        let group = values[0].rule_context().parent().unwrap();
        assert!(matches!(group.kind(), CssRuleContextKindRef::Media(_)));
        assert!(group.same_context(values[1].rule_context().parent().unwrap()));
    }
}

#[test]
fn scoped_style_partition_survives_structural_chunk_boundary() {
    // 63/64 scope wrappers place the style body on either side of the bounded
    // structural parser split. Direct scope declarations are a separate contract.
    for depth in [1, 63, 64] {
        let source = format!(
            "{}.p{{color:red;.bad,{{}}color:green}}{}",
            "@scope{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1);
        let [CssRule::Scope(scope)] = report.syntax().rules() else {
            panic!("outer scope")
        };
        let mut rules = scope.rules().rules();
        for _ in 1..depth {
            let [CssScopedRule::Scope(scope)] = rules else {
                panic!("scope chain")
            };
            rules = scope.rules().rules();
        }
        let [CssScopedRule::Style(parent)] = rules else {
            panic!("scoped style")
        };
        assert_eq!(parent.declarations().len(), 1);
        color(&parent.declarations()[0], "red");
        let [CssRule::NestedDeclarations(green)] = parent.rules() else {
            panic!("scoped style trailing run")
        };
        assert_eq!(green.declarations().len(), 1);
        color(&green.declarations()[0], "green");
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let values: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(values.len(), 2);
        assert!(matches!(
            values[0].rule_context().kind(),
            CssRuleContextKindRef::ScopedStyle(_)
        ));
        assert!(
            values[1]
                .rule_context()
                .parent()
                .unwrap()
                .same_context(values[0].rule_context())
        );
        assert!(
            values[0]
                .selector_context()
                .same_context(values[1].selector_context())
        );
    }
}

#[test]
fn declaration_only_and_valid_child_terminal_controls_do_not_duplicate_runs() {
    for source in [".p{color:red}", ".p{color:red", ".p{color:red;"] {
        let report = parse_sheet(source);
        recovery(
            source,
            &report,
            None,
            if source.ends_with('}') { 0 } else { 1 },
        );
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("parent")
        };
        assert_eq!(parent.declarations().len(), 1);
        color(&parent.declarations()[0], "red");
        assert!(parent.rules().is_empty());
    }
    for source in [
        ".p{.ok{}color:green}",
        ".p{.ok{}color:green",
        ".p{.ok{}color:green;",
    ] {
        let report = parse_sheet(source);
        recovery(
            source,
            &report,
            None,
            if source.ends_with('}') { 0 } else { 1 },
        );
        let [CssRule::Style(parent)] = report.syntax().rules() else {
            panic!("parent")
        };
        assert!(parent.declarations().is_empty());
        let [CssRule::Style(child), CssRule::NestedDeclarations(green)] = parent.rules() else {
            panic!("child and terminal run")
        };
        assert!(child.declarations().is_empty());
        assert_eq!(green.declarations().len(), 1);
        color(&green.declarations()[0], "green");
        assert_eq!(
            green.position().byte_offset().value(),
            source.find("color:green").unwrap()
        );
    }
}
