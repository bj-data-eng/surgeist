//! Independent real-font qualification of requested width-action evidence.
//! Register as a child of vertical_position_tests to reuse its owned font
//! builder. These new-API cases make no preimplementation RED claim.
#[path = "width_feature_ancestry_qualification.rs"]
mod ancestry_qualification;
#[path = "width_feature_data_qualification.rs"]
mod data_qualification;
#[path = "width_feature_lazy_qualification.rs"]
mod lazy_anchor_qualification;

use super::{font, input, make_plan, old_positions};
use crate::hb::completion_fixture::words;
use crate::{
    BufferFlags, ComponentCoverage, ComponentPath, Direction, Feature, FeatureSource, GlyphBuffer,
    ShapingRefusal, ShapingStatus, Tag, UnicodeBuffer, VerticalOriginReference, WidthFeature,
    WidthInputOutcome as Outcome, WidthObservationCoverage as Coverage, WidthObservationPath,
};
use alloc::{vec, vec::Vec};
use core::convert::Infallible;
use read_fonts::{types::GlyphId, TableProvider};

fn observed(raw: &str, direction: Direction, feature: WidthFeature) -> UnicodeBuffer {
    let mut value = input(raw, direction);
    value.observe_width_feature(feature);
    value
}
fn enabled(feature: WidthFeature) -> Feature {
    Feature::new(feature.tag(), 1, ..)
}
fn ids(buffer: &GlyphBuffer) -> Vec<u32> {
    buffer
        .glyph_infos()
        .iter()
        .map(|glyph| glyph.glyph_id)
        .collect()
}
#[track_caller]
fn expect(buffer: &GlyphBuffer, feature: WidthFeature, outcomes: &[Outcome]) {
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    let receipt = buffer
        .width_feature_receipt()
        .unwrap()
        .expect("actual requested observation");
    assert_eq!(receipt.feature(), feature);
    assert_eq!(receipt.input_len(), outcomes.len());
    assert_eq!(receipt.coverage(), Coverage::Complete);
    for (ordinal, outcome) in outcomes.iter().enumerate() {
        let evidence = receipt.input(ordinal).unwrap();
        assert_eq!(
            evidence.outcome(),
            *outcome,
            "provider input root {ordinal}"
        );
        if matches!(outcome, Outcome::NoAction | Outcome::NoSurvivingOperand) {
            assert!(evidence.provenance().is_empty());
        } else {
            assert!(evidence
                .provenance()
                .iter()
                .any(|entry| entry.tag() == feature.tag()));
        }
    }
    assert!(receipt.input(outcomes.len()).is_none());
}
fn nominal(font: &crate::Shaper<'_>) {
    assert_eq!(font.units_per_em(), 1000);
    assert_eq!(
        font.get_nominal_glyph('A' as u32).unwrap().to_u32(),
        u32::from(font::A)
    );
    assert_eq!(
        font.font
            .hmtx()
            .unwrap()
            .advance(GlyphId::new(u32::from(font::A))),
        Some(1000)
    );
}
fn single_subst(from: u16, to: u16) -> (u16, Vec<u8>) {
    (1, words(&[2, 8, 1, to, 1, 1, from]))
}
fn multiple(from: u16, output: &[u16]) -> (u16, Vec<u8>) {
    let mut data = words(&[
        1,
        u16::try_from(10 + output.len() * 2).unwrap(),
        1,
        8,
        u16::try_from(output.len()).unwrap(),
    ]);
    data.extend(words(output));
    data.extend(words(&[1, 1, from]));
    (2, data)
}
fn ligature(first: u16, second: u16, output: u16) -> (u16, Vec<u8>) {
    (
        4,
        words(&[1, 18, 1, 8, 1, 4, output, 2, second, 1, 1, first]),
    )
}
fn single_pos(glyph: u16, format: u16, values: &[u16]) -> (u16, Vec<u8>) {
    assert_eq!(values.len(), format.count_ones() as usize);
    let mut data = words(&[1, u16::try_from(6 + values.len() * 2).unwrap(), format]);
    data.extend(words(values));
    data.extend(words(&[1, 1, glyph]));
    (1, data)
}
fn pair_pos(class_based: bool, first: u16, second: u16) -> (u16, Vec<u8>) {
    let count = (first.count_ones() + second.count_ones()) as usize;
    let zeroes = vec![0; count];
    let data = if class_based {
        let coverage = 16 + count * 2;
        let mut data = words(&[
            2,
            coverage as u16,
            first,
            second,
            (coverage + 6) as u16,
            (coverage + 10) as u16,
            1,
            1,
        ]);
        data.extend(words(&zeroes));
        data.extend(words(&[1, 1, font::A, 2, 0, 2, 0])); // coverage and two empty ClassDefs: default class0.
        data
    } else {
        let mut data = words(&[
            1,
            u16::try_from(16 + count * 2).unwrap(),
            first,
            second,
            1,
            12,
            1,
            font::B,
        ]);
        data.extend(words(&zeroes));
        data.extend(words(&[1, 1, font::A]));
        data
    };
    (2, data)
}

