#![forbid(unsafe_code)]
//! Independent pinned Syntax/Selectors/Nesting rule admissions and owned recovery.
//! Transitions 2 starting-style is outside the selected profile; its parent survives.
//! Admission sources: Syntax 3 (2021-12-24), Selectors 4 (2026-01-22),
//! and Nesting 1 (2026-01-22), as pinned in the standards catalog.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#parse-a-rule
//! https://www.w3.org/TR/2026/WD-selectors-4-20260122/#invalid
//! https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/
use surgeist_css::{
    CssDeclarationList, CssErrorCode, CssKnownPropertyValueRef, CssNamespaceContext,
    CssRecoveryAction, CssRule, CssSelector, CssStyleSelector, parse_rule,
};

type ExpectedDiagnostic = (&'static str, &'static str, usize, u32, u32);
type ExpectedRule = (
    &'static str,
    bool,
    &'static [ExpectedDiagnostic],
    &'static [&'static str],
    &'static [&'static str],
    &'static [[u8; 3]],
);

#[test]
fn rule_corpus_preserves_outer_admission_inner_recovery_and_typed_content() {
    let expectations: &[ExpectedRule] = &[
        (
            "rule/Rule.json#/declaration with ~1~1",
            true,
            &[("invalid_qualified_rule", "drop_qualified_rule", 17, 0, 17)],
            &[".test"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.0",
            true,
            &[("unknown_property", "drop_declaration", 2, 0, 2)],
            &["s"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.1",
            true,
            &[
                ("unknown_property", "drop_declaration", 2, 0, 2),
                ("unknown_property", "drop_declaration", 8, 0, 8),
            ],
            &["s"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.2",
            true,
            &[("unknown_property", "drop_declaration", 6, 0, 6)],
            &["s0", "s1"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.3",
            true,
            &[
                ("unknown_property", "drop_declaration", 6, 0, 6),
                ("unknown_property", "drop_declaration", 12, 0, 12),
            ],
            &["s0", "s1"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.c.0",
            true,
            &[("unknown_property", "drop_declaration", 18, 0, 18)],
            &["s"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.c.1",
            true,
            &[
                ("unknown_property", "drop_declaration", 18, 0, 18),
                ("unknown_property", "drop_declaration", 56, 0, 56),
            ],
            &["s"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.c.2",
            true,
            &[("unknown_property", "drop_declaration", 38, 0, 38)],
            &["s0", "s1"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.c.3",
            true,
            &[
                ("unknown_property", "drop_declaration", 38, 0, 38),
                ("unknown_property", "drop_declaration", 76, 0, 76),
            ],
            &["s0", "s1"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.s.0",
            true,
            &[("unknown_property", "drop_declaration", 6, 0, 6)],
            &["s"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.s.1",
            true,
            &[
                ("unknown_property", "drop_declaration", 6, 0, 6),
                ("unknown_property", "drop_declaration", 20, 0, 20),
            ],
            &["s"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.s.2",
            true,
            &[("unknown_property", "drop_declaration", 14, 0, 14)],
            &["s0", "s1"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.s.3",
            true,
            &[
                ("unknown_property", "drop_declaration", 14, 0, 14),
                ("unknown_property", "drop_declaration", 28, 0, 28),
            ],
            &["s0", "s1"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/rule.s.6",
            true,
            &[],
            &[".test"],
            &[],
            &[[255, 0, 0]],
        ),
        (
            "rule/Rule.json#/shouldn't parse a selector when parseRulePrelude is false",
            false,
            &[("invalid_selector", "reject_input", 14, 0, 14)],
            &[],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/unclosed block",
            true,
            &[("unexpected_end", "retain_with_implicit_closure", 11, 0, 11)],
            &["selector"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/value.color.ident.0",
            true,
            &[("unknown_property", "drop_declaration", 2, 0, 2)],
            &["s"],
            &[],
            &[],
        ),
        (
            "rule/Rule.json#/value.color.ident.1",
            true,
            &[("unknown_property", "drop_declaration", 7, 0, 7)],
            &["yellow"],
            &[],
            &[],
        ),
        (
            "rule/legacy.json#/rule.4",
            true,
            &[("invalid_property_value", "drop_declaration", 13, 0, 13)],
            &[".test"],
            &[],
            &[[255, 0, 0]],
        ),
        (
            "rule/legacy.json#/rule.5",
            true,
            &[("invalid_property_value", "drop_declaration", 23, 0, 23)],
            &[".test"],
            &[],
            &[[255, 0, 0]],
        ),
        (
            "rule/legacy.json#/rule.s.4",
            true,
            &[("invalid_property_value", "drop_declaration", 19, 1, 11)],
            &[".test"],
            &[],
            &[[255, 0, 0]],
        ),
        (
            "rule/legacy.json#/rule.s.5",
            true,
            &[("invalid_property_value", "drop_declaration", 34, 2, 11)],
            &[".test"],
            &[],
            &[[255, 0, 0]],
        ),
        (
            "rule/nested-atrule.json#/basic",
            true,
            &[("unknown_at_rule", "drop_at_rule", 6, 0, 6)],
            &[".test"],
            &[],
            &[],
        ),
        (
            "rule/nested-atrule.json#/recursion",
            true,
            &[("unknown_at_rule", "drop_at_rule", 6, 0, 6)],
            &[".test"],
            &[],
            &[],
        ),
        (
            "rule/nested-atrule.json#/with block",
            true,
            &[("unknown_at_rule", "drop_at_rule", 6, 0, 6)],
            &[".test"],
            &[],
            &[],
        ),
        (
            "rule/nested-atrule.json#/with prelude",
            true,
            &[("unknown_at_rule", "drop_at_rule", 6, 0, 6)],
            &[".test"],
            &[],
            &[],
        ),
        (
            "rule/nesting.json#/basic",
            true,
            &[("unknown_at_rule", "drop_at_rule", 45, 0, 45)],
            &[".foo"],
            &["Style"],
            &[[0, 0, 255], [0, 128, 0]],
        ),
        (
            "rule/nesting.json#/don't parse nested rule when it not started with &",
            true,
            &[],
            &[".foo"],
            &["Style", "Style"],
            &[[0, 128, 0], [255, 0, 0]],
        ),
        (
            "rule/nesting.json#/nested @container",
            true,
            &[],
            &["selector"],
            &["Container", "Container", "NestedDeclarations"],
            &[[0, 128, 0]],
        ),
        (
            "rule/nesting.json#/nested @media",
            true,
            &[],
            &[".foo"],
            &["Media", "NestedDeclarations", "Media", "NestedDeclarations"],
            &[[0, 0, 255], [255, 0, 0], [0, 128, 0]],
        ),
        (
            "rule/nesting.json#/nested @starting-style",
            true,
            &[("unknown_at_rule", "drop_at_rule", 11, 0, 11)],
            &["selector"],
            &[],
            &[],
        ),
        (
            "rule/nesting.json#/nested @supports",
            true,
            &[],
            &[".foo"],
            &[
                "Supports",
                "NestedDeclarations",
                "Supports",
                "NestedDeclarations",
            ],
            &[[0, 0, 255], [255, 0, 0], [0, 128, 0]],
        ),
        (
            "rule/tolerant.json#/bad selector",
            false,
            &[("invalid_selector", "reject_input", 4, 0, 4)],
            &[],
            &[],
            &[],
        ),
    ];
    let fixtures = [
        include_str!("corpus/csstree/expectations/rule/Rule.json"),
        include_str!("corpus/csstree/expectations/rule/legacy.json"),
        include_str!("corpus/csstree/expectations/rule/nested-atrule.json"),
        include_str!("corpus/csstree/expectations/rule/nesting.json"),
        include_str!("corpus/csstree/expectations/rule/tolerant.json"),
    ];
    let cases: Vec<serde_json::Value> = fixtures
        .iter()
        .flat_map(|s| {
            serde_json::from_str::<serde_json::Value>(s).unwrap()["cases"]
                .as_array()
                .unwrap()
                .clone()
        })
        .collect();
    assert_eq!(cases.len(), expectations.len());
    let classes: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    for &(id, retained, diagnostics, selectors, kinds, colors) in expectations {
        let inputs: Vec<_> = cases.iter().filter(|c| c["id"] == id).collect();
        assert_eq!(inputs.len(), 1, "{id}");
        let input = inputs[0]["input"].as_str().unwrap();
        let report = parse_rule(input, &CssNamespaceContext::default());
        assert_eq!(report.syntax().is_some(), retained, "{id}: {report:?}");
        assert_eq!(
            report.is_clean(),
            diagnostics.is_empty(),
            "{id}: {report:?}"
        );
        assert_eq!(
            report.diagnostics().len(),
            diagnostics.len(),
            "{id}: {report:?}"
        );
        for (actual, &(code, action, byte, line, column)) in
            report.diagnostics().iter().zip(diagnostics)
        {
            let expected_code = match code {
                "unknown_property" => CssErrorCode::UnknownProperty,
                "unknown_at_rule" => CssErrorCode::UnknownAtRule,
                "invalid_property_value" => CssErrorCode::InvalidPropertyValue,
                "invalid_selector" => CssErrorCode::InvalidSelector,
                "invalid_qualified_rule" => CssErrorCode::InvalidQualifiedRule,
                "unexpected_end" => CssErrorCode::UnexpectedEnd,
                _ => panic!("code"),
            };
            let expected_action = match action {
                "drop_declaration" => CssRecoveryAction::DropDeclaration,
                "drop_at_rule" => CssRecoveryAction::DropAtRule,
                "reject_input" => CssRecoveryAction::RejectInput,
                "drop_qualified_rule" => CssRecoveryAction::DropQualifiedRule,
                "retain_with_implicit_closure" => CssRecoveryAction::RetainWithImplicitClosure,
                _ => panic!("action"),
            };
            assert_eq!(actual.error().code(), expected_code, "{id}");
            assert_eq!(actual.action(), expected_action, "{id}");
            let position = actual.error().position();
            assert_eq!(
                (
                    position.byte_offset().value(),
                    position.line().value(),
                    position.column().value()
                ),
                (byte, line, column),
                "{id}"
            );
            assert!(
                actual.span().end().byte_offset().value() <= input.len(),
                "{id}"
            );
            if !retained {
                assert_eq!(actual.span().start().byte_offset().value(), 0);
                assert_eq!(actual.span().end().byte_offset().value(), input.len());
            }
        }
        let mut expected = serde_json::json!({"kind":if diagnostics.is_empty(){"clean"}else if retained{"recovered"}else{"strict_rejected"},"retained_syntax":{"extractor":{"kind":"sheet_rules"},"predicate":{"relation":if retained{"nonempty"}else{"empty"}}}});
        if !diagnostics.is_empty() {
            expected["diagnostics"]=serde_json::Value::Array(diagnostics.iter().map(|(code,action,..)|serde_json::json!({"code":code,"action":action,"payload_relation":if *action == "retain_with_implicit_closure" { "ends_at" } else { "intersects" }})).collect());
        }
        let matched: Vec<_> = classes["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] == id)
            .collect();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0]["class"], expected, "{id}");
        if retained {
            let CssRule::Style(rule) = report.syntax().as_ref().unwrap() else {
                panic!("outer style: {id}")
            };
            assert_eq!(rule.selectors().selectors().len(), selectors.len(), "{id}");
            for (actual, expected) in rule.selectors().selectors().iter().zip(selectors) {
                match (actual, expected.strip_prefix('.')) {
                    (CssStyleSelector::Selector(CssSelector::Class(actual)), Some(expected)) => {
                        assert_eq!(actual, expected, "{id}")
                    }
                    (CssStyleSelector::Selector(CssSelector::Tag(actual)), None) => {
                        assert_eq!(actual, expected, "{id}")
                    }
                    _ => panic!("outer selector shape: {id}"),
                }
            }
            let mut actual_colors = Vec::new();
            collect_colors(rule.declarations(), &mut actual_colors);
            let mut actual_kinds = Vec::new();
            for child in rule.rules() {
                collect_rules(child, &mut actual_kinds, &mut actual_colors);
            }
            assert_eq!(actual_kinds, kinds, "{id}");
            assert_eq!(actual_colors, colors, "{id}");
        }
    }
}
fn collect_colors(declarations: &CssDeclarationList, colors: &mut Vec<[u8; 3]>) {
    for declaration in declarations.iter() {
        let CssKnownPropertyValueRef::Color(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("color")
        };
        let value = value.i01_subset().unwrap().as_rgba().unwrap();
        assert_eq!(value.alpha(), 1.0);
        colors.push([value.red(), value.green(), value.blue()]);
    }
}
fn collect_rules(rule: &CssRule, kinds: &mut Vec<&'static str>, colors: &mut Vec<[u8; 3]>) {
    let children = match rule {
        CssRule::Style(rule) => {
            kinds.push("Style");
            collect_colors(rule.declarations(), colors);
            rule.rules()
        }
        CssRule::Media(rule) => {
            kinds.push("Media");
            rule.rules()
        }
        CssRule::Supports(rule) => {
            kinds.push("Supports");
            rule.rules()
        }
        CssRule::Container(rule) => {
            kinds.push("Container");
            rule.rules()
        }
        CssRule::NestedDeclarations(rule) => {
            kinds.push("NestedDeclarations");
            collect_colors(rule.declarations(), colors);
            return;
        }
        _ => panic!("unexpected child rule"),
    };
    for child in children {
        collect_rules(child, kinds, colors);
    }
}
