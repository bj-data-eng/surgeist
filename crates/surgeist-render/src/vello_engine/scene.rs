// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Modified for surgeist-render; see NOTICE-VELLO.md for provenance and adaptations.

use std::{collections::HashMap, sync::Arc};

use peniko::{Blob, BrushRef, Fill, ImageBrushRef};
use surgeist_font::{FontResource, ResourceId};
use vello_encoding::{
    DrawBeginClip, Encoding, Glyph, GlyphRun, Patch, Transform as VelloTransform,
};

#[cfg(test)]
use vello_encoding::{DrawTag, PathTag, Style};

use crate::{BackendErrorCode, Error, Result, TextRun, encode::glyph_paint_brush};

use super::glyph::{
    BitmapEncoding, SelectedGlyphRepresentation, ValidatedGlyphRun, preflight_selected_glyphs,
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
    image_expansions: HashMap<usize, super::image_atlas::ImageExpansion>,
    font_backings: HashMap<ResourceId, Blob<u8>>,
}

struct DecodedFontBytes(FontResource);

impl AsRef<[u8]> for DecodedFontBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.bytes()
    }
}

impl VelloScene {
    /// Associates integer expansion with only the matching image draws appended
    /// by this callback. Geometry stays in its original coordinates; the callback
    /// applies the supplied compensation to the image brush transform.
    pub(crate) fn with_pixelated_image<T>(
        &mut self,
        image: &crate::Image,
        source_to_device: kurbo::Affine,
        encode: impl FnOnce(&mut Self, kurbo::Affine) -> Result<T>,
    ) -> Result<T> {
        if image.data.width == 0 || image.data.height == 0 {
            return encode(self, kurbo::Affine::IDENTITY);
        }
        let expansion = super::image_atlas::ImageExpansion::try_new(&image.data, source_to_device)?;
        let first_patch = self.encoding.resources.patches.len();
        let result = encode(self, expansion.compensation())?;
        self.mark_image_expansion(first_patch, image.data.data.id(), expansion);
        Ok(result)
    }

    fn mark_image_expansion(
        &mut self,
        first_patch: usize,
        source_id: u64,
        expansion: super::image_atlas::ImageExpansion,
    ) {
        for patch in &self.encoding.resources.patches[first_patch..] {
            if let Patch::Image {
                image,
                draw_data_offset,
            } = patch
                && image.data.id() == source_id
            {
                self.image_expansions.insert(*draw_data_offset, expansion);
            }
        }
    }

