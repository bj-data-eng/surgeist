#![forbid(unsafe_code)]
//! Style-nested scopes retain declaration runs and inherited selector identity.
//! Oracle: Nesting1 2026-01-22 §§3.3, 3.3.1, 5 and Cascade6 2024-09-06
//! §§2.5.2–2.5.5. Direct runs inherit the nearest style's entire selector
//! context, while rule ancestry independently retains active scope boundaries.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nested-declarations-rule
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-nesting
use surgeist_css::*;

fn declarations(sheet: &CssNormalizedSheet) -> Vec<&CssNormalizedDeclaration> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect()
}

fn rules(sheet: &CssNormalizedSheet) -> Vec<&CssRuleContext> {
    sheet
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(value) => Some(value),
            _ => None,
        })
        .collect()
}

fn clean(source: &str) -> CssNormalizedSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    normalize_sheet(report.syntax()).expect("supported color normalization")
}

fn color(declaration: &CssNormalizedDeclaration, source: &str, value: &str, order: usize) {
    assert_eq!(
        declaration
            .source()
            .known()
            .unwrap()
            .property()
            .canonical_name(),
        "color"
    );
    assert_eq!(
        declaration
            .source()
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        value
    );
    assert_eq!(declaration.order(), order);
    let offset = source.find(&format!("color:{value}")).unwrap();
    assert_eq!(
        declaration
            .source()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        offset
    );
    let span = declaration.source().parsed_value().unwrap().span();
    assert_eq!(span.start().byte_offset().value(), offset + 6);
    assert_eq!(span.end().byte_offset().value(), offset + 6 + value.len());
}

fn outer_selectors(sheet: &CssNormalizedSheet) -> &CssSelectorContext {
    let CssNormalizedItem::Rule(rule) = &sheet.items()[0] else {
        panic!("outer style")
    };
    let CssRuleContextKindRef::Style(selectors) = rule.kind() else {
        panic!("outer style context")
    };
    selectors
}

fn scope(rule: &CssRuleContext) {
    assert!(matches!(rule.kind(), CssRuleContextKindRef::Scope { .. }));
}

fn fragment_child_value(scope: &CssScopeRule) {
    let child = scope
        .rules()
        .rules()
        .iter()
        .find_map(|rule| match rule {
            CssScopedRule::Style(style) => Some(style),
            _ => None,
        })
        .expect("valid scoped child remains");
    let [declaration] = child.declarations().as_slice() else {
        panic!("one child color")
    };
    assert_eq!(
        declaration.known().unwrap().property().canonical_name(),
        "color"
    );
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        "blue"
    );
}

#[test]
fn style_nested_scope_keeps_direct_color_and_original_selector_identity() {
    let source = ".p{@scope{color:red}}";
    let sheet = clean(source);
    let values = declarations(&sheet);
    assert_eq!(values.len(), 1);
    color(values[0], source, "red", 0);
    let selectors = outer_selectors(&sheet);
    assert!(values[0].selector_context().same_context(selectors));
    assert_eq!(
        selectors.selectors()[0].selector(),
        &CssSelector::Class("p".into())
    );
    assert_eq!(
        selectors.selectors()[0].binding(),
        CssSelectorBinding::Absolute
    );
    assert!(selectors.scope_context().is_none());
    let run = values[0].rule_context();
    assert!(
        matches!(run.kind(), CssRuleContextKindRef::NestedDeclarations(s) if s.same_context(selectors))
    );
    assert_eq!(
        run.position().unwrap().byte_offset().value(),
        source.find("color").unwrap()
    );
    let enclosing = run.parent().unwrap();
    assert!(matches!(
        enclosing.kind(),
        CssRuleContextKindRef::Scope {
            root: None,
            limit: None
        }
    ));
    assert!(enclosing.parent().unwrap().same_context(rules(&sheet)[0]));
}

