// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Modified for surgeist-render; see NOTICE-VELLO.md for provenance and adaptations.

use std::io::Cursor;

use png::{BitDepth, ColorType, Transformations};
use skrifa::{
    GlyphId, MetadataProvider, Tag,
    bitmap::MaskData,
    color::{Brush as ColorBrush, ColorGlyph, ColorPainter, CompositeMode},
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    raw::{
        FontData as RawFontData, ReadError as RawReadError, TableProvider,
        tables::{
            bitmap::{
                BitmapContent as RawBitmapContent, BitmapData as RawBitmapData,
                BitmapDataFormat as RawBitmapDataFormat, BitmapLocation,
                BitmapMetrics as RawBitmapMetrics, BitmapSize, IndexSubtable,
            },
            cpal::Cpal,
        },
        types::BoundingBox,
    },
};

use super::bounded::{self, Growth, OwnedEncoding};
use super::geometry::{self, QualifiedTransform};
use crate::accounting::{Account, AccountedVec, RenderPhase, RenderWork};
use crate::{
    BackendErrorCode, Error, FontData, Result, TextGlyph, TextPaint, TextRun, Transform,
    text::invalid_font_data,
};
use vello_encoding::StreamOffsets;

#[derive(Clone, Copy)]
struct BitmapSelection<'a> {
    font: &'a FontData,
    account: Option<&'a Account>,
}
impl BitmapSelection<'_> {
    fn record(self) -> Result<()> {
        if let Some(account) = self.account {
            account
                .at(RenderPhase::Bitmap)
                .work(RenderWork::FontRecordVisits, 1)?;
        }
        Ok(())
    }
}

const SKRIFA_NORMALIZED_COORDS: &[skrifa::instance::NormalizedCoord] = &[];

pub(crate) struct ValidatedGlyphRun<'a> {
    font_data: &'a FontData,
    size: f32,
    transform: Transform,
    paint: &'a TextPaint,
    representations: AccountedVec<SelectedGlyphRepresentation>,
    drawables: AccountedVec<DrawableGlyph<'a>>,
    stream_sizes: StreamOffsets,
    #[cfg(test)]
    selected_glyph_traces: AccountedVec<SelectedGlyphTrace>,
}

// One authoritative occurrence record couples the actual source glyph, its
// diagnostic ordinal, and the native position admitted for that same glyph.
struct DrawableGlyph<'a> {
    glyph: &'a TextGlyph,
    source_index: usize,
    position: [f32; 2],
}

#[derive(Clone, Copy)]
pub(crate) struct GlyphOccurrence<'a> {
    font_data: &'a FontData,
    size: &'a f32,
    drawable: &'a DrawableGlyph<'a>,
}

impl<'a> GlyphOccurrence<'a> {
    pub(crate) fn glyph(self) -> &'a TextGlyph {
        self.drawable.glyph
    }
    pub(crate) fn source_index(self) -> usize {
        self.drawable.source_index
    }
    pub(crate) fn native_position(self) -> [f32; 2] {
        self.drawable.position
    }
}

pub(crate) enum SelectedGlyphRepresentation {
    Outline,
    Colr,
    Bitmap(BitmapEncoding),
}

pub(crate) enum BitmapEncoding {
    Bgra,
    Png,
    PackedMask,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BitmapSourceForTest {
    Sbix,
    Cbdt,
    Ebdt,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectedGlyphTrace {
    Outline,
    Colr,
    Bitmap {
        source: BitmapSourceForTest,
        ppem: u16,
    },
}

pub(crate) struct SelectedBitmap<'a> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) source: SelectedBitmapSource,
    pub(crate) data: SelectedBitmapData<'a>,
    #[cfg(test)]
    selection: Option<SelectedBitmapSelection>,
}

#[derive(Clone, Copy)]
pub(crate) enum SelectedBitmapSource {
    Sbix { ppem: u16, x: i16, y: i16 },
    Other,
}

pub(crate) enum SelectedBitmapData<'a> {
    Bgra(&'a [u8]),
    Png {
        data: &'a [u8],
        expected_dimensions: Option<(u32, u32)>,
    },
    Mask {
        bpp: u8,
        is_packed: bool,
        data: &'a [u8],
    },
    Unsupported,
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct SelectedBitmapSelection {
    source: Option<BitmapSourceForTest>,
    ppem: u16,
}

#[cfg(test)]
impl<'a> SelectedBitmap<'a> {
    fn with_selection(mut self, source: BitmapSourceForTest, ppem: u16) -> Self {
        self.selection = Some(SelectedBitmapSelection {
            source: Some(source),
            ppem,
        });
        self
    }

    fn with_ppem(mut self, ppem: u16) -> Self {
        self.selection = Some(SelectedBitmapSelection { source: None, ppem });
        self
    }

    fn with_source(mut self, source: BitmapSourceForTest) -> Self {
        if let Some(selection) = &mut self.selection {
            selection.source = Some(source);
        }
        self
    }

    fn selected_trace(&self) -> Option<SelectedGlyphTrace> {
        let selection = self.selection?;
        Some(SelectedGlyphTrace::Bitmap {
            source: selection.source?,
            ppem: selection.ppem,
        })
    }
}

impl<'a> ValidatedGlyphRun<'a> {
    pub(crate) const fn size(&self) -> f32 {
        self.size
    }

    pub(crate) const fn transform(&self) -> Transform {
        self.transform
    }

    pub(crate) const fn font_data(&self) -> &FontData {
        self.font_data
    }

    pub(crate) const fn paint(&self) -> &TextPaint {
        self.paint
    }

    pub(crate) fn stream_sizes(&self) -> &StreamOffsets {
        &self.stream_sizes
    }

    pub(crate) fn representations(&self) -> &[SelectedGlyphRepresentation] {
        self.representations.as_slice()
    }

    pub(crate) fn occurrences(&self) -> impl ExactSizeIterator<Item = GlyphOccurrence<'_>> {
        self.drawables.iter().map(|drawable| GlyphOccurrence {
            font_data: self.font_data,
            size: &self.size,
            drawable,
        })
    }

    pub(crate) fn drawable_count(&self) -> usize {
        self.drawables.len()
    }

    #[cfg(test)]
    pub(crate) fn selected_glyph_traces_for_test(&self) -> &[SelectedGlyphTrace] {
        self.selected_glyph_traces.as_slice()
    }
}

pub(crate) fn preflight_selected_glyphs<'a>(
    run: &'a TextRun<'a>,
    account: &Account,
) -> Result<ValidatedGlyphRun<'a>> {
    let font_data = run
        .font()
        .data
        .as_ref()
        .ok_or_else(|| Error::invalid_input_message("text run font data is required"))?;
    let font = skrifa::FontRef::from_index(font_data.bytes(), font_data.index())
        .map_err(|_| font_data_error(font_data))?;
    let glyph_count = font
        .maxp()
        .map_err(|_| font_data_error(font_data))?
        .num_glyphs() as u32;
    let colr = font
        .table_data(skrifa::Tag::new(b"COLR"))
        .map(|_| font.colr().map_err(|_| font_data_error(font_data)))
        .transpose()?;
    let outlines = font.outline_glyphs();
    let mut representations = account.temporary(run.glyphs().len())?;
    let mut stream_sizes = StreamOffsets::default();
    let mut drawables = account.temporary(run.glyphs().len())?;
    #[cfg(test)]
    let mut selected_glyph_traces = account.temporary(run.glyphs().len())?;

    for (glyph_index, glyph) in run.glyphs().iter().enumerate() {
        account.work(RenderWork::GlyphVisits, 1)?;
        let mut drawable = true;
        if glyph.id() >= glyph_count {
            return Err(missing_glyph_error(glyph.id()));
        }
        let glyph_id = GlyphId::new(glyph.id());
        let representation = if selected_color_root(colr.as_ref(), glyph_id, font_data)?.is_some() {
            preflight_selected_image_head(&font, font_data)?;
            let cpal = font.cpal().map_err(|_| font_data_error(font_data))?;
            let color = font
                .color_glyphs()
                .get(glyph_id)
                .ok_or_else(|| font_data_error(font_data))?;
            preflight_color_glyph(
                &color,
                &outlines,
                &cpal,
                run.size(),
                font_data,
                glyph_index,
                account,
            )?;
            #[cfg(test)]
            selected_glyph_traces.push(SelectedGlyphTrace::Colr)?;
            SelectedGlyphRepresentation::Colr
        } else if let Some(bitmap) = select_bitmap(
            &font,
            Size::new(run.size()),
            glyph_id,
            font_data,
            Some(account),
        )? {
            preflight_selected_image_head(&font, font_data)?;
            let encoding = preflight_bitmap(&bitmap, font_data, account)?;
            #[cfg(test)]
            selected_glyph_traces.push(
                bitmap
                    .selected_trace()
                    .ok_or_else(|| font_data_error(font_data))?,
            )?;
            SelectedGlyphRepresentation::Bitmap(encoding)
        } else {
            let sizes = preflight_outline(
                &outlines,
                glyph_id,
                run.size(),
                font_data,
                glyph_index,
                account,
            )
            .map_err(|error| glyph_geometry_error(error, glyph_index))?;
            drawable = sizes.path_data != 0;
            bounded::sum_streams(&mut stream_sizes, &sizes, account)?;
            #[cfg(test)]
            selected_glyph_traces.push(SelectedGlyphTrace::Outline)?;
            SelectedGlyphRepresentation::Outline
        };
        if drawable {
            let reason = crate::GeometryRealizationError::UnrepresentableGlyph { glyph_index };
            account.work(RenderWork::CoordinateVisits, 1)?;
            let x = glyph.x();
            account.work(RenderWork::CoordinateVisits, 1)?;
            let y = glyph.y();
            let x = geometry::scalar(x, reason, account)?;
            let y = geometry::scalar(y, reason, account)?;
            let slot = drawables.append_slot()?;
            account.work(RenderWork::RasterResourceCopies, 1)?;
            slot.write(DrawableGlyph {
                glyph,
                source_index: glyph_index,
                position: [x, y],
            });
        }
        account.work(RenderWork::RasterResourceCopies, 1)?;
        representations.push(representation)?;
    }

    let drawable_count = drawables.len();
    if drawable_count != 0 {
        stream_sizes.path_tags = bounded::add(
            stream_sizes.path_tags,
            bounded::add(drawable_count, 2, account)?,
            account,
        )?;
        stream_sizes.transforms = bounded::add(
            stream_sizes.transforms,
            bounded::add(drawable_count, 1, account)?,
            account,
        )?;
    }
    Ok(ValidatedGlyphRun {
        font_data,
        size: run.size(),
        transform: run.transform(),
        paint: run.paint(),
        representations,
        drawables,
        stream_sizes,
        #[cfg(test)]
        selected_glyph_traces,
    })
}

