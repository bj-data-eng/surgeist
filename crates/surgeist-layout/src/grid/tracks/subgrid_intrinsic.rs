use super::*;

use crate::geometry::FlowAxes;

#[derive(Clone, Copy)]
pub(in crate::grid) struct IntrinsicGridChildInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    pub(in crate::grid) child_style: &'a GridItemProjection<'input, S>,
    pub(in crate::grid) child_container: Option<&'a GridContainerInput<'input, S>>,
    pub(in crate::grid) grid: IntrinsicGrid<'input, 'a, Node, S>,
    pub(in crate::grid) area: GridArea<S>,
    pub(in crate::grid) columns: &'a [S],
    pub(in crate::grid) rows: &'a [S],
    pub(in crate::grid) subgrid_item: Option<SubgridItemReport<Node>>,
    pub(in crate::grid) input: ComputeInputOf<S>,
}

pub(super) fn compute_intrinsic_grid_child<'input, Tree, M>(
    tree: &mut Tree,
    child: <Tree as Traverse>::Node,
    args: IntrinsicGridChildInput<'input, '_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    prepare_intrinsic_grid_child::<Tree, M>(child, &args)?.compute(child, |input| {
        compute_prepared_intrinsic_grid_child(tree, child, args, input)
    })
}

pub(super) fn prepare_intrinsic_grid_child<'input, Tree, M>(
    child: Tree::Node,
    args: &IntrinsicGridChildInput<'input, '_, Tree::Node, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, GridMinimumQuery<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let prepared = intrinsic_stage_child_input::<Tree, M>(child, args)?;
    if args.grid.query.purpose != GridContributionPurpose::Minimum {
        return Ok(GridMinimumQuery::Final(prepared));
    }
    let axis = args
        .grid
        .query
        .axis
        .expect("minimum collection names its queried axis");
    let grid = args.grid;
    let (
        start,
        end,
        opposite_start,
        opposite_end,
        opposite_axis,
        gap,
        opposite_gap,
        gutters,
        opposite_gutters,
    ) = match axis {
        GridAxisKind::Column => (
            args.area.column,
            args.area.column_end,
            args.area.row,
            args.area.row_end,
            GridAxisKind::Row,
            grid.gap.inline,
            grid.gap.block,
            grid.column_gutters,
            grid.row_gutters,
        ),
        GridAxisKind::Row => (
            args.area.row,
            args.area.row_end,
            args.area.column,
            args.area.column_end,
            GridAxisKind::Column,
            grid.gap.block,
            grid.gap.inline,
            grid.row_gutters,
            grid.column_gutters,
        ),
    };
    let missing_functions = || {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(child),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridTrackInheritance),
        )
    };
    let functions = grid
        .minimum_functions
        .for_axis(axis)
        .and_then(|functions| functions.mapped_span(start, end, false))
        .ok_or_else(missing_functions)?;
    let span_gutter = gutters.map_or_else(
        || gap * Tree::Scalar::from_usize(end.saturating_sub(start + 1)),
        |gutters| gutters.span_gap_total(start, end),
    );
    let opposite_fixed_max_span = if grid
        .style
        .display
        .establishes_grid_lanes_formatting_context()
        && axis == grid_axis_for_grid_lanes(grid.style)
    {
        // The opposite Lane stacking axis has no Grid track maximum span.
        None
    } else {
        let opposite_functions = grid
            .minimum_functions
            .for_axis(opposite_axis)
            .and_then(|functions| functions.mapped_span(opposite_start, opposite_end, false))
            .ok_or_else(missing_functions)?;
        let opposite_span_gutter = opposite_gutters.map_or_else(
            || {
                opposite_gap
                    * Tree::Scalar::from_usize(opposite_end.saturating_sub(opposite_start + 1))
            },
            |gutters| gutters.span_gap_total(opposite_start, opposite_end),
        );
        opposite_functions
            .span_facts(opposite_span_gutter)
            .map_err(|error| sizing_resolution_error(child, error))?
            .fixed_max_span()
    };
    let subgrid_requests = args
        .child_container
        .map_or(Size::new(false, false), |container| {
            let projection = container.projection();
            let requests = |tracks: &[TrackComponentOf<Tree::Scalar>]| {
                tracks
                    .iter()
                    .any(|track| matches!(track, TrackComponentOf::Subgrid(_)))
            };
            Size::new(
                requests(projection.grid_template_columns),
                requests(projection.grid_template_rows),
            )
        });
    let margin_basis = match axis {
        GridAxisKind::Column => grid
            .column_tracks
            .get(start..end)
            .and_then(|tracks| grid.column_contribution_margin_basis(tracks, prepared.parent()))
            .unwrap_or(prepared.parent()),
        GridAxisKind::Row => prepared.parent(),
    };
    GridMinimumQuery::begin(
        child,
        GridMinimumContext {
            item: args.child_style,
            flow_axes: grid.sizing_flow_axes,
            axis,
            prepared,
            margin_basis,
            span: functions
                .span_facts(span_gutter)
                .map_err(|error| sizing_resolution_error(child, error))?,
            opposite_fixed_max_span,
            subgrid_requests,
            algorithm: sizing_algorithm_for_grid_display(grid.style.display),
        },
    )
}

