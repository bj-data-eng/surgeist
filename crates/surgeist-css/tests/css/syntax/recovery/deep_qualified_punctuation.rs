#![forbid(unsafe_code)]
//! Syntax 3 §§5.4.1/5.4.3 keep a rule-list semicolon in the qualified prelude.
use surgeist_css::{
    CssErrorCode, CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssSelector, CssSheet,
    CssTokenKind, ErrorKind, parse_sheet, validate_sheet,
};

fn assert_style(rule: &CssRule, source: &str, class: &str, value: &str) {
    let CssRule::Style(style) = rule else {
        panic!("retained style {class}")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one class selector")
    };
    assert_eq!(selector.selector(), &CssSelector::Class(class.into()));
    assert_eq!(
        style.position().byte_offset().value(),
        source.find(&format!(".{class}{{")).unwrap()
    );
    let [declaration] = style.declarations().as_slice() else {
        panic!("one retained color declaration")
    };
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        value
    );
    let origin = declaration.parsed_value().unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert!(
        origin
            .source()
            .same_snapshot(declaration.parsed_name().unwrap().source())
    );
}

fn assert_style_semicolon_control() {
    let source = ".host{;color:red;;}";
    let report = parse_sheet(source);
    let validation = validate_sheet(source);
    assert!(
        report.is_clean(),
        "style-body separator control: {:?}",
        report.diagnostics()
    );
    let [host] = report.syntax().rules() else {
        panic!("one real style-body control")
    };
    assert_style(host, source, "host", "red");
    let CssRule::Style(style) = host else {
        unreachable!()
    };
    assert!(style.rules().is_empty());
    assert_eq!(validation.unwrap(), *report.syntax());
}

fn failed_unit(body_media_depth: usize) -> String {
    format!(
        ";/**/@supports (color:red){{{}.hidden{{color:red}}{}}}",
        "@media all{".repeat(body_media_depth),
        "}".repeat(body_media_depth)
    )
}

fn assert_semicolon_diagnostic(diagnostic: &CssRecoveryDiagnostic, source: &str, unit: &str) {
    let start = source.find(unit).unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.error().position().byte_offset().value(), start);
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + unit.len()
    );
    assert_eq!(diagnostic.error().position().line().value(), 1);
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..start]
            .rsplit_once("\r\n")
            .unwrap()
            .1
            .encode_utf16()
            .count()
    );
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("selector owns semicolon-prefixed qualified input")
    };
    let encountered = detail.encountered().unwrap();
    assert_eq!(encountered.kind(), CssTokenKind::Semicolon);
    assert_eq!(encountered.authored(), ";");
}

fn assert_nested_siblings(sheet: &CssSheet, source: &str, ancestors: usize, max_depth: usize) {
    let [CssRule::Media(first), after] = sheet.rules() else {
        panic!("Media chain and root sibling at maximum structural depth {max_depth}")
    };
    let mut parent = first;
    for level in 1..ancestors {
        let [CssRule::Media(nested)] = parent.rules() else {
            panic!(
                "missing Media at level {}, maximum structural depth {max_depth}",
                level + 1
            )
        };
        parent = nested;
    }
    let [kept] = parent.rules() else {
        panic!(
            "failed qualified unit must leave only its later child at Media depth {ancestors}, maximum structural depth {max_depth}; got {} retained children",
            parent.rules().len()
        )
    };
    assert_style(kept, source, "kept", "blue");
    assert_style(after, source, "after", "red");
}

fn check_nested(ancestors: usize, body_media_depth: usize) {
    let max_depth = ancestors + body_media_depth + 2;
    let unit = failed_unit(body_media_depth);
    let source = format!(
        "/*😀*/\r\n{}{unit}.kept{{color:blue}}{}.after{{color:red}}",
        "@media all{".repeat(ancestors),
        "}".repeat(ancestors)
    );
    let report = parse_sheet(&source);
    let validation = validate_sheet(&source);
    // This genuine distinct-grammar control completes before any failed-unit assertion.
    assert_style_semicolon_control();
    assert_nested_siblings(report.syntax(), &source, ancestors, max_depth);
    let [diagnostic] = report.diagnostics() else {
        panic!("one complete semicolon-prefixed unit at maximum structural depth {max_depth}")
    };
    assert_semicolon_diagnostic(diagnostic, &source, &unit);
    assert_eq!(validation.unwrap_err().diagnostics(), report.diagnostics());
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
fn shallow_rule_list_semicolon_keeps_supports_in_one_failed_qualified_unit() {
    check_nested(1, 0);
}

#[test]
fn selected_chunk_semicolon_keeps_supports_in_one_failed_qualified_unit() {
    check_nested(63, 0);
}

#[test]
fn bounded_deep_semicolon_keeps_supports_in_one_failed_qualified_unit() {
    check_nested(63, 64);
}

#[test]
fn root_rule_list_semicolon_keeps_supports_in_one_failed_qualified_unit() {
    let unit = failed_unit(0);
    let source = format!("/*😀*/\r\n{unit}.kept{{color:blue}}.after{{color:red}}");
    let report = parse_sheet(&source);
    let validation = validate_sheet(&source);
    assert_style_semicolon_control();
    let [kept, after] = report.syntax().rules() else {
        panic!("root qualified unit cannot promote Supports or its hidden child")
    };
    assert_style(kept, &source, "kept", "blue");
    assert_style(after, &source, "after", "red");
    let [diagnostic] = report.diagnostics() else {
        panic!("one root semicolon-prefixed qualified unit")
    };
    assert_semicolon_diagnostic(diagnostic, &source, &unit);
    assert_eq!(validation.unwrap_err().diagnostics(), report.diagnostics());
    assert_eq!(
        report
            .clone()
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        report.diagnostics()
    );
}
