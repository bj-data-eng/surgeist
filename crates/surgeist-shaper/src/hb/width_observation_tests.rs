//! Functional qualification of a new observer, using authored real SFNT rules.
//! These tests claim no preimplementation RED for the new public operation.
use crate::hb::completion_fixture::words;
use crate::hb::width_observation::{
    WidthFeature, WidthInputOutcome as Outcome, WidthObservationCoverage as Coverage,
    WidthObservationPath,
};
use crate::{
    Direction, Feature, FeatureRole, GlyphBuffer, ShapePlan, ShapingRefusal, ShapingStatus, Tag,
    UnicodeBuffer,
};
use alloc::{vec, vec::Vec};
use read_fonts::TableProvider;

use super::font;

fn input(text: &str, observed: bool) -> UnicodeBuffer {
    let mut value = UnicodeBuffer::new();
    if observed {
        value.observe_width_feature(WidthFeature::HalfWidth);
    }
    for (i, ch) in text.chars().enumerate() {
        value.add(ch, u32::try_from(i).unwrap());
    }
    value.set_direction(Direction::LeftToRight);
    value.set_script(crate::script::LATIN);
    value
}
fn feature(tag: &[u8; 4]) -> Feature {
    Feature::new(Tag::new(tag), 1, ..)
}
fn ids(value: &GlyphBuffer) -> Vec<u32> {
    value.glyph_infos().iter().map(|g| g.glyph_id).collect()
}
fn positions(value: &GlyphBuffer) -> Vec<(i32, i32, i32, i32)> {
    value
        .glyph_positions()
        .iter()
        .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
        .collect()
}
fn multiple(from: u16, to: &[u16]) -> (u16, Vec<u8>) {
    let mut result = words(&[
        1,
        u16::try_from(10 + 2 * to.len()).unwrap(),
        1,
        8,
        u16::try_from(to.len()).unwrap(),
    ]);
    result.extend(words(to));
    result.extend(words(&[1, 1, from]));
    (2, result)
}
fn identity(glyph: u16) -> (u16, Vec<u8>) {
    (1, words(&[2, 8, 1, glyph, 1, 1, glyph]))
}
fn layout(features: &[(&[u8; 4], Vec<u16>)], lookups: &[(u16, Vec<u8>)]) -> Vec<u8> {
    // One DFLT language system selects all feature records. Lookup indices and
    // every actual action operand are independently encoded below.
    let n = |v| u16::try_from(v).unwrap();
    let mut scripts = words(&[1]);
    scripts.extend(*b"DFLT");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, n(features.len())]));
    scripts.extend(words(&(0..features.len()).map(n).collect::<Vec<_>>()));
    let mut list = words(&[n(features.len())]);
    let mut offset = 2 + 6 * features.len();
    for (tag, indices) in features {
        list.extend(**tag);
        list.extend(words(&[n(offset)]));
        offset += 4 + 2 * indices.len();
    }
    for (_, indices) in features {
        list.extend(words(&[0, n(indices.len())]));
        list.extend(words(indices));
    }
    let mut actions = words(&[n(lookups.len())]);
    let mut offset = 2 + 2 * lookups.len();
    for (_, data) in lookups {
        actions.extend(words(&[n(offset)]));
        offset += 8 + data.len();
    }
    for (kind, data) in lookups {
        actions.extend(words(&[*kind, 0, 1, 8]));
        actions.extend(data);
    }
    let mut result = words(&[
        1,
        0,
        10,
        n(10 + scripts.len()),
        n(10 + scripts.len() + list.len()),
    ]);
    result.extend(scripts);
    result.extend(list);
    result.extend(actions);
    result
}

