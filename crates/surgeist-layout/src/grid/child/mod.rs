use super::*;
use crate::error::{layout_child_geometry_error, sizing_resolution_error};
use crate::geometry::{
    FlowAxes, LogicalAxis, LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis,
    PhysicalProgression,
};
use crate::layout_math::MaxBeforeMinOptionalSizeClampExt;
use crate::output::PhysicalBaseline;
use crate::scroll::{ScrollContributionAccumulatorOf, UsedOverflow};
use crate::sizing::resolve::{
    SizingResolutionError, resolve_maximum_optional, resolve_minimum_optional,
    resolve_preferred_optional,
};
use crate::{
    BaselinesOf, LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant,
    LayoutOperation,
};

pub(super) mod absolute;
mod baseline;
mod minimum;
mod scroll;
mod settlement;
mod subgrid_context;

pub(super) use absolute::*;
pub(super) use baseline::*;
pub(super) use minimum::*;
pub(super) use scroll::*;
pub(super) use subgrid_context::*;

use settlement::{OrdinaryGridItemPlacement, stage_ordinary_grid_items};

pub(super) struct GridChildrenLayout<Node, S: LayoutScalar = Scalar> {
    pub(super) executed_baselines: Option<ExecutedBaselineInventory<Node, S>>,
    pub(super) visible_content_size: Size<S>,
    pub(super) contributions: ScrollContributionAccumulatorOf<S>,
    pub(super) baselines: BaselinesOf<S>,
    pub(super) baseline_groups: GridBaselineGroups<S>,
}

pub(super) type GridChildrenLayoutResult<Node, S, M> =
    LayoutResultOf<Node, GridChildrenLayout<Node, S>, S, M>;

pub(super) type GridChildPlacementResult<'input, Node, S, M> =
    LayoutResultOf<Node, GridChildPlacement<'input, Node, S>, S, M>;

pub(super) type GridChildPlacementStart<'input, Node, S, M> = LayoutResultOf<
    Node,
    ControlFlow<GridChildrenLayout<Node, S>, GridChildPlacement<'input, Node, S>>,
    S,
    M,
>;

pub(super) struct GridChildPlacement<'input, Node, S: LayoutScalar> {
    node: Node,
    positioned: Option<PositionedGridItems<'input, Node, S>>,
    contributions: Vec<GridChildContribution<S>>,
    alignment: TrackAlignmentSubject<S>,
    padding: Edges<S>,
    reporting: GridLayoutReporting<Node, S>,
    contained_track_size: Option<Size<S>>,
}

pub(super) enum TrackAlignmentSubject<S: LayoutScalar> {
    Ordinary(GridTrackScrollBounds<S>),
    Lanes(Result<crate::ScrollRectOf<S>, crate::ScrollRectErrorOf<S>>),
}

impl<'input, Node: Copy, S: LayoutScalar> GridChildPlacement<'input, Node, S> {
    pub(super) fn begin<M>(
        placement: PositionedGridPlacement<'input, '_, Node, S>,
        pending: Vec<PendingAbsoluteGridItem<Node>>,
        contributions: Vec<GridChildContribution<S>>,
        alignment: TrackAlignmentSubject<S>,
        reporting: GridLayoutReporting<Node, S>,
    ) -> GridChildPlacementResult<'input, Node, S, M> {
        let node = placement.node;
        let padding = placement.constants.padding;
        let positioned = PositionedGridItems::begin(placement, pending, &contributions)?;
        Ok(Self {
            node,
            positioned,
            contributions,
            alignment,
            padding,
            reporting,
            contained_track_size: None,
        })
    }

    pub(super) fn with_contained_track_size(mut self, size: Size<S>) -> Self {
        self.contained_track_size = Some(size);
        self
    }

    pub(super) fn next<'tree, Tree, M>(&mut self, tree: &Tree) -> PositionedStep<'input, Node, S, M>
    where
        Tree: Compute<'tree, M, Node = Node, Scalar = S>,
    {
        match &mut self.positioned {
            Some(positioned) => positioned.next(tree),
            None => Ok(None),
        }
    }

    pub(super) fn accept<'tree, Tree, M>(
        &mut self,
        tree: &mut Tree,
        child: PositionedGridChild<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'tree, M, Node = Node, Scalar = S>,
    {
        self.contributions.push(child.finish(tree, output)?);
        Ok(())
    }

    pub(super) fn finish<M>(self) -> GridChildrenLayoutResult<Node, S, M> {
        let Self {
            node,
            contributions,
            alignment,
            padding,
            reporting,
            contained_track_size,
            ..
        } = self;
        let range_error = |error| layout_child_geometry_error(node, node, error);
        let flow_axes = reporting.flow_axes;
        let bounds = match alignment {
            TrackAlignmentSubject::Ordinary(bounds) => Some(bounds),
            TrackAlignmentSubject::Lanes(_) => None,
        };
        let contributions = grid_scroll_contributions(contributions, bounds, flow_axes, padding)
            .map_err(range_error)?;
        let track_subject = match alignment {
            TrackAlignmentSubject::Ordinary(bounds) => bounds.rect,
            TrackAlignmentSubject::Lanes(subject) => {
                subject.map_err(|error| layout_child_geometry_error(node, node, error))?
            }
        };
        let mut layout = report_grid_children(node, contributions, track_subject, reporting)?;
        debug_assert_eq!(
            layout
                .contributions
                .content_size_from_anchor(Point::ZERO)
                .ok(),
            Some(layout.visible_content_size)
        );
        if let Some(size) = contained_track_size {
            layout.visible_content_size = max_size(layout.visible_content_size, size);
        }
        Ok(layout)
    }

    pub(super) fn compute<'tree, Tree, M>(
        mut self,
        tree: &mut Tree,
    ) -> GridChildrenLayoutResult<Node, S, M>
    where
        Tree: Compute<'tree, M, Node = Node, Scalar = S>,
    {
        while let Some(child) = self.next(tree)? {
            let output = tree.compute_child(child.node, child.input)?;
            self.accept(tree, child, output)?;
        }
        self.finish()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct GridLines {
    pub(super) column_explicit_start: usize,
    pub(super) column_explicit_count: usize,
    pub(super) row_explicit_start: usize,
    pub(super) row_explicit_count: usize,
}

#[derive(Clone, Copy)]
pub(super) enum FinalGridChildGeometry<'a, S: LayoutScalar> {
    Ordinary {
        columns: &'a UsedGridAxisGeometryOf<S>,
        rows: &'a UsedGridAxisGeometryOf<S>,
    },
    Lanes {
        grid_axis: GridAxisKind,
    },
}

struct FinalGridChildRequest<'a, S: LayoutScalar> {
    input: ComputeInputOf<S>,
    parent_axes: FlowAxes,
    sizing_stage: GridSizingStage,
    geometry: FinalGridChildGeometry<'a, S>,
    standalone_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
}

type FinalGridChildResult<Node, S, M> = LayoutResultOf<
    Node,
    (
        ComputeOutputOf<S>,
        Option<ExecutedBaselineInventory<Node, S>>,
    ),
    S,
    M,
>;

