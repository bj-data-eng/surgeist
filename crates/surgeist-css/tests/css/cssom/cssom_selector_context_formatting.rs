#![forbid(unsafe_code)]
//! Independent CSSOM/Nesting/Namespaces expectations for pure formatting.
use surgeist_css::{
    CssAdmittedStyleSelectors, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix,
    CssRecoveryAction, CssRule, CssScopeNestingContext, CssScopeSelectorCssomContext,
    CssSpecifiedValueSerializationErrorKind as Kind,
    CssSpecifiedValueSerializationLimits as Limits, CssStyleAncestor, CssStyleSelectorContext,
    parse_relative_selector_list, parse_rule, parse_selector_list, parse_sheet,
    parse_style_selector_list,
};

fn namespaces(default: Option<&str>, named: &[(&str, &str)]) -> CssNamespaceContext {
    CssNamespaceContext::from_bindings(
        default
            .map(|uri| (None, CssNamespaceName::new(uri)))
            .into_iter()
            .chain(named.iter().map(|(prefix, uri)| {
                (
                    Some(CssNamespacePrefix::try_new(*prefix).unwrap()),
                    CssNamespaceName::new(*uri),
                )
            })),
    )
}

#[test]
fn actual_uri_identity_controls_type_prefixes_and_null_attributes() {
    // CSSOM simple-selector steps compare namespace identity, not prefix text.
    for (default, authored, expected) in [
        (None, "n|leaf[n|a]", "n|leaf[n|a]"),
        (Some("urn:x"), "n|leaf[n|a]", "leaf[n|a]"),
        (Some("urn:y"), "n|leaf[n|a]", "n|leaf[n|a]"),
        (Some(""), "n|leaf[n|a]", "n|leaf[n|a]"),
    ] {
        let context = namespaces(default, &[("n", "urn:x")]);
        let report = parse_selector_list(authored, &context);
        assert!(report.is_clean());
        let list = report.syntax().as_ref().unwrap();
        let before = list.clone();
        assert_eq!(
            list.serialize_cssom(&context, CssStyleSelectorContext::Ordinary)
                .unwrap(),
            expected
        );
        let surgeist_css::CssStyleSelector::Selector(member) = &list.selectors()[0] else {
            panic!("ordinary selector")
        };
        assert_eq!(member.to_specified_css().unwrap(), authored);
        assert_eq!(list, &before);
    }
    for default in [None, Some(""), Some("urn:x")] {
        let context = namespaces(default, &[("null", "")]);
        let report = parse_selector_list("null|leaf[null|a][|b][c]", &context);
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .serialize_cssom(&context, CssStyleSelectorContext::Ordinary)
                .unwrap(),
            "|leaf[a][b][c]"
        );
    }
    let empty = namespaces(Some(""), &[]);
    let report = parse_selector_list("leaf, *", &empty);
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_cssom(&empty, CssStyleSelectorContext::Ordinary)
            .unwrap(),
        "|leaf, |*"
    );
}

#[test]
fn aliases_last_binding_and_named_escape_are_distinct_from_any_wildcard() {
    let context = namespaces(
        Some("urn:x"),
        &[
            ("n", "old"),
            ("n", "urn:x"),
            ("alias", "urn:x"),
            ("1n", "urn:y"),
        ],
    );
    let report = parse_selector_list(
        r"n|leaf, alias|leaf, \31 n|leaf[\31 n|a], *|leaf[*|a]",
        &context,
    );
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_cssom(&context, CssStyleSelectorContext::Ordinary)
            .unwrap(),
        r"leaf, leaf, \31 n|leaf[\31 n|a], *|leaf[*|a]"
    );
}

#[test]
fn any_and_protected_universal_omission_remain_the_accepted_policy() {
    for default in [None, Some(""), Some("urn:x")] {
        let context = namespaces(default, &[("n", "urn:x")]);
        let report = parse_selector_list(
            "*|*.c[*|a], *|*::before, *|*:hover, *|*:is(*|*), *.c, *:is(*), n|*.c",
            &context,
        );
        assert!(report.is_clean());
        // Protected stars stay explicit. CSSOM simple-selector step2 still
        // gives each null-namespace universal its required `|` separator.
        let protected = if default == Some("") {
            "|*:is(|*)"
        } else {
            "*:is(*)"
        };
        let named = if default == Some("urn:x") {
            "*.c"
        } else {
            "n|*.c"
        };
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .serialize_cssom(&context, CssStyleSelectorContext::Ordinary)
                .unwrap(),
            format!("*|*.c[*|a], *|*::before, *|*:hover, *|*:is(*|*), .c, {protected}, {named}")
        );
    }
}

