use crate::{
    Display, FlowAxes, FragmentationBreakControl, FragmentationBreakInside, FragmentationRole,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInputViewOf, LayoutOperation,
    LayoutResultOf, LayoutScalar, LayoutTree, LayoutUnsupportedCapability, LengthAutoOf, LengthOf,
    NodeInputOf, Position, SourceIndex, Traverse,
};

pub(super) struct SourceNode<Node> {
    pub(super) node: Node,
    pub(super) parent: Option<usize>,
    pub(super) children: Vec<usize>,
    pub(super) source_index: SourceIndex,
    pub(super) first: usize,
    pub(super) end: usize,
    pub(super) atomic_owner: Option<usize>,
    pub(super) hidden: bool,
}
pub(super) struct Inventory<Node> {
    pub(super) nodes: Vec<SourceNode<Node>>,
    pub(super) leaves: Vec<usize>,
}

pub(super) fn unsupported<Node, S: LayoutScalar, M>(node: Node) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        LayoutErrorSiteOf::Node(node),
        LayoutOperation::RootLayout,
        LayoutErrorKindOf::UnsupportedCapability(
            LayoutUnsupportedCapability::BlockFragmentationProfile,
        ),
    )
}

impl<Node: Copy + Eq> Inventory<Node> {
    pub(super) fn new<Tree: LayoutTree<Node = Node>>(
        tree: &Tree,
        graph: &crate::engine::validation::ValidationGraph<Node>,
        axes: FlowAxes,
    ) -> LayoutResultOf<Node, Self, Tree::Scalar, Tree::MeasureError> {
        if graph.path_counts().iter().any(|&count| count != 1) {
            return Err(unsupported(graph.nodes[0]));
        }
        let mut nodes: Vec<SourceNode<Node>> = Vec::new();
        let mut leaves = Vec::new();
        nodes.try_reserve(graph.nodes.len()).map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(graph.nodes[0]),
                LayoutOperation::RootLayout,
                LayoutErrorKindOf::Allocation,
            )
        })?;
        for (index, &node) in graph.nodes.iter().enumerate() {
            let parent = graph.parents[index].first().copied();
            let inherited = parent.and_then(|parent| nodes[parent].atomic_owner);
            let hidden = parent.is_some_and(|parent| nodes[parent].hidden)
                || tree.node_input(node).display == Display::None;
            let style = tree
                .layout_input(node)
                .as_box()
                .ok_or_else(|| unsupported(node))?;
            if !hidden
                && inherited.is_none()
                && (style.display != Display::Block
                    || style.position != Position::Relative
                    || !style.float.is_none())
            {
                return Err(unsupported(node));
            }
            let own_axes = FlowAxes::new(style.writing_mode, style.direction);
            let atomic = inherited.or_else(|| {
                (hidden
                    || tree.has_leaf_measurement(node)
                    || graph.children[index].is_empty()
                    || own_axes.block_axis() != axes.block_axis()
                    || style.item_is_replaced)
                    .then_some(index)
            });
            if index == 0
                && (style.margin != crate::Edges::all(LengthAutoOf::ZERO)
                    || style.inset != crate::Edges::all(LengthAutoOf::AUTO)
                    || hidden
                    || own_axes != axes
                    || !axes.logical_size(style.size.clone()).block.is_auto())
            {
                return Err(unsupported(node));
            }
            if !hidden
                && inherited.is_none()
                && atomic.is_none()
                && !axes.logical_size(style.size.clone()).block.is_auto()
            {
                // Fixed-height descendant overflow needs its independent parallel flow owner.
                return Err(unsupported(node));
            }
            let source_index = parent.map_or(SourceIndex::ZERO, |parent| {
                SourceIndex::new(
                    graph.children[parent]
                        .iter()
                        .position(|&child| child == index)
                        .expect("validated edge retains its actual source position"),
                )
            });
            let first = if let Some(owner) = inherited {
                nodes[owner].first
            } else {
                leaves.len()
            };
            if atomic == Some(index) {
                leaves.push(index);
            }
            nodes.push(SourceNode {
                node,
                parent,
                children: graph.children[index].clone(),
                source_index,
                first,
                end: first + usize::from(atomic.is_some()),
                atomic_owner: atomic,
                hidden,
            });
        }
        for &index in &graph.finish {
            if nodes[index].atomic_owner.is_none() {
                nodes[index].end = nodes[index]
                    .children
                    .iter()
                    .map(|&child| nodes[child].end)
                    .max()
                    .unwrap_or(nodes[index].first);
            }
        }
        Ok(Self { nodes, leaves })
    }

    pub(super) fn validate_continuation<Tree: LayoutTree<Node = Node>>(
        &self,
        tree: &Tree,
        range: &std::ops::Range<usize>,
        axes: FlowAxes,
    ) -> LayoutResultOf<Node, (), Tree::Scalar, Tree::MeasureError> {
        for entry in &self.nodes {
            let continuing = entry.first < range.end
                && entry.end > range.start
                && (range.start > entry.first || range.end < entry.end);
            if !continuing || entry.atomic_owner.is_some() {
                continue;
            }
            let style = tree.node_input(entry.node);
            let own_axes = FlowAxes::new(style.writing_mode, style.direction);
            if block_constrained(style, axes)
                || preferred_intrinsic(&own_axes.logical_size(style.size.clone()).inline)
                || minimum_intrinsic(&own_axes.logical_size(style.min_size.clone()).inline)
                || maximum_intrinsic(&own_axes.logical_size(style.max_size.clone()).inline)
                || style.overflow.x() != crate::Overflow::Visible
                || style.overflow.y() != crate::Overflow::Visible
                || (own_axes.block_axis() == axes.block_axis()
                    && own_axes.block_start() != axes.block_start())
            {
                return Err(unsupported(entry.node));
            }
        }
        Ok(())
    }

    pub(super) fn controls<Tree: LayoutTree<Node = Node>>(
        &self,
        tree: &Tree,
        boundary: usize,
    ) -> (
        Vec<FragmentationBreakControl>,
        Vec<FragmentationBreakControl>,
        Vec<FragmentationBreakInside>,
    ) {
        let mut before = Vec::new();
        let mut after = Vec::new();
        let mut inside = Vec::new();
        for (index, entry) in self.nodes.iter().enumerate() {
            if entry.hidden || entry.atomic_owner.is_some_and(|owner| owner != index) {
                continue;
            }
            let style = tree.node_input(entry.node);
            let control = |value| FragmentationBreakControl {
                value,
                role: FragmentationRole::Normal,
                source_order: index,
            };
            if entry.first == boundary {
                before.push(control(style.break_before));
            }
            if entry.end == boundary {
                after.push(control(style.break_after));
            }
            if entry.first < boundary && entry.end > boundary {
                inside.push(style.break_inside);
            }
        }
        (before, after, inside)
    }
}

