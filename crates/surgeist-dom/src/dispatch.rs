//! Exact association custody for an already-started dispatch continuation.
use std::{
    fmt,
    mem::size_of,
    sync::{Arc, OnceLock},
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
    /// Includes the scope header and exact boxed row backing, excluding opaque
    /// standard allocator metadata. The DOM separately charges its weak index.
    pub fn maximum_retained_bytes(targets: usize) -> Result<usize, DomError> {
        targets
            .checked_mul(size_of::<Row>())
            .and_then(|bytes| bytes.checked_add(size_of::<Scope>()))
            .ok_or(DomError::RepresentationExhausted)
    }
    /// Actual producer-owned scope payload, using the same units as the maximum.
    pub fn retained_bytes(&self) -> usize {
        // Construction already checked this exact extent.
        size_of::<Scope>() + self.scope.rows.len() * size_of::<Row>()
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
}
pub(crate) struct Row {
    pub(crate) target: NodeId,
    pub(crate) retired: OnceLock<DomDocumentStamp>,
}
