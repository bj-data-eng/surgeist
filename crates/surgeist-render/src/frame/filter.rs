use super::{
    FrameContext,
    bounds::{
        FrameSpatialPlan, LogicalBounds, NonEmptyFrameSpatialPlan, NonEmptyLogicalBounds,
        RasterScale, SemanticSourceBounds, SemanticSourceContribution, checked_div, checked_mul,
    },
};
use crate::accounting::{Account, RenderPhase, RenderWork, SharedValue};
#[cfg(test)]
use crate::style::FilterList;
use crate::{
    command::NormalizedLayer,
    error::{Error, Result},
    filter::{
        AlgorithmColorFilterRun, AlgorithmFilterPlan, AlgorithmFilterStep,
        CSS_FILTER_KERNEL_SUPPORT_STANDARD_DEVIATIONS,
    },
    geometry::{Point, Transform},
    style::{FilterBlur, FilterDropShadow, FilterOp, FilterOpKind},
};

impl FrameContext {
    pub(super) fn plan_filter_operations(
        self,
        source_bounds: LogicalBounds,
        transform: Transform,
        operations: &[FilterOp],
        source_role: FilterSourceRole,
        account: &Account,
    ) -> Result<ResolvedFrameFilterPlan> {
        let account = account.at(RenderPhase::Filter);
        let algorithm = AlgorithmFilterPlan::from_ops(operations, &account)?;
        account.phase(RenderPhase::Filter, || {
            let initial_spatial = self.plan_local_bounds(source_bounds, transform)?;
            let FrameSpatialPlan::NonEmpty(initial_spatial) = initial_spatial else {
                return Ok(ResolvedFrameFilterPlan::Empty(EmptyResolvedFilterPlan {
                    source_bounds,
                    authored_operation_count: algorithm.authored_operation_count(),
                }));
            };

            let initial_bounds = initial_spatial.logical_bounds;
            let raster_scale = initial_spatial.raster_scale;
            let mut current_bounds = initial_bounds;
            let mut steps = account.retained(algorithm.steps().len())?;

            for algorithm_step in algorithm.steps().iter().cloned() {
                account.work(RenderWork::FilterOperationVisits, 1)?;
                let source_bounds = current_bounds;
                let (result_bounds, edge_policy, operation_intent) = match algorithm_step {
                    AlgorithmFilterStep::ColorRun(run) => (
                        source_bounds,
                        FilterEdgePolicy::NoSampling,
                        ResolvedFilterOperationIntent::ColorRun(run),
                    ),
                    AlgorithmFilterStep::Blur(blur) => {
                        let support = InclusiveFilterKernelSupport::try_new(blur, raster_scale)?;
                        if support.device_radius == 0 {
                            // A positive authored sigma can round to zero on the device grid.
                            continue;
                        }
                        let result_bounds = support.inflate(source_bounds)?;
                        let edge_policy = match source_role {
                            FilterSourceRole::Ordinary => FilterEdgePolicy::TransparentBlack,
                            FilterSourceRole::Backdrop => FilterEdgePolicy::SemanticBorderMirror {
                                semantic_border: initial_bounds,
                            },
                        };
                        (
                            result_bounds,
                            edge_policy,
                            ResolvedFilterOperationIntent::Blur(ResolvedBlurIntent {
                                authored_blur: blur,
                                support,
                            }),
                        )
                    }
                    AlgorithmFilterStep::DropShadow(shadow) => {
                        let support =
                            InclusiveFilterKernelSupport::try_new(shadow.blur(), raster_scale)?;
                        let result_bounds =
                            support.include_shadow(source_bounds, shadow.offset())?;
                        let edge_policy = match source_role {
                            FilterSourceRole::Ordinary => FilterEdgePolicy::TransparentBlack,
                            FilterSourceRole::Backdrop => FilterEdgePolicy::SemanticBorderMirror {
                                semantic_border: initial_bounds,
                            },
                        };
                        (
                            result_bounds,
                            edge_policy,
                            ResolvedFilterOperationIntent::DropShadow(ResolvedDropShadowIntent {
                                authored_shadow: shadow,
                                alpha_source: DropShadowAlphaSource::SourceAlpha,
                                support,
                                offset_sampling: DropShadowOffsetSampling::ContinuousLinear,
                                source_composition:
                                    DropShadowSourceComposition::RetainUnchangedForSourceOver,
                            }),
                        )
                    }
                };
                let spatial_mapping = ResolvedFilterSpatialMapping {
                    source: self.plan_non_empty_local_bounds(source_bounds, transform)?,
                    result: self.plan_non_empty_local_bounds(result_bounds, transform)?,
                };
                steps.append_slot()?.write(ResolvedFilterStep {
                    source_bounds,
                    result_bounds,
                    spatial_mapping,
                    edge_policy,
                    operation_intent,
                });
                current_bounds = result_bounds;
            }

            Ok(ResolvedFrameFilterPlan::NonEmpty(
                NonEmptyResolvedFilterPlan {
                    initial_bounds,
                    final_bounds: current_bounds,
                    authored_operation_count: algorithm.authored_operation_count(),
                    output_is_always_transparent: algorithm.output_is_always_transparent(),
                    steps: SharedValue::from_accounted(steps, &account.storage())?,
                },
            ))
        })
    }

