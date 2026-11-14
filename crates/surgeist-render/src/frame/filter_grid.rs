//! Shared native source grids, relative coverage, aligned rebasing and periodic reconstruction.
use super::{
    FrameContext, GraphLoweringSpatialDescriptor,
    bounds::{FrameSpatialPlan, LogicalBounds, NonEmptyFrameSpatialPlan},
    convolution::intersection,
    graph::{SemanticResourceDescriptor, SemanticResourceRole},
};
use crate::{
    Rect, Transform,
    accounting::{Account, RenderWork},
    error::{Error, Result},
    filter_source::FilterSourceDescriptor,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum FilterGridOwner {
    Gaussian,
    Morphology,
}
impl FilterGridOwner {
    fn mapping_error(self, value: f64) -> Error {
        match self {
            Self::Gaussian => crate::GaussianPreparationError::Numeric {
                phase: crate::GaussianNumericPhase::GridMapping,
                value,
            }
            .into(),
            Self::Morphology => crate::MorphologyPreparationError::Numeric {
                phase: crate::MorphologyNumericPhase::EndpointMapping,
                value,
            }
            .into(),
        }
    }
    fn index_error(self, value: f64) -> Error {
        match self {
            Self::Gaussian => crate::GaussianPreparationError::Numeric {
                phase: crate::GaussianNumericPhase::GridIndex,
                value,
            }
            .into(),
            Self::Morphology => crate::MorphologyPreparationError::Numeric {
                phase: crate::MorphologyNumericPhase::GridIndex,
                value,
            }
            .into(),
        }
    }
    fn sampling_error(self, value: f64) -> Error {
        match self {
            Self::Gaussian => self.mapping_error(value),
            Self::Morphology => crate::MorphologyPreparationError::Numeric {
                phase: crate::MorphologyNumericPhase::Sampling,
                value,
            }
            .into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FilterGridMapping {
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
    color: crate::Color,
    owner: FilterGridOwner,
}
impl FilterGridMapping {
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
    pub(super) fn new(
        source: Option<NonEmptyFrameSpatialPlan>,
        target: NonEmptyFrameSpatialPlan,
        owner: FilterGridOwner,
    ) -> Self {
        Self {
            source_spatial: source,
            target_spatial: target,
            source_base: [0; 2],
            source_clip: full_clip(source.map(extent).unwrap_or([1; 2])),
            output_clip: full_clip(extent(target)),
            periodic_base: [0.0; 2],
            periodic_step: [1.0; 2],
            periodic: false,
            direct: false,
            converted: false,
            color: crate::Color::TRANSPARENT,
            owner,
        }
    }
    pub(super) const fn source_plan(self) -> Option<NonEmptyFrameSpatialPlan> {
        self.source_spatial
    }
    pub(super) const fn target_plan(self) -> NonEmptyFrameSpatialPlan {
        self.target_spatial
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
                .ok_or_else(|| self.owner.index_error(base[axis] as f64))?;
            let high = base[axis]
                .checked_add(i64::from(size[axis]) - 1)
                .and_then(|n| n.checked_add(i64::from(radius[axis])))
                .ok_or_else(|| self.owner.index_error(base[axis] as f64))?;
            i32::try_from(low).map_err(|_| self.owner.index_error(low as f64))?;
            i32::try_from(high).map_err(|_| self.owner.index_error(high as f64))?;
            self.source_base[axis] =
                i32::try_from(base[axis]).map_err(|_| self.owner.index_error(base[axis] as f64))?;
        }
        Ok(self)
    }
    pub(super) fn with_source_clip(
        mut self,
        domain: Rect,
        spatial: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        self.source_clip = clip(domain, spatial, account, self.owner)?;
        Ok(self)
    }
    pub(super) fn with_output_clip(
        mut self,
        domain: Rect,
        spatial: NonEmptyFrameSpatialPlan,
        account: &Account,
    ) -> Result<Self> {
        self.output_clip = clip(domain, spatial, account, self.owner)?;
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
                return Err(self.owner.sampling_error(phase));
            }
            self.periodic_base[axis] =
                (phase * period.density[axis]).rem_euclid(f64::from(sizes[axis]));
            self.periodic_step[axis] = period.density[axis] / result.raster_scale.get();
            let b = uploaded(self.periodic_base[axis], self.owner)?;
            let step = uploaded(self.periodic_step[axis], self.owner)?;
            if step <= 0.0 || !step.is_normal() {
                return Err(self.owner.sampling_error(self.periodic_step[axis]));
            }
            // Exactly the multiply/add/subtract consumed by WGSL. Monotone
            // endpoints bound every interior invocation and both +1 neighbors.
            for p in [0.5_f32, self.target_extent()[axis] as f32 - 0.5] {
                let product = finite(p * step, self.owner)?;
                let shifted = finite(product + b, self.owner)?;
                let coordinate = finite(shifted - 0.5, self.owner)?;
                let floor = f64::from(coordinate.floor());
                if floor < f64::from(i32::MIN) || floor + 1.0 > f64::from(i32::MAX) {
                    return Err(self.owner.index_error(floor));
                }
                // Reconstruction needs a distinguishable fraction and next
                // integer; if f32 cannot represent it refuse that actual map.
                if coordinate.abs() >= 8_388_608.0 {
                    return Err(self.owner.sampling_error(f64::from(coordinate)));
                }
            }
        }
        self.periodic = true;
        Ok(self)
    }
}
pub(super) fn spatial_descriptor(
    spatial: NonEmptyFrameSpatialPlan,
) -> GraphLoweringSpatialDescriptor {
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
        Self::try_new_for(
            context,
            period,
            raster_transform,
            account,
            FilterGridOwner::Gaussian,
        )
    }
    pub(super) fn try_new_for(
        context: FrameContext,
        period: Rect,
        raster_transform: Transform,
        account: &Account,
        owner: FilterGridOwner,
    ) -> Result<Self> {
        let rho = context
            .local_raster_scale(raster_transform)?
            .ok_or_else(|| owner.mapping_error(0.0))?
            .get();
        let mut sizes = [0; 2];
        let mut density = [0.0; 2];
        for (axis, length) in [period.width(), period.height()].into_iter().enumerate() {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let size = (length * rho).ceil().max(1.0);
            if !size.is_finite() || size > 8_388_608.0 {
                return Err(owner.index_error(size));
            }
            sizes[axis] = size as u32;
            density[axis] = size / length;
            let d = uploaded(density[axis], owner)?;
            if !d.is_normal() || !uploaded(1.0 / density[axis], owner)?.is_normal() {
                return Err(owner.mapping_error(density[axis]));
            }
        }
        let spatial = allocation_spatial(context, sizes, owner)?;
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
            uploaded(value, owner)?;
        }
        let transform =
            Transform::try_new(coefficients).map_err(|_| owner.mapping_error(coefficients[4]))?;
        Ok(Self {
            period,
            spatial,
            context: scoped,
            transform,
            density,
        })
    }
}

