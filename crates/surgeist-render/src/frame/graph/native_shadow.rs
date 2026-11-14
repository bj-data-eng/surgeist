use super::*;
use crate::frame::{
    OffsetPurpose,
    gaussian::{self as grid, GaussianPurpose},
    native_shadow::NativeShadowMapping,
};
use crate::{
    gaussian::{GaussianForeground, GaussianSourceDescriptor},
    native_shadow::DropShadowForeground,
};

impl SemanticFrameGraphPlanner<'_> {
    pub(super) fn prepare_native_shadow_source(
        &mut self,
        children: RenderCommands,
        foreground: DropShadowForeground,
        transform: Transform,
    ) -> Result<PreparedLayerSource> {
        // Source capture and actual period canvas remain owned by Gaussian's
        // existing scene capture algorithm, without another authored tree.
        self.prepare_gaussian_source(
            children,
            GaussianForeground {
                operation: foreground.operation.gaussian(),
                source: foreground.source,
            },
            transform,
        )
    }

    pub(super) fn apply_native_shadow(
        &mut self,
        mut original: Option<PlannedGraphResource>,
        foreground: DropShadowForeground,
        transform: Transform,
    ) -> Result<Option<PlannedGraphResource>> {
        self.account.work(RenderWork::FilterOperationVisits, 1)?;
        let output = grid::result_domain(foreground.source);
        if grid::empty(output) {
            return Ok(None);
        }
        let result = grid::require_spatial(self.context, output, transform)?;
        let alpha_foreground = GaussianForeground {
            operation: foreground.operation.gaussian(),
            source: foreground.source.for_filter_output(),
        };
        if grid::source_is_empty(foreground.source) {
            original = None;
        }
        // Complete backdrop before arriving here; restore the exact raw D
        // lattice once and retain that actual resource through final merge.
        if let (Some(raw), Some(domain)) = (original, grid::finite_domain(foreground.source)) {
            let required = grid::require_spatial(self.context, domain, transform)?;
            if raw.spatial != required {
                let alpha_result =
                    grid::require_spatial(self.context, foreground.source.filter(), transform)?;
                let mapping = GaussianMapping::new(Some(raw.spatial), required, [None; 2])
                    .with_route(required, alpha_result, None)
                    .with_base(
                        grid::base_between(raw.spatial, required)?,
                        [0; 2],
                        self.account,
                    )?;
                original = Some(self.declare_gaussian_pass_for_purpose(
                    &[raw],
                    alpha_foreground,
                    mapping,
                    GaussianStage::RebaseSource,
                    GaussianPurpose::ConsumedAlpha,
                )?);
            }
        }
        let constant = matches!(foreground.source, GaussianSourceDescriptor::Constant { .. });
        let nonempty_source = constant || original.is_some();
        let transparent_constant = matches!(foreground.source,
            GaussianSourceDescriptor::Constant { color, .. } if color.a() == 0.0);
        let shadow_enabled = nonempty_source
            && !transparent_constant
            && foreground.operation.flood_color().a() > 0.0
            && foreground.operation.flood_opacity() > 0.0;
        let mut shifted = None;
        if shadow_enabled {
            let filter = foreground.source.filter();
            let alpha_spatial = grid::require_spatial(self.context, filter, transform)?;
            let domains = crate::FilterDomains::try_new(filter, filter, output)?;
            let offset = foreground.operation.offset(domains);
            if let Some(mapping) =
                OffsetMapping::try_covered_alpha(&offset, alpha_spatial, result, self.account)?
            {
                let alpha = self
                    .apply_gaussian_for_purpose(
                        original,
                        alpha_foreground,
                        transform,
                        GaussianPurpose::ConsumedAlpha,
                    )?
                    .ok_or_else(|| {
                        Error::new(
                            BackendErrorCode::RenderFailed,
                            "native shadow lost its finite alpha image",
                        )
                    })?;
                shifted = Some(self.declare_offset_pass_for_purpose(
                    alpha,
                    result,
                    &offset,
                    mapping,
                    OffsetStage::Sample,
                    OffsetPurpose::CoveredAlphaDeferred,
                )?);
            }
        }
        let mapping = NativeShadowMapping::try_new(
            self.context,
            foreground.source,
            original.map(|raw| raw.spatial),
            result,
            transform,
            shifted.is_some(),
            self.account,
        )?;
        let facts = super::super::NativeShadowPassFacts::try_new(
            foreground,
            mapping,
            &self.account.storage(),
        )?;
        self.native_shadows.grow(1)?;
        let resource = graph_build(self.builder.declare_resource(
            SemanticResourceDescriptor::new(SemanticResourceRole::FilterIntermediate, result, 0),
        ))?;
        let mut reads = self.account.temporary(2)?;
        if let Some(original) = original {
            reads.append_slot()?.write(original);
        }
        if let Some(shifted) = shifted {
            reads.append_slot()?.write(shifted);
        }
        let mut ids = self.account.temporary(reads.len())?;
        for read in &reads {
            ids.append_slot()?.write(read.id);
        }
        let pass = graph_build(self.builder.declare_pass(
            SemanticPassIntent::NativeShadowMerge,
            &dependencies_for(&reads, self.account)?,
            &ids,
            SemanticPassResult::Resource(resource),
        ))?;
        self.native_shadows
            .push(SemanticNativeShadowPlan { pass, facts })?;
        Ok(Some(PlannedGraphResource {
            id: resource,
            producer: Some(pass),
            logical_bounds: result.logical_bounds,
            spatial: result,
        }))
    }
}
