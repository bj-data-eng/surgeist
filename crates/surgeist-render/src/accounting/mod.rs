//! Render owns names, units and attempt outcomes; Ledger owns numeric accounting.
mod frame;
mod readback;
mod renderer_attempt;
#[cfg(test)]
pub(crate) use frame::fixture::FrameFixture;
pub(crate) use frame::frame_lifecycle;
pub use frame::{RenderFrameReport, RenderFrameReportRef};
#[cfg(test)]
pub(crate) use readback::fixture::ReadbackFixture;
pub(crate) use readback::{FinishedReadback, ReadbackWork, readback_lifecycle};
pub use readback::{RenderReadbackReport, RenderReadbackReportRef};
mod cpu;
mod cpu_model;
mod generation;
mod generation_model;
mod history;
pub(crate) use cpu::{CpuAccount, CpuCheckpoint, CpuHold, CpuImport, RendererCpu};
pub use cpu_model::*;
pub use generation_model::*;
pub(crate) use history::{HistoryCommit, HistoryObserver, ImageHistory};
mod model;
mod report;
use report::{IntervalNumbers, ObserverBudget, ReportBacking};
mod storage;
pub(crate) use generation::{
    CacheRegistration, Generation, GenerationCheckpoint, GenerationClass, GpuObjectKind, OwnedGpu,
};
#[cfg(test)]
mod tests;
pub use model::*;
#[cfg(test)]
pub(crate) use storage::packed_size;
pub(crate) use storage::{
    AccountedVec, DisposalCompletion, DisposingVec, Hold, OwnedBox, PackedProducer,
    PreparationValue, SharedValue, StorageAccount, StorageContractError, StorageSizing,
    preparation_backing, reserve_backing_with,
};

