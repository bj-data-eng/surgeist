#![forbid(unsafe_code)]
//! Cascade 5 #at-import admits string/URL targets independently of loading.
//! Values 4 (2024-03-12) #urls and #empty-urls recognize empty URL values;
//! Syntax 3 (2021-12-24) #consume-url-token distinguishes them from bad URLs.
use surgeist_css::{
    CssImportLayer, CssImportRule, CssImportString, CssImportTarget, CssImportUrl, CssMediaQuery,
    CssMediaType, CssNamespaceContext, CssRecoveryAction, CssRule, CssSerializedOrigin,
    CssSupportsConditionKind, CssValueOrigin, parse_rule, parse_sheet, validate_sheet,
};

// Authored spelling, URL-versus-string target, exact decoded value.
const TARGETS: &[(&str, bool, &str)] = &[
    ("\"\"", false, ""),
    ("''", false, ""),
    ("url()", true, ""),
    ("url(\"\")", true, ""),
    ("url('')", true, ""),
    ("url(   )", true, ""),
    ("\" \"", false, " "),
    ("url(\" \")", true, " "),
    (r#""\20""#, false, " "),
    (r"url(\20)", true, " "),
    (r#"url("\20")"#, true, " "),
];

fn assert_target(import: &CssImportRule, url: bool, decoded: &str) {
    match import.target() {
        CssImportTarget::Url(value) if url => assert_eq!(value.as_str(), decoded),
        CssImportTarget::String(value) if !url => assert_eq!(value.as_str(), decoded),
        other => panic!("wrong authored target variant: {other:?}"),
    }
}

#[test]
fn decoded_target_constructors_preserve_empty_and_whitespace_values() {
    for value in ["", " ", " \t\n ", "\u{00a0}", " theme.css "] {
        assert_eq!(
            CssImportUrl::try_new(value)
                .expect("decoded URL values need not name a usable resource")
                .as_str(),
            value
        );
        assert_eq!(
            CssImportString::try_new(value)
                .expect("decoded import strings need not name a usable resource")
                .as_str(),
            value
        );
    }
}

#[test]
fn empty_and_whitespace_targets_are_clean_sheet_rules_with_following_styles() {
    for &(target, url, decoded) in TARGETS {
        let source = format!("@import {target}; .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(validate_sheet(&source).unwrap(), *report.syntax());
        let [CssRule::Import(import), CssRule::Style(_)] = report.syntax().rules() else {
            panic!("retained import and following style: {source}");
        };
        assert_target(import, url, decoded);
        assert!(import.layer().is_none());
        assert!(import.supports().is_none());
        assert!(import.media().is_none());
    }
}

#[test]
fn empty_and_whitespace_targets_are_clean_exact_one_rule_fragments() {
    for &(target, url, decoded) in TARGETS {
        let source = format!("@import {target};");
        let report = parse_rule(&source, &CssNamespaceContext::default());
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let Some(CssRule::Import(import)) = report.syntax() else {
            panic!("one import fragment: {source}");
        };
        assert_target(import, url, decoded);
    }
}

#[test]
fn canonical_empty_target_imports_preserve_spelling_and_are_idempotent() {
    for &(target, url, decoded) in TARGETS {
        let source = format!("@IMPORT {target};");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Import(import)] = report.syntax().rules() else {
            panic!("one authored import: {source}");
        };
        let output = import.serialize().unwrap();
        assert_eq!(output.as_css(), format!("@import {target};"));
        let reparsed = parse_rule(output.as_css(), &CssNamespaceContext::default());
        assert!(reparsed.is_clean(), "{:?}", reparsed.diagnostics());
        let Some(CssRule::Import(import)) = reparsed.syntax() else {
            panic!("one reparsed import");
        };
        assert_target(import, url, decoded);
        assert_eq!(import.serialize().unwrap().as_css(), output.as_css());
    }
}

#[test]
fn empty_targets_preserve_layer_supports_and_symbolic_media_clauses() {
    for target in ["\"\"", "url()"] {
        let source = format!("@import {target} layer(theme) supports(display:grid) print;");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let [CssRule::Import(import)] = report.syntax().rules() else {
            panic!("one conditional import");
        };
        assert!(matches!(
            import.layer(),
            Some(CssImportLayer::Named(name)) if name.components() == ["theme"]
        ));
        let CssSupportsConditionKind::Declaration(declaration) =
            import.supports().unwrap().condition().kind()
        else {
            panic!("retained declaration supports condition");
        };
        assert_eq!(declaration.property(), "display");
        assert_eq!(declaration.authored().unwrap(), "display:grid");
        assert!(matches!(
            import.media().unwrap().queries(),
            [CssMediaQuery::Typed(query)] if query.media_type() == CssMediaType::Print
        ));
        assert_eq!(import.serialize().unwrap().as_css(), source);
    }
}

#[test]
fn initial_layer_statements_allow_empty_target_imports_and_following_imports() {
    let source = concat!(
        "@layer reset; @import ''; @import url(); ",
        "@import 'theme.css'; .after { color: red; }"
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [
        CssRule::LayerStatement(_),
        CssRule::Import(string),
        CssRule::Import(url),
        CssRule::Import(named),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("initial layer, all imports, and following style");
    };
    assert_target(string, false, "");
    assert_target(url, true, "");
    assert_target(named, false, "theme.css");
}

#[test]
fn empty_target_imports_advance_the_import_phase_before_later_layer_statements() {
    // Cascade 5 (2022-01-13), section 2: valid intervening rules break import
    // adjacency, even though an initial empty layer statement permits imports.
    let source = concat!(
        "@import ''; @layer middle; @import 'late.css'; ",
        ".after { color: red; }"
    );
    let report = parse_sheet(source);
    let [
        CssRule::Import(import),
        CssRule::LayerStatement(_),
        CssRule::Style(_),
    ] = report.syntax().rules()
    else {
        panic!("empty import survives and the late import is dropped");
    };
    assert_target(import, false, "");
    let [diagnostic] = report.diagnostics() else {
        panic!("only the late import is invalid");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert!(validate_sheet(source).is_err());
}

#[test]
fn an_empty_string_import_at_eof_gets_a_programmatic_semicolon_without_recovery() {
    let source = "@import ''";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(validate_sheet(source).unwrap(), *report.syntax());
    let [CssRule::Import(import)] = report.syntax().rules() else {
        panic!("one import statement ending at EOF");
    };
    assert_target(import, false, "");
    let output = import.serialize().unwrap();
    assert_eq!(output.as_css(), "@import '';");
    assert!(matches!(
        output.origin_at(output.as_css().len() - 1),
        Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
    ));
}

#[test]
fn eof_unclosed_empty_urls_retain_targets_with_implicit_closure_diagnostics() {
    // Syntax 3 returns the URL token at EOF and implicitly closes functions.
    // Empty target values are valid; the missing delimiter still needs recovery.
    for (source, expected) in [
        ("@import url(", "@import url();"),
        ("@import url(\"\"", "@import url(\"\");"),
    ] {
        let report = parse_sheet(source);
        let [CssRule::Import(import)] = report.syntax().rules() else {
            panic!("retained EOF import: {source}");
        };
        assert_target(import, true, "");
        let [diagnostic] = report.diagnostics() else {
            panic!("one missing URL delimiter: {source}");
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert!(validate_sheet(source).is_err(), "{source}");
        let output = import.serialize().unwrap();
        assert_eq!(output.as_css(), expected);
        assert!(matches!(
            output.origin_at(output.as_css().len() - 1),
            Some(CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
        ));
        assert!(parse_sheet(output.as_css()).is_clean());
    }
}

#[test]
fn missing_numeric_and_lexically_bad_targets_still_reject_without_losing_styles() {
    for invalid in [
        "@import;",
        "@import 1;",
        "@import url(a b);",
        "@import \"a\n;",
    ] {
        let fragment = parse_rule(invalid, &CssNamespaceContext::default());
        assert!(fragment.syntax().is_none(), "{invalid}");
        assert!(!fragment.is_clean(), "{invalid}");
        let source = format!("{invalid} .after {{ color: red; }}");
        let report = parse_sheet(&source);
        assert!(!report.is_clean(), "{invalid}");
        assert!(matches!(report.syntax().rules(), [CssRule::Style(_)]));
    }
}
