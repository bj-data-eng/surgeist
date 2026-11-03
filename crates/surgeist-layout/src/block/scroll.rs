use super::{BlockChildProjection, Constants};
use crate::error::layout_own_geometry_error;
use crate::geometry::PhysicalSide;
use crate::scroll::{
    CanonicalRetainedScrollSourceOf, CanonicalScrollBoxSourceOf, CanonicalScrollGeometryErrorOf,
    CanonicalScrollRangeSeedPolicy, CanonicalScrollSourceBuilderOf, ScrollBoxProjection,
    ScrollContributionAccumulatorOf, ScrollContributionErrorOf, ScrollOriginAxes,
    ScrollOriginProgression, ScrollTargetProjection, canonical_scroll_box_from_source,
    scrollbar_size_from_overflow,
};
use crate::{
    Compute, Edges, FlowAxes, LayoutResultOf, LayoutScalar, LogicalAxis, Point, RunMode,
    ScrollGeometryOf, ScrollRectOf, Size, Traverse,
};

/// Accepted Block contributions and their directed terminal content edges.
/// Descendant/absolute overflow belongs only to geometry, never to these ends.
pub(super) struct BlockScrollContributions<S: LayoutScalar> {
    pub(super) geometry: ScrollContributionAccumulatorOf<S>,
    axes: FlowAxes,
    inline_end: Option<S>,
    block_end: Option<S>,
}

impl<S: LayoutScalar> BlockScrollContributions<S> {
    pub(super) fn new(axes: FlowAxes, seed: ScrollRectOf<S>) -> Self {
        Self {
            geometry: ScrollContributionAccumulatorOf::new(seed),
            axes,
            inline_end: None,
            block_end: None,
        }
    }

    pub(super) fn include_in_flow_geometry(
        &mut self,
        location: Point<S>,
        margin: Edges<S>,
        geometry: ScrollGeometryOf<S>,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        self.geometry
            .include_in_flow_geometry(location, margin, geometry)?;
        let border = geometry.border_box();
        if border.size().width > S::ZERO && border.size().height > S::ZERO {
            let rect = ScrollRectOf::try_new(
                Point::new(
                    location.x + border.origin().x,
                    location.y + border.origin().y,
                ),
                border.size(),
            )?;
            self.include_rect_end(LogicalAxis::Inline, rect, margin)?;
            self.include_rect_end(LogicalAxis::Block, rect, margin)?;
        }
        Ok(())
    }

    pub(super) fn include_line(
        &mut self,
        rect: ScrollRectOf<S>,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        // A genuine line allocation remains a line even with a zero dimension.
        self.geometry.include_direct_line(rect);
        self.include_rect_end(LogicalAxis::Inline, rect, Edges::ZERO)?;
        self.include_rect_end(LogicalAxis::Block, rect, Edges::ZERO)
    }

    pub(super) fn include_inline_strip(
        &mut self,
        location: Point<S>,
        size: Size<S>,
        margin: Edges<S>,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        self.include_rect_end(
            LogicalAxis::Inline,
            ScrollRectOf::try_new(location, size)?,
            margin,
        )
    }

    fn include_rect_end(
        &mut self,
        axis: LogicalAxis,
        rect: ScrollRectOf<S>,
        margin: Edges<S>,
    ) -> Result<(), ScrollContributionErrorOf<S>> {
        let side = match axis {
            LogicalAxis::Inline => self.axes.inline_end(),
            LogicalAxis::Block => self.axes.block_end(),
        };
        // Optional margin outsets cannot retract the actual border endpoint.
        let candidate = match side {
            PhysicalSide::Top => rect.origin().y - margin.top.max(S::ZERO),
            PhysicalSide::Right => rect.origin().x + rect.size().width + margin.right.max(S::ZERO),
            PhysicalSide::Bottom => {
                rect.origin().y + rect.size().height + margin.bottom.max(S::ZERO)
            }
            PhysicalSide::Left => rect.origin().x - margin.left.max(S::ZERO),
        };
        if !candidate.is_finite() {
            return Err(ScrollContributionErrorOf::NonFiniteFinalInFlowEnd {
                axis,
                value: candidate,
            });
        }
        let end = match axis {
            LogicalAxis::Inline => &mut self.inline_end,
            LogicalAxis::Block => &mut self.block_end,
        };
        *end = Some(end.map_or(candidate, |current| match side {
            PhysicalSide::Top | PhysicalSide::Left => current.min(candidate),
            PhysicalSide::Right | PhysicalSide::Bottom => current.max(candidate),
        }));
        Ok(())
    }

