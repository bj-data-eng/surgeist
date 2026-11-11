//! Custody of the actual mutable CSSOM owner, independently of execution leases.
use crate::NodeId;
use std::sync::{Arc, Mutex, TryLockError, atomic::AtomicBool};
use surgeist_cssom::{
    CssomBatch, CssomCommit, CssomEditGuard, CssomError, CssomSnapshot, CssomStore,
};

/// Unavailable access preserves the retained owner. No poisoned owner is guessed
/// back into a usable state, and no blocking lock is held across caller work.
#[derive(Clone, Debug)]
pub enum DomStyleCssomRootError {
    /// Another operation is using the actual owner; access can be retried.
    Busy,
    /// A previous panic poisoned the owner; custody remains retained.
    Poisoned,
    /// The caller's commit envelope contains no unattempted batch.
    MissingBatch,
    /// The actual producer rejected an attempted batch operation.
    Cssom(CssomError),
}
impl std::fmt::Display for DomStyleCssomRootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DomStyleCssomRootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cssom(error) => Some(error),
            _ => None,
        }
    }
}
/// The private lock phase cannot attempt a batch or return producer errors.
#[derive(Debug)]
pub(crate) enum AccessError {
    Busy,
    Poisoned,
}
impl From<AccessError> for DomStyleCssomRootError {
    fn from(value: AccessError) -> Self {
        match value {
            AccessError::Busy => Self::Busy,
            AccessError::Poisoned => Self::Poisoned,
        }
    }
}
impl DomStyleCssomRootError {
    pub(crate) fn into_dom_error(self) -> crate::DomError {
        match self {
            Self::Busy => crate::DomError::StyleCssomRootBusy,
            Self::Poisoned => crate::DomError::StyleCssomRootPoisoned,
            Self::MissingBatch | Self::Cssom(_) => crate::DomError::StyleProtocol,
        }
    }
}

/// A capability for one actual root's mutable CSSOM store. Reissued access shares custody;
/// historical DOM snapshots do not. There is no mutable-store extraction API.
pub struct DomStyleCssomRoot {
    root: NodeId,
    store: Arc<Mutex<CssomStore>>,
}
impl std::fmt::Debug for DomStyleCssomRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleCssomRoot")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}
impl DomStyleCssomRoot {
    /// Transfers the actual mutable owner; no snapshot can substitute for it.
    pub fn new(root: NodeId, store: CssomStore) -> Self {
        Self {
            root,
            store: Arc::new(Mutex::new(store)),
        }
    }
    /// Exact canonical root generation for this owner capability.
    pub fn root(&self) -> NodeId {
        self.root
    }
    /// Captures the current producer revision without exposing its live writer.
    pub fn snapshot(&self) -> Result<CssomSnapshot, DomStyleCssomRootError> {
        Ok(self.lock()?.snapshot())
    }
    /// Forms an ordinary producer batch at the caller's exact guard.
    pub fn batch(&self, guard: CssomEditGuard) -> Result<CssomBatch, DomStyleCssomRootError> {
        self.lock()?
            .batch(guard)
            .map_err(DomStyleCssomRootError::Cssom)
    }
    /// Busy/poisoned access leaves the unattempted envelope intact. Once the
    /// producer is called its ordinary consuming commit contract applies.
    pub fn commit(
        &self,
        batch: &mut Option<CssomBatch>,
    ) -> Result<CssomCommit, DomStyleCssomRootError> {
        let mut store = self.lock()?;
        let batch = batch.take().ok_or(DomStyleCssomRootError::MissingBatch)?;
        store.commit(batch).map_err(DomStyleCssomRootError::Cssom)
    }
    /// Prepare the batch and check cancellation at the publication fence
    /// under one loan of the actual Store. A lock refusal preserves the
    /// envelope. A reached preparation refusal precedes the cancellation check.
    pub fn commit_with_cancellation(
        &self,
        batch: &mut Option<CssomBatch>,
        cancel: &AtomicBool,
    ) -> Result<CssomCommit, DomStyleCssomRootError> {
        let mut store = self.lock()?;
        let batch = batch.take().ok_or(DomStyleCssomRootError::MissingBatch)?;
        let prepared = store
            .prepare_commit(batch)
            .map_err(DomStyleCssomRootError::Cssom)?;
        let (commit, retirement) = prepared
            .publish_with_cancellation(cancel)
            .map_err(DomStyleCssomRootError::Cssom)?;
        drop(store);
        drop(retirement);
        Ok(commit)
    }

