#[macro_use]
#[path = "support/font_face.rs"]
mod font_face_support;

use surgeist_css::{
    CssAbsoluteFontWeight, CssAuthoredFontFeatureSettings, CssAuthoredFontFeatureValue,
    CssErrorCode, CssFontFaceDescriptorKind, CssFontFaceSource, CssFontFaceWeight,
    CssFontFaceWidth, CssFontFormatHint, CssFontTechHint, CssFontWidth, CssFontWidthKeyword,
    CssRecoveryAction, CssRule, parse_sheet,
};

fn assert_strict_parity(source: &str) {
    {
        let ordinary = parse_sheet(source);
        match surgeist_css::validate_sheet(source) {
            Ok(sheet) => {
                assert!(ordinary.is_clean());
                assert_eq!(&sheet, ordinary.syntax());
            }
            Err(failure) => assert_eq!(failure.diagnostics(), ordinary.diagnostics()),
        }
    }
}

#[test]
fn font_sources_drop_obsolete_multiple_formats_and_preserve_selected_hints() {
    let source = concat!(
        "@font-face { font-family: Demo; src: ",
        "local(Installed Demo), ",
        "url(demo-a.bin) format(\"woff2\", \"opentype\"), ",
        "url(demo-b.bin) format(\"zebra\"), ",
        "url(demo-c.bin) format(woff2) tech(variations, color-colrv1); }",
    );
    let report = parse_sheet(source);

    let [diagnostic] = report.diagnostics() else {
        panic!("expected the Fonts3 multiple-format source to be discarded");
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropFontSourceListItem
    );
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("expected one retained font-face rule");
    };
    let sources = ordinary_face!(rule.descriptors(), Src).unwrap().sources();
    assert_eq!(sources.len(), 3);
    let CssFontFaceSource::Local(local) = &sources[0] else {
        panic!("expected local source");
    };
    assert_eq!(local.as_str(), "Installed Demo");

    let CssFontFaceSource::Url(arbitrary) = &sources[1] else {
        panic!("expected first retained URL source");
    };
    assert_eq!(arbitrary.url().as_str(), "demo-b.bin");
    assert_eq!(string_format!(arbitrary), "zebra");
    assert_eq!(
        arbitrary
            .format()
            .and_then(surgeist_css::CssFontFormat::recognized_format),
        None
    );

    let CssFontFaceSource::Url(keyword) = &sources[2] else {
        panic!("expected second retained URL source");
    };
    assert_eq!(keyword.url().as_str(), "demo-c.bin");
    assert_eq!(
        keyword.format(),
        Some(&surgeist_css::CssFontFormat::Keyword(
            CssFontFormatHint::Woff2
        ))
    );
    assert_eq!(
        keyword
            .format()
            .and_then(surgeist_css::CssFontFormat::recognized_format),
        Some(CssFontFormatHint::Woff2)
    );
    assert_eq!(
        keyword.tech(),
        &[CssFontTechHint::Variations, CssFontTechHint::ColorCOLRv1]
    );
    assert_strict_parity(source);
}

#[test]
fn font_face_family_and_local_names_distinguish_quoted_reserved_names() {
    let valid = concat!(
        "@font-face{font-family:\"serif\";src:",
        "local(\"inherit\"),local(Font Face),url(face.woff)}",
    );
    let report = parse_sheet(valid);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("expected font-face");
    };
    assert_eq!(
        ordinary_face!(rule.descriptors(), FontFamily)
            .unwrap()
            .as_str(),
        "serif"
    );
    let [
        CssFontFaceSource::Local(global),
        CssFontFaceSource::Local(sequence),
        _,
    ] = ordinary_face!(rule.descriptors(), Src).unwrap().sources()
    else {
        panic!("expected two local names and a URL");
    };
    assert_eq!(global.as_str(), "inherit");
    assert_eq!(sequence.as_str(), "Font Face");

    for source in [
        "@font-face{font-family:serif;src:url(face.woff)}.after{color:red}",
        "@font-face{font-family:Demo;src:local(inherit)}.after{color:red}",
        "@font-face{font-family:Demo;src:local(sans-serif)}.after{color:red}",
    ] {
        let report = parse_sheet(source);
        assert!(matches!(
            report.syntax().rules(),
            [CssRule::FontFace(_), CssRule::Style(_)]
        ));
        assert_eq!(
            report
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.action())
                .collect::<Vec<_>>(),
            [CssRecoveryAction::DropDescriptor]
        );
        assert_strict_parity(source);
    }
}

