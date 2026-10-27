use read_fonts::types::GlyphId;

use super::buffer::hb_buffer_t;
use super::face::hb_glyph_extents_t;
use super::ot_shape_plan::hb_ot_shape_plan_t;
use super::unicode::*;
use super::{hb_font_t, Direction};

fn recategorize_combining_class(u: u32, mut class: u8) -> u8 {
    use modified_combining_class as mcc;

    if class >= 200 {
        return class;
    }

    // Thai / Lao need some per-character work.
    if u & !0xFF == 0x0E00 {
        if class == 0 {
            match u {
                0x0E31 | 0x0E34 | 0x0E35 | 0x0E36 | 0x0E37 | 0x0E47 | 0x0E4C | 0x0E4D | 0x0E4E => {
                    class = combining_class::AboveRight;
                }

                0x0EB1 | 0x0EB4 | 0x0EB5 | 0x0EB6 | 0x0EB7 | 0x0EBB | 0x0ECC | 0x0ECD => {
                    class = combining_class::Above;
                }

                0x0EBC => class = combining_class::Below,

                _ => {}
            }
        } else {
            // Thai virama is below-right
            if u == 0x0E3A {
                class = combining_class::BelowRight;
            }
        }
    }

    match class {
        // Hebrew
        mcc::CCC10 => combining_class::Below,         // sheva
        mcc::CCC11 => combining_class::Below,         // hataf segol
        mcc::CCC12 => combining_class::Below,         // hataf patah
        mcc::CCC13 => combining_class::Below,         // hataf qamats
        mcc::CCC14 => combining_class::Below,         // hiriq
        mcc::CCC15 => combining_class::Below,         // tsere
        mcc::CCC16 => combining_class::Below,         // segol
        mcc::CCC17 => combining_class::Below,         // patah
        mcc::CCC18 => combining_class::Below,         // qamats & qamats qatan
        mcc::CCC20 => combining_class::Below,         // qubuts
        mcc::CCC22 => combining_class::Below,         // meteg
        mcc::CCC23 => combining_class::AttachedAbove, // rafe
        mcc::CCC24 => combining_class::AboveRight,    // shin dot
        mcc::CCC25 => combining_class::AboveLeft,     // sin dot
        mcc::CCC19 => combining_class::AboveLeft,     // holam & holam haser for vav
        mcc::CCC26 => combining_class::Above,         // point varika
        mcc::CCC21 => class,                          // dagesh

        // Arabic and Syriac
        mcc::CCC27 => combining_class::Above, // fathatan
        mcc::CCC28 => combining_class::Above, // dammatan
        mcc::CCC30 => combining_class::Above, // fatha
        mcc::CCC31 => combining_class::Above, // damma
        mcc::CCC33 => combining_class::Above, // shadda
        mcc::CCC34 => combining_class::Above, // sukun
        mcc::CCC35 => combining_class::Above, // superscript alef
        mcc::CCC36 => combining_class::Above, // superscript alaph
        mcc::CCC29 => combining_class::Below, // kasratan
        mcc::CCC32 => combining_class::Below, // kasra

        // Thai
        mcc::CCC103 => combining_class::BelowRight, // sara u / sara uu
        mcc::CCC107 => combining_class::AboveRight, // mai

        // Lao
        mcc::CCC118 => combining_class::Below, // sign u / sign uu
        mcc::CCC122 => combining_class::Above, // mai

        // Tibetian
        mcc::CCC129 => combining_class::Below, // sign aa
        mcc::CCC130 => combining_class::Above, // sign i
        mcc::CCC132 => combining_class::Below, // sign u

        _ => class,
    }
}

pub fn _hb_ot_shape_fallback_mark_position_recategorize_marks(
    _: &hb_ot_shape_plan_t,
    _: &hb_font_t,
    buffer: &mut hb_buffer_t,
) {
    let len = buffer.len;
    for info in &mut buffer.info[..len] {
        if info.general_category() == GeneralCategory::NON_SPACING_MARK {
            let mut class = info.modified_combining_class();
            class = recategorize_combining_class(info.glyph_id, class);
            info.set_modified_combining_class(class);
        }
    }
}

fn zero_mark_advances(
    buffer: &mut hb_buffer_t,
    start: usize,
    end: usize,
    adjust_offsets_when_zeroing: bool,
) {
    for (info, pos) in buffer.info[start..end]
        .iter()
        .zip(&mut buffer.pos[start..end])
    {
        if info.general_category() == GeneralCategory::NON_SPACING_MARK {
            if adjust_offsets_when_zeroing {
                pos.x_offset = buffer.position_math.sub(pos.x_offset, pos.x_advance);
                pos.y_offset = buffer.position_math.sub(pos.y_offset, pos.y_advance);
            }
            pos.x_advance = 0;
            pos.y_advance = 0;
        }
    }
}

