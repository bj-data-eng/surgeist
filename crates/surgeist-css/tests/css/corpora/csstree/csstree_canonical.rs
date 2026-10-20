//! Independent structural evidence from upstream CSSTree canonical output.
//!
//! Unlike `csstree/oracle.json`, which is captured from Surgeist's own parser
//! and therefore only characterizes current behavior, the compared text here is
//! CSSTree's `generate` expectation for each upstream-parsed fixture. Surgeist's
//! side is serialized from its public parse result, so a reinterpreted
//! combinator, a dropped declaration or a changed value fails unless the case is
//! listed with a cited reason in `css/support/csstree/canonical_exceptions.rs`.

use crate::css::support;

use support::csstree::{
    CANONICAL_EXCEPTIONS, CanonicalExceptionKind, compare_csstree_canonical_forms,
    normalize_canonical_css, validate_shared_support_surface,
};

#[test]
fn upstream_parsed_cases_match_csstree_canonical_output() {
    assert!(validate_shared_support_surface());
    let summary = compare_csstree_canonical_forms().unwrap_or_else(|failures| {
        panic!(
            "{} canonical comparison failures:\n{}",
            failures.len(),
            failures.join("\n")
        )
    });
    // (context, matching cases, listed exceptions)
    assert_eq!(
        summary.compared(),
        [
            ("atrule", 49, 11),
            ("atrulePrelude", 1, 0),
            ("block", 7, 22),
            ("declaration", 22, 0),
            ("declarationList", 6, 13),
            ("mediaQuery", 41, 2),
            ("rule", 5, 26),
            ("selector", 97, 49),
            ("selectorList", 2, 0),
            ("stylesheet", 14, 34),
            ("value", 33, 18),
        ]
    );
    assert_eq!(
        summary.excluded(),
        [
            ("atrule", "no_retained_syntax", 56),
            ("atrulePrelude", "no_public_projection", 1),
            ("declaration", "no_retained_syntax", 51),
            ("rule", "no_retained_syntax", 2),
            ("selector", "no_public_projection", 16),
            ("selector", "no_retained_syntax", 34),
            ("value", "no_public_projection", 91),
            ("value", "no_retained_syntax", 18),
        ]
    );
    let compared: usize = summary.compared().iter().map(|row| row.1 + row.2).sum();
    let excluded: usize = summary.excluded().iter().map(|row| row.2).sum();
    assert_eq!((compared, excluded, compared + excluded), (452, 269, 721));
    assert_eq!(
        summary.exceptions(CanonicalExceptionKind::Deliberate).len(),
        175
    );
    assert!(
        summary
            .exceptions(CanonicalExceptionKind::Defect)
            .is_empty()
    );
    assert_eq!(CANONICAL_EXCEPTIONS.len(), 175);
}

#[test]
fn normalization_removes_presentation_and_keeps_structure() {
    let same =
        |left: &str, right: &str| normalize_canonical_css(left) == normalize_canonical_css(right);
    // Presentation choices of the two canonical generators are removed.
    assert!(same("[a|='b']", "[a|=\"b\"]"));
    assert!(same("[a=\"b\"i]", "[a=\"b\" i]"));
    assert!(same("s{p:v}", "s { p: v; }"));
    assert!(same("a,b", "a, b"));
    assert!(same("a>b", "a > b"));
    assert!(same("a/*x*/{color:red}", "a{color:red}"));
    assert!(same("(1<2<3)", "(1 < 2 < 3)"));
    assert!(same("color:red!important", "color: red !important;"));
    assert!(same("url(a\\ b.png)", "url(\"a b.png\")"));
    assert!(same("url(/*x*/a)", "url(\"/*x*/a\")"));
    // Structural and value differences remain.
    assert!(!same("a b", "a>b"));
    assert!(!same("a b", "ab"));
    assert!(!same("a :hover", "a:hover"));
    assert!(!same("a .b", "a.b"));
    assert!(!same("a [b]", "a[b]"));
    assert!(!same("a{color:red;width:1px}", "a{color:red}"));
    assert!(!same("color:red", "color:blue"));
    assert!(!same("width:1px", "width:2px"));
    assert!(!same("width:1px", "width:1em"));
    assert!(!same("width:.5px", "width:0.5px"));
    assert!(!same("A", "a"));
    assert!(!same("a{}b{}", "a{}"));
    assert!(!same("@media screen{a{}}", "@media print{a{}}"));
    assert!(!same("\"a b\"", "\"ab\""));
    assert!(!same("[a=b]", "[a=\"b\"]"));
}
