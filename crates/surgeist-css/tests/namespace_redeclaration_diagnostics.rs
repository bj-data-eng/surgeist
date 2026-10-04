#![forbid(unsafe_code)]
//! Namespaces 3 §§3.2–3.3: declarations are stylesheet-local; repeated decoded
//! prefixes/defaults are nonconforming, but the retained last declaration wins.
//! https://www.w3.org/TR/2014/REC-css-namespaces-3-20140320/#prefixes

use surgeist_css::{
    CssNamespaceConstraint, CssNamespaceContext, CssNamespacePrefix, CssParseReport,
    CssRecoveryAction, CssRule, CssSelector, CssSheet, parse_selector, parse_sheet, validate_sheet,
};

fn prefix(value: &str) -> CssNamespacePrefix {
    CssNamespacePrefix::try_new(value).expect("decoded namespace prefix")
}

fn declarations(sheet: &CssSheet) -> Vec<(Option<&str>, &str)> {
    sheet
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Namespace(rule) => Some((
                rule.prefix().map(|prefix| prefix.as_str()),
                rule.name().as_str(),
            )),
            _ => None,
        })
        .collect()
}

fn assert_selector_namespace(
    source: &str,
    context: &CssNamespaceContext,
    expected: CssNamespaceConstraint,
) {
    let report = parse_selector(source, context);
    assert!(report.is_clean(), "{source}: {report:?}");
    let Some(CssSelector::Compound(compound)) = report.syntax() else {
        panic!("qualified compound retained: {report:?}");
    };
    assert_eq!(compound.type_selector().unwrap().namespace(), &expected);
    assert_eq!(
        report.clone().into_validation_result().unwrap(),
        *report.syntax()
    );
}

