use super::*;

#[derive(Clone, Copy)]
pub(in crate::grid) struct SubgridChildParentContextInput<
    'input,
    'a,
    Node,
    S: LayoutScalar = Scalar,
> {
    pub(in crate::grid) item: SubgridItemReport<Node>,
    pub(in crate::grid) parent_baseline_references: crate::GridBaselineReferences,
    pub(in crate::grid) child_style: &'a GridItemProjection<'input, S>,
    pub(in crate::grid) child_container_style: Option<GridContainerProjection<'input, S>>,
    pub(in crate::grid) area: GridArea<S>,
    pub(in crate::grid) content_box_size: Size<S>,
    pub(in crate::grid) columns: &'a [S],
    pub(in crate::grid) rows: &'a [S],
    pub(in crate::grid) minimum_functions: GridMinimumParentFunctions<'a, S>,
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) parent_named_columns: &'a NamedGridLines,
    pub(in crate::grid) parent_named_rows: &'a NamedGridLines,
    pub(in crate::grid) parent_area_facts: Option<&'a GridAreaNameFacts>,
    pub(in crate::grid) margin: Edges<Option<S>>,
    pub(in crate::grid) border: Edges<S>,
    pub(in crate::grid) padding: Edges<S>,
    pub(in crate::grid) scrollbar_gutter: Edges<S>,
}

impl<Node, S: LayoutScalar> SubgridChildParentContextInput<'_, '_, Node, S> {
    pub(in crate::grid) fn for_scrollbar_pass(mut self, constants: &Constants<S>) -> Self {
        self.scrollbar_gutter = constants.scrollbar_gutter;
        self.content_box_size =
            (self.content_box_size - self.scrollbar_gutter.sum_axes()).max(Size::ZERO);
        self
    }
}

pub(in crate::grid) fn subgrid_child_parent_context<'input, Node, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'input, '_, Node, S>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        None,
        TemplateAreaExpandedAxes::default(),
        None,
        None,
        None,
    )
}

pub(in crate::grid) fn subgrid_child_parent_context_with_geometry<'input, Node, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'input, '_, Node, S>,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        None,
        TemplateAreaExpandedAxes::default(),
        None,
        column_geometry,
        row_geometry,
    )
}

#[cfg(test)]
pub(in crate::grid) fn subgrid_child_parent_context_from_ancestor_groups<
    'input,
    Node,
    S: LayoutScalar,
>(
    input: SubgridChildParentContextInput<'input, '_, Node, S>,
    ancestor_baseline_groups: &FinalAncestorBaselineGroups<Node, S>,
    parent_grid: Node,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        Some(ancestor_baseline_groups),
        TemplateAreaExpandedAxes::default(),
        Some(parent_grid),
        None,
        None,
    )
}

pub(in crate::grid) fn subgrid_child_parent_context_from_ancestor_groups_with_geometry<
    'input,
    Node,
    S: LayoutScalar,
>(
    input: SubgridChildParentContextInput<'input, '_, Node, S>,
    ancestor_baseline_groups: &FinalAncestorBaselineGroups<Node, S>,
    parent_template_area_expanded_axes: TemplateAreaExpandedAxes,
    parent_grid: Node,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    subgrid_child_parent_context_with_ancestor_groups(
        input,
        Some(ancestor_baseline_groups),
        parent_template_area_expanded_axes,
        Some(parent_grid),
        column_geometry,
        row_geometry,
    )
}

fn subgrid_child_parent_context_with_ancestor_groups<'input, Node, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'input, '_, Node, S>,
    ancestor_baseline_groups: Option<&FinalAncestorBaselineGroups<Node, S>>,
    parent_template_area_expanded_axes: TemplateAreaExpandedAxes,
    parent_grid: Option<Node>,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<GridParentContext<S, Node>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    let axis_context = |axis| {
        let mut source =
            subgrid_child_axis_context_input(input, axis, column_geometry, row_geometry);
        source.parent_grid = parent_grid;
        source.parent_template_area_expanded_axes = parent_template_area_expanded_axes;
        source.ancestor_baseline_groups = ancestor_baseline_groups;
        subgrid_child_axis_context(source)
    };
    Ok(GridParentContext {
        columns: axis_context(GridAxisKind::Column)?,
        rows: axis_context(GridAxisKind::Row)?,
    })
}

pub(in crate::grid) fn subgrid_child_axis_parent_context_with_geometry<
    'input,
    Node,
    S: LayoutScalar,