fn position_mark(
    _plan: &hb_ot_shape_plan_t,
    face: &hb_font_t,
    direction: Direction,
    glyph: GlyphId,
    base_extents: &mut hb_glyph_extents_t,
    combining_class: u8,
    math: &mut super::vertical_position::PositionMath,
) -> Option<(i32, i32)> {
    let mut mark_extents = hb_glyph_extents_t::default();
    if !face.glyph_extents(glyph, &mut mark_extents) {
        return None;
    }
    // These are raw design operands, including every integer quotient. Only
    // the resulting mark offset is encoded by the caller, exactly once.
    let y_gap = face.units_per_em as i32 / 16;
    let x_offset = match combining_class {
        combining_class::DoubleBelow | combining_class::DoubleAbove
            if direction.is_horizontal() =>
        {
            let edge = math.add(
                base_extents.x_bearing,
                if direction.is_forward() {
                    base_extents.width
                } else {
                    0
                },
            );
            let center = math.sub(edge, mark_extents.width / 2);
            math.sub(center, mark_extents.x_bearing)
        }
        combining_class::AttachedBelowLeft
        | combining_class::BelowLeft
        | combining_class::AboveLeft => math.sub(base_extents.x_bearing, mark_extents.x_bearing),
        combining_class::AttachedAboveRight
        | combining_class::BelowRight
        | combining_class::AboveRight => {
            let edge = math.add(base_extents.x_bearing, base_extents.width);
            let edge = math.sub(edge, mark_extents.width);
            math.sub(edge, mark_extents.x_bearing)
        }
        _ => {
            let difference = math.sub(base_extents.width, mark_extents.width);
            let center = math.add(base_extents.x_bearing, difference / 2);
            math.sub(center, mark_extents.x_bearing)
        }
    };
    let is_attached = matches!(
        combining_class,
        combining_class::AttachedBelowLeft
            | combining_class::AttachedBelow
            | combining_class::AttachedAbove
            | combining_class::AttachedAboveRight
    );
    let mut y_offset = 0;
    match combining_class {
        combining_class::DoubleBelow
        | combining_class::BelowLeft
        | combining_class::Below
        | combining_class::BelowRight
        | combining_class::AttachedBelowLeft
        | combining_class::AttachedBelow => {
            if !is_attached {
                base_extents.height = math.sub(base_extents.height, y_gap);
            }
            let edge = math.add(base_extents.y_bearing, base_extents.height);
            y_offset = math.sub(edge, mark_extents.y_bearing);
            if (y_gap > 0) == (y_offset > 0) {
                base_extents.height = math.sub(base_extents.height, y_offset);
                y_offset = 0;
            }
            base_extents.height = math.add(base_extents.height, mark_extents.height);
        }
        combining_class::DoubleAbove
        | combining_class::AboveLeft
        | combining_class::Above
        | combining_class::AboveRight
        | combining_class::AttachedAbove
        | combining_class::AttachedAboveRight => {
            if !is_attached {
                base_extents.y_bearing = math.add(base_extents.y_bearing, y_gap);
                base_extents.height = math.sub(base_extents.height, y_gap);
            }
            let edge = math.add(mark_extents.y_bearing, mark_extents.height);
            y_offset = math.sub(base_extents.y_bearing, edge);
            if (y_gap > 0) != (y_offset > 0) {
                let correction = math.neg(y_offset) / 2;
                base_extents.y_bearing = math.add(base_extents.y_bearing, correction);
                base_extents.height = math.sub(base_extents.height, correction);
                y_offset = math.add(y_offset, correction);
            }
            base_extents.y_bearing = math.sub(base_extents.y_bearing, mark_extents.height);
            base_extents.height = math.add(base_extents.height, mark_extents.height);
        }
        _ => {}
    }
    Some((x_offset, y_offset))
}

