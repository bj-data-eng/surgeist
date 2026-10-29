//! Functional acceptance of the new invocation-local half-design-unit owner.
//! This is a wholly new operation, not historical RED evidence. Expected values
//! come from authored tables/nominal records and the existing OT attachment rules.
use crate::hb::completion_fixture::words;
use crate::{
    script, Direction, Feature, GlyphBuffer, ShapePlan, Shaper, ShapingRefusal, ShapingStatus,
    UnicodeBuffer, VerticalGlyphBuffer, VerticalNominalMetrics, VerticalOriginReference,
    VerticalPositionStage, VerticalShapeContextError, VerticalShapeError,
};
use alloc::{boxed::Box, vec, vec::Vec};
use core::{convert::Infallible, mem::size_of};
use read_fonts::TableProvider;

#[path = "vertical_position_fixture.rs"]
mod font;

#[path = "backward_vertical_tests.rs"]
mod backward;

fn input(text: &str, direction: Direction) -> UnicodeBuffer {
    let mut input = UnicodeBuffer::new();
    // Explicit scalar ordinals are the authentic provider source/cluster domain.
    for (index, ch) in text.chars().enumerate() {
        input.add(ch, u32::try_from(index).unwrap());
    }
    input.set_direction(direction);
    input.set_script(script::LATIN);
    input
}
fn make_plan(font: &Shaper<'_>, input: &UnicodeBuffer, features: &[Feature]) -> ShapePlan {
    ShapePlan::new(
        font,
        input.direction(),
        Some(input.script()),
        None,
        features,
    )
}
fn vertical(font: &Shaper<'_>, text: &str, features: &[Feature]) -> VerticalGlyphBuffer {
    let input = input(text, Direction::TopToBottom);
    let plan = make_plan(font, &input, features);
    font.shape_vertical_with_metrics(
        &plan,
        input,
        features,
        VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
        |gid| Ok::<_, Infallible>(font::metrics(font, gid)),
    )
    .unwrap()
}
fn ordinary(
    font: &Shaper<'_>,
    text: &str,
    direction: Direction,
    features: &[Feature],
) -> GlyphBuffer {
    font.shape(input(text, direction), features)
}
fn positions(buffer: &VerticalGlyphBuffer) -> Vec<(f64, f64, f64, f64)> {
    buffer
        .glyph_positions()
        .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
        .collect()
}
fn old_positions(buffer: &GlyphBuffer) -> Vec<(i32, i32, i32, i32)> {
    buffer
        .glyph_positions()
        .iter()
        .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
        .collect()
}
fn ids(buffer: &VerticalGlyphBuffer) -> Vec<(u32, u32)> {
    buffer
        .glyph_infos()
        .iter()
        .map(|g| (g.glyph_id, g.cluster))
        .collect()
}
fn feature(tag: &[u8; 4]) -> [Feature; 1] {
    [Feature::new(crate::Tag::new(tag), 1, ..)]
}
fn failure<E>(result: Result<VerticalGlyphBuffer, VerticalShapeError<E>>) -> VerticalShapeError<E> {
    match result {
        Err(error) => error,
        Ok(_) => panic!("invalid invocation must not return an accepted glyph buffer"),
    }
}

fn qualify(font: &Shaper<'_>, native: bool, odd: bool) {
    assert_eq!(font.units_per_em(), 1000);
    assert!(font.coords().is_empty());
    assert_eq!(
        font.get_nominal_glyph('A' as u32).unwrap().to_u32(),
        u32::from(font::A)
    );
    assert_eq!(
        font.get_nominal_glyph('B' as u32).unwrap().to_u32(),
        u32::from(font::B)
    );
    assert!(font.get_nominal_glyph('Z' as u32).is_none());
    assert_eq!(
        font.font
            .hmtx()
            .unwrap()
            .advance(read_fonts::types::GlyphId::new(u32::from(font::A))),
        Some(if odd { 1001 } else { 1000 })
    );
    assert_eq!(font.font.vhea().is_ok(), native);
    assert_eq!(font.font.vmtx().is_ok(), native);
    let metrics = font::metrics(font, u32::from(font::A));
    assert_eq!(
        (
            metrics.advance(),
            metrics.origin_x_twice(),
            metrics.origin_y()
        ),
        (
            1200,
            if odd { 1001 } else { 1000 },
            if native { 900 } else { 1000 }
        )
    );
}

