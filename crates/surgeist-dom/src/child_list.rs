//! Native child-list observer identities, admission and immutable records.

use crate::{DomError, DomOwnerId, DomResource, NodeId, store::SharedVec};

/// Finite capacity of the canonical child-list observer bank.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DomChildListObservationLimits {
    pub(crate) observers: usize,
    pub(crate) registrations: usize,
    pub(crate) records: usize,
    pub(crate) references: usize,
}

impl DomChildListObservationLimits {
    /// The bounds count retained observer slots, live direct and transient
    /// registrations, pending records and their retained node references.
    pub fn new(
        observers: usize,
        registrations: usize,
        records: usize,
        references: usize,
    ) -> Result<Self, DomError> {
        for (count, resource) in [
            (observers, DomResource::ChildListObservers),
            (registrations, DomResource::ChildListRegistrations),
            (records, DomResource::ChildListRecords),
            (references, DomResource::ChildListRecordReferences),
        ] {
            if count > isize::MAX as usize / size_of::<NodeId>() {
                return Err(DomError::InvalidLimit(resource));
            }
        }
        Ok(Self {
            observers,
            registrations,
            records,
            references,
        })
    }

    pub fn observers(self) -> usize {
        self.observers
    }

    pub fn registrations(self) -> usize {
        self.registrations
    }

    pub fn records(self) -> usize {
        self.records
    }

    pub fn references(self) -> usize {
        self.references
    }
}

/// An observer identity qualified by its canonical owner and slot generation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomChildListObserverId {
    pub(crate) owner: DomOwnerId,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}

impl DomChildListObserverId {
    pub fn owner(self) -> DomOwnerId {
        self.owner
    }
}

/// An immutable record of one reached child-list mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomChildListRecord {
    pub(crate) target: NodeId,
    pub(crate) added: SharedVec<NodeId>,
    pub(crate) removed: SharedVec<NodeId>,
    pub(crate) previous: Option<NodeId>,
    pub(crate) next: Option<NodeId>,
}

impl DomChildListRecord {
    pub fn target(&self) -> NodeId {
        self.target
    }

    pub fn added_nodes(&self) -> &[NodeId] {
        &self.added
    }

    pub fn removed_nodes(&self) -> &[NodeId] {
        &self.removed
    }

    pub fn previous_sibling(&self) -> Option<NodeId> {
        self.previous
    }

    pub fn next_sibling(&self) -> Option<NodeId> {
        self.next
    }
}