fn position_around_base(
    plan: &hb_ot_shape_plan_t,
    face: &hb_font_t,
    buffer: &mut hb_buffer_t,
    base: usize,
    end: usize,
    adjust_offsets_when_zeroing: bool,
) {
    let mut horizontal_dir = Direction::Invalid;
    buffer.unsafe_to_break(Some(base), Some(end));

    let base_info = &buffer.info[base];
    let base_pos = &buffer.pos[base];
    let base_glyph = base_info.as_glyph();

    let mut base_extents = hb_glyph_extents_t::default();
    if !face.glyph_extents(base_glyph, &mut base_extents) {
        zero_mark_advances(buffer, base + 1, end, adjust_offsets_when_zeroing);
        return;
    }

    let mut math = buffer.position_math;
    let base_x_offset = base_pos.x_offset;
    let baseline_offset = math.raw(base_pos.y_offset);
    base_extents.y_bearing = math.add(base_extents.y_bearing, baseline_offset);
    base_extents.x_bearing = 0;

    // Use horizontal advance for horizontal positioning.
    // Generally a better idea. Also works for zero-ink glyphs. See:
    // https://github.com/harfbuzz/harfbuzz/issues/1532
    base_extents.width = face.glyph_h_advance(base_glyph);

    let lig_id = base_info.lig_id() as u32;
    let num_lig_components = base_info.lig_num_comps() as i32;

    let mut x_offset = 0;
    let mut y_offset = 0;
    if buffer.direction.is_forward() {
        x_offset = math.sub(x_offset, base_pos.x_advance);
        y_offset = math.sub(y_offset, base_pos.y_advance);
    }

    let mut last_lig_component: i32 = -1;
    let mut last_combining_class: u8 = 255;
    let mut component_extents = base_extents;
    let mut cluster_extents = base_extents;

    for (info, pos) in buffer.info[base + 1..end]
        .iter()
        .zip(&mut buffer.pos[base + 1..end])
    {
        if info.modified_combining_class() != 0 {
            if num_lig_components > 1 {
                let this_lig_id = info.lig_id() as u32;
                let mut this_lig_component = info.lig_comp() as i32 - 1;

                // Conditions for attaching to the last component.
                if lig_id == 0 || lig_id != this_lig_id || this_lig_component >= num_lig_components
                {
                    this_lig_component = num_lig_components - 1;
                }

                if last_lig_component != this_lig_component {
                    last_lig_component = this_lig_component;
                    last_combining_class = 255;
                    component_extents = base_extents;

                    if horizontal_dir == Direction::Invalid {
                        horizontal_dir = if plan.direction.is_horizontal() {
                            plan.direction
                        } else {
                            plan.script
                                .and_then(Direction::from_script)
                                .unwrap_or(Direction::LeftToRight)
                        };
                    }

                    let component = if horizontal_dir == Direction::LeftToRight {
                        this_lig_component
                    } else {
                        num_lig_components - 1 - this_lig_component
                    };
                    let numerator = math.mul(component, component_extents.width);
                    component_extents.x_bearing =
                        math.add(component_extents.x_bearing, numerator / num_lig_components);
                    component_extents.width /= num_lig_components;
                }
            }

            let this_combining_class = info.modified_combining_class();
            if last_combining_class != this_combining_class {
                last_combining_class = this_combining_class;
                cluster_extents = component_extents;
            }

            if let Some((raw_x, raw_y)) = position_mark(
                plan,
                face,
                buffer.direction,
                info.as_glyph(),
                &mut cluster_extents,
                this_combining_class,
                &mut math,
            ) {
                pos.x_offset = math.encode(raw_x);
                if buffer.direction.is_vertical() {
                    // Raw centering is relative to the base's font origin;
                    // keep its actual translated origin in the active domain.
                    pos.x_offset = math.add(pos.x_offset, base_x_offset);
                }
                pos.y_offset = math.encode(raw_y);
            }

            pos.x_advance = 0;
            pos.y_advance = 0;
            pos.x_offset = math.add(pos.x_offset, x_offset);
            pos.y_offset = math.add(pos.y_offset, y_offset);
        } else {
            if buffer.direction.is_forward() {
                x_offset = math.sub(x_offset, pos.x_advance);
                y_offset = math.sub(y_offset, pos.y_advance);
            } else {
                x_offset = math.add(x_offset, pos.x_advance);
                y_offset = math.add(y_offset, pos.y_advance);
            }
        }
        if math.failure.is_some() {
            break;
        }
    }
    buffer.position_math = math;
}

fn position_cluster_impl(
    plan: &hb_ot_shape_plan_t,
    face: &hb_font_t,
    buffer: &mut hb_buffer_t,
    start: usize,
    end: usize,
    adjust_offsets_when_zeroing: bool,
) {
    // Find the base glyph
    let mut i = start;
    while i < end {
        if !buffer.info[i].is_unicode_mark() {
            // Find mark glyphs
            let mut j = i + 1;
            while j < end
                && (buffer.info[j].is_hidden()
                    || buffer.info[j].is_default_ignorable()
                    || buffer.info[j].is_unicode_mark())
            {
                j += 1;
            }

            position_around_base(plan, face, buffer, i, j, adjust_offsets_when_zeroing);
            if buffer.position_math.failure.is_some() {
                return;
            }
            i = j - 1;
        }
        i += 1;
    }
}

