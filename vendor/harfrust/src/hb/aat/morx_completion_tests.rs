use super::*;
use crate::hb::buffer::ShapingStatus;
use crate::hb::completion_fixture::{extended_machine, glyphs, with_font, words};
use crate::{Direction, ShapePlan};
use read_fonts::{FontData, FontRead};

fn plan(font: &hb_font_t<'_>) -> ShapePlan {
    ShapePlan::new(font, Direction::LeftToRight, None, None, &[])
}

#[test]
fn nonadvancing_machine_reports_only_budget_forced_progress() {
    with_font(|font| {
        let plan = plan(font);
        for flags in [0, RearrangementCtx::DONT_ADVANCE] {
            let bytes = extended_machine(&words(&[0, flags]), &words(&[0, 0]));
            let machine = ExtendedStateTable::<NoPayload>::read(FontData::new(&bytes)).unwrap();
            let mut buffer = glyphs(2);
            buffer.max_ops = 0;
            let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
            let cache = ClassCache::new();
            ac.machine_class_cache = Some(&cache);
            drive(
                &machine,
                &mut RearrangementCtx { start: 0, end: 0 },
                &mut ac,
            );
            assert_eq!(ac.buffer.idx, 2);
            assert_eq!(
                ac.buffer.shaping_status(),
                if flags == 0 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                }
            );
            assert!(ac.buffer.successful);
        }
    });
}

#[test]
fn rearrangement_capacity_preserves_actual_bounded_action_and_skips() {
    with_font(|font| {
        let plan = plan(font);
        for (count, verb) in [(64, 1), (65, 1), (65, 0), (1, 15)] {
            let mut buffer = glyphs(count);
            for (i, info) in buffer.info.iter_mut().enumerate() {
                info.glyph_id = i as u32;
            }
            let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
            let entry = StateEntry::read(FontData::new(&words(&[0, verb]))).unwrap();
            let mut ctx = RearrangementCtx {
                start: 0,
                end: count,
            };
            assert!(ctx.transition(&entry, &mut ac).is_some());
            assert_eq!(
                ac.buffer.shaping_status(),
                if count == 65 && verb != 0 {
                    ShapingStatus::Refused(ShapingRefusal::ContextLimit)
                } else {
                    ShapingStatus::Complete
                }
            );
            if count == 64 {
                assert_eq!(ac.buffer.info[count - 1].glyph_id, 0);
            }
            if count == 65 {
                assert_eq!(ac.buffer.info[0].glyph_id, 0);
            }
            assert!(ac.buffer.successful);
        }
    });
}

#[test]
fn marked_and_current_insertions_keep_their_distinct_zero_boundaries() {
    with_font(|font| {
        let plan = plan(font);
        let inserted = [BigEndian::from(GlyphId16::new(2))];
        for marked in [false, true] {
            for work in [0, 1, 2] {
                let mut buffer = glyphs(1);
                buffer.clear_output();
                buffer.max_ops = work;
                let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
                let mut ctx = InsertionCtx {
                    mark: 0,
                    glyphs: &inserted,
                };
                let entry = StateEntry {
                    new_state: 0,
                    flags: if marked {
                        1 | InsertionCtx::MARKED_INSERT_BEFORE
                    } else {
                        (1 << 5) | InsertionCtx::CURRENT_INSERT_BEFORE
                    },
                    payload: InsertionEntryData {
                        current_insert_index: (if marked { u16::MAX } else { 0 }).into(),
                        marked_insert_index: (if marked { 0 } else { u16::MAX }).into(),
                    },
                };
                assert!(ctx.transition(&entry, &mut ac).is_some());
                let refused = if marked { work <= 1 } else { work < 1 };
                assert_eq!(
                    ac.buffer.shaping_status(),
                    if refused {
                        ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                    } else {
                        ShapingStatus::Complete
                    }
                );
                assert_eq!(ac.buffer.out_len, usize::from(!refused));
                if !refused {
                    assert_eq!(ac.buffer.out_info()[0].glyph_id, 2);
                }
                assert!(ac.buffer.successful);
            }
        }
    });
}

fn ligature_context<'a>(
    machine: ExtendedStateTable<'a, BigEndian<u16>>,
    actions: &'a [BigEndian<u32>],
    components: &'a [BigEndian<u16>],
    ligatures: &'a [BigEndian<GlyphId16>],
) -> LigatureCtx<'a> {
    LigatureCtx {
        table: LigatureSubtable {
            state_table: machine,
            ligature_actions: actions,
            components,
            ligatures,
        },
        match_length: 0,
        match_positions: [0; LIGATURE_MAX_MATCHES],
        match_indices: [None; LIGATURE_MAX_MATCHES],
    }
}

fn capture(ctx: &mut LigatureCtx<'_>, ac: &mut AatApplyContext<'_>, count: usize) {
    let entry = StateEntry {
        new_state: 0,
        flags: LigatureCtx::SET_COMPONENT,
        payload: 0.into(),
    };
    for _ in 0..count {
        assert!(ctx.transition(&entry, ac).is_some());
        ac.buffer.next_glyph();
    }
}