fn selected_color_root<'a>(
    colr: Option<&skrifa::raw::tables::colr::Colr<'a>>,
    glyph_id: GlyphId,
    font_data: &FontData,
) -> Result<Option<skrifa::color::ColorGlyphFormat>> {
    let Some(colr) = colr else {
        return Ok(None);
    };
    if !matches!(colr.version(), 0 | 1) {
        return Err(font_data_error(font_data));
    }
    if colr.version() == 1 {
        match colr.base_glyph_list_offset() {
            Some(offset) if offset.is_null() => {}
            Some(_) => {
                if colr
                    .v1_base_glyph(glyph_id)
                    .map_err(|_| font_data_error(font_data))?
                    .is_some()
                {
                    return Ok(Some(skrifa::color::ColorGlyphFormat::ColrV1));
                }
            }
            None => return Err(font_data_error(font_data)),
        }
    }
    if colr.num_base_glyph_records() != 0
        && colr
            .v0_base_glyph(glyph_id)
            .map_err(|_| font_data_error(font_data))?
            .is_some()
    {
        return Ok(Some(skrifa::color::ColorGlyphFormat::ColrV0));
    }
    Ok(None)
}

pub(crate) fn selected_color_format(
    font: &skrifa::FontRef<'_>,
    glyph: GlyphId,
    data: &FontData,
) -> Result<Option<skrifa::color::ColorGlyphFormat>> {
    let colr = font
        .table_data(Tag::new(b"COLR"))
        .map(|_| {
            font.colr()
                .map_err(|_| Error::from(crate::GlyphPreparationError::InvalidFont))
        })
        .transpose()?;
    selected_color_root(colr.as_ref(), glyph, data)
        .map_err(|_| crate::GlyphPreparationError::InvalidFont.into())
}

fn preflight_selected_image_head(font: &skrifa::FontRef<'_>, font_data: &FontData) -> Result<()> {
    font.head()
        .map_err(|_| font_data_error(font_data))?
        .units_per_em();
    Ok(())
}

fn preflight_outline(
    outlines: &skrifa::OutlineGlyphCollection<'_>,
    glyph_id: GlyphId,
    size: f32,
    font_data: &FontData,
    glyph_index: usize,
    account: &Account,
) -> Result<StreamOffsets> {
    let outline = outlines
        .get(glyph_id)
        .ok_or_else(|| font_data_error(font_data))?;
    let mut pen = QualifiedPen {
        elements: account.temporary(0)?,
        account,
        error: None,
    };
    let drawn = outline.draw(
        DrawSettings::unhinted(Size::new(size), LocationRef::new(SKRIFA_NORMALIZED_COORDS)),
        &mut pen,
    );
    if let Some(error) = pen.error.take() {
        return Err(error);
    }
    drawn.map_err(|_| font_data_error(font_data))?;
    if !geometry::has_segments(&pen.elements, account)? {
        // The actual scaled provider may have quantized a visible outline to
        // coincident points before PathEncoder. Release that consumed buffer
        // before obtaining a qualification-only witness from the same outline.
        drop(pen);
        let mut witness = QualifiedPen {
            elements: account.temporary(0)?,
            account,
            error: None,
        };
        account.work(RenderWork::GlyphVisits, 1)?;
        let drawn = outline.draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::new(SKRIFA_NORMALIZED_COORDS)),
            &mut witness,
        );
        if let Some(error) = witness.error.take() {
            return Err(error);
        }
        drawn.map_err(|_| font_data_error(font_data))?;
        if geometry::has_segments(&witness.elements, account)? {
            return Err(
                crate::GeometryRealizationError::UnrepresentableGlyph { glyph_index }.into(),
            );
        }
        return Ok(StreamOffsets::default());
    }
    // Same default size/coordinates, unhinted nonzero fill, zero embolden and
    // finish(false) as the locked glyph cache. No narrower prepared-outline gate.
    let mut owned = OwnedEncoding::temporary();
    let (encoding, backing) = owned.parts();
    backing.reserve(
        encoding,
        Growth {
            path_tags: 1,
            styles: 1,
            ..Growth::default()
        },
        account,
    )?;
    encoding.encode_fill_style(peniko::Fill::NonZero);
    backing.path(encoding, pen.elements.len(), account)?;
    let qualified = geometry::qualify_transform(kurbo::Affine::IDENTITY, account)?;
    if !bounded::encode_elements_with_marker(
        encoding,
        &pen.elements,
        true,
        false,
        &qualified,
        account,
    )? {
        encoding.reset();
    }
    Ok(encoding.stream_offsets())
}

