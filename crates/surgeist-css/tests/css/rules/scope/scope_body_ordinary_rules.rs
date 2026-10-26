//! Cascade 6 §2.5: a scope body is a list of ordinary rules; the differences are
//! placement and the scoping root (#1386). Placement is a check of the
//! construction and assembly context, not a separate rule type.
use crate::css::support::rules::clean_sheet as sheet;
use surgeist_css::*;

fn context() -> CssNamespaceContext {
    CssNamespaceContext::default()
}

fn scope_body(source: &str) -> Vec<CssRule> {
    let input = sheet(&format!("@scope (.root) {{ {source} }}"));
    let [CssRule::Scope(scope)] = input.rules() else {
        panic!("one scope");
    };
    scope.rules().rules().to_vec()
}

fn assemble(
    rules: Vec<CssRule>,
    nesting: CssScopeNestingContext,
) -> Result<CssScopeRule, CssRuleConstructionError> {
    CssScopeRule::try_new(None, None, rules, &context(), nesting)
}

#[test]
fn a_scope_body_holds_the_same_rule_values_as_any_rule_list() {
    let input = sheet(
        "@scope (.a) { .b { color: red } @media (width > 1px) { .c { color: blue } } @layer l { .d { color: green } } }",
    );
    let [CssRule::Scope(scope)] = input.rules() else {
        panic!("one scope");
    };
    let [
        CssRule::Style(style),
        CssRule::Media(media),
        CssRule::LayerBlock(layer),
    ] = scope.rules().rules()
    else {
        panic!("ordinary rules in authored order");
    };
    assert_eq!(style.declarations().len(), 1);
    assert!(matches!(media.rules(), [CssRule::Style(_)]));
    assert!(matches!(layer.rules(), [CssRule::Style(_)]));
}

#[test]
fn import_and_namespace_are_rejected_by_the_assembly_context_not_by_a_type() {
    let import = sheet("@import 'a';").rules()[0].clone();
    let namespace = sheet("@namespace p 'urn:p';").rules()[0].clone();
    for rule in [import, namespace] {
        let error = assemble(vec![rule], CssScopeNestingContext::None).unwrap_err();
        assert!(error.is_invalid_placement(), "{error:?}");
        assert_eq!(error.path(), &[0]);
    }
}

#[test]
fn declaration_runs_need_a_style_ancestor_in_the_assembly_context() {
    let input = sheet("p { @scope (&) { color: red; } }");
    let [CssRule::Style(host)] = input.rules() else {
        panic!("host");
    };
    let [CssRule::Scope(scope)] = host.rules() else {
        panic!("scope");
    };
    let rules = scope.rules().rules().to_vec();
    assert!(matches!(rules.as_slice(), [CssRule::NestedDeclarations(_)]));
    for nesting in [CssScopeNestingContext::None, CssScopeNestingContext::Scope] {
        let error = assemble(rules.clone(), nesting).unwrap_err();
        assert!(error.is_invalid_placement(), "{nesting:?}: {error:?}");
        assert_eq!(error.path(), &[0]);
    }
    let rebuilt = assemble(rules.clone(), CssScopeNestingContext::Style).unwrap();
    assert_eq!(rebuilt.rules().rules(), rules.as_slice());
}

#[test]
fn page_is_excluded_from_a_scope_body_but_not_from_a_group_inside_it() {
    let direct = scope_body("@media all { @page {} }");
    let [CssRule::Media(media)] = direct.as_slice() else {
        panic!("media");
    };
    let page = media.rules()[0].clone();
    let error = assemble(vec![page.clone()], CssScopeNestingContext::None).unwrap_err();
    assert!(error.is_invalid_placement(), "{error:?}");
    assert!(
        CssMediaRule::try_new(CssMediaQueryList::new(Vec::new()), vec![page], &context()).is_ok()
    );
}

