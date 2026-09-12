#![forbid(unsafe_code)]

//! Declaration candidates must satisfy declaration grammar before claiming that
//! supports branch. Semicolons and nonterminal/invalid importance delimiters are
//! still legal in general-enclosed <any-value>, so this is interpretation rather
//! than support evaluation or blanket invalidation.
//! https://www.w3.org/TR/2024/CRD-css-conditional-3-20240815/#at-supports
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#any-value
//! Import clauses are optional, and the complete absent-supports derivation may
//! interpret the whole supports(...) function as an opaque media operand.
//! https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#at-import
//! https://www.w3.org/TR/2026/WD-mediaqueries-5-20260219/#mq-syntax

use surgeist_css::{
    CssImportLayer, CssImportRule, CssImportance, CssMediaConditionKind, CssMediaQuery,
    CssRecoveryAction, CssRule, CssSupportsCondition, CssSupportsConditionKind, parse_sheet,
    validate_sheet,
};

const INVALID_DECLARATIONS: &[&str] = &[
    "color:red;",
    "color:red !bogus",
    "color:red !important !important",
    "color:red !important trailing",
    "color:red !",
];

fn condition(contents: &str) -> CssSupportsCondition {
    let source = format!("@supports ({contents}) {{}} .after{{color:red}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {report:?}");
    assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
    let [CssRule::Supports(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("retained condition and following style: {source}: {report:?}");
    };
    assert_eq!(after.declarations().len(), 1);
    rule.condition().clone()
}

fn import(tail: &str) -> CssImportRule {
    let source = format!("@import 'x.css' {tail}; .after{{color:red}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {report:?}");
    assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
    let [CssRule::Import(rule), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("retained import and following style: {source}: {report:?}");
    };
    assert_eq!(after.declarations().len(), 1);
    rule.clone()
}

fn opaque_supports(condition: &CssSupportsCondition, expected: &str) {
    let CssSupportsConditionKind::GeneralEnclosed(value) = condition.kind() else {
        panic!("expected opaque supports operand {expected}: {condition:?}");
    };
    assert_eq!(value.authored(), Some(expected));
    assert_eq!(value.serialize().unwrap().as_css(), expected);
}

#[test]
fn supports_invalid_declaration_candidates_remain_opaque_conditions() {
    for contents in INVALID_DECLARATIONS {
        opaque_supports(&condition(contents), &format!("({contents})"));
    }
}

#[test]
fn supports_valid_declaration_tests_keep_empty_unknown_custom_and_terminal_importance() {
    for (contents, property, important, known) in [
        ("color:red", "color", false, true),
        ("color:red !important", "color", true, true),
        ("color:red !/**/IMPORTANT", "color", true, true),
        ("color:", "color", false, false),
        ("mystery:anything", "mystery", false, false),
        ("color:not-a-color", "color", false, false),
        ("--custom:", "--custom", false, false),
        ("--custom:future(a; !bogus)", "--custom", false, false),
        ("mystery:[a; !bogus]", "mystery", false, false),
        (
            "mystery:future(a; !bogus) !important",
            "mystery",
            true,
            false,
        ),
    ] {
        let condition = condition(contents);
        let CssSupportsConditionKind::Declaration(declaration) = condition.kind() else {
            panic!("valid authored declaration: {contents}: {condition:?}");
        };
        assert_eq!(declaration.authored(), contents);
        assert_eq!(declaration.property(), property);
        assert_eq!(declaration.known().is_some(), known, "{contents}");
        assert_eq!(
            declaration.importance(),
            if important {
                CssImportance::Important
            } else {
                CssImportance::Normal
            },
            "{contents}"
        );

        let import = import(&format!("supports({contents})"));
        assert!(import.media().is_none(), "{contents}: {import:?}");
        let CssSupportsConditionKind::Declaration(import_declaration) =
            import.supports().unwrap().condition().kind()
        else {
            panic!("valid bare import declaration: {contents}");
        };
        assert_eq!(import_declaration.authored(), contents);
        assert_eq!(import_declaration.importance(), declaration.importance());
    }
}

#[test]
fn supports_invalid_bare_import_candidates_select_absent_clause_opaque_media() {
    for contents in INVALID_DECLARATIONS {
        let function = format!("supports({contents})");
        for layer in [false, true] {
            let tail = if layer {
                format!("layer(theme) {function}")
            } else {
                function.clone()
            };
            let import = import(&tail);
            assert!(import.supports().is_none(), "{tail}: {import:?}");
            if layer {
                assert!(
                    matches!(import.layer(), Some(CssImportLayer::Named(name)) if name.components() == ["theme"])
                );
            } else {
                assert!(import.layer().is_none());
            }
            let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
                panic!("one opaque media query: {tail}: {import:?}");
            };
            let CssMediaConditionKind::GeneralEnclosed(value) = condition.kind() else {
                panic!("whole supports function is media: {tail}: {condition:?}");
            };
            assert_eq!(value.authored(), Some(function.as_str()));
        }
    }
}

#[test]
fn supports_parenthesized_import_candidates_select_present_opaque_condition_clause() {
    for contents in INVALID_DECLARATIONS {
        let import = import(&format!("supports(({contents}))"));
        assert!(import.media().is_none(), "{import:?}");
        opaque_supports(
            import.supports().unwrap().condition(),
            &format!("({contents})"),
        );
    }
}

#[test]
fn supports_invalid_bare_import_candidates_compose_as_complete_media_conjunctions() {
    for contents in INVALID_DECLARATIONS {
        let function = format!("supports({contents})");
        let import = import(&format!("{function} and (color)"));
        assert!(import.supports().is_none(), "{import:?}");
        let [CssMediaQuery::Condition(condition)] = import.media().unwrap().queries() else {
            panic!("one media conjunction: {import:?}");
        };
        let CssMediaConditionKind::And(operands) = condition.kind() else {
            panic!("media conjunction: {condition:?}");
        };
        let [opaque, feature] = operands.conditions() else {
            panic!("two media operands");
        };
        assert!(
            matches!(opaque.kind(), CssMediaConditionKind::GeneralEnclosed(value) if value.authored() == Some(function.as_str()))
        );
        assert!(matches!(feature.kind(), CssMediaConditionKind::Feature(_)));
    }
}

#[test]
fn supports_bad_url_and_bad_string_tokens_cannot_be_declarations_or_opaque_fallbacks() {
    // Both branches exclude bad tokens. Unlike ';' or '!', these cannot be
    // admitted through <any-value>. The actual closing parenthesis is present.
    for contents in ["color:url(a b)", "color:\"broken\n"] {
        let source = format!("@supports ({contents}) {{}} .after{{color:red}}");
        let report = parse_sheet(&source);
        assert!(
            matches!(report.syntax().rules(), [CssRule::Style(_)]),
            "{source}: {report:?}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one failed supports rule: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}