pub(crate) fn qualify_native_outline(
    occurrence: GlyphOccurrence<'_>,
    qualified: &QualifiedTransform,
    account: &Account,
) -> Result<()> {
    let font_data = occurrence.font_data;
    let font = skrifa::FontRef::from_index(font_data.bytes(), font_data.index())
        .map_err(|_| font_data_error(font_data))?;
    account.work(RenderWork::GlyphVisits, 1)?;
    let glyph = occurrence.glyph();
    let glyph_index = occurrence.source_index();
    let outlines = font.outline_glyphs();
    let outline = outlines
        .get(GlyphId::new(glyph.id()))
        .ok_or_else(|| font_data_error(font_data))?;
    let mut pen = QualifiedPen {
        elements: account.temporary(0)?,
        account,
        error: None,
    };
    let drawn = outline.draw(
        DrawSettings::unhinted(
            Size::new(*occurrence.size),
            LocationRef::new(SKRIFA_NORMALIZED_COORDS),
        ),
        &mut pen,
    );
    if let Some(error) = pen.error {
        return Err(error);
    }
    drawn.map_err(|_| font_data_error(font_data))?;
    let mut owned = OwnedEncoding::temporary();
    let (encoding, backing) = owned.parts();
    backing.path(encoding, pen.elements.len(), account)?;
    bounded::encode_elements_with_marker(encoding, &pen.elements, true, false, qualified, account)
        .map_err(|error| glyph_geometry_error(error, glyph_index))?;
    Ok(())
}

pub(crate) fn glyph_geometry_error(error: Error, glyph_index: usize) -> Error {
    match error.geometry_realization_diagnostic() {
        Some(
            crate::GeometryRealizationError::UnrepresentableTransform
            | crate::GeometryRealizationError::UnrepresentablePath { .. }
            | crate::GeometryRealizationError::UnrepresentableGlyph { .. },
        ) => crate::GeometryRealizationError::UnrepresentableGlyph { glyph_index }.into(),
        _ => error,
    }
}

fn preflight_color_glyph(
    color: &ColorGlyph<'_>,
    outlines: &skrifa::OutlineGlyphCollection<'_>,
    cpal: &Cpal<'_>,
    size: f32,
    font_data: &FontData,
    glyph_index: usize,
    account: &Account,
) -> Result<()> {
    let mut painter = PreflightColorPainter {
        font_data,
        outlines,
        cpal,
        size,
        glyph_index,
        failed: false,
        account,
        error: None,
    };
    let painted = color.paint(LocationRef::new(SKRIFA_NORMALIZED_COORDS), &mut painter);
    if let Some(error) = painter.error {
        return Err(error);
    }
    painted.map_err(|_| font_data_error(font_data))?;
    if painter.failed {
        return Err(font_data_error(font_data));
    }
    Ok(())
}

pub(crate) fn select_bitmap<'a>(
    font: &skrifa::FontRef<'a>,
    size: Size,
    glyph_id: GlyphId,
    font_data: &FontData,
    account: Option<&Account>,
) -> Result<Option<SelectedBitmap<'a>>> {
    let selection = BitmapSelection {
        font: font_data,
        account,
    };
    let run = || select_bitmap_inner(font, size, glyph_id, selection);
    if let Some(account) = account {
        account.phase(RenderPhase::Bitmap, run)
    } else {
        run()
    }
}
fn select_bitmap_inner<'a>(
    font: &skrifa::FontRef<'a>,
    size: Size,
    glyph_id: GlyphId,
    selection: BitmapSelection<'_>,
) -> Result<Option<SelectedBitmap<'a>>> {
    let font_data = selection.font;
    if font_has_table(font, Tag::new(b"sbix"), selection)? {
        return select_sbix_bitmap(font, size, glyph_id, selection);
    }
    if bitmap_table_pair_is_present(font, Tag::new(b"CBLC"), Tag::new(b"CBDT"), selection)? {
        let cblc = font.cblc().map_err(|_| font_data_error(font_data))?;
        let cbdt = font.cbdt().map_err(|_| font_data_error(font_data))?;
        let bitmap = select_bdt_bitmap(
            cblc.bitmap_sizes(),
            cblc.num_sizes(),
            cblc.offset_data(),
            size,
            glyph_id,
            selection,
            |location| cbdt.data(location),
        )?;
        #[cfg(test)]
        let bitmap = bitmap.map(|bitmap| bitmap.with_source(BitmapSourceForTest::Cbdt));
        return Ok(bitmap);
    }
    if bitmap_table_pair_is_present(font, Tag::new(b"EBLC"), Tag::new(b"EBDT"), selection)? {
        let eblc = font.eblc().map_err(|_| font_data_error(font_data))?;
        let ebdt = font.ebdt().map_err(|_| font_data_error(font_data))?;
        let bitmap = select_bdt_bitmap(
            eblc.bitmap_sizes(),
            eblc.num_sizes(),
            eblc.offset_data(),
            size,
            glyph_id,
            selection,
            |location| ebdt.data(location),
        )?;
        #[cfg(test)]
        let bitmap = bitmap.map(|bitmap| bitmap.with_source(BitmapSourceForTest::Ebdt));
        return Ok(bitmap);
    }
    Ok(None)
}

fn font_has_table(
    font: &skrifa::FontRef<'_>,
    tag: Tag,
    selection: BitmapSelection<'_>,
) -> Result<bool> {
    for record in font.table_directory().table_records() {
        selection.record()?;
        if record.tag() == tag {
            return Ok(true);
        }
    }
    Ok(false)
}

fn bitmap_table_pair_is_present(
    font: &skrifa::FontRef<'_>,
    location_tag: Tag,
    data_tag: Tag,
    selection: BitmapSelection<'_>,
) -> Result<bool> {
    let font_data = selection.font;
    match (
        font_has_table(font, location_tag, selection)?,
        font_has_table(font, data_tag, selection)?,
    ) {
        (false, false) => Ok(false),
        (true, true) => Ok(true),
        _ => Err(font_data_error(font_data)),
    }
}

