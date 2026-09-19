#![forbid(unsafe_code)]
//! Nested root & refers to the nearest style, while limit/body & refers to scope.
//! Parentless & and repeated anchors are authored selectors, not parse errors.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nested-scope-rules
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nest-selector
//! The unchanged optional selector-list bounds exclude pseudo-elements.
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-syntax
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-bounds
use surgeist_css::*;

fn anchor(selector: &CssSelector, nesting: usize, scope: bool) {
    let CssSelector::Compound(compound) = selector else {
        panic!("authored anchor compound: {selector:?}")
    };
    assert_eq!(compound.nesting_selectors(), nesting);
    assert_eq!(compound.has_scope_anchor(), scope);
    assert!(compound.pseudo_classes().is_empty());
    assert!(compound.type_selector().is_none());
    assert!(compound.classes().is_empty());
    assert!(compound.ids().is_empty());
}

fn single(list: &CssScopeSelectorList) -> &CssSelector {
    let [selector] = list.selectors() else {
        panic!("one boundary selector")
    };
    selector
}

fn simple_bounds(scope: &CssScopeRule, root_nesting: usize, root_scope: bool) {
    anchor(single(scope.root().unwrap()), root_nesting, root_scope);
    anchor(single(scope.limit().unwrap()), 0, true);
    let [CssScopedRule::Style(child)] = scope.rules().rules() else {
        panic!("one scoped child style")
    };
    assert_eq!(child.declarations().len(), 1);
    assert_eq!(
        child.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert_eq!(
        child.declarations()[0]
            .value_components()
            .serialize()
            .unwrap()
            .as_css()
            .trim(),
        "red"
    );
}

fn context_at<'a>(
    normalized: &'a CssNormalizedSheet,
    source: &str,
    marker: &str,
) -> &'a CssRuleContext {
    let offset = source.find(marker).unwrap();
    let contexts: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Rule(context)
                if context
                    .position()
                    .is_some_and(|position| position.byte_offset().value() == offset) =>
            {
                Some(context)
            }
            _ => None,
        })
        .collect();
    let [context] = contexts.as_slice() else {
        panic!("one rule at {marker}")
    };
    context
}

fn normalized_bounds(context: &CssRuleContext, root_nesting: usize, root_scope: bool) {
    let CssRuleContextKindRef::Scope {
        root: Some(root),
        limit: Some(limit),
    } = context.kind()
    else {
        panic!("scope bounds")
    };
    anchor(single(root), root_nesting, root_scope);
    anchor(single(limit), 0, true);
}

#[test]
fn style_nested_root_and_limit_keep_distinct_anchor_categories() {
    let source = ".parent{@scope(&) to (&){.child{color:red}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    let [CssRule::Scope(scope)] = parent.rules() else {
        panic!("scope")
    };
    simple_bounds(scope, 1, false);
    assert_eq!(
        scope.position().byte_offset().value(),
        source.find("@scope").unwrap()
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let parent = context_at(&normalized, source, ".parent");
    let scope = context_at(&normalized, source, "@scope");
    let child = context_at(&normalized, source, ".child");
    normalized_bounds(scope, 1, false);
    assert!(scope.parent().unwrap().same_context(parent));
    assert!(child.parent().unwrap().same_context(scope));
    let CssRuleContextKindRef::ScopedStyle(selectors) = child.kind() else {
        panic!("scoped child")
    };
    assert!(selectors.parent().is_none());
    assert!(selectors.scope_context().unwrap().same_context(scope));
}

#[test]
fn ordinary_root_is_parentless_while_limit_belongs_to_introduced_scope() {
    let source = "@scope(&) to (&){.child{color:red}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    simple_bounds(scope, 1, false);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let scope = context_at(&normalized, source, "@scope");
    normalized_bounds(scope, 1, false);
    assert!(scope.parent().is_none());
}

#[test]
fn ordinary_scope_ancestor_supplies_root_scope_but_not_limit_scope_identity() {
    let source = "@scope(.outer){@media all{@scope(&) to (&){.child{color:red}}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(outer)] = report.syntax().rules() else {
        panic!("outer")
    };
    let [CssScopedRule::Media(media)] = outer.rules().rules() else {
        panic!("media")
    };
    let [CssScopedRule::Scope(inner)] = media.rules().rules() else {
        panic!("inner")
    };
    simple_bounds(inner, 0, true);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let outer = context_at(&normalized, source, "@scope(.outer)");
    let media = context_at(&normalized, source, "@media");
    let inner = context_at(&normalized, source, "@scope(&)");
    normalized_bounds(inner, 0, true);
    assert!(inner.parent().unwrap().same_context(media));
    assert!(media.parent().unwrap().same_context(outer));
    assert!(!inner.same_context(outer));
}