#[test]
fn width_observer_request_does_not_enable_and_clear_removes_all_evidence() {
    let gsub = font::layout_table(b"hwid", &[font::replace()]);
    font::with_font(true, true, Some(&gsub), None, &[], |font| {
        assert_eq!(font.get_nominal_glyph(u32::from('A')).unwrap().to_u32(), 35);
        assert_eq!(
            font.font
                .gsub()
                .unwrap()
                .feature_list()
                .unwrap()
                .feature_records()[0]
                .feature_tag()
                .into_bytes(),
            *b"hwid"
        );
        let ordinary = font.shape(input("A", false), &[]);
        assert!(ordinary.width_feature_receipt().unwrap().is_none());
        assert_eq!(ordinary.0.components.bytes(), Some(0));
        let mut source = input("A", false);
        source.observe_width_feature(WidthFeature::ThirdWidth);
        source.observe_width_feature(WidthFeature::HalfWidth);
        let result = font.shape(source, &[]);
        assert_eq!(ids(&result), [35]);
        assert_eq!(positions(&result), positions(&ordinary));
        let receipt = result.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), WidthFeature::HalfWidth);
        assert_eq!(receipt.coverage(), Coverage::Complete);
        assert_eq!(receipt.input_len(), 1);
        assert_eq!(receipt.input(0).unwrap().outcome(), Outcome::NoAction);
        assert!(receipt.input(0).unwrap().provenance().is_empty());
        assert!(receipt.input(1).is_none());
        let mut reused = result.clear();
        reused.push_str("A");
        reused.set_direction(Direction::LeftToRight);
        reused.set_script(crate::script::LATIN);
        let result = font.shape(reused, &[feature(b"hwid")]);
        assert_eq!(ids(&result), [58]);
        assert!(result.width_feature_receipt().unwrap().is_none());
        assert!(result.component_receipt().unwrap().is_none());
    });
}

#[test]
fn width_observer_body_decomposition_keeps_uncovered_mark_branch() {
    let gsub = font::layout_table(b"hwid", &[identity(font::A)]);
    font::with_font(true, true, Some(&gsub), None, &[], |font| {
        assert!(font.get_nominal_glyph(0xc0).is_none());
        assert_eq!(font.get_nominal_glyph(0x300).unwrap().to_u32(), 100);
        for (text, expected) in [
            ("A\u{300}", vec![Outcome::Covered, Outcome::NoAction]),
            ("\u{c0}", vec![Outcome::Partial]),
        ] {
            let features = [feature(b"hwid")];
            let ordinary = font.shape(input(text, false), &features);
            let result = font.shape(input(text, true), &features);
            assert_eq!(ids(&result), [35, 100]);
            assert_eq!(positions(&result), positions(&ordinary));
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.coverage(), Coverage::Complete);
            assert_eq!(receipt.input_len(), expected.len());
            for (i, expected) in expected.into_iter().enumerate() {
                assert_eq!(receipt.input(i).unwrap().outcome(), expected);
            }
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(
                components.components(1),
                Some(if text == "\u{c0}" { &[0][..] } else { &[1][..] })
            );
            assert_eq!(
                components.attachments().count(),
                usize::from(text != "\u{c0}")
            );
            let input = receipt.input(0).unwrap();
            assert!(input
                .provenance()
                .iter()
                .any(|p| p.tag() == Tag::new(b"hwid") && p.role() == FeatureRole::Unclassified));
        }
    });
}

#[test]
fn width_observer_split_reconvergence_requires_all_branches_or_later_action() {
    for after_merge in [false, true] {
        let mut lookups = vec![
            multiple(font::A, &[font::A, font::B]),
            identity(font::A),
            font::ligature(),
        ];
        let mut width = vec![1];
        if after_merge {
            width.push(3);
            lookups.push(identity(font::R));
        }
        // All three are explicit user features in the same native stage. The
        // compiled stage visits sorted lookup indices: split, act on A only,
        // merge actual A+B operands, then optionally act on the merged result.
        let gsub = layout(
            &[(b"hwid", width), (b"salt", vec![0]), (b"ss01", vec![2])],
            &lookups,
        );
        font::with_font(true, true, Some(&gsub), None, &[], |font| {
            assert_eq!(
                font.font
                    .gsub()
                    .unwrap()
                    .lookup_list()
                    .unwrap()
                    .lookup_count(),
                if after_merge { 4 } else { 3 }
            );
            let features = [feature(b"hwid"), feature(b"salt"), feature(b"ss01")];
            let ordinary = font.shape(input("A", false), &features);
            let result = font.shape(input("A", true), &features);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(ids(&result), [58]);
            assert_eq!(positions(&result), positions(&ordinary));
            assert_eq!(positions(&result), [(1201, 0, 0, 0)]);
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.coverage(), Coverage::Complete);
            assert_eq!(
                receipt.input(0).unwrap().outcome(),
                if after_merge {
                    Outcome::Covered
                } else {
                    Outcome::Partial
                }
            );
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.ligatures().count(), 1);
        });
    }
}

