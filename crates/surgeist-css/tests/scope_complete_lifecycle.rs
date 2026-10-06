#![forbid(unsafe_code)]
//! Scope lifecycle: Cascade6 2024-09-06 §2.5.2/2.5.4/2.5.5 and Nesting1 2026-01-22 §3.2/3.3.
//! https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-scope
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#syntax-examples
//! Contextual relative bounds follow frozen WebKit 73aa6c89e2cb77c46184a81aec944e4ab99d114d:
//! CSSParser.cpp:1148-1208 and CSSSelectorParser.cpp:83-105,358-367,426-440.
//! Exact output whitespace proposes the existing canonical rule convention;
//! omission, selector/child order and declaration runs are authored contracts.

use surgeist_css::*;

fn sheet(source: &str) -> CssSheet {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

fn scope(source: &str) -> CssScopeRule {
    let input = sheet(source);
    let [CssRule::Scope(rule)] = input.rules() else {
        panic!("one scope for {source}");
    };
    rule.clone()
}

fn classes(names: &[&str]) -> CssScopeSelectorList {
    CssScopeSelectorList::try_new(
        names
            .iter()
            .map(|name| CssScopeSelector::Selector(CssSelector::Class((*name).into())))
            .collect(),
    )
    .unwrap()
}

fn assert_classes(list: Option<&CssScopeSelectorList>, names: Option<&[&str]>) {
    assert_eq!(list.is_some(), names.is_some());
    if let Some(names) = names {
        assert_eq!(list.unwrap(), &classes(names));
    }
}

fn assert_last_scope_bound_count(input: &CssSheet, source: &str, is_root: bool, count: usize) {
    let normalized = normalize_sheet(input).unwrap();
    let target = source.rfind("@scope").unwrap();
    let bounds: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| {
            let CssNormalizedItem::Rule(rule) = item else {
                return None;
            };
            if rule
                .position()
                .is_none_or(|position| position.byte_offset().value() != target)
            {
                return None;
            }
            let CssRuleContextKindRef::Scope { root, limit } = rule.kind() else {
                return None;
            };
            if is_root { root } else { limit }
        })
        .collect();
    let [bound] = bounds.as_slice() else {
        panic!("one retained boundary at target scope")
    };
    assert_eq!(bound.selectors().len(), count);
}

#[test]
fn parsed_scope_preserves_all_boundary_omissions_and_selector_order() {
    for (source, roots, limits) in [
        ("@scope{}", None, None),
        (
            "@scope(.root,.alternate){}",
            Some(&["root", "alternate"][..]),
            None,
        ),
        (
            "@scope TO (.stop,.other){}",
            None,
            Some(&["stop", "other"][..]),
        ),
        (
            "@scope(.root,.alternate)to (.stop,.other){}",
            Some(&["root", "alternate"][..]),
            Some(&["stop", "other"][..]),
        ),
    ] {
        let input = scope(source);
        assert_classes(input.root(), roots);
        assert_classes(input.limit(), limits);
        assert!(input.rules().rules().is_empty());
        assert_eq!(input.position().unwrap().byte_offset().value(), 0);
        let fragment = parse_rule(source, &CssNamespaceContext::default());
        assert!(
            fragment.is_clean(),
            "{source}: {:?}",
            fragment.diagnostics()
        );
        assert_eq!(fragment.syntax(), &Some(CssRule::Scope(input)));
    }
}

