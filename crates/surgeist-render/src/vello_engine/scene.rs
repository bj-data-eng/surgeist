// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Modified for surgeist-render; see NOTICE-VELLO.md for provenance and adaptations.

use peniko::{Blob, BrushRef, Fill, ImageBrushRef};
use surgeist_font::{FontResource, ResourceId};
use vello_encoding::{
    DrawBeginClip, Encoding, Glyph, GlyphRun, Patch, StreamOffsets, Transform as VelloTransform,
};

#[cfg(test)]
use vello_encoding::{DrawTag, PathTag, Style};

use crate::{BackendErrorCode, Error, Result, TextRun, encode::glyph_paint_brush};

use super::bounded::{self, EncodingBacking, Growth, PayloadCustody};
use super::dash::QualifiedDash;
use super::geometry::{self, QualifiedTransform};
use super::gradient_endpoints::{EndpointRecords, SceneBrush};
use crate::accounting::{Account, AccountedVec, RenderPhase, RenderWork, SharedValue};

use super::glyph::{
    BitmapEncoding, SelectedGlyphRepresentation, ValidatedGlyphRun, glyph_geometry_error,
    preflight_selected_glyphs, qualify_native_outline,
};
use super::raster::{self, PreparedVelloPass, RasterParameters};

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VelloRasterScenario {
    Base,
    LargePath,
    Clip,
    LargePathAndClip,
}

#[derive(Default)]
pub(crate) struct VelloScene {
    encoding: Encoding,
    backing: EncodingBacking,
    glyph_sizes: StreamOffsets,
    endpoints: EndpointRecords,
    image_expansions: super::image_atlas::ImageExpansions,
    font_backings: Option<AccountedVec<(ResourceId, Blob<u8>)>>,
    // Drop source owners before releasing their CPU payload leases.
    payloads: PayloadCustody,
}

impl Drop for VelloScene {
    fn drop(&mut self) {
        std::mem::take(&mut self.backing).dispose(std::mem::take(&mut self.encoding));
    }
}

/// Coupled geometry and paint settings for one rounded-rectangle blur.
#[derive(Clone, Copy)]
pub(crate) struct RoundedRectBlur {
    pub(crate) rect: kurbo::Rect,
    pub(crate) color: peniko::Color,
    pub(crate) radius: f64,
    pub(crate) std_dev: f64,
}

struct DecodedFontBytes(FontResource);

impl AsRef<[u8]> for DecodedFontBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.bytes()
    }
}

fn geometry_action<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

// The immutable style borrow is the same one qualified and encoded. Empty
// strokes are eliminated by their owning branch before constructing this state.
struct QualifiedStroke<'a>(&'a kurbo::Stroke);

impl<'a> QualifiedStroke<'a> {
    fn try_new(
        style: &'a kurbo::Stroke,
        transform: &QualifiedTransform,
        account: &Account,
    ) -> Result<Self> {
        let width = geometry_action(account, || style.width)?;
        if geometry_action(account, || width == 0.0)? {
            return Err(crate::GeometryRealizationError::UnrepresentableStroke.into());
        }
        geometry::qualify_stroke(style, transform, account)?;
        Ok(Self(style))
    }

    fn encode(self, encoding: &mut Encoding) -> Result<()> {
        if !encoding.encode_stroke_style(self.0) {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "locked native nonzero stroke correspondence changed",
            ));
        }
        Ok(())
    }
}

impl VelloScene {
    pub(crate) fn import_image(&mut self, image: &crate::Image, account: &Account) -> Result<()> {
        self.payloads.keep(account.import_image(image)?, account)
    }

    /// Associates integer expansion with only the matching image draws appended
    /// by this callback. Geometry stays in its original coordinates; the callback
    /// applies the supplied compensation to the image brush transform.
    pub(crate) fn with_pixelated_image<T>(
        &mut self,
        image: &crate::Image,
        source_to_device: kurbo::Affine,
        encode: impl FnOnce(&mut Self, kurbo::Affine) -> Result<T>,
        account: &Account,
    ) -> Result<T> {
        self.import_image(image, account)?;
        if image.data.width == 0 || image.data.height == 0 {
            return encode(self, kurbo::Affine::IDENTITY);
        }
        let expansion = super::image_atlas::ImageExpansion::try_new(&image.data, source_to_device)?;
        let first_patch = self.encoding.resources.patches.len();
        let result = encode(self, expansion.compensation())?;
        self.mark_image_expansion(first_patch, image.data.data.id(), expansion, account)?;
        Ok(result)
    }

    fn mark_image_expansion(
        &mut self,
        first_patch: usize,
        source_id: u64,
        expansion: super::image_atlas::ImageExpansion,
        account: &Account,
    ) -> Result<()> {
        for patch in &self.encoding.resources.patches[first_patch..] {
            account.work(RenderWork::ResourceProbes, 1)?;
            if let Patch::Image {
                image,
                draw_data_offset,
            } = patch
                && image.data.id() == source_id
            {
                self.image_expansions
                    .insert(*draw_data_offset, expansion, account)?;
            }
        }
        Ok(())
    }

    fn transform_and_style(
        &mut self,
        transform: kurbo::Affine,
        fill: Option<Fill>,
        stroke: Option<&kurbo::Stroke>,
        account: &Account,
    ) -> Result<QualifiedTransform> {
        let qualified = geometry::qualify_transform(transform, account)?;
        let stroke = stroke
            .map(|style| QualifiedStroke::try_new(style, &qualified, account))
            .transpose()?;
        self.backing.reserve(
            &mut self.encoding,
            Growth {
                path_tags: 2,
                transforms: 1,
                styles: 1,
                ..Growth::default()
            },
            account,
        )?;
        self.encoding.encode_transform(qualified.native());
        if let Some(fill) = fill {
            self.encoding.encode_fill_style(fill);
        }
        if let Some(stroke) = stroke {
            stroke.encode(&mut self.encoding)?;
        }
        Ok(qualified)
    }

    fn brush(
        &mut self,
        brush: SceneBrush<'_>,
        effective_transform: &QualifiedTransform,
        account: &Account,
    ) -> Result<()> {
        let (brush, sampling) = brush.qualify(effective_transform, account)?.into_parts();
        account.work(RenderWork::PaintVisits, 1)?;
        if matches!(brush, BrushRef::Image(_)) {
            geometry::qualify_inverse(effective_transform, account)?;
        }
        let word = self.encoding.draw_data.len();
        let tag_index = self.encoding.draw_tags.len();
        self.backing
            .reserve(&mut self.encoding, bounded::brush_growth(brush), account)?;
        if let BrushRef::Gradient(gradient) = brush {
            for _ in &gradient.stops.0 {
                account.work(RenderWork::GradientStopVisits, 1)?;
            }
        }
        if let BrushRef::Image(image) = brush {
            self.payloads
                .keep(account.import_image_data(image.image)?, account)?;
        }
        self.encoding.encode_brush(brush, 1.0);
        if let Some(sampling) = sampling {
            account.work(RenderWork::ResourceProbes, 1)?;
            let Some([tag]) = self.encoding.draw_tags.get(tag_index..) else {
                return Err(Error::new(
                    BackendErrorCode::RenderFailed,
                    "native brush draw-record count changed",
                ));
            };
            self.endpoints.capture(word, sampling, *tag, account)?;
        }
        Ok(())
    }

    fn brush_transform(
        &mut self,
        transform: kurbo::Affine,
        account: &Account,
    ) -> Result<QualifiedTransform> {
        let qualified = geometry::qualify_transform(transform, account)?;
        self.backing.reserve(
            &mut self.encoding,
            Growth {
                path_tags: 1,
                transforms: 1,
                ..Growth::default()
            },
            account,
        )?;
        if self.encoding.encode_transform(qualified.native()) {
            self.encoding.swap_last_path_tags();
        }
        Ok(qualified)
    }

