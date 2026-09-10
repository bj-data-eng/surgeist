#![forbid(unsafe_code)]

//! Public authored font-source hints and their intrinsic compatibility meaning.
//!
//! Legacy-string equivalents and the technology grammar follow Fonts4 section 4.3.1:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-parsing
//! Every technology is required by section 4.3.3; TrueType/OpenType synonymy is
//! specified by section 11.2:
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-face-src-formats
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-format-definitions
//! ASCII-insensitive aliases apply the CSS-controlled syntax rule by inference:
//! https://www.w3.org/TR/2011/REC-CSS2-20110607/syndata.html#characters
//! Preserved authored order/equality and first-occurrence projection order are
//! Surgeist contracts. The consumer does not load or select a font resource.

use surgeist_css::{
    CssFontFaceSource, CssFontFaceUrlSource, CssFontFormatHint, CssFontFormatList,
    CssFontFormatString, CssFontTechHint, CssRule, parse_sheet,
};

fn parsed_source(hints: &str) -> CssFontFaceUrlSource {
    let source = format!("@font-face{{src:url(font) {hints}}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{hints}: {:?}", report.diagnostics());
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("expected one font face");
    };
    let occurrence = face.descriptors().src().unwrap();
    assert_eq!(
        occurrence.position().byte_offset().value(),
        source.find("src:").unwrap()
    );
    let [CssFontFaceSource::Url(url)] = occurrence.sources() else {
        panic!("expected one URL source");
    };
    url.clone()
}

fn authored_format(value: &str) -> CssFontFormatList {
    CssFontFormatList::try_new(vec![CssFontFormatString::try_new(value).unwrap()]).unwrap()
}

fn required(source: &CssFontFaceUrlSource) -> Vec<CssFontTechHint> {
    source.required_technologies().collect()
}