#[test]
fn font_source_lists_retain_fallbacks_beside_empty_members() {
    // Fonts4 parses src members independently; an empty member does not remove
    // a valid fallback: https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
    for source in [
        "@font-face{font-family:Demo;src:,url(a)}",
        "@font-face{font-family:Demo;src:url(a),}",
    ] {
        let report = parse_sheet(source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("expected retained font-face: {source}");
        };
        let [CssFontFaceSource::Url(url)] =
            ordinary_face!(rule.descriptors(), Src).unwrap().sources()
        else {
            panic!("expected retained URL fallback: {source}");
        };
        assert_eq!(url.url().as_str(), "a");
        assert_eq!(
            ordinary_face!(rule.descriptors(), FontFamily)
                .unwrap()
                .as_str(),
            "Demo"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("expected one discarded empty member: {source}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidDescriptorValue
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropFontSourceListItem
        );
        assert_strict_parity(source);
    }
}

#[test]
fn font_source_lists_reject_all_invalid_items_and_invalid_hint_order() {
    let cases = [
        "@font-face{font-family:Demo;src:}",
        "@font-face{font-family:Demo;src:mystery}",
        "@font-face{font-family:Demo;src:local()}",
        "@font-face{font-family:Demo;src:url(a) format()}",
        "@font-face{font-family:Demo;src:url(a) format(woff3)}",
        "@font-face{font-family:Demo;src:url(a) tech(variations) format(\"woff2\")}",
        "@font-face{font-family:Demo;src:url(a) format(\"woff2\") format(\"opentype\")}",
        "@font-face{font-family:Demo;src:url(a) tech(variations) tech(color-colrv1)}",
    ];
    for source in cases {
        let report = parse_sheet(source);
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("invalid src must not discard its accepted outer rule: {source}");
        };
        assert!(ordinary_face!(rule.descriptors(), Src).is_none());
        assert_eq!(
            ordinary_face!(rule.descriptors(), FontFamily)
                .unwrap()
                .as_str(),
            "Demo"
        );
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidDescriptorValue,
            "{source}"
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDescriptor,
            "{source}"
        );
        assert_strict_parity(source);
    }
}

#[test]
fn selected_fonts4_format_and_technology_keywords_remain_ordered() {
    let source = concat!(
        "@font-face{font-family:Demo;src:",
        "url(a) format(woff),url(b) format(woff2),url(c) format(truetype),",
        "url(d) format(opentype),url(e) format(collection),",
        "url(f) format(embedded-opentype),url(g) format(svg),",
        "url(h) tech(variations,color-colrv0,color-colrv1,color-svg,color-sbix,",
        "color-cbdt,features-opentype,features-aat,features-graphite,incremental)}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("expected font-face");
    };
    let expected_formats = [
        CssFontFormatHint::Woff,
        CssFontFormatHint::Woff2,
        CssFontFormatHint::TrueType,
        CssFontFormatHint::OpenType,
        CssFontFormatHint::Collection,
        CssFontFormatHint::EmbeddedOpenType,
        CssFontFormatHint::Svg,
    ];
    for (source, expected) in ordinary_face!(rule.descriptors(), Src).unwrap().sources()[..7]
        .iter()
        .zip(expected_formats)
    {
        let CssFontFaceSource::Url(source) = source else {
            panic!("expected URL source");
        };
        assert_eq!(
            source
                .format()
                .and_then(surgeist_css::CssFontFormat::recognized_format),
            Some(expected)
        );
    }
    let CssFontFaceSource::Url(technology) =
        &ordinary_face!(rule.descriptors(), Src).unwrap().sources()[7]
    else {
        panic!("expected technology URL source");
    };
    assert_eq!(
        technology.tech(),
        &[
            CssFontTechHint::Variations,
            CssFontTechHint::ColorCOLRv0,
            CssFontTechHint::ColorCOLRv1,
            CssFontTechHint::ColorSVG,
            CssFontTechHint::ColorSbix,
            CssFontTechHint::ColorCBDT,
            CssFontTechHint::FeaturesOpenType,
            CssFontTechHint::FeaturesAAT,
            CssFontTechHint::FeaturesGraphite,
            CssFontTechHint::Incremental,
        ]
    );
    assert_strict_parity(source);
}

