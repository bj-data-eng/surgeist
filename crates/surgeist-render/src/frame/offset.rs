//! Qualified inverse translation on the actual layer-local capture lattices.
use super::{
    FrameContext,
    bounds::{FrameSpatialPlan, LogicalBounds, NonEmptyFrameSpatialPlan},
    convolution::intersection,
};
use crate::{
    ImageRendering, PreparedOffset, Rect, Transform,
    accounting::{Account, RenderWork},
    error::{Error, Result},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OffsetStage {
    RebaseSource,
    Sample,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OffsetSampling {
    Copy,
    Nearest,
    Bilinear,
    Cubic,
}

impl OffsetSampling {
    pub(crate) const fn footprint(self) -> u64 {
        match self {
            Self::Copy | Self::Nearest => 1,
            Self::Bilinear => 4,
            Self::Cubic => 16,
        }
    }
    pub(crate) const fn shader_value(self) -> u32 {
        match self {
            Self::Copy => 0,
            Self::Nearest => 1,
            Self::Bilinear => 2,
            Self::Cubic => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OffsetMapping {
    source_domain: Rect,
    result_domain: Rect,
    source_origin: [f64; 2],
    result_origin: [f64; 2],
    source_scale: f64,
    result_scale: f64,
    translation: [f64; 2],
    ratio: [f64; 2],
    source_clip: [f64; 4],
    result_clip: [f64; 4],
    source_extent: [u32; 2],
    result_extent: [u32; 2],
    sampling: OffsetSampling,
}

impl OffsetMapping {
    pub(crate) const fn source_domain(self) -> Rect {
        self.source_domain
    }
    pub(crate) const fn result_domain(self) -> Rect {
        self.result_domain
    }
    pub(crate) const fn source_origin(self) -> [f64; 2] {
        self.source_origin
    }
    pub(crate) const fn result_origin(self) -> [f64; 2] {
        self.result_origin
    }
    pub(crate) const fn source_scale(self) -> f64 {
        self.source_scale
    }
    pub(crate) const fn result_scale(self) -> f64 {
        self.result_scale
    }
    pub(crate) const fn translation(self) -> [f64; 2] {
        self.translation
    }
    pub(crate) const fn ratio(self) -> [f64; 2] {
        self.ratio
    }
    pub(crate) const fn source_clip(self) -> [f64; 4] {
        self.source_clip
    }
    pub(crate) const fn result_clip(self) -> [f64; 4] {
        self.result_clip
    }
    pub(crate) const fn source_extent(self) -> [u32; 2] {
        self.source_extent
    }
    pub(crate) const fn result_extent(self) -> [u32; 2] {
        self.result_extent
    }
    pub(crate) const fn sampling(self) -> OffsetSampling {
        self.sampling
    }

    /// None proves that every possible reconstruction tap is transparent.
    pub(super) fn try_new(
        operation: &PreparedOffset,
        source: NonEmptyFrameSpatialPlan,
        result: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Option<Self>> {
        Self::try_from_parts(operation, source, result, OffsetStage::Sample, account)
    }

    pub(super) fn try_rebase(
        operation: &PreparedOffset,
        source: NonEmptyFrameSpatialPlan,
        result: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        Self::try_from_parts(
            operation,
            source,
            result,
            OffsetStage::RebaseSource,
            account,
        )?
        .ok_or_else(|| mapping_error(0.0))
    }

    fn try_from_parts(
        operation: &PreparedOffset,
        source: NonEmptyFrameSpatialPlan,
        result: NonEmptyFrameSpatialPlan,
        stage: OffsetStage,
        account: &Account,
    ) -> Result<Option<Self>> {
        let domains = operation.domains();
        let source_domain = intersection(domains.source(), domains.filter());
        let result_domain = if stage == OffsetStage::RebaseSource {
            source_domain
        } else {
            intersection(domains.result(), domains.filter())
        };
        let source_origin = [
            source.texel_center_mapping.origin.x(),
            source.texel_center_mapping.origin.y(),
        ];
        let result_origin = [
            result.texel_center_mapping.origin.x(),
            result.texel_center_mapping.origin.y(),
        ];
        let source_scale = source.raster_scale.get();
        let result_scale = result.raster_scale.get();
        let source_extent = [source.device_extent.width, source.device_extent.height];
        let result_extent = [result.device_extent.width, result.device_extent.height];
        let displacement = if stage == OffsetStage::RebaseSource {
            [0.0; 2]
        } else {
            [operation.dx(), operation.dy()]
        };
        let ratio = [source_scale / result_scale; 2];
        let source_device_origin = [source.device_origin.x, source.device_origin.y];
        let result_device_origin = [result.device_origin.x, result.device_origin.y];
        let mut translation = [0.0; 2];
        let mut aligned = source_scale == result_scale;
        for axis in 0..2 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            // The capture owner constructs c = device_origin / rho. Shared
            // density therefore gives an exact integer lattice correspondence;
            // subtracting the rounded quotients would invent a fractional
            // phase (for example rho=1.5, device origins -4 and -3).
            let shared_device_lattice = source_scale == result_scale
                && source_origin[axis] == f64::from(source_device_origin[axis]) / source_scale
                && result_origin[axis] == f64::from(result_device_origin[axis]) / result_scale;
            let lattice_delta = if shared_device_lattice {
                (i64::from(result_device_origin[axis]) - i64::from(source_device_origin[axis]))
                    as f64
            } else {
                (result_origin[axis] - source_origin[axis]) * source_scale
            };
            let displacement_texels = displacement[axis] * source_scale;
            // Keep the half-cell phase separate so a small translation on
            // equal-density grids is not lost to adding/subtracting one half.
            translation[axis] = if shared_device_lattice {
                lattice_delta - displacement_texels
            } else {
                ((result_origin[axis] - source_origin[axis]) - displacement[axis]) * source_scale
                    + (0.5 * ratio[axis] - 0.5)
            };
            aligned &= shared_device_lattice
                && lattice_delta.is_finite()
                && lattice_delta.fract() == 0.0
                && displacement_texels.is_finite()
                && displacement_texels.fract() == 0.0
                && translation[axis].is_finite()
                && translation[axis].fract() == 0.0;
        }
        // This comparison precedes f32 narrowing and all sampling indices.
        // Two cells cover the largest adopted kernel's finite support.
        if stage == OffsetStage::Sample
            && (0..2).any(|axis| {
                let first = translation[axis];
                let last = first + f64::from(result_extent[axis] - 1) * ratio[axis];
                first > f64::from(source_extent[axis]) + 2.0
                    || last < -2.0
                    || (first == f64::INFINITY && last == f64::INFINITY)
                    || (first == f64::NEG_INFINITY && last == f64::NEG_INFINITY)
            })
        {
            return Ok(None);
        }
        if stage == OffsetStage::RebaseSource && !aligned {
            return Err(mapping_error(translation[0]));
        }
        let sampling = if aligned {
            OffsetSampling::Copy
        } else {
            match operation.rendering() {
                ImageRendering::CrispEdges => OffsetSampling::Nearest,
                ImageRendering::HighQuality => OffsetSampling::Cubic,
                ImageRendering::Auto | ImageRendering::Smooth | ImageRendering::Pixelated => {
                    OffsetSampling::Bilinear
                }
            }
        };
        for scale in [source_scale, result_scale] {
            account.work(RenderWork::CoordinateVisits, 1)?;
            if !(scale as f32).is_normal() || scale <= 0.0 || !((1.0 / scale) as f32).is_normal() {
                return Err(mapping_error(scale));
            }
        }
        let source_clip = rebased_clip(
            source_domain,
            source_origin,
            source_scale,
            source_extent,
            account,
        )?;
        let result_clip = rebased_clip(
            result_domain,
            result_origin,
            result_scale,
            result_extent,
            account,
        )?;
        for axis in 0..2 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            // Position's half-cell, cell coverage and neighboring integer
            // coordinates must remain distinguishable in the uploaded format.
            for extent in [source_extent[axis], result_extent[axis]] {
                if extent > 8_388_608 {
                    return Err(index_error(f64::from(extent)));
                }
            }
            let t = uploaded(translation[axis])?;
            let r = uploaded(ratio[axis])?;
            if !r.is_normal() || r <= 0.0 {
                return Err(mapping_error(ratio[axis]));
            }
            let last_index = (result_extent[axis] - 1) as f32;
            let product = finite(last_index * r)?;
            let last = finite(t + product)?;
            // Endpoint monotonicity bounds every intermediate multiplication
            // and addition. The shader guards exterior coordinates before
            // floor; inside that guard the full cubic neighborhood fits i32.
            for coordinate in [t.max(-2.0), last.min(source_extent[axis] as f32 + 1.0)] {
                qualify_index(coordinate, sampling)?;
            }
            if sampling == OffsetSampling::Copy
                && (f64::from(t) != translation[axis]
                    || r != 1.0
                    || last.fract() != 0.0
                    || (result_extent[axis] > 1 && (t + r == t || last - r == last)))
            {
                return Err(mapping_error(translation[axis]));
            }
        }
        Ok(Some(Self {
            source_domain,
            result_domain,
            source_origin,
            result_origin,
            source_scale,
            result_scale,
            translation,
            ratio,
            source_clip,
            result_clip,
            source_extent,
            result_extent,
            sampling,
        }))
    }
}

fn rebased_clip(
    domain: Rect,
    origin: [f64; 2],
    scale: f64,
    extent: [u32; 2],
    account: &Account,
) -> Result<[f64; 4]> {
    let edges = [domain.x(), domain.y(), domain.max().x(), domain.max().y()];
    let mut clip = [0.0; 4];
    for index in 0..4 {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let axis = index % 2;
        let value = (edges[index] - origin[axis]) * scale;
        // The allocation was rounded outward; tiny host division rounding at
        // its boundary cannot create negative or extra semantic coverage.
        if !value.is_finite() {
            return Err(mapping_error(value));
        }
        clip[index] = value.clamp(0.0, f64::from(extent[axis]));
        uploaded(clip[index])?;
    }
    for axis in 0..2 {
        let low = clip[axis] as f32;
        let high = clip[axis + 2] as f32;
        if high <= low {
            return Err(mapping_error(clip[axis + 2]));
        }
        for cell in [0.0_f32, (extent[axis] - 1) as f32] {
            finite((cell + 1.0).min(high) - cell.max(low))?;
        }
    }
    Ok(clip)
}

fn uploaded(value: f64) -> Result<f32> {
    let narrowed = value as f32;
    if !narrowed.is_finite() || (value != 0.0 && (narrowed == 0.0 || !narrowed.is_normal())) {
        Err(mapping_error(value))
    } else {
        Ok(narrowed)
    }
}

fn finite(value: f32) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(mapping_error(f64::from(value)))
    }
}

fn qualify_index(coordinate: f32, sampling: OffsetSampling) -> Result<()> {
    let rounded = if sampling == OffsetSampling::Nearest {
        finite(coordinate + 0.5)?.floor()
    } else {
        coordinate.floor()
    };
    let (before, after) = match sampling {
        OffsetSampling::Copy | OffsetSampling::Nearest => (0.0, 0.0),
        OffsetSampling::Bilinear => (0.0, 1.0),
        OffsetSampling::Cubic => (1.0, 2.0),
    };
    let index = f64::from(rounded);
    if !index.is_finite()
        || index - before < f64::from(i32::MIN)
        || index + after > f64::from(i32::MAX)
    {
        return Err(index_error(index));
    }
    Ok(())
}

pub(super) fn domains_are_empty(operation: &PreparedOffset) -> bool {
    let d = operation.domains();
    [
        intersection(d.source(), d.filter()),
        intersection(d.result(), d.filter()),
    ]
    .iter()
    .any(|rect| rect.width() == 0.0 || rect.height() == 0.0)
}

pub(super) fn require_spatial(
    context: FrameContext,
    rect: Rect,
    transform: Transform,
) -> Result<NonEmptyFrameSpatialPlan> {
    let planned = context
        .plan_local_bounds(
            LogicalBounds::try_from_rect(rect, "offset domain")?,
            transform,
        )
        .map_err(|error| {
            if error.code() == crate::ErrorCode::InvalidInput {
                mapping_error(
                    rect.x()
                        .abs()
                        .max(rect.y().abs())
                        .max(rect.width())
                        .max(rect.height()),
                )
            } else {
                error
            }
        })?;
    match planned {
        FrameSpatialPlan::NonEmpty(spatial) => Ok(spatial),
        FrameSpatialPlan::Empty(_) => Err(mapping_error(0.0)),
    }
}

fn mapping_error(value: f64) -> Error {
    crate::OffsetPreparationError::Numeric {
        phase: crate::OffsetNumericPhase::GridMapping,
        value,
    }
    .into()
}
fn index_error(value: f64) -> Error {
    crate::OffsetPreparationError::Numeric {
        phase: crate::OffsetNumericPhase::GridIndex,
        value,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Color, FilterDomains, OffsetInput, RenderAccountingContext, Size, renderer::Antialiasing,
    };

    fn context(scale: f64) -> FrameContext {
        FrameContext::try_new(
            Size::new(32.0, 32.0),
            scale,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap()
    }

    fn prepare(
        source: Rect,
        result: Rect,
        dx: f64,
        dy: f64,
        hint: ImageRendering,
    ) -> PreparedOffset {
        crate::prepare_offset(
            OffsetInput::new()
                .with_dx(dx)
                .with_dy(dy)
                .with_rendering(hint),
            FilterDomains::try_new(source, Rect::new(-32.0, -32.0, 96.0, 96.0), result).unwrap(),
            &mut RenderAccountingContext::default(),
        )
        .unwrap()
    }

    fn mapping(
        operation: &PreparedOffset,
        scale: f64,
        transform: Transform,
    ) -> Result<Option<OffsetMapping>> {
        let d = operation.domains();
        let source = require_spatial(
            context(scale),
            intersection(d.source(), d.filter()),
            transform,
        )?;
        let result = require_spatial(
            context(scale),
            intersection(d.result(), d.filter()),
            transform,
        )?;
        OffsetMapping::try_new(operation, source, result, &Account::for_test())
    }

    #[test]
    fn inverse_translation_retains_positive_negative_and_fractional_displacements() {
        let source = Rect::new(1.25, -2.5, 4.0, 4.0);
        let result = Rect::new(2.0, -1.0, 2.0, 1.0);
        // At density two the source origin is (1,-2.5), result (2,-1).
        // Independent center equation: source cell = (result-origin +
        // (j+.5)/2 - displacement - source-origin)*2 -.5.
        for (dx, dy, expected) in [
            (0.0, 0.0, [2.0, 3.0]),
            (0.5, -0.25, [1.0, 3.5]),
            (-0.25, 0.5, [2.5, 2.0]),
        ] {
            let operation = prepare(source, result, dx, dy, ImageRendering::Smooth);
            let mapping = mapping(&operation, 2.0, Transform::identity())
                .unwrap()
                .unwrap();
            assert_eq!(mapping.translation(), expected);
            assert_eq!(mapping.ratio(), [1.0; 2]);
            assert_eq!(mapping.source_origin(), [1.0, -2.5]);
            assert_eq!(mapping.result_origin(), [2.0, -1.0]);
            assert_eq!(mapping.source_extent(), [9, 8]);
            assert_eq!(mapping.result_extent(), [4, 2]);
        }
    }

    #[test]
    fn every_hint_copies_proved_integral_translation_and_fractional_hints_keep_their_kernel() {
        let bounds = Rect::new(0.0, 0.0, 8.0, 8.0);
        for (hint, expected) in [
            (ImageRendering::Auto, OffsetSampling::Bilinear),
            (ImageRendering::Smooth, OffsetSampling::Bilinear),
            (ImageRendering::Pixelated, OffsetSampling::Bilinear),
            (ImageRendering::CrispEdges, OffsetSampling::Nearest),
            (ImageRendering::HighQuality, OffsetSampling::Cubic),
        ] {
            for dx in [-1.0, 0.0, 1.0] {
                let operation = prepare(bounds, bounds, dx, 0.0, hint);
                let map = mapping(&operation, 1.0, Transform::identity())
                    .unwrap()
                    .unwrap();
                assert_eq!(map.sampling(), OffsetSampling::Copy);
                assert_eq!(map.sampling().footprint(), 1);
            }
            let fractional = prepare(bounds, bounds, 0.5, 0.0, hint);
            assert_eq!(
                mapping(&fractional, 1.0, Transform::identity())
                    .unwrap()
                    .unwrap()
                    .sampling(),
                expected
            );
        }
    }

    #[test]
    fn copy_is_not_inferred_from_a_fractional_displacement_rounded_to_integer_f32() {
        let bounds = Rect::new(0.0, 0.0, 8.0, 1.0);
        let operation = prepare(
            bounds,
            bounds,
            1.0 + 1.0e-9,
            0.0,
            ImageRendering::HighQuality,
        );
        let map = mapping(&operation, 1.0, Transform::identity())
            .unwrap()
            .unwrap();
        assert_eq!(map.translation()[0] as f32, -1.0);
        assert_eq!(map.sampling(), OffsetSampling::Cubic);
        assert_eq!(map.sampling().footprint(), 16);
    }

    #[test]
    fn partial_source_and_result_clips_are_independent_rebased_cell_boundaries() {
        let operation = prepare(
            Rect::new(0.5, 0.0, 0.5, 1.0),
            Rect::new(0.75, 0.0, 1.0, 1.0),
            0.5,
            0.0,
            ImageRendering::Auto,
        );
        let map = mapping(&operation, 1.0, Transform::identity())
            .unwrap()
            .unwrap();
        assert_eq!(map.source_clip(), [0.5, 0.0, 1.0, 1.0]);
        assert_eq!(map.result_clip(), [0.75, 0.0, 1.75, 1.0]);
        assert_eq!(map.translation(), [-0.5, 0.0]);
        assert_eq!(map.source_extent(), [1, 1]);
        assert_eq!(map.result_extent(), [2, 1]);
    }

    #[test]
    fn full_outer_affine_changes_density_while_translation_stays_layer_local() {
        let bounds = Rect::new(0.0, 0.0, 4.0, 4.0);
        let transform = Transform::try_new([2.0, 1.0, -1.0, 3.0, 7.0, -4.0]).unwrap();
        let operation = prepare(bounds, bounds, 0.25, -0.5, ImageRendering::Smooth);
        let map = mapping(&operation, 1.5, transform).unwrap().unwrap();
        assert!(map.source_scale() > 1.5);
        assert_eq!(map.source_scale(), map.result_scale());
        assert_eq!(
            map.translation(),
            [-0.25 * map.source_scale(), 0.5 * map.source_scale()]
        );
        assert_eq!(map.ratio(), [1.0; 2]);
    }

    #[test]
    fn huge_finite_disjoint_translation_is_transparent_before_narrowing_or_indices() {
        let bounds = Rect::new(0.0, 0.0, 4.0, 4.0);
        for dx in [f64::MAX, -f64::MAX] {
            let operation = prepare(bounds, bounds, dx, 0.0, ImageRendering::HighQuality);
            assert!(
                mapping(&operation, 2.0, Transform::identity())
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn tiny_nonzero_map_and_lost_half_cell_precision_have_typed_numeric_refusals() {
        let bounds = Rect::new(0.0, 0.0, 4.0, 1.0);
        let operation = prepare(bounds, bounds, f64::MIN_POSITIVE, 0.0, ImageRendering::Auto);
        let error = mapping(&operation, 1.0, Transform::identity()).unwrap_err();
        assert!(matches!(
            error.offset_diagnostic(),
            Some(crate::OffsetPreparationError::Numeric {
                phase: crate::OffsetNumericPhase::GridMapping,
                ..
            })
        ));
        let operation = prepare(bounds, bounds, 0.5, 0.0, ImageRendering::Auto);
        let mut source = require_spatial(context(1.0), bounds, Transform::identity()).unwrap();
        let result = source;
        source.device_extent.width = 8_388_609;
        let error =
            OffsetMapping::try_new(&operation, source, result, &Account::for_test()).unwrap_err();
        assert!(matches!(
            error.offset_diagnostic(),
            Some(crate::OffsetPreparationError::Numeric {
                phase: crate::OffsetNumericPhase::GridIndex,
                ..
            })
        ));
    }

    #[test]
    fn raw_rebase_restores_d_lattice_and_rejects_a_resampling_rebase() {
        let domain = Rect::new(1.0, 0.0, 2.0, 1.0);
        let union = Rect::new(0.0, 0.0, 6.0, 1.0);
        let operation = prepare(domain, domain, 0.5, 0.0, ImageRendering::HighQuality);
        let source = require_spatial(context(1.0), union, Transform::identity()).unwrap();
        let result = require_spatial(context(1.0), domain, Transform::identity()).unwrap();
        let map =
            OffsetMapping::try_rebase(&operation, source, result, &Account::for_test()).unwrap();
        assert_eq!(map.translation(), [1.0, 0.0]);
        assert_eq!(map.source_domain(), domain);
        assert_eq!(map.result_domain(), domain);
        assert_eq!(map.source_clip(), [1.0, 0.0, 3.0, 1.0]);
        assert_eq!(map.result_clip(), [0.0, 0.0, 2.0, 1.0]);
        assert_eq!(map.sampling(), OffsetSampling::Copy);
        let rescaled = require_spatial(context(2.0), domain, Transform::identity()).unwrap();
        assert!(
            OffsetMapping::try_rebase(&operation, source, rescaled, &Account::for_test()).is_err()
        );
    }

    #[test]
    fn noninteger_density_rebase_uses_proved_integer_device_lattice_without_snapping() {
        let domain = Rect::new(-2.0, -2.0, 2.0, 2.0);
        let union = Rect::new(-2.5, -2.5, 6.0, 6.0);
        let operation = prepare(domain, domain, 0.25, 0.0, ImageRendering::HighQuality);
        let source = require_spatial(context(1.5), union, Transform::identity()).unwrap();
        let result = require_spatial(context(1.5), domain, Transform::identity()).unwrap();
        assert_eq!([source.device_origin.x, result.device_origin.x], [-4, -3]);
        let rounded_world_delta =
            (result.texel_center_mapping.origin.x() - source.texel_center_mapping.origin.x()) * 1.5;
        assert_ne!(rounded_world_delta, 1.0);
        let map =
            OffsetMapping::try_rebase(&operation, source, result, &Account::for_test()).unwrap();
        assert_eq!(map.translation(), [1.0, 1.0]);
        assert_eq!(map.ratio(), [1.0, 1.0]);
        assert_eq!(map.sampling(), OffsetSampling::Copy);
    }

    fn plan(scene: crate::Scene) -> super::super::FramePlan {
        let account = Account::for_test();
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &account)
            .unwrap();
        super::super::FramePlan::try_from_commands(commands, context(1.0), &account).unwrap()
    }

    #[test]
    fn semantic_offset_keeps_source_and_result_and_full_outer_transform() {
        let source = Rect::new(0.0, 0.0, 8.0, 2.0);
        let result = Rect::new(3.0, 0.0, 2.0, 1.0);
        let operation = prepare(source, result, 0.5, 0.0, ImageRendering::Smooth);
        let transform = Transform::try_new([-1.0, 0.25, 0.5, 1.0, 12.0, 4.0]).unwrap();
        let mut scene = crate::Scene::new();
        scene.layer(
            crate::Layer::new()
                .with_offset(operation)
                .with_transform(transform),
            |child| {
                child.fill(source, Color::BLACK);
            },
        );
        let super::super::FramePlan::GpuGraph(graph) = plan(scene) else {
            panic!("offset requires graph");
        };
        assert_eq!(graph.offsets.len(), 1);
        let entry = &graph.offsets[0];
        assert_eq!(entry.stage, OffsetStage::Sample);
        assert_eq!(entry.mapping.source_domain(), source);
        assert_eq!(entry.mapping.result_domain(), result);
        let mut found_transform = false;
        for composite in &graph.composites {
            if let super::super::graph::SemanticCompositeKind::Layer {
                transform: actual, ..
            } = composite.kind
            {
                found_transform |= actual == transform;
            }
        }
        assert!(found_transform);
        graph.lowering_view(&Account::for_test()).unwrap();
    }

    #[test]
    fn empty_domains_skip_sources_and_keep_destructive_transparent_composition() {
        let full = Rect::new(0.0, 0.0, 4.0, 1.0);
        for (source, result) in [
            (Rect::new(0.0, 0.0, 0.0, 1.0e200), full),
            (full, Rect::new(0.0, 0.0, 0.0, 1.0e200)),
        ] {
            let operation = prepare(source, result, 0.0, 0.0, ImageRendering::Auto);
            let mut scene = crate::Scene::new();
            scene.fill(full, Color::BLACK);
            scene.layer(
                crate::Layer::new().with_offset(operation).with_composition(
                    crate::LayerComposition::normal_porter_duff(crate::PorterDuffOperator::Copy),
                ),
                |child| {
                    child.fill(full, Color::BLACK);
                },
            );
            let super::super::FramePlan::GpuGraph(graph) = plan(scene) else {
                panic!("Copy requires graph");
            };
            assert!(graph.offsets.is_empty());
            assert!(
                graph
                    .vello_spans
                    .iter()
                    .all(|span| span.scope
                        != super::super::graph::SemanticVelloSpanScope::LayerSource)
            );
            assert!(
                graph
                    .composites
                    .iter()
                    .any(|composite| matches!(&composite.kind,
                super::super::graph::SemanticCompositeKind::Layer { composition, .. }
                if composition.operator() == Some(crate::PorterDuffOperator::Copy)))
            );
            graph.lowering_view(&Account::for_test()).unwrap();
        }
    }

    #[test]
    fn qualification_copies_shared_layer_metadata_before_pruning_empty_offset_input() {
        use crate::{BackdropCaptureBounds, BackdropFilterInput, FilterBlur, FilterList, FilterOp};
        let account = Account::for_test();
        let full = Rect::new(0.0, 0.0, 4.0, 1.0);
        let operation = prepare(
            Rect::new(0.0, 0.0, 0.0, 1.0),
            full,
            0.0,
            0.0,
            ImageRendering::Auto,
        );
        let backdrop = BackdropFilterInput::try_new(
            FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(0.0).unwrap())]).unwrap(),
            BackdropCaptureBounds::try_new(full).unwrap(),
            None,
        )
        .unwrap();
        let mut scene = crate::Scene::new();
        scene.layer(
            crate::Layer::new()
                .with_offset(operation)
                .try_backdrop_filter(backdrop)
                .unwrap(),
            |child| {
                child.fill(full, Color::BLACK);
            },
        );
        let normalized = scene
            .normalize(crate::Capabilities::CURRENT, &account)
            .unwrap();
        let original = normalized.clone();
        let [
            crate::command::RenderCommand::Layer {
                layer: before,
                children,
            },
        ] = original.as_ref()
        else {
            panic!("the original normalized layer must remain available");
        };
        assert!(before.backdrop.is_some());
        assert!(!children.is_empty());
        let context = context(1.0);
        let qualified = super::super::bounds::SemanticSourceContribution::try_from_commands(
            normalized,
            context.initial_parent_contribution(),
            super::super::bounds::SemanticContributionDomain::RootOutputBounded(
                context.output_bounds,
            ),
            context,
            Transform::identity(),
            &account,
        )
        .unwrap();
        assert!(qualified.commands.is_empty());
        let [
            crate::command::RenderCommand::Layer {
                layer: after,
                children,
            },
        ] = original.as_ref()
        else {
            panic!("qualification must not alter another command owner");
        };
        assert!(before.same_allocation(after));
        assert!(after.backdrop.is_some());
        assert!(!children.is_empty());
    }

    #[test]
    fn backdrop_only_input_is_retained_and_rebased_before_sampling() {
        let full = Rect::new(0.0, 0.0, 6.0, 1.0);
        let d = Rect::new(1.0, 0.0, 2.0, 1.0);
        let operation = prepare(d, d, 0.5, 0.0, ImageRendering::Auto);
        let backdrop = crate::BackdropFilterInput::try_new(
            crate::FilterList::try_ops(vec![crate::FilterOp::blur(
                crate::FilterBlur::try_new(0.0).unwrap(),
            )])
            .unwrap(),
            crate::BackdropCaptureBounds::try_new(full).unwrap(),
            None,
        )
        .unwrap();
        let mut scene = crate::Scene::new();
        scene.fill(full, Color::BLACK);
        scene.layer(
            crate::Layer::new()
                .with_offset(operation)
                .try_backdrop_filter(backdrop)
                .unwrap(),
            |_| {},
        );
        let super::super::FramePlan::GpuGraph(graph) = plan(scene) else {
            panic!("backdrop requires graph");
        };
        assert_eq!(graph.offsets.len(), 2);
        assert_eq!(graph.offsets[0].stage, OffsetStage::RebaseSource);
        assert_eq!(graph.offsets[1].stage, OffsetStage::Sample);
        assert_eq!(graph.offsets[0].mapping.source_origin(), [0.0, 0.0]);
        assert_eq!(graph.offsets[1].mapping.source_origin(), [1.0, 0.0]);
        graph.lowering_view(&Account::for_test()).unwrap();
    }

    #[test]
    fn malformed_mapping_metadata_is_rejected_before_lowering() {
        let full = Rect::new(0.0, 0.0, 4.0, 1.0);
        let operation = prepare(full, full, 0.5, 0.0, ImageRendering::Auto);
        let mut scene = crate::Scene::new();
        scene.layer(crate::Layer::new().with_offset(operation), |child| {
            child.fill(full, Color::BLACK);
        });
        let super::super::FramePlan::GpuGraph(mut graph) = plan(scene) else {
            panic!("offset requires graph");
        };
        graph.offsets[0].mapping.translation[0] = 0.0;
        assert!(graph.lowering_view(&Account::for_test()).is_err());
    }
}