    pub(crate) fn fill_elements<'a>(
        &mut self,
        transform: kurbo::Affine,
        brush: impl Into<SceneBrush<'a>>,
        brush_transform: Option<kurbo::Affine>,
        elements: &[kurbo::PathEl],
        account: &Account,
    ) -> Result<()> {
        self.fill_path(
            Fill::NonZero,
            transform,
            brush.into(),
            brush_transform,
            elements,
            account,
        )
    }

    fn fill_path(
        &mut self,
        style: Fill,
        transform: kurbo::Affine,
        brush: SceneBrush<'_>,
        brush_transform: Option<kurbo::Affine>,
        elements: &[kurbo::PathEl],
        account: &Account,
    ) -> Result<()> {
        if !geometry::has_segments(elements, account)? {
            return Ok(());
        }
        let qualified = self.transform_and_style(transform, Some(style), None, account)?;
        self.backing
            .path(&mut self.encoding, elements.len(), account)?;
        if bounded::encode_elements(&mut self.encoding, elements, true, &qualified, account)? {
            let effective = if let Some(brush_transform) = brush_transform {
                self.brush_transform(
                    geometry::affine_product(transform, brush_transform, account)?,
                    account,
                )?
            } else {
                qualified
            };
            self.brush(brush, &effective, account)?;
        }
        Ok(())
    }

    pub(crate) fn fill<'a>(
        &mut self,
        style: Fill,
        transform: kurbo::Affine,
        brush: impl Into<SceneBrush<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
        account: &Account,
    ) -> Result<()> {
        let elements = bounded::materialize(shape.path_elements(0.1), account)?;
        self.fill_path(
            style,
            transform,
            brush.into(),
            brush_transform,
            &elements,
            account,
        )
    }

    pub(crate) fn stroke<'a>(
        &mut self,
        style: &kurbo::Stroke,
        transform: kurbo::Affine,
        brush: impl Into<SceneBrush<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
        account: &Account,
    ) -> Result<()> {
        if style.width == 0.0 {
            return Ok(());
        }
        let tolerance = if geometry_action(account, || style.dash_pattern.is_empty())? {
            0.1
        } else {
            0.01
        };
        let source = bounded::materialize(shape.path_elements(tolerance), account)?;
        if !geometry::has_segments(&source, account)? {
            return Ok(());
        }
        let elements = QualifiedDash::try_new(style, account)?.materialize(source, account)?;
        if !geometry::has_segments(&elements, account)? {
            return Ok(());
        }
        let qualified = self.transform_and_style(transform, None, Some(style), account)?;
        self.backing
            .path(&mut self.encoding, elements.len(), account)?;
        if bounded::encode_elements(&mut self.encoding, &elements, false, &qualified, account)? {
            let effective = if let Some(brush_transform) = brush_transform {
                self.brush_transform(
                    geometry::affine_product(transform, brush_transform, account)?,
                    account,
                )?
            } else {
                qualified
            };
            self.brush(brush.into(), &effective, account)?;
        }
        Ok(())
    }

    pub(crate) fn draw_image<'a>(
        &mut self,
        image: impl Into<ImageBrushRef<'a>>,
        transform: kurbo::Affine,
        account: &Account,
    ) -> Result<()> {
        let image = image.into();
        let rect = kurbo::Rect::new(
            0.0,
            0.0,
            f64::from(image.image.width),
            f64::from(image.image.height),
        );
        self.fill(Fill::NonZero, transform, image, None, &rect, account)
    }

    pub(crate) fn push_layer(
        &mut self,
        fill: Fill,
        blend: peniko::BlendMode,
        alpha: f32,
        transform: kurbo::Affine,
        clip: &impl kurbo::Shape,
        account: &Account,
    ) -> Result<()> {
        self.push_layer_inner(
            DrawBeginClip::new(blend, alpha.clamp(0.0, 1.0)),
            fill,
            transform,
            clip,
            account,
        )
    }

    pub(crate) fn push_clip_layer(
        &mut self,
        fill: Fill,
        transform: kurbo::Affine,
        clip: &impl kurbo::Shape,
        account: &Account,
    ) -> Result<()> {
        self.push_layer_inner(DrawBeginClip::clip(), fill, transform, clip, account)
    }

    pub(crate) fn pop_layer(&mut self, account: &Account) -> Result<()> {
        self.backing.reserve(
            &mut self.encoding,
            Growth {
                path_tags: 1,
                draw_tags: 1,
                ..Growth::default()
            },
            account,
        )?;
        self.encoding.encode_end_clip();
        Ok(())
    }

    pub(crate) fn draw_blurred_rounded_rect(
        &mut self,
        transform: kurbo::Affine,
        rect: kurbo::Rect,
        brush: peniko::Color,
        radius: f64,
        std_dev: f64,
        account: &Account,
    ) -> Result<()> {
        let kernel_size = geometry_action(account, || 2.5 * std_dev)?;
        account.work(RenderWork::CoordinateVisits, 4)?;
        let (x0, y0, x1, y1) = (rect.x0, rect.y0, rect.x1, rect.y1);
        let x0 = geometry_action(account, || x0 - kernel_size)?;
        let y0 = geometry_action(account, || y0 - kernel_size)?;
        let x1 = geometry_action(account, || x1 + kernel_size)?;
        let y1 = geometry_action(account, || y1 + kernel_size)?;
        let inflated = kurbo::Rect::new(x0, y0, x1, y1);
        self.draw_blurred_rounded_rect_in(
            &inflated,
            transform,
            RoundedRectBlur {
                rect,
                color: brush,
                radius,
                std_dev,
            },
            account,
        )
    }

    pub(crate) fn draw_blurred_rounded_rect_in(
        &mut self,
        shape: &impl kurbo::Shape,
        transform: kurbo::Affine,
        blur: RoundedRectBlur,
        account: &Account,
    ) -> Result<()> {
        let RoundedRectBlur {
            rect,
            color: brush,
            radius,
            std_dev,
        } = blur;
        let elements = bounded::materialize(shape.path_elements(0.1), account)?;
        if !geometry::has_segments(&elements, account)? {
            return Ok(());
        }
        let qualified = self.transform_and_style(transform, Some(Fill::NonZero), None, account)?;
        self.backing
            .path(&mut self.encoding, elements.len(), account)?;
        if bounded::encode_elements(&mut self.encoding, &elements, true, &qualified, account)? {
            account.work(RenderWork::CoordinateVisits, 4)?;
            let (x0, y0, x1, y1) = (rect.x0, rect.y0, rect.x1, rect.y1);
            let width = geometry_action(account, || x1 - x0)?;
            let height = geometry_action(account, || y1 - y0)?;
            let sum_x = geometry_action(account, || x0 + x1)?;
            let sum_y = geometry_action(account, || y0 + y1)?;
            let center_x = geometry_action(account, || 0.5 * sum_x)?;
            let center_y = geometry_action(account, || 0.5 * sum_y)?;
            let center = kurbo::Affine::translate((center_x, center_y));
            let blur_transform = self.brush_transform(
                geometry::affine_product(transform, center, account)?,
                account,
            )?;
            geometry::qualify_inverse(&blur_transform, account)?;
            let reason = crate::GeometryRealizationError::UnrepresentableBlur;
            let width = geometry::scalar(width, reason, account)?;
            let height = geometry::scalar(height, reason, account)?;
            let radius = geometry::scalar(radius, reason, account)?;
            let std_dev = geometry::scalar(std_dev, reason, account)?;
            self.backing.reserve(
                &mut self.encoding,
                Growth {
                    draw_tags: 1,
                    draw_data: std::mem::size_of::<vello_encoding::DrawBlurRoundedRect>() / 4,
                    ..Growth::default()
                },
                account,
            )?;
            account.work(RenderWork::PaintVisits, 1)?;
            self.encoding
                .encode_blurred_rounded_rect(brush, width, height, radius, std_dev);
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn encode_text_run(&mut self, run: &TextRun<'_>, account: &Account) -> Result<()> {
        let validated = preflight_selected_glyphs(run, account)?;
        self.append_validated_glyph_run(
            validated,
            kurbo::Affine::IDENTITY,
            crate::PhysicalSize::new(2, 2),
            account,
        )
    }

    pub(crate) fn encode_text_run_with_transform(
        &mut self,
        run: &TextRun<'_>,
        transform: kurbo::Affine,
        target_extent: crate::PhysicalSize,
        account: &Account,
    ) -> Result<()> {
        let validated = preflight_selected_glyphs(run, account)?;
        self.append_validated_glyph_run(validated, transform, target_extent, account)
    }

    pub(crate) fn prepare_raster(
        &self,
        parameters: RasterParameters,
        account: &Account,
    ) -> Result<PreparedVelloPass> {
        let account = account.at(RenderPhase::Encode);
        account.phase(RenderPhase::Encode, || {
            raster::prepare(
                &self.encoding,
                &self.image_expansions,
                &self.glyph_sizes,
                self.endpoints.as_slice(),
                parameters,
                &account,
            )
        })
    }

    fn append_validated_glyph_run(
        &mut self,
        validated: ValidatedGlyphRun<'_>,
        transform: kurbo::Affine,
        target_extent: crate::PhysicalSize,
        account: &Account,
    ) -> Result<()> {
        let mut unsupported = None;
        for representation in validated.representations() {
            account.work(RenderWork::GlyphVisits, 1)?;
            if !matches!(representation, SelectedGlyphRepresentation::Outline) {
                unsupported = Some(representation);
                break;
            }
        }
        if let Some(representation) = unsupported {
            let context = match representation {
                SelectedGlyphRepresentation::Outline => "outline glyph",
                SelectedGlyphRepresentation::Colr => "COLR glyph",
                SelectedGlyphRepresentation::Bitmap(BitmapEncoding::Bgra) => "BGRA bitmap glyph",
                SelectedGlyphRepresentation::Bitmap(BitmapEncoding::Png) => "PNG bitmap glyph",
                SelectedGlyphRepresentation::Bitmap(BitmapEncoding::PackedMask) => {
                    "packed-mask bitmap glyph"
                }
            };
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                format!("internal Vello scene cannot append selected {context}"),
            ));
        }
        let Some(first_occurrence) = validated.occurrences().next() else {
            return Ok(());
        };
        if let crate::paint::PaintKind::Image(image) = validated.paint().fill().kind() {
            self.import_image(image, account)?;
        }
        account.work(RenderWork::GlyphVisits, 1)?;
        let first_drawable = first_occurrence.source_index();
        let paint_transform = geometry::affine_product(
            transform,
            kurbo::Affine::from(validated.transform()),
            account,
        )
        .map_err(|error| glyph_geometry_error(error, first_drawable))?;
        let native_run = geometry::qualify_transform(paint_transform, account)
            .map_err(|error| glyph_geometry_error(error, first_drawable))?;
        let brush = glyph_paint_brush(
            validated.paint().fill(),
            paint_transform,
            target_extent,
            account,
        )?;
        let gradient_transform =
            brush.native_glyph_transform(paint_transform, target_extent, account)?;
        let image_expansion = match validated.paint().fill().kind() {
            crate::paint::PaintKind::Image(image)
                if image.rendering_hint() == Some(crate::ImageRendering::Pixelated)
                    && image.data.width != 0
                    && image.data.height != 0 =>
            {
                Some((
                    image.data.data.id(),
                    super::image_atlas::ImageExpansion::try_new(&image.data, paint_transform)?,
                ))
            }
            _ => None,
        };
        self.backing.reserve(
            &mut self.encoding,
            Growth {
                glyphs: validated.drawable_count(),
                glyph_runs: 1,
                patches: 1,
                ..Growth::default()
            },
            account,
        )?;
        let stream_offsets = self.encoding.stream_offsets();
        let glyph_start = self.encoding.resources.glyphs.len();
        for occurrence in validated.occurrences() {
            account.work(RenderWork::GlyphVisits, 1)?;
            let glyph = occurrence.glyph();
            let glyph_index = occurrence.source_index();
            account.work(RenderWork::CoordinateVisits, 2)?;
            let [x, y] = occurrence.native_position();
            let native_placement = VelloTransform {
                matrix: [1.0, 0.0, 0.0, -1.0],
                translation: [x, y],
            };
            let product = geometry::native_product(native_run.native(), native_placement, account)
                .map_err(|error| glyph_geometry_error(error, glyph_index))?;
            account.work(RenderWork::CoordinateVisits, 2)?;
            let source_position = kurbo::Affine::new([1.0, 0.0, 0.0, -1.0, glyph.x(), glyph.y()]);
            let source_product =
                geometry::affine_product(paint_transform, source_position, account)
                    .map_err(|error| glyph_geometry_error(error, glyph_index))?;
            let qualified = geometry::qualify_native_transform(source_product, product, account)
                .map_err(|error| glyph_geometry_error(error, glyph_index))?;
            qualify_native_outline(occurrence, &qualified, account)?;
            self.encoding.resources.glyphs.push(Glyph {
                id: glyph.id(),
                x,
                y,
            });
        }
        let glyph_end = self.encoding.resources.glyphs.len();
        if glyph_start == glyph_end {
            return Ok(());
        }
        // The glyph resolver shares its cache within this encoding. Use one
        // blob identity per backing and keep the original face index on each run.
        let native_brush = if let Some((_, expansion)) = image_expansion {
            Some(
                geometry::qualify_transform(expansion.compensation(), account)
                    .map_err(|error| glyph_geometry_error(error, first_drawable))?
                    .native(),
            )
        } else {
            gradient_transform
        };
        let native_paint = if let Some(native_brush) = native_brush {
            account.work(RenderWork::CoordinateVisits, 6)?;
            let brush_source = native_brush.to_kurbo();
            let source_product = geometry::affine_product(paint_transform, brush_source, account)
                .map_err(|error| glyph_geometry_error(error, first_drawable))?;
            let actual_product =
                geometry::native_product(native_run.native(), native_brush, account)
                    .map_err(|error| glyph_geometry_error(error, first_drawable))?;
            geometry::qualify_native_transform(source_product, actual_product, account)
                .map_err(|error| glyph_geometry_error(error, first_drawable))?
        } else {
            // Resolver composes this actual identity brush factor as well.
            let actual_product =
                geometry::native_product(native_run.native(), VelloTransform::IDENTITY, account)
                    .map_err(|error| glyph_geometry_error(error, first_drawable))?;
            geometry::qualify_native_transform(paint_transform, actual_product, account)
                .map_err(|error| glyph_geometry_error(error, first_drawable))?
        };
        let face = validated.font_data().admitted_face();
        let resource = face.resource();
        let backing = self.font_backing(resource, account)?;
        let index = self.encoding.resources.glyph_runs.len();
        self.encoding.resources.glyph_runs.push(GlyphRun {
            font: peniko::FontData::new(backing, face.index()),
            transform: native_run.native(),
            glyph_transform: None,
            brush_transform: native_brush,
            font_size: validated.size(),
            font_embolden: vello_encoding::FontEmbolden::default(),
            hint: false,
            normalized_coords: self.encoding.resources.normalized_coords.len()
                ..self.encoding.resources.normalized_coords.len(),
            style: Fill::NonZero.into(),
            glyphs: glyph_start..glyph_end,
            stream_offsets,
        });
        self.encoding
            .resources
            .patches
            .push(Patch::GlyphRun { index });
        let first_patch = self.encoding.resources.patches.len();
        self.brush(brush.scene_brush(target_extent), &native_paint, account)?;
        if let Some((source_id, expansion)) = image_expansion {
            self.mark_image_expansion(first_patch, source_id, expansion, account)?;
        }
        bounded::sum_streams(&mut self.glyph_sizes, validated.stream_sizes(), account)?;
        self.encoding.force_next_transform_and_style();
        Ok(())
    }

    fn font_backing(&mut self, resource: &FontResource, account: &Account) -> Result<Blob<u8>> {
        if let Some(backings) = &self.font_backings {
            for (identity, backing) in backings {
                account.work(RenderWork::ResourceProbes, 1)?;
                if *identity == resource.identity() {
                    account.work(RenderWork::RasterResourceCopies, 1)?;
                    return Ok(backing.clone());
                }
            }
        }
        let import = account.import_font(resource)?;
        account.work(RenderWork::RasterResourceCopies, 1)?;
        let source = resource.clone();
        let bytes =
            SharedValue::with_parts(DecodedFontBytes(source), None, import, &account.storage())?;
        let backing = Blob::new(bytes.into_bytes_arc());
        let backings = match &mut self.font_backings {
            Some(backings) => backings,
            slot @ None => slot.insert(account.retained(0)?),
        };
        bounded::reserve_record(backings)?;
        let slot = backings.append_slot()?;
        account.work(RenderWork::RasterResourceCopies, 1)?;
        let registered = backing.clone();
        account.work(RenderWork::RasterResourceCopies, 1)?;
        slot.write((resource.identity(), registered));
        Ok(backing)
    }

    fn push_layer_inner(
        &mut self,
        parameters: DrawBeginClip,
        fill: Fill,
        transform: kurbo::Affine,
        clip: &impl kurbo::Shape,
        account: &Account,
    ) -> Result<()> {
        let elements = bounded::materialize(clip.path_elements(0.1), account)?;
        if geometry::has_segments(&elements, account)? {
            let qualified = self.transform_and_style(transform, Some(fill), None, account)?;
            self.backing
                .path(&mut self.encoding, elements.len(), account)?;
            if !bounded::encode_elements(&mut self.encoding, &elements, true, &qualified, account)?
            {
                self.backing.path(&mut self.encoding, 0, account)?;
                self.encoding.encode_empty_shape();
            }
        } else {
            self.backing.path(&mut self.encoding, 0, account)?;
            self.encoding.encode_empty_shape();
        }
        self.backing.reserve(
            &mut self.encoding,
            Growth {
                draw_tags: 1,
                draw_data: std::mem::size_of::<DrawBeginClip>() / 4,
                ..Growth::default()
            },
            account,
        )?;
        self.encoding.encode_begin_clip(parameters);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn prepare_raster_scenario_for_test(
        scenario: VelloRasterScenario,
        parameters: RasterParameters,
    ) -> Result<PreparedVelloPass> {
        let account = Account::for_test();
        let mut scene = Self::default();
        scene.append_raster_scenario_for_test(scenario, &account)?;
        scene.prepare_raster(parameters, &account)
    }

    #[cfg(test)]
    pub(crate) fn observation_for_test(&self) -> VelloSceneObservation<'_> {
        VelloSceneObservation {
            encoding: &self.encoding,
            glyph_runs: &self.encoding.resources.glyph_runs,
            glyphs: &self.encoding.resources.glyphs,
            patch_count: self.encoding.resources.patches.len(),
            normalized_coordinate_count: self.encoding.resources.normalized_coords.len(),
        }
    }

    #[cfg(test)]
    fn append_raster_scenario_for_test(
        &mut self,
        scenario: VelloRasterScenario,
        account: &Account,
    ) -> Result<()> {
        match scenario {
            VelloRasterScenario::Base => Ok(()),
            VelloRasterScenario::LargePath => self.append_large_path_scenario_for_test(account),
            VelloRasterScenario::Clip => self.append_clip_scenario_for_test(account),
            VelloRasterScenario::LargePathAndClip => {
                self.append_large_path_scenario_for_test(account)?;
                self.append_clip_scenario_for_test(account)
            }
        }
    }

    #[cfg(test)]
    fn append_large_path_scenario_for_test(&mut self, account: &Account) -> Result<()> {
        const LARGE_PATH_SCAN_LINE_SEGMENTS: usize = 4 * 256 * 256 + 1;
        let qualified =
            self.transform_and_style(kurbo::Affine::IDENTITY, Some(Fill::NonZero), None, account)?;
        self.backing.path(
            &mut self.encoding,
            LARGE_PATH_SCAN_LINE_SEGMENTS + 1,
            account,
        )?;
        let mut path = self.encoding.encode_path(true);
        account.work(RenderWork::PathElementVisits, 1)?;
        path.move_to(0.0, 0.0);
        let mut x = 1.0;
        for segment in 0..LARGE_PATH_SCAN_LINE_SEGMENTS {
            account.work(RenderWork::PathElementVisits, 1)?;
            path.line_to(x, if segment % 2 == 0 { 1.0 } else { 0.0 });
            x += 1.0;
        }
        path.finish(true);
        self.brush(peniko::Color::BLACK.into(), &qualified, account)
    }

    #[cfg(test)]
    fn append_clip_scenario_for_test(&mut self, account: &Account) -> Result<()> {
        const CLIP_PAIRS_FOR_REDUCE: usize = 129;
        for _ in 0..CLIP_PAIRS_FOR_REDUCE {
            self.push_clip_layer(
                Fill::NonZero,
                kurbo::Affine::IDENTITY,
                &kurbo::Rect::new(0.0, 0.0, 1.0, 1.0),
                account,
            )?;
            self.pop_layer(account)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) struct VelloSceneObservation<'a> {
    encoding: &'a Encoding,
    glyph_runs: &'a [GlyphRun],
    glyphs: &'a [Glyph],
    patch_count: usize,
    normalized_coordinate_count: usize,
}

