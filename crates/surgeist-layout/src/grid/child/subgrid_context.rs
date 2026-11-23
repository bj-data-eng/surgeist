use super::*;

#[derive(Clone, Copy)]
pub(in crate::grid) struct SubgridChildParentContextInput<
    'input,
    'a,
    Node,
    S: LayoutScalar = DefaultScalar,
> {
    pub(in crate::grid) item: SubgridItemReport<Node>,
    pub(in crate::grid) parent_baseline_references: crate::GridBaselineReferences,
    pub(in crate::grid) child_style: &'a GridItemProjection<'input, S>,
    pub(in crate::grid) child_container: Option<GridContainerInput<'input, S>>,
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

impl<'input, Node: Copy, S: LayoutScalar> SubgridChildParentContextInput<'input, '_, Node, S> {
    pub(in crate::grid) fn container(self) -> GridContainerInput<'input, S> {
        self.child_container
            .expect("contextual grid retains its container input")
            .with_inherited_axes(LogicalSizeOf::new(
                self.item.column.can_inherit(),
                self.item.row.can_inherit(),
            ))
    }

    pub(in crate::grid) fn for_scrollbar_pass(mut self, constants: &Constants<S>) -> Self {
        self.scrollbar_gutter = constants.scrollbar_gutter;
        self.content_box_size =
            (self.content_box_size - self.scrollbar_gutter.sum_axes()).max(Size::ZERO);
        self
    }
}

/// An owned copy of one subgrid item's whole parent loan, retaining every
/// parent track's minimum functions. The borrowed input is rebuilt from it for
/// each scrollbar pass of the child, so the loan survives the suspension of
/// the parent that granted it. Contextual requests share their parent's data
/// instead (`SubgridItemLoan`).
#[derive(Clone, Debug)]
pub(in crate::grid) struct OwnedSubgridContext<'input, Node, S: LayoutScalar> {
    item: SubgridItemReport<Node>,
    parent_baseline_references: crate::GridBaselineReferences,
    child_style: GridItemProjection<'input, S>,
    container: GridContainerInput<'input, S>,
    area: GridArea<S>,
    content_box_size: Size<S>,
    columns: Vec<S>,
    rows: Vec<S>,
    column_minimum_functions: Option<Vec<GridMinimumTrackFact<S>>>,
    row_minimum_functions: Option<Vec<GridMinimumTrackFact<S>>>,
    gap: LogicalSizeOf<S>,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    margin: Edges<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
}

impl<'input, Node: Copy, S: LayoutScalar> OwnedSubgridContext<'input, Node, S> {
    /// Retains every parent track's minimum functions.
    pub(in crate::grid) fn new(
        source: SubgridChildParentContextInput<'input, '_, Node, S>,
    ) -> Result<Self, SubgridChildContextError<S>> {
        let column_minimum_functions = source
            .minimum_functions
            .columns
            .map(GridMinimumTrackFunctions::retain)
            .transpose()
            .map_err(SubgridChildContextError::MinimumFunctions)?;
        let row_minimum_functions = source
            .minimum_functions
            .rows
            .map(GridMinimumTrackFunctions::retain)
            .transpose()
            .map_err(SubgridChildContextError::MinimumFunctions)?;
        Ok(Self {
            item: source.item,
            parent_baseline_references: source.parent_baseline_references,
            child_style: source.child_style.clone(),
            container: source
                .child_container
                .expect("a retained subgrid context has its original container"),
            area: source.area,
            content_box_size: source.content_box_size,
            columns: source.columns.to_vec(),
            rows: source.rows.to_vec(),
            column_minimum_functions,
            row_minimum_functions,
            gap: source.gap,
            named_columns: source.parent_named_columns.clone(),
            named_rows: source.parent_named_rows.clone(),
            area_facts: source.parent_area_facts.cloned(),
            margin: source.margin,
            border: source.border,
            padding: source.padding,
        })
    }

