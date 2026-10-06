#![forbid(unsafe_code)]
//! CSS Syntax 3 (2021-12-24), sections 3, 3.2, 5.4.1 and 9.3.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/
//! These UTF-8 string fronts do not run the byte-stream fallback algorithm.
use surgeist_css::{
    CssErrorCode, CssNamespaceContext, CssParseReport, CssRecoveryAction, CssRecoveryDiagnostic,
    CssRule, CssSelector, CssSheet, CssSourcePosition, CssTokenKind, ErrorKind, parse_rule,
    parse_sheet, validate_sheet,
};

type ExpectedStyle<'a> = (&'a str, (usize, u32, u32), (usize, usize));
type ExpectedDrop<'a> = ((usize, u32, u32), usize, &'a str);

fn assert_position(position: CssSourcePosition, expected: (usize, u32, u32)) {
    assert_eq!(position.byte_offset().value(), expected.0);
    assert_eq!(position.line().value(), expected.1);
    assert_eq!(position.column().value(), expected.2);
}

fn assert_styles(source: &str, rules: &[CssRule], expected: &[ExpectedStyle<'_>]) {
    assert_eq!(rules.len(), expected.len(), "{rules:?}");
    let mut first_origin = None;
    for (rule, &(name, position, value_span)) in rules.iter().zip(expected) {
        let CssRule::Style(style) = rule else {
            panic!("one authored style rule: {rule:?}")
        };
        let [selector] = style.selectors().selectors() else {
            panic!("one authored selector")
        };
        assert_eq!(selector.selector(), &CssSelector::Class(name.into()));
        assert_position(style.position(), position);
        assert!(style.rules().is_empty());
        let [declaration] = style.declarations().as_slice() else {
            panic!("one retained color declaration")
        };
        let name_origin = declaration.parsed_name().expect("original property name");
        let value_origin = declaration.parsed_value().expect("original property value");
        assert_eq!(name_origin.source().as_str(), source);
        assert_eq!(value_origin.source().as_str(), source);
        assert!(name_origin.source().same_snapshot(value_origin.source()));
        assert_eq!(
            value_origin.span().start().byte_offset().value(),
            value_span.0
        );
        assert_eq!(
            value_origin.span().end().byte_offset().value(),
            value_span.1
        );
        assert_eq!(
            declaration.value_components().serialize().unwrap().as_css(),
            &source[value_span.0..value_span.1]
        );
        if let Some(first) = first_origin {
            assert!(value_origin.source().same_snapshot(first));
        } else {
            first_origin = Some(value_origin.source());
        }
    }
}

fn assert_validator_parity(source: &str, report: &CssParseReport<CssSheet>) {
    assert_eq!(
        validate_sheet(source),
        report.clone().into_validation_result()
    );
}

fn assert_unknown(
    diagnostic: &CssRecoveryDiagnostic,
    expected: ExpectedDrop<'_>,
    action: CssRecoveryAction,
) {
    let (position, end, name) = expected;
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(diagnostic.action(), action);
    assert_position(diagnostic.error().position(), position);
    assert_position(diagnostic.span().start(), position);
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    let ErrorKind::UnknownAtRule(detail) = diagnostic.error().kind() else {
        panic!("an unrecognized at-rule: {diagnostic:?}")
    };
    assert_eq!(detail.name().as_str(), name);
}

fn assert_charset_sheet(source: &str, drops: &[ExpectedDrop<'_>], styles: &[ExpectedStyle<'_>]) {
    let report = parse_sheet(source);
    assert_styles(source, report.syntax().rules(), styles);
    assert_eq!(report.diagnostics().len(), drops.len(), "{report:?}");
    for (diagnostic, &expected) in report.diagnostics().iter().zip(drops) {
        assert_unknown(diagnostic, expected, CssRecoveryAction::DropAtRule);
    }
    assert!(!report.is_clean());
    assert_validator_parity(source, &report);
}

// Each fixture is its own executable test, so one leading-metadata failure
// cannot hide a later invalid-spelling or source-coordinate contract.
macro_rules! charset_sheet_case {
    ($name:ident, $source:literal, $end:literal, $start:literal, $value_start:literal, $value_end:literal) => {
        #[test]
        fn $name() {
            assert_charset_sheet(
                $source,
                &[((0, 0, 0), $end, "charset")],
                &[("after", ($start, 0, $start), ($value_start, $value_end))],
            );
        }
    };
}

charset_sheet_case!(
    exact_ascii_charset_prefix_is_an_unknown_string_input_rule,
    "@charset \"UTF-8\";.after{color:blue}",
    17,
    17,
    30,
    34
);
charset_sheet_case!(
    multiple_spaces_do_not_make_a_charset_rule_valid,
    "@charset  \"UTF-8\";.after{color:blue}",
    18,
    18,
    31,
    35
);
charset_sheet_case!(
    comment_in_charset_prefix_does_not_make_a_charset_rule_valid,
    "@charset/**/\"UTF-8\";.after{color:blue}",
    20,
    20,
    33,
    37
);
charset_sheet_case!(
    single_quoted_charset_is_unknown_rather_than_an_encoding_error,
    "@charset 'UTF-8';.after{color:blue}",
    17,
    17,
    30,
    34
);
charset_sheet_case!(
    identifier_charset_label_is_unknown_rather_than_an_encoding_error,
    "@charset UTF-8;.after{color:blue}",
    15,
    15,
    28,
    32
);
charset_sheet_case!(
    empty_charset_label_is_unknown_rather_than_an_encoding_error,
    "@charset \"\";.after{color:blue}",
    12,
    12,
    25,
    29
);
charset_sheet_case!(
    block_charset_drops_its_complete_balanced_body_and_retains_next_rule,
    "@charset \"UTF-8\"{ignored:{x;y}}.after{color:blue}",
    31,
    31,
    44,
    48
);

#[test]
fn uppercase_charset_is_unrecognized_with_its_authored_name() {
    assert_charset_sheet(
        "@CHARSET \"UTF-8\";.after{color:blue}",
        &[((0, 0, 0), 17, "CHARSET")],
        &[("after", (17, 0, 17), (30, 34))],
    );
}

#[test]
fn leading_unicode_comment_and_crlf_preserve_original_charset_and_sibling_positions() {
    assert_charset_sheet(
        "/*😀*/\r\n@charset \"UTF-8\";.after{color:blue}",
        &[((10, 1, 0), 27, "charset")],
        &[("after", (27, 1, 17), (40, 44))],
    );
}

#[test]
fn a_legacy_encoding_label_does_not_redecode_the_supplied_utf8_string() {
    assert_charset_sheet(
        "@charset \"windows-1252\";.café{color:red}",
        &[((0, 0, 0), 24, "charset")],
        &[("café", (24, 0, 24), (37, 40))],
    );
}

#[test]
fn nonleading_charset_is_unknown_and_keeps_both_original_siblings() {
    assert_charset_sheet(
        ".before{color:red}@charset \"UTF-8\";.after{color:blue}",
        &[((18, 0, 18), 35, "charset")],
        &[
            ("before", (0, 0, 0), (14, 17)),
            ("after", (35, 0, 35), (48, 52)),
        ],
    );
}

#[test]
fn every_duplicate_charset_occurrence_has_its_own_unknown_rule_diagnostic() {
    assert_charset_sheet(
        "@charset \"UTF-8\";@charset \"UTF-16LE\";.after{color:blue}",
        &[((0, 0, 0), 17, "charset"), ((17, 0, 17), 37, "charset")],
        &[("after", (37, 0, 37), (50, 54))],
    );
}

#[test]
fn nested_charset_is_unknown_inside_its_retained_parent() {
    let source = "@media all{.before{color:red}@charset \"UTF-8\";.after{color:blue}}";
    let report = parse_sheet(source);
    let [CssRule::Media(media)] = report.syntax().rules() else {
        panic!("one retained media parent: {report:?}")
    };
    assert_position(media.position().unwrap(), (0, 0, 0));
    assert_styles(
        source,
        media.rules(),
        &[
            ("before", (11, 0, 11), (25, 28)),
            ("after", (46, 0, 46), (59, 63)),
        ],
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one unknown nested charset: {report:?}")
    };
    assert_unknown(
        diagnostic,
        ((29, 0, 29), 46, "charset"),
        CssRecoveryAction::DropAtRule,
    );
    assert_validator_parity(source, &report);
}

#[test]
fn validator_rejects_an_exact_charset_prefix_with_the_ordinary_unknown_rule_report() {
    let source = "@charset \"UTF-8\";.after{color:blue}";
    let failure = validate_sheet(source).expect_err("an unrecognized rule is not clean input");
    let [diagnostic] = failure.diagnostics() else {
        panic!("one unknown charset")
    };
    assert_unknown(
        diagnostic,
        ((0, 0, 0), 17, "charset"),
        CssRecoveryAction::DropAtRule,
    );
    assert_validator_parity(source, &parse_sheet(source));
}

fn assert_charset_fragment(source: &str, position: (usize, u32, u32)) {
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one rejected outer rule: {report:?}")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownAtRule);
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_position(diagnostic.error().position(), position);
    assert_position(diagnostic.span().start(), (0, 0, 0));
    assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    let ErrorKind::UnknownAtRule(detail) = diagnostic.error().kind() else {
        panic!("unrecognized charset fragment")
    };
    assert_eq!(detail.name().as_str(), "charset");
}

#[test]
fn exact_one_charset_rule_is_rejected_as_unknown_without_sheet_metadata() {
    assert_charset_fragment("@charset \"UTF-8\";", (0, 0, 0));
}

#[test]
fn commented_charset_fragment_rejects_the_complete_input_with_original_error_position() {
    assert_charset_fragment("/*😀*/\r\n@charset 'UTF-8';", (10, 1, 0));
}

#[test]
fn top_level_cdo_before_between_and_after_rules_is_clean_and_keeps_source_origins() {
    let source = "<!--.a{color:red}<!--.b{color:blue}<!--";
    let report = parse_sheet(source);
    assert_styles(
        source,
        report.syntax().rules(),
        &[("a", (4, 0, 4), (13, 16)), ("b", (21, 0, 21), (30, 34))],
    );
    assert!(report.diagnostics().is_empty(), "{report:?}");
    assert!(report.is_clean());
    assert_validator_parity(source, &report);
}

#[test]
fn top_level_cdc_before_between_and_after_rules_is_clean_and_keeps_source_origins() {
    let source = "-->.a{color:red}-->.b{color:blue}-->";
    let report = parse_sheet(source);
    assert_styles(
        source,
        report.syntax().rules(),
        &[("a", (3, 0, 3), (12, 15)), ("b", (19, 0, 19), (28, 32))],
    );
    assert!(report.diagnostics().is_empty(), "{report:?}");
    assert!(report.is_clean());
    assert_validator_parity(source, &report);
}

#[test]
fn only_top_level_html_comment_tokens_and_trivia_form_a_clean_empty_sheet() {
    let source = " \r\n/**/ <!-- -->\t";
    let report = parse_sheet(source);
    assert!(report.syntax().rules().is_empty());
    assert!(report.diagnostics().is_empty(), "{report:?}");
    assert!(report.is_clean());
    assert_validator_parity(source, &report);
}

#[test]
fn validator_accepts_top_level_html_comment_tokens_with_exact_ordinary_syntax() {
    let source = "<!--.a{color:red}-->";
    let sheet = validate_sheet(source).expect("top-level CDO and CDC are ignored without error");
    assert_styles(source, sheet.rules(), &[("a", (4, 0, 4), (13, 16))]);
    assert_validator_parity(source, &parse_sheet(source));
}

#[test]
fn ignored_html_comment_tokens_do_not_hide_an_unknown_rule_diagnostic() {
    let source = ".before{color:red}<!--@mystery x;-->.after{color:blue}";
    let report = parse_sheet(source);
    assert_styles(
        source,
        report.syntax().rules(),
        &[
            ("before", (0, 0, 0), (14, 17)),
            ("after", (36, 0, 36), (49, 53)),
        ],
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("only the unknown rule is an error: {report:?}")
    };
    assert_unknown(
        diagnostic,
        ((22, 0, 22), 33, "mystery"),
        CssRecoveryAction::DropAtRule,
    );
    assert_validator_parity(source, &report);
}

fn assert_nested_html_token(
    source: &str,
    token: CssTokenKind,
    authored: &str,
    end: usize,
    kept: ExpectedStyle<'_>,
    after: ExpectedStyle<'_>,
) {
    let report = parse_sheet(source);
    let [CssRule::Media(media), after_rule] = report.syntax().rules() else {
        panic!("retained parent and outer sibling: {report:?}")
    };
    assert_styles(source, media.rules(), &[kept]);
    assert_styles(source, std::slice::from_ref(after_rule), &[after]);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid nested selector: {report:?}")
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_position(diagnostic.error().position(), (11, 0, 11));
    assert_position(diagnostic.span().start(), (11, 0, 11));
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    let ErrorKind::InvalidSelector(detail) = diagnostic.error().kind() else {
        panic!("invalid selector")
    };
    let encountered = detail.encountered().expect("responsible nested HTML token");
    assert_eq!(encountered.kind(), token);
    assert_eq!(encountered.authored(), authored);
    assert_validator_parity(source, &report);
}

#[test]
fn nested_cdo_begins_a_discarded_qualified_rule_and_preserves_siblings() {
    assert_nested_html_token(
        "@media all{<!--.bad{color:red}.kept{color:blue}}.after{color:red}",
        CssTokenKind::Cdo,
        "<!--",
        30,
        ("kept", (30, 0, 30), (42, 46)),
        ("after", (48, 0, 48), (61, 64)),
    );
}

#[test]
fn nested_cdc_begins_a_discarded_qualified_rule_and_preserves_siblings() {
    assert_nested_html_token(
        "@media all{-->.bad{color:red}.kept{color:blue}}.after{color:red}",
        CssTokenKind::Cdc,
        "-->",
        29,
        ("kept", (29, 0, 29), (41, 45)),
        ("after", (47, 0, 47), (60, 63)),
    );
}

#[test]
fn ordinary_unknown_at_rule_remains_a_local_error_with_the_same_validator_report() {
    let source = "@mystery x;.after{color:blue}";
    let report = parse_sheet(source);
    assert_styles(
        source,
        report.syntax().rules(),
        &[("after", (11, 0, 11), (24, 28))],
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one unknown at-rule")
    };
    assert_unknown(
        diagnostic,
        ((0, 0, 0), 11, "mystery"),
        CssRecoveryAction::DropAtRule,
    );
    assert_validator_parity(source, &report);
}

fn assert_root_token(source: &str, token: CssTokenKind, authored: &str) {
    let report = parse_sheet(source);
    assert_styles(
        source,
        report.syntax().rules(),
        &[("after", (1, 0, 1), (14, 18))],
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one malformed root token")
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidQualifiedRule
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
    assert_position(diagnostic.error().position(), (0, 0, 0));
    assert_position(diagnostic.span().start(), (0, 0, 0));
    assert_position(diagnostic.span().end(), (1, 0, 1));
    let ErrorKind::InvalidQualifiedRule(detail) = diagnostic.error().kind() else {
        panic!("malformed root qualified rule")
    };
    let encountered = detail.encountered().unwrap();
    assert_eq!(encountered.kind(), token);
    assert_eq!(encountered.authored(), authored);
    assert_validator_parity(source, &report);
}

#[test]
fn root_semicolon_keeps_its_qualified_rule_error_and_next_original_rule() {
    assert_root_token(";.after{color:blue}", CssTokenKind::Semicolon, ";");
}

#[test]
fn unmatched_root_brace_keeps_its_qualified_rule_error_and_next_original_rule() {
    assert_root_token("}.after{color:blue}", CssTokenKind::CloseCurlyBracket, "}");
}

#[test]
fn a_bom_in_a_utf8_rule_string_remains_an_identifier_character() {
    let source = "\u{feff}a{color:red}";
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{report:?}");
    let Some(CssRule::Style(style)) = report.syntax() else {
        panic!("one authored style rule")
    };
    let [selector] = style.selectors().selectors() else {
        panic!("one authored tag selector")
    };
    assert_eq!(selector.selector(), &CssSelector::Tag("\u{feff}a".into()));
    assert_position(style.position(), (0, 0, 0));
    assert_eq!(
        style.declarations()[0]
            .parsed_value()
            .unwrap()
            .source()
            .as_str(),
        source
    );
}

#[test]
fn html_comment_tokens_are_not_stripped_from_an_exact_one_rule_fragment() {
    let source = "<!--.a{color:red}-->";
    let report = parse_rule(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none());
    let [diagnostic] = report.diagnostics() else {
        panic!("one outer rejection")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
    assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidSelector);
    assert_position(diagnostic.error().position(), (0, 0, 0));
    assert_position(diagnostic.span().start(), (0, 0, 0));
    assert_eq!(diagnostic.span().end().byte_offset().value(), 20);
}
