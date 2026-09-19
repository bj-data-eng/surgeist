#![forbid(unsafe_code)]
//! CSS2 page payloads inside ordinary conditional bodies remain admitted even
//! when the condition is nested inside a scope. Direct scope children use the
//! selected Syntax3/Cascade6 content-category interpretation, not an explicit
//! page prohibition. Nesting1 keeps the style-ancestor restriction.
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/page.html#page-box
//! https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-syntax
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting-other-at-rules
//! https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#layer-block
//! https://www.w3.org/TR/2025/WD-css-conditional-5-20251030/#container-rule
use surgeist_css::*;

fn contexts(sheet: &CssNormalizedSheet) -> Vec<&CssRuleContext> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context) => Some(context),
            _ => None,
        })
        .collect()
}

fn assert_admitted(group: &str, expected_diagnostics: usize) {
    let bad = if expected_diagnostics == 0 {
        ""
    } else {
        "color:red;"
    };
    let source =
        format!("@scope (.root){{{group}{{before{{}}@page :left{{{bad}margin:1px}}after{{}}}}}}");
    let report = parse_sheet(&source);
    assert_eq!(
        report.diagnostics().len(),
        expected_diagnostics,
        "{:?}",
        report.diagnostics()
    );
    for diagnostic in report.diagnostics() {
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find("color:").unwrap()
        );
    }
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let rules = contexts(&normalized);
    assert_eq!(rules.len(), 5);
    assert!(matches!(
        rules[0].kind(),
        CssRuleContextKindRef::Scope { .. }
    ));
    match group.split_whitespace().next().unwrap() {
        "@media" => assert!(matches!(rules[1].kind(), CssRuleContextKindRef::Media(_))),
        "@supports" => assert!(matches!(
            rules[1].kind(),
            CssRuleContextKindRef::Supports(_)
        )),
        "@layer" => assert!(matches!(
            rules[1].kind(),
            CssRuleContextKindRef::LayerBlock(_)
        )),
        "@container" => assert!(matches!(
            rules[1].kind(),
            CssRuleContextKindRef::Container { .. }
        )),
        _ => panic!("known group fixture"),
    }
    assert!(rules[1].parent().unwrap().same_context(rules[0]));
    for rule in &rules[2..] {
        assert!(rule.parent().unwrap().same_context(rules[1]));
    }
    let CssRuleContextKindRef::Page(page) = rules[3].kind() else {
        panic!("retained page payload")
    };
    assert_eq!(page.selector(), Some(CssPageSelector::Left));
    assert_eq!(
        page.position().byte_offset().value(),
        source.find("@page").unwrap()
    );
    assert_eq!(page.declarations().len(), 1);
    assert_eq!(
        page.declarations().as_slice()[0]
            .known()
            .unwrap()
            .property()
            .canonical_name(),
        "margin"
    );
    for (index, marker) in [(2, "before{}"), (4, "after{}")] {
        assert!(matches!(
            rules[index].kind(),
            CssRuleContextKindRef::ScopedStyle(_)
        ));
        assert_eq!(
            rules[index].position().unwrap().byte_offset().value(),
            source.find(marker).unwrap()
        );
    }
}

#[test]
fn scope_media_body_retains_page_payload_and_order() {
    assert_admitted("@media print", 0);
}

#[test]
fn scope_supports_body_retains_page_payload_and_order() {
    assert_admitted("@supports (display:grid)", 0);
}

#[test]
fn scope_layer_stylesheet_body_retains_page_payload_and_order() {
    assert_admitted("@layer audit", 0);
}

#[test]
fn scope_container_conditional_body_retains_page_payload_and_order() {
    assert_admitted("@container (width > 1px)", 0);
}

#[test]
fn scope_conditional_page_uses_local_page_declaration_recovery() {
    assert_admitted("@media print", 1);
}

