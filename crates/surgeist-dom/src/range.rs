//! Canonical native live Range identities and immutable boundary observations.
use std::sync::Arc;

use crate::{DomOwnerId, NodeId};

/// A store-qualified, generation-safe live Range identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomRangeId {
    pub(crate) owner: DomOwnerId,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}
impl DomRangeId {
    pub fn owner(self) -> DomOwnerId {
        self.owner
    }
}

/// A creation ticket belonging to exactly one batch.
#[derive(Clone, Debug)]
pub struct CreatedRange {
    pub(crate) owner: Arc<crate::batch::BatchIdentity>,
    pub(crate) index: usize,
}
#[derive(Clone, Debug)]
pub enum DomRangeOperand {
    Existing(DomRangeId),
    Created(CreatedRange),
}
impl From<DomRangeId> for DomRangeOperand {
    fn from(id: DomRangeId) -> Self {
        Self::Existing(id)
    }
}
impl From<&CreatedRange> for DomRangeOperand {
    fn from(ticket: &CreatedRange) -> Self {
        Self::Created(ticket.clone())
    }
}

/// An exact boundary in the snapshot in which it was captured.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomRangeBoundary {
    pub(crate) container: NodeId,
    pub(crate) offset: usize,
}
impl DomRangeBoundary {
    pub fn container(self) -> NodeId {
        self.container
    }
    pub fn offset(self) -> usize {
        self.offset
    }
}

/// Qualification of a Document's one explicit native active association.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomActiveRangeState {
    Unavailable,
    Known(Option<DomRangeId>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Boundary<N> {
    pub container: N,
    pub offset: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Record<N> {
    pub start: Boundary<N>,
    pub end: Boundary<N>,
}
impl<N: Copy> Record<N> {
    pub(crate) fn map<M>(&self, f: impl Fn(N) -> M) -> Record<M> {
        Record {
            start: Boundary {
                container: f(self.start.container),
                offset: self.start.offset,
            },
            end: Boundary {
                container: f(self.end.container),
                offset: self.end.offset,
            },
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Slot<N> {
    pub generation: u64,
    pub retired: bool,
    pub record: Option<Record<N>>,
}

/// A borrowed historical view; later mutation or retirement cannot change it.
#[derive(Clone, Copy, Debug)]
pub struct DomRange<'a> {
    pub(crate) id: DomRangeId,
    pub(crate) record: &'a Record<NodeId>,
}
impl DomRange<'_> {
    pub fn id(self) -> DomRangeId {
        self.id
    }
    pub fn start(self) -> DomRangeBoundary {
        DomRangeBoundary {
            container: self.record.start.container,
            offset: self.record.start.offset,
        }
    }
    pub fn end(self) -> DomRangeBoundary {
        DomRangeBoundary {
            container: self.record.end.container,
            offset: self.record.end.offset,
        }
    }
    pub fn collapsed(self) -> bool {
        self.record.start == self.record.end
    }
}