#[cfg(test)]
impl VelloSceneObservation<'_> {
    pub(crate) fn glyph_font_ids_for_test(&self) -> impl Iterator<Item = u64> + '_ {
        self.glyph_runs.iter().map(|run| run.font.data.id())
    }

    pub(crate) const fn glyph_run_count_for_test(&self) -> usize {
        self.glyph_runs.len()
    }

    pub(crate) const fn glyph_count_for_test(&self) -> usize {
        self.glyphs.len()
    }

    pub(crate) const fn patch_count_for_test(&self) -> usize {
        self.patch_count
    }

    pub(crate) const fn normalized_coordinate_count_for_test(&self) -> usize {
        self.normalized_coordinate_count
    }

    pub(crate) fn first_glyph_run_for_test(&self) -> Option<VelloGlyphRunObservation<'_>> {
        self.glyph_runs
            .first()
            .map(|glyph_run| VelloGlyphRunObservation { glyph_run })
    }

    pub(crate) fn first_glyph_for_test(&self) -> Option<VelloGlyphObservation<'_>> {
        self.glyphs
            .first()
            .map(|glyph| VelloGlyphObservation { glyph })
    }

    pub(crate) fn solid_path_draws_for_test(&self) -> Option<Vec<VelloPathDrawObservationForTest>> {
        // Decode the emitted Vello streams themselves so this observation cannot
        // stay green when a scene method stops recording one of these facts.
        let paths = decode_vello_paths_for_test(self.encoding)?;
        let draws = decode_vello_draws_for_test(self.encoding)?;
        Some(
            paths
                .into_iter()
                .zip(draws)
                .map(
                    |((transform, fill_rule, geometry), draw)| VelloPathDrawObservationForTest {
                        transform,
                        fill_rule,
                        geometry,
                        draw,
                    },
                )
                .collect(),
        )
    }

    pub(crate) fn path_geometry_for_test(&self) -> Option<Vec<EncodedVelloPathForTest>> {
        decode_vello_paths_for_test(self.encoding)
    }
}

