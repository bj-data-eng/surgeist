//! Bad declaration-value tokens identify their first responsible authored byte.
//!
//! `Error::position`, `CssRecoveryDiagnostic::error`, and `CssTokenSummary::authored`
//! promise original responsible-token provenance. CSS Syntax 3 sections 4.3.5,
//! 4.3.6, and 8.2 reject newline-terminated strings, bad URLs, and unmatched
//! closers. These expectations come from the literal source below; recovery
//! spans still cover the complete discarded declaration, including its semicolon.
//! Pinned grammar: <https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#any-value>.

use std::ops::Range;

use crate::css::support::authored_syntax::assert_position_at as assert_position;
use crate::css::support::failure_classes::ComponentFailure;
use crate::css::support::failure_classes::component_failure;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::CssDeclarationList;
use surgeist_css::diagnostic::{
    CssErrorCode, CssRecoveryAction, CssRecoveryDiagnostic, CssTokenKind,
};
use surgeist_css::property::CssKnownProperty;
use surgeist_css::rule::CssRule;

struct ExpectedToken {
    property: Option<CssKnownProperty>,
    kind: CssTokenKind,
    authored: &'static str,
    position: (usize, u32, u32),
    declaration_span: Range<usize>,
}

fn assert_retained_known(declarations: &CssDeclarationList, expected: &[CssKnownProperty]) {
    let actual = declarations
        .iter()
        .map(|declaration| {
            declaration
                .known()
                .expect("retained known declaration")
                .property()
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn assert_diagnostic(diagnostic: &CssRecoveryDiagnostic, expected: &ExpectedToken) {
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        expected.declaration_span.start,
        "the whole discarded declaration keeps its original start",
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        expected.declaration_span.end,
        "responsible-token correction must not shorten the recovery unit",
    );

    let error = diagnostic.error();
    let token = match (error.code(), expected.property) {
        (CssErrorCode::InvalidPropertyValue, Some(property)) => {
            assert_eq!(error.known_property(), Some(property));
            error.encountered().expect("responsible bad token")
        }
        (CssErrorCode::UnexpectedToken, None) => {
            error.encountered().expect("responsible unexpected token")
        }
        (_, property) => panic!("unexpected declaration error {error:?} for {property:?}"),
    };
    assert_eq!(token.kind(), expected.kind);
    // Check the diagnosed defect before spelling: an end cursor can also make
    // the error's source-spelling recovery inspect the wrong next token.
    assert_position(diagnostic.error().position(), expected.position);
    assert_eq!(token.authored(), expected.authored);
}

fn assert_rejected_attribute(source: &str, expected: ExpectedToken, retained: &[CssKnownProperty]) {
    let report = CssParser::default().parse_block_contents(source);
    assert!(!report.is_clean());
    assert_retained_known(report.syntax(), retained);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one bad declaration must yield one diagnostic: {:?}",
            report.diagnostics()
        );
    };
    let failure = CssParser::default()
        .parse_block_contents(source)
        .into_validation_result()
        .expect_err("recovery is not clean validation");
    assert_eq!(failure.first(), diagnostic);
    assert_eq!(failure.diagnostics(), report.diagnostics());
    assert_diagnostic(diagnostic, &expected);

    // The independently positioned component failure already owns the complete
    // bad token. Both public views must refer to the same literal source bytes;
    // neither parser's output supplies the expected coordinate for the other.
    let component = CssComponentValues::parse(&CssParser::default(), source)
        .expect_err("the bad component is intrinsic");
    let expected_component_kind = match expected.kind {
        CssTokenKind::BadUrl => ComponentFailure::BadUrl,
        CssTokenKind::BadString => ComponentFailure::BadString,
        CssTokenKind::CloseParenthesis | CssTokenKind::CloseSquareBracket => {
            ComponentFailure::UnmatchedClosingDelimiter
        }
        _ => panic!("this regression covers only bad declaration-value tokens"),
    };
    assert_eq!(component_failure(&component), expected_component_kind);
    let CssValueOrigin::Parsed(origin) = component.origin() else {
        panic!("the responsible token retains parsed source provenance");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_position(origin.span().start(), expected.position);
    let end = expected.position.0 + expected.authored.len();
    assert_eq!(origin.span().end().byte_offset().value(), end);
    assert_eq!(&source[expected.position.0..end], expected.authored);
}

fn assert_clean_attribute(source: &str, expected_count: usize) {
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), expected_count);
    assert_eq!(
        CssParser::default()
            .parse_block_contents(source)
            .into_validation_result(),
        Ok(report.syntax().clone())
    );
}

