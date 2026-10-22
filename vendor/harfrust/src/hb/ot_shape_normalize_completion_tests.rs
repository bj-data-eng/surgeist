use super::*;
use crate::hb::completion_fixture::with_font;
use crate::{ShapingRefusal, ShapingStatus, UnicodeBuffer};

fn marks(count: usize) -> UnicodeBuffer {
    let mut input = UnicodeBuffer::new();
    input.push_str("B\u{0315}"); // CCC232 precedes CCC230: actual sorting is required.
    for _ in 1..count {
        input.add('\u{0300}', input.len() as u32);
    }
    input.guess_segment_properties();
    input
}

#[test]
fn combining_run_capacity_refuses_skipped_reorder_and_survives_real_shape() {
    with_font(|font| {
        for count in [32, 33] {
            let input = marks(count);
            let plan = hb_ot_shape_plan_t::new(font, input.0.direction, input.0.script, None, &[]);
            let mut buffer = input.0;
            buffer.allocate_unicode_vars();
            for info in &mut buffer.info[..buffer.len] {
                info.init_unicode_props(&mut buffer.scratch_flags);
            }
            _hb_ot_shape_normalize(&plan, &mut buffer, font);
            assert_eq!(buffer.len, count + 1);
            assert_eq!(
                buffer.info[1].glyph_id,
                if count == 32 { 0x0300 } else { 0x0315 }
            );
            assert!(buffer.successful);
            let result = font.shape(marks(count), &[]);
            assert!(!result.is_empty());
            let expected = if count == 32 {
                ShapingStatus::Complete
            } else {
                ShapingStatus::Refused(ShapingRefusal::ContextLimit)
            };
            assert_eq!(
                (buffer.shaping_status(), result.shaping_status()),
                (expected, expected)
            );
        }
    });
}
