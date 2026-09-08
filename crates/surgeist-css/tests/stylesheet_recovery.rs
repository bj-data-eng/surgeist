use surgeist_css::{CssErrorCode, CssRecoveryAction, CssRule, ErrorKind, parse_sheet};

fn style_rule_names(report: &surgeist_css::CssParseReport<surgeist_css::CssSheet>) -> Vec<&str> {
    report
        .syntax()
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Style(rule) => match rule.selector() {
                surgeist_css::CssSelector::Class(name) => Some(name.as_str()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn assert_drop(
    source: &str,
    diagnostic: &surgeist_css::CssRecoveryDiagnostic,
    code: CssErrorCode,
    action: CssRecoveryAction,
    span: &str,
) {
    assert_eq!(diagnostic.error().code(), code);
    assert_eq!(diagnostic.action(), action);
    let start = source
        .find(span)
        .expect("recovery unit must occur in source");
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + span.len()
    );
    assert!(diagnostic.span().start() < diagnostic.span().end());
}

fn assert_root_token_drop(
    source: &str,
    diagnostic: &surgeist_css::CssRecoveryDiagnostic,
    start: usize,
    authored: &str,
    kind: surgeist_css::CssTokenKind,
) {
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidQualifiedRule
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_eq!(diagnostic.error().position().byte_offset().value(), start);
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + authored.len()
    );
    assert_eq!(&source[start..start + authored.len()], authored);
    assert!(diagnostic.span().start() < diagnostic.span().end());

    let ErrorKind::InvalidQualifiedRule(detail) = diagnostic.error().kind() else {
        panic!("expected invalid qualified-rule detail")
    };
    assert_eq!(detail.production().as_str(), "css.qualified-rule");
    assert_eq!(detail.expectation().as_str(), "valid CSS syntax");
    let encountered = detail.encountered().expect("responsible root token");
    assert_eq!(encountered.kind(), kind);
    assert_eq!(encountered.authored(), authored);
}

fn assert_nonleading_encoding_drop(source: &str, diagnostic: &surgeist_css::CssRecoveryDiagnostic) {
    assert_drop(
        source,
        diagnostic,
        CssErrorCode::InvalidEncodingDeclaration,
        CssRecoveryAction::DropAtRule,
        "@charset \"UTF-8\";",
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("\"UTF-8\"").unwrap()
    );
    let ErrorKind::InvalidEncodingDeclaration(detail) = diagnostic.error().kind() else {
        panic!("expected encoding declaration detail")
    };
    assert_eq!(
        detail.expectation().as_str(),
        "a non-empty double-quoted encoding label followed by a semicolon"
    );
    let encountered = detail.encountered().expect("responsible encoding token");
    assert_eq!(encountered.kind(), surgeist_css::CssTokenKind::String);
    assert_eq!(encountered.authored(), "\"UTF-8\"");
}

#[test]
fn stylesheet_recovery_front_door_has_report_signature() {
    fn require_signature(_: fn(&str) -> surgeist_css::CssParseReport<surgeist_css::CssSheet>) {}

    require_signature(parse_sheet);
}

#[test]
fn stylesheet_recovery_empty_input_returns_clean_empty_report_and_parts() {
    let report = parse_sheet("");

    assert!(report.is_clean());
    assert!(report.syntax().rules().is_empty());
    assert!(report.syntax().encoding().is_none());

    let (sheet, diagnostics) = report.into_parts();
    assert!(sheet.rules().is_empty());
    assert!(sheet.encoding().is_none());
    assert!(diagnostics.is_empty());
}

#[test]
fn stylesheet_recovery_top_level_cdo_is_reported_before_and_between_valid_rules() {
    let source = "<!-- .before { color: red; } <!-- .after { color: blue; }";

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, start) in report.diagnostics().iter().zip([0, 29]) {
        assert_eq!(diagnostic.action(), CssRecoveryAction::IgnoreLegacyToken);
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(diagnostic.span().end().byte_offset().value(), start + 4);
    }
}

#[test]
fn stylesheet_recovery_top_level_cdc_is_reported_before_and_between_valid_rules() {
    let source = "--> .before { color: red; } --> .after { color: blue; }";

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, start) in report.diagnostics().iter().zip([0, 28]) {
        assert_eq!(diagnostic.action(), CssRecoveryAction::IgnoreLegacyToken);
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedToken);
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(diagnostic.span().end().byte_offset().value(), start + 3);
    }
}