#[test]
fn scope_run_preserves_pseudo_element_member_of_parent_selector_list() {
    let source = ".p,.q::before{@scope{color:red}}";
    let sheet = clean(source);
    let values = declarations(&sheet);
    assert_eq!(values.len(), 1);
    color(values[0], source, "red", 0);
    let selectors = values[0].selector_context();
    assert!(selectors.same_context(outer_selectors(&sheet)));
    assert_eq!(selectors.selectors().len(), 2);
    assert_eq!(
        selectors.selectors()[0].selector(),
        &CssSelector::Class("p".into())
    );
    assert!(!selectors.selectors()[0].selector().has_pseudo_elements());
    assert!(selectors.selectors()[1].selector().has_pseudo_elements());
    for entry in selectors.selectors() {
        assert_eq!(entry.binding(), CssSelectorBinding::Absolute);
    }
    scope(values[0].rule_context().parent().unwrap());
}

#[test]
fn explicit_scope_boundaries_survive_without_rebinding_inherited_selectors() {
    let source = ".p{@scope (.root) to (:scope .limit){color:red}}";
    let sheet = clean(source);
    let values = declarations(&sheet);
    assert_eq!(values.len(), 1);
    color(values[0], source, "red", 0);
    let enclosing = values[0].rule_context().parent().unwrap();
    let CssRuleContextKindRef::Scope {
        root: Some(root),
        limit: Some(limit),
    } = enclosing.kind()
    else {
        panic!("both authored boundaries")
    };
    assert_eq!(root.selectors(), &[CssSelector::Class("root".into())]);
    let [CssSelector::Complex(limit)] = limit.selectors() else {
        panic!("one descendant limit")
    };
    let start = limit.first();
    assert_eq!(start.pseudo_classes(), &[CssPseudoClass::Scope]);
    assert!(start.classes().is_empty());
    assert!(start.type_selector().is_none());
    assert!(start.ids().is_empty());
    assert!(start.attributes().is_empty());
    assert_eq!(start.nesting_selectors(), 0);
    assert!(!start.has_scope_anchor());
    assert!(!start.has_pseudo_elements());
    let [part] = limit.rest() else {
        panic!("one descendant step")
    };
    assert_eq!(part.combinator(), CssSelectorCombinator::Descendant);
    let end = part.selector();
    assert_eq!(end.classes(), &["limit".to_owned()]);
    assert!(end.pseudo_classes().is_empty());
    assert!(end.type_selector().is_none());
    assert!(end.ids().is_empty());
    assert!(end.attributes().is_empty());
    assert_eq!(end.nesting_selectors(), 0);
    assert!(!end.has_scope_anchor());
    assert!(!end.has_pseudo_elements());
    assert!(
        values[0]
            .selector_context()
            .same_context(outer_selectors(&sheet))
    );
    assert!(values[0].selector_context().scope_context().is_none());
}

#[test]
fn scope_groups_and_inner_scopes_keep_full_ancestry_and_source_order() {
    let source =
        ".p{@scope{color:red;@media screen{color:green}@scope (.inner){color:blue}color:black}}";
    let sheet = clean(source);
    let values = declarations(&sheet);
    assert_eq!(values.len(), 4);
    let outer = values[0].rule_context().parent().unwrap();
    scope(outer);
    for (order, value) in ["red", "green", "blue", "black"].iter().enumerate() {
        color(values[order], source, value, order);
        assert!(
            values[order]
                .selector_context()
                .same_context(outer_selectors(&sheet))
        );
    }
    let media = values[1].rule_context().parent().unwrap();
    assert!(matches!(media.kind(), CssRuleContextKindRef::Media(_)));
    assert!(media.parent().unwrap().same_context(outer));
    assert_eq!(
        media.position().unwrap().byte_offset().value(),
        source.find("@media").unwrap()
    );
    let inner = values[2].rule_context().parent().unwrap();
    let CssRuleContextKindRef::Scope {
        root: Some(root),
        limit: None,
    } = inner.kind()
    else {
        panic!("inner boundary")
    };
    assert_eq!(root.selectors(), &[CssSelector::Class("inner".into())]);
    assert!(!inner.same_context(outer));
    assert!(inner.parent().unwrap().same_context(outer));
    assert!(
        values[3]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(outer)
    );
    assert!(
        !values[0]
            .rule_context()
            .same_context(values[3].rule_context())
    );
}

