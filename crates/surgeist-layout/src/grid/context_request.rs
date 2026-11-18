//! A context-bearing Grid request owns the parent loan its scrollbar passes rebuild.
use super::*;

/// Where a Grid pass takes its inherited tracks. The source is owned, so a
/// suspended request rebuilds each scrollbar pass's context from the same
/// loan the parent granted.
pub(super) enum GridParentContextSource<'input, Node, S: LayoutScalar> {
    /// An ordinary Grid container inherits nothing.
    None,
    /// The loan could not be copied; its pass reports the error.
    Rejected(SubgridChildContextError<S>),
    /// A subgrid item's loan of its parent placement, for its final layout or
    /// for the refresh of its baselines.
    Loan(Box<SubgridItemLoan<'input, Node, S>>),
}

impl<'input, Node: Copy + PartialEq, S: LayoutScalar> GridParentContextSource<'input, Node, S> {
    pub(super) fn loan(loan: SubgridItemLoan<'input, Node, S>) -> Self {
        Self::Loan(Box::new(loan))
    }

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
            Self::Loan(loan) => {
                let share = loan.share();
                let input = loan.input().for_scrollbar_pass(constants);
                match share.refresh() {
                    None => subgrid_child_parent_context_with_geometry(
                        input,
                        share.column_geometry(),
                        share.row_geometry(),
                    ),
                    Some(refresh) => {
                        subgrid_child_parent_context_from_ancestor_groups_with_geometry(
                            input,
                            &refresh.ancestor_baseline_groups,
                            refresh.template_area_expanded_axes,
                            refresh.parent,
                            share.column_geometry(),
                            share.row_geometry(),
                        )
                    }
                }
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
        let start = tree.begin_effect_capture(crate::engine::CaptureKind::Request);
        let outcome = compute_grid_with_context_source(
            tree,
            node,
            input,
            minimum,
            child,
            stage,
            |constants| parent_context.build(container, node, constants),
        );
        let effects = tree.end_effect_capture(start);
        outcome.map(|result| result.with_effects(effects))
    }
}
