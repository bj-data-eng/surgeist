//! Real-font qualification of demanded lookup data and cached recovery.
//! Native tolerant output has its own byte-derived oracle. Qualification must
//! retain failed reads without changing that output or inventing an action.
use super::*;

fn data_set(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

fn data_flags(buffer: &GlyphBuffer) -> Vec<(bool, bool, bool)> {
    buffer
        .glyph_infos()
        .iter()
        .map(|glyph| {
            (
                glyph.unsafe_to_break(),
                glyph.unsafe_to_concat(),
                glyph.safe_to_insert_tatweel(),
            )
        })
        .collect()
}

fn data_shape(font: &crate::Shaper<'_>, raw: &str, requests: &[Feature]) -> GlyphBuffer {
    let flags = BufferFlags::PRODUCE_UNSAFE_TO_CONCAT | BufferFlags::PRODUCE_SAFE_TO_INSERT_TATWEEL;
    let mut plain = input(raw, Direction::LeftToRight);
    plain.set_flags(flags);
    let plan = make_plan(font, &plain, requests);
    // This very Shaper (including its caches), FontRef and plan are reused.
    // Equal bytes loaded into a second font would not establish off->on reuse.
    let ordinary = font.shape_with_plan(&plan, plain, requests);
    assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
    assert!(ordinary.width_feature_receipt().unwrap().is_none());
    let mut marked = observed(raw, Direction::LeftToRight, WidthFeature::HalfWidth);
    marked.set_flags(flags);
    let result = font.shape_with_plan(&plan, marked, requests);
    assert_eq!(result.shaping_status(), ShapingStatus::Complete);
    assert_eq!(ids(&result), ids(&ordinary));
    assert_eq!(old_positions(&result), old_positions(&ordinary));
    assert_eq!(data_flags(&result), data_flags(&ordinary));
    assert_eq!(
        result
            .glyph_infos()
            .iter()
            .map(|glyph| glyph.cluster)
            .collect::<Vec<_>>(),
        ordinary
            .glyph_infos()
            .iter()
            .map(|glyph| glyph.cluster)
            .collect::<Vec<_>>()
    );
    result
}

fn data_packet(buffer: &GlyphBuffer, glyphs: &[u16], advances: &[i32], clusters: &[u32]) {
    assert_eq!(glyphs.len(), advances.len());
    assert_eq!(
        ids(buffer),
        glyphs.iter().copied().map(u32::from).collect::<Vec<_>>()
    );
    assert_eq!(
        old_positions(buffer),
        advances
            .iter()
            .map(|advance| (*advance, 0, 0, 0))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        buffer
            .glyph_infos()
            .iter()
            .map(|glyph| glyph.cluster)
            .collect::<Vec<_>>(),
        clusters
    );
}

fn data_limit(buffer: &GlyphBuffer, inputs: usize, positioning: bool) {
    let receipt = buffer.width_feature_receipt().unwrap().unwrap();
    assert_eq!(receipt.feature(), WidthFeature::HalfWidth);
    assert_eq!(receipt.input_len(), inputs);
    match receipt.coverage() {
        Coverage::Unqualified(WidthObservationPath::LookupAdmission) => {}
        Coverage::Unqualified(WidthObservationPath::ValueRecord) if positioning => {}
        coverage => panic!("demanded malformed data was not qualified: {coverage:?}"),
    }
    // Outcomes remain available for diagnosis. They cannot erase qualification;
    // cache admission and runtime reads need not choose an identical hook order.
    for ordinal in 0..inputs {
        assert!(receipt.input(ordinal).is_some());
    }
    assert!(receipt.input(inputs).is_none());
}

#[test]
fn selected_substitute_declared_count_failures_differ_from_empty_coverage_and_unselected_data() {
    let feature = WidthFeature::HalfWidth;
    for declared in [0, 0xffff] {
        // Single2 header6, real Coverage at6 with A at index0. Count0 has no
        // selected substitute; count65535 requests 131070 bytes, not this body.
        let lookup = (1, words(&[2, 6, declared, 1, 1, font::A]));
        let gsub = font::layout_table(b"hwid", core::slice::from_ref(&lookup));
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "A", &[enabled(feature)]);
            data_packet(&result, &[font::A], &[1000], &[0]);
            data_limit(&result, 1, false);
            expect(
                &data_shape(font, "R", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
            expect(
                &data_shape(font, "A", &[Feature::new(feature.tag(), 0, ..)]),
                feature,
                &[Outcome::NoAction],
            );
        });
        let gsub = font::layout_table(b"zzzz", &[lookup]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(
                font,
                "A",
                &[enabled(feature), Feature::new(Tag::new(b"zzzz"), 1, ..)],
            );
            data_packet(&result, &[font::A], &[1000], &[0]);
            expect(&result, feature, &[Outcome::NoAction]);
        });
    }
    let gsub = font::layout_table(b"hwid", &[empty_single()]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        expect(
            &data_shape(font, "A", &[enabled(feature)]),
            feature,
            &[Outcome::NoAction],
        );
    });
}

