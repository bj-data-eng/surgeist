#![cfg(feature = "app-strict")]

use surgeist_css::{
    CssDeclarationList, CssMediaConditionKind, CssMediaQuery, CssParseReport, CssRecoveryAction,
    CssRule, CssSheet, parse_sheet, parse_style_attribute, validate_sheet,
    validate_style_attribute,
};

fn assert_sheet_parity(source: &str) -> CssParseReport<CssSheet> {
    let ordinary = parse_sheet(source);
    let strict = validate_sheet(source);

    if ordinary.is_clean() {
        assert_eq!(strict, Ok(ordinary.syntax().clone()), "{source}");
    } else {
        let failure = strict.expect_err("recovered sheet must fail strict validation");
        assert_eq!(failure.first(), &ordinary.diagnostics()[0], "{source}");
        assert_eq!(failure.diagnostics(), ordinary.diagnostics(), "{source}");
        assert_eq!(
            failure.into_diagnostics(),
            ordinary.diagnostics(),
            "{source}"
        );
    }

    ordinary
}

fn assert_style_parity(source: &str) -> CssParseReport<CssDeclarationList> {
    let ordinary = parse_style_attribute(source);
    let strict = validate_style_attribute(source);

    if ordinary.is_clean() {
        assert_eq!(strict, Ok(ordinary.syntax().clone()), "{source}");
    } else {
        let failure = strict.expect_err("recovered style attribute must fail strict validation");
        assert_eq!(failure.first(), &ordinary.diagnostics()[0], "{source}");
        assert_eq!(failure.diagnostics(), ordinary.diagnostics(), "{source}");
        assert_eq!(
            failure.into_diagnostics(),
            ordinary.diagnostics(),
            "{source}"
        );
    }

    ordinary
}

#[test]
fn app_strict_accepts_defined_false_media_syntax_and_rejects_only_malformed_recovery() {
    let defined_false = assert_sheet_parity(concat!(
        "@media only future-screen and (unknown: calc(1foo + 2px)), ",
        "(width: -1px) {}",
    ));
    assert!(defined_false.is_clean());
    let [CssRule::Media(rule)] = defined_false.syntax().rules() else {
        panic!("expected retained media rule")
    };
    assert!(matches!(rule.query().queries()[0], CssMediaQuery::Typed(_)));
    assert!(matches!(
        &rule.query().queries()[1],
        CssMediaQuery::Condition(condition)
            if matches!(condition.kind(), CssMediaConditionKind::DefinedFalse(_))
    ));

    for source in [
        "@media screen,layer,print {}",
        "@media screen,(scripting: enabled),print {}",
        "@media screen,,print {}",
    ] {
        let malformed = assert_sheet_parity(source);
        assert!(malformed.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::ReplaceMediaQueryWithNever
        }));
    }
}

#[test]
fn app_strict_supports_conditions_match_ordinary_retention_and_recovery() {
    let clean = assert_sheet_parity(concat!(
        "@supports (display: grid) and (mystery: value) {}",
        "@supports selector(.x > .y) {}",
        "@supports future(any([tokens])) {}",
    ));
    assert!(clean.is_clean());
    assert!(
        clean
            .syntax()
            .rules()
            .iter()
            .all(|rule| matches!(rule, CssRule::Supports(_)))
    );

    let recovered = assert_sheet_parity(concat!(
        "@supports (a:b) and (c:d) or (e:f) {}",
        "@supports (color: red) {}",
    ));
    assert!(matches!(recovered.syntax().rules(), [CssRule::Supports(_)]));
    assert_eq!(recovered.diagnostics().len(), 1);
    assert_eq!(
        recovered.diagnostics()[0].action(),
        CssRecoveryAction::DropAtRule
    );
}

#[test]
fn app_strict_conditional_imports_and_prelude_phases_match_ordinary_reports() {
    let clean = assert_sheet_parity(concat!(
        "@layer reset; ",
        "@import 'theme.css' layer(theme) supports(display: grid) screen;",
    ));
    assert!(clean.is_clean());
    assert!(matches!(
        clean.syntax().rules(),
        [CssRule::LayerStatement(_), CssRule::Import(_)]
    ));

    for source in [
        "@import 'x.css' supports(display: grid) layer(theme);",
        "@import 'x.css'; @layer theme; @import 'late.css';",
        "@media screen { @import 'nested.css'; }",
    ] {
        let recovered = assert_sheet_parity(source);
        assert!(!recovered.is_clean(), "{source}");
        assert!(
            recovered
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
        );
    }
}

