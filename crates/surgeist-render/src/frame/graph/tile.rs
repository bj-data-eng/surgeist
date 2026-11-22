use super::*;
use crate::frame::tile::{TileCapture, TilePlan};

impl SemanticFrameGraphPlanner<'_> {
    pub(super) fn prepare_tile_source(
        &mut self,
        children: RenderCommands,
        capture: Option<TileCapture>,
    ) -> Result<PreparedLayerSource> {
        let Some(capture) = capture else {
            return Ok(ControlFlow::Break(None));
        };
        let parent = self.declare_transparent_parent(capture.spatial)?;
        Ok(ControlFlow::Continue(self.command_frame(
            children,
            parent,
            SemanticCommandPlanningState {
                context: capture.context,
                parent_locality: CaptureParentLocality::CaptureLocal,
                span_scope: SemanticVelloSpanScope::LayerSource,
                bound_spans_to_parent: true,
                capture_transform: capture.transform,
                parent_to_surface: Transform::identity(),
                capture_space: CaptureBoundsCoordinateSpace::Local,
                outer_clips: shared_copy(&[], self.account)?,
            },
        )?))
    }

    pub(super) fn declare_tile_pass(
        &mut self,
        plan: TilePlan,
        source: Option<PlannedGraphResource>,
    ) -> Result<PlannedGraphResource> {
        let facts = crate::frame::TilePassFacts::try_new(plan, self.account)?;
        if source.map(|resource| super::super::filter_grid::spatial_descriptor(resource.spatial))
            != facts.source_plan()
        {
            return graph_build(Err(GraphValidationError::InvalidPassResultRole));
        }
        self.tile.grow(1)?;
        let spatial = plan.spatial();
        let resource = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(SemanticResourceRole::FilterIntermediate, spatial, 0),
        ))?;
        let ids = source.map(|input| input.id);
        let pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::Tile,
            &dependencies_for(source.as_slice(), self.account)?,
            ids.as_slice(),
            SemanticPassResult::Resource(resource),
        ))?;
        self.tile.push(SemanticTilePlan { pass, facts })?;
        Ok(PlannedGraphResource {
            id: resource,
            producer: Some(pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        })
    }
}
