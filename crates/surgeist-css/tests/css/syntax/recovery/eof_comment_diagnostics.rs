//! CSS Syntax 3 §4.3.2 consumes comments without emitting a syntax token, but
//! reaching EOF before `*/` is a parse error. Recovery therefore preserves the
//! surrounding grammar while making the report unclean, independently of whether
//! a surrounding rule survives. Token payloads are not comments.
//! https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#consume-comments

use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::property::{
    CssKnownProperty, CssPropertyGrammar, CssPropertyNameRef, CssPropertyTextTarget,
};
use surgeist_css::query::media::{CssCustomMediaBody, CssMediaQuery, CssMediaQueryList};
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontFaceDescriptorKind,
};
use surgeist_css::rule::{CssRule, CssStyleBlock};
use surgeist_css::selector::{
    CssNamespaceContext, CssParsedStyleSelectors, CssSelector, CssStyleSelectorContext,
};
use surgeist_css::{CssParseReport, CssParser};

fn assert_comment<T>(source: &str, report: &CssParseReport<T>, opening: usize) {
    let comments = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            diagnostic.error().code() == CssErrorCode::UnexpectedEnd
                && diagnostic.span().start().byte_offset().value() == opening
        })
        .collect::<Vec<_>>();
    assert_eq!(comments.len(), 1, "{source:?}: {:?}", report.diagnostics());
    let comment = comments[0];
    assert_eq!(
        comment.error().position().byte_offset().value(),
        source.len()
    );
    assert_eq!(comment.span().end().byte_offset().value(), source.len());
    assert!(!report.is_clean());
}

#[test]
fn unfinished_comment_only_inputs_are_recovered_empty_but_unclean() {
    for source in ["/*", "/*/", "/**", "/*text", "/**//*unfinished"] {
        let opening = if source.starts_with("/**/") { 4 } else { 0 };
        let sheet = CssParser::default().parse_sheet(source);
        assert!(sheet.syntax().rules().is_empty());
        assert_comment(source, &sheet, opening);
        assert_eq!(sheet.diagnostics().len(), 1);
        assert_eq!(
            CssParser::default()
                .parse_sheet(source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            sheet.diagnostics()
        );

        let declarations = CssParser::default().parse_block_contents(source);
        assert!(declarations.syntax().is_empty());
        assert_comment(source, &declarations, opening);
        assert_eq!(declarations.diagnostics().len(), 1);
        assert_eq!(
            CssParser::default()
                .parse_block_contents(source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            declarations.diagnostics()
        );

        let media = CssMediaQueryList::parse(&CssParser::default(), source);
        assert!(media.syntax().queries().is_empty());
        assert_comment(source, &media, opening);
        assert_eq!(media.diagnostics().len(), 1);
    }
}

#[test]
fn trailing_comments_preserve_completed_sheet_and_declaration_units() {
    let source = ".x{color:red}/*";
    let sheet = CssParser::default().parse_sheet(source);
    let [CssRule::Style(style)] = sheet.syntax().rules() else {
        panic!("retained style: {sheet:?}");
    };
    assert_eq!(style.declarations().len(), 1);
    assert_comment(source, &sheet, 13);
    assert_eq!(sheet.diagnostics().len(), 1);
    assert_eq!(
        CssParser::default()
            .parse_sheet(source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        sheet.diagnostics()
    );

    let source = "color:red;/*";
    let declarations = CssParser::default().parse_block_contents(source);
    assert_eq!(declarations.syntax().len(), 1);
    assert_comment(source, &declarations, 10);
    assert_eq!(declarations.diagnostics().len(), 1);
    assert_eq!(
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result()
            .unwrap_err()
            .diagnostics(),
        declarations.diagnostics()
    );
}

#[test]
fn successful_singular_fragments_preserve_syntax_and_report_comment_eof() {
    let context = CssNamespaceContext::default();
    macro_rules! retained {
        ($source:literal, $parse:expr) => {{
            let report = $parse;
            assert!(report.syntax().is_some(), "{}: {:?}", $source, report);
            assert_comment($source, &report, $source.len() - 2);
            assert_eq!(report.diagnostics().len(), 1);
        }};
    }
    retained!(
        "color:red/*",
        CssDeclaration::parse(&CssParser::default(), "color:red/*")
    );
    retained!(
        ".x/*",
        CssSelector::parse(&CssParser::default(), ".x/*", &context)
    );
    retained!(
        ".x,.y/*",
        CssParsedStyleSelectors::parse(
            &CssParser::default(),
            ".x,.y/*",
            &context,
            CssStyleSelectorContext::Ordinary
        )
    );
    retained!(
        ".x{}/*",
        CssRule::parse(&CssParser::default(), ".x{}/*", &context)
    );
    retained!(
        "{color:red}/*",
        CssStyleBlock::parse(&CssParser::default(), "{color:red}/*", &context)
    );
    retained!(
        "Demo/*",
        CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            "Demo/*",
            CssFontFaceDescriptorKind::FontFamily
        )
    );
    retained!(
        "red/*",
        CssParser::default().parse_property_value(
            "red/*",
            CssPropertyTextTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Color)),
            CssImportance::Normal
        )
    );
    retained!(
        "red/*",
        CssParser::default().parse_property_value(
            "red/*",
            CssPropertyTextTarget::Grammar(CssPropertyGrammar::from_name("color").unwrap()),
            CssImportance::Normal
        )
    );
}