    fn plan_non_empty_local_bounds(
        self,
        logical_bounds: NonEmptyLogicalBounds,
        transform: Transform,
    ) -> Result<NonEmptyFrameSpatialPlan> {
        match self.plan_local_bounds(LogicalBounds::NonEmpty(logical_bounds), transform)? {
            FrameSpatialPlan::NonEmpty(plan) => Ok(plan),
            FrameSpatialPlan::Empty(_) => Err(Error::invalid_value(
                "filter step spatial mapping",
                "empty",
                "must remain non-empty after the frame transform was validated",
            )),
        }
    }
}

pub(super) fn foreground_filter_bounds(
    mut bounds: NonEmptyLogicalBounds,
    scale: RasterScale,
    operations: &[FilterOp],
    account: &Account,
) -> Result<NonEmptyLogicalBounds> {
    let account = account.at(RenderPhase::Filter);
    account.phase(RenderPhase::Filter, || {
        for operation in operations {
            account.work(RenderWork::FilterOperationVisits, 1)?;
            bounds = match operation.kind() {
                FilterOpKind::Blur(blur) => {
                    InclusiveFilterKernelSupport::try_new(*blur, scale)?.inflate(bounds)?
                }
                FilterOpKind::DropShadow(shadow) => {
                    InclusiveFilterKernelSupport::try_new(shadow.blur(), scale)?
                        .include_shadow(bounds, shadow.offset())?
                }
                FilterOpKind::Brightness(_)
                | FilterOpKind::Contrast(_)
                | FilterOpKind::Grayscale(_)
                | FilterOpKind::HueRotate(_)
                | FilterOpKind::Invert(_)
                | FilterOpKind::Opacity(_)
                | FilterOpKind::Saturate(_)
                | FilterOpKind::Sepia(_) => bounds,
            };
        }
        Ok(bounds)
    })
}

