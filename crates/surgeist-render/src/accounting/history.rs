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

enum Observation {
    Open,
    Frozen,
}
enum BaselineCustody {
    Retained {
        old: Records,
        observation: Observation,
    },
    DraftOnly,
}
enum Custody {
    Canonical,
    Transaction {
        attempt: AttemptId,
        baseline: BaselineCustody,
    },
}
struct HistoryState {
    records: Records,
    next_attempt: u64,
    custody: Custody,
}
#[cfg(test)]
impl HistoryState {
    fn canonical_for_test(&self) -> impl Iterator<Item = &ContentRecord> {
        let (records, excluded) = match &self.custody {
            Custody::Canonical => (&self.records, None),
            Custody::Transaction {
                baseline: BaselineCustody::Retained { old, .. },
                ..
            } => (old, None),
            Custody::Transaction {
                attempt,
                baseline: BaselineCustody::DraftOnly,
            } => (&self.records, Some(*attempt)),
        };
        records
            .iter()
            .filter(move |record| Some(record.introduced_by) != excluded)
    }
}
struct HistoryContents {
    // Actual records and backing drop before the owning observation ends.
    state: Mutex<HistoryState>,
    cleanup_tail: Option<super::cpu::MaintenancePhase>,
}
struct HistoryInner {
    contents: HistoryContents,
    cpu: RendererCpu,
    // The known Arc payload is explicit; opaque Arc/allocator headers are not.
    _hold: CpuHold,
}
impl HistoryInner {
    fn lock(&self) -> MutexGuard<'_, HistoryState> {
        self.contents
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn storage(&self) -> StorageAccount {
        StorageAccount::Renderer(self.cpu.at(RenderCpuScope::ImageHistory))
    }
}
impl Drop for HistoryInner {
    fn drop(&mut self) {
        self.contents.cleanup_tail = Some(
            self.cpu
                .at(RenderCpuScope::Cleanup)
                .begin_maintenance_phase(),
        );
    }
}

/// The one renderer-owned canonical set, independent of device generations.
pub(crate) struct ImageHistory {
    inner: Arc<HistoryInner>,
}
impl ImageHistory {
    #[cfg(test)]
    pub(crate) fn identities_for_test(&self) -> std::collections::HashSet<ImageContentIdentity> {
        let state = self.inner.lock();
        state
            .canonical_for_test()
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
                    contents: HistoryContents {
                        state: Mutex::new(HistoryState {
                            records,
                            next_attempt: 1,
                            custody: Custody::Canonical,
                        }),
                        cleanup_tail: None,
                    },
                    cpu: cpu.clone(),
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
            if !matches!(state.custody, Custody::Canonical) {
                return Err(RenderAccountingError::Invariant.into());
            }
            let attempt = AttemptId(state.next_attempt);
            let next = state
                .next_attempt
                .checked_add(1)
                .ok_or(RenderAccountingError::Setup(
                    RenderAccountingSetupError::IdentityExhausted,
                ))?;
            let mut records = AccountedVec::new_in(
                &self.inner.storage(),
                state.records.len(),
                RenderResource::RetainedCpuBytes,
            )?;
            for record in &state.records {
                records.append_slot()?.write(record.try_clone(forward)?);
            }
            let old = std::mem::replace(&mut state.records, records);
            state.custody = Custody::Transaction {
                attempt,
                baseline: BaselineCustody::Retained {
                    old,
                    observation: Observation::Open,
                },
            };
            state.next_attempt = next;
            Ok(HistoryDraft {
                lease: HistoryLease {
                    inner: Arc::clone(&self.inner),
                    attempt,
                    disposition: LeaseDisposition::Rollback,
                },
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
            if !matches!(
                &state.custody,
                Custody::Transaction {
                    attempt,
                    baseline: BaselineCustody::Retained {
                        observation: Observation::Open,
                        ..
                    },
                } if *attempt == self.attempt
            ) {
                return Err(RenderAccountingError::Invariant.into());
            }
            observe(&inner.cpu, &mut state.records, self.attempt, image, forward)
        })
    }
}

pub(crate) struct HistoryDraft {
    lease: HistoryLease,
}
impl HistoryDraft {
    pub(crate) fn observer(&self) -> HistoryObserver {
        HistoryObserver {
            inner: Arc::downgrade(&self.lease.inner),
            attempt: self.lease.attempt,
        }
    }

    /// Freezes observation and gives one guard sole old/draft rollback custody.
    pub(crate) fn into_publication(self) -> Result<HistoryPublication> {
        {
            let mut state = self.lease.inner.lock();
            match &mut state.custody {
                Custody::Transaction {
                    attempt,
                    baseline: BaselineCustody::Retained { observation, .. },
                } if *attempt == self.lease.attempt && matches!(observation, Observation::Open) => {
                    *observation = Observation::Frozen;
                }
                _ => return Err(RenderAccountingError::Invariant.into()),
            }
        }
        Ok(HistoryPublication { lease: self.lease })
    }
}

