use super::*;
use crate::frame::{MorphologyPassFacts, MorphologyStage, morphology::MorphologyRoute};
use crate::morphology::MorphologyForeground;
impl SemanticFrameGraphPlanner<'_> {
    pub(super) fn apply_morphology(
        &mut self,
        source: Option<PlannedGraphResource>,
        foreground: MorphologyForeground,
        transform: Transform,
    ) -> Result<Option<PlannedGraphResource>> {
        self.account.work(RenderWork::FilterOperationVisits, 1)?;
        if super::super::filter_grid::empty(super::super::filter_grid::result_domain(
            foreground.source,
        )) {
            return Ok(None);
        }
        let route = MorphologyRoute::try_new(
            self.context,
            foreground,
            source.map(|s| s.spatial),
            transform,
            self.account,
        )?;
        let mut current = source;
        let mut table = None;
        let mut previous = None;
        let mut axis = None;
        for index in 0..route.len() {
            let facts = MorphologyPassFacts::try_new(route.clone(), index, self.account)?;
            let stage = facts.stage();
            if let Some(next) = stage.axis()
                && axis != Some(next)
            {
                axis = Some(next);
                table = current;
                previous = None;
            }
            let mut reads = [None; 2];
            match stage {
                MorphologyStage::Constant => {}
                MorphologyStage::Table { .. }
                | MorphologyStage::FirstQuery { .. }
                | MorphologyStage::Query { .. } => reads[0] = table,
                MorphologyStage::RefineQuery { .. } => {
                    reads[0] = table;
                    reads[1] = previous;
                }
                _ => reads[0] = current,
            }
            let mut resources = self.account.temporary(stage.read_count())?;
            for read in reads.into_iter().take(stage.read_count()) {
                resources.append_slot()?.write(read.ok_or_else(|| {
                    Error::new(
                        BackendErrorCode::RenderFailed,
                        "morphology route lost a required input",
                    )
                })?);
            }
            let out = self.declare_morphology_pass(&resources, facts.clone())?;
            match stage {
                MorphologyStage::Table { .. } => table = Some(out),
                MorphologyStage::FirstQuery { .. } | MorphologyStage::RefineQuery { .. } => {
                    previous = Some(out);
                    if facts
                        .mapping()
                        .query()
                        .is_some_and(|q| q.max_level() == stage.level())
                    {
                        current = Some(out);
                    }
                }
                _ => current = Some(out),
            }
        }
        Ok(current)
    }
    fn declare_morphology_pass(
        &mut self,
        reads: &[PlannedGraphResource],
        facts: MorphologyPassFacts,
    ) -> Result<PlannedGraphResource> {
        self.morphologies.grow(1)?;
        let spatial = facts.mapping().target_plan();
        let stage = facts.stage();
        let role = if stage.float_output() {
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
            SemanticPassIntent::Morphology(stage),
            &dependencies_for(reads, self.account)?,
            &ids,
            SemanticPassResult::Resource(result),
        ))?;
        self.morphologies
            .push(SemanticMorphologyPlan { pass, facts })?;
        Ok(PlannedGraphResource {
            id: result,
            producer: Some(pass),
            logical_bounds: spatial.logical_bounds,
            spatial,
        })
    }
}
