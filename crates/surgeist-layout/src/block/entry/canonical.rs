//! Source-cut normalization of one still-open adjoining entry component.
use super::*;

/// The actual receiving parent's unconsumed component. This does not describe
/// a guessed position of the queried child or advance the parent's cursor.
#[derive(Clone, Copy)]
pub(crate) struct EntryPosition<S: LayoutScalar> {
    pub(crate) cursor: S,
    pub(crate) active: CollapsibleMarginOf<S>,
    pub(crate) start_open: bool,
    pub(crate) collapses_start: bool,
    pub(crate) child_adjoins_parent: bool,
    pub(crate) own_start: CollapsibleMarginOf<S>,
    pub(crate) inline_offset: S,
}
impl<S: LayoutScalar> EntryPosition<S> {
    pub(crate) fn hypothetical(self, exported: CollapsibleMarginOf<S>) -> S {
        self.cursor
            + reduce_start(
                self.start_open,
                self.collapses_start,
                self.child_adjoins_parent,
                self.active,
                self.own_start.collapse_with(exported),
            )
            .offset
    }
}

#[derive(Clone, Copy)]
pub(super) struct InlineProjection<S: LayoutScalar> {
    anchor: S,
    reversed: bool,
}
impl<S: LayoutScalar> InlineProjection<S> {
    pub(super) fn for_frame<Node: Copy, M>(
        query: &EntryQuery<Node, S>,
        parent: Option<&PrefixFrame<Node, S>>,
        frame: &PrefixFrame<Node, S>,
    ) -> LayoutResultOf<Node, Self, S, M> {
        let outer = frame
            .constants
            .logical_node_inner_size()
            .inline
            .ok_or_else(|| float_context::missing(frame.node))?
            + frame.constants.logical_content_box_inset().inline_sum();
        let (parent_projection, parent_axes, offset) = if let Some(parent) = parent {
            let margin = in_flow::resolve_logical_inline_margin(
                parent
                    .child_inline_margin
                    .ok_or_else(|| float_context::missing(frame.node))?,
                outer,
                parent.constants.logical_node_inner_size().inline,
            );
            (
                parent
                    .inline
                    .ok_or_else(|| float_context::missing(frame.node))?,
                parent.constants.flow_axes,
                in_flow::in_flow_child_inline_offset(outer, margin, &parent.constants),
            )
        } else {
            let incoming = query
                .incoming
                .as_ref()
                .ok_or_else(|| float_context::missing(frame.node))?;
            (
                Self {
                    anchor: S::ZERO,
                    reversed: false,
                },
                incoming.frame.parent_axes(),
                query
                    .position
                    .ok_or_else(|| float_context::missing(frame.node))?
                    .inline_offset,
            )
        };
        let reverse_child = frame.constants.flow_axes.inline_start() != parent_axes.inline_start();
        let distance = offset + if reverse_child { outer } else { S::ZERO };
        Ok(Self {
            anchor: if parent_projection.reversed {
                parent_projection.anchor - distance
            } else {
                parent_projection.anchor + distance
            },
            reversed: parent_projection.reversed != reverse_child,
        })
    }
    fn point(self, offset: S) -> S {
        if self.reversed {
            self.anchor - offset
        } else {
            self.anchor + offset
        }
    }
}

pub(super) struct SourceFloat<Node, S: LayoutScalar> {
    pending: PendingFloat<Node, S>,
    inline_start: S,
    inline_end: S,
}
impl<Node: Copy, S: LayoutScalar> SourceFloat<Node, S> {
    pub(super) fn new(
        mut pending: PendingFloat<Node, S>,
        inline: InlineProjection<S>,
        constants: &Constants<S>,
    ) -> Self {
        if inline.reversed {
            pending.side = match pending.side {
                crate::Float::Left => crate::Float::Right,
                crate::Float::Right => crate::Float::Left,
                value => value,
            };
            pending.clear = match pending.clear {
                crate::Clear::Left => crate::Clear::Right,
                crate::Clear::Right => crate::Clear::Left,
                value => value,
            };
        }
        let start = constants.logical_content_box_inset().inline_start;
        let end = start
            + constants
                .logical_node_inner_size()
                .inline
                .expect("accepted entry inline extent");
        let a = inline.point(start);
        let b = inline.point(end);
        Self {
            pending,
            inline_start: a.min(b),
            inline_end: a.max(b),
        }
    }
}