impl SemanticSourceContribution {
    pub(super) fn include_backdrop_contribution(
        mut source_bounds: SemanticSourceBounds,
        current_parent: SemanticSourceBounds,
        layer: &mut NormalizedLayer,
        context: FrameContext,
        layer_to_surface: Transform,
        account: &Account,
    ) -> Result<SemanticSourceBounds> {
        let Some(backdrop) = layer.backdrop.as_ref() else {
            return Ok(source_bounds);
        };
        let capture_bounds = LogicalBounds::try_from_rect(
            backdrop.capture_bounds().rect(),
            "backdrop capture bounds",
        )?;
        let resolved_filter_plan = context.plan_filter_operations(
            capture_bounds,
            layer_to_surface,
            backdrop.filters().ops(),
            FilterSourceRole::Backdrop,
            account,
        )?;
        let (mut backdrop_contribution, output_is_always_transparent) = match resolved_filter_plan {
            ResolvedFrameFilterPlan::Empty(_) => (SemanticSourceBounds::exactly_empty(), true),
            ResolvedFrameFilterPlan::NonEmpty(plan) => (
                SemanticSourceBounds::exact_known(plan.final_bounds),
                plan.output_is_always_transparent,
            ),
        };
        if let Some(clip) = backdrop.clip() {
            let clip_bounds = SemanticSourceBounds::try_for_clip(clip, account)?;
            backdrop_contribution = backdrop_contribution
                .try_intersect(clip_bounds, "post-filter backdrop clip intersection")?;
        }
        let captured_parent = current_parent.try_intersect(
            SemanticSourceBounds::from_logical_bounds(capture_bounds),
            "backdrop current-parent intersection",
        )?;
        if captured_parent.is_exactly_empty()
            || output_is_always_transparent
            || backdrop_contribution.is_exactly_empty()
        {
            layer.backdrop = None;
        } else {
            source_bounds = source_bounds.try_union(backdrop_contribution)?;
        }
        Ok(source_bounds)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FilterSourceRole {
    Ordinary,
    Backdrop,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ResolvedFrameFilterPlan {
    Empty(EmptyResolvedFilterPlan),
    NonEmpty(NonEmptyResolvedFilterPlan),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EmptyResolvedFilterPlan {
    pub(super) source_bounds: LogicalBounds,
    pub(super) authored_operation_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NonEmptyResolvedFilterPlan {
    pub(super) initial_bounds: NonEmptyLogicalBounds,
    pub(super) final_bounds: NonEmptyLogicalBounds,
    pub(super) authored_operation_count: usize,
    output_is_always_transparent: bool,
    pub(super) steps: SharedValue<Vec<ResolvedFilterStep>>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ResolvedFilterStep {
    pub(super) source_bounds: NonEmptyLogicalBounds,
    pub(super) result_bounds: NonEmptyLogicalBounds,
    pub(super) spatial_mapping: ResolvedFilterSpatialMapping,
    pub(super) edge_policy: FilterEdgePolicy,
    pub(super) operation_intent: ResolvedFilterOperationIntent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResolvedFilterSpatialMapping {
    pub(super) source: NonEmptyFrameSpatialPlan,
    pub(super) result: NonEmptyFrameSpatialPlan,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum FilterEdgePolicy {
    NoSampling,
    TransparentBlack,
    SemanticBorderMirror {
        semantic_border: NonEmptyLogicalBounds,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ResolvedFilterOperationIntent {
    ColorRun(AlgorithmColorFilterRun),
    Blur(ResolvedBlurIntent),
    DropShadow(ResolvedDropShadowIntent),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResolvedBlurIntent {
    pub(super) authored_blur: FilterBlur,
    pub(super) support: InclusiveFilterKernelSupport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResolvedDropShadowIntent {
    pub(super) authored_shadow: FilterDropShadow,
    pub(super) alpha_source: DropShadowAlphaSource,
    pub(super) support: InclusiveFilterKernelSupport,
    pub(super) offset_sampling: DropShadowOffsetSampling,
    pub(super) source_composition: DropShadowSourceComposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DropShadowAlphaSource {
    SourceAlpha,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DropShadowOffsetSampling {
    ContinuousLinear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DropShadowSourceComposition {
    RetainUnchangedForSourceOver,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct InclusiveFilterKernelSupport {
    pub(super) device_radius: u32,
    logical_radius: f64,
}

impl InclusiveFilterKernelSupport {
    pub(super) fn try_new(blur: FilterBlur, raster_scale: RasterScale) -> Result<Self> {
        let scaled_standard_deviation = checked_mul(
            blur.radius(),
            raster_scale.get(),
            "filter blur scaled standard deviation",
        )?;
        let inclusive_radius = checked_mul(
            CSS_FILTER_KERNEL_SUPPORT_STANDARD_DEVIATIONS,
            scaled_standard_deviation,
            "filter blur inclusive support",
        )?
        .ceil();
        if inclusive_radius < 0.0 || inclusive_radius > f64::from(u32::MAX) {
            return Err(Error::invalid_value(
                "filter blur inclusive support",
                inclusive_radius,
                "must fit in u32 device taps",
            ));
        }
        let device_radius = inclusive_radius as u32;
        let logical_radius = checked_div(
            inclusive_radius,
            raster_scale.get(),
            "filter blur logical support",
        )?;
        Ok(Self {
            device_radius,
            logical_radius,
        })
    }

    pub(super) fn inflate(self, bounds: NonEmptyLogicalBounds) -> Result<NonEmptyLogicalBounds> {
        bounds.try_inflate_uniform(self.logical_radius, "filter blur result bounds")
    }

    fn include_shadow(
        self,
        bounds: NonEmptyLogicalBounds,
        offset: Point,
    ) -> Result<NonEmptyLogicalBounds> {
        let shadow = bounds
            .try_inflate_uniform(self.logical_radius, "filter drop-shadow alpha bounds")?
            .try_translate(offset, "filter drop-shadow offset alpha bounds")?;
        bounds.try_union(shadow, "filter drop-shadow result bounds")
    }
}

#[cfg(test)]
use crate::{filter::ColorClampBoundary, geometry::Rect, style::ColorFilterOp};

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum OrderedFilterEdgeObservation {
    NoSampling,
    TransparentBlack,
    SemanticBorderMirror([f64; 4]),
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum OrderedFilterIntentObservation {
    ColorRun {
        operations: Vec<ColorFilterOp>,
        clamp_boundaries_after_operation: Vec<usize>,
    },
    Blur {
        standard_deviation: f64,
        inclusive_support_taps: u32,
    },
    DropShadow {
        offset: (f64, f64),
        standard_deviation: f64,
        inclusive_support_taps: u32,
        uses_source_alpha: bool,
        retains_unchanged_source: bool,
        continuous_offset: bool,
    },
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedFilterStepObservation {
    pub(crate) source_bounds: [f64; 4],
    pub(crate) result_bounds: Option<[f64; 4]>,
    pub(crate) source_device_origin: Option<(i32, i32)>,
    pub(crate) source_device_extent: Option<(u32, u32)>,
    pub(crate) result_device_origin: Option<(i32, i32)>,
    pub(crate) result_device_extent: Option<(u32, u32)>,
    pub(crate) edge: OrderedFilterEdgeObservation,
    pub(crate) intent: OrderedFilterIntentObservation,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedFilterPlanObservation {
    pub(crate) initial_bounds: [f64; 4],
    pub(crate) final_bounds: [f64; 4],
    pub(crate) authored_operation_count: usize,
    pub(crate) is_empty: bool,
    pub(crate) has_spatial_mapping: bool,
    pub(crate) steps: Vec<OrderedFilterStepObservation>,
}

#[cfg(test)]
pub(crate) fn ordered_filter_plan_for_test(
    filters: &FilterList,
    source_rect: Rect,
    transform: Transform,
    surface_scale: f64,
    backdrop: bool,
) -> Result<OrderedFilterPlanObservation> {
    let source_bounds = LogicalBounds::try_from_rect(source_rect, "filter plan source bounds")?;
    let source_role = if backdrop {
        FilterSourceRole::Backdrop
    } else {
        FilterSourceRole::Ordinary
    };
    let plan = FrameContext::try_for_spatial_test(surface_scale)?.plan_filter_operations(
        source_bounds,
        transform,
        filters.ops(),
        source_role,
        &Account::for_test(),
    )?;

    match plan {
        ResolvedFrameFilterPlan::Empty(plan) => {
            let bounds = logical_rect_values(plan.source_bounds.rect());
            Ok(OrderedFilterPlanObservation {
                initial_bounds: bounds,
                final_bounds: bounds,
                authored_operation_count: plan.authored_operation_count,
                is_empty: true,
                has_spatial_mapping: false,
                steps: Vec::new(),
            })
        }
        ResolvedFrameFilterPlan::NonEmpty(plan) => {
            let steps = plan
                .steps
                .iter()
                .cloned()
                .map(observe_resolved_filter_step)
                .collect();
            Ok(OrderedFilterPlanObservation {
                initial_bounds: logical_rect_values(plan.initial_bounds.rect()),
                final_bounds: logical_rect_values(plan.final_bounds.rect()),
                authored_operation_count: plan.authored_operation_count,
                is_empty: false,
                has_spatial_mapping: true,
                steps,
            })
        }
    }
}

#[cfg(test)]
fn observe_resolved_filter_step(step: ResolvedFilterStep) -> OrderedFilterStepObservation {
    let source_device_origin = Some((
        step.spatial_mapping.source.device_origin.x,
        step.spatial_mapping.source.device_origin.y,
    ));
    let source_device_extent = Some((
        step.spatial_mapping.source.device_extent.width,
        step.spatial_mapping.source.device_extent.height,
    ));
    let result_device_origin = Some((
        step.spatial_mapping.result.device_origin.x,
        step.spatial_mapping.result.device_origin.y,
    ));
    let result_device_extent = Some((
        step.spatial_mapping.result.device_extent.width,
        step.spatial_mapping.result.device_extent.height,
    ));
    let edge = match step.edge_policy {
        FilterEdgePolicy::NoSampling => OrderedFilterEdgeObservation::NoSampling,
        FilterEdgePolicy::TransparentBlack => OrderedFilterEdgeObservation::TransparentBlack,
        FilterEdgePolicy::SemanticBorderMirror { semantic_border } => {
            OrderedFilterEdgeObservation::SemanticBorderMirror(logical_rect_values(
                semantic_border.rect(),
            ))
        }
    };
    let intent = match step.operation_intent {
        ResolvedFilterOperationIntent::ColorRun(run) => {
            let operations = run
                .operations()
                .iter()
                .copied()
                .map(|operation| operation.operation())
                .collect();
            let clamp_boundaries_after_operation = run
                .operations()
                .iter()
                .copied()
                .enumerate()
                .filter_map(|(index, operation)| {
                    (operation.clamp_boundary()
                        == ColorClampBoundary::ClampStraightRgbaToUnitThenPremultiply)
                        .then_some(index)
                })
                .collect();
            OrderedFilterIntentObservation::ColorRun {
                operations,
                clamp_boundaries_after_operation,
            }
        }
        ResolvedFilterOperationIntent::Blur(intent) => OrderedFilterIntentObservation::Blur {
            standard_deviation: intent.authored_blur.radius(),
            inclusive_support_taps: intent.support.device_radius,
        },
        ResolvedFilterOperationIntent::DropShadow(intent) => {
            let offset = intent.authored_shadow.offset();
            OrderedFilterIntentObservation::DropShadow {
                offset: (offset.x(), offset.y()),
                standard_deviation: intent.authored_shadow.blur().radius(),
                inclusive_support_taps: intent.support.device_radius,
                uses_source_alpha: intent.alpha_source == DropShadowAlphaSource::SourceAlpha,
                retains_unchanged_source: intent.source_composition
                    == DropShadowSourceComposition::RetainUnchangedForSourceOver,
                continuous_offset: intent.offset_sampling
                    == DropShadowOffsetSampling::ContinuousLinear,
            }
        }
    };

    OrderedFilterStepObservation {
        source_bounds: logical_rect_values(step.source_bounds.rect()),
        result_bounds: Some(logical_rect_values(step.result_bounds.rect())),
        source_device_origin,
        source_device_extent,
        result_device_origin,
        result_device_extent,
        edge,
        intent,
    }
}

#[cfg(test)]
fn logical_rect_values(rect: Rect) -> [f64; 4] {
    [rect.x(), rect.y(), rect.width(), rect.height()]
}
