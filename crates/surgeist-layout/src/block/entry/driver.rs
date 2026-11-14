use super::*;

pub(super) struct PrefixDriver<Node, S: LayoutScalar> {
    query: EntryQuery<Node, S>,
    frames: Vec<PrefixFrame<Node, S>>,
    offers: Vec<super::canonical::Offer<Node, S>>,
    native: Vec<NativeObservation<Node>>,
    floats: Vec<super::canonical::SourceFloat<Node, S>>,
    funding: Option<EntryFunding<S>>,
    frames_custody: EntryBacking,
    offers_custody: EntryBacking,
    native_custody: EntryBacking,
    floats_custody: EntryBacking,
}

enum Progress<Node, S: LayoutScalar> {
    Descend(Node, ComputeInputOf<S>, PhysicalSide),
    Fold(Prefix<S>),
}

impl<Node: Copy + Eq, S: LayoutScalar> PrefixDriver<Node, S> {
    pub(super) fn new(query: EntryQuery<Node, S>) -> Self {
        Self {
            query,
            frames: Vec::new(),
            offers: Vec::new(),
            native: Vec::new(),
            floats: Vec::new(),
            funding: None,
            frames_custody: EntryBacking::default(),
            offers_custody: EntryBacking::default(),
            native_custody: EntryBacking::default(),
            floats_custody: EntryBacking::default(),
        }
    }