fn projected_bound<Node: Copy, S: LayoutScalar, M>(
    summary: &EntrySummary<Node, S>,
    support: CollapsibleMarginOf<S>,
    count: usize,
    clear: crate::Clear,
    axes: crate::FlowAxes,
) -> LayoutResultOf<Node, Option<super::super::floats::CommonBfcPosition<S>>, S, M> {
    let incoming = summary
        .query
        .incoming
        .as_ref()
        .ok_or_else(|| float_context::missing(summary.query.node))?;
    let position = summary
        .query
        .position
        .ok_or_else(|| float_context::missing(summary.query.node))?;
    let floats = summary
        .floats
        .get(..count)
        .ok_or_else(|| float_context::missing(summary.query.node))?;
    let Some(first) = floats.first() else {
        return incoming.frame.entry_relevant_bound::<M>(
            summary.query.node,
            clear,
            axes,
            summary.funding.as_ref(),
        );
    };
    let (_ledger_lease, mut ledger) = incoming
        .frame
        .entry_ledger_funded(
            first.inline_start,
            first.inline_end,
            count,
            summary.funding.as_ref(),
        )
        .map(|(ledger, lease)| (lease, ledger))
        .map_err(|error| case_error(summary.query.node, error))?;
    for source in floats {
        case_work(summary.funding.as_ref(), source.pending.node, 1)?;
        ledger.entry_inline_window(source.inline_start, source.inline_end);
        let mut pending = source.pending.clone();
        pending.block_start = position.hypothetical(support);
        ledger.entry_place_float::<M>(summary.query.node, &pending, summary.funding.as_ref())?;
    }
    ledger.entry_relevant_bound::<M>(summary.query.node, clear, axes, summary.funding.as_ref())
}

