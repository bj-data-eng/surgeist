//! Existing-callable RED for supplied backward vertical positioning and native
//! mirroring fallback. Expected glyphs and packets come from authored SFNT
//! operands, not a second implementation of mirroring or observed output.
use super::{font, old_positions, positions, qualify};
use crate::hb::completion_fixture::words;
use crate::{
    script, ComponentCoverage, Direction, Feature, GlyphBuffer, ShapePlan, Shaper, ShapingRefusal,
    ShapingStatus, Tag, UnicodeBuffer, VerticalGlyphBuffer, VerticalNominalMetrics,
    VerticalOriginReference, VerticalShapeContextError, VerticalShapeError,
};
use alloc::{vec, vec::Vec};
use core::convert::Infallible;
use read_fonts::TableProvider;

const LEFT: char = '\u{3008}';
const RIGHT: char = '\u{3009}';
const CLUSTERS: [u32; 2] = [17, 901];

fn word(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn cmap(entries: &[(u32, u16)]) -> Vec<u8> {
    let mut entries = entries.to_vec();
    entries.sort_unstable();
    assert!(entries.windows(2).all(|pair| pair[0].0 != pair[1].0));
    let mut result = words(&[0, 1, 3, 10]);
    result.extend(12_u32.to_be_bytes());
    result.extend(words(&[12, 0]));
    result.extend(
        u32::try_from(16 + 12 * entries.len())
            .unwrap()
            .to_be_bytes(),
    );
    result.extend(0_u32.to_be_bytes());
    result.extend(u32::try_from(entries.len()).unwrap().to_be_bytes());
    for (scalar, glyph) in entries {
        result.extend(scalar.to_be_bytes());
        result.extend(scalar.to_be_bytes());
        result.extend(u32::from(glyph).to_be_bytes());
    }
    result
}

fn mappings(mirrored: bool) -> Vec<(u32, u16)> {
    let mut entries = vec![
        (32, font::SPACE),
        (46, font::DOT),
        (65, font::A),
        (66, font::B),
        (82, font::R),
        (u32::from(LEFT), font::A),
    ];
    if mirrored {
        entries.push((u32::from(RIGHT), font::B));
    }
    entries
}

fn with_mirror_font(mirrored: bool, f: impl FnOnce(&Shaper<'_>)) {
    let cmap = cmap(&mappings(mirrored));
    let gsub = font::layout_table(b"rtlm", &[font::replace()]);
    // Independently read the actual lookup's coverage and replacement operand.
    let list = usize::from(word(&gsub, 8));
    assert_eq!(word(&gsub, list), 1);
    let lookup = list + usize::from(word(&gsub, list + 2));
    assert_eq!((word(&gsub, lookup), word(&gsub, lookup + 4)), (1, 1));
    let sub = lookup + usize::from(word(&gsub, lookup + 6));
    assert_eq!(
        (word(&gsub, sub), word(&gsub, sub + 4), word(&gsub, sub + 6)),
        (2, 1, 58)
    );
    let coverage = sub + usize::from(word(&gsub, sub + 2));
    assert_eq!(
        (
            word(&gsub, coverage),
            word(&gsub, coverage + 2),
            word(&gsub, coverage + 4)
        ),
        (1, 1, 35)
    );
    font::with_cmap_font(true, true, Some(&gsub), None, &[], &cmap, |font| {
        qualify(font, true, true);
        assert_eq!(
            font.get_nominal_glyph(u32::from(LEFT)).unwrap().to_u32(),
            35
        );
        assert_eq!(
            font.get_nominal_glyph(u32::from(RIGHT)).map(|g| g.to_u32()),
            mirrored.then_some(36)
        );
        // Unicode17 BidiMirroring.txt supplies U3008 <-> U3009; both are
        // VerticalOrientation=Tr. The fixture maps neither vertical form, so
        // an unrelated vertical presentation fallback cannot select a glyph.
        assert!(font.get_nominal_glyph(0xfe3f).is_none());
        assert!(font.get_nominal_glyph(0xfe40).is_none());
        let gsub = font.font.gsub().unwrap();
        let features = gsub.feature_list().unwrap();
        assert_eq!(features.feature_count(), 1);
        assert_eq!(
            features.feature_records()[0].feature_tag().into_bytes(),
            *b"rtlm"
        );
        let list = gsub.lookup_list().unwrap();
        assert_eq!(list.lookup_count(), 1);
        assert!(matches!(
            list.lookups().get(0).unwrap(),
            read_fonts::tables::gsub::SubstitutionLookup::Single(_)
        ));
        for (glyph, expected) in [
            (35, (1200, 1001, 900)),
            (36, (1400, 1000, 1000)),
            (58, (1600, 1201, 1000)),
            (16, (701, 500, 900)),
        ] {
            let metrics = font::metrics(font, glyph);
            assert_eq!(
                (
                    metrics.advance(),
                    metrics.origin_x_twice(),
                    metrics.origin_y()
                ),
                expected
            );
        }
        f(font);
    });
}

fn input(direction: Direction) -> UnicodeBuffer {
    let mut input = UnicodeBuffer::new();
    input.observe_components();
    input.add(LEFT, CLUSTERS[0]);
    input.add('.', CLUSTERS[1]);
    input.set_direction(direction);
    input.set_script(script::LATIN);
    input
}

fn ordinary(font: &Shaper<'_>, direction: Direction) -> GlyphBuffer {
    font.shape(input(direction), &[])
}

fn ids(buffer: &GlyphBuffer) -> Vec<(u32, u32)> {
    buffer
        .glyph_infos()
        .iter()
        .map(|g| (g.glyph_id, g.cluster))
        .collect()
}

#[test]
fn mirrored_cmap_backward_supplied_metrics_use_actual_glyph_and_original_input_ordinals() {
    with_mirror_font(true, |font| {
        let ttb = ordinary(font, Direction::TopToBottom);
        assert_eq!(ids(&ttb), [(35, 17), (16, 901)]);
        assert_eq!(
            old_positions(&ttb),
            [(0, -1200, -500, -900), (0, -701, -250, -900)]
        );
        let btt = ordinary(font, Direction::BottomToTop);
        assert_eq!(ids(&btt), [(16, 901), (36, 17)]);
        assert_eq!(
            old_positions(&btt),
            [(0, -701, -250, -900), (0, -1400, -500, -1000)]
        );
        let rtl = ordinary(font, Direction::RightToLeft);
        assert_eq!(ids(&rtl), [(16, 901), (36, 17)]);
        assert_eq!(old_positions(&rtl), [(500, 0, 0, 0), (1000, 0, 0, 0)]);
        let source = input(Direction::BottomToTop);
        let plan = ShapePlan::new(font, Direction::BottomToTop, Some(script::LATIN), None, &[]);
        let mut queried = Vec::new();
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |glyph| {
                    queried.push(glyph);
                    Ok::<_, Infallible>(font::metrics(font, glyph))
                },
            )
            .expect("BTT must use the existing native mirroring and supplied-metric operation");
        assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        assert_eq!(
            result
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(16, 901), (36, 17)]
        );
        assert_eq!(
            positions(&result),
            [
                (0.0, -701.0, -250.0, -900.0),
                (0.0, -1400.0, -500.0, -1000.0)
            ]
        );
        queried.sort_unstable();
        queried.dedup();
        assert_eq!(queried, [16, 36]);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.input_len(), 2);
        assert_eq!(receipt.components(0), Some(&[1][..]));
        assert_eq!(receipt.components(1), Some(&[0][..]));
        assert!(receipt.deleted_input_positions().is_empty());
        let di = result.default_ignorable_receipt().unwrap();
        assert!(di.residual_input_positions().is_empty());
        assert!(di.deleted_input_positions().is_empty());
        assert!(di.residual_glyph_positions().is_empty());
    });
}

