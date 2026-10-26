//! Request-bound shared-BFC float handoffs through the ordinary child dispatcher.
use super::floats::{FloatExclusions, InheritedFloatExclusions};
use crate::{
    ComputeInputOf, ComputeOutputOf, FlowAxes, LayoutScalar, NodeInputOf, ParentFormattingContext,
};

/// `Normal` does not override another actual formatting-context boundary.
pub(crate) fn shares_parent_context<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    containing: FlowAxes,
    parent: ParentFormattingContext,
) -> bool {
    parent == ParentFormattingContext::BlockFlow
        && style.display.inner_display() == crate::Display::Block
        && !style.display.is_inline_level()
        && style.float.is_none()
        && style.position != crate::Position::Absolute
        && !style.item_is_replaced
        && style.block_flow_context == crate::BlockFlowContext::Normal
        && !style.overflow.establishes_independent_formatting_context()
        && style.writing_mode == containing.writing_mode()
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Role {
    Owner,
    Shared,
}
impl Role {
    pub(crate) fn for_input<S: LayoutScalar>(
        style: &NodeInputOf<S>,
        input: ComputeInputOf<S>,
    ) -> Self {
        if shares_parent_context(
            style,
            input.containing_layout_context().flow_axes(),
            input.parent_formatting_context(),
        ) {
            Self::Shared
        } else {
            Self::Owner
        }
    }
}

pub(super) fn sizes_float_context<S: LayoutScalar>(
    role: Role,
    input: ComputeInputOf<S>,
    finite_inline: Option<S>,
    inherited: bool,
    work: bool,
) -> bool {
    input.run_mode() == crate::RunMode::ComputeSize
        && input.sizing_mode() == crate::SizingMode::InherentSize
        && finite_inline.is_some()
        && (inherited || role == Role::Owner && work)
}

#[derive(Clone)]
pub(crate) struct Incoming<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) request: ComputeInputOf<S>,
    pub(crate) frame: std::sync::Arc<InheritedFloatExclusions<S, Node>>,
}
#[derive(Clone)]
pub(crate) struct Completed<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) request: ComputeInputOf<S>,
    pub(crate) used: ComputeInputOf<S>,
    pub(crate) output: ComputeOutputOf<S>,
    pub(crate) ledger: FloatExclusions<S, Node>,
    pub(crate) inherited: usize,
    pub(crate) frame: Option<std::sync::Arc<InheritedFloatExclusions<S, Node>>>,
}
pub(crate) struct ChildOutput<Node, S: LayoutScalar> {
    pub(crate) output: ComputeOutputOf<S>,
    pub(crate) floats: Completed<Node, S>,
}
pub(crate) struct State<Node, S: LayoutScalar> {
    incoming: Vec<Incoming<Node, S>>,
    completed: Option<Completed<Node, S>>,
    work: Option<std::sync::Arc<[bool]>>,
}
impl<Node: Copy + Eq, S: LayoutScalar> State<Node, S> {
    pub(crate) fn new() -> Self {
        Self {
            incoming: Vec::new(),
            completed: None,
            work: None,
        }
    }
    pub(crate) fn configure<Tree: crate::LayoutTree<Node = Node, Scalar = S>>(
        &mut self,
        tree: &Tree,
        graph: &crate::engine::validation::ValidationGraph<Node>,
    ) {
        let mut work = vec![false; graph.nodes.len()];
        for &index in &graph.finish {
            let node = graph.nodes[index];
            let style = tree.node_input(node);
            if tree.has_leaf_measurement(node)
                || style.display.inner_display() != crate::Display::Block
            {
                continue;
            }
            let axes = FlowAxes::new(style.writing_mode, style.direction);
            work[index] = graph.children[index].iter().copied().any(|child| {
                let crate::LayoutInputViewOf::Box(style) = tree.layout_input(graph.nodes[child])
                else {
                    return false;
                };
                style.display != crate::Display::None
                    && style.position != crate::Position::Absolute
                    && (!style.float.is_none()
                        || shares_parent_context(style, axes, ParentFormattingContext::BlockFlow)
                            && work[child])
            });
        }
        self.work = Some(work.into());
    }
    pub(crate) fn needs_work(&self, index: usize) -> Option<bool> {
        self.work.as_ref().and_then(|work| work.get(index).copied())
    }
    pub(crate) fn fork(&self) -> Self {
        Self {
            work: self.work.clone(),
            ..Self::new()
        }
    }
    pub(crate) fn reset(&mut self) {
        self.incoming.clear();
        self.completed = None;
    }
    pub(crate) fn push(
        &mut self,
        node: Node,
        request: ComputeInputOf<S>,
        frame: InheritedFloatExclusions<S, Node>,
    ) {
        self.completed = None;
        self.incoming.push(Incoming {
            node,
            request,
            frame: std::sync::Arc::new(frame),
        });
    }
    pub(crate) fn incoming(
        &self,
        node: Node,
        input: ComputeInputOf<S>,
    ) -> Option<Incoming<Node, S>> {
        self.incoming
            .last()
            .filter(|entry| {
                entry.node == node
                    && entry.request.with_settled_auto_scrollbars(
                        crate::scroll::SettledAutoScrollbarState::INITIAL,
                    ) == input
            })
            .cloned()
    }
    pub(crate) fn blocks_cache(&self, node: Node, input: &ComputeInputOf<S>) -> bool {
        self.incoming.last().is_some_and(|entry| {
            entry.node == node
                && entry
                    .request
                    .with_settled_auto_scrollbars(crate::scroll::SettledAutoScrollbarState::INITIAL)
                    == *input
        })
    }
    pub(crate) fn record(&mut self, completed: Completed<Node, S>) {
        self.completed = Some(completed);
    }
    pub(crate) fn pop(
        &mut self,
        node: Node,
        request: ComputeInputOf<S>,
    ) -> Option<Completed<Node, S>> {
        let incoming = self.incoming.pop()?;
        if incoming.node != node || incoming.request != request {
            return None;
        }
        self.completed.take().filter(|completed| {
            completed.node == node
                && completed.request == request
                && completed
                    .used
                    .with_settled_auto_scrollbars(crate::scroll::SettledAutoScrollbarState::INITIAL)
                    == request.with_settled_auto_scrollbars(
                        crate::scroll::SettledAutoScrollbarState::INITIAL,
                    )
        })
    }
}