#[test]
fn real_native_and_absent_records_keep_odd_half_origins_and_ordinary_positions() {
    for native in [false, true] {
        for odd in [false, true] {
            font::with_font(native, odd, None, None, &[], |font| {
                qualify(font, native, odd);
                let h = ordinary(font, "AB", Direction::LeftToRight, &[]);
                assert_eq!(
                    old_positions(&h),
                    [(if odd { 1001 } else { 1000 }, 0, 0, 0), (1000, 0, 0, 0)]
                );
                let old = ordinary(font, "A", Direction::TopToBottom, &[]);
                // Ordinary integer W/2 remains500; absent-table provider ink-centering
                // remains1000 for this actual800-high box and1200 line height.
                assert_eq!(
                    old_positions(&old),
                    [(0, -1200, -500, if native { -900 } else { -1000 })]
                );
                let result = vertical(font, "AB", &[]);
                assert_eq!(result.shaping_status(), ShapingStatus::Complete);
                assert_eq!(ids(&result), [(35, 0), (36, 1)]);
                assert_eq!(
                    positions(&result),
                    [
                        (
                            0.0,
                            -1200.0,
                            if odd { -500.5 } else { -500.0 },
                            if native { -900.0 } else { -1000.0 }
                        ),
                        (0.0, if native { -1400.0 } else { -1200.0 }, -500.0, -1000.0)
                    ]
                );
                assert_eq!(
                    old_positions(&ordinary(font, "A", Direction::TopToBottom, &[])),
                    old_positions(&old)
                );
                assert_eq!(size_of::<crate::GlyphInfo>(), 32);
                assert_eq!(size_of::<crate::GlyphPosition>(), 32);
            });
        }
    }
}

#[test]
fn actual_glyph_zero_and_signed_scalar_records_are_consumed_without_inventory_scan() {
    font::with_font(true, true, None, None, &[], |font| {
        let result = vertical(font, "Z", &[]);
        assert_eq!(ids(&result), [(0, 0)]);
        assert_eq!(positions(&result), [(0.0, -600.0, -250.5, 100.0)]);
        let input = input("A", Direction::TopToBottom);
        let plan = make_plan(font, &input, &[]);
        let mut asked = Vec::new();
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                input,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |gid| {
                    asked.push(gid);
                    if gid == u32::from(font::A) {
                        Ok::<_, &'static str>(VerticalNominalMetrics::new(0, -3, -20))
                    } else {
                        Err("unrequested corrupt record")
                    }
                },
            )
            .unwrap();
        assert!(asked.iter().all(|&gid| gid == u32::from(font::A)));
        assert!(!asked.is_empty());
        assert_eq!(positions(&result), [(0.0, 0.0, 1.5, 20.0)]);
    });
}

#[test]
fn supplier_queries_actual_replacement_and_ligature_glyphs_after_real_gsub() {
    for (lookup, text, cluster) in [(font::replace(), "A", 0), (font::ligature(), "AB", 0)] {
        let gsub = font::layout_table(b"ccmp", &[lookup]);
        font::with_font(true, true, Some(&gsub), None, &[], |font| {
            qualify(font, true, true);
            assert_eq!(
                font.font
                    .gsub()
                    .unwrap()
                    .lookup_list()
                    .unwrap()
                    .lookup_count(),
                1
            );
            let control = ordinary(font, text, Direction::TopToBottom, &[]);
            assert_eq!(
                control
                    .glyph_infos()
                    .iter()
                    .map(|g| (g.glyph_id, g.cluster))
                    .collect::<Vec<_>>(),
                [(u32::from(font::R), cluster)]
            );
            let input = input(text, Direction::TopToBottom);
            let plan = make_plan(font, &input, &[]);
            let mut queried = Vec::new();
            let result = font
                .shape_vertical_with_metrics(
                    &plan,
                    input,
                    &[],
                    VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                    |gid| {
                        queried.push(gid);
                        Ok::<_, Infallible>(font::metrics(font, gid))
                    },
                )
                .unwrap();
            assert!(!queried.is_empty());
            assert!(queried.iter().all(|&gid| gid == u32::from(font::R)));
            assert_eq!(ids(&result), [(58, cluster)]);
            assert_eq!(positions(&result), [(0.0, -1600.0, -600.5, -1000.0)]);
        });
    }
}

