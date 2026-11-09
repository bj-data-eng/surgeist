use crate::{LayoutErrorOf, LayoutResultOf, LayoutScalar};

/// Identifies an owned request held by the current scope driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScopeRequestId(pub(crate) usize);

/// Internal computation control. Suspension never crosses the public layout
/// error boundary and carries no borrowed formatter state.
#[derive(Debug)]
pub(crate) enum ComputeStop<Node, S: LayoutScalar, M> {
    Layout(LayoutErrorOf<Node, S, M>),
    NeedScope(ScopeRequestId),
}

impl<Node, S: LayoutScalar, M> From<LayoutErrorOf<Node, S, M>> for ComputeStop<Node, S, M> {
    fn from(error: LayoutErrorOf<Node, S, M>) -> Self {
        Self::Layout(error)
    }
}

pub(crate) type ComputeResult<Node, T, S, M> = Result<T, ComputeStop<Node, S, M>>;

pub(crate) fn ordinary<Node, T, S: LayoutScalar, M>(
    result: ComputeResult<Node, T, S, M>,
) -> LayoutResultOf<Node, T, S, M> {
    result.map_err(|stop| match stop {
        ComputeStop::Layout(error) => error,
        ComputeStop::NeedScope(_) => {
            unreachable!("ordinary Block execution has no scope driver")
        }
    })
}

/// Requests remain owned by one driver until taken for preparation.
pub(crate) struct Requests<T> {
    entries: Vec<(ScopeRequestId, T)>,
    next: usize,
}
#[derive(Debug)]
pub(crate) enum RequestInsertError {
    Allocation,
    Exhausted,
}
impl<T> Requests<T> {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            next: 0,
        }
    }
    pub(crate) fn insert(&mut self, request: T) -> Result<ScopeRequestId, RequestInsertError> {
        let next = self
            .next
            .checked_add(1)
            .ok_or(RequestInsertError::Exhausted)?;
        self.entries
            .try_reserve(1)
            .map_err(|_| RequestInsertError::Allocation)?;
        let id = ScopeRequestId(self.next);
        self.entries.push((id, request));
        self.next = next;
        Ok(id)
    }
    pub(crate) fn peek(&self, id: ScopeRequestId) -> Option<&T> {
        self.entries
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    pub(crate) fn take(&mut self, id: ScopeRequestId) -> Option<T> {
        let index = self.entries.iter().position(|(key, _)| *key == id)?;
        Some(self.entries.swap_remove(index).1)
    }
}

#[cfg(test)]
#[path = "block_scope_tests.rs"]
mod tests;

#[derive(Clone)]
pub(crate) struct Dispatch<Node, S: LayoutScalar> {
    pub(crate) scope: ReceiptId,
    pub(crate) view: SourceViewId,
    pub(crate) target: crate::LayoutTargetBasis,
    pub(crate) owners: std::sync::Arc<[Node]>,
    pub(crate) paths: std::sync::Arc<[Node]>,
    pub(crate) ready: std::rc::Rc<std::cell::RefCell<Vec<Ready<Node, S>>>>,
    pub(crate) requests: std::rc::Rc<std::cell::RefCell<Requests<Request<Node, S>>>>,
}