#[test]
fn root_ancestry_selects_nearer_scoped_style_instead_of_outer_style() {
    let source = ".outer{@scope(.middle){.near{@media all{@scope(&) to (&){.child{color:red}}}}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let outer = context_at(&normalized, source, ".outer");
    let middle = context_at(&normalized, source, "@scope(.middle)");
    let near = context_at(&normalized, source, ".near");
    let inner = context_at(&normalized, source, "@scope(&)");
    normalized_bounds(inner, 1, false);
    assert!(matches!(outer.kind(), CssRuleContextKindRef::Style(_)));
    let CssRuleContextKindRef::ScopedStyle(selectors) = near.kind() else {
        panic!("nearer scoped style")
    };
    assert_eq!(selectors.selectors().len(), 1);
    assert_eq!(
        selectors.selectors()[0].selector(),
        &CssSelector::Class("near".to_owned())
    );
    assert!(selectors.scope_context().unwrap().same_context(middle));
    let mut ancestor = inner.parent().unwrap();
    while !matches!(
        ancestor.kind(),
        CssRuleContextKindRef::Style(_) | CssRuleContextKindRef::ScopedStyle(_)
    ) {
        ancestor = ancestor.parent().unwrap();
    }
    assert!(ancestor.same_context(near));
    assert!(!ancestor.same_context(outer));
    assert!(near.parent().unwrap().same_context(middle));
    assert!(middle.parent().unwrap().same_context(outer));
}

#[test]
fn anchor_modes_propagate_through_exact_function_members_and_combinators() {
    let source = ".parent{@media all{@scope(:is(&,.fallback)) to (:not(&)){.child{color:red}}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let scope = context_at(&normalized, source, "@scope");
    let CssRuleContextKindRef::Scope {
        root: Some(root),
        limit: Some(limit),
    } = scope.kind()
    else {
        panic!("scope bounds")
    };
    let CssSelector::PseudoClass(CssPseudoClass::Is(root)) = single(root) else {
        panic!("is function")
    };
    let [parent, fallback] = root.selectors() else {
        panic!("both authored members, no forgiving loss")
    };
    anchor(parent, 1, false);
    assert_eq!(fallback, &CssSelector::Class("fallback".to_owned()));
    let CssSelector::PseudoClass(CssPseudoClass::Not(limit)) = single(limit) else {
        panic!("not function")
    };
    let [limit] = limit.selectors() else {
        panic!("one not member")
    };
    anchor(limit, 0, true);

    let report = parse_sheet(".parent{@scope(& > .root) to (& .stop){.child{color:red}}}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    let [CssRule::Scope(scope)] = parent.rules() else {
        panic!("scope")
    };
    for (list, nesting, scope_anchor, combinator, class) in [
        (
            scope.root().unwrap(),
            1,
            false,
            CssSelectorCombinator::Child,
            "root",
        ),
        (
            scope.limit().unwrap(),
            0,
            true,
            CssSelectorCombinator::Descendant,
            "stop",
        ),
    ] {
        let CssSelector::Complex(complex) = single(list) else {
            panic!("complex boundary")
        };
        assert_eq!(complex.first().nesting_selectors(), nesting);
        assert_eq!(complex.first().has_scope_anchor(), scope_anchor);
        let [part] = complex.rest() else {
            panic!("one combinator")
        };
        assert_eq!(part.combinator(), combinator);
        assert_eq!(part.selector().classes(), [class]);
    }
}

#[test]
fn repeated_boundary_anchors_are_admitted_without_rewriting_to_pseudo_classes() {
    let report = parse_sheet(".parent{@scope(&&) to (&&){.child{color:red}}}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    let [CssRule::Scope(scope)] = parent.rules() else {
        panic!("scope")
    };
    simple_bounds(scope, 2, false);
    // Existing API proves scope-anchor category and admission. Exact scope count
    // is an additional assertion when its functional accessor is introduced.
}

