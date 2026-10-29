use super::*;
use crate::{BufferFlags, Direction, GlyphBuffer, UnicodeBuffer};
use alloc::{string::ToString, vec, vec::Vec};

#[path = "default_ignorable_fixture.rs"]
mod font;

fn shape(
    shaper: &crate::Shaper<'_>,
    text: &str,
    flags: BufferFlags,
    direction: Direction,
) -> GlyphBuffer {
    let mut input = UnicodeBuffer::new();
    input.push_str(text);
    input.set_direction(direction);
    input.set_flags(flags);
    input.guess_segment_properties();
    shaper.shape(input, &[])
}

fn receipt(buffer: &GlyphBuffer) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    let value = buffer.default_ignorable_receipt().unwrap();
    (
        value.residual_input_positions().to_vec(),
        value.deleted_input_positions().to_vec(),
        value.residual_glyph_positions().to_vec(),
    )
}

fn glyphs(buffer: &GlyphBuffer) -> Vec<(u32, i32)> {
    buffer
        .glyph_infos()
        .iter()
        .zip(buffer.glyph_positions())
        .map(|(g, p)| (g.glyph_id, p.x_advance))
        .collect()
}

#[test]
fn full_property_is_distinct_from_lookup_skipping() {
    assert_eq!(
        (0..=0x0010_ffff)
            .filter(|&cp| is_default_ignorable(cp))
            .count(),
        4174
    );
    for cp in [0x180f, 0x115f, 0x1160, 0x3164, 0xffa0, 0x1bca0, 0xe0fff] {
        assert!(is_default_ignorable(cp));
    }
    for cp in [
        0x0600, 0x06dd, 0x070f, 0x0890, 0xfff9, 0xfffa, 0x13430, 0x1343f,
    ] {
        assert!(!is_default_ignorable(cp));
    }
}

#[test]
fn real_nominal_coverage_does_not_authorize_fallback_output() {
    for cp in ['\u{2065}', '\u{180f}', '\u{e0100}', '\u{1bca0}'] {
        for mapped in [false, true] {
            font::with_font(mapped.then_some(cp), &[], None, false, |shaper| {
                assert_eq!(
                    glyphs(&shape(
                        shaper,
                        "ABR ",
                        BufferFlags::empty(),
                        Direction::LeftToRight
                    )),
                    vec![(35, 1000), (36, 2000), (58, 1500), (274, 1000)]
                );
                for direction in [Direction::LeftToRight, Direction::RightToLeft] {
                    let result = shape(
                        shaper,
                        &alloc::format!("A{cp}B"),
                        BufferFlags::empty(),
                        direction,
                    );
                    // Mark selectors stay with A's grapheme under reversal:
                    // B, A, selector. Format controls remain separate groups.
                    let slot = if direction == Direction::RightToLeft
                        && matches!(cp, '\u{180f}' | '\u{e0100}')
                    {
                        2
                    } else {
                        1
                    };
                    assert_eq!(
                        receipt(&result),
                        (vec![1], vec![], vec![slot as u32]),
                        "{cp:?} {direction:?}"
                    );
                    assert_eq!(result.glyph_positions()[slot].x_advance, 0);
                    assert_eq!(result.glyph_infos()[slot].glyph_id, u32::from(font::SPACE));
                }
            });
        }
    }
}

#[test]
fn actual_gsub_descendants_are_classified_only_at_final_output() {
    for (lookups, expected, deleted) in [
        (vec![font::multiple(font::D, &[])], vec![], true),
        (
            vec![
                font::multiple(font::D, &[font::R]),
                font::multiple(font::R, &[]),
            ],
            vec![],
            true,
        ),
        (
            vec![
                font::multiple(font::D, &[font::R, font::Q]),
                font::multiple(font::Q, &[]),
            ],
            vec![(58, 1500)],
            false,
        ),
        (
            vec![
                font::multiple(font::D, &[font::R, font::Q]),
                font::multiple(font::Q, &[]),
                font::multiple(font::R, &[]),
            ],
            vec![],
            true,
        ),
        (
            vec![font::multiple(font::D, &[font::D])],
            vec![(100, 700)],
            false,
        ),
    ] {
        font::with_font(Some('\u{2065}'), &lookups, None, false, |shaper| {
            // Ordinary C maps to the identical input glyph and qualifies real table behavior.
            assert_eq!(
                glyphs(&shape(
                    shaper,
                    "C",
                    BufferFlags::empty(),
                    Direction::LeftToRight
                )),
                expected
            );
            let result = shape(
                shaper,
                "\u{2065}",
                BufferFlags::empty(),
                Direction::LeftToRight,
            );
            assert_eq!(glyphs(&result), expected);
            assert_eq!(
                receipt(&result),
                (vec![], if deleted { vec![0] } else { vec![] }, vec![])
            );
        });
    }
}