fn select_sbix_bitmap<'a>(
    font: &skrifa::FontRef<'a>,
    size: Size,
    glyph_id: GlyphId,
    selection: BitmapSelection<'_>,
) -> Result<Option<SelectedBitmap<'a>>> {
    let font_data = selection.font;
    let sbix = font.sbix().map_err(|_| font_data_error(font_data))?;
    let strike_count = checked_count(sbix.num_strikes(), 0, font_data)?;
    let strikes = sbix.strikes();
    if strikes.len() != strike_count {
        return Err(font_data_error(font_data));
    }

    let requested_size = size.ppem().unwrap_or(f32::MAX);
    let mut best = None;
    for index in 0..strike_count {
        selection.record()?;
        let strike = strikes.get(index).map_err(|_| font_data_error(font_data))?;
        let strike_size = f32::from(strike.ppem());
        if !should_consider_bitmap_strike(strike_size, requested_size, best.as_ref()) {
            continue;
        }
        if let Some(bitmap) = select_sbix_glyph(&strike, glyph_id, font_data)? {
            #[cfg(test)]
            let bitmap = bitmap.with_selection(BitmapSourceForTest::Sbix, strike.ppem());
            best = Some((strike_size, bitmap));
        }
    }
    Ok(best.map(|(_, bitmap)| bitmap))
}

fn select_sbix_glyph<'a>(
    strike: &skrifa::raw::tables::sbix::Strike<'a>,
    glyph_id: GlyphId,
    font_data: &FontData,
) -> Result<Option<SelectedBitmap<'a>>> {
    let Some(glyph) = strike
        .glyph_data(glyph_id)
        .map_err(|_| font_data_error(font_data))?
    else {
        return Ok(None);
    };
    if glyph.graphic_type() != Tag::new(b"png ") {
        return Ok(Some(SelectedBitmap {
            width: 0,
            height: 0,
            source: SelectedBitmapSource::Other,
            data: SelectedBitmapData::Unsupported,
            #[cfg(test)]
            selection: None,
        }));
    }
    Ok(Some(SelectedBitmap {
        width: 0,
        height: 0,
        source: SelectedBitmapSource::Sbix {
            ppem: strike.ppem(),
            x: glyph.origin_offset_x(),
            y: glyph.origin_offset_y(),
        },
        data: SelectedBitmapData::Png {
            data: glyph.data(),
            expected_dimensions: None,
        },
        #[cfg(test)]
        selection: None,
    }))
}

fn select_bdt_bitmap<'a, F>(
    bitmap_sizes: &'a [BitmapSize],
    declared_size_count: u32,
    offset_data: RawFontData<'a>,
    size: Size,
    glyph_id: GlyphId,
    selection: BitmapSelection<'_>,
    bitmap_data: F,
) -> Result<Option<SelectedBitmap<'a>>>
where
    F: Fn(&BitmapLocation) -> std::result::Result<RawBitmapData<'a>, RawReadError>,
{
    let font_data = selection.font;
    if bitmap_sizes.len() != checked_count(declared_size_count, 0, font_data)? {
        return Err(font_data_error(font_data));
    }

    let requested_size = size.ppem().unwrap_or(f32::MAX);
    let mut best = None;
    for bitmap_size in bitmap_sizes {
        selection.record()?;
        let strike_size = f32::from(bitmap_size.ppem_y());
        if !should_consider_bitmap_strike(strike_size, requested_size, best.as_ref()) {
            continue;
        }
        let Some(location) = select_bdt_location(bitmap_size, offset_data, glyph_id, selection)?
        else {
            continue;
        };
        if location.is_empty() {
            continue;
        }
        let data = bitmap_data(&location).map_err(|_| font_data_error(font_data))?;
        let bitmap = selected_bdt_glyph(bitmap_size, data, font_data)?;
        #[cfg(test)]
        let bitmap = bitmap.with_ppem(u16::from(bitmap_size.ppem_y()));
        best = Some((strike_size, bitmap));
    }
    Ok(best.map(|(_, bitmap)| bitmap))
}