#[test]
fn stylesheet_recovery_root_semicolon_before_valid_rule_is_one_exact_drop() {
    let source = "; .after { color: blue; }";

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["after"]);
    assert_eq!(report.diagnostics().len(), 1);
    assert_root_token_drop(
        source,
        &report.diagnostics()[0],
        0,
        ";",
        surgeist_css::CssTokenKind::Semicolon,
    );
}

#[test]
fn stylesheet_recovery_root_semicolon_between_valid_rules_is_one_exact_drop() {
    let source = ".before { color: red; } ; .after { color: blue; }";
    let stray = source.find("} ;").unwrap() + 2;

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert_eq!(report.diagnostics().len(), 1);
    assert_root_token_drop(
        source,
        &report.diagnostics()[0],
        stray,
        ";",
        surgeist_css::CssTokenKind::Semicolon,
    );
}

#[test]
fn stylesheet_recovery_unmatched_root_closing_brace_before_valid_rule_is_one_exact_drop() {
    let source = "} .after { color: blue; }";

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["after"]);
    assert_eq!(report.diagnostics().len(), 1);
    assert_root_token_drop(
        source,
        &report.diagnostics()[0],
        0,
        "}",
        surgeist_css::CssTokenKind::CloseCurlyBracket,
    );
}

#[test]
fn stylesheet_recovery_unmatched_root_closing_brace_between_valid_rules_is_one_exact_drop() {
    let source = ".before { color: red; } } .after { color: blue; }";
    let stray = source.find("} }").unwrap() + 2;

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert_eq!(report.diagnostics().len(), 1);
    assert_root_token_drop(
        source,
        &report.diagnostics()[0],
        stray,
        "}",
        surgeist_css::CssTokenKind::CloseCurlyBracket,
    );
}

#[test]
fn stylesheet_recovery_root_semicolon_makes_following_encoding_nonleading() {
    let source = "; @charset \"UTF-8\"; .after { color: blue; }";

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["after"]);
    assert!(report.syntax().encoding().is_none());
    assert_eq!(report.diagnostics().len(), 2);
    assert_root_token_drop(
        source,
        &report.diagnostics()[0],
        0,
        ";",
        surgeist_css::CssTokenKind::Semicolon,
    );
    assert_nonleading_encoding_drop(source, &report.diagnostics()[1]);
    assert!(report.diagnostics()[0].span() < report.diagnostics()[1].span());
}

#[test]
fn stylesheet_recovery_unmatched_root_closing_brace_makes_following_encoding_nonleading() {
    let source = "} @charset \"UTF-8\"; .after { color: blue; }";

    let report = parse_sheet(source);

    assert_eq!(style_rule_names(&report), ["after"]);
    assert!(report.syntax().encoding().is_none());
    assert_eq!(report.diagnostics().len(), 2);
    assert_root_token_drop(
        source,
        &report.diagnostics()[0],
        0,
        "}",
        surgeist_css::CssTokenKind::CloseCurlyBracket,
    );
    assert_nonleading_encoding_drop(source, &report.diagnostics()[1]);
    assert!(report.diagnostics()[0].span() < report.diagnostics()[1].span());
}

#[test]
fn stylesheet_recovery_trivia_and_legacy_tokens_alone_report_each_legacy_token() {
    let report = parse_sheet(" \n/**/ <!-- --> \t");

    assert_eq!(report.diagnostics().len(), 2);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.action() == CssRecoveryAction::IgnoreLegacyToken)
    );
    assert!(report.syntax().rules().is_empty());
    assert!(report.syntax().encoding().is_none());
}

#[test]
fn stylesheet_recovery_unknown_block_at_rule_keeps_surrounding_rules_and_balanced_span() {
    let failed = "@mystery one(foo; bar) { nested: {x; y}; }";
    let source = format!(".before {{ color: red; }} {failed} .after {{ color: blue; }}");

    let report = parse_sheet(&source);

    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_drop(
        &source,
        diagnostic,
        CssErrorCode::UnknownAtRule,
        CssRecoveryAction::DropAtRule,
        failed,
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find(failed).unwrap()
    );
    let ErrorKind::UnknownAtRule(detail) = diagnostic.error().kind() else {
        panic!("expected unknown at-rule detail")
    };
    assert_eq!(detail.name().as_str(), "mystery");
}