#[test]
fn checked_scope_preserves_boundary_omissions_and_descendant_origins() {
    let source = "@scope(.root)to (.stop){@scope(.inner){.child{opacity:.5}}}";
    let parsed = scope(source);
    let before = parsed.clone();
    for (root, limit) in [
        (None, None),
        (Some(classes(&["root", "alternate"])), None),
        (None, Some(classes(&["stop", "other"]))),
        (
            Some(classes(&["root", "alternate"])),
            Some(classes(&["stop", "other"])),
        ),
    ] {
        let value = CssScopeRule::try_new(
            root.clone(),
            limit.clone(),
            parsed.rules().rules().to_vec(),
            &CssNamespaceContext::default(),
            CssScopeNestingContext::None,
        )
        .unwrap();
        assert_eq!(value.root(), root.as_ref());
        assert_eq!(value.limit(), limit.as_ref());
        assert_eq!(value.position(), None);
        assert_eq!(value.rules(), parsed.rules());
        let composed = CssSheet::try_from_rules(vec![CssRule::Scope(value)]).unwrap();
        let [CssRule::Scope(outer)] = composed.rules() else {
            panic!("constructed scope")
        };
        let [CssScopedRule::Scope(inner)] = outer.rules().rules() else {
            panic!("parsed descendant")
        };
        assert_eq!(
            inner.position().unwrap().byte_offset().value(),
            source.find("@scope(.inner)").unwrap()
        );
    }
    assert_eq!(parsed, before);
}

#[test]
fn scope_relative_limits_are_admitted_in_ordinary_style_and_scope_contexts() {
    for combinator in [">", "+", "~"] {
        let bound = format!("{combinator} .stop, :scope .other");
        for source in [
            format!("@scope to ({bound}){{}}"),
            format!("@scope(.root)to ({bound}){{}}"),
            format!(".parent{{@scope(.root)to ({bound}){{}}}}"),
            format!("@scope(.outer){{@scope(.root)to ({bound}){{}}}}"),
        ] {
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().rules().len(), 1);
            assert_last_scope_bound_count(report.syntax(), &source, false, 2);
        }
    }
}

#[test]
fn scope_relative_roots_require_style_or_scope_ancestry() {
    for combinator in [">", "+", "~"] {
        for source in [
            format!(".parent{{@scope({combinator} .root){{}}}}"),
            format!("@scope(.outer){{@scope({combinator} .root){{}}}}"),
        ] {
            let report = parse_sheet(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().rules().len(), 1);
            assert_last_scope_bound_count(report.syntax(), &source, true, 1);
        }
        let source = format!("@scope({combinator} .root){{}}.after{{}}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{source}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|item| item.action() == CssRecoveryAction::DropAtRule)
        );
        let [CssRule::Style(after)] = report.syntax().rules() else {
            panic!("surviving neighbor")
        };
        assert_eq!(
            after.position().byte_offset().value(),
            source.find(".after").unwrap()
        );
    }
}

#[test]
fn malformed_and_pseudo_element_boundaries_drop_scope_without_losing_neighbors() {
    for prelude in [
        "()",
        "(.root,)",
        "to ()",
        "(.root)to",
        "(.root)to (.stop,)",
        "(.root)(.other)",
        "(.root)to (.stop)junk",
        "(.root::before)",
        "(.root)to (.stop::after)",
        "to (>.stop::before)",
    ] {
        for source in [
            format!("@scope{prelude}{{.lost{{}}}}.after{{}}"),
            format!(".parent{{@scope{prelude}{{.lost{{}}}}.after{{}}}}"),
            format!("@scope(.outer){{@scope{prelude}{{.lost{{}}}}.after{{}}}}"),
        ] {
            let report = parse_sheet(&source);
            assert!(!report.is_clean(), "{source}");
            assert!(
                report
                    .diagnostics()
                    .iter()
                    .any(|item| item.action() == CssRecoveryAction::DropAtRule),
                "{source}: {:?}",
                report.diagnostics()
            );
            let normalized = normalize_report(&report).unwrap();
            let styles: Vec<_> = normalized
                .syntax()
                .items()
                .iter()
                .filter_map(|item| {
                    let CssNormalizedItem::Rule(rule) = item else {
                        return None;
                    };
                    match rule.kind() {
                        CssRuleContextKindRef::Style(_) | CssRuleContextKindRef::ScopedStyle(_) => {
                            Some(rule)
                        }
                        _ => None,
                    }
                })
                .collect();
            assert!(
                styles.iter().any(|rule| rule
                    .position()
                    .is_some_and(
                        |position| position.byte_offset().value() == source.find(".after").unwrap()
                    ))
            );
            assert!(
                !styles.iter().any(|rule| rule
                    .position()
                    .is_some_and(
                        |position| position.byte_offset().value() == source.find(".lost").unwrap()
                    ))
            );
        }
    }
}