#[test]
fn single_and_pair_gpos_values_remain_raw_integer_contributions_in_half_domain() {
    let features = feature(b"vkrn");
    for (lookup, text, expected) in [
        (font::single(), "A", vec![(0.0, -1250.0, -470.5, -860.0)]),
        (
            font::pair(),
            "AB",
            vec![
                (0.0, -1250.0, -470.5, -860.0),
                (0.0, -1390.0, -520.0, -1030.0),
            ],
        ),
    ] {
        let gpos = font::layout_table(b"vkrn", &[lookup]);
        font::with_font(true, true, None, Some(&gpos), &[], |font| {
            qualify(font, true, true);
            let table = font.font.gpos().unwrap();
            assert_eq!(table.lookup_list().unwrap().lookup_count(), 1);
            let old = ordinary(font, text, Direction::TopToBottom, &features);
            assert_eq!(old_positions(&old)[0], (0, -1250, -470, -860));
            let result = vertical(font, text, &features);
            assert_eq!(positions(&result), expected);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
        });
    }
}

#[test]
fn real_mark_to_base_and_mark_to_mark_overwrite_then_propagate_parent_half_origin() {
    for odd in [false, true] {
        let gpos = font::layout_table(
            b"mark",
            &[
                font::mark(font::A, font::MARK, (300, 700), (20, 100), 4),
                font::mark(font::MARK, font::MARK2, (70, 400), (10, 50), 6),
            ],
        );
        font::with_font(true, odd, None, Some(&gpos), &[], |font| {
            qualify(font, true, odd);
            assert_eq!(
                font.font
                    .gpos()
                    .unwrap()
                    .lookup_list()
                    .unwrap()
                    .lookup_count(),
                2
            );
            let features = feature(b"mark");
            let old = ordinary(font, "A\u{300}\u{315}", Direction::TopToBottom, &features);
            assert_eq!(
                old.glyph_infos()
                    .iter()
                    .map(|g| g.glyph_id)
                    .collect::<Vec<_>>(),
                [35, 100, 101]
            );
            assert_eq!(
                old_positions(&old),
                [
                    (0, -1200, -500, -900),
                    (0, 0, -220, 900),
                    (0, 0, -160, 1250)
                ]
            );
            let result = vertical(font, "A\u{300}\u{315}", &features);
            let half = if odd { 0.5 } else { 0.0 };
            assert_eq!(
                positions(&result),
                [
                    (0.0, -1200.0, -500.0 - half, -900.0),
                    (0.0, 0.0, -220.0 - half, 900.0),
                    (0.0, 0.0, -160.0 - half, 1250.0)
                ]
            );
            assert!(result.glyph_infos()[1..].iter().all(|info| info.is_mark()));
        });
    }
}

#[test]
fn real_vertical_cursive_advances_and_cross_attachment_preserve_parent_half_origin() {
    let gpos = font::layout_table(b"curs", &[font::cursive()]);
    font::with_font(true, true, None, Some(&gpos), &[], |font| {
        let features = feature(b"curs");
        let old = ordinary(font, "AB", Direction::TopToBottom, &features);
        assert_eq!(
            old_positions(&old),
            [(0, -400, -500, -900), (0, -600, -300, -200)]
        );
        let result = vertical(font, "AB", &features);
        assert_eq!(
            positions(&result),
            [(0.0, -400.0, -500.5, -900.0), (0.0, -600.0, -300.5, -200.0)]
        );
        assert_eq!(ids(&result), [(35, 0), (36, 1)]);
    });
}