    pub(crate) fn fill_elements<'a>(
        &mut self,
        transform: kurbo::Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<kurbo::Affine>,
        elements: &[kurbo::PathEl],
    ) {
        self.encoding
            .encode_transform(VelloTransform::from_kurbo(&transform));
        self.encoding.encode_fill_style(Fill::NonZero);
        if self
            .encoding
            .encode_path_elements(elements.iter().copied(), true)
        {
            if let Some(brush_transform) = brush_transform
                && self
                    .encoding
                    .encode_transform(VelloTransform::from_kurbo(&(transform * brush_transform)))
            {
                self.encoding.swap_last_path_tags();
            }
            self.encoding.encode_brush(brush, 1.0);
        }
    }
    pub(crate) fn fill<'a>(
        &mut self,
        style: Fill,
        transform: kurbo::Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
    ) {
        self.encoding
            .encode_transform(VelloTransform::from_kurbo(&transform));
        self.encoding.encode_fill_style(style);
        if self.encoding.encode_shape(shape, true) {
            if let Some(brush_transform) = brush_transform
                && self
                    .encoding
                    .encode_transform(VelloTransform::from_kurbo(&(transform * brush_transform)))
            {
                self.encoding.swap_last_path_tags();
            }
            self.encoding.encode_brush(brush, 1.0);
        }
    }

    pub(crate) fn stroke<'a>(
        &mut self,
        style: &kurbo::Stroke,
        transform: kurbo::Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<kurbo::Affine>,
        shape: &impl kurbo::Shape,
    ) {
        if style.width == 0.0 {
            return;
        }
        self.encoding
            .encode_transform(VelloTransform::from_kurbo(&transform));
        let encoded_stroke = self.encoding.encode_stroke_style(style);
        debug_assert!(encoded_stroke, "non-zero strokes must encode");
        let encoded_shape = if style.dash_pattern.is_empty() {
            self.encoding.encode_shape(shape, false)
        } else {
            let dashed = kurbo::dash(
                shape.path_elements(0.01),
                style.dash_offset,
                &style.dash_pattern,
            )
            .collect::<Vec<_>>();
            self.encoding
                .encode_path_elements(dashed.into_iter(), false)
        };
        if encoded_shape {
            if let Some(brush_transform) = brush_transform
                && self
                    .encoding
                    .encode_transform(VelloTransform::from_kurbo(&(transform * brush_transform)))
            {
                self.encoding.swap_last_path_tags();
            }
            self.encoding.encode_brush(brush, 1.0);
        }
    }

    pub(crate) fn draw_image<'a>(
        &mut self,
        image: impl Into<ImageBrushRef<'a>>,
        transform: kurbo::Affine,
    ) {
        let image = image.into();
        let rect = kurbo::Rect::new(
            0.0,
            0.0,
            f64::from(image.image.width),
            f64::from(image.image.height),
        );
        self.fill(Fill::NonZero, transform, image, None, &rect);
    }

    pub(crate) fn push_layer(
        &mut self,
        fill: Fill,
        blend: peniko::BlendMode,
        alpha: f32,
        transform: kurbo::Affine,
        clip: &impl kurbo::Shape,
    ) {
        self.push_layer_inner(
            DrawBeginClip::new(blend, alpha.clamp(0.0, 1.0)),
            fill,
            transform,
            clip,
        );
    }

    pub(crate) fn push_clip_layer(
        &mut self,
        fill: Fill,
        transform: kurbo::Affine,
        clip: &impl kurbo::Shape,
    ) {
        self.push_layer_inner(DrawBeginClip::clip(), fill, transform, clip);
    }

    pub(crate) fn pop_layer(&mut self) {
        self.encoding.encode_end_clip();
    }

    pub(crate) fn draw_blurred_rounded_rect(
        &mut self,
        transform: kurbo::Affine,
        rect: kurbo::Rect,
        brush: peniko::Color,
        radius: f64,
        std_dev: f64,
    ) {
        let kernel_size = 2.5 * std_dev;
        self.draw_blurred_rounded_rect_in(
            &rect.inflate(kernel_size, kernel_size),
            transform,
            rect,
            brush,
            radius,
            std_dev,
        );
    }

    pub(crate) fn draw_blurred_rounded_rect_in(
        &mut self,
        shape: &impl kurbo::Shape,
        transform: kurbo::Affine,
        rect: kurbo::Rect,
        brush: peniko::Color,
        radius: f64,
        std_dev: f64,
    ) {
        self.encoding
            .encode_transform(VelloTransform::from_kurbo(&transform));
        self.encoding.encode_fill_style(Fill::NonZero);
        if self.encoding.encode_shape(shape, true) {
            let brush_transform =
                VelloTransform::from_kurbo(&(transform.pre_translate(rect.center().to_vec2())));
            if self.encoding.encode_transform(brush_transform) {
                self.encoding.swap_last_path_tags();
            }
            self.encoding.encode_blurred_rounded_rect(
                brush,
                rect.width() as f32,
                rect.height() as f32,
                radius as f32,
                std_dev as f32,
            );
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "The private text encoding entry point is consumed by scene lowering."
        )
    )]
    pub(crate) fn encode_text_run(&mut self, run: &TextRun<'_>) -> Result<()> {
        let validated = preflight_selected_glyphs(run)?;
        self.append_validated_glyph_run(validated, kurbo::Affine::IDENTITY)
    }

    pub(crate) fn encode_text_run_with_transform(
        &mut self,
        run: &TextRun<'_>,
        transform: kurbo::Affine,
    ) -> Result<()> {
        let validated = preflight_selected_glyphs(run)?;
        self.append_validated_glyph_run(validated, transform)
    }

    pub(crate) fn prepare_raster(&self, parameters: RasterParameters) -> Result<PreparedVelloPass> {
        raster::prepare(&self.encoding, &self.image_expansions, parameters)
    }

    fn append_validated_glyph_run(
        &mut self,
        validated: ValidatedGlyphRun<'_>,
        transform: kurbo::Affine,
    ) -> Result<()> {
        if let Some(representation) = validated
            .representations()
            .iter()
            .find(|representation| !matches!(representation, SelectedGlyphRepresentation::Outline))
        {
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
        let brush = glyph_paint_brush(validated.paint().fill())?;
        let image_expansion = match validated.paint().fill().kind() {
            crate::paint::PaintKind::Image(image)
                if image.rendering_hint() == Some(crate::ImageRendering::Pixelated)
                    && image.data.width != 0
                    && image.data.height != 0 =>
            {
                Some((
                    image.data.data.id(),
                    super::image_atlas::ImageExpansion::try_new(
                        &image.data,
                        transform * kurbo::Affine::from(validated.transform()),
                    )?,
                ))
            }
            _ => None,
        };
        let stream_offsets = self.encoding.stream_offsets();
        let glyph_start = self.encoding.resources.glyphs.len();
        self.encoding
            .resources
            .glyphs
            .extend(validated.glyphs().iter().map(|glyph| Glyph {
                id: glyph.id(),
                // Selected-glyph preflight qualified every coordinate in this
                // immutable borrowed run before any encoding was appended.
                x: glyph.x() as f32,
                y: glyph.y() as f32,
            }));
        let glyph_end = self.encoding.resources.glyphs.len();
        if glyph_start == glyph_end {
            return Ok(());
        }
        // The glyph resolver shares its cache within this encoding. Use one
        // blob identity per backing and keep the original face index on each run.
        let face = validated.font_data().admitted_face();
        let resource = face.resource();
        let backing = self
            .font_backings
            .entry(resource.identity())
            .or_insert_with(|| Blob::new(Arc::new(DecodedFontBytes(resource.clone()))));
        let index = self.encoding.resources.glyph_runs.len();
        self.encoding.resources.glyph_runs.push(GlyphRun {
            font: peniko::FontData::new(backing.clone(), face.index()),
            transform: VelloTransform::from_kurbo(
                &(transform * kurbo::Affine::from(validated.transform())),
            ),
            glyph_transform: None,
            brush_transform: image_expansion
                .map(|(_, expansion)| VelloTransform::from_kurbo(&expansion.compensation())),
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
        self.encoding.encode_brush(&brush, 1.0);
        if let Some((source_id, expansion)) = image_expansion {
            self.mark_image_expansion(first_patch, source_id, expansion);
        }
        self.encoding.force_next_transform_and_style();
        Ok(())
    }

    fn push_layer_inner(
        &mut self,
        parameters: DrawBeginClip,
        fill: Fill,
        transform: kurbo::Affine,
        clip: &impl kurbo::Shape,
    ) {
        self.encoding
            .encode_transform(VelloTransform::from_kurbo(&transform));
        self.encoding.encode_fill_style(fill);
        if !self.encoding.encode_shape(clip, true) {
            self.encoding.encode_empty_shape();
        }
        self.encoding.encode_begin_clip(parameters);
    }

    #[cfg(test)]
    pub(crate) fn prepare_raster_scenario_for_test(
        scenario: VelloRasterScenario,
        parameters: RasterParameters,
    ) -> Result<PreparedVelloPass> {
        let mut scene = Self::default();
        scene.append_raster_scenario_for_test(scenario);
        scene.prepare_raster(parameters)
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
    fn append_raster_scenario_for_test(&mut self, scenario: VelloRasterScenario) {
        match scenario {
            VelloRasterScenario::Base => {}
            VelloRasterScenario::LargePath => self.append_large_path_scenario_for_test(),
            VelloRasterScenario::Clip => self.append_clip_scenario_for_test(),
            VelloRasterScenario::LargePathAndClip => {
                self.append_large_path_scenario_for_test();
                self.append_clip_scenario_for_test();
            }
        }
    }

    #[cfg(test)]
    fn append_large_path_scenario_for_test(&mut self) {
        const LARGE_PATH_SCAN_LINE_SEGMENTS: usize = 4 * 256 * 256 + 1;

        self.encoding.encode_transform(VelloTransform::IDENTITY);
        self.encoding.encode_fill_style(Fill::NonZero);
        let mut path = self.encoding.encode_path(true);
        path.move_to(0.0, 0.0);
        let mut x = 1.0;
        for segment in 0..LARGE_PATH_SCAN_LINE_SEGMENTS {
            path.line_to(x, if segment % 2 == 0 { 1.0 } else { 0.0 });
            x += 1.0;
        }
        path.finish(true);
        self.encoding.encode_color(peniko::Color::BLACK);
    }

    #[cfg(test)]
    fn append_clip_scenario_for_test(&mut self) {
        const CLIP_PAIRS_FOR_REDUCE: usize = 129;

        for _ in 0..CLIP_PAIRS_FOR_REDUCE {
            self.encoding.encode_transform(VelloTransform::IDENTITY);
            self.encoding.encode_fill_style(Fill::NonZero);
            let encoded = self
                .encoding
                .encode_shape(&kurbo::Rect::new(0.0, 0.0, 1.0, 1.0), true);
            debug_assert!(encoded);
            self.encoding
                .encode_begin_clip(vello_encoding::DrawBeginClip::clip());
            self.encoding.encode_end_clip();
        }
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
    fn empty_pixelated_brush_keeps_transparent_draw_without_expansion() {
        let image = crate::Image::from_rgba(crate::Size::new(0.0, 2.0), Vec::<u8>::new()).unwrap();
        let brush = crate::encode::image_brush(&image);
        let mut scene = VelloScene::default();
        scene
            .with_pixelated_image(&image, kurbo::Affine::scale(4.0), |scene, compensation| {
                assert_eq!(compensation, kurbo::Affine::IDENTITY);
                scene.fill(
                    peniko::Fill::NonZero,
                    kurbo::Affine::IDENTITY,
                    &brush,
                    None,
                    &kurbo::Rect::new(0.0, 0.0, 4.0, 4.0),
                );
                Ok(())
            })
            .unwrap();
        assert!(scene.image_expansions.is_empty());
        let plan = super::super::image_atlas::ImageAtlasPlan::prepare(
            &scene.encoding,
            &scene.image_expansions,
        )
        .unwrap();
        assert!(plan.entries.is_empty());
        assert_eq!(plan.encoding.draw_data[1], 0);
    }

    #[test]
    fn images_gradients_and_pixelated_glyph_paint_keep_their_resolved_draw_data() {
        let image = crate::Image::from_rgba(
            crate::Size::new(2.0, 1.0),
            vec![255, 0, 0, 255, 0, 0, 255, 255],
        )
        .unwrap();
        let brush = crate::encode::image_brush(&image);
        let mut scene = VelloScene::default();
        scene.draw_image(&brush, kurbo::Affine::IDENTITY);
        let gradient = peniko::Gradient::new_linear((0.0, 0.0), (16.0, 0.0))
            .with_stops([peniko::Color::BLACK, peniko::Color::WHITE]);
        scene.fill(
            Fill::NonZero,
            kurbo::Affine::IDENTITY,
            &gradient,
            None,
            &kurbo::Rect::new(0.0, 0.0, 16.0, 4.0),
        );
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
        scene.encode_text_run(&run).unwrap();
        scene.draw_image(&brush, kurbo::Affine::IDENTITY);
        let glyph_brush = scene.encoding.resources.glyph_runs[0]
            .brush_transform
            .unwrap();
        assert_eq!(glyph_brush.matrix, [0.5, 0.0, 0.0, 0.25]);
        let plan = super::super::image_atlas::ImageAtlasPlan::prepare(
            &scene.encoding,
            &scene.image_expansions,
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
        let image = crate::Image::from_rgba(
            crate::Size::new(2.0, 1.0),
            vec![255, 0, 0, 255, 0, 0, 255, 255],
        )
        .unwrap();
        let brush = crate::encode::image_brush(&image);
        let mut scene = VelloScene::default();
        scene.draw_image(&brush, kurbo::Affine::IDENTITY);
        scene
            .with_pixelated_image(
                &image,
                kurbo::Affine::scale_non_uniform(2.2, 3.7),
                |scene, compensation| {
                    assert_eq!(compensation.as_coeffs(), [0.5, 0.0, 0.0, 0.25, 0.0, 0.0]);
                    scene.fill(
                        peniko::Fill::NonZero,
                        kurbo::Affine::IDENTITY,
                        &brush,
                        Some(compensation),
                        &kurbo::Rect::new(0.0, 0.0, 2.0, 1.0),
                    );
                    Ok(())
                },
            )
            .unwrap();
        assert!(!scene.image_expansions.contains_key(&0));
        assert!(scene.image_expansions.contains_key(&3));
        for patch in &scene.encoding.resources.patches {
            if let Patch::Image { image: source, .. } = patch {
                assert_eq!((source.width, source.height), (2, 1));
                assert_eq!(source.data.data(), image.data.data.data());
            }
        }
        let plan = super::super::image_atlas::ImageAtlasPlan::prepare(
            &scene.encoding,
            &scene.image_expansions,
        )
        .unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].extent(), crate::PhysicalSize::new(2, 1));
        assert_eq!(plan.entries[1].extent(), crate::PhysicalSize::new(4, 4));
    }
}
