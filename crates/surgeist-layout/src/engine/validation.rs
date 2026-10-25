use crate::{
    AtomicInlineParticipationRoleError, FloatExclusion, FloatExclusionRoleError, LayoutErrorKindOf,
    LayoutErrorOf, LayoutErrorSiteOf, LayoutInputViewOf, LayoutInvalidInputOf, LayoutOperation,
    LayoutResultOf, LayoutScalar, LayoutTree, NodeInputOf, NonBoxNodeRoleError, Position,
};
use crate::{Display, LayoutUnsupportedCapability};

pub(crate) fn validate_layout_request<Tree>(
    tree: &Tree,
    root: Tree::Node,
    changed_nodes: &[Tree::Node],
) -> LayoutResultOf<Tree::Node, Vec<Tree::Node>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    validate_layout_request_with_text(tree, root, changed_nodes, &[])
}

pub(crate) fn validate_layout_request_with_text<Tree>(
    tree: &Tree,
    root: Tree::Node,
    changed_nodes: &[Tree::Node],
    paragraphs: &[crate::TextParagraphBinding<Tree::Node>],
) -> LayoutResultOf<Tree::Node, Vec<Tree::Node>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    validate_layout_request_with_text_graph(tree, root, changed_nodes, paragraphs)
        .map(|(_, invalidated)| invalidated)
}

/// One discovery of the immutable acyclic graph. Integer edges preserve child
/// order and all incoming edges without requiring host node hashing or ordering.
#[derive(Clone)]
pub(crate) struct ValidationGraph<Node> {
    pub nodes: Vec<Node>,
    pub children: Vec<Vec<usize>>,
    pub parents: Vec<Vec<usize>>,
    pub finish: Vec<usize>,
}

impl<Node: Copy + Eq> ValidationGraph<Node> {
    pub fn index(&self, node: Node) -> Option<usize> {
        self.nodes.iter().position(|&candidate| candidate == node)
    }

    pub fn path_counts(&self) -> Vec<u8> {
        let mut counts = vec![0u8; self.nodes.len()];
        counts[0] = 1;
        for &index in self.finish.iter().rev() {
            for &child in &self.children[index] {
                counts[child] = counts[child].saturating_add(counts[index]).min(2);
            }
        }
        counts
    }

    pub fn invalidation<S: LayoutScalar, M>(
        &self,
        changed_nodes: &[Node],
    ) -> LayoutResultOf<Node, Vec<Node>, S, M> {
        let mut dirty = Vec::new();
        for &node in changed_nodes {
            let index = self.index(node).ok_or_else(|| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::CacheInvalidation,
                    LayoutErrorKindOf::InvalidInput(
                        LayoutInvalidInputOf::InvalidationNodeNotReachable,
                    ),
                )
            })?;
            dirty.push(index);
        }
        let mut included = vec![false; self.nodes.len()];
        while let Some(index) = dirty.pop() {
            if !included[index] {
                included[index] = true;
                dirty.extend(self.parents[index].iter().copied());
            }
        }
        Ok(self
            .nodes
            .iter()
            .copied()
            .zip(included)
            .filter_map(|(node, include)| include.then_some(node))
            .collect())
    }
}

type ValidatedRequest<Node> = (ValidationGraph<Node>, Vec<Node>);

pub(crate) fn validate_layout_request_with_text_graph<Tree>(
    tree: &Tree,
    root: Tree::Node,
    changed_nodes: &[Tree::Node],
    paragraphs: &[crate::TextParagraphBinding<Tree::Node>],
) -> LayoutResultOf<Tree::Node, ValidatedRequest<Tree::Node>, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    let graph = discover_graph(tree, root)?;
    let invalidated = graph.invalidation(changed_nodes)?;
    if validate_layout_tree(tree, &graph, paragraphs)? {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(root),
            LayoutOperation::RootLayout,
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior),
        ));
    }
    Ok((graph, invalidated))
}

