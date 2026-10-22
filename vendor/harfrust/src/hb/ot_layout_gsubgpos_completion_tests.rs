use super::*;
use crate::hb::completion_fixture::{glyphs, gsub_lookup, with_font, with_gsub, words};
use crate::{ShapingStatus, UnicodeBuffer};

#[test]
fn matching_context_capacity_distinguishes_real_match_and_limit() {
    with_font(|font| {
        for count in [64, 65] {
            let mut buffer = glyphs(count);
            let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, font, &mut buffer);
            ctx.update_matchers();
            let mut end = 0;
            assert_eq!(
                match_input(&mut ctx, count as u16 - 1, match_always, &mut end, None),
                count == 64
            );
            assert_eq!(
                ctx.buffer.shaping_status(),
                if count == 64 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::ContextLimit)
                }
            );
        }
        let mut buffer = glyphs(1);
        let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, font, &mut buffer);
        ctx.update_matchers();
        assert!(!match_input(&mut ctx, 1, match_always, &mut 0, None));
        assert_eq!(ctx.buffer.shaping_status(), ShapingStatus::Complete);
    });
}

#[test]
fn lookup_nesting_work_and_absent_lookup_have_distinct_outcomes() {
    with_font(|font| {
        for (nesting, work, expected) in [
            (
                0,
                1,
                ShapingStatus::Refused(ShapingRefusal::LookupNestingLimit),
            ),
            (1, 0, ShapingStatus::Refused(ShapingRefusal::WorkLimit)),
            (1, 1, ShapingStatus::Complete),
        ] {
            let mut buffer = glyphs(1);
            buffer.max_ops = work;
            let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, font, &mut buffer);
            ctx.nesting_level_left = nesting;
            assert!(ctx.recurse(u16::MAX).is_none());
            assert_eq!(ctx.buffer.shaping_status(), expected);
            assert_eq!(ctx.nesting_level_left, nesting);
        }
    });
}

#[test]
fn exact_zero_only_refuses_when_another_context_record_is_skipped() {
    with_font(|font| {
        let record = SequenceLookupRecord {
            sequence_index: 0.into(),
            lookup_list_index: u16::MAX.into(),
        };
        for count in [1, 2] {
            let mut buffer = glyphs(1);
            buffer.clear_output();
            buffer.max_ops = 1;
            let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, font, &mut buffer);
            ctx.match_positions[0] = 0;
            ctx.match_positions_len = 1;
            apply_lookup(&mut ctx, 0, 1, &[record; 2][..count]);
            assert_eq!(
                ctx.buffer.shaping_status(),
                if count == 1 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                }
            );
            // The diagnostic must not turn an old unlatched skip into a stop flag.
            assert!(ctx.buffer.successful);
        }
    });
}

#[test]
fn actual_multiple_substitution_can_overflow_context_after_partial_work() {
    let gsub = gsub_lookup(2, &words(&[1, 14, 1, 8, 2, 1, 1, 1, 1, 1]));
    with_gsub(&gsub, |font| {
        for count in [63, 64] {
            let mut buffer = glyphs(count);
            buffer.clear_output();
            let mut ctx = hb_ot_apply_context_t::new(TableIndex::GSUB, font, &mut buffer);
            ctx.update_matchers();
            ctx.match_positions = (0..count as u32).collect();
            ctx.match_positions_len = count;
            let record = SequenceLookupRecord {
                sequence_index: 0.into(),
                lookup_list_index: 0.into(),
            };
            apply_lookup(&mut ctx, count - 1, count, &[record]);
            assert_eq!(
                ctx.buffer.backtrack_len() + ctx.buffer.lookahead_len(),
                count + 1
            );
            assert_eq!(
                ctx.buffer.shaping_status(),
                if count == 63 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::ContextLimit)
                }
            );
            assert!(ctx.buffer.successful);
        }
    });
}

#[test]
fn real_recursive_font_returns_nonempty_partial_result_with_sticky_refusal() {
    let mut gid = 0;
    with_font(|font| gid = font.get_nominal_glyph('A' as u32).unwrap().to_u32() as u16);
    let gsub = gsub_lookup(5, &words(&[3, 1, 1, 12, 0, 0, 1, 1, gid]));
    with_gsub(&gsub, |font| {
        let mut input = UnicodeBuffer::new();
        input.push_str("A");
        input.guess_segment_properties();
        let result = font.shape(input, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result.glyph_infos()[0].glyph_id, u32::from(gid));
        assert_eq!(
            result.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::LookupNestingLimit)
        );
        assert_eq!(result.0.max_ops, hb_buffer_t::MAX_OPS_DEFAULT);
        // A missing optional table/no-match is not a resource refusal.
        let mut next = result.clear();
        next.push_str("B");
        next.guess_segment_properties();
        assert_eq!(
            font.shape(next, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}
