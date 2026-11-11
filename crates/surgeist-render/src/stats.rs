use crate::accounting::{RenderFrameReport, RenderPhase, RenderWork};
use crate::{RenderAccountingError, Result};
use std::time::Duration;

/// GPU execution route used by one successfully published frame.
///
/// [`Self::DirectVello`] is the single-pass effect-free route;
/// [`Self::GpuGraph`] owns the closed supported graph subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderRoute {
    /// One transaction-owned internal Vello raster pass.
    DirectVello,
    /// The closed crate-owned WGPU image/composite graph.
    GpuGraph,
}

/// Working texture precision selected for a successfully published GPU graph frame.
///
/// High maps to `Rgba16Float`; Reduced maps to `Rgba8Unorm`. DirectVello
/// frames do not select an effect precision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectPrecision {
    /// High-precision `Rgba16Float` working textures.
    High,
    /// Explicit reduced-precision `Rgba8Unorm` working textures.
    Reduced,
}

/// Telemetry for the renderer's last successful complete frame publication.
///
/// Failed or canceled attempts do not replace this value. Counts describe work
/// owned by that frame, durations use [`Duration`], and resource fields measure
/// bytes. [`Default`] represents no successful publication yet.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    /// Execution route, or `None` before the first successful frame.
    pub route: Option<RenderRoute>,
    /// Graph working precision, or `None` for DirectVello/no successful frame.
    pub effect_precision: Option<EffectPrecision>,
    /// Number of internal Vello raster passes encoded for the frame.
    pub vello_passes: u64,
    /// Number of crate-owned GPU image-processing passes encoded for the frame.
    pub image_passes: u64,
    /// Number of crate-owned GPU composition passes encoded for the frame.
    pub composite_passes: u64,
    /// Number of GPU texture-copy operations encoded for the frame.
    pub texture_copy_operations: u64,
    /// Number of custom graph-to-surface presentation passes.
    pub custom_present_passes: u64,
    /// Observed descriptor-bearing managed native allocations during the frame, including provisional handles.
    pub cache_allocations: u64,
    /// Compatible managed cache allocations reused during the frame.
    pub cache_reuses: u64,
    /// Idle managed cache byte classification after successful cleanup; not additional physical storage.
    pub idle_cache_bytes: u64,
    /// Wall-clock duration of the complete successful frame operation.
    pub frame_time: Option<Duration>,
    /// Duration spent validating, normalizing, and encoding frame inputs.
    pub preparation_time: Option<Duration>,
    /// Inclusive host-observed execution and waits, including acquisition/presentation.
    /// Do not add its child presentation/acquisition durations to this value.
    pub execution_time: Option<Duration>,
    /// Duration of actual presentation preparation and host presentation work.
    pub presentation_time: Option<Duration>,
    /// Number of normalized render command records emitted, including nested records.
    pub commands: u64,
    /// Number of normalized fill commands.
    pub fills: u64,
    /// Number of normalized stroke commands.
    pub strokes: u64,
    /// Number of normalized shadow instances.
    pub shadows: u64,
    /// Number of normalized image draws.
    pub images: u64,
    /// Number of normalized glyph instances.
    pub glyphs: u64,
    /// Number of normalized layer commands.
    pub layers: u64,
    /// Number of exact image contents already observed by this renderer.
    pub image_content_hits: u64,
    /// Number of first-observed exact image contents for this renderer.
    pub image_content_misses: u64,
    /// Straight-alpha source-image bytes first observed by this renderer.
    pub first_observed_image_bytes: u64,
    /// Actual source bytes passed to queue buffer/texture uploads during this attempt.
    pub queue_upload_bytes: u64,
    /// Host-observed duration of actual surface acquisition calls, absent when untimed.
    pub surface_acquisition_time: Option<Duration>,
}

