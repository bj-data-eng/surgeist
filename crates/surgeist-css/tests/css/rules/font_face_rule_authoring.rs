//! `@font-face` (CSS Fonts 4) descriptors, sources and numeric ranges through
//! the public stylesheet parser and constructors.

use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValue;
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::numeric::{CssAngle, CssNonNegative, CssNumber, CssNumeric, CssPercentage};
use surgeist_css::rule::CssRule;
use surgeist_css::rule::font_face::{
    CssAuthoredFontFaceDescriptorValue, CssFontDisplay, CssFontFaceDescriptor,
    CssFontFaceDescriptorKind, CssFontFaceDescriptorValue, CssFontFaceDescriptors,
    CssFontFaceFamily, CssFontFaceRule, CssFontFaceSource, CssFontFaceSourceList,
    CssFontFaceUrlSource, CssFontFormat, CssFontFormatHint, CssFontLocalName, CssFontTechHint,
    CssUnicodeRange, CssUnicodeRangeList,
};
use surgeist_css::values::font::{
    CssAbsoluteFontWeight, CssFontFaceObliqueRange, CssFontFaceStyle, CssFontFaceWeight,
    CssFontFaceWidth, CssFontStyleKeyword, CssFontWeightRange, CssFontWidth, CssFontWidthKeyword,
    CssObliqueAngleRange,
};

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn oblique_angle(spelling: &str) -> CssNumeric<CssAngle, CssObliqueAngleRange> {
    CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_component(
        CssComponentValue::try_token(spelling).unwrap(),
    )
    .expect("valid exact oblique angle")
}

fn font_face_rule(rule: &CssRule) -> &CssFontFaceRule {
    match rule {
        CssRule::FontFace(rule) => rule,
        unexpected => panic!("expected font-face rule, got {unexpected:?}"),
    }
}

#[test]
fn font_face_descriptor_collection_preserves_optional_matching_fields() {
    let empty = CssFontFaceDescriptors::new(Vec::new());
    assert!(
        empty
            .effective(CssFontFaceDescriptorKind::FontFamily)
            .is_none()
    );
    assert!(empty.effective(CssFontFaceDescriptorKind::Src).is_none());
    assert_eq!(empty.occurrences().len(), 0);
    assert_eq!(
        CssFontFaceUrlSource::new(
            surgeist_css::values::common::CssUrl::new(""),
            None,
            Vec::new()
        )
        .url()
        .as_str(),
        ""
    );
    assert_eq!(
        CssFontFaceUrlSource::new(
            surgeist_css::values::common::CssUrl::new("   "),
            None,
            Vec::new()
        )
        .url()
        .as_str(),
        "   "
    );

    let family = CssFontFaceFamily::try_new("Avenir Next").unwrap();
    let src = CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Local(
        CssFontLocalName::try_new("Avenir Next").unwrap(),
    )])
    .unwrap();
    let family_record =
        CssFontFaceDescriptor::new(CssFontFaceDescriptorValue::FontFamily(family.clone()).into());
    assert_eq!(family_record.position(), None);
    let source_record =
        CssFontFaceDescriptor::new(CssFontFaceDescriptorValue::Src(src.clone()).into());
    let family_only = CssFontFaceDescriptors::new(vec![family_record.clone()]);
    assert_eq!(face_value!(family_only, FontFamily), &family);
    assert!(
        family_only
            .effective(CssFontFaceDescriptorKind::Src)
            .is_none()
    );
    let source_only = CssFontFaceDescriptors::new(vec![source_record.clone()]);
    assert!(
        source_only
            .effective(CssFontFaceDescriptorKind::FontFamily)
            .is_none()
    );
    assert_eq!(face_value!(source_only, Src), &src);

    let descriptors = CssFontFaceDescriptors::new(vec![
        family_record,
        source_record,
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::FontWeight(CssFontFaceWeight::Range {
                start: CssAbsoluteFontWeight::Number(weight_number("400")),
                end: Some(CssAbsoluteFontWeight::Number(weight_number("700"))),
            })
            .into(),
        ),
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::FontStyle(CssFontFaceStyle::Oblique {
                range: Some(CssFontFaceObliqueRange::new(
                    oblique_angle("-10deg"),
                    Some(oblique_angle("20deg")),
                )),
            })
            .into(),
        ),
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::FontWidth(CssFontFaceWidth::Range {
                start: CssFontWidth::Percentage(
                    CssNumeric::<CssPercentage, CssNonNegative>::try_from_component(
                        CssComponentValue::try_token("75%").unwrap(),
                    )
                    .unwrap(),
                ),
                end: Some(CssFontWidth::Percentage(
                    CssNumeric::<CssPercentage, CssNonNegative>::try_from_component(
                        CssComponentValue::try_token("125%").unwrap(),
                    )
                    .unwrap(),
                )),
            })
            .into(),
        ),
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::FontDisplay(CssFontDisplay::Swap).into(),
        ),
        CssFontFaceDescriptor::new(
            CssFontFaceDescriptorValue::UnicodeRange(
                CssUnicodeRangeList::try_new(vec![CssUnicodeRange::try_new(0, 0x7f).unwrap()])
                    .unwrap(),
            )
            .into(),
        ),
    ]);
    assert_eq!(descriptors.occurrences().len(), 7);
    assert_eq!(face_value!(descriptors, FontFamily), &family);
    assert_eq!(face_value!(descriptors, Src), &src);
    assert_eq!(
        face_value!(descriptors, FontWeight)
            .serialize_specified(Default::default())
            .unwrap(),
        "400 700"
    );
    assert!(matches!(
        face_value!(descriptors, FontStyle),
        CssFontFaceStyle::Oblique { range: Some(_) }
    ));
    assert_eq!(
        face_value!(descriptors, FontWidth)
            .end()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "125%"
    );
    assert_eq!(face_value!(descriptors, FontDisplay), &CssFontDisplay::Swap);
    assert_eq!(
        face_value!(descriptors, UnicodeRange).ranges(),
        &[CssUnicodeRange::try_new(0, 0x7f).unwrap()]
    );
}