#[test]
fn missing_actual_bindings_are_distinct_and_detached_rules_preserve_symbolic_names() {
    let context = namespaces(Some("urn:x"), &[("n", "urn:y")]);
    for authored in ["n|leaf", "*.c", "[n|a]"] {
        let report = parse_selector_list(authored, &context);
        let list = report.syntax().as_ref().unwrap();
        assert_eq!(
            list.serialize_cssom(
                &CssNamespaceContext::default(),
                CssStyleSelectorContext::Ordinary
            )
            .unwrap_err()
            .kind(),
            Kind::NamespaceBindingUnavailable
        );
        assert!(
            list.serialize_cssom(&context, CssStyleSelectorContext::Ordinary)
                .is_ok()
        );
    }
    let report = parse_rule("n|leaf[n|a]{}", &context);
    assert_eq!(
        report.syntax().as_ref().unwrap().serialize_cssom().unwrap(),
        "n|leaf[n|a] { }"
    );
    let absent = CssNamespaceContext::default();
    let any = parse_selector_list("leaf, *.c", &absent);
    for incompatible in [namespaces(Some(""), &[]), namespaces(Some("urn:x"), &[])] {
        assert_eq!(
            any.syntax()
                .as_ref()
                .unwrap()
                .serialize_cssom(&incompatible, CssStyleSelectorContext::Ordinary)
                .unwrap_err()
                .kind(),
            Kind::UnrepresentableValue
        );
    }
}

#[test]
fn actual_sheet_bindings_apply_through_groups_and_do_not_leak_between_sheets() {
    let same =
        parse_sheet("@namespace 'urn:x'; @namespace n 'urn:x'; @media all { n|leaf[n|a]{} }");
    assert!(same.is_clean());
    assert_eq!(
        same.syntax().serialize_cssom().unwrap(),
        "@namespace url(\"urn:x\");\n@namespace n url(\"urn:x\");\n@media all {\n  leaf[n|a] { }\n}"
    );
    let other = parse_sheet("@namespace 'urn:y'; @namespace n 'urn:x'; n|leaf{}");
    assert!(
        other
            .syntax()
            .serialize_cssom()
            .unwrap()
            .ends_with("n|leaf { }")
    );
    assert!(
        same.syntax()
            .serialize_cssom()
            .unwrap()
            .ends_with("leaf[n|a] { }\n}")
    );
}

#[test]
fn supplied_role_not_selector_content_controls_implicit_descendants() {
    let context = CssNamespaceContext::default();
    let raw = " /*😀*/ .child, > .next, &:is(.x,&:unknown()) ";
    let report = parse_style_selector_list(raw, &context, CssStyleSelectorContext::Nested);
    let parsed = report.syntax().as_ref().unwrap();
    let before = parsed.clone();
    assert_eq!(
        parsed.serialize_cssom(&context).unwrap(),
        "& .child, & > .next, &:is(.x,&:unknown())"
    );
    assert_eq!(parsed.origin().source().as_str(), raw);
    assert_eq!(parsed, &before);
    let CssAdmittedStyleSelectors::Ordinary(list) = parsed.selectors() else {
        panic!("ordinary domain")
    };
    assert_eq!(
        list.serialize_cssom(&context, CssStyleSelectorContext::Ordinary)
            .unwrap_err()
            .kind(),
        Kind::UnrepresentableValue
    );
    assert_eq!(
        list.serialize_cssom(
            &context,
            CssStyleSelectorContext::Scoped(CssStyleAncestor::Present)
        )
        .unwrap_err()
        .kind(),
        Kind::UnrepresentableValue
    );
    for role in [
        CssStyleSelectorContext::Ordinary,
        CssStyleSelectorContext::Nested,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Absent),
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    ] {
        let parsed = parse_style_selector_list(".same", &context, role)
            .syntax()
            .clone()
            .unwrap();
        let expected = if matches!(
            role,
            CssStyleSelectorContext::Nested
                | CssStyleSelectorContext::Scoped(CssStyleAncestor::Present)
        ) {
            "& .same"
        } else {
            ".same"
        };
        assert_eq!(parsed.serialize_cssom(&context).unwrap(), expected);
    }
}

