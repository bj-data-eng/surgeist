//! Cascade6 §2.5.4/2.5.5 and Nesting1 §3.2 contextual boundary contracts.
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-scope
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#syntax-examples
use crate::css::support::rules::clean_sheet as sheet;
use surgeist_css::*;

fn relative(combinator: CssSelectorCombinator, selector: CssSelector) -> CssScopeSelectorList {
    CssScopeSelectorList::try_new(vec![CssStyleSelector::Relative(CssRelativeSelector::new(
        combinator, selector,
    ))])
    .unwrap()
}

fn new_scope(
    root: Option<CssScopeSelectorList>,
    limit: Option<CssScopeSelectorList>,
    nesting: CssScopeNestingContext,
) -> Result<CssScopeRule, CssRuleConstructionError> {
    CssScopeRule::try_new(
        root,
        limit,
        Vec::new(),
        &CssNamespaceContext::default(),
        nesting,
    )
}

#[test]
fn checked_relative_members_preserve_combinators_and_contextual_root_admission() {
    for combinator in [
        CssSelectorCombinator::Descendant,
        CssSelectorCombinator::Child,
        CssSelectorCombinator::NextSibling,
        CssSelectorCombinator::SubsequentSibling,
    ] {
        let boundary = relative(combinator, CssSelector::Class("target".into()));
        assert!(
            new_scope(Some(boundary.clone()), None, CssScopeNestingContext::None)
                .unwrap_err()
                .is_invalid_placement()
        );
        for nesting in [CssScopeNestingContext::Style, CssScopeNestingContext::Scope] {
            let scope = new_scope(Some(boundary.clone()), Some(boundary.clone()), nesting).unwrap();
            assert_eq!(scope.position(), None);
            let [CssStyleSelector::Relative(member)] = scope.root().unwrap().selectors() else {
                panic!("relative member")
            };
            assert_eq!(member.combinator(), combinator);
            assert_eq!(member.selector(), &CssSelector::Class("target".into()));
            assert_eq!(
                member.selector(),
                scope.limit().unwrap().selectors()[0].selector()
            );
        }
        let scope = new_scope(None, Some(boundary), CssScopeNestingContext::None).unwrap();
        assert!(CssSheet::try_from_rules(vec![CssRule::Scope(scope)]).is_ok());
    }
}

#[test]
fn checked_assembly_revalidates_actual_scope_ancestry_and_parsed_position() {
    let source = "@scope(.outer){@media all{@scope(> .root) to (+ .stop){}}}";
    let input = sheet(source);
    let [CssRule::Scope(outer)] = input.rules() else {
        panic!("outer")
    };
    let [CssRule::Media(media)] = outer.rules().rules() else {
        panic!("media")
    };
    let [CssRule::Scope(inner)] = media.rules() else {
        panic!("inner")
    };
    let before = inner.clone();
    let error = CssSheet::try_from_rules(vec![CssRule::Scope(inner.clone())]).unwrap_err();
    assert!(error.is_invalid_placement(), "{error:?}");
    assert_eq!(error.path(), &[0]);
    assert_eq!(error.position(), inner.position());
    assert_eq!(
        error.position().unwrap().byte_offset().value(),
        source.find("@scope(>").unwrap()
    );
    let checked = CssScopeRule::try_new(
        None,
        None,
        vec![CssRule::Media(media.clone())],
        &CssNamespaceContext::default(),
        CssScopeNestingContext::None,
    )
    .unwrap();
    assert!(CssSheet::try_from_rules(vec![CssRule::Scope(checked)]).is_ok());
    let detached = new_scope(
        Some(relative(
            CssSelectorCombinator::Child,
            CssSelector::Class("root".into()),
        )),
        None,
        CssScopeNestingContext::Scope,
    )
    .unwrap();
    assert_eq!(
        CssSheet::try_from_rules(vec![CssRule::Scope(detached)])
            .unwrap_err()
            .position(),
        None
    );
    assert_eq!(*inner, before);
}

#[test]
fn relative_boundary_payloads_share_checked_namespace_admission() {
    let input = sheet("@namespace svg 'urn:svg';.parent{@scope(> svg|root) to (+ svg|stop){}}");
    let [CssRule::Namespace(_), CssRule::Style(style)] = input.rules() else {
        panic!("namespace and style")
    };
    let [CssRule::Scope(scope)] = style.rules() else {
        panic!("nested scope")
    };
    let context = CssNamespaceContext::from_sheet(&input);
    let before = scope.clone();
    let checked = CssScopeRule::try_new(
        scope.root().cloned(),
        scope.limit().cloned(),
        Vec::new(),
        &context,
        CssScopeNestingContext::Style,
    )
    .unwrap();
    assert_eq!(checked.root(), scope.root());
    assert_eq!(checked.limit(), scope.limit());
    let error = CssScopeRule::try_new(
        scope.root().cloned(),
        scope.limit().cloned(),
        Vec::new(),
        &CssNamespaceContext::default(),
        CssScopeNestingContext::Style,
    )
    .unwrap_err();
    assert!(error.is_namespace_mismatch(), "{error:?}");
    assert!(error.path().is_empty());
    assert_eq!(error.position(), None);
    assert_eq!(*scope, before);
}