#[test]
fn font_face_names_and_urls_preserve_empty_and_whitespace_authored_strings() {
    assert_eq!(
        CssFontFaceFamily::try_new("Avenir Next").unwrap().as_str(),
        "Avenir Next"
    );
    assert_eq!(
        CssFontLocalName::try_new("Avenir Next").unwrap().as_str(),
        "Avenir Next"
    );
    assert_eq!(
        CssFontFaceUrlSource::new(
            surgeist_css::values::common::CssUrl::new("fonts/avenir.woff2"),
            Some(surgeist_css::rule::font_face::CssFontFormat::Keyword(
                CssFontFormatHint::Woff2
            )),
            vec![CssFontTechHint::Variations]
        )
        .url()
        .as_str(),
        "fonts/avenir.woff2"
    );

    for value in ["", " \t\n "] {
        assert_eq!(CssFontFaceFamily::try_new(value).unwrap().as_str(), value);
        assert_eq!(CssFontLocalName::try_new(value).unwrap().as_str(), value);
        assert_eq!(
            CssFontFaceUrlSource::new(
                surgeist_css::values::common::CssUrl::new(value),
                None,
                Vec::new()
            )
            .url()
            .as_str(),
            value
        );
    }
}

#[test]
fn font_face_source_and_unicode_lists_reject_empty_values() {
    assert_eq!(CssFontFaceSourceList::try_new(Vec::new()), None);
    assert_eq!(CssUnicodeRangeList::try_new(Vec::new()), None);

    let url = CssFontFaceUrlSource::new(
        surgeist_css::values::common::CssUrl::new("fonts/avenir.woff2"),
        Some(surgeist_css::rule::font_face::CssFontFormat::Keyword(
            CssFontFormatHint::Woff2,
        )),
        vec![CssFontTechHint::ColorCOLRv1],
    );
    assert_eq!(
        url.format().and_then(CssFontFormat::recognized_format),
        Some(CssFontFormatHint::Woff2)
    );
    assert_eq!(url.tech(), &[CssFontTechHint::ColorCOLRv1]);

    let sources = CssFontFaceSourceList::try_new(vec![CssFontFaceSource::Url(url)]).unwrap();
    assert!(matches!(sources.sources(), [CssFontFaceSource::Url(_)]));

    let ranges =
        CssUnicodeRangeList::try_new(vec![CssUnicodeRange::try_new(0x20, 0x7e).unwrap()]).unwrap();
    assert_eq!(ranges.ranges()[0].start(), 0x20);
    assert_eq!(ranges.ranges()[0].end(), 0x7e);
}

