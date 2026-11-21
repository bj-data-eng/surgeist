use super::*;

impl SemanticFrameGraphPlanner<'_> {
    pub(super) fn prepare_lighting_source(
        &mut self,
        children: RenderCommands,
        grid: &LightingGrid,
        context: FrameContext,
        raster_transform: Transform,
    ) -> Result<PreparedLayerSource> {
        let Some(capture) = grid.capture.filter(|_| !children.is_empty()) else {
            return Ok(ControlFlow::Break(None));
        };
        let parent = self.declare_transparent_parent(capture.spatial)?;
        Ok(ControlFlow::Continue(self.command_frame(
            children,
            parent,
            SemanticCommandPlanningState {
                context,
                parent_locality: CaptureParentLocality::CaptureLocal,
                span_scope: SemanticVelloSpanScope::LayerSource,
                capture_transform: Transform::identity(),
                parent_to_surface: raster_transform,
                capture_space: CaptureBoundsCoordinateSpace::Local,
                outer_clips: shared_copy(&[], self.account)?,
            },
        )?))
    }

    pub(super) fn apply_lighting(
        &mut self,
        source: Option<PlannedGraphResource>,
        plan: SharedValue<LightingOperationPlan>,
    ) -> Result<PlannedGraphResource> {
        self.account.work(RenderWork::FilterOperationVisits, 1)?;
        let grid = plan.grid;
        let resample = source
            .map(|source| {
                LightingPassFacts::try_new(
                    plan.clone(),
                    LightingStage::ResampleBump,
                    Some(source.spatial),
                    self.account,
                )
            })
            .transpose()?;
        let shade = LightingPassFacts::try_new(
            plan.clone(),
            LightingStage::Shade,
            source.map(|_| grid.bump.spatial),
            self.account,
        )?;
        let resolve = LightingPassFacts::try_new(
            plan,
            LightingStage::Resolve,
            Some(grid.shade.spatial),
            self.account,
        )?;
        let bump = resample
            .map(|facts| self.declare_lighting_pass(source, facts, grid.bump.spatial))
            .transpose()?;
        let shade = self.declare_lighting_pass(bump, shade, grid.shade.spatial)?;
        self.declare_lighting_pass(Some(shade), resolve, grid.output)
    }

    fn declare_lighting_pass(
        &mut self,
        source: Option<PlannedGraphResource>,
        facts: LightingPassFacts,
        spatial: NonEmptyFrameSpatialPlan,
    ) -> Result<PlannedGraphResource> {
        self.lighting.grow(1)?;
        let resource = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(SemanticResourceRole::FilterIntermediate, spatial, 0),
        ))?;
        let ids = source.map(|source| source.id);
        let pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::Lighting(facts.stage()),
            &dependencies_for(source.as_slice(), self.account)?,
            ids.as_slice(),
            SemanticPassResult::Resource(resource),
        ))?;
        self.lighting.push(SemanticLightingPlan { pass, facts })?;
        Ok(PlannedGraphResource {
            id: resource,
            producer: Some(pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        })
    }
}