#[test]
fn width_observer_null_and_unresolved_adjustments_cannot_be_default_zero_witnesses() {
    for (literal, offset) in [(false, 0), (true, 0), (false, 0xffff), (true, 0xffff)] {
        let format = if literal { 0x11 } else { 0x10 };
        let mut sub = words(&[1, if literal { 10 } else { 8 }, format]);
        if literal {
            sub.extend(words(&[0]));
        }
        sub.extend(words(&[offset, 1, 1, font::A]));
        let gpos = font::layout_table(b"hwid", &[(1, sub)]);
        font::with_font(true, true, None, Some(&gpos), &[], |font| {
            assert!(font.ot_tables.var_store.is_none());
            let features = [feature(b"hwid")];
            let ordinary = font.shape(input("A", false), &features);
            let result = font.shape(input("A", true), &features);
            assert_eq!(ids(&result), [35]);
            assert_eq!(positions(&result), positions(&ordinary));
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert_eq!(
                receipt.coverage(),
                if offset == 0 {
                    Coverage::Complete
                } else {
                    Coverage::Unqualified(WidthObservationPath::ValueRecord)
                }
            );
            assert_eq!(
                receipt.input(0).unwrap().outcome(),
                if literal {
                    Outcome::Covered
                } else {
                    Outcome::NoAction
                }
            );
            // Under Unqualified these are retained diagnostics, never proof.
            assert_eq!(receipt.input(0).unwrap().provenance().is_empty(), !literal);
        });
    }
}

#[test]
fn width_observer_generated_dotted_circle_cannot_claim_mark_source_action() {
    let gsub = font::layout_table(b"hwid", &[identity(font::R)]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let mut value = input("\u{300}", true);
        value.set_flags(crate::BufferFlags::BEGINNING_OF_TEXT);
        let result = font.shape(value, &[feature(b"hwid")]);
        assert_eq!(ids(&result), [58, 100]);
        let components = result.component_receipt().unwrap().unwrap();
        assert_eq!(components.generated(0), Some(true));
        assert_eq!(components.components(0), Some(&[][..]));
        assert_eq!(components.generated_context(0), Some(&[0][..]));
        let receipt = result.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.input_len(), 1);
        assert_eq!(receipt.input(0).unwrap().outcome(), Outcome::NoAction);
        assert!(receipt.input(0).unwrap().provenance().is_empty());
    });
}

#[test]
fn width_observer_cache_admission_survives_off_then_on_and_keeps_positive_history() {
    let valid = identity(font::A).1;
    let mut mixed = font::layout_table(b"hwid", &[(1, valid.clone())]);
    let list = u16::from_be_bytes(mixed[8..10].try_into().unwrap()) as usize;
    let lookup = list + u16::from_be_bytes(mixed[list + 2..list + 4].try_into().unwrap()) as usize;
    mixed.truncate(lookup);
    // A real valid SingleSubst sibling and a rejected unknown subtable format.
    mixed.extend(words(&[
        1,
        0,
        2,
        10,
        u16::try_from(10 + valid.len()).unwrap(),
    ]));
    mixed.extend(valid);
    mixed.extend(words(&[0xffff]));
    let rejected = font::layout_table(b"hwid", &[(1, words(&[0xffff]))]);
    let empty = font::layout_table(b"hwid", &[(1, words(&[2, 6, 0, 1, 0]))]);
    for (gsub, expected, coverage) in [
        (
            mixed,
            Outcome::Covered,
            Coverage::Unqualified(WidthObservationPath::LookupAdmission),
        ),
        (
            rejected,
            Outcome::NoAction,
            Coverage::Unqualified(WidthObservationPath::LookupAdmission),
        ),
        (empty, Outcome::NoAction, Coverage::Complete),
    ] {
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let features = [feature(b"hwid")];
            // The same actual ShaperData cache is first exercised undemanded.
            let ordinary = font.shape(input("A", false), &features);
            assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
            let result = font.shape(input("A", true), &features);
            assert_eq!(ids(&result), ids(&ordinary));
            assert_eq!(positions(&result), positions(&ordinary));
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.coverage(), coverage);
            assert_eq!(receipt.input(0).unwrap().outcome(), expected);
            let disabled = font.shape(input("A", true), &[Feature::new(Tag::new(b"hwid"), 0, ..)]);
            assert_eq!(
                disabled
                    .width_feature_receipt()
                    .unwrap()
                    .unwrap()
                    .coverage(),
                Coverage::Complete
            );
        });
    }
}