#[test]
fn default_backward_rtlm_fallback_is_selected_before_supplied_used_glyph_metrics() {
    with_mirror_font(false, |font| {
        let ttb = ordinary(font, Direction::TopToBottom);
        assert_eq!(ids(&ttb), [(35, 17), (16, 901)]);
        assert_eq!(
            old_positions(&ttb),
            [(0, -1200, -500, -900), (0, -701, -250, -900)]
        );
        let rtl = ordinary(font, Direction::RightToLeft);
        assert_eq!(ids(&rtl), [(16, 901), (58, 17)]);
        assert_eq!(old_positions(&rtl), [(500, 0, 0, 0), (1201, 0, 0, 0)]);
        // This assertion isolates native plan fallback independently of the
        // supplied operation's earlier BTT admission check.
        let btt = ordinary(font, Direction::BottomToTop);
        assert_eq!(
            ids(&btt),
            [(16, 901), (58, 17)],
            "an unavailable Unicode mirror must retain native rtlm fallback"
        );
        assert_eq!(
            old_positions(&btt),
            [(0, -701, -250, -900), (0, -1600, -600, -1000)]
        );
        let source = input(Direction::BottomToTop);
        let plan = ShapePlan::new(font, Direction::BottomToTop, Some(script::LATIN), None, &[]);
        let mut queried = Vec::new();
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |glyph| {
                    queried.push(glyph);
                    Ok::<_, Infallible>(font::metrics(font, glyph))
                },
            )
            .expect("BTT fallback must query the actual substituted glyph");
        assert_eq!(
            result
                .glyph_infos()
                .iter()
                .map(|g| (g.glyph_id, g.cluster))
                .collect::<Vec<_>>(),
            [(16, 901), (58, 17)]
        );
        assert_eq!(
            positions(&result),
            [
                (0.0, -701.0, -250.0, -900.0),
                (0.0, -1600.0, -600.5, -1000.0)
            ]
        );
        queried.sort_unstable();
        queried.dedup();
        assert_eq!(queried, [16, 58]);
    });
}

