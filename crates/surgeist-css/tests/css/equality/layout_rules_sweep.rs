//! Rule preludes and descriptors compare equal across the whitespace and comments their
//! grammars ignore.

use surgeist_css::CssParser;

#[test]
fn rules_equal_across_whitespace_and_comments() {
    let mut failures = Vec::new();
    for (compact, spaced) in [
        ("@media (1px<width<2px){}", "@media ( 1px < width < 2px ){}"),
        (
            "@media (400px<=width<=700px){}",
            "@media (400px <= width <= 700px){}",
        ),
        (
            "@media (width>=1px) and (height<2px){}",
            "@media ( width >= 1px )  and  ( height < 2px ){}",
        ),
        ("@media screen,print{}", "@media screen , print {}"),
        (
            "@media only screen and (color){}",
            "@media  only  screen  and  ( color ) {}",
        ),
        (
            "@media (aspect-ratio:1/2){}",
            "@media (aspect-ratio: 1 / 2){}",
        ),
        (
            "@media (resolution:2dppx){}",
            "@media ( resolution : 2dppx ){}",
        ),
        ("@media not all{}", "@media  not  all {}"),
        (
            "@media (prefers-color-scheme:dark){}",
            "@media ( prefers-color-scheme : dark ){}",
        ),
        ("@supports selector(a>b){}", "@supports selector( a > b ){}"),
        (
            "@supports font-tech(color-colrv1){}",
            "@supports font-tech( color-colrv1 ){}",
        ),
        (
            "@supports font-format(woff2){}",
            "@supports font-format( woff2 ){}",
        ),
        (
            "@supports at-rule(@media){}",
            "@supports at-rule( @media ){}",
        ),
        ("@supports (a:b){}", "@supports ( a : b ){}"),
        ("@supports (--x:1){}", "@supports ( --x : 1 ){}"),
        (
            "@supports (width:calc(1px + 2px)){}",
            "@supports ( width : calc( 1px + 2px ) ){}",
        ),
        (
            "@supports (not (color:red)){}",
            "@supports ( not ( color : red ) ){}",
        ),
        (
            "@supports (color:red) or (width:1px){}",
            "@supports ( color : red )  or  ( width : 1px ){}",
        ),
        ("@container (width>=1px){}", "@container ( width >= 1px ){}"),
        (
            "@container a,b (width>1px){}",
            "@container a , b ( width > 1px ){}",
        ),
        (
            "@container scroll-state(stuck:top){}",
            "@container scroll-state( stuck : top ){}",
        ),
        ("@container style(--a:b){}", "@container style( --a : b ){}"),
        ("@container (a b){}", "@container ( a  b ){}"),
        (
            "@keyframes k{0%,100%{opacity:0}}",
            "@keyframes k{ 0% , 100% { opacity : 0 } }",
        ),
        (
            "@keyframes k{from{opacity:0}to{opacity:1}}",
            "@keyframes k{ from { opacity:0 } to { opacity:1 } }",
        ),
        ("@layer a,b;", "@layer a , b ;"),
        ("@layer a.b{}", "@layer  a.b  {}"),
        (
            "@counter-style c{symbols:'a' 'b'}",
            "@counter-style c{ symbols : 'a'  'b' }",
        ),
        (
            "@counter-style c{range:1 2}",
            "@counter-style c{ range : 1  2 }",
        ),
        (
            "@counter-style c{speak-as:auto}",
            "@counter-style c{speak-as: auto}",
        ),
        (
            "@counter-style c{pad:1 'a'}",
            "@counter-style c{ pad : 1  'a' }",
        ),
        (
            "@font-face{src:url(a) format('x')}",
            "@font-face{ src : url( a )  format( 'x' ) }",
        ),
        (
            "@font-face{src:local(a),url(b)}",
            "@font-face{ src : local( a ) , url( b ) }",
        ),
        (
            "@font-face{unicode-range:U+0-7F}",
            "@font-face{ unicode-range : U+0-7F }",
        ),
        (
            "@font-face{font-weight:100 200}",
            "@font-face{ font-weight : 100  200 }",
        ),
        ("@font-face{font-family:a}", "@font-face{ font-family : a }"),
        (
            "@font-feature-values a,b{@styleset{x:1 2}}",
            "@font-feature-values a , b { @styleset { x : 1  2 } }",
        ),
        (
            "@color-profile --c{components:a,b}",
            "@color-profile --c{ components : a , b }",
        ),
        ("@page{margin:1px 2px}", "@page{ margin : 1px  2px }"),
        ("@page a,b{margin:1px}", "@page a , b { margin:1px }"),
        (
            "@page{@top-left{content:'a'}}",
            "@page{ @top-left { content : 'a' } }",
        ),
        ("@import url(a) layer;", "@import  url(a)  layer ;"),
        (
            "@import 'a' screen and (color);",
            "@import  'a'  screen  and  ( color ) ;",
        ),
        ("@scope (a) to (b){}", "@scope ( a ) to ( b ){}"),
        (
            "@when media(width>1px) and supports(a:b){}",
            "@when  media( width > 1px )  and  supports( a : b ) {}",
        ),
        (
            "@when media(width>1px){}@else{}",
            "@when media(width>1px) { } @else { }",
        ),
        (
            "@custom-media --m screen,print;",
            "@custom-media --m  screen , print ;",
        ),
        ("@custom-media --m true;", "@custom-media  --m  true ;"),
        ("@namespace a url(b);", "@namespace  a  url( b ) ;"),
        (
            "a{@media (width>1px){color:red}}",
            "a{ @media ( width > 1px ) { color : red } }",
        ),
    ] {
        let (ra, rb) = (
            CssParser::default().parse_sheet(compact),
            CssParser::default().parse_sheet(spaced),
        );
        if !ra.is_clean() || !rb.is_clean() {
            failures.push(format!("not clean: {compact} / {spaced}"));
        } else if ra.syntax() != rb.syntax() {
            failures.push(format!("== false: {compact} vs {spaced}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

// Raw operands keep the whitespace that separates their tokens, and lose only the edge
// whitespace and comments their grammar ignores.
#[test]
fn raw_operands_ignore_edge_layout_and_keep_token_boundaries() {
    let parse = |source: &str| {
        let report = CssParser::default().parse_sheet(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        report.syntax().clone()
    };
    for (compact, spaced) in [
        ("@supports (a b){}", "@supports ( a b ){}"),
        ("@supports (a b){}", "@supports (/*c*/ a b /*d*/){}"),
        ("@supports future( a b ){}", "@supports future(a b){}"),
        ("@media (a b){}", "@media ( a  b ){}"),
        (
            "@media (unknown-feature:1px 2px){}",
            "@media (unknown-feature : 1px  2px){}",
        ),
        ("@container (a b){}", "@container ( a b ){}"),
        ("@when (a b){}", "@when ( a b ){}"),
    ] {
        assert_eq!(parse(compact), parse(spaced), "{compact} vs {spaced}");
    }
    for (left, right) in [
        ("@supports (a b){}", "@supports (ab){}"),
        ("@media (a b){}", "@media (ab){}"),
        ("@container (a b){}", "@container (ab){}"),
        ("@supports future(a b){}", "@supports future(ab){}"),
        (
            "@media (unknown-feature:1px 2px){}",
            "@media (unknown-feature:1px2px){}",
        ),
    ] {
        assert_ne!(parse(left), parse(right), "{left} vs {right}");
    }
    use surgeist_css::selector::{CssNamespaceContext, CssSelector};
    let selector = |source: &str| {
        CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        )
        .syntax()
        .clone()
        .unwrap()
    };
    // The descendant combinator is whitespace, so it is data.
    assert_ne!(selector("a b"), selector("ab"));
    assert_ne!(selector("& :hover"), selector("&:hover"));
}
