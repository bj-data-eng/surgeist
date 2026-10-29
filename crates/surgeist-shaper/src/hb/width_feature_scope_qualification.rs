//! Real selected-script/language and FeatureVariations width observations.
//! All tags, substitutions and metric expectations are authored independently
//! of WidthFeature::tag. These qualify the new API, not a historical RED.
use super::{font, input, old_positions};
use crate::hb::completion_fixture::words;
use crate::{
    script, Direction, Feature, FeatureSource, GlyphBuffer, Script, ShaperData, ShaperInstance,
    ShapingStatus, Tag, UnicodeBuffer, Variation, WidthFeature, WidthInputOutcome as Outcome,
    WidthObservationCoverage as Coverage, WidthObservationPath,
};
use alloc::{vec, vec::Vec};
use read_fonts::TableProvider;

fn u16n(value: usize) -> u16 {
    u16::try_from(value).unwrap()
}
fn longs(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}
fn single(to: u16) -> (u16, Vec<u8>) {
    // GSUB SingleSubst2, Coverage1 containing A35, one actual replacement.
    (1, words(&[2, 8, 1, to, 1, 1, font::A]))
}
fn rejected_single() -> (u16, Vec<u8>) {
    // A real selected SingleSubst lookup with invalid Coverage format99.
    (1, words(&[2, 8, 1, font::B, 99, 0]))
}
fn source(script: Script, language: &str, observation: Option<WidthFeature>) -> UnicodeBuffer {
    let raw = if script == script::CYRILLIC {
        "\u{0410}"
    } else {
        "A"
    };
    let mut value = input(raw, Direction::LeftToRight);
    value.set_script(script);
    value.set_language(language.parse().unwrap());
    if let Some(feature) = observation {
        value.observe_width_feature(feature);
    }
    value
}
fn expect(
    buffer: &GlyphBuffer,
    feature: WidthFeature,
    request: Feature,
    coverage: Coverage,
    outcome: Outcome,
    glyph: u32,
    advance: i32,
) {
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    assert_eq!(buffer.glyph_infos().len(), 1);
    assert_eq!(buffer.glyph_infos()[0].glyph_id, glyph);
    assert_eq!(buffer.glyph_infos()[0].cluster, 0);
    assert_eq!(old_positions(buffer), vec![(advance, 0, 0, 0)]);
    let receipt = buffer.width_feature_receipt().unwrap().unwrap();
    assert_eq!(receipt.feature(), feature);
    assert_eq!(receipt.input_len(), 1);
    assert_eq!(receipt.coverage(), coverage);
    let evidence = receipt.input(0).unwrap();
    assert_eq!(evidence.outcome(), outcome);
    if outcome == Outcome::Covered {
        assert!(evidence.provenance().iter().any(|entry| {
            entry.tag() == request.tag && entry.source() == FeatureSource::Request(request)
        }));
    } else {
        assert!(evidence.provenance().is_empty());
    }
    assert!(receipt.input(1).is_none());
}

#[test]
fn literal_hwid_twid_qwid_tables_match_the_corresponding_public_variants() {
    // The retained builder gives B36 advance1000, R58 advance1201 and
    // period16 advance500. A35 starts at1000. Coverage does not imply fit.
    for (literal, feature, replacement, glyph, advance) in [
        (*b"hwid", WidthFeature::HalfWidth, font::B, 36, 1000),
        (*b"twid", WidthFeature::ThirdWidth, font::R, 58, 1201),
        (*b"qwid", WidthFeature::QuarterWidth, font::DOT, 16, 500),
    ] {
        let gsub = font::layout_table(&literal, &[single(replacement)]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            assert_eq!(font.units_per_em(), 1000);
            assert_eq!(font.get_nominal_glyph('A' as u32).unwrap().to_u32(), 35);
            let request = Feature::new(Tag::new(&literal), 1, ..);
            let ordinary = font.shape(source(script::LATIN, "en", None), &[request]);
            assert!(ordinary.width_feature_receipt().unwrap().is_none());
            let observed = font.shape(source(script::LATIN, "en", Some(feature)), &[request]);
            expect(
                &observed,
                feature,
                request,
                Coverage::Complete,
                Outcome::Covered,
                glyph,
                advance,
            );
            assert_eq!(old_positions(&observed), old_positions(&ordinary));
            for other in [
                WidthFeature::HalfWidth,
                WidthFeature::ThirdWidth,
                WidthFeature::QuarterWidth,
            ] {
                if other == feature {
                    continue;
                }
                let unrelated = font.shape(source(script::LATIN, "en", Some(other)), &[request]);
                expect(
                    &unrelated,
                    other,
                    request,
                    Coverage::Complete,
                    Outcome::NoAction,
                    glyph,
                    advance,
                );
            }
        });
    }
}

fn script_cmap() -> Vec<u8> {
    // Real Latin A and Cyrillic A both map to the authored base glyph35.
    // Script selection therefore uses authentic source scalars, not a Latin
    // scalar assigned an unrelated script merely to visit a branch.
    let mut format = words(&[12, 0]);
    format.extend(longs(&[40, 0, 2, 0x41, 0x41, 35, 0x410, 0x410, 35]));
    let mut table = words(&[0, 1, 3, 10]);
    table.extend(longs(&[12]));
    table.extend(format);
    table
}

