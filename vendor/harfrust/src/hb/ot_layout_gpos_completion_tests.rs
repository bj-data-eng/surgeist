use super::*;
use crate::hb::completion_fixture::{glyphs, with_font};
use crate::{ShapingRefusal, ShapingStatus};

#[test]
fn attachment_depth_boundary_reports_only_actual_truncation() {
    with_font(|font| {
        for links in [64, 65] {
            let mut buffer = glyphs(links + 1);
            buffer.allocate_gsubgpos_vars();
            buffer.scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT;
            for pos in &mut buffer.pos[..links] {
                pos.set_attach_chain(1);
                pos.set_attach_type(attach_type::CURSIVE);
                pos.y_offset = 1;
            }
            GPOS::position_finish_offsets(font, &mut buffer);
            assert_eq!(
                buffer.shaping_status(),
                if links == 64 {
                    ShapingStatus::Complete
                } else {
                    ShapingStatus::Refused(ShapingRefusal::AttachmentNestingLimit)
                }
            );
            assert!(buffer.successful);
            assert!(buffer.pos[0].y_offset > 1);
            assert!(buffer.pos.iter().all(|p| p.attach_chain() == 0));
        }
        let mut buffer = glyphs(1);
        buffer.allocate_gsubgpos_vars();
        buffer.scratch_flags |= HB_BUFFER_SCRATCH_FLAG_HAS_GPOS_ATTACHMENT;
        buffer.pos[0].set_attach_chain(-1);
        GPOS::position_finish_offsets(font, &mut buffer);
        assert_eq!(buffer.shaping_status(), ShapingStatus::Complete);
    });
}
