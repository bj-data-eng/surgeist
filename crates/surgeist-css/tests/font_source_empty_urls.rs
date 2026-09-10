//! Empty font-source URLs are authored syntax, even though they cannot load a resource.
//! Fonts4 imports the Values4 <url> production:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#url-empty
//! The quoted-string URL syntax also permits whitespace as authored content:
//! https://www.w3.org/TR/2024/WD-css-values-4-20240312/#urls

use surgeist_css::{
    CssFontDisplay, CssFontFaceRule, CssFontFaceSource, CssFontFaceUrlSource, CssFontFormatHint,
    CssFontTechHint, CssParseReport, CssRule, CssSheet, parse_sheet, validate_sheet,
};

fn font_face(report: &CssParseReport<CssSheet>) -> &CssFontFaceRule {
    let [CssRule::FontFace(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("expected retained font-face and following style rule");
    };
    assert_eq!(rule.descriptors().font_family().unwrap().as_str(), "Demo");
    assert_eq!(
        rule.descriptors().font_display().unwrap().value(),
        &CssFontDisplay::Swap
    );
    rule
}

fn assert_clean_urls(value: &str, expected: &[&str]) -> CssParseReport<CssSheet> {
    let source =
        format!("@font-face{{font-family:Demo;src:{value};font-display:swap}}.after{{color:red}}");
    let report = parse_sheet(&source);
    assert!(
        report.is_clean(),
        "valid authored URL sources must not need recovery: {value:?}: {:?}",
        report.diagnostics()
    );
    let actual = font_face(&report)
        .descriptors()
        .src()
        .expect("grammar-valid URL sources must retain the src descriptor")
        .sources()
        .iter()
        .map(|source| match source {
            CssFontFaceSource::Url(url) => url.url(),
            other => panic!("expected URL source, found {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "authored source order for {value:?}");
    assert_eq!(&validate_sheet(&source).unwrap(), report.syntax());
    report
}

#[test]
fn empty_unquoted_font_url_retains_a_clean_source_descriptor() {
    assert_clean_urls("url()", &[""]);
}

#[test]
fn empty_quoted_font_urls_retain_clean_source_descriptors() {
    for value in ["url(\"\")", "url('')"] {
        assert_clean_urls(value, &[""]);
    }
}

#[test]
fn whitespace_font_urls_preserve_the_decoded_authored_value() {
    // Unquoted surrounding whitespace is syntax; quoted whitespace is content.
    assert_clean_urls("url( \t )", &[""]);
    assert_clean_urls("url(\" \t \" )", &[" \t "]);
}

#[test]
fn empty_font_url_members_preserve_the_complete_source_order() {
    assert_clean_urls(
        "url(first.woff2),url(),url(\"\"),url(\"  \"),url(last.woff2)",
        &["first.woff2", "", "", "  ", "last.woff2"],
    );
}

#[test]
fn empty_and_whitespace_font_urls_cross_the_typed_constructor_boundary() {
    for authored in ["", " ", " \t "] {
        let source = CssFontFaceUrlSource::try_new(authored, None, Vec::new())
            .expect("an authored URL does not need to identify a loadable resource");
        assert_eq!(source.url(), authored);
        assert_eq!(source.format(), None);
        assert_eq!(source.formats(), None);
        assert!(source.tech().is_empty());
    }
}

#[test]
fn empty_font_urls_preserve_valid_format_and_technology_hints() {
    let report = assert_clean_urls("url() format(woff2) tech(variations)", &[""]);
    let [CssFontFaceSource::Url(parsed)] =
        font_face(&report).descriptors().src().unwrap().sources()
    else {
        panic!("expected exactly one URL source");
    };
    let constructed = CssFontFaceUrlSource::try_new(
        "",
        Some(CssFontFormatHint::Woff2),
        vec![CssFontTechHint::Variations],
    )
    .expect("format and technology hints do not make an empty authored URL invalid");
    assert_eq!(parsed, &constructed);
    assert_eq!(constructed.format(), Some(&CssFontFormatHint::Woff2));
    assert_eq!(constructed.tech(), &[CssFontTechHint::Variations]);
}

#[test]
fn nonempty_font_urls_remain_authored_values_without_resource_resolution() {
    let urls = ["font.woff2", "relative font.woff2", "about:invalid"];
    assert_clean_urls(
        "url(font.woff2),url(\"relative font.woff2\"),url(about:invalid)",
        &urls,
    );
    for authored in urls {
        let source = CssFontFaceUrlSource::try_new(authored, None, Vec::new()).unwrap();
        assert_eq!(source.url(), authored);
    }
}
