//! Exact-content history and its allocation-free publication rollback proof.
use super::{
    Account, AccountedVec, CpuAccount, CpuHold, CpuImport, RenderAccountingError,
    RenderAccountingSetupError, RenderCpuScope, RenderResource, RenderWork, RendererCpu,
    storage::StorageAccount,
};
use crate::{Image, Result, image::ImageContentIdentity};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

type Records = AccountedVec<ContentRecord>;

#[derive(Clone, Copy, Eq, PartialEq)]
struct AttemptId(u64);

/// Every record owns the original canonical identity, not the source of a later
/// equal occurrence. A lease follows the physical Arc independently of equality.
struct ContentRecord {
    identity: ImageContentIdentity,
    import: CpuImport,
    introduced_by: AttemptId,
    cleanup: CpuAccount,
}
impl ContentRecord {
    fn try_clone(&self, forward: &Account) -> Result<Self> {
        let import = self.import.try_clone_with(Some(forward))?;
        forward.work(RenderWork::ImageIdentityCopies, 1)?;
        Ok(Self {
            identity: self.identity.clone(),
            import,
            introduced_by: self.introduced_by,
            cleanup: self.cleanup.clone(),
        })
    }
}
impl Drop for ContentRecord {
    fn drop(&mut self) {
        self.cleanup.maintenance(RenderWork::ImageIdentityDrops, 1);
    }
}

struct DraftState {
    attempt: AttemptId,
    records: Records,
}
struct HistoryState {
    committed: Option<Records>,
    draft: Option<DraftState>,
    active: Option<AttemptId>,
    next_attempt: u64,
}
struct HistoryInner {
    cpu: RendererCpu,
    state: Mutex<HistoryState>,
    // The known Arc payload is explicit; opaque Arc/allocator headers are not.
    _hold: CpuHold,
}
impl HistoryInner {
    fn lock(&self) -> MutexGuard<'_, HistoryState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn storage(&self) -> StorageAccount {
        StorageAccount::Renderer(self.cpu.at(RenderCpuScope::ImageHistory))
    }
}
impl Drop for HistoryInner {
    fn drop(&mut self) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let committed = state.committed.take();
        let draft = state.draft.take();
        self.cpu.at(RenderCpuScope::Cleanup).maintenance_phase(|| {
            drop(committed);
            drop(draft);
        });
    }
}

/// The one renderer-owned canonical set, independent of device generations.
pub(crate) struct ImageHistory {
    inner: Arc<HistoryInner>,
}
impl ImageHistory {
    #[cfg(test)]
    pub(crate) fn identities_for_test(&self) -> std::collections::HashSet<ImageContentIdentity> {
        self.inner
            .lock()
            .committed
            .as_ref()
            .expect("canonical history is observable outside publication")
            .iter()
            .map(|record| record.identity.clone())
            .collect()
    }

    pub(crate) fn new(cpu: &RendererCpu) -> Result<Self> {
        let account = cpu.at(RenderCpuScope::ImageHistory);
        account.phase(|| {
            let hold = account.acquire(
                RenderResource::RetainedCpuBytes,
                std::mem::size_of::<HistoryInner>(),
            )?;
            let records = AccountedVec::new_in(
                &StorageAccount::Renderer(account.clone()),
                0,
                RenderResource::RetainedCpuBytes,
            )?;
            Ok(Self {
                inner: Arc::new(HistoryInner {
                    cpu: cpu.clone(),
                    state: Mutex::new(HistoryState {
                        committed: Some(records),
                        draft: None,
                        active: None,
                        next_attempt: 1,
                    }),
                    _hold: hold,
                }),
            })
        })
    }