fn langsys(indices: &[u16]) -> Vec<u8> {
    let mut table = words(&[0, 0xffff, u16n(indices.len())]);
    table.extend(words(indices));
    table
}
fn latin_script(default_feature: u16, turkish_feature: u16) -> Vec<u8> {
    let default = langsys(&[default_feature]);
    let mut table = words(&[10, 1]);
    table.extend(*b"TRK ");
    table.extend(words(&[u16n(10 + default.len())]));
    table.extend(default);
    table.extend(langsys(&[turkish_feature]));
    table
}
fn selected_scripts(default_feature: u16, turkish_feature: u16) -> Vec<u8> {
    let mut cyrillic = words(&[4, 0]);
    cyrillic.extend(langsys(&[1]));
    let latin = latin_script(default_feature, turkish_feature);
    let mut table = words(&[2]);
    table.extend(*b"cyrl");
    table.extend(words(&[14]));
    table.extend(*b"latn");
    table.extend(words(&[u16n(14 + cyrillic.len())]));
    table.extend(cyrillic);
    table.extend(latin);
    table
}
fn default_script() -> Vec<u8> {
    let mut table = words(&[1]);
    table.extend(*b"DFLT");
    table.extend(words(&[8, 4, 0]));
    table.extend(langsys(&[0]));
    table
}

/// Actual OT ScriptList/FeatureList/LookupList; the shared builder still owns
/// the SFNT directory, cmap, outlines and metrics. Duplicate hwid records are
/// intentional: the selected LangSys chooses the actual feature INDEX.
fn layout(scripts: &[u8], features: &[&[u16]], lookups: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut feature_list = words(&[u16n(features.len())]);
    let mut offset = 2 + 6 * features.len();
    for indices in features {
        feature_list.extend(*b"hwid");
        feature_list.extend(words(&[u16n(offset)]));
        offset += 4 + 2 * indices.len();
    }
    for indices in features {
        feature_list.extend(words(&[0, u16n(indices.len())]));
        feature_list.extend(words(indices));
    }
    let mut lookup_list = words(&[u16n(lookups.len())]);
    offset = 2 + 2 * lookups.len();
    for (_, data) in lookups {
        lookup_list.extend(words(&[u16n(offset)]));
        offset += 8 + data.len();
    }
    for (kind, data) in lookups {
        lookup_list.extend(words(&[*kind, 0, 1, 8]));
        lookup_list.extend(data);
    }
    let mut table = words(&[
        1,
        0,
        10,
        u16n(10 + scripts.len()),
        u16n(10 + scripts.len() + feature_list.len()),
    ]);
    table.extend(scripts);
    table.extend(feature_list);
    table.extend(lookup_list);
    table
}

#[test]
fn actual_langsys_and_script_selection_bound_rejected_lookup_admission() {
    let cmap = script_cmap();
    // Feature0 is good A35->R58. Feature1 is either rejected lookup1 or a
    // nonexistent lookup23. Neither may taint a different selected LangSys.
    for bad_index in [1, 23] {
        for (default_feature, turkish_feature) in [(0, 1), (1, 0)] {
            let gsub = layout(
                &selected_scripts(default_feature, turkish_feature),
                &[&[0], &[bad_index]],
                &[single(font::R), rejected_single()],
            );
            font::with_cmap_font(true, false, Some(&gsub), None, &[], &cmap, |font| {
                assert_eq!(font.get_nominal_glyph(0x410).unwrap().to_u32(), 35);
                let request = Feature::new(Tag::new(b"hwid"), 1, ..);
                for (script, language, good) in [
                    (script::LATIN, "en", default_feature == 0),
                    (script::LATIN, "tr", turkish_feature == 0),
                    (script::CYRILLIC, "en", false),
                ] {
                    let ordinary = font.shape(source(script, language, None), &[request]);
                    let observed = font.shape(
                        source(script, language, Some(WidthFeature::HalfWidth)),
                        &[request],
                    );
                    expect(
                        &observed,
                        WidthFeature::HalfWidth,
                        request,
                        if good {
                            Coverage::Complete
                        } else {
                            Coverage::Unqualified(WidthObservationPath::LookupAdmission)
                        },
                        if good {
                            Outcome::Covered
                        } else {
                            Outcome::NoAction
                        },
                        if good { 58 } else { 35 },
                        if good { 1201 } else { 1000 },
                    );
                    assert_eq!(old_positions(&observed), old_positions(&ordinary));
                    assert_eq!(
                        observed.glyph_infos()[0].glyph_id,
                        ordinary.glyph_infos()[0].glyph_id
                    );
                }
            });
        }
    }
}

