//! Width evidence requires qualified removal of actual surviving ancestry,
//! including removals performed by other native features. These tests preserve
//! tolerant native shaping while distinguishing valid empty from failed data.
use super::*;

fn ancestry_flags(buffer: &GlyphBuffer) -> Vec<(bool, bool, bool)> {
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

fn ancestry_shape(font: &crate::Shaper<'_>, raw: &str, requests: &[Feature]) -> GlyphBuffer {
    let make = || {
        let mut value = input(raw, Direction::LeftToRight);
        value.set_flags(
            BufferFlags::PRODUCE_UNSAFE_TO_CONCAT | BufferFlags::PRODUCE_SAFE_TO_INSERT_TATWEEL,
        );
        value
    };
    let plain = make();
    let plan = make_plan(font, &plain, requests);
    // Keep the actual Shaper/FontRef/cache and plan across observation off->on.
    let ordinary = font.shape_with_plan(&plan, plain, requests);
    assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
    assert!(ordinary.width_feature_receipt().unwrap().is_none());
    let mut marked = make();
    marked.observe_width_feature(WidthFeature::HalfWidth);
    let result = font.shape_with_plan(&plan, marked, requests);
    assert_eq!(result.shaping_status(), ShapingStatus::Complete);
    assert_eq!(ids(&result), ids(&ordinary));
    assert_eq!(old_positions(&result), old_positions(&ordinary));
    assert_eq!(ancestry_flags(&result), ancestry_flags(&ordinary));
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

fn ancestry_limit(buffer: &GlyphBuffer, inputs: usize) {
    let receipt = buffer.width_feature_receipt().unwrap().unwrap();
    assert_eq!(receipt.feature(), WidthFeature::HalfWidth);
    assert_eq!(receipt.input_len(), inputs);
    assert_eq!(
        receipt.coverage(),
        Coverage::Unqualified(WidthObservationPath::LookupAdmission)
    );
    for root in 0..inputs {
        assert!(receipt.input(root).is_some());
    }
    assert!(receipt.input(inputs).is_none());
}

#[test]
fn non_width_ccmp_malformed_removal_cannot_certify_no_surviving_operand() {
    let feature = WidthFeature::HalfWidth;
    for declared in [0u16, 0xffff] {
        let mut deletion = multiple(font::A, &[]);
        assert_eq!(deletion.1.len(), 16);
        // Multiple1 Sequence at8, glyphCount at8, IDs starting10. Its real
        // Coverage remains at10. Count0 is a native supported empty deletion;
        // 65535 requests131070 bytes, recovers an empty view and also deletes A.
        deletion.1[8..10].copy_from_slice(&declared.to_be_bytes());
        let gsub = font::layout_table(b"ccmp", &[deletion.clone()]);
        assert!(gsub.len() < 131_070);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            // ccmp is an actual default required feature. No hwid lookup or
            // request is present: the failed removal alone affects ancestry.
            let result = ancestry_shape(font, "A", &[]);
            assert!(result.glyph_infos().is_empty());
            assert!(result.glyph_positions().is_empty());
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 1);
            assert_eq!(components.deleted_input_positions(), &[0]);
            assert_eq!(components.components(0), None);
            if declared == 0 {
                expect(&result, feature, &[Outcome::NoSurvivingOperand]);
            } else {
                // Any local NoSurvivingOperand is diagnostic. An unreadable
                // native removal cannot supply Complete width evidence.
                ancestry_limit(&result, 1);
            }
            let unmatched = ancestry_shape(font, "B", &[]);
            assert_eq!(ids(&unmatched), vec![u32::from(font::B)]);
            assert_eq!(old_positions(&unmatched), vec![(1_000, 0, 0, 0)]);
            assert_eq!(unmatched.glyph_infos()[0].cluster, 0);
            expect(&unmatched, feature, &[Outcome::NoAction]);
            expect(&ancestry_shape(font, "", &[]), feature, &[]);
        });
        // A real context references child1, but its required B is unavailable.
        // The malformed child is neither selected directly nor actually run.
        let gsub = layout(&[(b"ccmp", &[0])], &[context(Some(1)), deletion]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = ancestry_shape(font, "A", &[]);
            assert_eq!(ids(&result), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&result), vec![(1_000, 0, 0, 0)]);
            expect(&result, feature, &[Outcome::NoAction]);
        });
    }
}