fn should_consider_bitmap_strike(
    candidate_size: f32,
    requested_size: f32,
    best: Option<&(f32, SelectedBitmap<'_>)>,
) -> bool {
    let Some((best_size, _)) = best else {
        return true;
    };
    (candidate_size >= requested_size && candidate_size < *best_size)
        || (*best_size < requested_size && candidate_size > *best_size)
}

fn select_bdt_location(
    bitmap_size: &BitmapSize,
    offset_data: RawFontData<'_>,
    glyph_id: GlyphId,
    selection: BitmapSelection<'_>,
) -> Result<Option<BitmapLocation>> {
    let font_data = selection.font;
    if !(bitmap_size.start_glyph_index()..=bitmap_size.end_glyph_index()).contains(&glyph_id) {
        return Ok(None);
    }
    let subtable_list = bitmap_size
        .index_subtable_list(offset_data)
        .map_err(|_| font_data_error(font_data))?;
    let records = subtable_list.index_subtable_records();
    if records.len() != checked_count(bitmap_size.number_of_index_subtables(), 0, font_data)? {
        return Err(font_data_error(font_data));
    }
    let mut selected = None;
    for record in records {
        selection.record()?;
        if (record.first_glyph_index()..=record.last_glyph_index()).contains(&glyph_id) {
            selected = Some(record);
            break;
        }
    }
    let Some(record) = selected else {
        return Ok(None);
    };
    let subtable = record
        .index_subtable(subtable_list.offset_data())
        .map_err(|_| font_data_error(font_data))?;
    let glyph_index =
        checked_glyph_index(glyph_id, record.first_glyph_index().to_u32(), font_data)?;
    selected_bdt_location_for_subtable(bitmap_size, &subtable, glyph_id, glyph_index, selection)
}

fn selected_bdt_location_for_subtable(
    bitmap_size: &BitmapSize,
    subtable: &IndexSubtable<'_>,
    glyph_id: GlyphId,
    glyph_index: usize,
    selection: BitmapSelection<'_>,
) -> Result<Option<BitmapLocation>> {
    let font_data = selection.font;
    let mut location = BitmapLocation {
        bit_depth: bitmap_size.bit_depth(),
        ..BitmapLocation::default()
    };
    match subtable {
        IndexSubtable::Format1(_) | IndexSubtable::Format3(_) => {
            select_dense_bdt_location(subtable, glyph_index, font_data, &mut location)?;
        }
        IndexSubtable::Format2(subtable) => {
            location.format = subtable.image_format();
            location.data_size =
                usize::try_from(subtable.image_size()).map_err(|_| font_data_error(font_data))?;
            location.data_offset = checked_indexed_offset(
                subtable.image_data_offset(),
                glyph_index,
                subtable.image_size(),
                font_data,
            )?;
            location.metrics = Some(
                *subtable
                    .big_metrics()
                    .first()
                    .ok_or_else(|| font_data_error(font_data))?,
            );
        }
        IndexSubtable::Format4(subtable) => {
            let glyphs = subtable.glyph_array();
            if glyphs.len() != checked_count(subtable.num_glyphs(), 1, font_data)? {
                return Err(font_data_error(font_data));
            }
            let (sentinel, glyphs) = glyphs
                .split_last()
                .ok_or_else(|| font_data_error(font_data))?;
            let Some(glyph_index) =
                selected_sparse_glyph_index(glyphs, glyph_id, selection, |entry| {
                    entry.glyph_id().to_u32()
                })?
            else {
                return Ok(None);
            };
            let start = usize::from(
                glyphs
                    .get(glyph_index)
                    .ok_or_else(|| font_data_error(font_data))?
                    .sbit_offset(),
            );
            let next_index = glyph_index
                .checked_add(1)
                .ok_or_else(|| font_data_error(font_data))?;
            let end = if let Some(entry) = glyphs.get(next_index) {
                usize::from(entry.sbit_offset())
            } else {
                if sentinel.glyph_id().to_u32() != u32::from(u16::MAX) {
                    return Err(font_data_error(font_data));
                }
                usize::from(sentinel.sbit_offset())
            };
            location.format = subtable.image_format();
            location.data_offset = start;
            location.data_size = end
                .checked_sub(start)
                .ok_or_else(|| font_data_error(font_data))?;
        }
        IndexSubtable::Format5(subtable) => {
            let glyphs = subtable.glyph_array();
            if glyphs.len() != checked_count(subtable.num_glyphs(), 0, font_data)? {
                return Err(font_data_error(font_data));
            }
            let Some(glyph_index) =
                selected_sparse_glyph_index(glyphs, glyph_id, selection, |entry| {
                    entry.get().to_u32()
                })?
            else {
                return Ok(None);
            };
            location.format = subtable.image_format();
            location.data_size =
                usize::try_from(subtable.image_size()).map_err(|_| font_data_error(font_data))?;
            location.data_offset = checked_indexed_offset(
                subtable.image_data_offset(),
                glyph_index,
                subtable.image_size(),
                font_data,
            )?;
            location.metrics = Some(
                *subtable
                    .big_metrics()
                    .first()
                    .ok_or_else(|| font_data_error(font_data))?,
            );
        }
    }
    Ok(Some(location))
}

fn select_dense_bdt_location(
    subtable: &IndexSubtable<'_>,
    glyph_index: usize,
    font_data: &FontData,
    location: &mut BitmapLocation,
) -> Result<()> {
    let next_index = glyph_index
        .checked_add(1)
        .ok_or_else(|| font_data_error(font_data))?;
    let (format, start, end) = match subtable {
        IndexSubtable::Format1(subtable) => {
            let offsets = subtable.sbit_offsets();
            let start = offsets
                .get(glyph_index)
                .ok_or_else(|| font_data_error(font_data))?
                .get();
            let end = offsets
                .get(next_index)
                .ok_or_else(|| font_data_error(font_data))?
                .get();
            (
                subtable.image_format(),
                checked_offset(subtable.image_data_offset(), start, font_data)?,
                checked_offset(subtable.image_data_offset(), end, font_data)?,
            )
        }
        IndexSubtable::Format3(subtable) => {
            let offsets = subtable.sbit_offsets();
            let start = u32::from(
                offsets
                    .get(glyph_index)
                    .ok_or_else(|| font_data_error(font_data))?
                    .get(),
            );
            let end = u32::from(
                offsets
                    .get(next_index)
                    .ok_or_else(|| font_data_error(font_data))?
                    .get(),
            );
            (
                subtable.image_format(),
                checked_offset(subtable.image_data_offset(), start, font_data)?,
                checked_offset(subtable.image_data_offset(), end, font_data)?,
            )
        }
        _ => return Err(font_data_error(font_data)),
    };
    location.format = format;
    location.data_offset = start;
    location.data_size = end
        .checked_sub(start)
        .ok_or_else(|| font_data_error(font_data))?;
    Ok(())
}

fn selected_bdt_glyph<'a>(
    bitmap_size: &BitmapSize,
    bitmap_data: RawBitmapData<'a>,
    font_data: &FontData,
) -> Result<SelectedBitmap<'a>> {
    let (width, height) = match bitmap_data.metrics {
        RawBitmapMetrics::Small(metrics) => {
            (u32::from(metrics.width()), u32::from(metrics.height()))
        }
        RawBitmapMetrics::Big(metrics) => (u32::from(metrics.width()), u32::from(metrics.height())),
    };
    let data = match (bitmap_size.bit_depth(), bitmap_data.content) {
        (32, RawBitmapContent::Data(RawBitmapDataFormat::Png, data)) => SelectedBitmapData::Png {
            data,
            expected_dimensions: Some((width, height)),
        },
        (32, RawBitmapContent::Data(RawBitmapDataFormat::ByteAligned, data)) => {
            SelectedBitmapData::Bgra(data)
        }
        (1 | 2 | 4 | 8, RawBitmapContent::Data(RawBitmapDataFormat::ByteAligned, data)) => {
            SelectedBitmapData::Mask {
                bpp: bitmap_size.bit_depth(),
                is_packed: false,
                data,
            }
        }
        (1 | 2 | 4 | 8, RawBitmapContent::Data(RawBitmapDataFormat::BitAligned, data)) => {
            SelectedBitmapData::Mask {
                bpp: bitmap_size.bit_depth(),
                is_packed: true,
                data,
            }
        }
        (1 | 2 | 4 | 8 | 32, _) => SelectedBitmapData::Unsupported,
        _ => return Err(font_data_error(font_data)),
    };
    Ok(SelectedBitmap {
        width,
        height,
        source: SelectedBitmapSource::Other,
        data,
        #[cfg(test)]
        selection: None,
    })
}

fn checked_count(count: u32, extra: usize, font_data: &FontData) -> Result<usize> {
    usize::try_from(count)
        .ok()
        .and_then(|count| count.checked_add(extra))
        .ok_or_else(|| font_data_error(font_data))
}

fn checked_glyph_index(
    glyph_id: GlyphId,
    first_glyph_id: u32,
    font_data: &FontData,
) -> Result<usize> {
    usize::try_from(glyph_id.to_u32())
        .ok()
        .and_then(|glyph_id| {
            usize::try_from(first_glyph_id)
                .ok()
                .and_then(|first_glyph_id| glyph_id.checked_sub(first_glyph_id))
        })
        .ok_or_else(|| font_data_error(font_data))
}

fn selected_sparse_glyph_index<T>(
    glyphs: &[T],
    glyph_id: GlyphId,
    selection: BitmapSelection<'_>,
    glyph_id_from_entry: impl Fn(&T) -> u32,
) -> Result<Option<usize>> {
    let font_data = selection.font;
    let selected_glyph_id = glyph_id.to_u32();
    let mut selected_index = None;
    for (index, entry) in glyphs.iter().enumerate() {
        selection.record()?;
        if glyph_id_from_entry(entry) == selected_glyph_id
            && selected_index.replace(index).is_some()
        {
            return Err(font_data_error(font_data));
        }
    }
    let Some(selected_index) = selected_index else {
        return Ok(None);
    };
    let mut failed = None;
    let found = glyphs.binary_search_by(|entry| {
        if failed.is_some() {
            return std::cmp::Ordering::Equal;
        }
        if let Err(error) = selection.record() {
            failed = Some(error);
            return std::cmp::Ordering::Equal;
        }
        glyph_id_from_entry(entry).cmp(&selected_glyph_id)
    });
    if let Some(error) = failed {
        return Err(error);
    }
    match found {
        Ok(binary_index) if binary_index == selected_index => Ok(Some(selected_index)),
        _ => Err(font_data_error(font_data)),
    }
}

