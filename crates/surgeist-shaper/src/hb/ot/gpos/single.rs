use crate::hb::ot_layout_gsubgpos::Apply;
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use read_fonts::tables::gpos::{SinglePosFormat1, SinglePosFormat2};

impl Apply for SinglePosFormat1<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        ctx.with_width_reads(true, |ctx, reads| {
            let glyph = ctx.buffer.cur(0).as_glyph();
            reads.coverage_read(self.coverage())?.get(glyph)?;
            let format = self.value_format();
            let offset = self.value_record_byte_range().start;
            super::apply_value(ctx, ctx.buffer.idx, &self.offset_data(), offset, format);
            ctx.buffer.idx += 1;
            Some(())
        })
    }
}

impl Apply for SinglePosFormat2<'_> {
    fn apply(&self, ctx: &mut hb_ot_apply_context_t) -> Option<()> {
        ctx.with_width_reads(true, |ctx, reads| {
            let glyph = ctx.buffer.cur(0).as_glyph();
            let index = reads.coverage_read(self.coverage())?.get(glyph)? as usize;
            reads.check(index < usize::from(self.value_count()));
            let format = self.value_format();
            let offset = self.value_records_byte_range().start + (format.record_byte_len() * index);
            super::apply_value(ctx, ctx.buffer.idx, &self.offset_data(), offset, format);
            ctx.buffer.idx += 1;
            Some(())
        })
    }
}