#[test]
fn width_observer_failed_references_and_lookup_list_reads_are_not_valid_absence() {
    let invalid = layout(&[(b"hwid", vec![7])], &[identity(font::A)]);
    let mut broken_list = font::layout_table(b"hwid", &[identity(font::A)]);
    broken_list[8..10].copy_from_slice(&0xffffu16.to_be_bytes());
    let empty = layout(&[(b"hwid", vec![])], &[]);
    for (gsub, expected) in [
        (
            invalid,
            Coverage::Unqualified(WidthObservationPath::LookupAdmission),
        ),
        (
            broken_list,
            Coverage::Unqualified(WidthObservationPath::LookupAdmission),
        ),
        (empty, Coverage::Complete),
    ] {
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let features = [feature(b"hwid")];
            let ordinary = font.shape(input("A", false), &features);
            let result = font.shape(input("A", true), &features);
            assert_eq!(ids(&result), [35]);
            assert_eq!(positions(&result), positions(&ordinary));
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.coverage(), expected);
            assert_eq!(receipt.input(0).unwrap().outcome(), Outcome::NoAction);
            assert!(receipt.input(0).unwrap().provenance().is_empty());
        });
    }
}

#[test]
fn width_observer_resource_and_prior_refusals_admit_no_receipt_and_clear_recovers() {
    font::with_font(true, false, None, None, &[], |font| {
        let mut value = input("A", true);
        value.0.refuse(ShapingRefusal::ContextLimit);
        let failed = font.shape(value, &[]);
        assert!(matches!(
            failed.width_feature_receipt(),
            Err(ShapingRefusal::ContextLimit)
        ));
        let mut clean = failed.clear();
        clean.observe_width_feature(WidthFeature::HalfWidth);
        clean.push_str("A");
        clean.set_direction(Direction::LeftToRight);
        clean.set_script(crate::script::LATIN);
        let result = font.shape(clean, &[]);
        assert_eq!(
            result
                .width_feature_receipt()
                .unwrap()
                .unwrap()
                .input(0)
                .unwrap()
                .outcome(),
            Outcome::NoAction
        );
        // Actual initialization and final-reduction guards, with a finite zero
        // work budget at the owning boundary; no fabricated evidence receipt.
        let mut value = input("A", true);
        let plan = ShapePlan::new(
            font,
            Direction::LeftToRight,
            Some(crate::script::LATIN),
            None,
            &[],
        );
        assert!(crate::hb::components::begin(&mut value.0, &plan));
        value.0.max_ops = 0;
        crate::hb::width_observation::finish(&mut value.0);
        assert_eq!(
            value.0.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::WorkLimit)
        );
        assert!(matches!(
            GlyphBuffer(value.0).width_feature_receipt(),
            Err(ShapingRefusal::WorkLimit)
        ));
        let mut value = input("A", true);
        value.0.max_len = 0;
        assert!(!crate::hb::components::begin(&mut value.0, &plan));
        assert_eq!(
            value.0.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
        );
    });
}

