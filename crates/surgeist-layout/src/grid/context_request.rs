//! A context-bearing Grid request owns the parent loan its scrollbar passes rebuild.
use super::*;

/// Where a Grid pass takes its inherited tracks. The source is owned, so a
/// suspended request rebuilds each scrollbar pass's context from the same
/// loan the parent granted.
pub(super) enum GridParentContextSource<'input, Node, S: LayoutScalar> {
    /// An ordinary Grid container inherits nothing.
    None,
    /// The loan could not be retained; its pass reports the retention error.
    Rejected(SubgridChildContextError<S>),
    /// A subgrid item placed by its parent's final geometry.
    Final(Box<FinalParentContext<'input, Node, S>>),
    /// A subgrid item refreshed with its parent's baseline groups.
    Refresh(Box<RefreshParentContext<'input, Node, S>>),
}

pub(super) struct FinalParentContext<'input, Node, S: LayoutScalar> {
    context: OwnedSubgridContext<'input, Node, S>,
    column_geometry: Option<UsedGridAxisGeometryOf<S>>,
    row_geometry: Option<UsedGridAxisGeometryOf<S>>,
}

pub(super) struct RefreshParentContext<'input, Node, S: LayoutScalar> {
    context: OwnedSubgridContext<'input, Node, S>,
    ancestor_baseline_groups: FinalAncestorBaselineGroups<Node, S>,
    template_area_expanded_axes: TemplateAreaExpandedAxes,
    parent: Node,
    column_geometry: UsedGridAxisGeometryOf<S>,
    row_geometry: UsedGridAxisGeometryOf<S>,
}

impl<'input, Node, S: LayoutScalar> GridParentContextSource<'input, Node, S> {
    pub(super) fn final_geometry(
        context: OwnedSubgridContext<'input, Node, S>,
        column_geometry: Option<UsedGridAxisGeometryOf<S>>,
        row_geometry: Option<UsedGridAxisGeometryOf<S>>,
    ) -> Self {
        Self::Final(Box::new(FinalParentContext {
            context,
            column_geometry,
            row_geometry,
        }))
    }

    pub(super) fn refresh(
        context: OwnedSubgridContext<'input, Node, S>,
        ancestor_baseline_groups: FinalAncestorBaselineGroups<Node, S>,
        template_area_expanded_axes: TemplateAreaExpandedAxes,
        parent: Node,
        column_geometry: UsedGridAxisGeometryOf<S>,
        row_geometry: UsedGridAxisGeometryOf<S>,
    ) -> Self {
        Self::Refresh(Box::new(RefreshParentContext {
            context,
            ancestor_baseline_groups,
            template_area_expanded_axes,
            parent,
            column_geometry,
            row_geometry,
        }))
    }
}

impl<'input, Node: Copy + PartialEq, S: LayoutScalar> GridParentContextSource<'input, Node, S> {
    /// Rebuilds the inherited tracks for one scrollbar pass of `subject`, the
    /// child of `container`.
    pub(super) fn build<M>(
        &self,
        container: Node,
        subject: Node,
        constants: &Constants<S>,
    ) -> LayoutResultOf<Node, GridParentContext<S, Node>, S, M> {
        let context = match self {
            Self::None => return Ok(GridParentContext::none()),
            Self::Rejected(error) => Err(*error),
            Self::Final(source) => subgrid_child_parent_context_with_geometry(
                source.context.input().for_scrollbar_pass(constants),
                source.column_geometry.as_ref(),
                source.row_geometry.as_ref(),
            ),
            Self::Refresh(refresh) => {
                subgrid_child_parent_context_from_ancestor_groups_with_geometry(
                    refresh.context.input().for_scrollbar_pass(constants),
                    &refresh.ancestor_baseline_groups,
                    refresh.template_area_expanded_axes,
                    refresh.parent,
                    Some(&refresh.column_geometry),
                    Some(&refresh.row_geometry),
                )
            }
        };
        context.map_err(|error| subgrid_child_context_container_error(container, subject, error))
    }
}

/// One contextual Grid computation: a subgrid item of `container`, its
/// standalone-minimum probe and scrollbar passes included. It bypasses child
/// cache and memo admission, as the synchronous computation did.
pub(crate) struct GridContextRequest<'input, Node, S: LayoutScalar> {
    pub(super) container: Node,
    pub(super) node: Node,
    pub(super) input: ComputeInputOf<S>,
    pub(super) minimum: Size<Option<StandaloneIntrinsicMinimum>>,
    pub(super) child: GridContainerInput<'input, S>,
    pub(super) stage: GridSizingStage,
    pub(super) parent_context: GridParentContextSource<'input, Node, S>,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridContextRequest<'input, Node, S> {
    pub(crate) fn node(&self) -> Node {
        self.node
    }

    pub(super) fn input(&self) -> ComputeInputOf<S> {
        self.input
    }

    /// Computes the request on the native stack, for the synchronous drains.
    pub(super) fn compute_sync<Tree, M>(self, tree: &mut Tree) -> GridComputeOutcome<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            container,
            node,
            input,
            minimum,
            child,
            stage,
            parent_context,
        } = self;
        compute_grid_with_context_source(tree, node, input, minimum, child, stage, |constants| {
            parent_context.build(container, node, constants)
        })
    }
}