#[cfg(test)]
pub(crate) type EncodedVelloPathForTest = (
    [f32; 6],
    VelloFillRuleObservationForTest,
    Vec<VelloPathElementObservationForTest>,
);

#[cfg(test)]
fn decode_vello_paths_for_test(encoding: &Encoding) -> Option<Vec<EncodedVelloPathForTest>> {
    let mut paths = Vec::with_capacity(encoding.n_paths as usize);
    let mut geometry = Vec::new();
    let mut path_data_index = 0;
    let mut transform_index = 0;
    let mut style_index = 0;
    let mut current_transform = None;
    let mut geometry_transform = None;
    let mut current_fill_rule = None;
    let mut starts_subpath = true;
    let mut subpath_start = None;

    for tag in &encoding.path_tags {
        if *tag == PathTag::TRANSFORM {
            let transform = *encoding.transforms.get(transform_index)?;
            transform_index += 1;
            current_transform = Some([
                transform.matrix[0],
                transform.matrix[1],
                transform.matrix[2],
                transform.matrix[3],
                transform.translation[0],
                transform.translation[1],
            ]);
        } else if *tag == PathTag::STYLE {
            let style = *encoding.styles.get(style_index)?;
            style_index += 1;
            current_fill_rule = Some(vello_fill_rule_for_test(style)?);
        } else if *tag == PathTag::PATH {
            if !starts_subpath {
                return None;
            }
            paths.push((
                geometry_transform.take().or(current_transform)?,
                current_fill_rule?,
                std::mem::take(&mut geometry),
            ));
        } else if tag.is_path_segment() {
            if !tag.is_f32() {
                return None;
            }
            // Geometry uses the transform at its segment tags. A subsequent
            // brush transform immediately before PATH belongs to the paint.
            let transform = current_transform?;
            if geometry_transform.is_some_and(|actual| actual != transform) {
                return None;
            }
            geometry_transform = Some(transform);
            if starts_subpath {
                let start = read_vello_point_for_test(&encoding.path_data, &mut path_data_index)?;
                geometry.push(VelloPathElementObservationForTest::MoveTo(start));
                subpath_start = Some(start);
                starts_subpath = false;
            }
            geometry.push(decode_vello_path_segment_for_test(
                *tag,
                &encoding.path_data,
                &mut path_data_index,
            )?);
            if tag.is_subpath_end() {
                if matches!(
                    geometry.last(),
                    Some(VelloPathElementObservationForTest::LineTo(point))
                        if Some(*point) == subpath_start
                ) {
                    geometry.pop();
                }
                geometry.push(VelloPathElementObservationForTest::Close);
                starts_subpath = true;
                subpath_start = None;
            }
        } else {
            return None;
        }
    }

    if path_data_index != encoding.path_data.len()
        || transform_index != encoding.transforms.len()
        || style_index != encoding.styles.len()
        || paths.len() != encoding.draw_tags.len()
        || paths.len() != encoding.n_paths as usize
    {
        return None;
    }
    Some(paths)
}

