use super::*;
use crate::hb::completion_fixture::{glyphs, with_font};

#[test]
fn actual_growth_boundary_and_first_refusal_survive_other_work() {
    let mut b = hb_buffer_t::new();
    b.max_len = 2;
    assert!(b.ensure(2));
    assert_eq!(b.shaping_status(), ShapingStatus::Complete);
    assert!(!b.ensure(3));
    assert!(!b.successful);
    b.len = 2;
    b.max_ops = 0;
    b.set_masks(1, 1, 0, u32::MAX);
    assert_eq!(
        b.shaping_status(),
        ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
    );
}

#[test]
fn work_charges_have_exact_zero_and_negative_boundaries() {
    for operation in 0..4 {
        for budget in [2, 1] {
            let mut b = glyphs(2);
            b.max_ops = budget;
            match operation {
                0 => b.set_masks(1, 1, 0, u32::MAX),
                1 => b.merge_clusters(0, 2),
                2 => {
                    b.clear_output();
                    b.next_glyph();
                    b.next_glyph();
                    b.merge_out_clusters(0, 2);
                }
                _ => {
                    b.clear_output();
                    assert_eq!(b.shift_forward(1), budget == 2);
                }
            }
            assert_eq!(
                b.shaping_status(),
                if budget == 2 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::WorkLimit)
                }
            );
            assert_eq!(b.successful, budget == 2);
        }
    }
}

#[test]
fn pre_shape_refusal_survives_actual_return_and_clear_reuses_cleanly() {
    with_font(|font| {
        let mut input = UnicodeBuffer::new();
        input.push_str("A");
        input.guess_segment_properties();
        input.0.max_len = 1;
        assert!(!input.reserve(2));
        let result = font.shape(input, &[]);
        assert!(!result.is_empty());
        assert_eq!(
            result.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
        );
        assert_eq!(result.0.max_ops, hb_buffer_t::MAX_OPS_DEFAULT);
        let _ = result.serialize(font, SerializeFlags::default());
        assert_eq!(
            result.shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::GrowthLimit)
        );
        let mut reused = result.clear();
        reused.push_str("AB");
        reused.guess_segment_properties();
        let complete = font.shape(reused, &[]);
        assert_eq!(complete.shaping_status(), ShapingStatus::Complete);
        assert_eq!(complete.len(), 2);
        assert!(complete.glyph_infos().iter().all(|g| g.glyph_id != 0));
        let mut empty = complete.clear();
        empty.clear();
        empty.guess_segment_properties();
        let empty = font.shape(empty, &[]);
        assert!(empty.is_empty());
        assert_eq!(empty.shaping_status(), ShapingStatus::Complete);
    });
}

#[test]
fn specified_glyph_flag_capacity_reports_actual_skipped_flags_only() {
    for operation in 0..5 {
        for count in [255, 256] {
            let mut buffer = glyphs(count);
            buffer.reset_masks(0);
            buffer.flags.insert(
                BufferFlags::PRODUCE_UNSAFE_TO_CONCAT | BufferFlags::PRODUCE_SAFE_TO_INSERT_TATWEEL,
            );
            let flag = match operation {
                1 | 4 => UNSAFE_TO_CONCAT,
                2 => SAFE_TO_INSERT_TATWEEL,
                _ => UNSAFE_TO_BREAK,
            };
            if operation >= 3 {
                buffer.clear_output();
                buffer.next_glyph();
            }
            match operation {
                0 => buffer.unsafe_to_break(Some(0), Some(count)),
                1 => buffer.unsafe_to_concat(Some(0), Some(count)),
                2 => buffer.safe_to_insert_tatweel(Some(0), Some(count)),
                3 => buffer.unsafe_to_break_from_outbuffer(Some(0), Some(count)),
                _ => buffer.unsafe_to_concat_from_outbuffer(Some(0), Some(count)),
            }
            assert!(buffer.info[1..count]
                .iter()
                .all(|g| (g.mask & flag != 0) == (count == 255)));
            assert_eq!(
                buffer.shaping_status(),
                if count == 255 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::ContextLimit)
                }
            );
            assert!(buffer.successful);
        }
    }
    let mut whole = glyphs(256);
    whole.reset_masks(0);
    whole.unsafe_to_break(None, None);
    assert!(whole.info[1..256].iter().all(|g| g.unsafe_to_break()));
    assert_eq!(whole.shaping_status(), ShapingStatus::Complete);
    let mut disabled = glyphs(256);
    disabled.reset_masks(0);
    disabled.unsafe_to_concat(Some(0), Some(256));
    disabled.unsafe_to_concat_from_outbuffer(Some(0), Some(256));
    assert!(disabled.info[..256].iter().all(|g| g.mask == 0));
    assert_eq!(disabled.shaping_status(), ShapingStatus::Complete);
}