fn adjustment_fvar() -> Vec<u8> {
    // One real axis, [0,1] with default0. A public all-zero ShaperInstance
    // becomes empty coordinates; nondefault normalized1 remains a real axis.
    let mut value = words(&[1, 0, 16, 2, 1, 20, 0, 8]);
    value.extend(*b"wdth");
    value.extend([0u32, 0, 0x10000].into_iter().flat_map(u32::to_be_bytes));
    value.extend(words(&[0, 256]));
    value
}
fn adjustment_gdef(with_store: bool, null_outer: bool, delta: i16) -> Vec<u8> {
    if !with_store {
        return words(&[1, 0, 0, 0, 0, 0]);
    }
    // GDEF1.3 -> ItemVariationStore at18. Store region list at12 and data
    // at22: one axis, one region [0,1,1], one item with one signed delta.
    let mut value = words(&[1, 3, 0, 0, 0, 0, 0]);
    value.extend(18u32.to_be_bytes());
    value.extend(words(&[1]));
    value.extend(12u32.to_be_bytes());
    value.extend(words(&[1]));
    value.extend((if null_outer { 0u32 } else { 22 }).to_be_bytes());
    value.extend(words(&[1, 1, 0, 0x4000, 0x4000, 1, 1, 1, 0, delta as u16]));
    value
}

#[test]
fn width_observer_variation_recovery_is_not_a_resolved_zero_action() {
    use crate::{NormalizedCoord, ShaperData, ShaperInstance};
    use read_fonts::types::GlyphId;
    let unqualified = Coverage::Unqualified(WidthObservationPath::ValueRecord);
    // Native defined-zero paths, malformed demanded targets, and a real zero
    // row share numeric0 while requiring distinct semantic evidence.
    for (
        name,
        coordinate,
        outer,
        inner,
        with_store,
        null_outer,
        delta,
        outcome,
        coverage,
        expected_x,
    ) in [
        (
            "default",
            0,
            0,
            0,
            true,
            false,
            7,
            Outcome::NoAction,
            Coverage::Complete,
            0,
        ),
        (
            "unused index",
            0x4000,
            0xffff,
            0xffff,
            true,
            false,
            7,
            Outcome::NoAction,
            Coverage::Complete,
            0,
        ),
        (
            "null outer",
            0x4000,
            0,
            0,
            true,
            true,
            7,
            Outcome::NoAction,
            Coverage::Complete,
            0,
        ),
        (
            "invalid outer",
            0x4000,
            1,
            0,
            true,
            false,
            7,
            Outcome::NoAction,
            unqualified,
            0,
        ),
        (
            "invalid inner",
            0x4000,
            0,
            1,
            true,
            false,
            7,
            Outcome::NoAction,
            unqualified,
            0,
        ),
        (
            "resolved zero",
            0x4000,
            0,
            0,
            true,
            false,
            0,
            Outcome::Covered,
            Coverage::Complete,
            0,
        ),
        (
            "resolved seven",
            0x4000,
            0,
            0,
            true,
            false,
            7,
            Outcome::Covered,
            Coverage::Complete,
            7,
        ),
        (
            "default without store",
            0,
            0,
            0,
            false,
            false,
            7,
            Outcome::NoAction,
            Coverage::Complete,
            0,
        ),
        (
            "unused without store",
            0x4000,
            0xffff,
            0xffff,
            false,
            false,
            7,
            Outcome::NoAction,
            Coverage::Complete,
            0,
        ),
        (
            "demanded missing store",
            0x4000,
            0,
            0,
            false,
            false,
            7,
            Outcome::NoAction,
            unqualified,
            0,
        ),
    ] {
        for literal in [false, true] {
            let mut sub = words(&[
                1,
                if literal { 10 } else { 8 },
                if literal { 0x11 } else { 0x10 },
            ]);
            if literal {
                sub.extend(words(&[0]));
            }
            sub.extend(words(&[
                if literal { 16 } else { 14 },
                1,
                1,
                font::A,
                outer,
                inner,
                0x8000,
            ]));
            let gpos = font::layout_table(b"hwid", &[(1, sub)]);
            let fvar = adjustment_fvar();
            let gdef = adjustment_gdef(with_store, null_outer, delta);
            crate::hb::completion_fixture::with_tables(
                &[(*b"GPOS", &gpos), (*b"GDEF", &gdef), (*b"fvar", &fvar)],
                |base| {
                    assert_eq!(base.font.fvar().unwrap().axis_count(), 1);
                    let instance = ShaperInstance::from_coords(
                        &base.font,
                        [NormalizedCoord::from_bits(coordinate)],
                    );
                    let data = ShaperData::new(&base.font);
                    let font = data.shaper(&base.font).instance(Some(&instance)).build();
                    assert_eq!(font.coords().is_empty(), coordinate == 0);
                    assert_eq!(
                        font.ot_tables.var_store.is_some(),
                        with_store && coordinate != 0,
                        "{name}: the native cache omits variation stores at default coordinates"
                    );
                    if with_store {
                        let gdef = font.font.gdef().unwrap();
                        let store = gdef.item_var_store().unwrap().unwrap();
                        assert_eq!(store.item_variation_data_count(), 1);
                        assert_eq!(store.item_variation_data().get(0).is_none(), null_outer);
                        if !null_outer {
                            let row = store.item_variation_data().get(0).unwrap().unwrap();
                            assert_eq!((row.item_count(), row.region_index_count()), (1, 1));
                            assert_eq!(row.delta_sets(), delta.to_be_bytes());
                        }
                    }
                    let glyph = font.get_nominal_glyph(u32::from('A')).unwrap();
                    assert_eq!(glyph.to_u32(), 35);
                    let advance = font.font.hmtx().unwrap().advance(GlyphId::new(35)).unwrap();
                    let features = [feature(b"hwid")];
                    let ordinary = font.shape(input("A", false), &features);
                    let result = font.shape(input("A", true), &features);
                    assert_eq!(result.shaping_status(), ShapingStatus::Complete, "{name}");
                    assert_eq!(positions(&result), positions(&ordinary), "{name}");
                    assert_eq!(
                        positions(&result),
                        [(i32::from(advance), 0, expected_x, 0)],
                        "{name}"
                    );
                    let receipt = result.width_feature_receipt().unwrap().unwrap();
                    assert_eq!(receipt.coverage(), coverage, "{name}");
                    assert_eq!(
                        receipt.input(0).unwrap().outcome(),
                        if literal { Outcome::Covered } else { outcome },
                        "{name}"
                    );
                    assert_eq!(
                        receipt.input(0).unwrap().provenance().is_empty(),
                        !literal && outcome != Outcome::Covered,
                        "{name}"
                    );
                },
            );
        }
    }
}