#[test]
fn scoped_retained_invalid_members_and_strict_rejection_preserve_real_origins() {
    let context = CssNamespaceContext::default();
    for role in [
        CssStyleSelectorContext::Nested,
        CssStyleSelectorContext::Scoped(CssStyleAncestor::Present),
    ] {
        let source = " :is(.ok,:unknown(&)) ";
        let report = parse_style_selector_list(source, &context, role);
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::PreserveInvalidSelectorListItem)
        );
        let parsed = report.syntax().as_ref().unwrap();
        assert_eq!(
            parsed.serialize_cssom(&context).unwrap(),
            ":is(.ok,:unknown(&))"
        );
        assert_eq!(parsed.origin().source().as_str(), source);
        assert!(
            parse_style_selector_list(":not(.ok,:unknown(&))", &context, role)
                .syntax()
                .is_none()
        );
        if let CssAdmittedStyleSelectors::Scoped(list) = parsed.selectors() {
            assert_eq!(
                list.serialize_cssom(&context, CssStyleSelectorContext::Nested)
                    .unwrap_err()
                    .kind(),
                Kind::UnrepresentableValue
            );
        }
    }
}

#[test]
fn logical_relative_arguments_and_general_relative_lists_keep_their_relationships() {
    let context = CssNamespaceContext::default();
    let general = parse_relative_selector_list(".a, > .b, + .c, ~ .d, || .e", &context);
    assert_eq!(
        general
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_cssom(&context)
            .unwrap(),
        ".a, > .b, + .c, ~ .d, || .e"
    );
    let parsed = parse_style_selector_list(
        ":has(> [|a], + *.c)",
        &context,
        CssStyleSelectorContext::Nested,
    );
    assert_eq!(
        parsed
            .syntax()
            .as_ref()
            .unwrap()
            .serialize_cssom(&context)
            .unwrap(),
        "& :has(> [a], + *.c)"
    );
}

#[test]
fn scope_bounds_format_independently_and_preserve_nullable_identity() {
    let context = CssNamespaceContext::default();
    let report = parse_sheet("@scope (.root) to (> [|a]) {} @scope {}");
    assert!(report.is_clean(), "{report:?}");
    let CssRule::Scope(rule) = &report.syntax().rules()[0] else {
        panic!("scope")
    };
    assert_eq!(
        rule.serialize_cssom_start(&context, CssScopeNestingContext::None)
            .unwrap(),
        Some(".root".into())
    );
    assert_eq!(
        rule.serialize_cssom_start(&context, CssScopeNestingContext::Style)
            .unwrap(),
        Some("& .root".into())
    );
    assert_eq!(
        rule.serialize_cssom_end(&context).unwrap(),
        Some("& > [a]".into())
    );
    assert_eq!(
        rule.limit()
            .unwrap()
            .serialize_cssom(
                &context,
                CssScopeSelectorCssomContext::Start(CssScopeNestingContext::None)
            )
            .unwrap_err()
            .kind(),
        Kind::UnrepresentableValue
    );
    let CssRule::Scope(omitted) = &report.syntax().rules()[1] else {
        panic!("scope")
    };
    assert_eq!(
        omitted
            .serialize_cssom_start_with_limits(
                &context,
                CssScopeNestingContext::None,
                Limits::new(0, 0, 0)
            )
            .unwrap(),
        None
    );
    assert_eq!(
        omitted
            .serialize_cssom_end_with_limits(&context, Limits::new(0, 0, 0))
            .unwrap(),
        None
    );
}

