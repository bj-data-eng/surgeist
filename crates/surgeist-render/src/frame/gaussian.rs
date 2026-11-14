//! Native Gaussian grids. All shader coordinates are relative to integer
//! allocations; the full authored affine belongs to final composition only.
use super::{
    FrameContext, GraphLoweringSpatialDescriptor,
    bounds::{LogicalBounds, NonEmptyFrameSpatialPlan},
};
use crate::{
    Rect, Transform,
    accounting::{Account, RenderWork, SharedValue},
    error::{Error, Result},
    filter_source::FilterSourceDescriptor,
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
    source: crate::filter_source::FilterSourceDescriptor,
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
    pub(crate) fn source(&self) -> crate::filter_source::FilterSourceDescriptor {
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
    grid: super::filter_grid::FilterGridMapping,
    geometries: [Option<GaussianKernelGeometry>; 2],
    original_source: Option<NonEmptyFrameSpatialPlan>,
    result: NonEmptyFrameSpatialPlan,
    period: Option<PeriodCapture>,
}
impl GaussianMapping {
    pub(crate) fn source_spatial(self) -> Option<GraphLoweringSpatialDescriptor> {
        self.grid.source_spatial()
    }
    pub(crate) fn target_spatial(self) -> GraphLoweringSpatialDescriptor {
        self.grid.target_spatial()
    }
    pub(crate) fn source_extent(self) -> [u32; 2] {
        self.grid.source_extent()
    }
    pub(crate) const fn target_extent(self) -> [u32; 2] {
        self.grid.target_extent()
    }
    pub(crate) const fn color(self) -> crate::Color {
        self.grid.color()
    }
    pub(crate) const fn source_base(self) -> [i32; 2] {
        self.grid.source_base()
    }
    pub(crate) const fn source_clip(self) -> [f64; 4] {
        self.grid.source_clip()
    }
    pub(crate) const fn output_clip(self) -> [f64; 4] {
        self.grid.output_clip()
    }
    pub(crate) const fn periodic_base(self) -> [f64; 2] {
        self.grid.periodic_base()
    }
    pub(crate) const fn periodic_step(self) -> [f64; 2] {
        self.grid.periodic_step()
    }
    pub(crate) const fn periodic(self) -> bool {
        self.grid.periodic()
    }
    pub(crate) const fn direct(self) -> bool {
        self.grid.direct()
    }
    pub(crate) const fn converted(self) -> bool {
        self.grid.converted()
    }
    pub(super) fn output_footprint(self) -> super::FilterOutputFootprint {
        self.grid.output_footprint()
    }
    pub(super) const fn with_color(mut self, color: crate::Color) -> Self {
        self.grid = self.grid.with_color(color);
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
        Self {
            grid: super::filter_grid::FilterGridMapping::new(
                source,
                target,
                super::filter_grid::FilterGridOwner::Gaussian,
            ),
            geometries,
            original_source: source,
            result: target,
            period: None,
        }
    }
    pub(super) fn with_base(
        mut self,
        base: [i64; 2],
        radius: [u32; 2],
        account: &Account,
    ) -> Result<Self> {
        self.grid = self.grid.with_base(base, radius, account)?;
        Ok(self)
    }
    pub(super) fn with_source_clip(
        mut self,
        domain: Rect,
        spatial: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        self.grid = self.grid.with_source_clip(domain, spatial, account)?;
        Ok(self)
    }
    pub(super) fn with_output_clip(
        mut self,
        domain: Rect,
        spatial: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        self.grid = self.grid.with_output_clip(domain, spatial, account)?;
        Ok(self)
    }
    pub(super) const fn with_flags(
        mut self,
        periodic: bool,
        direct: bool,
        converted: bool,
    ) -> Self {
        self.grid = self.grid.with_flags(periodic, direct, converted);
        self
    }
    pub(super) fn with_periodic_resolve(
        mut self,
        period: PeriodCapture,
        result: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        self.grid = self.grid.with_periodic_resolve(period, result, account)?;
        Ok(self)
    }
    /// Re-establish the concrete private mapping at closure, under its actual
    /// source declaration and operation. No graph scan or coefficient work.
    pub(crate) fn proves_exact(
        self,
        operation: &crate::PreparedGaussian,
        source: FilterSourceDescriptor,
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
                FilterSourceDescriptor::Constant { color, .. } => color,
                _ => crate::Color::TRANSPARENT,
            };
            let expected = Self::new(None, self.result, [None; 2])
                .with_color(self.grid.color())
                .with_output_clip(output, self.result, account)?;
            return Ok(self == expected && self.grid.color() == color);
        }
        let Some(original) = self.original_source else {
            return Ok(false);
        };
        let expected_period = match source {
            FilterSourceDescriptor::Periodic { period, .. } => Some(PeriodCapture::try_new(
                context,
                period,
                Transform::identity(),
                account,
            )?),
            FilterSourceDescriptor::Finite { .. } => None,
            FilterSourceDescriptor::Constant { .. } => return Ok(false),
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
                let Some(raw) = self.grid.source_plan() else {
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
        self.grid.target_plan()
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
pub(super) use super::filter_grid::{
    PeriodCapture, empty, extent, finite_domain, result_domain, source_is_empty,
};
pub(super) fn allocation_spatial(
    context: FrameContext,
    size: [u32; 2],
) -> Result<NonEmptyFrameSpatialPlan> {
    super::filter_grid::allocation_spatial(
        context,
        size,
        super::filter_grid::FilterGridOwner::Gaussian,
    )
}
pub(super) fn require_spatial(
    context: FrameContext,
    rect: Rect,
    transform: Transform,
) -> Result<NonEmptyFrameSpatialPlan> {
    super::filter_grid::require_spatial(
        context,
        rect,
        transform,
        super::filter_grid::FilterGridOwner::Gaussian,
    )
}
pub(super) fn base_between(
    source: NonEmptyFrameSpatialPlan,
    target: NonEmptyFrameSpatialPlan,
) -> Result<[i64; 2]> {
    super::filter_grid::base_between(
        source,
        target,
        super::filter_grid::FilterGridOwner::Gaussian,
    )
}
fn mapping_error(value: f64) -> Error {
    crate::GaussianPreparationError::Numeric {
        phase: crate::GaussianNumericPhase::GridMapping,
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
        let source = FilterSourceDescriptor::Constant {
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
                source: crate::filter_source::FilterSourceDescriptor::Constant {
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