#[test]
fn coverage_declared_body_recovery_cannot_turn_a_failed_primary_digest_into_valid_absence() {
    let feature = WidthFeature::HalfWidth;
    for count in [0, 0xffff] {
        // Single1 delta0; Coverage1 at6 has count0 or65535 and no glyph IDs.
        // The valid empty Coverage and failed declared body both give native an
        // empty primary digest. Only the former proves actual absence.
        let gsub = font::layout_table(b"hwid", &[(1, words(&[1, 6, 0, 1, count]))]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "A", &[enabled(feature)]);
            data_packet(&result, &[font::A], &[1000], &[0]);
            if count == 0 {
                expect(&result, feature, &[Outcome::NoAction]);
            } else {
                data_limit(&result, 1, false);
            }
            expect(&data_shape(font, "", &[enabled(feature)]), feature, &[]);
            expect(
                &data_shape(font, "A", &[Feature::new(feature.tag(), 0, ..)]),
                feature,
                &[Outcome::NoAction],
            );
        });
    }
}

#[test]
fn multiple_genuine_empty_deletion_is_qualified_but_truncated_declared_sequence_is_not() {
    let feature = WidthFeature::HalfWidth;
    for declared in [0, 0xffff] {
        let mut lookup = multiple(font::A, &[]);
        // Sequence at8 has glyphCount at8; Coverage starts10. Count0 is the
        // native supported empty deletion. Count65535 cannot fit in this font,
        // so Read Fonts recovers an empty ID slice and native still deletes A.
        assert_eq!(lookup.1.len(), 16);
        data_set(&mut lookup.1, 8, declared);
        let gsub = font::layout_table(b"hwid", &[lookup]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "A", &[enabled(feature)]);
            data_packet(&result, &[], &[], &[]);
            if declared == 0 {
                expect(&result, feature, &[Outcome::NoSurvivingOperand]);
            } else {
                data_limit(&result, 1, false);
            }
        });
    }
}

#[test]
fn selected_sequence_offset_failure_is_not_a_qualified_empty_deletion() {
    let feature = WidthFeature::HalfWidth;
    let mut lookup = multiple(font::A, &[]);
    data_set(&mut lookup.1, 6, 0xffff); // Required Sequence offset, not nullable.
    let gsub = font::layout_table(b"hwid", &[lookup]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = data_shape(font, "A", &[enabled(feature)]);
        data_packet(&result, &[font::A], &[1000], &[0]);
        data_limit(&result, 1, false);
        expect(
            &data_shape(font, "R", &[enabled(feature)]),
            feature,
            &[Outcome::NoAction],
        );
    });
}