#[inline(always)]
fn position_cluster(
    plan: &hb_ot_shape_plan_t,
    face: &hb_font_t,
    buffer: &mut hb_buffer_t,
    start: usize,
    end: usize,
    adjust_offsets_when_zeroing: bool,
) {
    if end - start < 2 {
        return;
    }

    position_cluster_impl(plan, face, buffer, start, end, adjust_offsets_when_zeroing);
}

pub fn position_marks(
    plan: &hb_ot_shape_plan_t,
    face: &hb_font_t,
    buffer: &mut hb_buffer_t,
    adjust_offsets_when_zeroing: bool,
) {
    buffer.position_math.stage = super::vertical_position::VerticalPositionStage::Fallback;
    buffer.assert_gsubgpos_vars();

    let mut start = 0;
    let len = buffer.len;
    for i in 1..len {
        if !buffer.info[i].is_unicode_mark()
            && !buffer.info[i].is_hidden()
            && !buffer.info[i].is_default_ignorable()
        {
            position_cluster(plan, face, buffer, start, i, adjust_offsets_when_zeroing);
            if buffer.position_math.failure.is_some() {
                return;
            }
            start = i;
        }
    }

    position_cluster(plan, face, buffer, start, len, adjust_offsets_when_zeroing);
}

pub fn _hb_ot_shape_fallback_kern(_: &hb_ot_shape_plan_t, _: &hb_font_t, _: &mut hb_buffer_t) {
    // STUB: this is deprecated in HarfBuzz
}

pub fn _hb_ot_shape_fallback_spaces(
    _: &hb_ot_shape_plan_t,
    face: &hb_font_t,
    buffer: &mut hb_buffer_t,
    mut metrics: Option<&mut super::vertical_position::MetricSource<'_>>,
) -> bool {
    use super::unicode::hb_unicode_funcs_t as t;
    buffer.position_math.stage = super::vertical_position::VerticalPositionStage::Fallback;
    let horizontal = buffer.direction.is_horizontal();
    for index in 0..buffer.len {
        let info = &buffer.info[index];
        if !info.is_unicode_space() || info.ligated() {
            continue;
        }
        let space_type = info.unicode_space_fallback_type();
        match space_type {
            t::SPACE_EM
            | t::SPACE_EM_2
            | t::SPACE_EM_3
            | t::SPACE_EM_4
            | t::SPACE_EM_5
            | t::SPACE_EM_6
            | t::SPACE_EM_16 => {
                let raw = (face.units_per_em as i32 + (space_type as i32) / 2) / space_type as i32;
                let length = buffer.position_math.encode(raw);
                if horizontal {
                    buffer.pos[index].x_advance = length;
                } else {
                    buffer.pos[index].y_advance = buffer.position_math.neg(length);
                }
            }
            t::SPACE_4_EM_18 => {
                let raw = ((face.units_per_em as i64) * 4 / 18) as i32;
                let length = buffer.position_math.encode(raw);
                if horizontal {
                    buffer.pos[index].x_advance = length;
                } else {
                    buffer.pos[index].y_advance = buffer.position_math.neg(length);
                }
            }
            t::SPACE_FIGURE | t::SPACE_PUNCTUATION => {
                let glyph = if space_type == t::SPACE_FIGURE {
                    ('0'..='9').find_map(|c| face.get_nominal_glyph(c as u32))
                } else {
                    face.get_nominal_glyph('.' as u32)
                        .or_else(|| face.get_nominal_glyph(',' as u32))
                };
                if let Some(glyph) = glyph {
                    if horizontal {
                        buffer.pos[index].x_advance = face.glyph_h_advance(glyph);
                    } else if let Some(source) = metrics.as_deref_mut() {
                        let Some((advance, _, _)) = source.get(buffer, glyph.to_u32()) else {
                            return false;
                        };
                        buffer.pos[index].y_advance = advance;
                    } else {
                        buffer.pos[index].y_advance = face.glyph_v_advance(glyph);
                    }
                }
            }
            t::SPACE_NARROW => {
                let pos = &mut buffer.pos[index];
                if horizontal {
                    pos.x_advance /= 2;
                } else {
                    pos.y_advance = buffer.position_math.divide_raw(pos.y_advance, 2);
                }
            }
            _ => {}
        }
    }
    buffer.position_math.failure.is_none()
}
