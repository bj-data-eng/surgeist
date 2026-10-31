//! Real registered-font operands for native combined-cell qualification.
//!
//! This cfg(test) child uses the existing Text shaping fixture builders.
//! Native packet/action and registry tests qualify the font independently of
//! future cell preparation, candidate selection, compression and placement.
//!
//! The retained Ahem resource is public domain with CC0 fallback; preserve
//! `tests/fixtures/fonts/ahem/{PROVENANCE.md,COPYING,UPSTREAM-README}`. These are
//! repository-authored SFNT table derivatives, not a fake glyph provider.
//! The maintained Shaper qualification supplies the real SingleSubst/table
//! basis; Text's existing parent owns its SFNT assembly and checksums.
//!
//! Literal operands at UPEM1000: A35=1000, B36=500, R58=400, period16=300.
//! Normal AA therefore has advance2000; requested hwid/twid/qwid give
//! 1000/800/600. For AAA, qwid gives900; AAAA gives1200. These are native
//! advances, not assertions about future cell placement, trial selection or
//! compression. Ahem's real outlines/cmap/face tables remain retained.

#![cfg(test)]

#[path = "combined_tab_width.rs"]
mod tabs;
pub(crate) use tabs::{
    COMBINED_TAB_WIDTH_FAMILY, combined_tab_width_ahem, qualify_combined_tab_width,
};

use super::{AHEM, layout_table, single_substitution};
use crate::test_fonts::{replace_tables, table, u16_at};

pub(crate) const COMBINED_WIDTH_FAMILY: &str = "Surgeist Combined Width Ahem";
pub(crate) const COMBINED_ZERO_FAMILY: &str = "Surgeist Combined Zero Ahem";

fn width_features() -> Vec<([u8; 4], u16, Vec<u8>)> {
    vec![
        (*b"hwid", 1, single_substitution(&[35], &[36])),
        (*b"twid", 1, single_substitution(&[35], &[58])),
        (*b"qwid", 1, single_substitution(&[35], &[16])),
    ]
}

/// Real three-tag GSUB font, uniquely selectable beside original Ahem.
pub(crate) fn combined_width_ahem() -> Vec<u8> {
    fixture(
        COMBINED_WIDTH_FAMILY,
        layout_table(*b"DFLT", width_features()),
    )
}

/// Same real width operands plus an explicitly requested salt A35 -> glyph253.
/// Glyph253 is Ahem's retained zero-advance blank outline. Actual A input is
/// still supplied to the provider; this is neither source-empty nor deletion.
pub(crate) fn combined_zero_ahem() -> Vec<u8> {
    let mut features = width_features();
    features.push((*b"salt", 1, single_substitution(&[35], &[253])));
    fixture(COMBINED_ZERO_FAMILY, layout_table(*b"DFLT", features))
}

fn fixture(family: &str, gsub: Vec<u8>) -> Vec<u8> {
    let count = u16_at(table(AHEM, *b"maxp"), 4);
    assert!(count > 253);
    assert!(u16_at(table(AHEM, *b"hhea"), 34) >= count);
    let mut hmtx = table(AHEM, *b"hmtx").to_vec();
    for (glyph, advance) in [(35_usize, 1000_u16), (36, 500), (58, 400), (16, 300)] {
        hmtx[glyph * 4..glyph * 4 + 2].copy_from_slice(&advance.to_be_bytes());
    }
    replace_tables(
        AHEM,
        vec![
            (*b"hmtx", hmtx),
            (*b"GSUB", gsub),
            (*b"name", renamed_names(family)),
        ],
    )
}

fn identity_name(name: u16, family: &str) -> Option<String> {
    match name {
        1 | 4 | 16 | 18 | 21 => Some(family.to_owned()),
        2 | 17 | 22 => Some("Regular".to_owned()),
        3 => Some(format!("{family};Surgeist784;1")),
        6 => Some(format!("{}-Regular", family.replace(' ', ""))),
        _ => None,
    }
}

fn encoded_identity(platform: u16, encoding: u16, value: &str) -> Vec<u8> {
    assert!(value.is_ascii());
    match platform {
        0 | 3 => value.encode_utf16().flat_map(u16::to_be_bytes).collect(),
        1 => {
            assert_eq!(encoding, 0, "retained Ahem uses Macintosh Roman");
            value.as_bytes().to_vec()
        }
        _ => panic!("unexpected retained Ahem name platform"),
    }
}