/// An immutable fragment-window projection of the actual source. Only the
/// selected source interval and Slice edge/margin rules differ; Block still
/// owns sizing, margin collapse, leaf invocation, placement and baseline facts.
pub(super) struct WindowTree<'a, Tree: LayoutTree> {
    pub(super) tree: &'a Tree,
    pub(super) inventory: &'a Inventory<Tree::Node>,
    styles: Vec<NodeInputOf<Tree::Scalar>>,
    children: Vec<Vec<Tree::Node>>,
}
impl<'a, Tree: LayoutTree> WindowTree<'a, Tree> {
    pub(super) fn new(
        tree: &'a Tree,
        inventory: &'a Inventory<Tree::Node>,
        range: std::ops::Range<usize>,
        axes: FlowAxes,
        trimmed: &[crate::block::fragment_margins::MarginOwner<Tree::Node>],
    ) -> Self {
        let mut styles = Vec::with_capacity(inventory.nodes.len());
        let mut children = Vec::with_capacity(inventory.nodes.len());
        let zero_edge = |edges: &mut crate::Edges<LengthOf<Tree::Scalar>>, side| match side {
            crate::PhysicalSide::Left => edges.left = LengthOf::ZERO,
            crate::PhysicalSide::Right => edges.right = LengthOf::ZERO,
            crate::PhysicalSide::Top => edges.top = LengthOf::ZERO,
            crate::PhysicalSide::Bottom => edges.bottom = LengthOf::ZERO,
        };
        let zero_margin = |edges: &mut crate::Edges<LengthAutoOf<Tree::Scalar>>, side| match side {
            crate::PhysicalSide::Left => edges.left = LengthAutoOf::ZERO,
            crate::PhysicalSide::Right => edges.right = LengthAutoOf::ZERO,
            crate::PhysicalSide::Top => edges.top = LengthAutoOf::ZERO,
            crate::PhysicalSide::Bottom => edges.bottom = LengthAutoOf::ZERO,
        };
        for entry in &inventory.nodes {
            let mut style = tree.node_input(entry.node).clone();
            if entry.atomic_owner.is_none() {
                if entry.first < range.start {
                    zero_edge(&mut style.padding, axes.block_start());
                    zero_edge(&mut style.border, axes.block_start());
                }
                if entry.end > range.end {
                    zero_edge(&mut style.padding, axes.block_end());
                    zero_edge(&mut style.border, axes.block_end());
                }
            }
            if entry.first < range.start && entry.end > range.start {
                zero_margin(&mut style.margin, axes.block_start());
            }
            if entry.first < range.end && entry.end > range.end {
                zero_margin(&mut style.margin, axes.block_end());
            }
            for owner in trimmed.iter().filter(|owner| owner.node == entry.node) {
                zero_margin(&mut style.margin, owner.side);
            }
            children.push(
                entry
                    .children
                    .iter()
                    .copied()
                    .filter(|&child| {
                        inventory.nodes[child].first < range.end
                            && inventory.nodes[child].end > range.start
                    })
                    .map(|child| inventory.nodes[child].node)
                    .collect(),
            );
            styles.push(style);
        }
        Self {
            tree,
            inventory,
            styles,
            children,
        }
    }
    fn index(&self, node: Tree::Node) -> usize {
        self.inventory
            .nodes
            .iter()
            .position(|entry| entry.node == node)
            .expect("window traversal only uses actual discovered source nodes")
    }
}
impl<Tree: LayoutTree> Traverse for WindowTree<'_, Tree> {
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, Self::Node>>
    where
        Self: 'a;
    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.children[self.index(node)].iter().copied()
    }
    fn child_count(&self, node: Self::Node) -> usize {
        self.children[self.index(node)].len()
    }
    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children[self.index(node)][index]
    }
}
impl<Tree: LayoutTree> LayoutTree for WindowTree<'_, Tree> {
    type MeasureError = Tree::MeasureError;
    fn node_input(&self, node: Self::Node) -> &NodeInputOf<Self::Scalar> {
        &self.styles[self.index(node)]
    }
    fn layout_input(&self, node: Self::Node) -> LayoutInputViewOf<'_, Self::Scalar> {
        LayoutInputViewOf::Box(self.node_input(node))
    }
    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        self.tree.has_leaf_measurement(node)
    }
    fn measure_leaf(
        &self,
        node: Self::Node,
        input: crate::LeafMeasureInputOf<Self::Scalar>,
    ) -> Option<Result<crate::Size<Self::Scalar>, Self::MeasureError>> {
        self.tree.measure_leaf(node, input)
    }
}