#[test]
fn relative_selectors_belong_to_a_scope_body_and_are_rechecked_when_detached() {
    let body = scope_body("> .child { color: red } @media all { + .sibling { color: blue } }");
    let rebuilt = assemble(body.clone(), CssScopeNestingContext::None).unwrap();
    assert_eq!(rebuilt.rules().rules(), body.as_slice());
    // A group built from the relative child alone has no scope or style ancestor.
    let CssRule::Style(_) = &body[0] else {
        panic!("relative style rule");
    };
    let error = CssMediaRule::try_new(
        CssMediaQueryList::new(Vec::new()),
        vec![body[0].clone()],
        &context(),
    )
    .unwrap_err();
    assert!(error.is_invalid_placement(), "{error:?}");
    assert!(CssSheet::try_from_rules(vec![CssRule::Scope(rebuilt)]).is_ok());
}

/// The style rule `&.x {}` as a scope body holds it: `&` is a scope anchor.
fn scope_anchored_rule() -> CssRule {
    scope_body("&.x { color: red }").remove(0)
}

/// The style rule `&.x {}` as a sheet holds it: `&` is a nesting anchor.
fn nesting_anchored_rule() -> CssRule {
    sheet("&.x { color: red }").rules()[0].clone()
}

#[test]
fn a_scope_anchored_style_rule_is_only_valid_in_a_scope_body() {
    let rule = scope_anchored_rule();
    // Cascade 6 §2.5: in a scope body `&` is the scoping root. Outside one it would
    // be a nesting anchor of another domain, which this list cannot hold.
    let error = CssMediaRule::try_new(
        CssMediaQueryList::new(Vec::new()),
        vec![rule.clone()],
        &context(),
    )
    .unwrap_err();
    assert!(error.is_invalid_boundary_anchor(), "{error:?}");
    let error = CssSheet::try_from_rules(vec![rule.clone()]).unwrap_err();
    assert!(error.is_invalid_boundary_anchor(), "{error:?}");
    assert_eq!(error.path(), &[0]);
    // The scope body, directly or through a group, accepts it.
    assert!(assemble(vec![rule.clone()], CssScopeNestingContext::None).is_ok());
    let body = scope_body("@media all { &.x { color: red } }");
    assert!(assemble(body, CssScopeNestingContext::None).is_ok());
}

#[test]
fn a_nesting_anchored_style_rule_is_not_valid_directly_in_a_scope_body() {
    let rule = nesting_anchored_rule();
    let error = assemble(vec![rule.clone()], CssScopeNestingContext::None).unwrap_err();
    assert!(error.is_invalid_boundary_anchor(), "{error:?}");
    assert_eq!(error.path(), &[0]);
    // Through a group inside the scope body it is rejected at the same rule.
    let media = CssMediaRule::try_new(
        CssMediaQueryList::new(Vec::new()),
        vec![rule.clone()],
        &context(),
    )
    .unwrap();
    let error = assemble(vec![CssRule::Media(media)], CssScopeNestingContext::None).unwrap_err();
    assert!(error.is_invalid_boundary_anchor(), "{error:?}");
    assert_eq!(error.path(), &[0, 0]);
    // A sheet holds it.
    assert!(CssSheet::try_from_rules(vec![rule]).is_ok());
}

#[test]
fn the_children_of_a_scoped_style_rule_use_nesting_anchors_of_that_rule() {
    // `&` below a scoped style rule refers to that rule, not to the scoping root.
    let body = scope_body(".a { &.b { color: red } }");
    let [CssRule::Style(parent)] = body.as_slice() else {
        panic!("one style rule");
    };
    assert!(matches!(parent.rules(), [CssRule::Style(_)]));
    let rebuilt = assemble(body.clone(), CssScopeNestingContext::None).unwrap();
    assert_eq!(rebuilt.rules().rules(), body.as_slice());
    // The same holds below a group in that style rule's body.
    let body = scope_body(".a { @media all { &.b { color: red } } }");
    assert!(assemble(body, CssScopeNestingContext::None).is_ok());
}

#[test]
fn boundary_selectors_stay_free_of_pseudo_elements() {
    let pseudo = sheet("a::before {}").rules()[0].clone();
    let CssRule::Style(style) = pseudo else {
        panic!("style");
    };
    assert_eq!(
        CssScopeSelectorList::try_new(style.selectors().selectors().to_vec()),
        None
    );
    let ordinary = sheet("a {}").rules()[0].clone();
    let CssRule::Style(style) = ordinary else {
        panic!("style");
    };
    assert!(CssScopeSelectorList::try_new(style.selectors().selectors().to_vec()).is_some());
}
