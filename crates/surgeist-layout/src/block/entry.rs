//! Entry facts that do not require the completed descendant body.
use super::*;

mod adjoining;
pub(super) use adjoining::AdjoiningFlow;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StartFact<S: LayoutScalar> {
    pub(crate) margin: CollapsibleMarginOf<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Qualification<S: LayoutScalar> {
    Known(StartFact<S>),
    RequiresBody,
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
    next: usize,
    first_in_flow: bool,
    start: CollapsibleMarginOf<S>,
    through: Through<S>,
    possible_closed_start: Option<CollapsibleMarginOf<S>>,
    child_margin: (S, S),
}

enum PrefixStep<Node, S: LayoutScalar> {
    Child(Node, ComputeInputOf<S>, PhysicalSide),
    Complete(Prefix<S>),
    RequiresBody,
}

impl<Node: Copy, S: LayoutScalar> PrefixFrame<Node, S> {
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

    fn step<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
    ) -> LayoutResultOf<Node, PrefixStep<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
        Node: 'input,
    {
        if !matches!(self.through, Through::Blocked) {
            while let Some(&child) = self.children.get(self.next) {
                self.next += 1;
                let projection = InlineParticipantProjection::lookup::<Tree, M>(tree, child);
                let InlineParticipantKindOf::Box(style) = projection.kind() else {
                    return Ok(PrefixStep::RequiresBody);
                };
                if style.display == crate::Display::None
                    || style.common.position == Position::Absolute
                {
                    continue;
                }
                if !style.float.is_none() || style.display.is_inline_level() {
                    return Ok(PrefixStep::RequiresBody);
                }
                if !in_flow::child_margin_can_collapse_with_parent(&style) {
                    self.through = Through::Blocked;
                    break;
                }
                if !self.prepared_inline {
                    let container = block_container_projection::<Tree, M>(tree, self.node);
                    let outer = intrinsic::accepted_inline(
                        tree,
                        self.node,
                        &container,
                        &self.constants,
                        self.input,
                        |tree, op| {
                            if let Some(value) =
                                tree.fragment_inline_content(self.node, self.input, op)?
                            {
                                Ok(value)
                            } else {
                                intrinsic::content(tree, self.node, &self.constants, self.input, op)
                            }
                        },
                    )?;
                    let inner = self.constants.accepted_inline_inner(self.node, outer)?;
                    self.constants = self.constants.with_logical_node_inner_size(
                        LogicalSizeOf::new(
                            Some(inner),
                            self.constants.logical_node_inner_size().block,
                        ),
                        false,
                    );
                    self.prepared_inline = true;
                }
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

pub(crate) fn qualify<'input, Tree, M>(
    tree: &mut Tree,
    mut node: Tree::Node,
    mut input: ComputeInputOf<Tree::Scalar>,
    mut reporting_side: PhysicalSide,
) -> LayoutResultOf<Tree::Node, Qualification<Tree::Scalar>, Tree::Scalar, M>
where
    Tree: Compute<'input, M>,
{
    let mut frames: Vec<PrefixFrame<Tree::Node, Tree::Scalar>> = Vec::new();
    'descend: loop {
        let raw = tree.node_input(node);
        let own = crate::FlowAxes::new(raw.writing_mode, raw.direction);
        let mut fact = if tree.has_leaf_measurement(node) {
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
            let contained = containment_sizing(node, raw, input, tree.fragment_minimum(node))?
                .map_or(Size::NONE, |value| value.outer);
            let style = block_container_projection::<Tree, M>(tree, node);
            let constants =
                Constants::from_projection::<Tree, M>(tree, node, &style, input, contained)?;
            if own.block_start() != reporting_side {
                if own.block_end() != reporting_side || constants.collapse_bottom_margin {
                    return Ok(Qualification::RequiresBody);
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
                    first_in_flow: true,
                    start: CollapsibleMarginOf::ZERO,
                    through: Through::Known(CollapsibleMarginOf::ZERO),
                    possible_closed_start: None,
                    child_margin: (Tree::Scalar::ZERO, Tree::Scalar::ZERO),
                };
                match frame.step::<Tree, M>(tree)? {
                    PrefixStep::Child(child, request, side) => {
                        frames.push(frame);
                        (node, input, reporting_side) = (child, request, side);
                        continue 'descend;
                    }
                    PrefixStep::Complete(fact) => fact,
                    PrefixStep::RequiresBody => return Ok(Qualification::RequiresBody),
                }
            }
        };
        loop {
            let Some(mut frame) = frames.pop() else {
                return Ok(Qualification::Known(StartFact { margin: fact.start }));
            };
            if !frame.accept(fact) {
                return Ok(Qualification::RequiresBody);
            }
            match frame.step::<Tree, M>(tree)? {
                PrefixStep::Child(child, request, side) => {
                    frames.push(frame);
                    (node, input, reporting_side) = (child, request, side);
                    continue 'descend;
                }
                PrefixStep::Complete(result) => fact = result,
                PrefixStep::RequiresBody => return Ok(Qualification::RequiresBody),
            }
        }
    }
}

#[cfg(test)]
#[path = "entry_tests.rs"]
mod tests;
