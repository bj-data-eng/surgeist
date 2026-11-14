//! Entry facts that do not require the completed descendant body.
use super::*;
use crate::text_layout::scope_alignment::{EntryBacking, EntryFunding, ScopeLease};

mod adjoining;
pub(super) use adjoining::AdjoiningFlow;
pub(crate) mod canonical;
mod driver;
mod semantic;
pub(crate) use driver::EntryFloat;
use driver::PrefixDriver;

#[cfg(test)]
mod settlement_tests;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StartFact<S: LayoutScalar> {
    pub(crate) margin: CollapsibleMarginOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Qualification<S: LayoutScalar> {
    Known(StartFact<S>),
    RequiresBody,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EntryPurpose {
    StartOnly,
    FramedEntry,
    Component,
}
impl EntryPurpose {
    fn needs_frame(self) -> bool {
        self == Self::FramedEntry || self == Self::Component
    }
}

pub(crate) struct EntryQuery<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    pub(crate) reporting_side: PhysicalSide,
    pub(crate) purpose: EntryPurpose,
    pub(crate) incoming: Option<float_context::Incoming<Node, S>>,
    pub(crate) position: Option<canonical::EntryPosition<S>>,
}

/// A completed entry component, not a completed descendant layout.
pub(crate) struct EntrySummary<Node, S: LayoutScalar> {
    pub(crate) query: EntryQuery<Node, S>,
    start: CollapsibleMarginOf<S>,
    through: Through<S>,
    pub(crate) separation: Option<canonical::Separation<Node, S>>,
    native: Vec<NativeObservation<Node>>,
    floats: Vec<canonical::SourceFloat<Node, S>>,
    funding: Option<EntryFunding<S>>,
    _native_custody: EntryBacking,
    _float_custody: EntryBacking,
}

impl<Node, S: LayoutScalar> EntrySummary<Node, S> {
    pub(crate) fn start_fact(&self) -> StartFact<S> {
        StartFact {
            margin: match self.through {
                Through::Known(group) => self.start.collapse_with(group),
                Through::Blocked | Through::Unknown => self.start,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EntryFrontier {
    Leaf,
    NativeRun,
    Float,
    Clearance,
    OppositeSide,
    AmbiguousStart,
}

/// Owns the suspended prefix stack and its exact next producer request.
/// No output, fragment or completed float handoff can escape this boundary.
pub(crate) struct EntryDemand<Node, S: LayoutScalar> {
    pub(crate) kind: EntryFrontier,
    pub(crate) node: Node,
    pub(crate) input: ComputeInputOf<S>,
    pub(crate) reporting_side: PhysicalSide,
    pub(crate) window: Option<EntryWindow>,
    driver: PrefixDriver<Node, S>,
    native_tail: Option<EntryWindow>,
}

#[derive(Clone)]
pub(crate) struct EntryWindow {
    pub(crate) nodes: std::ops::Range<usize>,
    pub(crate) segment: Option<surgeist_text::PreparedSourceSegment>,
}

pub(crate) type EntryResult<Node, S, M> = LayoutResultOf<Node, EntryObservation<Node, S>, S, M>;

struct NativeObservation<Node> {
    container: Node,
    window: EntryWindow,
    proof: crate::text_layout::scope_alignment::SemanticWindowProof,
}

pub(crate) enum EntryObservation<Node, S: LayoutScalar> {
    Summary(EntrySummary<Node, S>),
    Demand(EntryDemand<Node, S>),
}

pub(super) struct StartReduction<S: LayoutScalar> {
    pub(super) offset: S,
    pub(super) exposes_child: bool,
}

/// An exposed leading component is already outside the parent's content.
/// Placement, preview and independently scheduled scopes share this reduction.
pub(super) fn reduce_start<S: LayoutScalar>(
    first: bool,
    collapses_start: bool,
    child_collapses: bool,
    active: CollapsibleMarginOf<S>,
    child_start: CollapsibleMarginOf<S>,
) -> StartReduction<S> {
    let exposes_child = first && collapses_start && child_collapses;
    StartReduction {
        offset: if exposes_child {
            S::ZERO
        } else {
            active.collapse_with(child_start).resolve()
        },
        exposes_child,
    }
}

#[derive(Clone, Copy)]
enum Through<S: LayoutScalar> {
    Blocked,
    Known(CollapsibleMarginOf<S>),
    Unknown,
}

#[derive(Clone, Copy)]
struct Prefix<S: LayoutScalar> {
    start: CollapsibleMarginOf<S>,
    through: Through<S>,
}

struct PrefixFrame<Node, S: LayoutScalar> {
    node: Node,
    input: ComputeInputOf<S>,
    constants: Constants<S>,
    prepared_inline: bool,
    children: Vec<Node>,
    _children_lease: Option<ScopeLease>,
    next: usize,
    slots: Vec<crate::text_layout::capture::NativeRunSlot>,
    _slots_lease: Option<ScopeLease>,
    slot_cursor: usize,
    window: Option<EntryWindow>,
    first_in_flow: bool,
    start: CollapsibleMarginOf<S>,
    through: Through<S>,
    possible_closed_start: Option<CollapsibleMarginOf<S>>,
    child_margin: (S, S),
    suppress_clear: bool,
    child_inline_margin: Option<LogicalEdgesOf<ResolvedLengthAutoOf<S>>>,
    inline: Option<canonical::InlineProjection<S>>,
    stop_before: Option<(usize, Node)>,
}

enum PrefixStep<Node, S: LayoutScalar> {
    Child(Node, ComputeInputOf<S>, PhysicalSide),
    Complete(Prefix<S>),
    Frontier(EntryFrontier),
}

fn case_error<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    error: surgeist_text::Error,
) -> crate::LayoutErrorOf<Node, S, M> {
    super::inline_run::text_run_error(node, error.into())
}

fn case_work<Node: Copy, S: LayoutScalar, M>(
    funding: Option<&EntryFunding<S>>,
    node: Node,
    count: usize,
) -> LayoutResultOf<Node, (), S, M> {
    funding
        .map_or(Ok(()), |funding| funding.visit(count))
        .map_err(|error| case_error(node, error))
}

type CaseChildren<Node> = (Vec<Node>, Option<ScopeLease>);

fn case_children<'input, Tree, M>(
    tree: &Tree,
    node: Tree::Node,
    funding: Option<&EntryFunding<Tree::Scalar>>,
) -> LayoutResultOf<Tree::Node, CaseChildren<Tree::Node>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let Some(funding) = funding else {
        return Ok((tree.children(node).collect(), None));
    };
    let count = tree.child_count(node);
    if count == 0 {
        return Ok((Vec::new(), None));
    }
    let (lease, mut children) = funding
        .buffer(count)
        .map(|(rows, lease)| (lease, rows))
        .map_err(|error| case_error(node, error))?;
    for child in tree.children(node) {
        funding.visit(1).map_err(|error| case_error(node, error))?;
        if children.len() == count {
            return Err(float_context::missing(node));
        }
        children.push(child);
    }
    if children.len() != count {
        return Err(float_context::missing(node));
    }
    Ok((children, Some(lease)))
}

impl<Node: Copy + Eq, S: LayoutScalar> PrefixFrame<Node, S> {
    fn accept(&mut self, child: Prefix<S>) -> bool {
        self.start = self
            .start
            .collapse_with(child.start)
            .collapse_with_margin(self.child_margin.0);
        if matches!(child.through, Through::Unknown) {
            self.possible_closed_start.get_or_insert(self.start);
        }
        self.through = match child.through {
            Through::Known(group) => {
                self.start = self
                    .start
                    .collapse_with(group)
                    .collapse_with_margin(self.child_margin.1);
                Through::Known(self.start)
            }
            Through::Unknown => {
                self.start = self.start.collapse_with_margin(self.child_margin.1);
                Through::Unknown
            }
            through => through,
        };
        // An opaque body may stop the component at its own start. Certify a
        // start only while every possible continuation retains those extrema.
        self.possible_closed_start
            .is_none_or(|closed| closed == self.start)
    }

