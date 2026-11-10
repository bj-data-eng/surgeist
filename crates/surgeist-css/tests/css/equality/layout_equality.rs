//! Whitespace and comments that a grammar gives no meaning are spelling: `==` ignores them
//! and `same_provenance` compares them. Whitespace the grammar distinguishes stays data.

use surgeist_css::CssParser;
use surgeist_css::rule::CssSheet;
use surgeist_css::source::same_provenance;

fn sheet(source: &str) -> CssSheet {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

/// Every pair is equal under `==` and distinct under `same_provenance`.
fn assert_layout_equal(pairs: &[(&str, &str)]) {
    let mut failures = Vec::new();
    for (left, right) in pairs {
        let (a, b) = (sheet(left), sheet(right));
        if a != b {
            failures.push(format!("== false: {left:?} vs {right:?}"));
        } else if same_provenance(&a, &b) {
            failures.push(format!("same_provenance true: {left:?} vs {right:?}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn media_conditions_equal_across_whitespace_and_comments() {
    assert_layout_equal(&[
        ("@media (min-width:1px){}", "@media (min-width: 1px){}"),
        ("@media (min-width:1px){}", "@media ( min-width:1px){}"),
        ("@media (min-width:1px){}", "@media (min-width:1px ){}"),
        ("@media (min-width:1px){}", "@media (min-width:/*c*/1px){}"),
        ("@media (width>=1px){}", "@media (width >= 1px){}"),
        (
            "@media screen and (min-width:1px){}",
            "@media screen  and  (min-width: 1px){}",
        ),
        (
            "@media not (min-width:1px){}",
            "@media not  (min-width: 1px){}",
        ),
        (
            "@media (min-width:1px) and (color){}",
            "@media (min-width: 1px)  and  (color){}",
        ),
        (
            "@media (unknown-feature:1px){}",
            "@media (unknown-feature: 1px){}",
        ),
        ("@media (--narrow){}", "@media ( --narrow ){}"),
    ]);
}

#[test]
fn query_conditions_equal_across_whitespace_and_comments() {
    assert_layout_equal(&[
        ("@supports (color:red){}", "@supports (color: red){}"),
        ("@supports (color:red){}", "@supports (color:red) {}"),
        ("@supports (color:red){}", "@supports ( color:red ){}"),
        (
            "@supports (color:red) and (width:1px){}",
            "@supports (color: red)  and  (width: 1px){}",
        ),
        (
            "@supports not (color:red){}",
            "@supports not  (color: red){}",
        ),
        ("@supports selector(a>b){}", "@supports selector( a > b ){}"),
        (
            "@container (min-width:1px){}",
            "@container (min-width: 1px){}",
        ),
        (
            "@container (min-width:1px){}",
            "@container ( min-width:1px ){}",
        ),
        (
            "@container n (min-width:1px){}",
            "@container n  (min-width: 1px){}",
        ),
        ("@container style(--x:1){}", "@container style( --x: 1 ){}"),
        (
            "@when media(min-width:1px){}",
            "@when media(min-width: 1px){}",
        ),
        (
            "@when media(min-width:1px){}",
            "@when  media( min-width:1px ){}",
        ),
    ]);
}

#[test]
fn rule_bodies_equal_across_whitespace_and_comments() {
    assert_layout_equal(&[
        (
            "@keyframes k{from{opacity:0}}",
            "@keyframes k{from{opacity: 0}}",
        ),
        (
            "@keyframes k{from{opacity:0}}",
            "@keyframes k{ from { opacity : 0 } }",
        ),
        (
            "@counter-style c{system:cyclic}",
            "@counter-style c{system: cyclic}",
        ),
        (
            "@counter-style c{system:cyclic}",
            "@counter-style c{ system : cyclic ; }",
        ),
        (
            "@font-feature-values a{@styleset{x:1}}",
            "@font-feature-values a{@styleset{x: 1}}",
        ),
        (
            "@font-feature-values a{@styleset{x:1}}",
            "@font-feature-values a { @styleset { x : 1 } }",
        ),
        (
            "@font-palette-values --p{font-family:a}",
            "@font-palette-values --p{font-family: a}",
        ),
        (
            "@font-palette-values --p{font-family:a}",
            "@font-palette-values --p { font-family : a }",
        ),
        (
            "@color-profile --c{src:url(a)}",
            "@color-profile --c{src: url(a)}",
        ),
        (
            "@color-profile --c{src:url(a)}",
            "@color-profile --c { src : url(a) }",
        ),
        ("@page{margin:1px}", "@page{margin: 1px}"),
        ("@page :first{margin:1px}", "@page  :first { margin : 1px }"),
        ("a{color:red}", "a { color : red }"),
        ("a{color:red}", "a{/*c*/color:red/*d*/}"),
    ]);
}

#[test]
fn preludes_equal_across_whitespace_and_comments() {
    assert_layout_equal(&[
        (
            "@custom-media --m (min-width:1px);",
            "@custom-media --m (min-width: 1px);",
        ),
        (
            "@custom-media --m (min-width:1px);",
            "@custom-media  --m  ( min-width:1px ) ;",
        ),
        (
            "@import url(a) supports(color:red);",
            "@import url(a) supports(color: red);",
        ),
        ("@import url(a) layer(x);", "@import url(a) layer( x );"),
        (
            "@import url(a) layer(x) supports(color:red) screen;",
            "@import url(a)  layer( x )  supports( color: red )  screen ;",
        ),
        ("@import 'a' screen;", "@import  'a'  screen ;"),
    ]);
}

#[test]
fn whitespace_that_the_grammar_distinguishes_stays_data() {
    use surgeist_css::selector::{CssNamespaceContext, CssSelector};
    let parse = |source: &str| {
        CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        )
        .syntax()
        .clone()
        .unwrap()
    };
    // The descendant combinator is whitespace.
    assert_ne!(parse("a b"), parse("ab"));
    assert_eq!(parse("a b"), parse("a  b"));
    assert_eq!(parse("a > b"), parse("a>b"));
    // A raw general-enclosed operand keeps its token boundaries.
    assert_ne!(sheet("@supports (a b){}"), sheet("@supports (ab){}"));
    assert_ne!(sheet("@media (a b){}"), sheet("@media (ab){}"));
    assert_eq!(sheet("@media (a b){}"), sheet("@media (a  b){}"));
    assert_ne!(
        sheet("@media (min-width:1px){}"),
        sheet("@media (min-width:2px){}")
    );
}

// These preludes keep no origin of their own, so the layout leaves nothing for
// `same_provenance` to separate.
#[test]
fn preludes_without_origins_equal_across_whitespace() {
    for (left, right) in [
        ("@namespace svg url(a);", "@namespace  svg  url(a) ;"),
        ("@scope (a) to (b){}", "@scope  ( a )  to  ( b ) {}"),
        ("@layer a{}", "@layer  a { }"),
        (
            "@font-face{font-display:swap}",
            "@font-face{font-display: swap}",
        ),
    ] {
        assert_eq!(sheet(left), sheet(right), "{left:?} vs {right:?}");
    }
}