#[test]
fn stylesheet_recovery_malformed_namespace_rule_is_distinct() {
    let failed = "@namespace svg ident;";
    let source = format!("{failed} .after {{ color: blue; }}");

    let report = parse_sheet(&source);

    assert_eq!(style_rule_names(&report), ["after"]);
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_drop(
        &source,
        diagnostic,
        CssErrorCode::InvalidAtRulePrelude,
        CssRecoveryAction::DropAtRule,
        failed,
    );
    let ErrorKind::InvalidAtRulePrelude(detail) = diagnostic.error().kind() else {
        panic!("expected namespace prelude detail")
    };
    assert_eq!(detail.name().as_str(), "namespace");
    assert_eq!(detail.production().as_str(), "later.rule.namespace");
}

#[test]
fn stylesheet_recovery_malformed_qualified_rule_keeps_surrounding_rules() {
    let failed = "??? { width: 1px; nested: fn({x;y}); }";
    let source = format!(".before {{ color: red; }} {failed} .after {{ color: blue; }}");

    let report = parse_sheet(&source);

    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert_eq!(report.diagnostics().len(), 1);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        failed,
    );
}

#[test]
fn stylesheet_recovery_valid_leading_encoding_is_metadata_not_a_rule() {
    let source = "\u{feff} /* leading */ @charset \"UTF-8\"; .after { color: blue; }";

    let report = parse_sheet(source);

    assert!(report.is_clean());
    assert_eq!(style_rule_names(&report), ["after"]);
    let encoding = report
        .syntax()
        .encoding()
        .expect("leading encoding metadata");
    assert_eq!(encoding.label(), "UTF-8");
    assert_eq!(
        encoding.position().byte_offset().value(),
        source.find("@charset").unwrap()
    );
}

#[test]
fn stylesheet_recovery_invalid_encoding_forms_drop_once_and_resume() {
    for failed in [
        "@charset UTF-8;",
        "@charset \"\";",
        "@charset \"UTF-8\" { ignored; }",
    ] {
        let source = format!("{failed} .after {{ color: blue; }}");
        let report = parse_sheet(&source);

        assert_eq!(style_rule_names(&report), ["after"], "{failed}");
        assert!(report.syntax().encoding().is_none(), "{failed}");
        assert_eq!(report.diagnostics().len(), 1, "{failed}");
        assert_drop(
            &source,
            &report.diagnostics()[0],
            CssErrorCode::InvalidEncodingDeclaration,
            CssRecoveryAction::DropAtRule,
            failed,
        );
    }
}

#[test]
fn stylesheet_recovery_duplicate_and_nonleading_encoding_are_dropped() {
    let duplicate = "@charset \"latin1\";";
    let source = format!(
        "@charset \"UTF-8\"; .before {{ color: red; }} {duplicate} .after {{ color: blue; }}"
    );

    let report = parse_sheet(&source);

    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert_eq!(report.syntax().encoding().unwrap().label(), "UTF-8");
    assert_eq!(report.diagnostics().len(), 1);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidEncodingDeclaration,
        CssRecoveryAction::DropAtRule,
        duplicate,
    );

    let source = ".before { color: red; } @charset \"UTF-8\"; .after { color: blue; }";
    let report = parse_sheet(source);
    assert_eq!(style_rule_names(&report), ["before", "after"]);
    assert!(report.syntax().encoding().is_none());
    assert_eq!(report.diagnostics().len(), 1);
    assert_drop(
        source,
        &report.diagnostics()[0],
        CssErrorCode::InvalidEncodingDeclaration,
        CssRecoveryAction::DropAtRule,
        "@charset \"UTF-8\";",
    );
}

#[test]
fn stylesheet_recovery_top_level_failure_classes_have_one_exact_parent_drop() {
    struct Case {
        failed: &'static str,
        code: CssErrorCode,
        action: CssRecoveryAction,
        responsible: &'static str,
    }

    let cases = [
        Case {
            failed: "@unknown value;",
            code: CssErrorCode::UnknownAtRule,
            action: CssRecoveryAction::DropAtRule,
            responsible: "@unknown",
        },
        Case {
            failed: "@import \"late.css\";",
            code: CssErrorCode::InvalidAtRulePlacement,
            action: CssRecoveryAction::DropAtRule,
            responsible: "@import",
        },
        Case {
            failed: "@font-face nope { font-family: Test; src: url(test.woff2); }",
            code: CssErrorCode::InvalidAtRulePrelude,
            action: CssRecoveryAction::DropAtRule,
            responsible: "nope",
        },
        Case {
            failed: "@media screen;",
            code: CssErrorCode::InvalidAtRuleBody,
            action: CssRecoveryAction::DropAtRule,
            responsible: "<end>",
        },
        Case {
            failed: "??? { color: red; }",
            code: CssErrorCode::InvalidSelector,
            action: CssRecoveryAction::DropQualifiedRule,
            responsible: "???",
        },
    ];

    for case in cases {
        let source = format!(
            ".before {{ color: red; }} {} .after {{ color: blue; }}",
            case.failed
        );
        let report = parse_sheet(&source);
        assert_eq!(
            style_rule_names(&report),
            ["before", "after"],
            "{}",
            case.failed
        );
        assert_eq!(report.diagnostics().len(), 1, "{}", case.failed);
        let diagnostic = &report.diagnostics()[0];
        assert_drop(&source, diagnostic, case.code, case.action, case.failed);
        let failed_start = source.find(case.failed).unwrap();
        let responsible = if case.responsible == "<end>" {
            failed_start + case.failed.len()
        } else {
            source[failed_start..]
                .find(case.responsible)
                .map(|offset| failed_start + offset)
                .unwrap()
        };
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible,
            "{}",
            case.failed
        );
    }
}