    pub(super) fn begin<'input, Tree, M>(
        mut self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, EntryObservation<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if self.query.purpose.needs_frame()
            && let Some(state) = tree.text_computation()
        {
            self.funding = state
                .entry_funding()
                .map_err(|error| case_error(self.query.node, error))?;
        }
        let initial =
            Progress::Descend(self.query.node, self.query.input, self.query.reporting_side);
        self.advance(tree, initial)
    }

    fn push_frame<M>(&mut self, frame: PrefixFrame<Node, S>) -> LayoutResultOf<Node, (), S, M> {
        self.frames_custody
            .reserve_push(&mut self.frames, self.funding.as_ref())
            .map_err(|error| case_error(frame.node, error))?;
        self.frames.push(frame);
        Ok(())
    }

    fn suspend(
        self,
        kind: EntryFrontier,
        node: Node,
        input: ComputeInputOf<S>,
        reporting_side: PhysicalSide,
    ) -> EntryObservation<Node, S> {
        let window = self.frames.last().and_then(|frame| frame.window.clone());
        EntryObservation::Demand(EntryDemand {
            kind,
            node,
            input,
            reporting_side,
            window,
            driver: self,
            native_tail: None,
        })
    }

    fn advance<'input, Tree, M>(
        mut self,
        tree: &mut Tree,
        mut progress: Progress<Node, S>,
    ) -> LayoutResultOf<Node, EntryObservation<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        'drive: loop {
            let mut fact = match progress {
                Progress::Fold(fact) => {
                    case_work(self.funding.as_ref(), self.query.node, 1)?;
                    fact
                }
                Progress::Descend(node, input, reporting_side) => {
                    case_work(self.funding.as_ref(), node, 1)?;
                    let raw = tree.node_input(node);
                    let own = crate::FlowAxes::new(raw.writing_mode, raw.direction);
                    if tree.has_leaf_measurement(node) {
                        if self.query.purpose.needs_frame() {
                            return Ok(self.suspend(
                                EntryFrontier::Leaf,
                                node,
                                input,
                                reporting_side,
                            ));
                        }
                        Prefix {
                            start: CollapsibleMarginOf::ZERO,
                            through: Through::Unknown,
                        }
                    } else if raw.display.inner_display() != crate::Display::Block
                        || own.block_axis() != reporting_side.axis()
                    {
                        Prefix {
                            start: CollapsibleMarginOf::ZERO,
                            through: Through::Blocked,
                        }
                    } else {
                        let contained =
                            containment_sizing(node, raw, input, tree.fragment_minimum(node))?
                                .map_or(Size::NONE, |value| value.outer);
                        let style = block_container_projection::<Tree, M>(tree, node);
                        let constants = Constants::from_projection::<Tree, M>(
                            tree, node, &style, input, contained,
                        )?;
                        if own.block_start() != reporting_side {
                            if own.block_end() != reporting_side || constants.collapse_bottom_margin
                            {
                                return Ok(self.suspend(
                                    EntryFrontier::OppositeSide,
                                    node,
                                    input,
                                    reporting_side,
                                ));
                            }
                            Prefix {
                                start: constants.own_bottom_margin,
                                through: Through::Blocked,
                            }
                        } else if !constants.collapse_top_margin {
                            Prefix {
                                start: constants.own_top_margin,
                                through: Through::Blocked,
                            }
                        } else {
                            let (children_lease, children) =
                                case_children(tree, node, self.funding.as_ref())
                                    .map(|(rows, lease)| (lease, rows))?;
                            let (slots_lease, slots) = if self.query.purpose.needs_frame() {
                                match tree.text_computation() {
                                    Some(state) => state
                                        .semantic_slots(node, self.funding.as_ref())
                                        .map(|(rows, lease)| (lease, rows))?,
                                    None => (None, Vec::new()),
                                }
                            } else {
                                (None, Vec::new())
                            };
                            let mut frame = PrefixFrame {
                                node,
                                input,
                                constants,
                                prepared_inline: false,
                                children,
                                _children_lease: children_lease,
                                next: 0,
                                slots,
                                _slots_lease: slots_lease,
                                slot_cursor: 0,
                                window: None,
                                first_in_flow: true,
                                start: CollapsibleMarginOf::ZERO,
                                through: Through::Known(CollapsibleMarginOf::ZERO),
                                possible_closed_start: None,
                                child_margin: (S::ZERO, S::ZERO),
                                suppress_clear: false,
                                child_inline_margin: None,
                                inline: None,
                                stop_before: self
                                    .query
                                    .incoming
                                    .as_ref()
                                    .and_then(|incoming| incoming.cut)
                                    .filter(|cut| cut.container == node)
                                    .map(|cut| (cut.index, cut.node)),
                            };
                            if self.query.purpose == EntryPurpose::Component {
                                frame.prepare_inline::<Tree, M>(tree)?;
                                frame.inline = Some(super::canonical::InlineProjection::for_frame(
                                    &self.query,
                                    self.frames.last(),
                                    &frame,
                                )?);
                            }
                            match frame.step::<Tree, M>(
                                tree,
                                self.query.purpose,
                                self.funding.as_ref(),
                            )? {
                                PrefixStep::Child(child, request, side) => {
                                    self.push_frame(frame)?;
                                    progress = Progress::Descend(child, request, side);
                                    continue 'drive;
                                }
                                PrefixStep::Complete(fact) => fact,
                                PrefixStep::Frontier(kind) => {
                                    self.push_frame(frame)?;
                                    return Ok(self.suspend(kind, node, input, reporting_side));
                                }
                            }
                        }
                    }
                }
            };
            loop {
                let Some(mut frame) = self.frames.pop() else {
                    let mut summary = EntrySummary {
                        query: self.query,
                        start: fact.start,
                        through: fact.through,
                        separation: None,
                        native: self.native,
                        floats: self.floats,
                        funding: self.funding,
                        _native_custody: self.native_custody,
                        _float_custody: self.floats_custody,
                    };
                    if summary.query.purpose == EntryPurpose::Component {
                        super::canonical::resolve(&mut summary, self.offers, self.offers_custody)?;
                    }
                    return Ok(EntryObservation::Summary(summary));
                };
                case_work(self.funding.as_ref(), frame.node, 1)?;
                if !frame.accept(fact) {
                    let (node, input, side) = (
                        frame.node,
                        frame.input,
                        frame.constants.flow_axes.block_start(),
                    );
                    self.push_frame(frame)?;
                    return Ok(self.suspend(EntryFrontier::AmbiguousStart, node, input, side));
                }
                match frame.step::<Tree, M>(tree, self.query.purpose, self.funding.as_ref())? {
                    PrefixStep::Child(child, request, side) => {
                        self.push_frame(frame)?;
                        progress = Progress::Descend(child, request, side);
                        continue 'drive;
                    }
                    PrefixStep::Complete(result) => fact = result,
                    PrefixStep::Frontier(kind) => {
                        let (node, input, side) = (
                            frame.node,
                            frame.input,
                            frame.constants.flow_axes.block_start(),
                        );
                        self.push_frame(frame)?;
                        return Ok(self.suspend(kind, node, input, side));
                    }
                }
            }
        }
    }
}