#[test]
fn joint_members_transfer_but_skipped_members_do_not() {
    for (second, expected, residual) in [
        (font::D, vec![(58, 1500), (36, 2000)], false),
        (font::B, vec![(58, 1500), (274, 0)], true),
    ] {
        font::with_font(
            Some('\u{2065}'),
            &[font::ligature(font::A, second)],
            None,
            false,
            |shaper| {
                let result = shape(
                    shaper,
                    "A\u{2065}B",
                    BufferFlags::empty(),
                    Direction::LeftToRight,
                );
                assert_eq!(glyphs(&result), expected);
                assert_eq!(
                    receipt(&result),
                    if residual {
                        (vec![1], vec![], vec![1])
                    } else {
                        (vec![], vec![], vec![])
                    }
                );
            },
        );
    }
    font::with_font(
        Some('\u{2065}'),
        &[
            font::ligature(font::A, font::D),
            font::multiple(font::R, &[]),
        ],
        None,
        false,
        |shaper| {
            let result = shape(
                shaper,
                "A\u{2065}",
                BufferFlags::empty(),
                Direction::LeftToRight,
            );
            assert!(result.is_empty());
            assert_eq!(receipt(&result), (vec![], vec![1], vec![]));
        },
    );
}

#[test]
fn supported_uvs_consumes_only_its_selector() {
    for supported in [false, true] {
        font::with_font(None, &[], None, supported, |shaper| {
            let result = shape(
                shaper,
                "A\u{fe0f}",
                BufferFlags::empty(),
                Direction::LeftToRight,
            );
            assert_eq!(
                glyphs(&result),
                if supported {
                    vec![(58, 1500)]
                } else {
                    vec![(35, 1000), (274, 0)]
                }
            );
            assert_eq!(
                receipt(&result),
                if supported {
                    (vec![], vec![], vec![])
                } else {
                    (vec![1], vec![], vec![1])
                }
            );
        });
    }
}

#[test]
fn four_filler_exception_requires_real_nonzero_nominal_resource() {
    for cp in ['\u{115f}', '\u{1160}', '\u{3164}', '\u{ffa0}'] {
        for mapped in [false, true] {
            font::with_font(mapped.then_some(cp), &[], None, false, |shaper| {
                let result = shape(
                    shaper,
                    &cp.to_string(),
                    BufferFlags::empty(),
                    Direction::LeftToRight,
                );
                if mapped {
                    assert_eq!(glyphs(&result), vec![(100, 700)]);
                    assert_eq!(receipt(&result), (vec![], vec![], vec![]));
                } else {
                    assert_eq!(receipt(&result), (vec![0], vec![], vec![0]));
                    assert_eq!(result.glyph_positions()[0].x_advance, 0);
                }
            });
        }
    }
}

#[test]
fn complete_morx_contextual_deletion_retires_actual_carrier() {
    let morx = font::morx(&[font::contextual(font::D, 0xffff)]);
    font::with_font(Some('\u{2065}'), &[], Some(&morx), false, |shaper| {
        assert_eq!(
            glyphs(&shape(
                shaper,
                "ACB",
                BufferFlags::empty(),
                Direction::LeftToRight
            )),
            vec![(35, 1000), (36, 2000)]
        );
        let result = shape(
            shaper,
            "A\u{2065}B",
            BufferFlags::empty(),
            Direction::LeftToRight,
        );
        assert_eq!(glyphs(&result), vec![(35, 1000), (36, 2000)]);
        assert_eq!(receipt(&result), (vec![], vec![1], vec![]));
    });
}

#[test]
fn complete_morx_insertion_splits_normal_input_and_residual_anchor() {
    for marked in [false, true] {
        for delete_inserted in [false, true] {
            let mut subtables = vec![font::insertion(marked)];
            if delete_inserted {
                subtables.push(font::contextual(font::R, 0xffff));
            }
            let morx = font::morx(&subtables);
            font::with_font(Some('\u{2065}'), &[], Some(&morx), false, |shaper| {
                let literal = shape(shaper, "C", BufferFlags::empty(), Direction::LeftToRight);
                assert_eq!(
                    glyphs(&literal),
                    if delete_inserted {
                        vec![(100, 700)]
                    } else {
                        vec![(58, 1500), (100, 700)]
                    }
                );
                let result = shape(
                    shaper,
                    "\u{2065}",
                    BufferFlags::empty(),
                    Direction::LeftToRight,
                );
                assert_eq!(
                    glyphs(&result),
                    if delete_inserted {
                        vec![(274, 0)]
                    } else {
                        vec![(58, 1500), (274, 0)]
                    }
                );
                assert_eq!(
                    receipt(&result),
                    if delete_inserted {
                        (vec![0], vec![], vec![0])
                    } else {
                        (vec![], vec![], vec![1])
                    }
                );
            });
        }
    }
}

#[test]
fn complete_morx_ligature_transfers_exact_components_and_can_later_disappear() {
    for deleted in [false, true] {
        let mut subtables = vec![font::morx_ligature(font::D)];
        if deleted {
            subtables.push(font::contextual(font::R, 0xffff));
        }
        let morx = font::morx(&subtables);
        font::with_font(Some('\u{2065}'), &[], Some(&morx), false, |shaper| {
            let control = shape(shaper, "AC", BufferFlags::empty(), Direction::LeftToRight);
            assert_eq!(
                glyphs(&control),
                if deleted { vec![] } else { vec![(58, 1500)] }
            );
            let result = shape(
                shaper,
                "A\u{2065}",
                BufferFlags::empty(),
                Direction::LeftToRight,
            );
            assert_eq!(glyphs(&result), glyphs(&control));
            assert_eq!(
                receipt(&result),
                (vec![], if deleted { vec![1] } else { vec![] }, vec![])
            );
        });
    }
}