fn compute_final_grid_child<'input, Tree, M>(
    tree: &mut Tree,
    container: Tree::Node,
    child: Tree::Node,
    context: SubgridChildParentContextInput<'input, '_, Tree::Node, Tree::Scalar>,
    request: FinalGridChildRequest<'_, Tree::Scalar>,
) -> FinalGridChildResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let input = request.input;
    let (mut output, executed_baselines) = if context.item.needs_context() {
        let (minimum, columns, rows) = match request.geometry {
            FinalGridChildGeometry::Ordinary { columns, rows } => {
                (request.standalone_minimum, Some(columns), Some(rows))
            }
            FinalGridChildGeometry::Lanes { .. } => (Size::NONE, None, None),
        };
        let child_axes = FlowAxes::new(
            context.child_style.writing_mode,
            context.child_style.direction,
        );
        let stage = request.sizing_stage.mapped(request.parent_axes, child_axes);
        // The nested grid uses the current parent's track and scrollbar context.
        let result = compute_grid_with_context_source(
            tree,
            child,
            input,
            minimum,
            context.container(),
            stage,
            |constants| {
                subgrid_child_parent_context_with_geometry(
                    context.for_scrollbar_pass(constants),
                    columns,
                    rows,
                )
                .map_err(|error| subgrid_child_context_container_error(container, child, error))
            },
        )?;
        let source = require_executed_baselines(
            container,
            child,
            child_axes,
            result.output,
            input,
            stage,
            result.executed_baselines,
        )?;
        (result.output, Some(source))
    } else {
        (tree.compute_child(child, input)?, None)
    };
    output.scroll_geometry = Some(
        retained_grid_child_scroll_geometry(
            context.child_style,
            output.size,
            output.content_size,
            context.padding,
            context.border,
            output.scroll_geometry,
        )
        .map_err(|error| layout_child_geometry_error(container, child, error))?,
    );
    Ok((output, executed_baselines))
}

pub(super) enum GridItemDisposition<S: LayoutScalar> {
    Skip,
    Hidden,
    Placed {
        area: GridArea<S>,
        space: GridItemLayoutSpace<S>,
    },
}

pub(super) struct GridChildItemsInput<'input, 'a, Node, S: LayoutScalar> {
    pub(super) children: &'a [Node],
    pub(super) item_count: usize,
    pub(super) style: &'a GridContainerProjection<'a, S>,
    pub(super) constants: &'a Constants<S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) minimum_functions: GridMinimumParentFunctions<'a, S>,
    pub(super) track_basis: LogicalSizeOf<Option<S>>,
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) named_columns: &'a NamedGridLines,
    pub(super) named_rows: &'a NamedGridLines,
    pub(super) area_facts: Option<&'a GridAreaNameFacts>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) placements: &'a GridPlacementContext<'input, Node, S>,
    pub(super) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    pub(super) sizing_stage: GridSizingStage,
    pub(super) geometry: FinalGridChildGeometry<'a, S>,
}

pub(super) struct PendingGridItems<'input, Node, S: LayoutScalar> {
    pub(super) in_flow: Vec<PendingGridItem<'input, Node, S>>,
    pub(super) absolute: Vec<PendingAbsoluteGridItem<Node>>,
}

type PendingGridItemsResult<'input, Node, S, M> =
    LayoutResultOf<Node, PendingGridItems<'input, Node, S>, S, M>;

pub(super) fn stage_hidden_grid_child<'input, Tree, M>(
    tree: &mut Tree,
    child: Tree::Node,
    source_index: usize,
    input: ComputeInputOf<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
        child,
        crate::output::NormalGeometryOf::metadata(Point::ZERO, Size::ZERO),
        NodeOutputOf::with_source_index(crate::SourceIndex::new(source_index)),
    ));
    tree.compute_child(child, input).map(|_| ())
}