/// Authored OpenType lists, not another SFNT/font encoder. The retained font
/// directory, metrics, cmap and outlines still come from the existing builder.
fn layout(features: &[(&[u8; 4], &[u16])], lookups: &[(u16, Vec<u8>)]) -> Vec<u8> {
    assert!(features.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let mut scripts = words(&[1]);
    scripts.extend(*b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, features.len() as u16]));
    scripts.extend(words(
        &(0..features.len())
            .map(|index| index as u16)
            .collect::<Vec<_>>(),
    ));
    let mut feature_list = words(&[features.len() as u16]);
    let mut offset = 2 + 6 * features.len();
    for (tag, indices) in features {
        feature_list.extend(**tag);
        feature_list.extend(words(&[offset as u16]));
        offset += 4 + 2 * indices.len();
    }
    for (_, indices) in features {
        feature_list.extend(words(&[0, indices.len() as u16]));
        feature_list.extend(words(indices));
    }
    let mut lookup_list = words(&[lookups.len() as u16]);
    offset = 2 + lookups.len() * 2;
    for (_, data) in lookups {
        lookup_list.extend(words(&[offset as u16]));
        offset += 8 + data.len();
    }
    for (kind, data) in lookups {
        lookup_list.extend(words(&[*kind, 0, 1, 8]));
        lookup_list.extend(data);
    }
    let mut result = words(&[
        1,
        0,
        10,
        (10 + scripts.len()) as u16,
        (10 + scripts.len() + feature_list.len()) as u16,
    ]);
    result.extend(scripts);
    result.extend(feature_list);
    result.extend(lookup_list);
    result
}
fn context(child: Option<u16>) -> (u16, Vec<u8>) {
    let coverage = if child.is_some() { 14 } else { 10 };
    let mut data = words(&[3, 2, u16::from(child.is_some()), coverage, coverage + 6]);
    if let Some(child) = child {
        data.extend(words(&[1, child]));
    } // B is the child operand; A is the first input and lookup launch point.
    data.extend(words(&[1, 1, font::A, 1, 1, font::B]));
    (5, data)
}

#[test]
fn observing_a_width_tag_does_not_enable_it_or_change_ordinary_packets() {
    for feature in [
        WidthFeature::HalfWidth,
        WidthFeature::ThirdWidth,
        WidthFeature::QuarterWidth,
    ] {
        let tag = feature.tag().to_be_bytes();
        let gpos = font::layout_table(&tag, &[single_pos(font::A, 4, &[0])]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            nominal(font);
            let ordinary = font.shape(input("A", Direction::LeftToRight), &[]);
            assert!(ordinary.width_feature_receipt().unwrap().is_none());
            assert!(ordinary.component_receipt().unwrap().is_none());
            let absent = font.shape(observed("A", Direction::LeftToRight, feature), &[]);
            expect(&absent, feature, &[Outcome::NoAction]);
            assert_eq!(ids(&absent), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&absent), old_positions(&ordinary));
            let request = enabled(feature);
            let acted = font.shape(observed("A", Direction::LeftToRight, feature), &[request]);
            expect(&acted, feature, &[Outcome::Covered]);
            assert_eq!(old_positions(&acted), vec![(1000, 0, 0, 0)]);
            assert!(acted
                .width_feature_receipt()
                .unwrap()
                .unwrap()
                .input(0)
                .unwrap()
                .provenance()
                .iter()
                .any(|entry| entry.source() == FeatureSource::Request(request)));
            let disabled = font.shape(
                observed("A", Direction::LeftToRight, feature),
                &[Feature::new(feature.tag(), 0, ..)],
            );
            expect(&disabled, feature, &[Outcome::NoAction]);
        });
    }
}