#[test]
fn valid_media_fragments_keep_their_query_when_a_comment_reaches_eof() {
    let source = "screen/*";
    let query = CssMediaQuery::parse(&CssParser::default(), source);
    assert!(!matches!(query.syntax(), CssMediaQuery::Never(_)));
    assert_comment(source, &query, 6);
    assert_eq!(query.diagnostics().len(), 1);
    let list = CssMediaQueryList::parse(&CssParser::default(), source);
    assert_eq!(list.syntax().queries().len(), 1);
    assert!(!matches!(
        list.syntax().queries()[0],
        CssMediaQuery::Never(_)
    ));
    assert_comment(source, &list, 6);
    assert_eq!(list.diagnostics().len(), 1);
}

#[test]
fn custom_media_boolean_and_empty_query_definitions_survive_comment_eof() {
    for (source, boolean) in [
        ("@custom-media --x true/*", true),
        ("@custom-media --x/*", false),
    ] {
        let report = CssParser::default().parse_sheet(source);
        let [CssRule::CustomMedia(definition)] = report.syntax().rules() else {
            panic!("retained definition: {report:?}");
        };
        if boolean {
            assert!(matches!(definition.body(), CssCustomMediaBody::True));
        } else {
            let CssCustomMediaBody::Media(queries) = definition.body() else {
                panic!("empty authored query list");
            };
            assert!(queries.queries().is_empty());
        }
        assert_comment(source, &report, source.len() - 2);
        assert_eq!(report.diagnostics().len(), 2);
        let statement = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainNonconformingRule)
            .expect("distinct statement EOF diagnostic");
        let detail = statement.error();
        assert_eq!(
            detail.code(),
            CssErrorCode::UnexpectedEnd,
            "typed statement EOF"
        );
        assert_eq!(
            detail.expectation().unwrap().as_str(),
            "a semicolon or block terminating an at-rule"
        );
        assert_eq!(
            statement.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(statement.span().start().byte_offset().value(), 0);
        assert_eq!(statement.span().end().byte_offset().value(), source.len());
        assert!(
            !report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action()
                    == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert_eq!(
            CssParser::default()
                .parse_sheet(source)
                .into_validation_result()
                .unwrap_err()
                .diagnostics(),
            report.diagnostics()
        );
    }
}

#[test]
fn discarded_rules_and_rejected_fragments_do_not_suppress_or_duplicate_comments() {
    for source in [
        "@unknown x;/*",
        "@unknown fn(/*",
        "@custom-media ordinary true/*",
    ] {
        let report = CssParser::default().parse_sheet(source);
        assert!(report.syntax().rules().is_empty());
        assert_comment(source, &report, source.len() - 2);
        assert_eq!(report.diagnostics().len(), 2, "{source}: {report:?}");
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|d| d.action() == CssRecoveryAction::DropAtRule)
                .count(),
            1
        );
    }
    let source = "color:???/*";
    let report = CssDeclaration::parse(&CssParser::default(), source);
    assert!(report.syntax().is_none());
    assert_comment(source, &report, source.len() - 2);
    assert_eq!(report.diagnostics().len(), 2);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|d| d.action() == CssRecoveryAction::RejectInput)
            .count(),
        1
    );
}

#[test]
fn comment_error_composes_once_with_retained_structural_eof_closures() {
    for depth in [1, 65, 129] {
        let source = format!("{} .x{{color:red}}/*", "@layer x{".repeat(depth));
        let report = CssParser::default().parse_sheet(&source);
        assert_eq!(report.syntax().rules().len(), 1);
        assert_comment(&source, &report, source.len() - 2);
        assert_eq!(report.diagnostics().len(), depth + 1, "{report:?}");
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .filter(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
                .count(),
            depth
        );
    }
}

#[test]
fn eof_comment_coordinates_use_original_bytes_and_utf16_after_crlf() {
    let source = "/*🦊*/\r\n.x{}/*🦊";
    let report = CssParser::default().parse_sheet(source);
    assert_comment(source, &report, 14);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.span().start().line().value(), 1);
    assert_eq!(diagnostic.span().start().column().value(), 4);
    for position in [diagnostic.error().position(), diagnostic.span().end()] {
        assert_eq!(position.byte_offset().value(), 20);
        assert_eq!(position.line().value(), 1);
        assert_eq!(position.column().value(), 8);
    }
}

#[test]
fn terminated_comments_and_comment_lookalikes_in_token_payloads_stay_clean() {
    for source in [
        "/**/",
        "/***/",
        "/*\\*/",
        "/*/*/",
        ".x{--text:\"/*\"}",
        "@font-face{src:url(a/*b)}",
    ] {
        let report = CssParser::default().parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert_eq!(
            CssParser::default()
                .parse_sheet(source)
                .into_validation_result()
                .unwrap(),
            *report.syntax()
        );
    }
    let source = format!("/*{}*/.x{{}}", "([{\"".repeat(300));
    let report = CssParser::default().parse_sheet(&source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().rules().len(), 1);
}

#[test]
fn bad_url_and_bad_string_payloads_do_not_create_comment_diagnostics() {
    for source in ["@font-face{src:url(a b/*)}", ".x{--text:\"/*\n}"] {
        let report = CssParser::default().parse_sheet(source);
        assert!(!report.is_clean(), "{source}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| d.error().code() != CssErrorCode::UnexpectedEnd),
            "payload is not an EOF comment: {source}: {report:?}"
        );
    }
}
