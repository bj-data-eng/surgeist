use super::*;

pub(in crate::block) enum InFlowStep<'input, Node, S: LayoutScalar> {
    Continue,
    Complete,
    Child(InFlowChild<'input, Node, S>),
}

pub(crate) struct InFlowChild<'input, Node, S: LayoutScalar> {
    request: scope::ChildRequest<Node, S>,
    pending: PendingInFlowChild<'input, Node, S>,
}

pub(crate) enum PendingInFlowChild<'input, Node, S: LayoutScalar> {
    Hidden,
    Box(PendingBox<'input, Node, S>),
    InlineReceiver(crate::block::inline_run::AtomicReceiverChild<Node, S>),
    Inline(crate::block::inline_run::PendingInlinePreparation<'input, Node, S>),
    FloatBand(FloatBandRequest<S>),
}

type ChildCompletion<'input, Node, S> = (
    PendingInFlowChild<'input, Node, S>,
    BlockChildReceipt<Node, S>,
);

impl<Node: Copy, S: LayoutScalar> PendingInFlowChild<'_, Node, S> {
    pub(crate) fn shared_request(&self) -> Option<(Node, ComputeInputOf<S>)> {
        match self {
            Self::Box(child) => child.shared_request(),
            _ => None,
        }
    }

    pub(crate) fn float_trial(&self) -> Option<crate::engine::FloatChildTrial> {
        match self {
            Self::FloatBand(band) => band.trial,
            Self::Inline(child) => child.float_trial(),
            _ => None,
        }
    }
}

impl<'input, Node: Copy, S: LayoutScalar> InFlowChild<'input, Node, S> {
    pub(crate) fn into_request(
        self,
    ) -> (
        scope::ChildRequest<Node, S>,
        PendingInFlowChild<'input, Node, S>,
    ) {
        (self.request, self.pending)
    }

    pub(in crate::block) fn compute<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> ComputeResult<Node, ChildCompletion<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self { request, pending } = self;
        if let Some(trial) = pending.float_trial() {
            let packet = tree.probe_float_child(request.node, request.input, trial)?;
            return Ok((pending, BlockChildReceipt::FloatTrial(packet)));
        }
        let output = match &pending {
            PendingInFlowChild::Box(child) if child.preparation.inherits_float_exclusions => {
                match tree.compute_child(request.node, request.input) {
                    Ok(output) => output,
                    Err(error) => {
                        let _ = tree.end_child_float_context(request.node, request.input);
                        return Err(error.into());
                    }
                }
            }
            PendingInFlowChild::Box(_) => tree.compute_block_child(request)?,
            PendingInFlowChild::Hidden
            | PendingInFlowChild::InlineReceiver(_)
            | PendingInFlowChild::Inline(_)
            | PendingInFlowChild::FloatBand(_) => {
                tree.compute_child(request.node, request.input)?
            }
        };
        Ok((pending, BlockChildReceipt::Layout(output)))
    }
}

pub(crate) struct PendingBox<'input, Node, S: LayoutScalar> {
    child: Node,
    style: BlockChildProjection<'input, S>,
    prepared: PreparedChild<S>,
    preparation: InFlowBoxPreparation<S>,
}

impl<Node: Copy, S: LayoutScalar> PendingBox<'_, Node, S> {
    pub(crate) fn shared_request(&self) -> Option<(Node, ComputeInputOf<S>)> {
        self.preparation
            .inherits_float_exclusions
            .then(|| (self.child, self.preparation.shared_input()))
    }
}

struct ActiveInlineRun<'input, Node, S: LayoutScalar> {
    transition: VisibleInlineRunTransition,
    context: InlineRunContext<S>,
    preparation: crate::block::inline_run::InlineRunPreparation<'input, Node, S>,
    fitting: Option<Box<crate::block::inline_run::NumericFit<S>>>,
}

