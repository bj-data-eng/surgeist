//! Qualified, fixed native shadow merge facts. The original remains raw until
//! this consumer; only the private Gaussian branch projects its alpha.
use super::{
    FilterOutputFootprint, FrameContext, GraphLoweringSpatialDescriptor,
    bounds::{LogicalBounds, NonEmptyFrameSpatialPlan},
    gaussian::{self as grid, GaussianMapping, PeriodCapture},
};
use crate::{
    Color, PreparedDropShadow, Transform,
    accounting::{Account, SharedValue},
    error::Result,
    filter_source::FilterSourceDescriptor,
    native_shadow::DropShadowForeground,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeShadowOriginal {
    Finite,
    Periodic,
    Constant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativeShadowMapping {
    original: GaussianMapping,
    kind: NativeShadowOriginal,
    filter_clip: [f64; 4],
    output_clip: [f64; 4],
    target: NonEmptyFrameSpatialPlan,
    shifted_alpha: bool,
}
impl NativeShadowMapping {
    pub(crate) const fn original_mapping(self) -> GaussianMapping {
        self.original
    }
    pub(crate) const fn original_kind(self) -> NativeShadowOriginal {
        self.kind
    }
    pub(crate) const fn original_filter_clip(self) -> [f64; 4] {
        self.filter_clip
    }
    pub(crate) const fn output_clip(self) -> [f64; 4] {
        self.output_clip
    }
    pub(crate) const fn target_extent(self) -> [u32; 2] {
        grid::extent(self.target)
    }
    pub(crate) fn target_spatial(self) -> GraphLoweringSpatialDescriptor {
        self.original.target_spatial()
    }
    pub(crate) const fn has_shifted_alpha(self) -> bool {
        self.shifted_alpha
    }
    pub(super) fn output_footprint(self) -> FilterOutputFootprint {
        FilterOutputFootprint::from_qualified_grid(self.target_spatial(), self.output_clip)
    }
    pub(super) fn try_new(
        context: FrameContext,
        source: FilterSourceDescriptor,
        original: Option<NonEmptyFrameSpatialPlan>,
        target: NonEmptyFrameSpatialPlan,
        transform: Transform,
        shifted_alpha: bool,
        account: &Account,
    ) -> Result<Self> {
        Self::try_new_inner(
            context,
            source,
            original,
            target,
            transform,
            shifted_alpha,
            account,
        )
        .map_err(merge_error)
    }
    fn try_new_inner(
        context: FrameContext,
        source: FilterSourceDescriptor,
        original: Option<NonEmptyFrameSpatialPlan>,
        target: NonEmptyFrameSpatialPlan,
        transform: Transform,
        shifted_alpha: bool,
        account: &Account,
    ) -> Result<Self> {
        for spatial in [Some(target), original].into_iter().flatten() {
            for size in grid::extent(spatial) {
                account.work(crate::RenderWork::CoordinateVisits, 1)?;
                if size > 8_388_608 {
                    return Err(crate::DropShadowPreparationError::Numeric {
                        phase: crate::DropShadowNumericPhase::GridIndex,
                        value: f64::from(size),
                    }
                    .into());
                }
            }
        }
        let base = GaussianMapping::new(original, target, [None; 2]);
        let (kind, mapping) = match (source, original) {
            (FilterSourceDescriptor::Finite { .. }, Some(raw)) => {
                let domain = grid::finite_domain(source).ok_or_else(|| numeric(0.0))?;
                (
                    NativeShadowOriginal::Finite,
                    base.with_base(grid::base_between(raw, target)?, [0; 2], account)?
                        .with_source_clip(domain, raw, account)?,
                )
            }
            (FilterSourceDescriptor::Periodic { period, .. }, Some(raw)) => {
                let period = PeriodCapture::try_new(context, period, transform, account)?;
                if raw != period.spatial {
                    return Err(numeric(0.0));
                }
                (
                    NativeShadowOriginal::Periodic,
                    base.with_periodic_resolve(period, target, account)?,
                )
            }
            (FilterSourceDescriptor::Constant { color, .. }, None) => {
                (NativeShadowOriginal::Constant, base.with_color(color))
            }
            (_, None) => (
                NativeShadowOriginal::Constant,
                base.with_color(Color::TRANSPARENT),
            ),
            _ => return Err(numeric(0.0)),
        };
        let filter_clip = GaussianMapping::new(None, target, [None; 2])
            .with_output_clip(source.filter(), target, account)?
            .output_clip();
        let output_clip = GaussianMapping::new(None, target, [None; 2])
            .with_output_clip(grid::result_domain(source), target, account)?
            .output_clip();
        Ok(Self {
            original: mapping,
            kind,
            filter_clip,
            output_clip,
            target,
            shifted_alpha,
        })
    }
    pub(crate) fn proves_exact(
        self,
        operation: &PreparedDropShadow,
        source: FilterSourceDescriptor,
        account: &Account,
    ) -> Result<bool> {
        self.proves_exact_inner(operation, source, account)
            .map_err(merge_error)
    }
    fn proves_exact_inner(
        self,
        operation: &PreparedDropShadow,
        source: FilterSourceDescriptor,
        account: &Account,
    ) -> Result<bool> {
        let context = FrameContext {
            output_bounds: LogicalBounds::NonEmpty(self.target.logical_bounds),
            surface_scale: self.target.raster_scale.get(),
            antialiasing: crate::renderer::Antialiasing::Area,
            base_color: Color::TRANSPARENT,
        };
        if self.target.logical_bounds.rect() != grid::result_domain(source)
            || grid::require_spatial(context, grid::result_domain(source), Transform::identity())?
                != self.target
        {
            return Ok(false);
        }
        let original = match self.kind {
            NativeShadowOriginal::Finite => Some(grid::require_spatial(
                context,
                grid::finite_domain(source).ok_or_else(|| numeric(0.0))?,
                Transform::identity(),
            )?),
            NativeShadowOriginal::Periodic => match source {
                FilterSourceDescriptor::Periodic { period, .. } => Some(
                    PeriodCapture::try_new(context, period, Transform::identity(), account)?
                        .spatial,
                ),
                _ => return Ok(false),
            },
            NativeShadowOriginal::Constant => None,
        };
        let nonempty = original.is_some()
            || matches!(source, FilterSourceDescriptor::Constant { color, .. } if color.a() > 0.0);
        let mut shifted = false;
        if nonempty && operation.flood_color().a() > 0.0 && operation.flood_opacity() > 0.0 {
            let filter = source.filter();
            let alpha = grid::require_spatial(context, filter, Transform::identity())?;
            let offset = operation.offset(crate::FilterDomains::try_new(
                filter,
                filter,
                grid::result_domain(source),
            )?);
            shifted = super::offset::OffsetMapping::try_covered_alpha(
                &offset,
                alpha,
                self.target,
                account,
            )?
            .is_some();
        }
        Ok(self.shifted_alpha == shifted
            && self
                == Self::try_new(
                    context,
                    source,
                    original,
                    self.target,
                    Transform::identity(),
                    shifted,
                    account,
                )?)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeShadowPassFacts(SharedValue<NativeShadowPassData>);
#[derive(Debug, PartialEq)]
struct NativeShadowPassData {
    foreground: DropShadowForeground,
    mapping: NativeShadowMapping,
}
impl NativeShadowPassFacts {
    pub(super) fn try_new(
        foreground: DropShadowForeground,
        mapping: NativeShadowMapping,
        storage: &crate::accounting::StorageAccount,
    ) -> Result<Self> {
        SharedValue::new(
            NativeShadowPassData {
                foreground,
                mapping,
            },
            storage,
        )
        .map(Self)
    }
    pub(crate) fn operation(&self) -> &PreparedDropShadow {
        &self.0.foreground.operation
    }
    pub(crate) fn source(&self) -> FilterSourceDescriptor {
        self.0.foreground.source
    }
    pub(crate) fn mapping(&self) -> NativeShadowMapping {
        self.0.mapping
    }
}
fn merge_error(error: crate::Error) -> crate::Error {
    match error.gaussian_diagnostic() {
        Some(crate::GaussianPreparationError::Numeric { phase, value }) => {
            crate::DropShadowPreparationError::Numeric {
                phase: if *phase == crate::GaussianNumericPhase::GridIndex {
                    crate::DropShadowNumericPhase::GridIndex
                } else {
                    crate::DropShadowNumericPhase::MergeMapping
                },
                value: *value,
            }
            .into()
        }
        _ => error,
    }
}

fn numeric(value: f64) -> crate::Error {
    crate::DropShadowPreparationError::Numeric {
        phase: crate::DropShadowNumericPhase::MergeMapping,
        value,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DropShadowInput, DropShadowSource, FilterColorSpace, FilterDomains, Layer,
        RenderAccountingContext, Scene, Size,
        frame::{FramePlan, GaussianChannels, GaussianPurpose, GaussianStage, OffsetPurpose},
        renderer::Antialiasing,
    };

    fn operation(sigma: f64, dx: f64) -> PreparedDropShadow {
        crate::prepare_drop_shadow(
            DropShadowInput::new()
                .with_standard_deviations(sigma, 0.0)
                .with_dx(dx)
                .with_dy(0.0),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap()
    }
    fn content(rect: crate::Rect) -> Scene {
        let mut scene = Scene::new();
        scene.fill(rect, Color::BLACK);
        scene
    }
    fn planned(scene: &Scene) -> crate::accounting::OwnedBox<super::super::GpuRenderGraph> {
        let account = Account::for_test();
        let context = FrameContext::try_new(
            Size::new(32.0, 8.0),
            1.0,
            Antialiasing::Area,
            Color::TRANSPARENT,
        )
        .unwrap();
        let commands = scene
            .normalize(crate::Capabilities::CURRENT, &account)
            .unwrap();
        match FramePlan::try_from_commands(commands, context, &account).unwrap() {
            FramePlan::GpuGraph(graph) => graph,
            _ => panic!("native shadow must use the graph"),
        }
    }
    fn finite_scene(sigma: f64, dx: f64) -> Scene {
        let f = crate::Rect::new(0.0, 0.0, 8.0, 1.0);
        let d = crate::Rect::new(0.5, 0.0, 1.0, 1.0);
        let p = crate::Rect::new(2.5, 0.0, 2.0, 1.0);
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                Layer::new(),
                operation(sigma, dx),
                DropShadowSource::finite(content(d), FilterDomains::try_new(d, f, p).unwrap()),
            )
            .unwrap();
        scene
    }
    #[test]
    fn alpha_route_keeps_filter_and_result_grids_and_real_original_lifetime() {
        let graph = planned(&finite_scene(4.0, 2.0));
        let merge = &graph.native_shadows[0];
        assert_eq!(merge.facts.mapping().target_extent(), [3, 1]);
        assert_eq!(merge.facts.mapping().output_clip(), [0.5, 0.0, 2.5, 1.0]);
        for pass in &graph.gaussians {
            assert_eq!(pass.facts.purpose(), GaussianPurpose::ConsumedAlpha);
            assert_eq!(pass.facts.channels(), GaussianChannels::Alpha);
            assert_eq!(
                pass.facts.source().result(),
                crate::Rect::new(0.0, 0.0, 8.0, 1.0)
            );
        }
        assert!(
            graph.gaussians.len() >= 4,
            "the witness must cross the 32-record batch boundary"
        );
        let last = graph.gaussians.last().unwrap();
        assert_eq!(last.facts.stage(), GaussianStage::Resolve);
        assert_eq!(last.facts.mapping().target_extent(), [8, 1]);
        assert_eq!(
            graph.offsets[0].purpose,
            OffsetPurpose::CoveredAlphaDeferred
        );
        let merge_pass = &graph.passes[merge.pass.index.as_usize().unwrap()];
        let original = &graph.resources[merge_pass.reads[0].index.as_usize().unwrap()];
        assert_eq!(original.descriptor.expected_reads, 2);
        assert_eq!(original.recorded_reads, 2);
        assert_eq!(original.releasable_after, Some(merge.pass));
        assert_eq!(
            original.descriptor.logical_bounds.rect(),
            crate::Rect::new(0.5, 0.0, 1.0, 1.0)
        );
        assert!(graph.composites.last().unwrap().filter_footprint.is_some());
        graph.lowering_view(&Account::for_test()).unwrap();
    }
    #[test]
    fn constant_alpha_resolves_finitely_before_offset_and_original_uses_descriptor() {
        let f = crate::Rect::new(0.0, 0.0, 2.0, 1.0);
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                Layer::new(),
                operation(2.0, 1.0),
                DropShadowSource::constant(Color::BLACK, f, f).unwrap(),
            )
            .unwrap();
        let graph = planned(&scene);
        assert_eq!(graph.gaussians.len(), 1);
        assert_eq!(graph.gaussians[0].facts.stage(), GaussianStage::Constant);
        assert_eq!(graph.gaussians[0].facts.mapping().target_extent(), [2, 1]);
        assert_eq!(graph.offsets.len(), 1);
        assert_eq!(
            graph.native_shadows[0].facts.mapping().original_kind(),
            NativeShadowOriginal::Constant
        );
        assert_eq!(
            graph.passes[graph.native_shadows[0].pass.index.as_usize().unwrap()]
                .reads
                .len(),
            1
        );
        graph.lowering_view(&Account::for_test()).unwrap();
    }
    #[test]
    fn fractional_period_original_shares_the_actual_unequal_density_canvas() {
        let t = crate::Rect::new(0.25, 0.0, 2.5, 1.25);
        let f = crate::Rect::new(0.0, 0.0, 8.0, 2.0);
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                Layer::new(),
                operation(1.0, 2.0),
                DropShadowSource::periodic(content(t), t, f, f).unwrap(),
            )
            .unwrap();
        let graph = planned(&scene);
        let map = graph.native_shadows[0].facts.mapping();
        assert_eq!(map.original_kind(), NativeShadowOriginal::Periodic);
        assert_eq!(map.original_mapping().source_extent(), [3, 2]);
        assert_eq!(map.original_mapping().periodic_step(), [1.2, 1.6]);
        assert!(map.original_mapping().periodic());
        assert_eq!(graph.offsets[0].mapping.source_extent(), [8, 2]);
        assert_eq!(graph.offsets[0].mapping.source_domain(), f);
        graph.lowering_view(&Account::for_test()).unwrap();
    }
    #[test]
    fn transparent_flood_skips_shadow_work_and_keeps_original_merge() {
        let f = crate::Rect::new(0.0, 0.0, 2.0, 1.0);
        let admitted = crate::prepare_drop_shadow(
            DropShadowInput::new()
                .with_standard_deviation(f64::MAX)
                .with_flood_opacity(0.0),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        let mut scene = Scene::new();
        scene
            .drop_shadow_layer(
                Layer::new(),
                admitted,
                DropShadowSource::finite(content(f), FilterDomains::try_new(f, f, f).unwrap()),
            )
            .unwrap();
        let graph = planned(&scene);
        assert!(graph.gaussians.is_empty());
        assert!(graph.offsets.is_empty());
        assert!(!graph.native_shadows[0].facts.mapping().has_shifted_alpha());
        let merge = &graph.native_shadows[0];
        let raw = graph.passes[merge.pass.index.as_usize().unwrap()].reads[0];
        assert_eq!(
            graph.resources[raw.index.as_usize().unwrap()]
                .descriptor
                .expected_reads,
            1
        );
        graph.lowering_view(&Account::for_test()).unwrap();
    }
    #[test]
    fn last_use_cannot_retire_original_at_alpha_preparation() {
        let mut graph = planned(&finite_scene(4.0, 2.0));
        let merge = graph.native_shadows[0].pass;
        let raw = graph.passes[merge.index.as_usize().unwrap()].reads[0];
        graph.resources[raw.index.as_usize().unwrap()].releasable_after =
            Some(graph.gaussians[0].pass);
        assert!(graph.lowering_view(&Account::for_test()).is_err());
    }
    #[test]
    fn merged_original_read_cannot_be_swapped_for_shifted_alpha() {
        let account = Account::for_test();
        let mut graph = planned(&finite_scene(1.0, 2.0));
        let merge = graph.native_shadows[0].pass.index.as_usize().unwrap();
        let shifted = graph.passes[merge].reads[1];
        let mut reads = account.retained(2).unwrap();
        reads.push(shifted).unwrap();
        reads.push(shifted).unwrap();
        graph.passes[merge].reads = SharedValue::from_accounted(reads, &account.storage()).unwrap();
        assert!(graph.lowering_view(&account).is_err());
    }
    #[test]
    fn merged_original_read_cannot_be_removed() {
        let account = Account::for_test();
        let mut graph = planned(&finite_scene(1.0, 2.0));
        let merge = graph.native_shadows[0].pass.index.as_usize().unwrap();
        let mut reads = account.retained(1).unwrap();
        reads.push(graph.passes[merge].reads[1]).unwrap();
        graph.passes[merge].reads = SharedValue::from_accounted(reads, &account.storage()).unwrap();
        assert!(graph.lowering_view(&account).is_err());
    }
    #[test]
    fn standalone_modes_cannot_be_substituted_into_the_consumed_alpha_chain() {
        let account = Account::for_test();
        let mut graph = planned(&finite_scene(1.0, 2.0));
        let facts = graph.gaussians[0].facts.clone();
        graph.gaussians[0].facts = super::super::GaussianPassFacts::try_new(
            facts.foreground(),
            facts.mapping(),
            facts.stage(),
            &account.storage(),
        )
        .unwrap();
        assert_eq!(graph.gaussians[0].facts.channels(), GaussianChannels::Rgba);
        assert!(graph.lowering_view(&account).is_err());
        let mut graph = planned(&finite_scene(1.0, 2.0));
        graph.offsets[0].purpose = OffsetPurpose::Standalone;
        assert!(graph.lowering_view(&account).is_err());
    }
    #[test]
    fn merge_footprint_requires_actual_final_merge_producer() {
        let mut graph = planned(&finite_scene(1.0, 2.0));
        let last = graph
            .gaussians
            .last()
            .unwrap()
            .facts
            .mapping()
            .output_footprint();
        graph.composites.last_mut().unwrap().filter_footprint = Some(last);
        assert!(graph.lowering_view(&Account::for_test()).is_err());
    }
    #[test]
    fn consumed_route_cannot_omit_required_shifted_alpha_or_change_fractional_coverage() {
        let graph = planned(&finite_scene(0.0, 2.0));
        let facts = &graph.native_shadows[0].facts;
        let mut map = facts.mapping();
        map.shifted_alpha = false;
        assert!(
            !map.proves_exact(facts.operation(), facts.source(), &Account::for_test())
                .unwrap()
        );
        let mut map = facts.mapping();
        map.output_clip[0] = 0.0;
        assert!(
            !map.proves_exact(facts.operation(), facts.source(), &Account::for_test())
                .unwrap()
        );
    }
}