#[test]
fn present_active_zero_fields_are_actions_but_absent_inapplicable_and_null_records_are_not() {
    let feature = WidthFeature::HalfWidth;
    for direction in [Direction::LeftToRight, Direction::TopToBottom] {
        for (format, values, active) in [
            (0, vec![], false),
            (1, vec![0], true),
            (2, vec![0], true),
            (4, vec![0], direction == Direction::LeftToRight),
            (8, vec![0], direction == Direction::TopToBottom),
            (10, vec![0, 0], true), // YPlacement plus YAdvance: placement remains applicable horizontally.
            (0x10, vec![0], false),
            (0xf0, vec![0, 0, 0, 0], false),
        ] {
            let gpos = font::layout_table(b"hwid", &[single_pos(font::A, format, &values)]);
            font::with_font(true, false, None, Some(&gpos), &[], |font| {
                let ordinary = font.shape(input("A", direction), &[enabled(feature)]);
                let result = font.shape(observed("A", direction, feature), &[enabled(feature)]);
                expect(
                    &result,
                    feature,
                    &[if active {
                        Outcome::Covered
                    } else {
                        Outcome::NoAction
                    }],
                );
                assert_eq!(old_positions(&result), old_positions(&ordinary));
                assert_eq!(ids(&result), vec![u32::from(font::A)]);
                let position = result.glyph_positions()[0];
                assert_eq!(
                    (position.x_advance, position.y_advance),
                    if direction == Direction::LeftToRight {
                        (1000, 0)
                    } else {
                        (0, -1200)
                    }
                );
            });
        }
    }
    let gpos = font::layout_table(b"hwid", &[single_pos(font::B, 4, &[0])]);
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        expect(
            &font.shape(
                observed("A", Direction::LeftToRight, feature),
                &[enabled(feature)],
            ),
            feature,
            &[Outcome::NoAction],
        );
    });
    font::with_font(true, false, None, None, &[], |font| {
        let narrow = font.shape(
            observed(".", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        expect(&narrow, feature, &[Outcome::NoAction]);
        assert_eq!(narrow.glyph_positions()[0].x_advance, 500); // already narrow is not an action witness.
    });
}

#[test]
fn pair_formats_credit_each_actual_value_record_operand_independently() {
    let feature = WidthFeature::HalfWidth;
    for class_based in [false, true] {
        for direction in [Direction::LeftToRight, Direction::TopToBottom] {
            let active = if direction == Direction::LeftToRight {
                4
            } else {
                8
            };
            let skipped = if active == 4 { 8 } else { 4 };
            for (first, second, outcomes) in [
                (active, 0, [Outcome::Covered, Outcome::NoAction]),
                (0, active, [Outcome::NoAction, Outcome::Covered]),
                (active, active, [Outcome::Covered, Outcome::Covered]),
                (0, 0, [Outcome::NoAction, Outcome::NoAction]),
                (skipped, 0, [Outcome::NoAction, Outcome::NoAction]),
                (0, skipped, [Outcome::NoAction, Outcome::NoAction]),
                (2, 2, [Outcome::Covered, Outcome::Covered]),
            ] {
                let gpos = font::layout_table(b"hwid", &[pair_pos(class_based, first, second)]);
                font::with_font(true, false, None, Some(&gpos), &[], |font| {
                    let ordinary = font.shape(input("AB", direction), &[enabled(feature)]);
                    let result =
                        font.shape(observed("AB", direction, feature), &[enabled(feature)]);
                    expect(&result, feature, &outcomes);
                    assert_eq!(ids(&result), vec![u32::from(font::A), u32::from(font::B)]);
                    assert_eq!(old_positions(&result), old_positions(&ordinary));
                });
            }
        }
    }
}

#[test]
fn identity_gsub_and_zero_gpos_remain_covered_after_a_later_empty_match() {
    let feature = WidthFeature::HalfWidth;
    for identity in [false, true] {
        let gsub = identity.then(|| font::layout_table(b"hwid", &[single_subst(font::A, font::A)]));
        let mut lookups = vec![];
        if !identity {
            lookups.push(single_pos(font::A, 4, &[0]));
        }
        lookups.push(single_pos(font::A, 0, &[]));
        let gpos = font::layout_table(b"hwid", &lookups);
        font::with_font(true, false, gsub.as_deref(), Some(&gpos), &[], |font| {
            let result = font.shape(
                observed("A", Direction::LeftToRight, feature),
                &[enabled(feature)],
            );
            expect(&result, feature, &[Outcome::Covered]);
            assert_eq!(ids(&result), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&result), vec![(1000, 0, 0, 0)]);
        });
    }
}

#[test]
fn split_sibling_partial_survives_merge_and_only_surviving_paths_control_final_coverage() {
    let feature = WidthFeature::HalfWidth;
    for terminal in [0, 1, 2, 3] {
        let mut lookups = vec![
            multiple(font::A, &[font::B, font::R]),
            single_subst(font::B, font::B),
        ];
        let mut features: Vec<(&[u8; 4], &[u16])> = vec![(b"ccmp", &[0]), (b"hwid", &[1])];
        let extra = match terminal {
            0 => None,
            1 => Some(ligature(font::B, font::R, font::A)),
            2 => Some(multiple(font::R, &[])),
            _ => Some(multiple(font::B, &[])),
        };
        if let Some(extra) = extra {
            lookups.push(extra);
            features.push((b"zzzz", &[2]));
        }
        let gsub = layout(&features, &lookups);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let requests = [enabled(feature), Feature::new(Tag::new(b"zzzz"), 1, ..)];
            let result = font.shape(observed("A", Direction::LeftToRight, feature), &requests);
            let (glyphs, outcome) = match terminal {
                0 => (
                    vec![u32::from(font::B), u32::from(font::R)],
                    Outcome::Partial,
                ),
                1 => (vec![u32::from(font::A)], Outcome::Partial),
                2 => (vec![u32::from(font::B)], Outcome::Covered),
                _ => (vec![u32::from(font::R)], Outcome::NoAction),
            };
            assert_eq!(ids(&result), glyphs);
            expect(&result, feature, &[outcome]);
            if terminal == 1 {
                assert_eq!(result.glyph_positions()[0].x_advance, 1000);
            }
        });
    }
    let gsub = layout(
        &[(b"hwid", &[0]), (b"zzzz", &[1])],
        &[
            single_subst(font::A, font::A),
            multiple(font::A, &[font::B, font::R]),
        ],
    );
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = font.shape(
            observed("A", Direction::LeftToRight, feature),
            &[enabled(feature), Feature::new(Tag::new(b"zzzz"), 1, ..)],
        );
        assert_eq!(ids(&result), vec![u32::from(font::B), u32::from(font::R)]);
        expect(&result, feature, &[Outcome::Covered]); // action before split covers both actual descendants.
    });
    let gsub = layout(
        &[(b"ccmp", &[0]), (b"hwid", &[1, 3]), (b"zzzz", &[2])],
        &[
            multiple(font::A, &[font::B, font::R]),
            single_subst(font::B, font::B),
            ligature(font::B, font::R, font::A),
            single_subst(font::A, font::A),
        ],
    );
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = font.shape(
            observed("A", Direction::LeftToRight, feature),
            &[enabled(feature), Feature::new(Tag::new(b"zzzz"), 1, ..)],
        );
        assert_eq!(ids(&result), vec![u32::from(font::A)]);
        expect(&result, feature, &[Outcome::Covered]); // actual later action covers the previously partial merged operand.
    });
}