    pub(in crate::grid) fn container_input(&self) -> GridContainerInput<'input, S> {
        self.container
    }

    pub(in crate::grid) fn input(&self) -> SubgridChildParentContextInput<'input, '_, Node, S> {
        SubgridChildParentContextInput {
            item: self.item,
            parent_baseline_references: self.parent_baseline_references,
            child_style: &self.child_style,
            child_container: Some(self.container),
            area: self.area,
            content_box_size: self.content_box_size,
            columns: &self.columns,
            rows: &self.rows,
            minimum_functions: GridMinimumParentFunctions {
                columns: self
                    .column_minimum_functions
                    .as_deref()
                    .map(GridMinimumTrackFunctions::from_qualified),
                rows: self
                    .row_minimum_functions
                    .as_deref()
                    .map(GridMinimumTrackFunctions::from_qualified),
            },
            gap: self.gap,
            parent_named_columns: &self.named_columns,
            parent_named_rows: &self.named_rows,
            parent_area_facts: self.area_facts.as_ref(),
            margin: self.margin,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: Edges::ZERO,
        }
    }
}

/// What every subgrid item of one parent placement reads of its parent. It is
/// copied once per placement and shared by the items' requests, so a parent
/// with many subgrid items copies its tracks, names and geometry once.
pub(in crate::grid) struct SubgridParentShare<Node, S: LayoutScalar> {
    parent_baseline_references: crate::GridBaselineReferences,
    columns: Vec<S>,
    rows: Vec<S>,
    column_minimum_functions: Option<OwnedMinimumTrackFunctions<S>>,
    row_minimum_functions: Option<OwnedMinimumTrackFunctions<S>>,
    gap: LogicalSizeOf<S>,
    named_columns: NamedGridLines,
    named_rows: NamedGridLines,
    area_facts: Option<GridAreaNameFacts>,
    column_geometry: Option<UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<UsedGridAxisGeometryOf<S>>,
    refresh: Option<RefreshShare<Node, S>>,
}

/// What the baseline refresh adds to the share of its parent placement.
pub(in crate::grid) struct RefreshShare<Node, S: LayoutScalar> {
    pub(in crate::grid) ancestor_baseline_groups: FinalAncestorBaselineGroups<Node, S>,
    pub(in crate::grid) template_area_expanded_axes: TemplateAreaExpandedAxes,
    pub(in crate::grid) parent: Node,
}

impl<Node, S: LayoutScalar> SubgridParentShare<Node, S> {
    pub(in crate::grid) fn column_geometry(&self) -> Option<&UsedGridAxisGeometryOf<S>> {
        self.column_geometry.as_ref()
    }

    pub(in crate::grid) fn row_geometry(&self) -> Option<&UsedGridAxisGeometryOf<S>> {
        self.row_geometry.as_ref()
    }

    pub(in crate::grid) fn refresh(&self) -> Option<&RefreshShare<Node, S>> {
        self.refresh.as_ref()
    }

    /// Copies the per-parent facts of `source`. Only the `Vec` buffers are
    /// reserved fallibly, and report an allocation error: the columns and
    /// rows, and the tracks or facts of the minimum functions. Cloning the
    /// elements (track sizing calculations), the named lines, the area facts,
    /// the used geometry and the baseline groups allocates infallibly, as the
    /// crate's other retained-context clones do.
    pub(in crate::grid) fn new(
        source: &SubgridChildParentContextInput<'_, '_, Node, S>,
        column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
        row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
        refresh: Option<RefreshShare<Node, S>>,
    ) -> Result<std::sync::Arc<Self>, SubgridChildContextError<S>>
    where
        Node: Copy,
    {
        let allocation = || {
            SubgridChildContextError::MinimumFunctions(
                GridMinimumFunctionRetentionError::Allocation,
            )
        };
        let owned = |functions: Option<GridMinimumTrackFunctions<'_, S>>| {
            functions
                .map(OwnedMinimumTrackFunctions::new)
                .transpose()
                .map_err(SubgridChildContextError::MinimumFunctions)
        };
        Ok(std::sync::Arc::new(Self {
            parent_baseline_references: source.parent_baseline_references,
            columns: try_clone_slice(source.columns).map_err(|_| allocation())?,
            rows: try_clone_slice(source.rows).map_err(|_| allocation())?,
            column_minimum_functions: owned(source.minimum_functions.columns)?,
            row_minimum_functions: owned(source.minimum_functions.rows)?,
            gap: source.gap,
            named_columns: source.parent_named_columns.clone(),
            named_rows: source.parent_named_rows.clone(),
            area_facts: source.parent_area_facts.cloned(),
            column_geometry: column_geometry.cloned(),
            row_geometry: row_geometry.cloned(),
            refresh,
        }))
    }
}

