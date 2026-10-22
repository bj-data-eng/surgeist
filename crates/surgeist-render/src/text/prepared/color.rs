//! Bounded COLRv0 and static COLRv1 solid-glyph programs. The raw v1 graph is
//! qualified before provider traversal; callbacks cannot admit another graph.

use super::*;
use skrifa::{
    MetadataProvider,
    color::{Brush, ColorGlyphFormat, ColorPainter, CompositeMode},
    raw::tables::colr::Paint,
};

#[derive(Clone, Copy)]
struct Layer {
    glyph: GlyphId,
    palette: u16,
    alpha: f32,
    translation: (f32, f32),
}

struct Recorder {
    layers: Vec<Layer>,
    expected: usize,
    translation: (f32, f32),
    clip: Option<skrifa::raw::types::BoundingBox<f32>>,
    failed: bool,
}

impl ColorPainter for Recorder {
    fn push_transform(&mut self, transform: skrifa::color::Transform) {
        if transform.xx != 1.0 || transform.yy != 1.0 || transform.xy != 0.0 || transform.yx != 0.0
        {
            self.failed = true;
        }
        self.translation = (transform.dx, transform.dy);
    }
    fn pop_transform(&mut self) {
        self.translation = (0.0, 0.0);
    }
    fn push_clip_glyph(&mut self, _: GlyphId) {
        self.failed = true;
    }
    fn push_clip_box(&mut self, clip: skrifa::raw::types::BoundingBox<f32>) {
        self.clip = Some(clip);
    }
    fn pop_clip(&mut self) {}
    fn fill(&mut self, _: Brush<'_>) {
        self.failed = true;
    }
    fn fill_glyph(
        &mut self,
        glyph: GlyphId,
        transform: Option<skrifa::color::Transform>,
        brush: Brush<'_>,
    ) {
        if self.failed {
            return;
        }
        let Brush::Solid {
            palette_index,
            alpha,
        } = brush
        else {
            self.failed = true;
            return;
        };
        if transform.is_some()
            || self.layers.len() == self.expected
            || !alpha.is_finite()
            || !(0.0..=1.0).contains(&alpha)
        {
            self.failed = true;
            return;
        }
        self.layers.push(Layer {
            glyph,
            palette: palette_index,
            alpha,
            translation: self.translation,
        });
    }
    fn push_layer(&mut self, _: CompositeMode) {
        self.failed = true;
    }
}

fn invalid() -> Error {
    GlyphPreparationError::InvalidFont.into()
}
fn unsupported(paint: &Paint<'_>) -> Error {
    GlyphPreparationError::UnsupportedColorPaint {
        format: paint.format(),
    }
    .into()
}

/// A single optional translate, then one glyph and one solid. This rejects
/// recursive/layer/variable graphs before the nonfallible painter is invoked.
fn qualify_v1(mut paint: Paint<'_>) -> Result<()> {
    if let Paint::Translate(translate) = paint {
        paint = translate.paint().map_err(|_| invalid())?;
    }
    let Paint::Glyph(glyph) = &paint else {
        return Err(unsupported(&paint));
    };
    let solid = glyph.paint().map_err(|_| invalid())?;
    if !matches!(solid, Paint::Solid(_)) {
        return Err(unsupported(&solid));
    }
    Ok(())
}