#[test]
fn app_strict_namespace_rules_match_ordinary_retention_ordering_and_recovery() {
    let clean = assert_sheet_parity(concat!(
        "@import 'theme.css'; ",
        "@namespace \"urn:default\"; ",
        "@namespace svg url(urn:svg);",
    ));
    assert!(clean.is_clean());
    assert!(matches!(
        clean.syntax().rules(),
        [
            CssRule::Import(_),
            CssRule::Namespace(_),
            CssRule::Namespace(_)
        ]
    ));

    for source in [
        "@namespace svg ident; .kept {}",
        "@layer reset; @namespace svg 'urn:late'; @import 'kept.css';",
        "@media screen { @namespace svg 'urn:nested'; .kept {} }",
        "@namespace svg 'urn:missing-semicolon'",
    ] {
        let recovered = assert_sheet_parity(source);
        assert!(!recovered.is_clean(), "{source}");
        assert!(
            recovered
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action() == CssRecoveryAction::DropAtRule)
        );
    }
}

#[test]
fn app_strict_namespace_qualified_selectors_match_ordinary_results() {
    let clean = assert_sheet_parity(concat!(
        "@namespace \"urn:default\";",
        "@namespace svg \"urn:svg\";",
        "svg|a,svg|*,*|a,|a,a[svg|href][*|title][|lang][plain] { color: red; }",
        "@media screen { svg|media { color: red; } }",
        "@supports selector(svg|supported) { svg|supports { color: red; } }",
        "@scope (svg|root) to (svg|limit) { svg|scoped { color: red; } }",
    ));
    assert!(clean.is_clean(), "{:?}", clean.diagnostics());
    assert!(matches!(
        clean.syntax().rules(),
        [
            CssRule::Namespace(_),
            CssRule::Namespace(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Style(_),
            CssRule::Media(_),
            CssRule::Supports(_),
            CssRule::Scope(_),
        ]
    ));

    let recovered = assert_sheet_parity(concat!(
        "@namespace svg \"urn:svg\";",
        ".before {} missing|a { color: red; } .after {}",
    ));
    assert!(matches!(
        recovered.syntax().rules(),
        [CssRule::Namespace(_), CssRule::Style(_), CssRule::Style(_)]
    ));
    let [diagnostic] = recovered.diagnostics() else {
        panic!("expected one undeclared-prefix diagnostic")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
}

#[test]
fn app_strict_selectors3_pseudos_repeated_ids_and_recovery_match_ordinary_results() {
    let clean = assert_sheet_parity(concat!(
        "a#first#second:link[data-ready] > .target:target { color: red; }",
        ".language:lang(e\\6e) { color: red; }",
        ".line:first-line { color: red; }",
        ".letter::first-letter { color: red; }",
    ));
    assert!(clean.is_clean(), "{:?}", clean.diagnostics());
    assert_eq!(clean.syntax().rules().len(), 4);

    let recovered = assert_sheet_parity(concat!(
        ".before:visited { color: red; }",
        ".bad:lang() { color: black; }",
        ".bad:marker { color: black; }",
        ".after:target { color: blue; }",
    ));
    assert!(matches!(
        recovered.syntax().rules(),
        [CssRule::Style(_), CssRule::Style(_)]
    ));
    assert_eq!(recovered.diagnostics().len(), 2);
    assert!(
        recovered
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::DropQualifiedRule)
    );
}

fn nested_selector(depth: usize) -> String {
    format!("{}{}{}", ":is(".repeat(depth), ".leaf", ")".repeat(depth)) + "{color:red}"
}

#[test]
fn app_strict_parity_clean_and_recovered_sheet_and_style_results_are_exact() {
    assert!(assert_sheet_parity(".x { color: red; }").is_clean());
    assert!(assert_style_parity("color: red").is_clean());

    let sheet = assert_sheet_parity(".x { mystery: 1; }");
    assert_eq!(sheet.diagnostics().len(), 1);
    let style = assert_style_parity("mystery: 1");
    assert_eq!(style.diagnostics().len(), 1);

    assert!(
        assert_sheet_parity(concat!(
            "@font-face{font-family:One;font-family:Two;src:url(one);src:url(two);",
            "font-feature-settings:\"kern\" on}"
        ))
        .is_clean()
    );
}

#[test]
fn app_strict_counter_style_descriptors_match_ordinary_retention_and_recovery() {
    let clean = assert_sheet_parity(concat!(
        "@counter-style roman { system: additive; range: 1 infinite; ",
        "negative: \"-\"; pad: 2 \"0\"; fallback: decimal; ",
        "additive-symbols: 10 X, 1 I; speak-as: words; }",
    ));
    assert!(clean.is_clean(), "{:?}", clean.diagnostics());
    assert!(matches!(clean.syntax().rules(), [CssRule::CounterStyle(_)]));

    let recovered = assert_sheet_parity(concat!(
        "@counter-style kept { system: additive; additive-symbols: 10 X, 10 I; ",
        "additive-symbols: 10 X, 1 I; pad: -1 \"0\"; } .after {}",
    ));
    assert!(matches!(
        recovered.syntax().rules(),
        [CssRule::CounterStyle(_), CssRule::Style(_)]
    ));
    assert!(
        recovered
            .diagnostics()
            .iter()
            .all(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropDescriptor })
    );
}

