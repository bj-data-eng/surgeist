use super::*;
use crate::{
    filter_source::FilterSourceDescriptor,
    frame::gaussian::{self as grid, PeriodCapture},
};

impl SemanticFrameGraphPlanner<'_> {
    pub(super) fn prepare_filter_source(
        &mut self,
        children: RenderCommands,
        source: FilterSourceDescriptor,
        raster_transform: Transform,
    ) -> Result<PreparedLayerSource> {
        if grid::empty(grid::result_domain(source))
            || grid::source_is_empty(source)
            || children.is_empty()
        {
            return Ok(ControlFlow::Break(None));
        }
        let (spatial, context, capture_transform, parent_to_surface) = match source {
            FilterSourceDescriptor::Constant { .. } => return Ok(ControlFlow::Break(None)),
            FilterSourceDescriptor::Finite { .. } => (
                grid::require_spatial(
                    self.context,
                    grid::finite_domain(source).ok_or_else(|| {
                        Error::new(
                            BackendErrorCode::RenderFailed,
                            "finite Gaussian source lost its domain",
                        )
                    })?,
                    raster_transform,
                )?,
                self.context,
                Transform::identity(),
                raster_transform,
            ),
            FilterSourceDescriptor::Periodic { period, .. } => {
                let period =
                    PeriodCapture::try_new(self.context, period, raster_transform, self.account)?;
                (
                    period.spatial,
                    period.context,
                    period.transform,
                    Transform::identity(),
                )
            }
        };
        let parent = self.declare_transparent_parent(spatial)?;
        Ok(ControlFlow::Continue(self.command_frame(
            children,
            parent,
            SemanticCommandPlanningState {
                context,
                parent_locality: CaptureParentLocality::CaptureLocal,
                span_scope: SemanticVelloSpanScope::LayerSource,
                capture_transform,
                parent_to_surface,
                capture_space: CaptureBoundsCoordinateSpace::Local,
                outer_clips: shared_copy(&[], self.account)?,
            },
        )?))
    }
}
