use std::sync::atomic::{AtomicU64, Ordering};

use crate::DomError;

static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);

/// Immutable identity of one exclusive store, independent of node documents.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomOwnerId(u64);

impl DomOwnerId {
    pub(crate) fn allocate() -> Result<Self, DomError> {
        Self::allocate_from(&NEXT_OWNER)
    }

    fn allocate_from(next_owner: &AtomicU64) -> Result<Self, DomError> {
        next_owner
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map(Self)
            .map_err(|_| DomError::OwnerIdentityExhausted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_counter_exhaustion_never_wraps_or_reissues_an_identity() {
        let next = AtomicU64::new(u64::MAX - 1);
        assert_eq!(
            DomOwnerId::allocate_from(&next),
            Ok(DomOwnerId(u64::MAX - 1))
        );
        assert_eq!(
            DomOwnerId::allocate_from(&next),
            Err(DomError::OwnerIdentityExhausted)
        );
        assert_eq!(
            DomOwnerId::allocate_from(&next),
            Err(DomError::OwnerIdentityExhausted)
        );
        assert_eq!(next.load(Ordering::Relaxed), u64::MAX);
    }
}

/// Store-qualified, generation-safe node identity. Fields cannot be fabricated.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId {
    pub(crate) owner: DomOwnerId,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
}

impl NodeId {
    pub fn owner(self) -> DomOwnerId {
        self.owner
    }
}

/// A checked identity of a Document node.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DocumentId(pub(crate) NodeId);

impl DocumentId {
    pub fn node(self) -> NodeId {
        self.0
    }
}

/// Checked store revision, advancing once per observably changed batch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DomRevision(pub(crate) u64);

impl DomRevision {
    pub fn get(self) -> u64 {
        self.0
    }
}

/// Exact owner/revision basis for change coverage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomStamp {
    pub(crate) owner: DomOwnerId,
    pub(crate) revision: DomRevision,
}

impl DomStamp {
    pub fn owner(self) -> DomOwnerId {
        self.owner
    }
    pub fn revision(self) -> DomRevision {
        self.revision
    }
}
