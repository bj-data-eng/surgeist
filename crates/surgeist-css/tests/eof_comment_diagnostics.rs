#![forbid(unsafe_code)]

//! CSS Syntax 3 §4.3.2 consumes comments without emitting a syntax token, but
//! reaching EOF before `*/` is a parse error. Recovery therefore preserves the
//! surrounding grammar while making the report unclean, independently of whether
//! a surrounding rule survives. Token payloads are not comments.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-comments

use surgeist_css::{
    CssCustomMediaBody, CssFontFaceDescriptorKind, CssImportance, CssKnownProperty, CssMediaQuery,
    CssNamespaceContext, CssParseReport, CssPropertyGrammar, CssPropertyNameRef, CssRecoveryAction,
    CssRule, ErrorKind, parse_declaration, parse_font_face_descriptor_value, parse_media_query,
    parse_media_query_list, parse_property_value_text, parse_property_value_text_for_grammar,
    parse_rule, parse_selector, parse_selector_list, parse_sheet, parse_style_attribute,
    parse_style_block, validate_sheet, validate_style_attribute,
};

fn assert_comment<T>(source: &str, report: &CssParseReport<T>, opening: usize) {
    let comments = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            matches!(diagnostic.error().kind(), ErrorKind::UnexpectedEnd(_))
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
        let sheet = parse_sheet(source);
        assert!(sheet.syntax().rules().is_empty());
        assert_comment(source, &sheet, opening);
        assert_eq!(sheet.diagnostics().len(), 1);
        assert_eq!(
            validate_sheet(source).unwrap_err().diagnostics(),
            sheet.diagnostics()
        );

        let declarations = parse_style_attribute(source);
        assert!(declarations.syntax().is_empty());
        assert_comment(source, &declarations, opening);
        assert_eq!(declarations.diagnostics().len(), 1);
        assert_eq!(
            validate_style_attribute(source).unwrap_err().diagnostics(),
            declarations.diagnostics()
        );

        let media = parse_media_query_list(source);
        assert!(media.syntax().queries().is_empty());
        assert_comment(source, &media, opening);
        assert_eq!(media.diagnostics().len(), 1);
    }
}

#[test]
fn trailing_comments_preserve_completed_sheet_and_declaration_units() {
    let source = ".x{color:red}/*";
    let sheet = parse_sheet(source);
    let [CssRule::Style(style)] = sheet.syntax().rules() else {
        panic!("retained style: {sheet:?}");
    };
    assert_eq!(style.declarations().len(), 1);
    assert_comment(source, &sheet, 13);
    assert_eq!(sheet.diagnostics().len(), 1);
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        sheet.diagnostics()
    );

    let source = "color:red;/*";
    let declarations = parse_style_attribute(source);
    assert_eq!(declarations.syntax().len(), 1);
    assert_comment(source, &declarations, 10);
    assert_eq!(declarations.diagnostics().len(), 1);
    assert_eq!(
        validate_style_attribute(source).unwrap_err().diagnostics(),
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
    retained!("color:red/*", parse_declaration("color:red/*"));
    retained!(".x/*", parse_selector(".x/*", &context));
    retained!(".x,.y/*", parse_selector_list(".x,.y/*", &context));
    retained!(".x{}/*", parse_rule(".x{}/*", &context));
    retained!(
        "{color:red}/*",
        parse_style_block("{color:red}/*", &context)
    );
    retained!(
        "Demo/*",
        parse_font_face_descriptor_value("Demo/*", CssFontFaceDescriptorKind::FontFamily)
    );
    retained!(
        "red/*",
        parse_property_value_text(
            "red/*",
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            CssImportance::Normal,
        )
    );
    retained!(
        "red/*",
        parse_property_value_text_for_grammar(
            "red/*",
            CssPropertyGrammar::from_name("color").unwrap(),
            CssImportance::Normal,
        )
    );
}

#[test]
fn valid_media_fragments_keep_their_query_when_a_comment_reaches_eof() {
    let source = "screen/*";
    let query = parse_media_query(source);
    assert!(!matches!(query.syntax(), CssMediaQuery::Never(_)));
    assert_comment(source, &query, 6);
    assert_eq!(query.diagnostics().len(), 1);
    let list = parse_media_query_list(source);
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
        let report = parse_sheet(source);
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
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            validate_sheet(source).unwrap_err().diagnostics(),
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
        let report = parse_sheet(source);
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
    let report = parse_declaration(source);
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
        let report = parse_sheet(&source);
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
    let report = parse_sheet(source);
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
        let report = parse_sheet(source);
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(validate_sheet(source).is_ok());
    }
    let source = format!("/*{}*/.x{{}}", "([{\"".repeat(300));
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().rules().len(), 1);
}

#[test]
fn bad_url_and_bad_string_payloads_do_not_create_comment_diagnostics() {
    for source in ["@font-face{src:url(a b/*)}", ".x{--text:\"/*\n}"] {
        let report = parse_sheet(source);
        assert!(!report.is_clean(), "{source}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|d| !matches!(d.error().kind(), ErrorKind::UnexpectedEnd(_))),
            "payload is not an EOF comment: {source}: {report:?}"
        );
    }
}
