use core::ops::Range;

use super::contracts::{CacheAccess, Compute};
use crate::scroll::SettledAutoScrollbarState;
use crate::{
    ComputeInputOf, ContainingLayoutContext, LayoutInputViewOf, LayoutOutputEntryOf,
    LayoutResultOf, LayoutScalar, ParentFormattingContext, SourceIndex,
};

pub(super) struct HiddenRequest<Node> {
    pub(super) node: Node,
    pub(super) source_index: SourceIndex,
    pub(super) containing_context: ContainingLayoutContext,
    pub(super) containing_pass: SettledAutoScrollbarState,
}

/// Source-ordered descendants of a hidden node after its own cache and output
/// have been staged.
pub(super) struct HiddenChildren<Node> {
    node: Node,
    indices: Range<usize>,
    context: ContainingLayoutContext,
    pass: SettledAutoScrollbarState,
}

type NextHiddenChild<Node, S, M> = LayoutResultOf<Node, Option<(Node, ComputeInputOf<S>)>, S, M>;

impl<Node: Copy + Eq> HiddenChildren<Node> {
    pub(super) fn begin<'input, Tree, S, M>(
        tree: &mut Tree,
        request: HiddenRequest<Node>,
    ) -> LayoutResultOf<Node, Self, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S> + CacheAccess<M, Node = Node, Scalar = S>,
        S: LayoutScalar,
    {
        clear_hidden(tree, request.node, request.source_index)?;
        Ok(Self {
            node: request.node,
            indices: 0..tree.child_count(request.node),
            context: ContainingLayoutContext::new(
                request.containing_context.flow_axes(),
                ParentFormattingContext::NoParent,
            ),
            pass: request.containing_pass,
        })
    }

    pub(super) fn has_remaining(&self) -> bool {
        !self.indices.is_empty()
    }

    pub(super) fn next_box<'input, Tree, S, M>(
        &mut self,
        tree: &mut Tree,
    ) -> NextHiddenChild<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S> + CacheAccess<M, Node = Node, Scalar = S>,
        S: LayoutScalar,
    {
        for index in self.indices.by_ref() {
            let child = tree.child(self.node, index);
            let source = SourceIndex::new(index);
            match tree.layout_input(child) {
                LayoutInputViewOf::Box(_) => {
                    tree.set_unrounded(LayoutOutputEntryOf::hidden_unrounded(child, source));
                    return Ok(Some((
                        child,
                        ComputeInputOf::hidden_in_containing_pass(self.context, self.pass),
                    )));
                }
                LayoutInputViewOf::InlineText(_)
                | LayoutInputViewOf::LineBreak(_)
                | LayoutInputViewOf::InlineBoundary(_)
                | LayoutInputViewOf::TextControl(_) => clear_hidden(tree, child, source)?,
            }
        }
        Ok(None)
    }
}

fn clear_hidden<'input, Tree, Node, S, M>(
    tree: &mut Tree,
    node: Node,
    source: SourceIndex,
) -> LayoutResultOf<Node, (), S, M>
where
    Tree: Compute<'input, M, Node = Node, Scalar = S> + CacheAccess<M, Node = Node, Scalar = S>,
    Node: Copy + Eq,
    S: LayoutScalar,
{
    tree.cache_clear(node)?;
    tree.set_unrounded_inline_fragment_state(node, None);
    tree.set_unrounded(LayoutOutputEntryOf::hidden_unrounded(node, source));
    Ok(())
}

#[cfg(test)]
pub(crate) fn compute_hidden<'input, Tree, Node, S, M>(
    tree: &mut Tree,
    node: Node,
    source_index: SourceIndex,
    containing_context: ContainingLayoutContext,
    containing_pass: SettledAutoScrollbarState,
) -> LayoutResultOf<Node, crate::ComputeOutputOf<S>, S, M>
where
    Tree: Compute<'input, M, Node = Node, Scalar = S> + CacheAccess<M, Node = Node, Scalar = S>,
    Node: Copy + Eq,
    S: LayoutScalar,
{
    let mut children = HiddenChildren::begin(
        tree,
        HiddenRequest {
            node,
            source_index,
            containing_context,
            containing_pass,
        },
    )?;
    while let Some((child, input)) = children.next_box(tree)? {
        tree.compute_child(child, input)?;
    }
    Ok(crate::ComputeOutputOf::HIDDEN)
}