#[test]
fn non_width_malformed_sibling_removal_cannot_upgrade_partial_width_history_to_complete_covered() {
    let feature = WidthFeature::HalfWidth;
    let hwid = enabled(feature);
    let zzzz = Feature::new(Tag::new(b"zzzz"), 1, ..);
    for declared in [0u16, 0xffff] {
        let mut deletion = multiple(font::R, &[]);
        deletion.1[8..10].copy_from_slice(&declared.to_be_bytes());
        // Actual Latin GSUB uses one stage for these features and sorts lookup
        // indices. ccmp0 splits A into B/R, hwid1 acts identically on B only,
        // then explicitly requested zzzz2 removes the still-uncovered R branch.
        let gsub = layout(
            &[(b"ccmp", &[0]), (b"hwid", &[1]), (b"zzzz", &[2])],
            &[
                multiple(font::A, &[font::B, font::R]),
                single_subst(font::B, font::B),
                deletion,
            ],
        );
        assert!(gsub.len() < 131_070);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            // Without the explicit zzzz request both actual descendants remain.
            // One acted branch and one unacted branch are Complete Partial.
            let before_removal = ancestry_shape(font, "A", &[hwid]);
            assert_eq!(
                ids(&before_removal),
                vec![u32::from(font::B), u32::from(font::R)]
            );
            assert_eq!(
                old_positions(&before_removal),
                vec![(1_000, 0, 0, 0), (1_201, 0, 0, 0)]
            );
            assert_eq!(
                before_removal
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>(),
                vec![0, 0]
            );
            expect(&before_removal, feature, &[Outcome::Partial]);
            let components = before_removal.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.components(1), Some(&[0][..]));
            assert!(components.deleted_input_positions().is_empty());

            // Both count0 and unreadable declared-nonempty views reach native
            // empty deletion. B remains with its original hmtx1000, cluster0
            // and source root0. Their qualification must differ.
            let result = ancestry_shape(font, "A", &[hwid, zzzz]);
            assert_eq!(ids(&result), vec![u32::from(font::B)]);
            assert_eq!(old_positions(&result), vec![(1_000, 0, 0, 0)]);
            assert_eq!(result.glyph_infos()[0].cluster, 0);
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 1);
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.components(1), None);
            assert!(components.deleted_input_positions().is_empty());
            if declared == 0 {
                expect(&result, feature, &[Outcome::Covered]);
            } else {
                ancestry_limit(&result, 1);
                // The local outcome may remain Covered for diagnosis, but the
                // earlier genuine B action and its request history must remain.
                let receipt = result.width_feature_receipt().unwrap().unwrap();
                let evidence = receipt.input(0).unwrap();
                let provenance = evidence.provenance();
                assert!(
                    provenance
                        .iter()
                        .any(|entry| entry.source() == FeatureSource::Request(hwid)),
                    "prior hwid B action disappeared: provenance={provenance:?}"
                );
            }
        });
    }
}