pub(super) fn result_domain(source: FilterSourceDescriptor) -> Rect {
    intersection(source.filter(), source.result())
}
pub(super) fn finite_domain(source: FilterSourceDescriptor) -> Option<Rect> {
    match source {
        FilterSourceDescriptor::Finite { domains, .. } => {
            Some(intersection(domains.source(), domains.filter()))
        }
        _ => None,
    }
}
pub(super) fn source_is_empty(source: FilterSourceDescriptor) -> bool {
    finite_domain(source).is_some_and(empty)
}
pub(super) fn empty(rect: Rect) -> bool {
    rect.width() == 0.0 || rect.height() == 0.0
}
pub(super) fn allocation_spatial(
    context: FrameContext,
    size: [u32; 2],
    owner: FilterGridOwner,
) -> Result<NonEmptyFrameSpatialPlan> {
    let mut unit_context = context;
    unit_context.surface_scale = 1.0;
    require_spatial(
        unit_context,
        Rect::new(0.0, 0.0, f64::from(size[0]), f64::from(size[1])),
        Transform::identity(),
        owner,
    )
}
pub(super) fn require_spatial(
    context: FrameContext,
    rect: Rect,
    transform: Transform,
    owner: FilterGridOwner,
) -> Result<NonEmptyFrameSpatialPlan> {
    let plan = context
        .plan_local_bounds(
            LogicalBounds::try_from_rect(rect, "Gaussian domain")?,
            transform,
        )
        .map_err(|error| {
            if error.code() == crate::ErrorCode::InvalidInput {
                owner.mapping_error(
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
        return Err(owner.mapping_error(0.0));
    };
    for size in extent(spatial) {
        if size > 8_388_608 {
            return Err(owner.index_error(f64::from(size)));
        }
    }
    Ok(spatial)
}
pub(super) fn base_between(
    source: NonEmptyFrameSpatialPlan,
    target: NonEmptyFrameSpatialPlan,
    owner: FilterGridOwner,
) -> Result<[i64; 2]> {
    // This is a proof about raw aligned lattices, not rounded f32 origins.
    if source.raster_scale != target.raster_scale {
        return Err(owner.mapping_error(source.raster_scale.get()));
    }
    for spatial in [source, target] {
        if spatial.texel_center_mapping.origin.x()
            != f64::from(spatial.device_origin.x) / spatial.raster_scale.get()
            || spatial.texel_center_mapping.origin.y()
                != f64::from(spatial.device_origin.y) / spatial.raster_scale.get()
        {
            return Err(owner.mapping_error(spatial.texel_center_mapping.origin.x()));
        }
    }
    Ok([
        i64::from(target.device_origin.x) - i64::from(source.device_origin.x),
        i64::from(target.device_origin.y) - i64::from(source.device_origin.y),
    ])
}
fn clip(
    domain: Rect,
    spatial: NonEmptyFrameSpatialPlan,
    account: &Account,
    owner: FilterGridOwner,
) -> Result<[f64; 4]> {
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
            return Err(owner.mapping_error(value));
        }
        result[index] = value.clamp(0.0, f64::from(size[axis]));
        uploaded(result[index], owner)?;
    }
    for axis in 0..2 {
        if result[axis + 2] as f32 <= result[axis] as f32 {
            return Err(owner.mapping_error(result[axis + 2]));
        }
        for cell in [0.0_f32, (size[axis] - 1) as f32] {
            finite(
                (cell + 1.0).min(result[axis + 2] as f32) - cell.max(result[axis] as f32),
                owner,
            )?;
        }
    }
    Ok(result)
}
fn uploaded(value: f64, owner: FilterGridOwner) -> Result<f32> {
    let n = value as f32;
    if !n.is_finite() || (value != 0.0 && !n.is_normal()) {
        Err(owner.mapping_error(value))
    } else {
        Ok(n)
    }
}
fn finite(value: f32, owner: FilterGridOwner) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(owner.mapping_error(f64::from(value)))
    }
}