#[test]
fn body_ordinals_survive_equal_clusters_context_reversal_and_reuse() {
    font::with_font(None, &[], None, false, |shaper| {
        let mut input = UnicodeBuffer::new();
        input.add('\u{e0100}', 7);
        input.add('A', 7);
        input.add('\u{2065}', 7);
        input.set_pre_context("\u{2065}");
        input.set_post_context("\u{2065}");
        input.set_direction(Direction::RightToLeft);
        input.guess_segment_properties();
        let result = shaper.shape(input, &[]);
        assert_eq!(receipt(&result), (vec![0, 2], vec![], vec![0, 2]));
        assert_eq!(receipt(&result), receipt(&result));
        let _ = result.serialize(shaper, crate::SerializeFlags::empty());
        let mut input = result.clear();
        input.push_str("ABC");
        input.reset_clusters();
        input.guess_segment_properties();
        let result = shaper.shape(input, &[]);
        assert_eq!(receipt(&result), (vec![], vec![], vec![]));
        let mut input = result.clear();
        input.push_str("\u{2065}");
        input.clear();
        input.guess_segment_properties();
        let empty = shaper.shape(input, &[]);
        assert!(empty.is_empty());
        assert_eq!(receipt(&empty), (vec![], vec![], vec![]));
    });
}

#[test]
fn explicit_remove_preserve_and_precedence_have_defined_receipts() {
    font::with_font(Some('\u{2065}'), &[], None, false, |shaper| {
        let removed = shape(
            shaper,
            "\u{2065}",
            BufferFlags::REMOVE_DEFAULT_IGNORABLES,
            Direction::LeftToRight,
        );
        assert!(removed.is_empty());
        assert_eq!(receipt(&removed), (vec![], vec![0], vec![]));
        for flags in [
            BufferFlags::PRESERVE_DEFAULT_IGNORABLES,
            BufferFlags::PRESERVE_DEFAULT_IGNORABLES | BufferFlags::REMOVE_DEFAULT_IGNORABLES,
        ] {
            let result = shape(shaper, "\u{2065}", flags, Direction::LeftToRight);
            assert_eq!(glyphs(&result), vec![(100, 700)]);
            assert_eq!(receipt(&result), (vec![0], vec![], vec![0]));
        }
    });
}

#[test]
fn logical_discovery_union_and_finalization_refuse_without_admitting_receipt() {
    let mut input = UnicodeBuffer::new();
    input.push_str("\u{2065}\u{2065}");
    input.0.enter();
    input.0.max_ops = 0;
    assert!(!begin(&mut input.0));
    assert_eq!(
        input.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::WorkLimit)
    );
    input.clear();
    input.push_str("\u{2065}\u{2065}");
    input.0.enter();
    assert!(begin(&mut input.0));
    let (a, b) = (
        input.0.info[0].default_ignorable,
        input.0.info[1].default_ignorable,
    );
    input.0.max_ops = 0;
    assert!(union(&mut input.0, a, b).is_none());
    assert_eq!(input.0.info[0].default_ignorable, a);
    assert_eq!(input.0.info[1].default_ignorable, b);
    let result = GlyphBuffer(input.0);
    assert!(matches!(
        result.default_ignorable_receipt(),
        Err(ShapingRefusal::WorkLimit)
    ));
    let mut input = result.clear();
    input.push_str("\u{2065}");
    input.0.enter();
    assert!(begin(&mut input.0));
    input.0.max_ops = 0;
    finish(&mut input.0);
    assert!(matches!(
        GlyphBuffer(input.0).default_ignorable_receipt(),
        Err(ShapingRefusal::WorkLimit)
    ));
}