#[test]
fn non_width_child_removal_requires_qualified_enabling_class_route_but_child_miss_does_not() {
    let feature = WidthFeature::HalfWidth;
    for class_format in [2, 99] {
        // Context2 header10; RuleSet10 points to Rule14: two inputs, second
        // class0, child1 launched on the second input. Coverage24 admits A.
        // ClassDef30 is either genuine Format2/count0 (all glyphs class0), or
        // invalid Format99, which native cached metadata recovers to class0.
        // Only parent0 is registered under ccmp; valid child1 deletes B.
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
        let gsub = layout(&[(b"ccmp", &[0])], &[parent, multiple(font::B, &[])]);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = ancestry_shape(font, "AB", &[]);
            assert_eq!(ids(&result), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&result), vec![(1_000, 0, 0, 0)]);
            assert_eq!(result.glyph_infos()[0].cluster, 0);
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 2);
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.deleted_input_positions(), &[1]);
            if class_format == 2 {
                expect(
                    &result,
                    feature,
                    &[Outcome::NoAction, Outcome::NoSurvivingOperand],
                );
            } else {
                // The deleting Sequence is valid; the actual enabling class
                // route is not. Complete cannot certify exclusion of root1.
                ancestry_limit(&result, 2);
            }
            // With AR the same recovered class0 route can launch child1, but
            // child Coverage(B) genuinely misses R. No descendant is removed:
            // an unrelated malformed class read alone is not width demand.
            let missed_child = ancestry_shape(font, "AR", &[]);
            assert_eq!(
                ids(&missed_child),
                vec![u32::from(font::A), u32::from(font::R)]
            );
            assert_eq!(
                old_positions(&missed_child),
                vec![(1_000, 0, 0, 0), (1_201, 0, 0, 0)]
            );
            assert_eq!(
                missed_child
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            expect(
                &missed_child,
                feature,
                &[Outcome::NoAction, Outcome::NoAction],
            );
            let components = missed_child.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.components(1), Some(&[1][..]));
            assert!(components.deleted_input_positions().is_empty());
            let missed_parent = ancestry_shape(font, "B", &[]);
            assert_eq!(ids(&missed_parent), vec![u32::from(font::B)]);
            expect(&missed_parent, feature, &[Outcome::NoAction]);
        });
    }
}

#[test]
fn malformed_class_enabled_rewrite_remains_a_dependency_of_later_clean_removal() {
    let feature = WidthFeature::HalfWidth;
    let zzzz = Feature::new(Tag::new(b"zzzz"), 1, ..);
    for class_format in [2, 99] {
        // Same genuine Context2 class0 rule as the direct-deletion case, but
        // child1 now makes the valid B->R rewrite. Only parent0 is selected by
        // default ccmp. Later requested zzzz2 has a genuine empty R Sequence.
        // Latin's shared stage sorts indices0/2; child1 runs only on recursion.
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
            &[(b"ccmp", &[0]), (b"zzzz", &[2])],
            &[
                parent,
                single_subst(font::B, font::R),
                multiple(font::R, &[]),
            ],
        );
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            // The rewritten R is a real descendant of original B/root1.
            // Without zzzz, no ancestry is excluded and no hwid action occurs.
            let rewritten = ancestry_shape(font, "AB", &[]);
            assert_eq!(
                ids(&rewritten),
                vec![u32::from(font::A), u32::from(font::R)]
            );
            assert_eq!(
                old_positions(&rewritten),
                vec![(1_000, 0, 0, 0), (1_201, 0, 0, 0)]
            );
            assert_eq!(
                rewritten
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            expect(&rewritten, feature, &[Outcome::NoAction, Outcome::NoAction]);
            let components = rewritten.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.components(1), Some(&[1][..]));
            assert!(components.deleted_input_positions().is_empty());

            // A later clean lookup removes R only because the earlier child
            // rewrote B. That real enabling dependency crosses lookup scopes.
            let removed = ancestry_shape(font, "AB", &[zzzz]);
            assert_eq!(ids(&removed), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&removed), vec![(1_000, 0, 0, 0)]);
            assert_eq!(removed.glyph_infos()[0].cluster, 0);
            let components = removed.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 2);
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.deleted_input_positions(), &[1]);
            if class_format == 2 {
                expect(
                    &removed,
                    feature,
                    &[Outcome::NoAction, Outcome::NoSurvivingOperand],
                );
            } else {
                ancestry_limit(&removed, 2);
            }
            // If R was already the source glyph, child Coverage(B) misses it.
            // Its clean later deletion does not depend on the failed class
            // route: visiting unrelated bad data must not poison this removal.
            let independent = ancestry_shape(font, "AR", &[zzzz]);
            assert_eq!(ids(&independent), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&independent), vec![(1_000, 0, 0, 0)]);
            expect(
                &independent,
                feature,
                &[Outcome::NoAction, Outcome::NoSurvivingOperand],
            );
            let components = independent.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.deleted_input_positions(), &[1]);
        });
    }
}