pub(crate) struct HistoryPublication {
    lease: HistoryLease,
}
impl HistoryPublication {
    /// Retires duplicate old metadata before interval extraction. The complete
    /// draft keeps every old canonical record and its shared payload lease live.
    /// A later failure can still restore baseline membership without allocation.
    pub(crate) fn retire_old(self) -> Result<HistoryCommit> {
        let old = {
            let mut state = self.lease.inner.lock();
            let baseline = match &mut state.custody {
                Custody::Transaction { attempt, baseline } if *attempt == self.lease.attempt => {
                    baseline
                }
                _ => return Err(RenderAccountingError::Invariant.into()),
            };
            // Rearm rollback before disposal can fault or return. An invalid
            // protocol phase retains its complete original baseline custody.
            match std::mem::replace(baseline, BaselineCustody::DraftOnly) {
                BaselineCustody::Retained {
                    old,
                    observation: Observation::Frozen,
                } => old,
                unexpected => {
                    *baseline = unexpected;
                    return Err(RenderAccountingError::Invariant.into());
                }
            }
        };
        dispose(&self.lease.inner.cpu, old)?;
        self.lease.inner.cpu.ensure_healthy()?;
        Ok(HistoryCommit { lease: self.lease })
    }
}

enum LeaseDisposition {
    Rollback,
    Complete,
}
/// The single owning participant moves whole between consuming phase carriers.
/// Its mandatory Arc is never extracted or replaced during a live operation.
struct HistoryLease {
    inner: Arc<HistoryInner>,
    attempt: AttemptId,
    disposition: LeaseDisposition,
}
impl HistoryLease {
    fn restore_baseline(&self) {
        let cleanup = self.inner.cpu.at(RenderCpuScope::Cleanup);
        cleanup.maintenance_phase(|| {
            let mut state = self.inner.lock();
            let records = &mut state.records;
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
        // Keep the transaction unavailable to begin until Cleanup has ended.
        self.inner.lock().custody = Custody::Canonical;
    }
}
impl Drop for HistoryLease {
    fn drop(&mut self) {
        if matches!(self.disposition, LeaseDisposition::Complete) {
            return;
        }
        let old = {
            let mut state = self.inner.lock();
            let baseline = match &mut state.custody {
                Custody::Transaction { attempt, baseline } if *attempt == self.attempt => {
                    std::mem::replace(baseline, BaselineCustody::DraftOnly)
                }
                _ => {
                    // A mismatched owner cannot steal canonical or another
                    // transaction's records. Preserve them and latch the fault.
                    self.inner
                        .cpu
                        .at(RenderCpuScope::Cleanup)
                        .note_fault(RenderAccountingError::Invariant);
                    return;
                }
            };
            match baseline {
                BaselineCustody::Retained {
                    old,
                    observation: Observation::Open,
                } => {
                    let abandoned = std::mem::replace(&mut state.records, old);
                    state.custody = Custody::Canonical;
                    drop(state);
                    let _ = dispose(&self.inner.cpu, abandoned);
                    return;
                }
                BaselineCustody::Retained {
                    old,
                    observation: Observation::Frozen,
                } => old,
                BaselineCustody::DraftOnly => {
                    drop(state);
                    self.restore_baseline();
                    return;
                }
            }
        };
        let _ = dispose(&self.inner.cpu, old);
        self.restore_baseline();
    }
}

/// Old metadata is gone, but this still owns rollback until finalization finishes.
pub(crate) struct HistoryCommit {
    lease: HistoryLease,
}
impl HistoryCommit {
    /// The final barrier authorizes the existing vector header. No record copies,
    /// admission, filtering, phase completion or report extraction remains here.
    pub(crate) fn commit(mut self) {
        self.lease.inner.lock().custody = Custody::Canonical;
        self.lease.disposition = LeaseDisposition::Complete;
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
    records.grow(1)?;
    let import = cpu
        .at(RenderCpuScope::ImagePayloads)
        .import_image_pixels(identity, Some(forward))?;
    // One new slot plus each actual suffix destination is written once.
    let slot = records.insert_slot(insertion)?;
    forward.work(RenderWork::ImageIdentityCopies, slot.movement_count())?;
    let record = ContentRecord {
        identity: identity.clone(),
        import,
        introduced_by: attempt,
        cleanup: cpu.at(RenderCpuScope::Cleanup),
    };
    slot.write(record);
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
