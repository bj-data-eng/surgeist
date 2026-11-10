//! The tokens a media query is read as, following Syntax 3 (2026 CRD) §4.3.
//!
//! - §4.3.7 and §4.3.8: a backslash followed by a newline is not a valid
//!   escape, so it never continues the token before it; it is a delimiter token
//!   and a parse error at the backslash.
//! - §4.3.9: `-` followed by `\` and a newline does not start an ident
//!   sequence, so `-` is a delimiter token, and a media query does not start
//!   with one.
//! - §4.3.1: `$=`, `*=`, `^=`, `|=` and `~=` are not tokens; each is two
//!   delimiter tokens. Media Queries 4 `<general-enclosed>` takes any tokens
//!   between its parentheses, so a pair of delimiters there is valid syntax.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::query::CssConditionKind;
use surgeist_css::query::media::{CssMediaLeaf, CssMediaQuery, CssMediaQueryList};

const MATCHERS: [&str; 5] = ["$=", "*=", "^=", "|=", "~="];

fn codes(report: &surgeist_css::CssParseReport<impl Sized>) -> Vec<CssErrorCode> {
    report
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.error().code())
        .collect()
}

fn is_general_enclosed(query: &CssMediaQuery) -> bool {
    matches!(
        query,
        CssMediaQuery::Condition(condition)
            if matches!(condition.kind(), CssConditionKind::Leaf(CssMediaLeaf::GeneralEnclosed(_)))
    )
}

#[test]
fn a_hyphen_before_a_backslash_newline_is_the_delimiter_a_media_query_cannot_start_with() {
    for newline in ["\n", "\r\n", "\r", "\u{c}"] {
        let source = format!("-\\{newline}");
        let report = CssMediaQueryList::parse(&CssParser::default(), &source);
        assert!(matches!(
            report.syntax().queries(),
            [CssMediaQuery::Never(_)]
        ));
        let rejected = report
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidMediaQuery)
            .expect("the rejected query");
        let token = rejected.error().encountered().expect("the rejected token");
        assert_eq!(format!("{:?}", token.kind()), "Delim", "{source:?}");
        assert_eq!(token.authored(), "-", "{source:?}");
        assert_eq!(rejected.error().position().byte_offset().value(), 0);
    }
}

#[test]
fn a_hyphen_and_a_backslash_newline_in_a_general_enclosed_are_two_delimiters() {
    for source in ["(width: 1-\\\n)", "(-\\\n)"] {
        let report = CssMediaQueryList::parse(&CssParser::default(), source);
        let [query] = report.syntax().queries() else {
            panic!("one query: {source:?}");
        };
        assert!(is_general_enclosed(query), "{source:?}");
        assert_eq!(
            codes(&report),
            [CssErrorCode::EscapeParseError],
            "{source:?}"
        );
    }
}

#[test]
fn a_matcher_in_a_general_enclosed_is_two_delimiters_that_it_takes() {
    for operator in MATCHERS {
        for source in [
            format!("({operator})"),
            format!("(width: {operator})"),
            format!("(width {operator} 1)"),
            format!("not ({operator})"),
        ] {
            let report = CssMediaQueryList::parse(&CssParser::default(), &source);
            assert!(report.diagnostics().is_empty(), "{source:?}");
            assert!(
                !matches!(report.syntax().queries(), [CssMediaQuery::Never(_)]),
                "{source:?}"
            );
        }
        let source = format!("screen and ({operator})");
        let report = CssMediaQueryList::parse(&CssParser::default(), &source);
        assert!(report.diagnostics().is_empty(), "{source:?}");
        assert!(matches!(
            report.syntax().queries(),
            [CssMediaQuery::Typed(_)]
        ));
    }
}

#[test]
fn a_matcher_that_starts_a_media_query_is_rejected_at_its_first_delimiter() {
    for operator in MATCHERS {
        let report = CssMediaQueryList::parse(&CssParser::default(), operator);
        assert!(matches!(
            report.syntax().queries(),
            [CssMediaQuery::Never(_)]
        ));
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected query: {operator:?}");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
        let token = diagnostic
            .error()
            .encountered()
            .expect("the rejected token");
        assert_eq!(format!("{:?}", token.kind()), "Delim", "{operator:?}");
        assert_eq!(token.authored(), &operator[..1], "{operator:?}");
        assert_eq!(diagnostic.error().position().byte_offset().value(), 0);
    }
}

#[test]
fn an_import_keeps_a_media_condition_that_holds_a_matcher() {
    for operator in MATCHERS {
        let source = format!("@import url(x) screen and ({operator});");
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.diagnostics().is_empty(), "{source:?}");
        assert_eq!(report.syntax().rules().len(), 1, "{source:?}");
        let source = format!("@import url(x) {operator};");
        let report = CssParser::default().parse_sheet(&source);
        assert_eq!(report.syntax().rules().len(), 1, "{source:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected query: {source:?}");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
        assert_eq!(diagnostic.error().position().byte_offset().value(), 15);
    }
}

#[test]
fn an_import_with_clauses_keeps_a_media_list_that_holds_a_matcher() {
    for operator in MATCHERS {
        let source =
            format!("@import url(x) layer(a) supports(display: grid) screen and ({operator});");
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.diagnostics().is_empty(), "{source:?}");
        assert_eq!(report.syntax().rules().len(), 1, "{source:?}");
        let source = format!("@import url(x) supports(display: grid) {operator};");
        let report = CssParser::default().parse_sheet(&source);
        assert_eq!(report.syntax().rules().len(), 1, "{source:?}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one rejected query: {source:?}");
        };
        assert_eq!(diagnostic.error().code(), CssErrorCode::InvalidMediaQuery);
        let token = diagnostic
            .error()
            .encountered()
            .expect("the rejected token");
        assert_eq!(token.authored(), &operator[..1], "{source:?}");
        assert_eq!(diagnostic.error().position().byte_offset().value(), 39);
    }
}

#[test]
fn an_import_media_list_after_its_clauses_starts_at_a_delimiter_for_a_hyphen_and_a_backslash_newline()
 {
    let source = "@import url(x) layer(a) supports(display: grid) -\\\n;";
    let report = CssParser::default().parse_sheet(source);
    assert_eq!(report.syntax().rules().len(), 1);
    let rejected = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidMediaQuery)
        .expect("the rejected query");
    let token = rejected.error().encountered().expect("the rejected token");
    assert_eq!(token.authored(), "-");
    assert_eq!(rejected.error().position().byte_offset().value(), 48);
    assert!(
        report
            .diagnostics()
            .iter()
            .any(
                |diagnostic| diagnostic.error().code() == CssErrorCode::EscapeParseError
                    && diagnostic.error().position().byte_offset().value() == 49
            )
    );
}

#[test]
fn a_custom_media_body_reads_a_matcher_and_a_hyphen_before_a_backslash_newline_as_delimiters() {
    for operator in MATCHERS {
        let source = format!("@custom-media --c ({operator});");
        let report = CssParser::default().parse_sheet(&source);
        assert!(report.diagnostics().is_empty(), "{source:?}");
        assert_eq!(report.syntax().rules().len(), 1, "{source:?}");
    }
    let source = "@custom-media --c -\\\n;";
    let report = CssParser::default().parse_sheet(source);
    let rejected = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidMediaQuery)
        .expect("the rejected query");
    let token = rejected.error().encountered().expect("the rejected token");
    assert_eq!(token.authored(), "-");
    assert_eq!(rejected.error().position().byte_offset().value(), 18);
}
