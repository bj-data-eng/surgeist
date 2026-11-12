use super::*;

pub(super) struct OrdinaryGridItemPlacement<'a, Node, S: LayoutScalar> {
    pub(super) node: Node,
    pub(super) style: &'a GridContainerProjection<'a, S>,
    pub(super) flow_axes: FlowAxes,
    pub(super) containing_size: Size<S>,
    pub(super) logical_column_offsets: &'a [S],
    pub(super) logical_row_offsets: &'a [S],
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) track_basis: LogicalSizeOf<Option<S>>,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) ancestor_baseline_groups: &'a FinalAncestorBaselineGroups<Node, S>,
}

pub(super) fn stage_ordinary_grid_items<'input, Tree, M>(
    tree: &mut Tree,
    placement: OrdinaryGridItemPlacement<'_, Tree::Node, Tree::Scalar>,
    pending_items: &mut [PendingGridItem<'input, Tree::Node, Tree::Scalar>],
    child_contributions: &mut Vec<GridChildContribution<Tree::Scalar>>,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let OrdinaryGridItemPlacement {
        node,
        style,
        flow_axes,
        containing_size,
        logical_column_offsets,
        logical_row_offsets,
        column_geometry,
        row_geometry,
        row_tracks,
        track_basis,
        subgrid_report,
        ancestor_baseline_groups,
    } = placement;
    let mut prepared_item_placements = Vec::with_capacity(pending_items.len());
    for item in pending_items.iter() {
        let area_origin =
            grid_area_logical_origin(logical_column_offsets, logical_row_offsets, item.area);
        let child_style = &item.style;
        let subgrid_item = subgrid_report.items.get(item.source_index).copied();
        let inherited_columns =
            ancestor_baseline_groups.inherited_targets_for_axis(GridAxisKind::Column);
        let column_group = ancestor_baseline_groups.for_axis(GridAxisKind::Column);
        let inline_axis_offset = baseline_aligned_axis_offset(BaselineAlignedAxisInput {
            item,
            child_style,
            container_style: style,
            group: column_group,
            axis: GridAxisKind::Column,
            geometry: column_geometry,
            row_tracks,
            track_basis,
            subgrid_item,
            container_flow_axes: flow_axes,
            inherited_owner_targets: inherited_columns,
            current_grid: node,
        })
        .map_err(|error| {
            subgrid_child_context_container_error(
                node,
                item.node,
                SubgridChildContextError::BaselineInheritance(error),
            )
        })?
        .unwrap_or(item.horizontal_axis.offset);
        let inherited_rows = ancestor_baseline_groups.inherited_targets_for_axis(GridAxisKind::Row);
        let row_baseline_group = ancestor_baseline_groups.for_axis(GridAxisKind::Row);
        let block_axis_offset = baseline_aligned_axis_offset(BaselineAlignedAxisInput {
            item,
            child_style,
            container_style: style,
            group: row_baseline_group,
            axis: GridAxisKind::Row,
            geometry: row_geometry,
            row_tracks,
            track_basis,
            subgrid_item,
            container_flow_axes: flow_axes,
            inherited_owner_targets: inherited_rows,
            current_grid: node,
        })
        .map_err(|error| {
            subgrid_child_context_container_error(
                node,
                item.node,
                SubgridChildContextError::BaselineInheritance(error),
            )
        })?
        .unwrap_or(item.vertical_axis.offset);
        let normal_location = flow_axes.physical_point(
            LogicalPointOf::new(
                area_origin.inline + inline_axis_offset,
                area_origin.block + block_axis_offset,
            ),
            flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let logical_location = LogicalPointOf::new(
            area_origin.inline + inline_axis_offset + item.logical_relative_offset.inline,
            area_origin.block + block_axis_offset + item.logical_relative_offset.block,
        );
        let location = flow_axes.physical_point(
            logical_location,
            flow_axes.logical_size(item.output.size),
            containing_size,
        );
        let scroll_geometry = item
            .output
            .scroll_geometry
            .expect("pending grid item retains canonical geometry");
        debug_assert_eq!(scroll_geometry.used_overflow_x(), item.overflow.x().value());
        debug_assert_eq!(scroll_geometry.used_overflow_y(), item.overflow.y().value());
        let (horizontal, vertical) = subgrid_parent_propagation_axes(
            subgrid_report.items[item.source_index],
            flow_axes,
            item.child_flow_axes,
        );
        let contribution = GridChildContribution {
            source_index: crate::SourceIndex::new(item.source_index),
            location,
            margin: item.margin,
            geometry: scroll_geometry,
            descendants: scroll_geometry
                .propagatable_descendant_intervals()
                .retain_physical_axes(horizontal, vertical),
            in_flow: true,
        };

        prepared_item_placements.push((block_axis_offset, normal_location, location, contribution));
    }

    for (item, (block_axis_offset, normal_location, location, contribution)) in
        pending_items.iter_mut().zip(prepared_item_placements)
    {
        item.block_offset = block_axis_offset;
        item.location = normal_location;
        child_contributions.push(contribution);
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            item.node,
            crate::output::NormalGeometryOf::box_geometry(
                normal_location,
                item.output.size,
                contribution.geometry.content_box(),
            ),
            NodeOutputOf {
                source_index: crate::SourceIndex::new(item.source_index),
                location,
                size: item.output.size,
                named_baselines: item.output.named_baselines,
                content_size: item.output.content_size,
                scroll_geometry: Some(contribution.geometry),
                border: item.border,
                padding: item.padding,
                margin: item.margin,
            },
        ));
    }
    Ok(())
}