fn discover_graph<Tree: LayoutTree>(
    tree: &Tree,
    root: Tree::Node,
) -> LayoutResultOf<Tree::Node, ValidationGraph<Tree::Node>, Tree::Scalar, Tree::MeasureError> {
    let mut graph = ValidationGraph {
        nodes: vec![root],
        children: vec![Vec::new()],
        parents: vec![Vec::new()],
        finish: Vec::new(),
    };
    let mut complete = vec![false];
    let mut path = vec![(0, tree.children(root).collect::<Vec<_>>(), 0)];
    while let Some((index, children, next)) = path.last_mut() {
        if *next == children.len() {
            complete[*index] = true;
            graph.finish.push(*index);
            path.pop();
            continue;
        }
        let parent = *index;
        let child = children[*next];
        *next += 1;
        if let Some(child_index) = graph.index(child) {
            if !complete[child_index] {
                return Err(LayoutErrorOf::new(
                    LayoutErrorSiteOf::ContainerSubject {
                        container: graph.nodes[parent],
                        subject: child,
                    },
                    LayoutOperation::CacheInvalidation,
                    LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TreeTopologyCycle),
                ));
            }
            graph.children[parent].push(child_index);
            graph.parents[child_index].push(parent);
        } else {
            let child_index = graph.nodes.len();
            graph.children[parent].push(child_index);
            graph.nodes.push(child);
            graph.children.push(Vec::new());
            graph.parents.push(vec![parent]);
            complete.push(false);
            path.push((child_index, tree.children(child).collect(), 0));
        }
    }
    Ok(graph)
}

fn validate_layout_tree<Tree>(
    tree: &Tree,
    graph: &ValidationGraph<Tree::Node>,
    paragraphs: &[crate::TextParagraphBinding<Tree::Node>],
) -> LayoutResultOf<Tree::Node, bool, Tree::Scalar, Tree::MeasureError>
where
    Tree: LayoutTree,
{
    fn visit<Tree>(
        tree: &Tree,
        node: Tree::Node,
        is_root: bool,
        parent_accepts_inline_text: bool,
        later_behavior: &mut bool,
        paragraphs: &[crate::TextParagraphBinding<Tree::Node>],
    ) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, Tree::MeasureError>
    where
        Tree: LayoutTree,
    {
        let layout_input = tree.layout_input(node);
        match layout_input {
            LayoutInputViewOf::Box(_) => {
                let input = tree.node_input(node);
                if input.block_generation == crate::BlockGeneration::Anonymous {
                    if input.display != Display::Block
                        || input.position == Position::Absolute
                        || input.float != crate::Float::None
                    {
                        return Err(root_input_error(
                            node,
                            LayoutInvalidInputOf::AnonymousBlockRole,
                        ));
                    }
                    if is_root {
                        return Err(LayoutErrorOf::new(
                            LayoutErrorSiteOf::Node(node),
                            LayoutOperation::RootLayout,
                            LayoutErrorKindOf::MissingContext(
                                crate::LayoutMissingContext::AnonymousGeneratingBlock,
                            ),
                        ));
                    }
                }
                let text_bound = paragraphs
                    .iter()
                    .any(|paragraph| paragraph.atomics.iter().any(|binding| binding.node == node));
                if text_bound && input.atomic_inline_participation.is_some() {
                    return Err(root_input_error(
                        node,
                        LayoutInvalidInputOf::AtomicInlineParticipation {
                            reason: AtomicInlineParticipationRoleError::UnexpectedForPreparedAtomic,
                        },
                    ));
                }
                match (
                    !text_bound && !is_root && input.display.is_inline_level(),
                    input.atomic_inline_participation.is_some(),
                ) {
                    (true, false) => {
                        return Err(root_input_error(
                            node,
                            LayoutInvalidInputOf::AtomicInlineParticipation {
                                reason: AtomicInlineParticipationRoleError::MissingForAtomicInline,
                            },
                        ));
                    }
                    (false, true) => {
                        return Err(root_input_error(
                            node,
                            LayoutInvalidInputOf::AtomicInlineParticipation {
                                reason: AtomicInlineParticipationRoleError::UnexpectedForNonAtomic,
                            },
                        ));
                    }
                    (true, true) | (false, false) => {}
                }

                if input.float_exclusion == FloatExclusion::Shape {
                    let reason = if input.display == Display::None {
                        Some(FloatExclusionRoleError::Hidden)
                    } else if input.position == Position::Absolute {
                        Some(FloatExclusionRoleError::Absolute)
                    } else if input.float == crate::Float::None {
                        Some(FloatExclusionRoleError::NonFloating)
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        return Err(root_input_error(
                            node,
                            LayoutInvalidInputOf::FloatExclusionRole { reason },
                        ));
                    }
                }
                Ok(())
            }
            LayoutInputViewOf::InlineText(_) | LayoutInputViewOf::TextControl(_) => {
                if let Some(reason) = non_box_node_role_error(tree, node) {
                    return Err(root_input_error(
                        node,
                        LayoutInvalidInputOf::NonBoxNodeRole { reason },
                    ));
                }
                if !parent_accepts_inline_text {
                    *later_behavior = true;
                }
                Ok(())
            }
            LayoutInputViewOf::LineBreak(_) | LayoutInputViewOf::InlineBoundary(_) => {
                if let Some(reason) = non_box_node_role_error(tree, node) {
                    return Err(root_input_error(
                        node,
                        LayoutInvalidInputOf::NonBoxNodeRole { reason },
                    ));
                }
                Ok(())
            }
        }
    }

    let mut later_behavior = false;
    visit(
        tree,
        graph.nodes[0],
        true,
        false,
        &mut later_behavior,
        paragraphs,
    )?;
    for (parent, children) in graph.children.iter().enumerate() {
        let accepts = tree
            .layout_input(graph.nodes[parent])
            .as_box()
            .is_some_and(|input| {
                input.display == Display::None || input.display.inner_display() == Display::Block
            });
        // Parent-dependent roles are checked for every incoming edge, including
        // shared DAG nodes. Graph discovery itself visits each node only once.
        for &child in children {
            visit(
                tree,
                graph.nodes[child],
                false,
                accepts,
                &mut later_behavior,
                paragraphs,
            )?;
        }
    }
    Ok(later_behavior)
}

