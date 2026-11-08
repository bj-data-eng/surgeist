//! Inline receiving authority for a carrier path, qualified by adopted content.
use super::content_insets::ContentInsetBinding;
use super::*;
use crate::engine::block_scope::ProductionId;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Origin<S: LayoutScalar> {
    pub(crate) normal: Point<S>,
    pub(crate) visual: Point<S>,
}

pub(crate) struct Fact<Node, S: LayoutScalar> {
    node: Node,
    parent: Option<Node>,
    role: crate::BlockGeneration,
    axes: crate::FlowAxes,
    input: ComputeInputOf<S>,
    normal: Point<S>,
    outer: S,
    inner: S,
    padding: Edges<S>,
    border: Edges<S>,
    source_style: crate::NodeInputOf<S>,
}
pub(crate) struct PreparedPath<Node, S: LayoutScalar> {
    binding: ContentInsetBinding<Node, S>,
    facts: Vec<Fact<Node, S>>,
    request: crate::LayoutRootRequestOf<S>,
    frame: crate::BlockFragmentainerOf<S>,
    snapshot: std::sync::Arc<intrinsic::FragmentContext<Node, S>>,
}
/// Normal completion qualifies the candidate's producer inputs, but grants no
/// receiving authority until the owning production is reached by adoption.
pub(crate) struct PerformedPath<Node, S: LayoutScalar>(PreparedPath<Node, S>);
pub(crate) struct AcceptedPath<Node, S: LayoutScalar> {
    production: ProductionId,
    path: PreparedPath<Node, S>,
}
type FactResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    Fact<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
    <Tree as Traverse>::Scalar,
    M,
>;
type PreparedFactResult<Tree, M> = LayoutResultOf<
    <Tree as Traverse>::Node,
    (
        Fact<<Tree as Traverse>::Node, <Tree as Traverse>::Scalar>,
        content::Context<<Tree as Traverse>::Scalar>,
    ),
    <Tree as Traverse>::Scalar,
    M,
>;

