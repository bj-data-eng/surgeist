#![forbid(unsafe_code)]
use surgeist_css::{
    CssErrorCode, CssRecoveryAction, CssRecoveryDiagnostic, CssRule, parse_page_block, parse_sheet,
};

fn closures(diagnostics: &[CssRecoveryDiagnostic], source: &str, expected: usize) {
    assert_eq!(diagnostics.len(), expected, "{source}: {diagnostics:?}");
    for diagnostic in diagnostics {
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(
            diagnostic.error().position().column().value(),
            u32::try_from(source.encode_utf16().count()).unwrap()
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.len()
        );
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn retained_margin_child_and_page_each_report_their_own_implicit_eof_closure() {
    for (source, expected) in [
        ("@page{@top-left{content:'🦀'", 2),
        ("@page{@top-left{content:'🦀'}", 1),
        ("@page{@top-left{content:'🦀'}}", 0),
    ] {
        let report = parse_sheet(source);
        closures(report.diagnostics(), source, expected);
        let [CssRule::Page(page)] = report.syntax().rules() else {
            panic!("retained Page")
        };
        assert_eq!(page.margin_rules().len(), 1);
        assert_eq!(
            page.margin_rules()[0].declarations().properties()[0]
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            source.find("content").unwrap()
        );
    }
}

#[test]
fn raw_page_body_retains_each_margin_eof_closure_without_fabricated_outer_text() {
    for (source, expected) in [
        ("{@top-left{content:'🦀'", 2),
        ("{@top-left{content:'🦀'}", 1),
        ("{@top-left{content:'🦀'}}", 0),
    ] {
        let report = parse_page_block(source);
        closures(report.diagnostics(), source, expected);
        let body = report.syntax().as_ref().unwrap().body();
        assert_eq!(body.margin_rules().len(), 1);
        assert_eq!(
            body.margin_rules()[0].declarations().properties()[0]
                .position()
                .unwrap()
                .byte_offset()
                .value(),
            source.find("content").unwrap()
        );
    }
}