    fn publish_ends(&mut self) -> Result<(), ScrollContributionErrorOf<S>> {
        for (axis, coordinate) in [
            (LogicalAxis::Inline, self.inline_end),
            (LogicalAxis::Block, self.block_end),
        ] {
            if let Some(coordinate) = coordinate {
                self.geometry
                    .record_final_in_flow_end(self.axes, axis, coordinate)?;
            }
        }
        Ok(())
    }
}

pub(super) fn prepare_scroll_contributions<Node, S, M>(
    node: Node,
    run_mode: RunMode,
    projection: ScrollBoxProjection<'_, S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    mut contributions: BlockScrollContributions<S>,
) -> LayoutResultOf<Node, BlockScrollContributions<S>, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    let final_scroll_box =
        canonical_scroll_box_from_source(CanonicalScrollBoxSourceOf::from_projection(
            projection,
            output_size,
            constants.border,
            constants.padding,
            constants.settled_auto_scrollbars,
        ))
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    contributions
        .geometry
        .replace_container_seed(final_scroll_box.padding_box());
    contributions.geometry.exclude_reserved_gutter_from_range();
    Ok(contributions)
}

pub(super) struct PublishedScrollOf<S: LayoutScalar> {
    pub(super) geometry: ScrollGeometryOf<S>,
    pub(super) content_size: Size<S>,
}

pub(super) fn finish_scroll_geometry<'input, Tree, S, M>(
    node: <Tree as Traverse>::Node,
    run_mode: RunMode,
    box_projection: ScrollBoxProjection<'_, S>,
    target_projection: ScrollTargetProjection<'_, S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    mut contributions: BlockScrollContributions<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, PublishedScrollOf<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    contributions
        .publish_ends()
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    contributions
        .geometry
        .include_terminal_padding(constants.padding)
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    let scroll_geometry = block_scroll_geometry::<Tree, S, M>(
        node,
        run_mode,
        box_projection,
        target_projection,
        constants,
        output_size,
        contributions.geometry,
    )?;
    let content_size = contributions
        .geometry
        .content_size_from_anchor(scroll_geometry.content_box().origin())
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    Ok(PublishedScrollOf {
        geometry: scroll_geometry,
        content_size,
    })
}

fn block_scroll_geometry<'input, Tree, S, M>(
    node: <Tree as Traverse>::Node,
    run_mode: RunMode,
    box_projection: ScrollBoxProjection<'_, S>,
    target_projection: ScrollTargetProjection<'_, S>,
    constants: &Constants<S>,
    output_size: Size<S>,
    contributions: ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ScrollGeometryOf<S>, S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let target_border_box = ScrollRectOf::try_new(Point::ZERO, output_size)
        .map_err(|error| layout_own_geometry_error(node, run_mode, error))?;
    CanonicalScrollSourceBuilderOf::for_node(
        box_projection,
        target_projection,
        output_size,
        constants.border,
        constants.padding,
        constants.settled_auto_scrollbars,
        ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
    )
    .geometry_from_contributions(contributions, target_border_box)
    .map_err(|error| layout_own_geometry_error(node, run_mode, error))
}

pub(super) fn retained_child_scroll_geometry<S: LayoutScalar>(
    box_projection: ScrollBoxProjection<'_, S>,
    target_projection: ScrollTargetProjection<'_, S>,
    size: Size<S>,
    content_size: Size<S>,
    padding: Edges<S>,
    border: Edges<S>,
    child_compute_geometry: Option<ScrollGeometryOf<S>>,
) -> Result<ScrollGeometryOf<S>, CanonicalScrollGeometryErrorOf<S>> {
    let settled_auto_scrollbars = crate::scroll::SettledAutoScrollbarState::INITIAL;
    let source = match child_compute_geometry {
        Some(ref geometry) => CanonicalRetainedScrollSourceOf::Existing(geometry),
        None => CanonicalRetainedScrollSourceOf::Reconstruct { content_size },
    };
    CanonicalScrollSourceBuilderOf::for_node(
        box_projection,
        target_projection,
        size,
        border,
        padding,
        settled_auto_scrollbars,
        ScrollOriginAxes::new(
            ScrollOriginProgression::FlowEndward,
            ScrollOriginProgression::FlowEndward,
        ),
    )
    .geometry_from_retained_source(
        source,
        CanonicalScrollRangeSeedPolicy::ExcludeReservedGutter,
    )
}

pub(super) fn child_scrollbar_size<S: LayoutScalar>(
    style: &BlockChildProjection<'_, S>,
) -> Size<S> {
    scrollbar_size_from_overflow(
        style.common.overflow,
        style.common.item_is_replaced,
        style.scrollbar_width.get(),
    )
}