fn preferred_intrinsic<S: LayoutScalar>(value: &crate::PreferredSizeOf<S>) -> bool {
    use crate::sizing::{PreferredSizeCalcBasis as B, PreferredSizeView as V};
    match value.view() {
        V::MinContent | V::MaxContent | V::FitContent | V::FitContentFunction(_) | V::Contain => {
            true
        }
        V::CalcSize(basis, calculation) => {
            calculation.depends_on_size()
                && matches!(
                    basis,
                    B::MinContent | B::MaxContent | B::FitContent | B::Contain
                )
        }
        _ => false,
    }
}
fn minimum_intrinsic<S: LayoutScalar>(value: &crate::MinSizeOf<S>) -> bool {
    use crate::sizing::{MinSizeCalcBasis as B, MinSizeView as V};
    match value.view() {
        V::MinContent | V::MaxContent | V::FitContent | V::FitContentFunction(_) | V::Contain => {
            true
        }
        V::CalcSize(basis, calculation) => {
            calculation.depends_on_size()
                && matches!(
                    basis,
                    B::MinContent | B::MaxContent | B::FitContent | B::Contain
                )
        }
        _ => false,
    }
}
fn maximum_intrinsic<S: LayoutScalar>(value: &crate::MaxSizeOf<S>) -> bool {
    use crate::sizing::{MaxSizeCalcBasis as B, MaxSizeView as V};
    match value.view() {
        V::MinContent | V::MaxContent | V::FitContent | V::FitContentFunction(_) | V::Contain => {
            true
        }
        V::CalcSize(basis, calculation) => {
            calculation.depends_on_size()
                && matches!(
                    basis,
                    B::MinContent | B::MaxContent | B::FitContent | B::Contain
                )
        }
        _ => false,
    }
}

pub(super) fn block_constrained<S: LayoutScalar>(style: &NodeInputOf<S>, axes: FlowAxes) -> bool {
    axes.logical_size(style.min_size.clone()).block != crate::MinSizeOf::AUTO
        || axes.logical_size(style.max_size.clone()).block != crate::MaxSizeOf::NONE
        || style.aspect_ratio.is_some()
}
