//! Native Gaussian grids. All shader coordinates are relative to integer
//! allocations; the full authored affine belongs to final composition only.
use super::{
    FrameContext, GraphLoweringSpatialDescriptor,
    bounds::{FrameSpatialPlan, LogicalBounds, NonEmptyFrameSpatialPlan},
    convolution::intersection,
    graph::{SemanticResourceDescriptor, SemanticResourceRole},
};
use crate::{
    Rect, Transform,
    accounting::{Account, RenderWork, SharedValue},
    error::{Error, Result},
    gaussian::GaussianSourceDescriptor,
    resource::{GaussianKernelGeometry, GaussianKernelSamplingForm},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GaussianAxis {
    X,
    Y,
}
impl GaussianAxis {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
        }
    }
    pub(crate) const fn shader_value(self) -> u32 {
        self.index() as u32
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GaussianStage {
    RebaseSource,
    PassThrough,
    Prepare,
    Accumulate {
        axis: GaussianAxis,
        first_tap: u32,
        tap_count: u32,
        first: bool,
    },
    Resolve,
    Constant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GaussianPurpose {
    Standalone,
    ConsumedAlpha,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GaussianChannels {
    Rgba,
    Alpha,
}

/// Immutable checked pass facts shared by semantic planning, lowering and
/// runtime. One admitted payload retains the accounting custody until its final
/// graph or runtime reference is released. Cloning only shares that payload.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GaussianPassFacts(SharedValue<GaussianPassData>);

#[derive(Debug, PartialEq)]
struct GaussianPassData {
    operation: crate::PreparedGaussian,
    source: crate::gaussian::GaussianSourceDescriptor,
    mapping: GaussianMapping,
    stage: GaussianStage,
    purpose: GaussianPurpose,
}
impl GaussianPassFacts {
    pub(super) fn try_new(
        foreground: crate::gaussian::GaussianForeground,
        mapping: GaussianMapping,
        stage: GaussianStage,
        storage: &crate::accounting::StorageAccount,
    ) -> Result<Self> {
        Self::try_new_for_purpose(
            foreground,
            mapping,
            stage,
            GaussianPurpose::Standalone,
            storage,
        )
    }
    pub(super) fn try_new_for_purpose(
        foreground: crate::gaussian::GaussianForeground,
        mapping: GaussianMapping,
        stage: GaussianStage,
        purpose: GaussianPurpose,
        storage: &crate::accounting::StorageAccount,
    ) -> Result<Self> {
        SharedValue::new(
            GaussianPassData {
                operation: foreground.operation,
                source: foreground.source,
                mapping,
                stage,
                purpose,
            },
            storage,
        )
        .map(Self)
    }
    #[cfg(test)]
    pub(crate) fn with_purpose_for_test(
        &self,
        purpose: GaussianPurpose,
        account: &Account,
    ) -> Result<Self> {
        Self::try_new_for_purpose(
            self.foreground(),
            self.mapping(),
            self.stage(),
            purpose,
            &account.storage(),
        )
    }
    pub(crate) fn purpose(&self) -> GaussianPurpose {
        self.0.purpose
    }
    pub(crate) fn channels(&self) -> GaussianChannels {
        match self.0.purpose {
            GaussianPurpose::Standalone => GaussianChannels::Rgba,
            GaussianPurpose::ConsumedAlpha => GaussianChannels::Alpha,
        }
    }
    pub(crate) fn operation(&self) -> &crate::PreparedGaussian {
        &self.0.operation
    }
    pub(crate) fn source(&self) -> crate::gaussian::GaussianSourceDescriptor {
        self.0.source
    }
    pub(crate) fn mapping(&self) -> GaussianMapping {
        self.0.mapping
    }
    pub(crate) fn stage(&self) -> GaussianStage {
        self.0.stage
    }
    pub(super) fn foreground(&self) -> crate::gaussian::GaussianForeground {
        crate::gaussian::GaussianForeground {
            operation: self.0.operation,
            source: self.0.source,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GaussianMapping {
    source_spatial: Option<NonEmptyFrameSpatialPlan>,
    target_spatial: NonEmptyFrameSpatialPlan,
    source_base: [i32; 2],
    source_clip: [f64; 4],
    output_clip: [f64; 4],
    periodic_base: [f64; 2],
    periodic_step: [f64; 2],
    periodic: bool,
    direct: bool,
    converted: bool,
    geometries: [Option<GaussianKernelGeometry>; 2],
    original_source: Option<NonEmptyFrameSpatialPlan>,
    result: NonEmptyFrameSpatialPlan,
    period: Option<PeriodCapture>,
    color: crate::Color,
}
impl GaussianMapping {
    pub(crate) fn source_spatial(self) -> Option<GraphLoweringSpatialDescriptor> {
        self.source_spatial.map(spatial_descriptor)
    }
    pub(crate) fn target_spatial(self) -> GraphLoweringSpatialDescriptor {
        spatial_descriptor(self.target_spatial)
    }
    pub(crate) fn source_extent(self) -> [u32; 2] {
        self.source_spatial.map(extent).unwrap_or([1; 2])
    }
    pub(crate) const fn target_extent(self) -> [u32; 2] {
        extent(self.target_spatial)
    }
    pub(super) fn output_footprint(self) -> super::FilterOutputFootprint {
        super::FilterOutputFootprint::from_qualified_grid(self.target_spatial(), self.output_clip)
    }
    pub(crate) const fn color(self) -> crate::Color {
        self.color
    }
    pub(super) const fn with_color(mut self, color: crate::Color) -> Self {
        self.color = color;
        self
    }
    pub(super) const fn with_route(
        mut self,
        original_source: NonEmptyFrameSpatialPlan,
        result: NonEmptyFrameSpatialPlan,
        period: Option<PeriodCapture>,
    ) -> Self {
        self.original_source = Some(original_source);
        self.result = result;
        self.period = period;
        self
    }
    pub(crate) const fn source_base(self) -> [i32; 2] {
        self.source_base
    }
    pub(crate) const fn source_clip(self) -> [f64; 4] {
        self.source_clip
    }
    pub(crate) const fn output_clip(self) -> [f64; 4] {
        self.output_clip
    }
    pub(crate) const fn periodic_base(self) -> [f64; 2] {
        self.periodic_base
    }
    pub(crate) const fn periodic_step(self) -> [f64; 2] {
        self.periodic_step
    }
    pub(crate) const fn periodic(self) -> bool {
        self.periodic
    }
    pub(crate) const fn direct(self) -> bool {
        self.direct
    }
    pub(crate) const fn converted(self) -> bool {
        self.converted
    }
    pub(crate) const fn geometry(self, axis: GaussianAxis) -> Option<GaussianKernelGeometry> {
        self.geometries[axis.index()]
    }
    pub(crate) const fn active_axes(self) -> [bool; 2] {
        [self.geometries[0].is_some(), self.geometries[1].is_some()]
    }
    pub(super) fn new(
        source: Option<NonEmptyFrameSpatialPlan>,
        target: NonEmptyFrameSpatialPlan,
        geometries: [Option<GaussianKernelGeometry>; 2],
    ) -> Self {
        let source_extent = source.map(extent).unwrap_or([1; 2]);
        let target_extent = extent(target);
        Self {
            source_spatial: source,
            target_spatial: target,
            source_base: [0; 2],
            source_clip: full_clip(source_extent),
            output_clip: full_clip(target_extent),
            periodic_base: [0.0; 2],
            periodic_step: [1.0; 2],
            periodic: false,
            direct: false,
            converted: false,
            geometries,
            original_source: source,
            result: target,
            period: None,
            color: crate::Color::TRANSPARENT,
        }
    }
    pub(super) fn with_base(
        mut self,
        base: [i64; 2],
        radius: [u32; 2],
        account: &Account,
    ) -> Result<Self> {
        let size = self.target_extent();
        for axis in 0..2 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let low = base[axis]
                .checked_sub(i64::from(radius[axis]))
                .ok_or_else(|| index_error(base[axis] as f64))?;
            let high = base[axis]
                .checked_add(i64::from(size[axis]) - 1)
                .and_then(|n| n.checked_add(i64::from(radius[axis])))
                .ok_or_else(|| index_error(base[axis] as f64))?;
            i32::try_from(low).map_err(|_| index_error(low as f64))?;
            i32::try_from(high).map_err(|_| index_error(high as f64))?;
            self.source_base[axis] =
                i32::try_from(base[axis]).map_err(|_| index_error(base[axis] as f64))?;
        }
        Ok(self)
    }
    pub(super) fn with_source_clip(
        mut self,
        domain: Rect,
        spatial: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        self.source_clip = clip(domain, spatial, account)?;
        Ok(self)
    }
    pub(super) fn with_output_clip(
        mut self,
        domain: Rect,
        spatial: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        self.output_clip = clip(domain, spatial, account)?;
        Ok(self)
    }
    pub(super) const fn with_flags(
        mut self,
        periodic: bool,
        direct: bool,
        converted: bool,
    ) -> Self {
        self.periodic = periodic;
        self.direct = direct;
        self.converted = converted;
        self
    }
    pub(super) fn with_periodic_resolve(
        mut self,
        period: PeriodCapture,
        result: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        let origin = result.texel_center_mapping.origin;
        let values = [origin.x(), origin.y()];
        let tile = [period.period.x(), period.period.y()];
        let sizes = extent(period.spatial);
        for axis in 0..2 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            // Reducing before upload preserves the phase for far-away output.
            // Divide in logical units first to avoid overflow from a large
            // difference times density, then multiply the bounded remainder.
            let length = if axis == 0 {
                period.period.width()
            } else {
                period.period.height()
            };
            let phase = (values[axis].rem_euclid(length) - tile[axis].rem_euclid(length))
                .rem_euclid(length);
            if !phase.is_finite() {
                return Err(mapping_error(phase));
            }
            self.periodic_base[axis] =
                (phase * period.density[axis]).rem_euclid(f64::from(sizes[axis]));
            self.periodic_step[axis] = period.density[axis] / result.raster_scale.get();
            let b = uploaded(self.periodic_base[axis])?;
            let step = uploaded(self.periodic_step[axis])?;
            if step <= 0.0 || !step.is_normal() {
                return Err(mapping_error(self.periodic_step[axis]));
            }
            // Exactly the multiply/add/subtract consumed by WGSL. Monotone
            // endpoints bound every interior invocation and both +1 neighbors.
            for p in [0.5_f32, self.target_extent()[axis] as f32 - 0.5] {
                let product = finite(p * step)?;
                let shifted = finite(product + b)?;
                let coordinate = finite(shifted - 0.5)?;
                let floor = f64::from(coordinate.floor());
                if floor < f64::from(i32::MIN) || floor + 1.0 > f64::from(i32::MAX) {
                    return Err(index_error(floor));
                }
                // Reconstruction needs a distinguishable fraction and next
                // integer; if f32 cannot represent it refuse that actual map.
                if coordinate.abs() >= 8_388_608.0 {
                    return Err(mapping_error(f64::from(coordinate)));
                }
            }
        }
        self.periodic = true;
        Ok(self)
    }
    /// Re-establish the concrete private mapping at closure, under its actual
    /// source declaration and operation. No graph scan or coefficient work.
    pub(crate) fn proves_exact(
        self,
        operation: &crate::PreparedGaussian,
        source: GaussianSourceDescriptor,
        stage: GaussianStage,
        account: &Account,
    ) -> Result<bool> {
        account.work(RenderWork::GraphNodeVisits, 1)?;
        let output = result_domain(source);
        if empty(output) || self.result.logical_bounds.rect() != output {
            return Ok(false);
        }
        let context = FrameContext {
            output_bounds: LogicalBounds::NonEmpty(self.result.logical_bounds),
            surface_scale: self.result.raster_scale.get(),
            antialiasing: crate::renderer::Antialiasing::Area,
            base_color: crate::Color::TRANSPARENT,
        };
        if require_spatial(context, output, Transform::identity())? != self.result {
            return Ok(false);
        }
        if stage == GaussianStage::Constant {
            let color = match source {
                GaussianSourceDescriptor::Constant { color, .. } => color,
                _ => crate::Color::TRANSPARENT,
            };
            let expected = Self::new(None, self.result, [None; 2])
                .with_color(self.color)
                .with_output_clip(output, self.result, account)?;
            return Ok(self == expected && self.color == color);
        }
        let Some(original) = self.original_source else {
            return Ok(false);
        };
        let expected_period = match source {
            GaussianSourceDescriptor::Periodic { period, .. } => Some(PeriodCapture::try_new(
                context,
                period,
                Transform::identity(),
                account,
            )?),
            GaussianSourceDescriptor::Finite { .. } => None,
            GaussianSourceDescriptor::Constant { .. } => return Ok(false),
        };
        // The context's antialiasing is irrelevant to the grid. Preserve its
        // actual captured value in the equality witness rather than fabricate it.
        let period = match (self.period, expected_period) {
            (Some(actual), Some(expected))
                if actual.period == expected.period
                    && actual.spatial == expected.spatial
                    && actual.transform == expected.transform
                    && actual.density == expected.density
                    && actual.context.surface_scale == 1.0
                    && actual.context.output_bounds == expected.context.output_bounds =>
            {
                Some(actual)
            }
            (None, None) => None,
            _ => return Ok(false),
        };
        let expected_original = if let Some(period) = period {
            period.spatial
        } else {
            require_spatial(
                context,
                finite_domain(source).ok_or_else(|| mapping_error(0.0))?,
                Transform::identity(),
            )?
        };
        if original != expected_original {
            return Ok(false);
        }
        let geometries = if matches!(
            stage,
            GaussianStage::RebaseSource | GaussianStage::PassThrough
        ) {
            [None; 2]
        } else {
            kernel_geometries(
                *operation,
                period
                    .map(|p| p.density)
                    .unwrap_or([original.raster_scale.get(); 2]),
                account,
            )?
        };
        if geometries != self.geometries {
            return Ok(false);
        }
        let route = |mapping: Self| mapping.with_route(original, self.result, period);
        let expected = match stage {
            GaussianStage::RebaseSource => {
                if period.is_some() {
                    return Ok(false);
                }
                let Some(raw) = self.source_spatial else {
                    return Ok(false);
                };
                let bounds = raw.logical_bounds.rect();
                let d = original.logical_bounds.rect();
                if bounds.x() > d.x()
                    || bounds.y() > d.y()
                    || bounds.max().x() < d.max().x()
                    || bounds.max().y() < d.max().y()
                {
                    return Ok(false);
                }
                route(Self::new(Some(raw), original, geometries)).with_base(
                    base_between(raw, original)?,
                    [0; 2],
                    account,
                )?
            }
            GaussianStage::PassThrough => {
                if !operation.is_passthrough() {
                    return Ok(false);
                }
                let mapping = route(Self::new(Some(original), self.result, geometries))
                    .with_output_clip(output, self.result, account)?;
                if let Some(period) = period {
                    mapping.with_periodic_resolve(period, self.result, account)?
                } else {
                    mapping
                        .with_base(base_between(original, self.result)?, [0; 2], account)?
                        .with_source_clip(original.logical_bounds.rect(), original, account)?
                }
            }
            GaussianStage::Prepare => {
                if operation.is_passthrough() {
                    return Ok(false);
                }
                let prepared = allocation_spatial(context, extent(original))?;
                let mapping = route(Self::new(Some(original), prepared, geometries)).with_flags(
                    period.is_some(),
                    false,
                    false,
                );
                if period.is_some() {
                    mapping
                } else {
                    mapping.with_source_clip(original.logical_bounds.rect(), original, account)?
                }
            }
            GaussianStage::Accumulate {
                axis,
                first_tap,
                tap_count,
                first,
            } => {
                let Some(geometry) = geometries[axis.index()] else {
                    return Ok(false);
                };
                if tap_count == 0
                    || tap_count > 32
                    || first_tap
                        .checked_add(tap_count)
                        .is_none_or(|end| end > geometry.sample_count())
                    || first != (first_tap == 0)
                    || first_tap % 32 != 0
                    || tap_count != (geometry.sample_count() - first_tap).min(32)
                {
                    return Ok(false);
                }
                let both = geometries[0].is_some() && geometries[1].is_some();
                let ry = geometries[1].map(|g| g.support_radius()).unwrap_or(0);
                let result_size = extent(self.result);
                let halo_height = u64::from(result_size[1]) + 2 * u64::from(ry);
                let halo = if period.is_none() && both {
                    if halo_height > 8_388_608 {
                        return Ok(false);
                    }
                    Some(allocation_spatial(
                        context,
                        [result_size[0], halo_height as u32],
                    )?)
                } else {
                    None
                };
                let prepared = allocation_spatial(context, extent(original))?;
                let final_grid = if period.is_some() {
                    original
                } else {
                    allocation_spatial(context, result_size)?
                };
                let direct = period.is_none() && both && axis == GaussianAxis::Y;
                let input = if direct {
                    halo.ok_or_else(|| mapping_error(0.0))?
                } else {
                    prepared
                };
                let target = if axis == GaussianAxis::X {
                    halo.unwrap_or(final_grid)
                } else {
                    final_grid
                };
                let base = if period.is_some() {
                    [0; 2]
                } else if direct {
                    [0, i64::from(ry)]
                } else {
                    let mut base = base_between(original, self.result)?;
                    if both && axis == GaussianAxis::X {
                        base[1] -= i64::from(ry);
                    }
                    base
                };
                let mut radii = [0; 2];
                radii[axis.index()] = geometry.support_radius();
                route(Self::new(Some(input), target, geometries))
                    .with_flags(period.is_some(), direct, false)
                    .with_base(base, radii, account)?
            }
            GaussianStage::Resolve => {
                if operation.is_passthrough() {
                    return Ok(false);
                }
                let final_grid = if period.is_some() {
                    original
                } else {
                    allocation_spatial(context, extent(self.result))?
                };
                let mapping = route(Self::new(Some(final_grid), self.result, geometries))
                    .with_flags(period.is_some(), false, true)
                    .with_output_clip(output, self.result, account)?;
                if let Some(period) = period {
                    mapping.with_periodic_resolve(period, self.result, account)?
                } else {
                    mapping
                }
            }
            GaussianStage::Constant => return Ok(false),
        };
        Ok(self == expected)
    }
    pub(super) const fn target_plan(self) -> NonEmptyFrameSpatialPlan {
        self.target_spatial
    }
}
fn spatial_descriptor(spatial: NonEmptyFrameSpatialPlan) -> GraphLoweringSpatialDescriptor {
    GraphLoweringSpatialDescriptor::from_semantic(SemanticResourceDescriptor::new(
        SemanticResourceRole::FilterIntermediate,
        spatial,
        0,
    ))
}
pub(super) const fn extent(spatial: NonEmptyFrameSpatialPlan) -> [u32; 2] {
    [spatial.device_extent.width, spatial.device_extent.height]
}
fn full_clip(size: [u32; 2]) -> [f64; 4] {
    [0.0, 0.0, f64::from(size[0]), f64::from(size[1])]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PeriodCapture {
    pub(super) period: Rect,
    pub(super) spatial: NonEmptyFrameSpatialPlan,
    pub(super) context: FrameContext,
    pub(super) transform: Transform,
    pub(super) density: [f64; 2],
}
impl PeriodCapture {
    pub(super) fn try_new(
        context: FrameContext,
        period: Rect,
        raster_transform: Transform,
        account: &Account,
    ) -> Result<Self> {
        let rho = context
            .local_raster_scale(raster_transform)?
            .ok_or_else(|| mapping_error(0.0))?
            .get();
        let mut sizes = [0; 2];
        let mut density = [0.0; 2];
        for (axis, length) in [period.width(), period.height()].into_iter().enumerate() {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let size = (length * rho).ceil().max(1.0);
            if !size.is_finite() || size > 8_388_608.0 {
                return Err(index_error(size));
            }
            sizes[axis] = size as u32;
            density[axis] = size / length;
            let d = uploaded(density[axis])?;
            if !d.is_normal() || !uploaded(1.0 / density[axis])?.is_normal() {
                return Err(mapping_error(density[axis]));
            }
        }
        let spatial = allocation_spatial(context, sizes)?;
        let mut scoped = context;
        scoped.surface_scale = 1.0;
        scoped.output_bounds = LogicalBounds::NonEmpty(spatial.logical_bounds);
        let coefficients = [
            density[0],
            0.0,
            0.0,
            density[1],
            -period.x() * density[0],
            -period.y() * density[1],
        ];
        for value in coefficients {
            account.work(RenderWork::CoordinateVisits, 1)?;
            uploaded(value)?;
        }
        let transform =
            Transform::try_new(coefficients).map_err(|_| mapping_error(coefficients[4]))?;
        Ok(Self {
            period,
            spatial,
            context: scoped,
            transform,
            density,
        })
    }
}

pub(super) fn kernel_geometries(
    operation: crate::PreparedGaussian,
    density: [f64; 2],
    account: &Account,
) -> Result<[Option<GaussianKernelGeometry>; 2]> {
    let mut geometries = [None; 2];
    if let Some(sigma) = operation.sigma() {
        for axis in 0..2 {
            if sigma[axis] > 0.0 {
                geometries[axis] = Some(GaussianKernelGeometry::try_new(
                    sigma[axis],
                    density[axis],
                    4.0,
                    GaussianKernelSamplingForm::FullNearest,
                    account,
                )?);
            }
        }
    }
    Ok(geometries)
}
pub(super) fn result_domain(source: GaussianSourceDescriptor) -> Rect {
    intersection(source.filter(), source.result())
}
pub(super) fn finite_domain(source: GaussianSourceDescriptor) -> Option<Rect> {
    match source {
        GaussianSourceDescriptor::Finite { domains, .. } => {
            Some(intersection(domains.source(), domains.filter()))
        }
        _ => None,
    }
}
pub(super) fn source_is_empty(source: GaussianSourceDescriptor) -> bool {
    finite_domain(source).is_some_and(empty)
}
pub(super) fn empty(rect: Rect) -> bool {
    rect.width() == 0.0 || rect.height() == 0.0
}
pub(super) fn allocation_spatial(
    context: FrameContext,
    size: [u32; 2],
) -> Result<NonEmptyFrameSpatialPlan> {
    let mut unit_context = context;
    unit_context.surface_scale = 1.0;
    require_spatial(
        unit_context,
        Rect::new(0.0, 0.0, f64::from(size[0]), f64::from(size[1])),
        Transform::identity(),
    )
}
pub(super) fn require_spatial(
    context: FrameContext,
    rect: Rect,
    transform: Transform,
) -> Result<NonEmptyFrameSpatialPlan> {
    let plan = context
        .plan_local_bounds(
            LogicalBounds::try_from_rect(rect, "Gaussian domain")?,
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
    let FrameSpatialPlan::NonEmpty(spatial) = plan else {
        return Err(mapping_error(0.0));
    };
    for size in extent(spatial) {
        if size > 8_388_608 {
            return Err(index_error(f64::from(size)));
        }
    }
    Ok(spatial)
}
pub(super) fn base_between(
    source: NonEmptyFrameSpatialPlan,
    target: NonEmptyFrameSpatialPlan,
) -> Result<[i64; 2]> {
    // This is a proof about raw aligned lattices, not rounded f32 origins.
    if source.raster_scale != target.raster_scale {
        return Err(mapping_error(source.raster_scale.get()));
    }
    for spatial in [source, target] {
        if spatial.texel_center_mapping.origin.x()
            != f64::from(spatial.device_origin.x) / spatial.raster_scale.get()
            || spatial.texel_center_mapping.origin.y()
                != f64::from(spatial.device_origin.y) / spatial.raster_scale.get()
        {
            return Err(mapping_error(spatial.texel_center_mapping.origin.x()));
        }
    }
    Ok([
        i64::from(target.device_origin.x) - i64::from(source.device_origin.x),
        i64::from(target.device_origin.y) - i64::from(source.device_origin.y),
    ])
}
fn clip(domain: Rect, spatial: NonEmptyFrameSpatialPlan, account: &Account) -> Result<[f64; 4]> {
    let origin = [
        spatial.texel_center_mapping.origin.x(),
        spatial.texel_center_mapping.origin.y(),
    ];
    let edges = [domain.x(), domain.y(), domain.max().x(), domain.max().y()];
    let size = extent(spatial);
    let mut result = [0.0; 4];
    for index in 0..4 {
        account.work(RenderWork::CoordinateVisits, 1)?;
        let axis = index % 2;
        let value = (edges[index] - origin[axis]) * spatial.raster_scale.get();
        if !value.is_finite() {
            return Err(mapping_error(value));
        }
        result[index] = value.clamp(0.0, f64::from(size[axis]));
        uploaded(result[index])?;
    }
    for axis in 0..2 {
        if result[axis + 2] as f32 <= result[axis] as f32 {
            return Err(mapping_error(result[axis + 2]));
        }
        for cell in [0.0_f32, (size[axis] - 1) as f32] {
            finite((cell + 1.0).min(result[axis + 2] as f32) - cell.max(result[axis] as f32))?;
        }
    }
    Ok(result)
}
fn uploaded(value: f64) -> Result<f32> {
    let n = value as f32;
    if !n.is_finite() || (value != 0.0 && !n.is_normal()) {
        Err(mapping_error(value))
    } else {
        Ok(n)
    }
}
fn finite(value: f32) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(mapping_error(f64::from(value)))
    }
}
fn mapping_error(value: f64) -> Error {
    crate::GaussianPreparationError::Numeric {
        phase: crate::GaussianNumericPhase::GridMapping,
        value,
    }
    .into()
}
fn index_error(value: f64) -> Error {
    crate::GaussianPreparationError::Numeric {
        phase: crate::GaussianNumericPhase::GridIndex,
        value,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Color, FilterColorSpace, FilterDomains, GaussianInput, GaussianSource, Layer,
        RenderAccountingContext, Scene, Size, renderer::Antialiasing,
    };
    fn context(scale: f64) -> FrameContext {
        FrameContext::try_new(
            Size::new(64.0, 64.0),
            scale,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap()
    }
    fn operation(x: f64, y: f64) -> crate::PreparedGaussian {
        crate::prepare_gaussian(
            GaussianInput::new().with_standard_deviations(x, y),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap()
    }
    fn planned(
        scene: &Scene,
        scale: f64,
    ) -> crate::accounting::OwnedBox<super::super::GpuRenderGraph> {
        let account = Account::for_test();
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &account)
            .unwrap();
        match super::super::FramePlan::try_from_commands(commands, context(scale), &account)
            .unwrap()
        {
            super::super::FramePlan::GpuGraph(graph) => graph,
            _ => panic!("native Gaussian must retain graph execution"),
        }
    }
    fn tile(period: Rect) -> Scene {
        let mut scene = Scene::new();
        scene.fill(period, Color::BLACK);
        scene
    }
    #[test]
    fn fractional_period_maps_to_full_integer_cells_with_distinct_axis_density() {
        let period = Rect::new(-2.25, 3.5, 2.25, 1.25);
        let capture = PeriodCapture::try_new(
            context(2.0),
            period,
            Transform::identity(),
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!(extent(capture.spatial), [5, 3]);
        assert_eq!(capture.density, [5.0 / 2.25, 3.0 / 1.25]);
        assert_eq!(capture.context.surface_scale, 1.0);
        assert_eq!(
            capture.context.output_bounds,
            LogicalBounds::try_from_rect(Rect::new(0.0, 0.0, 5.0, 3.0), "canvas").unwrap()
        );
        let [a, _, _, d, x, y] = capture.transform.as_array();
        assert_eq!(a * period.x() + x, 0.0);
        assert_eq!(d * period.y() + y, 0.0);
        assert!((a * period.max().x() + x - 5.0).abs() < 1e-14);
        assert!((d * period.max().y() + y - 3.0).abs() < 1e-14);
        let geometries =
            kernel_geometries(operation(1.0, 1.0), capture.density, &Account::for_test()).unwrap();
        assert_ne!(geometries[0].unwrap().key(), geometries[1].unwrap().key());
    }
    #[test]
    fn raw_rebase_proves_device_origin_difference_at_fractional_density() {
        let source = require_spatial(
            context(1.5),
            Rect::new(-2.5, 0.0, 6.0, 1.0),
            Transform::identity(),
        )
        .unwrap();
        let target = require_spatial(
            context(1.5),
            Rect::new(-2.0, 0.0, 2.0, 1.0),
            Transform::identity(),
        )
        .unwrap();
        // Outward origins -4 and -3 on the shared rho=1.5 lattice differ by
        // exactly one texel, despite rounded quotients in logical coordinates.
        assert_eq!(base_between(source, target).unwrap(), [1, 0]);
        let mapping = GaussianMapping::new(Some(source), target, [None; 2])
            .with_base(
                base_between(source, target).unwrap(),
                [0; 2],
                &Account::for_test(),
            )
            .unwrap();
        assert_eq!(mapping.source_base(), [1, 0]);
    }
    #[test]
    fn far_periodic_reconstruction_reduces_phase_before_float_upload() {
        let account = Account::for_test();
        let period = PeriodCapture::try_new(
            context(2.0),
            Rect::new(0.25, 0.0, 2.25, 1.0),
            Transform::identity(),
            &account,
        )
        .unwrap();
        let near = require_spatial(
            context(2.0),
            Rect::new(1.0, 0.0, 2.0, 1.0),
            Transform::identity(),
        )
        .unwrap();
        let far = require_spatial(
            context(2.0),
            Rect::new(900_000_001.0, 0.0, 2.0, 1.0),
            Transform::identity(),
        )
        .unwrap();
        let map = |result| {
            GaussianMapping::new(Some(period.spatial), result, [None; 2])
                .with_periodic_resolve(period, result, &account)
                .unwrap()
        };
        assert_eq!(map(near).periodic_base(), map(far).periodic_base());
        assert_eq!(map(near).periodic_step(), map(far).periodic_step());
    }
    #[test]
    fn huge_period_origin_keeps_small_output_phase_before_subtraction() {
        let account = Account::for_test();
        // 2^60 mod 768 = 256, hence (2^60+256) mod 768 = 512.
        // The output origin .25 therefore has phase 256.25, or 1025 texels
        // at density four. Subtracting the huge origin first loses the .25.
        let period = PeriodCapture::try_new(
            context(4.0),
            Rect::new(1_152_921_504_606_847_232.0, 0.0, 768.0, 1.0),
            Transform::identity(),
            &account,
        )
        .unwrap();
        let result = require_spatial(
            context(4.0),
            Rect::new(0.25, 0.0, 1.0, 1.0),
            Transform::identity(),
        )
        .unwrap();
        let mapping = GaussianMapping::new(Some(period.spatial), result, [None; 2])
            .with_periodic_resolve(period, result, &account)
            .unwrap();
        assert_eq!(mapping.periodic_base(), [1025.0, 0.0]);
        assert_eq!(mapping.periodic_step(), [1.0, 1.0]);
    }
    #[test]
    fn analytic_constant_mapping_cannot_substitute_transparent_color() {
        let account = Account::for_test();
        let domain = Rect::new(0.0, 0.0, 1.0, 1.0);
        let result = require_spatial(context(1.0), domain, Transform::identity()).unwrap();
        let source = GaussianSourceDescriptor::Constant {
            color: Color::BLACK,
            filter: domain,
            result: domain,
        };
        let mapping = GaussianMapping::new(None, result, [None; 2]);
        assert!(
            !mapping
                .proves_exact(
                    &operation(1.0, 1.0),
                    source,
                    GaussianStage::Constant,
                    &account
                )
                .unwrap()
        );
        assert!(
            mapping
                .with_color(Color::BLACK)
                .proves_exact(
                    &operation(1.0, 1.0),
                    source,
                    GaussianStage::Constant,
                    &account
                )
                .unwrap()
        );
    }
    #[test]
    fn two_active_axes_keep_horizontal_vertical_halo_and_direct_vertical_reads() {
        let d = Rect::new(0.0, 0.0, 3.0, 1.0);
        let mut scene = Scene::new();
        scene
            .gaussian_layer(
                Layer::new(),
                operation(1.0, 1.0),
                GaussianSource::finite(tile(d), FilterDomains::try_new(d, d, d).unwrap()),
            )
            .unwrap();
        let graph = planned(&scene, 1.0);
        let stages = &graph.gaussians;
        assert_eq!(stages.len(), 4);
        assert_eq!(stages[0].facts.stage(), GaussianStage::Prepare);
        assert_eq!(stages[1].facts.mapping().target_extent(), [3, 9]);
        assert_eq!(stages[1].facts.mapping().source_base(), [0, -4]);
        assert!(!stages[1].facts.mapping().direct());
        assert_eq!(stages[2].facts.mapping().target_extent(), [3, 1]);
        assert_eq!(stages[2].facts.mapping().source_base(), [0, 4]);
        assert!(stages[2].facts.mapping().direct());
        assert_eq!(stages[3].facts.stage(), GaussianStage::Resolve);
        graph.lowering_view(&Account::for_test()).unwrap();
    }
    #[test]
    fn zero_axis_has_no_kernel_and_batch_gaps_fail_closed_lowering() {
        let d = Rect::new(0.0, 0.0, 3.0, 1.0);
        let mut scene = Scene::new();
        scene
            .gaussian_layer(
                Layer::new(),
                operation(4.0, 0.0),
                GaussianSource::finite(tile(d), FilterDomains::try_new(d, d, d).unwrap()),
            )
            .unwrap();
        let mut graph = planned(&scene, 1.0);
        assert!(
            graph.gaussians[0]
                .facts
                .mapping()
                .geometry(GaussianAxis::Y)
                .is_none()
        );
        assert_eq!(graph.gaussians.len(), 4);
        assert_eq!(
            graph.gaussians[2].facts.stage(),
            GaussianStage::Accumulate {
                axis: GaussianAxis::X,
                first_tap: 32,
                tap_count: 1,
                first: false
            }
        );
        let original = &graph.gaussians[2].facts;
        graph.gaussians[2].facts = crate::frame::GaussianPassFacts::try_new(
            crate::gaussian::GaussianForeground {
                operation: *original.operation(),
                source: original.source(),
            },
            original.mapping(),
            GaussianStage::Accumulate {
                axis: GaussianAxis::X,
                first_tap: 31,
                tap_count: 1,
                first: false,
            },
            &Account::for_test().storage(),
        )
        .unwrap();
        assert!(graph.lowering_view(&Account::for_test()).is_err());
    }
    #[test]
    fn periodic_capture_restores_root_density_before_the_next_period() {
        let mut scene = Scene::new();
        for period in [
            Rect::new(0.25, 0.0, 2.25, 1.25),
            Rect::new(0.0, 0.0, 4.25, 3.25),
        ] {
            scene
                .gaussian_layer(
                    Layer::new(),
                    operation(1.0, 0.0),
                    GaussianSource::periodic(
                        tile(period),
                        period,
                        Rect::new(0.0, 0.0, 8.0, 8.0),
                        Rect::new(0.0, 0.0, 8.0, 8.0),
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let graph = planned(&scene, 2.0);
        let prepares: Vec<_> = graph
            .gaussians
            .iter()
            .filter(|p| p.facts.stage() == GaussianStage::Prepare)
            .map(|p| p.facts.mapping().source_extent())
            .collect();
        assert_eq!(prepares, [[5, 3], [9, 7]]);
        graph.lowering_view(&Account::for_test()).unwrap();
    }
}

#[cfg(test)]
mod pass_storage_tests {
    use super::*;
    use crate::Color;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingError, RenderAccountingOptions, RenderCpuScope,
        RenderLimits, RenderQuantity, RenderRefusalReason, RenderResource, RendererCpu,
        StorageAccount,
    };
    use crate::renderer::Antialiasing;

    fn parts() -> (crate::gaussian::GaussianForeground, GaussianMapping) {
        let domain = Rect::new(0.0, 0.0, 1.0, 1.0);
        let operation = crate::prepare_gaussian(
            crate::GaussianInput::new(),
            crate::FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let context = crate::frame::FrameContext::try_new(
            crate::Size::new(1.0, 1.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let spatial =
            crate::frame::gaussian::require_spatial(context, domain, Transform::identity())
                .unwrap();
        (
            crate::gaussian::GaussianForeground {
                operation,
                source: crate::gaussian::GaussianSourceDescriptor::Constant {
                    color: Color::BLACK,
                    filter: domain,
                    result: domain,
                },
            },
            GaussianMapping::new(None, spatial, [None; 2]).with_color(Color::BLACK),
        )
    }
    fn retained(cpu: &RendererCpu) -> u64 {
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes)
    }

    #[test]
    fn gaussian_pass_payload_accepts_exact_bytes_and_clones_release_only_at_last_owner() {
        let baseline = retained(&RendererCpu::new(RenderAccountingOptions::default()).unwrap());
        let payload = SharedValue::<GaussianPassData>::allocation_size() as u64;
        let exact = baseline + payload;
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_retained_bytes(exact)),
        )
        .unwrap();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let (foreground, mapping) = parts();
        let value =
            GaussianPassFacts::try_new(foreground, mapping, GaussianStage::Constant, &storage)
                .unwrap();
        assert_eq!(retained(&cpu), exact);
        assert_eq!(value.operation(), &foreground.operation);
        assert_eq!(value.source(), foreground.source);
        assert_eq!(value.mapping(), mapping);
        assert_eq!(value.stage(), GaussianStage::Constant);
        assert!(
            value
                .mapping()
                .proves_exact(
                    value.operation(),
                    value.source(),
                    value.stage(),
                    &Account::for_test()
                )
                .unwrap()
        );
        let live = value.0.allocation_liveness_for_test();
        let first_view = value.clone();
        let last_view = first_view.clone();
        assert!(value.0.same_allocation(&first_view.0));
        assert!(value.0.same_allocation(&last_view.0));
        assert_eq!(retained(&cpu), exact);
        drop(value);
        drop(first_view);
        assert!(live());
        assert_eq!(retained(&cpu), exact);
        drop(last_view);
        assert!(!live());
        assert_eq!(retained(&cpu), baseline);
        assert_eq!(cpu.snapshot().unwrap().fault(), None);
    }

    #[test]
    fn gaussian_pass_payload_refuses_one_byte_under_without_retaining_a_partial_owner() {
        let baseline = retained(&RendererCpu::new(RenderAccountingOptions::default()).unwrap());
        let payload = SharedValue::<GaussianPassData>::allocation_size() as u64;
        let limit = baseline + payload - 1;
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_retained_bytes(limit)),
        )
        .unwrap();
        let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
        let (foreground, mapping) = parts();
        let error =
            GaussianPassFacts::try_new(foreground, mapping, GaussianStage::Constant, &storage)
                .unwrap_err();
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::CpuRefused {
                scope: RenderCpuScope::Root,
                quantity: RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
                current: baseline,
                amount: payload,
                reason: RenderRefusalReason::Limit { limit },
            })
        );
        assert_eq!(retained(&cpu), baseline);
        assert_eq!(cpu.snapshot().unwrap().fault(), None);
    }
}