#[test]
fn scoped_child_replaces_handle_locally_and_sibling_run_restores_outer_handle() {
    let source = ".p{@scope{.child{color:blue;@media screen{color:green}}color:red}}";
    let sheet = clean(source);
    let values = declarations(&sheet);
    assert_eq!(values.len(), 3);
    for (order, value) in ["blue", "green", "red"].iter().enumerate() {
        color(values[order], source, value, order);
    }
    let child = values[0].selector_context();
    assert_eq!(
        child.selectors()[0].selector(),
        &CssSelector::Class("child".into())
    );
    assert!(child.parent().is_none());
    assert!(!child.same_context(outer_selectors(&sheet)));
    assert!(values[1].selector_context().same_context(child));
    let enclosing = values[2].rule_context().parent().unwrap();
    scope(enclosing);
    assert!(child.scope_context().unwrap().same_context(enclosing));
    assert!(
        values[0]
            .rule_context()
            .parent()
            .unwrap()
            .same_context(enclosing)
    );
    assert!(
        values[1]
            .rule_context()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .same_context(values[0].rule_context())
    );
    assert!(
        values[2]
            .selector_context()
            .same_context(outer_selectors(&sheet))
    );
    assert!(values[2].selector_context().scope_context().is_none());
}

#[test]
fn unstyled_scope_rule_list_recovery_does_not_admit_declarations() {
    for source in [
        "@scope{.child{color:blue}color:red;}",
        "@scope{@media screen{.child{color:blue}color:red;}}",
        "@media screen{@scope{.child{color:blue}color:red;}}",
    ] {
        let report = parse_sheet(source);
        assert!(!report.is_clean());
        let normalized = normalize_report(&report).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let values = declarations(normalized.syntax());
        assert_eq!(values.len(), 1);
        color(values[0], source, "blue", 0);
        assert_eq!(
            values[0].selector_context().selectors()[0].selector(),
            &CssSelector::Class("child".into())
        );
        assert!(values[0].selector_context().parent().is_none());
        scope(values[0].selector_context().scope_context().unwrap());
    }
    // In a rule list, the declaration-looking prefix and following child form
    // one invalid qualified rule. A semicolon does not end its prelude.
    for source in [
        "@scope{color:red;.child{color:blue}}",
        "@scope{@media screen{color:red;.child{color:blue}}}",
        "@media screen{@scope{color:red;.child{color:blue}}}",
    ] {
        let report = parse_sheet(source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid qualified rule")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        let normalized = normalize_report(&report).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        assert!(declarations(normalized.syntax()).is_empty());
    }
}

#[test]
fn public_style_fragments_admit_scope_runs_with_ordinary_boundaries() {
    let ns = CssNamespaceContext::default();
    let report = parse_rule(".p{@scope (.root){color:red;.child{color:blue}}}", &ns);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssRule::Style(style)) = report.syntax() else {
        panic!("style fragment")
    };
    let [CssRule::Scope(scope)] = style.rules() else {
        panic!("scope child")
    };
    assert_eq!(
        scope.root().unwrap().selectors(),
        &[CssSelector::Class("root".into())]
    );
    fragment_child_value(scope);
    let block = parse_style_block("{@scope (.root){color:red;.child{color:blue}}}", &ns);
    assert!(block.is_clean(), "{:?}", block.diagnostics());
    let [CssRule::Scope(scope)] = block.syntax().as_ref().unwrap().rules() else {
        panic!("block scope")
    };
    assert_eq!(
        scope.root().unwrap().selectors(),
        &[CssSelector::Class("root".into())]
    );
    fragment_child_value(scope);
    // New authored run payload checks belong in the later new-variant tests.
    let unstyled = parse_rule("@scope (.root){.child{color:blue}color:red;}", &ns);
    assert!(!unstyled.is_clean());
    let Some(CssRule::Scope(scope)) = unstyled.syntax() else {
        panic!("scope fragment retained")
    };
    fragment_child_value(scope);
}