#[cfg(test)]
fn decode_vello_path_segment_for_test(
    tag: PathTag,
    data: &[u32],
    index: &mut usize,
) -> Option<VelloPathElementObservationForTest> {
    match tag.path_segment_type().0 {
        1 => Some(VelloPathElementObservationForTest::LineTo(
            read_vello_point_for_test(data, index)?,
        )),
        2 => Some(VelloPathElementObservationForTest::QuadTo(
            read_vello_point_for_test(data, index)?,
            read_vello_point_for_test(data, index)?,
        )),
        3 => Some(VelloPathElementObservationForTest::CubicTo(
            read_vello_point_for_test(data, index)?,
            read_vello_point_for_test(data, index)?,
            read_vello_point_for_test(data, index)?,
        )),
        _ => None,
    }
}

#[cfg(test)]
fn decode_vello_draws_for_test(encoding: &Encoding) -> Option<Vec<VelloDrawObservationForTest>> {
    let mut draw_data_index = 0;
    let draws = encoding
        .draw_tags
        .iter()
        .map(|tag| {
            if *tag == DrawTag::BEGIN_CLIP {
                let blend_mode = *encoding.draw_data.get(draw_data_index)?;
                let alpha = f32::from_bits(*encoding.draw_data.get(draw_data_index + 1)?);
                draw_data_index += 2;
                Some(VelloDrawObservationForTest::BeginClip { blend_mode, alpha })
            } else if *tag == DrawTag::COLOR {
                let rgba = *encoding.draw_data.get(draw_data_index)?;
                draw_data_index += 1;
                Some(VelloDrawObservationForTest::SolidColor { rgba })
            } else if *tag == DrawTag::END_CLIP {
                Some(VelloDrawObservationForTest::EndClip)
            } else {
                None
            }
        })
        .collect::<Option<Vec<_>>>()?;
    (draw_data_index == encoding.draw_data.len()).then_some(draws)
}

#[cfg(test)]
fn read_vello_point_for_test(data: &[u32], index: &mut usize) -> Option<[f32; 2]> {
    let point = [
        f32::from_bits(*data.get(*index)?),
        f32::from_bits(*data.get(*index + 1)?),
    ];
    *index += 2;
    Some(point)
}

