#![forbid(unsafe_code)]
//! Conditional 5 defines authored named supports conditions and a bare named
//! reference in `@import supports()`. Their test bodies have no rendering effect.
//! Conditional 3 and Nesting 1 permit definitions in ordinary rule-list contexts.
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#at-ruledef-supports-condition
//! https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#use
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#conditionals

use surgeist_css::{
    CssErrorCode, CssNormalizedDeclaration, CssNormalizedItem, CssNormalizedSheet,
    CssRecoveryAction, CssRule, CssRuleContext, CssRuleContextKindRef, normalize_sheet,
    parse_sheet, validate_sheet,
};

fn rule_at(sheet: &CssNormalizedSheet, offset: usize) -> &CssRuleContext {
    sheet
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(rule)
                if rule
                    .position()
                    .is_some_and(|position| position.byte_offset().value() == offset) =>
            {
                Some(rule)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no retained rule at byte {offset}"))
}

fn declarations(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(declaration) => Some(declaration),
            _ => None,
        })
        .collect()
}

#[test]
fn empty_declaration_only_and_unknown_test_bodies_remain_clean_nonstyle_rules() {
    let source = concat!(
        "@supports-condition --empty{}",
        "@supports-condition --terminal{color:red}",
        "@supports-condition --future{mystery:future(value);&{color:blue}}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert!(validate_sheet(source).is_ok());
    assert_eq!(report.syntax().rules().len(), 3);

    let normalized = normalize_sheet(report.syntax()).unwrap();
    let offsets: Vec<_> = source
        .match_indices("@supports-condition")
        .map(|(offset, _)| offset)
        .collect();
    assert_eq!(normalized.items().len(), offsets.len());
    for offset in offsets {
        let rule = rule_at(&normalized, offset);
        assert!(rule.parent().is_none());
        assert!(!matches!(
            rule.kind(),
            CssRuleContextKindRef::Style(_)
                | CssRuleContextKindRef::ScopedStyle(_)
                | CssRuleContextKindRef::NestedDeclarations(_)
        ));
    }
    assert!(declarations(&normalized).is_empty());
}

#[test]
fn bare_named_import_operand_is_a_supports_clause_without_media() {
    let report = parse_sheet("@import url(theme.css) supports(--theme);");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("one retained import: {report:?}");
    };
    assert!(
        import.supports().is_some(),
        "bare name is a supports clause"
    );
    assert!(import.media().is_none(), "bare name is not a media suffix");
}

#[test]
fn definitions_remain_inside_each_supported_rule_list() {
    for (prefix, suffix) in [
        ("@media all{", "}"),
        ("@supports(display:grid){", "}"),
        ("@container (width > 1px){", "}"),
        ("@layer theme{", "}"),
        ("@scope (.host){", "}"),
        (".host{@supports(display:grid){", "}}"),
    ] {
        let source = format!("{prefix}@supports-condition --probe{{}}{suffix}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let definition = rule_at(&normalized, prefix.len());
        let parent = definition
            .parent()
            .expect("definition retains its parent rule");
        let parent_offset = if prefix.starts_with(".host{") {
            ".host{".len()
        } else {
            0
        };
        assert_eq!(
            parent.position().unwrap().byte_offset().value(),
            parent_offset,
            "{source}"
        );
        assert!(declarations(&normalized).is_empty(), "{source}");
    }
}

#[test]
fn definitions_preserve_import_and_namespace_prelude_admission() {
    let source = concat!(
        "@supports-condition --first{}",
        "@import url(first.css);",
        "@supports-condition --between{}",
        "@import url(second.css);",
        "@namespace n \"urn:n\";",
        "@supports-condition --last{}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().rules().len(), 6);
    assert!(matches!(report.syntax().rules()[1], CssRule::Import(_)));
    assert!(matches!(report.syntax().rules()[3], CssRule::Import(_)));
    assert!(matches!(report.syntax().rules()[4], CssRule::Namespace(_)));
    let normalized = normalize_sheet(report.syntax()).unwrap();
    for (offset, _) in source.match_indices("@supports-condition") {
        assert!(rule_at(&normalized, offset).parent().is_none());
    }

    let source = concat!(
        "@namespace n \"urn:n\";",
        "@supports-condition --after-namespace{}",
        "@import url(late.css);",
    );
    let report = parse_sheet(source);
    let [diagnostic] = report.diagnostics() else {
        panic!("import phase stays closed after namespace: {report:?}");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    assert_eq!(report.syntax().rules().len(), 2);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert!(
        rule_at(&normalized, source.find("@supports-condition").unwrap())
            .parent()
            .is_none()
    );

    let source = concat!(
        "@namespace n \"urn:n\";",
        ".body{}",
        "@supports-condition --after-body{}",
        "@import url(late.css);",
        "@namespace late \"urn:late\";",
    );
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 2, "{report:?}");
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.action() == CssRecoveryAction::DropAtRule
            && diagnostic.error().code() == CssErrorCode::InvalidAtRulePlacement
    }));
    assert_eq!(report.syntax().rules().len(), 3);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    assert!(
        rule_at(&normalized, source.find("@supports-condition").unwrap())
            .parent()
            .is_none()
    );
}

#[test]
fn style_nested_definition_is_terminal_and_preserves_only_surrounding_declarations() {
    let source = concat!(
        ".host{color:red;",
        "@supports-condition --probe{color:blue;mystery:future(value);&{color:blue}}",
        "color:green;}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let outer = rule_at(&normalized, 0);
    let definition = rule_at(&normalized, source.find("@supports-condition").unwrap());
    assert!(definition.parent().unwrap().same_context(outer));

    let values = declarations(&normalized);
    assert_eq!(values.len(), 2, "test-body declarations do not contribute");
    for (declaration, color) in values.iter().zip(["red", "green"]) {
        assert_eq!(
            declaration
                .source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            color
        );
    }
    assert_eq!(values[0].order(), 0);
    assert_eq!(values[1].order(), 1);
    assert_eq!(
        values[0].source().position().unwrap().byte_offset().value(),
        source.find("color:red").unwrap()
    );
    assert_eq!(
        values[1].source().position().unwrap().byte_offset().value(),
        source.rfind("color:green").unwrap()
    );
}