#[test]
fn no_gpos_mark_fallback_retains_raw_centering_and_gap_quotients() {
    font::with_font(true, true, None, None, &[], |font| {
        assert!(font.font.gpos().is_err());
        let old = ordinary(font, "A\u{300}", Direction::TopToBottom, &[]);
        // Width(1001−100)/2 truncates450, then each actual base X origin
        // translates it into the vertical pen. UPEM1000/16 is62. With base
        // originY−900 and ink top800, above-mark correction is38/2=19.
        assert_eq!(
            old_positions(&old),
            [(0, -1200, -500, -900), (0, 0, -50, 1181)]
        );
        let result = vertical(font, "A\u{300}", &[]);
        assert_eq!(
            positions(&result),
            [(0.0, -1200.0, -500.5, -900.0), (0.0, 0.0, -50.5, 1181.0)]
        );
    });
}

#[test]
fn missing_space_fallback_queries_real_digit_and_punctuation_and_preserves_raw_division() {
    font::with_font(true, true, None, None, &[], |font| {
        for (text, expected) in [
            ("\u{2007}", -1701.0_f64),
            ("\u{2008}", -701.0),
            ("\u{202f}", -500.0),
            ("\u{2004}", -333.0),
        ] {
            assert!(font
                .get_nominal_glyph(text.chars().next().unwrap() as u32)
                .is_none());
            let old = ordinary(font, text, Direction::TopToBottom, &[]);
            assert_eq!(old.glyph_positions()[0].y_advance, expected as i32);
            let input = input(text, Direction::TopToBottom);
            let plan = make_plan(font, &input, &[]);
            let mut asked = Vec::new();
            let result = font
                .shape_vertical_with_metrics(
                    &plan,
                    input,
                    &[],
                    VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                    |gid| {
                        asked.push(gid);
                        Ok::<_, Infallible>(font::metrics(font, gid))
                    },
                )
                .unwrap();
            assert_eq!(positions(&result)[0].1.to_bits(), expected.to_bits());
            if text == "\u{2007}" {
                assert!(asked.contains(&u32::from(font::DIGIT)));
            }
            if text == "\u{2008}" {
                assert!(asked.contains(&u32::from(font::DOT)));
            }
        }
    });
}

#[derive(Debug, PartialEq)]
struct SupplierError {
    marker: u64,
}

#[test]
fn later_distinct_needed_glyph_preserves_original_supplier_error_and_invocation_locality() {
    font::with_font(true, true, None, None, &[], |font| {
        let input = input("AB", Direction::TopToBottom);
        let plan = make_plan(font, &input, &[]);
        let mut asked = Vec::new();
        let error = failure(font.shape_vertical_with_metrics(
            &plan,
            input,
            &[],
            VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
            |gid| {
                asked.push(gid);
                if gid == u32::from(font::B) {
                    Err(SupplierError { marker: 0x1527 })
                } else {
                    Ok(font::metrics(font, gid))
                }
            },
        ));
        assert!(asked.contains(&u32::from(font::A)) && asked.contains(&u32::from(font::B)));
        assert!(matches!(
            error,
            VerticalShapeError::Supplier {
                glyph_id: 36,
                error: SupplierError { marker: 0x1527 }
            }
        ));
        assert_eq!(
            positions(&vertical(font, "A", &[])),
            [(0.0, -1200.0, -500.5, -900.0)]
        );
    });
}