impl<Node: Copy + PartialEq, S: LayoutScalar> SubgridParentShare<Node, S> {
    fn same_minimum_functions(&self, other: &Self) -> bool {
        self.column_minimum_functions
            .as_ref()
            .map(OwnedMinimumTrackFunctions::view)
            == other
                .column_minimum_functions
                .as_ref()
                .map(OwnedMinimumTrackFunctions::view)
            && self
                .row_minimum_functions
                .as_ref()
                .map(OwnedMinimumTrackFunctions::view)
                == other
                    .row_minimum_functions
                    .as_ref()
                    .map(OwnedMinimumTrackFunctions::view)
    }

    /// Whether the two shares give a child the same parent facts. The baseline
    /// refresh data of either share is not compared: it only adds to a child's
    /// context where the refresh has ancestor baseline targets, which the
    /// caller excludes. A used geometry that a share lacks is the uniform
    /// geometry of its tracks, as the track inheritance reads it.
    fn same_parent_facts(&self, other: &Self) -> bool {
        let uniform =
            |tracks: &[S], gap: S| UsedGridAxisGeometryOf::new(tracks.to_vec(), Vec::new(), gap);
        let same_geometry = |first: &Option<UsedGridAxisGeometryOf<S>>,
                             next: &Option<UsedGridAxisGeometryOf<S>>,
                             tracks: &[S],
                             gap: S| {
            match (first, next) {
                (None, None) => true,
                (Some(first), Some(next)) => first == next,
                (None, Some(next)) => *next == uniform(tracks, gap),
                (Some(_), None) => false,
            }
        };
        self.parent_baseline_references == other.parent_baseline_references
            && self.columns == other.columns
            && self.rows == other.rows
            && self.same_minimum_functions(other)
            && self.gap == other.gap
            && self.named_columns == other.named_columns
            && self.named_rows == other.named_rows
            && self.area_facts == other.area_facts
            && same_geometry(
                &self.column_geometry,
                &other.column_geometry,
                &self.columns,
                self.gap.inline,
            )
            && same_geometry(
                &self.row_geometry,
                &other.row_geometry,
                &self.rows,
                self.gap.block,
            )
    }
}

/// One subgrid item's loan of its parent placement: the shared parent facts
/// and the item's own. The borrowed input is rebuilt from it for each
/// scrollbar pass of the child.
#[derive(Clone)]
pub(in crate::grid) struct SubgridItemLoan<'input, Node, S: LayoutScalar> {
    share: std::sync::Arc<SubgridParentShare<Node, S>>,
    item: SubgridItemReport<Node>,
    child_style: GridItemProjection<'input, S>,
    container: GridContainerInput<'input, S>,
    area: GridArea<S>,
    content_box_size: Size<S>,
    margin: Edges<Option<S>>,
    border: Edges<S>,
    padding: Edges<S>,
}

/// Every input of one contextual Grid computation of a subgrid item, as its
/// first request read them. The baseline refresh compares them with its own.
pub(in crate::grid) struct ContextualFingerprint<'input, Node, S: LayoutScalar> {
    input: ComputeInputOf<S>,
    minimum: Size<Option<StandaloneIntrinsicMinimum>>,
    stage: GridSizingStage,
    container: GridContainerInput<'input, S>,
    loan: SubgridItemLoan<'input, Node, S>,
    /// The writes the computation staged, which a reuse stages again.
    effects: Option<crate::engine::StagedEffects<Node, S>>,
}

