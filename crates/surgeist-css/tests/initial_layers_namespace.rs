//! Cascade 5 extends Namespaces 3 placement: initial layer statements may
//! precede both imports and namespace declarations. A layer after either
//! import or namespace declarations closes that prelude sequence.
//! https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#layer-empty

use surgeist_css::{CssErrorCode, CssRecoveryAction, CssRule, parse_sheet, validate_sheet};

fn rule_names(rules: &[CssRule]) -> Vec<&'static str> {
    rules
        .iter()
        .map(|rule| match rule {
            CssRule::LayerStatement(_) => "layer",
            CssRule::LayerBlock(_) => "layer-block",
            CssRule::Import(_) => "import",
            CssRule::Namespace(_) => "namespace",
            CssRule::Style(_) => "style",
            other => panic!("unexpected rule: {other:?}"),
        })
        .collect()
}

#[test]
fn initial_layer_statements_allow_namespaces_with_or_without_imports() {
    for (prefix, expected) in [
        ("@layer reset; ", vec!["layer", "namespace", "style"]),
        (
            "@layer reset; @layer theme; @import 'a.css'; @import 'b.css'; ",
            vec!["layer", "layer", "import", "import", "namespace", "style"],
        ),
        (
            "@charset \"UTF-8\"; @LaYeR reset; @\\69mport 'a.css'; ",
            vec!["layer", "import", "namespace", "style"],
        ),
    ] {
        let source = format!("{prefix}@namespace svg 'urn:svg'; svg|rect {{margin:0}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(validate_sheet(&source).is_ok(), "{source}");
        assert_eq!(rule_names(report.syntax().rules()), expected);
        let namespace = report
            .syntax()
            .rules()
            .iter()
            .find_map(|rule| match rule {
                CssRule::Namespace(namespace) => Some(namespace),
                _ => None,
            })
            .unwrap();
        assert_eq!(namespace.prefix().unwrap().as_str(), "svg");
        assert_eq!(namespace.name().as_str(), "urn:svg");
        assert_eq!(namespace.position().byte_offset().value(), prefix.len());
    }
}

#[test]
fn accepted_namespace_after_initial_layers_closes_imports_and_activates_bindings() {
    let source = concat!(
        "@layer reset; @namespace svg 'urn:svg'; ",
        "@import 'late.css'; svg|rect {margin:0}",
    );
    let report = parse_sheet(source);
    assert_eq!(
        rule_names(report.syntax().rules()),
        ["layer", "namespace", "style"]
    );
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "only the late import should be dropped: {:?}",
            report.diagnostics()
        );
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidAtRulePlacement
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("@import").unwrap()
    );
    assert!(validate_sheet(source).is_err());
}

#[test]
fn layers_after_imports_or_namespaces_still_close_both_preludes() {
    for prefix in [
        "@import 'first.css'; @layer later; ",
        "@namespace svg 'urn:svg'; @layer later; ",
        "@layer first; @import 'first.css'; @layer later; ",
        "@layer first {} ",
        ".body {} ",
    ] {
        let source = format!("{prefix}@namespace late 'urn:late'; @import 'late.css'; .kept {{}}");
        let report = parse_sheet(&source);
        assert_eq!(
            report.diagnostics().len(),
            2,
            "{source}: {:?}",
            report.diagnostics()
        );
        for diagnostic in report.diagnostics() {
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidAtRulePlacement,
                "{source}"
            );
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        }
        assert!(matches!(
            report.syntax().rules().last(),
            Some(CssRule::Style(_))
        ));
    }
}

#[test]
fn invalid_prelude_rules_do_not_cancel_later_valid_namespace_bindings() {
    let report = parse_sheet(concat!(
        "@layer; @unknown ignored; @layer reset; ",
        "@namespace malformed; @namespace svg 'urn:svg'; svg|rect {margin:0}",
    ));
    assert_eq!(
        rule_names(report.syntax().rules()),
        ["layer", "namespace", "style"]
    );
    assert_eq!(report.diagnostics().len(), 3);
    assert!(report.diagnostics().iter().all(|diagnostic| {
        diagnostic.action() == CssRecoveryAction::DropAtRule
            && diagnostic.error().code() != CssErrorCode::InvalidAtRulePlacement
    }));
}