#[test]
fn nominal_and_actual_gpos_attachment_fallback_overflows_refuse_at_their_owner_stage() {
    for (gpos, text, record, stage) in [
        (
            None,
            "A",
            VerticalNominalMetrics::new(u32::MAX, 1001, 900),
            VerticalPositionStage::Nominal,
        ),
        (
            Some(font::layout_table(
                b"mark",
                &[(
                    1,
                    words(&[1, 14, 15, (-30i16) as u16, 0, 0, 0, 1, 1, font::A]),
                )],
            )),
            "A",
            VerticalNominalMetrics::new(0, i32::MAX, 0),
            VerticalPositionStage::Gpos,
        ),
        (
            Some(font::layout_table(
                b"mark",
                &[font::mark(font::A, font::MARK, (300, 700), (20, 100), 4)],
            )),
            "A\u{300}",
            VerticalNominalMetrics::new(1200, -i32::MAX, 900),
            VerticalPositionStage::Attachment,
        ),
        (
            None,
            "A\u{300}",
            VerticalNominalMetrics::new(1200, 1001, -1_073_741_823),
            VerticalPositionStage::Fallback,
        ),
    ] {
        font::with_font(true, true, None, gpos.as_deref(), &[], |font| {
            let features = feature(b"mark");
            let input = input(text, Direction::TopToBottom);
            let plan = make_plan(font, &input, &features);
            let error = failure(font.shape_vertical_with_metrics(
                &plan,
                input,
                &features,
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |gid| {
                    Ok::<_, Infallible>(if gid == u32::from(font::A) {
                        record
                    } else {
                        font::metrics(font, gid)
                    })
                },
            ));
            assert!(
                matches!(error, VerticalShapeError::CoordinateRange { stage: actual } if actual == stage)
            );
        });
    }
}

#[test]
fn actual_prelatched_growth_work_lookup_and_attachment_refusals_win_over_would_fail_supplier() {
    use crate::hb::{
        buffer::HB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT,
        ot_layout::TableIndex,
        ot_layout_gpos_table::{attach_type, GPOS},
        ot_layout_gsubgpos::OT::hb_ot_apply_context_t,
    };
    font::with_font(true, true, None, None, &[], |font| {
        for reason in [
            ShapingRefusal::GrowthLimit,
            ShapingRefusal::WorkLimit,
            ShapingRefusal::LookupNestingLimit,
            ShapingRefusal::AttachmentNestingLimit,
        ] {
            let mut source = input("A", Direction::TopToBottom);
            match reason {
                ShapingRefusal::GrowthLimit => {
                    source.0.max_len = 1;
                    assert!(!source.reserve(2));
                }
                ShapingRefusal::WorkLimit => {
                    source.0.max_ops = 0;
                    source.0.set_masks(1, 1, 0, u32::MAX);
                }
                ShapingRefusal::LookupNestingLimit => {
                    let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, font, &mut source.0);
                    ctx.nesting_level_left = 0;
                    assert!(ctx.recurse(u16::MAX).is_none());
                }
                _ => {
                    source.0 = crate::hb::completion_fixture::glyphs(66);
                    source.0.direction = Direction::TopToBottom;
                    source.0.allocate_gsubgpos_vars();
                    source.0.scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT;
                    for position in &mut source.0.pos[..65] {
                        position.set_attach_chain(1);
                        position.set_attach_type(attach_type::CURSIVE);
                        position.x_offset = 1;
                    }
                    GPOS::position_finish_offsets(font, &mut source.0);
                }
            }
            assert_eq!(source.0.shaping_status(), ShapingStatus::Refused(reason));
            let plan = make_plan(font, &source, &[]);
            let mut called = false;
            let error = failure(font.shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |_| {
                    called = true;
                    Err::<VerticalNominalMetrics, _>(SupplierError { marker: 7 })
                },
            ));
            assert!(!called);
            assert!(matches!(error, VerticalShapeError::Refused(actual) if actual == reason));
        }
    });
}

#[test]
fn accepted_di_receipt_and_clear_reuse_keep_original_scalar_custody_and_reset_domain() {
    font::with_font(true, true, None, None, &[], |font| {
        let result = vertical(font, "A\u{2065}", &[]);
        let receipt = result.default_ignorable_receipt().unwrap();
        assert_eq!(receipt.residual_input_positions(), [1]);
        assert!(receipt.deleted_input_positions().is_empty());
        assert_eq!(receipt.residual_glyph_positions(), [1]);
        assert_eq!(ids(&result), [(35, 0), (274, 1)]);
        assert_eq!(positions(&result)[1].1.to_bits(), 0.0_f64.to_bits());
        let mut reused = result.clear();
        assert!(reused.is_empty());
        reused.add('A', 17);
        reused.set_direction(Direction::LeftToRight);
        reused.set_script(script::LATIN);
        let ordinary = font.shape(reused, &[]);
        assert_eq!(ordinary.shaping_status(), ShapingStatus::Complete);
        assert_eq!(ordinary.glyph_infos()[0].cluster, 17);
        assert_eq!(old_positions(&ordinary), [(1001, 0, 0, 0)]);
        assert!(ordinary
            .default_ignorable_receipt()
            .unwrap()
            .residual_input_positions()
            .is_empty());
    });
}

