//! Identities and results for the canonical DOM TreeWalker.

use std::fmt;

use crate::internal::identity::{DomOwnerId, NodeId};
use crate::internal::store::{DomTreeWalkerFilterRequest, DomTreeWalkerMatchRequest};

/// Identity of a live walker in its canonical DOM owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomTreeWalkerId {
    pub(crate) owner: DomOwnerId,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}

impl DomTreeWalkerId {
    pub fn owner(self) -> DomOwnerId {
        self.owner
    }
}

/// Whether candidates admitted by the mask reach an external filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomTreeWalkerFilter {
    None,
    Callback,
}

/// One observation of a live walker's canonical cursor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomTreeWalker {
    pub(crate) id: DomTreeWalkerId,
    pub(crate) root: NodeId,
    pub(crate) current: NodeId,
    pub(crate) mask: u32,
    pub(crate) filter: DomTreeWalkerFilter,
}

impl DomTreeWalker {
    pub fn id(self) -> DomTreeWalkerId {
        self.id
    }

    pub fn root(self) -> NodeId {
        self.root
    }

    pub fn current_node(self) -> NodeId {
        self.current
    }

    pub fn what_to_show(self) -> u32 {
        self.mask
    }

    pub fn filter(self) -> DomTreeWalkerFilter {
        self.filter
    }
}

/// Navigation from the walker's current node through the ordinary tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomTreeWalkerDirection {
    Parent,
    FirstChild,
    LastChild,
    PreviousSibling,
    NextSibling,
    PreviousNode,
    NextNode,
}

/// A finished search, an accepted match to prepare, or a reached filter call.
#[must_use]
pub enum DomTreeWalkerProgress<'cancel, A> {
    NoMatch,
    Match(DomTreeWalkerMatchRequest<'cancel, A>),
    Filter(DomTreeWalkerFilterRequest<'cancel, A>),
}

impl<A> fmt::Debug for DomTreeWalkerProgress<'_, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoMatch => f.write_str("NoMatch"),
            Self::Match(request) => f.debug_tuple("Match").field(request).finish(),
            Self::Filter(request) => f.debug_tuple("Filter").field(request).finish(),
        }
    }
}