pub(super) fn layout_grid_child_items<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: GridChildItemsInput<'input, '_, Tree::Node, Tree::Scalar>,
    mut place: impl FnMut(
        usize,
        &GridItemProjection<'input, Tree::Scalar>,
    ) -> GridItemDisposition<Tree::Scalar>,
) -> PendingGridItemsResult<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let GridChildItemsInput {
        children,
        item_count,
        style,
        constants,
        columns,
        rows,
        row_tracks,
        minimum_functions,
        track_basis,
        gap,
        named_columns,
        named_rows,
        area_facts,
        subgrid_report,
        placements,
        containing_auto_scrollbar_pass,
        sizing_stage,
        geometry,
    } = input;
    let flow = constants.flow_axes;
    let hidden_context =
        crate::ContainingLayoutContext::new(flow, crate::ParentFormattingContext::Grid);
    let hidden = match geometry {
        FinalGridChildGeometry::Ordinary { .. } => ComputeInputOf::hidden_in_containing_pass(
            hidden_context,
            containing_auto_scrollbar_pass,
        ),
        FinalGridChildGeometry::Lanes { .. } => ComputeInputOf::hidden(hidden_context),
    };
    let mut in_flow = Vec::new();
    let mut absolute = Vec::new();
    for (source_index, (child, placement)) in placements
        .checked_child_placements(children)
        .take(item_count)
        .enumerate()
    {
        let child_style = placements.item_input(source_index).clone();
        if child_style.display == Display::None {
            stage_hidden_grid_child(tree, child, source_index, hidden)?;
            continue;
        }
        if child_style.position == Position::Absolute {
            absolute.push((
                source_index,
                child,
                placement.absolute_column,
                placement.absolute_row,
            ));
            continue;
        }
        let (area, space) = match place(source_index, &child_style) {
            GridItemDisposition::Skip => continue,
            GridItemDisposition::Hidden => {
                stage_hidden_grid_child(tree, child, source_index, hidden)?;
                continue;
            }
            GridItemDisposition::Placed { area, space } => (area, space),
        };
        let containing_size = space.containing_size;
        let subgrid_item = subgrid_report.items[source_index];
        let FinalGridItemPreparation {
            sizing: item,
            input: child_input,
            padding,
            border,
            content_box_size: subgrid_content_box_size,
        } = prepare_final_grid_item::<Tree, M>(
            tree,
            child,
            &child_style,
            style,
            subgrid_item,
            space,
            GridItemRequestContext {
                containing_auto_scrollbar_pass,
                orthogonal_fallback: constants.child_orthogonal_fallback,
            },
        )?;
        let child_input = grid_item_baseline_demand(style, &child_style, None, child_input)
            .map_err(|error| grid_baseline_error(node, child, error))?;
        let child_container = placements.nested_container_input(source_index).copied();
        let child_context = SubgridChildParentContextInput {
            item: subgrid_item,
            parent_baseline_references: style.baseline_references,
            child_style: &child_style,
            child_container,
            area,
            content_box_size: subgrid_content_box_size,
            columns,
            rows,
            minimum_functions,
            gap,
            parent_named_columns: named_columns,
            parent_named_rows: named_rows,
            parent_area_facts: area_facts,
            margin: item.unresolved_margin,
            border,
            padding,
            scrollbar_gutter: Edges::ZERO,
        };
        let (output, executed_baselines) = compute_final_grid_child(
            tree,
            node,
            child,
            child_context,
            FinalGridChildRequest {
                input: child_input,
                parent_axes: flow,
                sizing_stage,
                geometry,
                standalone_minimum: item.standalone_intrinsic_minimum,
            },
        )?;
        let logical_output_size = flow.logical_size(output.size);
        let logical_margin = flow.logical_edges(item.unresolved_margin);
        let inline_axis = logical_grid_item_axis(
            area.size.inline,
            logical_output_size.inline,
            logical_margin.inline_start,
            logical_margin.inline_end,
            item.justify_self,
        );
        let block_axis = logical_grid_item_axis(
            area.size.block,
            logical_output_size.block,
            logical_margin.block_start,
            logical_margin.block_end,
            item.align_self,
        );
        let child_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let (reference, block_auto_margins, cyclic) = match geometry {
            FinalGridChildGeometry::Ordinary { .. } => (
                style.baseline_references.rows,
                logical_margin.block_start.is_none() || logical_margin.block_end.is_none(),
                grid_baseline_span_would_cycle(
                    row_tracks.get(area.row..area.row_end).unwrap_or(&[]),
                    track_basis.block,
                ),
            ),
            FinalGridChildGeometry::Lanes { grid_axis } => (
                if grid_axis == GridAxisKind::Row {
                    style.baseline_references.rows
                } else {
                    crate::GridBaselineReference::Physical
                },
                child_axes.line_over_edge(item.unresolved_margin).is_none()
                    || child_axes.line_under_edge(item.unresolved_margin).is_none(),
                false,
            ),
        };
        let participate = || {
            grid_baseline_participation(
                reference,
                item.align_self,
                block_auto_margins,
                cyclic,
                output,
                child_axes,
                flow.block_axis(),
            )
            .map_err(|error| grid_baseline_error(node, child, error))
        };
        let relative_offset = || {
            child_style
                .inset
                .zip_size(containing_size.map(Some), resolve_auto_optional)
                .transpose_with_node(tree, child)
                .map(|inset| logical_relative_inset_offset(inset, flow, child_style.position))
        };
        let (baseline_participation, logical_relative_offset) = match geometry {
            FinalGridChildGeometry::Ordinary { .. } => (participate()?, relative_offset()?),
            FinalGridChildGeometry::Lanes { .. } => {
                let offset = relative_offset()?;
                (participate()?, offset)
            }
        };
        let margin = flow.physical_edges(LogicalEdgesOf::new(
            inline_axis.margin_start,
            inline_axis.margin_end,
            block_axis.margin_start,
            block_axis.margin_end,
        ));
        let baselines = output.baselines();
        let overflow =
            UsedOverflow::from_computed(child_style.overflow, child_style.item_is_replaced);
        in_flow.push(PendingGridItem {
            executed_baselines: PendingExecutedBaselines::Uncaptured(executed_baselines),
            node: child,
            style: child_style,
            nested_container: child_container,
            source_index,
            area,
            output,
            horizontal_axis: inline_axis,
            vertical_axis: block_axis,
            child_flow_axes: child_axes,
            logical_relative_offset,
            first_baseline: baselines.first_or_synthesize_block_baseline(child_axes, output.size),
            last_baseline: baselines.last_or_synthesize_block_baseline(child_axes, output.size),
            location: Point::ZERO,
            block_offset: block_axis.offset,
            block_auto_margins,
            baseline_participation,
            margin,
            border,
            padding,
            overflow,
        });
    }
    Ok(PendingGridItems { in_flow, absolute })
}