#[test]
fn non_width_deleting_sibling_retains_cached_lookup_rejection_but_nonmutation_is_complete() {
    let feature = WidthFeature::HalfWidth;
    for rejected in [false, true] {
        // Multiple1 first sibling has seqCount0 and Coverage1/count0 at6:
        // true admitted empty coverage. Format99 instead rejects this sibling
        // at actual cache admission. The second sibling validly deletes A.
        let mut empty = words(&[1, 6, 0, 1, 0]);
        if rejected {
            empty[6..8].copy_from_slice(&99u16.to_be_bytes());
        }
        let (_, deletion) = multiple(font::A, &[]);
        let mut gsub = font::layout_table(b"ccmp", &[(2, vec![])]);
        let list = usize::from(u16::from_be_bytes(gsub[8..10].try_into().unwrap()));
        let lookup = list
            + usize::from(u16::from_be_bytes(
                gsub[list + 2..list + 4].try_into().unwrap(),
            ));
        gsub.truncate(lookup);
        // One actual type2 Lookup, flags0, two relative subtable offsets.
        // Reuse the owned layout encoder and author only its raw Lookup body.
        gsub.extend(words(&[
            2,
            0,
            2,
            10,
            u16::try_from(10 + empty.len()).unwrap(),
        ]));
        gsub.extend(empty);
        gsub.extend(deletion);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            // ancestry_shape first executes this same cached font/plan with
            // observation off. The second admitted sibling still deletes A.
            let result = ancestry_shape(font, "A", &[]);
            assert!(result.glyph_infos().is_empty());
            assert!(result.glyph_positions().is_empty());
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 1);
            assert_eq!(components.deleted_input_positions(), &[0]);
            if rejected {
                ancestry_limit(&result, 1);
            } else {
                expect(&result, feature, &[Outcome::NoSurvivingOperand]);
            }
            // All admitted Coverage facts miss B and native makes no change.
            // This non-width lookup's rejected content cannot be a font-wide
            // poison when it contributes no mutation/removal of this root.
            let missed = ancestry_shape(font, "B", &[]);
            assert_eq!(ids(&missed), vec![u32::from(font::B)]);
            assert_eq!(old_positions(&missed), vec![(1_000, 0, 0, 0)]);
            assert_eq!(missed.glyph_infos()[0].cluster, 0);
            expect(&missed, feature, &[Outcome::NoAction]);
            let components = missed.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert!(components.deleted_input_positions().is_empty());
        });
    }
}

#[test]
fn first_clean_deleting_subtable_does_not_inherit_unreached_later_rejection() {
    let feature = WidthFeature::HalfWidth;
    for rejected in [false, true] {
        let (_, deletion) = multiple(font::A, &[]);
        let mut later = words(&[1, 6, 0, 1, 0]); // true empty Coverage1 at6.
        if rejected {
            later[6..8].copy_from_slice(&99u16.to_be_bytes());
        }
        let mut gsub = font::layout_table(b"ccmp", &[(2, vec![])]);
        let list = usize::from(u16::from_be_bytes(gsub[8..10].try_into().unwrap()));
        let lookup = list
            + usize::from(u16::from_be_bytes(
                gsub[list + 2..list + 4].try_into().unwrap(),
            ));
        gsub.truncate(lookup);
        // Reverse the earlier witness's actual subtable order: the valid
        // deleting sibling is first. Native stops on its successful deletion;
        // later rejected content cannot have enabled or preceded that action.
        gsub.extend(words(&[
            2,
            0,
            2,
            10,
            u16::try_from(10 + deletion.len()).unwrap(),
        ]));
        gsub.extend(deletion);
        gsub.extend(later);
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = ancestry_shape(font, "A", &[]);
            assert!(result.glyph_infos().is_empty());
            assert!(result.glyph_positions().is_empty());
            expect(&result, feature, &[Outcome::NoSurvivingOperand]);
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 1);
            assert_eq!(components.deleted_input_positions(), &[0]);
            let missed = ancestry_shape(font, "B", &[]);
            assert_eq!(ids(&missed), vec![u32::from(font::B)]);
            assert_eq!(old_positions(&missed), vec![(1_000, 0, 0, 0)]);
            assert_eq!(missed.glyph_infos()[0].cluster, 0);
            expect(&missed, feature, &[Outcome::NoAction]);
        });
    }
}

