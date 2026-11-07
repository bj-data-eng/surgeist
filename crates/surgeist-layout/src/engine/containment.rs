use super::contracts::{Compute, UnroundedEntryResult};
use crate::geometry::{FlowAxes, LogicalSizeOf, Size};
use crate::grid::{GridOverflowObservation, GridOverflowProbe};
use crate::output::{ComputeInputOf, NamedBaselineDemand, RunMode};
use crate::{
    BlockGeneration, ComputeOutputOf, Display, LayoutInputViewOf, LayoutOutputEntryOf,
    LayoutResultOf, LayoutScalar, NodeInputOf, SizeContainment, Traverse,
};

#[cfg(test)]
mod tests;

pub(crate) fn size_containment<S: LayoutScalar>(style: &NodeInputOf<S>) -> SizeContainment {
    if style.block_generation == BlockGeneration::Principal
        && style.display != Display::None
        && !style.item_is_table
    {
        style.size_containment
    } else {
        SizeContainment::None
    }
}

pub(crate) fn contained_outer_size<S: LayoutScalar>(
    containment: SizeContainment,
    flow_axes: FlowAxes,
    empty: Size<S>,
) -> Size<Option<S>> {
    match containment {
        SizeContainment::None => Size::NONE,
        SizeContainment::InlineAxis => flow_axes.physical_size(LogicalSizeOf::new(
            Some(flow_axes.logical_size(empty).inline),
            None,
        )),
        SizeContainment::BothAxes => empty.map(Some),
    }
}

pub(crate) const fn empty_sizing_input<S: LayoutScalar>(
    input: ComputeInputOf<S>,
) -> ComputeInputOf<S> {
    input
        .with_run_mode(RunMode::ComputeSize)
        .with_named_baseline_demand(NamedBaselineDemand::None)
        .with_physical_baseline_demand(false)
}

pub(crate) fn compute_empty_box<'a, Node: Copy + Eq, S: LayoutScalar, T>(
    node: Node,
    style: &'a NodeInputOf<S>,
    compute: impl FnOnce(&mut EmptyBox<'a, Node, S>) -> T,
) -> T {
    compute(&mut EmptyBox {
        node,
        style,
        minimum: None,
    })
}

/// A sizing invocation borrows only the original box, with no participants or
/// host cache. The formatter can inspect its own box and tracks.
pub(crate) struct EmptyBox<'a, Node, S: LayoutScalar> {
    node: Node,
    style: &'a NodeInputOf<S>,
    minimum: Option<crate::block::fragment_minimum::Override<S>>,
}

impl<Node, S: LayoutScalar> EmptyBox<'_, Node, S> {
    pub(crate) fn install_fragment_minimum(
        &mut self,
        minimum: Option<crate::block::fragment_minimum::Override<S>>,
    ) {
        self.minimum = minimum;
    }
}

impl<Node: Copy + Eq, S: LayoutScalar> Traverse for EmptyBox<'_, Node, S> {
    type Node = Node;
    type Scalar = S;
    type Children<'a>
        = std::iter::Empty<Node>
    where
        Self: 'a;

    fn children(&self, _node: Node) -> Self::Children<'_> {
        std::iter::empty()
    }

    fn child_count(&self, _node: Node) -> usize {
        0
    }

    fn child(&self, _node: Node, _index: usize) -> Node {
        unreachable!("the empty sizing box has no children")
    }
}

impl<'input, Node: Copy + Eq + 'input, S: LayoutScalar, M> Compute<'input, M>
    for EmptyBox<'input, Node, S>
{
    fn fragment_minimum(&self, _node: Node) -> Option<crate::block::fragment_minimum::Override<S>> {
        self.minimum
    }
    fn node_input(&self, node: Node) -> &'input NodeInputOf<S> {
        assert!(
            node == self.node,
            "the empty sizing invocation owns one box"
        );
        self.style
    }

    fn layout_input(&self, node: Node) -> LayoutInputViewOf<'input, S> {
        LayoutInputViewOf::Box(<Self as Compute<'input, M>>::node_input(self, node))
    }

    fn staged_unrounded(&self, _node: Node) -> UnroundedEntryResult<Node, S, M> {
        unreachable!("the empty sizing box stages no child geometry")
    }

    fn set_unrounded(&mut self, _entry: LayoutOutputEntryOf<Node, S>) {
        unreachable!("the empty sizing box publishes no child geometry")
    }

    fn compute_child(
        &mut self,
        _node: Node,
        _input: ComputeInputOf<S>,
    ) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M> {
        unreachable!("the empty sizing box has no children")
    }

    fn observe_grid_overflow(
        &mut self,
        node: Node,
        probe: GridOverflowProbe<'input, '_, Node, S>,
    ) -> LayoutResultOf<Node, GridOverflowObservation<S>, S, M> {
        probe.observe(self, node)
    }
}