#[test]
fn explicit_boundary_anchors_cannot_silently_change_categories_when_reassembled() {
    let ordinary = sheet("@scope(&) to (&){}");
    let [CssRule::Scope(ordinary)] = ordinary.rules() else {
        panic!("scope")
    };
    let scoped = sheet("@scope(.outer){@scope(&) to (&){}}");
    let [CssRule::Scope(outer)] = scoped.rules() else {
        panic!("outer")
    };
    let [CssRule::Scope(inner)] = outer.rules().rules() else {
        panic!("inner")
    };
    for (root, nesting) in [
        (ordinary.root().cloned(), CssScopeNestingContext::Scope),
        (inner.root().cloned(), CssScopeNestingContext::Style),
        (inner.root().cloned(), CssScopeNestingContext::None),
    ] {
        assert!(
            new_scope(root, None, nesting)
                .unwrap_err()
                .is_invalid_boundary_anchor()
        );
    }
    assert!(
        new_scope(None, ordinary.root().cloned(), CssScopeNestingContext::None)
            .unwrap_err()
            .is_invalid_boundary_anchor()
    );
    let moved = CssScopeRule::try_new(
        None,
        None,
        vec![CssRule::Scope(ordinary.clone())],
        &CssNamespaceContext::default(),
        CssScopeNestingContext::None,
    )
    .unwrap_err();
    assert!(moved.is_invalid_boundary_anchor(), "{moved:?}");
    assert_eq!(moved.path(), &[0]);
    assert_eq!(moved.position(), ordinary.position());
    assert!(
        new_scope(
            ordinary.root().cloned(),
            inner.limit().cloned(),
            CssScopeNestingContext::Style
        )
        .is_ok()
    );
}

#[test]
fn relative_payloads_reject_pseudo_elements_bad_identifiers_and_excess_function_depth() {
    let input = sheet(".item::before{}");
    let [CssRule::Style(style)] = input.rules() else {
        panic!("style")
    };
    let selector = style.selectors().selectors()[0].selector().clone();
    assert!(
        CssScopeSelectorList::try_new(vec![CssStyleSelector::Relative(CssRelativeSelector::new(
            CssSelectorCombinator::Child,
            selector
        ))])
        .is_none()
    );
    for nesting in [
        CssScopeNestingContext::None,
        CssScopeNestingContext::Style,
        CssScopeNestingContext::Scope,
    ] {
        let bad = relative(
            CssSelectorCombinator::Child,
            CssSelector::Class("\0".into()),
        );
        assert!(
            new_scope(None, Some(bad), nesting)
                .unwrap_err()
                .is_invalid_selector_identifier()
        );
    }
    let mut selector = CssSelector::Class("leaf".into());
    for _ in 0..256 {
        selector = CssSelector::PseudoClass(CssPseudoClass::Is(
            CssPseudoSelectorList::try_new(vec![selector], Default::default()).unwrap(),
        ));
    }
    assert!(
        new_scope(
            None,
            Some(relative(CssSelectorCombinator::Child, selector.clone())),
            CssScopeNestingContext::None
        )
        .is_ok()
    );
    selector = CssSelector::PseudoClass(CssPseudoClass::Is(
        CssPseudoSelectorList::try_new(vec![selector], Default::default()).unwrap(),
    ));
    assert!(
        new_scope(
            None,
            Some(relative(CssSelectorCombinator::Child, selector)),
            CssScopeNestingContext::None
        )
        .unwrap_err()
        .is_selector_nesting_limit()
    );
}

#[test]
fn parser_split_preserves_relative_members_and_exact_style_and_scope_ancestry() {
    for prefix in [".é{", "@scope(.outer){"] {
        let source = format!(
            "{prefix}{}@scope(> &&.root) to (+ &&.stop){{}}{}}}",
            "@media all{".repeat(70),
            "}".repeat(70)
        );
        let input = sheet(&source);
        assert!(CssSheet::try_from_rules(input.rules().to_vec()).is_ok());
        let normalized = input.normalize(Default::default()).unwrap();
        let rule = normalized
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Rule(rule)
                    if rule.position().is_some_and(|position| {
                        position.byte_offset().value() == source.find("@scope(>").unwrap()
                    }) =>
                {
                    Some(rule)
                }
                _ => None,
            })
            .unwrap();
        let CssRuleContextKindRef::Scope {
            root: Some(root),
            limit: Some(limit),
        } = rule.kind()
        else {
            panic!("scope boundaries")
        };
        for (list, combinator, nesting, scopes) in [
            (
                root,
                CssSelectorCombinator::Child,
                usize::from(prefix == ".é{") * 2,
                usize::from(prefix != ".é{") * 2,
            ),
            (limit, CssSelectorCombinator::NextSibling, 0, 2),
        ] {
            let [CssStyleSelector::Relative(relative)] = list.selectors() else {
                panic!("relative boundary")
            };
            assert_eq!(relative.combinator(), combinator);
            let CssSelector::Compound(compound) = relative.selector() else {
                panic!("explicit repeated anchors")
            };
            assert_eq!(compound.nesting_selectors(), nesting);
            assert_eq!(compound.scope_anchors(), scopes);
        }
        assert!(rule.parent().is_some());
    }
}