// Preserve actual platform/encoding/language records and every nonidentity
// payload (including copyright, description, vendor and licensing URLs).
// The retained source has name format0, independently checked again below.
fn renamed_names(family: &str) -> Vec<u8> {
    let source = table(AHEM, *b"name");
    assert_eq!(u16_at(source, 0), 0);
    let count = usize::from(u16_at(source, 2));
    let strings = usize::from(u16_at(source, 4));
    let mut output = source[..6 + count * 12].to_vec();
    let string_offset = u16::try_from(output.len()).unwrap();
    output[4..6].copy_from_slice(&string_offset.to_be_bytes());
    let mut payload = Vec::new();
    for index in 0..count {
        let row = 6 + index * 12;
        let platform = u16_at(source, row);
        let encoding = u16_at(source, row + 2);
        let name = u16_at(source, row + 6);
        let start = strings + usize::from(u16_at(source, row + 10));
        let len = usize::from(u16_at(source, row + 8));
        let replacement =
            identity_name(name, family).map(|value| encoded_identity(platform, encoding, &value));
        let bytes = replacement
            .as_deref()
            .unwrap_or(&source[start..start + len]);
        output[row + 8..row + 10]
            .copy_from_slice(&u16::try_from(bytes.len()).unwrap().to_be_bytes());
        output[row + 10..row + 12]
            .copy_from_slice(&u16::try_from(payload.len()).unwrap().to_be_bytes());
        payload.extend_from_slice(bytes);
    }
    output.extend(payload);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrifa::{GlyphId, MetadataProvider};
    use surgeist_shaper::{
        Direction, Feature, FeatureSource, FontRef, GlyphBuffer, ShaperData, ShapingStatus, Tag,
        UnicodeBuffer, WidthFeature, WidthInputOutcome, WidthObservationCoverage, script,
    };

    fn shape(
        bytes: &[u8],
        count: usize,
        direction: Direction,
        features: &[Feature],
        observation: Option<WidthFeature>,
    ) -> GlyphBuffer {
        let font = FontRef::new(bytes).unwrap();
        let data = ShaperData::new(&font);
        let shaper = data.shaper(&font).build();
        let mut input = UnicodeBuffer::new();
        for ordinal in 0..count {
            input.add('A', u32::try_from(ordinal).unwrap());
        }
        input.set_direction(direction);
        input.set_script(script::LATIN);
        input.set_language("en".parse().unwrap());
        if let Some(feature) = observation {
            input.observe_width_feature(feature);
        }
        shaper.shape(input, features)
    }

    fn packet(buffer: &GlyphBuffer, count: usize, glyph: u32, advance: i32) {
        assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
        assert_eq!(buffer.glyph_infos().len(), count);
        assert_eq!(buffer.glyph_positions().len(), count);
        for (info, position) in buffer.glyph_infos().iter().zip(buffer.glyph_positions()) {
            assert_eq!(info.glyph_id, glyph);
            assert_eq!(
                (
                    position.x_advance,
                    position.y_advance,
                    position.x_offset,
                    position.y_offset
                ),
                (advance, 0, 0, 0)
            );
        }
        let mut clusters = buffer
            .glyph_infos()
            .iter()
            .map(|info| info.cluster)
            .collect::<Vec<_>>();
        clusters.sort_unstable();
        assert_eq!(
            clusters,
            (0..u32::try_from(count).unwrap()).collect::<Vec<_>>()
        );
    }

    fn evidence(
        buffer: &GlyphBuffer,
        count: usize,
        feature: WidthFeature,
        request: Option<Feature>,
    ) {
        let receipt = buffer.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), feature);
        assert_eq!(receipt.input_len(), count);
        assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
        for ordinal in 0..count {
            let input = receipt.input(ordinal).unwrap();
            match request {
                Some(request) => {
                    assert_eq!(input.outcome(), WidthInputOutcome::Covered);
                    assert!(input.provenance().iter().any(|entry| {
                        entry.tag() == request.tag
                            && entry.source() == FeatureSource::Request(request)
                    }));
                }
                None => {
                    assert_eq!(input.outcome(), WidthInputOutcome::NoAction);
                    assert!(input.provenance().is_empty());
                }
            }
        }
        assert!(receipt.input(count).is_none());
    }

    #[test]
    fn literal_sfnt_routing_metrics_names_and_retained_resources() {
        let bytes = combined_width_ahem();
        let parsed = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(u16_at(table(&bytes, *b"head"), 18), 1000);
        assert_eq!(parsed.charmap().map('A'), Some(GlyphId::new(35)));
        for (glyph, advance) in [(35, 1000), (36, 500), (58, 400), (16, 300), (253, 0)] {
            assert_eq!(u16_at(table(&bytes, *b"hmtx"), glyph * 4), advance);
        }
        for tag in [
            *b"glyf", *b"loca", *b"maxp", *b"cmap", *b"hhea", *b"OS/2", *b"post",
        ] {
            assert_eq!(table(&bytes, tag), table(AHEM, tag));
        }
        let source = table(AHEM, *b"name");
        let renamed = table(&bytes, *b"name");
        assert_eq!(u16_at(source, 0), 0);
        assert_eq!(u16_at(source, 2), u16_at(renamed, 2));
        for index in 0..usize::from(u16_at(source, 2)) {
            let row = 6 + index * 12;
            assert_eq!(&source[row..row + 8], &renamed[row..row + 8]);
            let old_start = usize::from(u16_at(source, 4)) + usize::from(u16_at(source, row + 10));
            let old_len = usize::from(u16_at(source, row + 8));
            let new_start =
                usize::from(u16_at(renamed, 4)) + usize::from(u16_at(renamed, row + 10));
            let new_len = usize::from(u16_at(renamed, row + 8));
            let expected =
                identity_name(u16_at(source, row + 6), COMBINED_WIDTH_FAMILY).map(|value| {
                    encoded_identity(u16_at(source, row), u16_at(source, row + 2), &value)
                });
            assert_eq!(
                &renamed[new_start..new_start + new_len],
                expected
                    .as_deref()
                    .unwrap_or(&source[old_start..old_start + old_len])
            );
        }
        // Independent literal layout readback: DFLT's actual default LangSys
        // exposes all three optional feature records, each with one GSUB1.
        let gsub = table(&bytes, *b"GSUB");
        let scripts = usize::from(u16_at(gsub, 4));
        assert_eq!(u16_at(gsub, scripts), 1);
        assert_eq!(&gsub[scripts + 2..scripts + 6], b"DFLT");
        let script = scripts + usize::from(u16_at(gsub, scripts + 6));
        let language = script + usize::from(u16_at(gsub, script));
        assert_eq!(u16_at(gsub, language + 2), 0xffff);
        assert_eq!(u16_at(gsub, language + 4), 3);
        for index in 0..3 {
            assert_eq!(usize::from(u16_at(gsub, language + 6 + index * 2)), index);
        }
        let features = usize::from(u16_at(gsub, 6));
        let lookups = usize::from(u16_at(gsub, 8));
        assert_eq!(u16_at(gsub, features), 3);
        assert_eq!(u16_at(gsub, lookups), 3);
        for (index, (tag, output)) in [(*b"hwid", 36), (*b"qwid", 16), (*b"twid", 58)]
            .into_iter()
            .enumerate()
        {
            let row = features + 2 + index * 6;
            assert_eq!(&gsub[row..row + 4], &tag);
            let feature = features + usize::from(u16_at(gsub, row + 4));
            assert_eq!(u16_at(gsub, feature), 0);
            assert_eq!(u16_at(gsub, feature + 2), 1);
            assert_eq!(usize::from(u16_at(gsub, feature + 4)), index);
            let lookup = lookups + usize::from(u16_at(gsub, lookups + 2 + index * 2));
            assert_eq!(u16_at(gsub, lookup), 1);
            assert_eq!(u16_at(gsub, lookup + 2), 0);
            assert_eq!(u16_at(gsub, lookup + 4), 1);
            let sub = lookup + usize::from(u16_at(gsub, lookup + 6));
            assert_eq!(u16_at(gsub, sub), 2);
            assert_eq!(u16_at(gsub, sub + 4), 1);
            assert_eq!(u16_at(gsub, sub + 6), output);
            let coverage = sub + usize::from(u16_at(gsub, sub + 2));
            assert_eq!(&gsub[coverage..coverage + 6], &[0, 1, 0, 1, 0, 35]);
        }
    }

    #[test]
    fn each_requested_width_tag_has_exact_horizontal_packets_and_complete_actions() {
        let bytes = combined_width_ahem();
        for (tag, feature, glyph, advance) in [
            (*b"hwid", WidthFeature::HalfWidth, 36, 500),
            (*b"twid", WidthFeature::ThirdWidth, 58, 400),
            (*b"qwid", WidthFeature::QuarterWidth, 16, 300),
        ] {
            let request = Feature::new(Tag::new(&tag), 1, ..);
            for count in [2, 3, 4] {
                for direction in [Direction::LeftToRight, Direction::RightToLeft] {
                    let ordinary = shape(&bytes, count, direction, &[request], None);
                    packet(&ordinary, count, glyph, advance);
                    assert!(ordinary.width_feature_receipt().unwrap().is_none());
                    let observed = shape(&bytes, count, direction, &[request], Some(feature));
                    packet(&observed, count, glyph, advance);
                    evidence(&observed, count, feature, Some(request));
                }
            }
        }
    }

    #[test]
    fn unrequested_observed_and_explicitly_disabled_tags_do_not_change_normal_packets() {
        let bytes = combined_width_ahem();
        let ordinary = shape(&bytes, 2, Direction::LeftToRight, &[], None);
        packet(&ordinary, 2, 35, 1000);
        assert!(ordinary.width_feature_receipt().unwrap().is_none());
        for (tag, feature) in [
            (*b"hwid", WidthFeature::HalfWidth),
            (*b"twid", WidthFeature::ThirdWidth),
            (*b"qwid", WidthFeature::QuarterWidth),
        ] {
            for requests in [vec![], vec![Feature::new(Tag::new(&tag), 0, ..)]] {
                let result = shape(&bytes, 2, Direction::LeftToRight, &requests, Some(feature));
                packet(&result, 2, 35, 1000);
                evidence(&result, 2, feature, None);
            }
        }
        let hwid = Feature::new(Tag::new(b"hwid"), 1, ..);
        let unrelated = shape(
            &bytes,
            2,
            Direction::LeftToRight,
            &[hwid],
            Some(WidthFeature::ThirdWidth),
        );
        packet(&unrelated, 2, 36, 500);
        evidence(&unrelated, 2, WidthFeature::ThirdWidth, None);
    }

    #[test]
    fn renamed_family_is_selected_by_real_registered_only_text_shaping() {
        use crate::{
            Font, FontCatalogueMode, Id, LineHeight, Options, Source, Style, System, SystemOptions,
            WhiteSpace, Wrap,
        };
        let bytes = combined_width_ahem();
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        system.register_fonts(bytes.clone()).unwrap();
        for (setting, glyph, advance) in [
            (None, 35, 10.0),
            (Some("\"hwid\" off"), 35, 10.0),
            (Some("\"hwid\" on"), 36, 5.0),
            (Some("\"twid\" on"), 58, 4.0),
            (Some("\"qwid\" on"), 16, 3.0),
        ] {
            let mut font = Font::new().family(COMBINED_WIDTH_FAMILY);
            if let Some(setting) = setting {
                font = font.feature(setting);
            }
            let layout = system
                .layout(
                    Source::identified("AA", Id::from_u64(78451), 0),
                    Style {
                        font,
                        size: 10.0,
                        line_height: LineHeight::Absolute(10.0),
                        white_space: WhiteSpace::Preserve,
                        wrap: Wrap::None,
                        ..Style::default()
                    },
                    Options {
                        quantize: false,
                        ..Options::default()
                    },
                )
                .unwrap();
            let mut count = 0;
            for run in layout.run_views() {
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.font_size(), 10.0);
                for actual in run.glyphs() {
                    assert_eq!((actual.id(), actual.advance()), (glyph, advance));
                    count += 1;
                }
            }
            assert_eq!(count, 2);
        }
    }

    #[test]
    fn salt_has_real_nonempty_provider_input_and_zero_advance_blank_output() {
        let bytes = combined_zero_ahem();
        assert_eq!(u16_at(table(&bytes, *b"hmtx"), 253 * 4), 0);
        assert_eq!(u16_at(table(&bytes, *b"head"), 50), 0);
        let loca = table(&bytes, *b"loca");
        assert_eq!(u16_at(loca, 253 * 2), u16_at(loca, 254 * 2));
        let ordinary = shape(&bytes, 3, Direction::LeftToRight, &[], None);
        packet(&ordinary, 3, 35, 1000);
        let request = Feature::new(Tag::new(b"salt"), 1, ..);
        let result = shape(
            &bytes,
            3,
            Direction::LeftToRight,
            &[request],
            Some(WidthFeature::HalfWidth),
        );
        packet(&result, 3, 253, 0);
        assert!(!result.is_empty());
        evidence(&result, 3, WidthFeature::HalfWidth, None);
    }
}