#[test]
fn row_layout_and_frozen_aggregate_allowance_include_di_storage() {
    assert_eq!(size_of::<GlyphInfo>(), 32);
    assert_eq!(size_of::<super::super::buffer::GlyphPosition>(), 32);
    let mut input = UnicodeBuffer::new();
    input.push_str("\u{2065}");
    input.0.enter();
    assert!(begin(&mut input.0));
    let bytes = input.0.default_ignorables.bytes().unwrap();
    assert!(bytes > 0);
    let row_storage = input.0.info.capacity() * size_of::<GlyphInfo>()
        + input.0.pos.capacity() * size_of::<super::super::buffer::GlyphPosition>();
    input.0.max_len = (row_storage + bytes).div_ceil(48);
    assert!(super::super::original_joining::room(&mut input.0, 1, 0));
    assert!(!super::super::original_joining::room(&mut input.0, 1, 48));
    assert_eq!(
        input.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
}

#[test]
fn real_single_alternate_and_reverse_substitution_are_normal_witnesses() {
    use crate::hb::completion_fixture::words;
    for lookup in [
        (1, words(&[2, 8, 1, font::R, 1, 1, font::D])),
        (3, words(&[1, 12, 1, 8, 1, font::R, 1, 1, font::D])),
        (8, words(&[1, 12, 0, 0, 1, font::R, 1, 1, font::D])),
    ] {
        font::with_font(Some('\u{2065}'), &[lookup], None, false, |shaper| {
            let literal = shape(shaper, "C", BufferFlags::empty(), Direction::LeftToRight);
            assert_eq!(glyphs(&literal), vec![(58, 1500)]);
            let result = shape(
                shaper,
                "\u{2065}",
                BufferFlags::empty(),
                Direction::LeftToRight,
            );
            assert_eq!(glyphs(&result), vec![(58, 1500)]);
            assert_eq!(receipt(&result), (vec![], vec![], vec![]));
        });
    }
}

#[test]
fn complete_morx_skipped_carrier_is_not_a_ligature_component() {
    let morx = font::morx(&[font::morx_ligature(font::B)]);
    font::with_font(Some('\u{2065}'), &[], Some(&morx), false, |shaper| {
        let literal = shape(shaper, "ACB", BufferFlags::empty(), Direction::LeftToRight);
        assert_eq!(glyphs(&literal), vec![(58, 1500), (100, 700)]);
        let result = shape(
            shaper,
            "A\u{2065}B",
            BufferFlags::empty(),
            Direction::LeftToRight,
        );
        assert_eq!(glyphs(&result), vec![(58, 1500), (274, 0)]);
        assert_eq!(receipt(&result), (vec![1], vec![], vec![1]));
    });
}

#[test]
fn complete_morx_noncontextual_replacement_and_deletion_are_native_operations() {
    use crate::hb::completion_fixture::words;
    for output in [font::R, 0xffff] {
        let morx = font::morx(&[(4, words(&[8, font::D, 1, output]))]);
        font::with_font(Some('\u{2065}'), &[], Some(&morx), false, |shaper| {
            let literal = shape(shaper, "C", BufferFlags::empty(), Direction::LeftToRight);
            assert_eq!(
                glyphs(&literal),
                if output == 0xffff {
                    vec![]
                } else {
                    vec![(58, 1500)]
                }
            );
            let result = shape(
                shaper,
                "\u{2065}",
                BufferFlags::empty(),
                Direction::LeftToRight,
            );
            assert_eq!(glyphs(&result), glyphs(&literal));
            assert_eq!(
                receipt(&result),
                (
                    vec![],
                    if output == 0xffff { vec![0] } else { vec![] },
                    vec![]
                )
            );
        });
    }
}

#[test]
fn real_synthetic_dotted_circle_does_not_authorize_its_di_anchor() {
    font::with_font(Some('\u{034f}'), &[], None, false, |shaper| {
        let result = shape(
            shaper,
            "\u{034f}",
            BufferFlags::BEGINNING_OF_TEXT,
            Direction::LeftToRight,
        );
        assert_eq!(glyphs(&result), vec![(277, 2500), (274, 0)]);
        assert_eq!(receipt(&result), (vec![0], vec![], vec![1]));
    });
}

#[test]
fn aggregate_budget_combines_joining_di_cc_and_relocation_storage() {
    use crate::{script, JoiningSource};
    fn prepared(joining: bool, di: bool) -> UnicodeBuffer {
        let text = "\u{1}\u{2065}A\u{2065}";
        let mut input = UnicodeBuffer::new();
        input.push_str(text);
        input.set_direction(Direction::LeftToRight);
        input.set_script(script::LATIN);
        input.set_ordinary_symbol_positions(&[0]).unwrap();
        if joining {
            let source =
                JoiningSource::analyze(text, script::LATIN, Direction::LeftToRight, &[0]).unwrap();
            input
                .set_joining_sources(
                    &source,
                    &[Some(0), Some(1), Some(2), Some(3)],
                    &source,
                    &[0, 1, 2, 3],
                )
                .unwrap();
        }
        input.0.enter();
        if di {
            assert!(begin(&mut input.0));
        }
        if joining {
            crate::hb::original_joining::begin(&mut input.0);
        }
        input
    }
    fn minimum(joining: bool, di: bool) -> usize {
        (1..200)
            .find(|&limit| {
                let mut input = prepared(joining, di);
                input.0.max_len = limit;
                crate::hb::original_joining::room(&mut input.0, 4, 0)
            })
            .unwrap()
    }
    let combined = minimum(true, true);
    assert!(combined > minimum(true, false));
    assert!(combined > minimum(false, true));
    let mut input = prepared(true, true);
    input.0.max_len = combined;
    assert!(crate::hb::original_joining::room(&mut input.0, 4, 0));
    let capacities = (input.0.info.capacity(), input.0.pos.capacity());
    // The retained old rows and new rows overlap during actual reserve.
    assert!(!input.0.ensure(8));
    assert_eq!(
        (input.0.info.capacity(), input.0.pos.capacity()),
        capacities
    );
    assert_eq!(
        input.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
}

// Real complete font + normal native pipeline; stop only at its publication
// boundary so tests can inject the existing private resource limits there.
fn before_receipt(shaper: &crate::Shaper<'_>, text: &str) -> hb_buffer_t {
    use crate::hb::ot_shape::{hb_ot_shape_context_t, shape_internal};
    let mut input = UnicodeBuffer::new();
    input.push_str(text);
    input.set_direction(Direction::LeftToRight);
    input.guess_segment_properties();
    let mut buffer = input.0;
    buffer.enter();
    assert!(begin(&mut buffer));
    let plan = crate::ShapePlan::new(shaper, buffer.direction, buffer.script, None, &[]);
    shape_internal(&mut hb_ot_shape_context_t {
        plan: &plan,
        face: shaper,
        buffer: &mut buffer,
        target_direction: Direction::LeftToRight,
        features: &[],
    });
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    buffer
}

#[test]
fn actual_finalization_refuses_atomically_for_live_and_all_deleted_outputs() {
    for deleted in [false, true] {
        let lookups = if deleted {
            vec![font::multiple(font::D, &[])]
        } else {
            vec![]
        };
        font::with_font(Some('\u{2065}'), &lookups, None, false, |shaper| {
            for growth in [false, true] {
                let mut buffer = before_receipt(shaper, "\u{2065}");
                assert_eq!(buffer.len, usize::from(!deleted));
                if growth {
                    buffer.max_len = 0;
                } else {
                    buffer.max_ops = 0;
                }
                finish(&mut buffer);
                let expected = if growth {
                    ShapingRefusal::GrowthLimit
                } else {
                    ShapingRefusal::WorkLimit
                };
                assert_eq!(buffer.shaping_status(), ShapingStatus::Refused(expected));
                assert!(!buffer.default_ignorables.finalized);
                assert!(buffer.default_ignorables.residual_inputs.is_empty());
                assert!(buffer.default_ignorables.deleted_inputs.is_empty());
                assert!(buffer.default_ignorables.residual_glyphs.is_empty());
                buffer.refuse(ShapingRefusal::ContextLimit);
                assert!(
                    matches!(GlyphBuffer(buffer).default_ignorable_receipt(),Err(reason) if reason==expected)
                );
            }
        });
    }
}

#[test]
fn first_refusal_denies_receipt_even_with_real_plausible_partial_glyphs() {
    font::with_font(None, &[], None, false, |shaper| {
        let mut input = UnicodeBuffer::new();
        input.push_str("A");
        input.guess_segment_properties();
        input.0.max_len = 1;
        assert!(!input.reserve(2));
        let result = shaper.shape(input, &[]);
        assert_eq!(glyphs(&result), vec![(35, 1000)]);
        assert_eq!(
            result.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
        );
        assert!(matches!(
            result.default_ignorable_receipt(),
            Err(ShapingRefusal::GrowthLimit)
        ));
    });
}

#[test]
fn union_growth_and_deletion_work_preflight_preserve_carrier_custody() {
    let mut input = UnicodeBuffer::new();
    input.push_str("\u{2065}\u{2065}");
    input.0.enter();
    assert!(begin(&mut input.0));
    let (a, b) = (
        input.0.info[0].default_ignorable,
        input.0.info[1].default_ignorable,
    );
    let rows = input.0.info.capacity() * size_of::<GlyphInfo>()
        + input.0.pos.capacity() * size_of::<super::super::buffer::GlyphPosition>();
    let bytes = input.0.default_ignorables.bytes().unwrap();
    input.0.max_len = (rows + bytes).div_ceil(48);
    assert!(union(&mut input.0, a, b).is_none());
    assert_eq!(
        input.0.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
    assert_eq!(input.0.info[0].default_ignorable, a);
    assert_eq!(input.0.info[1].default_ignorable, b);
    let mut input = GlyphBuffer(input.0).clear();
    input.push_str("\u{2065}");
    input.0.enter();
    assert!(begin(&mut input.0));
    input.0.clear_output();
    input.0.max_ops = 0;
    input.0.delete_glyph();
    assert_eq!((input.0.idx, input.0.out_len), (1, 0));
    assert_eq!(input.0.info[0].default_ignorable, 1);
    assert!(matches!(
        GlyphBuffer(input.0).default_ignorable_receipt(),
        Err(ShapingRefusal::WorkLimit)
    ));
}

#[test]
fn work_refusal_keeps_native_preprocessing_and_selector_tail_progress_bounded() {
    use crate::hb::ot_shape::{hb_ot_shape_context_t, shape_internal};
    use crate::script;
    font::with_font(None, &[], None, false, |shaper| {
        for (text, script) in [
            ("A\u{e0100}\u{e0101}", script::LATIN),
            ("\u{0905}\u{093e}", script::DEVANAGARI),
            ("\u{302e}\u{115f}\u{1160}", script::HANGUL),
            ("\u{0e33}", script::THAI),
        ] {
            for budget in [0, 1, 2, 3, 4, 8, 16, 32] {
                let mut input = UnicodeBuffer::new();
                input.push_str(text);
                input.set_script(script);
                input.set_direction(Direction::LeftToRight);
                let mut buffer = input.0;
                buffer.enter();
                assert!(begin(&mut buffer));
                buffer.max_ops = budget;
                let plan =
                    crate::ShapePlan::new(shaper, buffer.direction, buffer.script, None, &[]);
                shape_internal(&mut hb_ot_shape_context_t {
                    plan: &plan,
                    face: shaper,
                    buffer: &mut buffer,
                    target_direction: Direction::LeftToRight,
                    features: &[],
                });
                finish(&mut buffer);
                let result = GlyphBuffer(buffer);
                if budget == 0 {
                    assert_eq!(
                        result.shaping_status(),
                        ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                    );
                }
                match result.shaping_status() {
                    ShapingStatus::Complete => {
                        let _ = result.default_ignorable_receipt().unwrap();
                    }
                    ShapingStatus::Refused(reason) => assert!(
                        matches!(result.default_ignorable_receipt(),Err(actual) if actual==reason)
                    ),
                }
            }
        }
    });
}

#[test]
fn logical_growth_refusal_preserves_progress_through_generated_repair_loops() {
    use crate::hb::ot_shape::{hb_ot_shape_context_t, shape_internal};
    font::with_font(None, &[], None, false, |shaper| {
        let text = "\u{0905}\u{093e}".repeat(20);
        let mut input = UnicodeBuffer::new();
        input.push_str(&text);
        input.set_script(crate::script::DEVANAGARI);
        input.set_direction(Direction::LeftToRight);
        let mut buffer = input.0;
        buffer.enter();
        assert!(begin(&mut buffer));
        let rows = buffer.info.capacity() * size_of::<GlyphInfo>()
            + buffer.pos.capacity() * size_of::<super::super::buffer::GlyphPosition>();
        buffer.max_len = rows.div_ceil(48);
        let plan = crate::ShapePlan::new(shaper, buffer.direction, buffer.script, None, &[]);
        shape_internal(&mut hb_ot_shape_context_t {
            plan: &plan,
            face: shaper,
            buffer: &mut buffer,
            target_direction: Direction::LeftToRight,
            features: &[],
        });
        finish(&mut buffer);
        assert_eq!(
            buffer.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
        );
        assert!(matches!(
            GlyphBuffer(buffer).default_ignorable_receipt(),
            Err(ShapingRefusal::GrowthLimit)
        ));
    });
}

#[test]
fn review_hangul_decomposition_copy_refusal_is_typed() {
    use crate::hb::completion_fixture::{with_tables, words};
    use crate::hb::ot_shape::{hb_ot_shape_context_t, shape_internal};
    let mut cmap = words(&[0, 1, 3, 10]);
    cmap.extend(12u32.to_be_bytes());
    cmap.extend(words(&[12, 0]));
    for value in [40u32, 0, 2, 0x1100, 0x1100, 35, 0x1161, 0x1161, 36] {
        cmap.extend(value.to_be_bytes());
    }
    with_tables(&[(*b"cmap", &cmap)], |shaper| {
        assert_eq!(shaper.get_nominal_glyph(0xac00), None);
        assert_eq!(shaper.get_nominal_glyph(0x1100).unwrap().to_u32(), 35);
        assert_eq!(shaper.get_nominal_glyph(0x1161).unwrap().to_u32(), 36);
        for text in ["\u{1100}\u{1161}", "\u{ac00}"] {
            let mut input = UnicodeBuffer::new();
            input.push_str(text);
            input.set_script(crate::script::HANGUL);
            input.set_direction(Direction::LeftToRight);
            let result = shaper.shape(input, &[]);
            assert_eq!(result.shaping_status(), ShapingStatus::Complete);
            assert_eq!(glyphs(&result), vec![(35, 1000), (36, 1000)]);
        }
        for reason in [ShapingRefusal::WorkLimit, ShapingRefusal::GrowthLimit] {
            let mut input = UnicodeBuffer::new();
            input.push_str("\u{ac00}");
            input.set_script(crate::script::HANGUL);
            input.set_direction(Direction::LeftToRight);
            let mut buffer = input.0;
            buffer.enter();
            assert!(begin(&mut buffer));
            if reason == ShapingRefusal::WorkLimit {
                buffer.max_ops = 2;
            } else {
                buffer.max_len = 0;
            }
            let plan = crate::ShapePlan::new(shaper, buffer.direction, buffer.script, None, &[]);
            shape_internal(&mut hb_ot_shape_context_t {
                plan: &plan,
                face: shaper,
                buffer: &mut buffer,
                target_direction: Direction::LeftToRight,
                features: &[],
            });
            finish(&mut buffer);
            assert_eq!(buffer.shaping_status(), ShapingStatus::Refused(reason));
            assert!(
                matches!(GlyphBuffer(buffer).default_ignorable_receipt(), Err(actual) if actual == reason)
            );
        }
    });
}

// Retain real font-produced glyphs and DI custody while injecting the budget at
// the native table boundary; no fabricated glyph records or font attestations.
fn table_input(shaper: &crate::Shaper<'_>, text: &str) -> hb_buffer_t {
    let mut buffer = shape(
        shaper,
        text,
        BufferFlags::PRESERVE_DEFAULT_IGNORABLES,
        Direction::LeftToRight,
    )
    .0;
    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    buffer.default_ignorables.finalized = false;
    buffer.default_ignorables.residual_inputs.clear();
    buffer.default_ignorables.deleted_inputs.clear();
    buffer.default_ignorables.residual_glyphs.clear();
    // Existing capacity makes the work oracle independent of relocation.
    assert!(buffer.ensure(buffer.len + 8));
    buffer
}

#[test]
fn actual_multiple_expansion_charges_existing_di_prefix_before_mutation() {
    use crate::hb::ot_layout::TableIndex;
    use crate::hb::ot_layout_gsubgpos::{Apply, OT::hb_ot_apply_context_t};
    use read_fonts::{tables::gsub::MultipleSubstFormat1, FontData, FontRead};
    let lookup = font::multiple(font::B, &[font::R, font::Q]);
    font::with_font(
        Some('\u{2065}'),
        core::slice::from_ref(&lookup),
        None,
        false,
        |native| {
            assert_eq!(
                glyphs(&shape(
                    native,
                    "\u{2065}AAAB",
                    BufferFlags::PRESERVE_DEFAULT_IGNORABLES,
                    Direction::LeftToRight
                )),
                vec![
                    (100, 700),
                    (35, 1000),
                    (35, 1000),
                    (35, 1000),
                    (58, 1500),
                    (277, 2500)
                ]
            );
            font::with_font(Some('\u{2065}'), &[], None, false, |raw| {
                for budget in [6, 7] {
                    let mut buffer = table_input(raw, "\u{2065}AAAB");
                    buffer.clear_output();
                    buffer.next_glyphs(4);
                    buffer.max_ops = budget;
                    let carrier = buffer.out_info()[0].default_ignorable;
                    let capacities = (buffer.info.capacity(), buffer.pos.capacity());
                    let table = MultipleSubstFormat1::read(FontData::new(&lookup.1)).unwrap();
                    let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, native, &mut buffer);
                    let _ = table.apply(&mut ctx);
                    if budget == 7 {
                        assert_eq!(ctx.buffer.shaping_status(), ShapingStatus::Complete);
                        assert_eq!(
                            ctx.buffer.max_ops, 0,
                            "four copied prefix rows + two outputs + one skip"
                        );
                        assert!(ctx.buffer.have_separate_output);
                        assert_eq!(ctx.buffer.out_info()[0].default_ignorable, carrier);
                    } else {
                        assert_eq!(
                            ctx.buffer.shaping_status(),
                            ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                        );
                        assert_eq!((ctx.buffer.idx, ctx.buffer.out_len), (4, 4));
                        assert!(!ctx.buffer.have_separate_output);
                        assert_eq!(ctx.buffer.out_info()[0].default_ignorable, carrier);
                    }
                    assert_eq!(
                        (ctx.buffer.info.capacity(), ctx.buffer.pos.capacity()),
                        capacities
                    );
                }
            });
        },
    );
}

#[test]
fn actual_marked_morx_rewind_and_return_debit_every_carrier_copy() {
    let morx = font::morx(&[font::insertion_at(true, font::B)]);
    font::with_font(Some('\u{2065}'), &[], Some(&morx), false, |native| {
        assert_eq!(
            glyphs(&shape(
                native,
                "\u{2065}AAAB",
                BufferFlags::PRESERVE_DEFAULT_IGNORABLES,
                Direction::LeftToRight
            )),
            vec![
                (58, 1500),
                (100, 700),
                (35, 1000),
                (35, 1000),
                (35, 1000),
                (36, 2000)
            ]
        );
        font::with_font(Some('\u{2065}'), &[], None, false, |raw| {
            for budget in [12, 14] {
                let mut buffer = table_input(raw, "\u{2065}AAAB");
                let plan =
                    crate::ShapePlan::new(native, buffer.direction, buffer.script, None, &[]);
                let carrier = buffer.info[0].default_ignorable;
                let capacities = (buffer.info.capacity(), buffer.pos.capacity());
                buffer.max_ops = budget;
                crate::hb::aat::layout::substitute(&plan, native, &mut buffer, &[]);
                if budget == 14 {
                    assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
                    assert_eq!(
                        buffer.max_ops, 0,
                        "four traverse + one insertion + four rewind + four return + one traverse"
                    );
                    assert_eq!(buffer.info[1].default_ignorable, carrier);
                } else {
                    assert_eq!(
                        buffer.shaping_status(),
                        ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                    );
                    assert_eq!(buffer.info[0].default_ignorable, carrier);
                    assert!(!buffer.info[..buffer.len]
                        .iter()
                        .any(|g| g.glyph_id == u32::from(font::R)));
                }
                assert_eq!((buffer.info.capacity(), buffer.pos.capacity()), capacities);
            }
        });
    });
}

#[test]
fn real_gdef_ligature_or_mark_class_without_gsub_does_not_establish_normal_support() {
    use crate::hb::completion_fixture::words;
    for class in [2, 3] {
        let gdef = words(&[1, 0, 12, 0, 0, 0, 1, font::D, 1, class]);
        font::with_extra_tables(
            Some('\u{2065}'),
            &[],
            None,
            false,
            &[(*b"GDEF", &gdef)],
            |shaper| {
                assert!(shaper.ot_tables.has_glyph_classes());
                let result = shape(
                    shaper,
                    "\u{2065}",
                    BufferFlags::PRESERVE_DEFAULT_IGNORABLES,
                    Direction::LeftToRight,
                );
                assert_eq!(result.glyph_infos()[0].glyph_id, 100);
                if class == 2 {
                    assert!(result.glyph_infos()[0].is_ligature());
                } else {
                    assert!(result.glyph_infos()[0].is_mark());
                }
                assert_eq!(receipt(&result), (vec![0], vec![], vec![0]));
            },
        );
    }
}

#[test]
fn actual_contextual_gsub_forward_rewind_and_return_debit_existing_rows() {
    use crate::hb::completion_fixture::words;
    use crate::hb::ot_layout::TableIndex;
    use crate::hb::ot_layout_gsubgpos::{Apply, OT::hb_ot_apply_context_t};
    use read_fonts::{tables::layout::SequenceContextFormat3, FontData, FontRead};
    let contextual = words(&[3, 2, 2, 18, 24, 1, 1, 0, 2, 1, 1, font::A, 1, 1, font::B]);
    let lookups = vec![
        (5, contextual.clone()),
        (1, words(&[2, 8, 1, font::B, 1, 1, font::B])),
        (1, words(&[2, 8, 1, font::A, 1, 1, font::A])),
    ];
    font::with_font(Some('\u{2065}'), &lookups, None, false, |native| {
        let complete = shape(
            native,
            "\u{2065}AB",
            BufferFlags::PRESERVE_DEFAULT_IGNORABLES,
            Direction::LeftToRight,
        );
        assert_eq!(glyphs(&complete), vec![(100, 700), (35, 1000), (36, 2000)]);
        assert!(complete.glyph_infos()[1].substituted());
        assert!(complete.glyph_infos()[2].substituted());
        font::with_font(Some('\u{2065}'), &[], None, false, |raw| {
            for budget in [7, 8] {
                let mut buffer = table_input(raw, "\u{2065}AB");
                buffer.reset_masks(1);
                buffer.clear_output();
                buffer.next_glyph();
                let carrier = buffer.out_info()[0].default_ignorable;
                let capacities = (buffer.info.capacity(), buffer.pos.capacity());
                let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, native, &mut buffer);
                ctx.lookup_props = 0;
                ctx.update_matchers();
                ctx.buffer.max_ops = budget;
                let table = SequenceContextFormat3::read(FontData::new(&contextual)).unwrap();
                let _ = table.apply(&mut ctx);
                if budget == 8 {
                    assert_eq!(ctx.buffer.shaping_status(), ShapingStatus::Complete);
                    assert_eq!(
                        ctx.buffer.max_ops, 0,
                        "forward1 + rewind2 + return1 + two recursions + two replacements"
                    );
                    assert_eq!((ctx.buffer.idx, ctx.buffer.out_len), (3, 3));
                } else {
                    assert_eq!(
                        ctx.buffer.shaping_status(),
                        ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                    );
                }
                assert_eq!(ctx.buffer.out_info()[0].default_ignorable, carrier);
                assert_eq!(
                    (ctx.buffer.info.capacity(), ctx.buffer.pos.capacity()),
                    capacities
                );
            }
        });
    });
}

#[test]
fn rewind_shift_and_carrier_copy_are_admitted_together_before_any_mutation() {
    font::with_font(Some('\u{2065}'), &[], None, false, |shaper| {
        for budget in [4, 5] {
            let mut buffer = table_input(shaper, "\u{2065}AB");
            buffer.clear_output();
            buffer.copy_glyph();
            buffer.copy_glyph();
            let carrier = buffer.out_info()[0].default_ignorable;
            let capacities = (buffer.info.capacity(), buffer.pos.capacity());
            buffer.max_ops = budget;
            assert_eq!(buffer.move_to(0), budget == 5);
            if budget == 5 {
                assert_eq!(
                    buffer.max_ops, 0,
                    "three shifted input rows + two rewound output rows"
                );
                assert_eq!((buffer.idx, buffer.len, buffer.out_len), (0, 5, 0));
                assert_eq!(buffer.info[0].default_ignorable, carrier);
            } else {
                assert_eq!(
                    buffer.shaping_status(),
                    ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                );
                assert_eq!((buffer.idx, buffer.len, buffer.out_len), (0, 3, 2));
                assert_eq!(buffer.out_info()[0].default_ignorable, carrier);
            }
            assert_eq!((buffer.info.capacity(), buffer.pos.capacity()), capacities);
        }
    });
}

#[test]
fn actual_multiple_gap_fill_charges_the_prefix_at_the_later_mode_switch() {
    use crate::hb::completion_fixture::words;
    use crate::hb::ot_layout::TableIndex;
    use crate::hb::ot_layout_gsubgpos::{Apply, OT::hb_ot_apply_context_t};
    use read_fonts::{tables::gsub::MultipleSubstFormat1, FontData, FontRead};
    // One actual lookup deletes A and expands B. The output catches up with
    // input before the third B descendant forces the alternate-buffer prefix.
    let bytes = words(&[
        1,
        20,
        2,
        10,
        12,
        0,
        3,
        font::R,
        font::Q,
        font::Q,
        1,
        2,
        font::A,
        font::B,
    ]);
    font::with_font(
        Some('\u{2065}'),
        &[(2, bytes.clone())],
        None,
        false,
        |native| {
            let result = shape(
                native,
                "\u{2065}AAB",
                BufferFlags::PRESERVE_DEFAULT_IGNORABLES,
                Direction::LeftToRight,
            );
            assert_eq!(
                glyphs(&result),
                vec![(100, 700), (58, 1500), (277, 2500), (277, 2500)]
            );
            font::with_font(Some('\u{2065}'), &[], None, false, |raw| {
                for budget in [6, 7] {
                    let mut buffer = table_input(raw, "\u{2065}AAB");
                    buffer.clear_output();
                    buffer.next_glyph();
                    buffer.delete_glyph();
                    buffer.delete_glyph();
                    assert_eq!((buffer.idx, buffer.out_len), (3, 1));
                    let carrier = buffer.out_info()[0].default_ignorable;
                    let capacities = (buffer.info.capacity(), buffer.pos.capacity());
                    buffer.max_ops = budget;
                    let table = MultipleSubstFormat1::read(FontData::new(&bytes)).unwrap();
                    let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, native, &mut buffer);
                    let _ = table.apply(&mut ctx);
                    if budget == 7 {
                        assert_eq!(ctx.buffer.shaping_status(), ShapingStatus::Complete);
                        assert_eq!(
                            ctx.buffer.max_ops, 0,
                            "three outputs + one skip + three prefix rows at switch"
                        );
                        assert_eq!((ctx.buffer.idx, ctx.buffer.out_len), (4, 4));
                        assert!(ctx.buffer.have_separate_output);
                    } else {
                        assert_eq!(
                            ctx.buffer.shaping_status(),
                            ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                        );
                        assert_eq!((ctx.buffer.idx, ctx.buffer.out_len), (3, 1));
                        assert!(!ctx.buffer.have_separate_output);
                    }
                    assert_eq!(ctx.buffer.out_info()[0].default_ignorable, carrier);
                    assert_eq!(
                        (ctx.buffer.info.capacity(), ctx.buffer.pos.capacity()),
                        capacities
                    );
                }
            });
        },
    );
}