#[test]
fn extension_routes_only_count_their_successful_real_leaf_actions() {
    let feature = WidthFeature::HalfWidth;
    for active in [false, true] {
        let (_, leaf) = single_pos(
            font::A,
            if active { 4 } else { 0 },
            if active { &[0] } else { &[] },
        );
        let mut extension = words(&[1, 1, 0, 8]);
        extension.extend(leaf);
        let gpos = font::layout_table(b"hwid", &[(9, extension)]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let result = font.shape(
                observed("A", Direction::LeftToRight, feature),
                &[enabled(feature)],
            );
            expect(
                &result,
                feature,
                &[if active {
                    Outcome::Covered
                } else {
                    Outcome::NoAction
                }],
            );
            assert_eq!(old_positions(&result), vec![(1000, 0, 0, 0)]);
        });
    }
}

#[test]
fn ligature_operands_exclude_a_real_skipped_source_mark() {
    let feature = WidthFeature::HalfWidth;
    let mut gsub = font::layout_table(b"hwid", &[font::ligature()]);
    let list = u16::from_be_bytes(gsub[8..10].try_into().unwrap()) as usize;
    let lookup = list + u16::from_be_bytes(gsub[list + 2..list + 4].try_into().unwrap()) as usize;
    gsub[lookup + 2..lookup + 4].copy_from_slice(&8u16.to_be_bytes()); // real IgnoreMarks flag.
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = font.shape(
            observed("A\u{300}B", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        assert_eq!(
            ids(&result),
            vec![u32::from(font::R), u32::from(font::MARK)]
        );
        expect(
            &result,
            feature,
            &[Outcome::Covered, Outcome::NoAction, Outcome::Covered],
        );
        let components = result.component_receipt().unwrap().unwrap();
        assert_eq!(components.ligatures().next().unwrap().components(), &[0, 2]);
        assert_eq!(components.components(1), Some(&[1][..]));
    });
}

#[test]
fn attachment_reference_is_an_observer_while_dependent_mark_and_cursive_operands_act() {
    let feature = WidthFeature::HalfWidth;
    let gpos = font::layout_table(
        b"hwid",
        &[font::mark(font::A, font::MARK, (500, 700), (10, 20), 4)],
    );
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        let result = font.shape(
            observed("A\u{300}", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        expect(&result, feature, &[Outcome::NoAction, Outcome::Covered]);
        assert_eq!(
            (
                result.glyph_positions()[1].x_offset,
                result.glyph_positions()[1].y_offset
            ),
            (-510, 680)
        );
        let components = result.component_receipt().unwrap().unwrap();
        let attachment = components.attachments().next().unwrap();
        assert_eq!(attachment.dependent_components(), &[1]);
        assert_eq!(attachment.base_components(), &[0]);
    });
    let gpos = font::layout_table(b"hwid", &[font::cursive()]);
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        let result = font.shape(
            observed("AB", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        expect(&result, feature, &[Outcome::Covered, Outcome::Covered]);
        assert_eq!(
            result
                .component_receipt()
                .unwrap()
                .unwrap()
                .attachments()
                .count(),
            1
        );
    });
    let gpos = font::layout_table(
        b"hwid",
        &[font::mark(font::MARK, font::MARK2, (500, 700), (10, 20), 6)],
    );
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        let result = font.shape(
            observed("A\u{300}\u{315}", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        expect(
            &result,
            feature,
            &[Outcome::NoAction, Outcome::NoAction, Outcome::Covered],
        );
        let components = result.component_receipt().unwrap().unwrap();
        let attachment = components.attachments().next().unwrap();
        assert_eq!(attachment.dependent_components(), &[2]);
        assert_eq!(attachment.base_components(), &[1]);
    });
}

#[test]
fn normalization_roots_distinguish_precomposed_body_branches_from_authored_mark_inputs() {
    let feature = WidthFeature::HalfWidth;
    let gsub = font::layout_table(b"hwid", &[single_subst(font::A, font::A)]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let decomposed = font.shape(
            observed("\u{c0}", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        let authored = font.shape(
            observed("A\u{300}", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        assert_eq!(
            ids(&decomposed),
            vec![u32::from(font::A), u32::from(font::MARK)]
        );
        assert_eq!(ids(&decomposed), ids(&authored));
        assert_eq!(old_positions(&decomposed), old_positions(&authored));
        expect(&decomposed, feature, &[Outcome::Partial]);
        expect(&authored, feature, &[Outcome::Covered, Outcome::NoAction]);
        let components = decomposed.component_receipt().unwrap().unwrap();
        assert_eq!(components.components(0), Some(&[0][..]));
        assert_eq!(components.components(1), Some(&[0][..]));
        let components = authored.component_receipt().unwrap().unwrap();
        assert_eq!(components.components(1), Some(&[1][..]));
        // Shaper reports provider roots only. Source EGC/attached-mark policy
        // and any exemption belong to Text and are deliberately not inferred.
    });
}

#[test]
fn generated_circle_actions_cannot_cover_source_mark_and_generated_source_merge_is_unqualified() {
    let feature = WidthFeature::HalfWidth;
    for glyph in [font::R, font::MARK] {
        let gsub = font::layout_table(b"hwid", &[single_subst(glyph, glyph)]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let mut value = observed("\u{300}", Direction::LeftToRight, feature);
            value.set_flags(BufferFlags::BEGINNING_OF_TEXT);
            let result = font.shape(value, &[enabled(feature)]);
            assert_eq!(
                ids(&result),
                vec![u32::from(font::R), u32::from(font::MARK)]
            );
            expect(
                &result,
                feature,
                &[if glyph == font::R {
                    Outcome::NoAction
                } else {
                    Outcome::Covered
                }],
            );
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.generated(0), Some(true));
            assert_eq!(components.components(0), Some(&[][..]));
            assert_eq!(components.generated_context(0), Some(&[0][..]));
        });
    }
    let gsub = font::layout_table(b"hwid", &[ligature(font::R, font::MARK, font::R)]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let mut value = observed("\u{300}", Direction::LeftToRight, feature);
        value.set_flags(BufferFlags::BEGINNING_OF_TEXT);
        let result = font.shape(value, &[enabled(feature)]);
        assert_eq!(ids(&result), vec![u32::from(font::R)]);
        assert_eq!(
            result.width_feature_receipt().unwrap().unwrap().coverage(),
            Coverage::Unqualified(WidthObservationPath::Native(ComponentPath::GeneratedRepair))
        );
    });
}