pub(super) fn layout_grid_children<'input, Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    context: GridLayoutContext<'input, '_, <Tree as Traverse>::Node, Tree::Scalar>,
) -> GridChildPlacementResult<'input, Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let GridLayoutContext {
        scroll_box,
        style,
        constants,
        container_content_size,
        columns,
        rows,
        collapsed_columns,
        collapsed_rows,
        row_tracks,
        minimum_functions,
        track_basis,
        gap,
        column_gutters,
        row_gutters,
        lines,
        named_columns,
        named_rows,
        area_facts,
        template_area_expanded_axes,
        inherited_column_offset,
        inherited_row_offset,
        subgrid_report,
        parent_context,
        placements,
        containing_auto_scrollbar_pass,
        sizing_stage,
        execution_input,
    } = context;
    // Absolute children still require layout when either track axis is empty.
    // The ordinary loop separates them from in-flow placement and track sizing.
    let inherited_column_geometry = parent_context.columns.as_ref().map(|axis| &axis.geometry);
    let inherited_row_geometry = parent_context.rows.as_ref().map(|axis| &axis.geometry);
    let intrinsic_column_geometry = inherited_column_geometry.cloned().unwrap_or_else(|| {
        UsedGridAxisGeometryOf::from_sizing_gutters(columns.to_vec(), column_gutters)
    });
    let intrinsic_row_geometry = inherited_row_geometry
        .cloned()
        .unwrap_or_else(|| UsedGridAxisGeometryOf::from_sizing_gutters(rows.to_vec(), row_gutters));
    let logical_content_size = LogicalSizeOf::new(
        intrinsic_column_geometry.total_extent(),
        intrinsic_row_geometry.total_extent(),
    );
    let physical_content_size = grid_area_physical_size(constants.flow_axes, logical_content_size);
    let legacy_content_box_size =
        constants
            .node_inner_size
            .unwrap_or(if style.writing_mode.is_vertical() {
                physical_content_size
            } else {
                container_content_size
            });
    let logical_content_box_size = constants
        .flow_axes
        .logical_size(constants.node_inner_size.unwrap_or(container_content_size));
    let containing_size = constants
        .node_outer_size
        .unwrap_or(container_content_size + constants.content_box_inset.sum_axes());
    let logical_content_box_inset = constants
        .flow_axes
        .logical_edges(constants.content_box_inset);
    let alignment_free_space = logical_content_box_size - logical_content_size;
    let ordinary_column_alignment = ordinary_grid_axis_alignment(
        alignment_free_space.inline,
        column_gutters,
        style.justify_content.unwrap_or(AlignContent::Stretch),
    );
    let ordinary_row_alignment = ordinary_grid_axis_alignment(
        alignment_free_space.block,
        row_gutters,
        style.align_content.unwrap_or(AlignContent::Stretch),
    );
    let column_alignment = GridAlignment {
        start: ordinary_column_alignment.start,
        gap: gap.inline,
    };
    let row_alignment = GridAlignment {
        start: ordinary_row_alignment.start,
        gap: gap.block,
    };
    let column_geometry = inherited_column_geometry.cloned().unwrap_or_else(|| {
        UsedGridAxisGeometryOf::from_active_boundary_gutters(
            columns.to_vec(),
            collapsed_columns.to_vec(),
            column_gutters.active_boundary_after().to_vec(),
            ordinary_column_alignment.gutter_after,
        )
    });
    let row_geometry = inherited_row_geometry.cloned().unwrap_or_else(|| {
        UsedGridAxisGeometryOf::from_active_boundary_gutters(
            rows.to_vec(),
            collapsed_rows.to_vec(),
            row_gutters.active_boundary_after().to_vec(),
            ordinary_row_alignment.gutter_after,
        )
    });
    let logical_column_geometry = column_geometry.clone().translated(
        inherited_column_offset.unwrap_or(Tree::Scalar::ZERO)
            + logical_content_box_inset.inline_start
            + column_alignment.start,
    );
    let logical_row_geometry = row_geometry.clone().translated(
        inherited_row_offset.unwrap_or(Tree::Scalar::ZERO)
            + logical_content_box_inset.block_start
            + row_alignment.start,
    );
    let logical_column_offsets = (0..columns.len())
        .filter_map(|line| logical_column_geometry.line_offset(line))
        .collect::<Vec<_>>();
    let logical_row_offsets = (0..rows.len())
        .filter_map(|line| logical_row_geometry.line_offset(line))
        .collect::<Vec<_>>();
    let content_box_left = effective_content_box_left(constants, container_content_size);
    let row_offsets = grid_axis_offsets(GridAxisOffsetsInput {
        style,
        axis: GridAxisKind::Row,
        tracks: rows,
        geometry: &row_geometry,
        inherited_offset: inherited_row_offset,
        content_box_left,
        content_box_size: legacy_content_box_size,
        content_box_inset: constants.content_box_inset,
        alignment: row_alignment,
    });
    let children = tree.children(node).collect::<Vec<_>>();
    let placed_areas = resolve_grid_child_areas_with_geometry(
        ResolveGridChildAreasInput {
            children: &children,
            placements,
            style,
            columns,
            rows,
            gap,
            lines,
        },
        Some(&column_geometry),
        Some(&row_geometry),
    );
    let mut child_contributions = Vec::new();
    let PendingGridItems {
        in_flow: mut pending_items,
        absolute: pending_absolute,
    } = layout_grid_child_items(
        tree,
        node,
        GridChildItemsInput {
            children: &children,
            item_count: placed_areas.len().min(subgrid_report.items.len()),
            style,
            constants,
            columns,
            rows,
            row_tracks,
            minimum_functions,
            track_basis,
            gap,
            named_columns: &named_columns,
            named_rows: &named_rows,
            area_facts: area_facts.as_ref(),
            subgrid_report,
            placements,
            containing_auto_scrollbar_pass,
            sizing_stage,
            geometry: FinalGridChildGeometry::Ordinary {
                columns: &column_geometry,
                rows: &row_geometry,
            },
        },
        |source_index, _| {
            let Some(area) = placed_areas[source_index] else {
                return GridItemDisposition::Skip;
            };
            if area.row >= rows.len() || area.column >= columns.len() {
                return GridItemDisposition::Hidden;
            }
            let size = grid_area_physical_size(constants.flow_axes, area.size);
            GridItemDisposition::Placed {
                area,
                space: GridItemLayoutSpace {
                    alignment_size: size,
                    containing_size: size,
                    natural_stacking_margin_box: None,
                },
            }
        },
    )?;

    let executed_frame = ExecutedBaselineFrame {
        style,
        size: containing_size,
        execution_input,
        stage: sizing_stage,
        columns: sizing_stage
            .has_resolved_axis(GridAxisKind::Column)
            .then_some(ExecutedBaselineAxisFrame {
                starts: logical_column_geometry.line_offsets(),
                sizes: logical_column_geometry.sizes(),
            }),
        rows: sizing_stage.has_resolved_axis(GridAxisKind::Row).then_some(
            ExecutedBaselineAxisFrame {
                starts: logical_row_geometry.line_offsets(),
                sizes: logical_row_geometry.sizes(),
            },
        ),
        sizing: ExecutedBaselineSizing {
            row_tracks,
            track_basis,
        },
    };
    let executed_inventory =
        capture_executed_baselines(node, executed_frame, &mut pending_items, subgrid_report)?;
    let ancestor_baseline_groups =
        final_ancestor_baseline_groups(&executed_inventory).with_parent_context(parent_context);
    let baseline_group_set = ancestor_baseline_groups.placement_groups();
    refresh_subgrid_items_with_baselines(
        tree,
        SubgridBaselineRefreshInput {
            node,
            container_style: style,
            columns,
            rows,
            minimum_functions,
            column_geometry: &column_geometry,
            row_geometry: &row_geometry,
            row_tracks,
            track_basis,
            gap,
            named_columns: named_columns.clone(),
            named_rows: named_rows.clone(),
            area_facts: area_facts.clone(),
            template_area_expanded_axes,
            subgrid_report,
            ancestor_baseline_groups: &ancestor_baseline_groups,
            executed_frame,
            original_executed: &executed_inventory,
            orthogonal_fallback: constants.child_orthogonal_fallback,
            containing_auto_scrollbar_pass,
            sizing_stage,
        },
        &mut pending_items,
        |item| {
            let size = constants.flow_axes.physical_size(item.area.size);
            GridItemLayoutSpace {
                alignment_size: size,
                containing_size: size,
                natural_stacking_margin_box: None,
            }
        },
    )?;
    let published_group_set = baseline_groups(
        &pending_items,
        rows.len(),
        columns.len(),
        constants.flow_axes,
    );

    stage_ordinary_grid_items(
        tree,
        OrdinaryGridItemPlacement {
            node,
            style,
            flow_axes: constants.flow_axes,
            containing_size,
            logical_column_offsets: &logical_column_offsets,
            logical_row_offsets: &logical_row_offsets,
            column_geometry: &column_geometry,
            row_geometry: &row_geometry,
            row_tracks,
            track_basis,
            subgrid_report,
            ancestor_baseline_groups: &ancestor_baseline_groups,
        },
        &mut pending_items,
        &mut child_contributions,
    )?;
    let baselines = if parent_context.has_inherited_axis() {
        grid_container_baselines(
            &pending_items,
            &baseline_group_set,
            &row_offsets,
            rows,
            constants.flow_axes,
        )
    } else {
        logical_grid_container_baselines(
            &pending_items,
            &baseline_group_set,
            &logical_row_offsets,
            rows,
            constants.flow_axes,
            containing_size,
        )
    };

    let track_subject = grid_track_alignment_subject(GridTrackAlignmentGeometry {
        sizes: LogicalSizeOf::new(columns, rows),
        offsets: LogicalSizeOf::new(&logical_column_offsets, &logical_row_offsets),
        fallback_origin: LogicalPointOf::new(
            logical_content_box_inset.inline_start,
            logical_content_box_inset.block_start,
        ),
        flow_axes: constants.flow_axes,
        containing_size,
        stacking_range: None,
    })
    .map_err(|error| layout_child_geometry_error(node, node, error))?;
    // A bounded probe can carry an estimate for an unissued axis.
    let resolved_axes = LogicalSizeOf::new(
        sizing_stage.has_resolved_axis(GridAxisKind::Column),
        sizing_stage.has_resolved_axis(GridAxisKind::Row),
    );
    let track_bounds = GridTrackScrollBounds {
        rect: track_subject,
        present: LogicalSizeOf::new(
            resolved_axes.inline && !columns.is_empty(),
            resolved_axes.block && !rows.is_empty(),
        ),
    };
    GridChildPlacement::begin(
        PositionedGridPlacement {
            node,
            style,
            constants,
            containing_size,
            scroll_box,
            placements,
            column_geometry: logical_column_geometry,
            row_geometry: logical_row_geometry,
            lines,
            containing_auto_scrollbar_pass,
            bounds: PositionedGridBounds::Ordinary(track_bounds),
        },
        pending_absolute,
        child_contributions,
        TrackAlignmentSubject::Ordinary(track_bounds),
        GridLayoutReporting {
            active_alignment_axes: LogicalSizeOf::new(
                resolved_axes.inline && style.justify_content.is_some(),
                resolved_axes.block && style.align_content.is_some(),
            ),
            flow_axes: constants.flow_axes,
            baselines: baselines.baselines,
            baseline_groups: published_group_set,
            execution_input,
            executed_inventory,
        },
    )
}

