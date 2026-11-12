use super::{
    ComputeInputOf, ComputeOutputOf, DefaultScalar, LayoutScalar, RequestedAxis, RunMode, Size,
    SizingMode, SizingOperation, SizingRequestOf,
};
use crate::ContainingLayoutContext;
use crate::scroll::SettledAutoScrollbarState;

const CACHE_SIZE: usize = 9;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CacheKeyContext;

impl CacheKeyContext {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CacheKeyOf<S: LayoutScalar = DefaultScalar> {
    run_mode: RunMode,
    sizing_mode: SizingMode,
    axis: RequestedAxis,
    known: Size<Option<S>>,
    indefinite_percentage_axis: Option<crate::PhysicalAxis>,
    border_box_offer_axis: Option<crate::PhysicalAxis>,
    parent: Size<Option<S>>,
    containing_layout_context: ContainingLayoutContext,
    sizing_request: SizingRequestOf<S>,
    named_baseline_demand: crate::output::NamedBaselineDemand,
    physical_baseline_demand: bool,
    requires_fixed_grid_tracks: bool,
    settled_auto_scrollbars: SettledAutoScrollbarState,
    containing_auto_scrollbar_pass: SettledAutoScrollbarState,
    context: CacheKeyContext,
    block_indent_context: Option<crate::text_indent::BlockIndentContext<S>>,
}

impl<S: LayoutScalar> CacheKeyOf<S> {
    fn from_input(input: &ComputeInputOf<S>, context: CacheKeyContext) -> Self {
        Self {
            run_mode: input.run_mode(),
            sizing_mode: input.sizing_mode(),
            axis: input.requested_axis(),
            known: input.known(),
            indefinite_percentage_axis: input.indefinite_percentage_axis(),
            border_box_offer_axis: input.border_box_offer_axis(),
            parent: input.parent(),
            containing_layout_context: input.containing_layout_context(),
            sizing_request: input.sizing_request(),
            named_baseline_demand: input.named_baseline_demand(),
            physical_baseline_demand: input.physical_baseline_demand(),
            requires_fixed_grid_tracks: input.requires_fixed_grid_tracks(),
            settled_auto_scrollbars: input.settled_auto_scrollbars(),
            containing_auto_scrollbar_pass: input.containing_auto_scrollbar_pass(),
            context,
            block_indent_context: input.block_indent_context(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EntryOf<S: LayoutScalar, T> {
    key: CacheKeyOf<S>,
    content: T,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CacheOf<S: LayoutScalar = DefaultScalar> {
    final_layout: Option<EntryOf<S, ComputeOutputOf<S>>>,
    measures: [Option<EntryOf<S, ComputeOutputOf<S>>>; CACHE_SIZE],
    empty: bool,
}

pub type Cache = CacheOf<DefaultScalar>;

impl<S: LayoutScalar> CacheOf<S> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            final_layout: None,
            measures: [None; CACHE_SIZE],
            empty: true,
        }
    }

    #[must_use]
    pub fn get_with_context(
        &self,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        match input.run_mode() {
            RunMode::PerformRootLayout | RunMode::PerformLayout => self
                .final_layout
                .filter(|entry| matches_output(input, context, entry))
                .map(|entry| entry.content),
            RunMode::ComputeSize => {
                for entry in self.measures.iter().flatten() {
                    if matches_output(input, context, entry) {
                        return Some(entry.content);
                    }
                }
                None
            }
            RunMode::PerformHiddenLayout => None,
        }
    }

    pub fn store_with_context(
        &mut self,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
        output: ComputeOutputOf<S>,
    ) {
        let key = CacheKeyOf::from_input(input, context);
        match input.run_mode() {
            RunMode::PerformRootLayout | RunMode::PerformLayout => {
                self.empty = false;
                self.final_layout = Some(EntryOf {
                    key,
                    content: output,
                });
            }
            RunMode::ComputeSize => {
                self.empty = false;
                let slot = cache_slot(input.known(), input.sizing_request());
                self.measures[slot] = Some(EntryOf {
                    key,
                    content: output,
                });
            }
            RunMode::PerformHiddenLayout => {}
        }
    }

    /// Removes performed-layout reuse while retaining independently valid measurements.
    pub fn clear_performed(&mut self) -> ClearState {
        let present = self.final_layout.take().is_some();
        self.empty = self.is_empty();
        if present {
            ClearState::Cleared
        } else {
            ClearState::AlreadyEmpty
        }
    }

    pub fn clear(&mut self) -> ClearState {
        if self.empty {
            return ClearState::AlreadyEmpty;
        }
        self.empty = true;
        self.final_layout = None;
        self.measures = [None; CACHE_SIZE];
        ClearState::Cleared
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.final_layout.is_none() && self.measures.iter().all(Option::is_none)
    }
}

impl<S: LayoutScalar> Default for CacheOf<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClearState {
    Cleared,
    AlreadyEmpty,
}

fn cache_slot<S: LayoutScalar>(known: Size<Option<S>>, request: SizingRequestOf<S>) -> usize {
    let axes = request.axes();
    let width_min = axes.width.operation() == SizingOperation::MinContent;
    let height_min = axes.height.operation() == SizingOperation::MinContent;
    match (known.width.is_some(), known.height.is_some()) {
        (true, true) => 0,
        (true, false) => 1 + usize::from(height_min),
        (false, true) => 3 + usize::from(width_min),
        (false, false) => 5 + usize::from(height_min) + 2 * usize::from(width_min),
    }
}

fn matches_output<S: LayoutScalar, T>(
    input: &ComputeInputOf<S>,
    context: CacheKeyContext,
    entry: &EntryOf<S, T>,
) -> bool {
    let key = CacheKeyOf::from_input(input, context);
    let requested = key.sizing_request.axes();
    let saved = entry.key.sizing_request.axes();
    input.run_mode() == entry.key.run_mode
        && input.sizing_mode() == entry.key.sizing_mode
        && input.requested_axis() == entry.key.axis
        && input.parent() == entry.key.parent
        && input.containing_layout_context() == entry.key.containing_layout_context
        && key.settled_auto_scrollbars == entry.key.settled_auto_scrollbars
        && key.containing_auto_scrollbar_pass == entry.key.containing_auto_scrollbar_pass
        && key.block_indent_context == entry.key.block_indent_context
        && key.named_baseline_demand == entry.key.named_baseline_demand
        && key.physical_baseline_demand == entry.key.physical_baseline_demand
        && key.requires_fixed_grid_tracks == entry.key.requires_fixed_grid_tracks
        && context == entry.key.context
        // Original content sizing axis purposes can qualify suppressed preferred
        // bounds for descendant fallback. A known body dimension does not erase
        // that cross-axis authority, including when both dimensions are known.
        && (input.sizing_mode() == SizingMode::InherentSize
            || (requested.width.operation() == saved.width.operation()
                && requested.height.operation() == saved.height.operation()))
        && effective_operation(input.known().width, requested.width) == effective_operation(entry.key.known.width, saved.width)
        && effective_operation(input.known().height, requested.height) == effective_operation(entry.key.known.height, saved.height)
        && input.known() == entry.key.known
        && key.indefinite_percentage_axis == entry.key.indefinite_percentage_axis
        && key.border_box_offer_axis == entry.key.border_box_offer_axis
        && key.sizing_request.orthogonal_fallback() == entry.key.sizing_request.orthogonal_fallback()
        && (input.known().width.is_some() || saved.width.space().roughly_eq(requested.width.space()))
        && (input.known().height.is_some() || saved.height.space().roughly_eq(requested.height.space()))
}

fn effective_operation<S: LayoutScalar>(
    known: Option<S>,
    request: crate::AxisSizingRequestOf<S>,
) -> SizingOperation {
    if known.is_some() {
        SizingOperation::Normal
    } else {
        request.operation()
    }
}

#[cfg(test)]
mod sizing_request_cache_tests {
    use super::*;
    use crate::layout_math::OptionalSizeExt;
    use crate::{AvailableSpaceOf, AxisSizingRequestOf, FlowAxes, NonNegativeFiniteOf};

    fn input<S: LayoutScalar>(
        known: Option<S>,
        available: S,
        operation: SizingOperation,
        fallback: crate::OrthogonalFallbackOf<S>,
    ) -> ComputeInputOf<S> {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(available).unwrap()),
                operation,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        );
        ComputeInputOf::leaf_layout(
            Size::new(known, None),
            Size::NONE,
            crate::ContainingLayoutContext::new(
                FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(axes, fallback),
        )
        .unwrap()
    }

    fn measure<S: LayoutScalar>(
        input: ComputeInputOf<S>,
    ) -> (ComputeOutputOf<S>, crate::LeafMeasureInputOf<S>) {
        let mut packet = None;
        let output = crate::compute_leaf(input, &crate::NodeInputOf::<S>::default(), |observed| {
            packet = Some(observed);
            // An opaque native leaf with fixed 20x10 content; imposed
            // content dimensions are honored by the measurement provider.
            Ok::<_, ()>(
                observed
                    .known_content_size()
                    .unwrap_or(Size::new(S::from_f64(20.0), S::from_f64(10.0))),
            )
        })
        .unwrap();
        (output, packet.unwrap())
    }

    #[test]
    fn cache_distinguishes_allocations_with_indefinite_percentage_basis() {
        let context = CacheKeyContext::new();
        let definite = input(
            Some(40.0_f32),
            40.0,
            SizingOperation::Normal,
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Unknown,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
        );
        let indefinite =
            definite.with_indefinite_percentage_axis(Some(crate::PhysicalAxis::Horizontal));
        let output = ComputeOutputOf::from_outer_size(Size::new(40.0, 20.0));

        assert_eq!(definite.known(), indefinite.known());
        let mut cache = CacheOf::new();
        cache.store_with_context(&definite, context, output);
        assert_eq!(cache.get_with_context(&definite, context), Some(output));
        assert_eq!(cache.get_with_context(&indefinite, context), None);

        cache.store_with_context(&indefinite, context, output);
        assert_eq!(cache.get_with_context(&definite, context), None);
        assert_eq!(cache.get_with_context(&indefinite, context), Some(output));
    }

    #[test]
    fn cache_distinguishes_fixed_grid_track_requirement() {
        let context = CacheKeyContext::new();
        let input = input(
            None,
            40.0_f32,
            SizingOperation::Normal,
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Unknown,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
        );
        let output = ComputeOutputOf::from_outer_size(Size::new(40.0, 20.0));
        for mode in [RunMode::ComputeSize, RunMode::PerformRootLayout] {
            let ordinary = input.with_run_mode(mode);
            let fixed = ordinary.with_fixed_grid_track_requirement(true);
            let mut cache = CacheOf::new();

            cache.store_with_context(&ordinary, context, output);
            assert_eq!(cache.get_with_context(&ordinary, context), Some(output));
            assert_eq!(cache.get_with_context(&fixed, context), None);

            cache.store_with_context(&fixed, context, output);
            assert_eq!(cache.get_with_context(&ordinary, context), None);
            assert_eq!(cache.get_with_context(&fixed, context), Some(output));
        }
    }

    #[test]
    fn cache_distinguishes_margin_and_border_box_offers() {
        let context = CacheKeyContext::new();
        let margin_box = input(
            None,
            40.0_f32,
            SizingOperation::Normal,
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Unknown,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
        );
        let border_box =
            margin_box.with_border_box_offer_axis(Some(crate::PhysicalAxis::Horizontal));
        let output = ComputeOutputOf::from_outer_size(Size::new(40.0, 20.0));
        let mut cache = CacheOf::new();
        cache.store_with_context(&margin_box, context, output);
        assert_eq!(cache.get_with_context(&margin_box, context), Some(output));
        assert_eq!(cache.get_with_context(&border_box, context), None);
    }

    fn qualify<S: LayoutScalar>() {
        let px = S::from_f64;
        let nn = |value| NonNegativeFiniteOf::new(px(value)).unwrap();
        let unbounded = Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded);
        let source_a = crate::OrthogonalFallbackOf::new(
            Size::splat(Some(nn(800.0))),
            crate::NearestScrollportOf::Present(Size::splat(
                crate::OrthogonalFallbackLimitOf::Finite(nn(100.0)),
            )),
            unbounded,
        );
        let source_b = crate::OrthogonalFallbackOf::new(
            Size::splat(Some(nn(100.0))),
            crate::NearestScrollportOf::Present(Size::splat(
                crate::OrthogonalFallbackLimitOf::Finite(nn(800.0)),
            )),
            unbounded,
        );
        let unknown_max = input(None, px(50.0), SizingOperation::MaxContent, source_a);
        let imposed_max = input(
            Some(px(20.0)),
            px(50.0),
            SizingOperation::MaxContent,
            source_a,
        );
        let imposed_min = input(
            Some(px(20.0)),
            px(50.0),
            SizingOperation::MinContent,
            source_a,
        );
        let (output, packet) = measure(unknown_max);
        assert_eq!(output.size.width, px(20.0));
        assert_eq!(
            packet.content_sizing_request().axes().width.operation(),
            SizingOperation::MaxContent
        );
        let mut cache = CacheOf::new();
        let context = CacheKeyContext::new();
        cache.store_with_context(&unknown_max, context, output);
        assert_eq!(cache.get_with_context(&unknown_max, context), Some(output));
        // The cached scalar happens to equal the newly imposed dimension. That
        // does not prove a MAX execution reusable as a Normal body execution.
        assert!(cache.get_with_context(&imposed_max, context).is_none());
        assert!(cache.get_with_context(&imposed_min, context).is_none());
        let unknown_normal = input(None, px(50.0), SizingOperation::Normal, source_a);
        let (normal_output, _) = measure(unknown_normal);
        cache.store_with_context(&unknown_normal, context, normal_output);
        assert!(cache.get_with_context(&imposed_max, context).is_none());

        let (known_output, known_packet) = measure(imposed_max);
        let (_, min_packet) = measure(imposed_min);
        assert_eq!(known_packet, min_packet);
        assert_eq!(
            known_packet
                .content_sizing_request()
                .axes()
                .width
                .operation(),
            SizingOperation::Normal
        );
        cache.store_with_context(&imposed_max, context, known_output);
        // Both executions have the SAME imposed width. Availability-
        // only changes may be relaxed, and the known axis's raw op is ineffective.
        let changed_space = input(
            Some(px(20.0)),
            px(70.0),
            SizingOperation::MaxContent,
            source_a,
        );
        assert_eq!(
            cache.get_with_context(&changed_space, context),
            Some(known_output)
        );
        assert_eq!(
            cache.get_with_context(&imposed_min, context),
            Some(known_output)
        );

        assert_eq!(
            source_a.resolve(crate::PhysicalAxis::Horizontal),
            source_b.resolve(crate::PhysicalAxis::Horizontal)
        );
        let (other_output, other_packet) = measure(input(
            Some(px(20.0)),
            px(50.0),
            SizingOperation::MaxContent,
            source_b,
        ));
        assert_eq!(known_output, other_output);
        assert_eq!(known_packet, other_packet);
        let descendant_a = source_a.for_child(
            unbounded,
            crate::ScrollportFallbackChangeOf::Enter(unbounded),
        );
        let descendant_b = source_b.for_child(
            unbounded,
            crate::ScrollportFallbackChangeOf::Enter(unbounded),
        );
        assert_eq!(
            descendant_a.resolve(crate::PhysicalAxis::Horizontal),
            Ok(nn(800.0))
        );
        assert_eq!(
            descendant_b.resolve(crate::PhysicalAxis::Horizontal),
            Ok(nn(100.0))
        );
        // Equal collapsed current fallbacks do not certify descendant equivalence.
        let other_source = input(
            Some(px(20.0)),
            px(50.0),
            SizingOperation::MaxContent,
            source_b,
        );
        assert!(cache.get_with_context(&other_source, context).is_none());
    }