// The real child/subgrid query is shared by content suggestions and the final
// used-minimum query; neither route synthesizes a measured output.
pub(in crate::grid) fn compute_prepared_intrinsic_grid_child<'input, Tree, M>(
    tree: &mut Tree,
    child: Tree::Node,
    args: IntrinsicGridChildInput<'input, '_, Tree::Node, Tree::Scalar>,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, ComputeOutputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let input = grid_item_baseline_demand(
        args.grid.style,
        args.child_style,
        args.grid.query.axis,
        input,
    )
    .map_err(|error| grid_baseline_error_at_site(LayoutErrorSiteOf::Node(child), error))?;
    let IntrinsicGridChildInput {
        child_style,
        child_container,
        grid,
        area,
        columns,
        rows,
        subgrid_item,
        ..
    } = args;

    let Some(subgrid_item) = subgrid_item else {
        return tree.compute_child(child, input);
    };
    if !subgrid_item.needs_context() {
        return tree.compute_child(child, input);
    }
    let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
    let column_constraint =
        intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
            report: subgrid_item.column,
            area,
            parent_flow_axes: grid.constants.flow_axes,
            child_flow_axes,
            explicit_parent_content_size: grid.constants.explicit_definite_content_size,
            parent_column_count: columns.len(),
            parent_row_count: rows.len(),
            tracks: grid.column_tracks,
            gap: grid.gap.inline,
        });
    let row_constraint = intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
        report: subgrid_item.row,
        area,
        parent_flow_axes: grid.constants.flow_axes,
        child_flow_axes,
        explicit_parent_content_size: grid.constants.explicit_definite_content_size,
        parent_column_count: columns.len(),
        parent_row_count: rows.len(),
        tracks: grid.row_tracks,
        gap: grid.gap.block,
    });
    let mut physical_area_size = grid_area_physical_size(grid.constants.flow_axes, area.size);
    apply_resolved_intrinsic_subgrid_area_constraints(
        &mut physical_area_size,
        [column_constraint, row_constraint],
    );
    let sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
        child,
        child_style,
        grid.style,
        Some(subgrid_item),
        physical_area_size,
        physical_area_size.map(Some),
        grid.sizing_flow_axes,
    )?;
    let input = intrinsic_subgrid_child_input(input, sizing, [column_constraint, row_constraint])
        .map_err(|error| sizing_resolution_error(child, error))?;
    if !matches!(
        child_style.display.inner_display(),
        Display::Grid | Display::GridLanes
    ) {
        return tree.compute_child(child, input);
    }
    let child_container = child_container
        .expect("an intrinsic subgrid child must retain its container projection input");
    let child_container_style = child_container.projection();
    let needs_context =
        needs_intrinsic_subgrid_context(&child_container_style, child_style, subgrid_item, area);
    if !input.run_mode().is_perform_layout()
        && !needs_context
        && !child_container.requires_context()
        && grid.query.stage.through == GridSizingPhase::Complete
    {
        return tree.compute_child(child, input);
    }

    let area_parent = physical_area_size.map(Some);
    let padding = grid
        .constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.padding,
            area_parent,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let border = grid
        .constants
        .flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.border,
            area_parent,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let margin = sizing
        .unresolved_margin
        .map(|margin| margin.unwrap_or(Tree::Scalar::ZERO));
    let content_box_size =
        (physical_area_size - margin.sum_axes() - padding.sum_axes() - border.sum_axes())
            .max(Size::ZERO);
    let child_context_input = SubgridChildParentContextInput {
        minimum_functions: grid.minimum_functions,
        parent_baseline_references: grid.style.baseline_references,
        item: subgrid_item,
        child_style,
        child_container: Some(*child_container),
        area,
        content_box_size,
        columns,
        rows,
        gap: grid.gap,
        parent_named_columns: grid.named_columns,
        parent_named_rows: grid.named_rows,
        parent_area_facts: grid.area_facts,
        margin: sizing.unresolved_margin,
        border,
        padding,
        scrollbar_gutter: Edges::ZERO,
    };
    if grid.query.stage.through != GridSizingPhase::Complete {
        let observation = tree.observe_grid_overflow(
            child,
            GridOverflowProbe {
                layout: GridOverflowLayout::Intrinsic(GridIntrinsicProbe {
                    input,
                    stage: grid.query.stage,
                    kind: GridIntrinsicProbeKind::AllocatedPrefix,
                    intrinsic_minimum: sizing.standalone_intrinsic_minimum,
                    source: child_context_input,
                    parent_column_geometry: None,
                    parent_row_geometry: None,
                    parent_flow: grid.constants.flow_axes,
                }),
            },
        )?;
        return Ok(match observation {
            GridOverflowObservation::Intrinsic(observation) => observation.output,
            GridOverflowObservation::Children(_) => {
                unreachable!("a bounded intrinsic layout returns its prefix")
            }
        });
    }
    Ok(compute_grid_with_context_source(
        tree,
        child,
        input,
        sizing.standalone_intrinsic_minimum,
        child_context_input.container(),
        grid.query
            .stage
            .mapped(grid.constants.flow_axes, child_flow_axes),
        |constants| {
            subgrid_child_parent_context(child_context_input.for_scrollbar_pass(constants))
                .map_err(|error| subgrid_child_context_error(child, error))
        },
    )?
    .output)
}

// A prepared opposite track span is a per-item containing area, not the
// aggregate container availability. The queried dimension stays intrinsic.
pub(in crate::grid) fn intrinsic_stage_child_input<'input, Tree, M>(
    child: Tree::Node,
    args: &IntrinsicGridChildInput<'input, '_, Tree::Node, Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, ComputeInputOf<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let grid = args.grid;
    if grid
        .style
        .display
        .establishes_grid_lanes_formatting_context()
        && grid.query.axis == Some(grid_axis_for_grid_lanes(grid.style))
    {
        return prepare_grid_lanes_intrinsic_child_input(
            args.child_style,
            grid.sizing_flow_axes,
            grid_axis_for_grid_lanes(grid.style),
            grid.query,
            args.input,
        )
        .map_err(|error| sizing_resolution_error(child, error));
    }

    let (opposite, estimate) = match grid.query.axis {
        Some(GridAxisKind::Row) if grid.query.columns.is_some() => (GridAxisKind::Column, None),
        Some(GridAxisKind::Column) if grid.query.rows.is_some() => (GridAxisKind::Row, None),
        Some(GridAxisKind::Row) => match grid.initial_opposite_span(
            GridAxisKind::Column,
            args.area.column,
            args.area.column_end,
        ) {
            Some(extent) => (GridAxisKind::Column, Some(extent)),
            None => return Ok(args.input),
        },
        Some(GridAxisKind::Column) => {
            match grid.initial_opposite_span(GridAxisKind::Row, args.area.row, args.area.row_end) {
                Some(extent) => (GridAxisKind::Row, Some(extent)),
                None => return Ok(args.input),
            }
        }
        None => return Ok(args.input),
    };
    let mut logical_area = args.area.size;
    if let Some(extent) = estimate {
        match opposite {
            GridAxisKind::Column => logical_area.inline = extent,
            GridAxisKind::Row => logical_area.block = extent,
        }
    }
    let area = grid_area_physical_size(grid.sizing_flow_axes, logical_area);
    let sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
        child,
        args.child_style,
        grid.style,
        args.subgrid_item,
        area,
        area.map(Some),
        grid.sizing_flow_axes,
    )?;
    let flow = grid.sizing_flow_axes;
    let mut known = flow.logical_size(args.input.known());
    let mut parent = flow.logical_size(args.input.parent());
    let mut available = flow.logical_size(args.input.sizing_request().axes());
    let resolved_known = flow.logical_size(sizing.known);
    let resolved_available = flow.logical_size(sizing.available);
    let parent_area = flow.logical_size(area);
    match opposite {
        GridAxisKind::Column => {
            known.inline = resolved_known.inline;
            parent.inline = Some(parent_area.inline);
            available.inline = AxisSizingRequestOf::new(
                crate::layout_math::checked_available_space(resolved_available.inline).map_err(
                    |_| {
                        crate::error::value_resolution_error(
                            child,
                            LengthResolutionStatus::InvalidNumeric {
                                value: resolved_available.inline,
                            },
                        )
                    },
                )?,
                SizingOperation::Normal,
            );
        }
        GridAxisKind::Row => {
            known.block = resolved_known.block;
            parent.block = Some(parent_area.block);
            available.block = AxisSizingRequestOf::new(
                crate::layout_math::checked_available_space(resolved_available.block).map_err(
                    |_| {
                        crate::error::value_resolution_error(
                            child,
                            LengthResolutionStatus::InvalidNumeric {
                                value: resolved_available.block,
                            },
                        )
                    },
                )?,
                SizingOperation::Normal,
            );
        }
    }
    Ok(ComputeInputOf::for_child(
        args.input.run_mode(),
        args.input.sizing_mode(),
        args.input.requested_axis(),
        flow.physical_size(known),
        flow.physical_size(parent),
        args.input.containing_layout_context(),
        args.input
            .sizing_request()
            .with_axes(flow.physical_size(available)),
    )
    .with_baseline_demand_from(args.input))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum IntrinsicSubgridAxisAuthority<S: LayoutScalar> {
    FinalContainerContent(S),
    FinalTrackSpan(S),
    Unknown,
}