#[test]
fn width_observer_known_device_paths_are_unused_and_unknown_format_is_unqualified() {
    for format in [1, 2, 3, 0xffff] {
        // A's sole present field is XPlacementDevice pointing at a real Device
        // header/payload, not a literal placement. Native GPOS ignores ppem data.
        let sub = words(&[1, 8, 0x10, 14, 1, 1, font::A, 10, 10, format, 0]);
        let gpos = font::layout_table(b"hwid", &[(1, sub)]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let features = [feature(b"hwid")];
            let ordinary = font.shape(input("A", false), &features);
            let result = font.shape(input("A", true), &features);
            assert_eq!(positions(&result), positions(&ordinary));
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert_eq!(
                receipt.coverage(),
                if format == 0xffff {
                    Coverage::Unqualified(WidthObservationPath::ValueRecord)
                } else {
                    Coverage::Complete
                }
            );
            assert_eq!(receipt.input(0).unwrap().outcome(), Outcome::NoAction);
            assert!(receipt.input(0).unwrap().provenance().is_empty());
        });
    }
}

#[test]
fn width_observer_declared_delta_views_distinguish_truncation_from_valid_empty_rows() {
    use crate::{NormalizedCoord, ShaperData, ShaperInstance};
    use read_fonts::types::GlyphId;
    for (name, case, qualified, index_view, delta_view) in [
        ("valid signed16 zero", 0, true, 1, 2),
        ("missing declared indices", 1, false, 0, 0),
        ("missing declared delta", 2, false, 1, 0),
        ("incomplete declared row array", 3, false, 1, 0),
        ("word count exceeds regions", 4, false, 1, 4),
        ("valid zero-byte row", 5, true, 0, 0),
        ("valid signed32 zero", 6, true, 1, 4),
        ("valid signed8 zero", 7, true, 1, 1),
        ("truncated signed32 row", 8, false, 1, 0),
    ] {
        let mut gdef = adjustment_gdef(true, false, 0);
        // GDEF header18 + store data offset22 = ItemVariationData at40.
        // Its three counts occupy6 bytes, then one region index and delta.
        match case {
            1 => gdef.truncate(46),
            2 => gdef.truncate(48),
            3 => gdef[40..42].copy_from_slice(&2u16.to_be_bytes()),
            4 => {
                gdef[42..44].copy_from_slice(&2u16.to_be_bytes());
                gdef.extend(words(&[0]));
            }
            5 => {
                gdef.truncate(40);
                gdef.extend(words(&[1, 0, 0]));
            }
            6 => {
                gdef[42..44].copy_from_slice(&0x8001u16.to_be_bytes());
                gdef.extend(words(&[0]));
            }
            7 => {
                gdef[42..44].copy_from_slice(&0u16.to_be_bytes());
                gdef.truncate(49);
            }
            8 => gdef[42..44].copy_from_slice(&0x8001u16.to_be_bytes()),
            _ => {}
        }
        let fvar = adjustment_fvar();
        for coordinate in [0, 0x4000] {
            for literal in [false, true] {
                let mut sub = words(&[
                    1,
                    if literal { 10 } else { 8 },
                    if literal { 0x11 } else { 0x10 },
                ]);
                if literal {
                    sub.extend(words(&[0]));
                }
                sub.extend(words(&[
                    if literal { 16 } else { 14 },
                    1,
                    1,
                    font::A,
                    0,
                    0,
                    0x8000,
                ]));
                let gpos = font::layout_table(b"hwid", &[(1, sub)]);
                crate::hb::completion_fixture::with_tables(
                    &[(*b"GPOS", &gpos), (*b"GDEF", &gdef), (*b"fvar", &fvar)],
                    |base| {
                        let instance = ShaperInstance::from_coords(
                            &base.font,
                            [NormalizedCoord::from_bits(coordinate)],
                        );
                        let data = ShaperData::new(&base.font);
                        let font = data.shaper(&base.font).instance(Some(&instance)).build();
                        assert_eq!(font.coords().is_empty(), coordinate == 0);
                        let gdef = font.font.gdef().unwrap();
                        let store = gdef.item_var_store().unwrap().unwrap();
                        let row = store.item_variation_data().get(0).unwrap().unwrap();
                        assert_eq!(row.region_indexes().len(), index_view, "{name}");
                        assert_eq!(row.delta_sets().len(), delta_view, "{name}");
                        assert!(row.item_count() > 0); // Actual requested inner0 exists.
                        let glyph = font.get_nominal_glyph(u32::from('A')).unwrap();
                        assert_eq!(glyph.to_u32(), 35);
                        let advance = font.font.hmtx().unwrap().advance(GlyphId::new(35)).unwrap();
                        let features = [feature(b"hwid")];
                        let ordinary = font.shape(input("A", false), &features);
                        let result = font.shape(input("A", true), &features);
                        assert_eq!(result.shaping_status(), ShapingStatus::Complete, "{name}");
                        assert_eq!(positions(&result), positions(&ordinary), "{name}");
                        assert_eq!(
                            positions(&result),
                            [(i32::from(advance), 0, 0, 0)],
                            "{name}"
                        );
                        let receipt = result.width_feature_receipt().unwrap().unwrap();
                        assert_eq!(
                            receipt.coverage(),
                            if coordinate == 0 || qualified {
                                Coverage::Complete
                            } else {
                                Coverage::Unqualified(WidthObservationPath::ValueRecord)
                            },
                            "{name}"
                        );
                        let acted = literal || coordinate != 0 && qualified;
                        assert_eq!(
                            receipt.input(0).unwrap().outcome(),
                            if acted {
                                Outcome::Covered
                            } else {
                                Outcome::NoAction
                            },
                            "{name}"
                        );
                        assert_eq!(
                            receipt.input(0).unwrap().provenance().is_empty(),
                            !acted,
                            "{name}"
                        );
                    },
                );
            }
        }
    }
}
