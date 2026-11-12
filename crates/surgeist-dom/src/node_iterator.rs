//! Identities and results for live, ordinary-tree traversal.

use crate::{DomOwnerId, NodeId};

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

/// The result of advancing, or the exact callback awaiting a response.
#[derive(Debug)]
#[must_use]
pub enum DomNodeIteratorProgress {
    Complete(Option<NodeId>),
    Filter(crate::store::DomNodeIteratorFilterRequest),
}

/// The DOM NodeFilter decision. For NodeIterator, Reject and Skip both continue
/// traversal without pruning descendants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomNodeFilterDecision {
    Accept,
    Reject,
    Skip,
}