>(
    input: SubgridChildParentContextInput<'input, '_, Node, S>,
    axis: GridAxisKind,
    gap_basis: Size<Option<S>>,
    column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
) -> Result<Option<InheritedGridAxis<S, Node>>, SubgridChildContextError<S>>
where
    Node: Copy + PartialEq,
{
    let mut source = subgrid_child_axis_context_input(input, axis, column_geometry, row_geometry);
    source.gap_basis = gap_basis;
    subgrid_child_axis_context(source)
}

fn subgrid_child_axis_context_input<'input, 'a, Node: Copy, S: LayoutScalar>(
    input: SubgridChildParentContextInput<'input, 'a, Node, S>,
    axis: GridAxisKind,
    column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
) -> SubgridChildAxisContextInput<'input, 'a, Node, S> {
    SubgridChildAxisContextInput {
        parent_grid: None,
        parent_baseline_references: input.parent_baseline_references,
        current_grid: input.item.node,
        axis,
        report: match axis {
            GridAxisKind::Column => input.item.column,
            GridAxisKind::Row => input.item.row,
        },
        child_style: input.child_style,
        child_container_style: input.child_container_style,
        area: input.area,
        gap_basis: input.content_box_size.map(Some),
        parent_columns: input.columns,
        parent_rows: input.rows,
        minimum_functions: input.minimum_functions,
        parent_column_geometry: column_geometry,
        parent_row_geometry: row_geometry,
        parent_gap: input.gap,
        parent_named_columns: input.parent_named_columns,
        parent_named_rows: input.parent_named_rows,
        parent_area_facts: input.parent_area_facts,
        parent_template_area_expanded_axes: TemplateAreaExpandedAxes::default(),
        ancestor_baseline_groups: None,
        margin: input.margin,
        border: input.border,
        padding: input.padding,
        scrollbar_gutter: input.scrollbar_gutter,
    }
}

#[derive(Clone, Copy)]
struct SubgridChildAxisContextInput<'input, 'a, Node, S: LayoutScalar = Scalar> {
    parent_grid: Option<Node>,
    parent_baseline_references: crate::GridBaselineReferences,
    current_grid: Node,
    axis: GridAxisKind,
    report: SubgridAxisReport,
    child_style: &'a GridItemProjection<'input, S>,
    child_container_style: Option<GridContainerProjection<'input, S>>,
    area: GridArea<S>,
    gap_basis: Size<Option<S>>,
    parent_columns: &'a [S],
    parent_rows: &'a [S],
    minimum_functions: GridMinimumParentFunctions<'a, S>,
    parent_column_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    parent_row_geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    parent_gap: LogicalSizeOf<S>,
    parent_named_columns: &'a NamedGridLines,
    parent_named_rows: &'a NamedGridLines,
    parent_area_facts: Option<&'a GridAreaNameFacts>,
    parent_template_area_expanded_axes: TemplateAreaExpandedAxes,
    ancestor_baseline_groups: Option<&'a FinalAncestorBaselineGroups<Node, S>>,
    margin: Edges<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
    scrollbar_gutter: Edges<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) enum SubgridChildContextError<S: LayoutScalar> {
    MinimumFunctions(GridMinimumFunctionRetentionError<S>),
    ValueResolution(LengthResolutionStatus<S>),
    TrackInheritance(SubgridTrackInheritanceError),
    BaselineInheritance(InheritedCurrentGridBaselinePlacementError),
}

