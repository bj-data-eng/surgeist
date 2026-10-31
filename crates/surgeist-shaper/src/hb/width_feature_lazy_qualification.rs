//! Independent real-font qualification of demanded lazy anchor data.
//! Malformed reads, nullable absence and unselected paths have separate evidence;
//! ordinary tolerant shaping and earlier action history retain their ownership.
use super::*;

fn lazy_word(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn lazy_set(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
}
fn lazy_shape(font: &crate::Shaper<'_>, raw: &str, requests: &[Feature]) -> GlyphBuffer {
    let plain = input(raw, Direction::LeftToRight);
    let plan = make_plan(font, &plain, requests);
    // Keep this exact Shaper/FontRef and ShapePlan across off->on calls.
    let ordinary = font.shape_with_plan(&plan, plain, requests);
    assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
    assert!(ordinary.width_feature_receipt().unwrap().is_none());
    let result = font.shape_with_plan(
        &plan,
        observed(raw, Direction::LeftToRight, WidthFeature::HalfWidth),
        requests,
    );
    assert_eq!(result.shaping_status(), ShapingStatus::Complete);
    assert_eq!(ids(&result), ids(&ordinary));
    assert_eq!(old_positions(&result), old_positions(&ordinary));
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
    let flags = |buffer: &GlyphBuffer| {
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
            .collect::<Vec<_>>()
    };
    assert_eq!(flags(&result), flags(&ordinary));
    result
}
fn lazy_value_limit(result: &GlyphBuffer, outcomes: &[Outcome]) {
    let receipt = result.width_feature_receipt().unwrap().unwrap();
    assert_eq!(receipt.feature(), WidthFeature::HalfWidth);
    assert_eq!(receipt.input_len(), outcomes.len());
    assert_eq!(
        receipt.coverage(),
        Coverage::Unqualified(WidthObservationPath::ValueRecord)
    );
    for (index, outcome) in outcomes.iter().enumerate() {
        assert_eq!(receipt.input(index).unwrap().outcome(), *outcome);
    }
}
fn lazy_cursive() -> (u16, Vec<u8>) {
    let lookup = font::cursive();
    // Cursive1 header: Coverage14; two EntryExitRecords. A has exit22,
    // B has entry28. Each nonnull anchor is AnchorFormat1 with real x/y.
    assert_eq!(lazy_word(&lookup.1, 2), 14);
    assert_eq!(lazy_word(&lookup.1, 4), 2);
    assert_eq!(lazy_word(&lookup.1, 8), 22);
    assert_eq!(lazy_word(&lookup.1, 10), 28);
    assert_eq!(lazy_word(&lookup.1, 22), 1);
    assert_eq!(lazy_word(&lookup.1, 28), 1);
    lookup
}

#[test]
fn lazy_cursive_matched_nonnull_anchor_decode_errors_are_unqualified_value_evidence() {
    let feature = WidthFeature::HalfWidth;
    // Invalid B entry format/offset, and invalid A exit format/offset. The
    // header and real A/B Coverage remain admitted; only consumed anchors fail.
    for (at, value) in [(28, 99), (10, 0xffff), (22, 99), (8, 0xffff)] {
        let mut lookup = lazy_cursive();
        lazy_set(&mut lookup.1, at, value);
        let gpos = font::layout_table(b"hwid", &[lookup]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let result = lazy_shape(font, "AB", &[enabled(feature)]);
            assert_eq!(ids(&result), vec![u32::from(font::A), u32::from(font::B)]);
            assert_eq!(
                old_positions(&result),
                vec![(1000, 0, 0, 0), (1000, 0, 0, 0)]
            );
            lazy_value_limit(&result, &[Outcome::NoAction, Outcome::NoAction]);
        });
    }
}

#[test]
fn lazy_cursive_legitimate_null_anchors_and_unreached_bad_anchors_remain_complete() {
    let feature = WidthFeature::HalfWidth;
    for nullable_offset in [10, 8] {
        let mut lookup = lazy_cursive();
        lazy_set(&mut lookup.1, nullable_offset, 0); // true absent B entry or A exit.
        let gpos = font::layout_table(b"hwid", &[lookup]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            expect(
                &lazy_shape(font, "AB", &[enabled(feature)]),
                feature,
                &[Outcome::NoAction, Outcome::NoAction],
            );
        });
    }
    let mut lookup = lazy_cursive();
    lazy_set(&mut lookup.1, 28, 99);
    let gpos = font::layout_table(b"hwid", &[lookup]);
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        // R is outside Coverage. A is covered but has a genuine null entry,
        // so neither route demands the bad B entry. No font-wide validation.
        for raw in ["R", "A"] {
            expect(
                &lazy_shape(font, raw, &[enabled(feature)]),
                feature,
                &[Outcome::NoAction],
            );
        }
    });
}

