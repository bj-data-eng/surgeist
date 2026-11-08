use crate::{
    Display, FlowAxes, FragmentationBreakControl, FragmentationBreakInside, FragmentationRole,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutOperation, LayoutResultOf,
    LayoutScalar, LayoutTree, LayoutUnsupportedCapability, LengthAutoOf, LengthOf, NodeInputOf,
    Position, SourceIndex,
};

pub(super) struct SourceNode<Node> {
    pub(super) node: Node,
    pub(super) parent: Option<usize>,
    pub(super) children: Vec<usize>,
    pub(super) source_index: SourceIndex,
    pub(super) role: crate::BlockGeneration,
    pub(super) first: usize,
    pub(super) end: usize,
    pub(super) atomic_owner: Option<usize>,
    pub(super) hidden: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SourceKey<Node> {
    Atomic(Node),
    Owner(Node),
    PrincipalEnd(Node),
    BoxEnd(Node),
}
pub(super) struct Inventory<Node> {
    pub(super) nodes: Vec<SourceNode<Node>>,
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
        let mut atomic_count = 0;
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
                atomic_count
            };
            if atomic == Some(index) {
                atomic_count += 1;
            }
            nodes.push(SourceNode {
                node,
                parent,
                children: graph.children[index].clone(),
                source_index,
                role: style.block_generation,
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
        Ok(Self { nodes })
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
            let propagated = |start: bool| {
                let mut owner = index;
                while let Some(parent) = self.nodes[owner].parent {
                    if self.nodes[parent].atomic_owner.is_some() {
                        break;
                    }
                    let mut children = self.nodes[parent]
                        .children
                        .iter()
                        .copied()
                        .filter(|&child| !self.nodes[child].hidden);
                    let edge = if start {
                        children.next()
                    } else {
                        children.next_back()
                    };
                    if edge != Some(owner) {
                        break;
                    }
                    owner = parent;
                }
                if start {
                    self.nodes[owner].first
                } else {
                    self.nodes[owner].end
                }
            };
            if propagated(true) == boundary {
                before.push(control(style.break_before));
            }
            if propagated(false) == boundary {
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
pub(super) fn clear_edge<S: LayoutScalar>(
    edges: &mut crate::Edges<LengthOf<S>>,
    side: crate::PhysicalSide,
) {
    match side {
        crate::PhysicalSide::Left => edges.left = LengthOf::ZERO,
        crate::PhysicalSide::Right => edges.right = LengthOf::ZERO,
        crate::PhysicalSide::Top => edges.top = LengthOf::ZERO,
        crate::PhysicalSide::Bottom => edges.bottom = LengthOf::ZERO,
    }
}
pub(super) fn clear_margin<S: LayoutScalar>(
    edges: &mut crate::Edges<LengthAutoOf<S>>,
    side: crate::PhysicalSide,
) {
    match side {
        crate::PhysicalSide::Left => edges.left = LengthAutoOf::ZERO,
        crate::PhysicalSide::Right => edges.right = LengthAutoOf::ZERO,
        crate::PhysicalSide::Top => edges.top = LengthAutoOf::ZERO,
        crate::PhysicalSide::Bottom => edges.bottom = LengthAutoOf::ZERO,
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

#[derive(Clone, Copy)]
pub(super) enum ContinuationProfile {
    Minimum,
    Maximum,
}

/// Static admission to one owned continuation profile. Sources outside it
/// retain ordinary whole-box production and are refused only if they continue.
pub(super) fn unsupported_continuation<S: LayoutScalar>(
    style: &NodeInputOf<S>,
    axes: FlowAxes,
    profile: ContinuationProfile,
) -> Result<bool, crate::sizing::resolve::SizingResolutionError<S>> {
    let own_axes = FlowAxes::new(style.writing_mode, style.direction);
    let minimum = own_axes.logical_size(style.min_size.clone()).block;
    let maximum = own_axes.logical_size(style.max_size.clone()).block;
    let maximum_supported = match profile {
        ContinuationProfile::Minimum => maximum == crate::MaxSizeOf::NONE,
        ContinuationProfile::Maximum => matches!(
            crate::sizing::resolve::fixed_fallback_maximum(
                &maximum,
                crate::SizingAlgorithm::Block,
                own_axes.block_axis(),
                None,
            )?,
            crate::sizing::resolve::FixedFallbackSource::Fixed(_)
        ),
    };
    let inset_limit = matches!(profile, ContinuationProfile::Maximum) || !minimum.is_auto();
    Ok((!minimum.is_auto()
        && !crate::block::fragment_minimum::absolute(&minimum, own_axes.block_axis())?)
        || !maximum_supported
        || style.aspect_ratio.is_some()
        || (inset_limit
            && crate::block::fragment_minimum::basis_dependent_border_box(style, own_axes))
        || (preferred_intrinsic(&own_axes.logical_size(style.size.clone()).inline)
            && !own_axes
                .logical_size(style.size.clone())
                .inline
                .is_min_content()
            && !own_axes
                .logical_size(style.size.clone())
                .inline
                .is_max_content())
        || minimum_intrinsic(&own_axes.logical_size(style.min_size.clone()).inline)
        || maximum_intrinsic(&own_axes.logical_size(style.max_size.clone()).inline)
        || style.overflow.x() != crate::Overflow::Visible
        || style.overflow.y() != crate::Overflow::Visible
        || (own_axes.block_axis() == axes.block_axis()
            && own_axes.block_start() != axes.block_start()))
}