impl<'input, Node: Copy + PartialEq, S: LayoutScalar> ContextualFingerprint<'input, Node, S> {
    pub(in crate::grid) fn retain_effects(
        &mut self,
        effects: Option<crate::engine::StagedEffects<Node, S>>,
    ) {
        self.effects = effects;
    }

    /// The inputs of `request`, or `None` when its parent context is not a
    /// loan that can be compared.
    pub(in crate::grid) fn of(request: &GridContextRequest<'input, Node, S>) -> Option<Self> {
        let GridParentContextSource::Loan(loan) = &request.parent_context else {
            return None;
        };
        Some(Self {
            input: request.input,
            minimum: request.minimum,
            stage: request.stage,
            container: request.child,
            loan: (**loan).clone(),
            effects: None,
        })
    }

    /// Whether `request` computes the same result as the request these inputs
    /// came from. A contextual Grid computation reads only the fields of its
    /// request and the parent facts its loan shares, so a request equal in all
    /// of them returns the same output, baselines and error. They are equal
    /// when:
    /// - the child's compute input, stage and container are the same;
    /// - the standalone-minimum phases the computation resolves are the same:
    ///   the request's own, completed by the container's;
    /// - the loan's item report, style, area, content box and margin, border
    ///   and padding are the same; and
    /// - the parent facts the loans share are the same: tracks, minimum
    ///   functions, gap, named lines, area facts, baseline references and used
    ///   geometry.
    ///
    /// The refresh share adds ancestor baseline groups and the parent's
    /// template-area expansion to the context, and the caller establishes
    /// that they add nothing for this item.
    pub(in crate::grid) fn is_reproduced_by(
        &self,
        request: &GridContextRequest<'input, Node, S>,
    ) -> bool {
        let GridParentContextSource::Loan(next) = &request.parent_context else {
            return false;
        };
        let resolved = |request_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
                        container: &GridContainerInput<'input, S>| {
            let contextual = standalone_intrinsic_minimum_phases(
                &container.projection(),
                container.inherited_axes(),
            );
            Size::new(
                request_minimum.width.or(contextual.width),
                request_minimum.height.or(contextual.height),
            )
        };
        self.input == request.input
            && self.stage == request.stage
            && self.container.same_container(&request.child)
            && resolved(self.minimum, &self.container) == resolved(request.minimum, &request.child)
            && self.loan.item == next.item
            && self.loan.child_style == next.child_style
            && self.loan.area == next.area
            && self.loan.content_box_size == next.content_box_size
            && self.loan.margin == next.margin
            && self.loan.border == next.border
            && self.loan.padding == next.padding
            && self.loan.share.same_parent_facts(&next.share)
    }
}

impl<'input, Node: Copy, S: LayoutScalar> SubgridItemLoan<'input, Node, S> {
    pub(in crate::grid) fn new(
        share: std::sync::Arc<SubgridParentShare<Node, S>>,
        source: &SubgridChildParentContextInput<'input, '_, Node, S>,
    ) -> Self {
        Self {
            share,
            item: source.item,
            child_style: source.child_style.clone(),
            container: source
                .child_container
                .expect("a retained subgrid loan has its original container"),
            area: source.area,
            content_box_size: source.content_box_size,
            margin: source.margin,
            border: source.border,
            padding: source.padding,
        }
    }

    pub(in crate::grid) fn share(&self) -> &SubgridParentShare<Node, S> {
        &self.share
    }

