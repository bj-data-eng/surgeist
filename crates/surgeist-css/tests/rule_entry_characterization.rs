#![forbid(unsafe_code)]
//! Pinned Syntax 3 (2021-12-24) §§5, 5.3.5, 5.4.1–5.4.3.
//! Existing string fronts only; supplied token/component consumers are separate.
use surgeist_css::{
    CssErrorCode, CssImportTarget, CssImportance, CssNamespaceContext, CssRecoveryAction, CssRule,
    CssSelector, CssSupportsConditionKind, CssTokenKind, ErrorKind, parse_rule, parse_sheet,
};

fn assert_style(rule: &CssRule, source: &str, class: &str, value: &str) {
    let CssRule::Style(style) = rule else {
        panic!("retained style {class}: {rule:?}")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one selector")
    };
    assert_eq!(selector.selector(), &CssSelector::Class(class.into()));
    assert_eq!(
        style.position().byte_offset().value(),
        source.find(&format!(".{class}{{")).unwrap()
    );
    assert!(style.rules().is_empty());
    let [declaration] = style.declarations().as_slice() else {
        panic!("one color declaration")
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

fn assert_statement(rule: &CssRule, kind: &str) {
    match (rule, kind) {
        (CssRule::LayerStatement(layer), "layer") => {
            let [name] = layer.names().names() else {
                panic!("one layer name")
            };
            assert_eq!(name.components(), &["theme".to_owned()]);
        }
        (CssRule::Import(import), "import") => {
            let CssImportTarget::String(target) = import.target() else {
                panic!("authored import string")
            };
            assert_eq!(target.as_str(), "theme.css");
            assert!(import.layer().is_none());
            assert!(import.supports().is_none());
            assert!(import.media().is_none());
        }
        (CssRule::Namespace(namespace), "namespace") => {
            assert_eq!(namespace.prefix().unwrap().as_str(), "p");
            assert_eq!(namespace.name().as_str(), "urn:p");
        }
        _ => panic!("{kind}: {rule:?}"),
    }
}

#[test]
fn statement_eof_returns_the_context_valid_at_rule_with_an_original_eof_error() {
    let context = CssNamespaceContext::default();
    // All fronts execute before an EOF assertion can fail. The source requires
    // an error, but does not prescribe a new public error category or action.
    let cases = [
        ("@layer theme", "@layer theme;", "layer"),
        ("@import 'theme.css'", "@import 'theme.css';", "import"),
        ("@namespace p 'urn:p'", "@namespace p 'urn:p';", "namespace"),
    ]
    .map(|(source, terminated, kind)| {
        (
            source,
            kind,
            parse_sheet(source),
            parse_rule(source, &context),
            parse_sheet(terminated),
            parse_rule(terminated, &context),
        )
    });
    for (source, kind, sheet, fragment, terminated_sheet, terminated_fragment) in cases {
        assert!(terminated_sheet.is_clean(), "{terminated_sheet:?}");
        assert!(terminated_fragment.is_clean(), "{terminated_fragment:?}");
        let [rule] = sheet.syntax().rules() else {
            panic!("EOF returns the at-rule: {sheet:?}")
        };
        assert_statement(rule, kind);
        assert_statement(
            fragment.syntax().as_ref().expect("EOF returns one at-rule"),
            kind,
        );
        assert_statement(&terminated_sheet.syntax().rules()[0], kind);
        assert_statement(terminated_fragment.syntax().as_ref().unwrap(), kind);
        assert_eq!(fragment.syntax().as_ref().unwrap(), rule);
        assert!(!sheet.is_clean(), "Syntax 3 reports at-rule EOF: {source}");
        assert!(
            !fragment.is_clean(),
            "Syntax 3 reports at-rule EOF: {source}"
        );
        assert_eq!(sheet.diagnostics(), fragment.diagnostics());
        assert!(
            sheet.diagnostics().iter().any(|diagnostic| diagnostic
                .error()
                .position()
                .byte_offset()
                .value()
                == source.len())
        );
    }
}

#[test]
fn delimiters_inside_prelude_components_do_not_end_the_outer_at_rule() {
    let source =
        "/*😀*/\r\n@supports future(\";{}\" [a;b] {c;d} (@hidden;)){.kept{color:red!important}}";
    let sheet = parse_sheet(source);
    let fragment = parse_rule(source, &CssNamespaceContext::default());
    assert!(sheet.is_clean(), "{sheet:?}");
    assert!(fragment.is_clean(), "{fragment:?}");
    let [CssRule::Supports(supports)] = sheet.syntax().rules() else {
        panic!("one supports rule")
    };
    assert_eq!(
        fragment.syntax().as_ref().unwrap(),
        &sheet.syntax().rules()[0]
    );
    assert!(matches!(
        supports.condition().kind(),
        CssSupportsConditionKind::GeneralEnclosed(_)
    ));
    assert_eq!(
        supports.condition().serialize().unwrap().as_css().trim(),
        "future(\";{}\" [a;b] {c;d} (@hidden;))"
    );
    assert_eq!(
        supports.position().unwrap().byte_offset().value(),
        source.find("@supports").unwrap()
    );
    let [kept] = supports.rules() else {
        panic!("one retained child")
    };
    assert_style(kept, source, "kept", "red");
    let CssRule::Style(style) = kept else {
        unreachable!()
    };
    assert_eq!(
        style.declarations()[0].importance(),
        CssImportance::Important
    );
}

#[test]
fn semicolon_and_at_keyword_inside_qualified_prelude_do_not_restart_rule_consumption() {
    let failed = ".bad;@supports (color:red){.hidden{color:blue}}";
    let source = format!("{failed}.after{{color:red}}");
    let sheet = parse_sheet(&source);
    let fragment = parse_rule(failed, &CssNamespaceContext::default());
    let [after] = sheet.syntax().rules() else {
        panic!("only following sibling survives: {sheet:?}")
    };
    assert_style(after, &source, "after", "red");
    assert!(fragment.syntax().is_none());
    for (diagnostics, action) in [
        (sheet.diagnostics(), CssRecoveryAction::DropQualifiedRule),
        (fragment.diagnostics(), CssRecoveryAction::RejectInput),
    ] {
        let [diagnostic] = diagnostics else {
            panic!("one failed qualified unit")
        };
        assert_eq!(diagnostic.action(), action);
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.error().position().byte_offset().value(), 4);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), failed.len());
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("selector owns invalid prelude")
        };
        let token = detail.encountered().unwrap();
        assert_eq!(token.kind(), CssTokenKind::Semicolon);
        assert_eq!(token.authored(), ";");
    }
}

