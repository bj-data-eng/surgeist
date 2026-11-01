//! Complete native family admission, retained on the actual ordering profile.
mod views;
use super::edge_plan::AdmittedGeneratedBoxProof;
use super::*;
use crate::{LayoutInputViewOf, LayoutTree, engine::validation::ValidationGraph};
use std::ops::Range;
pub(crate) use views::{NativeFormatterCompletion, NativePublication, pieces};
pub use views::{
    NativeInlineBoxEdgeView, NativeInlineBoxFamily, NativeInlineBoxFragment,
    NativeInlineBoxMemberView, NativeInlineBoxPieceView,
};

#[derive(Debug)]
pub(crate) struct NativeFamilyData {
    pub(crate) proof: Arc<super::edge_plan::GeneratedBoxProofData>,
    pub(crate) root: usize,
    pub(crate) scope_keys: Vec<Option<usize>>,
    pub(crate) components: surgeist_text::SuppliedInlineBoxComponents,
    pub(crate) rows: Vec<NativeFamilyRow>,
    pub(crate) row_for_rank: Vec<usize>,
    pub(crate) empties: Vec<super::ordering::FamilyEmpty>,
    pub(crate) ports: Vec<Option<(usize, usize)>>,
    pub(crate) body_ports: Vec<(usize, usize)>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativeFamilyRow {
    Operand {
        rank: usize,
        ordinal: usize,
        body: usize,
    },
    Metadata,
    Empty {
        owner: usize,
        body: usize,
    },
}
pub(super) struct Admission {
    pub(super) root: usize,
    pub(super) active: Vec<usize>,
    pub(super) scope_keys: Vec<Option<usize>>,
}
fn push<T>(rows: &mut Vec<T>, value: T) -> Result<(), WrappingScopeError> {
    rows.try_reserve(1)
        .map_err(|_| WrappingScopeError::Allocation)?;
    rows.push(value);
    Ok(())
}
impl<Node: Copy + Eq> AdmittedGeneratedBoxProof<Node> {
    pub(super) fn family<Tree: LayoutTree<Node = Node>>(
        &self,
        tree: &Tree,
        graph: &ValidationGraph<Node>,
        scopes: &[ScopeRow<Node>],
        flow: Node,
    ) -> Result<Admission, WrappingScopeError> {
        let index = graph
            .index(flow)
            .ok_or(WrappingScopeError::NodeNotReachable)?;
        let input = tree
            .layout_input(flow)
            .as_box()
            .ok_or(WrappingScopeError::DifferentFlow)?;
        if tree.has_leaf_measurement(flow) {
            return Err(WrappingScopeError::UnsupportedIncidence);
        }
        let mut ancestor = index;
        loop {
            if tree
                .layout_input(graph.nodes[ancestor])
                .as_box()
                .is_some_and(|b| b.display == crate::Display::None)
            {
                return Err(WrappingScopeError::UnsupportedIncidence);
            }
            let Some(&parent) = graph.parents[ancestor].first() else {
                break;
            };
            ancestor = parent;
        }
        if input.display.inner_display() != crate::Display::Block
            || input.display == crate::Display::None
        {
            return Err(WrappingScopeError::DifferentFlow);
        }
        let root = self
            .boxes
            .iter()
            .position(|b| b.box_ == GeneratedBoxRef::Node(flow))
            .ok_or(WrappingScopeError::MissingChildOrder)?;
        let mut active = Vec::new();
        let mut visited = Vec::new();
        let mut pending = Vec::new();
        push(&mut pending, root)?;
        while let Some(owner) = pending.pop() {
            if visited.contains(&owner) {
                return Err(WrappingScopeError::ChildOrder);
            }
            push(&mut visited, owner)?;
            let fact = &self.boxes[owner];
            if !fact.unique {
                return Err(WrappingScopeError::AmbiguousNode);
            }
            let normal = owner == root || matches!(fact.box_, GeneratedBoxRef::Inline(_));
            if !normal {
                continue;
            }
            push(&mut active, owner)?;
            let order = self
                .data
                .orders
                .iter()
                .find(|order| order.parent == owner)
                .ok_or(WrappingScopeError::MissingChildOrder)?;
            if order.prefix != ChildOrderCompleteness::Complete
                || order.suffix != ChildOrderCompleteness::Complete
            {
                return Err(WrappingScopeError::IncompleteFamily);
            }
            for &(child, _) in order.children.iter().rev() {
                push(&mut pending, child)?;
            }
        }
        // All generated children belonging to this formatter must be in the
        // declared closure. Opaque children are leaves, even when they format
        // a different, separately demanded family. The designated root's own
        // original global parent is intentionally outside this traversal.
        for (key, fact) in self.boxes.iter().enumerate() {
            if fact.flow == flow && key != root && !visited.contains(&key) {
                return Err(WrappingScopeError::IncompleteFamily);
            }
        }
        for &child in &graph.children[index] {
            if graph.parents[child].len() != 1
                || graph.children[index]
                    .iter()
                    .filter(|&&n| n == child)
                    .count()
                    != 1
            {
                return Err(WrappingScopeError::AmbiguousNode);
            }
            if let LayoutInputViewOf::Box(input) = tree.layout_input(graph.nodes[child])
                && input.display != crate::Display::None
            {
                let key = self
                    .boxes
                    .iter()
                    .position(|b| b.box_ == GeneratedBoxRef::Node(graph.nodes[child]))
                    .ok_or(WrappingScopeError::IncompleteFamily)?;
                if !visited.contains(&key) {
                    return Err(WrappingScopeError::IncompleteFamily);
                }
            }
        }
        let mut scope_keys = Vec::new();
        scope_keys
            .try_reserve_exact(scopes.len())
            .map_err(|_| WrappingScopeError::Allocation)?;
        for scope in scopes {
            let key = if scope.flow == flow {
                let identity = GeneratedBoxRef::Inline(if let Some(capture) = &scope.capture {
                    InlineBoxRef::Scope(capture.clone())
                } else {
                    let (start, end) = scope.pair.ok_or(WrappingScopeError::IncompleteFamily)?;
                    InlineBoxRef::NativePair { start, end }
                });
                let key = self
                    .boxes
                    .iter()
                    .position(|b| b.box_ == identity)
                    .ok_or(WrappingScopeError::IncompleteFamily)?;
                if !active.contains(&key) {
                    return Err(WrappingScopeError::IncompleteFamily);
                }
                Some(key)
            } else {
                None
            };
            scope_keys.push(key);
        }
        // Native containment must match the original normalized child proof;
        // Complete on an arbitrary root list cannot authenticate omissions or
        // a skipped normal ancestor. Captured identities already passed the
        // canonical capture/order adapter before this retained proof was made.
        for &key in &visited {
            if key == root {
                continue;
            }
            let fact = &self.boxes[key];
            if let Some(span) = &fact.native {
                let mut expected = root;
                let mut width = usize::MAX;
                for &candidate in &active {
                    if candidate == key || candidate == root {
                        continue;
                    }
                    if let Some(parent) = &self.boxes[candidate].native
                        && parent.first < span.first
                        && span.last < parent.last
                        && parent.last - parent.first < width
                    {
                        expected = candidate;
                        width = parent.last - parent.first;
                    }
                }
                if self.data.parents[key] != Some(expected)
                    && !matches!(
                        self.boxes[self.data.parents[key].ok_or(WrappingScopeError::ChildOrder)?]
                            .box_,
                        GeneratedBoxRef::Inline(InlineBoxRef::Scope(_))
                    )
                {
                    return Err(WrappingScopeError::ChildOrder);
                }
            }
        }
        Ok(Admission {
            root,
            active,
            scope_keys,
        })
    }
    pub(super) fn captured_empty_leaf(&self, key: usize) -> bool {
        self.captured_orders_validated
            && matches!(
                self.boxes[key].box_,
                GeneratedBoxRef::Inline(InlineBoxRef::Scope(_))
            )
            && self.data.orders.iter().any(|order| {
                order.parent == key
                    && order.children.is_empty()
                    && order.prefix == ChildOrderCompleteness::Complete
                    && order.suffix == ChildOrderCompleteness::Complete
            })
    }

