//! Exact association custody for an already-started dispatch continuation.
use std::{
    fmt,
    mem::size_of,
    sync::{Arc, Mutex, OnceLock, Weak},
};

use crate::{DocumentId, DomDocumentStamp, DomError, DomOwnerId, DomResource, NodeId};

/// Aggregate admission across every dispatch registry using one canonical DOM.
/// The default has zero capacity: dispatch custody must be configured explicitly.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DomDispatchCustodyLimits {
    pub(crate) scopes: usize,
    pub(crate) references: usize,
    pub(crate) retained_bytes: usize,
    pub(crate) scratch_bytes: usize,
    pub(crate) work: u64,
}
impl DomDispatchCustodyLimits {
    pub fn new(
        scopes: usize,
        references: usize,
        retained_bytes: usize,
        scratch_bytes: usize,
        work: u64,
    ) -> Result<Self, DomError> {
        for (value, width, resource) in [
            (
                scopes,
                size_of::<std::sync::Weak<Scope>>(),
                DomResource::DispatchCustodyScopes,
            ),
            (
                references,
                size_of::<Row>(),
                DomResource::DispatchCustodyReferences,
            ),
            (retained_bytes, 1, DomResource::DispatchCustodyRetainedBytes),
            (scratch_bytes, 1, DomResource::DispatchCustodyScratch),
        ] {
            if value > isize::MAX as usize / width {
                return Err(DomError::InvalidLimit(resource));
            }
        }
        Ok(Self {
            scopes,
            references,
            retained_bytes,
            scratch_bytes,
            work,
        })
    }
    pub fn scopes(self) -> usize {
        self.scopes
    }
    pub fn references(self) -> usize {
        self.references
    }
    pub fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
    pub fn scratch_bytes(self) -> usize {
        self.scratch_bytes
    }
    pub fn work(self) -> u64 {
        self.work
    }
}

/// Current association or original-owner evidence of successful retirement.
/// A retirement stamp grants no current Document or Global execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDispatchAssociation {
    Current(DocumentId),
    Retired(DomDocumentStamp),
}
impl DomDispatchAssociation {
    pub fn document(self) -> DocumentId {
        match self {
            Self::Current(document) => document,
            Self::Retired(stamp) => stamp.document(),
        }
    }
}

/// Affine custody of exact invocation targets. Dropping it releases its rows.
/// Association observations require the issuing canonical [`crate::Dom`].
pub struct DomDispatchCustody {
    pub(crate) scope: Arc<Scope>,
}
impl DomDispatchCustody {
    /// Maximum producer-owned payload for the requested number of target rows.
    /// Includes the scope header, admitted release node, weak ledger backing
    /// occurrence and exact boxed rows, excluding opaque allocator metadata.
    /// The DOM charges its index and its ledger occurrence separately.
    pub fn maximum_retained_bytes(targets: usize) -> Result<usize, DomError> {
        targets
            .checked_mul(size_of::<Row>())
            .and_then(|bytes| bytes.checked_add(size_of::<Scope>()))
            .and_then(|bytes| bytes.checked_add(size_of::<ReleaseNode>()))
            .and_then(|bytes| bytes.checked_add(size_of::<Mutex<Ledger>>()))
            .ok_or(DomError::RepresentationExhausted)
    }
    /// Actual producer-owned scope payload, using the same units as the maximum.
    pub fn retained_bytes(&self) -> usize {
        // Construction already checked this exact extent.
        size_of::<Scope>()
            + size_of::<ReleaseNode>()
            + size_of::<Mutex<Ledger>>()
            + self.scope.rows.len() * size_of::<Row>()
    }
    pub fn targets(&self) -> usize {
        self.scope.rows.len()
    }
}
impl fmt::Debug for DomDispatchCustody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never expose retirement cells independently from the original &Dom.
        f.debug_struct("DomDispatchCustody")
            .field("targets", &self.targets())
            .finish_non_exhaustive()
    }
}

pub(crate) struct Scope {
    pub(crate) owner: DomOwnerId,
    pub(crate) rows: Box<[Row]>,
    pub(crate) release: OnceLock<Release>,
}

/// One node is allocated before registration and transferred on external Drop.
/// Its backing remains charged until the owner actually prunes the weak scope.
pub(crate) struct ReleaseNode {
    pub(crate) scope: usize,
    pub(crate) next: Option<Box<[ReleaseNode]>>,
}
pub(crate) struct Release {
    pub(crate) ledger: Weak<Mutex<Ledger>>,
    pub(crate) node: Box<[ReleaseNode]>,
}
#[derive(Default)]
pub(crate) struct Ledger {
    pub(crate) references: usize,
    pub(crate) live_scopes: usize,
    pub(crate) released: Option<Box<[ReleaseNode]>>,
}
impl Ledger {
    pub(crate) fn discard(mut released: Option<Box<[ReleaseNode]>>) {
        // An arbitrarily long abandoned-scope list must not recurse on Drop.
        while let Some(mut node) = released {
            released = node[0].next.take();
        }
    }
}
impl Drop for Ledger {
    fn drop(&mut self) {
        Self::discard(self.released.take());
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let Some(Release {
            ledger: weak,
            mut node,
        }) = self.release.take()
        else {
            return;
        };
        let Some(ledger) = weak.upgrade() else {
            return;
        };
        let mut ledger = ledger.lock().unwrap_or_else(|error| error.into_inner());
        let count = self.rows.len();
        // Release the rows under the same lock used by aggregate observations.
        drop(std::mem::take(&mut self.rows));
        drop(weak);
        ledger.references = ledger
            .references
            .checked_sub(count)
            .expect("registered dispatch rows retain their ledger debit");
        ledger.live_scopes = ledger
            .live_scopes
            .checked_sub(1)
            .expect("initialized weak ledger occurrence");
        node[0].next = ledger.released.take();
        ledger.released = Some(node);
    }
}
pub(crate) struct Row {
    pub(crate) target: NodeId,
    pub(crate) retired: OnceLock<DomDocumentStamp>,
}