#[test]
fn immediate_scope_body_keeps_its_own_admission_through_group_ancestors() {
    for (prefix, suffix) in [
        ("@scope (.root){", "}"),
        ("@media print{@scope (.root){", "}}"),
        ("@supports(display:grid){@scope (.root){", "}}"),
        ("@layer audit{@scope (.root){", "}}"),
        ("@container(width>1px){@scope (.root){", "}}"),
        ("@scope (.outer){@scope (.root){", "}}"),
        ("@scope (.outer){@layer audit{@scope (.root){", "}}}"),
        (
            "@scope (.outer){@container(width>1px){@scope (.root){",
            "}}}",
        ),
    ] {
        let source = format!("{prefix}before{{}}@page :left{{margin:1px}}after{{}}{suffix}");
        let report = parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let rules = contexts(&normalized);
        assert!(
            !rules
                .iter()
                .any(|r| matches!(r.kind(), CssRuleContextKindRef::Page(_)))
        );
        let styles: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.kind(), CssRuleContextKindRef::ScopedStyle(_)))
            .collect();
        assert_eq!(styles.len(), 2);
        assert_eq!(
            styles[0].position().unwrap().byte_offset().value(),
            source.find("before{}").unwrap()
        );
        assert_eq!(
            styles[1].position().unwrap().byte_offset().value(),
            source.find("after{}").unwrap()
        );
    }
}

#[test]
fn style_ancestor_remains_restrictive_through_scope_and_conditional_bodies() {
    for (prefix, suffix) in [
        ("host{@scope{@media print{", "}}}"),
        ("@scope (.root){host{@media print{", "}}}"),
        ("host{@media print{@scope{", "}}}"),
        ("host{@scope{@layer audit{", "}}}"),
        ("host{@scope{@container(width>1px){", "}}}"),
        ("@scope (.root){host{@layer audit{", "}}}"),
        ("@scope (.root){host{@container(width>1px){", "}}}"),
    ] {
        let source = format!("{prefix}before{{}}@page :left{{margin:1px}}after{{}}{suffix}");
        let report = parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let rules = contexts(&normalized);
        assert!(
            !rules
                .iter()
                .any(|r| matches!(r.kind(), CssRuleContextKindRef::Page(_)))
        );
        for marker in ["before{}", "after{}"] {
            assert!(rules.iter().any(|r| {
                r.position()
                    .is_some_and(|p| p.byte_offset().value() == source.find(marker).unwrap())
            }));
        }
    }
}

#[test]
fn scope_anonymous_layer_body_retains_page_payload_and_order() {
    assert_admitted("@layer", 0);
}

fn assert_chunked_admission(group: &str) {
    // Public parse/normalize results must not depend on the 64-block split.
    // Payloads at depths 63, 64, 65 exercise both carrier conversions.
    let mut source = "@scope(.root){".to_owned();
    source.push_str(&format!("{group}{{").repeat(61));
    for _ in 0..3 {
        source.push_str(&format!(
            "{group}{{before{{}}@page :left{{margin:1px}}after{{}}"
        ));
    }
    source.push_str(&"}".repeat(65));
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let rules = contexts(&normalized);
    let pages: Vec<_> = rules
        .iter()
        .filter(|r| matches!(r.kind(), CssRuleContextKindRef::Page(_)))
        .collect();
    assert_eq!(pages.len(), 3);
    let group_offsets: Vec<_> = source
        .match_indices(&format!("{group}{{"))
        .map(|(offset, _)| offset)
        .collect();
    for (index, (page_context, (offset, _))) in
        pages.iter().zip(source.match_indices("@page")).enumerate()
    {
        let CssRuleContextKindRef::Page(page) = page_context.kind() else {
            unreachable!()
        };
        assert_eq!(page.selector(), Some(CssPageSelector::Left));
        assert_eq!(page.declarations().len(), 1);
        assert_eq!(page.position().byte_offset().value(), offset);
        let parent = page_context.parent().unwrap();
        let mut ancestor = Some(parent);
        for &expected_offset in group_offsets.iter().take(62 + index).rev() {
            let context = ancestor.expect("every authored group ancestor survives");
            match group.split_whitespace().next().unwrap() {
                "@media" => assert!(matches!(context.kind(), CssRuleContextKindRef::Media(_))),
                "@supports(display:grid)" => {
                    assert!(matches!(context.kind(), CssRuleContextKindRef::Supports(_)))
                }
                "@layer" => assert!(matches!(
                    context.kind(),
                    CssRuleContextKindRef::LayerBlock(_)
                )),
                "@container(width>1px)" => assert!(matches!(
                    context.kind(),
                    CssRuleContextKindRef::Container { .. }
                )),
                _ => panic!("known group fixture"),
            }
            assert_eq!(
                context.position().unwrap().byte_offset().value(),
                expected_offset
            );
            ancestor = context.parent();
        }
        let scope = ancestor.expect("outer scope survives");
        assert!(matches!(scope.kind(), CssRuleContextKindRef::Scope { .. }));
        assert_eq!(scope.position().unwrap().byte_offset().value(), 0);
        assert!(scope.parent().is_none());
        let siblings: Vec<_> = rules
            .iter()
            .filter(|rule| rule.parent().is_some_and(|p| p.same_context(parent)))
            .collect();
        let page_index = siblings
            .iter()
            .position(|r| r.same_context(page_context))
            .unwrap();
        assert!(matches!(
            siblings[page_index - 1].kind(),
            CssRuleContextKindRef::ScopedStyle(_)
        ));
        assert!(matches!(
            siblings[page_index + 1].kind(),
            CssRuleContextKindRef::ScopedStyle(_)
        ));
        assert_eq!(
            siblings[page_index - 1]
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            source[..offset].rfind("before{}").unwrap()
        );
        assert_eq!(
            siblings[page_index + 1]
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            offset + source[offset..].find("after{}").unwrap()
        );
    }
}