impl<Node: Copy + Eq, S: LayoutScalar> EntryDemand<Node, S> {
    /// A native suspension transports the same admitted source interval as the
    /// real dispatcher. A continuation cannot substitute an equal-sized window.
    pub(crate) fn validate_window<'input, Tree, M>(
        &self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if !self.driver.query.purpose.needs_frame() || self.kind != EntryFrontier::NativeRun {
            return Ok(());
        }
        case_work(self.driver.funding.as_ref(), self.node, 1)?;
        let frame = self
            .driver
            .frames
            .last()
            .ok_or_else(|| float_context::missing(self.node))?;
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| float_context::missing(self.node))?;
        let nodes = frame
            .children
            .get(window.nodes.clone())
            .ok_or_else(|| float_context::missing(self.node))?;
        match &window.segment {
            Some(segment) => {
                let state = tree
                    .text_computation()
                    .ok_or_else(|| float_context::missing(self.node))?;
                let run = if nodes.is_empty() {
                    state.entry_semantic_run(self.node, segment, self.driver.funding.as_ref())
                } else {
                    state.entry_run(self.node, nodes, self.driver.funding.as_ref())
                }
                .map_err(|error| case_error(self.node, error))?
                .ok_or_else(|| float_context::missing(self.node))?;
                if run.segment != *segment {
                    return Err(float_context::missing(self.node));
                }
            }
            None => {
                let first = nodes
                    .first()
                    .copied()
                    .ok_or_else(|| float_context::missing(self.node))?;
                if !crate::inline::starts_run(tree.layout_input(first)) {
                    return Err(float_context::missing(self.node));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn is_component(&self) -> bool {
        self.driver.query.purpose == EntryPurpose::Component
    }

    pub(crate) fn prepare_float<'input, Tree, M>(
        mut self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, EntryFloat<'input, Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if self.kind != EntryFrontier::Float || !self.is_component() {
            return Err(float_context::missing(self.node));
        }
        self.driver
            .floats_custody
            .reserve_push(&mut self.driver.floats, self.driver.funding.as_ref())
            .map_err(|error| case_error(self.node, error))?;
        let frame = self
            .driver
            .frames
            .last_mut()
            .ok_or_else(|| float_context::missing(self.node))?;
        frame.prepare_inline::<Tree, M>(tree)?;
        let index = frame
            .next
            .checked_sub(1)
            .ok_or_else(|| float_context::missing(self.node))?;
        let node = frame.children[index];
        let style = BlockChildProjection::from_node(tree.node_input(node));
        let prepared = in_flow::prepare_child(
            tree,
            node,
            &style,
            &frame.constants,
            in_flow::ChildSizingContext {
                inner_size: frame.constants.node_inner_size,
                request: frame.constants.sizing_request,
                run_mode: RunMode::ComputeSize,
                axis: RequestedAxis::Both,
                first_in_flow: &mut frame.first_in_flow,
            },
        )?;
        Ok(EntryFloat {
            demand: self,
            node,
            index,
            style,
            prepared,
        })
    }

    pub(crate) fn observe_native<'input, Tree, M>(
        mut self,
        tree: &mut Tree,
    ) -> EntryResult<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if self.kind != EntryFrontier::NativeRun || !self.is_component() {
            return Err(float_context::missing(self.node));
        }
        case_work(self.driver.funding.as_ref(), self.node, 1)?;
        let frame = self
            .driver
            .frames
            .last()
            .ok_or_else(|| float_context::missing(self.node))?;
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| float_context::missing(self.node))?;
        let nodes = frame
            .children
            .get(window.nodes.clone())
            .ok_or_else(|| float_context::missing(self.node))?;
        let separates = if window.segment.is_some() {
            self.driver
                .native_custody
                .reserve_push(&mut self.driver.native, self.driver.funding.as_ref())
                .map_err(|error| case_error(self.node, error))?;
            let proof = super::semantic::observe(
                tree,
                self.node,
                window,
                nodes,
                &frame.constants,
                self.driver.funding.as_ref(),
            )?;
            let separates = super::semantic::separates(&proof, self.driver.funding.as_ref())
                .map_err(|error| case_error(self.node, error))?;
            self.driver.native.push(NativeObservation {
                container: self.node,
                window: window.clone(),
                proof,
            });
            separates
        } else {
            let mut next = nodes.iter();
            let mut failure = None;
            let participation = crate::inline::entry_participation(std::iter::from_fn(|| {
                let &node = next.next()?;
                if let Err(error) = case_work(self.driver.funding.as_ref(), node, 1) {
                    failure = Some(error);
                    return None;
                }
                Some(tree.layout_input(node))
            }));
            if let Some(error) = failure {
                return Err(error);
            }
            participation.ok_or_else(|| float_context::missing(self.node))?
        };
        let window = window.clone();
        let frame = self
            .driver
            .frames
            .last_mut()
            .expect("native window retains its owning frame");
        if separates {
            frame.through = Through::Blocked;
            frame.next = window.nodes.end;
            frame.window = None;
            self.window = None;
            self.resume_frame(tree)
        } else {
            frame.next = window.nodes.start;
            self.native_tail = Some(window);
            self.resume_native_tail(tree)
        }
    }

    fn resume_native_tail<'input, Tree, M>(mut self, tree: &mut Tree) -> EntryResult<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        let window = self
            .native_tail
            .as_ref()
            .ok_or_else(|| float_context::missing(self.node))?;
        let frame = self
            .driver
            .frames
            .last_mut()
            .ok_or_else(|| float_context::missing(self.node))?;
        while frame.next < window.nodes.end {
            case_work(self.driver.funding.as_ref(), self.node, 1)?;
            let node = frame.children[frame.next];
            frame.next += 1;
            if let crate::LayoutInputViewOf::Box(style) = tree.layout_input(node)
                && style.display != crate::Display::None
                && style.position != Position::Absolute
                && !style.float.is_none()
            {
                self.kind = EntryFrontier::Float;
                return Ok(EntryObservation::Demand(self));
            }
        }
        frame.window = None;
        self.window = None;
        self.native_tail = None;
        self.resume_frame(tree)
    }

    pub(crate) fn suppress_clear<'input, Tree, M>(
        mut self,
        tree: &mut Tree,
    ) -> EntryResult<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if self.kind != EntryFrontier::Clearance
            || self.driver.query.purpose != EntryPurpose::Component
        {
            return Err(float_context::missing(self.node));
        }
        self.driver
            .offers_custody
            .reserve_push(&mut self.driver.offers, self.driver.funding.as_ref())
            .map_err(|error| case_error(self.node, error))?;
        let frame = self
            .driver
            .frames
            .last()
            .ok_or_else(|| float_context::missing(self.node))?;
        let index = frame
            .next
            .checked_sub(1)
            .ok_or_else(|| float_context::missing(self.node))?;
        let child = frame.children[index];
        let clear = tree.node_input(child).clear;
        let axes = frame.constants.flow_axes;
        let mut before = frame.start;
        for ancestor in self.driver.frames[..self.driver.frames.len() - 1]
            .iter()
            .rev()
        {
            case_work(self.driver.funding.as_ref(), ancestor.node, 1)?;
            before = ancestor
                .start
                .collapse_with(before)
                .collapse_with_margin(ancestor.child_margin.0);
        }
        self.driver.offers.push(super::canonical::Offer {
            container: frame.node,
            node: child,
            index,
            before,
            clear,
            axes,
            floats_before: self.driver.floats.len(),
        });
        let frame = self
            .driver
            .frames
            .last_mut()
            .expect("retained current source frame");
        frame.next = index;
        frame.suppress_clear = true;
        self.resume_frame(tree)
    }

    fn resume_frame<'input, Tree, M>(mut self, tree: &mut Tree) -> EntryResult<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        let mut frame = self
            .driver
            .frames
            .pop()
            .ok_or_else(|| float_context::missing(self.node))?;
        case_work(self.driver.funding.as_ref(), frame.node, 1)?;
        match frame.step::<Tree, M>(
            tree,
            self.driver.query.purpose,
            self.driver.funding.as_ref(),
        )? {
            PrefixStep::Child(child, input, side) => {
                self.driver.push_frame(frame)?;
                self.driver
                    .advance(tree, Progress::Descend(child, input, side))
            }
            PrefixStep::Complete(fact) => self.driver.advance(tree, Progress::Fold(fact)),
            PrefixStep::Frontier(kind) => {
                let node = frame.node;
                let input = frame.input;
                let side = frame.constants.flow_axes.block_start();
                self.driver.push_frame(frame)?;
                Ok(self.driver.suspend(kind, node, input, side))
            }
        }
    }

    pub(crate) fn measure_leaf<'input, Tree, M>(
        self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, EntryObservation<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if self.kind != EntryFrontier::Leaf || !tree.has_leaf_measurement(self.node) {
            return Err(float_context::missing(self.node));
        }
        case_work(self.driver.funding.as_ref(), self.node, 1)?;
        let output = tree.measure_entry_leaf(self.node, self.input)?;
        let own = tree.node_input(self.node);
        let axes = crate::FlowAxes::new(own.writing_mode, own.direction);
        let start = output.block_margin_collapse.at(self.reporting_side);
        let through = if axes.block_axis() == self.reporting_side.axis()
            && output.block_margin_collapse.can_collapse_through(axes)
        {
            Through::Known(
                start.collapse_with(
                    output
                        .block_margin_collapse
                        .at(self.reporting_side.opposite()),
                ),
            )
        } else {
            Through::Blocked
        };
        self.driver
            .advance(tree, Progress::Fold(Prefix { start, through }))
    }
}