/// The same native work proof for private executors without the public session's
/// already-validated graph. This observes boxes, never source offsets or sizes.
pub(crate) fn discover_work<
    'input,
    Tree: crate::engine::contracts::Compute<'input, M> + ?Sized,
    M,
>(
    tree: &Tree,
    node: Tree::Node,
) -> bool {
    if tree.has_leaf_measurement(node) {
        return false;
    }
    let style = tree.node_input(node);
    if style.display.inner_display() != crate::Display::Block {
        return false;
    }
    let axes = FlowAxes::new(style.writing_mode, style.direction);
    tree.children(node).any(|child| {
        let crate::LayoutInputViewOf::Box(style) = tree.layout_input(child) else {
            return false;
        };
        style.display != crate::Display::None
            && style.position != crate::Position::Absolute
            && (!style.float.is_none()
                || shares_parent_context(style, axes, ParentFormattingContext::BlockFlow)
                    && discover_work(tree, child))
    })
}
pub(crate) fn missing<Node, S: LayoutScalar, M>(node: Node) -> crate::LayoutErrorOf<Node, S, M> {
    crate::LayoutErrorOf::new(
        crate::LayoutErrorSiteOf::Node(node),
        crate::LayoutOperation::ChildLayout,
        crate::LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::RequiredBasis),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::layout_tree::PublicLayoutTreeOf;
    fn block<S: LayoutScalar>(width: f64, height: Option<f64>) -> NodeInputOf<S> {
        NodeInputOf {
            display: crate::Display::Block,
            size: crate::Size::new(
                crate::PreferredSizeOf::px(S::from_f64(width)),
                height.map_or(crate::PreferredSizeOf::AUTO, |height| {
                    crate::PreferredSizeOf::px(S::from_f64(height))
                }),
            ),
            ..NodeInputOf::default()
        }
    }
    fn floated<S: LayoutScalar>(width: f64, height: f64) -> NodeInputOf<S> {
        NodeInputOf {
            float: crate::Float::Left,
            ..block(width, Some(height))
        }
    }
    fn output<S: LayoutScalar>(
        batch: &crate::CompletedLayoutBatchOf<u32, S>,
        node: u32,
    ) -> crate::NodeOutputOf<S> {
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output()
    }
    fn fence_lane<S: LayoutScalar>() {
        let independent = NodeInputOf {
            block_flow_context: crate::BlockFlowContext::Independent,
            ..block::<S>(40.0, Some(10.0))
        };
        let clear = NodeInputOf {
            clear: crate::Clear::Both,
            ..block::<S>(100.0, Some(10.0))
        };
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 6])
            .children(1, [2, 3])
            .children(2, [])
            .children(3, [4])
            .children(4, [])
            .children(6, [])
            .style(0, block::<S>(100.0, None))
            .style(1, block(100.0, None))
            .style(2, floated(20.0, 30.0))
            .style(3, independent)
            .style(4, floated(10.0, 200.0))
            .style(6, clear);
        let request = crate::LayoutRootRequestOf::viewport(crate::Size::new(
            crate::AvailableOf::definite(S::from_f64(100.0)),
            crate::AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let batch = crate::compute_layout(&tree, 0, request).unwrap();
        // The normal block's only in-flow child is the fixed-height flow root.
        // Its own float belongs to the outer context; the 200-unit interior
        // float remains inside the independent context despite visible overflow.
        assert_eq!(output(&batch, 1).size.height, S::from_f64(10.0));
        assert_eq!(
            output(&batch, 3).location,
            crate::Point::new(S::from_f64(20.0), S::ZERO)
        );
        assert_eq!(output(&batch, 6).location.y, S::from_f64(30.0));
        assert_eq!(output(&batch, 0).size.height, S::from_f64(40.0));
        assert_eq!(output(&batch, 4).size.height, S::from_f64(200.0));
    }
    #[test]
    fn independent_descendant_overflow_does_not_escape_the_shared_float_context() {
        fence_lane::<f32>();
        fence_lane::<f64>();
    }
    #[test]
    fn shared_normal_child_preview_preserves_authored_width_and_excludes_float_height() {
        fn lane<S: LayoutScalar>() {
            use crate::engine::contracts::Compute;
            let tree = PublicLayoutTreeOf::new()
                .children(1, [2])
                .children(2, [])
                .style(1, block::<S>(100.0, None))
                .style(2, floated(20.0, 30.0));
            let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
            let input = ComputeInputOf::for_child(
                crate::RunMode::ComputeSize,
                crate::SizingMode::InherentSize,
                crate::RequestedAxis::Both,
                crate::Size::NONE,
                crate::Size::new(Some(S::from_f64(100.0)), None),
                crate::ContainingLayoutContext::new(
                    FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
                    ParentFormattingContext::BlockFlow,
                ),
                crate::Size::new(
                    crate::AvailableOf::definite(S::from_f64(100.0)),
                    crate::AvailableOf::MAX_CONTENT,
                ),
            );
            let output = session.compute_child(1, input).unwrap();
            assert_eq!(output.size, crate::Size::new(S::from_f64(100.0), S::ZERO));
        }
        lane::<f32>();
        lane::<f64>();
    }

    #[test]
    fn finite_owner_preview_contains_floats_through_fixed_shared_descendants() {
        fn lane<S: LayoutScalar>() {
            use crate::engine::contracts::Compute;
            use crate::geometry::LogicalSizeOf;

            for writing_mode in [
                crate::WritingMode::HorizontalTb,
                crate::WritingMode::VerticalRl,
                crate::WritingMode::VerticalLr,
                crate::WritingMode::SidewaysRl,
                crate::WritingMode::SidewaysLr,
            ] {
                for direction in [crate::Direction::Ltr, crate::Direction::Rtl] {
                    let axes = FlowAxes::new(writing_mode, direction);
                    let logical_box = |inline: f64, block: Option<f64>| NodeInputOf {
                        display: crate::Display::Block,
                        writing_mode,
                        direction,
                        size: axes.physical_size(LogicalSizeOf::new(
                            crate::PreferredSizeOf::px(S::from_f64(inline)),
                            block.map_or(crate::PreferredSizeOf::AUTO, |value| {
                                crate::PreferredSizeOf::px(S::from_f64(value))
                            }),
                        )),
                        ..NodeInputOf::default()
                    };
                    for independent in [false, true] {
                        for nested in [false, true] {
                            let mut owner = logical_box(100.0, None);
                            if independent {
                                owner.block_flow_context = crate::BlockFlowContext::Independent;
                            } else {
                                owner.overflow = crate::ComputedOverflow::try_new(
                                    crate::Overflow::Hidden,
                                    crate::Overflow::Hidden,
                                )
                                .unwrap();
                            }
                            let floating = NodeInputOf {
                                float: crate::Float::Left,
                                ..logical_box(20.0, Some(30.0))
                            };
                            let tree = PublicLayoutTreeOf::new().children(1, [2]).style(1, owner);
                            let tree = if nested {
                                tree.children(2, [3])
                                    .children(3, [])
                                    .style(2, logical_box(100.0, Some(5.0)))
                                    .style(3, floating)
                            } else {
                                tree.children(2, []).style(2, floating)
                            };
                            let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
                            let input = ComputeInputOf::for_child(
                                crate::RunMode::ComputeSize,
                                crate::SizingMode::InherentSize,
                                crate::RequestedAxis::Both,
                                crate::Size::NONE,
                                axes.physical_size(LogicalSizeOf::new(
                                    Some(S::from_f64(100.0)),
                                    None,
                                )),
                                crate::ContainingLayoutContext::new(
                                    axes,
                                    ParentFormattingContext::BlockFlow,
                                ),
                                axes.physical_size(LogicalSizeOf::new(
                                    crate::AvailableOf::definite(S::from_f64(100.0)),
                                    crate::AvailableOf::MAX_CONTENT,
                                )),
                            );
                            let output = session.compute_child(1, input).unwrap();
                            // The five-unit shared box owns no separate float
                            // context: its thirty-unit float reaches this BFC's
                            // auto height without changing the shared box height.
                            assert_eq!(
                                axes.logical_size(output.size),
                                LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(30.0)),
                                "{writing_mode:?} {direction:?}, independent={independent}, nested={nested}",
                            );
                        }
                    }
                }
            }
        }
        lane::<f32>();
        lane::<f64>();
    }
}