#[test]
fn wrong_direction_and_plan_properties_refuse_without_supplier_or_ordinary_api_alias() {
    font::with_font(true, true, None, None, &[], |font| {
        for (direction, plan_direction, plan_script, expected) in [
            (
                Direction::LeftToRight,
                Direction::LeftToRight,
                script::LATIN,
                VerticalShapeContextError::Direction,
            ),
            (
                Direction::TopToBottom,
                Direction::LeftToRight,
                script::LATIN,
                VerticalShapeContextError::PlanDirection,
            ),
            (
                Direction::TopToBottom,
                Direction::TopToBottom,
                script::ARABIC,
                VerticalShapeContextError::PlanScript,
            ),
        ] {
            let source = input("A", direction);
            let plan = ShapePlan::new(font, plan_direction, Some(plan_script), None, &[]);
            let mut called = false;
            let error = failure(font.shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |_| {
                    called = true;
                    Ok::<_, Infallible>(font::metrics(font, 35))
                },
            ));
            assert!(!called);
            assert!(matches!(error, VerticalShapeError::Context(actual) if actual == expected));
        }
    });
}

#[test]
fn selected_native_kerx_refuses_but_dormant_data_with_actual_ot_gpos_admits() {
    let mut kerx = [0x0002_0000u32, 1, 34, 0x8000_0000, 0, 1, 6, 0, 0]
        .into_iter()
        .flat_map(u32::to_be_bytes)
        .collect::<Vec<_>>();
    kerx.extend(words(&[font::A, font::B, 25]));
    let gsub = font::layout_table(b"ccmp", &[]);
    let gpos = font::layout_table(b"vkrn", &[font::single()]);
    for ot in [false, true] {
        font::with_font(
            true,
            true,
            ot.then_some(gsub.as_slice()),
            ot.then_some(gpos.as_slice()),
            &[(*b"kerx", &kerx)],
            |font| {
                assert!(font.aat_tables.kerx.is_some());
                let features = feature(b"vkrn");
                let source = input("A", Direction::TopToBottom);
                let plan = make_plan(font, &source, &features);
                assert_eq!(plan.apply_kerx, !ot);
                assert_eq!(plan.apply_gpos, ot);
                let mut asked = false;
                let result = font.shape_vertical_with_metrics(
                    &plan,
                    source,
                    &features,
                    VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                    |gid| {
                        asked = true;
                        Ok::<_, Infallible>(font::metrics(font, gid))
                    },
                );
                if ot {
                    assert!(asked);
                    assert_eq!(
                        positions(&result.unwrap()),
                        [(0.0, -1250.0, -470.5, -860.0)]
                    );
                } else {
                    assert!(!asked);
                    assert!(matches!(
                        failure(result),
                        VerticalShapeError::Context(VerticalShapeContextError::Kerx)
                    ));
                }
            },
        );
    }
}