use crate::{Error, GlyphPreparationError, GlyphResource, GlyphRunLimits, Result};
use std::{
    mem::size_of,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use surgeist_ledger::{Change, Checkpoint, Limits, Operation, Scope, SharedOperation, Tariffs};

pub(crate) type RenderLedger = SharedOperation<RenderWork, RenderResource>;
type LedgerError = surgeist_ledger::Error<RenderWork, RenderResource>;

/// Configuration and one latest terminal record for standalone allocating operations.
/// Glyph preparation, scene inspection, repeat and style normalization borrow the
/// context exclusively. Renderer frames use their renderer-owned authorities.
/// Setup failure preserves
/// its previous record. Inspect or take the record after an operation returns or
/// unwinds, before dropping this context. A taken record remains observable after
/// context destruction. Returned output and caller clones are outside its custody.
#[derive(Default)]
pub struct RenderAccountingContext {
    options: RenderAccountingOptions,
    report: Option<RenderStandaloneReport>,
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
    /// Borrows the latest started attempt's terminal record, including cleanup faults.
    /// A failed setup preserves the preceding record.
    pub fn last_attempt(&self) -> Option<&RenderStandaloneReport> {
        self.report.as_ref()
    }
    /// Moves the report to caller custody without allocating or cloning.
    pub fn take_last_attempt(&mut self) -> Option<RenderStandaloneReport> {
        self.report.take()
    }

    pub(crate) fn run<T>(
        &mut self,
        kind: RenderAttemptKind,
        phase: RenderPhase,
        run: impl FnOnce(&Account) -> Result<T>,
    ) -> Result<T> {
        let mut guard = Attempt::begin(
            self,
            kind,
            GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
            #[cfg(feature = "render-text")]
            None,
        )?;
        let account = guard.account.at(phase);
        let result = run(&account);
        guard.complete(result)
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
        guard.complete(result)
    }
}

pub(crate) struct Attempt<'a> {
    context: &'a mut RenderAccountingContext,
    account: Account,
    checkpoint: Option<Checkpoint>,
    report_backing: Option<ReportBacking<RenderPhase, IntervalNumbers>>,
    kind: RenderAttemptKind,
}
impl<'a> Attempt<'a> {
    fn begin(
        context: &'a mut RenderAccountingContext,
        kind: RenderAttemptKind,
        legacy: GlyphRunLimits,
        #[cfg(feature = "render-text")] text_caps: Option<[usize; 3]>,
    ) -> Result<Self> {
        let (account, checkpoint, report_backing) = build_account(
            context.options,
            kind,
            legacy,
            #[cfg(feature = "render-text")]
            text_caps,
            None,
        )?;
        Ok(Self {
            context,
            account,
            checkpoint: Some(checkpoint),
            report_backing: Some(report_backing),
            kind,
        })
    }
    /// Closes the attempt around the operation's result. A ledger failure found
    /// while closing, or latched by a hold dropped during the operation, prevents
    /// a successful result. Earlier semantic failures retain their own identity.
    fn complete<T>(&mut self, result: Result<T>) -> Result<T> {
        let refusal = *self
            .account
            .first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let status = match &result {
            Ok(_) => RenderAttemptStatus::Succeeded,
            Err(_) if refusal.is_some() => RenderAttemptStatus::Refused,
            Err(error) => RenderAttemptStatus::Failed(error.code()),
        };
        let result = match (result, self.account.latched_fault()) {
            (Ok(value), Some(fault)) => {
                // Dispose real output while its attempt custody is still live.
                drop(value);
                Err(fault.into())
            }
            (result, _) => result,
        };
        self.close(status, result.is_ok());
        match (
            result,
            self.context
                .report
                .as_ref()
                .and_then(RenderStandaloneReport::cleanup_fault),
        ) {
            (Ok(value), Some(fault)) => {
                drop(value);
                Err(fault.into())
            }
            (result, _) => result,
        }
    }
    fn close(&mut self, status: RenderAttemptStatus, transfer: bool) {
        let Some(checkpoint) = self.checkpoint.take() else {
            return;
        };
        let account = &self.account;
        // Every backing acquisition originates at Preparation. Resource scopes
        // remain fixed; child phase attribution is work/timing, not duplicate custody.
        let mut fault = account.latched_fault();
        account.custody_open.store(false, Ordering::Release);
        let cleanup = || {
            let scope = account.scopes[RenderPhase::Preparation.index()];
            let usage = match account.ledger.usage(scope) {
                Ok(usage) => usage,
                Err(error) => {
                    fault.get_or_insert(account.typed_ledger_error(error));
                    return;
                }
            };
            let retained = usage.live(RenderResource::RetainedCpuBytes);
            let retained_release = Change::Release(RenderResource::RetainedCpuBytes, retained);
            let scratch_release = Change::Release(
                RenderResource::ScratchCpuBytes,
                usage.live(RenderResource::ScratchCpuBytes),
            );
            // Transfer cannot commit if either mandatory release is refused.
            // On failure, still attempt every release without transferring output.
            let released = if transfer && fault.is_none() {
                account.ledger.apply_batch(
                    scope,
                    &[
                        Change::Work(RenderWork::PreparedTransferredBytes, retained),
                        retained_release,
                        scratch_release,
                    ],
                )
            } else {
                account
                    .ledger
                    .apply_batch(scope, &[retained_release, scratch_release])
            };
            if let Err(error) = released {
                fault.get_or_insert(account.typed_ledger_error(error));
                for release in [retained_release, scratch_release] {
                    if let Err(error) = account.ledger.apply(scope, release) {
                        fault.get_or_insert(account.typed_ledger_error(error));
                    }
                }
            }
        };
        if let Err(error) = account
            .ledger
            .with_phase(account.scopes[RenderPhase::Cleanup.index()], cleanup)
        {
            fault.get_or_insert(account.typed_ledger_error(error));
        }
        let interval = account
            .ledger
            .end_checkpoint(&checkpoint)
            .map_err(|error| account.typed_ledger_error(error));
        let refusal = *account
            .first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut attempt = interval.and_then(|interval| {
            let backing = self
                .report_backing
                .take()
                .ok_or(RenderAccountingError::Invariant)?;
            RenderAttemptReport::new(
                self.kind,
                status,
                interval,
                refusal,
                account.scopes.as_ref(),
                backing,
            )
        });
        if let Err(error) = &attempt {
            fault.get_or_insert(*error);
        }
        let status = if status == RenderAttemptStatus::Succeeded && fault.is_some() {
            RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
        } else {
            status
        };
        if let Ok(attempt) = &mut attempt {
            attempt.status = status;
        }
        self.context.report = Some(RenderStandaloneReport {
            kind: self.kind,
            status,
            attempt,
            cleanup_fault: fault,
        });
    }
}
impl Drop for Attempt<'_> {
    fn drop(&mut self) {
        self.close(RenderAttemptStatus::Canceled, false);
    }
}
fn build_account(
    options: RenderAccountingOptions,
    kind: RenderAttemptKind,
    legacy: GlyphRunLimits,
    #[cfg(feature = "render-text")] text_caps: Option<[usize; 3]>,
    cpu: Option<CpuAccount>,
) -> Result<(
    Account,
    Checkpoint,
    ReportBacking<RenderPhase, IntervalNumbers>,
)> {
    options.limits.validate(kind)?;
    let report_backing = ReportBacking::interval(ObserverBudget::new(options.profile.ledger()))?;
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
    let mut work = RenderWorkTable::splat(u64::MAX);
    work.GlyphInputs = as_u64(glyphs)?;
    work.PathElementsEmitted = as_u64(paths)?;
    #[cfg(feature = "render-text")]
    if let Some([runs, coords, decorations]) = text_caps {
        work.TextRuns = as_u64(runs)?;
        work.CoordinateInputs = as_u64(coords)?;
        work.TextDecorations = as_u64(decorations)?;
    }
    let caps = Limits::new(
        work,
        RenderResourceTable {
            ScratchCpuBytes: as_u64(scratch)?,
            RetainedCpuBytes: as_u64(retained)?,
            ..RenderResourceTable::splat(u64::MAX)
        },
    );
    for phase in RenderPhase::ALL.into_iter().skip(1) {
        let parent = phase.parent().ok_or(RenderAccountingError::Invariant)?;
        let parent_scope = *scopes
            .get(parent.index())
            .ok_or(RenderAccountingError::Invariant)?;
        if parent != RenderPhase::Attempt && parent_scope == root {
            return Err(RenderAccountingError::Invariant.into());
        }
        scopes[phase.index()] = operation
            .child(
                parent_scope,
                phase.name(),
                if phase == RenderPhase::Preparation {
                    caps
                } else if phase == RenderPhase::RepeatQuota {
                    Limits::new(
                        RenderWorkTable::splat(u64::MAX),
                        RenderResourceTable {
                            LocalRepeatTileQuota: 1_000_000,
                            ..RenderResourceTable::splat(u64::MAX)
                        },
                    )
                } else if phase == RenderPhase::AtlasQuota {
                    let texels = u64::from(u16::MAX) * u64::from(u16::MAX);
                    Limits::new(
                        RenderWorkTable::splat(u64::MAX),
                        RenderResourceTable {
                            LocalAtlasSourceByteQuota: texels * 4,
                            LocalAtlasTexelQuota: texels,
                            ..RenderResourceTable::splat(u64::MAX)
                        },
                    )
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
        scopes: Arc::new(scopes),
        phase: match kind {
            RenderAttemptKind::Frame => RenderPhase::Frame,
            RenderAttemptKind::Readback => RenderPhase::Readback,
            _ => RenderPhase::Preparation,
        },
        cpu,
        history: None,
        custody_open: Arc::new(AtomicBool::new(true)),
        first_refusal: Arc::new(Mutex::new(None)),
        fault: Arc::new(Mutex::new(None)),
        kind,
    };
    Ok((account, checkpoint, report_backing))
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
        LedgerError::Defect(defect) => RenderAccountingError::LedgerDefect(defect),
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
    scopes: Arc<[Scope; PHASE_COUNT]>,
    phase: RenderPhase,
    cpu: Option<CpuAccount>,
    history: Option<HistoryObserver>,
    custody_open: Arc<AtomicBool>,
    first_refusal: Arc<Mutex<Option<RenderRefusal>>>,
    /// First ledger failure met where no error can be returned, such as a drop.
    fault: Arc<Mutex<Option<RenderAccountingError>>>,
    kind: RenderAttemptKind,
}

/// Lends one pinned future without moving its inline state into Ledger's
/// callback. The callback owns this guard, so cancellation drops the underlying
/// future in place before Ledger closes the phase. The outer owner then sees None.
/// Reborrowing consumed storage stays pending without work or an output; a
/// present future retains its own polling contract.
struct PhaseFuture<'a, F> {
    future: std::pin::Pin<&'a mut Option<F>>,
}
impl<F: std::future::Future> std::future::Future for PhaseFuture<'_, F> {
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.get_mut().future.as_mut().as_pin_mut() {
            Some(future) => future.poll(context),
            None => std::task::Poll::Pending,
        }
    }
}
impl<F> Drop for PhaseFuture<'_, F> {
    fn drop(&mut self) {
        self.future.set(None);
    }
}

