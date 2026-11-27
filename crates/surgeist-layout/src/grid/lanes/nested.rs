//! The nested Grid Lanes intrinsic collector on a heap stack.
//!
//! A Grid Lanes container whose lane child is a subgrid wrapper collects the
//! contributions of the items below the wrapper, and below the wrappers nested
//! in it, as the lane contributions of the container. The levels of the
//! wrappers wait on the stack of a `WrapperFrames`, so the depth of the nesting
//! does not bound the collection, and every query of an item is a request the
//! request executor runs.
use super::*;
use crate::grid::subgrid::{
    FrameStep, GridMinimumWrapperInput, GridMinimumWrapperTraversal, LevelBegun, WrapperFrames,
    WrapperLevel, minimum_source_error,
};
use crate::output::SourceIndex;

impl<Node: Copy, S: LayoutScalar> LaneSeed<Node, S> {
    pub(in crate::grid) fn wrapper(&self) -> Node {
        self.wrapper
    }
}

/// What a visit of a wrapper carries beyond its level.
pub(in crate::grid) struct LaneSeed<Node, S: LayoutScalar> {
    wrapper: Node,
    projection: NestedLaneIntrinsicProjectionOf<S>,
    /// Where the memo of the wrapper is: the memo of the container's lane
    /// child, followed by one source index for each wrapper below it that has
    /// a memo of its own.
    memo_path: Vec<SourceIndex>,
    purpose: GridContributionPurpose,
}

/// The state of a collection that its frames share.
pub(in crate::grid) struct LaneEnv<'a, Node, S: LayoutScalar> {
    pub(in crate::grid) memo: &'a mut NestedLaneTraversalMemoOf<S, Node>,
    pub(in crate::grid) contributions: &'a mut LanePurposeContributions<Node, S>,
}

/// A wrapper entered, whose level is begun.
pub(in crate::grid) struct LaneEntered<'input, Node, S: LayoutScalar> {
    pub(in crate::grid) begun: LevelBegun<'input, Node, S>,
    pub(in crate::grid) seed: LaneSeed<Node, S>,
}

/// What one step of a wrapper visit leaves to do.
pub(in crate::grid) enum LaneAction<'input, Node, S: LayoutScalar> {
    /// The wrapper's items are collected.
    Done,
    /// The visit moved; step it again.
    Continue,
    /// A nested wrapper begins.
    Enter(Box<LaneEntered<'input, Node, S>>),
    /// The visit waits on a computation.
    Request(TrackChildRequest<'input, Node, S>),
}

/// What of a lane child the visit derives once and keeps while the child's
/// queries run.
struct ChildPlan<S: LayoutScalar> {
    wrapper_gap: S,
    local_projection: NestedLaneIntrinsicProjectionOf<S>,
    child_span: usize,
    explicit_range: Option<(usize, usize)>,
    explicit_local_start: Option<usize>,
    local_starts: Vec<usize>,
}

enum LeafState<S: LayoutScalar> {
    Asked,
    Answered(ComputeOutputOf<S>),
}

/// The query of an item whose contribution is outstanding.
struct LeafQuery<S: LayoutScalar> {
    query: GridMinimumQuery<S>,
    prepared: ComputeInputOf<S>,
    state: LeafState<S>,
}

/// The lane child a visit is collecting: the candidate start it is at, the
/// candidate groups of that start it has entered, and its outstanding query.
struct ChildCursor<S: LayoutScalar> {
    slot: usize,
    plan: ChildPlan<S>,
    start: usize,
    group: usize,
    leaf: Option<LeafQuery<S>>,
}

/// A wrapper whose items the collection is visiting.
pub(in crate::grid) struct NestedLaneVisit<'input, Node, S: LayoutScalar> {
    level: WrapperLevel<'input, Node, S>,
    seed: LaneSeed<Node, S>,
    slot: usize,
    cursor: Option<ChildCursor<S>>,
}

type LaneStep<'input, Node, S, M> = LayoutResultOf<Node, LaneAction<'input, Node, S>, S, M>;