// Additional functional controls were authored after the genuine RED checkpoint.
// They are not additional historical RED evidence.
fn observed(direction: Direction, entries: &[(char, u32)]) -> UnicodeBuffer {
    let mut source = UnicodeBuffer::new();
    source.observe_components();
    for &(ch, cluster) in entries {
        source.add(ch, cluster);
    }
    source.set_direction(direction);
    source.set_script(script::LATIN);
    source
}

fn supplied(font: &Shaper<'_>, source: UnicodeBuffer, features: &[Feature]) -> VerticalGlyphBuffer {
    let plan = ShapePlan::new(
        font,
        source.direction(),
        Some(source.script()),
        None,
        features,
    );
    font.shape_vertical_with_metrics(
        &plan,
        source,
        features,
        VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
        |glyph| Ok::<_, Infallible>(font::metrics(font, glyph)),
    )
    .unwrap()
}

fn supplied_ids(buffer: &VerticalGlyphBuffer) -> Vec<(u32, u32)> {
    buffer
        .glyph_infos()
        .iter()
        .map(|glyph| (glyph.glyph_id, glyph.cluster))
        .collect()
}

#[test]
fn nonvertical_and_mismatched_backward_contexts_refuse_before_the_metric_supplier() {
    with_mirror_font(true, |font| {
        for (direction, plan_direction, plan_script, expected) in [
            (
                Direction::Invalid,
                Direction::TopToBottom,
                script::LATIN,
                VerticalShapeContextError::Direction,
            ),
            (
                Direction::LeftToRight,
                Direction::LeftToRight,
                script::LATIN,
                VerticalShapeContextError::Direction,
            ),
            (
                Direction::RightToLeft,
                Direction::RightToLeft,
                script::LATIN,
                VerticalShapeContextError::Direction,
            ),
            (
                Direction::BottomToTop,
                Direction::TopToBottom,
                script::LATIN,
                VerticalShapeContextError::PlanDirection,
            ),
            (
                Direction::BottomToTop,
                Direction::BottomToTop,
                script::ARABIC,
                VerticalShapeContextError::PlanScript,
            ),
        ] {
            let source = input(direction);
            let plan = ShapePlan::new(font, plan_direction, Some(plan_script), None, &[]);
            let mut called = false;
            let result = font.shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |_| {
                    called = true;
                    Ok::<_, Infallible>(font::metrics(font, 35))
                },
            );
            assert!(!called);
            assert!(
                matches!(result, Err(VerticalShapeError::Context(actual)) if actual == expected)
            );
        }
    });
}

