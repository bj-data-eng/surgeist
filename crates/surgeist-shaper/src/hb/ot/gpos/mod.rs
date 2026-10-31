//! OpenType GPOS lookups.

use super::{observe_unused_adjustment, observe_variation_adjustment, AdjustmentObservation};
use crate::hb::ot_layout_gsubgpos::OT::hb_ot_apply_context_t;
use read_fonts::{
    tables::{
        gpos::{DeviceOrVariationIndex, ValueFormat},
        variations::DeltaSetIndex,
    },
    FontData, FontRead,
};

mod cursive;
mod mark;
mod pair;
mod single;

fn apply_value(
    ctx: &mut hb_ot_apply_context_t,
    idx: usize,
    data: &FontData,
    offset: usize,
    format: ValueFormat,
) -> Option<bool> {
    let mut observation = ValueObservation {
        requested: ctx.width_active(),
        ..ValueObservation::default()
    };
    let result = apply_value_inner(ctx, idx, data, offset, format, &mut observation);
    ctx.width_value_result(
        idx,
        observation.acted,
        observation.unqualified || result.is_none(),
    );
    result
}

#[derive(Default)]
struct ValueObservation {
    requested: bool,
    acted: bool,
    unqualified: bool,
}
impl ValueObservation {
    fn adjustment(&mut self, value: AdjustmentObservation) {
        self.acted |= value == AdjustmentObservation::Applied;
        self.unqualified |= value == AdjustmentObservation::Unqualified;
    }
}

#[allow(unused_assignments)]
fn apply_value_inner(
    ctx: &mut hb_ot_apply_context_t,
    idx: usize,
    data: &FontData,
    mut offset: usize,
    format: ValueFormat,
    observation: &mut ValueObservation,
) -> Option<bool> {
    let math = &mut ctx.buffer.position_math;
    let pos = &mut ctx.buffer.pos[idx];
    let is_horizontal = ctx.buffer.direction.is_horizontal();
    let mut worked = false;
    macro_rules! read_value {
        () => {{
            let value = data.read_at::<i16>(offset).ok()? as i32;
            observation.acted = true;
            worked |= value != 0;
            offset += 2;
            math.encode(value)
        }};
    }
    if format.contains(ValueFormat::X_PLACEMENT) {
        let value = read_value!();
        pos.x_offset = math.add(pos.x_offset, value);
    }
    if format.contains(ValueFormat::Y_PLACEMENT) {
        let value = read_value!();
        pos.y_offset = math.add(pos.y_offset, value);
    }
    if format.contains(ValueFormat::X_ADVANCE) {
        if is_horizontal {
            let value = read_value!();
            pos.x_advance = math.add(pos.x_advance, value);
        } else {
            offset += 2;
        }
    }
    if format.contains(ValueFormat::Y_ADVANCE) {
        if !is_horizontal {
            let value = read_value!();
            pos.y_advance = math.sub(pos.y_advance, value);
        } else {
            offset += 2;
        }
    }
    if !format.intersects(ValueFormat::ANY_DEVICE_OR_VARIDX) {
        return Some(worked);
    }
    if let Some(vs) = &ctx.face.ot_tables.var_store {
        let coords = ctx.face.ot_tables.coords;
        macro_rules! read_delta {
            () => {{
                let rec_offset = data.read_at::<u16>(offset).ok()? as usize;
                offset += 2;
                let mut value = 0;
                // Offset is nullable
                if rec_offset != 0 {
                    let format = data.read_at::<u16>(rec_offset + 4).ok()?;
                    // DeltaFormat specifier for a VariationIndex table
                    // See <https://learn.microsoft.com/en-us/typography/opentype/spec/chapter2#device-and-variationindex-tables>
                    const VARIATION_INDEX_FORMAT: u16 = 0x8000;
                    if format == VARIATION_INDEX_FORMAT {
                        let outer = data.read_at::<u16>(rec_offset).ok()?;
                        let inner = data.read_at::<u16>(rec_offset + 2).ok()?;
                        let delta=vs.compute_delta(DeltaSetIndex { outer, inner }, coords);
                        if observation.requested {
                            observation.adjustment(observe_variation_adjustment(Some(vs),DeltaSetIndex {outer,inner},coords,Some(&delta)));
                        }
                        value = delta.unwrap_or_default();
                        worked |= value != 0;
                    } else {
                        // Known Device paths are unused by this native owner;
                        // unknown/malformed records cannot certify an action.
                        if observation.requested {
                            observation.adjustment(data.split_off(rec_offset).map_or(AdjustmentObservation::Unqualified,
                                |data|observe_unused_adjustment(Some(DeviceOrVariationIndex::read(data)),coords)));
                        }
                    }
                }
                math.encode(value)
            }};
        }
        if format.contains(ValueFormat::X_PLACEMENT_DEVICE) {
            let value = read_delta!();
            pos.x_offset = math.add(pos.x_offset, value);
        }
        if format.contains(ValueFormat::Y_PLACEMENT_DEVICE) {
            let value = read_delta!();
            pos.y_offset = math.add(pos.y_offset, value);
        }
        if format.contains(ValueFormat::X_ADVANCE_DEVICE) {
            if is_horizontal {
                let value = read_delta!();
                pos.x_advance = math.add(pos.x_advance, value);
            } else {
                offset += 2;
            }
        }
        if format.contains(ValueFormat::Y_ADVANCE_DEVICE) {
            if !is_horizontal {
                let value = read_delta!();
                pos.y_advance = math.sub(pos.y_advance, value);
            } else {
                offset += 2;
            }
        }
    } else if observation.requested {
        // Preserve the native no-var-store arithmetic/traversal. Nullable fields
        // can establish absence; a non-null unresolved adjustment cannot.
        for (field, active) in [
            (ValueFormat::X_PLACEMENT_DEVICE, true),
            (ValueFormat::Y_PLACEMENT_DEVICE, true),
            (ValueFormat::X_ADVANCE_DEVICE, is_horizontal),
            (ValueFormat::Y_ADVANCE_DEVICE, !is_horizontal),
        ] {
            if format.contains(field) {
                if active {
                    let value = match data.read_at::<u16>(offset) {
                        Ok(0) => AdjustmentObservation::NoAdjustment,
                        Ok(value) => match data.split_off(usize::from(value)) {
                            Some(data) => observe_unused_adjustment(
                                Some(DeviceOrVariationIndex::read(data)),
                                ctx.face.ot_tables.coords,
                            ),
                            None => AdjustmentObservation::Unqualified,
                        },
                        Err(_) => AdjustmentObservation::Unqualified,
                    };
                    observation.adjustment(value);
                }
                offset += 2;
            }
        }
    }
    Some(worked)
}
