//! Identities and results for live, ordinary-tree traversal.

use crate::{internal::identity::DomOwnerId, internal::identity::NodeId};

/// Identity of one live iterator in its canonical DOM owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomNodeIteratorId {
    pub(crate) owner: DomOwnerId,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}

impl DomNodeIteratorId {
    pub fn owner(self) -> DomOwnerId {
        self.owner
    }
}

/// Whether traversing a matching node invokes the iterator's filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomNodeIteratorFilter {
    None,
    Callback,
}

/// The public cursor state at the time it was read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNodeIterator {
    pub(crate) id: DomNodeIteratorId,
    pub(crate) root: NodeId,
    pub(crate) reference: NodeId,
    pub(crate) before: bool,
    pub(crate) mask: u32,
    pub(crate) filter: DomNodeIteratorFilter,
}

impl DomNodeIterator {
    pub fn id(self) -> DomNodeIteratorId {
        self.id
    }

    pub fn root(self) -> NodeId {
        self.root
    }

    pub fn reference_node(self) -> NodeId {
        self.reference
    }

    pub fn pointer_before_reference_node(self) -> bool {
        self.before
    }

    pub fn what_to_show(self) -> u32 {
        self.mask
    }

    pub fn filter(self) -> DomNodeIteratorFilter {
        self.filter
    }
}

/// Traversal completion, unfiltered output preparation, or a reached filter callback.
#[must_use]
pub enum DomNodeIteratorProgress<A = fn() -> Result<(), crate::DomError>> {
    Complete(Option<NodeId>),
    Match(crate::internal::store::DomNodeIteratorMatchRequest<A>),
    Filter(crate::internal::store::DomNodeIteratorFilterRequest<A>),
}

impl<A> std::fmt::Debug for DomNodeIteratorProgress<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Complete(node) => f.debug_tuple("Complete").field(node).finish(),
            Self::Match(request) => f.debug_tuple("Match").field(request).finish(),
            Self::Filter(request) => f.debug_tuple("Filter").field(request).finish(),
        }
    }
}

/// The DOM NodeFilter decision. For NodeIterator, Reject and Skip both continue
/// traversal without pruning descendants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomNodeFilterDecision {
    Accept,
    Reject,
    Skip,
}