impl<S: LayoutScalar> IntrinsicSubgridAxisAuthority<S> {
    fn extent(self) -> Option<S> {
        match self {
            Self::FinalContainerContent(extent) | Self::FinalTrackSpan(extent) => Some(extent),
            Self::Unknown => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct IntrinsicSubgridAxisConstraint<S: LayoutScalar> {
    physical_axis: crate::geometry::PhysicalAxis,
    authority: IntrinsicSubgridAxisAuthority<S>,
}

struct IntrinsicSubgridAxisConstraintInput<'a, S: LayoutScalar> {
    report: SubgridAxisReport,
    area: GridArea<S>,
    parent_flow_axes: FlowAxes,
    child_flow_axes: FlowAxes,
    explicit_parent_content_size: Size<Option<S>>,
    parent_column_count: usize,
    parent_row_count: usize,
    tracks: &'a [TrackSizingOf<S>],
    gap: S,
}

fn intrinsic_subgrid_axis_constraint<S: LayoutScalar>(
    input: IntrinsicSubgridAxisConstraintInput<'_, S>,
) -> Option<IntrinsicSubgridAxisConstraint<S>> {
    let IntrinsicSubgridAxisConstraintInput {
        report,
        area,
        parent_flow_axes,
        child_flow_axes,
        explicit_parent_content_size,
        parent_column_count,
        parent_row_count,
        tracks,
        gap,
    } = input;
    let physical_axis = inherited_subgrid_physical_axis(report, parent_flow_axes, child_flow_axes)?;
    let mapping = report.mapping;
    let (start, end, count) = match mapping.parent_axis {
        GridAxisKind::Column => (area.column, area.column_end, parent_column_count),
        GridAxisKind::Row => (area.row, area.row_end, parent_row_count),
    };
    let explicit_container_extent = if start == 0 && end == count {
        match physical_axis {
            crate::geometry::PhysicalAxis::Horizontal => explicit_parent_content_size.width,
            crate::geometry::PhysicalAxis::Vertical => explicit_parent_content_size.height,
        }
    } else {
        None
    };
    let authority = if let Some(extent) = explicit_container_extent {
        IntrinsicSubgridAxisAuthority::FinalContainerContent(extent)
    } else if let Some(extent) = exact_static_track_span(tracks, start, end, gap) {
        IntrinsicSubgridAxisAuthority::FinalTrackSpan(extent)
    } else {
        IntrinsicSubgridAxisAuthority::Unknown
    };
    Some(IntrinsicSubgridAxisConstraint {
        physical_axis,
        authority,
    })
}

fn exact_static_track_span<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    start: usize,
    end: usize,
    gap: S,
) -> Option<S> {
    let span = tracks.get(start..end)?;
    let mut extent = S::ZERO;
    for track in span {
        let (min, max) = (track.min.definite(None)?, track.max.definite(None)?);
        if min != max {
            return None;
        }
        extent = extent + min;
    }
    let gap_count = end.checked_sub(start)?.checked_sub(1)?;
    Some(extent + gap * S::from_f64(gap_count as f64))
}

pub(in crate::grid) fn needs_intrinsic_subgrid_context<'input, Node, S: LayoutScalar>(
    container_style: &GridContainerProjection<'_, S>,
    style: &GridItemProjection<'input, S>,
    item: SubgridItemReport<Node>,
    area: GridArea<S>,
) -> bool
where
    Node: Copy,
{
    let inherits_rows = item_inherits_parent_axis(item, GridAxisKind::Row);
    let inherits_columns = item_inherits_parent_axis(item, GridAxisKind::Column);
    let spans_multiple_inherited_columns = area.column_end > area.column + 1;

    (inherits_rows && inherits_columns && spans_multiple_inherited_columns)
        || (inherits_rows
            && ((if container_style
                .display
                .establishes_grid_lanes_formatting_context()
            {
                column_flow_for_grid_lanes(container_style)
            } else {
                container_style.grid_auto_flow.is_column()
            }) || track_components_have_percent_sizing(container_style.grid_template_columns)))
        || (inherits_columns
            && !style.size.height.is_auto()
            && track_components_have_percent_sizing(container_style.grid_template_rows))
}

pub(in crate::grid) fn track_components_have_percent_sizing<S: LayoutScalar>(
    components: &[TrackComponentOf<S>],
) -> bool {
    components
        .iter()
        .any(|component| track_component_has_percent_sizing(component))
}

fn track_component_has_percent_sizing<S: LayoutScalar>(component: &TrackComponentOf<S>) -> bool {
    match component {
        TrackComponentOf::Track(track) => track_has_percent_sizing(track),
        TrackComponentOf::Repeat(repeat) => repeat
            .components()
            .iter()
            .any(|component| track_component_has_percent_sizing(component)),
        _ => false,
    }
}

fn intrinsic_subgrid_child_input<S: LayoutScalar>(
    input: ComputeInputOf<S>,
    sizing: GridItemSizing<S>,
    constraints: [Option<IntrinsicSubgridAxisConstraint<S>>; 2],
) -> Result<ComputeInputOf<S>, SizingResolutionError<S>> {
    let mut known = input.known();
    let mut parent = input.parent();
    let mut available = input.sizing_request().axes();
    for constraint in constraints.into_iter().flatten() {
        let Some(raw_area_extent) = constraint.authority.extent() else {
            continue;
        };
        let border_box_extent =
            intrinsic_subgrid_border_box_extent(sizing, constraint.physical_axis, raw_area_extent);
        match constraint.physical_axis {
            crate::geometry::PhysicalAxis::Horizontal => {
                known.width = Some(border_box_extent);
                parent.width = Some(raw_area_extent);
                available.width = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(border_box_extent).map_err(
                        |_| {
                            SizingResolutionError::Status(LengthResolutionStatus::InvalidNumeric {
                                value: border_box_extent,
                            })
                        },
                    )?,
                    SizingOperation::Normal,
                );
            }
            crate::geometry::PhysicalAxis::Vertical => {
                known.height = Some(border_box_extent);
                parent.height = Some(raw_area_extent);
                available.height = AxisSizingRequestOf::new(
                    crate::layout_math::checked_available_space(border_box_extent).map_err(
                        |_| {
                            SizingResolutionError::Status(LengthResolutionStatus::InvalidNumeric {
                                value: border_box_extent,
                            })
                        },
                    )?,
                    SizingOperation::Normal,
                );
            }
        }
    }
    Ok(ComputeInputOf::for_child(
        input.run_mode(),
        input.sizing_mode(),
        input.requested_axis(),
        known,
        parent,
        input.containing_layout_context(),
        input.sizing_request().with_axes(available),
    )
    .with_baseline_demand_from(input))
}

fn intrinsic_subgrid_border_box_extent<S: LayoutScalar>(
    sizing: GridItemSizing<S>,
    axis: crate::geometry::PhysicalAxis,
    extent: S,
) -> S {
    let margin = sizing
        .unresolved_margin
        .map(|value| value.unwrap_or(S::ZERO));
    let margin_sum = match axis {
        crate::geometry::PhysicalAxis::Horizontal => margin.left + margin.right,
        crate::geometry::PhysicalAxis::Vertical => margin.top + margin.bottom,
    };
    (extent - margin_sum).max(S::ZERO)
}

fn apply_resolved_intrinsic_subgrid_area_constraints<S: LayoutScalar>(
    area_size: &mut Size<S>,
    constraints: [Option<IntrinsicSubgridAxisConstraint<S>>; 2],
) {
    for constraint in constraints.into_iter().flatten() {
        let Some(extent) = constraint.authority.extent() else {
            continue;
        };
        match constraint.physical_axis {
            crate::geometry::PhysicalAxis::Horizontal => area_size.width = extent,
            crate::geometry::PhysicalAxis::Vertical => area_size.height = extent,
        }
    }
}

pub(super) struct SubgridIntrinsicContributionInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    pub(super) grid: IntrinsicGrid<'input, 'a, Node, S>,
    pub(super) owner: Node,
    pub(super) constants: &'a Constants<S>,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) placements: &'a GridPlacementContext<'input, Node, S>,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [TrackSizingOf<S>],
    pub(super) sizes: &'a mut [S],
    pub(super) base_floors: &'a mut [S],
    pub(super) column_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) row_gutters: Option<&'a OrdinaryGridAxisGuttersOf<S>>,
    pub(super) container_gap: LogicalSizeOf<S>,
    pub(super) available: Size<AxisSizingRequestOf<S>>,
    pub(super) children: &'a [Node],
    pub(super) placed_areas: &'a [Option<GridArea<S>>],
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) column_sizes: &'a [S],
    pub(super) row_sizes: &'a [S],
    pub(super) query: IntrinsicGridQuery<'a, S>,
}

pub(super) struct SubgridIntrinsicContributionReport<Node, S: LayoutScalar = Scalar> {
    pub(super) row_contributions: Vec<RowIntrinsicContribution<Node, S>>,
    pub(super) ancestor_baseline_group: AncestorBaselineGroup<Node, S>,
    pub(super) receipts: Vec<GridContributionReceipt<Node, S>>,
}

impl<Node: Copy, S: LayoutScalar> SubgridIntrinsicContributionReport<Node, S> {
    pub(super) fn empty(
        owner: Node,
        axis: GridAxisKind,
        count: usize,
        flow: FlowAxes,
        reference: crate::GridBaselineReference,
    ) -> Self {
        Self {
            row_contributions: Vec::new(),
            receipts: Vec::new(),
            ancestor_baseline_group: AncestorBaselineGroup::reduce(
                owner,
                axis,
                grid_axis_physical_axis(flow, axis),
                reference,
                count,
                core::iter::empty(),
            ),
        }
    }
}

pub(super) type SubgridIntrinsicContributionResult<Node, S, M> =
    LayoutResultOf<Node, SubgridIntrinsicContributionReport<Node, S>, S, M>;