fn legacy_compatibility_and_construction() {
    for (authored, expected) in [
        ("woff2-variations", CssFontFormatHint::Woff2),
        ("WoFf-VaRiAtIoNs", CssFontFormatHint::Woff),
        ("truetype-variations", CssFontFormatHint::TrueType),
        ("OPENTYPE-VARIATIONS", CssFontFormatHint::OpenType),
    ] {
        let parsed = parsed_source(&format!("format(\"{authored}\")"));
        let constructed: CssFontFaceUrlSource = CssFontFaceUrlSource::new_with_formats(
            "font",
            Some(authored_format(authored)),
            Vec::new(),
        );
        assert_eq!(constructed, parsed);
        assert_eq!(constructed.format(), Some(&expected));
        assert_eq!(
            constructed.formats().unwrap().formats()[0].as_str(),
            authored
        );
        assert!(constructed.tech().is_empty());
        assert_eq!(required(&constructed), [CssFontTechHint::Variations]);
        // Repeated projection must be observationally pure, including authored Eq.
        let before = constructed.clone();
        assert_eq!(required(&constructed), [CssFontTechHint::Variations]);
        assert_eq!(constructed, before);
    }
    let escaped = parsed_source(r#"format("\77 off2-variations")"#);
    assert_eq!(required(&escaped), [CssFontTechHint::Variations]);
    assert_eq!(
        escaped.formats().unwrap().formats()[0].as_str(),
        "woff2-variations"
    );
    let literal = CssFontFaceUrlSource::new_with_formats(
        "font",
        Some(authored_format(r"\77 off2-variations")),
        Vec::new(),
    );
    assert_eq!(literal.format(), None);
    assert!(required(&literal).is_empty());
    assert_eq!(
        literal.formats().unwrap().formats()[0].as_str(),
        r"\77 off2-variations"
    );
    println!("legacy compatibility and construction: ok");
}

fn conjunctive_technology_requirements() {
    // Explicit requirements all remain effective; implied variations is appended
    // only if no authored occurrence already supplied it. Repetitions do not add
    // capability requirements, but remain visible in the authored tech() list.
    for (hints, authored, expected) in [
        (
            "format(\"woff2-variations\") tech(palettes,palettes,color-colrv1)",
            vec![
                CssFontTechHint::Palettes,
                CssFontTechHint::Palettes,
                CssFontTechHint::ColorCOLRv1,
            ],
            vec![
                CssFontTechHint::Palettes,
                CssFontTechHint::ColorCOLRv1,
                CssFontTechHint::Variations,
            ],
        ),
        (
            "format(\"woff2-variations\") tech(palettes,variations,palettes,variations)",
            vec![
                CssFontTechHint::Palettes,
                CssFontTechHint::Variations,
                CssFontTechHint::Palettes,
                CssFontTechHint::Variations,
            ],
            vec![CssFontTechHint::Palettes, CssFontTechHint::Variations],
        ),
        (
            "format(woff2) tech(incremental,palettes,incremental)",
            vec![
                CssFontTechHint::Incremental,
                CssFontTechHint::Palettes,
                CssFontTechHint::Incremental,
            ],
            vec![CssFontTechHint::Incremental, CssFontTechHint::Palettes],
        ),
        (
            "tech(color-colrv1,variations,color-colrv1)",
            vec![
                CssFontTechHint::ColorCOLRv1,
                CssFontTechHint::Variations,
                CssFontTechHint::ColorCOLRv1,
            ],
            vec![CssFontTechHint::ColorCOLRv1, CssFontTechHint::Variations],
        ),
        (
            "format(\"zebra\") tech(variations,variations,palettes)",
            vec![
                CssFontTechHint::Variations,
                CssFontTechHint::Variations,
                CssFontTechHint::Palettes,
            ],
            vec![CssFontTechHint::Variations, CssFontTechHint::Palettes],
        ),
    ] {
        let source = parsed_source(hints);
        let before = source.clone();
        assert_eq!(required(&source), expected, "{hints}");
        assert_eq!(source.tech(), authored, "{hints}");
        assert_eq!(source, before);
        let reconstructed = CssFontFaceUrlSource::new_with_formats(
            source.url(),
            source.formats().cloned(),
            source.tech().to_vec(),
        );
        assert_eq!(reconstructed, source);
        assert_eq!(required(&reconstructed), expected);
    }
    println!("conjunctive technology requirements: ok");
}

fn absent_and_unknown_format_hints() {
    let absent: CssFontFaceUrlSource = CssFontFaceUrlSource::new_with_formats("", None, Vec::new());
    assert_eq!(absent.formats(), None);
    assert_eq!(absent.format(), None);
    assert!(required(&absent).is_empty());
    for authored in [
        "",
        "zebra",
        "collection-variations",
        "woff2-variations ",
        "woff2-variationſ",
    ] {
        let source = CssFontFaceUrlSource::new_with_formats(
            "",
            Some(authored_format(authored)),
            vec![CssFontTechHint::Palettes, CssFontTechHint::Palettes],
        );
        assert_eq!(source.url(), "");
        assert_eq!(source.formats().unwrap().formats()[0].as_str(), authored);
        assert_eq!(source.format(), None, "{authored}");
        assert_eq!(required(&source), [CssFontTechHint::Palettes]);
        assert_eq!(
            source.tech(),
            [CssFontTechHint::Palettes, CssFontTechHint::Palettes]
        );
        assert_ne!(source, absent);
    }
    // Existing known-keyword construction remains compatible and contributes no
    // technology that the author did not request or a legacy alias did not imply.
    let keyword =
        CssFontFaceUrlSource::try_new("font", Some(CssFontFormatHint::Woff2), Vec::new()).unwrap();
    assert!(required(&keyword).is_empty());
    assert_eq!(keyword, parsed_source("format(woff2)"));
    for ordinary in [
        "woff",
        "woff2",
        "truetype",
        "opentype",
        "collection",
        "embedded-opentype",
        "svg",
    ] {
        // Section 11.2 explicitly rejects treating an OpenType hint as a
        // guarantee of OpenType layout tables; no FeaturesOpenType is implied.
        let source = parsed_source(&format!("format(\"{ordinary}\")"));
        assert!(required(&source).is_empty(), "{ordinary}");
    }
    println!("absent and unknown format hints: ok");
}

fn format_equivalence_and_authored_equality() {
    let formats = [
        CssFontFormatHint::Woff,
        CssFontFormatHint::Woff2,
        CssFontFormatHint::TrueType,
        CssFontFormatHint::OpenType,
        CssFontFormatHint::Collection,
        CssFontFormatHint::EmbeddedOpenType,
        CssFontFormatHint::Svg,
    ];
    for (left_index, left) in formats.into_iter().enumerate() {
        for (right_index, right) in formats.into_iter().enumerate() {
            // Section 11.2 groups precisely the TrueType and OpenType spellings.
            // Other named formats remain separate equivalence classes.
            let expected = left_index == right_index
                || (left_index == 2 && right_index == 3)
                || (left_index == 3 && right_index == 2);
            assert_eq!(
                left.is_equivalent_to(right),
                expected,
                "{left:?}, {right:?}"
            );
        }
    }
    assert_ne!(CssFontFormatHint::TrueType, CssFontFormatHint::OpenType);
    let truetype = parsed_source("format(\"truetype-variations\")");
    let opentype = parsed_source("format(\"opentype-variations\")");
    assert_ne!(truetype, opentype);
    assert!(
        truetype
            .format()
            .unwrap()
            .is_equivalent_to(*opentype.format().unwrap())
    );
    assert_eq!(required(&truetype), required(&opentype));
    println!("format equivalence and authored equality: ok");
}

fn main() {
    legacy_compatibility_and_construction();
    conjunctive_technology_requirements();
    absent_and_unknown_format_hints();
    format_equivalence_and_authored_equality();
}