#[test]
fn alternate_empty_and_out_of_range_choice_are_valid_but_unreadable_declared_sets_are_not() {
    let feature = WidthFeature::HalfWidth;
    // Multiple and Alternate format1 share these header/offset/ID-list layouts.
    // The lookup type, and therefore the actual native operation, is Alternate3.
    for (values, choice, outcome, glyph, advance) in [
        (vec![], 1, Outcome::NoAction, font::A, 1000),
        (vec![font::B], 2, Outcome::NoAction, font::A, 1000),
        (vec![font::B], 1, Outcome::Covered, font::B, 1000),
    ] {
        let (_, bytes) = multiple(font::A, &values);
        let gsub = font::layout_table(b"hwid", &[(3, bytes)]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "A", &[Feature::new(feature.tag(), choice, ..)]);
            data_packet(&result, &[glyph], &[advance], &[0]);
            expect(&result, feature, &[outcome]);
        });
    }
    for (at, value) in [(8, 0xffff), (6, 0xffff), (4, 0)] {
        let (_, mut bytes) = multiple(font::A, &[font::B]);
        // Respectively: unreadable declared ID slice; required Set offset Err;
        // Coverage index0 outside the declared Set-offset array of length0.
        data_set(&mut bytes, at, value);
        let gsub = font::layout_table(b"hwid", &[(3, bytes)]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "A", &[enabled(feature)]);
            data_packet(&result, &[font::A], &[1000], &[0]);
            data_limit(&result, 1, false);
            expect(
                &data_shape(font, "R", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
        });
    }
}

#[test]
fn ligature_cached_secondary_digest_cannot_hide_a_visited_bad_candidate_on_observation_reuse() {
    let feature = WidthFeature::HalfWidth;
    // Header8, Set at8 (count2, offsets6 andffff), valid Ligature at14,
    // Coverage at20. The valid candidate is A+R->R; the other offset is bad.
    // collect_seconds visits both while building its digest. The retained good
    // second R58 has low digest bit58; B36 has bit36 and is excluded by that
    // incomplete digest. This is authored input, not a digest-cache proxy.
    let lookup = (
        4,
        words(&[
            1,
            20,
            1,
            8,
            2,
            6,
            0xffff,
            font::R,
            2,
            font::R,
            1,
            1,
            font::A,
        ]),
    );
    let gsub = font::layout_table(b"hwid", &[lookup]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let failed = data_shape(font, "AB", &[enabled(feature)]);
        data_packet(&failed, &[font::A, font::B], &[1000, 1000], &[0, 1]);
        data_limit(&failed, 2, false);
        let acted = data_shape(font, "AR", &[enabled(feature)]);
        data_packet(&acted, &[font::R], &[1201], &[0]);
        data_limit(&acted, 2, false);
        let receipt = acted.width_feature_receipt().unwrap().unwrap();
        for root in 0..2 {
            assert_eq!(receipt.input(root).unwrap().outcome(), Outcome::Covered);
        }
        // Real primary Coverage excludes R: the malformed candidate is not
        // demanded merely because cache construction knows it exists.
        expect(
            &data_shape(font, "R", &[enabled(feature)]),
            feature,
            &[Outcome::NoAction],
        );
        expect(
            &data_shape(font, "AB", &[Feature::new(feature.tag(), 0, ..)]),
            feature,
            &[Outcome::NoAction, Outcome::NoAction],
        );
    });
}

#[test]
fn ligature_genuine_one_component_shortcut_differs_from_truncated_component_recovery() {
    let feature = WidthFeature::HalfWidth;
    for components in [1, 0xffff] {
        let mut lookup = ligature(font::A, font::B, font::R);
        // Ligature starts12; componentCount at14 includes its first glyph.
        // One means zero following IDs. 65535 cannot fit, recovers to zero IDs
        // and reaches the same native in-place replacement shortcut.
        data_set(&mut lookup.1, 14, components);
        let gsub = font::layout_table(b"hwid", &[lookup]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "A", &[enabled(feature)]);
            data_packet(&result, &[font::R], &[1201], &[0]);
            if components == 1 {
                expect(&result, feature, &[Outcome::Covered]);
            } else {
                data_limit(&result, 1, false);
            }
        });
    }
}