pub(super) fn grid_area_inline_offset<S: LayoutScalar>(offsets: &[S], area: GridArea<S>) -> S {
    grid_area_track_offset(offsets, area.column, area.column_end)
}

fn grid_area_logical_origin<S: LayoutScalar>(
    column_offsets: &[S],
    row_offsets: &[S],
    area: GridArea<S>,
) -> LogicalPointOf<S> {
    LogicalPointOf::new(
        grid_area_track_offset(column_offsets, area.column, area.column_end),
        grid_area_track_offset(row_offsets, area.row, area.row_end),
    )
}

pub(super) fn grid_area_track_offset<S: LayoutScalar>(
    offsets: &[S],
    start: usize,
    end: usize,
) -> S {
    offsets
        .get(start..end)
        .and_then(|offsets| offsets.iter().copied().reduce(S::min))
        .unwrap_or(S::ZERO)
}

pub(super) fn grid_area_physical_size<S: LayoutScalar>(
    containing_flow_axes: FlowAxes,
    size: LogicalSizeOf<S>,
) -> Size<S> {
    containing_flow_axes.physical_size(size)
}

pub(super) fn grid_axis_logical_offsets<S: LayoutScalar>(
    tracks: &[S],
    inherited_offset: Option<S>,
    content_box_start: S,
    alignment: GridAlignment<S>,
) -> Vec<S> {
    offsets(
        tracks,
        inherited_offset.unwrap_or(S::ZERO) + content_box_start + alignment.start,
        alignment.gap,
    )
}

#[derive(Clone, Copy)]
pub(super) struct GridAxisOffsetsInput<'a, S: LayoutScalar = Scalar> {
    pub(super) style: &'a GridContainerProjection<'a, S>,
    pub(super) axis: GridAxisKind,
    pub(super) tracks: &'a [S],
    pub(super) geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) inherited_offset: Option<S>,
    pub(super) content_box_left: S,
    pub(super) content_box_size: Size<S>,
    pub(super) content_box_inset: Edges<S>,
    pub(super) alignment: GridAlignment<S>,
}

pub(super) fn grid_axis_offsets<S: LayoutScalar>(input: GridAxisOffsetsInput<'_, S>) -> Vec<S> {
    let flow_axes = FlowAxes::new(input.style.writing_mode, input.style.direction);
    let logical_axis = input.axis.logical_axis();
    let physical_axis = match logical_axis {
        LogicalAxis::Inline => flow_axes.inline_axis(),
        LogicalAxis::Block => flow_axes.block_axis(),
    };
    let physical_start = match physical_axis {
        PhysicalAxis::Horizontal
            if input.axis == GridAxisKind::Column
                && input.inherited_offset.is_none()
                && flow_axes
                    .logical_axis_progression(logical_axis)
                    .is_decreasing() =>
        {
            input.content_box_left
        }
        PhysicalAxis::Horizontal => input.content_box_inset.left,
        PhysicalAxis::Vertical => input.content_box_inset.top,
    };
    let start = input
        .inherited_offset
        .map(|offset| offset + physical_start)
        .unwrap_or(physical_start);
    let logical_content_box_size = flow_axes.logical_size(input.content_box_size);
    let extent = match logical_axis {
        LogicalAxis::Inline => logical_content_box_size.inline,
        LogicalAxis::Block => logical_content_box_size.block,
    };

    if flow_axes
        .logical_axis_progression(logical_axis)
        .is_decreasing()
    {
        input
            .tracks
            .iter()
            .copied()
            .enumerate()
            .map(|(index, size)| {
                start + extent
                    - input.alignment.start
                    - input.geometry.line_offset(index).unwrap_or(S::ZERO)
                    - size
            })
            .collect()
    } else {
        (0..input.tracks.len())
            .map(|index| {
                start + input.alignment.start + input.geometry.line_offset(index).unwrap_or(S::ZERO)
            })
            .collect()
    }
}

fn grid_physical_axis_progression(
    writing_mode: crate::WritingMode,
    direction: Direction,
    axis: PhysicalAxis,
) -> PhysicalProgression {
    FlowAxes::new(writing_mode, direction).physical_axis_progression(axis)
}

pub(super) struct PendingGridItem<'input, Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) style: GridItemProjection<'input, S>,
    pub(super) nested_container: Option<GridContainerInput<'input, S>>,
    pub(super) source_index: usize,
    pub(super) area: GridArea<S>,
    pub(super) output: ComputeOutputOf<S>,
    pub(super) executed_baselines: PendingExecutedBaselines<Node, S>,
    pub(super) horizontal_axis: ResolvedGridItemAxis<S>,
    pub(super) vertical_axis: ResolvedGridItemAxis<S>,
    pub(super) child_flow_axes: FlowAxes,
    pub(super) logical_relative_offset: LogicalPointOf<S>,
    pub(super) first_baseline: PhysicalBaseline<S>,
    pub(super) last_baseline: PhysicalBaseline<S>,
    pub(super) location: Point<S>,
    pub(super) block_offset: S,
    pub(super) block_auto_margins: bool,
    pub(super) baseline_participation: BaselineParticipation,
    pub(super) margin: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) padding: Edges<S>,
    pub(super) overflow: UsedOverflow,
}

#[derive(Clone, Copy)]
pub(super) struct GridItemSizing<S: LayoutScalar = Scalar, Available = S> {
    pub(super) known: Size<Option<S>>,
    pub(super) available: Size<Available>,
    pub(super) unresolved_margin: Edges<Option<S>>,
    pub(super) justify_self: AlignItems,
    pub(super) align_self: AlignItems,
    pub(super) standalone_intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
}

/// The actual final item property frame shared by placement and baseline
/// census. Available area never becomes a known size except through the
/// canonical authored/stretch/aspect/minmax resolution below.
pub(super) struct FinalGridItemPreparation<S: LayoutScalar> {
    pub(super) sizing: GridItemSizing<S>,
    pub(super) input: ComputeInputOf<S>,
    pub(super) padding: Edges<S>,
    pub(super) border: Edges<S>,
    pub(super) content_box_size: Size<S>,
}