fn invalid<Node, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::ValueResolution,
        LayoutErrorKindOf::BlockFragmentation(crate::BlockFragmentationFailure::InvalidSource),
    )
}
fn fact<'input, Tree, M>(
    tree: &Tree,
    node: Tree::Node,
    parent: Option<Tree::Node>,
    input: ComputeInputOf<Tree::Scalar>,
    normal: Point<Tree::Scalar>,
    context: content::Context<Tree::Scalar>,
) -> FactResult<Tree, M>
where
    Tree: Compute<'input, M>,
{
    Ok(Fact {
        node,
        parent,
        role: tree.node_input(node).block_generation,
        axes: context.axes(),
        input,
        normal,
        outer: context.inline_outer,
        inner: context
            .constants
            .accepted_inline_inner(node, context.inline_outer)?,
        padding: context.constants.padding,
        border: context.constants.effective_border,
        source_style: tree.node_input(node).clone(),
    })
}
pub(crate) fn root<'input, Tree, M>(
    tree: &mut Tree,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    reference: Size<Tree::Scalar>,
) -> PreparedFactResult<Tree, M>
where
    Tree: Compute<'input, M>,
{
    let context = content::prepare(tree, node, input, tree.node_input(node))?;
    let pending = scope::root(tree, node, input)?;
    Ok((
        fact(
            tree,
            node,
            None,
            input,
            pending.overflow_origin(context.inline_outer, reference),
            context,
        )?,
        context,
    ))
}
pub(crate) fn child<'input, Tree, M>(
    tree: &mut Tree,
    parent: Tree::Node,
    node: Tree::Node,
    input: ComputeInputOf<Tree::Scalar>,
    parent_context: content::Context<Tree::Scalar>,
) -> PreparedFactResult<Tree, M>
where
    Tree: Compute<'input, M>,
{
    if !tree.children(parent).any(|child| child == node) {
        return Err(invalid(node));
    }
    let normal = scope::carrier_origin(tree, parent, node, input, parent_context)?;
    let context = content::prepare(tree, node, input, tree.node_input(node))?;
    Ok((
        fact(tree, node, Some(parent), input, normal, context)?,
        context,
    ))
}
impl<Node: Copy + Eq, S: LayoutScalar> PreparedPath<Node, S> {
    pub(crate) fn new<M>(
        binding: ContentInsetBinding<Node, S>,
        facts: Vec<Fact<Node, S>>,
        request: crate::LayoutRootRequestOf<S>,
        frame: crate::BlockFragmentainerOf<S>,
        snapshot: std::sync::Arc<intrinsic::FragmentContext<Node, S>>,
    ) -> LayoutResultOf<Node, Self, S, M> {
        if facts.first().is_none_or(|fact| fact.parent.is_some())
            || facts.last().is_none_or(|fact| {
                fact.node != binding.owner || fact.axes != binding.axes || fact.role != binding.role
            })
            || facts
                .windows(2)
                .any(|pair| pair[1].parent != Some(pair[0].node))
        {
            return Err(invalid(binding.owner));
        }
        Ok(Self {
            binding,
            facts,
            request,
            frame,
            snapshot,
        })
    }
    pub(crate) fn performed<M>(
        self,
        normal: &content::Receipt<Node, S>,
        context: content::Context<S>,
        expected: &ContentInsetBinding<Node, S>,
        receiving: &std::sync::Arc<intrinsic::FragmentContext<Node, S>>,
    ) -> LayoutResultOf<Node, PerformedPath<Node, S>, S, M> {
        let terminal = self.facts.last().ok_or_else(|| invalid(expected.owner))?;
        if self.binding != *expected
            || !std::sync::Arc::ptr_eq(&self.snapshot, receiving)
            || self.snapshot.target != expected.target
            || normal.binding.as_ref() != Some(expected)
            || !normal.normal_complete
            || normal.node != terminal.node
            || normal.axes != terminal.axes
            || normal.role != terminal.role
            || context.input() != terminal.input
            || context.inline_outer != terminal.outer
            || normal.accepted_inline_inner != terminal.inner
            || context
                .axes()
                .logical_edges(context.constants.padding)
                .inline_start
                != terminal.axes.logical_edges(terminal.padding).inline_start
            || context
                .axes()
                .logical_edges(context.constants.padding)
                .inline_end
                != terminal.axes.logical_edges(terminal.padding).inline_end
            || context
                .axes()
                .logical_edges(context.constants.effective_border)
                .inline_start
                != terminal.axes.logical_edges(terminal.border).inline_start
            || context
                .axes()
                .logical_edges(context.constants.effective_border)
                .inline_end
                != terminal.axes.logical_edges(terminal.border).inline_end
        {
            return Err(invalid(expected.owner));
        }
        Ok(PerformedPath(self))
    }
}
impl<Node: Copy + Eq, S: LayoutScalar> PerformedPath<Node, S> {
    pub(crate) fn adopt<Tree: crate::LayoutTree<Node = Node, Scalar = S>>(
        self,
        tree: &Tree,
        production: ProductionId,
        expected: &ContentInsetBinding<Node, S>,
        request: crate::LayoutRootRequestOf<S>,
        frame: crate::BlockFragmentainerOf<S>,
        snapshot: &std::sync::Arc<intrinsic::FragmentContext<Node, S>>,
    ) -> LayoutResultOf<Node, AcceptedPath<Node, S>, S, Tree::MeasureError> {
        if self.0.binding != *expected
            || self.0.request != request
            || self.0.frame != frame
            || !std::sync::Arc::ptr_eq(&self.0.snapshot, snapshot)
            || self.0.facts.iter().any(|fact| {
                fact.source_style != *tree.node_input(fact.node)
                    || fact.parent.is_some_and(|parent| {
                        !tree.children(parent).any(|child| child == fact.node)
                    })
            })
        {
            return Err(invalid(expected.owner));
        }
        Ok(AcceptedPath {
            production,
            path: self.0,
        })
    }
}
impl<Node: Copy + Eq, S: LayoutScalar> AcceptedPath<Node, S> {
    pub(crate) fn contains(&self, node: Node) -> bool {
        self.path.facts.iter().any(|fact| fact.node == node)
    }
    pub(crate) fn origin<'input, Tree: Compute<'input, M, Node = Node, Scalar = S>, M>(
        &self,
        tree: &Tree,
        production: ProductionId,
        expected: &ContentInsetBinding<Node, S>,
        node: Node,
        principal_parent: Option<crate::LayoutOutputEntryOf<Node, S>>,
    ) -> LayoutResultOf<Node, Origin<S>, S, M> {
        if self.production != production
            || self.path.binding != *expected
            || self
                .path
                .facts
                .iter()
                .any(|fact| fact.source_style != *tree.node_input(fact.node))
        {
            return Err(invalid(node));
        }
        let fact = self
            .path
            .facts
            .iter()
            .find(|fact| fact.node == node)
            .ok_or_else(|| invalid(node))?;
        let Some(parent) = fact.parent else {
            return Ok(Origin {
                normal: fact.normal,
                visual: fact.normal,
            });
        };
        let containing = self
            .path
            .facts
            .iter()
            .find(|fact| fact.node == parent)
            .ok_or_else(|| invalid(node))?;
        let offset = if let Some(entry) = principal_parent {
            if entry.node() != parent {
                return Err(invalid(node));
            }
            let basis = entry
                .measured_inset_basis()
                .flatten()
                .ok_or_else(|| invalid(parent))?;
            relative::resolve(
                tree,
                node,
                tree.node_input(node).inset,
                relative::InsetReference::Accepted(Some(basis)),
                containing.axes,
            )?
        } else {
            let binding = ContentInsetBinding {
                owner: parent,
                role: containing.role,
                axes: containing.axes,
                ..expected.clone()
            };
            let basis =
                content_insets::qualified_path_inline_basis(binding.clone(), containing.inner);
            relative::resolve_content(tree, node, tree.node_input(node).inset, &basis, &binding)?
        };
        let result = Point::new(fact.normal.x + offset.x, fact.normal.y + offset.y);
        if !result.x.is_finite() || !result.y.is_finite() {
            return Err(invalid(node));
        }
        Ok(Origin {
            normal: fact.normal,
            visual: result,
        })
    }
}

#[cfg(test)]
#[path = "carrier_tests.rs"]
mod tests;
