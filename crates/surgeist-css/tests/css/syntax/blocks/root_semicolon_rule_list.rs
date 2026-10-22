//! Pinned Syntax 3 CRD20261001 §§5.5.1/5.5.3: unmatched root punctuation is
//! qualified-rule input, not an independently discarded stylesheet separator.
//! Only §5.5.5 "consume a block's contents" discards a `<semicolon-token>`.
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-stylesheet-contents
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-qualified-rule
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-block-contents
use surgeist_css::{
    CssErrorCode, CssRecoveryAction, CssRecoveryDiagnostic, CssRule, CssSelector, CssTokenKind,
    ErrorKind, parse_sheet, validate_sheet,
};

fn assert_style(rule: &CssRule, source: &str, class: &str, value: &str) {
    let CssRule::Style(style) = rule else {
        panic!("style {class}: {rule:?}")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one class")
    };
    assert_eq!(selector.selector(), &CssSelector::Class(class.into()));
    assert_eq!(
        style.position().byte_offset().value(),
        source.find(&format!(".{class}{{")).unwrap()
    );
    let [declaration] = style.declarations().as_slice() else {
        panic!("one surviving color declaration")
    };
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        value
    );
    let origin = declaration.parsed_value().unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        &source[origin.span().start().byte_offset().value()
            ..origin.span().end().byte_offset().value()],
        value
    );
    assert!(
        origin
            .source()
            .same_snapshot(declaration.parsed_name().unwrap().source())
    );
}

fn assert_punctuation_unit(
    diagnostic: &CssRecoveryDiagnostic,
    source: &str,
    unit: &str,
    token: CssTokenKind,
) {
    let start = source.find(unit).unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + unit.len()
    );
    let position = diagnostic.error().position();
    assert_eq!(position.byte_offset().value(), start);
    assert_eq!(position.line().value(), 1);
    assert_eq!(
        position.column().value() as usize,
        source[..start]
            .rsplit_once("\r\n")
            .unwrap()
            .1
            .encode_utf16()
            .count()
    );
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("typed style selector context")
    };
    assert_eq!(
        detail.production().unwrap().as_str(),
        "baseline.selector.complex"
    );
    let encountered = detail
        .encountered()
        .expect("first responsible punctuation token");
    assert_eq!(encountered.kind(), token);
    assert_eq!(encountered.authored(), &unit[..1]);
}

#[test]
fn unmatched_root_punctuation_consumes_the_following_style_body_as_one_invalid_qualified_rule() {
    let cases: Vec<_> = [
        (";", CssTokenKind::Semicolon),
        ("}", CssTokenKind::CloseCurlyBracket),
    ]
    .into_iter()
    .map(|(punctuation, token)| {
        let unit = format!("{punctuation}/**/.after{{--punct:'}}';color:blue}}");
        let source = format!("/*😀*/\r\n.before{{color:red}} /*é*/\t{unit}.last{{color:green}}");
        let report = parse_sheet(&source);
        let validation = validate_sheet(&source);
        (source, unit, token, report, validation)
    })
    .collect();
    // Both token families and both public fronts execute before either admission assertion.
    for (source, unit, token, report, validation) in cases {
        let [before, last] = report.syntax().rules() else {
            panic!("only completed earlier/later siblings survive: {report:?}")
        };
        assert_style(before, &source, "before", "red");
        assert_style(last, &source, "last", "green");
        let [diagnostic] = report.diagnostics() else {
            panic!("one complete failed qualified unit")
        };
        assert_punctuation_unit(diagnostic, &source, &unit, token);
        assert_eq!(validation.unwrap_err().diagnostics(), report.diagnostics());
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn at_keyword_inside_punctuation_started_prelude_does_not_restart_at_rule_dispatch() {
    let cases: Vec<_> = [
        (";", CssTokenKind::Semicolon),
        ("}", CssTokenKind::CloseCurlyBracket),
    ]
    .into_iter()
    .map(|(punctuation, token)| {
        let unit = format!("{punctuation} @supports (color:red){{.hidden{{color:blue}}}}");
        let source = format!("/*😀*/\r\n.before{{color:red}} /*é*/ {unit}.last{{color:green}}");
        let report = parse_sheet(&source);
        let validation = validate_sheet(&source);
        (source, unit, token, report, validation)
    })
    .collect();
    for (source, unit, token, report, validation) in cases {
        let [before, last] = report.syntax().rules() else {
            panic!("no restarted supports node or escaped child: {report:?}")
        };
        assert_style(before, &source, "before", "red");
        assert_style(last, &source, "last", "green");
        let [diagnostic] = report.diagnostics() else {
            panic!("one punctuation-started qualified unit")
        };
        assert_punctuation_unit(diagnostic, &source, &unit, token);
        assert_eq!(validation.unwrap_err().diagnostics(), report.diagnostics());
    }
}

#[test]
fn block_contents_discard_semicolons_in_group_and_style_bodies_but_not_at_the_root() {
    // A group block is consumed by consume a block's contents, which discards
    // the `;`; only the stylesheet root keeps it as qualified-rule input.
    let nested_source =
        "@media all{.before{color:red};.kept{color:blue}.last{color:green}}.outside{color:red}";
    let style_source = ".host{;color:red;;}";
    let nested = parse_sheet(nested_source);
    let style = parse_sheet(style_source);
    let nested_validation = validate_sheet(nested_source);
    let style_validation = validate_sheet(style_source);
    let [CssRule::Media(media), outside] = nested.syntax().rules() else {
        panic!("real parent closing brace ends nested list")
    };
    let [before, kept, last] = media.rules() else {
        panic!("a rule after a discarded nested semicolon is retained: {media:?}")
    };
    assert_style(before, nested_source, "before", "red");
    assert_style(kept, nested_source, "kept", "blue");
    assert_style(last, nested_source, "last", "green");
    assert_style(outside, nested_source, "outside", "red");
    assert!(nested.is_clean(), "{nested:?}");
    assert_eq!(nested_validation.unwrap(), *nested.syntax());
    assert!(style.is_clean(), "{style:?}");
    let [host] = style.syntax().rules() else {
        panic!("one clean style rule")
    };
    // This helper's positional literal has optional separators before color.
    let CssRule::Style(host_style) = host else {
        panic!("host style")
    };
    assert_eq!(
        host_style.selectors().selectors()[0].selector(),
        &CssSelector::Class("host".into())
    );
    assert!(host_style.rules().is_empty());
    assert_eq!(host_style.declarations().len(), 1);
    assert_eq!(
        host_style.declarations()[0]
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        "red"
    );
    assert_eq!(style_validation.unwrap(), *style.syntax());
}

#[test]
fn top_level_semicolon_begins_a_dropped_qualified_rule() {
    // Consume a stylesheet's contents has no `<semicolon-token>` branch: `;.x{}`
    // is one qualified rule whose selector is invalid.
    let source = ";.x{color:red}.y{color:green}";
    let report = parse_sheet(source);
    let [CssRule::Style(y)] = report.syntax().rules() else {
        panic!("only .y survives: {report:?}")
    };
    assert_eq!(
        y.selectors().selectors()[0].selector(),
        &CssSelector::Class("y".into())
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one dropped qualified rule: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        source.find(".y").unwrap()
    );
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}