#[test]
fn repeated_scoped_body_anchors_share_the_same_authored_scope_category() {
    let source = "@scope(.root){&&{color:red}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    let [CssScopedRule::Style(style)] = scope.rules().rules() else {
        panic!("style")
    };
    let [CssScopedStyleSelector::Selector(selector)] = style.selectors().selectors() else {
        panic!("selector")
    };
    anchor(selector, 0, true);
    assert_eq!(style.declarations().len(), 1);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let context = context_at(&normalized, source, "&&");
    let CssRuleContextKindRef::ScopedStyle(selectors) = context.kind() else {
        panic!("scoped style")
    };
    assert_eq!(
        selectors.selectors()[0].binding(),
        CssSelectorBinding::ScopeAnchors
    );
    assert!(selectors.parent().is_none());
}

#[test]
fn fragment_front_doors_preserve_boundary_anchor_modes() {
    let report = parse_rule(
        "@scope(&) to (&){.child{color:red}}",
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssRule::Scope(scope)) = report.syntax() else {
        panic!("scope fragment")
    };
    simple_bounds(scope, 1, false);
    let report = parse_style_block(
        "{@scope(&) to (&){.child{color:red}}}",
        &CssNamespaceContext::default(),
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let block = report.syntax().as_ref().unwrap();
    let [CssRule::Scope(scope)] = block.rules() else {
        panic!("style block scope")
    };
    simple_bounds(scope, 1, false);
}

#[test]
fn scope_root_modes_survive_structural_chunk_boundaries() {
    // Target scope opens at structural depths 63, 64 and 65 in each context.
    for depth in [63, 64, 65] {
        for styled in [false, true] {
            let prefix = if styled {
                ".parent{"
            } else {
                "@scope(.outer){"
            };
            let source = format!(
                "{prefix}{}@scope(&) to (&){{.child{{color:red}}}}{} }}",
                "@scope{".repeat(depth - 2),
                "}".repeat(depth - 2)
            );
            let report = parse_sheet(&source);
            assert!(
                report.is_clean(),
                "depth={depth} styled={styled}: {:?}",
                report.diagnostics()
            );
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let inner = context_at(&normalized, &source, "@scope(&)");
            normalized_bounds(inner, usize::from(styled), !styled);
            let mut ancestor = inner.parent();
            let mut styles = Vec::new();
            while let Some(context) = ancestor {
                if matches!(
                    context.kind(),
                    CssRuleContextKindRef::Style(_) | CssRuleContextKindRef::ScopedStyle(_)
                ) {
                    styles.push(context);
                }
                ancestor = context.parent();
            }
            assert_eq!(styles.len(), usize::from(styled));
            if styled {
                assert!(styles[0].same_context(context_at(&normalized, &source, ".parent")));
            }
        }
    }
}

#[test]
fn omitted_bounds_explicit_scope_and_escaped_identifier_remain_distinct_controls() {
    let report = parse_sheet("@scope to (:scope){.child{color:red}}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    assert!(scope.root().is_none());
    assert_eq!(
        single(scope.limit().unwrap()),
        &CssSelector::PseudoClass(CssPseudoClass::Scope)
    );
    let report = parse_sheet("@scope(:scope) to (:scope){.child{color:red}}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    assert_eq!(
        single(scope.root().unwrap()),
        &CssSelector::PseudoClass(CssPseudoClass::Scope)
    );
    assert_eq!(
        single(scope.limit().unwrap()),
        &CssSelector::PseudoClass(CssPseudoClass::Scope)
    );
    let report = parse_sheet("@scope(\\26){.child{color:red}}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Scope(scope)] = report.syntax().rules() else {
        panic!("scope")
    };
    assert_eq!(
        single(scope.root().unwrap()),
        &CssSelector::Tag("&".to_owned())
    );
}

#[test]
fn invalid_boundary_lists_still_drop_scope_and_preserve_later_neighbor() {
    for prelude in [
        "()",
        "(.root,)",
        "(.root::before)",
        "(.root) to (::after)",
        "(> .root)",
        "(.root) to (> .stop)",
        "(.root) junk",
        "(&div)",
    ] {
        let source = format!("@scope{prelude}{{.lost{{color:blue}}}}.after{{color:red}}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
        );
        let [CssRule::Style(after)] = report.syntax().rules() else {
            panic!("only later neighbor for {source}")
        };
        assert_eq!(
            after.position().byte_offset().value(),
            source.find(".after").unwrap()
        );
        assert_eq!(after.declarations().len(), 1);
    }
}