    pub(crate) fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, CssomStore>, DomStyleCssomRootError> {
        self.acquire().map_err(Into::into)
    }
    pub(crate) fn acquire(&self) -> Result<std::sync::MutexGuard<'_, CssomStore>, AccessError> {
        self.store.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => AccessError::Busy,
            TryLockError::Poisoned(_) => AccessError::Poisoned,
        })
    }
    #[cfg(test)]
    pub(crate) fn poisoned_snapshot_for_test(&self) -> CssomSnapshot {
        match self.store.try_lock() {
            Err(TryLockError::Poisoned(error)) => error.into_inner().snapshot(),
            _ => panic!("owning inspection requires an authentically poisoned owner"),
        }
    }
    pub(crate) fn duplicate(&self) -> Self {
        Self {
            root: self.root,
            store: self.store.clone(),
        }
    }
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        self.root == other.root && Arc::ptr_eq(&self.store, &other.store)
    }
    pub(crate) const fn retained_owner_bytes() -> usize {
        std::mem::size_of::<Mutex<CssomStore>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> DomStyleCssomRoot {
        let mut dom = crate::Dom::new(
            crate::DomLimits::new(
                4,
                100,
                4,
                0,
                0,
                0,
                0,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = crate::DomBatch::new();
        let document = batch.create_document();
        let commit = dom
            .apply(batch, &std::sync::atomic::AtomicBool::new(false))
            .unwrap();
        let id = commit.document(&document).unwrap().node();
        DomStyleCssomRoot::new(
            id,
            CssomStore::new(Default::default(), Default::default()).unwrap(),
        )
    }
    #[test]
    fn cancellable_root_commit_preserves_preparation_refusal_and_published_state() {
        use std::sync::atomic::Ordering;
        use surgeist_css::rule::CssSheet;
        use surgeist_cssom::{CssomInputRole, CssomInputVersion, CssomSheetInputs};

        let root = root();
        let original = root.snapshot().unwrap();
        let guard = CssomEditGuard::from_snapshot(&original);
        let mut stale = Some(root.batch(guard.clone()).unwrap());
        let mut batch = root.batch(guard).unwrap();
        let inputs = CssomSheetInputs::constructed(
            CssomInputVersion {
                role: CssomInputRole::Document,
                identity: "native document".into(),
                revision: 1,
            },
            "native document".into(),
            None,
        );
        let ticket = batch.create_typed_sheet(CssSheet::new(), inputs).unwrap();
        let cancel = AtomicBool::new(false);
        let commit = root
            .commit_with_cancellation(&mut Some(batch), &cancel)
            .unwrap();
        let sheet = commit.sheet(&ticket).unwrap();
        let published = root.snapshot().unwrap();
        assert_ne!(published.revision(), original.revision());
        let mut current = Some(
            root.batch(CssomEditGuard::from_snapshot(&published))
                .unwrap(),
        );

        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            root.commit_with_cancellation(&mut stale, &cancel),
            Err(DomStyleCssomRootError::Cssom(CssomError::StaleRevision))
        ));
        assert!(stale.is_none());
        assert!(matches!(
            root.commit_with_cancellation(&mut current, &cancel),
            Err(DomStyleCssomRootError::Cssom(CssomError::Cancelled))
        ));
        assert!(current.is_none());
        let after = root.snapshot().unwrap();
        assert_eq!(after.revision(), published.revision());
        assert!(after.sheet_rules(sheet).unwrap().is_empty());
    }

    #[test]
    fn busy_and_poisoned_owners_preserve_the_unattempted_batch() {
        let root = root();
        let snapshot = root.snapshot().unwrap();
        let mut batch = Some(
            root.batch(CssomEditGuard::from_snapshot(&snapshot))
                .unwrap(),
        );
        let alias = root.duplicate();
        let held = alias.lock().unwrap();
        assert!(matches!(root.snapshot(), Err(DomStyleCssomRootError::Busy)));
        assert!(matches!(
            root.commit(&mut batch),
            Err(DomStyleCssomRootError::Busy)
        ));
        assert!(batch.is_some());
        drop(held);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _held = alias.lock().unwrap();
            panic!("poison the actual owner, without a caller callback API");
        }));
        assert!(matches!(
            root.commit(&mut batch),
            Err(DomStyleCssomRootError::Poisoned)
        ));
        assert!(batch.is_some());
    }
    #[test]
    fn clones_share_the_actual_owner_and_commit_consumes_only_after_access() {
        let root = root();
        let alias = root.duplicate();
        let before = root.snapshot().unwrap();
        let mut batch = Some(root.batch(CssomEditGuard::from_snapshot(&before)).unwrap());
        alias.commit(&mut batch).unwrap();
        assert!(batch.is_none());
        assert_eq!(root.snapshot().unwrap().revision(), before.revision());
        assert!(matches!(
            root.commit(&mut batch),
            Err(DomStyleCssomRootError::MissingBatch)
        ));
        assert!(root.same_owner(&alias));
    }
}