#[derive(Clone, Copy, Debug)]
enum DiOutcome {
    Residual,
    Normal,
    Deleted,
}

#[test]
fn backward_native_di_outcomes_keep_original_ordinals_and_actual_final_glyph_indices() {
    for outcome in [DiOutcome::Residual, DiOutcome::Normal, DiOutcome::Deleted] {
        let mut entries = mappings(false);
        // Nominal cmap coverage alone must remain Residual. Only the actual
        // successful ccmp substitution in Normal establishes normal output.
        entries.push((0x2065, font::A));
        let cmap = cmap(&entries);
        let lookup = match outcome {
            DiOutcome::Residual => None,
            DiOutcome::Normal => Some(font::replace()),
            DiOutcome::Deleted => {
                // MultipleSubst format1: coverage35, one empty sequence.
                // This is the backend's existing accepted native deletion.
                Some((2, words(&[1, 10, 1, 8, 0, 1, 1, 35])))
            }
        };
        let gsub = lookup.map(|lookup| font::layout_table(b"ccmp", &[lookup]));
        if let Some(table) = &gsub {
            let list = usize::from(word(table, 8));
            let lookup = list + usize::from(word(table, list + 2));
            let sub = lookup + usize::from(word(table, lookup + 6));
            let coverage = sub + usize::from(word(table, sub + 2));
            assert_eq!(word(table, coverage + 4), 35);
            match outcome {
                DiOutcome::Normal => {
                    assert_eq!(word(table, lookup), 1);
                    assert_eq!(word(table, sub + 6), 58);
                }
                DiOutcome::Deleted => {
                    assert_eq!(word(table, lookup), 2);
                    let sequence = sub + usize::from(word(table, sub + 6));
                    assert_eq!(word(table, sequence), 0);
                }
                DiOutcome::Residual => unreachable!(),
            }
        }
        font::with_cmap_font(true, true, gsub.as_deref(), None, &[], &cmap, |font| {
            qualify(font, true, true);
            assert_eq!(font.get_nominal_glyph(0x2065).unwrap().to_u32(), 35);
            let result = supplied(
                font,
                observed(Direction::BottomToTop, &[('.', 17), ('\u{2065}', 901)]),
                &[],
            );
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            let di = result.default_ignorable_receipt().unwrap();
            let components = result.component_receipt().unwrap().unwrap();
            assert_eq!(components.coverage(), ComponentCoverage::Complete);
            assert_eq!(components.input_len(), 2);
            let dot = (0.0, -701.0, -250.0, -900.0);
            match outcome {
                DiOutcome::Residual => {
                    assert_eq!(supplied_ids(&result), [(274, 901), (16, 17)]);
                    // The native residual zeroes vertical advance/origin Y;
                    // its horizontal offset still comes from nominal glyph35.
                    assert_eq!(positions(&result), [(0.0, 0.0, -500.5, 0.0), dot]);
                    assert_eq!(di.residual_input_positions(), [1]);
                    assert_eq!(di.residual_glyph_positions(), [0]);
                    assert!(di.deleted_input_positions().is_empty());
                    assert_eq!(components.components(0), Some(&[1][..]));
                    assert_eq!(components.components(1), Some(&[0][..]));
                    assert!(components.deleted_input_positions().is_empty());
                }
                DiOutcome::Normal => {
                    assert_eq!(supplied_ids(&result), [(58, 901), (16, 17)]);
                    assert_eq!(positions(&result), [(0.0, -1600.0, -600.5, -1000.0), dot]);
                    assert!(di.residual_input_positions().is_empty());
                    assert!(di.residual_glyph_positions().is_empty());
                    assert!(di.deleted_input_positions().is_empty());
                    assert_eq!(components.components(0), Some(&[1][..]));
                    assert_eq!(components.components(1), Some(&[0][..]));
                    assert!(components.deleted_input_positions().is_empty());
                }
                DiOutcome::Deleted => {
                    assert_eq!(supplied_ids(&result), [(16, 17)]);
                    assert_eq!(positions(&result), [dot]);
                    assert!(di.residual_input_positions().is_empty());
                    assert!(di.residual_glyph_positions().is_empty());
                    assert_eq!(di.deleted_input_positions(), [1]);
                    assert_eq!(components.components(0), Some(&[0][..]));
                    assert_eq!(components.deleted_input_positions(), [1]);
                }
            }
        });
    }
}