#[test]
fn contextual_required_matching_coverage_error_is_scoped_to_the_visited_callback() {
    let feature = WidthFeature::HalfWidth;
    let mut parent = context(Some(1));
    // Context3 A launch at Coverage14, B input Coverage20. The required second
    // Coverage offset field is at8. Exhaustion before B does not read it.
    data_set(&mut parent.1, 8, 0xffff);
    let lookups = [parent, single_subst(font::B, font::R)];
    let gsub = layout(&[(b"hwid", &[0])], &lookups);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = data_shape(font, "AB", &[enabled(feature)]);
        data_packet(&result, &[font::A, font::B], &[1000, 1000], &[0, 1]);
        data_limit(&result, 2, false);
        expect(
            &data_shape(font, "A", &[enabled(feature)]),
            feature,
            &[Outcome::NoAction],
        );
        expect(
            &data_shape(font, "R", &[enabled(feature)]),
            feature,
            &[Outcome::NoAction],
        );
        expect(
            &data_shape(font, "AB", &[Feature::new(feature.tag(), 0, ..)]),
            feature,
            &[Outcome::NoAction, Outcome::NoAction],
        );
    });
    let gsub = layout(&[(b"zzzz", &[0])], &lookups);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        expect(
            &data_shape(
                font,
                "AB",
                &[enabled(feature), Feature::new(Tag::new(b"zzzz"), 1, ..)],
            ),
            feature,
            &[Outcome::NoAction, Outcome::NoAction],
        );
    });
    let gsub = layout(
        &[(b"hwid", &[0])],
        &[context(Some(1)), single_subst(font::B, font::R)],
    );
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        expect(
            &data_shape(font, "AR", &[enabled(feature)]),
            feature,
            &[Outcome::NoAction, Outcome::NoAction],
        ); // valid Coverage mismatch.
    });
}

#[test]
fn contextual_nullable_rule_set_and_empty_rules_are_absent_but_required_decode_failure_is_not() {
    let feature = WidthFeature::HalfWidth;
    for offset in [0, 14, 0xffff] {
        // Context1: one nullable RuleSet offset, Coverage at8. Offset14 points
        // to an actual count0 RuleSet; zero is true nullable absence.
        let gsub = font::layout_table(b"hwid", &[(5, words(&[1, 8, 1, offset, 1, 1, font::A, 0]))]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "A", &[enabled(feature)]);
            data_packet(&result, &[font::A], &[1000], &[0]);
            if offset == 0xffff {
                data_limit(&result, 1, false);
            } else {
                expect(&result, feature, &[Outcome::NoAction]);
            }
        });
    }
}

#[test]
fn contextual_recovered_class_zero_is_not_genuine_default_class_on_cached_reuse() {
    let feature = WidthFeature::HalfWidth;
    for class_format in [2, 99] {
        // Context2 header10; ClassRuleSet10 -> Rule14. That real two-input
        // class0 rule launches child1 on B. Coverage24 admits A. ClassDef30 is
        // either valid Format2/count0 (every glyph class0) or invalid Format99.
        // Native cache metadata recovers the latter to default class0 and still
        // performs B->R, so successful output is not a qualification oracle.
        let parent = (
            5,
            words(&[
                2,
                24,
                30,
                1,
                10,
                1,
                4,
                2,
                1,
                0,
                1,
                1,
                1,
                1,
                font::A,
                class_format,
                0,
            ]),
        );
        let gsub = layout(
            &[(b"hwid", &[0])],
            &[parent, single_subst(font::B, font::R)],
        );
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "AB", &[enabled(feature)]);
            data_packet(&result, &[font::A, font::R], &[1000, 1201], &[0, 1]);
            if class_format == 2 {
                expect(&result, feature, &[Outcome::NoAction, Outcome::Covered]);
            } else {
                data_limit(&result, 2, false);
                assert_eq!(
                    result
                        .width_feature_receipt()
                        .unwrap()
                        .unwrap()
                        .input(1)
                        .unwrap()
                        .outcome(),
                    Outcome::Covered
                ); // actual child history survives.
            }
            expect(
                &data_shape(font, "R", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
        });
    }
}