    /// Allocates the complete baseline before exposing a draft observer. Failure
    /// leaves committed membership and canonical source identities untouched.
    pub(crate) fn begin(&self, forward: &Account) -> Result<HistoryDraft> {
        self.inner.cpu.ensure_healthy()?;
        let account = self.inner.cpu.at(RenderCpuScope::ImageHistory);
        account.phase(|| {
            let mut state = self.inner.lock();
            if state.active.is_some() || state.draft.is_some() {
                return Err(RenderAccountingError::Invariant.into());
            }
            let attempt = AttemptId(state.next_attempt);
            let next = state
                .next_attempt
                .checked_add(1)
                .ok_or(RenderAccountingError::Setup(
                    RenderAccountingSetupError::IdentityExhausted,
                ))?;
            let committed = state
                .committed
                .as_ref()
                .ok_or(RenderAccountingError::Invariant)?;
            let mut records = AccountedVec::new_in(
                &self.inner.storage(),
                committed.len(),
                RenderResource::RetainedCpuBytes,
            )?;
            for record in committed {
                records.push_reserved(record.try_clone(forward)?);
            }
            state.draft = Some(DraftState { attempt, records });
            state.active = Some(attempt);
            state.next_attempt = next;
            Ok(HistoryDraft {
                inner: Some(Arc::clone(&self.inner)),
                attempt,
            })
        })
    }
}

/// A weak bridge permits normalized image emission to update the owning draft
/// without retaining an attempt participant or borrowing Renderer across await.
#[derive(Clone)]
pub(crate) struct HistoryObserver {
    inner: Weak<HistoryInner>,
    attempt: AttemptId,
}
impl HistoryObserver {
    /// Returns true for a new exact content, false for an existing canonical hit.
    /// Semantic hit/miss/source-byte admissions belong to the calling Account.
    pub(crate) fn observe(&self, image: &Image, forward: &Account) -> Result<bool> {
        let inner = self
            .inner
            .upgrade()
            .ok_or(RenderAccountingError::Invariant)?;
        inner.cpu.ensure_healthy()?;
        let account = inner.cpu.at(RenderCpuScope::ImageHistory);
        account.phase(|| {
            let mut state = inner.lock();
            if state.active != Some(self.attempt) {
                return Err(RenderAccountingError::Invariant.into());
            }
            let draft = state
                .draft
                .as_mut()
                .filter(|draft| draft.attempt == self.attempt)
                .ok_or(RenderAccountingError::Invariant)?;
            observe(&inner.cpu, &mut draft.records, self.attempt, image, forward)
        })
    }
}

pub(crate) struct HistoryDraft {
    inner: Option<Arc<HistoryInner>>,
    attempt: AttemptId,
}
impl HistoryDraft {
    pub(crate) fn observer(&self) -> HistoryObserver {
        HistoryObserver {
            inner: Arc::downgrade(self.inner.as_ref().expect("owned history draft")),
            attempt: self.attempt,
        }
    }

    /// Freezes observation and gives one guard sole old/draft rollback custody.
    pub(crate) fn into_publication(mut self) -> Result<HistoryPublication> {
        let inner = self.inner.as_ref().expect("owned history draft");
        let (old, draft) = {
            let mut state = inner.lock();
            if state.active != Some(self.attempt)
                || state.committed.is_none()
                || !matches!(&state.draft, Some(draft) if draft.attempt == self.attempt)
            {
                return Err(RenderAccountingError::Invariant.into());
            }
            (
                state.committed.take(),
                state.draft.take().map(|draft| draft.records),
            )
        };
        Ok(HistoryPublication {
            inner: self.inner.take().expect("owned history draft"),
            attempt: self.attempt,
            old,
            draft,
        })
    }
}
impl Drop for HistoryDraft {
    fn drop(&mut self) {
        let Some(inner) = self.inner.take() else {
            return;
        };
        let draft = {
            let mut state = inner.lock();
            // Only this nonclone guard can end its active transaction.
            debug_assert_eq!(state.active.map(|id| id.0), Some(self.attempt.0));
            state.active = None;
            state.draft.take()
        };
        if let Some(draft) = draft {
            let _ = dispose(&inner.cpu, draft.records);
        }
    }
}