#[test]
fn selected_real_morx_kern_and_tracking_plans_refuse_before_supplier() {
    let morx = font::morx();
    let kern = font::kern();
    let trak = font::trak();
    let stat = font::stat();
    // A genuine DFLT feature record makes the explicit vertical kerning request
    // available to the map. No GSUB substitution is needed, and absent GPOS
    // leaves the actual legacy Kern positioning owner selected.
    let kern_feature = font::layout_table(b"vkrn", &[]);
    for path in 0..3 {
        let tables = match path {
            0 => vec![(*b"morx", morx.as_slice())],
            1 => vec![(*b"kern", kern.as_slice())],
            _ => vec![(*b"trak", trak.as_slice()), (*b"STAT", stat.as_slice())],
        };
        let gsub = (path == 1).then_some(kern_feature.as_slice());
        font::with_font(true, true, gsub, None, &tables, |font| {
            // Bind an actual configured point size for the real tracking case.
            let data = crate::ShaperData::new(&font.font);
            let configured = data.shaper(&font.font).point_size(Some(12.0)).build();
            let text = if path == 1 { "AB" } else { "A" };
            let features = feature(b"vkrn");
            let source = input(text, Direction::TopToBottom);
            let plan = make_plan(&configured, &source, &features);
            let control = ordinary(&configured, text, Direction::TopToBottom, &features);
            assert_eq!(control.shaping_status(), ShapingStatus::Complete);
            let expected = match path {
                0 => {
                    assert!(plan.apply_morx);
                    assert!(configured.aat_tables.morx.is_some());
                    assert_eq!(control.glyph_infos()[0].glyph_id, u32::from(font::R));
                    VerticalShapeContextError::Morx
                }
                1 => {
                    assert!(plan.apply_kern);
                    assert!(!plan.apply_gpos);
                    assert!(plan.requested_kerning);
                    assert!(plan
                        .ot_map
                        .get_feature_index(
                            crate::hb::ot_layout::TableIndex::GSUB,
                            crate::Tag::new(b"vkrn")
                        )
                        .is_some());
                    assert!(configured.aat_tables.kern.is_some());
                    assert_eq!(
                        control
                            .glyph_positions()
                            .iter()
                            .map(|p| p.y_advance)
                            .sum::<i32>(),
                        -2575
                    );
                    VerticalShapeContextError::Kern
                }
                _ => {
                    assert!(plan.apply_trak);
                    let track = configured.font.trak().unwrap();
                    let vertical = track.vert().unwrap().unwrap();
                    assert_eq!((vertical.n_tracks(), vertical.n_sizes()), (1, 1));
                    assert_eq!(configured.font.stat().unwrap().design_axis_count(), 0);
                    assert_eq!(control.glyph_positions()[0].y_advance, -1175);
                    VerticalShapeContextError::Trak
                }
            };
            let mut called = false;
            let error = failure(configured.shape_vertical_with_metrics(
                &plan,
                source,
                &features,
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |_| {
                    called = true;
                    Err::<VerticalNominalMetrics, _>(SupplierError { marker: 9 })
                },
            ));
            assert!(!called);
            assert!(matches!(error, VerticalShapeError::Context(actual) if actual == expected));
        });
    }
}

#[test]
fn real_recursive_gsub_limit_remains_refused_in_the_new_operation() {
    let gsub = font::layout_table(b"ccmp", &[(5, words(&[3, 1, 1, 12, 0, 0, 1, 1, font::A]))]);
    font::with_font(true, true, Some(&gsub), None, &[], |font| {
        let source = input("A", Direction::TopToBottom);
        let plan = make_plan(font, &source, &[]);
        let control = font.shape_with_plan(&plan, input("A", Direction::TopToBottom), &[]);
        assert_eq!(
            control.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::LookupNestingLimit)
        );
        let error = failure(font.shape_vertical_with_metrics(
            &plan,
            source,
            &[],
            VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
            |gid| Ok::<_, Infallible>(font::metrics(font, gid)),
        ));
        assert!(matches!(
            error,
            VerticalShapeError::Refused(ShapingRefusal::LookupNestingLimit)
        ));
    });
}