#[test]
fn cumulative_limits_include_implied_nodes_utf8_and_all_list_members_with_retry() {
    let context = CssNamespaceContext::default();
    let report = parse_style_selector_list(".é, > .b", &context, CssStyleSelectorContext::Nested);
    let parsed = report.syntax().as_ref().unwrap();
    let before = parsed.clone();
    let expected = "& .é, & > .b";
    assert_eq!(
        parsed
            .serialize_cssom_with_limits(&context, Limits::new(100, 100, expected.len()))
            .unwrap(),
        expected
    );
    assert_eq!(
        parsed
            .serialize_cssom_with_limits(&context, Limits::new(100, 100, expected.len() - 1))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(
        parsed
            .serialize_cssom_with_limits(&context, Limits::new(1, 100, 100))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        parsed
            .serialize_cssom_with_limits(&context, Limits::new(100, 1, 100))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
    assert_eq!(parsed.serialize_cssom(&context).unwrap(), expected);
    assert_eq!(parsed, &before);
    let general = parse_relative_selector_list(".a, > .b", &context)
        .syntax()
        .clone()
        .unwrap();
    assert_eq!(
        general
            .serialize_cssom_with_limits(&context, Limits::new(100, 100, 1))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    let ordinary = parse_selector_list("[|a], [|b]", &context)
        .syntax()
        .clone()
        .unwrap();
    assert_eq!(
        ordinary
            .serialize_cssom_with_limits(
                &context,
                CssStyleSelectorContext::Ordinary,
                Limits::new(100, 100, 7)
            )
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(
        ordinary
            .serialize_cssom(&context, CssStyleSelectorContext::Ordinary)
            .unwrap(),
        "[a], [b]"
    );
}

#[test]
fn grouping_inherits_real_style_ancestry_and_siblings_restore_ordinary_role() {
    let report = parse_sheet(
        ".parent { @media all { .child {} } .sibling {} } .outside {} @media all { .group {} }",
    );
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(
        report.syntax().serialize_cssom().unwrap(),
        ".parent {\n  @media all {\n  & .child { }\n}\n  & .sibling { }\n}\n.outside { }\n@media all {\n  .group { }\n}"
    );
}

#[test]
fn ordinary_checked_lists_and_scope_bounds_share_atomic_limits() {
    let context = namespaces(Some("urn:x"), &[("n", "urn:x")]);
    let input = parse_selector_list("n|leaf, [|a]", &context);
    let members = input
        .syntax()
        .as_ref()
        .unwrap()
        .selectors()
        .iter()
        .map(|member| {
            let surgeist_css::CssStyleSelector::Selector(selector) = member else {
                panic!("ordinary")
            };
            selector.clone()
        })
        .collect();
    let list = surgeist_css::CssSelectorList::try_new(members).unwrap();
    let before = list.clone();
    assert_eq!(list.serialize_cssom(&context).unwrap(), "leaf, [a]");
    assert_eq!(
        list.serialize_cssom_with_limits(&context, Limits::new(100, 100, 8))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(
        list.serialize_cssom_with_limits(&context, Limits::new(100, 1, 100))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
    assert_eq!(list.serialize_cssom(&context).unwrap(), "leaf, [a]");
    assert_eq!(list, before);

    let report = parse_sheet("@scope to (> [|a]) {}");
    let CssRule::Scope(scope) = &report.syntax().rules()[0] else {
        panic!("scope")
    };
    let empty = CssNamespaceContext::default();
    let before = scope.clone();
    assert_eq!(
        scope
            .serialize_cssom_end_with_limits(&empty, Limits::new(100, 100, 7))
            .unwrap(),
        Some("& > [a]".into())
    );
    assert_eq!(
        scope
            .serialize_cssom_end_with_limits(&empty, Limits::new(100, 100, 6))
            .unwrap_err()
            .kind(),
        Kind::ByteLimit
    );
    assert_eq!(
        scope
            .serialize_cssom_end_with_limits(&empty, Limits::new(1, 100, 100))
            .unwrap_err()
            .kind(),
        Kind::InputNodeLimit
    );
    assert_eq!(
        scope
            .serialize_cssom_end_with_limits(&empty, Limits::new(100, 1, 100))
            .unwrap_err()
            .kind(),
        Kind::ProjectionNodeLimit
    );
    assert_eq!(scope, &before);
}

#[test]
fn contextual_sheet_failure_retains_the_actual_child_path_and_atomic_retry() {
    let report = parse_sheet("@namespace 'urn:x'; @namespace n 'urn:x'; @media all {n|a{} n|b{}}");
    assert!(report.is_clean());
    let before = report.clone();
    let prefix =
        "@namespace url(\"urn:x\");\n@namespace n url(\"urn:x\");\n@media all {\n  a { }\n  ";
    let expected = format!("{prefix}b {{ }}\n}}");
    let error = report
        .syntax()
        .serialize_cssom_with_limits(Limits::new(100, 100, prefix.len()))
        .unwrap_err();
    assert_eq!(error.rule_path(), &[2, 1]);
    assert_eq!(
        error.kind(),
        surgeist_css::CssRuleCssomSerializationErrorKind::Resource(Kind::ByteLimit)
    );
    assert_eq!(report.syntax().serialize_cssom().unwrap(), expected);
    assert_eq!(report, before);
}
