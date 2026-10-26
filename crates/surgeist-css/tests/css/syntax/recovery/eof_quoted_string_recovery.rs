//! CSS Syntax 3 §4.3.5 returns a string and reports a parse error at EOF.
//! Retained recovery must not change decoded payloads or consume block depth.

use crate::css::support::authored_syntax::assert_strict_rejection;
use surgeist_css::*;

fn closures<T>(source: &str, report: &CssParseReport<T>, expected: usize) {
    assert_eq!(report.diagnostics().len(), expected, "{source}");
    for diagnostic in report.diagnostics() {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.span().start(), diagnostic.span().end());
    }
}

#[test]
fn descriptor_and_declaration_strings_report_only_actual_eof_termination() {
    for value in [r#""Demo"#, "'Demo", r#""Demo\""#, r#""Demo\\"#] {
        let descriptor = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            value,
            CssFontFaceDescriptorKind::FontFamily,
        );
        assert!(descriptor.syntax().is_some());
        closures(value, &descriptor, 1);
        assert_strict_rejection(
            descriptor.clone().into_validation_result(),
            descriptor.diagnostics(),
        );

        let source = format!("font-family:{value}");
        let declaration = CssParser::default().parse_block_contents(&source);
        assert_eq!(declaration.syntax().len(), 1);
        closures(&source, &declaration, 1);
        assert_strict_rejection(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result(),
            declaration.diagnostics(),
        );
    }
    for value in [r#""Demo""#, "'Demo'", r#""Demo\"""#, r#""Demo\\""#] {
        let report = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            value,
            CssFontFaceDescriptorKind::FontFamily,
        );
        assert!(report.is_clean(), "{value}: {report:?}");
        assert!(report.syntax().is_some());
    }
}

#[test]
fn nested_url_strings_and_unquoted_url_escapes_keep_distinct_closure_counts() {
    for (source, count) in [
        (r#"url("font.woff2"#, 2),
        (r#"url("font.woff2""#, 1),
        ("url(font.woff2", 1),
        (r"url(font.woff2\)", 1),
    ] {
        let report = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            source,
            CssFontFaceDescriptorKind::Src,
        );
        assert!(report.syntax().is_some(), "{source}: {report:?}");
        closures(source, &report, count);
    }
    for source in [r#"url("font.woff2")"#, r"url(font.woff2\))"] {
        let report = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            source,
            CssFontFaceDescriptorKind::Src,
        );
        assert!(report.is_clean(), "{source}: {report:?}");
        assert!(report.syntax().is_some());
    }
}

#[test]
fn an_eof_string_adds_a_diagnostic_without_spending_a_rule_nesting_level() {
    let source = format!("{}.a{{font-family:\"Demo", "@media all{".repeat(255));
    let report = CssParser::default().parse_sheet(&source);
    assert_eq!(report.syntax().rules().len(), 1);
    // 255 media blocks, one style block, and one retained EOF-ended string.
    closures(&source, &report, 257);
    assert_strict_rejection(
        report.clone().into_validation_result(),
        report.diagnostics(),
    );
}