fn checked_offset(base: u32, offset: u32, font_data: &FontData) -> Result<usize> {
    usize::try_from(base)
        .ok()
        .and_then(|base| {
            usize::try_from(offset)
                .ok()
                .and_then(|offset| base.checked_add(offset))
        })
        .ok_or_else(|| font_data_error(font_data))
}

fn checked_indexed_offset(
    base: u32,
    index: usize,
    stride: u32,
    font_data: &FontData,
) -> Result<usize> {
    usize::try_from(base)
        .ok()
        .and_then(|base| {
            usize::try_from(stride)
                .ok()
                .and_then(|stride| index.checked_mul(stride))
                .and_then(|offset| base.checked_add(offset))
        })
        .ok_or_else(|| font_data_error(font_data))
}

fn preflight_bitmap(
    bitmap: &SelectedBitmap<'_>,
    font_data: &FontData,
    account: &Account,
) -> Result<BitmapEncoding> {
    let account = account.at(RenderPhase::NativeGlyphBitmap);
    account.phase(RenderPhase::NativeGlyphBitmap, || {
        validate_bitmap_data(bitmap, font_data, &account)
    })
}

fn validate_bitmap_data(
    bitmap: &SelectedBitmap<'_>,
    font_data: &FontData,
    account: &Account,
) -> Result<BitmapEncoding> {
    match &bitmap.data {
        SelectedBitmapData::Bgra(data) => {
            let pixel_count = checked_pixel_count(bitmap.width, bitmap.height, font_data)?;
            let expected = pixel_count
                .checked_mul(4)
                .ok_or_else(|| font_data_error(font_data))?;
            if data.len() != expected {
                return Err(font_data_error(font_data));
            }
            Ok(BitmapEncoding::Bgra)
        }
        SelectedBitmapData::Png {
            data,
            expected_dimensions,
        } => preflight_png(data, *expected_dimensions, font_data, account),
        SelectedBitmapData::Mask {
            bpp,
            is_packed,
            data,
        } => {
            let pixel_count = checked_pixel_count(bitmap.width, bitmap.height, font_data)?;
            if !matches!(bpp, 1 | 2 | 4 | 8) {
                return Err(font_data_error(font_data));
            }
            if !is_packed {
                return Err(unsupported_image_error());
            }
            let bits = pixel_count
                .checked_mul(usize::from(*bpp))
                .ok_or_else(|| font_data_error(font_data))?;
            let expected = bits
                .checked_add(7)
                .and_then(|bits| bits.checked_div(8))
                .ok_or_else(|| font_data_error(font_data))?;
            if data.len() != expected {
                return Err(font_data_error(font_data));
            }
            let mut output = bitmap_decode_output(pixel_count, account)?;
            MaskData {
                bpp: *bpp,
                is_packed: *is_packed,
                data,
            }
            .decode_to_slice(bitmap.width, bitmap.height, &mut output)
            .map_err(|_| font_data_error(font_data))?;
            Ok(BitmapEncoding::PackedMask)
        }
        SelectedBitmapData::Unsupported => Err(unsupported_image_error()),
    }
}

fn preflight_png(
    data: &[u8],
    expected_dimensions: Option<(u32, u32)>,
    font_data: &FontData,
    account: &Account,
) -> Result<BitmapEncoding> {
    let mut decoder = png::Decoder::new(Cursor::new(data));
    decoder.set_transformations(Transformations::ALPHA | Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|_| font_data_error(font_data))?;
    let output_color_type = reader.output_color_type();
    let output_size = reader
        .output_buffer_size()
        .ok_or_else(|| font_data_error(font_data))?;
    let mut output = bitmap_decode_output(output_size, account)?;
    let info = reader
        .next_frame(&mut output)
        .map_err(|_| font_data_error(font_data))?;
    if let Some((width, height)) = expected_dimensions
        && (info.width != width || info.height != height)
    {
        return Err(font_data_error(font_data));
    }
    if output_color_type != (ColorType::Rgba, BitDepth::Eight) {
        return Err(unsupported_image_error());
    }
    Ok(BitmapEncoding::Png)
}

fn bitmap_decode_output(len: usize, account: &Account) -> Result<AccountedVec<u8>> {
    let mut output = account.temporary(len)?;
    account.work(RenderWork::NativeBitmapDecodeBytes, len)?;
    output.resize(len, 0)?;
    Ok(output)
}

fn checked_pixel_count(width: u32, height: u32, font_data: &FontData) -> Result<usize> {
    usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or_else(|| font_data_error(font_data))
}

fn font_data_error(font_data: &FontData) -> Error {
    invalid_font_data(font_data.bytes().len(), font_data.index())
}

fn missing_glyph_error(glyph_id: u32) -> Error {
    Error::invalid_value(
        "text_glyph.id",
        glyph_id,
        "must identify a drawable glyph in the selected FontData",
    )
}

fn unsupported_image_error() -> Error {
    Error::new(
        BackendErrorCode::RenderFailed,
        "internal Vello glyph image encoding is unsupported",
    )
}

struct QualifiedPen<'a> {
    elements: AccountedVec<kurbo::PathEl>,
    account: &'a Account,
    error: Option<Error>,
}
impl QualifiedPen<'_> {
    fn emit(&mut self, coordinates: usize, element: impl FnOnce() -> kurbo::PathEl) {
        if self.error.is_none() {
            self.error = self
                .account
                .work(RenderWork::PathElementVisits, 1)
                .and_then(|()| self.account.work(RenderWork::CoordinateVisits, coordinates))
                .and_then(|()| bounded::push_element(&mut self.elements, element()))
                .err();
        }
    }
}
impl OutlinePen for QualifiedPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.emit(2, || {
            kurbo::PathEl::MoveTo((f64::from(x), f64::from(y)).into())
        });
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.emit(2, || {
            kurbo::PathEl::LineTo((f64::from(x), f64::from(y)).into())
        });
    }
    fn quad_to(&mut self, ax: f32, ay: f32, x: f32, y: f32) {
        self.emit(4, || {
            kurbo::PathEl::QuadTo(
                (f64::from(ax), f64::from(ay)).into(),
                (f64::from(x), f64::from(y)).into(),
            )
        });
    }
    fn curve_to(&mut self, ax: f32, ay: f32, bx: f32, by: f32, x: f32, y: f32) {
        self.emit(6, || {
            kurbo::PathEl::CurveTo(
                (f64::from(ax), f64::from(ay)).into(),
                (f64::from(bx), f64::from(by)).into(),
                (f64::from(x), f64::from(y)).into(),
            )
        });
    }
    fn close(&mut self) {
        self.emit(0, || kurbo::PathEl::ClosePath);
    }
}

struct PreflightColorPainter<'a, 'font> {
    font_data: &'a FontData,
    outlines: &'a skrifa::OutlineGlyphCollection<'font>,
    cpal: &'a Cpal<'font>,
    size: f32,
    glyph_index: usize,
    failed: bool,
    account: &'a Account,
    error: Option<Error>,
}

