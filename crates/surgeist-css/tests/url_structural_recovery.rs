//! CSS Syntax 3 §4.3.6 consumes unquoted URL payload as one token: brackets,
//! braces, and comment-looking text remain URL code points, while an unescaped
//! closing parenthesis terminates the token. EOF closes that token and any
//! retained enclosing blocks (§5.4.8), not delimiters within its payload.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-url-token

use surgeist_css::{
    CssErrorCode, CssFontFaceSource, CssParseReport, CssRecoveryAction, CssRule, CssSheet,
    parse_sheet, validate_sheet,
};

fn assert_url(source: &str, expected: &str) -> CssParseReport<CssSheet> {
    let report = parse_sheet(source);
    let Some(CssRule::FontFace(face)) = report.syntax().rules().last() else {
        panic!("expected retained font face: {source}: {report:?}");
    };
    let [CssFontFaceSource::Url(url)] = face.descriptors().src().unwrap().sources() else {
        panic!("expected one retained URL source: {source}: {face:?}");
    };
    assert_eq!(url.url(), expected, "decoded URL: {source}");
    report
}

fn assert_clean_url(source: &str, expected: &str) {
    let report = assert_url(source, expected);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert!(validate_sheet(source).is_ok(), "{source}");
}

fn assert_eof_url(source: &str, expected: &str, discarded: usize, closures: usize) {
    let report = assert_url(source, expected);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::DropFontSourceListItem)
            .count(),
        discarded,
        "{source}: {:?}",
        report.diagnostics()
    );
    let retained = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure)
        .collect::<Vec<_>>();
    assert_eq!(
        retained.len(),
        closures,
        "{source}: {:?}",
        report.diagnostics()
    );
    assert_eq!(report.diagnostics().len(), discarded + closures);
    let last_line = source.rsplit('\n').next().unwrap();
    for diagnostic in retained {
        assert_eq!(diagnostic.error().code(), CssErrorCode::UnexpectedEnd);
        for position in [
            diagnostic.error().position(),
            diagnostic.span().start(),
            diagnostic.span().end(),
        ] {
            assert_eq!(position.byte_offset().value(), source.len(), "{source}");
            assert_eq!(
                position.line().value(),
                source.bytes().filter(|byte| *byte == b'\n').count() as u32
            );
            assert_eq!(
                position.column().value(),
                last_line.encode_utf16().count() as u32
            );
        }
    }
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn completed_unquoted_urls_keep_delimiters_and_comment_openers_in_their_payload() {
    for payload in ["a[b", "a{b", "a/*b", "a]b", "a}b", "a;[b", "a,[b"] {
        assert_clean_url(&format!("@font-face{{src:url({payload})}}"), payload);
    }
}

#[test]
fn balanced_delimiters_and_comment_lookalikes_are_ordinary_url_payload() {
    for payload in ["a[b]", "a{b}", "a/*b*/", "a[]{}/*x*/"] {
        assert_clean_url(&format!("@font-face{{src:url({payload})}}"), payload);
    }
}

#[test]
fn eof_urls_close_only_the_url_token_and_font_face_block() {
    for payload in ["a[b", "a{b", "a/*b", "a[b]", "a{b}", "a/*b*/"] {
        assert_eof_url(&format!("@font-face{{src:url({payload}"), payload, 0, 2);
    }
}

#[test]
fn escaped_url_names_and_payload_delimiters_preserve_token_boundaries() {
    for name in [r"u\72l", r"\75rl", r"u\000072 l", "URL"] {
        for (payload, decoded) in [("a[b", "a[b"), (r"a\[b", "a[b"), (r"a\)b", "a)b")] {
            assert_clean_url(&format!("@font-face{{src:{name}({payload})}}"), decoded);
            assert_eof_url(&format!("@font-face{{src:{name}({payload}"), decoded, 0, 2);
        }
    }
}

#[test]
fn escaped_final_parentheses_do_not_terminate_unquoted_url_tokens() {
    for (payload, decoded) in [(r"a\)", "a)"), (r"a\29 ", "a)"), (r"a\", "a\u{fffd}")] {
        assert_eof_url(&format!("@font-face{{src:url({payload}"), decoded, 0, 2);
    }
    assert_clean_url(r"@font-face{src:url(a\\)}", r"a\");
    assert_clean_url(r"@font-face{src:url(a\))}", "a)");
}

#[test]
fn completed_url_payloads_do_not_claim_discarded_sibling_closures() {
    for payload in ["a[b", "a{b", "a/*b"] {
        assert_eof_url(
            &format!(".before{{color:red}}@font-face{{font-family:Demo;src:url({payload}),local("),
            payload,
            1,
            1,
        );
    }
}

#[test]
fn eof_url_closures_preserve_non_bmp_byte_and_utf16_positions() {
    assert_eof_url("/*🦊*/\n@font-face{src:u\\72l(🦊[b", "🦊[b", 0, 2);
}

#[test]
fn url_payload_delimiters_do_not_consume_the_structural_depth_budget() {
    let payload = format!("a{}b", "[{".repeat(300));
    assert_clean_url(&format!("@font-face{{src:url({payload})}}"), &payload);
    assert_eof_url(&format!("@font-face{{src:url({payload}"), &payload, 0, 2);

    // One style block plus 255 functions reaches the public limit. The URL is
    // one atomic token inside the deepest function, not another nested block.
    let source = format!(
        ".x{{--value:{}url(a[b){}}}",
        r"\66 oo(".repeat(255),
        ")".repeat(255),
    );
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(style)] = report.syntax().rules() else {
        panic!("expected retained style: {report:?}");
    };
    assert_eq!(style.declarations().len(), 1);
}