pub(super) fn prepare(
    pen: &mut CollectingPen,
    context: &PreparationContext<'_, '_>,
    glyph: TextGlyph,
    format: ColorGlyphFormat,
) -> Result<()> {
    let PreparationContext {
        font,
        instance,
        coordinates,
        units,
        coordinate_bytes: coord_bytes,
        ..
    } = *context;
    default_image_instance(instance)?;
    let colr = font.colr().map_err(|_| invalid())?;
    let id = GlyphId::new(glyph.id());
    let count = match format {
        ColorGlyphFormat::ColrV0 => {
            let range = colr
                .v0_base_glyph(id)
                .map_err(|_| invalid())?
                .ok_or_else(invalid)?;
            let layers = colr
                .layer_records()
                .ok_or_else(invalid)?
                .map_err(|_| invalid())?;
            if range.end > layers.len() {
                return Err(invalid());
            }
            range.len()
        }
        ColorGlyphFormat::ColrV1 => {
            let (paint, _) = colr
                .v1_base_glyph(id)
                .map_err(|_| invalid())?
                .ok_or_else(invalid)?;
            qualify_v1(paint)?;
            if let Some(offset) = colr.clip_list_offset()
                && !offset.is_null()
            {
                // Variable clip boxes are not admitted by the default-only solid profile.
                if let Some(clip) = colr.v1_clip_box(id).map_err(|_| invalid())?
                    && !matches!(clip, skrifa::raw::tables::colr::ClipBox::Format1(_))
                {
                    return Err(GlyphPreparationError::UnsupportedColor.into());
                }
            }
            1
        }
    };
    let required = count
        .checked_mul(size_of::<Layer>())
        .and_then(|n| n.checked_add(coord_bytes))
        .ok_or(GlyphPreparationError::CapacityOverflow)?;
    check_limit(
        GlyphResource::ScratchBytes,
        required,
        pen.limits.scratch_bytes,
    )?;
    let layers = reserved(count, GlyphResource::ScratchBytes)?;
    let actual_scratch = layers
        .capacity()
        .checked_mul(size_of::<Layer>())
        .and_then(|n| n.checked_add(coord_bytes))
        .ok_or(GlyphPreparationError::CapacityOverflow)?;
    check_limit(
        GlyphResource::ScratchBytes,
        actual_scratch,
        pen.limits.scratch_bytes,
    )?;
    let mut recorder = Recorder {
        layers,
        expected: count,
        translation: (0.0, 0.0),
        clip: None,
        failed: false,
    };
    let colors = font.color_glyphs();
    let selected = colors.get_with_format(id, format).ok_or_else(invalid)?;
    selected
        .paint(LocationRef::new(coordinates), &mut recorder)
        .map_err(|_| invalid())?;
    if recorder.failed || recorder.layers.len() != count {
        return Err(invalid());
    }
    let cpal = font.cpal().map_err(|_| invalid())?;
    let records = cpal
        .color_records_array()
        .ok_or_else(invalid)?
        .map_err(|_| invalid())?;
    let start = usize::from(
        cpal.color_record_indices()
            .first()
            .ok_or_else(invalid)?
            .get(),
    );
    let end = start
        .checked_add(usize::from(cpal.num_palette_entries()))
        .ok_or_else(invalid)?;
    let palette = records.get(start..end).ok_or_else(invalid)?;
    let scale = f64::from(instance.size) / f64::from(units);
    let clip = recorder
        .clip
        .map(|clip| {
            let x0 = canonical_point(glyph.x() + f64::from(clip.x_min) * scale)?;
            let x1 = canonical_point(glyph.x() + f64::from(clip.x_max) * scale)?;
            let y0 = canonical_point(glyph.y() - f64::from(clip.y_max) * scale)?;
            let y1 = canonical_point(glyph.y() - f64::from(clip.y_min) * scale)?;
            if clip.x_max < clip.x_min || clip.y_max < clip.y_min {
                return Err(invalid());
            }
            if (clip.x_max > clip.x_min && x1 <= x0) || (clip.y_max > clip.y_min && y1 <= y0) {
                return Err(GlyphPreparationError::Numeric {
                    phase: GlyphNumericPhase::InkExtent,
                    value: (x1 - x0) * (y1 - y0),
                }
                .into());
            }
            Ok(kurbo::Rect::new(x0, y0, x1, y1))
        })
        .transpose()?;
    for layer in &recorder.layers {
        let color = if layer.palette == u16::MAX {
            if layer.alpha != 1.0 {
                return Err(GlyphPreparationError::UnsupportedColor.into());
            }
            None
        } else {
            let record = palette
                .get(usize::from(layer.palette))
                .ok_or_else(invalid)?;
            Some(crate::Color::try_rgba(
                f32::from(record.red()) / 255.0,
                f32::from(record.green()) / 255.0,
                f32::from(record.blue()) / 255.0,
                f32::from(record.alpha()) / 255.0 * layer.alpha,
            )?)
        };
        if color.is_some_and(|color| color.a() == 0.0) {
            continue;
        }
        let count = font.maxp().map_err(|_| invalid())?.num_glyphs();
        if layer.glyph.to_u32() >= u32::from(count) {
            return Err(GlyphPreparationError::GlyphRange {
                glyph: layer.glyph.to_u32(),
                count: u32::from(count),
            }
            .into());
        }
        let x = glyph.x() + f64::from(layer.translation.0) * scale;
        let y = glyph.y() - f64::from(layer.translation.1) * scale;
        for (delta, old, new) in [
            (layer.translation.0, glyph.x(), x),
            (layer.translation.1, glyph.y(), y),
        ] {
            if delta != 0.0 && new == old {
                return Err(GlyphPreparationError::Numeric {
                    phase: GlyphNumericPhase::PathPoint,
                    value: f64::from(delta),
                }
                .into());
            }
        }
        let placed = TextGlyph::try_new(layer.glyph.to_u32(), x, y, glyph.advance())?;
        let start = pen.elements.len();
        // Recorder storage remains live through actual qualified layer draws.
        draw_outline(pen, context, placed, clip, actual_scratch - coord_bytes)?;
        let clip = clip
            .map(|r| Rect::try_new(r.x0, r.y0, r.width(), r.height()))
            .transpose()?;
        pen.paint_fill(start, color, clip, false)?;
    }
    Ok(())
}