fn subgrid_child_axis_context<'input, Node: Copy + PartialEq, S: LayoutScalar>(
    input: SubgridChildAxisContextInput<'input, '_, Node, S>,
) -> Result<Option<InheritedGridAxis<S, Node>>, SubgridChildContextError<S>> {
    if !input.report.can_inherit() {
        return Ok(None);
    }
    let mapping = input.report.mapping;
    let (start_line, end_line) = match mapping.parent_axis {
        GridAxisKind::Column => (input.area.column + 1, input.area.column_end + 1),
        GridAxisKind::Row => (input.area.row + 1, input.area.row_end + 1),
    };
    let parent_axis = subgrid_parent_axis_data(&input, mapping.parent_axis);
    let child_flow_axes =
        FlowAxes::new(input.child_style.writing_mode, input.child_style.direction);
    let (start_mbp, end_mbp) = axis_margin_border_padding(
        input.axis,
        child_flow_axes,
        input.margin,
        input.border,
        input.padding,
        input.scrollbar_gutter,
    );
    let inherited = inherit_subgrid_tracks_with_geometry(
        SubgridTrackInheritanceInput {
            parent_tracks: parent_axis.tracks,
            parent_span: GridTrackSpan::new(start_line, end_line),
            reversed: mapping.reversed,
            start_mbp,
            end_mbp,
            parent_gap: parent_axis.gap,
            subgrid_gap: child_subgrid_gap(
                &input
                    .child_container_style
                    .expect("an inheriting subgrid axis must retain its container projection"),
                input.axis,
                input.gap_basis,
            )
            .map_err(SubgridChildContextError::ValueResolution)?,
        },
        parent_axis.geometry,
    )
    .map_err(SubgridChildContextError::TrackInheritance)?;
    let uniform_parent_boundary_gutters;
    let parent_boundary_gutters = if let Some(geometry) = parent_axis.geometry {
        geometry.gutter_after()
    } else {
        uniform_parent_boundary_gutters =
            vec![parent_axis.gap; parent_axis.tracks.len().saturating_sub(1)];
        &uniform_parent_boundary_gutters
    };
    let physical_axis = grid_axis_physical_axis(child_flow_axes, input.axis);
    let has_collapsed_track = inherited.collapsed.iter().any(|collapsed| *collapsed);
    let (layout_tracks, layout_gap) = if has_collapsed_track {
        (
            inherited.final_tracks.clone(),
            inherited.resolved_subgrid_gap,
        )
    } else {
        inherited_subgrid_layout_tracks(input.axis, &inherited)
    };
    let layout_boundary_gutters = if !has_collapsed_track
        && uses_shifted_column_subgrid_layout_tracks(input.axis, &inherited)
    {
        vec![S::ZERO; layout_tracks.len().saturating_sub(1)]
    } else {
        inherited.final_boundary_gutters.clone()
    };
    let geometry = UsedGridAxisGeometryOf::from_active_boundary_gutters(
        layout_tracks.clone(),
        inherited.collapsed.clone(),
        inherited.final_active_boundary_after.clone(),
        layout_boundary_gutters,
    );

    let mut owner_baseline_targets = None;
    if let Some(ancestor_groups) = input.ancestor_baseline_groups {
        let current_group = ancestor_groups.for_axis(mapping.parent_axis);
        if current_group.reference()
            != grid_baseline_reference(input.parent_baseline_references, mapping.parent_axis)
        {
            return Err(SubgridChildContextError::BaselineInheritance(
                InheritedCurrentGridBaselinePlacementError::ReferenceMismatch,
            ));
        }
        if current_group.axis() != mapping.parent_axis {
            return Err(SubgridChildContextError::BaselineInheritance(
                InheritedCurrentGridBaselinePlacementError::AxisMismatch,
            ));
        }
        if let Some(transported) = ancestor_groups.inherited_targets_for_axis(mapping.parent_axis) {
            if transported.group.reference() != current_group.reference() {
                return Err(SubgridChildContextError::BaselineInheritance(
                    InheritedCurrentGridBaselinePlacementError::ReferenceMismatch,
                ));
            }
            if transported.mapping.physical_axis() != physical_axis {
                return Err(SubgridChildContextError::BaselineInheritance(
                    InheritedCurrentGridBaselinePlacementError::PhysicalAxisMismatch,
                ));
            }
            let parent_grid =
                input
                    .parent_grid
                    .ok_or(SubgridChildContextError::BaselineInheritance(
                        InheritedCurrentGridBaselinePlacementError::OwnershipMismatch,
                    ))?;
            let parent_span = GridTrackSpan::new(start_line - 1, end_line - 1);
            let (
                parent_first_frame_origins,
                parent_last_frame_origins,
                current_first_frame_origins,
                current_last_frame_origins,
            ) = owner_progression_track_frame_origins(OwnerProgressionTrackFrameInput {
                parent_tracks: parent_axis.tracks,
                parent_boundary_gutters,
                parent_line_offsets: parent_axis
                    .geometry
                    .map(UsedGridAxisGeometryOf::line_offsets),
                parent_span,
                current_tracks: geometry.sizes(),
                current_boundary_gutters: geometry.gutter_after(),
                reversed: mapping.reversed,
                start_mbp,
            })
            .ok_or(SubgridChildContextError::BaselineInheritance(
                InheritedCurrentGridBaselinePlacementError::SpanOutOfRange,
            ))?;
            let composed_mapping = transported
                .mapping
                .compose(OwnerToCurrentPlacementBoundaryInput {
                    parent_grid,
                    current_grid: input.current_grid,
                    parent_axis: mapping.parent_axis,
                    current_axis: input.axis,
                    physical_axis,
                    parent_progression: transported.mapping.current_progression(),
                    current_progression: child_flow_axes.physical_axis_progression(physical_axis),
                    parent_span,
                    reversed: mapping.reversed,
                    parent_first_frame_origins: &parent_first_frame_origins,
                    parent_last_frame_origins: &parent_last_frame_origins,
                    current_first_frame_origins: &current_first_frame_origins,
                    current_last_frame_origins: &current_last_frame_origins,
                    inherited: true,
                })
                .map_err(|error| SubgridChildContextError::BaselineInheritance(error))?;
            if transported.group.has_any_target() {
                owner_baseline_targets = Some(InheritedGridOwnerBaselineTargets {
                    group: transported.group.clone(),
                    mapping: composed_mapping,
                });
            }
        }
    }

    let (named_lines, area_facts) = input
        .report
        .parent_name_context(parent_axis.named_lines, input.parent_area_facts);
    Ok(Some(InheritedGridAxis {
        offset: S::ZERO,
        gap: layout_gap,
        tracks: layout_tracks,
        minimum_functions: input
            .minimum_functions
            .for_axis(mapping.parent_axis)
            .map(|functions| {
                functions
                    .mapped_span(start_line - 1, end_line - 1, mapping.reversed)
                    .ok_or(SubgridChildContextError::TrackInheritance(
                        SubgridTrackInheritanceError::SpanOutOfRange,
                    ))?
                    .retain()
                    .map_err(SubgridChildContextError::MinimumFunctions)
            })
            .transpose()?,
        geometry,
        named_lines,
        area_facts,
        template_area_expanded: input
            .parent_template_area_expanded_axes
            .for_axis(mapping.parent_axis),
        owner_baseline_targets,
        baseline_reference: grid_baseline_reference(
            input.parent_baseline_references,
            mapping.parent_axis,
        ),
        parent_start: start_line - 1,
        parent_end: end_line - 1,
        reversed: mapping.reversed,
    }))
}