pub(super) fn apply_subgrid_intrinsic_contributions<'input, Tree, M>(
    tree: &mut Tree,
    input: SubgridIntrinsicContributionInput<'input, '_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> SubgridIntrinsicContributionResult<<Tree as Traverse>::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    if input.tracks.is_empty() || input.subgrid_report.items.is_empty() {
        return Ok(SubgridIntrinsicContributionReport {
            row_contributions: Vec::new(),
            receipts: Vec::new(),
            ancestor_baseline_group: AncestorBaselineGroup::reduce(
                input.owner,
                input.axis,
                grid_axis_physical_axis(input.constants.flow_axes, input.axis),
                grid_baseline_reference(input.grid.style.baseline_references, input.axis),
                input.tracks.len(),
                core::iter::empty::<AncestorBaselineMember<<Tree as Traverse>::Node, Tree::Scalar>>(
                ),
            ),
        });
    }

    let intrinsic_min_track_facts = input
        .tracks
        .iter()
        .map(|track| {
            track_minimum_is_intrinsic(
                track,
                match input.axis {
                    GridAxisKind::Column => input.grid.track_basis.inline,
                    GridAxisKind::Row => input.grid.track_basis.block,
                },
            )
        })
        .collect::<Vec<_>>();
    let Ok(report) = collect_grid_subgrid_intrinsic_traversal(
        tree,
        GridSubgridIntrinsicTraversalInput {
            grid: input.grid,
            axis: input.axis,
            containing_flow_axes: input.constants.flow_axes,
            children: input.children,
            placed_areas: input.placed_areas,
            subgrid_report: input.subgrid_report,
            placements: input.placements,
            named_columns: input.named_columns,
            named_rows: input.named_rows,
            area_facts: input.area_facts,
            parent_gap: Size::new(input.container_gap.inline, input.container_gap.block),
            column_gutters: input.column_gutters,
            row_gutters: input.row_gutters,
            column_sizes: input.column_sizes,
            row_sizes: input.row_sizes,
            container_size: input.constants.node_inner_size,
            intrinsic_min_track_facts: IntrinsicMinTrackFacts::Known(&intrinsic_min_track_facts),
            query: input.query,
        },
    )?
    else {
        return Ok(SubgridIntrinsicContributionReport {
            row_contributions: Vec::new(),
            receipts: Vec::new(),
            ancestor_baseline_group: AncestorBaselineGroup::reduce(
                input.owner,
                input.axis,
                grid_axis_physical_axis(input.constants.flow_axes, input.axis),
                grid_baseline_reference(input.grid.style.baseline_references, input.axis),
                input.tracks.len(),
                core::iter::empty::<AncestorBaselineMember<<Tree as Traverse>::Node, Tree::Scalar>>(
                ),
            ),
        });
    };

    for (index, lower_bound) in report.edge_lower_bounds.into_iter().enumerate() {
        if let Some(size) = input.base_floors.get_mut(index) {
            *size = size.max(lower_bound);
        }
    }

    let mut leaves = report.leaves;
    leaves.sort_by_key(|leaf| {
        leaf.ancestor_span
            .end
            .saturating_sub(leaf.ancestor_span.start)
    });
    let mut flattened_contributions = Vec::new();
    let mut ancestor_baseline_members = Vec::new();
    let mut adjustment_path = Vec::new();
    for leaf in leaves {
        let child_style = leaf.style.clone();
        if !is_in_flow_grid_child(&child_style) {
            continue;
        }
        if subgrid_leaf_size_depends_on_queried_axis(
            &child_style,
            input.constants.flow_axes,
            input.axis,
        ) {
            continue;
        }
        let start = leaf.ancestor_span.start - 1;
        let end = leaf.ancestor_span.end - 1;
        let Some(span_tracks) = input.tracks.get(start..end) else {
            continue;
        };
        if !span_tracks.iter().any(|track| {
            input.query.track_needs_contribution(
                track,
                match input.axis {
                    GridAxisKind::Column => input.grid.track_basis.inline,
                    GridAxisKind::Row => input.grid.track_basis.block,
                },
            )
        }) {
            continue;
        }

        let frame = leaf.min_frame.ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(leaf.node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::SubgridTrackInheritance,
                ),
            )
        })?;
        let child_input = frame.prepared().with_run_mode(
            if input.axis == GridAxisKind::Row
                && matches!(
                    leaf.align_self,
                    AlignItems::Baseline | AlignItems::LastBaseline
                )
            {
                RunMode::PerformLayout
            } else {
                RunMode::ComputeSize
            },
        );
        let child_input =
            child_input.with_named_baseline_demand(crate::output::NamedBaselineDemand::None);
        let leaf_flow = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let child_input = if matches!(
            leaf_flow.line_over_edge(child_style.margin),
            LengthAutoOf::Auto
        ) || matches!(
            leaf_flow.line_under_edge(child_style.margin),
            LengthAutoOf::Auto
        ) {
            child_input
        } else {
            grid_baseline_demand(
                input.grid.style.baseline_references,
                input.constants.flow_axes,
                leaf_flow,
                if input.axis == GridAxisKind::Column {
                    leaf.align_self
                } else {
                    AlignItems::Stretch
                },
                if input.axis == GridAxisKind::Row {
                    leaf.align_self
                } else {
                    AlignItems::Stretch
                },
                child_input,
            )
        };
        let subgrid_requests =
            leaf.standalone_parent_context
                .as_ref()
                .map_or(Size::new(false, false), |source| {
                    crate::grid::lanes::lane_child_subgrid_requests(Some(&source.container_input()))
                });
        let mut query = |input| {
            if let Some(parent_context) = &leaf.standalone_parent_context
                && (tree.child_count(leaf.node) > 0
                    || parent_context.container_input().requires_context())
            {
                parent_context.measure(tree, input)
            } else {
                tree.compute_child(leaf.node, input)
            }
        };
        let output = if input.query.purpose == GridContributionPurpose::Minimum {
            compute_grid_minimum_contribution(
                leaf.node,
                GridMinimumContext {
                    item: &child_style,
                    flow_axes: frame.flow_axes,
                    axis: frame.axis,
                    prepared: child_input,
                    margin_basis: child_input.parent(),
                    span: frame.span,
                    opposite_fixed_max_span: frame.opposite_fixed_max_span,
                    subgrid_requests,
                    algorithm: frame.algorithm,
                },
                query,
            )?
        } else {
            query(child_input)?
        };
        let margin = intrinsic_contribution_margin(
            &child_style,
            child_input.containing_flow_axes(),
            child_input.parent(),
        )
        .map_err(|status| crate::error::value_resolution_error(leaf.node, status))?;
        let Some((scalar_adjustment, baseline_adjustments)) =
            report.adjustments.for_leaf(&leaf, &mut adjustment_path)
        else {
            continue;
        };
        let contribution = grid_axis_size(input.constants.flow_axes, output.size, input.axis)
            + grid_axis_size(input.constants.flow_axes, margin.sum_axes(), input.axis)
            + scalar_adjustment;
        let flattened = FlattenedScalarContribution::new(
            leaf.node,
            input.axis,
            leaf.ancestor_span,
            contribution,
        );
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
            &child_style,
            child_input.containing_flow_axes(),
            child_input.parent(),
            child_flow_axes,
        )
        .map_err(|status| crate::error::value_resolution_error(leaf.node, status))?;
        let member = ancestor_baseline_member(AncestorBaselineMemberInput {
            reference: grid_baseline_reference(input.grid.style.baseline_references, input.axis),
            source: leaf.node,
            axis: input.axis,
            ancestor_span: leaf.ancestor_span,
            alignment_context_reversed: leaf.alignment_context_reversed,
            alignment: leaf.align_self,
            block_auto_margins,
            synthesized_baseline_cycle: grid_baseline_span_would_cycle(
                span_tracks,
                match input.axis {
                    GridAxisKind::Column => input.grid.track_basis.inline,
                    GridAxisKind::Row => input.grid.track_basis.block,
                },
            ),
            output,
            margin,
            child_flow_axes,
            containing_flow_axes: input.constants.flow_axes,
            start_adjustment: baseline_adjustments.start,
            end_adjustment: baseline_adjustments.end,
        })
        .map_err(|error| grid_baseline_error(input.owner, leaf.node, error))?;
        if let Some(member) = member {
            ancestor_baseline_members.push(member);
        }
        flattened_contributions.push((flattened, member));
    }

    if input.axis == GridAxisKind::Column {
        for (index, (child, area)) in input
            .children
            .iter()
            .copied()
            .zip(input.placed_areas.iter().copied())
            .enumerate()
        {
            let Some(area) = area else {
                continue;
            };
            let child_style = input.placements.item_input(index);
            if !is_in_flow_grid_child(child_style)
                || input
                    .subgrid_report
                    .items
                    .get(index)
                    .is_some_and(|item| item_inherits_parent_axis(*item, GridAxisKind::Column))
            {
                continue;
            }
            let alignment = child_style
                .justify_self
                .or(input.container_style.justify_items)
                .unwrap_or(AlignItems::Stretch);
            if !matches!(alignment, AlignItems::Baseline | AlignItems::LastBaseline) {
                continue;
            }
            let (start, end) = (area.column, area.column_end);
            let Some(span_tracks) = input.tracks.get(start..end) else {
                continue;
            };
            let output = compute_intrinsic_grid_child(
                tree,
                child,
                IntrinsicGridChildInput {
                    child_style,
                    child_container: input.placements.nested_container_input(index),
                    grid: input.grid,
                    area,
                    columns: input.column_sizes,
                    rows: input.row_sizes,
                    subgrid_item: input.subgrid_report.items.get(index).copied(),
                    input: ComputeInputOf::for_child(
                        RunMode::PerformLayout,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        input.constants.node_inner_size,
                        crate::ContainingLayoutContext::new(
                            input.constants.flow_axes,
                            crate::ParentFormattingContext::Grid,
                        ),
                        SizingRequestOf::new(
                            input.available,
                            input.constants.child_orthogonal_fallback,
                        ),
                    ),
                },
            )?;
            let margin_basis = input
                .grid
                .column_contribution_margin_basis(span_tracks, input.constants.node_inner_size)
                .unwrap_or(input.constants.node_inner_size);
            let margin =
                intrinsic_contribution_margin(child_style, input.constants.flow_axes, margin_basis)
                    .map_err(|status| crate::error::value_resolution_error(child, status))?;
            let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
            let block_auto_margins = block_auto_margins_for_intrinsic_contribution(
                child_style,
                input.constants.flow_axes,
                margin_basis,
                child_flow_axes,
            )
            .map_err(|status| crate::error::value_resolution_error(child, status))?;
            if let Some(member) = ancestor_baseline_member(AncestorBaselineMemberInput {
                reference: grid_baseline_reference(
                    input.grid.style.baseline_references,
                    GridAxisKind::Column,
                ),
                source: child,
                axis: GridAxisKind::Column,
                ancestor_span: GridTrackSpan::new(start + 1, end + 1),
                alignment_context_reversed: false,
                alignment,
                block_auto_margins,
                synthesized_baseline_cycle: grid_baseline_span_would_cycle(
                    span_tracks,
                    input.grid.track_basis.inline,
                ),
                output,
                margin,
                child_flow_axes,
                containing_flow_axes: input.constants.flow_axes,
                start_adjustment: Tree::Scalar::ZERO,
                end_adjustment: Tree::Scalar::ZERO,
            })
            .map_err(|error| grid_baseline_error(input.owner, child, error))?
            {
                ancestor_baseline_members.push(member);
            }
        }
    }

    let ancestor_baseline_group = AncestorBaselineGroup::reduce(
        input.owner,
        input.axis,
        grid_axis_physical_axis(input.constants.flow_axes, input.axis),
        grid_baseline_reference(input.grid.style.baseline_references, input.axis),
        input.tracks.len(),
        ancestor_baseline_members,
    );
    let mut row_contributions = Vec::new();
    let mut receipts = Vec::new();
    for (flattened, member) in flattened_contributions {
        let start = flattened.ancestor_span.start - 1;
        let end = flattened.ancestor_span.end - 1;
        if input.tracks.get(start..end).is_none() {
            continue;
        }
        let shim = member.map_or(BaselineShim::default(), |member| {
            ancestor_baseline_group.intrinsic_shim(member)
        });
        let contribution = flattened.contribution + shim.before + shim.after;
        if input.axis == GridAxisKind::Column {
            receipts.push(GridContributionReceipt {
                purpose: input.query.purpose,
                source: flattened.source,
                axis: input.axis,
                span: flattened.ancestor_span,
                value: contribution,
            });
        }
        if input.axis == GridAxisKind::Row {
            row_contributions.push(RowIntrinsicContribution {
                source: flattened.source,
                start,
                end,
                contributes_to_row_size: true,
                contribution: flattened.contribution,
                baseline_member: member,
            });
        } else if end == start + 1 {
            input.sizes[start] = input.sizes[start].max(contribution);
        }
    }
    let receipt_count = receipts.len();
    reduce_subgrid_candidate_receipts(&mut receipts, 0..receipt_count)
        .map_err(|error| grid_track_sizing_error(input.owner, error))?;
    Ok(SubgridIntrinsicContributionReport {
        row_contributions,
        ancestor_baseline_group,
        receipts,
    })
}

