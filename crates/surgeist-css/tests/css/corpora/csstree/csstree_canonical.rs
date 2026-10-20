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
    CANONICAL_EXCEPTIONS, CanonicalExceptionKind, canonically_equal,
    compare_csstree_canonical_forms, validate_shared_support_surface,
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
            ("selector", 118, 44),
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
            ("selector", "no_retained_syntax", 34),
            ("value", "no_public_projection", 91),
            ("value", "no_retained_syntax", 18),
        ]
    );
    let compared: usize = summary.compared().iter().map(|row| row.1 + row.2).sum();
    let excluded: usize = summary.excluded().iter().map(|row| row.2).sum();
    assert_eq!((compared, excluded, compared + excluded), (468, 253, 721));
    assert_eq!(
        summary.exceptions(CanonicalExceptionKind::Deliberate).len(),
        170
    );
    assert!(
        summary
            .exceptions(CanonicalExceptionKind::Defect)
            .is_empty()
    );
    assert_eq!(CANONICAL_EXCEPTIONS.len(), 170);
}

#[test]
fn normalization_removes_presentation_and_keeps_structure() {
    let same = canonically_equal;
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
    // A namespace prefix is not a descendant combinator before `|a`.
    assert!(!same("ns |a", "ns|a"));
    assert!(same("[a |= 'b']", "[a|=\"b\"]"));
    // A signed number token is not a `+` operator between two terms.
    assert!(!same("calc(1px + 2px)", "calc(1px+2px)"));
    assert!(!same("(x+ 1)", "(x+1)"));
    assert!(!same("calc(1px+ 2px)", "calc(1px + 2px)"));
    assert!(!same("calc(1px +2px)", "calc(1px + 2px)"));
    assert!(!same("calc(1px +.5px)", "calc(1px + .5px)"));
    assert!(!same("-/**/.5px", "-.5px"));
    // A comment between whitespace runs is still one optional run.
    assert!(!same("calc(1px + /**/ 2px)", "calc(1px+ 2px)"));
    assert!(!same("calc(1px /**/ + 2px)", "calc(1px +/**/2px)"));
    assert!(same("calc(1px + /**/ 2px)", "calc(1px + 2px)"));
    // Syntax 3 token boundaries: `1-.5px` is `1` then `-.5px`, `x1-.5` is
    // `x1-` then `.5`, and `1-5` is `1` then `-5`.
    assert!(!same("1-.5px", "1-/**/.5px"));
    assert!(same("1-.5px", "1/**/-.5px"));
    assert!(!same("1-.5px", "1/**/-/**/.5px"));
    assert!(same("x1-.5", "x1-/**/.5"));
    assert!(!same("x1-.5", "x1/**/-/**/.5"));
    assert!(same("1-5", "1/**/-5"));
    assert!(!same("1e-5px", "1e/**/-5px"));
    assert!(!same("1e", "1/**/e"));
    assert!(!same("1-px", "1/**/-px"));
    // CDC and CDO are their own tokens, not an ident and delimiters.
    assert!(!same(".-->a", ".-- >a"));
    assert!(!same(".-->a", ".--/**/>a"));
    assert!(!same("<!--a", "< ! --a"));
    // A backslash at end of input is a valid escape.
    assert!(!same("a\\", "a/**/\\"));
    // Syntax 3 preprocessing: CR LF is one newline and NUL is U+FFFD.
    assert!(same("a\r\nb", "a\nb"));
    assert!(!same("\\31\r\nx", "\\31\r x"));
    assert!(!same("a\0b", "a/**/\0/**/b"));
    // A hex escape absorbs one following whitespace.
    assert!(!same("\\31 x", "\\31/**/x"));
    // A bad url never equals a url.
    assert!(!same("url(\"a\" x)", "url(\"a\")"));
    assert!(!same("url(a b)", "url(\"a b\")"));
    // Closed and unclosed blocks stay distinct, whatever the last token holds.
    assert!(!same("(a\\)", "(a\\))"));
    assert!(!same("(/*)", "()"));
    assert!(!same("[a=\"]", "[a=\"]\"]"));
    assert!(!same("a{b:c\\}", "a{b:c\\}}"));
    assert!(!same("url(\"a)", "url(\"a)\")"));
    assert!(!same("f(a", "f(a)"));
    assert!(!same("f(a)", "f (a)"));
    // Unterminated strings and urls stay distinct from terminated ones.
    assert!(!same("\"a", "\"a\""));
    assert!(!same("url(a", "url(a)"));
    // An escaped backslash before the quote still terminates the string.
    assert!(same(r#"a{b:"c\\"}"#, r#"a{b:'c\\'}"#));
    // Backslash parity decides termination: an escaped backslash closes, an
    // escaped quote does not, and a lone quote is never closed.
    assert!(!same(r#""a\\""#, r#""a\\"#));
    assert!(!same(r#""a\""#, r#""a""#));
    assert!(!same("\"", "\"\""));
    assert!(!same(r"url(a\)", r"url(a\))"));
    assert!(same("a+b", "a + b"));
    assert!(same("calc(1px - -.5px)", "calc(1px - -.5px)"));
    // A comment still separates the tokens on either side of it.
    assert!(!same("1px/**/2px", "1px2px"));
    assert!(!same("a/**/b", "ab"));
    // `> =` is two delimiters, not the `>=` comparison.
    assert!(!same("(width > = 1px)", "(width>=1px)"));
    assert!(same("(width >= 1px)", "(width>=1px)"));
}