impl Stats {
    pub(crate) fn from_frame_report(
        report: &RenderFrameReport,
        route: RenderRoute,
        precision: Option<EffectPrecision>,
    ) -> Result<Self> {
        let attempt = report.attempt()?;
        let work = attempt.usage();
        let generation = report
            .generation()
            .ok_or(RenderAccountingError::Invariant)??;
        Ok(Self {
            route: Some(route),
            effect_precision: precision,
            vello_passes: work.work(RenderWork::VelloPasses),
            image_passes: work.work(RenderWork::ImagePasses),
            composite_passes: work.work(RenderWork::CompositePasses),
            texture_copy_operations: work.work(RenderWork::CopyOperations),
            custom_present_passes: work.work(RenderWork::CustomPresentPasses),
            cache_allocations: generation.cache_allocations(),
            cache_reuses: generation.cache_reuses(),
            idle_cache_bytes: generation.idle_cache_bytes(),
            frame_time: attempt.elapsed(),
            preparation_time: attempt.phase_elapsed(RenderPhase::FramePrepare),
            execution_time: attempt.phase_elapsed(RenderPhase::Render),
            presentation_time: attempt.phase_elapsed(RenderPhase::Present),
            surface_acquisition_time: attempt.phase_elapsed(RenderPhase::Acquire),
            commands: work.work(RenderWork::CommandsEmitted),
            fills: work.work(RenderWork::FillsEmitted),
            strokes: work.work(RenderWork::StrokesEmitted),
            shadows: work.work(RenderWork::ShadowsEmitted),
            images: work.work(RenderWork::ImagesEmitted),
            glyphs: work.work(RenderWork::GlyphsEmitted),
            layers: work.work(RenderWork::LayersEmitted),
            image_content_hits: work.work(RenderWork::ImageContentHits),
            image_content_misses: work.work(RenderWork::ImageContentMisses),
            first_observed_image_bytes: work.work(RenderWork::ImageFirstObservedBytes),
            queue_upload_bytes: work.work(RenderWork::UploadBytes),
        })
    }
}
#[cfg(test)]
pub(crate) use diagnostic::collect_render_stats;
#[cfg(test)]
mod diagnostic {
    use super::*;
    use crate::{
        Image, Paint,
        command::{RenderCommand, RenderPaint},
        image::ImageContentIdentity,
        paint::PaintKind,
    };
    fn collect_paint_stats(
        paint: &Paint,
        stats: &mut Stats,
        uploaded_images: &mut std::collections::HashSet<ImageContentIdentity>,
    ) {
        if let PaintKind::Image(image) = paint.kind() {
            collect_image_stats(image, stats, uploaded_images);
        }
    }
    pub(crate) fn collect_render_stats(
        commands: &[RenderCommand],
        stats: &mut Stats,
        uploaded_images: &mut std::collections::HashSet<ImageContentIdentity>,
    ) {
        for command in commands {
            stats.commands = stats.commands.saturating_add(1);
            match command {
                RenderCommand::Fill { paint, .. } => {
                    stats.fills = stats.fills.saturating_add(1);
                    collect_render_paint_stats(paint, stats, uploaded_images);
                }
                RenderCommand::Stroke { paint, .. } => {
                    stats.strokes = stats.strokes.saturating_add(1);
                    collect_render_paint_stats(paint, stats, uploaded_images);
                }
                RenderCommand::Shadow { .. } => {
                    stats.shadows = stats.shadows.saturating_add(1);
                }
                RenderCommand::Image { image, .. } | RenderCommand::ImageRegion { image, .. } => {
                    collect_image_stats(image, stats, uploaded_images);
                }
                RenderCommand::PreparedGlyphRun {
                    run: outline,
                    paint,
                    ..
                } => {
                    stats.glyphs = stats.glyphs.saturating_add(outline.glyphs().len() as u64);
                    collect_paint_stats(paint.fill(), stats, uploaded_images);
                    for operation in outline.paints() {
                        if let crate::text::GlyphPaint::Bitmap { image, .. } = operation {
                            collect_image_stats(image, stats, uploaded_images);
                        }
                    }
                }
                RenderCommand::TextRun { glyphs, .. } => {
                    stats.glyphs = stats.glyphs.saturating_add(glyphs.len() as u64);
                }
                RenderCommand::Layer { children, .. } => {
                    stats.layers = stats.layers.saturating_add(1);
                    collect_render_stats(children, stats, uploaded_images);
                }
            }
        }
    }

    fn collect_render_paint_stats(
        paint: &RenderPaint,
        stats: &mut Stats,
        uploaded_images: &mut std::collections::HashSet<ImageContentIdentity>,
    ) {
        if let RenderPaint::Image(image) = paint {
            collect_image_stats(image, stats, uploaded_images);
        }
    }

    fn collect_image_stats(
        image: &Image,
        stats: &mut Stats,
        uploaded_images: &mut std::collections::HashSet<ImageContentIdentity>,
    ) {
        stats.images = stats.images.saturating_add(1);
        if uploaded_images.insert(image.content_identity().clone()) {
            stats.image_content_misses = stats.image_content_misses.saturating_add(1);
            stats.first_observed_image_bytes = stats
                .first_observed_image_bytes
                .saturating_add(u64::try_from(image.bytes.len()).unwrap_or(u64::MAX));
        } else {
            stats.image_content_hits = stats.image_content_hits.saturating_add(1);
        }
    }
}