#[test]
fn pair_position_required_set_count_and_visited_search_failures_limit_evidence() {
    let feature = WidthFeature::HalfWidth;
    for (at, value) in [(10, 0xffff), (8, 0), (12, 0xffff)] {
        let mut lookup = pair_pos(false, 4, 0);
        // Pair1: declared Set count at8; required Set offset at10 ->12;
        // Pair count at12. Count65535 makes binary search visit an absent glyph
        // ID (mid32767), before any ValueRecord can be applied.
        data_set(&mut lookup.1, at, value);
        let gpos = font::layout_table(b"hwid", &[lookup]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let result = data_shape(font, "AB", &[enabled(feature)]);
            data_packet(&result, &[font::A, font::B], &[1000, 1000], &[0, 1]);
            data_limit(&result, 2, true);
            expect(
                &data_shape(font, "A", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
            expect(
                &data_shape(font, "R", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
            expect(
                &data_shape(font, "AB", &[Feature::new(feature.tag(), 0, ..)]),
                feature,
                &[Outcome::NoAction, Outcome::NoAction],
            );
        });
    }
}

#[test]
fn pair_position_empty_and_exhausted_valid_search_remain_complete_no_action() {
    let feature = WidthFeature::HalfWidth;
    for empty in [false, true] {
        let mut lookup = pair_pos(false, 4, 0);
        if empty {
            data_set(&mut lookup.1, 12, 0); // legitimate PairSet/count0.
        }
        let gpos = font::layout_table(b"hwid", &[lookup]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let result = data_shape(font, "AR", &[enabled(feature)]);
            data_packet(&result, &[font::A, font::R], &[1000, 1201], &[0, 1]);
            expect(&result, feature, &[Outcome::NoAction, Outcome::NoAction]);
            let result = data_shape(font, "AB", &[enabled(feature)]);
            expect(
                &result,
                feature,
                if empty {
                    &[Outcome::NoAction, Outcome::NoAction]
                } else {
                    &[Outcome::Covered, Outcome::NoAction]
                },
            );
        });
    }
}

#[test]
fn pair_position_class_outside_declared_matrix_cannot_borrow_coverage_bytes_as_proof() {
    let feature = WidthFeature::HalfWidth;
    let mut lookup = pair_pos(true, 4, 0);
    assert_eq!(lookup.1.len(), 32);
    // Actual ClassDef1 at32: A is class1. Header declares class1Count1, so only
    // class0 has a matrix record. Native offset16 + class1*2*class2Count1 =18
    // reads CoverageFormat1 as XAdvance+1. Keep that tolerant arithmetic; its
    // readable bytes are not an authored selected class1 record.
    data_set(&mut lookup.1, 8, 32);
    lookup.1.extend(words(&[1, font::A, 1, 1]));
    let gpos = font::layout_table(b"hwid", &[lookup]);
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        let result = data_shape(font, "AB", &[enabled(feature)]);
        data_packet(&result, &[font::A, font::B], &[1001, 1000], &[0, 1]);
        data_limit(&result, 2, true);
    });
}

