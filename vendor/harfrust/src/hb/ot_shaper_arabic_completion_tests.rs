use super::*;
use crate::hb::completion_fixture::with_font;
use crate::{ShapingRefusal, ShapingStatus, UnicodeBuffer};

const PRE: &str = "\u{0628}\u{064E}\u{064E}\u{064E}\u{064E}\u{064E}";
const POST: &str = "\u{064E}\u{064E}\u{064E}\u{064E}\u{064E}\u{0628}";

fn input(pre: bool, context: &str) -> UnicodeBuffer {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str("\u{0628}");
    if pre {
        buffer.set_pre_context(context);
    } else {
        buffer.set_post_context(context);
    }
    buffer.guess_segment_properties();
    buffer
}

fn joining_action(mut input: UnicodeBuffer) -> (u8, ShapingStatus) {
    input.0.allocate_unicode_vars();
    input.0.allocate_var(GlyphInfo::ARABIC_SHAPING_ACTION_VAR);
    for info in &mut input.0.info[..input.0.len] {
        info.init_unicode_props(&mut input.0.scratch_flags);
    }
    arabic_joining(&mut input.0);
    (
        input.0.info[0].arabic_shaping_action(),
        input.0.shaping_status(),
    )
}

#[test]
fn transparent_context_exhaustion_reports_lost_joining_context_on_return() {
    with_font(|font| {
        let mut observed = Vec::new();
        for (pre, long, complete_action) in [
            (true, PRE, arabic_action_t::FINA),
            (false, POST, arabic_action_t::INIT),
        ] {
            assert_eq!(
                joining_action(input(pre, "\u{0628}")),
                (complete_action, ShapingStatus::Complete)
            );
            let (action, status) = joining_action(input(pre, long));
            assert_eq!(action, arabic_action_t::ISOL); // Preserve the original bounded algorithm.
            let result = font.shape(input(pre, long), &[]);
            assert!(!result.is_empty());
            observed.push((status, result.shaping_status()));
        }
        assert_eq!(
            observed,
            [(
                ShapingStatus::Refused(ShapingRefusal::ContextLimit),
                ShapingStatus::Refused(ShapingRefusal::ContextLimit)
            ); 2]
        );
    });
}

#[test]
fn retained_context_terminator_and_unselected_shaper_are_complete() {
    with_font(|font| {
        for pre in [false, true] {
            let context = if pre {
                "abcdef\u{0628}"
            } else {
                "\u{0628}abcdef"
            };
            assert_eq!(
                joining_action(input(pre, context)).1,
                ShapingStatus::Complete
            );
            assert_eq!(
                font.shape(input(pre, context), &[]).shaping_status(),
                ShapingStatus::Complete
            );
        }
        let mut latin = UnicodeBuffer::new();
        latin.push_str("A");
        latin.set_pre_context(PRE);
        latin.set_post_context(POST);
        latin.guess_segment_properties();
        assert_eq!(
            font.shape(latin, &[]).shaping_status(),
            ShapingStatus::Complete
        );
        let mut empty = UnicodeBuffer::new();
        empty.set_script(script::ARABIC);
        empty.set_direction(Direction::RightToLeft);
        empty.set_pre_context(PRE);
        assert_eq!(
            font.shape(empty, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}

#[test]
fn context_overwrite_public_add_and_result_clear_remove_old_truncation() {
    with_font(|font| {
        let mut buffer = input(true, PRE);
        buffer.set_pre_context("\u{0628}");
        assert_eq!(
            font.shape(buffer, &[]).shaping_status(),
            ShapingStatus::Complete
        );
        let mut buffer = input(false, POST);
        buffer.add('\u{0628}', 1); // Existing public add clears post-context.
        assert_eq!(
            font.shape(buffer, &[]).shaping_status(),
            ShapingStatus::Complete
        );
        let result = font.shape(input(true, PRE), &[]);
        let mut recycled = result.clear();
        recycled.push_str("\u{0628}");
        recycled.guess_segment_properties();
        assert_eq!(
            font.shape(recycled, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}

#[test]
fn transparent_input_observes_pre_scan_capacity_but_nonjoining_post_needs_none() {
    with_font(|font| {
        let mut transparent = UnicodeBuffer::new();
        transparent.push_str("\u{064E}");
        transparent.set_script(script::ARABIC);
        transparent.set_direction(Direction::RightToLeft);
        transparent.set_pre_context(PRE);
        // This diagnoses the selected pre-scan's insufficient context, without
        // claiming that the ultimate glyph output must differ.
        assert_eq!(
            font.shape(transparent, &[]).shaping_status(),
            ShapingStatus::Refused(ShapingRefusal::ContextLimit)
        );
        let mut nonjoining = UnicodeBuffer::new();
        nonjoining.push_str(" ");
        nonjoining.set_script(script::ARABIC);
        nonjoining.set_direction(Direction::RightToLeft);
        nonjoining.set_post_context(POST);
        assert_eq!(
            font.shape(nonjoining, &[]).shaping_status(),
            ShapingStatus::Complete
        );
    });
}
