//! The ledger accounting of one capture builder.
//!
//! The capture limits are the caps of one ledger scope, `html-capture`. Every
//! operation first checks its charges, which refuses with the typed limit stop
//! and changes nothing, then applies them at the point the operation commits.
//! The typed usage is read back from the scope after each commit. Depth is a
//! gauge: the scope records the deepest subject admitted.

use super::*;
use crate::HtmlSemanticAccounting;
use crate::accounting::{Ledger, LedgerError};
use surgeist_html::accounting::{
    HtmlCapScope, HtmlChange, HtmlLimits, HtmlResource, HtmlWork, aggregate_tariffs,
};
use surgeist_ledger::{Change, Operation, RefusalReason, Scope};

/// Widens a platform amount; an amount beyond `u64` saturates and every finite cap refuses it.
fn wide(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn caps(limits: HtmlCaptureLimits) -> HtmlLimits {
    HtmlLimits::from_fn(
        |_| u64::MAX,
        |resource| match resource {
            HtmlResource::CapturedSubjects => wide(limits.subjects),
            HtmlResource::CapturedRelationships => wide(limits.relationships),
            HtmlResource::CapturedAttributes => wide(limits.attributes),
            HtmlResource::CapturedUtf16Units => wide(limits.utf16_units),
            HtmlResource::CapturedDepth => wide(limits.depth),
            HtmlResource::CapturedRegistries => wide(limits.registries),
            HtmlResource::CapturedDefinitions => wide(limits.definitions),
            HtmlResource::ElementValueBytes => wide(limits.element_value_bytes),
            _ => u64::MAX,
        },
    )
}

/// The capture resource a ledger dimension reports as.
fn resource(dimension: HtmlResource) -> Option<HtmlCaptureResource> {
    Some(match dimension {
        HtmlResource::CapturedSubjects => HtmlCaptureResource::Subjects,
        HtmlResource::CapturedRelationships => HtmlCaptureResource::Relationships,
        HtmlResource::CapturedAttributes => HtmlCaptureResource::Attributes,
        HtmlResource::CapturedUtf16Units => HtmlCaptureResource::Utf16Units,
        HtmlResource::CapturedDepth => HtmlCaptureResource::Depth,
        HtmlResource::CapturedRegistries => HtmlCaptureResource::Registries,
        HtmlResource::CapturedDefinitions => HtmlCaptureResource::Definitions,
        HtmlResource::ElementValueBytes => HtmlCaptureResource::ElementValueBytes,
        _ => return None,
    })
}

/// The typed refusal of a ledger error: a refused cap keeps its resource, its limit and the total
/// the change would have reached. A ledger error that is not a refusal of a capture cap is a
/// ledger state the builder's own sequencing rules out, except an allocation failure, which keeps
/// the builder's spelling of one.
pub(crate) fn refusal(error: LedgerError) -> HtmlCaptureError {
    let LedgerError::Refused(refusal) = error else {
        return match error {
            LedgerError::Allocation => HtmlCaptureError::RepresentationExhausted,
            _ => HtmlCaptureError::LedgerState,
        };
    };
    let RefusalReason::Limit { limit } = refusal.reason else {
        return match refusal.reason {
            RefusalReason::Overflow => HtmlCaptureError::RepresentationExhausted,
            _ => HtmlCaptureError::LedgerState,
        };
    };
    let amount = match refusal.change {
        Change::Work(_, amount) | Change::Acquire(_, amount) | Change::Release(_, amount) => amount,
    };
    let dimension = match refusal.change {
        Change::Acquire(dimension, _) => resource(dimension),
        _ => None,
    };
    match (
        dimension,
        usize::try_from(limit),
        refusal
            .current
            .checked_add(amount)
            .and_then(|required| usize::try_from(required).ok()),
    ) {
        (Some(resource), Ok(limit), Some(required)) => HtmlCaptureError::LimitExceeded {
            resource,
            limit,
            required,
        },
        (None, ..) => HtmlCaptureError::LedgerState,
        _ => HtmlCaptureError::RepresentationExhausted,
    }
}

/// What one capture operation charges: counts added to the capture's usage, and the depth it
/// reaches.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Charges {
    pub(super) subjects: usize,
    pub(super) relationships: usize,
    pub(super) attributes: usize,
    pub(super) units: usize,
    pub(super) registries: usize,
    pub(super) definitions: usize,
    /// Element value bytes the operation retains, admitted separately from the counts.
    pub(super) element_value_bytes: usize,
    pub(super) depth: Option<usize>,
}