#[test]
fn failed_nonmutating_class_rule_does_not_taint_independent_later_clean_deletion() {
    let feature = WidthFeature::HalfWidth;
    let zzzz = Feature::new(Tag::new(b"zzzz"), 1, ..);
    for class_format in [2, 99] {
        // Actual ClassDef2/count0, or failed Format99 recovered to class0.
        // Unlike the enabling route, this Rule14 requires SECOND input class1
        // (word9/byte18). Actual B is class0, so no child lookup is launched.
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
                1,
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
            &[(b"ccmp", &[0]), (b"zzzz", &[2])],
            &[
                parent,
                single_subst(font::B, font::R),
                multiple(font::B, &[]),
            ],
        );
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            // This native packet proves B was never rewritten by child1,
            // despite visiting the malformed/recovered class route first.
            let untouched = ancestry_shape(font, "AB", &[]);
            assert_eq!(
                ids(&untouched),
                vec![u32::from(font::A), u32::from(font::B)]
            );
            assert_eq!(
                old_positions(&untouched),
                vec![(1_000, 0, 0, 0), (1_000, 0, 0, 0)]
            );
            assert_eq!(
                untouched
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            expect(&untouched, feature, &[Outcome::NoAction, Outcome::NoAction]);
            // zzzz2 cleanly deletes the original B, independently of the failed
            // earlier rule. A nonmutating read is not a sticky dependency.
            let result = ancestry_shape(font, "AB", &[zzzz]);
            assert_eq!(ids(&result), vec![u32::from(font::A)]);
            assert_eq!(old_positions(&result), vec![(1_000, 0, 0, 0)]);
            assert_eq!(result.glyph_infos()[0].cluster, 0);
            expect(
                &result,
                feature,
                &[Outcome::NoAction, Outcome::NoSurvivingOperand],
            );
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 2);
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.deleted_input_positions(), &[1]);
        });
    }
}

