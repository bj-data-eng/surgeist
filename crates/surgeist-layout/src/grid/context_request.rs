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
    /// The parent facts an intrinsic probe or contribution owns; copying them
    /// may have failed, which its pass reports.
    Intrinsic(Result<Box<OwnedProbeContext<'input, Node, S>>, SubgridChildContextError<S>>),
}

/// The parent facts of one intrinsic probe or contribution, owned so that the
/// request outlives the borrows it was made from. They are the loan of a
/// contextual item's request: the parent's minimum functions wait until a span
/// of them is read, so a track the item does not span is never resolved. Each
/// scrollbar pass rebuilds the inherited tracks from them, adding the pass's
/// gutter to the spanned tracks of a content contribution.
pub(super) struct OwnedProbeContext<'input, Node, S: LayoutScalar> {
    loan: SubgridItemLoan<'input, Node, S>,
    parent_flow: crate::geometry::FlowAxes,
    kind: GridIntrinsicProbeKind,
}

impl<'input, Node: Copy + PartialEq, S: LayoutScalar> OwnedProbeContext<'input, Node, S> {
    fn key(&self, request: &GridContextRequest<'input, Node, S>) -> ProbeKey<'input, Node, S> {
        ProbeKey::new(request, &self.loan, self.parent_flow, self.kind)
    }

    fn build<M>(
        &self,
        subject: Node,
        constants: &Constants<S>,
    ) -> LayoutResultOf<Node, GridParentContext<S, Node>, S, M> {
        let source = self.loan.input();
        let child_flow = crate::geometry::FlowAxes::new(
            source.child_style.writing_mode,
            source.child_style.direction,
        );
        let mut columns = source.columns.to_vec();
        let mut rows = source.rows.to_vec();
        if let GridIntrinsicProbeKind::ContentContribution(queried_axis) = self.kind {
            let report = match queried_axis {
                GridAxisKind::Column => source.item.column,
                GridAxisKind::Row => source.item.row,
            };
            let gutter = self.parent_flow.logical_edges(constants.scrollbar_gutter);
            let (tracks, start, end, before, after) = match report.mapping.parent_axis {
                GridAxisKind::Column => (
                    &mut columns,
                    source.area.column,
                    source.area.column_end,
                    gutter.inline_start,
                    gutter.inline_end,
                ),
                GridAxisKind::Row => (
                    &mut rows,
                    source.area.row,
                    source.area.row_end,
                    gutter.block_start,
                    gutter.block_end,
                ),
            };
            if let Some(track) = tracks.get_mut(start) {
                *track = *track + before;
            }
            if let Some(last) = end.checked_sub(1).and_then(|end| tracks.get_mut(end)) {
                *last = *last + after;
            }
        }
        let column_geometry = self.loan.share().column_geometry().map(|geometry| {
            UsedGridAxisGeometryOf::from_sizing_gutters(columns.clone(), &geometry.sizing_gutters())
        });
        let row_geometry = self.loan.share().row_geometry().map(|geometry| {
            UsedGridAxisGeometryOf::from_sizing_gutters(rows.clone(), &geometry.sizing_gutters())
        });
        let mut frame = source;
        frame.columns = &columns;
        frame.rows = &rows;
        let mut extra = Size::ZERO;
        if let GridIntrinsicProbeKind::ContentContribution(queried_axis) = self.kind {
            let gutter_size = constants.scrollbar_gutter.sum_axes();
            match grid_axis_physical_axis(child_flow, queried_axis) {
                PhysicalAxis::Horizontal => extra.width = gutter_size.width,
                PhysicalAxis::Vertical => extra.height = gutter_size.height,
            }
        }
        frame.content_box_size = frame.content_box_size + extra;
        subgrid_child_parent_context_with_geometry(
            frame.for_scrollbar_pass(constants),
            column_geometry.as_ref(),
            row_geometry.as_ref(),
        )
        .map_err(|error| subgrid_child_context_error(subject, error))
    }
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
            Self::Intrinsic(owned) => {
                return match owned {
                    Ok(owned) => owned.build(subject, constants),
                    Err(error) => Err(subgrid_child_context_error(subject, *error)),
                };
            }
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

/// How a contextual Grid computation relates to the session that runs it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ContextRun {
    /// A subgrid item's final layout or baseline refresh. It captures the
    /// writes it stages for a later reuse.
    Item,
    /// An intrinsic contribution computed in the requesting session, which
    /// captures nothing.
    Contribution,
    /// An intrinsic probe. It runs in a disposable observation session of its
    /// own, from which only the observed output and gutter escape, and
    /// captures nothing.
    Probe,
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
    pub(super) run: ContextRun,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> GridContextRequest<'input, Node, S> {
    /// An intrinsic contribution or probe of the item `source` describes, with
    /// the parent facts the request owns.
    #[expect(
        clippy::too_many_arguments,
        reason = "an intrinsic request carries the inputs its parent context is rebuilt from"
    )]
    pub(super) fn intrinsic(
        run: ContextRun,
        node: Node,
        input: ComputeInputOf<S>,
        intrinsic_minimum: Size<Option<StandaloneIntrinsicMinimum>>,
        stage: GridSizingStage,
        kind: GridIntrinsicProbeKind,
        source: SubgridChildParentContextInput<'input, '_, Node, S>,
        column_geometry: Option<&UsedGridAxisGeometryOf<S>>,
        row_geometry: Option<&UsedGridAxisGeometryOf<S>>,
        parent_flow: crate::geometry::FlowAxes,
    ) -> Self {
        let child_flow = crate::geometry::FlowAxes::new(
            source.child_style.writing_mode,
            source.child_style.direction,
        );
        let parent_context = GridParentContextSource::Intrinsic(
            SubgridParentShare::new(&source, column_geometry, row_geometry, None).map(|share| {
                Box::new(OwnedProbeContext {
                    loan: SubgridItemLoan::new(share, &source),
                    parent_flow,
                    kind,
                })
            }),
        );
        Self {
            container: node,
            node,
            input,
            minimum: intrinsic_minimum,
            child: source.container(),
            stage: stage.mapped(parent_flow, child_flow),
            parent_context,
            run,
        }
    }

    /// An intrinsic contribution of a lane child that inherits nothing from
    /// its container.
    pub(super) fn standalone(
        container: Node,
        node: Node,
        input: ComputeInputOf<S>,
        child: GridContainerInput<'input, S>,
        stage: GridSizingStage,
    ) -> Self {
        Self {
            container,
            node,
            input,
            minimum: Size::NONE,
            child,
            stage,
            parent_context: GridParentContextSource::None,
            run: ContextRun::Contribution,
        }
    }

    pub(crate) fn node(&self) -> Node {
        self.node
    }

    pub(super) fn input(&self) -> ComputeInputOf<S> {
        self.input
    }

    /// The inputs that determine the result of an intrinsic probe, or `None`
    /// for a request that is not a probe or whose parent facts could not be
    /// copied.
    pub(crate) fn probe_key(&self) -> Option<ProbeKey<'input, Node, S>> {
        match (&self.parent_context, self.run) {
            (GridParentContextSource::Intrinsic(Ok(owned)), ContextRun::Probe) => {
                Some(owned.key(self))
            }
            _ => None,
        }
    }

    /// Computes the request on the native stack, for the synchronous drains.
    pub(super) fn compute_sync<Tree, M>(self, tree: &mut Tree) -> GridComputeOutcome<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match self.run {
            ContextRun::Item => {
                let start = tree.begin_effect_capture(crate::engine::CaptureKind::Request);
                let outcome = self.compute_in_session(tree);
                let effects = tree.end_effect_capture(start);
                outcome.map(|result| result.with_effects(effects))
            }
            ContextRun::Contribution => self.compute_in_session(tree),
            ContextRun::Probe => {
                let node = self.node;
                let observation = tree.observe_grid_overflow(
                    node,
                    GridOverflowProbe {
                        layout: GridOverflowLayout::Context(Box::new(self)),
                    },
                )?;
                match observation {
                    GridOverflowObservation::Intrinsic(observation) => {
                        Ok(GridComputeResult::probe_receipt(
                            observation.output,
                            observation.effective_gutter,
                        ))
                    }
                    GridOverflowObservation::Children(_) => {
                        unreachable!("an intrinsic probe returns its prepared output")
                    }
                }
            }
        }
    }

    /// Computes the request in the session that holds the tree, capturing
    /// nothing.
    pub(super) fn compute_in_session<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> GridComputeOutcome<Node, S, M>
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
            ..
        } = self;
        compute_grid_with_context_source(tree, node, input, minimum, child, stage, |constants| {
            parent_context.build(container, node, constants)
        })
    }

    /// Whether the request runs in an observation session of its own.
    pub(crate) fn is_probe(&self) -> bool {
        self.run == ContextRun::Probe
    }

    /// Whether the request captures the writes it stages.
    pub(crate) fn captures(&self) -> bool {
        self.run == ContextRun::Item
    }

    /// The request as the observation session runs it.
    pub(crate) fn observed(mut self) -> Self {
        self.run = ContextRun::Contribution;
        self
    }
}