#[test]
fn contextual_matches_and_static_width_references_are_not_successful_width_operands() {
    let feature = WidthFeature::HalfWidth;
    let empty = font::layout_table(b"hwid", &[context(None)]);
    font::with_font(true, false, Some(&empty), None, &[], |font| {
        expect(
            &font.shape(
                observed("AB", Direction::LeftToRight, feature),
                &[enabled(feature)],
            ),
            feature,
            &[Outcome::NoAction, Outcome::NoAction],
        );
    });
    let lookups = [context(Some(1)), single_subst(font::B, font::R)];
    let actual_width = layout(&[(b"hwid", &[0])], &lookups);
    font::with_font(true, false, Some(&actual_width), None, &[], |font| {
        let result = font.shape(
            observed("AB", Direction::LeftToRight, feature),
            &[enabled(feature)],
        );
        assert_eq!(ids(&result), vec![u32::from(font::A), u32::from(font::R)]);
        expect(&result, feature, &[Outcome::NoAction, Outcome::Covered]);
    });
    for shared_parent in [false, true] {
        let gsub = if shared_parent {
            layout(&[(b"hwid", &[0]), (b"zzzz", &[0])], &lookups)
        } else {
            layout(&[(b"ccmp", &[0]), (b"hwid", &[1])], &lookups)
        };
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let requests = [
                Feature::new(feature.tag(), 0, ..),
                Feature::new(feature.tag(), 1, 0..1),
                Feature::new(Tag::new(b"zzzz"), 1, ..),
            ];
            let result = font.shape(observed("AB", Direction::LeftToRight, feature), &requests);
            assert_eq!(ids(&result), vec![u32::from(font::A), u32::from(font::R)]);
            // In the shared case, hwid genuinely enables the lookup at A.
            // Its successful child at B inherits that launch provenance;
            // B's direct range mask does not re-gate a contextual child.
            // The ccmp-only launch cannot gain credit from the child's static
            // hwid registration. A receives no leaf action in either case.
            expect(
                &result,
                feature,
                &[
                    Outcome::NoAction,
                    if shared_parent {
                        Outcome::Covered
                    } else {
                        Outcome::NoAction
                    },
                ],
            );
            if shared_parent {
                let receipt = result.width_feature_receipt().unwrap().unwrap();
                let evidence = receipt.input(1).unwrap();
                assert!(evidence
                    .provenance()
                    .iter()
                    .any(|entry| entry.source() == FeatureSource::Request(requests[1])));
                assert!(!evidence
                    .provenance()
                    .iter()
                    .any(|entry| entry.source() == FeatureSource::Request(requests[0])));
            }
        });
    }
}

#[test]
fn passive_chain_lookahead_width_mask_does_not_enable_a_child_launched_by_another_feature() {
    let feature = WidthFeature::HalfWidth;
    // ChainContext format3: no backtrack, input A/B, passive lookahead R,
    // then SequenceLookupRecord(input1,lookup1) changes only B to R.
    // The20-byte header precedes real Coverage tables at20,26,32.
    let chain = (
        6,
        words(&[
            3,
            0,
            2,
            20,
            26,
            1,
            32,
            1,
            1,
            1,
            1,
            1,
            font::A,
            1,
            1,
            font::B,
            1,
            1,
            font::R,
        ]),
    );
    let gsub = layout(
        &[(b"hwid", &[0]), (b"zzzz", &[0])],
        &[chain, single_subst(font::B, font::R)],
    );
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let requests = [
            Feature::new(feature.tag(), 0, ..),
            Feature::new(feature.tag(), 1, 2..3),
            Feature::new(Tag::new(b"zzzz"), 1, ..),
        ];
        let result = font.shape(observed("ABR", Direction::LeftToRight, feature), &requests);
        assert_eq!(
            ids(&result),
            vec![u32::from(font::A), u32::from(font::R), u32::from(font::R)]
        );
        // Only zzzz enables the actual launch at A. The selected width tag is
        // enabled on passive lookahead R, which supplies no child provenance.
        expect(
            &result,
            feature,
            &[Outcome::NoAction, Outcome::NoAction, Outcome::NoAction],
        );
    });
}

