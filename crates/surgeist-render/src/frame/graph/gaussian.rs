use super::*;
use crate::frame::gaussian::{
    self as grid, GaussianAxis, GaussianMapping, GaussianPurpose, GaussianStage, PeriodCapture,
};
use crate::{filter_source::FilterSourceDescriptor, gaussian::GaussianForeground};

impl SemanticFrameGraphPlanner<'_> {
    pub(super) fn apply_gaussian(
        &mut self,
        source: Option<PlannedGraphResource>,
        foreground: GaussianForeground,
        raster_transform: Transform,
    ) -> Result<Option<PlannedGraphResource>> {
        self.apply_gaussian_for_purpose(
            source,
            foreground,
            raster_transform,
            GaussianPurpose::Standalone,
        )
    }

    pub(super) fn apply_gaussian_for_purpose(
        &mut self,
        mut source: Option<PlannedGraphResource>,
        foreground: GaussianForeground,
        raster_transform: Transform,
        purpose: GaussianPurpose,
    ) -> Result<Option<PlannedGraphResource>> {
        self.account.work(RenderWork::FilterOperationVisits, 1)?;
        let output_domain = grid::result_domain(foreground.source);
        if grid::empty(output_domain) {
            return Ok(None);
        }
        let result = grid::require_spatial(self.context, output_domain, raster_transform)?;
        if matches!(foreground.source, FilterSourceDescriptor::Constant { .. })
            || grid::source_is_empty(foreground.source)
            || source.is_none()
        {
            let color = match foreground.source {
                FilterSourceDescriptor::Constant { color, .. } => color,
                _ => crate::Color::TRANSPARENT,
            };
            let mapping = GaussianMapping::new(None, result, [None; 2])
                .with_color(color)
                .with_output_clip(output_domain, result, self.account)?;
            return self
                .declare_gaussian_pass_for_purpose(
                    &[],
                    foreground,
                    mapping,
                    GaussianStage::Constant,
                    purpose,
                )
                .map(Some);
        }
        let mut captured = source.take().ok_or_else(|| {
            Error::new(BackendErrorCode::RenderFailed, "Gaussian source is absent")
        })?;
        let period = match foreground.source {
            FilterSourceDescriptor::Periodic { period, .. } => Some(PeriodCapture::try_new(
                self.context,
                period,
                raster_transform,
                self.account,
            )?),
            _ => None,
        };
        let required = match period {
            Some(period) => period.spatial,
            None => grid::require_spatial(
                self.context,
                grid::finite_domain(foreground.source).ok_or_else(|| {
                    Error::new(
                        BackendErrorCode::RenderFailed,
                        "finite Gaussian source lost its domain",
                    )
                })?,
                raster_transform,
            )?,
        };
        if captured.spatial != required {
            if period.is_some() {
                return Err(Error::new(
                    BackendErrorCode::RenderFailed,
                    "Gaussian period capture changed its canvas",
                ));
            }
            // A same-layer backdrop may have enlarged the raw capture. Restore
            // D on the exact device lattice before applying coverage or color.
            let mapping = GaussianMapping::new(Some(captured.spatial), required, [None; 2])
                .with_route(required, result, period)
                .with_base(
                    grid::base_between(captured.spatial, required)?,
                    [0; 2],
                    self.account,
                )?;
            captured = self.declare_gaussian_pass_for_purpose(
                &[captured],
                foreground,
                mapping,
                GaussianStage::RebaseSource,
                purpose,
            )?;
        }
        if foreground.operation.is_passthrough() {
            let mut mapping = GaussianMapping::new(Some(required), result, [None; 2])
                .with_route(required, result, period)
                .with_output_clip(output_domain, result, self.account)?;
            mapping = if let Some(period) = period {
                mapping.with_periodic_resolve(period, result, self.account)?
            } else {
                mapping
                    .with_base(grid::base_between(required, result)?, [0; 2], self.account)?
                    .with_source_clip(
                        grid::finite_domain(foreground.source).ok_or_else(|| {
                            Error::new(
                                BackendErrorCode::RenderFailed,
                                "finite Gaussian source lost its domain",
                            )
                        })?,
                        required,
                        self.account,
                    )?
            };
            return self
                .declare_gaussian_pass_for_purpose(
                    &[captured],
                    foreground,
                    mapping,
                    GaussianStage::PassThrough,
                    purpose,
                )
                .map(Some);
        }
        let density = period
            .map(|p| p.density)
            .unwrap_or([required.raster_scale.get(); 2]);
        let geometries = grid::kernel_geometries(foreground.operation, density, self.account)?;
        let prepared_spatial = grid::allocation_spatial(self.context, grid::extent(required))?;
        let mut prepare = GaussianMapping::new(Some(required), prepared_spatial, geometries)
            .with_route(required, result, period)
            .with_flags(period.is_some(), false, false);
        if let Some(domain) = grid::finite_domain(foreground.source) {
            prepare = prepare.with_source_clip(domain, required, self.account)?;
        }
        let prepared = self.declare_gaussian_pass_for_purpose(
            &[captured],
            foreground,
            prepare,
            GaussianStage::Prepare,
            purpose,
        )?;
        let final_grid = if period.is_some() {
            required
        } else {
            grid::allocation_spatial(self.context, grid::extent(result))?
        };
        let mut current = prepared;
        for axis in [GaussianAxis::X, GaussianAxis::Y] {
            let Some(geometry) = geometries[axis.index()] else {
                continue;
            };
            let both = geometries[0].is_some() && geometries[1].is_some();
            let target = if period.is_some() {
                required
            } else if axis == GaussianAxis::X && both {
                let radius = geometries[1].map(|g| g.support_radius()).unwrap_or(0);
                let size = grid::extent(result);
                let height = u64::from(size[1]) + 2 * u64::from(radius);
                if height > 8_388_608 {
                    return Err(crate::GaussianPreparationError::Numeric {
                        phase: crate::GaussianNumericPhase::GridIndex,
                        value: height as f64,
                    }
                    .into());
                }
                grid::allocation_spatial(self.context, [size[0], height as u32])?
            } else {
                final_grid
            };
            let direct = period.is_none() && axis == GaussianAxis::Y && both;
            let base = if period.is_some() {
                [0; 2]
            } else if direct {
                [
                    0,
                    i64::from(geometries[1].map(|g| g.support_radius()).unwrap_or(0)),
                ]
            } else {
                let mut base = grid::base_between(required, result)?;
                if axis == GaussianAxis::X && both {
                    base[1] -= i64::from(geometries[1].map(|g| g.support_radius()).unwrap_or(0));
                }
                base
            };
            let mut radii = [0; 2];
            radii[axis.index()] = geometry.support_radius();
            let mapping = GaussianMapping::new(Some(current.spatial), target, geometries)
                .with_route(required, result, period)
                .with_flags(period.is_some(), direct, false)
                .with_base(base, radii, self.account)?;
            let axis_source = current;
            let mut previous = None;
            let mut first_tap = 0;
            while first_tap < geometry.sample_count() {
                self.account.work(RenderWork::FilterOperationVisits, 1)?;
                let tap_count = (geometry.sample_count() - first_tap).min(32);
                let inputs = [axis_source, previous.unwrap_or(axis_source)];
                let reads = if previous.is_none() {
                    &inputs[..1]
                } else {
                    &inputs[..]
                };
                previous = Some(self.declare_gaussian_pass_for_purpose(
                    reads,
                    foreground,
                    mapping,
                    GaussianStage::Accumulate {
                        axis,
                        first_tap,
                        tap_count,
                        first: previous.is_none(),
                    },
                    purpose,
                )?);
                first_tap += tap_count;
            }
            current = previous.ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "Gaussian axis has no coefficients",
                )
            })?;
        }
        let mut resolve = GaussianMapping::new(Some(current.spatial), result, geometries)
            .with_route(required, result, period)
            .with_flags(period.is_some(), false, true)
            .with_output_clip(output_domain, result, self.account)?;
        if let Some(period) = period {
            resolve = resolve.with_periodic_resolve(period, result, self.account)?;
        }
        self.declare_gaussian_pass_for_purpose(
            &[current],
            foreground,
            resolve,
            GaussianStage::Resolve,
            purpose,
        )
        .map(Some)
    }

    pub(super) fn declare_gaussian_pass(
        &mut self,
        reads: &[PlannedGraphResource],
        foreground: GaussianForeground,
        mapping: GaussianMapping,
        stage: GaussianStage,
    ) -> Result<PlannedGraphResource> {
        self.declare_gaussian_pass_for_purpose(
            reads,
            foreground,
            mapping,
            stage,
            GaussianPurpose::Standalone,
        )
    }

    pub(super) fn declare_gaussian_pass_for_purpose(
        &mut self,
        reads: &[PlannedGraphResource],
        foreground: GaussianForeground,
        mapping: GaussianMapping,
        stage: GaussianStage,
        purpose: GaussianPurpose,
    ) -> Result<PlannedGraphResource> {
        // Admit this immutable payload exactly once before publishing any
        // resource or pass record. Lowering and runtime retain its existing
        // owner; the last clone refunds the actual shared allocation.
        let facts = match purpose {
            GaussianPurpose::Standalone => crate::frame::GaussianPassFacts::try_new(
                foreground,
                mapping,
                stage,
                &self.account.storage(),
            ),
            GaussianPurpose::ConsumedAlpha => crate::frame::GaussianPassFacts::try_new_for_purpose(
                foreground,
                mapping,
                stage,
                purpose,
                &self.account.storage(),
            ),
        }?;
        self.gaussians.grow(1)?;
        let spatial = mapping.target_plan();
        let role = if matches!(
            stage,
            GaussianStage::Prepare | GaussianStage::Accumulate { .. }
        ) {
            SemanticResourceRole::FilterFloat
        } else {
            SemanticResourceRole::FilterIntermediate
        };
        let result = graph_build(
            self.builder
                .declare_resource(SemanticResourceDescriptor::new(role, spatial, 0)),
        )?;
        let mut ids = self.account.temporary(reads.len())?;
        for read in reads {
            ids.append_slot()?.write(read.id);
        }
        let pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::Gaussian(stage),
            &dependencies_for(reads, self.account)?,
            &ids,
            SemanticPassResult::Resource(result),
        ))?;
        self.gaussians.push(SemanticGaussianPlan { pass, facts })?;
        Ok(PlannedGraphResource {
            id: result,
            producer: Some(pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        })
    }
}
