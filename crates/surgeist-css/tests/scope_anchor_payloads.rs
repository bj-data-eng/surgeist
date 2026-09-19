#![forbid(unsafe_code)]
//! Counted authored anchors preserve the distinct Nesting 1 root and Cascade 6
//! limit/body categories, including functional members and detached payloads.
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nested-scope-rules
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nest-selector
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-bounds
use surgeist_css::*;

fn compound(selector: &CssSelector) -> &CssCompoundSelector {
    let CssSelector::Compound(compound) = selector else {
        panic!("expected authored compound")
    };
    compound
}

fn counts(selector: &CssSelector, nesting: usize, scope: usize, present: bool) {
    let compound = compound(selector);
    assert_eq!(compound.nesting_selectors(), nesting);
    assert_eq!(compound.scope_anchors(), scope);
    assert_eq!(compound.has_scope_anchor(), present);
}

fn single(list: &CssScopeSelectorList) -> &CssSelector {
    let [selector] = list.selectors() else {
        panic!("one boundary member")
    };
    selector
}

fn scope_context(normalized: &CssNormalizedSheet, offset: usize) -> &CssRuleContext {
    normalized
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(context)
                if context
                    .position()
                    .is_some_and(|position| position.byte_offset().value() == offset) =>
            {
                Some(context)
            }
            _ => None,
        })
        .expect("scope at original source position")
}

fn check_three(list: &CssScopeSelectorList, nesting: bool) {
    let [zero, one, multiple] = list.selectors() else {
        panic!("all three authored members retained")
    };
    counts(zero, 0, 0, false);
    if nesting {
        counts(one, 1, 0, false);
        counts(multiple, 3, 0, false);
    } else {
        counts(one, 0, 1, true);
        counts(multiple, 0, 3, true);
    }
    for selector in [zero, one, multiple] {
        assert_eq!(compound(selector).pseudo_classes(), [CssPseudoClass::Scope]);
    }
}

#[test]
fn root_limit_and_body_counts_survive_clone_and_normalization() {
    for prefix in ["", ".parent{", "@scope(.outer){"] {
        let suffix = if prefix.is_empty() { "" } else { "}" };
        let source = format!(
            "{prefix}@scope(.zero:scope,&:scope,&&&:scope) to (.zero:scope,&:scope,&&&:scope){{.zero:scope,&:scope,&&&:scope{{color:red}}}}{suffix}"
        );
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let inner = match &report.syntax().rules()[0] {
            CssRule::Scope(outer) if !prefix.is_empty() => {
                let [CssScopedRule::Scope(inner)] = outer.rules().rules() else {
                    panic!("inner scope")
                };
                inner
            }
            CssRule::Scope(scope) => scope,
            CssRule::Style(parent) => {
                let [CssRule::Scope(inner)] = parent.rules() else {
                    panic!("style-nested scope")
                };
                inner
            }
            _ => panic!("scope or parent"),
        };
        let nesting = prefix != "@scope(.outer){";
        check_three(inner.root().unwrap(), nesting);
        check_three(inner.limit().unwrap(), false);
        let [CssScopedRule::Style(body)] = inner.rules().rules() else {
            panic!("scoped body style")
        };
        let body_members = body.selectors().selectors();
        assert_eq!(body_members.len(), 3);
        for (member, expected, present) in [
            (&body_members[0], 0, false),
            (&body_members[1], 1, true),
            (&body_members[2], 3, true),
        ] {
            let CssScopedStyleSelector::Selector(selector) = member else {
                panic!("nonrelative body selector")
            };
            counts(selector, 0, expected, present);
            assert_eq!(compound(selector).pseudo_classes(), [CssPseudoClass::Scope]);
        }
        let detached = inner.clone();
        let normalized = normalize_sheet(report.syntax()).unwrap();
        let offset = source.find("@scope(.zero").unwrap();
        let CssRuleContextKindRef::Scope {
            root: Some(root),
            limit: Some(limit),
        } = scope_context(&normalized, offset).kind()
        else {
            panic!("normalized bounds")
        };
        check_three(root, nesting);
        check_three(limit, false);
        let normalized_body = normalized
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Rule(context) => match context.kind() {
                    CssRuleContextKindRef::ScopedStyle(selectors) => Some(selectors),
                    _ => None,
                },
                _ => None,
            })
            .unwrap();
        let selectors = normalized_body.selectors();
        assert_eq!(selectors.len(), 3);
        counts(selectors[0].selector(), 0, 0, false);
        counts(selectors[1].selector(), 0, 1, true);
        counts(selectors[2].selector(), 0, 3, true);
        drop(report);
        drop(normalized);
        check_three(detached.root().unwrap(), nesting);
        check_three(detached.limit().unwrap(), false);
    }
}

fn function_members(selector: &CssSelector) -> &[CssSelector] {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::Is(list)) => list.selectors(),
        CssSelector::PseudoClass(CssPseudoClass::Not(list)) => list.selectors(),
        _ => panic!("is or not function"),
    }
}