#[test]
fn shared_gpos_lookup_and_ranged_masks_keep_actual_enabling_request_provenance() {
    let feature = WidthFeature::HalfWidth;
    let gpos = layout(
        &[(b"hwid", &[0]), (b"kern", &[0])],
        &[single_pos(font::A, 4, &[0])],
    );
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        let disabled = Feature::new(feature.tag(), 0, ..);
        expect(
            &font.shape(observed("AA", Direction::LeftToRight, feature), &[disabled]),
            feature,
            &[Outcome::NoAction, Outcome::NoAction],
        );
        let ranged = Feature::new(feature.tag(), 1, 0..1);
        let result = font.shape(
            observed("AA", Direction::LeftToRight, feature),
            &[disabled, ranged],
        );
        expect(&result, feature, &[Outcome::Covered, Outcome::NoAction]);
        let receipt = result.width_feature_receipt().unwrap().unwrap();
        assert!(receipt
            .input(0)
            .unwrap()
            .provenance()
            .iter()
            .any(|entry| entry.source() == FeatureSource::Request(ranged)));
        assert!(!receipt
            .input(0)
            .unwrap()
            .provenance()
            .iter()
            .any(|entry| entry.source() == FeatureSource::Request(disabled)));
    });
}

#[test]
fn provider_input_ordinals_are_not_caller_cluster_aliases() {
    let feature = WidthFeature::HalfWidth;
    let gpos = font::layout_table(b"hwid", &[single_pos(font::A, 4, &[0])]);
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        let mut value = UnicodeBuffer::new();
        value.observe_width_feature(feature);
        value.add('A', 777);
        value.add('B', 777);
        value.set_direction(Direction::LeftToRight);
        value.set_script(crate::script::LATIN);
        let result = font.shape(value, &[enabled(feature)]);
        expect(&result, feature, &[Outcome::Covered, Outcome::NoAction]);
        assert!(result
            .glyph_infos()
            .iter()
            .all(|glyph| glyph.cluster == 777));
    });
}

#[test]
fn qualified_native_deletion_and_residual_di_are_not_positive_feature_actions() {
    let feature = WidthFeature::HalfWidth;
    for acted_before_deletion in [false, true] {
        let gsub = if acted_before_deletion {
            layout(
                &[(b"hwid", &[0]), (b"zzzz", &[1])],
                &[single_subst(font::A, font::A), multiple(font::A, &[])],
            )
        } else {
            font::layout_table(b"ccmp", &[multiple(font::A, &[])])
        };
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = font.shape(
                observed("A", Direction::LeftToRight, feature),
                &[enabled(feature), Feature::new(Tag::new(b"zzzz"), 1, ..)],
            );
            expect(&result, feature, &[Outcome::NoSurvivingOperand]);
            assert!(result.is_empty());
            assert_eq!(
                result
                    .component_receipt()
                    .unwrap()
                    .unwrap()
                    .deleted_input_positions(),
                &[0]
            );
        });
    }
    font::with_font(true, false, None, None, &[], |font| {
        for remove in [false, true] {
            let mut value = observed("\u{2065}", Direction::LeftToRight, feature);
            if remove {
                value.set_flags(BufferFlags::REMOVE_DEFAULT_IGNORABLES);
            }
            let result = font.shape(value, &[enabled(feature)]);
            expect(&result, feature, &[Outcome::NoSurvivingOperand]);
            let di = result.default_ignorable_receipt().unwrap();
            if remove {
                assert_eq!(di.deleted_input_positions(), &[0]);
                assert!(result.is_empty());
            } else {
                assert_eq!(di.residual_input_positions(), &[0]);
                assert_eq!(result.glyph_positions()[0].x_advance, 0);
            }
        }
    });
}

fn di_cmap() -> Vec<u8> {
    // This real cmap makes the DI nominal glyph A, without changing its actual
    // Unicode role. Reuse the font owner for all SFNT directory/metrics work.
    let entries = [
        (32u32, font::SPACE),
        (65, font::A),
        (66, font::B),
        (82, font::R),
        (0x2065, font::A),
    ];
    let mut data = words(&[0, 1, 3, 10]);
    data.extend(12u32.to_be_bytes());
    data.extend(words(&[12, 0]));
    for value in [16 + entries.len() as u32 * 12, 0, entries.len() as u32] {
        data.extend(value.to_be_bytes());
    }
    for (scalar, glyph) in entries {
        for value in [scalar, scalar, u32::from(glyph)] {
            data.extend(value.to_be_bytes());
        }
    }
    data
}
#[test]
fn actual_normal_di_descendant_requires_width_action_even_when_its_input_was_ignorable() {
    let feature = WidthFeature::HalfWidth;
    let cmap = di_cmap();
    for width in [false, true] {
        let lookups = [
            single_subst(font::A, font::R),
            single_subst(font::R, font::R),
        ];
        let gsub = if width {
            layout(&[(b"ccmp", &[0]), (b"hwid", &[1])], &lookups)
        } else {
            layout(&[(b"ccmp", &[0])], &lookups[..1])
        };
        font::with_cmap_font(true, false, Some(&gsub), None, &[], &cmap, |font| {
            assert_eq!(
                font.get_nominal_glyph(0x2065).unwrap().to_u32(),
                u32::from(font::A)
            );
            let result = font.shape(
                observed("\u{2065}", Direction::LeftToRight, feature),
                &[enabled(feature)],
            );
            assert_eq!(ids(&result), vec![u32::from(font::R)]);
            assert_eq!(result.glyph_positions()[0].x_advance, 1201);
            let di = result.default_ignorable_receipt().unwrap();
            assert!(di.residual_input_positions().is_empty());
            assert!(di.deleted_input_positions().is_empty());
            expect(
                &result,
                feature,
                &[if width {
                    Outcome::Covered
                } else {
                    Outcome::NoAction
                }],
            );
        });
    }
}