/// An original descendant can reach one ancestor span through multiple Lane
/// candidates. Reduce measured receipts after edge and baseline adjustments.
pub(super) fn reduce_subgrid_candidate_receipts<Node: Copy + Eq, S: LayoutScalar>(
    receipts: &mut Vec<GridContributionReceipt<Node, S>>,
    candidates: std::ops::Range<usize>,
) -> Result<(), SpanningContributionError<S>> {
    if candidates.len() < 2 {
        return Ok(());
    }
    let mut by_span = std::collections::HashMap::<_, Vec<usize>>::new();
    by_span
        .try_reserve(candidates.len())
        .map_err(|_| SpanningContributionError::Allocation)?;
    let mut unique_end = candidates.start;
    let end = candidates.end;
    for index in candidates {
        let candidate = receipts[index];
        let span = (candidate.span.start, candidate.span.end);
        let span_indexes = by_span.entry(span).or_default();
        if let Some(previous) = span_indexes.iter().copied().find(|&index| {
            let previous = receipts[index];
            previous.source == candidate.source
                && previous.axis == candidate.axis
                && previous.purpose == candidate.purpose
        }) {
            let current = &mut receipts[previous].value;
            if current.is_finite() && (!candidate.value.is_finite() || candidate.value > *current) {
                *current = candidate.value;
            }
        } else {
            span_indexes
                .try_reserve(1)
                .map_err(|_| SpanningContributionError::Allocation)?;
            span_indexes.push(unique_end);
            receipts[unique_end] = candidate;
            unique_end += 1;
        }
    }
    receipts.drain(unique_end..end);
    Ok(())
}

fn subgrid_leaf_size_depends_on_queried_axis<'input, S: LayoutScalar>(
    style: &GridItemProjection<'input, S>,
    flow_axes: FlowAxes,
    axis: GridAxisKind,
) -> bool {
    grid_axis_size(
        flow_axes,
        Size::new(&style.size.width, &style.size.height),
        axis,
    )
    .depends_on_basis()
}