fn non_box_node_role_error<Tree>(tree: &Tree, node: Tree::Node) -> Option<NonBoxNodeRoleError>
where
    Tree: LayoutTree,
{
    if tree.node_input(node) != &NodeInputOf::non_box() {
        Some(NonBoxNodeRoleError::NonCanonicalNodeInput)
    } else if tree.child_count(node) != 0 {
        Some(NonBoxNodeRoleError::HasChildren)
    } else if tree.has_leaf_measurement(node) {
        Some(NonBoxNodeRoleError::HasLeafMeasurement)
    } else {
        None
    }
}

fn root_input_error<Node, S, M>(
    node: Node,
    invalid: LayoutInvalidInputOf<S>,
) -> LayoutErrorOf<Node, S, M>
where
    S: LayoutScalar,
{
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        LayoutErrorKindOf::InvalidInput(invalid),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_diamonds_preserve_unique_graph_nodes_and_capped_occurrence_counts() {
        // There are 2^32 root-to-leaf paths, but only65 actual nodes. Ordinary
        // box DAG validation and invalidation operate on those nodes/edges.
        let mut tree = crate::test_support::layout_tree::PublicLayoutTreeOf::<f64>::new()
            .children(0, [1, 2])
            .style(0, crate::NodeInputOf::default());
        for level in 0..32u32 {
            for node in [2 * level + 1, 2 * level + 2] {
                tree = tree.style(node, crate::NodeInputOf::default());
                tree = if level == 31 {
                    tree.children(node, [])
                } else {
                    tree.children(node, [2 * level + 3, 2 * level + 4])
                };
            }
        }
        let (graph, invalidated) =
            validate_layout_request_with_text_graph(&tree, 0, &[63], &[]).unwrap();
        assert_eq!(graph.nodes.len(), 65);
        assert_eq!(invalidated.len(), 64);
        let counts = graph.path_counts();
        assert_eq!(counts[graph.index(0).unwrap()], 1);
        assert_eq!(counts[graph.index(1).unwrap()], 1);
        assert_eq!(counts[graph.index(2).unwrap()], 1);
        for node in 3..=64 {
            assert_eq!(counts[graph.index(node).unwrap()], 2);
        }
    }
    #[test]
    fn shared_text_role_is_checked_for_each_incoming_parent_edge() {
        use crate::test_support::text::{PreparedParagraph, ahem_style};
        let paragraph =
            PreparedParagraph::plain(3, surgeist_text::Source::new("X"), ahem_style(10.0, 10.0));
        let tree = crate::test_support::layout_tree::PublicLayoutTreeOf::<f64>::new()
            .children(0, [1, 2])
            .children(1, [3])
            .children(2, [3])
            .children(3, [])
            .style(
                0,
                crate::NodeInputOf {
                    display: Display::Block,
                    ..crate::NodeInputOf::default()
                },
            )
            .style(
                1,
                crate::NodeInputOf {
                    display: Display::Block,
                    ..crate::NodeInputOf::default()
                },
            )
            .style(
                2,
                crate::NodeInputOf {
                    display: Display::Flex,
                    ..crate::NodeInputOf::default()
                },
            )
            .input(3, paragraph.text_input(3));
        let Err(error) =
            crate::LayoutFlowInput::try_new(&tree, 0, vec![paragraph.binding(1)], Vec::new())
        else {
            panic!("a first valid block parent does not hide the flex parent role");
        };
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::UnsupportedCapability(LayoutUnsupportedCapability::LaterFriBehavior)
        ));
        let valid = tree.style(
            2,
            crate::NodeInputOf {
                display: Display::Block,
                ..crate::NodeInputOf::default()
            },
        );
        crate::LayoutFlowInput::try_new(&valid, 0, vec![paragraph.binding(1)], Vec::new()).unwrap();
    }
}