#[test]
fn observation_selection_clear_first_refusal_and_actual_observer_budgets_preserve_lifecycle() {
    let feature = WidthFeature::HalfWidth;
    font::with_font(true, false, None, None, &[], |font| {
        let mut value = input("A", Direction::LeftToRight);
        value.observe_width_feature(WidthFeature::QuarterWidth);
        value.observe_width_feature(feature);
        let result = font.shape(value, &[]);
        expect(&result, feature, &[Outcome::NoAction]);
        let mut clear = result.clear();
        clear.push_str("A");
        clear.set_direction(Direction::LeftToRight);
        clear.set_script(crate::script::LATIN);
        let result = font.shape(clear, &[]);
        assert!(result.width_feature_receipt().unwrap().is_none());
        assert!(result.component_receipt().unwrap().is_none());
        let mut clear = observed("A", Direction::LeftToRight, feature);
        clear.clear();
        clear.push_str("A");
        clear.set_direction(Direction::LeftToRight);
        clear.set_script(crate::script::LATIN);
        assert!(font
            .shape(clear, &[])
            .width_feature_receipt()
            .unwrap()
            .is_none());
        for refusal in [ShapingRefusal::GrowthLimit, ShapingRefusal::WorkLimit] {
            let mut value = observed("AB", Direction::LeftToRight, feature);
            let plan = make_plan(font, &value, &[]);
            if refusal == ShapingRefusal::GrowthLimit {
                value.0.max_len = 0;
            } else {
                value.0.max_ops = 0;
            }
            assert!(!crate::hb::components::begin(&mut value.0, &plan));
            assert_eq!(value.0.shaping_status(), ShapingStatus::Refused(refusal));
            value.0.refuse(ShapingRefusal::ContextLimit);
            let result = font.shape(value, &[]);
            assert_eq!(result.width_feature_receipt().err(), Some(refusal));
            let mut clean = result.clear();
            clean.observe_width_feature(feature);
            clean.push_str("A");
            clean.set_direction(Direction::LeftToRight);
            clean.set_script(crate::script::LATIN);
            expect(&font.shape(clean, &[]), feature, &[Outcome::NoAction]);
        }
    });
}

#[test]
fn supplied_vertical_front_delegates_the_same_requested_zero_action_evidence() {
    let feature = WidthFeature::HalfWidth;
    let gpos = font::layout_table(b"hwid", &[single_pos(font::A, 8, &[0])]);
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        let value = observed("A", Direction::TopToBottom, feature);
        let requests = [enabled(feature)];
        let plan = make_plan(font, &value, &requests);
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                value,
                &requests,
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |glyph| Ok::<_, Infallible>(font::metrics(font, glyph)),
            )
            .unwrap();
        let receipt = result.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), Coverage::Complete);
        assert_eq!(receipt.feature(), feature);
        assert_eq!(receipt.input_len(), 1);
        assert_eq!(receipt.input(0).unwrap().outcome(), Outcome::Covered);
        assert_eq!(
            result.glyph_positions().next().unwrap().y_advance.to_bits(),
            (-1200.0_f64).to_bits()
        );
    });
}

#[test]
fn actual_aat_execution_is_unqualified_instead_of_complete_width_absence() {
    let morx = font::morx();
    font::with_font(true, false, None, None, &[(*b"morx", &morx)], |font| {
        let result = font.shape(
            observed("A", Direction::LeftToRight, WidthFeature::HalfWidth),
            &[],
        );
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(ids(&result), vec![u32::from(font::R)]);
        assert_eq!(
            result.component_receipt().unwrap().unwrap().coverage(),
            ComponentCoverage::Unqualified(ComponentPath::AatSubstitution)
        );
        assert_eq!(
            result.width_feature_receipt().unwrap().unwrap().coverage(),
            Coverage::Unqualified(WidthObservationPath::Native(ComponentPath::AatSubstitution))
        );
    });
}

fn rejected_single() -> (u16, Vec<u8>) {
    // SingleSubst format2 has one real replacement but an invalid Coverage
    // format99. Cache admission rejects it; no runtime leaf action is claimed.
    (1, words(&[2, 8, 1, font::A, 99, 0]))
}
fn empty_single() -> (u16, Vec<u8>) {
    // A valid empty Coverage format1 and zero substitutes, not a decode error.
    (1, words(&[2, 6, 0, 1, 0]))
}
fn expect_admission_limit(buffer: &GlyphBuffer, outcomes: &[Outcome]) {
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    let receipt = buffer.width_feature_receipt().unwrap().unwrap();
    assert_eq!(receipt.feature(), WidthFeature::HalfWidth);
    assert_eq!(receipt.input_len(), outcomes.len());
    assert_eq!(
        receipt.coverage(),
        Coverage::Unqualified(WidthObservationPath::LookupAdmission)
    );
    for (index, outcome) in outcomes.iter().enumerate() {
        assert_eq!(receipt.input(index).unwrap().outcome(), *outcome);
    }
}

#[test]
fn actual_same_font_and_plan_keep_admission_truth_when_observation_changes_from_off_to_on() {
    let feature = WidthFeature::HalfWidth;
    for rejected in [false, true] {
        let gsub = font::layout_table(
            b"hwid",
            &[if rejected {
                rejected_single()
            } else {
                empty_single()
            }],
        );
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let requests = [enabled(feature)];
            let ordinary = input("A", Direction::LeftToRight);
            let plan = make_plan(font, &ordinary, &requests);
            // Both executions use this exact Shaper/FontRef and ShapePlan.
            // Rebuilding another font from equal bytes would not test reuse.
            let off = font.shape_with_plan(&plan, ordinary, &requests);
            assert_eq!(off.shaping_status(), ShapingStatus::Complete);
            assert!(off.width_feature_receipt().unwrap().is_none());
            let on = font.shape_with_plan(
                &plan,
                observed("A", Direction::LeftToRight, feature),
                &requests,
            );
            assert_eq!(ids(&on), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&on), vec![(1000, 0, 0, 0)]);
            if rejected {
                expect_admission_limit(&on, &[Outcome::NoAction]);
            } else {
                expect(&on, feature, &[Outcome::NoAction]);
            }
        });
    }
}

