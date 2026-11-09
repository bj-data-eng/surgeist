use super::*;

pub(super) struct PrefixDriver<Node, S: LayoutScalar> {
    query: EntryQuery<Node, S>,
    frames: Vec<PrefixFrame<Node, S>>,
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
        }
    }

    pub(super) fn begin<'input, Tree, M>(
        self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, EntryObservation<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        let initial =
            Progress::Descend(self.query.node, self.query.input, self.query.reporting_side);
        self.advance(tree, initial)
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
                Progress::Fold(fact) => fact,
                Progress::Descend(node, input, reporting_side) => {
                    let raw = tree.node_input(node);
                    let own = crate::FlowAxes::new(raw.writing_mode, raw.direction);
                    if tree.has_leaf_measurement(node) {
                        if self.query.purpose == EntryPurpose::FramedEntry {
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
                            let mut frame = PrefixFrame {
                                node,
                                input,
                                constants,
                                prepared_inline: false,
                                children: tree.children(node).collect(),
                                next: 0,
                                slots: if self.query.purpose == EntryPurpose::FramedEntry {
                                    match tree.text_computation() {
                                        Some(state) => state.semantic_slots(node)?,
                                        None => Vec::new(),
                                    }
                                } else {
                                    Vec::new()
                                },
                                slot_cursor: 0,
                                window: None,
                                first_in_flow: true,
                                start: CollapsibleMarginOf::ZERO,
                                through: Through::Known(CollapsibleMarginOf::ZERO),
                                possible_closed_start: None,
                                child_margin: (S::ZERO, S::ZERO),
                            };
                            match frame.step::<Tree, M>(tree, self.query.purpose)? {
                                PrefixStep::Child(child, request, side) => {
                                    self.frames.push(frame);
                                    progress = Progress::Descend(child, request, side);
                                    continue 'drive;
                                }
                                PrefixStep::Complete(fact) => fact,
                                PrefixStep::Frontier(kind) => {
                                    self.frames.push(frame);
                                    return Ok(self.suspend(kind, node, input, reporting_side));
                                }
                            }
                        }
                    }
                }
            };
            loop {
                let Some(mut frame) = self.frames.pop() else {
                    return Ok(EntryObservation::Summary(EntrySummary {
                        query: self.query,
                        start: fact.start,
                        through: fact.through,
                    }));
                };
                if !frame.accept(fact) {
                    let (node, input, side) = (
                        frame.node,
                        frame.input,
                        frame.constants.flow_axes.block_start(),
                    );
                    self.frames.push(frame);
                    return Ok(self.suspend(EntryFrontier::AmbiguousStart, node, input, side));
                }
                match frame.step::<Tree, M>(tree, self.query.purpose)? {
                    PrefixStep::Child(child, request, side) => {
                        self.frames.push(frame);
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
                        self.frames.push(frame);
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
        if self.driver.query.purpose == EntryPurpose::StartOnly
            || self.kind != EntryFrontier::NativeRun
        {
            return Ok(());
        }
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
                    state.semantic_run(self.node, segment)
                } else {
                    state.run(self.node, nodes)
                }
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