    /// Authenticate an attached source cut against the original generated order.
    /// The cut is a formatter frontier, never a synthetic span for this scope.
    pub(super) fn validate_attachment(
        &self,
        key: usize,
        root: usize,
        cut: usize,
    ) -> Result<(), WrappingScopeError> {
        if !self.captured_empty_leaf(key) {
            return Err(WrappingScopeError::UnsupportedIncidence);
        }
        let mut child = key;
        for _ in 0..=self.data.parents.len() {
            if child == root {
                return Ok(());
            }
            let parent = self.data.parents[child].ok_or(WrappingScopeError::IncompleteFamily)?;
            let order = self
                .data
                .orders
                .iter()
                .find(|order| order.parent == parent)
                .ok_or(WrappingScopeError::MissingChildOrder)?;
            let at = order
                .children
                .iter()
                .position(|&(owner, _)| owner == child)
                .ok_or(WrappingScopeError::ChildOrder)?;
            if order.children[..at].iter().any(|&(owner, _)| {
                self.boxes[owner]
                    .native
                    .as_ref()
                    .is_some_and(|span| cut <= span.last)
            }) || order.children[at + 1..].iter().any(|&(owner, _)| {
                self.boxes[owner]
                    .native
                    .as_ref()
                    .is_some_and(|span| span.first < cut)
            }) {
                return Err(WrappingScopeError::InvalidGapAnchor);
            }
            if parent != root
                && let Some(span) = &self.boxes[parent].native
                && (cut <= span.first || span.last < cut)
            {
                return Err(WrappingScopeError::InvalidGapAnchor);
            }
            child = parent;
        }
        Err(WrappingScopeError::ChildOrder)
    }