#[test]
fn lazy_cursive_decode_failure_outside_effective_width_scope_does_not_taint_it() {
    let feature = WidthFeature::HalfWidth;
    let mut lookup = lazy_cursive();
    lazy_set(&mut lookup.1, 28, 99);
    let gpos = font::layout_table(b"hwid", core::slice::from_ref(&lookup));
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        for request in [
            Feature::new(feature.tag(), 0, ..),
            Feature::new(feature.tag(), 1, 10..20),
            Feature::new(feature.tag(), 1, 0..1),
        ] {
            // A-only enablement cannot demand B's bad entry: A's own entry is
            // null and B is outside this direct lookup's effective mask.
            expect(
                &lazy_shape(font, "AB", &[request]),
                feature,
                &[Outcome::NoAction, Outcome::NoAction],
            );
        }
    });
    let gpos = font::layout_table(b"zzzz", &[lookup]);
    font::with_font(true, false, None, Some(&gpos), &[], |font| {
        expect(
            &lazy_shape(font, "AB", &[Feature::new(Tag::new(b"zzzz"), 1, ..)]),
            feature,
            &[Outcome::NoAction, Outcome::NoAction],
        );
    });
}

#[test]
fn lazy_anchor_failure_retains_prior_action_history_and_only_qualifies_selected_scope() {
    let feature = WidthFeature::HalfWidth;
    for selected in [false, true] {
        let mut malformed = lazy_cursive();
        lazy_set(&mut malformed.1, 28, 99);
        let lookups = [single_pos(font::A, 4, &[0]), malformed];
        let gpos = if selected {
            font::layout_table(b"hwid", &lookups)
        } else {
            layout(&[(b"hwid", &[0]), (b"zzzz", &[1])], &lookups)
        };
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            let requests = [enabled(feature), Feature::new(Tag::new(b"zzzz"), 1, ..)];
            let result = lazy_shape(font, "AB", &requests);
            if selected {
                lazy_value_limit(&result, &[Outcome::Covered, Outcome::NoAction]);
            } else {
                expect(&result, feature, &[Outcome::Covered, Outcome::NoAction]);
            }
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert!(receipt
                .input(0)
                .unwrap()
                .provenance()
                .iter()
                .any(|entry| entry.source() == FeatureSource::Request(requests[0])));
        });
    }
}

fn lazy_mark(mark_mark: bool) -> ((u16, Vec<u8>), &'static str, &'static str) {
    let (lookup, matched, no_reference) = if mark_mark {
        (
            font::mark(font::MARK, font::MARK2, (500, 700), (10, 20), 6),
            "A\u{300}\u{315}",
            "A\u{315}",
        )
    } else {
        (
            font::mark(font::A, font::MARK, (500, 700), (10, 20), 4),
            "A\u{300}",
            "B\u{300}",
        )
    };
    // MarkBase1/MarkMark1 share this owned fixture layout: Mark(1)Coverage12,
    // Base/Mark2Coverage18, classCount1, Mark(1)Array24, Base/Mark2Array36.
    // MarkAnchor offset6 at28 -> format1 at30; nullable class0 base offset4
    // at38 -> format1 at40. Both actual coverages are preserved below.
    assert_eq!(lazy_word(&lookup.1, 8), 24);
    assert_eq!(lazy_word(&lookup.1, 10), 36);
    assert_eq!(lazy_word(&lookup.1, 28), 6);
    assert_eq!(lazy_word(&lookup.1, 30), 1);
    assert_eq!(lazy_word(&lookup.1, 38), 4);
    assert_eq!(lazy_word(&lookup.1, 40), 1);
    (lookup, matched, no_reference)
}

#[test]
fn lazy_matched_markbase_and_markmark_anchor_or_array_errors_are_unqualified() {
    let feature = WidthFeature::HalfWidth;
    for mark_mark in [false, true] {
        // Nonnull anchor format/offset Err, plus actual lazy array-offset Err.
        // These preserve the mark Coverage used by native cache admission.
        for (at, value) in [
            (30, 99),
            (28, 0xffff),
            (40, 99),
            (38, 0xffff),
            (8, 0xffff),
            (10, 0xffff),
        ] {
            let (mut lookup, raw, _) = lazy_mark(mark_mark);
            lazy_set(&mut lookup.1, at, value);
            let gpos = font::layout_table(b"hwid", &[lookup]);
            font::with_font(true, false, None, Some(&gpos), &[], |font| {
                let result = lazy_shape(font, raw, &[enabled(feature)]);
                lazy_value_limit(&result, &vec![Outcome::NoAction; raw.chars().count()]);
            });
        }
    }
}