type LaneEnterOutcome<'input, Node, S, M> =
    LayoutResultOf<Node, Option<LaneEntered<'input, Node, S>>, S, M>;

fn allocation_error<Node: Copy, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    grid_track_sizing_error(node, SpanningContributionError::Allocation)
}

impl<'a, Node: Copy + Eq, S: LayoutScalar> LaneEnv<'a, Node, S> {
    pub(in crate::grid) fn new(
        memo: &'a mut NestedLaneTraversalMemoOf<S, Node>,
        contributions: &'a mut LanePurposeContributions<Node, S>,
    ) -> Self {
        Self {
            memo,
            contributions,
        }
    }
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> NestedLaneVisit<'input, Node, S> {
    pub(in crate::grid) fn new(
        level: WrapperLevel<'input, Node, S>,
        seed: LaneSeed<Node, S>,
    ) -> Self {
        Self {
            level,
            seed,
            slot: 0,
            cursor: None,
        }
    }

    pub(in crate::grid) fn node(&self) -> Node {
        self.seed.wrapper
    }

    /// Accepts the output of the query the visit waits on.
    pub(in crate::grid) fn accept<M>(
        &mut self,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let node = self.seed.wrapper;
        let leaf = self
            .cursor
            .as_mut()
            .and_then(|cursor| cursor.leaf.as_mut())
            .ok_or_else(|| minimum_source_error(node))?;
        if !matches!(leaf.state, LeafState::Asked) {
            return Err(minimum_source_error(node));
        }
        leaf.state = LeafState::Answered(output);
        Ok(())
    }