#[test]
fn app_strict_page_rules_match_ordinary_retention_and_recovery() {
    let clean = assert_sheet_parity(concat!(
        "@import \"print.css\"; ",
        "@page :left { margin: -1cm 2% auto 3pt !important; }",
    ));
    assert!(clean.is_clean(), "{:?}", clean.diagnostics());
    assert!(matches!(
        clean.syntax().rules(),
        [CssRule::Import(_), CssRule::Page(_)]
    ));

    let recovered = assert_sheet_parity(concat!(
        "@page :first { color: red; mystery: 1; margin-top: 1em; ",
        "margin-bottom: -2cm; } .after {}",
    ));
    assert!(matches!(
        recovered.syntax().rules(),
        [CssRule::Page(_), CssRule::Style(_)]
    ));
    assert_eq!(recovered.diagnostics().len(), 3);
    assert!(
        recovered
            .diagnostics()
            .iter()
            .all(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropDeclaration })
    );
}

#[test]
fn app_strict_parity_preserves_multiple_diagnostics_and_every_special_action() {
    let sheet = assert_sheet_parity("<!-- .x { mystery: 1; width: nope; } -->");
    assert!(sheet.diagnostics().len() >= 4);
    assert!(
        sheet
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.action() == CssRecoveryAction::IgnoreLegacyToken)
    );

    let style = assert_style_parity("mystery: 1; width: nope; color: red");
    assert_eq!(style.diagnostics().len(), 2);

    let never = assert_sheet_parity("@media screen, ??? { .x { color: red; } }");
    assert!(never.diagnostics().iter().any(|diagnostic| {
        diagnostic.action() == CssRecoveryAction::ReplaceMediaQueryWithNever
    }));

    let implicit_sheet = assert_sheet_parity(".x { color: red;");
    assert!(
        implicit_sheet.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
    let implicit_style = assert_style_parity("--x: f(value");
    assert!(
        implicit_style.diagnostics().iter().any(|diagnostic| {
            diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
        })
    );
}

#[test]
fn app_strict_parity_preserves_structural_and_specialized_depth_diagnostics() {
    let mut structural = ".x{".repeat(257);
    structural.push_str("color:red;");
    structural.push_str(&"}".repeat(257));
    let structural = assert_sheet_parity(&structural);
    assert!(
        structural
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::StopAtNestingLimit })
    );

    let selector = assert_sheet_parity(&nested_selector(257));
    assert!(
        selector
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::StopAtNestingLimit })
    );

    let mut style = String::from("--x:");
    style.push_str(&"f(".repeat(257));
    style.push('x');
    style.push_str(&")".repeat(257));
    let style = assert_style_parity(&style);
    assert!(
        style
            .diagnostics()
            .iter()
            .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::StopAtNestingLimit })
    );
}