#[test]
fn checked_boundary_lists_reject_empty_and_recursive_pseudo_elements() {
    assert!(CssScopeSelectorList::try_new(Vec::new()).is_none());
    let parsed = sheet(".root::before{}");
    let [CssRule::Style(style)] = parsed.rules() else {
        panic!("pseudo-element style")
    };
    let [CssStyleSelector::Selector(selector)] = style.selectors().selectors() else {
        panic!("selector")
    };
    // The recursive Is/Not(PE) boundary defenses remain in owning private tests;
    // these graphs cannot be admitted through the corrected public list front.
    assert!(CssPseudoSelectorList::try_new(vec![selector.clone()]).is_err());
    assert!(
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(selector.clone())]).is_none()
    );
    assert!(
        CssScopeSelectorList::try_new(vec![CssScopeSelector::Selector(CssSelector::PseudoClass(
            CssPseudoClass::Scope
        ))])
        .is_some()
    );
}

#[test]
fn contextual_relative_boundaries_still_reject_pseudo_elements() {
    for combinator in [">", "+", "~"] {
        for prelude in [
            format!("({combinator} .root::before)"),
            format!("(.root)to ({combinator} .stop::after)"),
        ] {
            for source in [
                format!(".parent{{@scope{prelude}{{}}}}"),
                format!("@scope(.outer){{@scope{prelude}{{}}}}"),
            ] {
                let report = parse_sheet(&source);
                assert!(!report.is_clean(), "{source}");
                assert!(
                    report
                        .diagnostics()
                        .iter()
                        .any(|item| item.action() == CssRecoveryAction::DropAtRule),
                    "{source}: {:?}",
                    report.diagnostics()
                );
            }
        }
    }
}

#[test]
fn ordinary_scope_rejects_declarations_while_style_nested_scope_retains_runs() {
    let source = "@scope{.child{}opacity:.5;}";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    let [CssRule::Scope(outer)] = report.syntax().rules() else {
        panic!("scope")
    };
    assert!(matches!(outer.rules().rules(), [CssScopedRule::Style(_)]));
    let styled = sheet(".parent{@scope{opacity:.25;.child{}opacity:.5;}}");
    let [CssRule::Style(parent)] = styled.rules() else {
        panic!("parent")
    };
    let [CssRule::Scope(outer)] = parent.rules() else {
        panic!("scope")
    };
    let [
        CssScopedRule::NestedDeclarations(first),
        CssScopedRule::Style(child),
        CssScopedRule::NestedDeclarations(last),
    ] = outer.rules().rules()
    else {
        panic!("two separate runs around child")
    };
    assert_eq!(first.declarations().len(), 1);
    assert_eq!(last.declarations().len(), 1);
    assert!(child.declarations().is_empty());
    assert_eq!(
        first.declarations()[0].known().unwrap().property(),
        CssKnownProperty::Opacity
    );
    assert_eq!(
        first.declarations()[0]
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        ".25"
    );
    assert_eq!(
        last.declarations()[0]
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        ".5"
    );
    assert_eq!(
        CssSheet::try_from_rules(styled.rules().to_vec())
            .unwrap()
            .rules(),
        styled.rules()
    );
    let detached = CssScopeRule::try_new(
        None,
        None,
        outer.rules().rules().to_vec(),
        &CssNamespaceContext::default(),
        CssScopeNestingContext::None,
    )
    .unwrap_err();
    assert_eq!(
        detached.kind(),
        CssRuleConstructionErrorKind::InvalidPlacement
    );
    assert_eq!(detached.path(), &[0]);
}