    /// Visits the next candidate of the wrapper's items: enters a nested
    /// wrapper, asks for the query of an item, or records the contribution an
    /// answered query leads to.
    pub(in crate::grid) fn step<Tree, M>(
        &mut self,
        tree: &mut Tree,
        env: &mut LaneEnv<'_, Node, S>,
    ) -> LayoutResultOf<Node, LaneAction<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let wrapper = self.seed.wrapper;
        let projection = &self.seed.projection;
        if self.cursor.is_none() {
            loop {
                if self.slot >= self.level.child_slots() {
                    return Ok(LaneAction::Done);
                }
                let slot = self.slot;
                self.slot += 1;
                let Some(view) = self.level.child::<Tree, M>(slot)? else {
                    continue;
                };
                let plan = ChildPlan::new::<Node, M>(wrapper, &view, projection)?;
                self.cursor = Some(ChildCursor {
                    slot,
                    plan,
                    start: 0,
                    group: 0,
                    leaf: None,
                });
                break;
            }
        }
        let cursor = self.cursor.as_mut().expect("an active lane child");
        let view = self
            .level
            .child::<Tree, M>(cursor.slot)?
            .ok_or_else(|| minimum_source_error(wrapper))?;
        let child = view.node;
        let child_style = view.execution.child_style;
        loop {
            let Some(&start) = cursor.plan.local_starts.get(cursor.start) else {
                self.cursor = None;
                return Ok(LaneAction::Continue);
            };
            let child_span = cursor.plan.child_span;
            let end = start + child_span;
            let groups = nested_lane_candidate_groups(
                &cursor.plan.local_projection,
                child_span,
                std::iter::once(start),
                cursor.plan.explicit_local_start,
            );
            if groups.is_empty() {
                cursor.start += 1;
                cursor.group = 0;
                continue;
            }
            let mut execution = view.execution;
            match projection.axis {
                GridAxisKind::Column => {
                    execution.area.column = start;
                    execution.area.column_end = end;
                    execution.area.size.inline = view.column_geometry.span_extent(start, end);
                }
                GridAxisKind::Row => {
                    execution.area.row = start;
                    execution.area.row_end = end;
                    execution.area.size.block = view.row_geometry.span_extent(start, end);
                }
            }
            let prepared = intrinsic_stage_child_input::<Tree, M>(child, &execution)?;
            let item_report = execution
                .subgrid_item
                .ok_or_else(|| minimum_source_error(child))?;
            if let Some(child_axis) =
                inherited_subgrid_axis_for_parent_axis(item_report, projection.axis)
            {
                let Some(group) = groups.get(cursor.group) else {
                    cursor.start += 1;
                    cursor.group = 0;
                    continue;
                };
                cursor.group += 1;
                let axis_report = match child_axis {
                    GridAxisKind::Column => item_report.column,
                    GridAxisKind::Row => item_report.row,
                };
                let mut parent_context = GridParentContext {
                    columns: intrinsic_subgrid_axis_parent_context(
                        item_report.column,
                        execution.area,
                        view.minimum_source.gap,
                        view.minimum_source,
                        execution.grid.named_columns,
                        execution.grid.named_rows,
                        execution.grid.area_facts,
                    )
                    .map_err(|error| subgrid_child_context_error(child, error))?,
                    rows: intrinsic_subgrid_axis_parent_context(
                        item_report.row,
                        execution.area,
                        view.minimum_source.gap,
                        view.minimum_source,
                        execution.grid.named_columns,
                        execution.grid.named_rows,
                        execution.grid.area_facts,
                    )
                    .map_err(|error| subgrid_child_context_error(child, error))?,
                };
                if projection.hypothetical_descendants {
                    let inherited = match child_axis {
                        GridAxisKind::Column => parent_context.columns.as_mut(),
                        GridAxisKind::Row => parent_context.rows.as_mut(),
                    };
                    let placed = view.execution.area;
                    let named_range = cursor.plan.explicit_range.unwrap_or(match projection.axis {
                        GridAxisKind::Column => (placed.column, placed.column_end),
                        GridAxisKind::Row => (placed.row, placed.row_end),
                    });
                    // A clipped automatic placement can be shorter than the
                    // candidate geometry. Its local names still use the candidate span.
                    if let Some(inherited) = inherited
                        && named_range.1 - named_range.0 == child_span
                    {
                        (inherited.parent_start, inherited.parent_end) = named_range;
                    }
                }
                let wrapper_edges = lane_child_edge_facts(
                    &*tree,
                    child,
                    child_style,
                    prepared.containing_flow_axes(),
                    prepared.parent(),
                    child_axis,
                )?;
                let mut memo_path = self.seed.memo_path.clone();
                if projection.hypothetical_descendants {
                    memo_path
                        .try_reserve(1)
                        .map_err(|_| allocation_error(child))?;
                    memo_path.push(view.source_index);
                }
                let request = GridMinimumWrapperInput {
                    axis: child_axis,
                    container: execution
                        .child_container
                        .ok_or_else(|| minimum_source_error(child))?,
                    parent_context: &parent_context,
                    prepared,
                    containing_flow_axes: view.minimum_source.flow_axes,
                    content_box_estimate: None,
                    known_extent: Size::NONE,
                    traversal: GridMinimumWrapperTraversal::LaneCandidates,
                    query: execution.grid.query,
                };
                let nested = NestedLaneIntrinsicProjectionOf {
                    root_track_count: projection.root_track_count,
                    axis: child_axis,
                    wrapper_span: child_span,
                    wrapper_starts: group.starts.clone(),
                    hypothetical_descendants: projection.hypothetical_descendants,
                    envelope_empty_leaf: projection.hypothetical_descendants,
                    reversed: projection.reversed ^ axis_report.mapping.reversed,
                    parent_gap: cursor.plan.wrapper_gap,
                    interior_half_gap: cursor.plan.local_projection.interior_half_gap,
                    accumulated_edges: group.edges,
                    wrapper_edges,
                };
                return Ok(
                    match enter_lane_wrapper::<Tree, M>(
                        tree,
                        env,
                        child,
                        request,
                        nested,
                        memo_path,
                        self.seed.purpose,
                    )? {
                        Some(entered) => LaneAction::Enter(Box::new(entered)),
                        None => LaneAction::Continue,
                    },
                );
            }
            let frame = view
                .minimum_source
                .frame(child, projection.axis, execution.area, prepared)?
                .ok_or_else(|| minimum_source_error(child))?;
            let frame_prepared = frame.prepared();
            let leaf = match cursor.leaf.take() {
                None => {
                    let minimum = GridMinimumContext {
                        item: child_style,
                        flow_axes: frame.flow_axes,
                        axis: frame.axis,
                        algorithm: frame.algorithm,
                        prepared: frame_prepared,
                        margin_basis: frame_prepared.parent(),
                        span: frame.span,
                        opposite_fixed_max_span: frame.opposite_fixed_max_span,
                        subgrid_requests: lane_child_subgrid_requests(execution.child_container),
                        subgridded: Size::new(false, false),
                    };
                    let (prepared, query) =
                        prepare_lane_contribution(child, minimum, self.seed.purpose)?;
                    let input = query.input();
                    cursor.leaf = Some(LeafQuery {
                        query,
                        prepared,
                        state: LeafState::Asked,
                    });
                    return leaf_request::<Tree, M>(tree, child, execution, input);
                }
                Some(leaf) => leaf,
            };
            let LeafQuery {
                query,
                prepared: lane_prepared,
                state,
            } = leaf;
            let LeafState::Answered(output) = state else {
                return Err(minimum_source_error(child));
            };
            let output = match query.accept(child, output)? {
                ControlFlow::Continue(next) => {
                    let query = GridMinimumQuery::Final(next);
                    let input = query.input();
                    cursor.leaf = Some(LeafQuery {
                        query,
                        prepared: lane_prepared,
                        state: LeafState::Asked,
                    });
                    return leaf_request::<Tree, M>(tree, child, execution, input);
                }
                ControlFlow::Break(output) => output,
            };
            let value = lane_contribution_value(
                child,
                child_style,
                frame.flow_axes,
                frame.axis,
                lane_prepared,
                output,
            )?;
            for group in groups {
                let adjustment = group.edges.start_mbp
                    + group.edges.end_mbp
                    + group.edges.start_half_gap
                    + group.edges.end_half_gap;
                let adjusted = value + adjustment;
                if !adjusted.is_finite() {
                    return Err(grid_track_sizing_error(
                        child,
                        SpanningContributionError::InvalidNumeric { value: adjusted },
                    ));
                }
                for group_start in group.starts {
                    env.contributions
                        .rows
                        .try_reserve(1)
                        .map_err(|_| allocation_error(child))?;
                    env.contributions.rows.push(LanePurposeContributionOf {
                        source: child,
                        span: GridTrackSpan::new(group_start + 1, group_start + child_span + 1),
                        value: adjusted.max(S::ZERO),
                    });
                }
            }
            cursor.start += 1;
            cursor.group = 0;
        }
    }
}