impl Charges {
    /// The counts as one batch in the order of the limits' precedence: subjects, relationships,
    /// attributes, UTF-16 units, registries, definitions. A zero amount cannot be refused, so the
    /// batch holds only the amounts that charge something.
    fn counts(&self) -> ([HtmlChange; 6], usize) {
        let mut batch = [Change::Acquire(HtmlResource::CapturedSubjects, 0); 6];
        let mut length = 0;
        for (dimension, amount) in [
            (HtmlResource::CapturedSubjects, self.subjects),
            (HtmlResource::CapturedRelationships, self.relationships),
            (HtmlResource::CapturedAttributes, self.attributes),
            (HtmlResource::CapturedUtf16Units, self.units),
            (HtmlResource::CapturedRegistries, self.registries),
            (HtmlResource::CapturedDefinitions, self.definitions),
        ] {
            if amount != 0
                && let Some(slot) = batch.get_mut(length)
            {
                *slot = Change::Acquire(dimension, wide(amount));
                length += 1;
            }
        }
        (batch, length)
    }
}

/// A capture builder's operation, scope and the usage last read from it.
pub(crate) struct CaptureAccount {
    ledger: Ledger,
    scope: Scope,
    /// The host scope the builder charges below, which imports and other scopes of the builder
    /// share.
    host: Option<HtmlSemanticAccounting>,
}

/// Where a builder's accounting is: not yet created, because a builder that charges its own
/// operation creates it at the first charge, or ready.
pub(crate) enum Account {
    Pending,
    Ready(CaptureAccount),
}

impl fmt::Debug for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pending => "Account::Pending",
            Self::Ready(_) => "Account::Ready",
        })
    }
}

impl Account {
    /// The accounting of a builder that charges below the host's scope.
    pub(crate) fn hosted(
        limits: HtmlCaptureLimits,
        host: &HtmlSemanticAccounting,
    ) -> Result<Self, LedgerError> {
        let mut ledger = Ledger::Shared(host.ledger.clone());
        let scope = ledger.child(
            host.scope,
            HtmlCapScope::Capture.name(),
            caps(limits),
            aggregate_tariffs(),
        )?;
        Ok(Self::Ready(CaptureAccount {
            ledger,
            scope,
            host: Some(host.clone()),
        }))
    }

    /// The ready accounting, created in an operation of the builder's own on first use.
    pub(crate) fn ready(
        &mut self,
        limits: HtmlCaptureLimits,
    ) -> Result<&mut CaptureAccount, HtmlCaptureError> {
        if matches!(self, Self::Pending) {
            let operation =
                Operation::new(HtmlCapScope::Capture.name(), caps(limits), 1).map_err(refusal)?;
            let scope = operation.root();
            *self = Self::Ready(CaptureAccount {
                ledger: Ledger::Owned(operation),
                scope,
                host: None,
            });
        }
        match self {
            Self::Ready(account) => Ok(account),
            Self::Pending => Err(HtmlCaptureError::LedgerState),
        }
    }
}

impl CaptureAccount {
    /// Checks the counts of `charges` against the caps without recording them.
    pub(crate) fn check(&mut self, charges: &Charges) -> Result<(), HtmlCaptureError> {
        let (batch, length) = charges.counts();
        let Some(batch) = batch.get(..length) else {
            return Err(HtmlCaptureError::LedgerState);
        };
        if batch.is_empty() {
            return Ok(());
        }
        self.ledger.check_batch(self.scope, batch).map_err(refusal)
    }

    /// Checks `pending` element value bytes against the cap without recording them.
    pub(crate) fn check_element_value_bytes(
        &mut self,
        pending: usize,
    ) -> Result<(), HtmlCaptureError> {
        self.ledger
            .check_batch(
                self.scope,
                &[Change::Acquire(
                    HtmlResource::ElementValueBytes,
                    wide(pending),
                )],
            )
            .map_err(refusal)
    }

    /// Checks that a gauge at `held` can take the value `required`.
    pub(crate) fn check_depth(
        &mut self,
        held: usize,
        required: usize,
    ) -> Result<(), HtmlCaptureError> {
        self.ledger
            .check_batch(
                self.scope,
                &[
                    Change::Release(HtmlResource::CapturedDepth, wide(held)),
                    Change::Acquire(HtmlResource::CapturedDepth, wide(required)),
                ],
            )
            .map_err(refusal)
    }