#[test]
fn mixed_ancestry_keeps_implicit_root_order_explicit_style_anchors_and_direct_runs_separate() {
    for (source, implicit_style) in [
        (
            ".parent{@scope(.outer){@scope(> &.root) to (+ &.stop){opacity:.5}}}",
            false,
        ),
        (
            "@scope(.outer){.parent{@scope(> &.root) to (+ &.stop){opacity:.5}}}",
            true,
        ),
    ] {
        let input = sheet(source);
        assert!(CssSheet::try_from_rules(input.rules().to_vec()).is_ok());
        let inner = match &input.rules()[0] {
            CssRule::Style(style) => {
                let [CssRule::Scope(scope)] = style.rules() else {
                    panic!("outer scope")
                };
                let [CssRule::Scope(inner)] = scope.rules().rules() else {
                    panic!("inner scope")
                };
                inner
            }
            CssRule::Scope(scope) => {
                let [CssRule::Style(style)] = scope.rules().rules() else {
                    panic!("scoped style")
                };
                let [CssRule::Scope(inner)] = style.rules() else {
                    panic!("inner scope")
                };
                inner
            }
            _ => panic!("style or scope"),
        };
        let before = inner.clone();
        for (list, combinator, nesting, scope) in [
            (inner.root().unwrap(), CssSelectorCombinator::Child, 1, 0),
            (
                inner.limit().unwrap(),
                CssSelectorCombinator::NextSibling,
                0,
                1,
            ),
        ] {
            let [CssStyleSelector::Relative(relative)] = list.selectors() else {
                panic!("relative")
            };
            assert_eq!(relative.combinator(), combinator);
            let CssSelector::Compound(compound) = relative.selector() else {
                panic!("authored & compound")
            };
            assert_eq!(compound.nesting_selectors(), nesting);
            assert_eq!(compound.scope_anchors(), scope);
        }
        let constructed = CssScopeRule::try_new(
            inner.root().cloned(),
            inner.limit().cloned(),
            inner.rules().rules().to_vec(),
            &CssNamespaceContext::default(),
            CssScopeNestingContext::Style,
        )
        .unwrap();
        assert_eq!(constructed.position(), None);
        assert_eq!(constructed.rules(), inner.rules());
        assert!(
            CssSheet::try_from_rules(vec![CssRule::Scope(constructed)])
                .unwrap_err()
                .is_invalid_placement()
        );
        assert!(
            CssScopeRule::try_new(
                inner.root().cloned(),
                inner.limit().cloned(),
                inner.rules().rules().to_vec(),
                &CssNamespaceContext::default(),
                CssScopeNestingContext::Scope
            )
            .unwrap_err()
            .is_invalid_boundary_anchor()
        );
        let normalized = input.normalize(Default::default()).unwrap();
        let inner_context = normalized
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Rule(rule) if rule.position() == inner.position() => Some(rule),
                _ => None,
            })
            .unwrap();
        let implicit_context = inner_context.parent().unwrap();
        assert_eq!(
            matches!(
                implicit_context.kind(),
                CssRuleContextKindRef::Style(_) | CssRuleContextKindRef::ScopedStyle(_)
            ),
            implicit_style
        );
        assert_eq!(
            matches!(implicit_context.kind(), CssRuleContextKindRef::Scope { .. }),
            !implicit_style
        );
        let parent_style = normalized
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Rule(rule)
                    if rule.position().is_some_and(|position| {
                        position.byte_offset().value() == source.find(".parent").unwrap()
                    }) =>
                {
                    match rule.kind() {
                        CssRuleContextKindRef::Style(selectors)
                        | CssRuleContextKindRef::ScopedStyle(selectors) => Some(selectors),
                        _ => None,
                    }
                }
                _ => None,
            })
            .unwrap();
        let run = normalized
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Rule(rule) => match rule.kind() {
                    CssRuleContextKindRef::NestedDeclarations(selectors) => Some((rule, selectors)),
                    _ => None,
                },
                _ => None,
            })
            .unwrap();
        assert!(run.0.parent().unwrap().same_context(inner_context));
        assert!(run.1.same_context(parent_style));
        assert_eq!(*inner, before);
    }
}