/// The request of an item's query in a nested wrapper.
fn leaf_request<'input, 'a, Tree, M>(
    tree: &Tree,
    child: Tree::Node,
    execution: IntrinsicGridChildInput<'input, 'a, Tree::Node, Tree::Scalar>,
    input: ComputeInputOf<Tree::Scalar>,
) -> LaneStep<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    Ok(LaneAction::Request(
        match prepare_intrinsic_grid_child_request::<Tree, M>(tree, child, execution, input)? {
            IntrinsicChildRequest::Child(input) => TrackChildRequest::child(child, input),
            IntrinsicChildRequest::Context(request) => TrackChildRequest::context(request),
        },
    ))
}

impl<S: LayoutScalar> ChildPlan<S> {
    /// What the visit of a wrapper's child derives from the child's view and
    /// the wrapper's projection.
    fn new<Node: Copy + Eq, M>(
        wrapper: Node,
        view: &GridMinimumWrapperChild<'_, '_, Node, S>,
        projection: &NestedLaneIntrinsicProjectionOf<S>,
    ) -> LayoutResultOf<Node, Self, S, M> {
        let child = view.node;
        // Inherited track geometry keeps the ancestor gutters. Authored gap
        // differences remain edge adjustments, resolved in this wrapper's
        // actual property frame rather than in the outer Lane container.
        let gap = child::child_subgrid_gap(
            view.execution.grid.style,
            projection.axis,
            view.execution.grid.constants.node_inner_size,
        )
        .map_err(|status| crate::error::value_resolution_error(wrapper, status))?;
        let wrapper_gap = match gap {
            ResolvedSubgridGap::Normal => projection.parent_gap,
            ResolvedSubgridGap::Length(value) => value,
        };
        let mut local_projection = projection.clone();
        local_projection.wrapper_edges =
            lane_wrapper_edges_with_gutter(projection, view.execution.grid.constants);
        let half_gap_difference = (wrapper_gap - projection.parent_gap) / S::from_f64(2.0);
        local_projection.interior_half_gap = projection.interior_half_gap + half_gap_difference;
        let placement = view.execution.grid.placements.items[view.source_index.get()];
        let placement = match projection.axis {
            GridAxisKind::Column => placement.column,
            GridAxisKind::Row => placement.row,
        };
        let explicit_range =
            signed_definite_axis_range(placement, projection.wrapper_span, projection.axis)
                .map_err(|error| grid_placement_demand_error(child, error))?
                .map(|(start, end)| {
                    clamp_subgrid_axis_range(
                        start.max(0) as usize,
                        end.max(0) as usize,
                        projection.wrapper_span,
                    )
                });
        let child_span = explicit_range.map_or_else(
            || {
                placement
                    .span()
                    .map_or(1, crate::value::GridSpan::get)
                    .min(projection.wrapper_span)
            },
            |(start, end)| end - start,
        );
        let explicit_local_start = explicit_range.map(|(start, _)| start);
        let local_starts = if let Some(start) = explicit_local_start
            && !projection.hypothetical_descendants
        {
            vec![start]
        } else {
            candidate_starts(projection.wrapper_span, child_span).collect()
        };
        Ok(Self {
            wrapper_gap,
            local_projection,
            child_span,
            explicit_range,
            explicit_local_start,
            local_starts,
        })
    }
}