impl PreflightColorPainter<'_, '_> {
    fn record(&mut self) -> bool {
        if self.failed {
            return false;
        }
        if let Err(error) = self.account.work(RenderWork::PaintVisits, 1) {
            self.error = Some(error);
            self.failed = true;
            return false;
        }
        true
    }

    fn validate_outline(&mut self, glyph_id: GlyphId) {
        if self.failed {
            return;
        }
        if let Err(error) = preflight_outline(
            self.outlines,
            glyph_id,
            self.size,
            self.font_data,
            self.glyph_index,
            self.account,
        ) {
            self.error = Some(error);
            self.failed = true;
        }
    }

    fn validate_palette_index(&mut self, palette_index: u16) {
        if self.failed || palette_index == u16::MAX {
            return;
        }
        let Some(records) = self.cpal.color_records_array() else {
            self.failed = true;
            return;
        };
        let Ok(records) = records else {
            self.failed = true;
            return;
        };
        if records.get(usize::from(palette_index)).is_none() {
            self.failed = true;
        }
    }

    fn validate_brush(&mut self, brush: ColorBrush<'_>) {
        if self.failed {
            return;
        }
        match brush {
            ColorBrush::Solid { palette_index, .. } => self.validate_palette_index(palette_index),
            ColorBrush::LinearGradient { color_stops, .. }
            | ColorBrush::RadialGradient { color_stops, .. }
            | ColorBrush::SweepGradient { color_stops, .. } => {
                for color_stop in color_stops {
                    if let Err(error) = self.account.work(RenderWork::GradientStopVisits, 1) {
                        self.error = Some(error);
                        self.failed = true;
                        break;
                    }
                    self.validate_palette_index(color_stop.palette_index);
                    if self.failed {
                        break;
                    }
                }
            }
        }
    }
}

impl ColorPainter for PreflightColorPainter<'_, '_> {
    fn push_transform(&mut self, _: skrifa::color::Transform) {
        self.record();
    }

    fn pop_transform(&mut self) {
        self.record();
    }

    fn push_clip_glyph(&mut self, glyph_id: GlyphId) {
        if !self.record() {
            return;
        }
        self.validate_outline(glyph_id);
    }

    fn push_clip_box(&mut self, _: BoundingBox<f32>) {
        self.record();
    }

    fn pop_clip(&mut self) {
        self.record();
    }

    fn fill(&mut self, brush: ColorBrush<'_>) {
        if !self.record() {
            return;
        }
        self.validate_brush(brush);
    }

    fn fill_glyph(
        &mut self,
        glyph_id: GlyphId,
        _: Option<skrifa::color::Transform>,
        brush: ColorBrush<'_>,
    ) {
        if !self.record() {
            return;
        }
        self.validate_outline(glyph_id);
        self.validate_brush(brush);
    }

    fn push_layer(&mut self, _: CompositeMode) {
        self.record();
    }
}

#[cfg(test)]
mod receiving_tests {
    use super::*;
    use crate::accounting::{RenderAttemptKind, RenderResource};
    use crate::{RenderAccountingContext, RenderAccountingOptions, RenderLimits};

    fn fixture() -> (FontData, u32, u32) {
        let bytes = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let provider = skrifa::FontRef::from_index(bytes, 0).unwrap();
        let visible = provider.charmap().map('A').unwrap().to_u32();
        let space = provider.charmap().map(' ').unwrap().to_u32();
        let font =
            FontData::try_from_bytes(bytes.to_vec(), 0, surgeist_test::fixtures::FONT_LIMITS)
                .unwrap();
        (font, visible, space)
    }

    #[test]
    fn native_png_preflight_requires_decoded_output_scratch() {
        let png = two_pixel_png();
        let bitmap = selected_png(&png);
        assert_bitmap_scratch_admission(&bitmap, 8, BitmapEncoding::Png);
    }

    fn two_pixel_png() -> Vec<u8> {
        let mut png = Vec::new();
        let mut encoder = png::Encoder::new(&mut png, 2, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&[0, 17, 34, 255, 51, 68, 85, 255])
            .unwrap();
        writer.finish().unwrap();
        png
    }