pub(in crate::grid) fn subgrid_child_context_error<Node, S, M>(
    subject: Node,
    error: SubgridChildContextError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    match error {
        SubgridChildContextError::MinimumFunctions(
            GridMinimumFunctionRetentionError::Allocation,
        ) => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(subject),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::Allocation,
        ),
        SubgridChildContextError::MinimumFunctions(GridMinimumFunctionRetentionError::Sizing(
            error,
        )) => sizing_resolution_error(subject, error),
        SubgridChildContextError::ValueResolution(status) => {
            crate::error::value_resolution_error(subject, status)
        }
        SubgridChildContextError::TrackInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(subject),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridTrackInheritance),
        ),
        SubgridChildContextError::BaselineInheritance(
            InheritedCurrentGridBaselinePlacementError::Selection(error),
        ) => grid_baseline_error_at_site(LayoutErrorSiteOf::Node(subject), error),
        SubgridChildContextError::BaselineInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(subject),
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::SubgridBaselineInheritance,
            ),
        ),
    }
}

pub(in crate::grid) fn subgrid_child_context_container_error<Node, S, M>(
    container: Node,
    subject: Node,
    error: SubgridChildContextError<S>,
) -> LayoutErrorOf<Node, S, M>
where
    Node: Copy,
    S: LayoutScalar,
{
    match error {
        SubgridChildContextError::MinimumFunctions(
            GridMinimumFunctionRetentionError::Allocation,
        ) => LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject { container, subject },
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::Allocation,
        ),
        SubgridChildContextError::MinimumFunctions(GridMinimumFunctionRetentionError::Sizing(
            error,
        )) => crate::error::sizing_resolution_error_at_site(
            LayoutErrorSiteOf::ContainerSubject { container, subject },
            error,
        ),
        SubgridChildContextError::ValueResolution(status) => {
            crate::error::value_resolution_error(subject, status)
        }
        SubgridChildContextError::TrackInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject { container, subject },
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(LayoutInternalInvariant::SubgridTrackInheritance),
        ),
        SubgridChildContextError::BaselineInheritance(
            InheritedCurrentGridBaselinePlacementError::Selection(error),
        ) => grid_baseline_error(container, subject, error),
        SubgridChildContextError::BaselineInheritance(_) => LayoutErrorOf::new(
            LayoutErrorSiteOf::ContainerSubject { container, subject },
            LayoutOperation::ChildLayout,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::SubgridBaselineInheritance,
            ),
        ),
    }
}