#[derive(Debug, Eq, PartialEq)]
struct UsedGlyphFailure {
    marker: u64,
}

#[test]
fn backward_supplier_failure_names_the_actual_used_glyph_without_accepted_partial_output() {
    for (mirrored, used) in [(true, 36), (false, 58)] {
        with_mirror_font(mirrored, |font| {
            let source = input(Direction::BottomToTop);
            let plan = ShapePlan::new(font, Direction::BottomToTop, Some(script::LATIN), None, &[]);
            let mut queried = Vec::new();
            let error = font
                .shape_vertical_with_metrics(
                    &plan,
                    source,
                    &[],
                    VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                    |glyph| {
                        queried.push(glyph);
                        if glyph == used {
                            Err(UsedGlyphFailure { marker: 0x1572 })
                        } else {
                            Ok(font::metrics(font, glyph))
                        }
                    },
                )
                .unwrap_err();
            assert_eq!(
                error,
                VerticalShapeError::Supplier {
                    glyph_id: used,
                    error: UsedGlyphFailure { marker: 0x1572 }
                }
            );
            queried.sort_unstable();
            queried.dedup();
            assert_eq!(queried, [16, used]);
            let fresh = supplied(font, input(Direction::BottomToTop), &[]);
            assert_eq!(fresh.shaping_status(), ShapingStatus::Complete);
            assert_eq!(supplied_ids(&fresh), [(16, 901), (used, 17)]);
        });
    }
}

#[test]
fn prior_backward_growth_refusal_wins_over_context_and_prevents_supplier_calls() {
    with_mirror_font(true, |font| {
        let mut source = input(Direction::BottomToTop);
        // Exercise the actual existing public reserve guard with a bounded
        // owning-harness budget; no fabricated status or refused output.
        source.0.max_len = 2;
        assert!(!source.reserve(3));
        assert_eq!(
            source.0.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
        );
        let plan = ShapePlan::new(font, Direction::TopToBottom, Some(script::LATIN), None, &[]);
        let mut called = false;
        let error = font
            .shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |_| {
                    called = true;
                    Err::<VerticalNominalMetrics, _>(UsedGlyphFailure { marker: 99 })
                },
            )
            .unwrap_err();
        assert!(!called);
        assert_eq!(
            error,
            VerticalShapeError::Refused(ShapingRefusal::GrowthLimit)
        );
    });
}

#[test]
fn backward_clear_reuse_discards_half_units_and_prior_component_and_di_custody() {
    with_mirror_font(false, |font| {
        let result = supplied(
            font,
            observed(
                Direction::BottomToTop,
                &[(LEFT, 17), ('.', 901), ('\u{2065}', 4001)],
            ),
            &[],
        );
        assert_eq!(supplied_ids(&result), [(274, 4001), (16, 901), (58, 17)]);
        assert_eq!(
            positions(&result),
            [
                (0.0, 0.0, -250.5, 0.0),
                (0.0, -701.0, -250.0, -900.0),
                (0.0, -1600.0, -600.5, -1000.0),
            ]
        );
        let di = result.default_ignorable_receipt().unwrap();
        assert_eq!(di.residual_input_positions(), [2]);
        assert_eq!(di.residual_glyph_positions(), [0]);
        assert!(di.deleted_input_positions().is_empty());
        let mut clean = result.clear();
        assert!(clean.is_empty());
        clean.add('A', 7001);
        clean.set_direction(Direction::LeftToRight);
        clean.set_script(script::LATIN);
        let ordinary = font.shape(clean, &[]);
        assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
        assert_eq!(ids(&ordinary), [(35, 7001)]);
        assert_eq!(old_positions(&ordinary), [(1001, 0, 0, 0)]);
        assert!(ordinary.component_receipt().unwrap().is_none());
        let di = ordinary.default_ignorable_receipt().unwrap();
        assert!(di.residual_input_positions().is_empty());
        assert!(di.deleted_input_positions().is_empty());
        assert!(di.residual_glyph_positions().is_empty());
        let mut again = ordinary.clear();
        again.observe_components();
        again.add(LEFT, 8001);
        again.set_direction(Direction::BottomToTop);
        again.set_script(script::LATIN);
        let again = supplied(font, again, &[]);
        assert_eq!(supplied_ids(&again), [(58, 8001)]);
        assert_eq!(positions(&again), [(0.0, -1600.0, -600.5, -1000.0)]);
        assert_eq!(again.component_receipt().unwrap().unwrap().input_len(), 1);
        assert!(again
            .default_ignorable_receipt()
            .unwrap()
            .residual_input_positions()
            .is_empty());
    });
}

