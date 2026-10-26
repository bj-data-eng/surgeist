//! Selected sbix PNG placement follows cached Skrifa0.42.1 bitmap.rs' documented
//! Skia/CoreText convention: outer LSB/yMin in font units, inner offsets in
//! strike pixels, BottomLeft origin. Pixels and placement are retained together.

use super::*;
use crate::vello_engine::glyph::{SelectedBitmap, SelectedBitmapData, SelectedBitmapSource};
use skrifa::MetadataProvider;

mod png;

fn invalid() -> Error {
    GlyphPreparationError::InvalidFont.into()
}

pub(super) fn prepare(
    pen: &mut CollectingPen,
    context: &PreparationContext<'_, '_>,
    glyph: TextGlyph,
    bitmap: SelectedBitmap<'_>,
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
    let SelectedBitmapSource::Sbix { ppem, x, y } = bitmap.source else {
        return Err(GlyphPreparationError::UnsupportedBitmap.into());
    };
    let SelectedBitmapData::Png { data, .. } = bitmap.data else {
        return Err(GlyphPreparationError::UnsupportedBitmap.into());
    };
    if ppem == 0 {
        return Err(invalid());
    }
    // Prove glyph data before using optional metrics. Missing bounds for a
    // genuinely empty glyf is zero; an invalid outline is not an empty bearing.
    let summary = glyf::qualify(
        font,
        GlyphId::new(glyph.id()),
        coordinates,
        pen.limits,
        coord_bytes,
    )?;
    let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::new(coordinates));
    let lsb = metrics
        .left_side_bearing(GlyphId::new(glyph.id()))
        .ok_or_else(invalid)?;
    let bottom = match metrics.bounds(GlyphId::new(glyph.id())) {
        Some(bounds) => bounds.y_min,
        None if summary.points == 0 => 0.0,
        None => return Err(invalid()),
    };
    let png::Decoded {
        width,
        height,
        pixels: output,
        retained_bytes: pixel_owner,
    } = png::decode(data, pen, coord_bytes)?;
    if output.chunks_exact(4).all(|pixel| pixel[3] == 0) {
        return Ok(());
    }
    let font_scale = f64::from(instance.size) / f64::from(units);
    let pixel_scale = f64::from(instance.size) / f64::from(ppem);
    let x0 = canonical_point(glyph.x() + f64::from(lsb) * font_scale + f64::from(x) * pixel_scale)?;
    let y0 = canonical_point(
        glyph.y()
            - f64::from(bottom) * font_scale
            - f64::from(y) * pixel_scale
            - f64::from(height) * pixel_scale,
    )?;
    let x1 = canonical_point(
        glyph.x()
            + f64::from(lsb) * font_scale
            + f64::from(x) * pixel_scale
            + f64::from(width) * pixel_scale,
    )?;
    let y1 =
        canonical_point(glyph.y() - f64::from(bottom) * font_scale - f64::from(y) * pixel_scale)?;
    if x1 <= x0 || y1 <= y0 {
        return Err(GlyphPreparationError::Numeric {
            phase: GlyphNumericPhase::InkExtent,
            value: (x1 - x0) * (y1 - y0),
        }
        .into());
    }
    let rect = Rect::try_new(x0, y0, x1 - x0, y1 - y0)?;
    pen.retain(pixel_owner)?;
    let pixels: Arc<[u8]> = output.into();
    let image = crate::Image::from_rgba(
        crate::Size::try_new(f64::from(width), f64::from(height))?,
        pixels,
    )?;
    let brush_transform =
        crate::encode::image_transform(image.size(), rect, crate::ImageFit::Fill)?;
    qualify_bounds_placement(brush_transform, None)?;
    pen.paint(GlyphPaint::Bitmap {
        image,
        rect,
        brush_transform,
    })?;
    let bounds = kurbo::Rect::new(x0, y0, x1, y1);
    pen.bounds = Some(pen.bounds.map_or(bounds, |previous| previous.union(bounds)));
    Ok(())
}