    fn selected_png(png: &[u8]) -> SelectedBitmap<'_> {
        SelectedBitmap {
            width: 2,
            height: 1,
            source: SelectedBitmapSource::Other,
            data: SelectedBitmapData::Png {
                data: png,
                expected_dimensions: Some((2, 1)),
            },
            selection: None,
        }
    }

    #[test]
    fn native_packed_mask_preflight_requires_decoded_output_scratch() {
        let bitmap = SelectedBitmap {
            width: 9,
            height: 1,
            source: SelectedBitmapSource::Other,
            data: SelectedBitmapData::Mask {
                bpp: 1,
                is_packed: true,
                data: &[0b1010_1010, 0b1000_0000],
            },
            selection: None,
        };
        assert_bitmap_scratch_admission(&bitmap, 9, BitmapEncoding::PackedMask);
    }

    fn assert_bitmap_scratch_admission(
        bitmap: &SelectedBitmap<'_>,
        decoded_bytes: u64,
        encoding: BitmapEncoding,
    ) {
        let (font, _, _) = fixture();
        for scratch_limit in [decoded_bytes - 1, decoded_bytes] {
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_scratch_bytes(scratch_limit));
            let mut context = RenderAccountingContext::new(options);
            let result = context.run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                preflight_bitmap(bitmap, &font, account).map(|result| {
                    assert_eq!(
                        std::mem::discriminant(&result),
                        std::mem::discriminant(&encoding)
                    );
                })
            });
            let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
            assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
            if scratch_limit < decoded_bytes {
                let error = result.expect_err("one-short scratch must refuse native bitmap decode");
                assert!(error.accounting_diagnostic().is_some());
                assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
                assert_eq!(usage.work(RenderWork::NativeBitmapDecodeBytes), 0);
            } else {
                result.unwrap();
                assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), decoded_bytes);
                assert_eq!(
                    usage.work(RenderWork::NativeBitmapDecodeBytes),
                    decoded_bytes
                );
            }
        }
    }

    #[test]
    fn native_png_work_refusal_and_decode_failure_release_frame_backing() {
        use crate::accounting::{
            FrameFixture, RenderClock, RenderCpuScope, RenderProfile, RendererCpu,
        };
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::Duration;

        static TICKS: AtomicU64 = AtomicU64::new(0);
        fn clock() -> Duration {
            Duration::from_nanos(TICKS.fetch_add(1, Ordering::Relaxed))
        }

        let (font, _, _) = fixture();
        let png = two_pixel_png();
        let idat = png.windows(4).position(|word| word == b"IDAT").unwrap();
        let truncated = &png[..idat + 5];
        let options = RenderAccountingOptions::new()
            .with_clock(RenderClock::Host(clock))
            .with_profile(RenderProfile::Summary { timing: true });
        let cpu = RendererCpu::new(options).unwrap();
        for (input, limit, expected_work) in [
            (png.as_slice(), 7, 0),
            (truncated, 8, 8),
            (png.as_slice(), 8, 8),
        ] {
            let options = options.with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::NativeBitmapDecodeBytes, limit)
                    .unwrap(),
            );
            let (result, _, report, fault) = FrameFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    preflight_bitmap(&selected_png(input), &font, attempt.account())
                })
                .unwrap()
                .into_parts();
            assert_eq!(fault, None);
            let attempt = report.attempt().unwrap();
            let phase = RenderPhase::NativeGlyphBitmap;
            assert_eq!(
                attempt
                    .phase_usage(phase)
                    .work(RenderWork::NativeBitmapDecodeBytes),
                expected_work
            );
            assert_eq!(
                attempt
                    .phase_usage(RenderPhase::Encode)
                    .work(RenderWork::NativeBitmapDecodeBytes),
                expected_work
            );
            assert_eq!(attempt.phase_count(phase), 1);
            assert!(attempt.phase_elapsed(phase).unwrap() > Duration::ZERO);
            let backing = report
                .renderer_cpu()
                .unwrap()
                .scope_usage(RenderCpuScope::FrameBacking);
            assert_eq!(backing.peak(RenderResource::ScratchCpuBytes), 8);
            assert_eq!(backing.live(RenderResource::ScratchCpuBytes), 0);
            if limit == 7 {
                let Some(crate::RenderAccountingError::Refused(refusal)) =
                    result.err().unwrap().accounting_diagnostic()
                else {
                    panic!("one-short decode work must refuse");
                };
                assert_eq!(
                    refusal.quantity(),
                    crate::RenderQuantity::Work(RenderWork::NativeBitmapDecodeBytes)
                );
                assert_eq!(refusal.amount(), 8);
            } else if input.len() < png.len() {
                let error = result.err().unwrap();
                assert_eq!(error.accounting_diagnostic(), None);
                assert_eq!(error.code(), font_data_error(&font).code());
                assert_eq!(error.message(), font_data_error(&font).message());
            } else {
                assert!(matches!(result, Ok(BitmapEncoding::Png)));
            }
        }
    }

    fn selected_outline(
        font: &FontData,
        glyph: u32,
        size: f32,
        index: usize,
        account: &Account,
    ) -> Result<StreamOffsets> {
        let provider = skrifa::FontRef::from_index(font.bytes(), font.index()).unwrap();
        preflight_outline(
            &provider.outline_glyphs(),
            GlyphId::new(glyph),
            size,
            font,
            index,
            account,
        )
    }

    #[test]
    fn drawable_occurrences_keep_source_identity_and_placement_after_empty_glyphs() {
        let (font, visible, space) = fixture();
        let glyphs = [
            TextGlyph::try_new(space, f64::MAX, f64::MAX, 16.0).unwrap(),
            TextGlyph::try_new(visible, 3.0, 16.0, 16.0).unwrap(),
            TextGlyph::try_new(visible, 11.0, 20.0, 16.0).unwrap(),
        ];
        let run = TextRun::try_new(
            crate::FontRef::new(1_u64).with_data(font),
            16.0,
            Transform::identity(),
            TextPaint::try_fill(crate::Paint::color(crate::Color::BLACK)).unwrap(),
            &glyphs,
            crate::TextRunBounds::unspecified(),
        )
        .unwrap();
        // The first empty source glyph contributes its representation record.
        // Refuse the next actual drawable record copy before it is appended.
        let mut limited = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::RasterResourceCopies, 1)
                    .unwrap(),
            ),
        );
        let error = limited
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                let _validated = preflight_selected_glyphs(&run, account)?;
                Ok(())
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let usage = limited.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::RasterResourceCopies), 1);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        let mut retry = RenderAccountingContext::default();
        retry
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                let validated = preflight_selected_glyphs(&run, account)?;
                let mut occurrences = validated.occurrences();
                assert_eq!(occurrences.len(), 2);
                let first = occurrences.next().unwrap();
                assert!(std::ptr::eq(first.glyph(), &glyphs[1]));
                assert_eq!(first.source_index(), 1);
                assert_eq!(first.native_position(), [3.0, 16.0]);
                let second = occurrences.next().unwrap();
                assert!(std::ptr::eq(second.glyph(), &glyphs[2]));
                assert_eq!(second.source_index(), 2);
                assert_eq!(second.native_position(), [11.0, 20.0]);
                assert!(occurrences.next().is_none());
                Ok(())
            })
            .unwrap();
        let usage = retry.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn scaled_empty_visible_outline_refuses_but_authentic_space_stays_empty() {
        let (font, visible, space) = fixture();
        let mut context = RenderAccountingContext::default();
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                selected_outline(&font, visible, 2.0_f32.powi(-43), 7, account)
            })
            .unwrap_err();
        assert_eq!(
            error.geometry_realization_diagnostic(),
            Some(&crate::GeometryRealizationError::UnrepresentableGlyph { glyph_index: 7 })
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
        let empty = context
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                selected_outline(&font, space, 2.0_f32.powi(-43), 7, account)
            })
            .unwrap();
        assert_eq!(empty.path_data, 0);
        assert_eq!(empty.path_tags, 0);
        assert_eq!(empty.transforms, 0);
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::PathElementVisits), 0);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
    }

    #[test]
    fn outline_capacity_refusal_preserves_accounting_and_allows_empty_then_fresh_retry() {
        let (font, visible, space) = fixture();
        let mut limited = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(RenderLimits::new().with_scratch_bytes(0)),
        );
        let error = limited
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                selected_outline(&font, visible, 2.0_f32.powi(-43), 4, account)
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(error.geometry_realization_diagnostic().is_none());
        let usage = limited.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::PathElementVisits), 1);
        assert_eq!(usage.peak(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        let empty = limited
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                selected_outline(&font, space, 2.0_f32.powi(-43), 4, account)
            })
            .unwrap();
        assert_eq!(empty.path_data, 0);
        let mut retry = RenderAccountingContext::default();
        let normal = retry
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                selected_outline(&font, visible, 16.0, 4, account)
            })
            .unwrap();
        assert!(normal.path_data > 0);
        assert_eq!(
            retry
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
    }

    #[test]
    fn unscaled_witness_refusal_preserves_real_prefix_cleanup_and_normal_retry() {
        let (font, visible, _) = fixture();
        // Independently observed Ahem contour: Move, three Lines, Close. The
        // scaled provider emits five records; empty-source classification walks
        // all five. Refuse the first witness record after those ten visits.
        const SCALED_PREFIX: u64 = 5 + 5;
        let mut limited = RenderAccountingContext::new(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::PathElementVisits, SCALED_PREFIX)
                    .unwrap(),
            ),
        );
        let error = limited
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                selected_outline(&font, visible, 2.0_f32.powi(-43), 2, account)
            })
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        assert!(error.geometry_realization_diagnostic().is_none());
        let usage = limited.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::PathElementVisits), SCALED_PREFIX);
        assert!(usage.peak(RenderResource::ScratchCpuBytes) > 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        let mut retry = RenderAccountingContext::default();
        let normal = retry
            .run(RenderAttemptKind::Frame, RenderPhase::Qualify, |account| {
                selected_outline(&font, visible, 16.0, 2, account)
            })
            .unwrap();
        assert!(normal.path_data > 0);
        assert_eq!(
            retry
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::ScratchCpuBytes),
            0
        );
    }
}