#[test]
fn backward_explicit_rtlm_values_keep_canonical_mirror_precedence_and_forward_controls() {
    for mirrored in [false, true] {
        with_mirror_font(mirrored, |font| {
            let disabled = [Feature::new(Tag::new(b"rtlm"), 0, ..)];
            for direction in [Direction::BottomToTop, Direction::RightToLeft] {
                let source = observed(direction, &[(LEFT, 17)]);
                let ordinary = font.shape(source, &disabled);
                let expected = if mirrored { 36 } else { 35 };
                assert_eq!(ids(&ordinary), [(expected, 17)]);
            }
            let backward = supplied(
                font,
                observed(Direction::BottomToTop, &[(LEFT, 17)]),
                &disabled,
            );
            assert_eq!(
                supplied_ids(&backward),
                [(if mirrored { 36 } else { 35 }, 17)]
            );
            assert_eq!(
                positions(&backward),
                if mirrored {
                    vec![(0.0, -1400.0, -500.0, -1000.0)]
                } else {
                    vec![(0.0, -1200.0, -500.5, -900.0)]
                }
            );
            let forward = supplied(font, observed(Direction::TopToBottom, &[(LEFT, 17)]), &[]);
            assert_eq!(supplied_ids(&forward), [(35, 17)]);
            assert_eq!(positions(&forward), [(0.0, -1200.0, -500.5, -900.0)]);
            let explicit = [Feature::new(Tag::new(b"rtlm"), 1, ..)];
            let forced = supplied(
                font,
                observed(Direction::TopToBottom, &[(LEFT, 17)]),
                &explicit,
            );
            assert_eq!(supplied_ids(&forced), [(58, 17)]);
            assert_eq!(positions(&forced), [(0.0, -1600.0, -600.5, -1000.0)]);
        });
    }
}

#[test]
fn backward_native_mirroring_does_not_enable_unrelated_rtla_alternates() {
    let cmap = cmap(&mappings(false));
    let gsub = font::layout_table(b"rtla", &[font::replace()]);
    let features = usize::from(word(&gsub, 6));
    assert_eq!(&gsub[features + 2..features + 6], b"rtla");
    let list = usize::from(word(&gsub, 8));
    let lookup = list + usize::from(word(&gsub, list + 2));
    let sub = lookup + usize::from(word(&gsub, lookup + 6));
    assert_eq!(
        (word(&gsub, lookup), word(&gsub, sub), word(&gsub, sub + 6)),
        (1, 2, 58)
    );
    let coverage = sub + usize::from(word(&gsub, sub + 2));
    assert_eq!(word(&gsub, coverage + 4), 35);
    font::with_cmap_font(true, true, Some(&gsub), None, &[], &cmap, |font| {
        qualify(font, true, true);
        assert!(font.get_nominal_glyph(u32::from(RIGHT)).is_none());
        for direction in [Direction::TopToBottom, Direction::BottomToTop] {
            let ordinary = font.shape(observed(direction, &[(LEFT, 17)]), &[]);
            assert_eq!(ids(&ordinary), [(35, 17)]);
            let result = supplied(font, observed(direction, &[(LEFT, 17)]), &[]);
            assert_eq!(supplied_ids(&result), [(35, 17)]);
            assert_eq!(positions(&result), [(0.0, -1200.0, -500.5, -900.0)]);
        }
        let rtl = font.shape(observed(Direction::RightToLeft, &[(LEFT, 17)]), &[]);
        assert_eq!(ids(&rtl), [(58, 17)]);
        assert_eq!(old_positions(&rtl), [(1201, 0, 0, 0)]);
        let explicit = [Feature::new(Tag::new(b"rtla"), 1, ..)];
        let backward = supplied(
            font,
            observed(Direction::BottomToTop, &[(LEFT, 17)]),
            &explicit,
        );
        assert_eq!(supplied_ids(&backward), [(58, 17)]);
        assert_eq!(positions(&backward), [(0.0, -1600.0, -600.5, -1000.0)]);
    });
}