struct SubgridParentAxisData<'a, S: LayoutScalar = Scalar> {
    tracks: &'a [S],
    geometry: Option<&'a UsedGridAxisGeometryOf<S>>,
    gap: S,
    named_lines: &'a NamedGridLines,
}

fn subgrid_parent_axis_data<'input, 'a, Node, S: LayoutScalar>(
    input: &'a SubgridChildAxisContextInput<'input, 'a, Node, S>,
    axis: GridAxisKind,
) -> SubgridParentAxisData<'a, S> {
    match axis {
        GridAxisKind::Column => SubgridParentAxisData {
            tracks: input.parent_columns,
            geometry: input.parent_column_geometry,
            gap: input.parent_gap.inline,
            named_lines: input.parent_named_columns,
        },
        GridAxisKind::Row => SubgridParentAxisData {
            tracks: input.parent_rows,
            geometry: input.parent_row_geometry,
            gap: input.parent_gap.block,
            named_lines: input.parent_named_rows,
        },
    }
}

pub(in crate::grid) fn inherited_subgrid_layout_tracks<S: LayoutScalar>(
    axis: GridAxisKind,
    inherited: &SubgridTrackInheritanceReport<S>,
) -> (Vec<S>, S) {
    if uses_shifted_column_subgrid_layout_tracks(axis, inherited) {
        let mut lines = Vec::with_capacity(inherited.end_mbp_removed.len() + 1);
        let mut cursor = S::ZERO;
        lines.push(cursor);
        for (index, track) in inherited.end_mbp_removed.iter().copied().enumerate() {
            cursor = cursor + track;
            if let (Some(parent_gutter), Some(final_gutter)) = (
                inherited.parent_boundary_gutters.get(index).copied(),
                inherited.final_boundary_gutters.get(index).copied(),
            ) {
                cursor = cursor + parent_gutter;
                lines.push(cursor + (final_gutter - parent_gutter) / S::from_f64(2.0));
            }
        }
        lines.push(cursor);

        return (
            lines
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).max(S::ZERO))
                .collect(),
            S::ZERO,
        );
    }

    (
        inherited.final_tracks.clone(),
        inherited.resolved_subgrid_gap,
    )
}

fn uses_shifted_column_subgrid_layout_tracks<S: LayoutScalar>(
    axis: GridAxisKind,
    inherited: &SubgridTrackInheritanceReport<S>,
) -> bool {
    axis == GridAxisKind::Column
        && inherited.gap_difference > S::ZERO
        && inherited.final_tracks.len() >= 2
        && inherited.final_tracks.contains(&S::ZERO)
}

fn grid_axis_physical_axis(flow_axes: FlowAxes, axis: GridAxisKind) -> PhysicalAxis {
    match axis.logical_axis() {
        crate::LogicalAxis::Inline => flow_axes.inline_axis(),
        crate::LogicalAxis::Block => flow_axes.block_axis(),
    }
}

fn axis_margin_border_padding<S: LayoutScalar>(
    axis: GridAxisKind,
    flow_axes: FlowAxes,
    margin: Edges<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
    scrollbar_gutter: Edges<S>,
) -> (S, S) {
    let margin = flow_axes.logical_edges(margin);
    let border = flow_axes.logical_edges(border);
    let padding = flow_axes.logical_edges(padding + scrollbar_gutter);
    match axis.logical_axis() {
        LogicalAxis::Inline => (
            margin.inline_start.unwrap_or(S::ZERO) + border.inline_start + padding.inline_start,
            margin.inline_end.unwrap_or(S::ZERO) + border.inline_end + padding.inline_end,
        ),
        LogicalAxis::Block => (
            margin.block_start.unwrap_or(S::ZERO) + border.block_start + padding.block_start,
            margin.block_end.unwrap_or(S::ZERO) + border.block_end + padding.block_end,
        ),
    }
}

pub(in crate::grid) fn child_subgrid_gap<S: LayoutScalar>(
    style: &GridContainerProjection<'_, S>,
    axis: GridAxisKind,
    content_basis: Size<Option<S>>,
) -> Result<ResolvedSubgridGap<S>, LengthResolutionStatus<S>> {
    let flow_axes = FlowAxes::new(style.writing_mode, style.direction);
    let logical_gap = flow_axes.logical_size(*style.gap);
    let logical_basis = flow_axes.logical_size(content_basis);
    let (gap, basis) = match axis.logical_axis() {
        LogicalAxis::Inline => (logical_gap.inline, logical_basis.inline),
        LogicalAxis::Block => (logical_gap.block, logical_basis.block),
    };
    match gap {
        LengthOf::Normal => Ok(ResolvedSubgridGap::Normal),
        gap => Ok(ResolvedSubgridGap::Length(resolve_grid_gap(
            gap,
            // Intrinsic cyclic percentages resolve against zero, retaining
            // every fixed term in a mixed length-percentage calculation.
            Some(basis.unwrap_or(S::ZERO)),
        )?)),
    }
}