fn assert_duplicate_diagnostics(
    source: &str,
    report: &CssParseReport<CssSheet>,
    duplicate_ranges: &[(usize, usize)],
) {
    assert!(
        !report.is_clean(),
        "redeclared bindings are nonconforming: {source}"
    );
    assert_eq!(report.diagnostics().len(), duplicate_ranges.len());
    for (diagnostic, &(start, end)) in report.diagnostics().iter().zip(duplicate_ranges) {
        let position = diagnostic.error().position().byte_offset().value();
        let span = diagnostic.span();
        assert!(
            (start..end).contains(&position),
            "duplicate declaration origin: {diagnostic:?}"
        );
        assert!(span.start().byte_offset().value() >= start);
        assert!(span.end().byte_offset().value() <= end);
        assert_ne!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_ne!(diagnostic.action(), CssRecoveryAction::RejectInput);
    }
    let failure = validate_sheet(source).expect_err("nonconforming sheet fails validation");
    assert_eq!(failure.diagnostics(), report.diagnostics());
    assert_eq!(
        report
            .clone()
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn decoded_named_redeclarations_retain_order_and_last_binding_with_local_diagnostics() {
    let first = "@namespace svg 'urn:first';\n";
    let different_case = "@namespace SVG 'urn:upper';\n";
    let second = "@namespace s\\76 g 'urn:second';\n";
    let third = "@namespace svg '';\n";
    let source = format!("{first}{different_case}{second}{third}svg|leaf{{}}.kept{{color:red}}");
    let report = parse_sheet(&source);
    assert_eq!(
        declarations(report.syntax()),
        [
            (Some("svg"), "urn:first"),
            (Some("SVG"), "urn:upper"),
            (Some("svg"), "urn:second"),
            (Some("svg"), "")
        ]
    );
    assert!(matches!(
        report.syntax().rules(),
        [
            CssRule::Namespace(_),
            CssRule::Namespace(_),
            CssRule::Namespace(_),
            CssRule::Namespace(_),
            CssRule::Style(_),
            CssRule::Style(_)
        ]
    ));
    let positions = report
        .syntax()
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Namespace(rule) => Some(rule.position().byte_offset().value()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let second_start = first.len() + different_case.len();
    let third_start = second_start + second.len();
    assert_eq!(positions, [0, first.len(), second_start, third_start]);
    let context = CssNamespaceContext::from_sheet(report.syntax());
    assert_eq!(
        context.named_namespace(&prefix("svg")).unwrap().as_str(),
        ""
    );
    assert_eq!(
        context.named_namespace(&prefix("SVG")).unwrap().as_str(),
        "urn:upper"
    );
    assert!(context.named_namespace(&prefix("Svg")).is_none());
    assert!(context.default_namespace().is_none());
    assert_selector_namespace(
        "svg|leaf.mark",
        &context,
        CssNamespaceConstraint::Named(prefix("svg")),
    );
    assert_duplicate_diagnostics(
        &source,
        &report,
        &[
            (second_start, third_start),
            (third_start, third_start + third.len()),
        ],
    );
}

#[test]
fn default_redeclarations_keep_null_and_literal_last_names_while_rejecting_validation() {
    for (first_name, last_name) in [
        ("urn:first", ""),
        ("", "not a URI"),
        ("", ""),
        ("urn:same", "urn:same"),
    ] {
        let first = format!("@namespace '{first_name}';");
        let second = format!("@namespace '{last_name}';");
        let source = format!("{first}{second}leaf.mark{{}}.kept{{color:red}}");
        let report = parse_sheet(&source);
        assert_eq!(
            declarations(report.syntax()),
            [(None, first_name), (None, last_name)]
        );
        assert!(matches!(
            report.syntax().rules(),
            [
                CssRule::Namespace(_),
                CssRule::Namespace(_),
                CssRule::Style(_),
                CssRule::Style(_)
            ]
        ));
        let context = CssNamespaceContext::from_sheet(report.syntax());
        assert_eq!(context.default_namespace().unwrap().as_str(), last_name);
        assert_selector_namespace("leaf.mark", &context, CssNamespaceConstraint::Default);
        assert_duplicate_diagnostics(
            &source,
            &report,
            &[(first.len(), first.len() + second.len())],
        );
    }
}

#[test]
fn identical_named_binding_is_still_a_nonconforming_redeclaration() {
    for name in ["urn:same", "", "not a URI"] {
        let declaration = format!("@namespace svg '{name}';");
        let source = format!("{declaration}{declaration}svg|leaf{{}}");
        let report = parse_sheet(&source);
        assert_eq!(
            declarations(report.syntax()),
            [(Some("svg"), name), (Some("svg"), name)]
        );
        let context = CssNamespaceContext::from_sheet(report.syntax());
        assert_eq!(
            context.named_namespace(&prefix("svg")).unwrap().as_str(),
            name
        );
        assert_duplicate_diagnostics(
            &source,
            &report,
            &[(declaration.len(), 2 * declaration.len())],
        );
    }
}

#[test]
fn different_case_named_prefixes_are_distinct_clean_bindings() {
    let source = "@namespace svg 'urn:lower';@namespace SVG '';svg|leaf{}SVG|leaf{}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let context = CssNamespaceContext::from_sheet(report.syntax());
    assert_eq!(
        context.named_namespace(&prefix("svg")).unwrap().as_str(),
        "urn:lower"
    );
    assert_eq!(
        context.named_namespace(&prefix("SVG")).unwrap().as_str(),
        ""
    );
    assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
}

#[test]
fn copied_redeclared_context_is_immutable_and_never_inherited_by_other_sheets() {
    let first = "@namespace svg 'urn:old';";
    let second = "@namespace svg '';";
    let source = format!("@import 'child.css';{first}{second}svg|leaf{{}}");
    let report = parse_sheet(&source);
    let context = CssNamespaceContext::from_sheet(report.syntax());
    let original = context.clone();
    assert_selector_namespace(
        "svg|leaf.mark",
        &context,
        CssNamespaceConstraint::Named(prefix("svg")),
    );
    // Parsing another stylesheet cannot inherit bindings in either import direction
    // or from a sibling. The parser owns syntax only and never loads child.css.
    for other in [
        "svg|leaf{}.kept{}",
        "@import 'parent.css';svg|leaf{}.kept{}",
        "@import 'sibling.css';svg|leaf{}.kept{}",
    ] {
        let isolated = parse_sheet(other);
        assert!(
            matches!(isolated.syntax().rules().last(), Some(CssRule::Style(style)) if style.selectors().selectors()[0].selector() == &CssSelector::Class("kept".into()))
        );
        assert_eq!(
            isolated.diagnostics().len(),
            1,
            "undeclared prefix in {other}"
        );
        let isolated_context = CssNamespaceContext::from_sheet(isolated.syntax());
        assert!(isolated_context.named_namespace(&prefix("svg")).is_none());
        assert!(isolated_context.default_namespace().is_none());
        assert!(
            parse_selector("svg|leaf", &isolated_context)
                .syntax()
                .is_none()
        );
    }
    assert_eq!(context, original);
    assert_eq!(
        context.named_namespace(&prefix("svg")).unwrap().as_str(),
        ""
    );
    let start = "@import 'child.css';".len() + first.len();
    assert_duplicate_diagnostics(&source, &report, &[(start, start + second.len())]);
}

#[test]
fn ignored_late_and_nested_redeclarations_never_replace_the_retained_binding() {
    for suffix in [
        "@namespace svg 'urn:ignored';.kept{}",
        "@media screen{@namespace svg 'urn:ignored';.nested{}}.kept{}",
    ] {
        let source = format!("@namespace svg 'urn:active';svg|leaf{{}}{suffix}");
        let report = parse_sheet(&source);
        assert_eq!(declarations(report.syntax()), [(Some("svg"), "urn:active")]);
        let context = CssNamespaceContext::from_sheet(report.syntax());
        assert_eq!(
            context.named_namespace(&prefix("svg")).unwrap().as_str(),
            "urn:active"
        );
        assert_eq!(
            report.diagnostics().len(),
            1,
            "ignored rule is not an active redeclaration"
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn malformed_and_block_declarations_are_ignored_without_redeclaring_the_active_prefix() {
    for ignored in ["@namespace svg ident;", "@namespace svg 'urn:ignored'{}"] {
        let source = format!("@namespace svg 'urn:active';{ignored}svg|leaf{{}}.kept{{}}");
        let report = parse_sheet(&source);
        assert_eq!(declarations(report.syntax()), [(Some("svg"), "urn:active")]);
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::Namespace(_), CssRule::Style(_), CssRule::Style(_)]
        ));
        let context = CssNamespaceContext::from_sheet(report.syntax());
        assert_eq!(
            context.named_namespace(&prefix("svg")).unwrap().as_str(),
            "urn:active"
        );
        assert_selector_namespace(
            "svg|leaf.mark",
            &context,
            CssNamespaceConstraint::Named(prefix("svg")),
        );
        assert_eq!(
            report.diagnostics().len(),
            1,
            "only the invalid declaration is diagnosed"
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropAtRule
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn decoded_namespace_prefix_construction_accepts_values_requiring_identifier_escapes() {
    // A decoded IDENT value is distinct from its CSS spelling. Both values can
    // be expressed as one IDENT by escaping the leading digit or embedded space.
    assert!(CssNamespacePrefix::try_new("").is_none());
    assert!(CssNamespacePrefix::try_new("a\0b").is_none());
    for decoded in ["1", "a b"] {
        let prefix = CssNamespacePrefix::try_new(decoded)
            .expect("a decoded identifier may require escapes in its CSS spelling");
        assert_eq!(prefix.as_str(), decoded);
    }
}

#[test]
fn escaped_namespace_prefixes_parse_as_clean_decoded_identifiers_without_unwinding() {
    for (source, decoded, fragment) in [
        (
            r"@namespace \31 'urn:digit';\31 |leaf{}",
            "1",
            r"\31 |leaf.mark",
        ),
        (
            r"@namespace a\ b 'urn:space';a\ b|leaf{}",
            "a b",
            r"a\ b|leaf.mark",
        ),
    ] {
        let report = std::panic::catch_unwind(|| parse_sheet(source))
            .expect("valid escaped namespace identifiers must not unwind");
        assert!(report.is_clean(), "{source}: {report:?}");
        let [CssRule::Namespace(namespace), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("valid namespace declaration and qualified rule retained: {report:?}");
        };
        let parsed_prefix = namespace.prefix().expect("named declaration");
        assert_eq!(parsed_prefix.as_str(), decoded);
        let context = CssNamespaceContext::from_sheet(report.syntax());
        assert_eq!(
            context.named_namespace(parsed_prefix),
            Some(namespace.name())
        );
        assert_selector_namespace(
            fragment,
            &context,
            CssNamespaceConstraint::Named(parsed_prefix.clone()),
        );
        assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
    }
}