pub(crate) struct HistoryPublication {
    inner: Arc<HistoryInner>,
    attempt: AttemptId,
    old: Option<Records>,
    draft: Option<Records>,
}
impl HistoryPublication {
    /// Retires duplicate old metadata before interval extraction. The complete
    /// draft keeps every old canonical record and its shared payload lease live.
    /// A later failure can still restore baseline membership without allocation.
    pub(crate) fn retire_old(mut self) -> Result<HistoryCommit> {
        if let Some(old) = self.old.take() {
            dispose(&self.inner.cpu, old)?;
        }
        self.inner.cpu.ensure_healthy()?;
        Ok(HistoryCommit { publication: self })
    }
}
impl Drop for HistoryPublication {
    fn drop(&mut self) {
        let Some(mut records) = self.draft.take() else {
            return;
        };
        if let Some(old) = self.old.take() {
            let _ = dispose(&self.inner.cpu, old);
        }
        let cleanup = self.inner.cpu.at(RenderCpuScope::Cleanup);
        cleanup.maintenance_phase(|| {
            // Work overflow faults the lifetime owner but cannot veto restoration.
            let mut write = 0;
            for read in 0..records.len() {
                cleanup.maintenance(RenderWork::ResourceProbes, 1);
                if records[read].introduced_by != self.attempt {
                    if read != write {
                        cleanup.maintenance(RenderWork::ImageIdentityCopies, 2);
                        records.swap(read, write);
                    }
                    write += 1;
                }
            }
            records.truncate(write);
        });
        let mut state = self.inner.lock();
        debug_assert_eq!(state.active.map(|id| id.0), Some(self.attempt.0));
        debug_assert!(state.committed.is_none() && state.draft.is_none());
        state.committed = Some(records);
        state.active = None;
    }
}

/// Old metadata is gone, but this still owns rollback until finalization finishes.
pub(crate) struct HistoryCommit {
    publication: HistoryPublication,
}
impl HistoryCommit {
    /// The final barrier moves only an existing vector header. No record copies,
    /// admission, filtering, phase completion or report extraction remains here.
    pub(crate) fn commit(mut self) {
        let mut state = self.publication.inner.lock();
        debug_assert_eq!(
            state.active.map(|id| id.0),
            Some(self.publication.attempt.0)
        );
        debug_assert!(state.committed.is_none() && state.draft.is_none());
        state.committed = self.publication.draft.take();
        state.active = None;
    }
}

fn observe(
    cpu: &RendererCpu,
    records: &mut Records,
    attempt: AttemptId,
    image: &Image,
    forward: &Account,
) -> Result<bool> {
    let identity = image.content_identity();
    let key = identity.content_key();
    let mut low = 0;
    let mut high = records.len();
    while low < high {
        let mid = low + (high - low) / 2;
        forward.work(RenderWork::ResourceProbes, 1)?;
        if records[mid].identity.content_key() < key {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    let mut insertion = low;
    while insertion < records.len() {
        forward.work(RenderWork::ResourceProbes, 1)?;
        let candidate = &records[insertion].identity;
        if candidate.content_key() != key {
            break;
        }
        if equal_bytes(candidate, identity, forward)? {
            return Ok(false);
        }
        insertion += 1;
    }
    records.grow_with(1, |moved| {
        forward.work(RenderWork::ImageIdentityCopies, moved)
    })?;
    let import = cpu
        .at(RenderCpuScope::ImagePayloads)
        .import_image_pixels(identity, Some(forward))?;
    // One new slot plus each actual suffix destination is written once.
    forward.work(
        RenderWork::ImageIdentityCopies,
        records.len() - insertion + 1,
    )?;
    let record = ContentRecord {
        identity: identity.clone(),
        import,
        introduced_by: attempt,
        cleanup: cpu.at(RenderCpuScope::Cleanup),
    };
    records.insert_reserved(insertion, record);
    Ok(true)
}

fn equal_bytes(
    left: &ImageContentIdentity,
    right: &ImageContentIdentity,
    forward: &Account,
) -> Result<bool> {
    let left = left.source_bytes();
    let right = right.source_bytes();
    if Arc::ptr_eq(left, right) {
        return Ok(true);
    }
    // Validated equal dimensions guarantee equal byte lengths.
    for (&a, &b) in left.iter().zip(right.iter()) {
        forward.work(RenderWork::ImageIdentityByteComparisons, 1)?;
        if a != b {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Mandatory disposal records its fault without making physical cleanup depend
/// on successful phase entry, a healthy owner, or an unspent caller budget.
fn dispose(cpu: &RendererCpu, records: Records) -> Result<()> {
    cpu.at(RenderCpuScope::Cleanup)
        .maintenance_phase(|| drop(records));
    cpu.ensure_healthy()
}

#[cfg(test)]
mod tests;