pub(super) fn apply_final_subgrid_axis_constraints<Node, S: LayoutScalar>(
    sizing: &mut GridItemSizing<S>,
    item: SubgridItemReport<Node>,
    parent_flow_axes: FlowAxes,
    child_flow_axes: FlowAxes,
) {
    apply_final_subgrid_axis_constraint(sizing, item.column, parent_flow_axes, child_flow_axes);
    apply_final_subgrid_axis_constraint(sizing, item.row, parent_flow_axes, child_flow_axes);
}

fn apply_final_subgrid_axis_constraint<S: LayoutScalar>(
    sizing: &mut GridItemSizing<S>,
    report: SubgridAxisReport,
    parent_flow_axes: FlowAxes,
    child_flow_axes: FlowAxes,
) {
    let Some(physical_axis) =
        inherited_subgrid_physical_axis(report, parent_flow_axes, child_flow_axes)
    else {
        return;
    };
    let extent = match physical_axis {
        PhysicalAxis::Horizontal => sizing.available.width,
        PhysicalAxis::Vertical => sizing.available.height,
    };
    match physical_axis {
        PhysicalAxis::Horizontal => {
            sizing.known.width = Some(extent);
        }
        PhysicalAxis::Vertical => {
            sizing.known.height = Some(extent);
        }
    }
}

pub(super) struct SubgridBaselineRefreshInput<'a, Node, S: LayoutScalar = Scalar> {
    pub(super) node: Node,
    pub(super) container_style: &'a GridContainerProjection<'a, S>,
    pub(super) columns: &'a [S],
    pub(super) rows: &'a [S],
    pub(super) minimum_functions: GridMinimumParentFunctions<'a, S>,
    pub(super) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(super) row_tracks: &'a [TrackSizingOf<S>],
    pub(super) track_basis: LogicalSizeOf<Option<S>>,
    pub(super) gap: LogicalSizeOf<S>,
    pub(super) named_columns: NamedGridLines,
    pub(super) named_rows: NamedGridLines,
    pub(super) area_facts: Option<GridAreaNameFacts>,
    pub(super) template_area_expanded_axes: TemplateAreaExpandedAxes,
    pub(super) subgrid_report: &'a GridSubgridReport<Node>,
    pub(super) ancestor_baseline_groups: &'a FinalAncestorBaselineGroups<Node, S>,
    pub(super) executed_frame: ExecutedBaselineFrame<'a, S>,
    pub(super) orthogonal_fallback: OrthogonalFallbackOf<S>,
    pub(super) original_executed: &'a ExecutedBaselineInventory<Node, S>,
    pub(super) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    pub(super) sizing_stage: GridSizingStage,
}