    #[test]
    fn compares_effective_operation_and_recursive_fallback_sources_f32() {
        qualify::<f32>();
    }
    #[test]
    fn compares_effective_operation_and_recursive_fallback_sources_f64() {
        qualify::<f64>();
    }
    fn qualify_content_size_purpose<S: LayoutScalar>() {
        let px = S::from_f64;
        let nn = |value| NonNegativeFiniteOf::new(px(value)).unwrap();
        let fallback = crate::OrthogonalFallbackOf::new(
            Size::splat(Some(nn(800.0))),
            crate::NearestScrollportOf::Absent,
            Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
        );
        let context = CacheKeyContext::new();
        for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
            for known_height in [None, Some(px(10.0))] {
                let input = |operation, width| {
                    ComputeInputOf::for_child(
                        run_mode,
                        SizingMode::ContentSize,
                        RequestedAxis::Both,
                        Size::new(Some(px(20.0)), known_height),
                        Size::NONE,
                        crate::ContainingLayoutContext::new(
                            FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
                            crate::ParentFormattingContext::NoParent,
                        ),
                        SizingRequestOf::new(
                            Size::new(
                                AxisSizingRequestOf::new(
                                    AvailableSpaceOf::Definite(nn(width)),
                                    operation,
                                ),
                                AxisSizingRequestOf::new(
                                    AvailableSpaceOf::Indefinite,
                                    SizingOperation::Normal,
                                ),
                            ),
                            fallback,
                        ),
                    )
                };
                let measure = |input| {
                    crate::compute_leaf(input, &crate::NodeInputOf::<S>::default(), |observed| {
                        Ok::<_, ()>(
                            observed
                                .known_content_size()
                                .unwrap_or(Size::new(px(20.0), px(10.0))),
                        )
                    })
                    .unwrap()
                };
                let intrinsic = input(SizingOperation::MinContent, 50.0);
                let normal = input(SizingOperation::Normal, 50.0);
                let intrinsic_output = measure(intrinsic);
                let normal_output = measure(normal);
                assert_eq!(intrinsic_output.size, Size::new(px(20.0), px(10.0)));
                assert_eq!(normal_output.size, intrinsic_output.size);
                let mut cache = CacheOf::new();
                cache.store_with_context(&intrinsic, context, intrinsic_output);
                assert_eq!(
                    cache.get_with_context(&intrinsic, context),
                    Some(intrinsic_output)
                );
                // Equal imposed body width/local output cannot certify equal
                // ContentSize descendant-bound authority across original purposes.
                assert_eq!(cache.get_with_context(&normal, context), None);
                let intrinsic_space_only = input(SizingOperation::MinContent, 70.0);
                assert_eq!(
                    cache.get_with_context(&intrinsic_space_only, context),
                    Some(intrinsic_output)
                );
                cache.store_with_context(&normal, context, normal_output);
                assert_eq!(cache.get_with_context(&intrinsic, context), None);
                let normal_space_only = input(SizingOperation::Normal, 70.0);
                assert_eq!(
                    cache.get_with_context(&normal_space_only, context),
                    Some(normal_output)
                );
            }
        }
    }

    #[test]
    fn content_size_preserves_original_purpose_before_known_relaxation_f32() {
        qualify_content_size_purpose::<f32>();
    }
    #[test]
    fn content_size_preserves_original_purpose_before_known_relaxation_f64() {
        qualify_content_size_purpose::<f64>();
    }
}
