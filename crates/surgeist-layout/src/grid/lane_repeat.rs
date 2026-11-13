use super::*;
use crate::GridSpan;

use super::intrinsic_stage::IntrinsicRepeatSizing;

#[derive(Clone, Copy)]
pub(super) struct LaneRepeatExpansion<'a> {
    pub(super) axis: GridAxisKind,
    pub(super) count: usize,
    pub(super) placement: LaneRepeatPlacement<'a>,
}

#[derive(Clone, Copy)]
pub(super) enum LaneRepeatPlacement<'a> {
    Seed,
    Hypothetical(&'a [Option<GridSpan>]),
    Actual,
}

pub(super) struct LaneRepeatProbe<'input, Node, S: LayoutScalar> {
    axis: GridAxisKind,
    components: &'input [TrackComponentOf<S>],
    group_tracks: usize,
    constraint: AutoRepeatConstraint<S>,
    basis: Option<S>,
    gap: S,
    constants: Constants<S>,
    tracks: InitializedGridTracks<'input, Node, S>,
    sizing: IntrinsicRepeatSizing<Node, S>,
}

type RepeatPreparation<'input, Node, S, M> = LayoutResultOf<
    Node,
    ControlFlow<Option<LaneRepeatExpansion<'static>>, Box<LaneRepeatProbe<'input, Node, S>>>,
    S,
    M,
>;

pub(super) fn prepare_intrinsic_repeat<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    style: &GridContainerProjection<'input, Tree::Scalar>,
    constants: &Constants<Tree::Scalar>,
    parent: &GridParentContext<Tree::Scalar, Tree::Node>,
    available: LogicalSizeOf<AxisSizingRequestOf<Tree::Scalar>>,
    contained_inline: Option<Tree::Scalar>,
) -> RepeatPreparation<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    if !style.display.establishes_grid_lanes_formatting_context() {
        return Ok(ControlFlow::Break(None));
    }
    let axis = grid_axis_for_grid_lanes(style);
    let components = match axis {
        GridAxisKind::Column if parent.columns.is_none() => style.grid_template_columns,
        GridAxisKind::Row if parent.rows.is_none() => style.grid_template_rows,
        _ => return Ok(ControlFlow::Break(None)),
    };
    let Some(group_tracks) = intrinsic_auto_repeat_track_count(components)
        .map_err(|error| grid_track_sizing_error(node, error))?
    else {
        return Ok(ControlFlow::Break(None));
    };
    let used = |count| {
        Some(LaneRepeatExpansion {
            axis,
            count,
            placement: LaneRepeatPlacement::Actual,
        })
    };
    let flow = constants.flow_axes;
    let constraint = constants
        .auto_repeat_constraint(axis)
        .map_err(|status| crate::error::value_resolution_error(node, status))?;
    if matches!(constraint, AutoRepeatConstraint::One) {
        return Ok(ControlFlow::Break(used(1)));
    }
    let initial_size = constants.initial_track_basis(constants.node_inner_size);
    let initial_available = constants.initial_track_basis(constants.available_inner_size);
    let basis = track_expansion_basis(
        components,
        axis.component(flow.logical_size(initial_size)),
        axis.component(flow.logical_size(initial_available)),
    );
    let seed = initialize_grid_tracks_with_repeat(
        tree,
        node,
        style,
        constants,
        parent,
        available,
        Some(LaneRepeatExpansion {
            axis,
            count: 1,
            placement: LaneRepeatPlacement::Seed,
        }),
    )?;
    let explicit_count = match axis {
        GridAxisKind::Column => seed.context.lines.column_explicit_count,
        GridAxisKind::Row => seed.context.lines.row_explicit_count,
    };
    let mut spans = Vec::new();
    spans
        .try_reserve_exact(seed.placements.items.len())
        .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
    let mut largest = None;
    // One seed group resolves named and negative line endpoints. Its starts
    // only determine span length; every in-flow item is automatic in the probe.
    for item in seed.placements.items {
        let span = if item.in_flow {
            let placement = match axis {
                GridAxisKind::Column => item.column,
                GridAxisKind::Row => item.row,
            };
            let range =
                signed_definite_axis_range(placement, explicit_count, axis).map_err(|_| {
                    grid_track_sizing_error(node, SpanningContributionError::Allocation)
                })?;
            let length = range.map_or_else(
                || placement.span().map_or(1, GridSpan::get),
                |(start, end)| start.abs_diff(end),
            );
            let span = GridSpan::new(length).ok_or_else(|| {
                grid_track_sizing_error(node, SpanningContributionError::InvalidIncidence)
            })?;
            largest = Some(largest.map_or(length, |current: usize| current.max(length)));
            Some(span)
        } else {
            None
        };
        spans.push(span);
    }
    let Some(largest) = largest else {
        return Ok(ControlFlow::Break(used(1)));
    };
    let count = hypothetical_repeat_groups(largest, group_tracks)
        .map_err(|error| grid_track_sizing_error(node, error))?;
    let probe = initialize_grid_tracks_with_repeat(
        tree,
        node,
        style,
        constants,
        parent,
        available,
        Some(LaneRepeatExpansion {
            axis,
            count,
            placement: LaneRepeatPlacement::Hypothetical(&spans),
        }),
    )?;
    let gap = axis.component(probe.context.gap);
    let measurement_constants = if axis == GridAxisKind::Row {
        constants.for_inline_track_basis(contained_inline)
    } else {
        *constants
    };
    Ok(ControlFlow::Continue(Box::new(LaneRepeatProbe {
        axis,
        components,
        group_tracks,
        constraint,
        basis,
        gap,
        constants: measurement_constants,
        tracks: probe,
        sizing: IntrinsicRepeatSizing::new(),
    })))
}