#[test]
fn quoted_url_after_comment_reports_the_bad_url_start_and_preserves_its_spelling() {
    // CSSTree value/Url.json#/error/0, with a separate following declaration.
    assert_clean_attribute("background-image:url('http://test.com');color:red;", 2);
    assert_rejected_attribute(
        "background-image:url(/*test*/'http://test.com'/*test*/);color:red;",
        ExpectedToken {
            property: Some(CssKnownProperty::BackgroundImage),
            kind: CssTokenKind::BadUrl,
            authored: "url(/*test*/'http://test.com'/*test*/)",
            position: (17, 0, 17),
            declaration_span: 0..56,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn unquoted_url_with_interior_whitespace_reports_the_bad_url_start() {
    // CSSTree value/Url.json#/error/2, preserving the complete original value.
    assert_clean_attribute("background-image:url(a-b.png);color:red;", 2);
    assert_rejected_attribute(
        "background-image:url(a - b.png);color:red;",
        ExpectedToken {
            property: Some(CssKnownProperty::BackgroundImage),
            kind: CssTokenKind::BadUrl,
            authored: "url(a - b.png)",
            position: (17, 0, 17),
            declaration_span: 0..32,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn newline_terminated_bad_string_keeps_unicode_and_crlf_source_coordinates() {
    // The first CRLF ends line 0; the second terminates the bad string.
    // On line 1, content: is 8 UTF-16 units and /*🦊*/ is 6: quote column 14.
    assert_clean_attribute("/*😀*/\r\ncontent:/*🦊*/\"bad\"\r\n;color:red;", 2);
    assert_rejected_attribute(
        "/*😀*/\r\ncontent:/*🦊*/\"bad\r\n;color:red;",
        ExpectedToken {
            property: Some(CssKnownProperty::Content),
            kind: CssTokenKind::BadString,
            authored: "\"bad",
            position: (26, 1, 14),
            declaration_span: 10..33,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn nested_custom_bad_url_keeps_unicode_and_crlf_source_coordinates() {
    // Byte prefix 10 + --x: (4) + /*🦊*/ (8) + fn( (3) puts url at 25.
    // The same line has only 4 + 6 + 3 = 13 UTF-16 units before that token.
    assert_clean_attribute("/*😀*/\r\n--x:/*🦊*/fn(url(a-b.png));color:red;", 2);
    assert_rejected_attribute(
        "/*😀*/\r\n--x:/*🦊*/fn(url(a - b.png));color:red;",
        ExpectedToken {
            property: None,
            kind: CssTokenKind::BadUrl,
            authored: "url(a - b.png)",
            position: (25, 1, 13),
            declaration_span: 10..41,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn known_property_var_fallback_rejects_bad_url_before_symbolic_retention() {
    assert_clean_attribute("background-image:var(--image,url(a-b.png));color:red;", 2);
    assert_rejected_attribute(
        "background-image:var(--image,url(a - b.png));color:red;",
        ExpectedToken {
            property: Some(CssKnownProperty::BackgroundImage),
            kind: CssTokenKind::BadUrl,
            authored: "url(a - b.png)",
            position: (29, 0, 29),
            declaration_span: 0..45,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn nested_custom_var_fallback_reports_the_inner_bad_url_token() {
    assert_clean_attribute("--x:var(--fallback,fn(url(a-b.png)));color:red;", 2);
    assert_rejected_attribute(
        "--x:var(--fallback,fn(url(a - b.png)));color:red;",
        ExpectedToken {
            property: None,
            kind: CssTokenKind::BadUrl,
            authored: "url(a - b.png)",
            position: (22, 0, 22),
            declaration_span: 0..39,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn mismatched_square_closer_in_known_function_reports_that_closer() {
    assert_clean_attribute("width:calc(1px + 2px);color:red;", 2);
    assert_rejected_attribute(
        "width:calc(1px + ]);color:red;",
        ExpectedToken {
            property: Some(CssKnownProperty::Width),
            kind: CssTokenKind::CloseSquareBracket,
            authored: "]",
            position: (17, 0, 17),
            declaration_span: 0..20,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn mismatched_parenthesis_in_custom_block_preserves_the_following_declaration() {
    assert_clean_attribute("--x:[()];color:red;", 2);
    assert_rejected_attribute(
        "--x:[)];color:red;",
        ExpectedToken {
            property: None,
            kind: CssTokenKind::CloseParenthesis,
            authored: ")",
            position: (5, 0, 5),
            declaration_span: 0..8,
        },
        &[CssKnownProperty::Color],
    );
}

#[test]
fn corpus_mismatched_custom_closer_reports_its_original_byte_without_wrapper_closure() {
    // CSSTree declaration/custom-property.json#/value should be parsed as balanced Raw/0.
    // This direct declaration source has no outer stylesheet block to retain at EOF.
    assert_clean_attribute("--var: ([])", 1);
    assert_rejected_attribute(
        "--var: ([)]",
        ExpectedToken {
            property: None,
            kind: CssTokenKind::CloseParenthesis,
            authored: ")",
            position: (9, 0, 9),
            declaration_span: 0..11,
        },
        &[],
    );
}

#[test]
fn bad_url_in_style_rule_keeps_the_declaration_span_and_later_rule() {
    let source = ".x{background-image:url(a - b.png);color:red;}.after{width:2px;}";
    let report = CssParser::default().parse_sheet(source);
    assert!(!report.is_clean());
    let [CssRule::Style(first), CssRule::Style(after)] = report.syntax().rules() else {
        panic!("both style rules must survive declaration recovery");
    };
    assert_retained_known(first.declarations(), &[CssKnownProperty::Color]);
    assert_retained_known(after.declarations(), &[CssKnownProperty::Width]);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "only the bad declaration is discarded: {:?}",
            report.diagnostics()
        );
    };
    let failure = CssParser::default()
        .parse_sheet(source)
        .into_validation_result()
        .expect_err("one recovery prevents clean validation");
    assert_eq!(failure.diagnostics(), report.diagnostics());
    assert_diagnostic(
        diagnostic,
        &ExpectedToken {
            property: Some(CssKnownProperty::BackgroundImage),
            kind: CssTokenKind::BadUrl,
            authored: "url(a - b.png)",
            position: (20, 0, 20),
            declaration_span: 3..35,
        },
    );
    let clean_source = ".x{background-image:url(a-b.png);color:red;}.after{width:2px;}";
    let clean = CssParser::default().parse_sheet(clean_source);
    assert!(clean.is_clean(), "{:?}", clean.diagnostics());
    assert_eq!(
        CssParser::default()
            .parse_sheet(clean_source)
            .into_validation_result(),
        Ok(clean.syntax().clone())
    );
}