#[derive(Clone, Copy)]
pub(super) struct GridItemRequestContext<S: LayoutScalar> {
    pub(super) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    pub(super) orthogonal_fallback: OrthogonalFallbackOf<S>,
}

#[derive(Clone, Copy)]
pub(super) struct GridItemLayoutSpace<S: LayoutScalar> {
    pub(super) alignment_size: Size<S>,
    pub(super) containing_size: Size<S>,
    pub(super) natural_stacking_margin_box: Option<S>,
}

pub(super) fn prepare_final_grid_item<'input, Tree, M>(
    tree: &Tree,
    child: Tree::Node,
    child_style: &GridItemProjection<'input, Tree::Scalar>,
    container_style: &GridContainerProjection<'_, Tree::Scalar>,
    subgrid_item: SubgridItemReport<Tree::Node>,
    space: GridItemLayoutSpace<Tree::Scalar>,
    request_context: GridItemRequestContext<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, FinalGridItemPreparation<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let flow_axes = FlowAxes::new(container_style.writing_mode, container_style.direction);
    let mut sizing = grid_item_sizing_for_grid_flow::<Tree, M>(
        child,
        child_style,
        container_style,
        Some(subgrid_item),
        space.alignment_size,
        space.containing_size.map(Some),
        flow_axes,
    )?;
    apply_final_subgrid_axis_constraints(
        &mut sizing,
        subgrid_item,
        flow_axes,
        FlowAxes::new(child_style.writing_mode, child_style.direction),
    );
    if let Some(natural) = space.natural_stacking_margin_box {
        super::lanes::normalize_grid_lanes_stacking_axis_sizing(
            child_style,
            container_style,
            lane_axis_for_grid_lanes(container_style),
            flow_axes,
            natural,
            &mut sizing,
        );
    }
    let area_parent = space.containing_size.map(Some);
    let padding = flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.padding,
            area_parent,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let border = flow_axes
        .zip_physical_edges_with_inline_extent(
            child_style.border,
            area_parent,
            resolve_length_or_zero,
        )
        .transpose_with_node(tree, child)?;
    let margin = sizing
        .unresolved_margin
        .map(|value| value.unwrap_or(Tree::Scalar::ZERO));
    let content_box_size =
        (space.alignment_size - margin.sum_axes() - padding.sum_axes() - border.sum_axes())
            .max(Size::ZERO);
    let available = sizing.available.map(|value| value.max(Tree::Scalar::ZERO));
    let available = Size::new(
        AxisSizingRequestOf::new(
            crate::layout_math::checked_available_space(available.width).map_err(|_| {
                crate::error::value_resolution_error(
                    child,
                    LengthResolutionStatus::InvalidNumeric {
                        value: available.width,
                    },
                )
            })?,
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(
            crate::layout_math::checked_available_space(available.height).map_err(|_| {
                crate::error::value_resolution_error(
                    child,
                    LengthResolutionStatus::InvalidNumeric {
                        value: available.height,
                    },
                )
            })?,
            SizingOperation::Normal,
        ),
    );
    let input = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        sizing.known,
        area_parent,
        crate::ContainingLayoutContext::new(flow_axes, crate::ParentFormattingContext::Grid),
        SizingRequestOf::new(available, request_context.orthogonal_fallback),
    )
    .with_containing_auto_scrollbar_pass(request_context.containing_auto_scrollbar_pass);
    Ok(FinalGridItemPreparation {
        sizing,
        input,
        padding,
        border,
        content_box_size,
    })
}

struct GridItemBoxEdges<S: LayoutScalar> {
    padding: Edges<S>,
    border: Edges<S>,
    box_sizing_adjustment: Size<S>,
}

fn grid_item_box_edges<S: LayoutScalar>(
    child_style: &GridItemProjection<'_, S>,
    containing_flow_axes: FlowAxes,
    containing_physical_size: Size<Option<S>>,
) -> Result<GridItemBoxEdges<S>, SizingResolutionError<S>> {
    let padding =
        transpose_edges_result(containing_flow_axes.zip_physical_edges_with_inline_extent(
            child_style.padding,
            containing_physical_size,
            |length, basis| resolve_length_or_zero(length, basis),
        ))?;
    let border =
        transpose_edges_result(containing_flow_axes.zip_physical_edges_with_inline_extent(
            child_style.border,
            containing_physical_size,
            |length, basis| resolve_length_or_zero(length, basis),
        ))?;
    let box_sizing_adjustment = if child_style.box_sizing == BoxSizing::ContentBox {
        (padding + border).sum_axes()
    } else {
        Size::ZERO
    };
    Ok(GridItemBoxEdges {
        padding,
        border,
        box_sizing_adjustment,
    })
}

enum GridItemMinimum<S: LayoutScalar> {
    Resolved(Option<S>),
    StandaloneIntrinsic(StandaloneIntrinsicMinimum),
}

impl<S: LayoutScalar> GridItemMinimum<S> {
    const fn resolved(&self) -> Option<S> {
        match self {
            Self::Resolved(value) => *value,
            Self::StandaloneIntrinsic(_) => None,
        }
    }

    const fn standalone_intrinsic(&self) -> Option<StandaloneIntrinsicMinimum> {
        match self {
            Self::Resolved(_) => None,
            Self::StandaloneIntrinsic(minimum) => Some(*minimum),
        }
    }
}

pub(super) fn grid_item_sizing_for_grid_flow<'input, Tree, M>(
    child: <Tree as Traverse>::Node,
    child_style: &GridItemProjection<'input, Tree::Scalar>,
    container_style: &GridContainerProjection<'_, Tree::Scalar>,
    subgrid_item: Option<SubgridItemReport<<Tree as Traverse>::Node>>,
    area_size: Size<Tree::Scalar>,
    containing_physical_size: Size<Option<Tree::Scalar>>,
    grid_flow_axes: FlowAxes,
) -> LayoutResultOf<<Tree as Traverse>::Node, GridItemSizing<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    grid_item_sizing_with_grid_flow_status(
        child_style,
        container_style,
        Size::new(
            subgrid_item.is_some_and(|item| item.column.can_inherit()),
            subgrid_item.is_some_and(|item| item.row.can_inherit()),
        ),
        area_size,
        containing_physical_size,
        grid_flow_axes,
    )
    .map_err(|error| sizing_resolution_error(child, error))
}

pub(super) fn grid_item_sizing_with_grid_flow_status<'input, S: LayoutScalar>(
    child_style: &GridItemProjection<'input, S>,
    container_style: &GridContainerProjection<'_, S>,
    subgrid_requests: Size<bool>,
    area_size: Size<S>,
    containing_physical_size: Size<Option<S>>,
    grid_flow_axes: FlowAxes,
) -> Result<GridItemSizing<S>, SizingResolutionError<S>> {
    let sizing = grid_item_sizing_with_optional_alignment(
        child_style,
        container_style,
        subgrid_requests,
        area_size.map(Some),
        containing_physical_size,
        grid_flow_axes,
    )?;
    Ok(GridItemSizing {
        known: sizing.known,
        available: sizing
            .available
            .map(|extent| extent.expect("both final Grid alignment extents are available")),
        unresolved_margin: sizing.unresolved_margin,
        justify_self: sizing.justify_self,
        align_self: sizing.align_self,
        standalone_intrinsic_minimum: sizing.standalone_intrinsic_minimum,
    })
}