#[test]
fn parsed_and_checked_scope_serialize_every_boundary_omission_explicitly() {
    for (source, expected, root, limit) in [
        ("@scope{}", "@scope { }", None, None),
        (
            "@scope(.root){}",
            "@scope (.root) { }",
            Some(classes(&["root"])),
            None,
        ),
        (
            "@scope to (.stop){}",
            "@scope to (.stop) { }",
            None,
            Some(classes(&["stop"])),
        ),
        (
            "@scope(.root,.alternate)to (.stop,.other){}",
            "@scope (.root, .alternate) to (.stop, .other) { }",
            Some(classes(&["root", "alternate"])),
            Some(classes(&["stop", "other"])),
        ),
    ] {
        let parsed = CssRule::Scope(scope(source));
        let constructed = CssRule::Scope(
            CssScopeRule::try_new(
                root,
                limit,
                Vec::new(),
                &CssNamespaceContext::default(),
                CssScopeNestingContext::None,
            )
            .unwrap(),
        );
        for value in [parsed, constructed] {
            let before = value.clone();
            assert_eq!(value.to_specified_css().unwrap(), expected);
            assert_eq!(value, before);
            let output = scope(expected);
            let CssRule::Scope(original) = &value else {
                unreachable!()
            };
            assert_eq!(output.root(), original.root());
            assert_eq!(output.limit(), original.limit());
            assert!(output.rules().rules().is_empty());
        }
    }
}

#[test]
fn scope_serialization_preserves_nested_rule_order_and_authored_origins() {
    let source = "@scope(.root)to (.stop){@scope(.first){}@scope to (.second){}}";
    let parsed = scope(source);
    let constructed = CssScopeRule::try_new(
        parsed.root().cloned(),
        parsed.limit().cloned(),
        parsed.rules().rules().to_vec(),
        &CssNamespaceContext::default(),
        CssScopeNestingContext::None,
    )
    .unwrap();
    let expected = "@scope (.root) to (.stop) { @scope (.first) { } @scope to (.second) { } }";
    for value in [parsed, constructed] {
        let before = value.clone();
        let text = CssRule::Scope(value.clone()).to_specified_css().unwrap();
        assert_eq!(text, expected);
        assert_eq!(value, before);
        let output = scope(&text);
        let [CssScopedRule::Scope(first), CssScopedRule::Scope(second)] = output.rules().rules()
        else {
            panic!("authored nested order")
        };
        assert_classes(first.root(), Some(&["first"]));
        assert!(first.limit().is_none());
        assert!(second.root().is_none());
        assert_classes(second.limit(), Some(&["second"]));
    }
}

#[test]
fn style_nested_scope_serialization_preserves_direct_declaration_run_order() {
    let input = sheet(".parent{@scope{opacity:.25;.child{}opacity:.5;}}");
    let before = input.clone();
    assert_eq!(
        input.to_specified_css().unwrap(),
        ".parent { @scope { opacity: 0.25; .child { } opacity: 0.5; } }"
    );
    assert_eq!(input, before);
}

#[test]
fn scope_serialization_byte_limits_are_atomic_and_cumulative_across_sheet_rules() {
    let first = CssRule::Scope(
        CssScopeRule::try_new(
            None,
            None,
            Vec::new(),
            &CssNamespaceContext::default(),
            CssScopeNestingContext::None,
        )
        .unwrap(),
    );
    let input = CssSheet::try_from_rules(vec![first.clone(), first]).unwrap();
    let before = input.clone();
    let expected = "@scope { }\n@scope { }";
    assert_eq!(
        input
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                65536,
                65536,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    let error = input
        .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
            65536,
            65536,
            expected.len() - 1,
        ))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedRuleSerializationErrorKind::Resource(
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        )
    );
    assert_eq!(error.rule_index(), Some(1));
    assert_eq!(input, before);
    assert_eq!(input.to_specified_css().unwrap(), expected);
}