#[test]
fn backward_required_ligature_preserves_actual_components_and_required_feature_provenance() {
    let gsub = font::layout_table(b"rlig", &[font::ligature()]);
    let list = usize::from(word(&gsub, 8));
    let lookup = list + usize::from(word(&gsub, list + 2));
    let sub = lookup + usize::from(word(&gsub, lookup + 6));
    assert_eq!(word(&gsub, lookup), 4);
    let coverage = sub + usize::from(word(&gsub, sub + 2));
    assert_eq!(word(&gsub, coverage + 4), 35);
    let set = sub + usize::from(word(&gsub, sub + 6));
    let ligature = set + usize::from(word(&gsub, set + 2));
    assert_eq!(
        (
            word(&gsub, ligature),
            word(&gsub, ligature + 2),
            word(&gsub, ligature + 4)
        ),
        (58, 2, 36)
    );
    font::with_font(true, true, Some(&gsub), None, &[], |font| {
        qualify(font, true, true);
        // Backward native grapheme order makes logical BA execute the real
        // required AB ->58 rule. Source ordinals still mean logical B0/A1.
        let source = observed(Direction::BottomToTop, &[('B', 17), ('A', 901)]);
        let result = supplied(font, source, &[]);
        assert_eq!(supplied_ids(&result), [(58, 17)]);
        assert_eq!(positions(&result), [(0.0, -1600.0, -600.5, -1000.0)]);
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.components(0), Some(&[0, 1][..]));
        let ligatures = receipt.ligatures().collect::<Vec<_>>();
        assert_eq!(ligatures.len(), 1);
        assert_eq!(ligatures[0].components(), [0, 1]);
        assert_eq!(ligatures[0].glyph_indices(), [0]);
        assert!(ligatures[0]
            .provenance()
            .iter()
            .any(|p| p.tag() == Tag::new(b"rlig") && p.role() == crate::FeatureRole::Required));
    });
}

#[test]
fn backward_real_gpos_attachment_keeps_the_used_parent_half_origin_and_input_custody() {
    let mut entries = mappings(false);
    entries.push((0x300, font::MARK));
    let cmap = cmap(&entries);
    let gpos = font::layout_table(
        b"mark",
        &[font::mark(font::A, font::MARK, (300, 700), (20, 100), 4)],
    );
    font::with_cmap_font(true, true, None, Some(&gpos), &[], &cmap, |font| {
        qualify(font, true, true);
        let source = observed(
            Direction::BottomToTop,
            &[('A', 17), ('\u{300}', 901), ('.', 4001)],
        );
        let result = supplied(font, source, &[]);
        assert_eq!(supplied_ids(&result), [(16, 4001), (35, 17), (100, 17)]);
        // A keeps its odd nominal X=-500.5. Actual anchor difference(280,600)
        // plus the parent's position and1200 advance gives mark(-220.5,900).
        assert_eq!(
            positions(&result),
            [
                (0.0, -701.0, -250.0, -900.0),
                (0.0, -1200.0, -500.5, -900.0),
                (0.0, 0.0, -220.5, 900.0),
            ]
        );
        let receipt = result.component_receipt().unwrap().unwrap();
        assert_eq!(receipt.coverage(), ComponentCoverage::Complete);
        assert_eq!(receipt.components(0), Some(&[2][..]));
        assert_eq!(receipt.components(1), Some(&[0][..]));
        assert_eq!(receipt.components(2), Some(&[1][..]));
        let attachments = receipt.attachments().collect::<Vec<_>>();
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].kind(), crate::ComponentAttachmentKind::Mark);
        assert_eq!(attachments[0].dependent_components(), [1]);
        assert_eq!(attachments[0].base_components(), [0]);
    });
}