/// Begins the visit of a wrapper of the collection, or settles it at once: a
/// wrapper that no item can start in, that repeats a visit already made, or
/// that contains nothing to visit.
pub(in crate::grid) fn enter_lane_wrapper<'input, 'a, Tree, M>(
    tree: &mut Tree,
    env: &mut LaneEnv<'_, Tree::Node, Tree::Scalar>,
    wrapper: Tree::Node,
    request: GridMinimumWrapperInput<'input, 'a, Tree::Node, Tree::Scalar>,
    projection: NestedLaneIntrinsicProjectionOf<Tree::Scalar>,
    memo_path: Vec<SourceIndex>,
    purpose: GridContributionPurpose,
) -> LaneEnterOutcome<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    if projection.wrapper_starts.is_empty() || projection.wrapper_span == 0 {
        return Ok(None);
    }
    if projection.hypothetical_descendants {
        let memo = env
            .memo
            .at(&memo_path)
            .map_err(|_| allocation_error(wrapper))?;
        if !memo
            .record(&request, &projection)
            .map_err(|_| allocation_error(wrapper))?
        {
            return Ok(None);
        }
    }
    if grid_minimum_wrapper_contained(tree.node_input(wrapper), projection.axis)
        || tree.children(wrapper).all(|child| {
            !is_in_flow_grid_child(GridChildInput::from_node(tree.node_input(child)).item())
        })
    {
        let memo = env
            .memo
            .at(&memo_path)
            .map_err(|_| allocation_error(wrapper))?;
        collect_empty_lane_subgrid_edges(
            &*tree,
            wrapper,
            &request,
            &projection,
            memo,
            &mut env.contributions.base_floors,
        )?;
        return Ok(None);
    }
    Ok(
        match WrapperLevel::begin::<Tree, M>(tree, wrapper, request)? {
            LevelBegun::Empty => None,
            begun => Some(LaneEntered {
                begun,
                seed: LaneSeed {
                    wrapper,
                    projection,
                    memo_path,
                    purpose,
                },
            }),
        },
    )
}