    fn prepare_inline<'input, Tree, M>(&mut self, tree: &mut Tree) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if self.prepared_inline {
            return Ok(());
        }
        let container = block_container_projection::<Tree, M>(tree, self.node);
        let outer = intrinsic::accepted_inline(
            tree,
            self.node,
            &container,
            &self.constants,
            self.input,
            |tree, op| {
                if let Some(value) = tree.fragment_inline_content(self.node, self.input, op)? {
                    Ok(value)
                } else {
                    intrinsic::content(tree, self.node, &self.constants, self.input, op)
                }
            },
        )?;
        let inner = self.constants.accepted_inline_inner(self.node, outer)?;
        self.constants = self.constants.with_logical_node_inner_size(
            LogicalSizeOf::new(Some(inner), self.constants.logical_node_inner_size().block),
            false,
        );
        self.prepared_inline = true;

        Ok(())
    }

    fn step<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        purpose: EntryPurpose,
        funding: Option<&EntryFunding<S>>,
    ) -> LayoutResultOf<Node, PrefixStep<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if !matches!(self.through, Through::Blocked) {
            case_work(funding, self.node, 1)?;
            if let Some(slot) = self
                .slots
                .get(self.slot_cursor)
                .filter(|slot| slot.gap == self.next)
            {
                self.slot_cursor += 1;
                self.window = Some(EntryWindow {
                    nodes: self.next..self.next,
                    segment: Some(slot.segment.clone()),
                });
                return Ok(PrefixStep::Frontier(EntryFrontier::NativeRun));
            }
            while let Some(&child) = self.children.get(self.next) {
                case_work(funding, child, 1)?;
                if self.stop_before == Some((self.next, child)) {
                    self.through = Through::Blocked;
                    break;
                }
                let start = self.next;
                self.next += 1;
                if purpose.needs_frame() {
                    let captured = match tree.text_computation() {
                        Some(state) => state
                            .entry_run_starting(self.node, &self.children[start..], funding)
                            .map_err(|error| case_error(self.node, error))?
                            .map(|run| run.segment.clone()),
                        None => None,
                    };
                    if captured.is_some() || crate::inline::starts_run(tree.layout_input(child)) {
                        let end = super::inline_run::inline_run_end(
                            tree,
                            self.node,
                            &self.children,
                            &self.constants,
                            self.next,
                            self.slots
                                .get(self.slot_cursor)
                                .map_or(self.children.len(), |slot| slot.gap),
                            funding,
                        )?;
                        self.window = Some(EntryWindow {
                            nodes: start..end,
                            segment: captured,
                        });
                        return Ok(PrefixStep::Frontier(EntryFrontier::NativeRun));
                    }
                }
                let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
                if purpose.needs_frame()
                    && matches!(projection.kind(), InlineParticipantKindOf::LineBreak(value) if value.display().is_none())
                {
                    continue;
                }
                let InlineParticipantKindOf::Box(style) = projection.kind() else {
                    return Ok(PrefixStep::Frontier(EntryFrontier::NativeRun));
                };
                if style.display == crate::Display::None
                    || style.common.position == Position::Absolute
                {
                    continue;
                }
                if !style.float.is_none() {
                    return Ok(PrefixStep::Frontier(EntryFrontier::Float));
                }
                if style.display.is_inline_level() {
                    return Ok(PrefixStep::Frontier(EntryFrontier::NativeRun));
                }
                if purpose.needs_frame() && style.clear != crate::Clear::None {
                    if self.suppress_clear {
                        self.suppress_clear = false;
                    } else {
                        return Ok(PrefixStep::Frontier(EntryFrontier::Clearance));
                    }
                }
                if !in_flow::child_margin_can_collapse_with_parent(&style) {
                    self.through = Through::Blocked;
                    break;
                }
                self.prepare_inline::<Tree, M>(tree)?;
                let prepared = in_flow::prepare_child(
                    tree,
                    child,
                    &style,
                    &self.constants,
                    in_flow::ChildSizingContext {
                        inner_size: self.constants.node_inner_size,
                        request: self.constants.sizing_request,
                        run_mode: RunMode::ComputeSize,
                        axis: RequestedAxis::Both,
                        first_in_flow: &mut self.first_in_flow,
                    },
                )?;
                let margin = self
                    .constants
                    .flow_axes
                    .logical_edges(prepared.unresolved_margin);
                self.child_inline_margin = Some(margin);
                self.child_margin = (
                    in_flow::resolved_length_auto_fallback_zero(margin.block_start),
                    in_flow::resolved_length_auto_fallback_zero(margin.block_end),
                );
                return Ok(PrefixStep::Child(
                    child,
                    prepared.input,
                    self.constants.flow_axes.block_start(),
                ));
            }
        }
        Ok(PrefixStep::Complete(Prefix {
            start: self.start,
            through: if self.constants.can_collapse_through {
                if self.possible_closed_start.is_some() && !matches!(self.through, Through::Blocked)
                {
                    Through::Unknown
                } else {
                    self.through
                }
            } else {
                Through::Blocked
            },
        }))
    }
}