#[test]
fn font_face_numeric_descriptors_enforce_invariants() {
    for valid in ["1", "400", "1000"] {
        assert_eq!(
            weight_number(valid)
                .serialize_specified(Default::default())
                .unwrap(),
            valid
        );
    }
    for invalid in ["0", "1001", "0.999", "1000.001"] {
        assert!(
            CssNumeric::<CssNumber, CssFontWeightRange>::try_from_component(
                CssComponentValue::try_number(invalid).unwrap()
            )
            .is_err()
        );
    }
    assert_eq!(
        CssFontFaceWeight::Range {
            start: CssAbsoluteFontWeight::Number(weight_number("700")),
            end: Some(CssAbsoluteFontWeight::Number(weight_number("400"))),
        }
        .serialize_specified(Default::default())
        .unwrap(),
        "700 400"
    );

    assert!(
        CssNumeric::<CssPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_token("0%").unwrap()
        )
        .is_ok()
    );
    assert!(
        CssNumeric::<CssPercentage, CssNonNegative>::try_from_component(
            CssComponentValue::try_token("-0.1%").unwrap()
        )
        .is_err()
    );
    let descending_width = CssFontFaceWidth::Range {
        start: CssFontWidth::Keyword(CssFontWidthKeyword::Expanded),
        end: Some(CssFontWidth::Keyword(CssFontWidthKeyword::Condensed)),
    };
    assert_eq!(
        descending_width
            .serialize_specified(Default::default())
            .unwrap(),
        "expanded condensed"
    );

    let descending =
        CssFontFaceObliqueRange::new(oblique_angle("90deg"), Some(oblique_angle("-90deg")));
    assert_eq!(
        descending
            .start()
            .serialize_specified(Default::default())
            .unwrap(),
        "90deg"
    );
    assert_eq!(
        descending
            .end()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "-90deg"
    );
    for invalid in ["-90.1deg", "90.1deg"] {
        assert!(
            CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_component(
                CssComponentValue::try_token(invalid).unwrap()
            )
            .is_err()
        );
    }

    assert_eq!(
        CssUnicodeRange::try_new(0x10ffff, 0x10ffff).unwrap().end(),
        0x10ffff
    );
    assert_eq!(CssUnicodeRange::try_new(2, 1), None);
    assert_eq!(CssUnicodeRange::try_new(0, 0x110000), None);
}

#[test]
fn font_face_rule_parser_accepts_descriptor_block() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"@font-face {
            font-family: "Inter";
            src: url("inter.woff2") format("woff2");
            font-weight: 400 700;
            font-style: normal;
            font-display: swap;
            unicode-range: U+0000-00FF, U+0100-017F;
        }"#,
        )
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one font-face rule");
    };

    let descriptors = font_face_rule(rule).descriptors();
    assert_eq!(face_value!(descriptors, FontFamily).as_str(), "Inter");
    let [CssFontFaceSource::Url(source)] = face_value!(descriptors, Src).sources() else {
        panic!("expected one URL font source");
    };
    assert_eq!(source.url().as_str(), "inter.woff2");
    assert_eq!(
        source.format().and_then(CssFontFormat::recognized_format),
        Some(CssFontFormatHint::Woff2)
    );
    assert!(source.tech().is_empty());
    assert_eq!(
        face_value!(descriptors, FontWeight)
            .serialize_specified(Default::default())
            .unwrap(),
        "400 700"
    );
    assert_eq!(
        Some(face_value!(descriptors, FontStyle)),
        Some(&CssFontFaceStyle::Keyword(CssFontStyleKeyword::Normal))
    );
    assert_eq!(
        Some(face_value!(descriptors, FontDisplay)),
        Some(&CssFontDisplay::Swap)
    );
    assert_eq!(
        face_value!(descriptors, UnicodeRange).ranges(),
        &[
            CssUnicodeRange::try_new(0x0000, 0x00ff).unwrap(),
            CssUnicodeRange::try_new(0x0100, 0x017f).unwrap()
        ]
    );
}