#[test]
fn admitted_identity_subtable_retains_positive_history_with_a_rejected_sibling() {
    let feature = WidthFeature::HalfWidth;
    let (_, good) = single_subst(font::A, font::A);
    let (_, bad) = rejected_single();
    let mut gsub = font::layout_table(b"hwid", &[(1, vec![])]);
    let list = u16::from_be_bytes(gsub[8..10].try_into().unwrap()) as usize;
    let lookup = list + u16::from_be_bytes(gsub[list + 2..list + 4].try_into().unwrap()) as usize;
    gsub.truncate(lookup);
    gsub.extend(words(&[
        1,
        0,
        2,
        10,
        u16::try_from(10 + good.len()).unwrap(),
    ]));
    gsub.extend(good);
    gsub.extend(bad);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let requests = [enabled(feature)];
        let ordinary = input("A", Direction::LeftToRight);
        let plan = make_plan(font, &ordinary, &requests);
        let off = font.shape_with_plan(&plan, ordinary, &requests);
        let on = font.shape_with_plan(
            &plan,
            observed("A", Direction::LeftToRight, feature),
            &requests,
        );
        assert_eq!(ids(&on), vec![u32::from(font::A)]);
        assert_eq!(old_positions(&on), old_positions(&off));
        // The good identity leaf acts. Its rejected sibling need not execute
        // for this selected lookup's admitted proof to remain incomplete.
        expect_admission_limit(&on, &[Outcome::Covered]);
        assert!(!on
            .width_feature_receipt()
            .unwrap()
            .unwrap()
            .input(0)
            .unwrap()
            .provenance()
            .is_empty());
    });
}

#[test]
fn disabled_ranges_empty_input_and_unselected_bad_lookup_do_not_taint_observation() {
    let feature = WidthFeature::HalfWidth;
    let gsub = font::layout_table(b"hwid", &[rejected_single()]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        for request in [
            Feature::new(feature.tag(), 0, ..),
            Feature::new(feature.tag(), 1, 10..20),
        ] {
            expect(
                &font.shape(observed("A", Direction::LeftToRight, feature), &[request]),
                feature,
                &[Outcome::NoAction],
            );
        }
        expect(
            &font.shape(
                observed("", Direction::LeftToRight, feature),
                &[enabled(feature)],
            ),
            feature,
            &[],
        );
    });
    let gsub = layout(
        &[(b"hwid", &[0]), (b"zzzz", &[1])],
        &[empty_single(), rejected_single()],
    );
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        expect(
            &font.shape(
                observed("A", Direction::LeftToRight, feature),
                &[enabled(feature)],
            ),
            feature,
            &[Outcome::NoAction],
        );
    });
}

#[test]
fn bad_recursive_child_only_qualifies_the_width_path_when_context_reaches_it() {
    let feature = WidthFeature::HalfWidth;
    let gsub = layout(&[(b"hwid", &[0])], &[context(Some(1)), rejected_single()]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let requests = [enabled(feature)];
        let source = input("A", Direction::LeftToRight);
        let plan = make_plan(font, &source, &requests);
        let unreached = font.shape_with_plan(
            &plan,
            observed("A", Direction::LeftToRight, feature),
            &requests,
        );
        expect(&unreached, feature, &[Outcome::NoAction]);
        let reached = font.shape_with_plan(
            &plan,
            observed("AB", Direction::LeftToRight, feature),
            &requests,
        );
        assert_eq!(ids(&reached), vec![u32::from(font::A), u32::from(font::B)]);
        expect_admission_limit(&reached, &[Outcome::NoAction, Outcome::NoAction]);
    });
}

#[test]
fn invalid_selected_lookup_index_and_lookup_list_header_are_not_complete_absence() {
    let feature = WidthFeature::HalfWidth;
    for broken_header in [false, true] {
        let mut gsub = font::layout_table(b"hwid", &[single_subst(font::A, font::A)]);
        if broken_header {
            // Keep the actual script/feature registration, but the selected
            // table's LookupList cannot be admitted at this real offset.
            gsub[8..10].copy_from_slice(&0xffffu16.to_be_bytes());
        } else {
            let list = u16::from_be_bytes(gsub[6..8].try_into().unwrap()) as usize;
            let table =
                list + u16::from_be_bytes(gsub[list + 6..list + 8].try_into().unwrap()) as usize;
            gsub[table + 4..table + 6].copy_from_slice(&23u16.to_be_bytes()); // feature names nonexistent lookup23.
        }
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = font.shape(
                observed("A", Direction::LeftToRight, feature),
                &[enabled(feature)],
            );
            assert_eq!(ids(&result), vec![u32::from(font::A)]);
            expect_admission_limit(&result, &[Outcome::NoAction]);
            let off = Feature::new(feature.tag(), 0, ..);
            expect(
                &font.shape(observed("A", Direction::LeftToRight, feature), &[off]),
                feature,
                &[Outcome::NoAction],
            );
        });
    }
}