#[test]
fn single_position_missing_declared_record_cannot_borrow_coverage_bytes_as_proof() {
    let feature = WidthFeature::HalfWidth;
    for declared in [0, 1] {
        let mut bytes = words(&[2, if declared == 0 { 8 } else { 10 }, 4, declared]);
        if declared == 1 {
            bytes.extend(words(&[0]));
        }
        bytes.extend(words(&[1, 1, font::A]));
        let gpos = font::layout_table(b"hwid", &[(1, bytes)]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let result = data_shape(font, "A", &[enabled(feature)]);
            // Single2 always computes byte8 for Coverage index0. The valid
            // declared record holds0; the missing one aliases CoverageFormat1.
            data_packet(
                &result,
                &[font::A],
                &[if declared == 0 { 1001 } else { 1000 }],
                &[0],
            );
            if declared == 0 {
                data_limit(&result, 1, true);
            } else {
                expect(&result, feature, &[Outcome::Covered]);
            }
        });
    }
}

#[test]
fn later_selected_data_failure_keeps_earlier_real_identity_action_and_unreached_child_is_not_tainted(
) {
    let feature = WidthFeature::HalfWidth;
    let missing = (1, words(&[2, 6, 0, 1, 1, font::A]));
    let gsub = font::layout_table(b"hwid", &[single_subst(font::A, font::A), missing.clone()]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = data_shape(font, "A", &[enabled(feature)]);
        data_packet(&result, &[font::A], &[1000], &[0]);
        data_limit(&result, 1, false);
        assert_eq!(
            result
                .width_feature_receipt()
                .unwrap()
                .unwrap()
                .input(0)
                .unwrap()
                .outcome(),
            Outcome::Covered
        );
    });
    // A/B match real Context3, but lookup-record count0 launches no child.
    // The malformed child1 exists in LookupList without being selected itself.
    let gsub = layout(&[(b"hwid", &[0])], &[context(None), missing]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = data_shape(font, "AB", &[enabled(feature)]);
        data_packet(&result, &[font::A, font::B], &[1000, 1000], &[0, 1]);
        expect(&result, feature, &[Outcome::NoAction, Outcome::NoAction]);
    });
}

#[test]
fn reverse_chain_selected_substitutes_and_visited_context_errors_are_not_complete_absence() {
    let feature = WidthFeature::HalfWidth;
    // ReverseChain1: backtrackCount1/offset22, lookaheadCount1/offset28,
    // glyphCount1/substituteR, primary Coverage16(A), context Coverages22/28(B).
    // Thus actual B-A-B permits A->R; both B inputs are only observers.
    let base = words(&[
        1,
        16,
        1,
        22,
        1,
        28,
        1,
        font::R,
        1,
        1,
        font::A,
        1,
        1,
        font::B,
        1,
        1,
        font::B,
    ]);
    assert_eq!(base.len(), 34);
    let gsub = font::layout_table(b"hwid", &[(8, base.clone())]);
    font::with_font(true, false, Some(&gsub), None, &[], |font| {
        let result = data_shape(font, "BAB", &[enabled(feature)]);
        data_packet(
            &result,
            &[font::B, font::R, font::B],
            &[1000, 1201, 1000],
            &[0, 1, 2],
        );
        expect(
            &result,
            feature,
            &[Outcome::NoAction, Outcome::Covered, Outcome::NoAction],
        );
    });
    for declared in [0, 0xffff] {
        let mut bytes = base.clone();
        // Count at12 governs the ID slice starting14. Count0 leaves Coverage
        // index0 without its required substitute; 65535 cannot fit in this
        // table and the returned view recovers empty. Native never substitutes.
        data_set(&mut bytes, 12, declared);
        let gsub = font::layout_table(b"hwid", &[(8, bytes)]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "BAB", &[enabled(feature)]);
            data_packet(
                &result,
                &[font::B, font::A, font::B],
                &[1000, 1000, 1000],
                &[0, 1, 2],
            );
            data_limit(&result, 3, false);
            expect(
                &data_shape(font, "BB", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction, Outcome::NoAction],
            ); // actual primary miss.
        });
    }
    for (at, value, backtrack) in [
        (6, 0xffff, true),
        (22, 99, true),
        (10, 0xffff, false),
        (28, 99, false),
    ] {
        let mut bytes = base.clone();
        // Required lazy Coverage offset Err, or a visited bad Coverage format.
        data_set(&mut bytes, at, value);
        let gsub = font::layout_table(b"hwid", &[(8, bytes)]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "BAB", &[enabled(feature)]);
            data_packet(
                &result,
                &[font::B, font::A, font::B],
                &[1000, 1000, 1000],
                &[0, 1, 2],
            );
            data_limit(&result, 3, false);
            expect(
                &data_shape(font, "BB", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction, Outcome::NoAction],
            );
            // Matching exhausts the needed side before invoking its callback:
            // AB has no predecessor of A; BA has no successor of A. In the BA
            // lane the valid B backtrack matches first. No bad offset is read.
            let (raw, glyphs) = if backtrack {
                ("AB", [font::A, font::B])
            } else {
                ("BA", [font::B, font::A])
            };
            let result = data_shape(font, raw, &[enabled(feature)]);
            data_packet(&result, &glyphs, &[1000, 1000], &[0, 1]);
            expect(&result, feature, &[Outcome::NoAction, Outcome::NoAction]);
            if !backtrack {
                // An earlier ordinary backtrack mismatch also prevents any
                // demand of the malformed lookahead Coverage, despite a next B.
                let result = data_shape(font, "RAB", &[enabled(feature)]);
                data_packet(
                    &result,
                    &[font::R, font::A, font::B],
                    &[1201, 1000, 1000],
                    &[0, 1, 2],
                );
                expect(
                    &result,
                    feature,
                    &[Outcome::NoAction, Outcome::NoAction, Outcome::NoAction],
                );
            }
        });
    }
}