    /// Records `charges`, then reads the typed usage.
    pub(crate) fn commit(
        &mut self,
        charges: &Charges,
        held_depth: usize,
    ) -> Result<HtmlCaptureUsage, HtmlCaptureError> {
        let (mut batch, length) = charges.counts();
        let mut length = length;
        if charges.element_value_bytes != 0 {
            let slot = batch.get_mut(length).ok_or(HtmlCaptureError::LedgerState)?;
            *slot = Change::Acquire(
                HtmlResource::ElementValueBytes,
                wide(charges.element_value_bytes),
            );
            length += 1;
        }
        if let Some(batch) = batch.get(..length)
            && !batch.is_empty()
        {
            self.ledger
                .apply_batch(self.scope, batch)
                .map_err(refusal)?;
        }
        if let Some(depth) = charges.depth
            && depth > held_depth
        {
            self.ledger
                .record_largest(self.scope, HtmlResource::CapturedDepth, wide(depth))
                .map_err(refusal)?;
        }
        self.read()
    }

    /// Gives back what an operation recorded since `to`, for a refused import.
    pub(crate) fn rollback(
        &mut self,
        from: HtmlCaptureUsage,
        to: HtmlCaptureUsage,
    ) -> Result<HtmlCaptureUsage, HtmlCaptureError> {
        let batch: Vec<HtmlChange> = [
            (HtmlResource::CapturedSubjects, from.subjects, to.subjects),
            (
                HtmlResource::CapturedRelationships,
                from.relationships,
                to.relationships,
            ),
            (
                HtmlResource::CapturedAttributes,
                from.attributes,
                to.attributes,
            ),
            (
                HtmlResource::CapturedUtf16Units,
                from.utf16_units,
                to.utf16_units,
            ),
            (
                HtmlResource::CapturedRegistries,
                from.registries,
                to.registries,
            ),
            (
                HtmlResource::CapturedDefinitions,
                from.definitions,
                to.definitions,
            ),
            (
                HtmlResource::ElementValueBytes,
                from.element_value_bytes,
                to.element_value_bytes,
            ),
        ]
        .into_iter()
        .filter_map(|(dimension, now, before)| {
            now.checked_sub(before)
                .filter(|amount| *amount != 0)
                .map(|amount| Change::Release(dimension, wide(amount)))
        })
        .collect();
        if !batch.is_empty() {
            self.ledger
                .apply_batch(self.scope, &batch)
                .map_err(refusal)?;
        }
        if from.depth != to.depth {
            self.ledger
                .record_largest(self.scope, HtmlResource::CapturedDepth, wide(to.depth))
                .map_err(refusal)?;
        }
        self.read()
    }

    /// The typed usage read from the scope.
    pub(crate) fn read(&self) -> Result<HtmlCaptureUsage, HtmlCaptureError> {
        self.ledger
            .read(self.scope, |usage| {
                let read = |dimension| {
                    usize::try_from(usage.live(dimension))
                        .map_err(|_| HtmlCaptureError::RepresentationExhausted)
                };
                Ok(HtmlCaptureUsage {
                    subjects: read(HtmlResource::CapturedSubjects)?,
                    relationships: read(HtmlResource::CapturedRelationships)?,
                    attributes: read(HtmlResource::CapturedAttributes)?,
                    utf16_units: read(HtmlResource::CapturedUtf16Units)?,
                    depth: read(HtmlResource::CapturedDepth)?,
                    registries: read(HtmlResource::CapturedRegistries)?,
                    definitions: read(HtmlResource::CapturedDefinitions)?,
                    element_value_bytes: read(HtmlResource::ElementValueBytes)?,
                })
            })
            .map_err(refusal)?
    }
}

fn import_caps(limits: HtmlConstructionCaptureLimits) -> HtmlLimits {
    HtmlLimits::from_fn(
        |work| match work {
            HtmlWork::SemanticStep => wide(limits.work()),
            _ => u64::MAX,
        },
        |resource| match resource {
            HtmlResource::MappingEntries => wide(limits.mapping_entries()),
            HtmlResource::ScratchRecords => wide(limits.scratch_entries()),
            _ => u64::MAX,
        },
    )
}

/// The typed refusal of a ledger error of an import.
fn import_refusal(error: LedgerError) -> HtmlConstructionCaptureError {
    use HtmlConstructionCaptureError as Error;
    let LedgerError::Refused(refusal) = error else {
        return match error {
            LedgerError::Allocation => Error::AllocationFailed,
            _ => Error::LedgerState,
        };
    };
    match (refusal.reason, refusal.change) {
        (
            RefusalReason::Limit { .. },
            Change::Work(HtmlWork::SemanticStep, _)
            | Change::Acquire(HtmlResource::MappingEntries | HtmlResource::ScratchRecords, _),
        ) => Error::LimitExceeded,
        (RefusalReason::Overflow, _) => Error::RepresentationExhausted,
        _ => Error::LedgerState,
    }
}