pub(in crate::block) struct InFlowTraversal<'input, Node, S: LayoutScalar> {
    active_inline: Option<ActiveInlineRun<'input, Node, S>>,
    float_avoidance: Option<(PendingBox<'input, Node, S>, FloatAvoidance<S>)>,
    node: Node,
    input: ComputeInputOf<S>,
    inline_operation: crate::SizingOperation,
    set_layout: bool,
    logical_node_inner_size: LogicalSizeOf<Option<S>>,
    node_inner_size: Size<Option<S>>,
    adjoining: entry::AdjoiningFlow<S>,
    remaining_cut: Option<entry::canonical::Separation<Node, S>>,
    shared_child: Option<PendingSharedBox<Node, S>>,
    occupied_block_end: S,
    intrinsic_inline_demand: S,
    owned_float_block_end: S,
    resolved_terminal_float_block_end: Option<S>,
    baselines: super::baseline_frame::PendingBaselines,
    static_positions: Vec<(Node, StaticPositionOf<S>)>,
    fragment_margins: Option<super::fragment_margins::Flow>,
    pending_floats: Vec<PendingFloat<Node, S>>,
    pass_request: crate::SizingRequestOf<S>,
    flow_request: AxisSizingRequestOf<S>,
    normal_operation: bool,
    child_run_mode: RunMode,
    float_intrinsics: FloatIntrinsics<S>,
    physical_float_context: bool,
    content_inline_size: S,
    containing_size: Size<S>,
    float_exclusions: FloatExclusions<S, Node>,
    inherited_floats: usize,
    contributions: BlockScrollContributions<S>,
    index: usize,
    formatted: Option<crate::text_layout::formatted::FormattedFlow<Node, S>>,
    first_in_flow: bool,
    inline_indent_state: Option<crate::text_indent::IndentLineState>,
    box_frontier: Option<crate::text_layout::boxes::BoxFrontier>,
    line_box_context: crate::line_box::LineBoxContext,
    hyphenation_state: crate::hyphenation::LineHyphenationState,
    tail_active: bool,
    tail_ledger: crate::hyphenation::BlockTailLedger,
    slots: Vec<crate::text_layout::capture::NativeRunSlot>,
    _slot_lease: Option<crate::text_layout::scope_alignment::ScopeLease>,
    slot_cursor: usize,
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> InFlowTraversal<'input, Node, S> {
    pub(in crate::block) fn begin<Tree, M>(
        tree: &mut Tree,
        node: Node,
        constants: &Constants<S>,
        input: ComputeInputOf<S>,
        pass: InFlowPassContext<'_, S, Node>,
    ) -> ComputeResult<Node, Self, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let InFlowPassContext {
            inline_operation,
            inner_inline,
            set_layout,
            inherited,
            projection,
        } = pass;
        let logical_node_inner_size =
            LogicalSizeOf::new(inner_inline, constants.logical_node_inner_size().block);
        let node_inner_size = constants.flow_axes.physical_size(logical_node_inner_size);
        let cursor = projection.map_or(
            constants.logical_content_box_inset().block_start,
            |reference| reference.content_origin,
        );
        let adjoining = entry::AdjoiningFlow::new(cursor, constants.collapse_top_margin);
        let remaining_cut = tree
            .block_float_state()
            .and_then(|state| state.entry_cut(node));
        let occupied_block_end = S::ZERO;
        let intrinsic_inline_demand = S::ZERO;
        let owned_float_block_end = adjoining.cursor;
        let resolved_terminal_float_block_end = None;
        let baselines = super::baseline_frame::PendingBaselines::new();
        let static_positions = Vec::new();
        let fragment_margins = tree
            .fragment_margin_state()
            .map(|_| super::fragment_margins::Flow::default());
        let pending_floats = Vec::new();
        let mut requests = constants.sizing_request.axes();
        let inline = constants
            .flow_axes
            .logical_size(requests)
            .inline
            .with_operation(inline_operation);
        match constants.flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => requests.width = inline,
            PhysicalAxis::Vertical => requests.height = inline,
        }
        let pass_request = constants.sizing_request.with_axes(requests);
        let flow_request = crate::layout_math::with_available_extent(
            constants.flow_axes.logical_size(pass_request.axes()).inline,
            inner_inline,
        )
        .map_err(|error| crate::error::sizing_resolution_error(node, error.into()))?;
        let normal_operation = inline_operation == crate::SizingOperation::Normal;
        let child_run_mode = if normal_operation {
            input.run_mode().for_child()
        } else {
            RunMode::ComputeSize
        };
        let float_intrinsics = FloatIntrinsics::new(flow_request);
        let finite_content_inline = inner_inline.or(constants
            .flow_axes
            .logical_size(constants.sizing_request.axes())
            .inline
            .space()
            .definite_size()
            .map(crate::NonNegativeFiniteOf::get));
        // A Shared ComputeSize preview has no incoming BFC frame. Its ordinary
        // float rows belong to the subsequent framed computation, not this trial.
        let physical_float_context = normal_operation
            && (input.run_mode().is_perform_layout()
                || float_context::sizes_float_context(
                    float_context::Role::for_input(tree.node_input(node), input),
                    input,
                    finite_content_inline,
                    inherited.is_some(),
                    tree.block_float_work(node),
                ));
        let content_inline_size = finite_content_inline.unwrap_or(S::ZERO);
        // A finite ordinary-child preview still needs the real float bottom of
        // an owning BFC. Its private ledger determines size without publishing
        // descendants. Intrinsic inline requests retain their separate frontiers.
        let containing_size = projection.map_or_else(
            || constants.containing_size(logical_node_inner_size),
            |reference| {
                let mut size = constants
                    .flow_axes
                    .logical_size(constants.containing_size(logical_node_inner_size));
                size.block = reference.block_reference;
                constants.flow_axes.physical_size(size)
            },
        );
        let mut float_exclusions = FloatExclusions::new(
            constants.flow_axes,
            containing_size,
            content_inline_size,
            constants.logical_content_box_inset(),
        );
        let entry_case_funding = tree
            .text_computation()
            .and_then(|text| text.entry_case_funding().cloned());
        float_exclusions.fund_entry(entry_case_funding.as_ref());
        if physical_float_context && let Some(inherited) = inherited {
            float_exclusions
                .inherit_into_child(inherited)
                .map_err(|error| super::inline_run::text_run_error(node, error.into()))?;
        }
        let inherited_floats = float_exclusions.len();
        let content_box_size = Size::new(
            inner_inline
                .or(constants.node_inner_size.width)
                .or(constants
                    .sizing_request
                    .axes()
                    .width
                    .space()
                    .definite_size()
                    .map(crate::NonNegativeFiniteOf::get))
                .unwrap_or(S::ZERO),
            constants.node_inner_size.height.unwrap_or(S::ZERO),
        );
        let content_box_origin = Point::new(
            constants.content_box_inset.left,
            constants.content_box_inset.top,
        );
        let contribution_seed = crate::ScrollRectOf::try_new(content_box_origin, content_box_size)
            .map_err(|error| layout_own_geometry_error(node, input.run_mode(), error))?;
        let contributions = BlockScrollContributions::new(constants.flow_axes, contribution_seed);

        let index = 0;
        if let Some(text) = tree.text_computation() {
            text.formatted_begin_flow(input, flow_request, set_layout)
                .map_err(|error| super::inline_run::text_run_error(node, error.into()))?;
        }
        if let Some(groups) = tree.group_computation() {
            groups.begin_flow(input, flow_request, set_layout);
        }
        let formatted = tree
            .text_computation()
            .and_then(|text| text.formatted_new_flow());
        let first_in_flow = true;
        let inline_indent_state = None;
        let box_frontier = None;
        let line_box_context = crate::line_box::LineBoxContext::new(
            constants.line_box_direction_policy,
            constants.direction,
        );
        let hyphenation_state =
            crate::hyphenation::LineHyphenationState::new(constants.hyphenate_limit_lines);
        let tail_active = constants.hyphenate_limit_last == crate::HyphenateLimitLast::Always
            && crate::layout_math::normal_available_size(flow_request).is_some();
        let tail_ledger = crate::hyphenation::BlockTailLedger::default();
        let (slots, _slot_lease) = match tree.text_computation() {
            Some(state) => state.semantic_slots(node, None)?,
            None => (Vec::new(), None),
        };
        let slot_cursor = 0;
        Ok(Self {
            active_inline: None,
            float_avoidance: None,
            node,
            input,
            inline_operation,
            set_layout,
            logical_node_inner_size,
            node_inner_size,
            adjoining,
            remaining_cut,
            shared_child: None,
            occupied_block_end,
            intrinsic_inline_demand,
            owned_float_block_end,
            resolved_terminal_float_block_end,
            baselines,
            static_positions,
            fragment_margins,
            pending_floats,
            pass_request,
            flow_request,
            normal_operation,
            child_run_mode,
            float_intrinsics,
            physical_float_context,
            content_inline_size,
            containing_size,
            float_exclusions,
            inherited_floats,
            contributions,
            index,
            formatted,
            first_in_flow,
            inline_indent_state,
            box_frontier,
            line_box_context,
            hyphenation_state,
            tail_active,
            tail_ledger,
            slots,
            _slot_lease,
            slot_cursor,
        })
    }

    pub(in crate::block) fn step<Tree, M>(
        &mut self,
        tree: &mut Tree,
        children: &[Node],
        constants: &Constants<S>,
    ) -> ComputeResult<Node, InFlowStep<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        if self.active_inline.is_some() {
            return self.advance_inline(tree, children, constants);
        }
        if let Some((pending, avoidance)) = &self.float_avoidance {
            let computation =
                self.box_computation(pending.child, &pending.style, &pending.prepared, constants);
            let band = avoidance.request(tree, &computation, &pending.preparation)?;
            return Ok(InFlowStep::Child(InFlowChild {
                request: scope::ChildRequest {
                    node: pending.child,
                    input: band.input,
                    entry: None,
                },
                pending: PendingInFlowChild::FloatBand(band),
            }));
        }
        if self.index >= children.len() && self.slot_cursor >= self.slots.len() {
            return Ok(InFlowStep::Complete);
        }
        let node = self.node;
        let input = self.input;
        let source_index = self.index;
        let semantic_end = self
            .slots
            .get(self.slot_cursor)
            .map_or(children.len(), |slot| slot.gap);
        let child_start = if let Some(slot) = self
            .slots
            .get(self.slot_cursor)
            .filter(|slot| slot.gap == self.index)
        {
            self.slot_cursor += 1;
            InFlowChildStart::VisibleInlineRun(VisibleInlineRunTransition::semantic(slot))
        } else {
            start_native_child(tree, node, children, constants, self.index, semantic_end)?
        };
        let (child, child_style) = match child_start {
            InFlowChildStart::Hidden { child, compute_box } => {
                if self.set_layout {
                    tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
                        child,
                        crate::output::NormalGeometryOf::metadata(
                            crate::Point::ZERO,
                            crate::Size::ZERO,
                        ),
                        NodeOutputOf::<S>::with_source_index(crate::SourceIndex::new(source_index)),
                    ));
                    if compute_box {
                        return Ok(InFlowStep::Child(InFlowChild {
                            request: scope::ChildRequest {
                                node: child,
                                input: ComputeInputOf::<S>::hidden_in_containing_pass(
                                    ContainingLayoutContext::new(
                                        constants.flow_axes,
                                        ParentFormattingContext::BlockFlow,
                                    ),
                                    input.settled_auto_scrollbars(),
                                ),
                                entry: None,
                            },
                            pending: PendingInFlowChild::Hidden,
                        }));
                    }
                }
                self.index += 1;
                return Ok(InFlowStep::Continue);
            }
            InFlowChildStart::Positioned { child, geometry } => {
                self.static_positions.push((
                    child,
                    absolute_static_position(
                        self.adjoining.cursor + self.adjoining.active.resolve(),
                        constants,
                        self.containing_size,
                        geometry,
                    ),
                ));
                self.index += 1;
                return Ok(InFlowStep::Continue);
            }
            InFlowChildStart::VisibleInlineRun(transition) => {
                let run_start = transition.start;
                let run_end = transition.end;
                self.index = run_end;
                let context = InlineRunContext {
                    inline_request: self.flow_request,
                    child_request: self.pass_request,
                    box_frontier: self.box_frontier.clone(),
                    tail: if !self.tail_active {
                        crate::text_layout::fit::TailContext::Unrestricted
                    } else if self.slot_cursor == self.slots.len()
                        && children[run_end..].iter().all(|&node| {
                            match InlineParticipantProjection::lookup::<Tree, M>(tree, node).kind()
                            {
                                InlineParticipantKindOf::Box(style) => {
                                    style.display == crate::Display::None
                                        || style.common.position == Position::Absolute
                                        || !style.float.is_none()
                                }
                                _ => false,
                            }
                        })
                    {
                        crate::text_layout::fit::TailContext::CompleteBlock
                    } else {
                        crate::text_layout::fit::TailContext::Unresolved
                    },
                    hyphenation_state: self.hyphenation_state,
                    line_box_context: self.line_box_context,
                    source_index_start: run_start,
                    cursor_block: self.adjoining.cursor,
                    owned_float_block_end: self.owned_float_block_end,
                    input,
                    node_inner_size: self.node_inner_size,
                    set_layout: self.set_layout,
                    indent_state: self.inline_indent_state.unwrap_or_else(|| {
                        crate::text_indent::IndentLineState::new(
                            self.first_in_flow && constants.first_formatted,
                        )
                    }),
                };
                let preparation = transition.begin(
                    tree,
                    node,
                    &children[run_start..run_end],
                    constants,
                    &context,
                    self.adjoining.adjoining_displacement(),
                )?;
                self.active_inline = Some(ActiveInlineRun {
                    transition,
                    context,
                    preparation,
                    fitting: None,
                });
                return self.advance_inline(tree, children, constants);
            }
            InFlowChildStart::FlowBox { child, style } => (child, style),
        };
        if child_style.float.is_none() {
            if self.tail_active {
                self.tail_ledger.child_block();
            }
            self.inline_indent_state = None;
        }

        let prepared = prepare_child(
            tree,
            child,
            &child_style,
            constants,
            ChildSizingContext {
                inner_size: self.node_inner_size,
                request: self.pass_request,
                run_mode: self.child_run_mode,
                axis: RequestedAxis::Both,
                first_in_flow: &mut self.first_in_flow,
            },
        )?;
        let computation = self.box_computation(child, &child_style, &prepared, constants);
        let (request, preparation) = prepare_in_flow_box(tree, &computation)?;
        let shared = preparation.inherits_float_exclusions;
        let request = if shared {
            let (frame, pending) =
                prepare_shared_box(tree, &computation, &preparation, self.remaining_cut)?;
            let input = preparation.shared_input();
            tree.begin_child_float_context(child, input, frame)?;
            self.shared_child = Some(pending);
            scope::ChildRequest {
                node: child,
                input,
                entry: None,
            }
        } else {
            request
        };
        Ok(InFlowStep::Child(InFlowChild {
            request,
            pending: PendingInFlowChild::Box(PendingBox {
                child,
                style: child_style,
                prepared,
                preparation,
            }),
        }))
    }

    fn advance_inline<Tree, M>(
        &mut self,
        tree: &mut Tree,
        children: &[Node],
        constants: &Constants<S>,
    ) -> ComputeResult<Node, InFlowStep<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let active = self.active_inline.as_mut().expect("active inline run");
        let run = &children[active.transition.start..active.transition.end];
        if active.fitting.is_none()
            && let Some(child) =
                active
                    .preparation
                    .next_child(tree, self.node, run, constants, &active.context)?
        {
            return Ok(InFlowStep::Child(InFlowChild {
                request: scope::ChildRequest {
                    node: child.node,
                    input: child.input,
                    entry: None,
                },
                pending: PendingInFlowChild::Inline(child.pending),
            }));
        }
        if active.fitting.is_none() {
            active.fitting = active.preparation.begin_numeric_fit(
                tree,
                self.node,
                run,
                constants,
                &active.context,
            )?;
        }
        if let Some(fitting) = active.fitting.as_mut()
            && let Some(child) = active.preparation.advance_numeric_fit(
                fitting,
                tree,
                self.node,
                constants,
                &self.float_exclusions,
            )?
        {
            return Ok(InFlowStep::Child(InFlowChild {
                request: scope::ChildRequest {
                    node: child.node,
                    input: child.input,
                    entry: None,
                },
                pending: PendingInFlowChild::InlineReceiver(child),
            }));
        }
        let ActiveInlineRun {
            transition,
            context,
            preparation,
            fitting,
        } = self.active_inline.take().expect("completed inline run");
        let numeric_fit = fitting.map(|fit| fit.finish(self.node)).transpose()?;
        transition.apply(
            tree,
            self.node,
            run,
            constants,
            InlineRunRequest {
                context,
                prepared: preparation.prepared,
                adjoining_margin: self.adjoining.adjoining_displacement(),
                numeric_fit,
            },
            InlineRunTransitionState {
                box_frontier: &mut self.box_frontier,
                formatted: &mut self.formatted,
                indent_state: &mut self.inline_indent_state,
                line_box_context: &mut self.line_box_context,
                hyphenation_state: &mut self.hyphenation_state,
                tail_ledger: &mut self.tail_ledger,
                adjoining: &mut self.adjoining,
                fragment_margins: &mut self.fragment_margins,
                intrinsic_inline_demand: &mut self.intrinsic_inline_demand,
                baselines: &mut self.baselines,
                static_positions: &mut self.static_positions,
                resolved_terminal_float_block_end: &mut self.resolved_terminal_float_block_end,
                float_exclusions: &mut self.float_exclusions,
                float_intrinsics: &mut self.float_intrinsics,
                pending_floats: &mut self.pending_floats,
                owned_float_block_end: &mut self.owned_float_block_end,
                contributions: &mut self.contributions,
            },
        )?;
        self.first_in_flow &= self.inline_indent_state.is_none();
        Ok(InFlowStep::Continue)
    }

    fn box_computation<'a>(
        &'a self,
        child: Node,
        child_style: &'a BlockChildProjection<'input, S>,
        prepared: &'a PreparedChild<S>,
        constants: &'a Constants<S>,
    ) -> InFlowBoxComputation<'a, 'input, Node, S> {
        InFlowBoxComputation {
            node: self.node,
            child,
            index: self.index,
            child_style,
            constants,
            prepared,
            adjoining: &self.adjoining,
            baselines: &self.baselines,
            float_exclusions: &self.float_exclusions,
            input: self.input,
            pass_request: self.pass_request,
            child_run_mode: self.child_run_mode,
            normal_operation: self.normal_operation,
            physical_float_context: self.physical_float_context,
            containing_size: self.containing_size,
            inner_inline: self.logical_node_inner_size.inline,
            content_inline_size: self.content_inline_size,
            occupied_block_end: self.occupied_block_end,
        }
    }

    pub(in crate::block) fn accept_child<Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
        pending: PendingInFlowChild<'input, Node, S>,
        receipt: BlockChildReceipt<Node, S>,
    ) -> ComputeResult<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let mut pending = match pending {
            PendingInFlowChild::InlineReceiver(child) => {
                let active = self.active_inline.as_mut().expect("active inline run");
                return active
                    .preparation
                    .accept_receiver(
                        active
                            .fitting
                            .as_mut()
                            .expect("named receiver belongs to numeric fitting"),
                        self.node,
                        child,
                        receipt.layout(),
                        constants.flow_axes,
                    )
                    .map_err(Into::into);
            }
            PendingInFlowChild::Inline(pending) => {
                let active = self.active_inline.as_mut().expect("active inline run");
                return active
                    .preparation
                    .accept_child(
                        tree,
                        self.node,
                        constants,
                        &active.context,
                        pending,
                        receipt,
                    )
                    .map_err(Into::into);
            }
            PendingInFlowChild::Box(pending) => Some(pending),
            PendingInFlowChild::Hidden => None,
            PendingInFlowChild::FloatBand(band) => {
                return self.accept_float_band(tree, constants, band, receipt);
            }
        };
        let output = receipt.layout();
        if let Some(preview) = pending.as_ref()
            && preview.preparation.places_against_float_exclusions
        {
            let computation =
                self.box_computation(preview.child, &preview.style, &preview.prepared, constants);
            let avoidance = FloatAvoidance::begin(&computation, &preview.preparation, output);
            self.float_avoidance = Some((pending.take().expect("float child preview"), avoidance));
            return Ok(());
        }
        if let Some(PendingBox {
            child,
            style,
            prepared,
            preparation,
        }) = pending
        {
            let shared = preparation
                .inherits_float_exclusions
                .then(|| self.shared_child.take().expect("prepared shared child"));
            let computation = self.box_computation(child, &style, &prepared, constants);
            let computed = if let Some(shared) = shared {
                let mut remaining_cut = self.remaining_cut;
                let computed = finish_shared_box(
                    tree,
                    computation,
                    preparation,
                    shared,
                    output,
                    &mut remaining_cut,
                )?;
                self.remaining_cut = remaining_cut;
                computed
            } else {
                finish_requested_box(computation, preparation, output, None, self.remaining_cut)
            };
            self.accept_box(tree, constants, self.index, child, &style, computed)?;
        }
        self.index += 1;
        Ok(())
    }

    fn accept_float_band<Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
        band: FloatBandRequest<S>,
        receipt: BlockChildReceipt<Node, S>,
    ) -> ComputeResult<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let (pending, mut avoidance) = self.float_avoidance.take().expect("active float avoidance");
        let (output, packet) = match receipt {
            BlockChildReceipt::Layout(output) => (output, None),
            BlockChildReceipt::FloatTrial(packet) => (packet.output, Some(packet)),
        };
        let computation =
            self.box_computation(pending.child, &pending.style, &pending.prepared, constants);
        let Some(measured) = avoidance.accept(&computation, &pending.preparation, band, output)
        else {
            self.float_avoidance = Some((pending, avoidance));
            return Ok(());
        };
        if let Some(packet) = packet {
            tree.accept_float_child(&packet)?;
        }
        let computed = finish_requested_box(
            computation,
            pending.preparation,
            output,
            Some(measured),
            self.remaining_cut,
        );
        self.accept_box(
            tree,
            constants,
            self.index,
            pending.child,
            &pending.style,
            computed,
        )?;
        self.index += 1;
        Ok(())
    }

    fn accept_box<Tree, M>(
        &mut self,
        tree: &mut Tree,
        constants: &Constants<S>,
        source_index: usize,
        child: Node,
        child_style: &BlockChildProjection<'input, S>,
        computed: InFlowComputedBox<Node, S>,
    ) -> ComputeResult<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let node = self.node;
        let InFlowComputedBox {
            output,
            child_input,
            child_margin,
            child_padding,
            child_border,
            logical_child_margin,
            logical_child_size,
            avoids_float_exclusions,
            accepted_float_position,
            shared_child,
            mut clearance_funding,
            introduced_cut,
        } = computed;
        let child_flow_axes = child_style.common.flow_axes;
        if !child_style.float.is_none() {
            place_in_flow_float(
                node,
                InFlowFloatItem {
                    child,
                    source_index,
                    child_style,
                    output,
                    child_border,
                    child_padding,
                    child_margin,
                    logical_child_size,
                    logical_child_margin,
                },
                InFlowFloatPass {
                    constants,
                    adjoining: &self.adjoining,
                    inline_operation: self.inline_operation,
                    physical_float_context: self.physical_float_context,
                    set_layout: self.set_layout,
                },
                InFlowFloatState {
                    float_exclusions: &mut self.float_exclusions,
                    float_intrinsics: &mut self.float_intrinsics,
                    pending_floats: &mut self.pending_floats,
                    intrinsic_inline_demand: &mut self.intrinsic_inline_demand,
                    owned_float_block_end: &mut self.owned_float_block_end,
                },
            )?;
            return Ok(());
        }
        self.resolved_terminal_float_block_end = None;
        if let Some(groups) = tree.group_computation() {
            groups.child(child);
        }
        if child_style.display.inner_display() == crate::Display::Block
            && let Some(flow) = &mut self.formatted
        {
            let anonymous =
                tree.node_input(child).block_generation == crate::BlockGeneration::Anonymous;
            if let Some(item) = tree
                .text_computation()
                .map(|text| text.formatted_child(child, child_input, anonymous))
                .transpose()
                .map_err(|error| super::inline_run::text_run_error(child, error.into()))?
                .flatten()
            {
                flow.push(item)
                    .map_err(|error| super::inline_run::text_run_error(child, error.into()))?;
            }
        }
        let deferred_inset =
            super::relative::needs_basis(*child_style.common.inset, constants.flow_axes);
        let inset_offset = if deferred_inset {
            Point::ZERO
        } else {
            super::relative::resolve(
                tree,
                child,
                *child_style.common.inset,
                super::relative::InsetReference::Measurement(self.node_inner_size),
                constants.flow_axes,
            )?
        };
        let margin_placement = begin_in_flow_child_margins(
            tree,
            InFlowChildMargins {
                node,
                child,
                index: self.index,
                child_style,
                output: &output,
                child_margin,
                constants,
                float_exclusions: &self.float_exclusions,
                clearance_funding: &mut clearance_funding,
                introduced_cut,
            },
            &mut self.adjoining,
            &mut self.remaining_cut,
            &mut self.fragment_margins,
        )?;
        let InFlowBoxOrigins {
            logical: logical_location,
            normal: flow_location,
        } = place_in_flow_box(
            tree,
            InFlowBoxPlacement {
                node,
                child,
                source_index,
                constants,
                cursor: self.adjoining.cursor,
                containing_size: self.containing_size,
                child_size: logical_child_size,
                logical_margin: logical_child_margin,
                child_margin,
                child_border,
                child_padding,
                output: &output,
                clear: child_style.clear,
                avoids_float_exclusions,
                accepted_float_position,
                shared_child,
                inset_offset,
                deferred_inset,
                set_layout: self.set_layout,
            },
            &mut self.float_exclusions,
            &mut self.owned_float_block_end,
            &mut self.contributions,
        )?;

        let child_block_end = if avoids_float_exclusions {
            constants
                .flow_axes
                .logical_point(flow_location, output.size, self.containing_size)
                .block
                + logical_child_size.block
        } else {
            logical_location.block + logical_child_size.block
        };
        self.occupied_block_end = self.occupied_block_end.max(child_block_end);
        // Intrinsic demand uses the signed margin box in logical flow.
        // The separate scroll contributions retain the full physical span.
        let margin_inline_end = constants
            .flow_axes
            .logical_point(flow_location, output.size, self.containing_size)
            .inline
            + logical_child_size.inline
            + logical_child_margin.inline_end
            - constants.logical_content_box_inset().inline_start;
        let child_inline_demand = signed_inline_demand(
            logical_child_size.inline,
            logical_child_margin,
            margin_inline_end,
        );
        self.intrinsic_inline_demand = self.intrinsic_inline_demand.max(child_inline_demand);
        // Export normal baseline facts; relative movement belongs to visual
        // placement, not to the baseline used by a parent's normal line.
        self.baselines
            .resolved_child(
                output,
                child_flow_axes,
                flow_location,
                super::baseline_frame::BaselineFrame::Measured {
                    used_size: self.containing_size.map(LayoutScalar::to_f64),
                    axes: constants.flow_axes,
                },
            )
            .map_err(|error| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(child),
                    crate::LayoutOperation::ChildLayout,
                    crate::LayoutErrorKindOf::TextNumeric {
                        value: error.value(),
                        reason: error.reason(),
                    },
                )
            })?;
        margin_placement.finish(
            tree,
            child,
            &mut self.adjoining,
            &mut self.fragment_margins,
            logical_child_size.block,
            child_block_end,
        )?;
        Ok(())
    }

    pub(in crate::block) fn finish<Tree, M>(
        self,
        tree: &mut Tree,
    ) -> ComputeResult<Node, InFlowResult<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            node,
            adjoining,
            remaining_cut,
            occupied_block_end,
            intrinsic_inline_demand,
            owned_float_block_end,
            resolved_terminal_float_block_end,
            baselines,
            static_positions,
            fragment_margins,
            pending_floats,
            float_exclusions,
            inherited_floats,
            contributions,
            formatted,
            first_in_flow,
            tail_active,
            tail_ledger,
            ..
        } = self;
        if tail_active && tail_ledger.needs_continuation() {
            return Err(super::inline_run::text_run_error(
                node,
                crate::text_layout::fit::FitError::MissingBlockTail,
            )
            .into());
        }
        if let Some(text) = tree.text_computation() {
            text.formatted_record_traversal(&formatted)
                .map_err(|error| super::inline_run::text_run_error(node, error.into()))?;
        }
        let groups = tree
            .group_computation()
            .map_or_else(Vec::new, |groups| groups.end_flow());
        Ok(InFlowResult {
            fragment_margins,
            float_context: float_exclusions,
            inherited_floats,
            remaining_cut,
            groups,
            formatted,
            intrinsic_inline_demand,
            owned_float_block_end,
            resolved_terminal_float_block_end,
            contributions,
            baselines,
            static_positions,
            pending_floats,
            cursor_block: adjoining.cursor,
            occupied_block_end,
            participated: !first_in_flow,
            top_margin: adjoining.top,
            active_margin: adjoining.active,
            active_margin_at_start: adjoining.start_open,
            active_margin_can_collapse_with_parent: adjoining.end_adjoins_parent,
            all_in_flow_children_can_collapse_through: adjoining.all_through,
        })
    }
}