#[cfg(test)]
#[path = "driver_tests.rs"]
mod tests;

pub(crate) struct EntryFloat<'input, Node, S: LayoutScalar> {
    demand: EntryDemand<Node, S>,
    node: Node,
    index: usize,
    style: BlockChildProjection<'input, S>,
    prepared: in_flow::PreparedChild<S>,
}

impl<'input, Node: Copy + Eq + 'input, S: LayoutScalar> EntryFloat<'input, Node, S> {
    pub(crate) fn request(&self) -> (Node, ComputeInputOf<S>) {
        (self.node, self.prepared.input)
    }

    pub(crate) fn complete<Tree, M>(
        self,
        tree: &mut Tree,
        output: ComputeOutputOf<S>,
    ) -> EntryResult<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            mut demand,
            node,
            index,
            style,
            prepared,
        } = self;
        let frame = demand
            .driver
            .frames
            .last_mut()
            .ok_or_else(|| float_context::missing(demand.node))?;
        let margin = in_flow::resolve_logical_in_flow_margin(
            frame
                .constants
                .flow_axes
                .logical_edges(prepared.unresolved_margin),
            frame.constants.flow_axes.logical_size(output.size),
            frame.constants.logical_node_inner_size().inline,
        );
        let pending = PendingFloat {
            placement: super::super::floats::PendingPlacement::Unresolved,
            node,
            source_index: index,
            side: style.float,
            clear: style.clear,
            block_start: S::ZERO,
            size: output.size,
            named_baselines: output.named_baselines,
            content_size: output.content_size,
            border: prepared.border,
            padding: prepared.padding,
            margin: frame.constants.flow_axes.physical_edges(margin),
            float_exclusion: style.float_exclusion,
            child_compute_geometry: output.scroll_geometry,
        };
        let inline = frame
            .inline
            .ok_or_else(|| float_context::missing(demand.node))?;
        demand
            .driver
            .floats
            .push(super::canonical::SourceFloat::new(
                pending,
                inline,
                &frame.constants,
            ));
        if demand.native_tail.is_some() {
            demand.resume_native_tail(tree)
        } else {
            demand.resume_frame(tree)
        }
    }
}