/// The lane child of a Grid Lanes container that is a subgrid wrapper in the
/// lane axis, with what its starts share.
pub(in crate::grid) struct LaneRootWrapper<S: LayoutScalar> {
    pub(in crate::grid) source_slot: usize,
    pub(in crate::grid) child_axis: GridAxisKind,
    pub(in crate::grid) wrapper_span: usize,
    pub(in crate::grid) starts: Vec<usize>,
    pub(in crate::grid) hypothetical_descendants: bool,
    pub(in crate::grid) reversed: bool,
    pub(in crate::grid) wrapper_edges: LaneIntrinsicEdgeFactsOf<S>,
    pub(in crate::grid) prepared: ComputeInputOf<S>,
}

/// The collection of the contributions of one lane child that is a subgrid
/// wrapper: its root starts one after the other, each with the nested
/// wrappers below it on a heap stack.
pub(in crate::grid) struct NestedLaneCollector<'input, Node, S: LayoutScalar> {
    root: LaneRootWrapper<S>,
    next_start: usize,
    frames: WrapperFrames<'input, Node, S>,
    memo: NestedLaneTraversalMemoOf<S, Node>,
    /// The fold of the traversals the wrappers' preparations start is their
    /// own; the stack takes this one only for the traversal it has none for.
    outer: SubgridIntrinsicFold<'input, Node, S>,
}

/// What a step of the collection leaves to do.
pub(in crate::grid) enum CollectorStep<'input, Node, S: LayoutScalar> {
    Request(TrackChildRequest<'input, Node, S>),
    Done,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> NestedLaneCollector<'input, Node, S> {
    pub(in crate::grid) fn new(root: LaneRootWrapper<S>) -> Self {
        Self {
            root,
            next_start: 0,
            frames: WrapperFrames::new(),
            memo: NestedLaneTraversalMemoOf::new(),
            outer: SubgridIntrinsicFold::new(IntrinsicMinTrackFacts::Known(&[])),
        }
    }

    /// Accepts the output of the computation the collection waits on, and the
    /// gutter it observed.
    pub(in crate::grid) fn accept<M>(
        &mut self,
        output: ComputeOutputOf<S>,
        gutter: Option<Edges<S>>,
    ) -> LayoutResultOf<Node, (), S, M> {
        self.frames.accept::<M>(output, gutter)
    }

    /// Takes the frames and adds to `starts` where the effect capture of each
    /// waiting preparation began.
    pub(in crate::grid) fn take_captures(&mut self, starts: &mut Vec<usize>) {
        self.frames.take_captures(starts);
    }

    /// Raises the floors of the tracks by the empty descendants the collection
    /// met.
    pub(in crate::grid) fn finish(self, floors: &mut [S]) {
        self.memo.emit_empty_leaf_floors(floors);
    }

    /// The next computation the collection waits on, or `Done` once every
    /// start of the wrapper is collected.
    pub(in crate::grid) fn step<Tree, M>(
        &mut self,
        tree: &mut Tree,
        input: LaneIntrinsicTrackSizeInput<'input, '_, Node, S>,
        purpose: GridContributionPurpose,
        contributions: &mut LanePurposeContributions<Node, S>,
    ) -> LayoutResultOf<Node, CollectorStep<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            let mut env = LaneEnv::new(&mut self.memo, contributions);
            match self
                .frames
                .step::<Tree, M>(tree, &mut self.outer, Some(&mut env))?
            {
                FrameStep::Request(request) => return Ok(CollectorStep::Request(request)),
                FrameStep::Continue => {}
                FrameStep::Idle => {
                    let Some(&start) = self.root.starts.get(self.next_start) else {
                        return Ok(CollectorStep::Done);
                    };
                    self.next_start += 1;
                    if let Some(entered) = enter_root_start::<Tree, M>(
                        tree, input, &self.root, start, &mut env, purpose,
                    )? {
                        self.frames.enter_lane::<M>(entered)?;
                    }
                }
            }
        }
    }
}