type RepeatStep<Node, S, M> = LayoutResultOf<
    Node,
    ControlFlow<LaneRepeatExpansion<'static>, (Node, ComputeInputOf<S>)>,
    S,
    M,
>;

impl<'input, Node: Copy + Eq, S: LayoutScalar> LaneRepeatProbe<'input, Node, S> {
    pub(super) fn advance<Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        style: &GridContainerProjection<'input, S>,
        parent: &GridParentContext<S, Node>,
    ) -> RepeatStep<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let input = repeat_input(self.axis, &self.tracks, style, &self.constants, parent);
        match self.sizing.advance(tree, node, &input)? {
            ControlFlow::Continue(request) => Ok(ControlFlow::Continue(request)),
            ControlFlow::Break(phases) => self.finish(node, phases).map(ControlFlow::Break),
        }
    }

    pub(super) fn accept<M>(
        &mut self,
        node: Node,
        style: &GridContainerProjection<'input, S>,
        parent: &GridParentContext<S, Node>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let input = repeat_input(self.axis, &self.tracks, style, &self.constants, parent);
        self.sizing.accept(node, &input, output)
    }

    pub(super) fn compute<Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        style: &GridContainerProjection<'input, S>,
        parent: &GridParentContext<S, Node>,
    ) -> LayoutResultOf<Node, LaneRepeatExpansion<'static>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            match self.advance(tree, node, style, parent)? {
                ControlFlow::Continue((child, request)) => {
                    let output = tree.compute_child(child, request)?;
                    self.accept(node, style, parent, output)?;
                }
                ControlFlow::Break(expansion) => return Ok(expansion),
            }
        }
    }

    fn finish<M>(
        &self,
        node: Node,
        phases: GridAxisTrackPhases<S>,
    ) -> LayoutResultOf<Node, LaneRepeatExpansion<'static>, S, M> {
        let (origins, tracks) = match self.axis {
            GridAxisKind::Column => (
                &self.tracks.context.topology.column_origins,
                &self.tracks.column_tracks,
            ),
            GridAxisKind::Row => (
                &self.tracks.context.topology.row_origins,
                &self.tracks.row_tracks,
            ),
        };
        let mut sizes = Vec::new();
        sizes
            .try_reserve_exact(self.group_tracks)
            .map_err(|_| grid_track_sizing_error(node, SpanningContributionError::Allocation))?;
        sizes.resize(self.group_tracks, S::ZERO);
        for (((origin, track), base), limit) in origins
            .iter()
            .zip(tracks)
            .zip(phases.base_sizes)
            .zip(phases.growth_limits)
        {
            if let Some(repeated) = origin.auto_repeat {
                let operand = definite_repeat_track_size(track, self.basis)
                    .unwrap_or_else(|| limit.unwrap_or(base));
                sizes[repeated.track_index] = sizes[repeated.track_index].max(operand);
            }
        }
        let reserved = standalone_reserved_track_space(
            self.components,
            self.group_tracks,
            self.basis,
            self.gap,
        )
        .map_err(|error| grid_track_sizing_error(node, error))?;
        let count =
            auto_repeat_count_from_hypothetical(&sizes, self.constraint, self.gap, &reserved)
                .map_err(|error| grid_track_sizing_error(node, error))?;
        Ok(LaneRepeatExpansion {
            axis: self.axis,
            count,
            placement: LaneRepeatPlacement::Actual,
        })
    }
}

fn repeat_input<'input, 'a, Node, S: LayoutScalar>(
    axis: GridAxisKind,
    probe: &'a InitializedGridTracks<'input, Node, S>,
    style: &'a GridContainerProjection<'input, S>,
    constants: &'a Constants<S>,
    parent: &'a GridParentContext<S, Node>,
) -> GridTrackResolutionInput<'input, 'a, Node, S> {
    GridTrackResolutionInput {
        style,
        constants,
        column_tracks: &probe.column_tracks,
        row_tracks: &probe.row_tracks,
        minimum_functions: minimum_parent_functions(
            &probe.column_tracks,
            &probe.row_tracks,
            probe.context.percent_basis,
            parent,
        ),
        context: std::borrow::Cow::Borrowed(&probe.context),
        subgrid_report: &probe.subgrid_report,
        sizing_flow_axes: constants.flow_axes,
        available: probe.track_available,
        placements: &probe.placements,
        stage: GridSizingStage {
            first_axis: axis,
            through: GridSizingPhase::InitialFirst,
        },
    }
}
