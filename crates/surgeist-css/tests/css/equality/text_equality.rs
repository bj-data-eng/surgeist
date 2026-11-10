//! Custom-property text and text that awaits substitution is the value, so every raw
//! operand of that kind compares as the declaration path compares it: the authored text
//! between the first and last significant token, whitespace and comments inside included.

use surgeist_css::CssParser;
use surgeist_css::rule::CssSheet;
use surgeist_css::source::same_provenance;

fn sheet(source: &str) -> CssSheet {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

#[test]
fn raw_text_values_differ_where_their_text_differs() {
    for (left, right) in [
        // The declaration path compares these as text.
        ("a{--x:a , b}", "a{--x:a,b}"),
        ("@supports (--x:a , b){}", "@supports (--x:a,b){}"),
        ("@supports (--x:a /**/ b){}", "@supports (--x:a b){}"),
        ("@supports (--x:a  b){}", "@supports (--x:a b){}"),
        (
            "@container style(--x:a , b){}",
            "@container style(--x:a,b){}",
        ),
        (
            "@container style(--x:a  b){}",
            "@container style(--x:a b){}",
        ),
        (
            "@container (min-width:var(--a,1px )){}",
            "@container (min-width:var(--a,1px)){}",
        ),
        (
            "@container (min-width:var( --a,1px)){}",
            "@container (min-width:var(--a,1px)){}",
        ),
    ] {
        assert_ne!(sheet(left), sheet(right), "{left} vs {right}");
    }
}

#[test]
fn raw_text_values_equal_across_the_edge_layout_the_declaration_path_trims() {
    for (left, right) in [
        ("a{--x:a b}", "a{ --x : a b }"),
        ("@supports (--x:a b){}", "@supports ( --x : a b ){}"),
        ("@supports (--x:a b){}", "@supports (--x:/*c*/a b/*d*/){}"),
        (
            "@container style(--x:a b){}",
            "@container style( --x : a b ){}",
        ),
        (
            "@container (min-width:var(--a,1px)){}",
            "@container ( min-width : var(--a,1px) ){}",
        ),
    ] {
        let (a, b) = (sheet(left), sheet(right));
        assert_eq!(a, b, "{left} vs {right}");
        assert!(!same_provenance(&a, &b), "{left} vs {right}");
    }
}

#[test]
fn unknown_media_feature_names_compare_case_insensitively() {
    let (upper, lower) = (sheet("@media (FOO: 1){}"), sheet("@media (foo: 1){}"));
    assert_eq!(upper, lower);
    assert!(!same_provenance(&upper, &lower));
    assert_ne!(sheet("@media (foo: 1){}"), sheet("@media (bar: 1){}"));
    // An extension name is a custom identifier, so its case is data.
    assert_ne!(sheet("@media (--FOO){}"), sheet("@media (--foo){}"));
}

#[test]
fn pending_font_face_descriptor_values_compare_their_text() {
    use surgeist_css::component::CssComponentValues;
    use surgeist_css::rule::font_face::{
        CssAuthoredFontFaceDescriptorValue as Authored, CssFontFaceDescriptorKind as Kind,
    };
    let pending = |text: &str| {
        let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        Authored::try_from_components(Kind::FontWidth, components, Default::default()).unwrap()
    };
    assert_ne!(pending("env(a , b)"), pending("env(a,b)"));
    assert_ne!(pending("env(a ,b)"), pending("env(a,b)"));
    assert_ne!(pending("env(a, b)"), pending("env(a,b)"));
    let tight = pending("env(a,b)");
    assert_eq!(tight, pending("env(a,b)"));
    assert!(same_provenance(&tight, &pending("env(a,b)")));
}