/// A checked forward observation borrowing the live attempt, never its payload.
pub(crate) struct ForwardPhase<'a> {
    account: &'a Account,
    observation: Option<surgeist_ledger::PhaseObservation>,
}

/// Field-ordered tail for mandatory native readback disposal. It owns the
/// staging owner's existing participant, so its observation needs no borrow.
pub(crate) struct ReadbackCleanupTail {
    account: Account,
    observation: Option<surgeist_ledger::PhaseObservation>,
}
impl ReadbackCleanupTail {
    pub(crate) fn account(&self) -> &Account {
        &self.account
    }
    pub(crate) fn begin(&mut self) {
        match self
            .account
            .ledger
            .begin_phase(self.account.scopes[RenderPhase::ReadbackCleanup.index()])
        {
            Ok(observation) => self.observation = Some(observation),
            Err(error) => self.account.note_drop_error(error),
        }
    }
}
impl Drop for ReadbackCleanupTail {
    fn drop(&mut self) {
        if let Some(observation) = self.observation.take()
            && self.account.ledger.end_phase(observation).is_err()
        {
            // Only a private association defect can reject an observation;
            // originating recording defects remain latched by Ledger itself.
            self.account.note_drop_error(LedgerError::ForeignScope);
        }
    }
}
impl Drop for ForwardPhase<'_> {
    fn drop(&mut self) {
        if let Some(observation) = self.observation.take()
            && self.account.ledger.end_phase(observation).is_err()
        {
            let mut fault = self
                .account
                .fault
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            fault.get_or_insert(RenderAccountingError::Invariant);
        }
    }
}