#[test]
fn font_face_preserves_occurrences_and_uses_last_valid_descriptor() {
    let source = concat!(
        "@font-face { font-family: One; src: url(one.woff2); ",
        "font-weight: normal; font-stretch: condensed; font-style: normal; ",
        "unicode-range: U+0-7F; font-feature-settings: normal; font-display: block; ",
        "font-family: Two; src: url(two.woff2); font-weight: bold; ",
        "font-stretch: expanded; font-style: italic; unicode-range: U+100-17F; ",
        "font-feature-settings: \"kern\" on; font-display: swap; }",
    );
    let report = parse_sheet(source);

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("expected one retained font-face rule");
    };
    assert_eq!(
        ordinary_face!(rule.descriptors(), FontFamily)
            .unwrap()
            .as_str(),
        "Two"
    );
    assert_eq!(
        ordinary_face!(rule.descriptors(), Src).unwrap().sources()[0],
        CssFontFaceSource::Url(surgeist_css::CssFontFaceUrlSource::new(
            surgeist_css::CssUrl::new("two.woff2"),
            None,
            Vec::new()
        ),)
    );
    assert_eq!(
        ordinary_face!(rule.descriptors(), FontWeight).unwrap(),
        &CssFontFaceWeight::Range {
            start: CssAbsoluteFontWeight::Bold,
            end: None
        }
    );
    assert!(matches!(
        ordinary_face!(rule.descriptors(), FontWidth).unwrap(),
        CssFontFaceWidth::Range {
            start: CssFontWidth::Keyword(CssFontWidthKeyword::Expanded),
            end: None
        }
    ));
    assert!(matches!(
        ordinary_face!(rule.descriptors(), FontFeatureSettings).unwrap(),
        CssAuthoredFontFeatureSettings::Features(_)
    ));
    let occurrences = rule.descriptors().occurrences().collect::<Vec<_>>();
    assert_eq!(occurrences.len(), 16);
    assert_eq!(
        occurrences[..8]
            .iter()
            .map(|item| item.value().kind())
            .collect::<Vec<_>>(),
        [
            CssFontFaceDescriptorKind::FontFamily,
            CssFontFaceDescriptorKind::Src,
            CssFontFaceDescriptorKind::FontWeight,
            CssFontFaceDescriptorKind::FontWidth,
            CssFontFaceDescriptorKind::FontStyle,
            CssFontFaceDescriptorKind::UnicodeRange,
            CssFontFaceDescriptorKind::FontFeatureSettings,
            CssFontFaceDescriptorKind::FontDisplay,
        ]
    );
    assert_eq!(
        occurrences[8].value().kind(),
        CssFontFaceDescriptorKind::FontFamily
    );
    assert_eq!(
        occurrences[15].value().kind(),
        CssFontFaceDescriptorKind::FontDisplay
    );
    assert_strict_parity(source);
}