/// One offered cut before its own top group, in the original source inventory.
/// The prefix extrema come from the shared driver, not a completed child body.
pub(super) struct Offer<Node, S: LayoutScalar> {
    pub(super) container: Node,
    pub(super) node: Node,
    pub(super) index: usize,
    pub(super) before: CollapsibleMarginOf<S>,
    pub(super) clear: crate::Clear,
    pub(super) axes: crate::FlowAxes,
    pub(super) floats_before: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct Separation<Node, S: LayoutScalar> {
    pub(crate) container: Node,
    pub(crate) node: Node,
    pub(crate) index: usize,
    pub(crate) hypothetical: super::super::floats::CommonBfcPosition<S>,
    pub(crate) relevant_bottom: super::super::floats::CommonBfcPosition<S>,
    support: CollapsibleMarginOf<S>,
    floats_before: usize,
}
impl<Node: Copy + Eq, S: LayoutScalar> Separation<Node, S> {
    pub(crate) fn same_cut(self, other: Self) -> bool {
        self.container == other.container
            && self.node == other.node
            && self.index == other.index
            && self.hypothetical == other.hypothetical
            && self.relevant_bottom == other.relevant_bottom
    }
    pub(crate) fn own(
        container: Node,
        node: Node,
        index: usize,
        hypothetical: super::super::floats::CommonBfcPosition<S>,
        relevant_bottom: super::super::floats::CommonBfcPosition<S>,
    ) -> Self {
        Self {
            container,
            node,
            index,
            hypothetical,
            relevant_bottom,
            support: CollapsibleMarginOf::ZERO,
            floats_before: 0,
        }
    }
}

/// No suffix is rebuilt after an introduced offer. Its source cut becomes the
/// next earlier demand's support; later hypothetical decisions are discarded.
/// Every step consumes one earlier source offer, so there is no entry wait cycle.
pub(super) fn resolve<Node: Copy + Eq, S: LayoutScalar, M>(
    summary: &mut EntrySummary<Node, S>,
    offers: Vec<Offer<Node, S>>,
    _offers_custody: EntryBacking,
) -> LayoutResultOf<Node, (), S, M> {
    let position = summary
        .query
        .position
        .ok_or_else(|| float_context::missing(summary.query.node))?;
    let incoming = summary
        .query
        .incoming
        .as_ref()
        .ok_or_else(|| float_context::missing(summary.query.node))?;
    for offer in offers.into_iter().rev() {
        case_work(summary.funding.as_ref(), offer.node, 1)?;
        let hypothetical = incoming
            .frame
            .common_border(position.hypothetical(summary.start))
            .map_err(|error| crate::error::sizing_resolution_error(offer.node, error.into()))?;
        let support = summary.start;
        if let Some(relevant_bottom) = projected_bound(
            summary,
            support,
            offer.floats_before,
            offer.clear,
            offer.axes,
        )? && hypothetical < relevant_bottom
        {
            summary.start = offer.before;
            summary.through = Through::Blocked;
            summary.separation = Some(Separation {
                container: offer.container,
                node: offer.node,
                index: offer.index,
                hypothetical,
                relevant_bottom,
                support,
                floats_before: offer.floats_before,
            });
        }
    }
    Ok(())
}

impl<Node: Copy + Eq, S: LayoutScalar> EntrySummary<Node, S> {
    /// Recheck the freshly observed cut occurrence and hypothetical ledger
    /// before returning the summary.
    pub(crate) fn validate_separation<'input, Tree, M>(
        &self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        for observation in &self.native {
            case_work(self.funding.as_ref(), observation.container, 1)?;
            let state = tree
                .text_computation()
                .ok_or_else(|| float_context::missing(observation.container))?;
            let segment = observation
                .window
                .segment
                .as_ref()
                .ok_or_else(|| float_context::missing(observation.container))?;
            let run = state
                .entry_run_from_segment(observation.container, segment, self.funding.as_ref())
                .map_err(|error| case_error(observation.container, error))?
                .ok_or_else(|| float_context::missing(observation.container))?;
            if run.nodes().len() != observation.window.nodes.len() {
                return Err(float_context::missing(observation.container));
            }
            let mut children = tree.children(observation.container);
            for _ in 0..observation.window.nodes.start {
                case_work(self.funding.as_ref(), observation.container, 1)?;
                if children.next().is_none() {
                    return Err(float_context::missing(observation.container));
                }
            }
            for &node in run.nodes() {
                case_work(self.funding.as_ref(), observation.container, 1)?;
                if children.next() != Some(node) {
                    return Err(float_context::missing(observation.container));
                }
            }
            if observation.proof.evidence().request().segment != run.segment
                || observation.proof.evidence().request().inline_boxes.as_ref()
                    != run.inline_boxes.family()
            {
                return Err(float_context::missing(observation.container));
            }
        }
        let Some(cut) = &self.separation else {
            return Ok(());
        };
        let mut children = tree.children(cut.container);
        let mut source = None;
        for _ in 0..=cut.index {
            case_work(self.funding.as_ref(), cut.container, 1)?;
            source = children.next();
            if source.is_none() {
                break;
            }
        }
        let raw = tree.node_input(cut.container);
        let axes = crate::FlowAxes::new(raw.writing_mode, raw.direction);
        if source != Some(cut.node)
            || projected_bound(
                self,
                cut.support,
                cut.floats_before,
                tree.node_input(cut.node).clear,
                axes,
            )? != Some(cut.relevant_bottom)
            || cut.hypothetical >= cut.relevant_bottom
        {
            return Err(float_context::missing(self.query.node));
        }
        Ok(())
    }
}
