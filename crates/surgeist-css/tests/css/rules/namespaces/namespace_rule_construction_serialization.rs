#![forbid(unsafe_code)]
//! The standalone namespace API shares the rule/sheet CSSOM writer; construction
//! keeps literal names and absent source positions independently of output.

use surgeist_css::{
    CssNamespaceName, CssNamespacePrefix, CssNamespaceRule, CssNormalizedItem, CssRule,
    CssRuleContextKindRef, CssSpecifiedRuleSerializationErrorKind,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits,
    normalize_report, parse_sheet,
};

#[test]
fn constructed_namespace_rules_emit_canonical_prefixes_and_literal_url_strings() {
    for (prefix, name, expected) in [
        (None, "", r#"@namespace url("");"#),
        (None, "not a URI", r#"@namespace url("not a URI");"#),
        (Some("svg"), "", r#"@namespace svg url("");"#),
        (
            Some("SVG"),
            "HTTP://EXAMPLE.TEST/a/../b%2f",
            r#"@namespace SVG url("HTTP://EXAMPLE.TEST/a/../b%2f");"#,
        ),
        (
            Some("1"),
            "urn:digit",
            r#"@namespace \31  url("urn:digit");"#,
        ),
        (Some("a b"), "a\"b\\c", r#"@namespace a\ b url("a\"b\\c");"#),
        (
            Some("*"),
            "line\nbreak",
            r#"@namespace \* url("line\a break");"#,
        ),
    ] {
        let rule = CssNamespaceRule::new(
            prefix.map(|prefix| CssNamespacePrefix::try_new(prefix).unwrap()),
            CssNamespaceName::new(name),
        );
        let original = rule.clone();
        assert_eq!(rule.to_specified_css().unwrap(), expected);
        assert_eq!(
            CssRule::Namespace(rule.clone()).to_specified_css().unwrap(),
            expected
        );
        assert_eq!(rule, original);
        assert_eq!(rule.position(), None);
        let reparsed = parse_sheet(expected);
        assert!(reparsed.is_clean(), "{expected}: {reparsed:?}");
        let [CssRule::Namespace(parsed)] = reparsed.syntax().rules() else {
            panic!("one namespace")
        };
        assert_eq!(parsed.prefix().map(CssNamespacePrefix::as_str), prefix);
        assert_eq!(parsed.name().as_str(), name);
        assert!(parsed.position().is_some());
        assert_eq!(parsed.to_specified_css().unwrap(), expected);
    }
}

#[test]
fn standalone_namespace_limits_charge_the_rule_optional_prefix_and_literal_name() {
    for prefix in [None, Some("a b")] {
        let rule = CssNamespaceRule::new(
            prefix.map(|prefix| CssNamespacePrefix::try_new(prefix).unwrap()),
            CssNamespaceName::new("a\"b\\c😀"),
        );
        let original = rule.clone();
        let expected = if prefix.is_some() {
            r#"@namespace a\ b url("a\"b\\c😀");"#
        } else {
            r#"@namespace url("a\"b\\c😀");"#
        };
        let nodes = if prefix.is_some() { 3 } else { 2 };
        let exact = CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len());
        assert_eq!(rule.to_specified_css_with_limits(exact).unwrap(), expected);
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(nodes - 1, nodes, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes - 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(nodes, nodes, 0),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                rule.to_specified_css_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            let error = CssRule::Namespace(rule.clone())
                .to_specified_css_with_limits(limits)
                .unwrap_err();
            assert_eq!(
                error.kind(),
                CssSpecifiedRuleSerializationErrorKind::Resource(kind)
            );
            assert_eq!(error.rule_index(), None);
            assert_eq!(rule, original);
        }
    }
}

#[test]
fn normalized_namespace_payloads_share_output_without_changing_redeclaration_provenance() {
    let source = "/*😀*/\r\n@namespace svg 'first';\r\n@namespace svg 'last';";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let original_diagnostics = report.diagnostics().to_vec();
    let normalized = normalize_report(&report).unwrap();
    assert_eq!(normalized.syntax().items().len(), 2);
    let expected = [
        r#"@namespace svg url("first");"#,
        r#"@namespace svg url("last");"#,
    ];
    for (index, item) in normalized.syntax().items().iter().enumerate() {
        let CssNormalizedItem::Rule(context) = item else {
            panic!("namespace occurrence")
        };
        let CssRuleContextKindRef::Namespace(namespace) = context.kind() else {
            panic!("namespace payload")
        };
        let original = namespace.clone();
        assert_eq!(namespace.to_specified_css().unwrap(), expected[index]);
        assert_eq!(namespace, &original);
        let position = namespace.position().unwrap();
        let offset = if index == 0 {
            source.find("@namespace").unwrap()
        } else {
            source.rfind("@namespace").unwrap()
        };
        assert_eq!(position.byte_offset().value(), offset);
        assert_eq!(position.line().value(), u32::try_from(index + 1).unwrap());
        assert_eq!(position.column().value(), 0);
    }
    assert_eq!(report.diagnostics(), original_diagnostics.as_slice());
    assert_eq!(normalized.diagnostics(), original_diagnostics.as_slice());
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        expected.join("\n")
    );
}

#[test]
fn cssom_replaces_programmatic_nul_in_output_without_changing_the_stored_literal_name() {
    let rule = CssNamespaceRule::new(None, CssNamespaceName::new("before\0after"));
    assert_eq!(
        rule.to_specified_css().unwrap(),
        "@namespace url(\"before�after\");"
    );
    assert_eq!(rule.name().as_str(), "before\0after");
    assert_eq!(rule.position(), None);
}

#[test]
fn namespaces_compose_with_other_supported_rules_under_one_shared_budget() {
    let report =
        parse_sheet("@namespace svg 'urn:svg';@supports-condition --feature { future:a; }");
    assert!(report.is_clean(), "{report:?}");
    let expected = "@namespace svg url(\"urn:svg\");\n@supports-condition --feature { future: a; }";
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    for (input, kind) in [
        (
            true,
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            false,
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        let limits = |nodes| {
            CssSpecifiedValueSerializationLimits::new(
                if input { nodes } else { usize::MAX },
                if input { usize::MAX } else { nodes },
                usize::MAX,
            )
        };
        let nodes = (1..32)
            .find(|nodes| {
                report
                    .syntax()
                    .rules()
                    .iter()
                    .all(|rule| rule.to_specified_css_with_limits(limits(*nodes)).is_ok())
                    && report
                        .syntax()
                        .to_specified_css_with_limits(limits(*nodes))
                        .is_err_and(|error| {
                            error.kind() == CssSpecifiedRuleSerializationErrorKind::Resource(kind)
                                && error.rule_index() == Some(1)
                        })
            })
            .expect("shared budget exhausts at supported sibling");
        assert_eq!(
            report
                .syntax()
                .to_specified_css_with_limits(limits(nodes))
                .unwrap_err()
                .rule_index(),
            Some(1)
        );
    }
}