pub(crate) fn observe<'input, Tree, M>(
    tree: &mut Tree,
    query: EntryQuery<Tree::Node, Tree::Scalar>,
) -> EntryResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let observation = PrefixDriver::new(query).begin(tree)?;
    if let EntryObservation::Demand(demand) = &observation {
        demand.validate_window(tree)?;
    }
    Ok(observation)
}

/// Resolve frontiers only after the executor installs disposable resident
/// custody. Both native private executors and the publishing session use this
/// same partial driver; their factories own output isolation.
type FramedEntryStep<'input, Node, S, M> =
    LayoutResultOf<Node, ControlFlow<EntryObservation<Node, S>, EntryFloat<'input, Node, S>>, S, M>;

/// Owns one framed prefix operation across actual source-float measurements.
pub(crate) struct FramedEntry<Node, S: LayoutScalar> {
    observation: EntryObservation<Node, S>,
}
impl<Node: Copy + Eq, S: LayoutScalar> FramedEntry<Node, S> {
    pub(crate) fn begin<'input, Tree, M>(
        tree: &mut Tree,
        query: EntryQuery<Node, S>,
    ) -> LayoutResultOf<Node, Self, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        let expected = query
            .incoming
            .as_ref()
            .ok_or_else(|| float_context::missing(query.node))?;
        let resident = tree
            .block_float_state()
            .and_then(|state| state.incoming(query.node, query.input))
            .ok_or_else(|| float_context::missing(query.node))?;
        if expected.node != query.node
            || expected.request != query.input
            || !std::sync::Arc::ptr_eq(&expected.frame, &resident.frame)
            || match (expected.cut, resident.cut) {
                (Some(expected), Some(actual)) => !expected.same_cut(actual),
                (None, None) => false,
                _ => true,
            }
        {
            return Err(float_context::missing(query.node));
        }
        Ok(Self {
            observation: observe(tree, query)?,
        })
    }

    pub(crate) fn after_float<'input, Tree, M>(
        tree: &mut Tree,
        pending: EntryFloat<'input, Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, Self, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        let observation = pending.complete(tree, output)?;
        if let EntryObservation::Demand(demand) = &observation {
            demand.validate_window(tree)?;
        }
        Ok(Self { observation })
    }

    pub(crate) fn advance<'input, Tree, M>(
        self,
        tree: &mut Tree,
    ) -> FramedEntryStep<'input, Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        let mut observation = self.observation;
        loop {
            observation = match observation {
                EntryObservation::Demand(demand) if demand.kind == EntryFrontier::Leaf => {
                    demand.measure_leaf(tree)?
                }
                EntryObservation::Demand(demand)
                    if demand.kind == EntryFrontier::Clearance && demand.is_component() =>
                {
                    demand.suppress_clear(tree)?
                }
                EntryObservation::Demand(demand)
                    if demand.kind == EntryFrontier::NativeRun && demand.is_component() =>
                {
                    demand.observe_native(tree)?
                }
                EntryObservation::Demand(demand)
                    if demand.kind == EntryFrontier::Float && demand.is_component() =>
                {
                    return Ok(ControlFlow::Continue(demand.prepare_float(tree)?));
                }
                EntryObservation::Summary(summary) => {
                    summary.validate_separation(tree)?;
                    return Ok(ControlFlow::Break(EntryObservation::Summary(summary)));
                }
                ready => return Ok(ControlFlow::Break(ready)),
            };
            if let EntryObservation::Demand(demand) = &observation {
                demand.validate_window(tree)?;
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn resolve_framed<'input, Tree, M>(
    tree: &mut Tree,
    query: EntryQuery<Tree::Node, Tree::Scalar>,
) -> EntryResult<Tree::Node, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut state = FramedEntry::begin(tree, query)?;
    loop {
        match state.advance(tree)? {
            ControlFlow::Break(observation) => return Ok(observation),
            ControlFlow::Continue(pending) => {
                let (node, input) = pending.request();
                let output = tree.measure_entry_float(node, input)?;
                state = FramedEntry::after_float(tree, pending, output)?;
            }
        }
    }
}

pub(crate) fn qualify<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    reporting_side: PhysicalSide,
) -> LayoutResultOf<Tree::Node, Qualification<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let observation = tree.qualify_entry(EntryQuery {
        node,
        input,
        reporting_side,
        purpose: EntryPurpose::StartOnly,
        incoming: None,
        position: None,
    })?;
    Ok(match observation {
        EntryObservation::Summary(summary) => {
            if summary.query.node != node
                || summary.query.input != input
                || summary.query.reporting_side != reporting_side
            {
                return Err(float_context::missing(node));
            }
            Qualification::Known(summary.start_fact())
        }
        EntryObservation::Demand(_) => Qualification::RequiresBody,
    })
}

#[cfg(test)]
#[path = "entry_tests.rs"]
mod tests;
