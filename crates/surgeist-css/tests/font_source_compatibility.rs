#![forbid(unsafe_code)]

//! Legacy format strings have the modern meaning listed in Fonts4 section 4.3.1:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
//! ASCII-insensitive alias recognition is inferred for CSS-defined strings from
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html#characters
//! Decoded strings, authored technology order, descriptor positions, and recovery
//! diagnostics are Surgeist contracts. This suite uses the existing public API.

use surgeist_css::{
    CssErrorCode, CssFontFaceSource, CssFontFaceUrlSource, CssFontFormatHint, CssFontTechHint,
    CssParseReport, CssRecoveryAction, CssRule, CssSheet, parse_sheet, validate_sheet,
};

fn url_sources(report: &CssParseReport<CssSheet>) -> Vec<&CssFontFaceUrlSource> {
    let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("expected the font face and following style rule");
    };
    assert_eq!(face.descriptors().font_family().unwrap().as_str(), "Demo");
    face.descriptors()
        .src()
        .expect("expected a retained src descriptor")
        .sources()
        .iter()
        .map(|source| match source {
            CssFontFaceSource::Url(source) => source,
            other => panic!("expected a URL source, received {other:?}"),
        })
        .collect()
}

fn clean_sources(value: &str) -> CssParseReport<CssSheet> {
    let source = format!("@font-face{{font-family:Demo;src:{value}}}.after{{color:red}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
    assert_eq!(&validate_sheet(&source).unwrap(), report.syntax());
    let [CssRule::FontFace(face), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("expected the font face and following style rule");
    };
    assert_eq!(
        face.descriptors()
            .src()
            .unwrap()
            .position()
            .byte_offset()
            .value(),
        source.find("src:").unwrap()
    );
    report
}

#[test]
fn legacy_variation_strings_project_to_their_base_format_without_rewriting_authored_hints() {
    // Each expected base is independently specified by the four compatibility
    // table rows, not inferred from a string suffix or the implementation.
    for (authored, expected) in [
        ("woff2-variations", CssFontFormatHint::Woff2),
        ("woff-variations", CssFontFormatHint::Woff),
        ("truetype-variations", CssFontFormatHint::TrueType),
        ("opentype-variations", CssFontFormatHint::OpenType),
    ] {
        let report = clean_sources(&format!("url(font) format(\"{authored}\")"));
        let urls = url_sources(&report);
        let [url] = urls.as_slice() else {
            panic!("expected exactly one source");
        };
        assert_eq!(url.format(), Some(&expected), "{authored}");
        assert_eq!(url.formats().unwrap().formats()[0].as_str(), authored);
        assert!(
            url.tech().is_empty(),
            "variations was not authored in tech()"
        );
    }
}

#[test]
fn compatibility_matching_preserves_ascii_case_and_decodes_css_string_escapes() {
    // Matching is ASCII-insensitive; decoding precedes semantic recognition.
    // CSS Syntax 3 section 4.3.7 consumes the escaped code point into the string:
    // https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/#consume-string-token
    for (argument, decoded, expected) in [
        (
            r#""WoFf2-VaRiAtIoNs""#,
            "WoFf2-VaRiAtIoNs",
            CssFontFormatHint::Woff2,
        ),
        (
            r#""WOFF-VARIATIONS""#,
            "WOFF-VARIATIONS",
            CssFontFormatHint::Woff,
        ),
        (
            r#""TrUeTyPe-VaRiAtIoNs""#,
            "TrUeTyPe-VaRiAtIoNs",
            CssFontFormatHint::TrueType,
        ),
        (
            r#""OPENTYPE-VARIATIONS""#,
            "OPENTYPE-VARIATIONS",
            CssFontFormatHint::OpenType,
        ),
        (
            r#""\77 off2-variations""#,
            "woff2-variations",
            CssFontFormatHint::Woff2,
        ),
    ] {
        let report = clean_sources(&format!("url(font) format({argument})"));
        let urls = url_sources(&report);
        let [url] = urls.as_slice() else {
            panic!("expected exactly one source");
        };
        assert_eq!(url.format(), Some(&expected), "{argument}");
        assert_eq!(url.formats().unwrap().formats()[0].as_str(), decoded);
        assert!(url.tech().is_empty());
    }
}

#[test]
fn missing_and_unrecognized_format_hints_remain_distinct_authored_states() {
    // The <string> branch is open, but the compatibility table is finite.
    // Section 4.3.3 distinguishes unknown formats from an absent hint:
    // https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-formats
    let report = clean_sources(concat!(
        "url(no-hint),url(empty) format(\"\"),",
        "url(unknown) format(\"zebra\")",
    ));
    let urls = url_sources(&report);
    assert_eq!(urls.len(), 3);
    assert_eq!(urls[0].formats(), None);
    for (url, authored) in [(urls[1], ""), (urls[2], "zebra")] {
        assert_eq!(url.formats().unwrap().formats()[0].as_str(), authored);
    }
    for url in urls {
        assert_eq!(url.format(), None);
        assert!(url.tech().is_empty());
    }
}

#[test]
fn compatibility_does_not_trim_strings_fold_unicode_or_accept_arbitrary_variation_suffixes() {
    for authored in [
        " woff2-variations",
        "woff2-variations ",
        "woff2 -variations",
        "collection-variations",
        "embedded-opentype-variations",
        "svg-variations",
        "zebra-variations",
        "woff2-variationſ",
    ] {
        let report = clean_sources(&format!("url(font) format(\"{authored}\")"));
        let urls = url_sources(&report);
        let [url] = urls.as_slice() else {
            panic!("expected exactly one source");
        };
        assert_eq!(url.format(), None, "{authored}");
        assert_eq!(url.formats().unwrap().formats()[0].as_str(), authored);
        assert!(url.tech().is_empty());
    }
}

#[test]
fn legacy_format_projection_preserves_explicit_technology_order_and_repetition() {
    let report = clean_sources(concat!(
        "url(font) format(\"woff2-variations\") ",
        "tech(palettes,variations,palettes,color-colrv1,variations)",
    ));
    let urls = url_sources(&report);
    let [url] = urls.as_slice() else {
        panic!("expected exactly one source");
    };
    assert_eq!(url.format(), Some(&CssFontFormatHint::Woff2));
    assert_eq!(
        url.formats().unwrap().formats()[0].as_str(),
        "woff2-variations"
    );
    assert_eq!(
        url.tech(),
        [
            CssFontTechHint::Palettes,
            CssFontTechHint::Variations,
            CssFontTechHint::Palettes,
            CssFontTechHint::ColorCOLRv1,
            CssFontTechHint::Variations,
        ]
    );
}

#[test]
fn string_compatibility_does_not_expand_the_keyword_or_hint_function_grammar() {
    // These inputs violate the selected production, even though the quoted
    // legacy string inside other cases is valid. All source members use the
    // section 4.3.1 independent comma-list recovery boundary.
    for invalid in [
        "url(invalid) format(woff2-variations)",
        "url(invalid) format(woff-variations)",
        "url(invalid) format(truetype-variations)",
        "url(invalid) format(opentype-variations)",
        "url(invalid) format(\"woff2-variations\",\"woff2\")",
        "url(invalid) tech(palettes) format(\"woff2-variations\")",
        "url(invalid) format(\"woff2-variations\") tech(palettes) tech(variations)",
        "url(invalid) format(\"woff2-variations\") tech(incremental-auto)",
    ] {
        let source = format!(
            "@font-face{{font-family:Demo;src:url(first),{invalid},url(last)}}.after{{color:red}}"
        );
        let report = parse_sheet(&source);
        assert_eq!(
            url_sources(&report)
                .iter()
                .map(|url| url.url())
                .collect::<Vec<_>>(),
            ["first", "last"],
            "{invalid}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one source-member diagnostic: {invalid}");
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropFontSourceListItem
        );
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        let start = source.find(invalid).unwrap();
        assert_eq!(diagnostic.span().start().byte_offset().value(), start);
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            start + invalid.len()
        );
        assert_eq!(
            validate_sheet(&source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}