/// Enters the wrapper of the lane child at one of its starts.
fn enter_root_start<'input, Tree, M>(
    tree: &mut Tree,
    input: LaneIntrinsicTrackSizeInput<'input, '_, Tree::Node, Tree::Scalar>,
    root: &LaneRootWrapper<Tree::Scalar>,
    start: usize,
    env: &mut LaneEnv<'_, Tree::Node, Tree::Scalar>,
    purpose: GridContributionPurpose,
) -> LaneEnterOutcome<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let LaneIntrinsicTrackSizeInput {
        grid,
        constants,
        axis,
        tracks,
        gap,
        gutters,
        placements,
        subgrid_report,
        ..
    } = input;
    let child = placements.children[root.source_slot];
    let item_report = subgrid_report
        .items
        .get(root.source_slot)
        .copied()
        .ok_or_else(|| minimum_source_error(child))?;
    let axis_report = match root.child_axis {
        GridAxisKind::Column => item_report.column,
        GridAxisKind::Row => item_report.row,
    };
    let functions = grid
        .minimum_functions
        .for_axis(axis)
        .ok_or_else(|| minimum_source_error(child))?;
    let geometry = gutters.map_or_else(
        || {
            UsedGridAxisGeometryOf::new(
                vec![Tree::Scalar::ZERO; tracks.len()],
                vec![false; tracks.len()],
                gap,
            )
        },
        |gutters| {
            UsedGridAxisGeometryOf::from_sizing_gutters(
                vec![Tree::Scalar::ZERO; tracks.len()],
                gutters,
            )
        },
    );
    let named_lines = match axis {
        GridAxisKind::Column => grid.named_columns,
        GridAxisKind::Row => grid.named_rows,
    };
    let inherited = intrinsic_subgrid_axis_parent_context_for_span(
        axis_report,
        GridTrackSpan::new(start + 1, start + root.wrapper_span + 1),
        gap,
        SubgridInheritedAxisSource {
            baseline_reference: grid_baseline_reference(grid.style.baseline_references, axis),
            geometry: Some(&geometry),
            allocated: match axis {
                GridAxisKind::Column => grid.query.columns,
                GridAxisKind::Row => grid.query.rows,
            },
            minimum_functions: Some(functions),
        },
        named_lines,
        grid.area_facts,
    )
    .map_err(|error| subgrid_child_context_error(child, error))?;
    let mut parent_context = GridParentContext::none();
    match root.child_axis {
        GridAxisKind::Column => parent_context.columns = inherited,
        GridAxisKind::Row => parent_context.rows = inherited,
    }
    let request = GridMinimumWrapperInput {
        axis: root.child_axis,
        container: placements
            .child_input(root.source_slot)
            .nested_container()
            .ok_or_else(|| minimum_source_error(child))?,
        parent_context: &parent_context,
        prepared: root.prepared,
        containing_flow_axes: constants.flow_axes,
        content_box_estimate: None,
        known_extent: Size::NONE,
        traversal: GridMinimumWrapperTraversal::LaneCandidates,
        query: grid.query,
    };
    let projection = NestedLaneIntrinsicProjectionOf {
        root_track_count: tracks.len(),
        axis: root.child_axis,
        wrapper_span: root.wrapper_span,
        wrapper_starts: vec![start],
        hypothetical_descendants: root.hypothetical_descendants,
        envelope_empty_leaf: false,
        reversed: root.reversed,
        parent_gap: gap,
        interior_half_gap: Tree::Scalar::ZERO,
        accumulated_edges: LaneIntrinsicEdgeFactsOf::default(),
        wrapper_edges: root.wrapper_edges,
    };
    enter_lane_wrapper::<Tree, M>(tree, env, child, request, projection, Vec::new(), purpose)
}