/// The accounting of one construction import: the import limits are the caps of one ledger scope,
/// `html-capture-import`, and the typed usage is read back after each charge.
pub(crate) struct ImportAccount {
    ledger: Ledger,
    scope: Scope,
    usage: HtmlConstructionCaptureUsage,
}

impl fmt::Debug for ImportAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImportAccount")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}

impl ImportAccount {
    pub(crate) fn usage(&self) -> HtmlConstructionCaptureUsage {
        self.usage
    }

    /// Reads the typed usage from the scope. The import calls it where the usage becomes
    /// observable: when it ends and when a finish refuses.
    pub(crate) fn sync(&mut self) -> Result<(), HtmlConstructionCaptureError> {
        self.usage = self
            .ledger
            .read(self.scope, |usage| {
                let read = |value: u64| {
                    usize::try_from(value)
                        .map_err(|_| HtmlConstructionCaptureError::RepresentationExhausted)
                };
                Ok::<_, HtmlConstructionCaptureError>(HtmlConstructionCaptureUsage {
                    mapping_entries: read(usage.live(HtmlResource::MappingEntries))?,
                    scratch_entries: read(usage.peak(HtmlResource::ScratchRecords))?,
                    work: read(usage.work(HtmlWork::SemanticStep))?,
                })
            })
            .map_err(import_refusal)??;
        Ok(())
    }

    fn apply(&mut self, change: HtmlChange) -> Result<(), HtmlConstructionCaptureError> {
        self.ledger
            .apply(self.scope, change)
            .map_err(import_refusal)
    }

    pub(crate) fn work(&mut self, count: usize) -> Result<(), HtmlConstructionCaptureError> {
        self.apply(Change::Work(HtmlWork::SemanticStep, wide(count)))
    }

    pub(crate) fn scratch(&mut self, count: usize) -> Result<(), HtmlConstructionCaptureError> {
        self.apply(Change::Acquire(HtmlResource::ScratchRecords, wide(count)))
    }

    pub(crate) fn mapping_entry(&mut self) -> Result<(), HtmlConstructionCaptureError> {
        self.apply(Change::Acquire(HtmlResource::MappingEntries, 1))
    }

    /// The live scratch entries, to give back to later with `release_to`.
    pub(crate) fn mark(&self) -> Result<usize, HtmlConstructionCaptureError> {
        let live = self
            .ledger
            .read(self.scope, |usage| usage.live(HtmlResource::ScratchRecords))
            .map_err(import_refusal)?;
        usize::try_from(live).map_err(|_| HtmlConstructionCaptureError::RepresentationExhausted)
    }

    /// Gives back the live scratch entries above `mark`. Fewer live entries than the mark is a
    /// ledger state the import's own sequencing rules out.
    pub(crate) fn release_to(&mut self, mark: usize) -> Result<(), HtmlConstructionCaptureError> {
        let live = self.mark()?;
        let above = live
            .checked_sub(mark)
            .ok_or(HtmlConstructionCaptureError::LedgerState)?;
        self.release_scratch(above)
    }

    pub(crate) fn release_scratch(
        &mut self,
        count: usize,
    ) -> Result<(), HtmlConstructionCaptureError> {
        if count == 0 {
            return Ok(());
        }
        self.apply(Change::Release(HtmlResource::ScratchRecords, wide(count)))
    }
}

impl CaptureAccount {
    /// The accounting of an import: below the host's scope when the builder charges there, else in
    /// an operation of its own.
    pub(crate) fn import_account(
        &self,
        limits: HtmlConstructionCaptureLimits,
    ) -> Result<ImportAccount, HtmlConstructionCaptureError> {
        let (ledger, scope) = match &self.host {
            Some(host) => {
                let mut ledger = Ledger::Shared(host.ledger.clone());
                let scope = ledger
                    .child(
                        host.scope,
                        HtmlCapScope::CaptureImport.name(),
                        import_caps(limits),
                        aggregate_tariffs(),
                    )
                    .map_err(import_refusal)?;
                (ledger, scope)
            }
            None => {
                let operation =
                    Operation::new(HtmlCapScope::CaptureImport.name(), import_caps(limits), 1)
                        .map_err(import_refusal)?;
                let scope = operation.root();
                (Ledger::Owned(operation), scope)
            }
        };
        Ok(ImportAccount {
            ledger,
            scope,
            usage: HtmlConstructionCaptureUsage::default(),
        })
    }
}
