//! Typed values compare equal across the whitespace and comments their grammar ignores.

use surgeist_css::CssParser;

fn assert_pairs_equal(pairs: &[(String, String)]) {
    let mut failures = Vec::new();
    for (a, b) in pairs {
        let (ra, rb) = (
            CssParser::default().parse_sheet(a),
            CssParser::default().parse_sheet(b),
        );
        if !ra.is_clean() || !rb.is_clean() {
            failures.push(format!("not clean: {a} / {b}"));
        } else if ra.syntax() != rb.syntax() {
            failures.push(format!("== false: {a} vs {b}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn property_values_equal_across_whitespace_and_comments() {
    let pairs: Vec<(String, String)> = [
        ("width", "calc(1px + 2px)", "calc( 1px + 2px )"),
        ("width", "calc(1px + 2px)", "calc(1px /*c*/ + 2px)"),
        ("width", "min(1px,2px)", "min( 1px , 2px )"),
        ("width", "clamp(1px,2px,3px)", "clamp( 1px, 2px, 3px )"),
        ("margin", "1px 2px", "1px   2px"),
        ("margin", "1px 2px", " 1px 2px "),
        ("margin", "1px 2px", "1px /*c*/ 2px"),
        ("color", "rgb(1 2 3)", "rgb( 1 2 3 )"),
        ("color", "rgb(1,2,3)", "rgb( 1 , 2 , 3 )"),
        ("color", "rgb(1 2 3/50%)", "rgb(1 2 3 / 50%)"),
        (
            "color",
            "color-mix(in srgb,red,blue)",
            "color-mix( in srgb , red , blue )",
        ),
        ("color", "light-dark(red,blue)", "light-dark( red , blue )"),
        ("color", "rgb(from red r g b)", "rgb( from red r g b )"),
        ("background-image", "url(a)", "url( a )"),
        (
            "background-image",
            "linear-gradient(red,blue)",
            "linear-gradient( red , blue )",
        ),
        (
            "background-image",
            "linear-gradient(to right,red 10%,blue)",
            "linear-gradient( to right , red 10% , blue )",
        ),
        (
            "background-image",
            "radial-gradient(circle at 1px 2px,red,blue)",
            "radial-gradient( circle at 1px 2px , red , blue )",
        ),
        ("background-position", "1px 2px", "1px  2px"),
        (
            "box-shadow",
            "1px 2px red,3px 4px blue",
            "1px 2px red , 3px 4px blue",
        ),
        ("transform", "translate(1px,2px)", "translate( 1px , 2px )"),
        (
            "transform",
            "rotate(1deg) scale(2)",
            "rotate( 1deg )  scale( 2 )",
        ),
        ("filter", "blur(1px)", "blur( 1px )"),
        ("clip-path", "inset(1px 2px)", "inset( 1px 2px )"),
        (
            "clip-path",
            "circle(1px at 2px 3px)",
            "circle( 1px at 2px 3px )",
        ),
        ("transition", "opacity 1s,color 2s", "opacity 1s , color 2s"),
        (
            "animation",
            "a 1s ease-in infinite",
            "a  1s  ease-in  infinite",
        ),
        (
            "animation-timing-function",
            "cubic-bezier(.1,.2,.3,.4)",
            "cubic-bezier( .1 , .2 , .3 , .4 )",
        ),
        (
            "animation-timing-function",
            "steps(1,end)",
            "steps( 1 , end )",
        ),
        (
            "grid-template-columns",
            "repeat(2,1px)",
            "repeat( 2 , 1px )",
        ),
        (
            "grid-template-columns",
            "minmax(1px,2fr)",
            "minmax( 1px , 2fr )",
        ),
        ("grid-template-areas", "'a b' 'c d'", "'a b'   'c d'"),
        ("font-family", "a,b", "a , b"),
        ("font-family", "a,'b c'", "a , 'b c'"),
        (
            "font-variation-settings",
            "'wght' 1,'wdth' 2",
            "'wght' 1 , 'wdth' 2",
        ),
        ("counter-reset", "a 1 b 2", "a  1  b  2"),
        ("content", "counter(a,decimal)", "counter( a , decimal )"),
        ("will-change", "a,b", "a , b"),
        ("aspect-ratio", "1/2", "1 / 2"),
        ("aspect-ratio", "auto 1/2", "auto  1 / 2"),
        ("cursor", "url(a) 1 2,auto", "url(a)  1  2 , auto"),
        ("border-radius", "1px 2px/3px 4px", "1px 2px / 3px 4px"),
        ("flex", "1 1 auto", "1  1  auto"),
        ("inset", "auto calc(1px + 2%)", "auto  calc( 1px + 2% )"),
        ("text-shadow", "1px 2px red", "1px  2px  red"),
        ("quotes", "'a' 'b'", "'a'  'b'"),
        ("--x", "1", " 1 "),
        ("transition-property", "a,b", "a , b"),
        ("color-scheme", "light dark", "light  dark"),
        ("container", "a/inline-size", "a / inline-size"),
        ("mask-image", "url(a),none", "url(a) , none"),
        ("font", "1px/2 a,b", "1px / 2 a , b"),
        ("margin", "calc(1px + 2px) 3px", "calc( 1px + 2px )  3px"),
    ]
    .into_iter()
    .map(|(property, compact, spaced)| {
        (
            format!("a{{{property}:{compact}}}"),
            format!("a{{ {property} : {spaced} }}"),
        )
    })
    .collect();
    assert_pairs_equal(&pairs);
}

#[test]
fn selectors_equal_across_whitespace_and_comments() {
    let pairs: Vec<(String, String)> = [
        ("a:is(b,c)", "a:is( b , c )"),
        ("a:not(b)", "a:not( b )"),
        ("a:where(b>c)", "a:where( b > c )"),
        ("a:has(>b)", "a:has( > b )"),
        ("a:nth-child(2n+1)", "a:nth-child( 2n + 1 )"),
        ("a:nth-child(2n+1 of b)", "a:nth-child( 2n + 1  of  b )"),
        ("a:lang(en,fr)", "a:lang( en , fr )"),
        ("a:dir(ltr)", "a:dir( ltr )"),
        ("a[b=c]", "a[ b = c ]"),
        ("a[b='c' i]", "a[ b = 'c'  i ]"),
        ("a::part(b)", "a::part( b )"),
        ("a>b", "a > b"),
        ("a+b", "a + b"),
        ("a~b", "a ~ b"),
        ("a,b", "a , b"),
        ("a b", "a   b"),
        ("a b", "a /*c*/ b"),
    ]
    .into_iter()
    .map(|(compact, spaced)| {
        (
            format!("{compact}{{color:red}}"),
            format!("{spaced}{{color:red}}"),
        )
    })
    .collect();
    assert_pairs_equal(&pairs);
}

// A custom property's value is its authored token sequence, and a value that awaits
// substitution keeps its authored text; both serialize that form, so their text is the value.
#[test]
fn authored_token_text_stays_data_where_it_is_the_value() {
    let parse = |source: &str| CssParser::default().parse_sheet(source).syntax().clone();
    for (left, right) in [
        ("a{--x:a b}", "a{--x:a  b}"),
        ("a{--x:a,b}", "a{--x:a , b}"),
        ("a{width:var(--a)}", "a{width:var( --a )}"),
        ("a{width:env(a,1px)}", "a{width:env( a , 1px )}"),
        ("a{content:attr(a) 'x'}", "a{content:attr( a ) 'x'}"),
    ] {
        assert_ne!(parse(left), parse(right), "{left} vs {right}");
    }
    assert_eq!(parse("a{--x:1}"), parse("a{ --x : 1 }"));
}
