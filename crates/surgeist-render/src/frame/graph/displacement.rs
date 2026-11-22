use super::*;
use crate::frame::displacement::DisplacementSourceKind;

impl SemanticFrameGraphPlanner<'_> {
    pub(super) fn prepare_displacement_source(
        &mut self,
        children: RenderCommands,
        route: &DisplacementRoute,
        context: FrameContext,
        raster_transform: Transform,
    ) -> Result<PreparedLayerSource> {
        if !route.needs_source() || children.is_empty() {
            return Ok(ControlFlow::Break(None));
        }
        let spatial = route.capture_plan().ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "displacement source lost its capture plan",
            )
        })?;
        let (context, capture_transform, parent_to_surface) = match route.period_context() {
            Some(period) => (
                period.capture.context,
                period.capture.transform,
                Transform::identity(),
            ),
            None => (context, Transform::identity(), raster_transform),
        };
        let parent = self.declare_transparent_parent(spatial)?;
        Ok(ControlFlow::Continue(self.command_frame(
            children,
            parent,
            SemanticCommandPlanningState {
                context,
                parent_locality: CaptureParentLocality::CaptureLocal,
                span_scope: SemanticVelloSpanScope::LayerSource,
                bound_spans_to_parent: false,
                capture_transform,
                parent_to_surface,
                capture_space: CaptureBoundsCoordinateSpace::Local,
                outer_clips: shared_copy(&[], self.account)?,
            },
        )?))
    }

    pub(super) fn apply_displacement(
        &mut self,
        source: Option<PlannedGraphResource>,
        route: DisplacementRoute,
    ) -> Result<PlannedGraphResource> {
        self.account.work(RenderWork::FilterOperationVisits, 1)?;
        let raw = source.map(|s| s.spatial);
        let mut current = source;
        for index in 0..route.len() {
            let facts = DisplacementPassFacts::try_new(route.clone(), index, raw, self.account)?;
            let count = facts.stage().read_count(facts.mapping().source_kind());
            let mut reads = self.account.temporary(count)?;
            if count != 0 {
                reads.append_slot()?.write(current.ok_or_else(|| {
                    Error::new(
                        BackendErrorCode::RenderFailed,
                        "displacement lost a required image read",
                    )
                })?);
            }
            current = Some(self.declare_displacement_pass(&reads, facts)?);
        }
        current.ok_or_else(|| {
            Error::new(
                BackendErrorCode::RenderFailed,
                "displacement emitted no result",
            )
        })
    }

    fn declare_displacement_pass(
        &mut self,
        reads: &[PlannedGraphResource],
        facts: DisplacementPassFacts,
    ) -> Result<PlannedGraphResource> {
        self.displacements.grow(1)?;
        let spatial = facts.mapping().target_plan();
        let result = graph_build(
            self.builder
                .declare_resource(SemanticResourceDescriptor::new(
                    SemanticResourceRole::FilterIntermediate,
                    spatial,
                    0,
                )),
        )?;
        let mut ids = self.account.temporary(reads.len())?;
        for read in reads {
            ids.append_slot()?.write(read.id);
        }
        let pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::Displacement(facts.stage()),
            &dependencies_for(reads, self.account)?,
            &ids,
            SemanticPassResult::Resource(result),
        ))?;
        if reads.is_empty() && facts.mapping().source_kind() != DisplacementSourceKind::Constant {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "displacement source kind has no image",
            ));
        }
        self.displacements
            .push(SemanticDisplacementPlan { pass, facts })?;
        Ok(PlannedGraphResource {
            id: result,
            producer: Some(pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        })
    }
}
