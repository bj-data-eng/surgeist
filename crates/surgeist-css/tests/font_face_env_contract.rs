#![forbid(unsafe_code)]

//! Public authored `@font-face` environment-value contracts from CSS Env 1 §3.

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssAuthoredFontFeatureSettings, CssFontDisplay,
    CssFontFaceDescriptor, CssFontFaceDescriptorKind as Kind,
    CssFontFaceDescriptorValue as Ordinary, CssFontFaceDescriptors, CssFontFaceSource,
    CssFontFaceStyle, CssFontFaceWeight, CssFontFaceWidth, CssFontWidth, CssFontWidthKeyword,
    CssNormalizedItem, CssRecoveryAction, CssRule, CssRuleContextKindRef, CssValueOrigin,
    normalize_report, parse_component_values, parse_font_face_descriptor_value, parse_sheet,
};

fn checked(kind: Kind, text: &str) -> Authored {
    Authored::try_from_components(kind, parse_component_values(text).unwrap()).unwrap()
}

#[test]
fn every_known_descriptor_defers_env_and_reenters_its_ordinary_grammar() {
    for (kind, replacement) in [
        (Kind::FontFamily, "Demo"),
        (Kind::Src, "url(face.woff2)"),
        (Kind::FontWeight, "bold"),
        (Kind::FontStyle, "italic"),
        (Kind::FontWidth, "condensed"),
        (Kind::FontDisplay, "swap"),
        (Kind::UnicodeRange, "U+20"),
        (Kind::FontFeatureSettings, "normal"),
    ] {
        let value = checked(kind, "env(selection, fallback)");
        assert_eq!(value.kind(), kind);
        let Authored::Pending(pending) = value else {
            panic!("{} must defer", kind.css_name());
        };
        assert_eq!(pending.kind(), kind);
        assert_eq!(pending.components().items().len(), 1);
        let ordinary = pending
            .reparse_after_substitution(parse_component_values(replacement).unwrap())
            .unwrap();
        assert_eq!(ordinary.kind(), kind);
        match ordinary {
            Ordinary::FontFamily(family) => assert_eq!(family.as_str(), "Demo"),
            Ordinary::Src(sources) => {
                let [CssFontFaceSource::Url(url)] = sources.sources() else {
                    panic!("one URL source")
                };
                assert_eq!(url.url(), "face.woff2");
            }
            Ordinary::FontWeight(weight) => assert_eq!(weight, CssFontFaceWeight::bold()),
            Ordinary::FontStyle(style) => assert_eq!(style, CssFontFaceStyle::Italic),
            Ordinary::FontWidth(width) => assert_eq!(
                width,
                CssFontFaceWidth::Range {
                    start: CssFontWidth::Keyword(CssFontWidthKeyword::Condensed),
                    end: None,
                }
            ),
            Ordinary::FontDisplay(display) => assert_eq!(display, CssFontDisplay::Swap),
            Ordinary::UnicodeRange(ranges) => {
                assert_eq!(ranges.ranges().len(), 1);
                assert_eq!(ranges.ranges()[0].start(), 0x20);
                assert_eq!(ranges.ranges()[0].end(), 0x20);
            }
            Ordinary::FontFeatureSettings(settings) => {
                assert_eq!(settings, CssAuthoredFontFeatureSettings::Normal)
            }
            _ => panic!("unexpected descriptor kind"),
        }
    }
}

#[test]
fn direct_typed_ordinary_record_has_no_invented_name_position() {
    let ordinary = Ordinary::FontWidth(CssFontFaceWidth::Auto);
    let record = CssFontFaceDescriptor::new(ordinary.clone().into());
    assert_eq!(record.position(), None);
    assert_eq!(record.value(), &Authored::Ordinary(ordinary));
    let descriptors = CssFontFaceDescriptors::new(vec![record]);
    assert_eq!(descriptors.occurrences().len(), 1);
    assert!(descriptors.effective(Kind::FontWidth).is_some());
    assert!(descriptors.effective(Kind::Src).is_none());
}

#[test]
fn last_admitted_occurrence_wins_even_when_pending_and_alias_names_mix() {
    let source = "@font-face{font-stretch:condensed;font-width:env(width);FONT-STRETCH:bad;font-family:Demo}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor
    );
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face retained")
    };
    let records = face.descriptors().occurrences().collect::<Vec<_>>();
    assert_eq!(records.len(), 3);
    assert_eq!(
        records[0].position().unwrap().byte_offset().value(),
        source.find("font-stretch:condensed").unwrap()
    );
    assert_eq!(
        records[1].position().unwrap().byte_offset().value(),
        source.find("font-width:env").unwrap()
    );
    assert!(matches!(
        records[0].value(),
        Authored::Ordinary(Ordinary::FontWidth(_))
    ));
    assert!(matches!(records[1].value(), Authored::Pending(_)));
    assert!(std::ptr::eq(
        face.descriptors().effective(Kind::FontWidth).unwrap(),
        records[1]
    ));
    assert!(face.descriptors().effective(Kind::FontDisplay).is_none());

    let source = "@font-face{font-width:env(width);font-stretch:75%}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face retained")
    };
    assert!(matches!(
        face.descriptors()
            .effective(Kind::FontWidth)
            .unwrap()
            .value(),
        Authored::Ordinary(Ordinary::FontWidth(CssFontFaceWidth::Range { .. }))
    ));
}

#[test]
fn raw_and_sheet_pending_values_retain_original_parsed_component_origin() {
    let raw = parse_font_face_descriptor_value("env(width, 75%)", Kind::FontWidth);
    assert!(raw.is_clean(), "{raw:?}");
    let Some(Authored::Pending(pending)) = raw.syntax() else {
        panic!("raw pending descriptor")
    };
    let CssValueOrigin::Parsed(origin) = pending.components().items()[0].origin() else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), "env(width, 75%)");

    let source = "@font-face{FoNt-StReTcH:env(width, 75%)}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face retained")
    };
    let record = face.descriptors().effective(Kind::FontWidth).unwrap();
    assert_eq!(
        record.position().unwrap().byte_offset().value(),
        source.find("FoNt-StReTcH").unwrap()
    );
    let Authored::Pending(pending) = record.value() else {
        panic!("sheet pending descriptor")
    };
    let CssValueOrigin::Parsed(origin) = pending.components().items()[0].origin() else {
        panic!("parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
}

#[test]
fn normalization_retains_ordered_pending_font_face_payload() {
    let report = parse_sheet("@font-face{font-width:75%;font-stretch:env(width)}");
    assert!(report.is_clean(), "{report:?}");
    let normalized = normalize_report(&report).unwrap();
    let [CssNormalizedItem::Rule(rule)] = normalized.syntax().items() else {
        panic!("one normalized font-face rule")
    };
    let CssRuleContextKindRef::FontFace(face) = rule.kind() else {
        panic!("font-face payload")
    };
    let records = face.descriptors().occurrences().collect::<Vec<_>>();
    assert_eq!(records.len(), 2);
    assert!(matches!(
        records[0].value(),
        Authored::Ordinary(Ordinary::FontWidth(_))
    ));
    assert!(matches!(records[1].value(), Authored::Pending(_)));
    assert!(std::ptr::eq(
        face.descriptors().effective(Kind::FontWidth).unwrap(),
        records[1]
    ));
}