#[test]
fn component_ring_refuses_used_stale_history_not_unused_overcapture() {
    with_font(|font| {
        let plan = plan(font);
        let bytes = extended_machine(&words(&[0, 0, 0]), &words(&[0, 0, 0]));
        let machine = ExtendedStateTable::read(FontData::new(&bytes)).unwrap();
        let components = [BigEndian::from(0u16); 3];
        let ligatures = [BigEndian::from(GlyphId16::new(2))];
        for action_count in [0, 1, 64, 65] {
            let mut actions = alloc::vec![BigEndian::from(0u32); action_count.max(1)];
            actions[action_count.max(1) - 1] = LigatureCtx::LIG_ACTION_LAST.into();
            let mut ctx = ligature_context(machine.clone(), &actions, &components, &ligatures);
            let mut buffer = glyphs(70);
            buffer.clear_output();
            let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
            capture(&mut ctx, &mut ac, 65);
            assert_eq!(ac.buffer.shaping_status(), ShapingStatus::Complete);
            if action_count != 0 {
                let entry = StateEntry {
                    new_state: 0,
                    flags: LigatureCtx::PERFORM_ACTION,
                    payload: 0.into(),
                };
                assert!(ctx.transition(&entry, &mut ac).is_some());
            }
            assert_eq!(
                ac.buffer.shaping_status(),
                if action_count == 65 {
                    ShapingStatus::Refused(ShapingRefusal::ComponentHistoryLimit)
                } else {
                    ShapingStatus::Complete
                }
            );
            assert!(ac.buffer.successful);
            assert!(ac.buffer.out_len > 0);
        }
    });
}

#[test]
fn component_stack_underflow_reuse_and_missing_action_are_defined_recovery() {
    with_font(|font| {
        let plan = plan(font);
        let bytes = extended_machine(&words(&[0, 0, 0]), &words(&[0, 0, 0]));
        let machine = ExtendedStateTable::read(FontData::new(&bytes)).unwrap();
        let actions = [BigEndian::from(0u32); 3];
        let components = [BigEndian::from(0u16); 2];
        let mut ctx = ligature_context(machine, &actions, &components, &[]);
        let mut buffer = glyphs(8);
        buffer.clear_output();
        let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
        capture(&mut ctx, &mut ac, 2);
        let mut entry = StateEntry {
            new_state: 0,
            flags: LigatureCtx::PERFORM_ACTION,
            payload: 0.into(),
        };
        assert!(ctx.transition(&entry, &mut ac).is_some());
        assert_eq!(ctx.match_length, 0);
        capture(&mut ctx, &mut ac, 2);
        entry.payload = 100.into();
        assert!(ctx.transition(&entry, &mut ac).is_some());
        assert_eq!(ac.buffer.shaping_status(), ShapingStatus::Complete);
    });
}

#[test]
fn overcapture_partial_store_and_repush_keep_actual_history_validity() {
    with_font(|font| {
        let plan = plan(font);
        let bytes = extended_machine(&words(&[0, 0, 0]), &words(&[0, 0, 0]));
        let machine = ExtendedStateTable::read(FontData::new(&bytes)).unwrap();
        let mut actions = alloc::vec![BigEndian::from(0u32); 68];
        for end in [63, 64, 67] {
            actions[end] = LigatureCtx::LIG_ACTION_LAST.into();
        }
        let components = [BigEndian::from(0u16); 3];
        let ligatures = [BigEndian::from(GlyphId16::new(2))];
        let mut ctx = ligature_context(machine, &actions, &components, &ligatures);
        let mut buffer = glyphs(160);
        buffer.clear_output();
        let mut ac = AatApplyContext::new(&plan, font, &mut buffer);
        capture(&mut ctx, &mut ac, 65);
        let mut entry = StateEntry {
            new_state: 0,
            flags: LigatureCtx::PERFORM_ACTION,
            payload: 0.into(),
        };
        assert!(ctx.transition(&entry, &mut ac).is_some());
        assert_eq!(ctx.match_length, 2); // STORE keeps index1 and drops later stack entries.
        assert_eq!(ac.buffer.shaping_status(), ShapingStatus::Complete);
        capture(&mut ctx, &mut ac, 1);
        assert_eq!(ctx.match_length, 3);
        entry.payload = 64.into(); // Only the freshly pushed component is required.
        assert!(ctx.transition(&entry, &mut ac).is_some());
        assert_eq!(ac.buffer.shaping_status(), ShapingStatus::Complete);
        entry.payload = 65.into(); // Fresh index2, surviving index1, then overwritten index0.
        assert!(ctx.transition(&entry, &mut ac).is_some());
        assert_eq!(
            ac.buffer.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::ComponentHistoryLimit)
        );
        assert!(ac.buffer.successful);
        assert!(ac.buffer.out_len > 0);
    });
}
