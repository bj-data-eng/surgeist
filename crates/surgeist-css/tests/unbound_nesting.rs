#![forbid(unsafe_code)]
//! Nesting 1 section 4: parentless & matches scope with zero anchor specificity.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nest-selector
use surgeist_css::{
    CssNamespaceContext, CssNormalizedItem, CssSelector, CssSelectorBinding, normalize_sheet,
    parse_selector, parse_selector_list, parse_sheet,
};

#[test]
fn ordinary_compounds_retain_unbound_anchors_without_scope_rewriting() {
    for (source, count) in [
        ("&", 1),
        ("&.a", 1),
        (".a&", 1),
        ("div&", 1),
        ("&&", 2),
        (".a&&", 2),
    ] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {report:?}");
        let CssSelector::Compound(compound) = report.syntax().as_ref().unwrap() else {
            panic!("compound")
        };
        assert_eq!(compound.nesting_selectors(), count);
        assert!(!compound.has_scope_anchor());
        assert!(compound.pseudo_classes().is_empty());
        assert!(parse_selector_list(source, &CssNamespaceContext::default()).is_clean());
    }
}

#[test]
fn normalized_anchor_context_distinguishes_missing_and_real_parents() {
    for (source, has_parent) in [
        ("& {color:red}", false),
        (".parent { & {color:red} }", true),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let declarations: Vec<_> = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        assert_eq!(declarations.len(), 1);
        let context = declarations[0].selector_context();
        assert_eq!(context.parent().is_some(), has_parent);
        assert_eq!(
            context.selectors()[0].binding(),
            CssSelectorBinding::ExplicitAnchors
        );
        let CssSelector::Compound(compound) = context.selectors()[0].selector() else {
            panic!("compound")
        };
        assert_eq!(compound.nesting_selectors(), 1);
        assert!(!compound.has_scope_anchor());
    }
}

#[test]
fn unbound_anchor_admission_does_not_enable_relative_roots_or_trailing_tokens() {
    for source in ["&div", ">&", "&.a{}", "&.a;", ""] {
        let report = parse_selector(source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
    }
}

#[test]
fn supports_selector_anchor_is_typed_instead_of_general_enclosed() {
    let report = parse_sheet("@supports selector(&) { a { color:red } }");
    assert!(report.is_clean(), "{report:?}");
    let [surgeist_css::CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("supports")
    };
    let surgeist_css::CssSupportsConditionKind::Selector(CssSelector::Compound(compound)) =
        rule.condition().kind()
    else {
        panic!("typed selector, not opaque fallback")
    };
    assert_eq!(compound.nesting_selectors(), 1);
}
