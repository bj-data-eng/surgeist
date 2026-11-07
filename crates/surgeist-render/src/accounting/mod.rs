//! Render owns names, units and attempt outcomes; Ledger owns numeric accounting.
mod generation;
mod model;
mod storage;
pub(crate) use generation::{
    CacheRegistration, Generation, GenerationCheckpoint, GenerationClass, GpuObjectKind, OwnedGpu,
};
#[cfg(test)]
mod tests;
pub use model::*;
pub(crate) use storage::AccountedVec;

use crate::{Error, GlyphPreparationError, GlyphResource, GlyphRunLimits, Result};
use std::{
    mem::size_of,
    sync::{Arc, Mutex},
};
use surgeist_ledger::{Change, Checkpoint, Limits, Operation, Scope, SharedOperation, Tariffs};

pub(crate) type RenderLedger = SharedOperation<WORK_COUNT, RESOURCE_COUNT>;
type LedgerError = surgeist_ledger::Error<WORK_COUNT, RESOURCE_COUNT>;

/// Reusable preparation configuration and one latest-started-attempt report slot.
/// The context is borrowed exclusively while preparing. Setup failure preserves
/// its previous report. Returned output and caller clones are outside its custody.
#[derive(Default)]
pub struct RenderAccountingContext {
    options: RenderAccountingOptions,
    report: Option<RenderAttemptReport>,
}
impl RenderAccountingContext {
    /// Configures future attempts. Actual report storage is reserved before work.
    #[must_use]
    pub const fn new(options: RenderAccountingOptions) -> Self {
        Self {
            options,
            report: None,
        }
    }
    /// Returns the current configuration.
    pub const fn options(&self) -> RenderAccountingOptions {
        self.options
    }
    /// Changes configuration for the next attempt, retaining the previous report.
    pub fn set_options(&mut self, options: RenderAccountingOptions) {
        self.options = options;
    }
    /// Borrows the most recently started, completed attempt.
    pub fn last_attempt(&self) -> Option<&RenderAttemptReport> {
        self.report.as_ref()
    }
    /// Moves the report to caller custody without allocating or cloning.
    pub fn take_last_attempt(&mut self) -> Option<RenderAttemptReport> {
        self.report.take()
    }

    pub(crate) fn prepare<T>(
        &mut self,
        kind: RenderAttemptKind,
        legacy: GlyphRunLimits,
        #[cfg(feature = "render-text")] text_caps: Option<[usize; 3]>,
        run: impl FnOnce(&Account) -> Result<T>,
    ) -> Result<T> {
        let mut guard = Attempt::begin(
            self,
            kind,
            legacy,
            #[cfg(feature = "render-text")]
            text_caps,
        )?;
        let account = guard.account.clone();
        let result = account.phase(RenderPhase::Preparation, || run(&account));
        guard.complete(&result);
        result
    }
}