pub(super) fn grid_item_sizing_with_optional_alignment<'input, S: LayoutScalar>(
    child_style: &GridItemProjection<'input, S>,
    container_style: &GridContainerProjection<'_, S>,
    subgrid_requests: Size<bool>,
    area_size: Size<Option<S>>,
    containing_physical_size: Size<Option<S>>,
    grid_flow_axes: FlowAxes,
) -> Result<GridItemSizing<S, Option<S>>, SizingResolutionError<S>> {
    let container_flow_axes =
        crate::geometry::FlowAxes::new(container_style.writing_mode, container_style.direction);
    let unresolved_margin =
        transpose_edges_result(container_flow_axes.zip_physical_edges_with_inline_extent(
            child_style.margin,
            containing_physical_size,
            |length, basis| resolve_auto_optional(length, basis),
        ))?;
    let margin = unresolved_margin.map(|margin| margin.unwrap_or(S::ZERO));
    let logical_area_size = grid_flow_axes.logical_size(area_size);
    let logical_margin = grid_flow_axes.logical_edges(margin);
    let remaining = |area: Option<S>, margin| {
        area.map(|extent| crate::layout_math::remaining_available_extent(extent, margin))
            .transpose()
    };
    let logical_available = LogicalSizeOf::new(
        remaining(logical_area_size.inline, logical_margin.inline_sum())?,
        remaining(logical_area_size.block, logical_margin.block_sum())?,
    );
    let available = grid_flow_axes.physical_size(logical_available);
    let GridItemBoxEdges {
        padding,
        border,
        box_sizing_adjustment,
    } = grid_item_box_edges(child_style, container_flow_axes, containing_physical_size)?;
    let area_parent = containing_physical_size;
    let deferred_axes = child_style.flex_preferred_axes;
    let child_owns_preference = deferred_axes.width || deferred_axes.height;
    let logical_deferred = grid_flow_axes.logical_size(deferred_axes);
    // A symbolic Flex preference is not an automatic size that ratio transfer
    // or stretch can supply. Leave it open for the child formatter.
    let preferred_ratio = child_style.aspect_ratio.filter(|_| !child_owns_preference);
    let algorithm = sizing_algorithm_for_grid_display(container_style.display);
    let raw_preferred = Size::new(
        resolve_preferred_optional(
            &child_style.size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )?,
        resolve_preferred_optional(
            &child_style.size.height,
            algorithm,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )?,
    );
    let inherent_size = raw_preferred
        .apply_aspect_ratio(preferred_ratio)
        .add_optional(box_sizing_adjustment);
    let minimum = Size::new(
        resolve_standalone_subgrid_item_minimum_optional(
            &child_style.min_size.width,
            child_style,
            subgrid_requests,
            algorithm,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )?,
        resolve_standalone_subgrid_item_minimum_optional(
            &child_style.min_size.height,
            child_style,
            subgrid_requests,
            algorithm,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )?,
    );
    let standalone_intrinsic_minimum = Size::new(
        minimum.width.standalone_intrinsic(),
        minimum.height.standalone_intrinsic(),
    );
    let raw_min = Size::new(minimum.width.resolved(), minimum.height.resolved());
    let raw_max = Size::new(
        resolve_maximum_optional(
            &child_style.max_size.width,
            algorithm,
            PhysicalAxis::Horizontal,
            area_parent.width,
            true,
        )?,
        resolve_maximum_optional(
            &child_style.max_size.height,
            algorithm,
            PhysicalAxis::Vertical,
            area_parent.height,
            true,
        )?,
    );
    let (raw_min, raw_max) =
        if let Some(ratio) = child_style.aspect_ratio.filter(|_| child_owns_preference) {
            crate::layout_math::transfer_aspect_ratio_limits(
                raw_preferred,
                raw_min,
                raw_max,
                ratio.get(),
            )
        } else {
            (raw_min, raw_max)
        };
    let min_size = raw_min
        .add_optional(box_sizing_adjustment)
        .or((padding + border).sum_axes().map(Some))
        .max_optional((padding + border).sum_axes().map(Some))
        .apply_aspect_ratio(preferred_ratio);
    let max_size = raw_max
        .apply_aspect_ratio(preferred_ratio)
        .add_optional(box_sizing_adjustment);
    let logical_inherent_size = grid_flow_axes.logical_size(inherent_size);
    let logical_style_size =
        grid_flow_axes.logical_size(Size::new(&child_style.size.width, &child_style.size.height));
    let justify_self = resolve_grid_item_normal_alignment(
        child_style.justify_self,
        container_style.justify_items,
        child_style.item_is_replaced,
        logical_style_size.inline.is_auto(),
        if logical_inherent_size.inline.is_some() || !logical_style_size.inline.is_auto() {
            AlignItems::Start
        } else {
            AlignItems::Stretch
        },
    );
    let align_self = resolve_grid_item_normal_alignment(
        child_style.align_self,
        container_style.align_items,
        child_style.item_is_replaced,
        logical_style_size.block.is_auto(),
        if logical_inherent_size.block.is_some()
            || !logical_style_size.block.is_auto()
            || (child_style.aspect_ratio.is_some()
                && grid_flow_axes
                    .logical_size(Size::new(
                        &child_style.min_size.width,
                        &child_style.min_size.height,
                    ))
                    .block
                    .is_auto())
        {
            AlignItems::Start
        } else {
            AlignItems::Stretch
        },
    );
    let logical_unresolved_margin = grid_flow_axes.logical_edges(unresolved_margin);
    let inline_stretches = !logical_deferred.inline
        && logical_unresolved_margin.inline_start.is_some()
        && logical_unresolved_margin.inline_end.is_some()
        && justify_self == AlignItems::Stretch;
    let block_stretches = !logical_deferred.block
        && logical_unresolved_margin.block_start.is_some()
        && logical_unresolved_margin.block_end.is_some()
        && align_self == AlignItems::Stretch;
    let logical_known = LogicalSizeOf::new(
        logical_inherent_size
            .inline
            .or(logical_available.inline.filter(|_| inline_stretches)),
        logical_inherent_size
            .block
            .or(logical_available.block.filter(|_| block_stretches)),
    );
    let known = if preferred_ratio.is_some()
        && logical_inherent_size.inline.is_none()
        && logical_known.block.is_some()
        && block_stretches
        && !grid_flow_axes
            .logical_size(Size::new(
                &child_style.min_size.width,
                &child_style.min_size.height,
            ))
            .block
            .is_auto()
    {
        grid_flow_axes
            .physical_size(LogicalSizeOf::new(None, logical_known.block))
            .apply_aspect_ratio(preferred_ratio)
    } else {
        grid_flow_axes
            .physical_size(logical_known)
            .apply_aspect_ratio(preferred_ratio)
    }
    .clamp_max_before_min_optional(min_size, max_size);

    Ok(GridItemSizing {
        known,
        available,
        unresolved_margin,
        justify_self,
        align_self,
        standalone_intrinsic_minimum,
    })
}