#[test]
fn matched_context_truncated_lookup_records_are_unqualified_while_declared_zero_is_absent() {
    let feature = WidthFeature::HalfWidth;
    for declared in [0, 0xffff] {
        let mut parent = context(None);
        // Context3 still has real A/B Coverages10/16 and glyphCount2. Only
        // seqLookupCount at4 changes. Native reads records starting10, after
        // the full A/B match. Zero requests zero bytes; 65535 requests262140
        // bytes from a 22-byte body, recovers an empty slice and launches none.
        assert_eq!(parent.1.len(), 22);
        data_set(&mut parent.1, 4, declared);
        let gsub = font::layout_table(b"hwid", &[parent]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = data_shape(font, "AB", &[enabled(feature)]);
            data_packet(&result, &[font::A, font::B], &[1000, 1000], &[0, 1]);
            if declared == 0 {
                expect(&result, feature, &[Outcome::NoAction, Outcome::NoAction]);
            } else {
                data_limit(&result, 2, false);
            }
            let result = data_shape(font, "AR", &[enabled(feature)]);
            data_packet(&result, &[font::A, font::R], &[1000, 1201], &[0, 1]);
            expect(&result, feature, &[Outcome::NoAction, Outcome::NoAction]); // real second-input miss.
            expect(
                &data_shape(font, "A", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
            expect(
                &data_shape(font, "B", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
            expect(
                &data_shape(font, "AB", &[Feature::new(feature.tag(), 0, ..)]),
                feature,
                &[Outcome::NoAction, Outcome::NoAction],
            );
        });
    }
}

#[test]
fn skipped_mark_retains_mask_assignment_cluster_provenance_after_native_ligature_cluster_merge() {
    let feature = WidthFeature::HalfWidth;
    let request = Feature::new(feature.tag(), 1, 1..2);
    let disabled = Feature::new(feature.tag(), 0, ..);
    // Native Latin's shared GSUB stage sorts lookup indices: rlig0 performs
    // A+B->R before hwid1 visits the skipped MARK with its identity substitute.
    // These features do not share a lookup or width-enabling provenance.
    let mut gsub = layout(
        &[(b"hwid", &[1]), (b"rlig", &[0])],
        &[
            ligature(font::A, font::B, font::R),
            single_subst(font::MARK, font::MARK),
        ],
    );
    let list = usize::from(u16::from_be_bytes(gsub[8..10].try_into().unwrap()));
    let lookup = list
        + usize::from(u16::from_be_bytes(
            gsub[list + 2..list + 4].try_into().unwrap(),
        ));
    data_set(&mut gsub, lookup + 2, 8); // Real IgnoreMarks on rlig lookup0 only.
                                        // A real later non-width attachment keeps source custody observable. Its
                                        // reference is the produced R (roots0,2), and its dependent is MARK(root1).
    let gpos = font::layout_table(
        b"mark",
        &[font::mark(font::R, font::MARK, (500, 700), (10, 20), 4)],
    );
    font::with_font(true, false, Some(&gsub), Some(&gpos), &[], |font| {
        for (active, requests) in [
            (true, vec![request]),
            (true, vec![disabled, request]),
            (false, vec![disabled]),
        ] {
            let make = || {
                // The owning input helper adds actual scalar clusters0/1/2.
                // MonotoneCharacters keeps them distinct through mask setup;
                // unlike grapheme grouping, it does not merge A+grave first.
                let mut value = input("A\u{300}B", Direction::LeftToRight);
                value.set_cluster_level(crate::BufferClusterLevel::MonotoneCharacters);
                value.set_flags(
                    BufferFlags::PRODUCE_UNSAFE_TO_CONCAT
                        | BufferFlags::PRODUCE_SAFE_TO_INSERT_TATWEEL,
                );
                value
            };
            let plain = make();
            let plan = make_plan(font, &plain, &requests);
            let ordinary = font.shape_with_plan(&plan, plain, &requests);
            assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
            assert!(ordinary.width_feature_receipt().unwrap().is_none());
            let mut marked = make();
            marked.observe_width_feature(feature);
            let result = font.shape_with_plan(&plan, marked, &requests);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(ids(&result), ids(&ordinary));
            assert_eq!(old_positions(&result), old_positions(&ordinary));
            assert_eq!(data_flags(&result), data_flags(&ordinary));
            assert_eq!(
                result
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>(),
                ordinary
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>()
            );
            // Only real rlig0 can produce R here. ligate_input merges the full
            // A/mark/B span to cluster0, including the skipped mark, but keeps
            // the mark's hwid bit originally assigned at cluster1. R advances
            //1201; mark anchors differ by(490,680), hence offset490-1201=-711.
            assert_eq!(
                ids(&result),
                vec![u32::from(font::R), u32::from(font::MARK)]
            );
            assert_eq!(
                result
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>(),
                vec![0, 0]
            );
            assert_eq!(
                old_positions(&result),
                vec![(1201, 0, 0, 0), (0, 0, -711, 680)]
            );
            expect(
                &result,
                feature,
                &[
                    Outcome::NoAction,
                    if active {
                        Outcome::Covered
                    } else {
                        Outcome::NoAction
                    },
                    Outcome::NoAction,
                ],
            );
            if active {
                let receipt = result.width_feature_receipt().unwrap().unwrap();
                let evidence = receipt.input(1).unwrap();
                let provenance = evidence.provenance();
                assert!(provenance.iter().any(|entry| entry.source() == FeatureSource::Request(request)),
                    "requests={requests:?}, root1 mask-assignment cluster1, final cluster0, provenance={provenance:?}");
            }
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.coverage(), ComponentCoverage::Complete);
            assert_eq!(components.components(0), Some(&[0, 2][..]));
            assert_eq!(components.components(1), Some(&[1][..]));
            let mut ligatures = components.ligatures();
            assert_eq!(ligatures.next().unwrap().components(), &[0, 2]);
            assert!(ligatures.next().is_none());
            let mut attachments = components.attachments();
            let attachment = attachments.next().unwrap();
            assert_eq!(attachment.dependent_components(), &[1]);
            assert_eq!(attachment.base_components(), &[0, 2]);
            assert!(attachments.next().is_none());
        }
    });
}