impl Account {
    pub(crate) fn readback_cleanup_tail(self) -> ReadbackCleanupTail {
        ReadbackCleanupTail {
            account: self,
            observation: None,
        }
    }
    pub(crate) fn begin_phase(&self, phase: RenderPhase) -> Result<ForwardPhase<'_>> {
        let observation = self
            .ledger
            .begin_phase(self.scopes[phase.index()])
            .map_err(|error| self.error(error))?;
        Ok(ForwardPhase {
            account: self,
            observation: Some(observation),
        })
    }
    pub(crate) fn at(&self, phase: RenderPhase) -> Self {
        Self {
            phase,
            ..self.clone()
        }
    }
    #[cfg(feature = "render-text")]
    pub(crate) fn glyph(&self) -> Self {
        Self {
            kind: RenderAttemptKind::GlyphRun,
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
                Change::Work(work, as_u64(n)?),
            )
            .map_err(|e| self.error(e))
    }
    pub(crate) fn coordinate<T>(&self, evaluate: impl FnOnce() -> T) -> Result<T> {
        self.work(RenderWork::CoordinateVisits, 1)?;
        Ok(evaluate())
    }
    pub(crate) fn storage(&self) -> StorageAccount {
        self.cpu.as_ref().map_or_else(
            || StorageAccount::Preparation(self.clone()),
            |cpu| StorageAccount::Renderer(cpu.clone()),
        )
    }
    pub(crate) fn work_u64(&self, work: RenderWork, amount: u64) -> Result<()> {
        self.ledger
            .apply(self.scopes[self.phase.index()], Change::Work(work, amount))
            .map_err(|e| self.error(e))
    }
    pub(crate) fn work_count(&self, work: RenderWork) -> Result<u64> {
        self.ledger
            .usage(self.scopes[RenderPhase::Attempt.index()])
            .map(|usage| usage.work(work))
            .map_err(|e| self.error(e))
    }
    pub(crate) async fn async_phase<T>(
        &self,
        phase: RenderPhase,
        run: impl std::future::Future<Output = Result<T>>,
    ) -> Result<T> {
        // Keep one owned future across Pending. The small borrowing guard keeps
        // its destruction inside the Ledger callback without duplicating the
        // large inline frame state or allocating another task/future owner.
        let mut run = std::pin::pin!(Some(run));
        self.ledger
            .with_async_phase(self.scopes[phase.index()], async || {
                PhaseFuture {
                    future: run.as_mut(),
                }
                .await
            })
            .await
            .map_err(|e| self.error(e))?
    }
    pub(crate) fn cpu(&self) -> Result<&CpuAccount> {
        self.cpu
            .as_ref()
            .ok_or_else(|| RenderAccountingError::Invariant.into())
    }
    pub(crate) fn with_history(&self, history: HistoryObserver) -> Self {
        let mut next = self.clone();
        next.history = Some(history);
        next
    }
    pub(crate) fn observe_image(&self, image: &crate::Image) -> Result<()> {
        let Some(history) = &self.history else {
            return Ok(());
        };
        let miss = history.observe(image, self)?;
        if miss {
            self.work_batch([
                (RenderWork::ImageContentMisses, 1),
                (
                    RenderWork::ImageFirstObservedBytes,
                    as_u64(image.bytes.len())?,
                ),
            ])
        } else {
            self.work(RenderWork::ImageContentHits, 1)
        }
    }
    pub(crate) fn import_image(&self, image: &crate::Image) -> Result<Option<CpuImport>> {
        self.cpu
            .as_ref()
            .map(|cpu| cpu.import_image(image, Some(self)))
            .transpose()
    }
    pub(crate) fn import_image_data(&self, image: &peniko::ImageData) -> Result<Option<CpuImport>> {
        self.cpu
            .as_ref()
            .map(|cpu| cpu.import_image_data(image, Some(self)))
            .transpose()
    }
    pub(crate) fn import_font(
        &self,
        font: &surgeist_font::FontResource,
    ) -> Result<Option<CpuImport>> {
        self.cpu
            .as_ref()
            .map(|cpu| cpu.import_font(font, Some(self)))
            .transpose()
    }
    pub(crate) fn import_prepared(
        &self,
        run: &crate::PreparedGlyphRun,
    ) -> Result<Option<CpuImport>> {
        self.cpu
            .as_ref()
            .map(|cpu| cpu.import_prepared(run, Some(self)))
            .transpose()
    }
    pub(crate) fn work_batch<const N: usize>(&self, changes: [(RenderWork, u64); N]) -> Result<()> {
        self.ledger
            .apply_batch(
                self.scopes[self.phase.index()],
                &changes.map(|(work, n)| Change::Work(work, n)),
            )
            .map_err(|error| self.error(error))
    }
    pub(crate) fn preview_work<const N: usize>(
        &self,
        changes: [(RenderWork, u64); N],
    ) -> Result<()> {
        self.ledger
            .check_batch(
                self.scopes[self.phase.index()],
                &changes.map(|(work, n)| Change::Work(work, n)),
            )
            .map_err(|error| self.error(error))
    }
    pub(crate) fn with_local_quota<T>(
        &self,
        phase: RenderPhase,
        resource: RenderResource,
        limit: u64,
        run: impl FnOnce(&LocalQuota) -> Result<T>,
    ) -> Result<T> {
        let quota = self.local_quota(phase, [limit, limit])?;
        if resource != RenderResource::LocalRepeatTileQuota {
            return Err(RenderAccountingError::Invariant.into());
        }
        run(&quota)
    }
    pub(crate) fn local_quota(&self, phase: RenderPhase, limits: [u64; 2]) -> Result<LocalQuota> {
        let origin = match phase {
            RenderPhase::Repeat if limits[0] == 1_000_000 => RenderPhase::RepeatQuota,
            RenderPhase::Atlas
                if limits
                    == [
                        u64::from(u16::MAX) * u64::from(u16::MAX) * 4,
                        u64::from(u16::MAX) * u64::from(u16::MAX),
                    ] =>
            {
                RenderPhase::AtlasQuota
            }
            _ => return Err(RenderAccountingError::Invariant.into()),
        };
        Ok(LocalQuota {
            account: self.at(origin),
        })
    }
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        let options = RenderAccountingOptions::default();
        let cpu = RendererCpu::new(options).expect("fixture CPU");
        let (account, checkpoint, _report_backing) = build_account(
            options,
            RenderAttemptKind::Frame,
            GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
            #[cfg(feature = "render-text")]
            None,
            Some(cpu.at(RenderCpuScope::FrameBacking)),
        )
        .expect("fixture account");
        account
            .ledger
            .end_checkpoint(&checkpoint)
            .expect("fixture checkpoint");
        account
    }
    pub(crate) fn preview(&self, resource: RenderResource, bytes: usize) -> Result<()> {
        if let Some(cpu) = &self.cpu {
            if !matches!(
                resource,
                RenderResource::RetainedCpuBytes | RenderResource::ScratchCpuBytes
            ) {
                return Err(RenderAccountingError::Invariant.into());
            }
            return cpu.preview(resource, bytes);
        }
        self.ledger
            .check_batch(
                self.scopes[RenderPhase::Preparation.index()],
                &[Change::Acquire(resource, as_u64(bytes)?)],
            )
            .map_err(|e| self.error(e))
    }
    pub(crate) fn acquire(&self, resource: RenderResource, bytes: usize) -> Result<Hold> {
        if let Some(cpu) = &self.cpu {
            if !matches!(
                resource,
                RenderResource::RetainedCpuBytes | RenderResource::ScratchCpuBytes
            ) {
                return Err(RenderAccountingError::Invariant.into());
            }
            return cpu.acquire(resource, bytes).map(Hold::Renderer);
        }
        let amount = as_u64(bytes)?;
        self.ledger
            .apply(
                self.scopes[RenderPhase::Preparation.index()],
                Change::Acquire(resource, amount),
            )
            .map_err(|e| self.error(e))?;
        Ok(Hold::Preparation(PreparationHold {
            account: self.clone(),
            resource,
            amount,
        }))
    }
    pub(crate) fn retain(&self, bytes: usize) -> Result<()> {
        self.acquire(RenderResource::RetainedCpuBytes, bytes)?
            .into_preparation()?
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
        #[cfg(feature = "render-paint")]
        if self.is_paint_gradient() {
            return crate::PaintGradientError::Allocation { resource }.into();
        }
        #[cfg(feature = "render-text")]
        if self.kind == RenderAttemptKind::TextLayout {
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
    pub(crate) fn capacity_error(&self, resource: RenderResource) -> Error {
        #[cfg(feature = "render-paint")]
        if self.is_paint_gradient() {
            return crate::PaintGradientError::CapacityOverflow { resource }.into();
        }
        let _ = resource;
        GlyphPreparationError::CapacityOverflow.into()
    }
    pub(crate) fn is_paint_gradient(&self) -> bool {
        cfg!(feature = "render-paint") && self.kind == RenderAttemptKind::PaintGradient
    }
    /// The first ledger failure met where no error could be returned.
    pub(crate) fn latched_fault(&self) -> Option<RenderAccountingError> {
        *self
            .fault
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    /// Pure projection used by both forward admission and mandatory cleanup.
    /// Cleanup must not register its refusal as forward work or apply legacy limits.
    fn typed_ledger_error(&self, error: LedgerError) -> RenderAccountingError {
        let LedgerError::Refused(r) = error else {
            return map_ledger_failure(error);
        };
        let Some(phase) = RenderPhase::ALL
            .into_iter()
            .find(|p| self.scopes[p.index()] == r.scope)
        else {
            return RenderAccountingError::Invariant;
        };
        let (quantity, amount) = match r.change {
            Change::Work(w, n) => (RenderQuantity::Work(w), n),
            Change::Acquire(d, n) | Change::Release(d, n) => (RenderQuantity::Resource(d), n),
        };
        let reason = match r.reason {
            surgeist_ledger::RefusalReason::Limit { limit } => RenderRefusalReason::Limit { limit },
            surgeist_ledger::RefusalReason::Overflow => RenderRefusalReason::Overflow,
            surgeist_ledger::RefusalReason::Underflow => RenderRefusalReason::Underflow,
        };
        RenderAccountingError::Refused(RenderRefusal {
            phase,
            quantity,
            current: r.current,
            amount,
            reason,
        })
    }

    fn note_drop_error(&self, error: LedgerError) {
        let fault = self.typed_ledger_error(error);
        self.fault
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_or_insert(fault);
    }

    fn error(&self, error: LedgerError) -> Error {
        let diagnostic = self.typed_ledger_error(error);
        let RenderAccountingError::Refused(refusal) = diagnostic else {
            return diagnostic.into();
        };
        self.first_refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_or_insert(refusal);
        if !self.is_paint_gradient()
            && refusal.phase != RenderPhase::Attempt
            && let RenderRefusalReason::Limit { limit } = refusal.reason
            && let Some(required) = refusal
                .current
                .checked_add(refusal.amount)
                .and_then(|n| usize::try_from(n).ok())
            && let Ok(limit) = usize::try_from(limit)
        {
            #[cfg(feature = "render-text")]
            if self.kind == RenderAttemptKind::TextLayout {
                use crate::TextLayoutResource as T;
                let resource = match refusal.quantity {
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
            let resource = match refusal.quantity {
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
pub(crate) struct PreparationHold {
    account: Account,
    resource: RenderResource,
    amount: u64,
}
impl PreparationHold {
    pub(crate) fn retain(mut self) {
        self.amount = 0;
    }
}
impl Drop for PreparationHold {
    fn drop(&mut self) {
        if !self.account.custody_open.load(Ordering::Acquire) {
            return;
        }
        if let Err(error) = self.account.ledger.apply(
            self.account.scopes[RenderPhase::Preparation.index()],
            Change::Release(self.resource, self.amount),
        ) {
            // No error can be returned from a drop. The attempt reports the
            // fault when it closes.
            self.account.note_drop_error(error);
        }
    }
}
pub(crate) fn bytes<T>(n: usize) -> Result<usize> {
    n.checked_mul(size_of::<T>())
        .ok_or_else(|| GlyphPreparationError::CapacityOverflow.into())
}

pub(crate) struct LocalQuota {
    account: Account,
}
impl LocalQuota {
    pub(crate) fn acquire(&self, resource: RenderResource, amount: u64) -> Result<()> {
        self.account
            .ledger
            .apply(
                self.account.scopes[self.account.phase.index()],
                Change::Acquire(resource, amount),
            )
            .map_err(|error| self.account.error(error))
    }
    pub(crate) fn live(&self, resource: RenderResource) -> Result<u64> {
        self.account
            .ledger
            .usage(self.account.scopes[self.account.phase.index()])
            .map(|usage| usage.live(resource))
            .map_err(|error| self.account.error(error))
    }
}
impl Drop for LocalQuota {
    fn drop(&mut self) {
        let scope = self.account.scopes[self.account.phase.index()];
        let usage = match self.account.ledger.usage(scope) {
            Ok(usage) => usage,
            Err(error) => {
                self.account.note_drop_error(error);
                return;
            }
        };
        for resource in [
            RenderResource::LocalRepeatTileQuota,
            RenderResource::LocalAtlasSourceByteQuota,
            RenderResource::LocalAtlasTexelQuota,
        ] {
            if let Err(error) = self
                .account
                .ledger
                .apply(scope, Change::Release(resource, usage.live(resource)))
            {
                self.account.note_drop_error(error);
            }
        }
    }
}