fn fvar() -> Vec<u8> {
    // fvar1.0, one20-byte wdth axis, no named instances. User range-1..1,
    // default0: user coordinates equal normalized coordinates on both sides.
    let mut table = words(&[1, 0, 16, 2, 1, 20, 0, 8]);
    table.extend(*b"wdth");
    table.extend((-65536i32).to_be_bytes());
    table.extend(0i32.to_be_bytes());
    table.extend(65536i32.to_be_bytes());
    table.extend(words(&[0, 256]));
    table
}
fn condition(minimum: u16, maximum: u16) -> Vec<u8> {
    // ConditionSet count1/Offset32=6, then ConditionFormat1 axis0/F2DOT14.
    let mut table = words(&[1]);
    table.extend(longs(&[6]));
    table.extend(words(&[1, 0, minimum, maximum]));
    table
}
fn feature_substitution(indices: &[u16]) -> Vec<u8> {
    // FeatureTableSubstitution1.0, one record replacing feature INDEX0.
    // Offset32=12 points to the real alternate Feature table in this structure.
    let mut table = words(&[1, 0, 1, 0]);
    table.extend(longs(&[12]));
    table.extend(words(&[0, u16n(indices.len())]));
    table.extend(words(indices));
    table
}
fn feature_variations(positive: &[u16], negative: &[u16]) -> Vec<u8> {
    let positive_condition = condition(0x2000, 0x4000); // +0.5..+1.
    let negative_condition = condition(0xc000, 0xe000); // -1..-0.5.
    let positive_feature = feature_substitution(positive);
    let negative_feature = feature_substitution(negative);
    let negative_condition_offset = 24 + positive_condition.len();
    let positive_feature_offset = negative_condition_offset + negative_condition.len();
    let negative_feature_offset = positive_feature_offset + positive_feature.len();
    let mut table = words(&[1, 0]);
    table.extend(longs(&[
        2,
        24,
        u32::try_from(positive_feature_offset).unwrap(),
        u32::try_from(negative_condition_offset).unwrap(),
        u32::try_from(negative_feature_offset).unwrap(),
    ]));
    table.extend(positive_condition);
    table.extend(negative_condition);
    table.extend(positive_feature);
    table.extend(negative_feature);
    table
}
fn variable_gsub(base: &[u16], positive: &[u16], negative: &[u16]) -> Vec<u8> {
    let original = layout(
        &default_script(),
        &[base],
        &[single(font::R), rejected_single()],
    );
    let offset = |at| u16::from_be_bytes(original[at..at + 2].try_into().unwrap()) + 4;
    // GSUB1.1 adds the real Offset32 FeatureVariations field to its header.
    let mut table = words(&[1, 1, offset(4), offset(6), offset(8)]);
    table.extend(longs(&[u32::try_from(original.len() + 4).unwrap()]));
    table.extend_from_slice(&original[10..]);
    table.extend(feature_variations(positive, negative));
    table
}

#[test]
fn real_instances_select_width_actions_and_only_their_rejected_variation_paths() {
    let axis = fvar();
    for rejected_index in [1, 23] {
        for bad_base in [false, true] {
            let base: &[u16] = if bad_base { &[rejected_index] } else { &[] };
            let negative: &[u16] = if bad_base { &[] } else { &[rejected_index] };
            let gsub = variable_gsub(base, &[0], negative);
            font::with_font(
                true,
                false,
                Some(&gsub),
                None,
                &[(*b"fvar", &axis)],
                |font| {
                    assert_eq!(font.font.fvar().unwrap().axis_count(), 1);
                    let data = ShaperData::new(&font.font);
                    for (coordinate, good_action, rejected) in [
                        (0.0, false, bad_base),
                        (0.25, false, bad_base),
                        (0.5, true, false),
                        (1.0, true, false),
                        (-0.5, false, !bad_base),
                        (-1.0, false, !bad_base),
                    ] {
                        let instance = ShaperInstance::from_variations(
                            &font.font,
                            [Variation {
                                tag: Tag::new(b"wdth"),
                                value: coordinate,
                            }],
                        );
                        if coordinate == 0.0 {
                            assert!(instance.coords().is_empty());
                        } else {
                            assert_eq!(instance.coords().len(), 1);
                            assert_eq!(
                                instance.coords()[0].to_f32().to_bits(),
                                coordinate.to_bits()
                            );
                        }
                        let shaper = data.shaper(&font.font).instance(Some(&instance)).build();
                        let request = Feature::new(Tag::new(b"hwid"), 1, ..);
                        let ordinary = shaper.shape(source(script::LATIN, "en", None), &[request]);
                        let observed = shaper.shape(
                            source(script::LATIN, "en", Some(WidthFeature::HalfWidth)),
                            &[request],
                        );
                        expect(
                            &observed,
                            WidthFeature::HalfWidth,
                            request,
                            if rejected {
                                Coverage::Unqualified(WidthObservationPath::LookupAdmission)
                            } else {
                                Coverage::Complete
                            },
                            if good_action {
                                Outcome::Covered
                            } else {
                                Outcome::NoAction
                            },
                            if good_action { 58 } else { 35 },
                            if good_action { 1201 } else { 1000 },
                        );
                        assert_eq!(old_positions(&observed), old_positions(&ordinary));
                        assert_eq!(
                            observed.glyph_infos()[0].glyph_id,
                            ordinary.glyph_infos()[0].glyph_id
                        );
                    }
                },
            );
        }
    }
}