fn resolve_standalone_subgrid_item_minimum_optional<'input, S: LayoutScalar>(
    value: &MinSizeOf<S>,
    child_style: &GridItemProjection<'input, S>,
    subgrid_requests: Size<bool>,
    algorithm: SizingAlgorithm,
    physical_axis: PhysicalAxis,
    basis: Option<S>,
    missing_basis_is_indefinite: bool,
) -> Result<GridItemMinimum<S>, SizingResolutionError<S>> {
    let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
    let queried_axis =
        if grid_axis_physical_axis(child_flow_axes, GridAxisKind::Column) == physical_axis {
            GridAxisKind::Column
        } else {
            GridAxisKind::Row
        };
    let other_axis = match queried_axis {
        GridAxisKind::Column => GridAxisKind::Row,
        GridAxisKind::Row => GridAxisKind::Column,
    };
    let queried_axis_requested = match queried_axis {
        GridAxisKind::Column => subgrid_requests.width,
        GridAxisKind::Row => subgrid_requests.height,
    };
    let other_axis_requested = match other_axis {
        GridAxisKind::Column => subgrid_requests.width,
        GridAxisKind::Row => subgrid_requests.height,
    };
    if (value.is_min_content() || value.is_max_content())
        && !queried_axis_requested
        && other_axis_requested
    {
        let minimum = if value.is_min_content() {
            StandaloneIntrinsicMinimum::MinContent
        } else {
            StandaloneIntrinsicMinimum::MaxContent
        };
        return Ok(GridItemMinimum::StandaloneIntrinsic(minimum));
    }
    resolve_minimum_optional(
        value,
        algorithm,
        physical_axis,
        basis,
        missing_basis_is_indefinite,
    )
    .map(GridItemMinimum::Resolved)
}

pub(crate) fn resolve_grid_item_normal_alignment(
    item_alignment: Option<AlignItems>,
    container_alignment: Option<AlignItems>,
    item_is_replaced: bool,
    axis_is_auto_sized: bool,
    non_replaced_normal: AlignItems,
) -> AlignItems {
    item_alignment.or(container_alignment).unwrap_or({
        if item_is_replaced && axis_is_auto_sized {
            AlignItems::Start
        } else {
            non_replaced_normal
        }
    })
}

fn transpose_edges_result<T, S: LayoutScalar>(
    edges: Edges<Result<T, LengthResolutionStatus<S>>>,
) -> Result<Edges<T>, LengthResolutionStatus<S>> {
    Ok(Edges::new(
        edges.top?,
        edges.right?,
        edges.bottom?,
        edges.left?,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PhysicalGridItemAxis<S: LayoutScalar = Scalar> {
    pub(super) area_size: S,
    pub(super) size: S,
    pub(super) margin_start: Option<S>,
    pub(super) margin_end: Option<S>,
    pub(super) alignment: AlignItems,
    pub(super) progression: PhysicalProgression,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ResolvedGridItemAxis<S: LayoutScalar = Scalar> {
    pub(super) offset: S,
    pub(super) margin_start: S,
    pub(super) margin_end: S,
}

#[derive(Clone, Copy)]
struct GridItemPhysicalAlignment {
    horizontal: AlignItems,
    vertical: AlignItems,
}

fn grid_item_physical_alignment(
    writing_mode: crate::WritingMode,
    justify_self: AlignItems,
    align_self: AlignItems,
) -> GridItemPhysicalAlignment {
    if writing_mode.is_vertical() {
        GridItemPhysicalAlignment {
            horizontal: align_self,
            vertical: justify_self,
        }
    } else {
        GridItemPhysicalAlignment {
            horizontal: justify_self,
            vertical: align_self,
        }
    }
}

pub(super) fn logical_grid_item_axis<S: LayoutScalar>(
    area_size: S,
    size: S,
    margin_start: Option<S>,
    margin_end: Option<S>,
    alignment: AlignItems,
) -> ResolvedGridItemAxis<S> {
    resolve_grid_item_axis(
        area_size,
        size,
        margin_start,
        margin_end,
        alignment,
        PhysicalProgression::Increasing,
    )
}

pub(super) fn physical_grid_item_axis<S: LayoutScalar>(
    axis: PhysicalGridItemAxis<S>,
) -> ResolvedGridItemAxis<S> {
    let PhysicalGridItemAxis {
        area_size,
        size,
        margin_start,
        margin_end,
        alignment,
        progression,
    } = axis;
    resolve_grid_item_axis(
        area_size,
        size,
        margin_start,
        margin_end,
        alignment,
        progression,
    )
}

fn resolve_grid_item_axis<S: LayoutScalar>(
    area_size: S,
    size: S,
    margin_start: Option<S>,
    margin_end: Option<S>,
    alignment: AlignItems,
    progression: PhysicalProgression,
) -> ResolvedGridItemAxis<S> {
    let non_auto_start = margin_start.unwrap_or(S::ZERO);
    let non_auto_end = margin_end.unwrap_or(S::ZERO);
    let raw_free_space = area_size - size - non_auto_start - non_auto_end;
    let free_space = raw_free_space.max(S::ZERO);
    let auto_margin_count = usize::from(margin_start.is_none()) + usize::from(margin_end.is_none());
    let auto_margin = if auto_margin_count > 0 {
        free_space / S::from_usize(auto_margin_count)
    } else {
        S::ZERO
    };
    let resolved_start = margin_start.unwrap_or(auto_margin);
    let resolved_end = margin_end.unwrap_or(auto_margin);
    let alignment = alignment.safe_fallback(raw_free_space);
    let offset = match alignment {
        AlignItems::Start | AlignItems::FlexStart | AlignItems::Baseline | AlignItems::Stretch => {
            if progression.is_decreasing() {
                area_size - size - resolved_end
            } else {
                resolved_start
            }
        }
        AlignItems::End | AlignItems::FlexEnd | AlignItems::LastBaseline => {
            if progression.is_decreasing() {
                resolved_start
            } else {
                area_size - size - resolved_end
            }
        }
        AlignItems::Center => (area_size - size + resolved_start - resolved_end) / S::from_f64(2.0),
        AlignItems::Safe(_) | AlignItems::Unsafe(_) => {
            unreachable!("safe_fallback returns unqualified item alignment")
        }
    };

    ResolvedGridItemAxis {
        offset,
        margin_start: resolved_start,
        margin_end: resolved_end,
    }
}
pub(super) fn logical_relative_inset_offset<S: LayoutScalar>(
    inset: Edges<Option<S>>,
    flow_axes: FlowAxes,
    position: Position,
) -> LogicalPointOf<S> {
    if position != Position::Relative {
        return LogicalPointOf::new(S::ZERO, S::ZERO);
    }

    let inset = flow_axes.logical_edges(inset);
    LogicalPointOf::new(
        inset
            .inline_start
            .or_else(|| inset.inline_end.map(|end| -end))
            .unwrap_or(S::ZERO),
        inset
            .block_start
            .or_else(|| inset.block_end.map(|end| -end))
            .unwrap_or(S::ZERO),
    )
}