#[test]
fn fonts3_descriptor_values_and_selected_fonts4_ranges_are_typed() {
    for authored in [
        "normal", "bold", "100", "200", "300", "400", "500", "600", "700", "800", "900",
    ] {
        let source = format!("@font-face{{font-family:Demo;src:url(face);font-weight:{authored}}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("expected font-face for {authored}");
        };
        let weight = ordinary_face!(rule.descriptors(), FontWeight).unwrap();
        assert!(matches!(weight, CssFontFaceWeight::Range { end: None, .. }));
        assert_eq!(weight.serialize_specified().unwrap(), authored);
        assert_strict_parity(&source);
    }

    for (authored, expected) in [
        ("ultra-condensed", CssFontWidthKeyword::UltraCondensed),
        ("extra-condensed", CssFontWidthKeyword::ExtraCondensed),
        ("condensed", CssFontWidthKeyword::Condensed),
        ("semi-condensed", CssFontWidthKeyword::SemiCondensed),
        ("normal", CssFontWidthKeyword::Normal),
        ("semi-expanded", CssFontWidthKeyword::SemiExpanded),
        ("expanded", CssFontWidthKeyword::Expanded),
        ("extra-expanded", CssFontWidthKeyword::ExtraExpanded),
        ("ultra-expanded", CssFontWidthKeyword::UltraExpanded),
    ] {
        let source =
            format!("@font-face{{font-family:Demo;src:url(face);font-stretch:{authored}}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{authored}: {:?}", report.diagnostics());
        let [CssRule::FontFace(rule)] = report.syntax().rules() else {
            panic!("expected font-face for {authored}");
        };
        assert_eq!(
            ordinary_face!(rule.descriptors(), FontWidth).unwrap(),
            &CssFontFaceWidth::Range {
                start: CssFontWidth::Keyword(expected),
                end: None
            }
        );
        assert_strict_parity(&source);
    }

    let source = concat!(
        "@font-face{font-family:Demo;src:url(face);font-weight:1 1000;",
        "font-stretch:0% 200%;font-style:oblique -90deg 90deg;",
        "unicode-range:U+0-7F,U+4??;",
        "font-feature-settings:\"kern\",\"liga\" on,\"clig\" off,\"ss01\" 0;",
        "font-display:optional}",
    );
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("expected font-face");
    };
    let features = match ordinary_face!(rule.descriptors(), FontFeatureSettings).unwrap() {
        CssAuthoredFontFeatureSettings::Features(features) => features.features(),
        _ => panic!("expected feature list"),
    };
    assert_eq!(features.len(), 4);
    assert_eq!(features[0].value(), &CssAuthoredFontFeatureValue::Omitted);
    assert_eq!(features[1].value(), &CssAuthoredFontFeatureValue::On);
    assert_eq!(features[2].value(), &CssAuthoredFontFeatureValue::Off);
    assert!(matches!(
        features[3].value(),
        CssAuthoredFontFeatureValue::Index(index) if index.i32_value() == Some(0)
    ));
    assert_eq!(
        ordinary_face!(rule.descriptors(), UnicodeRange)
            .unwrap()
            .ranges()
            .len(),
        2
    );
    assert_strict_parity(source);
}

#[test]
fn invalid_descriptor_occurrences_do_not_erase_valid_neighbors() {
    let source = concat!(
        "@font-face{font-family:One;font-family:serif;font-family:Two;",
        "src:url(one);src:nope;src:url(two);",
        "font-feature-settings:\"kern\" on;",
        "font-feature-settings:inherit;",
        "font-feature-settings:\"liga\" off;unknown:1;",
        "font-display:block!important;font-display:swap}",
        ".after{color:red}",
    );
    let report = parse_sheet(source);
    assert!(matches!(
        report.syntax().rules(),
        [CssRule::FontFace(_), CssRule::Style(_)]
    ));
    let CssRule::FontFace(rule) = &report.syntax().rules()[0] else {
        panic!("expected font-face");
    };
    assert_eq!(
        ordinary_face!(rule.descriptors(), FontFamily)
            .unwrap()
            .as_str(),
        "Two"
    );
    let CssFontFaceSource::Url(effective_source) =
        &ordinary_face!(rule.descriptors(), Src).unwrap().sources()[0]
    else {
        panic!("expected URL source");
    };
    assert_eq!(effective_source.url().as_str(), "two");
    let features = ordinary_face!(rule.descriptors(), FontFeatureSettings).unwrap();
    assert!(matches!(
        features,
        CssAuthoredFontFeatureSettings::Features(list)
            if list.features()[0].tag().as_str() == "liga"
    ));
    assert_eq!(rule.descriptors().occurrences().count(), 7);
    assert_eq!(report.diagnostics().len(), 5);
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|diagnostic| { diagnostic.action() == CssRecoveryAction::DropDescriptor })
    );
    assert_strict_parity(source);
}
