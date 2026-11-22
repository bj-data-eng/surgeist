//! Canonical custody for consumers that release a Range by dropping its owner.

use super::*;
use crate::internal::source::{DomSourceEdit, DomSourceLimits, DomSourceOutput, DomSourceProgress};
use std::sync::{Mutex, PoisonError};

/// Ownership of one freshly created canonical live Range. Dropping it moves
/// its funded release record to the original DOM, where an admitted cleanup
/// can reclaim the exact generation once no Document is associated with it.
pub struct DomOwnedRange {
    id: DomRangeId,
    releases: Arc<Releases>,
    release: Option<Box<Release>>,
}

impl DomOwnedRange {
    /// The exact live Range created for this owner.
    pub fn id(&self) -> DomRangeId {
        self.id
    }
}

impl Drop for DomOwnedRange {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            self.releases.enqueue(release);
        }
    }
}

struct Release {
    id: DomRangeId,
    next: Option<Box<Self>>,
    _backing: custody::Debit,
}

pub(in crate::internal::store) struct Releases {
    pending: Mutex<Option<Box<Release>>>,
    _backing: custody::Debit,
}

impl Releases {
    fn enqueue(&self, mut release: Box<Release>) {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        release.next = pending.take();
        *pending = Some(release);
    }
}

impl Drop for Releases {
    fn drop(&mut self) {
        let pending = self
            .pending
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner);
        while let Some(mut release) = pending.take() {
            *pending = release.next.take();
        }
    }
}

impl Dom {
    /// Create an owned Range through the canonical Source operation. Its
    /// release record is admitted before publication. Refusal returns no Range
    /// or deferred release, including inside an authenticated Source callback.
    pub fn create_owned_range(
        &mut self,
        document: DocumentId,
        limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<DomOwnedRange, DomError> {
        self.require_source_entry(cancel)?;
        check_cancel(cancel)?;
        let record_count = self
            .range_release_records
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(record_count, self.limits.ranges, DomResource::Ranges)?;
        let (releases, backing) = {
            let formation = self.source_observation_formation(cancel)?;
            let ledger = self.custody_registry.ledger();
            let reserve = |bytes| {
                ledger.retain(custody::Usage::new(bytes, 0), |usage| match &formation {
                    Some(formation) => formation.admit_custody_usage(usage, cancel),
                    None => check_cancel(cancel),
                })
            };
            let releases = if let Some(releases) = &self.range_releases {
                releases.clone()
            } else {
                Arc::new(Releases {
                    pending: Mutex::new(None),
                    _backing: reserve(size_of::<Releases>() + 2 * size_of::<usize>())?,
                })
            };
            (releases, reserve(size_of::<Release>())?)
        };
        let edit = DomSourceEdit::create_range(document);
        let completed = self
            .begin_source_edit(edit, limits, cancel)?
            .advance(cancel)?;
        let id = match completed {
            DomSourceProgress::Complete(completed) => match completed.output() {
                Some(DomSourceOutput::Range(id)) => id,
                _ => return Err(DomError::InvalidSourceOperation),
            },
            DomSourceProgress::Failed(failure) => return Err(failure.into_parts().1),
            DomSourceProgress::Request(_) => return Err(DomError::IncompleteSourceOperation),
        };
        self.range_releases.get_or_insert_with(|| releases.clone());
        self.range_release_records = record_count;
        Ok(DomOwnedRange {
            id,
            releases,
            release: Some(Box::new(Release {
                id,
                next: None,
                _backing: backing,
            })),
        })
    }

    /// Publish retirement for queued Range generations with no remaining
    /// Document association. Active generations and refused retirements remain
    /// queued for a later native boundary. Already retired generations have no
    /// effect on the current occupant of a reused slot.
    pub fn reap_released_ranges(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        let Some(releases) = self.range_releases.clone() else {
            return Ok(());
        };
        self.require_source_idle()?;
        let mut pending = releases
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        let mut result = Ok(());
        while let Some(mut release) = pending.take() {
            pending = release.next.take();
            if result.is_ok() {
                result = check_cancel(cancel);
            }
            if result.is_ok()
                && !self
                    .state
                    .range_dependencies
                    .has_associated_documents(release.id)
            {
                result = self.reclaim_owned_range(release.id, cancel);
                if result.is_ok() {
                    continue;
                }
            }
            releases.enqueue(release);
        }
        result
    }

    fn reclaim_owned_range(&mut self, id: DomRangeId, cancel: &AtomicBool) -> Result<(), DomError> {
        let records = self
            .range_release_records
            .checked_sub(1)
            .ok_or(DomError::InvalidSourceOperation)?;
        match self.snapshot().range(id) {
            Ok(_) => self.edit_native_state(cancel, |draft| draft.retire_range(id, cancel))?,
            Err(DomError::StaleRange) => {}
            Err(error) => return Err(error),
        }
        self.range_release_records = records;
        Ok(())
    }
}