pub(in crate::grid) fn item_inherits_parent_axis<Node>(
    item: SubgridItemReport<Node>,
    parent_axis: GridAxisKind,
) -> bool
where
    Node: Copy,
{
    [GridAxisKind::Column, GridAxisKind::Row]
        .into_iter()
        .any(|child_axis| {
            let report = match child_axis {
                GridAxisKind::Column => item.column,
                GridAxisKind::Row => item.row,
            };
            report.can_inherit() && report.mapping.parent_axis == parent_axis
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::layout_tree::OracleTree;
    use crate::{
        Length, LengthPercentageOf, NodeInput, SizingCalculationOf, TrackComponent, TrackSizing,
    };

    fn fri04_c03_grid_track_nested<S: LayoutScalar>(target: f64) -> SizingCalculationOf<S> {
        let value = |value| {
            SizingCalculationOf::value(
                LengthPercentageOf::px(S::from_f64(value)).expect("finite test track value"),
            )
        };
        SizingCalculationOf::clamp(
            Some(value(target - 10.0)),
            SizingCalculationOf::max(vec![
                value(target),
                SizingCalculationOf::min(vec![value(target - 5.0), value(target + 5.0)])
                    .expect("nested minimum is nonempty"),
            ])
            .expect("nested maximum is nonempty"),
            Some(value(target + 10.0)),
        )
    }

    #[test]
    fn fri04_c03_grid_track_exact_static_span_accepts_basis_independent_nested_programs() {
        let tracks = [
            TrackSizingOf::calculation(fri04_c03_grid_track_nested::<f64>(30.0)),
            TrackSizingOf::calculation(fri04_c03_grid_track_nested::<f64>(40.0)),
        ];
        assert_eq!(exact_static_track_span(&tracks, 0, 2, 7.0), Some(77.0));

        let dependent = SizingCalculationOf::max(vec![
            fri04_c03_grid_track_nested::<f64>(30.0),
            SizingCalculationOf::value(
                LengthPercentageOf::from_percent_fraction(0.1).expect("finite test percentage"),
            ),
        ])
        .expect("nested maximum is nonempty");
        assert_eq!(
            exact_static_track_span(&[TrackSizingOf::calculation(dependent)], 0, 1, 0.0),
            None
        );
    }

    #[test]
    fn intrinsic_subgrid_constraints_distinguish_final_authority_from_unknown_spans() {
        fn axis_report(parent_axis: GridAxisKind, eligible: bool) -> SubgridAxisReport {
            SubgridAxisReport {
                inherit_parent_names: true,
                mapping: GridAxisMappingReport {
                    queried_axis: parent_axis,
                    parent_axis,
                    child_axis: parent_axis,
                    reversed: false,
                },
                eligibility: if eligible {
                    SubgridEligibility::SharedTracks
                } else {
                    SubgridEligibility::Excluded(SubgridIneligibleReason::NotRequested)
                },
            }
        }

        fn assert_lanes<S: LayoutScalar>() {
            let column_constraint =
                intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
                    report: axis_report(GridAxisKind::Column, true),
                    area: GridArea {
                        column: 0,
                        column_end: 2,
                        row: 0,
                        row_end: 1,
                        size: LogicalSizeOf::new(S::ZERO, S::ZERO),
                    },
                    parent_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    child_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    explicit_parent_content_size: Size::new(
                        Some(S::from_f64(41.0)),
                        Some(S::from_f64(97.0)),
                    ),
                    parent_column_count: 2,
                    parent_row_count: 1,
                    tracks: &[
                        TrackSizingOf::px(S::from_f64(30.0)),
                        TrackSizingOf::px(S::from_f64(40.0)),
                    ],
                    gap: S::from_f64(7.0),
                })
                .expect("eligible full-span column subgrid has a constraint");
            assert_eq!(
                column_constraint,
                IntrinsicSubgridAxisConstraint {
                    physical_axis: crate::geometry::PhysicalAxis::Vertical,
                    authority: IntrinsicSubgridAxisAuthority::FinalContainerContent(S::from_f64(
                        97.0
                    )),
                }
            );

            let row_constraint =
                intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
                    report: axis_report(GridAxisKind::Row, true),
                    area: GridArea {
                        column: 0,
                        column_end: 1,
                        row: 1,
                        row_end: 2,
                        size: LogicalSizeOf::new(S::ZERO, S::ZERO),
                    },
                    parent_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    child_flow_axes: crate::geometry::FlowAxes::new(
                        crate::WritingMode::VerticalRl,
                        crate::Direction::Ltr,
                    ),
                    explicit_parent_content_size: Size::new(
                        Some(S::from_f64(41.0)),
                        Some(S::from_f64(97.0)),
                    ),
                    parent_column_count: 1,
                    parent_row_count: 2,
                    tracks: &[TrackSizingOf::AUTO, TrackSizingOf::px(S::from_f64(40.0))],
                    gap: S::from_f64(7.0),
                })
                .expect("eligible partial-span row subgrid has a constraint");
            assert_eq!(
                row_constraint,
                IntrinsicSubgridAxisConstraint {
                    physical_axis: crate::geometry::PhysicalAxis::Horizontal,
                    authority: IntrinsicSubgridAxisAuthority::FinalTrackSpan(S::from_f64(40.0)),
                }
            );

            let unknown = intrinsic_subgrid_axis_constraint(IntrinsicSubgridAxisConstraintInput {
                report: axis_report(GridAxisKind::Row, true),
                area: GridArea {
                    column: 0,
                    column_end: 1,
                    row: 1,
                    row_end: 2,
                    size: LogicalSizeOf::new(S::ZERO, S::ZERO),
                },
                parent_flow_axes: crate::geometry::FlowAxes::new(
                    crate::WritingMode::VerticalRl,
                    crate::Direction::Ltr,
                ),
                child_flow_axes: crate::geometry::FlowAxes::new(
                    crate::WritingMode::VerticalRl,
                    crate::Direction::Ltr,
                ),
                explicit_parent_content_size: Size::new(
                    Some(S::from_f64(41.0)),
                    Some(S::from_f64(97.0)),
                ),
                parent_column_count: 1,
                parent_row_count: 2,
                tracks: &[TrackSizingOf::AUTO, TrackSizingOf::AUTO],
                gap: S::from_f64(7.0),
            })
            .expect("eligible partial-span row subgrid has a constraint");
            assert_eq!(unknown.authority, IntrinsicSubgridAxisAuthority::Unknown);
        }

        assert_lanes::<f32>();
        assert_lanes::<f64>();
    }

    #[test]
    fn nested_orthogonal_partial_subgrids_keep_intrinsic_axes_provisional() {
        fn axis_report(parent_axis: GridAxisKind, child_axis: GridAxisKind) -> SubgridAxisReport {
            SubgridAxisReport {
                inherit_parent_names: true,
                mapping: GridAxisMappingReport {
                    queried_axis: child_axis,
                    parent_axis,
                    child_axis,
                    reversed: false,
                },
                eligibility: SubgridEligibility::SharedTracks,
            }
        }

        fn assert_lanes<S: LayoutScalar>() {
            let horizontal_style = NodeInput::default();
            let horizontal =
                FlowAxes::new(horizontal_style.writing_mode, horizontal_style.direction);
            let vertical = FlowAxes::new(crate::WritingMode::VerticalLr, crate::Direction::Ltr);
            let outer_constraint = IntrinsicSubgridAxisConstraint {
                physical_axis: inherited_subgrid_physical_axis(
                    axis_report(GridAxisKind::Column, GridAxisKind::Row),
                    horizontal,
                    vertical,
                )
                .expect("mapped outer partial subgrid has a physical axis"),
                authority: IntrinsicSubgridAxisAuthority::Unknown,
            };
            let outer = intrinsic_subgrid_child_input(
                ComputeInputOf::for_child(
                    RunMode::ComputeSize,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::new(None, Some(S::from_f64(19.0))),
                    Size::new(None, Some(S::from_f64(29.0))),
                    crate::ContainingLayoutContext::new(
                        horizontal,
                        crate::ParentFormattingContext::Grid,
                    ),
                    SizingRequestOf::new(
                        Size::new(
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Indefinite,
                                SizingOperation::MaxContent,
                            ),
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Indefinite,
                                SizingOperation::MinContent,
                            ),
                        ),
                        OrthogonalFallbackOf::new(
                            Size::NONE,
                            crate::NearestScrollportOf::Absent,
                            Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    ),
                ),
                GridItemSizing {
                    known: Size::new(Some(S::from_f64(31.0)), Some(S::from_f64(37.0))),
                    available: Size::new(S::from_f64(41.0), S::from_f64(43.0)),
                    unresolved_margin: Edges::ZERO.map(Some),
                    justify_self: AlignItems::Stretch,
                    align_self: AlignItems::Stretch,
                    standalone_intrinsic_minimum: Size::NONE,
                },
                [Some(outer_constraint), None],
            )
            .unwrap();
            assert_eq!(outer.known().width, None);
            assert_eq!(outer.parent().width, None);
            assert_eq!(
                outer.sizing_request().axes().width,
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent)
            );

            let inner_constraint = IntrinsicSubgridAxisConstraint {
                physical_axis: inherited_subgrid_physical_axis(
                    axis_report(GridAxisKind::Row, GridAxisKind::Column),
                    vertical,
                    horizontal,
                )
                .expect("mapped inner partial subgrid has a physical axis"),
                authority: IntrinsicSubgridAxisAuthority::Unknown,
            };
            let inner = intrinsic_subgrid_child_input(
                outer,
                GridItemSizing {
                    known: Size::new(Some(S::from_f64(47.0)), Some(S::from_f64(53.0))),
                    available: Size::new(S::from_f64(59.0), S::from_f64(61.0)),
                    unresolved_margin: Edges::ZERO.map(Some),
                    justify_self: AlignItems::Stretch,
                    align_self: AlignItems::Stretch,
                    standalone_intrinsic_minimum: Size::NONE,
                },
                [Some(inner_constraint), None],
            )
            .unwrap();
            assert_eq!(inner.known().width, None);
            assert_eq!(inner.parent().width, None);
            assert_eq!(
                inner.sizing_request().axes().width,
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent)
            );
        }

        assert_lanes::<f32>();
        assert_lanes::<f64>();
    }

    #[test]
    fn vertical_intrinsic_grid_percentage_edges_use_logical_area_basis() {
        let parent_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            justify_items: Some(AlignItems::Start),
            align_items: Some(AlignItems::Start),
            ..NodeInput::default()
        };
        let child_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack {
                name_components: Vec::new(),
            })],
            padding: Edges::all(Length::percent(0.1)),
            ..NodeInput::default()
        };
        let mut tree = OracleTree::new()
            .children(2, [])
            .style(2, child_style.clone());
        let constants = Constants {
            child_orthogonal_fallback: OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
            flow_axes: crate::geometry::FlowAxes::new(
                parent_style.writing_mode,
                parent_style.direction,
            ),
            explicit_definite_content_size: Size::new(Some(100.0), Some(200.0)),
            indefinite_percentage_axis: None,
            node_outer_size: Size::new(Some(100.0), Some(200.0)),
            node_inner_size: Size::new(Some(100.0), Some(200.0)),
            node_min_size: Size::NONE,
            node_definite_min_size: Size::NONE,
            fallback_minimum_outer: Size::splat(FixedFallbackSource::Unqualified),
            fallback_maximum_outer: Size::splat(FixedFallbackSource::Unqualified),
            node_max_size: Size::NONE,
            available_inner_size: Size::new(Some(100.0), Some(200.0)),
            content_box_inset: Edges::ZERO,
            scrollbar_gutter: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
        };
        let area = GridArea {
            column: 0,
            column_end: 1,
            row: 0,
            row_end: 1,
            size: LogicalSizeOf::new(200.0, 100.0),
        };
        let columns = [200.0];
        let rows = [100.0];
        let named_columns = NamedGridLines::new(GridAxisKind::Column, 1);
        let named_rows = NamedGridLines::new(GridAxisKind::Row, 1);
        let placements = GridPlacementContext::empty();
        let subgrid_report = GridSubgridReport { items: Vec::new() };
        let parent_projection = GridContainerProjection::from_node(&parent_style);
        let child_input = GridChildInput::from_node(&child_style);
        let subgrid_item = SubgridItemReport {
            node: 2,
            column: subgrid_axis_report(&parent_projection, &child_input, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_projection, &child_input, GridAxisKind::Row),
        };
        let output = compute_intrinsic_grid_child(
            &mut tree.borrow_compute(),
            2,
            IntrinsicGridChildInput {
                child_style: child_input.item(),
                child_container: child_input.nested_container(),
                grid: IntrinsicGrid {
                    style: &parent_projection,
                    constants: &constants,
                    sizing_flow_axes: constants.flow_axes,
                    track_basis: constants.flow_axes.logical_size(constants.node_inner_size),
                    minimum_functions: GridMinimumParentFunctions {
                        columns: None,
                        rows: None,
                    },
                    column_tracks: &[TrackSizing::px(200.0)],
                    row_tracks: &[TrackSizing::px(100.0)],
                    gap: LogicalSizeOf::new(0.0, 0.0),
                    column_gutters: None,
                    row_gutters: None,
                    lines: GridLines {
                        column_explicit_start: 0,
                        column_explicit_count: 1,
                        row_explicit_start: 0,
                        row_explicit_count: 1,
                    },
                    named_columns: &named_columns,
                    named_rows: &named_rows,
                    area_facts: None,
                    subgrid_report: &subgrid_report,
                    placements: &placements,
                    query: IntrinsicGridQuery::complete(GridContributionPurpose::MaxContent),
                },
                area,
                columns: &columns,
                rows: &rows,
                subgrid_item: Some(subgrid_item),
                input: ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::NONE,
                    crate::ContainingLayoutContext::new(
                        constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    SizingRequestOf::new(
                        Size::new(
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    crate::NonNegativeFiniteOf::new(100.0).unwrap(),
                                ),
                                SizingOperation::Normal,
                            ),
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    crate::NonNegativeFiniteOf::new(200.0).unwrap(),
                                ),
                                SizingOperation::Normal,
                            ),
                        ),
                        OrthogonalFallbackOf::new(
                            Size::NONE,
                            crate::NearestScrollportOf::Absent,
                            Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    ),
                ),
            },
        )
        .unwrap();

        assert_eq!(output.size, Size::new(40.0, 200.0));
    }

    #[test]
    fn vertical_intrinsic_subgrid_percentage_gap_uses_logical_content_box() {
        let parent_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            justify_items: Some(AlignItems::Start),
            align_items: Some(AlignItems::Start),
            ..NodeInput::default()
        };
        let child_style = NodeInput {
            display: Display::Grid,
            writing_mode: crate::WritingMode::VerticalRl,
            grid_template_columns: vec![TrackComponent::Subgrid(crate::SubgridTrack {
                name_components: Vec::new(),
            })],
            grid_template_rows: vec![TrackComponent::px(100.0)],
            gap: Size::new(Length::ZERO, Length::percent(0.1)),
            padding: Edges::all(Length::percent(0.1)),
            justify_items: Some(AlignItems::Start),
            align_items: Some(AlignItems::Start),
            ..NodeInput::default()
        };
        let mut tree = OracleTree::new()
            .children(2, [3, 4])
            .children(3, [])
            .children(4, [])
            .style(2, child_style.clone())
            .style(3, NodeInput::default())
            .style(4, NodeInput::default());
        let constants = Constants {
            child_orthogonal_fallback: OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
            flow_axes: crate::geometry::FlowAxes::new(
                parent_style.writing_mode,
                parent_style.direction,
            ),
            explicit_definite_content_size: Size::new(Some(100.0), Some(200.0)),
            indefinite_percentage_axis: None,
            node_outer_size: Size::new(Some(100.0), Some(200.0)),
            node_inner_size: Size::new(Some(100.0), Some(200.0)),
            node_min_size: Size::NONE,
            node_definite_min_size: Size::NONE,
            fallback_minimum_outer: Size::splat(FixedFallbackSource::Unqualified),
            fallback_maximum_outer: Size::splat(FixedFallbackSource::Unqualified),
            node_max_size: Size::NONE,
            available_inner_size: Size::new(Some(100.0), Some(200.0)),
            content_box_inset: Edges::ZERO,
            scrollbar_gutter: Edges::ZERO,
            padding: Edges::ZERO,
            border: Edges::ZERO,
        };
        let area = GridArea {
            column: 0,
            column_end: 2,
            row: 0,
            row_end: 1,
            size: LogicalSizeOf::new(200.0, 100.0),
        };
        let columns = [100.0, 100.0];
        let rows = [100.0];
        let named_columns = NamedGridLines::new(GridAxisKind::Column, 2);
        let named_rows = NamedGridLines::new(GridAxisKind::Row, 1);
        let placements = GridPlacementContext::empty();
        let subgrid_report = GridSubgridReport { items: Vec::new() };
        let parent_projection = GridContainerProjection::from_node(&parent_style);
        let child_input = GridChildInput::from_node(&child_style);
        let subgrid_item = SubgridItemReport {
            node: 2,
            column: subgrid_axis_report(&parent_projection, &child_input, GridAxisKind::Column),
            row: subgrid_axis_report(&parent_projection, &child_input, GridAxisKind::Row),
        };
        compute_intrinsic_grid_child(
            &mut tree.borrow_compute(),
            2,
            IntrinsicGridChildInput {
                child_style: child_input.item(),
                child_container: child_input.nested_container(),
                grid: IntrinsicGrid {
                    style: &parent_projection,
                    constants: &constants,
                    sizing_flow_axes: constants.flow_axes,
                    track_basis: constants.flow_axes.logical_size(constants.node_inner_size),
                    minimum_functions: GridMinimumParentFunctions {
                        columns: None,
                        rows: None,
                    },
                    column_tracks: &[TrackSizing::px(100.0), TrackSizing::px(100.0)],
                    row_tracks: &[TrackSizing::px(100.0)],
                    gap: LogicalSizeOf::new(0.0, 0.0),
                    column_gutters: None,
                    row_gutters: None,
                    lines: GridLines {
                        column_explicit_start: 0,
                        column_explicit_count: 2,
                        row_explicit_start: 0,
                        row_explicit_count: 1,
                    },
                    named_columns: &named_columns,
                    named_rows: &named_rows,
                    area_facts: None,
                    subgrid_report: &subgrid_report,
                    placements: &placements,
                    query: IntrinsicGridQuery::complete(GridContributionPurpose::MaxContent),
                },
                area,
                columns: &columns,
                rows: &rows,
                subgrid_item: Some(subgrid_item),
                input: ComputeInputOf::for_child(
                    RunMode::PerformLayout,
                    SizingMode::InherentSize,
                    RequestedAxis::Both,
                    Size::NONE,
                    Size::NONE,
                    crate::ContainingLayoutContext::new(
                        constants.flow_axes,
                        crate::ParentFormattingContext::Grid,
                    ),
                    SizingRequestOf::new(
                        Size::new(
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    crate::NonNegativeFiniteOf::new(100.0).unwrap(),
                                ),
                                SizingOperation::Normal,
                            ),
                            AxisSizingRequestOf::new(
                                AvailableSpaceOf::Definite(
                                    crate::NonNegativeFiniteOf::new(200.0).unwrap(),
                                ),
                                SizingOperation::Normal,
                            ),
                        ),
                        OrthogonalFallbackOf::new(
                            Size::NONE,
                            crate::NearestScrollportOf::Absent,
                            Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                        ),
                    ),
                ),
            },
        )
        .unwrap();

        assert_eq!(
            tree.layout(4)
                .expect("second subgrid item should be laid out")
                .location
                .y,
            108.0
        );
    }
}