pub(super) fn refresh_subgrid_items_with_baselines<'input, Tree, M>(
    tree: &mut Tree,
    input: SubgridBaselineRefreshInput<'_, <Tree as Traverse>::Node, Tree::Scalar>,
    pending_items: &mut [PendingGridItem<'input, <Tree as Traverse>::Node, Tree::Scalar>],
) -> LayoutResultOf<<Tree as Traverse>::Node, (), Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let container_flow_axes = FlowAxes::new(
        input.container_style.writing_mode,
        input.container_style.direction,
    );
    for item in pending_items.iter_mut() {
        let Some(subgrid_item) = input.subgrid_report.items.get(item.source_index).copied() else {
            continue;
        };
        if !subgrid_item.column.can_inherit() && !subgrid_item.row.can_inherit() {
            continue;
        }

        let child_style = item.style.clone();
        let physical_area_size = grid_area_physical_size(
            FlowAxes::new(
                input.container_style.writing_mode,
                input.container_style.direction,
            ),
            item.area.size,
        );
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let FinalGridItemPreparation {
            sizing,
            input: child_input,
            padding,
            border,
            content_box_size: subgrid_content_box_size,
        } = prepare_final_grid_item::<Tree, M>(
            tree,
            item.node,
            &child_style,
            input.container_style,
            subgrid_item,
            physical_area_size,
            GridItemRequestContext {
                containing_auto_scrollbar_pass: input.containing_auto_scrollbar_pass,
                orthogonal_fallback: input.orthogonal_fallback,
            },
        )?;
        let child_input =
            grid_item_baseline_demand(input.container_style, &child_style, None, child_input)
                .map_err(|error| grid_baseline_error(input.node, item.node, error))?;
        let child_container_style = item
            .nested_container
            .as_ref()
            .map(GridContainerInput::projection);
        let child_context_input = SubgridChildParentContextInput {
            item: subgrid_item,
            parent_baseline_references: input.container_style.baseline_references,
            child_style: &child_style,
            child_container_style,
            area: item.area,
            content_box_size: subgrid_content_box_size,
            columns: input.columns,
            rows: input.rows,
            minimum_functions: input.minimum_functions,
            gap: input.gap,
            parent_named_columns: &input.named_columns,
            parent_named_rows: &input.named_rows,
            parent_area_facts: input.area_facts.as_ref(),
            margin: sizing.unresolved_margin,
            border,
            padding,
            scrollbar_gutter: Edges::ZERO,
        };
        if !subgrid_item.column.can_inherit() && !subgrid_item.row.can_inherit() {
            continue;
        }
        let result = compute_grid_with_context_source(
            tree,
            item.node,
            child_input,
            sizing.standalone_intrinsic_minimum,
            LogicalSizeOf::new(
                child_context_input.item.column.can_inherit(),
                child_context_input.item.row.can_inherit(),
            ),
            input
                .sizing_stage
                .mapped(container_flow_axes, child_flow_axes),
            |constants| {
                subgrid_child_parent_context_from_ancestor_groups_with_geometry(
                    child_context_input.for_scrollbar_pass(constants),
                    input.ancestor_baseline_groups,
                    input.template_area_expanded_axes,
                    input.node,
                    Some(input.column_geometry),
                    Some(input.row_geometry),
                )
                .map_err(|error| {
                    subgrid_child_context_container_error(input.node, item.node, error)
                })
            },
        )?;
        let GridComputeResult {
            mut output,
            baseline_groups: _ordinary_baseline_groups,
            executed_baselines,
            ..
        } = result;
        let executed_source = require_executed_baselines(
            input.node,
            item.node,
            child_flow_axes,
            output,
            child_input,
            input
                .sizing_stage
                .mapped(container_flow_axes, child_flow_axes),
            executed_baselines,
        )?;
        let scroll_geometry = retained_grid_child_scroll_geometry(
            &child_style,
            output.size,
            output.content_size,
            padding,
            border,
            output.scroll_geometry,
        )
        .map_err(|error| layout_child_geometry_error(input.node, item.node, error))?;
        output.scroll_geometry = Some(scroll_geometry);
        let alignment = grid_item_physical_alignment(
            input.container_style.writing_mode,
            sizing.justify_self,
            sizing.align_self,
        );
        let physical_horizontal_axis = physical_grid_item_axis(PhysicalGridItemAxis {
            area_size: physical_area_size.width,
            size: output.size.width,
            margin_start: sizing.unresolved_margin.left,
            margin_end: sizing.unresolved_margin.right,
            alignment: alignment.horizontal,
            progression: grid_physical_axis_progression(
                input.container_style.writing_mode,
                input.container_style.direction,
                PhysicalAxis::Horizontal,
            ),
        });
        let physical_vertical_axis = physical_grid_item_axis(PhysicalGridItemAxis {
            area_size: physical_area_size.height,
            size: output.size.height,
            margin_start: sizing.unresolved_margin.top,
            margin_end: sizing.unresolved_margin.bottom,
            alignment: alignment.vertical,
            progression: grid_physical_axis_progression(
                input.container_style.writing_mode,
                input.container_style.direction,
                PhysicalAxis::Vertical,
            ),
        });
        let margin = Edges {
            left: physical_horizontal_axis.margin_start,
            right: physical_horizontal_axis.margin_end,
            top: physical_vertical_axis.margin_start,
            bottom: physical_vertical_axis.margin_end,
        };
        let logical_offset = container_flow_axes.logical_point(
            Point::new(
                physical_horizontal_axis.offset,
                physical_vertical_axis.offset,
            ),
            output.size,
            physical_area_size,
        );
        let logical_margin = container_flow_axes.logical_edges(margin);
        let horizontal_axis = ResolvedGridItemAxis {
            offset: logical_offset.inline,
            margin_start: logical_margin.inline_start,
            margin_end: logical_margin.inline_end,
        };
        let vertical_axis = ResolvedGridItemAxis {
            offset: logical_offset.block,
            margin_start: logical_margin.block_start,
            margin_end: logical_margin.block_end,
        };
        let baselines = output.baselines();
        let child_flow_axes = FlowAxes::new(child_style.writing_mode, child_style.direction);
        let first_baseline =
            baselines.first_or_synthesize_block_baseline(child_flow_axes, output.size);
        let last_baseline =
            baselines.last_or_synthesize_block_baseline(child_flow_axes, output.size);
        let block_auto_margins = child_flow_axes
            .line_over_edge(sizing.unresolved_margin)
            .is_none()
            || child_flow_axes
                .line_under_edge(sizing.unresolved_margin)
                .is_none();
        let row_span_tracks = input
            .row_tracks
            .get(item.area.row..item.area.row_end)
            .unwrap_or(&[]);
        let baseline_participation = grid_baseline_participation(
            input.container_style.baseline_references.rows,
            sizing.align_self,
            block_auto_margins,
            grid_baseline_span_would_cycle(row_span_tracks, input.track_basis.block),
            output,
            child_flow_axes,
            container_flow_axes.block_axis(),
        )
        .map_err(|error| grid_baseline_error(input.node, item.node, error))?;
        item.output = output;
        item.horizontal_axis = horizontal_axis;
        item.vertical_axis = vertical_axis;
        item.child_flow_axes = child_flow_axes;
        item.first_baseline = first_baseline;
        item.last_baseline = last_baseline;
        item.block_auto_margins = block_auto_margins;
        item.baseline_participation = baseline_participation;
        item.margin = margin;
        item.border = border;
        item.padding = padding;
        verify_refreshed_baselines(
            input.node,
            input.executed_frame,
            item,
            subgrid_item,
            &executed_source,
            input.original_executed,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod gap_basis_tests {
    use super::*;
    use crate::{LengthOf, LengthPercentageOf, NodeInputOf, WritingMode};

    fn assert_gap_bases<S: LayoutScalar>() {
        let calc = LengthOf::value(
            LengthPercentageOf::from_coefficients(S::from_f64(10.0), S::from_f64(0.2))
                .expect("finite mixed gap coefficients"),
        );
        for (writing_mode, axis, normal_axis) in [
            (
                WritingMode::HorizontalTb,
                GridAxisKind::Column,
                GridAxisKind::Row,
            ),
            (
                WritingMode::VerticalRl,
                GridAxisKind::Row,
                GridAxisKind::Column,
            ),
        ] {
            for (gap, basis, expected) in [
                (calc, None, 10.0),
                (calc, Some(S::ZERO), 10.0),
                (calc, Some(S::from_f64(50.0)), 20.0),
                (LengthOf::percent(S::from_f64(0.2)), None, 0.0),
                (LengthOf::px(S::from_f64(7.0)), None, 7.0),
            ] {
                let style = NodeInputOf {
                    display: Display::Grid,
                    writing_mode,
                    gap: Size::new(gap, LengthOf::NORMAL),
                    ..NodeInputOf::default()
                };
                let projection = GridContainerProjection::from_node(&style);
                let content_basis = Size::new(basis, Some(S::from_f64(20.0)));
                assert_eq!(
                    child_subgrid_gap(&projection, axis, content_basis),
                    Ok(ResolvedSubgridGap::Length(S::from_f64(expected)))
                );
                assert_eq!(
                    child_subgrid_gap(&projection, normal_axis, content_basis),
                    Ok(ResolvedSubgridGap::Normal)
                );
            }
        }
    }

    #[test]
    fn cyclic_gap_basis_preserves_fixed_terms_and_real_content_both_scalars() {
        assert_gap_bases::<f32>();
        assert_gap_bases::<f64>();
    }

    #[test]
    fn definite_gap_overflow_retains_its_numeric_error() {
        let style = NodeInputOf {
            display: Display::Grid,
            gap: Size::new(
                LengthOf::value(
                    LengthPercentageOf::from_coefficients(f64::MAX, 1.0)
                        .expect("finite authored coefficients"),
                ),
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };
        assert!(matches!(
            child_subgrid_gap(
                &GridContainerProjection::from_node(&style),
                GridAxisKind::Column,
                Size::new(Some(f64::MAX), None)
            ),
            Err(LengthResolutionStatus::InvalidNumeric { .. })
        ));
    }
}
