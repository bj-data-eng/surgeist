#![forbid(unsafe_code)]
//! Existing rule/sheet boundaries must serialize retained namespace declarations
//! using the selected CSSOM namespace-rule spelling, atomically and in order.

use surgeist_css::{
    CssErrorCode, CssNamespacePrefix, CssNormalizedItem, CssRule, CssRuleContextKindRef,
    CssSpecifiedRuleSerializationErrorKind, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, normalize_report, parse_sheet, validate_sheet,
};

#[test]
fn namespace_rule_output_preserves_literal_names_and_escaped_prefixes() {
    for (source, expected, prefix, name) in [
        (r#"@namespace '';"#, r#"@namespace url("");"#, None, ""),
        (
            r#"@namespace svg 'not a URI';"#,
            r#"@namespace svg url("not a URI");"#,
            Some("svg"),
            "not a URI",
        ),
        (
            r#"@namespace SVG url(HTTP://EXAMPLE.TEST/a/../b%2f);"#,
            r#"@namespace SVG url("HTTP://EXAMPLE.TEST/a/../b%2f");"#,
            Some("SVG"),
            "HTTP://EXAMPLE.TEST/a/../b%2f",
        ),
        (
            r#"@namespace \31 'urn:digit';"#,
            r#"@namespace \31  url("urn:digit");"#,
            Some("1"),
            "urn:digit",
        ),
        (
            r#"@namespace a\ b 'a\"b\\c';"#,
            r#"@namespace a\ b url("a\"b\\c");"#,
            Some("a b"),
            "a\"b\\c",
        ),
    ] {
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        let [original] = report.syntax().rules() else {
            panic!("one namespace")
        };
        let CssRule::Namespace(original_namespace) = original else {
            panic!("namespace")
        };
        let original_position = original_namespace.position();
        let output = original
            .to_specified_css()
            .expect("supported namespace rule");
        assert_eq!(output, expected);
        let reparsed = parse_sheet(&output);
        assert!(reparsed.is_clean(), "{output}: {reparsed:?}");
        let [CssRule::Namespace(namespace)] = reparsed.syntax().rules() else {
            panic!("one namespace")
        };
        assert_eq!(namespace.prefix().map(CssNamespacePrefix::as_str), prefix);
        assert_eq!(namespace.name().as_str(), name);
        assert_eq!(namespace.prefix(), original_namespace.prefix());
        assert_eq!(namespace.name(), original_namespace.name());
        assert_eq!(original_namespace.position(), original_position);
    }
}

#[test]
fn namespace_sheet_and_normalized_output_retain_duplicate_order_diagnostics_and_positions() {
    let source = "/*😀*/\r\n@namespace 'first';\r\n@namespace svg 'named';\r\n@namespace 'last';\r\n@namespace svg '';";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 2);
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::NamespaceRedeclaration
        );
    }
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let expected = "@namespace url(\"first\");\n@namespace svg url(\"named\");\n@namespace url(\"last\");\n@namespace svg url(\"\");";
    assert_eq!(
        report
            .syntax()
            .to_specified_css()
            .expect("namespace-only sheet"),
        expected
    );
    let normalized = normalize_report(&report).unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let mut emitted = Vec::new();
    for (index, item) in normalized.syntax().items().iter().enumerate() {
        let CssNormalizedItem::Rule(context) = item else {
            panic!("namespace rule occurrence")
        };
        let CssRuleContextKindRef::Namespace(namespace) = context.kind() else {
            panic!("namespace payload")
        };
        let CssRule::Namespace(original) = &report.syntax().rules()[index] else {
            panic!("namespace source")
        };
        assert_eq!(namespace, original);
        assert_eq!(
            namespace.position().unwrap().line().value(),
            u32::try_from(index + 1).unwrap()
        );
        assert_eq!(namespace.position().unwrap().column().value(), 0);
        emitted.push(
            CssRule::Namespace(namespace.clone())
                .to_specified_css()
                .expect("normalized namespace payload"),
        );
    }
    assert_eq!(emitted.join("\n"), expected);
    assert_eq!(normalized.diagnostics(), report.diagnostics());
}

#[test]
fn namespace_output_uses_shared_cumulative_resource_budgets() {
    let report = parse_sheet("@namespace svg 'first';@namespace other 'second';");
    assert!(report.is_clean(), "{report:?}");
    let expected = "@namespace svg url(\"first\");\n@namespace other url(\"second\");";
    assert_eq!(
        report.syntax().to_specified_css().expect("namespace sheet"),
        expected
    );
    let exact = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    let short =
        CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len() - 1);
    let error = report
        .syntax()
        .to_specified_css_with_limits(short)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedRuleSerializationErrorKind::Resource(
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        )
    );
    assert_eq!(error.rule_index(), Some(1));
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
        // Discover a policy that admits each individual rule yet exhausts
        // cumulative work when the same rules compose into a sheet.
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
            .expect("shared node budget exhausts on the second rule");
        let error = report
            .syntax()
            .to_specified_css_with_limits(limits(nodes))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(kind)
        );
        assert_eq!(error.rule_index(), Some(1));
    }
    assert_eq!(report.syntax().rules().len(), 2);
}

#[test]
fn namespace_and_qualified_style_sibling_share_canonical_output() {
    let report = parse_sheet("@namespace svg 'urn:svg';svg|leaf {}");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    assert_eq!(
        report.syntax().to_specified_css().unwrap(),
        "@namespace svg url(\"urn:svg\");\nsvg|leaf { }"
    );
    assert_eq!(report, before);
}