#[test]
fn font_face_rule_parser_accepts_source_list_forms() {
    let sheet = CssParser::default().parse_sheet(r#"@font-face {
            font-family: Avenir Next;
            src: local("Inter"), url("inter.woff2") format("woff2"), url("inter-var.woff2") tech(variations);
        }"#)
    .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one font-face rule");
    };

    let descriptors = font_face_rule(rule).descriptors();
    assert_eq!(face_value!(descriptors, FontFamily).as_str(), "Avenir Next");
    let [
        CssFontFaceSource::Local(local),
        CssFontFaceSource::Url(woff2),
        CssFontFaceSource::Url(variable),
    ] = face_value!(descriptors, Src).sources()
    else {
        panic!("expected local source and two URL sources");
    };
    assert_eq!(local.as_str(), "Inter");
    assert_eq!(woff2.url().as_str(), "inter.woff2");
    assert_eq!(
        woff2.format().and_then(CssFontFormat::recognized_format),
        Some(CssFontFormatHint::Woff2)
    );
    assert!(woff2.tech().is_empty());
    assert_eq!(variable.url().as_str(), "inter-var.woff2");
    assert_eq!(
        variable.format().and_then(CssFontFormat::recognized_format),
        None
    );
    assert_eq!(variable.tech(), &[CssFontTechHint::Variations]);
}

#[test]
fn font_face_rule_parser_accepts_strict_numeric_ranges() {
    let sheet = CssParser::default()
        .parse_sheet(
            r#"@font-face {
            font-family: Inter;
            src: url(inter.woff2);
            font-style: oblique -10deg 20deg;
            font-stretch: 75% 125%;
        }"#,
        )
        .expect("clean parse report");
    let [rule] = sheet.rules() else {
        panic!("expected one font-face rule");
    };

    let descriptors = font_face_rule(rule).descriptors();
    let Some(CssFontFaceStyle::Oblique {
        range: Some(oblique),
    }) = Some(face_value!(descriptors, FontStyle))
    else {
        panic!("expected oblique range");
    };
    assert_eq!(
        oblique
            .start()
            .serialize_specified(Default::default())
            .unwrap(),
        "-10deg"
    );
    assert_eq!(
        oblique
            .end()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "20deg"
    );
    assert_eq!(
        face_value!(descriptors, FontWidth)
            .start()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "75%"
    );
    assert_eq!(
        face_value!(descriptors, FontWidth)
            .end()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "125%"
    );
}

#[test]
fn font_face_rule_parser_rejects_invalid_descriptor_blocks() {
    for (css, rejection) in [
        (
            "@font-face { font-family: Inter; src: url(a.woff2); unknown: x; }",
            Rejection::new(CssErrorCode::UnknownDescriptor, Some("unknown"), None),
        ),
        (
            "@font-face { font-family: Inter; src: url(a.woff2); @media screen {} }",
            Rejection::placement("media"),
        ),
        (
            "@font-face { font-family: Inter; src: url(a.woff2); .nested {} }",
            Rejection::new(CssErrorCode::InvalidQualifiedRule, None, Some(".")),
        ),
        (
            "@font-face { font-family: Inter; src: url(a.woff2); font-weight: bolder; }",
            Rejection::new(
                CssErrorCode::InvalidDescriptorValue,
                Some("font-weight"),
                Some("bolder"),
            ),
        ),
        (
            "@font-face { font-family: Inter; src: url(a.woff2) format(woff3); }",
            Rejection::new(
                CssErrorCode::InvalidDescriptorValue,
                Some("src"),
                Some("woff3"),
            ),
        ),
        (
            "@font-face { font-family: Inter; src: url(a.woff2) tech(color-paint); }",
            Rejection::new(
                CssErrorCode::InvalidDescriptorValue,
                Some("src"),
                Some("color-paint"),
            ),
        ),
        (
            r#"@font-face { font-family: Inter; src: url("a.woff2") tech(variations) format(woff2); }"#,
            Rejection::new(
                CssErrorCode::InvalidDescriptorValue,
                Some("src"),
                Some("woff2"),
            ),
        ),
        (
            r#"@font-face { font-family: Inter; src: url("a.woff2") tech(variations color-svg); }"#,
            Rejection::new(
                CssErrorCode::InvalidDescriptorValue,
                Some("src"),
                Some("color-svg"),
            ),
        ),
        (
            "@font-face { font-family: Inter; src: url(a.woff2); unicode-range: U+110000-110001; }",
            Rejection::new(
                CssErrorCode::InvalidDescriptorValue,
                Some("unicode-range"),
                Some("-110001"),
            ),
        ),
        (
            ".panel { src: url(a.woff2); }",
            Rejection::unknown_property("src"),
        ),
    ] {
        assert_rejection(css, rejection);
    }
}