#[test]
fn chunked_scoped_media_keeps_pages() {
    assert_chunked_admission("@media print");
}
#[test]
fn chunked_scoped_supports_keeps_pages() {
    assert_chunked_admission("@supports(display:grid)");
}
#[test]
fn chunked_scoped_layer_keeps_pages() {
    assert_chunked_admission("@layer audit");
}
#[test]
fn chunked_scoped_container_keeps_pages() {
    assert_chunked_admission("@container(width>1px)");
}

#[test]
fn chunked_inner_scope_resets_body_admission() {
    // The direct-scope negative is the selected-source category inference.
    for depth in [62, 63, 64] {
        let source = format!(
            "@scope(.root){{{}@scope(.inner){{before{{}}@page :left{{margin:1px}}after{{}}}}{}}}",
            "@media all{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let rules = contexts(&normalized);
        assert!(
            !rules
                .iter()
                .any(|r| matches!(r.kind(), CssRuleContextKindRef::Page(_)))
        );
        for marker in ["before{}", "after{}"] {
            assert!(rules.iter().any(|r| {
                r.position()
                    .is_some_and(|p| p.byte_offset().value() == source.find(marker).unwrap())
            }));
        }
    }
}

#[test]
fn chunked_style_ancestry_remains_restrictive() {
    let source = format!(
        "host{{{}@scope{{@layer audit{{before{{}}@page :left{{margin:1px}}after{{}}}}}}{}}}",
        "@media all{".repeat(64),
        "}".repeat(64)
    );
    let report = parse_sheet(&source);
    assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let rules = contexts(&normalized);
    assert!(
        !rules
            .iter()
            .any(|r| matches!(r.kind(), CssRuleContextKindRef::Page(_)))
    );
    for marker in ["before{}", "after{}"] {
        assert!(rules.iter().any(|r| {
            r.position()
                .is_some_and(|p| p.byte_offset().value() == source.find(marker).unwrap())
        }));
    }
}

#[test]
fn isolated_scope_rule_uses_ordinary_group_admission() {
    let report = parse_rule(
        "@scope(.root){@media print{before{}@page :left{margin:1px}after{}}}",
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssRule::Scope(scope)) = report.syntax() else {
        panic!("scope fragment")
    };
    let [CssScopedRule::Media(media)] = scope.rules().rules() else {
        panic!("media child")
    };
    // Existing API can prove retention before a scoped Page variant is added.
    assert_eq!(media.rules().rules().len(), 3);
}

#[test]
fn isolated_scope_rule_keeps_direct_body_rejection() {
    let report = parse_rule(
        "@scope(.root){before{}@page :left{margin:1px}after{}}",
        &CssNamespaceContext::default(),
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
    let Some(CssRule::Scope(scope)) = report.syntax() else {
        panic!("retained scope fragment")
    };
    assert_eq!(scope.rules().rules().len(), 2);
}

#[test]
fn isolated_style_block_preserves_style_ancestry() {
    let report = parse_style_block(
        "{@scope{@container(width>1px){before{}@page :left{margin:1px}after{}}}}",
        &CssNamespaceContext::default(),
    );
    assert!(report.syntax().is_some());
    assert_eq!(report.diagnostics().len(), 1, "{:?}", report.diagnostics());
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}