fn contextual_observer_removal_lane(lane: u8) {
    let feature = WidthFeature::HalfWidth;
    // The observer is first input, second input, passive lookahead, or actual
    // backtrack. Nonfirst/lookahead have a clean current-entry A and child A;
    // backtrack has a clean current-entry B and child B, with R in OutInfo.
    for declared in [1u16, 0xffff] {
        let (from, target, observer) = if lane == 0 || lane == 3 {
            (font::A, font::B, 0usize)
        } else {
            (font::B, font::A, 1usize)
        };
        let deleted = 1 - observer;
        let mut rewrite = ligature(from, target, font::R);
        // Ligature starts12, componentCount at14 includes its first glyph.
        // One declares no following IDs and validly replaces only `from`.
        // 65535 cannot fit, recovers an empty ID view, and takes the SAME
        // native in-place shortcut. The produced R retains its source root.
        rewrite.1[14..16].copy_from_slice(&declared.to_be_bytes());
        let parent = match lane {
            0 => {
                let mut parent = context(Some(2));
                // Context3 primary Coverage14 contains its glyph at18:
                // R/B, SequenceLookupRecord(input1,child2) deletes B.
                parent.1[18..20].copy_from_slice(&font::R.to_be_bytes());
                parent
            }
            1 => {
                let mut parent = context(Some(2));
                // A/R: nonfirst Coverage20 contains its glyph at24.
                // Select input0 instead, deleting clean A, never observer R.
                parent.1[24..26].copy_from_slice(&font::R.to_be_bytes());
                parent.1[10..12].copy_from_slice(&0u16.to_be_bytes());
                parent
            }
            2 => {
                // ChainContext3: backtrack0, input1/Coverage18(A),
                // lookahead1/Coverage24(R), record(input0,child2).
                // R is not an input operand or a child target at all.
                (
                    6,
                    words(&[3, 0, 1, 18, 1, 24, 1, 0, 2, 1, 1, font::A, 1, 1, font::R]),
                )
            }
            _ => {
                // ChainContext3: backtrack1/Coverage18(R), input1/
                // Coverage24(B), lookahead0, record(input0,child2).
                // Forward lookup1 already emitted R while seeking B;
                // match_backtrack's prev() reads that real OutInfo glyph.
                (
                    6,
                    words(&[3, 1, 18, 1, 24, 0, 1, 0, 2, 1, 1, font::R, 1, 1, font::B]),
                )
            }
        };
        // Native shared-stage lookup0 rewrites, lookup1 matches context,
        // and only its actual recursion invokes valid deleting child2.
        let gsub = layout(
            &[(b"ccmp", &[0, 1])],
            &[rewrite, parent, multiple(target, &[])],
        );
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let result = ancestry_shape(font, "AB", &[]);
            // AB contains no R; only lookup0 can have produced this real
            // surviving glyph. Deleting the first cluster merges it forward
            // to0; deleting the second leaves the observer's cluster0.
            assert_eq!(ids(&result), vec![u32::from(font::R)]);
            assert_eq!(old_positions(&result), vec![(1_201, 0, 0, 0)]);
            assert_eq!(result.glyph_infos()[0].cluster, 0);
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 2);
            assert_eq!(
                components.components(0),
                Some(&[u32::try_from(observer).unwrap()][..])
            );
            assert_eq!(
                components.deleted_input_positions(),
                &[u32::try_from(deleted).unwrap()]
            );
            if declared == 1 {
                let mut outcomes = vec![Outcome::NoAction; 2];
                outcomes[deleted] = Outcome::NoSurvivingOperand;
                expect(&result, feature, &outcomes);
            } else {
                ancestry_limit(&result, 2);
            }
            // Matching dependencies are separate from source membership
            // and actual selected-width action history. Neither observer
            // nor deleted target acquired any hwid action in this font.
            let receipt = result.width_feature_receipt().unwrap().unwrap();
            assert_eq!(
                receipt.input(observer).unwrap().outcome(),
                Outcome::NoAction
            );
            for root in 0..2 {
                assert!(receipt.input(root).unwrap().provenance().is_empty());
            }

            // Actual malformed rewrite, but no available matching child:
            // Lone A becomes R with no child B in first/backtrack lanes;
            // lone B becomes R and misses primary A otherwise. No removal.
            let alone = ancestry_shape(font, if observer == 0 { "A" } else { "B" }, &[]);
            assert_eq!(ids(&alone), vec![u32::from(font::R)]);
            assert_eq!(old_positions(&alone), vec![(1_201, 0, 0, 0)]);
            expect(&alone, feature, &[Outcome::NoAction]);
            let components = alone.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert!(components.deleted_input_positions().is_empty());

            // An authored R makes lookup0 genuinely miss its primary
            // Coverage. The clean contextual deletion is independent of
            // that unconsumed malformed view, with the same final packet.
            let genuine = ancestry_shape(font, if observer == 0 { "RB" } else { "AR" }, &[]);
            assert_eq!(ids(&genuine), vec![u32::from(font::R)]);
            assert_eq!(old_positions(&genuine), vec![(1_201, 0, 0, 0)]);
            assert_eq!(genuine.glyph_infos()[0].cluster, 0);
            let mut outcomes = vec![Outcome::NoAction; 2];
            outcomes[deleted] = Outcome::NoSurvivingOperand;
            expect(&genuine, feature, &outcomes);
            let components = genuine.component_receipt().unwrap().unwrap();
            assert_eq!(
                components.components(0),
                Some(&[u32::try_from(observer).unwrap()][..])
            );
            assert_eq!(
                components.deleted_input_positions(),
                &[u32::try_from(deleted).unwrap()]
            );
        });
    }
}

#[test]
fn malformed_rewrite_of_contextual_observer_primary_limits_other_root_removal() {
    contextual_observer_removal_lane(0);
}

#[test]
fn malformed_rewrite_of_contextual_observer_nonfirst_input_limits_other_root_removal() {
    contextual_observer_removal_lane(1);
}

#[test]
fn malformed_rewrite_of_contextual_observer_lookahead_limits_other_root_removal() {
    contextual_observer_removal_lane(2);
}