#[test]
fn qualified_eof_without_a_body_returns_no_rule_and_preserves_previous_sheet_sibling() {
    let fragment_source = " /*😀*/\r\n.missing";
    let sheet_source = " /*😀*/\r\n.before{color:red}.missing";
    let fragment = parse_rule(fragment_source, &CssNamespaceContext::default());
    let sheet = parse_sheet(sheet_source);
    assert!(fragment.syntax().is_none());
    let [before] = sheet.syntax().rules() else {
        panic!("only completed sibling survives")
    };
    assert_style(before, sheet_source, "before", "red");
    for (source, diagnostics, action, span_start, column) in [
        (
            fragment_source,
            fragment.diagnostics(),
            CssRecoveryAction::RejectInput,
            0,
            8,
        ),
        (
            sheet_source,
            sheet.diagnostics(),
            CssRecoveryAction::DropQualifiedRule,
            sheet_source.find(".missing").unwrap(),
            26,
        ),
    ] {
        let [diagnostic] = diagnostics else {
            panic!("one missing qualified body")
        };
        assert_eq!(diagnostic.action(), action);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidQualifiedRule
        );
        let position = diagnostic.error().position();
        assert_eq!(position.byte_offset().value(), source.len());
        assert_eq!(
            (position.line().value(), position.column().value()),
            (1, column)
        );
        assert_eq!(diagnostic.span().start().byte_offset().value(), span_start);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
        let ErrorKind::InvalidQualifiedRule(detail) = diagnostic.error().kind() else {
            panic!("missing required block")
        };
        assert!(detail.encountered().is_none());
    }
}

#[test]
fn nested_html_tokens_keep_the_following_at_keyword_in_one_failed_qualified_unit() {
    for (html, token) in [("<!--", CssTokenKind::Cdo), ("-->", CssTokenKind::Cdc)] {
        let unit = format!("{html}@supports (color:red){{.hidden{{color:red}}}}");
        let top_source = format!("{unit}.kept{{color:blue}}");
        let nested_source = format!("@media all{{{unit}.kept{{color:blue}}}}.after{{color:red}}");
        let top = parse_sheet(&top_source);
        let nested = parse_sheet(&nested_source);
        assert!(top.is_clean(), "{top:?}");
        let [CssRule::Supports(supports), kept] = top.syntax().rules() else {
            panic!("top-level HTML token is ignored")
        };
        let [hidden] = supports.rules() else {
            panic!("top-level supports child")
        };
        assert_style(hidden, &top_source, "hidden", "red");
        assert_style(kept, &top_source, "kept", "blue");
        let [CssRule::Media(media), after] = nested.syntax().rules() else {
            panic!("retained nested parent and sibling")
        };
        let [kept] = media.rules() else {
            panic!("nested HTML token consumes following at-keyword and block")
        };
        assert_style(kept, &nested_source, "kept", "blue");
        assert_style(after, &nested_source, "after", "red");
        let [diagnostic] = nested.diagnostics() else {
            panic!("one failed nested qualified unit")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
        assert_eq!(diagnostic.error().position().byte_offset().value(), 11);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 11);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            11 + unit.len()
        );
        let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
            panic!("nested selector error")
        };
        assert_eq!(detail.encountered().unwrap().kind(), token);
        assert_eq!(detail.encountered().unwrap().authored(), html);
    }
}

#[test]
fn same_source_retained_parent_has_equivalent_contextual_pruning_and_inner_diagnostics() {
    let failed = "@UnKnOwN future(\";\" [a;b] {c;d}) {.lost{color:red}}";
    let source = format!("/*😀*/\r\n@media all{{{failed}.kept{{color:blue}}}}");
    let sheet = parse_sheet(&source);
    let fragment = parse_rule(&source, &CssNamespaceContext::default());
    let [rule @ CssRule::Media(media)] = sheet.syntax().rules() else {
        panic!("retained outer parent")
    };
    assert_eq!(fragment.syntax().as_ref().unwrap(), rule);
    assert_eq!(fragment.diagnostics(), sheet.diagnostics());
    let [kept] = media.rules() else {
        panic!("unknown candidate and entire body are contextually pruned")
    };
    assert_style(kept, &source, "kept", "blue");
    let [diagnostic] = sheet.diagnostics() else {
        panic!("one local unknown-rule diagnostic")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropAtRule);
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownAtRule);
    let start = source.find(failed).unwrap();
    assert_eq!(diagnostic.error().position().byte_offset().value(), start);
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + failed.len()
    );
    let ErrorKind::UnknownAtRule(detail) = diagnostic.error().kind() else {
        panic!("unknown authored name")
    };
    assert_eq!(detail.name().as_str(), "UnKnOwN");
    assert!(sheet.into_validation_result().is_err());
    assert!(fragment.into_validation_result().is_err());
}