/// Active ordinary entries are moved into the single suspension request.
/// They are discarded after placement is resolved, not retained per descendant.
pub(crate) struct Frame<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) entry: Option<crate::block::scope::Pending<Node, S>>,
}
pub(crate) struct Request<Node, S: LayoutScalar> {
    pub(crate) scope: ReceiptId,
    pub(crate) view: SourceViewId,
    pub(crate) entry: crate::block::scope::Pending<Node, S>,
    pub(crate) frames: Vec<Frame<Node, S>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReceiptId(pub(crate) usize);

/// Source windows and performed productions belong to one native operation,
/// independently of semantic source units or the presence of principal geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SourceViewId(pub(crate) usize);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProductionId(pub(crate) usize);
#[derive(Clone, Copy)]
pub(crate) struct Adoption {
    pub(crate) receipt: ReceiptId,
    pub(crate) projection: Option<ProductionId>,
}

/// One Block-produced principal result bound to its exact placement request.
#[derive(Clone, Copy)]
pub(crate) struct Ready<Node, S: LayoutScalar> {
    pub(crate) id: ReceiptId,
    pub(crate) view: SourceViewId,
    pub(crate) projection: Option<ProductionId>,
    pub(crate) entry: crate::block::scope::Pending<Node, S>,
    pub(crate) output: crate::ComputeOutputOf<S>,
}

/// A completed producer call, retaining boundary values that influence Block
/// placement independently of the staged box geometry.
#[derive(Clone, Copy)]
pub(crate) struct Body<Node, S: LayoutScalar> {
    pub(crate) node: Node,
    pub(crate) input: crate::ComputeInputOf<S>,
    pub(crate) output: crate::ComputeOutputOf<S>,
}

/// Unpublished source-local production, without an invented principal box or
/// a premature rounding phase. Child scopes remain references by receipt ID.
pub(crate) struct Packet<Node, S: LayoutScalar> {
    pub(crate) target: crate::LayoutTargetBasis,
    pub(crate) entries: Vec<crate::LayoutOutputEntryOf<Node, S>>,
    pub(crate) margins: crate::block::fragment_margins::State<Node, S>,
    pub(crate) bodies: Vec<Body<Node, S>>,
    pub(crate) projections: Vec<(Node, ProductionId)>,
    pub(crate) receipts: Vec<ReceiptId>,
    pub(crate) clears: Vec<crate::LayoutCacheClearEntry<Node>>,
}

pub(crate) struct FinalPacket<Node, S: LayoutScalar> {
    pub(crate) target: crate::LayoutTargetBasis,
    pub(crate) entries: Vec<crate::BoxGeometryPairOf<Node, S>>,
    pub(crate) anchors: Vec<(Node, super::FragmentAnchor<S>)>,
    pub(crate) receipts: Vec<ReceiptId>,
    pub(crate) clears: Vec<crate::LayoutCacheClearEntry<Node>>,
}

impl<Node: Copy + Eq, S: LayoutScalar> Packet<Node, S> {
    pub(crate) fn round<Tree: crate::LayoutTree<Node = Node, Scalar = S>>(
        self,
        tree: &Tree,
        roots: &[(Node, super::FragmentAnchor<S>)],
    ) -> LayoutResultOf<Node, FinalPacket<Node, S>, S, Tree::MeasureError> {
        let mut rounding = PacketRound {
            tree,
            entries: &self.entries,
            final_entries: Vec::new(),
            anchors: Vec::new(),
        };
        for &(root, anchor) in roots {
            super::rounding::round_fragment_subtree(&mut rounding, root, anchor)?;
        }
        let PacketRound {
            final_entries,
            anchors,
            ..
        } = rounding;
        let entries = self
            .entries
            .into_iter()
            .map(|unrounded| {
                final_entries
                    .iter()
                    .find(|entry| entry.node() == unrounded.node())
                    .copied()
                    .map(|final_entry| crate::BoxGeometryPairOf {
                        unrounded,
                        final_entry,
                    })
                    .ok_or_else(|| {
                        crate::LayoutErrorOf::new(
                            crate::LayoutErrorSiteOf::Node(unrounded.node()),
                            crate::LayoutOperation::RoundingFinalization,
                            crate::LayoutErrorKindOf::InternalInvariant(
                                crate::LayoutInternalInvariant::MissingStagedUnroundedOutput,
                            ),
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(FinalPacket {
            target: self.target,
            entries,
            anchors,
            receipts: self.receipts,
            clears: self.clears,
        })
    }
}

struct PacketRound<'a, Tree: crate::LayoutTree> {
    tree: &'a Tree,
    entries: &'a [crate::LayoutOutputEntryOf<Tree::Node, Tree::Scalar>],
    final_entries: Vec<crate::LayoutOutputEntryOf<Tree::Node, Tree::Scalar>>,
    anchors: Vec<(Tree::Node, super::FragmentAnchor<Tree::Scalar>)>,
}
impl<Tree: crate::LayoutTree> crate::Traverse for PacketRound<'_, Tree> {
    type Node = Tree::Node;
    type Scalar = Tree::Scalar;
    type Children<'a>
        = std::vec::IntoIter<Self::Node>
    where
        Self: 'a;
    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.tree
            .children(node)
            .filter(|child| self.entries.iter().any(|entry| entry.node() == *child))
            .collect::<Vec<_>>()
            .into_iter()
    }
    fn child_count(&self, node: Self::Node) -> usize {
        self.children(node).count()
    }
    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.children(node)
            .nth(index)
            .expect("local packet child exists")
    }
}
impl<Tree: crate::LayoutTree> super::contracts::Round<Tree::MeasureError>
    for PacketRound<'_, Tree>
{
    fn unrounded(
        &self,
        node: Self::Node,
    ) -> LayoutResultOf<
        Self::Node,
        crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>,
        Self::Scalar,
        Tree::MeasureError,
    > {
        self.entries
            .iter()
            .find(|entry| entry.node() == node)
            .copied()
            .ok_or_else(|| {
                crate::LayoutErrorOf::new(
                    crate::LayoutErrorSiteOf::Node(node),
                    crate::LayoutOperation::RoundingFinalization,
                    crate::LayoutErrorKindOf::InternalInvariant(
                        crate::LayoutInternalInvariant::MissingStagedUnroundedOutput,
                    ),
                )
            })
    }
    fn set_final(&mut self, mut entry: crate::LayoutOutputEntryOf<Self::Node, Self::Scalar>) {
        let hidden = match entry.geometry_state() {
            crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } => {
                measured_normal.is_hidden()
            }
            crate::output::GeometryStateOf::OrdinaryCanonical { normal }
            | crate::output::GeometryStateOf::CanonicalAtomic { normal, .. } => normal.is_hidden(),
        };
        if hidden {
            entry.non_box = Some(crate::NonBoxKind::Hidden);
        }
        self.final_entries.push(entry);
    }
    fn record_fragment_anchor(
        &mut self,
        node: Self::Node,
        anchor: super::FragmentAnchor<Self::Scalar>,
    ) {
        self.anchors.push((node, anchor));
    }
}