    pub(super) fn nearest(
        &self,
        key: usize,
        admission: &Admission,
    ) -> Result<usize, WrappingScopeError> {
        let mut at = key;
        let mut visits = 0;
        loop {
            if admission.active.contains(&at) {
                return Ok(at);
            }
            visits += 1;
            if visits > self.data.parents.len() {
                return Err(WrappingScopeError::ChildOrder);
            }
            at = self.data.parents[at].ok_or(WrappingScopeError::ChildOrder)?;
        }
    }
    pub(crate) fn identity(&self, key: usize) -> &GeneratedBoxRef<Node> {
        &self.boxes[key].box_
    }
}

/// Exact accepted occurrence receipt; candidate and measurement paths do not
/// publish it. All coordinates come from that line's executed native slots.
#[derive(Clone, Debug)]
pub(crate) struct NativeLineReceipt<S: crate::LayoutScalar> {
    pub(crate) profile: Arc<super::ordering::NativeOrderingData>,
    pub(crate) source: Range<usize>,
    pub(crate) body: usize,
    pub(crate) line: usize,
    pub(crate) axes: crate::FlowAxes,
    pub(crate) receipt: Arc<super::ordering::NativeOrderingProjection>,
    pub(crate) ranks: Vec<usize>,
    pub(crate) rows: Vec<usize>,
    pub(crate) edges: Vec<NativeExecutedEdge<S>>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NativeExecutedEdge<S: crate::LayoutScalar> {
    pub(crate) rank: usize,
    pub(crate) owner: usize,
    pub(crate) kind: crate::InlineBoundaryKind,
    pub(crate) alias: bool,
    pub(crate) component: usize,
    pub(crate) flow: (S, S),
    pub(crate) strip: (S, S),
}

impl<S: crate::LayoutScalar> PartialEq for NativeLineReceipt<S> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.profile, &other.profile)
            && self.receipt == other.receipt
            && self.source == other.source
            && self.body == other.body
            && self.line == other.line
            && self.axes == other.axes
            && self.ranks == other.ranks
            && self.rows == other.rows
            && self.edges == other.edges
    }
}