#[test]
fn stylesheet_recovery_repeated_drops_remain_in_source_order() {
    let first = "@one fn({a;b});";
    let second = "??? { nested: {x;y}; }";
    let source = format!(
        ".before {{ color: red; }} {first} .middle {{ color: green; }} {second} .after {{ color: blue; }}"
    );

    let report = parse_sheet(&source);

    assert_eq!(style_rule_names(&report), ["before", "middle", "after"]);
    assert_eq!(report.diagnostics().len(), 2);
    assert_drop(
        &source,
        &report.diagnostics()[0],
        CssErrorCode::UnknownAtRule,
        CssRecoveryAction::DropAtRule,
        first,
    );
    assert_drop(
        &source,
        &report.diagnostics()[1],
        CssErrorCode::InvalidSelector,
        CssRecoveryAction::DropQualifiedRule,
        second,
    );
    assert!(report.diagnostics()[0].span() < report.diagnostics()[1].span());

    let (sheet, diagnostics) = report.into_parts();
    assert_eq!(sheet.rules().len(), 3);
    assert_eq!(diagnostics.len(), 2);
}

#[test]
fn stylesheet_recovery_encoding_leading_trivia_is_not_a_recovery_unit() {
    for leading in ["", " \n\t", "/**/", "\u{feff}", "\u{feff} /* comment */ "] {
        let source = format!("{leading}@charset \"Shift_JIS\"; .after {{ color: blue; }}");
        let report = parse_sheet(&source);

        assert!(report.is_clean(), "{leading:?}");
        assert_eq!(report.syntax().encoding().unwrap().label(), "Shift_JIS");
        assert_eq!(style_rule_names(&report), ["after"]);
    }
}

#[test]
fn stylesheet_recovery_encoding_errors_expose_exact_payload_and_position() {
    struct Case {
        source: &'static str,
        position: usize,
        encountered: Option<surgeist_css::CssTokenKind>,
    }

    let cases = [
        Case {
            source: "@charset UTF-8;",
            position: 9,
            encountered: Some(surgeist_css::CssTokenKind::Ident),
        },
        Case {
            source: "@charset \"\";",
            position: 9,
            encountered: Some(surgeist_css::CssTokenKind::String),
        },
        Case {
            source: "@charset 'UTF-8';",
            position: 9,
            encountered: Some(surgeist_css::CssTokenKind::String),
        },
        Case {
            source: "@charset /*comment*/ 'UTF-8';",
            position: 21,
            encountered: Some(surgeist_css::CssTokenKind::String),
        },
        Case {
            source: "@charset \"UTF-8\"",
            position: 16,
            encountered: None,
        },
        Case {
            source: "@charset \"UTF-8\" {}",
            position: 17,
            encountered: None,
        },
    ];

    for case in cases {
        let report = parse_sheet(case.source);
        assert!(report.syntax().encoding().is_none(), "{}", case.source);
        assert!(report.syntax().rules().is_empty(), "{}", case.source);
        assert_eq!(report.diagnostics().len(), 1, "{}", case.source);
        let diagnostic = &report.diagnostics()[0];
        assert_drop(
            case.source,
            diagnostic,
            CssErrorCode::InvalidEncodingDeclaration,
            CssRecoveryAction::DropAtRule,
            case.source,
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            case.position,
            "{}",
            case.source
        );
        let ErrorKind::InvalidEncodingDeclaration(detail) = diagnostic.error().kind() else {
            panic!("expected encoding declaration detail")
        };
        assert_eq!(
            detail.expectation().as_str(),
            "a non-empty double-quoted encoding label followed by a semicolon"
        );
        assert_eq!(
            detail.encountered().map(|token| token.kind()),
            case.encountered
        );
    }
}