    pub(in crate::grid) fn input(&self) -> SubgridChildParentContextInput<'input, '_, Node, S> {
        let share = &*self.share;
        SubgridChildParentContextInput {
            item: self.item,
            parent_baseline_references: share.parent_baseline_references,
            child_style: &self.child_style,
            child_container: Some(self.container),
            area: self.area,
            content_box_size: self.content_box_size,
            columns: &share.columns,
            rows: &share.rows,
            minimum_functions: GridMinimumParentFunctions {
                columns: share
                    .column_minimum_functions
                    .as_ref()
                    .map(OwnedMinimumTrackFunctions::view),
                rows: share
                    .row_minimum_functions
                    .as_ref()
                    .map(OwnedMinimumTrackFunctions::view),
            },
            gap: share.gap,
            parent_named_columns: &share.named_columns,
            parent_named_rows: &share.named_rows,
            parent_area_facts: share.area_facts.as_ref(),
            margin: self.margin,
            border: self.border,
            padding: self.padding,
            scrollbar_gutter: Edges::ZERO,
        }
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
        child_container: input.child_container,
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
struct SubgridChildAxisContextInput<'input, 'a, Node, S: LayoutScalar = DefaultScalar> {
    parent_grid: Option<Node>,
    parent_baseline_references: crate::GridBaselineReferences,
    current_grid: Node,
    axis: GridAxisKind,
    report: SubgridAxisReport,
    child_style: &'a GridItemProjection<'input, S>,
    child_container: Option<GridContainerInput<'input, S>>,
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
                    .child_container
                    .expect("an inheriting subgrid axis must retain its container input")
                    .projection(),
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

struct SubgridParentAxisData<'a, S: LayoutScalar = DefaultScalar> {
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

pub(in crate::grid) fn apply_final_subgrid_axis_constraints<Node, S: LayoutScalar>(
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

pub(in crate::grid) struct SubgridBaselineRefreshInput<'a, Node, S: LayoutScalar = DefaultScalar> {
    pub(in crate::grid) node: Node,
    pub(in crate::grid) container_style: &'a GridContainerProjection<'a, S>,
    pub(in crate::grid) columns: &'a [S],
    pub(in crate::grid) rows: &'a [S],
    pub(in crate::grid) minimum_functions: GridMinimumParentFunctions<'a, S>,
    pub(in crate::grid) column_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) row_geometry: &'a UsedGridAxisGeometryOf<S>,
    pub(in crate::grid) row_tracks: &'a [TrackSizingOf<S>],
    pub(in crate::grid) track_basis: LogicalSizeOf<Option<S>>,
    pub(in crate::grid) gap: LogicalSizeOf<S>,
    pub(in crate::grid) named_columns: &'a NamedGridLines,
    pub(in crate::grid) named_rows: &'a NamedGridLines,
    pub(in crate::grid) area_facts: Option<&'a GridAreaNameFacts>,
    pub(in crate::grid) template_area_expanded_axes: TemplateAreaExpandedAxes,
    pub(in crate::grid) subgrid_report: &'a GridSubgridReport<Node>,
    pub(in crate::grid) ancestor_baseline_groups: &'a FinalAncestorBaselineGroups<Node, S>,
    pub(in crate::grid) executed_frame: ExecutedBaselineFrame<'a, S>,
    pub(in crate::grid) orthogonal_fallback: OrthogonalFallbackOf<S>,
    pub(in crate::grid) original_executed: &'a ExecutedBaselineInventory<Node, S>,
    pub(in crate::grid) containing_auto_scrollbar_pass: crate::scroll::SettledAutoScrollbarState,
    pub(in crate::grid) sizing_stage: GridSizingStage,
}

/// Whether the baseline refresh leaves the parent context of the item as the
/// first pass built it.
fn refresh_adds_nothing<Node: Copy + PartialEq, S: LayoutScalar>(
    input: &SubgridBaselineRefreshInput<'_, Node, S>,
    item: SubgridItemReport<Node>,
    sizing: &GridItemSizing<S>,
) -> bool {
    let baseline_aligned =
        |alignment| matches!(alignment, AlignItems::Baseline | AlignItems::LastBaseline);
    if baseline_aligned(sizing.justify_self) || baseline_aligned(sizing.align_self) {
        return false;
    }
    [item.column, item.row]
        .into_iter()
        .filter(|report| report.can_inherit())
        .all(|report| {
            let axis = report.mapping.parent_axis;
            let group = input.ancestor_baseline_groups.for_axis(axis);
            group.reference()
                == grid_baseline_reference(input.container_style.baseline_references, axis)
                && group.axis() == axis
                && input
                    .ancestor_baseline_groups
                    .inherited_targets_for_axis(axis)
                    .is_none()
                && !input.template_area_expanded_axes.for_axis(axis)
        })
}

type GridContextStep<'input, Node, S, M> =
    LayoutResultOf<Node, Option<GridContextRequest<'input, Node, S>>, S, M>;

/// The cursor of the post-placement subgrid baseline refresh. Each item that
/// needs context recomputes once, in source order, as a request the caller
/// executes; its receipt then replaces the item's output and baselines.
pub(in crate::grid) struct SubgridBaselineRefresh<Node, S: LayoutScalar = DefaultScalar> {
    next_index: usize,
    in_flight: Option<RefreshInFlight<S>>,
    /// The parent facts the items' requests share, copied when the first
    /// request is prepared.
    parent_share: Option<std::sync::Arc<SubgridParentShare<Node, S>>>,
}

// Facts of the request in flight that its receipt needs again.
struct RefreshInFlight<S: LayoutScalar> {
    index: usize,
    sizing: GridItemSizing<S>,
    child_input: ComputeInputOf<S>,
    padding: Edges<S>,
    border: Edges<S>,
    physical_area_size: Size<S>,
    stage: GridSizingStage,
}

impl<Node: Copy + Eq, S: LayoutScalar> SubgridBaselineRefresh<Node, S> {
    pub(in crate::grid) const fn new() -> Self {
        Self {
            next_index: 0,
            in_flight: None,
            parent_share: None,
        }
    }

    /// Prepares the next refreshed item's request, or `None` once every item
    /// has been refreshed.
    pub(in crate::grid) fn next<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        input: &SubgridBaselineRefreshInput<'_, Node, S>,
        pending_items: &[PendingGridItem<'input, Node, S>],
        item_space: impl Fn(&PendingGridItem<'input, Tree::Node, S>) -> GridItemLayoutSpace<S>,
    ) -> GridContextStep<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        debug_assert!(self.in_flight.is_none(), "a refresh request is outstanding");
        let container_flow_axes = FlowAxes::new(
            input.container_style.writing_mode,
            input.container_style.direction,
        );
        while let Some(item) = pending_items.get(self.next_index) {
            let index = self.next_index;
            self.next_index += 1;
            let Some(subgrid_item) = input.subgrid_report.items.get(item.source_index).copied()
            else {
                continue;
            };
            if !subgrid_item.needs_context() {
                continue;
            }

            let space = item_space(item);
            let physical_area_size = space.alignment_size;
            let child_flow_axes = FlowAxes::new(item.style.writing_mode, item.style.direction);
            let FinalGridItemPreparation {
                sizing,
                input: child_input,
                padding,
                border,
                content_box_size: subgrid_content_box_size,
            } = prepare_final_grid_item::<Tree, M>(
                tree,
                item.node,
                &item.style,
                input.container_style,
                subgrid_item,
                space,
                GridItemRequestContext {
                    containing_auto_scrollbar_pass: input.containing_auto_scrollbar_pass,
                    orthogonal_fallback: input.orthogonal_fallback,
                },
            )?;
            let child_input =
                grid_item_baseline_demand(input.container_style, &item.style, None, child_input)
                    .map_err(|error| grid_baseline_error(input.node, item.node, error))?;
            let child_container = item.nested_container.as_ref().copied();
            let child_context_input = SubgridChildParentContextInput {
                item: subgrid_item,
                parent_baseline_references: input.container_style.baseline_references,
                child_style: &item.style,
                child_container,
                area: item.area,
                content_box_size: subgrid_content_box_size,
                columns: input.columns,
                rows: input.rows,
                minimum_functions: input.minimum_functions,
                gap: input.gap,
                parent_named_columns: input.named_columns,
                parent_named_rows: input.named_rows,
                parent_area_facts: input.area_facts,
                margin: sizing.unresolved_margin,
                border,
                padding,
                scrollbar_gutter: Edges::ZERO,
            };
            let stage = input
                .sizing_stage
                .mapped(container_flow_axes, child_flow_axes);
            let child = child_context_input.container();
            let share = match &self.parent_share {
                Some(share) => Ok(share.clone()),
                None => SubgridParentShare::new(
                    &child_context_input,
                    Some(input.column_geometry),
                    Some(input.row_geometry),
                    Some(RefreshShare {
                        ancestor_baseline_groups: input.ancestor_baseline_groups.clone(),
                        template_area_expanded_axes: input.template_area_expanded_axes,
                        parent: input.node,
                    }),
                )
                .inspect(|share| self.parent_share = Some(share.clone())),
            };
            let parent_context = match share {
                Ok(share) => {
                    GridParentContextSource::loan(SubgridItemLoan::new(share, &child_context_input))
                }
                Err(error) => GridParentContextSource::Rejected(error),
            };
            let request = GridContextRequest {
                container: input.node,
                node: item.node,
                input: child_input,
                minimum: sizing.standalone_intrinsic_minimum,
                child,
                stage,
                parent_context,
            };
            // The refresh exists to give the item the ancestor baseline
            // groups the first pass could not know. The recomputation returns
            // what the first pass returned, and the item keeps its first
            // receipt, when all of these hold:
            // - the refresh adds nothing to the item's parent context: no
            //   ancestor baseline target reaches an inherited axis, the group
            //   references and axes agree (otherwise the context reports an
            //   error), no parent axis is template-area expanded, and the
            //   item aligns to no baseline;
            // - the refresh request equals the first-pass request in every
            //   input the computation reads (`is_reproduced_by`).
            // Any other item is recomputed, as before.
            if let Some(first) = &item.first_request
                && let Some(effects) = &first.effects
                && tree.reuses_subgrid_computations()
                && !crate::grid::subgrid_reuse_disabled()
                && refresh_adds_nothing(input, subgrid_item, &sizing)
                && first.is_reproduced_by(&request)
            {
                // The recomputation would stage its writes again, each
                // replacing its earlier write and moving last.
                tree.replay_effects(effects)?;
                continue;
            }
            tree.meter()
                .charge(crate::accounting::LayoutWork::BaselineRefresh, 1);
            self.in_flight = Some(RefreshInFlight {
                index,
                sizing,
                child_input,
                padding,
                border,
                physical_area_size,
                stage,
            });
            return Ok(Some(request));
        }
        Ok(None)
    }

    /// Applies the receipt of the request `next` prepared.
    pub(in crate::grid) fn accept<'input, M>(
        &mut self,
        input: &SubgridBaselineRefreshInput<'_, Node, S>,
        pending_items: &mut [PendingGridItem<'input, Node, S>],
        result: GridComputeResult<Node, S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let RefreshInFlight {
            index,
            sizing,
            child_input,
            padding,
            border,
            physical_area_size,
            stage,
        } = self
            .in_flight
            .take()
            .expect("a refresh receipt follows its request");
        let container_flow_axes = FlowAxes::new(
            input.container_style.writing_mode,
            input.container_style.direction,
        );
        let item = &mut pending_items[index];
        let subgrid_item = input.subgrid_report.items[item.source_index];
        let child_flow_axes = FlowAxes::new(item.style.writing_mode, item.style.direction);
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
            stage,
            executed_baselines,
        )?;
        let scroll_geometry = retained_grid_child_scroll_geometry(
            &item.style,
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
        Ok(())
    }
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