#[cfg(test)]
fn vello_fill_rule_for_test(style: Style) -> Option<VelloFillRuleObservationForTest> {
    if style.line_width != 0.0 {
        return None;
    }
    match style.flags_and_miter_limit {
        0 => Some(VelloFillRuleObservationForTest::NonZero),
        Style::FLAGS_FILL_BIT => Some(VelloFillRuleObservationForTest::EvenOdd),
        _ => None,
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VelloFillRuleObservationForTest {
    NonZero,
    EvenOdd,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VelloPathElementObservationForTest {
    MoveTo([f32; 2]),
    LineTo([f32; 2]),
    QuadTo([f32; 2], [f32; 2]),
    CubicTo([f32; 2], [f32; 2], [f32; 2]),
    Close,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VelloDrawObservationForTest {
    BeginClip { blend_mode: u32, alpha: f32 },
    SolidColor { rgba: u32 },
    EndClip,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VelloPathDrawObservationForTest {
    pub(crate) transform: [f32; 6],
    pub(crate) fill_rule: VelloFillRuleObservationForTest,
    pub(crate) geometry: Vec<VelloPathElementObservationForTest>,
    pub(crate) draw: VelloDrawObservationForTest,
}

#[cfg(test)]
pub(crate) struct VelloGlyphRunObservation<'a> {
    glyph_run: &'a GlyphRun,
}

#[cfg(test)]
impl VelloGlyphRunObservation<'_> {
    pub(crate) const fn font_collection_index_for_test(&self) -> u32 {
        self.glyph_run.font.index
    }

    pub(crate) fn font_data_matches_for_test(&self, expected: &[u8]) -> bool {
        self.glyph_run.font.data.as_ref() == expected
    }

    pub(crate) const fn transform_components_for_test(&self) -> [f32; 6] {
        let transform = self.glyph_run.transform;
        [
            transform.matrix[0],
            transform.matrix[1],
            transform.matrix[2],
            transform.matrix[3],
            transform.translation[0],
            transform.translation[1],
        ]
    }

    pub(crate) const fn has_glyph_transform_for_test(&self) -> bool {
        self.glyph_run.glyph_transform.is_some()
    }

    pub(crate) const fn has_brush_transform_for_test(&self) -> bool {
        self.glyph_run.brush_transform.is_some()
    }

    pub(crate) const fn font_size_for_test(&self) -> f32 {
        self.glyph_run.font_size
    }

    pub(crate) const fn embolden_amount_for_test(&self) -> [f64; 2] {
        [
            self.glyph_run.font_embolden.amount.xx,
            self.glyph_run.font_embolden.amount.yy,
        ]
    }

    pub(crate) const fn uses_hinting_for_test(&self) -> bool {
        self.glyph_run.hint
    }

    pub(crate) fn normalized_coordinate_range_for_test(&self) -> std::ops::Range<usize> {
        self.glyph_run.normalized_coords.clone()
    }

    pub(crate) fn glyph_range_for_test(&self) -> std::ops::Range<usize> {
        self.glyph_run.glyphs.clone()
    }

    pub(crate) const fn uses_nonzero_fill_for_test(&self) -> bool {
        matches!(self.glyph_run.style, peniko::Style::Fill(Fill::NonZero))
    }
}

#[cfg(test)]
pub(crate) struct VelloGlyphObservation<'a> {
    glyph: &'a Glyph,
}

#[cfg(test)]
impl VelloGlyphObservation<'_> {
    pub(crate) const fn id_for_test(&self) -> u32 {
        self.glyph.id
    }

    pub(crate) const fn x_for_test(&self) -> f32 {
        self.glyph.x
    }

    pub(crate) const fn y_for_test(&self) -> f32 {
        self.glyph.y
    }
}

#[cfg(test)]
mod pixelated_tests {
    use super::*;

    #[test]
    fn pixelated_native_lattice_refuses_before_path_materialization() {
        use crate::accounting::{FrameFixture, RendererCpu};
        use crate::{
            Image, ImageRendering, RenderAccountingError, RenderAccountingOptions,
            RenderAttemptStatus, RenderCpuScope, RenderEvent, RenderLimits, RenderPhaseFilter,
            RenderProfile, RenderQuantity, RenderRefusalReason, RenderResource, Size,
        };

        let image = Image::from_rgba(Size::new(2.0, 1.0), vec![255, 0, 0, 255, 0, 0, 255, 255])
            .unwrap()
            .rendering(ImageRendering::Pixelated);
        let brush = crate::encode::image_brush(&image);
        let transform = kurbo::Affine::scale_non_uniform(2.5, 1.0);
        let bounds = kurbo::Rect::new(0.0, 0.0, 2.0, 1.0);
        let phase = RenderPhase::Encode;
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::CoordinateVisits, 0)
            .unwrap()
            .with_work_limit(RenderWork::PathElementVisits, 0)
            .unwrap();
        let options = RenderAccountingOptions::new()
            .with_limits(limits)
            .with_profile(RenderProfile::Detailed {
                timing: false,
                event_limit: 32,
                filter: RenderPhaseFilter::Phase(phase),
            });
        let cpu = RendererCpu::new(options).unwrap();
        let mut callback_reached = false;
        let (result, started, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let account = attempt.account().at(phase);
                account.phase(phase, || {
                    let mut scene = VelloScene::default();
                    let result = scene.with_pixelated_image(
                        &image,
                        transform,
                        |scene, compensation| {
                            callback_reached = true;
                            scene.fill(
                                Fill::NonZero,
                                transform,
                                &brush,
                                Some(compensation),
                                &bounds,
                                &account,
                            )
                        },
                        &account,
                    );
                    assert!(scene.encoding.resources.patches.is_empty());
                    assert!(scene.image_expansions.is_empty());
                    result
                })
            })
            .unwrap()
            .into_parts();
        assert!(started);
        assert_eq!(fault, None);
        let attempt = report.attempt().unwrap();
        assert_eq!(attempt.status(), RenderAttemptStatus::Refused);
        let refusal = attempt.refusal().unwrap();
        assert_eq!(
            result.unwrap_err().accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        assert_eq!(refusal.phase(), RenderPhase::Attempt);
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
        assert!(attempt.events().any(|event| matches!(
            event,
            RenderEvent::Refused {
                phase: RenderPhase::Encode,
                ..
            }
        )));
        assert_eq!(
            (refusal.quantity(), refusal.current(), refusal.amount()),
            (RenderQuantity::Work(RenderWork::CoordinateVisits), 0, 1),
            "the first lattice coordinate must be admitted before the fill callback"
        );
        assert!(!callback_reached);
        let encode = attempt.phase_usage(phase);
        for work in [
            RenderWork::CoordinateVisits,
            RenderWork::PathElementVisits,
            RenderWork::PaintVisits,
            RenderWork::ImageSourceBytes,
            RenderWork::AtlasTexels,
        ] {
            assert_eq!(encode.work(work), 0, "{work:?}");
        }
        let cpu = report.renderer_cpu().unwrap();
        let image_payloads = cpu.scope_usage(RenderCpuScope::ImagePayloads);
        let retained = RenderResource::RetainedCpuBytes;
        assert!(image_payloads.peak(retained) > image_payloads.opening_live(retained));
        for scope in [RenderCpuScope::ImagePayloads, RenderCpuScope::FrameBacking] {
            let usage = cpu.scope_usage(scope);
            for resource in [retained, RenderResource::ScratchCpuBytes] {
                assert_eq!(
                    usage.live(resource),
                    usage.opening_live(resource),
                    "{scope:?} {resource:?}"
                );
            }
        }
    }

    #[test]
    fn original_font_glyph_diagnostic_precedes_sweep_field_capacity() {
        let font = crate::FontData::try_from_bytes(
            include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let gradient = crate::Gradient::try_sweep(
            crate::Point::new(f64::MAX, f64::MAX),
            vec![
                crate::GradientStop::try_new(0.0, crate::Color::BLACK).unwrap(),
                crate::GradientStop::try_new(1.0, crate::Color::TRANSPARENT).unwrap(),
            ],
        )
        .unwrap();
        let glyphs = [crate::TextGlyph::try_new(u32::MAX, 0.0, 16.0, 16.0).unwrap()];
        let run = crate::TextRun::try_new(
            crate::FontRef::new(2123).with_data(font),
            16.0,
            crate::Transform::identity(),
            crate::TextPaint::try_fill(gradient.into()).unwrap(),
            &glyphs,
            crate::TextRunBounds::unspecified(),
        )
        .unwrap();
        let error = VelloScene::default()
            .encode_text_run(&run, &Account::for_test())
            .unwrap_err();
        assert_eq!(error.code(), crate::ErrorCode::InvalidInput);
        assert_eq!(
            error.invalid_value_diagnostic().unwrap().field(),
            "text_glyph.id"
        );
        assert!(error.gradient_realization_diagnostic().is_none());
    }

    #[test]
    fn font_backing_miss_counts_source_clone_blob_clone_and_registry_write() {
        let bytes = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let resource =
            FontResource::decode(bytes.to_vec(), surgeist_test::fixtures::FONT_LIMITS).unwrap();
        let account = Account::for_test();
        let mut scene = VelloScene::default();
        let first = scene.font_backing(&resource, &account).unwrap();
        assert_eq!(first.data(), resource.bytes());
        // One retained source-owner clone, one Blob clone, and one new registry
        // record copy. The first empty registry has no elements to relocate.
        assert_eq!(
            account
                .work_count(RenderWork::RasterResourceCopies)
                .unwrap(),
            3
        );
        let second = scene.font_backing(&resource, &account).unwrap();
        assert_eq!(second.id(), first.id());
        // A hit clones only the existing Blob; it does not insert another record.
        assert_eq!(
            account
                .work_count(RenderWork::RasterResourceCopies)
                .unwrap(),
            4
        );
        drop((first, second));
        drop(scene);
    }

    #[test]
    fn empty_pixelated_brush_keeps_transparent_draw_without_expansion() {
        let account = Account::for_test();
        let image = crate::Image::from_rgba(crate::Size::new(0.0, 2.0), Vec::<u8>::new()).unwrap();
        let brush = crate::encode::image_brush(&image);
        let mut scene = VelloScene::default();
        scene.import_image(&image, &account).unwrap();
        scene
            .with_pixelated_image(
                &image,
                kurbo::Affine::scale(4.0),
                |scene, compensation| {
                    assert_eq!(compensation, kurbo::Affine::IDENTITY);
                    scene
                        .fill(
                            peniko::Fill::NonZero,
                            kurbo::Affine::IDENTITY,
                            &brush,
                            None,
                            &kurbo::Rect::new(0.0, 0.0, 4.0, 4.0),
                            &account,
                        )
                        .unwrap();
                    Ok(())
                },
                &account,
            )
            .unwrap();
        assert!(scene.image_expansions.is_empty());
        let plan = super::super::image_atlas::ImageAtlasPlan::prepare(
            &scene.encoding,
            &scene.image_expansions,
            &account,
        )
        .unwrap();
        assert!(plan.entries.is_empty());
        assert_eq!(plan.encoding.draw_data[1], 0);
    }

    #[test]
    fn images_gradients_and_pixelated_glyph_paint_keep_their_resolved_draw_data() {
        let account = Account::for_test();
        let image = crate::Image::from_rgba(
            crate::Size::new(2.0, 1.0),
            vec![255, 0, 0, 255, 0, 0, 255, 255],
        )
        .unwrap();
        let brush = crate::encode::image_brush(&image);
        let mut scene = VelloScene::default();
        scene.import_image(&image, &account).unwrap();
        scene
            .draw_image(&brush, kurbo::Affine::IDENTITY, &account)
            .unwrap();
        let gradient = peniko::Gradient::new_linear((0.0, 0.0), (16.0, 0.0))
            .with_stops([peniko::Color::BLACK, peniko::Color::WHITE]);
        scene
            .fill(
                Fill::NonZero,
                kurbo::Affine::IDENTITY,
                SceneBrush::new((&gradient).into(), None, crate::PhysicalSize::new(32, 16)),
                None,
                &kurbo::Rect::new(0.0, 0.0, 16.0, 4.0),
                &account,
            )
            .unwrap();
        let font = crate::FontData::try_from_bytes(
            include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let glyphs = [crate::TextGlyph::try_new(58, 0.0, 8.0, 8.0).unwrap()];
        let run = crate::TextRun::try_new(
            crate::FontRef::new(1_u64).with_data(font),
            8.0,
            crate::Transform::scale(2.2, 3.7).unwrap(),
            crate::TextPaint::try_fill(crate::Paint::image(
                image.clone().rendering(crate::ImageRendering::Pixelated),
            ))
            .unwrap(),
            &glyphs,
            crate::TextRunBounds::unspecified(),
        )
        .unwrap();
        scene.encode_text_run(&run, &account).unwrap();
        scene
            .draw_image(&brush, kurbo::Affine::IDENTITY, &account)
            .unwrap();
        let glyph_brush = scene.encoding.resources.glyph_runs[0]
            .brush_transform
            .unwrap();
        assert_eq!(glyph_brush.matrix, [0.5, 0.0, 0.0, 0.25]);
        let plan = super::super::image_atlas::ImageAtlasPlan::prepare(
            &scene.encoding,
            &scene.image_expansions,
            &account,
        )
        .unwrap();
        let mut resolver = vello_encoding::Resolver::default();
        let mut packed = Vec::new();
        let (layout, ramps, _) = resolver.resolve(&plan.encoding, &mut packed);
        assert!(ramps.height > 0);
        let data = layout.draw_data(&packed);
        assert_eq!(data[1], (2 << 16) | 1);
        assert_eq!(&data[4..8], &[0, 0, 16_f32.to_bits(), 0]);
        assert_eq!(data[9], (4 << 16) | 4);
        assert_eq!(data[12], (2 << 16) | 1);
        assert_eq!(data[0], data[11]);
        assert_ne!(data[0], data[8]);
        for index in [2, 10, 13] {
            assert_eq!(data[index], scene.encoding.draw_data[index]);
        }
        let prepared = scene
            .prepare_raster(
                RasterParameters::try_new(
                    crate::PhysicalSize::new(32, 32),
                    peniko::Color::TRANSPARENT,
                    crate::Antialiasing::Area,
                )
                .unwrap(),
                &account,
            )
            .unwrap();
        assert!(
            prepared
                .observation_for_test()
                .is_self_consistent_for_test()
        );
    }

    #[test]
    fn pixelated_callback_marks_only_new_draws_and_preserves_original_source() {
        let account = Account::for_test();
        let image = crate::Image::from_rgba(
            crate::Size::new(2.0, 1.0),
            vec![255, 0, 0, 255, 0, 0, 255, 255],
        )
        .unwrap();
        let brush = crate::encode::image_brush(&image);
        let mut scene = VelloScene::default();
        scene.import_image(&image, &account).unwrap();
        scene
            .draw_image(&brush, kurbo::Affine::IDENTITY, &account)
            .unwrap();
        scene
            .with_pixelated_image(
                &image,
                kurbo::Affine::scale_non_uniform(2.2, 3.7),
                |scene, compensation| {
                    assert_eq!(compensation.as_coeffs(), [0.5, 0.0, 0.0, 0.25, 0.0, 0.0]);
                    scene
                        .fill(
                            peniko::Fill::NonZero,
                            kurbo::Affine::IDENTITY,
                            &brush,
                            Some(compensation),
                            &kurbo::Rect::new(0.0, 0.0, 2.0, 1.0),
                            &account,
                        )
                        .unwrap();
                    Ok(())
                },
                &account,
            )
            .unwrap();
        assert!(scene.image_expansions.get(&0, &account).unwrap().is_none());
        assert!(scene.image_expansions.get(&3, &account).unwrap().is_some());
        for patch in &scene.encoding.resources.patches {
            if let Patch::Image { image: source, .. } = patch {
                assert_eq!((source.width, source.height), (2, 1));
                assert_eq!(source.data.data(), image.data.data.data());
            }
        }
        let plan = super::super::image_atlas::ImageAtlasPlan::prepare(
            &scene.encoding,
            &scene.image_expansions,
            &account,
        )
        .unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].extent(), crate::PhysicalSize::new(2, 1));
        assert_eq!(plan.entries[1].extent(), crate::PhysicalSize::new(4, 4));
    }

    #[test]
    fn empty_native_space_skips_unused_positions_and_visible_failure_keeps_source_index() {
        use skrifa::MetadataProvider;
        let bytes = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let provider = skrifa::FontRef::from_index(bytes, 0).unwrap();
        let visible = provider.charmap().map('A').unwrap().to_u32();
        let space = provider.charmap().map(' ').unwrap().to_u32();
        let font = crate::FontData::try_from_bytes(
            bytes.to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let glyphs = [
            crate::TextGlyph::try_new(space, f64::MAX, f64::MAX, 16.0).unwrap(),
            crate::TextGlyph::try_new(visible, f64::MIN_POSITIVE, 16.0, 16.0).unwrap(),
        ];
        fn make_run<'a>(
            font: &crate::FontData,
            glyphs: &'a [crate::TextGlyph],
        ) -> crate::TextRun<'a> {
            crate::TextRun::try_new(
                crate::FontRef::new(1_u64).with_data(font.clone()),
                16.0,
                crate::Transform::identity(),
                crate::TextPaint::try_fill(crate::Paint::color(crate::Color::BLACK)).unwrap(),
                glyphs,
                crate::TextRunBounds::unspecified(),
            )
            .unwrap()
        }
        let mut context = crate::RenderAccountingContext::default();
        let invalid = make_run(&font, &glyphs[..]);
        let error = context
            .run(
                crate::accounting::RenderAttemptKind::Frame,
                RenderPhase::Encode,
                |account| {
                    let mut scene = VelloScene::default();
                    scene.encode_text_run(&invalid, account)
                },
            )
            .unwrap_err();
        assert_eq!(
            error.geometry_realization_diagnostic(),
            Some(&crate::GeometryRealizationError::UnrepresentableGlyph { glyph_index: 1 })
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(crate::RenderResource::ScratchCpuBytes),
            0
        );
        let empty = make_run(&font, &glyphs[..1]);
        context
            .run(
                crate::accounting::RenderAttemptKind::Frame,
                RenderPhase::Encode,
                |account| {
                    let mut scene = VelloScene::default();
                    scene.encode_text_run(&empty, account)?;
                    assert!(scene.encoding.resources.glyphs.is_empty());
                    assert!(scene.encoding.resources.glyph_runs.is_empty());
                    assert!(scene.font_backings.is_none());
                    Ok(())
                },
            )
            .unwrap();
    }

    #[test]
    fn actual_native_provider_loss_refuses_the_original_glyph_occurrence() {
        use skrifa::MetadataProvider;
        let bytes = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let provider = skrifa::FontRef::from_index(bytes, 0).unwrap();
        let visible = provider.charmap().map('A').unwrap().to_u32();
        let font = crate::FontData::try_from_bytes(
            bytes.to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let glyphs = [crate::TextGlyph::try_new(visible, 0.0, 0.0, 0.0).unwrap()];
        let run = crate::TextRun::try_new(
            crate::FontRef::new(1_u64).with_data(font),
            2.0_f32.powi(-43),
            crate::Transform::scale(2.0_f64.powi(48), 2.0_f64.powi(48)).unwrap(),
            crate::TextPaint::try_fill(crate::Paint::color(crate::Color::BLACK)).unwrap(),
            &glyphs,
            crate::TextRunBounds::unspecified(),
        )
        .unwrap();
        let mut context = crate::RenderAccountingContext::default();
        let error = context
            .run(
                crate::accounting::RenderAttemptKind::Frame,
                RenderPhase::Encode,
                |account| {
                    let mut scene = VelloScene::default();
                    scene.encode_text_run(&run, account)
                },
            )
            .unwrap_err();
        assert_eq!(
            error.geometry_realization_diagnostic(),
            Some(&crate::GeometryRealizationError::UnrepresentableGlyph { glyph_index: 0 })
        );
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.live(crate::RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(crate::RenderResource::RetainedCpuBytes), 0);
    }

    #[test]
    fn geometry_role_remapping_preserves_capacity_and_accounting_domains() {
        let capacity: Error = crate::GeometryRealizationError::InsufficientNativeCapacity.into();
        assert_eq!(
            glyph_geometry_error(capacity, 3).geometry_realization_diagnostic(),
            Some(&crate::GeometryRealizationError::InsufficientNativeCapacity)
        );
        let path: Error =
            crate::GeometryRealizationError::UnrepresentablePath { element_index: 2 }.into();
        assert_eq!(
            glyph_geometry_error(path, 3).geometry_realization_diagnostic(),
            Some(&crate::GeometryRealizationError::UnrepresentableGlyph { glyph_index: 3 })
        );
        let mut context = crate::RenderAccountingContext::new(
            crate::RenderAccountingOptions::new().with_limits(
                crate::RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 0)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                crate::accounting::RenderAttemptKind::Frame,
                RenderPhase::Encode,
                |account| account.work(RenderWork::CoordinateVisits, 1),
            )
            .unwrap_err();
        let original = error.accounting_diagnostic();
        let error = glyph_geometry_error(error, 3);
        assert_eq!(error.accounting_diagnostic(), original);
        assert!(original.is_some());
        let font = Error::invalid_input_message("intrinsic font control");
        assert!(
            glyph_geometry_error(font, 3)
                .geometry_realization_diagnostic()
                .is_none()
        );
    }

    #[test]
    fn repeated_and_empty_late_glyphs_keep_exact_packed_stream_correspondence() {
        use skrifa::MetadataProvider;
        let account = Account::for_test();
        let bytes = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");
        let provider = skrifa::FontRef::from_index(bytes, 0).unwrap();
        let visible = provider.charmap().map('A').unwrap().to_u32();
        let empty = provider.charmap().map(' ').unwrap().to_u32();
        let font = crate::FontData::try_from_bytes(
            bytes.to_vec(),
            0,
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
        let outlines = provider.outline_glyphs();
        let outline = outlines.get(skrifa::GlyphId::new(empty)).unwrap();
        let mut empty_path = kurbo::BezPath::new();
        struct Pen<'a>(&'a mut kurbo::BezPath);
        impl skrifa::outline::OutlinePen for Pen<'_> {
            fn move_to(&mut self, x: f32, y: f32) {
                self.0.move_to((f64::from(x), f64::from(y)));
            }
            fn line_to(&mut self, x: f32, y: f32) {
                self.0.line_to((f64::from(x), f64::from(y)));
            }
            fn quad_to(&mut self, x0: f32, y0: f32, x: f32, y: f32) {
                self.0
                    .quad_to((f64::from(x0), f64::from(y0)), (f64::from(x), f64::from(y)));
            }
            fn curve_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x: f32, y: f32) {
                self.0.curve_to(
                    (f64::from(x0), f64::from(y0)),
                    (f64::from(x1), f64::from(y1)),
                    (f64::from(x), f64::from(y)),
                );
            }
            fn close(&mut self) {
                self.0.close_path();
            }
        }
        outline
            .draw(
                skrifa::outline::DrawSettings::unhinted(
                    skrifa::instance::Size::new(16.0),
                    skrifa::instance::LocationRef::new(&[]),
                ),
                &mut Pen(&mut empty_path),
            )
            .unwrap();
        assert!(
            empty_path.elements().is_empty(),
            "authentic Ahem space has no outline segments"
        );
        let glyphs = [
            crate::TextGlyph::try_new(visible, 0.0, 16.0, 16.0).unwrap(),
            crate::TextGlyph::try_new(visible, 16.0, 16.0, 16.0).unwrap(),
            crate::TextGlyph::try_new(empty, 32.0, 16.0, 16.0).unwrap(),
        ];
        let mut scene = VelloScene::default();
        for (size, transform) in [
            (16.0, crate::Transform::identity()),
            (32.0, crate::Transform::scale(2.2, 3.7).unwrap()),
        ] {
            let run = crate::TextRun::try_new(
                crate::FontRef::new(1_u64).with_data(font.clone()),
                size,
                transform,
                crate::TextPaint::try_fill(crate::Paint::color(crate::Color::BLACK)).unwrap(),
                &glyphs,
                crate::TextRunBounds::unspecified(),
            )
            .unwrap();
            scene.encode_text_run(&run, &account).unwrap();
        }
        assert_eq!(scene.encoding.resources.glyph_runs.len(), 2);
        assert_eq!(scene.encoding.resources.glyphs.len(), 4);
        assert!(
            scene
                .encoding
                .resources
                .glyphs
                .iter()
                .all(|glyph| glyph.id == visible)
        );
        assert!(scene.encoding.resources.normalized_coords.is_empty());
        let expected = bounded::packed_size(&scene.encoding, &scene.glyph_sizes, &account).unwrap();
        let packed =
            bounded::PackedProducer::new(&scene.encoding, &scene.glyph_sizes, &account).unwrap();
        let capacity = packed.capacity();
        let mut resolver = vello_encoding::Resolver::default();
        let resolved = packed.resolve(&mut resolver).unwrap();
        assert_eq!(resolved.packed_for_test().len(), expected);
        assert_eq!(resolved.packed_for_test().capacity(), capacity);
    }

    #[test]
    fn cff_and_default_variable_outlines_keep_the_existing_late_glyph_capability() {
        use skrifa::raw::TableProvider;
        // Existing test data belongs to surgeist-font and its checked-in
        // fixture provenance/notices. These test-only includes do not copy an
        // implementation or introduce another producer capability gate.
        let fixtures: &[(&[u8], bool)] = &[
            (
                include_bytes!(
                    "../../../surgeist-font/tests/fixtures/source-code-pro/SourceCodePro-Regular.otf"
                ),
                false,
            ),
            (
                include_bytes!(
                    "../../../surgeist-font/tests/fixtures/color-fonts/test_glyphs-glyf_colr_1_variable.ttf"
                ),
                true,
            ),
        ];
        for &(bytes, variable) in fixtures {
            let account = Account::for_test();
            let font = crate::FontData::try_from_bytes(
                bytes.to_vec(),
                0,
                surgeist_test::fixtures::FONT_LIMITS,
            )
            .unwrap();
            let provider = skrifa::FontRef::from_index(font.bytes(), 0).unwrap();
            assert_eq!(provider.fvar().is_ok(), variable);
            let mut selected = None;
            for id in 0..u32::from(provider.maxp().unwrap().num_glyphs()) {
                // The variable fixture also contains COLR programs. Pick an
                // ordinary nonempty outline to exercise the renderer's existing
                // default-coordinate outline path rather than its COLR refusal.
                if super::super::glyph::selected_color_format(
                    &provider,
                    skrifa::GlyphId::new(id),
                    &font,
                )
                .unwrap()
                .is_some()
                {
                    continue;
                }
                let glyphs = [crate::TextGlyph::try_new(id, 0.0, 20.0, 20.0).unwrap()];
                let run = crate::TextRun::try_new(
                    crate::FontRef::new(1_u64).with_data(font.clone()),
                    20.0,
                    crate::Transform::identity(),
                    crate::TextPaint::try_fill(crate::Paint::color(crate::Color::BLACK)).unwrap(),
                    &glyphs,
                    crate::TextRunBounds::unspecified(),
                )
                .unwrap();
                let qualified = preflight_selected_glyphs(&run, &account).unwrap();
                if qualified.stream_sizes().path_data != 0 {
                    selected = Some(id);
                    break;
                }
            }
            let id = selected.expect("fixture must expose a nonempty ordinary outline");
            let glyphs = [
                crate::TextGlyph::try_new(id, 0.0, 20.0, 20.0).unwrap(),
                crate::TextGlyph::try_new(id, 20.0, 20.0, 20.0).unwrap(),
            ];
            let run = crate::TextRun::try_new(
                crate::FontRef::new(1_u64).with_data(font),
                20.0,
                crate::Transform::scale(1.75, 0.6).unwrap(),
                crate::TextPaint::try_fill(crate::Paint::color(crate::Color::BLACK)).unwrap(),
                &glyphs,
                crate::TextRunBounds::unspecified(),
            )
            .unwrap();
            let mut scene = VelloScene::default();
            scene.encode_text_run(&run, &account).unwrap();
            assert!(scene.encoding.resources.normalized_coords.is_empty());
            assert!(!scene.encoding.resources.glyph_runs[0].hint);
            let expected =
                bounded::packed_size(&scene.encoding, &scene.glyph_sizes, &account).unwrap();
            let packed =
                bounded::PackedProducer::new(&scene.encoding, &scene.glyph_sizes, &account)
                    .unwrap();
            let capacity = packed.capacity();
            let mut resolver = vello_encoding::Resolver::default();
            let resolved = packed.resolve(&mut resolver).unwrap();
            assert_eq!(resolved.packed_for_test().len(), expected);
            assert_eq!(resolved.packed_for_test().capacity(), capacity);
        }
    }
}
