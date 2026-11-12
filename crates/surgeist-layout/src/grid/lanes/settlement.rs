use super::*;
use crate::FlowAxes;

pub(super) struct LaneItemPlacement<'a, Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) style: &'a GridContainerProjection<'a, S>,
    pub(super) flow_axes: FlowAxes,
    pub(super) containing_size: Size<S>,
    pub(super) lane_report: &'a PackedLanePlacement<S>,
    pub(super) lane_axis_alignment_start: S,
    pub(super) column_offsets: &'a [S],
    pub(super) row_offsets: &'a [S],
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) track_basis: LogicalSizeOf<Option<S>>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) ancestor_baseline_groups: &'a FinalAncestorBaselineGroups<Node, S>,
}

pub(super) fn stage_lane_items<'input, Tree, M>(
    tree: &mut Tree,
    placement: LaneItemPlacement<'_, Tree::Node, Tree::Scalar>,
    pending_items: &mut [PendingGridItem<'input, Tree::Node, Tree::Scalar>],
    child_contributions: &mut Vec<GridChildContribution<Tree::Scalar>>,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let LaneItemPlacement {
        node,
        style,
        flow_axes,
        containing_size,
        lane_report,
        lane_axis_alignment_start,
        column_offsets,
        row_offsets,
        column_geometry,
        row_geometry,
        row_tracks,
        track_basis,
        subgrid_report,
        ancestor_baseline_groups,
    } = placement;
    let grid_axis = lane_report.grid_axis;
    let lane_axis = lane_report.lane_axis;
    for item in pending_items {
        let baseline_offset = baseline_aligned_axis_offset(BaselineAlignedAxisInput {
            item,
            child_style: &item.style,
            container_style: style,
            group: ancestor_baseline_groups.for_axis(grid_axis),
            axis: grid_axis,
            geometry: match grid_axis {
                GridAxisKind::Column => column_geometry,
                GridAxisKind::Row => row_geometry,
            },
            row_tracks,
            track_basis,
            subgrid_item: subgrid_report.items.get(item.source_index).copied(),
            container_flow_axes: flow_axes,
            inherited_owner_targets: ancestor_baseline_groups.inherited_targets_for_axis(grid_axis),
            current_grid: node,
        })
        .map_err(|error| {
            subgrid_child_context_container_error(
                node,
                item.node,
                SubgridChildContextError::BaselineInheritance(error),
            )
        })?;
        let inline_offset = if grid_axis == GridAxisKind::Column {
            baseline_offset.unwrap_or(item.horizontal_axis.offset)
        } else {
            item.horizontal_axis.offset
        };
        let block_offset = if grid_axis == GridAxisKind::Row {
            baseline_offset.unwrap_or(item.vertical_axis.offset)
        } else {
            item.vertical_axis.offset
        };
        let lane_offset = lane_report
            .item(item.source_index)
            .expect("pending lane item retains its packed cell")
            .offset;
        let selected_inline_lane_offset = lane_axis_alignment_start;
        let selected_block_track_offset =
            grid_area_track_offset(row_offsets, item.area.row, item.area.row_end);
        let normal_logical_location = match lane_axis.logical_axis() {
            LogicalAxis::Inline => LogicalPointOf::new(
                selected_inline_lane_offset + lane_offset + inline_offset,
                selected_block_track_offset + block_offset,
            ),
            LogicalAxis::Block => LogicalPointOf::new(
                grid_area_inline_offset(column_offsets, item.area) + inline_offset,
                lane_axis_alignment_start + lane_offset + block_offset,
            ),
        };
        let normal_location = flow_axes.physical_point(
            normal_logical_location,
            flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let logical_location = LogicalPointOf::new(
            normal_logical_location.inline + item.logical_relative_offset.inline,
            normal_logical_location.block + item.logical_relative_offset.block,
        );
        let location = flow_axes.physical_point(
            logical_location,
            flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let baseline_location = match lane_axis.logical_axis() {
            LogicalAxis::Inline => flow_axes.physical_point(
                LogicalPointOf::new(
                    lane_axis_alignment_start + lane_offset + inline_offset,
                    grid_area_track_offset(row_offsets, item.area.row, item.area.row_end)
                        + block_offset,
                ),
                flow_axes.logical_size(item.output.size),
                containing_size,
            ),
            LogicalAxis::Block => normal_location,
        };
        let area_origin = LogicalPointOf::new(
            grid_area_inline_offset(column_offsets, item.area),
            grid_area_track_offset(row_offsets, item.area.row, item.area.row_end),
        );
        item.block_offset = normal_logical_location.block - area_origin.block;
        item.location = baseline_location;
        let scroll_geometry = item
            .output
            .scroll_geometry
            .expect("pending grid-lanes item retains canonical geometry");
        let subgrid_item = subgrid_report
            .items
            .get(item.source_index)
            .copied()
            .expect("grid-lanes subgrid report preserves source identity");
        let (horizontal, vertical) =
            subgrid_parent_propagation_axes(subgrid_item, flow_axes, item.child_flow_axes);
        child_contributions.push(GridChildContribution {
            source_index: crate::SourceIndex::new(item.source_index),
            location,
            margin: item.margin,
            geometry: scroll_geometry,
            descendants: scroll_geometry
                .propagatable_descendant_intervals()
                .retain_physical_axes(horizontal, vertical),
            in_flow: true,
        });
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            item.node,
            crate::output::NormalGeometryOf::box_geometry(
                normal_location,
                item.output.size,
                scroll_geometry.content_box(),
            ),
            NodeOutputOf {
                source_index: crate::SourceIndex::new(item.source_index),
                location,
                size: item.output.size,
                named_baselines: item.output.named_baselines,
                content_size: item.output.content_size,
                scroll_geometry: Some(scroll_geometry),
                border: item.border,
                padding: item.padding,
                margin: item.margin,
            },
        ));
    }
    Ok(())
}