pub(crate) struct Attempt<'a> {
    context: &'a mut RenderAccountingContext,
    account: Account,
    checkpoint: Option<Checkpoint>,
    kind: RenderAttemptKind,
}
impl<'a> Attempt<'a> {
    fn begin(
        context: &'a mut RenderAccountingContext,
        kind: RenderAttemptKind,
        legacy: GlyphRunLimits,
        #[cfg(feature = "render-text")] text_caps: Option<[usize; 3]>,
    ) -> Result<Self> {
        let options = context.options;
        let setup = |error| Error::from(map_ledger_failure(error));
        let mut operation = match options.clock {
            RenderClock::Platform => Operation::with_profile(
                "render.attempt",
                options.limits.ledger(),
                RenderPhase::ALL.len(),
                options.profile.ledger(),
            ),
            RenderClock::Host(clock) => Operation::with_profile_clock(
                "render.attempt",
                options.limits.ledger(),
                RenderPhase::ALL.len(),
                options.profile.ledger(),
                clock,
            ),
        }
        .map_err(setup)?
        .with_checkpoint_limit(1)
        .map_err(setup)?;
        let root = operation.root();
        let mut scopes = [root; PHASE_COUNT];
        let [glyphs, paths, scratch, retained] = legacy.capacities();
        let mut work = [u64::MAX; WORK_COUNT];
        work[RenderWork::GlyphInputs.index()] = as_u64(glyphs)?;
        work[RenderWork::PathElementsEmitted.index()] = as_u64(paths)?;
        #[cfg(feature = "render-text")]
        if let Some([runs, coords, decorations]) = text_caps {
            work[RenderWork::TextRuns.index()] = as_u64(runs)?;
            work[RenderWork::CoordinateInputs.index()] = as_u64(coords)?;
            work[RenderWork::TextDecorations.index()] = as_u64(decorations)?;
        }
        let mut resources = [u64::MAX; RESOURCE_COUNT];
        resources[RenderResource::ScratchCpuBytes.index()] = as_u64(scratch)?;
        resources[RenderResource::RetainedCpuBytes.index()] = as_u64(retained)?;
        let caps = Limits::new(work, resources);
        for phase in RenderPhase::ALL.into_iter().skip(1) {
            let parent = phase.parent().expect("nonroot phase has parent");
            scopes[phase.index()] = operation
                .child(
                    scopes[parent.index()],
                    phase.name(),
                    if phase == RenderPhase::Preparation {
                        caps
                    } else {
                        Limits::unlimited()
                    },
                    Tariffs::identity(),
                )
                .map_err(setup)?;
        }
        let checkpoint = operation.begin_checkpoint().map_err(setup)?;
        let account = Account {
            ledger: operation.share(),
            scopes,
            phase: RenderPhase::Preparation,
            first_refusal: Arc::new(Mutex::new(None)),
            #[cfg(feature = "render-text")]
            layout: kind == RenderAttemptKind::TextLayout,
        };
        Ok(Self {
            context,
            account,
            checkpoint: Some(checkpoint),
            kind,
        })
    }
    fn complete<T>(&mut self, result: &Result<T>) {
        let refusal = *self
            .account
            .first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let status = match result {
            Ok(_) => RenderAttemptStatus::Succeeded,
            Err(_) if refusal.is_some() => RenderAttemptStatus::Refused,
            Err(error) => RenderAttemptStatus::Failed(error.code()),
        };
        self.close(status, result.is_ok());
    }
    fn close(&mut self, status: RenderAttemptStatus, transfer: bool) {
        let Some(checkpoint) = self.checkpoint.take() else {
            return;
        };
        let account = &self.account;
        // Every backing acquisition originates at Preparation. Resource scopes
        // remain fixed; child phase attribution is work/timing, not duplicate custody.
        let cleanup = || {
            let scope = account.scopes[RenderPhase::Preparation.index()];
            let usage = account
                .ledger
                .usage(scope)
                .expect("owned preparation scope");
            let retained = usage.live(RenderResource::RetainedCpuBytes.dimension());
            if transfer {
                account
                    .ledger
                    .apply(
                        scope,
                        Change::Work(RenderWork::PreparedTransferredBytes.dimension(), retained),
                    )
                    .expect("unlimited transfer accounting");
            }
            for resource in [
                RenderResource::RetainedCpuBytes,
                RenderResource::ScratchCpuBytes,
            ] {
                let live = usage.live(resource.dimension());
                account
                    .ledger
                    .apply(scope, Change::Release(resource.dimension(), live))
                    .expect("owned preparation custody");
            }
        };
        account
            .ledger
            .with_phase(account.scopes[RenderPhase::Cleanup.index()], cleanup)
            .expect("owned cleanup phase");
        let interval = account
            .ledger
            .end_checkpoint(&checkpoint)
            .expect("owned active attempt checkpoint");
        let refusal = *account
            .first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.context.report = Some(RenderAttemptReport {
            kind: self.kind,
            status,
            interval,
            refusal,
        });
    }
}
impl Drop for Attempt<'_> {
    fn drop(&mut self) {
        self.close(RenderAttemptStatus::Canceled, false);
    }
}
fn map_ledger_failure(error: LedgerError) -> RenderAccountingError {
    match error {
        LedgerError::Allocation => {
            RenderAccountingError::Setup(RenderAccountingSetupError::Allocation)
        }
        LedgerError::TimingUnavailable => {
            RenderAccountingError::Setup(RenderAccountingSetupError::TimingUnavailable)
        }
        LedgerError::IdentityExhausted => {
            RenderAccountingError::Setup(RenderAccountingSetupError::IdentityExhausted)
        }
        // The fixed scope table and one-checkpoint custodian own all identities
        // and configuration. Violations are internal contracts, not exhaustion.
        LedgerError::ForeignScope
        | LedgerError::ForeignCheckpoint
        | LedgerError::StaleCheckpoint
        | LedgerError::CheckpointLimit { .. }
        | LedgerError::CheckpointsConfigured
        | LedgerError::ScopeLimit { .. }
        | LedgerError::Refused(_) => RenderAccountingError::Invariant,
    }
}
pub(crate) fn as_u64(n: usize) -> Result<u64> {
    u64::try_from(n).map_err(|_| GlyphPreparationError::CapacityOverflow.into())
}