#[test]
fn scoped_runs_survive_bounded_group_and_scope_carrier_splits() {
    for depth in [63, 64, 65, 127] {
        for group in [
            "@media screen",
            "@supports(display:grid)",
            "@layer audit",
            "@container(width>1px)",
            "@scope (.inner)",
        ] {
            let prefix = format!("{group}{{");
            let source = format!(
                ".p{{@scope{{{}color:red;.child{{color:blue}}color:green{}}}}}",
                prefix.repeat(depth - 2),
                "}".repeat(depth - 2)
            );
            let sheet = clean(&source);
            let values = declarations(&sheet);
            assert_eq!(values.len(), 3, "depth {depth}: {group}");
            for (order, value) in ["red", "blue", "green"].iter().enumerate() {
                color(values[order], &source, value, order);
            }
            for index in [0, 2] {
                assert!(
                    values[index]
                        .selector_context()
                        .same_context(outer_selectors(&sheet))
                );
                let mut ancestor = values[index].rule_context().parent();
                for (offset, _) in source
                    .match_indices(&prefix)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    let rule = ancestor.unwrap();
                    assert_eq!(rule.position().unwrap().byte_offset().value(), offset);
                    match group {
                        "@media screen" => {
                            assert!(matches!(rule.kind(), CssRuleContextKindRef::Media(_)))
                        }
                        "@supports(display:grid)" => {
                            assert!(matches!(rule.kind(), CssRuleContextKindRef::Supports(_)))
                        }
                        "@layer audit" => {
                            assert!(matches!(rule.kind(), CssRuleContextKindRef::LayerBlock(_)))
                        }
                        "@container(width>1px)" => assert!(matches!(
                            rule.kind(),
                            CssRuleContextKindRef::Container { .. }
                        )),
                        _ => scope(rule),
                    }
                    ancestor = rule.parent();
                }
                let outer = ancestor.unwrap();
                scope(outer);
                assert_eq!(outer.position().unwrap().byte_offset().value(), 3);
                assert!(outer.parent().unwrap().same_context(rules(&sheet)[0]));
            }
            assert!(
                !values[1]
                    .selector_context()
                    .same_context(outer_selectors(&sheet))
            );
            assert!(values[1].selector_context().parent().is_none());
            assert!(
                values[0]
                    .rule_context()
                    .parent()
                    .unwrap()
                    .same_context(values[2].rule_context().parent().unwrap())
            );
            assert!(
                !values[0]
                    .rule_context()
                    .same_context(values[2].rule_context())
            );
        }
    }
}

#[test]
fn retained_scoped_run_consumes_exact_normalization_resources_atomically() {
    let source = ".p{@scope{color:red}}";
    let report = parse_sheet(source);
    // Do not preempt this resource oracle with cleanliness: on the old behavior,
    // normalization succeeds because the declaration run was lost.
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(2, 2, 1, 1).unwrap(),
            CssNormalizationResource::Rules,
            2,
        ),
        (
            CssNormalizationLimits::try_new(1, 3, 1, 1).unwrap(),
            CssNormalizationResource::RuleDepth,
            1,
        ),
        (
            CssNormalizationLimits::try_new(2, 3, 0, 1).unwrap(),
            CssNormalizationResource::Declarations,
            0,
        ),
        (
            CssNormalizationLimits::try_new(2, 3, 1, 0).unwrap(),
            CssNormalizationResource::Contributions,
            0,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits)
            .expect_err("retained run must consume its budget");
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
        assert_eq!(
            error.position().unwrap().byte_offset().value(),
            source.find("color").unwrap()
        );
    }
    let sheet = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(2, 3, 1, 1).unwrap(),
    )
    .unwrap();
    let values = declarations(&sheet);
    assert_eq!(values.len(), 1);
    color(values[0], source, "red", 0);
    assert!(
        values[0]
            .selector_context()
            .same_context(outer_selectors(&sheet))
    );
    assert!(report.is_clean());
}
