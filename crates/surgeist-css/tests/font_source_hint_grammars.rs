//! Selected Fonts4 grammar for authored font-source hints.
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
//! Its optional format() takes one string or keyword; tech() takes a nonempty
//! comma-separated list that includes palettes. Font loading is a later phase.

use surgeist_css::{
    CssErrorCode, CssFontFaceRule, CssFontFaceSource, CssFontFaceUrlSource, CssFontFormatHint,
    CssFontFormatList, CssFontFormatString, CssParseReport, CssRecoveryAction, CssRule, CssSheet,
    parse_sheet, validate_sheet,
};

fn font_face(report: &CssParseReport<CssSheet>) -> &CssFontFaceRule {
    let [CssRule::FontFace(rule), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("expected retained font-face and following style rule");
    };
    assert_eq!(rule.descriptors().font_family().unwrap().as_str(), "Demo");
    rule
}

fn url_sources(report: &CssParseReport<CssSheet>) -> Vec<&CssFontFaceUrlSource> {
    font_face(report)
        .descriptors()
        .src()
        .expect("expected retained URL sources")
        .sources()
        .iter()
        .map(|source| match source {
            CssFontFaceSource::Url(url) => url,
            _ => panic!("expected a URL source"),
        })
        .collect()
}

fn assert_member_recovery(source: &str, report: &CssParseReport<CssSheet>, member: &str) {
    let [diagnostic] = report.diagnostics() else {
        panic!("expected exactly one discarded hint-bearing source: {source}");
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropFontSourceListItem
    );
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    let start = source.find(member).unwrap();
    assert_eq!(diagnostic.span().start().byte_offset().value(), start);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        start + member.len()
    );
    assert_eq!(
        validate_sheet(source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
}

#[test]
fn multiple_quoted_format_values_drop_only_the_invalid_source() {
    // Fonts3 allowed <string># here. The selected Fonts4 production has no
    // repetition inside format(), so this entire middle source is invalid.
    let invalid = "url(obsolete) format(\"opentype\",\"truetype\")";
    let source = format!(
        "@font-face{{src:url(first),{invalid},url(last);font-family:Demo}}.after{{color:red}}"
    );
    let report = parse_sheet(&source);

    assert_eq!(
        url_sources(&report)
            .iter()
            .map(|url| url.url())
            .collect::<Vec<_>>(),
        ["first", "last"]
    );
    assert_member_recovery(&source, &report, invalid);
}

#[test]
fn font_format_model_requires_exactly_one_authored_string() {
    assert!(CssFontFormatList::try_new(Vec::new()).is_none());
    let single = CssFontFormatList::try_new(vec![CssFontFormatString::try_new("WoFf2").unwrap()])
        .expect("one authored string satisfies the selected format() production");
    assert_eq!(single.formats().len(), 1);
    assert_eq!(single.formats()[0].as_str(), "WoFf2");
    assert!(
        CssFontFormatList::try_new(vec![
            CssFontFormatString::try_new("opentype").unwrap(),
            CssFontFormatString::try_new("truetype").unwrap(),
        ])
        .is_none(),
        "typed construction must reject the same multiple-format state as parsing"
    );
}

#[test]
fn empty_format_string_is_valid_authored_syntax_without_a_recognized_format() {
    let source = concat!(
        "@font-face{src:url(authored) format(\"\");font-family:Demo}",
        ".after{color:red}",
    );
    let report = parse_sheet(source);
    let urls = url_sources(&report);

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [url] = urls.as_slice() else {
        panic!("expected one retained source with an empty authored format string");
    };
    assert_eq!(url.url(), "authored");
    assert_eq!(url.formats().unwrap().formats().len(), 1);
    assert_eq!(url.formats().unwrap().formats()[0].as_str(), "");
    assert_eq!(url.format(), None);
    assert!(url.tech().is_empty());
    assert_eq!(&validate_sheet(source).unwrap(), report.syntax());
}

#[test]
fn empty_authored_format_string_crosses_the_same_typed_constructor_boundary() {
    let empty = CssFontFormatString::try_new("")
        .expect("the selected string grammar does not require a recognized or nonempty format");
    assert_eq!(empty.as_str(), "");
    let single = CssFontFormatList::try_new(vec![empty])
        .expect("one empty string is one format argument, not an absent argument");
    assert_eq!(single.formats().len(), 1);
    assert_eq!(single.formats()[0].as_str(), "");
}

#[test]
fn single_format_hints_preserve_authored_strings_and_known_keyword_meaning() {
    let source = concat!(
        "@font-face{src:url(no-hint),url(keyword) format(WOFF2),",
        "url(quoted) format(\"WoFf2\"),url(unknown) format(\"zebra\");",
        "font-family:Demo}.after{color:red}",
    );
    let report = parse_sheet(source);
    let urls = url_sources(&report);

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(urls.len(), 4);
    assert_eq!(urls[0].format(), None);
    assert_eq!(urls[0].formats(), None);
    for (url, spelling) in [(urls[1], "woff2"), (urls[2], "WoFf2")] {
        assert_eq!(url.formats().unwrap().formats().len(), 1);
        assert_eq!(url.formats().unwrap().formats()[0].as_str(), spelling);
        assert_eq!(url.format(), Some(&CssFontFormatHint::Woff2));
    }
    assert_eq!(urls[3].formats().unwrap().formats()[0].as_str(), "zebra");
    assert_eq!(urls[3].format(), None);
    assert_eq!(&validate_sheet(source).unwrap(), report.syntax());
}

#[test]
fn palettes_technology_is_typed_and_preserves_authored_order() {
    let source = concat!(
        "@font-face{src:url(color-font) format(opentype) ",
        "tech(variations,PaLeTtEs,color-colrv1,palettes);",
        "font-family:Demo}.after{color:red}",
    );
    let report = parse_sheet(source);
    let urls = url_sources(&report);

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [url] = urls.as_slice() else {
        panic!("expected one retained font source");
    };
    assert_eq!(url.format(), Some(&CssFontFormatHint::OpenType));
    // Debug is an existing observable boundary, so RED compiles before the
    // typed Palettes variant is added. Repeated list members remain authored.
    assert_eq!(
        url.tech()
            .iter()
            .map(|tech| format!("{tech:?}"))
            .collect::<Vec<_>>(),
        ["Variations", "Palettes", "ColorCOLRv1", "Palettes"]
    );
    let constructed = CssFontFaceUrlSource::try_new("another-font", None, url.tech().to_vec())
        .expect("the parsed technology values also satisfy typed source construction");
    assert_eq!(constructed.tech(), url.tech());
    assert_eq!(&validate_sheet(source).unwrap(), report.syntax());
}

#[test]
fn malformed_technology_hints_discard_the_source_and_retain_the_fallback() {
    for hint in [
        "tech()",
        "tech(\"palettes\")",
        "tech(palettes variations)",
        "tech(palettes,)",
        "tech(palettes,unknown)",
        "tech(palettes) format(opentype)",
    ] {
        let invalid = format!("url(invalid) {hint}");
        let source = format!(
            "@font-face{{src:{invalid},url(fallback);font-family:Demo}}.after{{color:red}}"
        );
        let report = parse_sheet(&source);

        assert_eq!(
            url_sources(&report)
                .iter()
                .map(|url| url.url())
                .collect::<Vec<_>>(),
            ["fallback"]
        );
        assert_member_recovery(&source, &report, &invalid);
    }
}