#[test]
fn joining_profile_custody_is_revalidated_and_unicode_clear_removes_old_binding() {
    font::with_font(true, true, None, None, &[], |font| {
        let joining =
            crate::JoiningSource::analyze("AB", script::LATIN, Direction::LeftToRight, &[])
                .unwrap();
        let mut source = input("AB", Direction::LeftToRight);
        source
            .set_joining_sources(&joining, &[Some(0), Some(1)], &joining, &[0, 1])
            .unwrap();
        source.set_direction(Direction::TopToBottom);
        let plan = make_plan(font, &source, &[]);
        let error = failure(font.shape_vertical_with_metrics(
            &plan,
            source,
            &[],
            VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
            |gid| Ok::<_, Infallible>(font::metrics(font, gid)),
        ));
        assert!(matches!(
            error,
            VerticalShapeError::Refused(ShapingRefusal::JoiningInput)
        ));
        let mut clean = input("AB", Direction::LeftToRight);
        clean
            .set_joining_sources(&joining, &[Some(0), Some(1)], &joining, &[0, 1])
            .unwrap();
        clean.clear();
        clean.add('A', 13);
        clean.set_script(script::LATIN);
        clean.set_direction(Direction::TopToBottom);
        let plan = make_plan(font, &clean, &[]);
        let result = font
            .shape_vertical_with_metrics(
                &plan,
                clean,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |gid| Ok::<_, Infallible>(font::metrics(font, gid)),
            )
            .unwrap();
        assert_eq!(ids(&result), [(35, 13)]);
        assert_eq!(positions(&result), [(0.0, -1200.0, -500.5, -900.0)]);
    });
}

#[test]
fn valid_empty_ttb_result_has_empty_receipt_no_supplier_and_clear_reuses_as_ordinary() {
    font::with_font(true, true, None, None, &[], |font| {
        let source = input("", Direction::TopToBottom);
        let plan = make_plan(font, &source, &[]);
        let mut called = false;
        let empty = font
            .shape_vertical_with_metrics(
                &plan,
                source,
                &[],
                VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
                |_| {
                    called = true;
                    Err::<VerticalNominalMetrics, _>(SupplierError { marker: 8 })
                },
            )
            .unwrap();
        assert!(!called);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert!(empty.glyph_infos().is_empty());
        assert_eq!(empty.glyph_positions().len(), 0);
        assert_eq!(empty.shaping_status(), ShapingStatus::Complete);
        let receipt = empty.default_ignorable_receipt().unwrap();
        assert!(receipt.residual_input_positions().is_empty());
        assert!(receipt.deleted_input_positions().is_empty());
        assert!(receipt.residual_glyph_positions().is_empty());
        let mut reused = empty.clear();
        reused.add('A', 9);
        reused.set_script(script::LATIN);
        reused.set_direction(Direction::LeftToRight);
        let old = font.shape(reused, &[]);
        assert_eq!(old.shaping_status(), ShapingStatus::Complete);
        assert_eq!(old.glyph_infos()[0].cluster, 9);
        assert_eq!(old_positions(&old), [(1001, 0, 0, 0)]);
    });
}

#[derive(Debug)]
struct OwnedSupplierError {
    marker: Box<u8>,
}
impl core::fmt::Display for OwnedSupplierError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("owned supplier error")
    }
}
impl core::error::Error for OwnedSupplierError {}

#[test]
fn supplier_error_source_retains_the_original_owned_error_without_string_flattening() {
    font::with_font(true, true, None, None, &[], |font| {
        let marker = Box::new(7u8);
        let original = core::ptr::from_ref(marker.as_ref());
        let mut supplied = Some(OwnedSupplierError { marker });
        let source = input("A", Direction::TopToBottom);
        let plan = make_plan(font, &source, &[]);
        let error = failure(font.shape_vertical_with_metrics(
            &plan,
            source,
            &[],
            VerticalOriginReference::NominalVerticalPenX { offset_twice: 0 },
            |_| {
                Err::<VerticalNominalMetrics, _>(
                    supplied
                        .take()
                        .expect("first actual supplier failure is retained"),
                )
            },
        ));
        let retained = core::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<OwnedSupplierError>()
            .unwrap();
        assert!(core::ptr::eq(retained.marker.as_ref(), original));
        assert!(matches!(
            &error,
            VerticalShapeError::Supplier { glyph_id: 35, .. }
        ));
        let VerticalShapeError::Supplier { error: owned, .. } = error else {
            unreachable!()
        };
        assert!(core::ptr::eq(owned.marker.as_ref(), original));
        assert_eq!(*owned.marker, 7);
    });
}

#[path = "vertical_fallback_origin_tests.rs"]
mod fallback_origin;

#[path = "vertical_reference_tests.rs"]
mod reference_tests;

#[path = "components_tests.rs"]
mod components;