#[test]
fn lazy_mark_nullable_absence_unmatched_reference_and_unselected_paths_are_complete() {
    let feature = WidthFeature::HalfWidth;
    for mark_mark in [false, true] {
        let (mut null, raw, _) = lazy_mark(mark_mark);
        // Base/Mark2 class anchors are nullable. The MarkRecord's own anchor
        // is nonnullable, so raw zero there is NOT used as an absence control.
        lazy_set(&mut null.1, 38, 0);
        let gpos = font::layout_table(b"hwid", &[null]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            expect(
                &lazy_shape(font, raw, &[enabled(feature)]),
                feature,
                &vec![Outcome::NoAction; raw.chars().count()],
            );
        });
        let (mut bad, raw, no_reference) = lazy_mark(mark_mark);
        lazy_set(&mut bad.1, 30, 99);
        let gpos = font::layout_table(b"hwid", &[bad.clone()]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            // A genuinely absent matching base/mark2 prevents anchor demand.
            expect(
                &lazy_shape(font, no_reference, &[enabled(feature)]),
                feature,
                &vec![Outcome::NoAction; no_reference.chars().count()],
            );
            for request in [
                Feature::new(feature.tag(), 0, ..),
                // Default MonotoneGraphemes merges A and its marks to cluster0
                // before feature masks. Range1..2 genuinely excludes them.
                Feature::new(feature.tag(), 1, 1..2),
            ] {
                expect(
                    &lazy_shape(font, raw, &[request]),
                    feature,
                    &vec![Outcome::NoAction; raw.chars().count()],
                );
            }
        });
        let gpos = font::layout_table(b"zzzz", &[bad]);
        font::with_font(true, false, None, Some(&gpos), &[], |font| {
            expect(
                &lazy_shape(font, raw, &[Feature::new(Tag::new(b"zzzz"), 1, ..)]),
                feature,
                &vec![Outcome::NoAction; raw.chars().count()],
            );
        });
    }
}

#[test]
fn lazy_mark_cluster_zero_range_enables_real_attachment_and_demands_malformed_anchor() {
    let feature = WidthFeature::HalfWidth;
    let request = Feature::new(feature.tag(), 1, 0..1);
    for mark_mark in [false, true] {
        // Both native request forms enable the same cluster0 operands. An
        // earlier explicit global0 must not erase the later ranged override.
        for requests in [
            vec![request],
            vec![Feature::new(feature.tag(), 0, ..), request],
        ] {
            let (lookup, raw, _) = lazy_mark(mark_mark);
            let roots = raw.chars().count();
            let dependent = roots - 1;
            let reference = usize::from(mark_mark);
            let glyphs = if mark_mark {
                vec![
                    u32::from(font::A),
                    u32::from(font::MARK),
                    u32::from(font::MARK2),
                ]
            } else {
                vec![u32::from(font::A), u32::from(font::MARK)]
            };
            // Input clusters are scalar ordinals, but the natural default cluster
            // policy groups A+grave(+0315) into cluster0 before masks are applied.
            // Ranges address those clusters, while component roots remain distinct.
            let gpos = font::layout_table(b"hwid", core::slice::from_ref(&lookup));
            font::with_font(true, false, None, Some(&gpos), &[], |font| {
                let result = lazy_shape(font, raw, &requests);
                assert_eq!(ids(&result), glyphs);
                assert_eq!(
                    result
                        .glyph_infos()
                        .iter()
                        .map(|glyph| glyph.cluster)
                        .collect::<Vec<_>>(),
                    vec![0; roots]
                );
                // MarkBase changes only grave; MarkMark changes only0315. The
                // base A or preceding grave is a reference, never a width operand.
                let mut outcomes = vec![Outcome::NoAction; roots];
                outcomes[dependent] = Outcome::Covered;
                expect(&result, feature, &outcomes);
                let receipt = result.width_feature_receipt().unwrap().unwrap();
                let evidence = receipt.input(dependent).unwrap();
                let provenance = evidence.provenance();
                assert!(
                    provenance
                        .iter()
                        .any(|entry| entry.source() == FeatureSource::Request(request)),
                    "requests={requests:?}, dependent root={dependent}, provenance={provenance:?}"
                );
                let components = result.component_receipt().unwrap().unwrap();
                let attachment = components.attachments().next().unwrap();
                assert_eq!(
                    attachment.dependent_components(),
                    &[u32::try_from(dependent).unwrap()]
                );
                assert_eq!(
                    attachment.base_components(),
                    &[u32::try_from(reference).unwrap()]
                );
                // Authored reference anchor(500,700) minus dependent anchor(10,20)
                // is(490,680). Restore the pen separation to compare actual physical
                // origins; this works for either reference without assuming where
                // native fallback has positioned the preceding grave itself.
                let positions = result.glyph_positions();
                let between = &positions[reference..dependent];
                assert_eq!(
                    positions[dependent].x_offset - positions[reference].x_offset
                        + between
                            .iter()
                            .map(|position| position.x_advance)
                            .sum::<i32>(),
                    490
                );
                assert_eq!(
                    positions[dependent].y_offset - positions[reference].y_offset
                        + between
                            .iter()
                            .map(|position| position.y_advance)
                            .sum::<i32>(),
                    680
                );
            });
            let mut bad = lookup;
            lazy_set(&mut bad.1, 30, 99); // Same demanded nonnull mark anchor, bad format.
            let gpos = font::layout_table(b"hwid", &[bad]);
            font::with_font(true, false, None, Some(&gpos), &[], |font| {
                let result = lazy_shape(font, raw, &requests);
                assert_eq!(ids(&result), glyphs);
                assert_eq!(
                    result
                        .glyph_infos()
                        .iter()
                        .map(|glyph| glyph.cluster)
                        .collect::<Vec<_>>(),
                    vec![0; roots]
                );
                lazy_value_limit(&result, &vec![Outcome::NoAction; roots]);
            });
        }
    }
}