#[test]
fn malformed_rewrite_of_contextual_observer_backtrack_outinfo_limits_other_root_removal() {
    contextual_observer_removal_lane(3);
}

#[test]
fn reverse_chain_observer_dependency_survives_clean_target_rewrite_until_later_removal() {
    let feature = WidthFeature::HalfWidth;
    let zzzz = Feature::new(Tag::new(b"zzzz"), 1, ..);
    for declared in [1u16, 0xffff] {
        let mut observer = ligature(font::B, font::A, font::R);
        observer.1[14..16].copy_from_slice(&declared.to_be_bytes());
        // ReverseChain1: primary Coverage14(A), no backtrack, lookahead1/
        // Coverage20(R), one valid substitute B. This is top-level lookup1,
        // so native reverse chaining is admitted (not nested in a context).
        let reverse = (
            8,
            words(&[1, 14, 0, 1, 20, 1, font::B, 1, 1, font::A, 1, 1, font::R]),
        );
        // Lookup0 B->R creates the observer. Reverse lookup1 uses it to rewrite
        // clean A/root0 to B. Only later requested zzzz2 removes that target.
        let gsub = layout(
            &[(b"ccmp", &[0, 1]), (b"zzzz", &[2])],
            &[observer, reverse, multiple(font::B, &[])],
        );
        font::with_font(true, false, Some(&gsub), None, &[], |font| {
            let rewritten = ancestry_shape(font, "AB", &[]);
            assert_eq!(
                ids(&rewritten),
                vec![u32::from(font::B), u32::from(font::R)]
            );
            assert_eq!(
                old_positions(&rewritten),
                vec![(1_000, 0, 0, 0), (1_201, 0, 0, 0)]
            );
            assert_eq!(
                rewritten
                    .glyph_infos()
                    .iter()
                    .map(|glyph| glyph.cluster)
                    .collect::<Vec<_>>(),
                vec![0, 1]
            );
            expect(&rewritten, feature, &[Outcome::NoAction, Outcome::NoAction]);
            let components = rewritten.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[0][..]));
            assert_eq!(components.components(1), Some(&[1][..]));
            assert!(components.deleted_input_positions().is_empty());

            let removed = ancestry_shape(font, "AB", &[zzzz]);
            // Clean B deletion merges its first cluster forward. The actual
            // surviving R remains ONLY observer root1, never the removed root0.
            assert_eq!(ids(&removed), vec![u32::from(font::R)]);
            assert_eq!(old_positions(&removed), vec![(1_201, 0, 0, 0)]);
            assert_eq!(removed.glyph_infos()[0].cluster, 0);
            let components = removed.component_receipt().unwrap().unwrap();
            assert_eq!(components.input_len(), 2);
            assert_eq!(components.components(0), Some(&[1][..]));
            assert_eq!(components.deleted_input_positions(), &[0]);
            if declared == 1 {
                expect(
                    &removed,
                    feature,
                    &[Outcome::NoSurvivingOperand, Outcome::NoAction],
                );
            } else {
                ancestry_limit(&removed, 2);
            }
            let receipt = removed.width_feature_receipt().unwrap().unwrap();
            assert_eq!(receipt.input(1).unwrap().outcome(), Outcome::NoAction);
            for root in 0..2 {
                assert!(receipt.input(root).unwrap().provenance().is_empty());
            }
            // Authored R makes malformed lookup0 genuinely miss. Reverse1 can
            // still enable the same clean deletion, now without bad ancestry.
            let genuine = ancestry_shape(font, "AR", &[zzzz]);
            assert_eq!(ids(&genuine), vec![u32::from(font::R)]);
            assert_eq!(old_positions(&genuine), vec![(1_201, 0, 0, 0)]);
            assert_eq!(genuine.glyph_infos()[0].cluster, 0);
            expect(
                &genuine,
                feature,
                &[Outcome::NoSurvivingOperand, Outcome::NoAction],
            );
            let components = genuine.component_receipt().unwrap().unwrap();
            assert_eq!(components.components(0), Some(&[1][..]));
            assert_eq!(components.deleted_input_positions(), &[0]);
        });
    }
}