#[test]
fn functional_members_retain_zero_one_and_multiple_anchors_in_each_context() {
    let source = "@scope(.outer){@scope(:is(.zero:scope,&:scope,&&&:scope)) to (:not(.zero:scope,&:scope,&&&:scope)){:is(.zero:scope,&:scope,&&&:scope){color:red}}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let CssRuleContextKindRef::Scope {
        root: Some(root),
        limit: Some(limit),
    } = scope_context(&normalized, source.find("@scope(:is").unwrap()).kind()
    else {
        panic!("inner normalized scope")
    };
    for selector in [single(root), single(limit)] {
        let [zero, one, multiple] = function_members(selector) else {
            panic!("every function member retained")
        };
        counts(zero, 0, 0, false);
        counts(one, 0, 1, true);
        counts(multiple, 0, 3, true);
        let detached = selector.clone();
        let [_, _, multiple] = function_members(&detached) else {
            panic!("cloned function members")
        };
        counts(multiple, 0, 3, true);
    }
    let body = normalized
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Rule(context) => match context.kind() {
                CssRuleContextKindRef::ScopedStyle(selectors) => Some(selectors),
                _ => None,
            },
            _ => None,
        })
        .unwrap();
    let [member] = body.selectors() else {
        panic!("body function")
    };
    let [zero, one, multiple] = function_members(member.selector()) else {
        panic!("body function members")
    };
    counts(zero, 0, 0, false);
    counts(one, 0, 1, true);
    counts(multiple, 0, 3, true);
}

#[test]
fn style_nested_function_root_counts_remain_separate_from_scope_limit_counts() {
    let report = parse_sheet(
        ".parent{@scope(:is(.zero:scope,&:scope,&&&:scope)) to (:not(.zero:scope,&:scope,&&&:scope)){.child{color:red}}}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(parent)] = report.syntax().rules() else {
        panic!("parent")
    };
    let [CssRule::Scope(scope)] = parent.rules() else {
        panic!("scope")
    };
    let [zero, one, multiple] = function_members(single(scope.root().unwrap())) else {
        panic!("three root members")
    };
    counts(zero, 0, 0, false);
    counts(one, 1, 0, false);
    counts(multiple, 3, 0, false);
    let [zero, one, multiple] = function_members(single(scope.limit().unwrap())) else {
        panic!("three limit members")
    };
    counts(zero, 0, 0, false);
    counts(one, 0, 1, true);
    counts(multiple, 0, 3, true);
}

fn nested_function(depth: usize) -> String {
    format!("{}&&{}", ":is(".repeat(depth), ")".repeat(depth))
}

fn deep_leaf(mut selector: &CssSelector, depth: usize, nesting: usize, scope: usize) {
    for _ in 0..depth {
        let [member] = function_members(selector) else {
            panic!("single retained function member")
        };
        selector = member;
    }
    counts(selector, nesting, scope, scope != 0);
}

#[test]
fn scope_boundary_parenthesis_and_functions_share_the_structural_ceiling() {
    // One boundary parenthesis plus 255 selector functions reaches 256.
    // The rule body is a later sibling block, not an enclosing prelude block.
    for limit in [false, true] {
        let prefix = if limit {
            "@scope(.root) to ("
        } else {
            "@scope("
        };
        let accepted = format!(
            "{prefix}{}){{.child{{color:red}}}}.after{{color:blue}}",
            nested_function(255)
        );
        let report = parse_sheet(&accepted);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Scope(scope), CssRule::Style(after)] = report.syntax().rules() else {
            panic!("accepted scope and following neighbor")
        };
        let boundary = if limit { scope.limit() } else { scope.root() }.unwrap();
        deep_leaf(
            single(boundary),
            255,
            if limit { 0 } else { 2 },
            if limit { 2 } else { 0 },
        );
        assert_eq!(
            after.position().byte_offset().value(),
            accepted.find(".after").unwrap()
        );
        let exceeded = format!(
            "{prefix}{}){{.child{{color:red}}}}.after{{color:blue}}",
            nested_function(256)
        );
        let report = parse_sheet(&exceeded);
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected over-limit scope: {:?}", report.diagnostics())
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
        assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit);
        // cssparser's function token starts after the pseudo-class colon.
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            prefix.len() + 255 * 4 + 1
        );
        let [CssRule::Style(after)] = report.syntax().rules() else {
            panic!("only following neighbor retained")
        };
        assert_eq!(
            after.position().byte_offset().value(),
            exceeded.find(".after").unwrap()
        );
        assert_eq!(
            after.declarations()[0].known().unwrap().property(),
            CssKnownProperty::Color
        );
    }
}

#[test]
fn deep_root_and_limit_payloads_remain_owned_after_parent_drop_on_two_mib_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let function = nested_function(255);
            let source = format!("@scope({function}) to ({function}){{.child{{color:red}}}}");
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{:?}", report.diagnostics());
            let [CssRule::Scope(scope)] = report.syntax().rules() else {
                panic!("scope")
            };
            let detached_root = single(scope.root().unwrap()).clone();
            let detached_limit = single(scope.limit().unwrap()).clone();
            let cloned_sheet = report.syntax().clone();
            let normalized = normalize_sheet(report.syntax()).unwrap();
            let CssRuleContextKindRef::Scope {
                root: Some(root),
                limit: Some(limit),
            } = scope_context(&normalized, 0).kind()
            else {
                panic!("normalized scope")
            };
            let normalized_root = single(root).clone();
            let normalized_limit = single(limit).clone();
            drop(report);
            drop(cloned_sheet);
            drop(normalized);
            for root in [detached_root, normalized_root] {
                deep_leaf(&root, 255, 2, 0);
                let clone = root.clone();
                drop(root);
                deep_leaf(&clone, 255, 2, 0);
                drop(clone);
            }
            for limit in [detached_limit, normalized_limit] {
                deep_leaf(&limit, 255, 0, 2);
                let clone = limit.clone();
                drop(limit);
                deep_leaf(&clone, 255, 0, 2);
                drop(clone);
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