#[derive(Clone)]
pub(crate) struct Account {
    ledger: RenderLedger,
    scopes: [Scope; PHASE_COUNT],
    phase: RenderPhase,
    first_refusal: Arc<Mutex<Option<RenderRefusal>>>,
    #[cfg(feature = "render-text")]
    layout: bool,
}
impl Account {
    pub(crate) fn at(&self, phase: RenderPhase) -> Self {
        Self {
            phase,
            ..self.clone()
        }
    }
    #[cfg(feature = "render-text")]
    pub(crate) fn glyph(&self) -> Self {
        Self {
            layout: false,
            ..self.at(RenderPhase::GlyphPrepare)
        }
    }
    pub(crate) fn phase<T>(&self, phase: RenderPhase, f: impl FnOnce() -> Result<T>) -> Result<T> {
        self.ledger
            .with_phase(self.scopes[phase.index()], f)
            .map_err(|e| self.error(e))?
    }
    pub(crate) fn work(&self, work: RenderWork, n: usize) -> Result<()> {
        self.ledger
            .apply(
                self.scopes[self.phase.index()],
                Change::Work(work.dimension(), as_u64(n)?),
            )
            .map_err(|e| self.error(e))
    }
    pub(crate) fn preview(&self, resource: RenderResource, bytes: usize) -> Result<()> {
        self.ledger
            .check_batch(
                self.scopes[RenderPhase::Preparation.index()],
                &[Change::Acquire(resource.dimension(), as_u64(bytes)?)],
            )
            .map_err(|e| self.error(e))
    }
    pub(crate) fn acquire(&self, resource: RenderResource, bytes: usize) -> Result<Hold> {
        let amount = as_u64(bytes)?;
        self.ledger
            .apply(
                self.scopes[RenderPhase::Preparation.index()],
                Change::Acquire(resource.dimension(), amount),
            )
            .map_err(|e| self.error(e))?;
        Ok(Hold {
            account: self.clone(),
            resource,
            amount,
        })
    }
    pub(crate) fn retain(&self, bytes: usize) -> Result<()> {
        self.acquire(RenderResource::RetainedCpuBytes, bytes)?
            .retain();
        Ok(())
    }
    pub(crate) fn retained<T>(&self, count: usize) -> Result<AccountedVec<T>> {
        AccountedVec::new(self, count, RenderResource::RetainedCpuBytes)
    }
    pub(crate) fn temporary<T>(&self, count: usize) -> Result<AccountedVec<T>> {
        AccountedVec::new(self, count, RenderResource::ScratchCpuBytes)
    }
    pub(crate) fn allocation_error(&self, resource: RenderResource) -> Error {
        #[cfg(feature = "render-text")]
        if self.layout {
            return crate::TextLayoutError::Allocation {
                resource: if resource == RenderResource::ScratchCpuBytes {
                    crate::TextLayoutResource::ScratchBytes
                } else {
                    crate::TextLayoutResource::RetainedBytes
                },
            }
            .into();
        }
        GlyphPreparationError::Allocation {
            resource: if resource == RenderResource::ScratchCpuBytes {
                GlyphResource::ScratchBytes
            } else {
                GlyphResource::RetainedBytes
            },
        }
        .into()
    }
    fn error(&self, error: LedgerError) -> Error {
        let LedgerError::Refused(r) = error else {
            return map_ledger_failure(error).into();
        };
        let phase = RenderPhase::ALL
            .into_iter()
            .find(|p| self.scopes[p.index()] == r.scope)
            .expect("owned refusal scope");
        let (quantity, amount) = match r.change {
            Change::Work(w, n) => (
                RenderQuantity::Work(
                    RenderWork::ALL
                        .into_iter()
                        .find(|k| k.dimension() == w)
                        .expect("owned work dimension"),
                ),
                n,
            ),
            Change::Acquire(d, n) | Change::Release(d, n) => (
                RenderQuantity::Resource(
                    RenderResource::ALL
                        .into_iter()
                        .find(|k| k.dimension() == d)
                        .expect("owned resource dimension"),
                ),
                n,
            ),
        };
        let reason = match r.reason {
            surgeist_ledger::RefusalReason::Limit { limit } => RenderRefusalReason::Limit { limit },
            surgeist_ledger::RefusalReason::Overflow => RenderRefusalReason::Overflow,
            surgeist_ledger::RefusalReason::Underflow => RenderRefusalReason::Underflow,
        };
        let refusal = RenderRefusal {
            phase,
            quantity,
            current: r.current,
            amount,
            reason,
        };
        self.first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_or_insert(refusal);
        if phase != RenderPhase::Attempt
            && let RenderRefusalReason::Limit { limit } = reason
            && let Some(required) = r
                .current
                .checked_add(amount)
                .and_then(|n| usize::try_from(n).ok())
            && let Ok(limit) = usize::try_from(limit)
        {
            #[cfg(feature = "render-text")]
            if self.layout {
                use crate::TextLayoutResource as T;
                let resource = match quantity {
                    RenderQuantity::Work(RenderWork::GlyphInputs) => Some(T::Glyphs),
                    RenderQuantity::Work(RenderWork::PathElementsEmitted) => Some(T::PathElements),
                    RenderQuantity::Work(RenderWork::TextRuns) => Some(T::Runs),
                    RenderQuantity::Work(RenderWork::CoordinateInputs) => Some(T::CoordinateValues),
                    RenderQuantity::Work(RenderWork::TextDecorations) => Some(T::Decorations),
                    RenderQuantity::Resource(RenderResource::ScratchCpuBytes) => {
                        Some(T::ScratchBytes)
                    }
                    RenderQuantity::Resource(RenderResource::RetainedCpuBytes) => {
                        Some(T::RetainedBytes)
                    }
                    _ => None,
                };
                if let Some(resource) = resource {
                    return crate::TextLayoutError::Limit {
                        resource,
                        required,
                        limit,
                    }
                    .into();
                }
            }
            let resource = match quantity {
                RenderQuantity::Work(RenderWork::GlyphInputs) => Some(GlyphResource::Glyphs),
                RenderQuantity::Work(RenderWork::PathElementsEmitted) => {
                    Some(GlyphResource::PathElements)
                }
                RenderQuantity::Resource(RenderResource::ScratchCpuBytes) => {
                    Some(GlyphResource::ScratchBytes)
                }
                RenderQuantity::Resource(RenderResource::RetainedCpuBytes) => {
                    Some(GlyphResource::RetainedBytes)
                }
                _ => None,
            };
            if let Some(resource) = resource {
                return GlyphPreparationError::Limit {
                    resource,
                    required,
                    limit,
                }
                .into();
            }
        }
        RenderAccountingError::Refused(refusal).into()
    }
}
pub(crate) struct Hold {
    account: Account,
    resource: RenderResource,
    amount: u64,
}
impl Hold {
    pub(crate) fn retain(mut self) {
        self.amount = 0;
    }
}
impl Drop for Hold {
    fn drop(&mut self) {
        self.account
            .ledger
            .apply(
                self.account.scopes[RenderPhase::Preparation.index()],
                Change::Release(self.resource.dimension(), self.amount),
            )
            .expect("owned backing custody");
    }
}
pub(crate) fn bytes<T>(n: usize) -> Result<usize> {
    n.checked_mul(size_of::<T>())
        .ok_or_else(|| GlyphPreparationError::CapacityOverflow.into())
}
