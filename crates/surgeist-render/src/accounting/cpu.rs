//! Renderer-lifetime explicit CPU backing. Ledger is the only byte/work authority.
use super::{
    Account, AccountedVec, IntervalNumbers, ObserverBudget, RenderAccountingError,
    RenderAccountingOptions, RenderClock, RenderCpuIdentity, RenderCpuInterval, RenderCpuScope,
    RenderCpuSnapshot, RenderLedger, RenderLifetimeUsage, RenderQuantity, RenderRefusalReason,
    RenderResource, RenderResourceTable, RenderWork, RenderWorkTable, ReportBacking,
    StorageAccount, map_ledger_failure,
};
use crate::{Error, Image, PreparedGlyphRun, Result, image::ImageContentIdentity};
use std::{
    fmt,
    ops::AsyncFnOnce,
    sync::{Arc, Mutex, MutexGuard},
};
use surgeist_ledger::{Change, Checkpoint, Limits, Operation, Scope, Tariffs};

const SCOPE_COUNT: usize = RenderCpuScope::ALL.len();

/// Independent lifetime core. A backing hold retains this core, never the import
/// registry or an attempt; registry-owned holds therefore cannot form a cycle.
struct Core {
    ledger: RenderLedger,
    scopes: [Scope; SCOPE_COUNT],
    observer_budget: ObserverBudget,
    fault: Mutex<Option<RenderAccountingError>>,
}
impl Core {
    fn lock(&self) -> MutexGuard<'_, Option<RenderAccountingError>> {
        self.fault
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
    fn diagnostic(&self, error: super::LedgerError) -> RenderAccountingError {
        let surgeist_ledger::Error::Refused(refusal) = error else {
            return map_ledger_failure(error);
        };
        let Some(scope) = RenderCpuScope::ALL
            .into_iter()
            .find(|s| self.scopes[s.index()] == refusal.scope)
        else {
            return RenderAccountingError::Invariant;
        };
        let (quantity, amount) = match refusal.change {
            Change::Work(d, n) => (RenderQuantity::Work(d), n),
            Change::Acquire(d, n) | Change::Release(d, n) => (RenderQuantity::Resource(d), n),
        };
        let reason = match refusal.reason {
            surgeist_ledger::RefusalReason::Limit { limit } => RenderRefusalReason::Limit { limit },
            surgeist_ledger::RefusalReason::Overflow => RenderRefusalReason::Overflow,
            surgeist_ledger::RefusalReason::Underflow => RenderRefusalReason::Underflow,
        };
        RenderAccountingError::CpuRefused {
            scope,
            quantity,
            current: refusal.current,
            amount,
            reason,
        }
    }
    fn record(&self, error: RenderAccountingError) -> Error {
        self.remember(error).into()
    }
    fn remember(&self, error: RenderAccountingError) -> RenderAccountingError {
        let mut fault = self.lock();
        *fault.get_or_insert(error)
    }
    fn mandatory_apply(&self, scope: RenderCpuScope, change: Change<RenderWork, RenderResource>) {
        let mut fault = self.lock();
        if let Err(error) = self.ledger.apply(self.scopes[scope.index()], change) {
            fault.get_or_insert(self.diagnostic(error));
        }
    }
    fn healthy(&self) -> Result<()> {
        match *self.lock() {
            Some(error) => Err(error.into()),
            None => Ok(()),
        }
    }
    fn apply(
        &self,
        scope: RenderCpuScope,
        change: Change<RenderWork, RenderResource>,
    ) -> Result<()> {
        let mut fault = self.lock();
        if let Some(error) = *fault {
            return Err(error.into());
        }
        self.ledger
            .apply(self.scopes[scope.index()], change)
            .map_err(|error| {
                let diagnostic = self.diagnostic(error);
                // Ordinary configured forward refusal does not poison lifetime custody.
                if matches!(
                    diagnostic,
                    RenderAccountingError::CpuRefused {
                        reason: RenderRefusalReason::Limit { .. },
                        ..
                    }
                ) {
                    diagnostic.into()
                } else {
                    (*fault.get_or_insert(diagnostic)).into()
                }
            })
    }
}

struct State {
    registry: Mutex<Registry>,
    core: Arc<Core>,
    // This is the actual separately allocated registry owner payload. Pure
    // Ledger/Core observer state and opaque Arc headers are excluded.
    _backing: Option<CpuHold>,
}
impl State {
    fn lock(&self) -> MutexGuard<'_, Registry> {
        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
impl Drop for State {
    fn drop(&mut self) {
        let registry = std::mem::take(
            self.registry
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if let Some(backing) = self._backing.take() {
            let account = backing.account();
            account.maintenance_phase(|| {
                drop(registry);
                drop(backing);
            });
        } else {
            drop(registry);
        }
    }
}
#[derive(Clone)]
pub(crate) struct RendererCpu(Arc<State>);
impl fmt::Debug for RendererCpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RendererCpu").finish_non_exhaustive()
    }
}
impl RendererCpu {
    #[cfg(all(test, not(target_arch = "wasm32")))]
    pub(crate) fn seed_readback_transfer_for_test(&self, amount: u64) -> Result<()> {
        self.0.core.apply(
            RenderCpuScope::ReadbackOutput,
            Change::Work(RenderWork::ReadbackTransferredBytes, amount),
        )
    }
    pub(crate) fn new(options: RenderAccountingOptions) -> Result<Self> {
        let limits = Limits::new(
            RenderWorkTable::splat(u64::MAX),
            RenderResourceTable {
                RetainedCpuBytes: options.limits.retained_bytes(),
                ScratchCpuBytes: options.limits.scratch_bytes(),
                ..RenderResourceTable::splat(u64::MAX)
            },
        );
        let profile = options.profile.ledger_cpu();
        let setup = |error| Error::from(map_ledger_failure(error));
        let mut operation = match options.clock {
            RenderClock::Platform => {
                Operation::with_profile("render.cpu", limits, SCOPE_COUNT, profile)
            }
            RenderClock::Host(clock) => {
                Operation::with_profile_clock("render.cpu", limits, SCOPE_COUNT, profile, clock)
            }
        }
        .map_err(setup)?
        .with_checkpoint_limit(1)
        .map_err(setup)?;
        let root = operation.root();
        let mut scopes = [root; SCOPE_COUNT];
        for scope in RenderCpuScope::ALL.into_iter().skip(1) {
            scopes[scope.index()] = operation
                .child(root, scope.name(), Limits::unlimited(), Tariffs::identity())
                .map_err(setup)?;
        }
        let core = Arc::new(Core {
            ledger: operation.share(),
            scopes,
            observer_budget: ObserverBudget::new(profile),
            fault: Mutex::new(None),
        });
        let account = CpuAccount {
            core: core.clone(),
            owner: None,
            scope: RenderCpuScope::ImagePayloads,
        };
        account.phase(|| {
            let backing = account.acquire(
                RenderResource::RetainedCpuBytes,
                std::mem::size_of::<State>(),
            )?;
            Ok(Self(Arc::new(State {
                registry: Mutex::new(Registry {
                    entries: Vec::new(),
                    backing: None,
                    next_key: 0,
                }),
                core,
                _backing: Some(backing),
            })))
        })
    }
    pub(crate) fn at(&self, scope: RenderCpuScope) -> CpuAccount {
        CpuAccount {
            core: self.0.core.clone(),
            owner: Some(self.0.clone()),
            scope,
        }
    }
    pub(crate) fn ensure_healthy(&self) -> Result<()> {
        self.0.core.healthy()
    }
    pub(crate) fn fault(&self) -> Option<RenderAccountingError> {
        *self.0.core.lock()
    }
    pub(crate) fn begin_interval(&self) -> Result<CpuCheckpoint> {
        let fault = self.0.core.lock();
        if let Some(error) = *fault {
            return Err(error.into());
        }
        let report_backing = ReportBacking::interval(self.0.core.observer_budget)?;
        let checkpoint = self
            .0
            .core
            .ledger
            .begin_checkpoint()
            .map_err(|e| Error::from(map_ledger_failure(e)))?;
        Ok(CpuCheckpoint {
            cpu: self.clone(),
            checkpoint: Some(checkpoint),
            report_backing: Some(report_backing),
        })
    }
    /// One coherent profiled report, or one exact root read when observation is
    /// off. This never changes the import registry, faults or attempt reports.
    pub(crate) fn snapshot(&self) -> Result<RenderCpuSnapshot> {
        let fault = self.0.core.lock();
        let backing = ReportBacking::snapshot(self.0.core.observer_budget)?;
        let profile = self
            .0
            .core
            .ledger
            .snapshot()
            .map_err(|e| Error::from(map_ledger_failure(e)))?;
        let usage = match &profile {
            Some(profile) => profile
                .scopes
                .get(RenderCpuScope::Root.index())
                .map(|root| root.usage)
                .ok_or(RenderAccountingError::Invariant)?,
            None => self
                .0
                .core
                .ledger
                .usage(self.0.core.scopes[RenderCpuScope::Root.index()])
                .map_err(|e| Error::from(map_ledger_failure(e)))?,
        };
        Ok(RenderCpuSnapshot::new(
            RenderCpuIdentity(self.0.core.scopes[RenderCpuScope::Root.index()]),
            RenderLifetimeUsage::new(usage),
            profile,
            *fault,
            &self.0.core.scopes,
            backing,
        )?)
    }
}

/// A lifetime-only participant. The registry is available to normal imports;
/// backing holds reconstruct a core-only participant for guarded growth.
#[derive(Clone)]
pub(crate) struct CpuAccount {
    core: Arc<Core>,
    owner: Option<Arc<State>>,
    scope: RenderCpuScope,
}
impl fmt::Debug for CpuAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpuAccount")
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}
impl CpuAccount {
    pub(crate) fn at(&self, scope: RenderCpuScope) -> Self {
        Self {
            scope,
            ..self.clone()
        }
    }
    pub(crate) fn ensure_healthy(&self) -> Result<()> {
        self.core.healthy()
    }
    pub(crate) fn note_fault(&self, error: RenderAccountingError) {
        self.core.remember(error);
    }
    /// Checked owner work. Unlike mandatory cleanup, refusal stops a pending
    /// physical relocation before any element moves.
    pub(crate) fn checked_work(&self, work: RenderWork, amount: u64) -> Result<()> {
        self.core.apply(self.scope, Change::Work(work, amount))
    }
    pub(crate) fn maintenance_phase<T>(&self, run: impl FnOnce() -> T) -> T {
        let mut run = Some(run);
        match self
            .core
            .ledger
            .with_phase(self.core.scopes[self.scope.index()], || {
                run.take().expect("owned maintenance callback")()
            }) {
            Ok(value) => value,
            Err(error) => {
                self.core.remember(self.core.diagnostic(error));
                run.take().expect("unstarted maintenance callback")()
            }
        }
    }
    pub(crate) fn preview(&self, resource: RenderResource, amount: usize) -> Result<()> {
        let fault = self.core.lock();
        if let Some(error) = *fault {
            return Err(error.into());
        }
        self.core
            .ledger
            .check_batch(
                self.core.scopes[self.scope.index()],
                &[Change::Acquire(resource, self.amount(resource, amount)?)],
            )
            .map_err(|e| Error::from(self.core.diagnostic(e)))
    }
    pub(crate) fn acquire(&self, resource: RenderResource, amount: usize) -> Result<CpuHold> {
        let amount = self.amount(resource, amount)?;
        self.core
            .apply(self.scope, Change::Acquire(resource, amount))?;
        Ok(CpuHold {
            core: self.core.clone(),
            scope: self.scope,
            resource,
            amount: Some(amount),
        })
    }
    pub(crate) fn allocation_error(&self, resource: RenderResource) -> Error {
        RenderAccountingError::CpuAllocation {
            scope: self.scope,
            resource,
        }
        .into()
    }
    pub(crate) fn capacity_error(&self, resource: RenderResource) -> Error {
        RenderAccountingError::CpuCapacityOverflow {
            scope: self.scope,
            resource,
        }
        .into()
    }
    pub(crate) fn sized<T>(&self, count: usize, resource: RenderResource) -> Result<usize> {
        count
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| self.capacity_error(resource))
    }
    fn amount(&self, resource: RenderResource, amount: usize) -> Result<u64> {
        u64::try_from(amount).map_err(|_| self.capacity_error(resource))
    }
    /// Includes the real transaction callback, outside Ledger's lock.
    pub(crate) fn phase<T>(&self, run: impl FnOnce() -> Result<T>) -> Result<T> {
        self.core.healthy()?;
        self.core
            .ledger
            .with_phase(self.core.scopes[self.scope.index()], run)
            .map_err(|e| self.core.record(self.core.diagnostic(e)))?
    }
    /// Starts on first poll, includes Pending and closes on cancellation.
    pub(crate) async fn async_phase<T>(&self, run: impl AsyncFnOnce() -> Result<T>) -> Result<T> {
        self.core.healthy()?;
        self.core
            .ledger
            .with_async_phase(self.core.scopes[self.scope.index()], run)
            .await
            .map_err(|e| self.core.record(self.core.diagnostic(e)))?
    }
    /// Mandatory actual cleanup is checked-unlimited. Its failure preserves the
    /// first health fault and cannot prevent other provable releases/rollback.
    pub(crate) fn maintenance(&self, work: RenderWork, amount: u64) {
        self.core
            .mandatory_apply(RenderCpuScope::Cleanup, Change::Work(work, amount));
    }
    fn probe(&self, forward: Option<&Account>) -> Result<()> {
        self.visit(RenderWork::ResourceProbes, forward)
    }
    fn visit(&self, work: RenderWork, forward: Option<&Account>) -> Result<()> {
        match forward {
            Some(account) => account.work(work, 1),
            None => self.core.apply(self.scope, Change::Work(work, 1)),
        }
    }
    fn owner(&self) -> Result<&Arc<State>> {
        self.owner
            .as_ref()
            .ok_or_else(|| RenderAccountingError::Invariant.into())
    }
    pub(crate) fn import_image(
        &self,
        image: &Image,
        forward: Option<&Account>,
    ) -> Result<CpuImport> {
        let account = self.at(RenderCpuScope::ImagePayloads);
        account.phase(|| {
            let owner = account.owner()?;
            let mut registry = owner.lock();
            let key = registry.image(&account, image, forward)?;
            Ok(CpuImport {
                owner: owner.clone(),
                key,
            })
        })
    }
    /// History retains the canonical pixel identity without retaining the
    /// public Image's separately allocated Blob adapter.
    pub(crate) fn import_image_pixels(
        &self,
        identity: &ImageContentIdentity,
        forward: Option<&Account>,
    ) -> Result<CpuImport> {
        let account = self.at(RenderCpuScope::ImagePayloads);
        account.phase(|| {
            let owner = account.owner()?;
            let mut registry = owner.lock();
            let key = registry.pixels(&account, identity.source_bytes(), forward)?;
            Ok(CpuImport {
                owner: owner.clone(),
                key,
            })
        })
    }
    /// A producer Blob must share an already admitted Image's actual adapter
    /// allocation. Slice addresses and Blob IDs cannot establish that proof,
    /// particularly for empty pixels; arbitrary Blob payloads are not estimated.
    pub(crate) fn import_image_data(
        &self,
        image: &peniko::ImageData,
        forward: Option<&Account>,
    ) -> Result<CpuImport> {
        let account = self.at(RenderCpuScope::ImagePayloads);
        account.phase(|| {
            let owner = account.owner()?;
            let mut registry = owner.lock();
            let (backing, _) = image.data.clone().into_raw_parts();
            for entry in &mut registry.entries {
                account.probe(forward)?;
                if let Payload::ImageAdapter {
                    backing: existing, ..
                } = &entry.payload
                    && Arc::ptr_eq(existing, &backing)
                {
                    entry.add_lease(&account.core)?;
                    return Ok(CpuImport {
                        owner: owner.clone(),
                        key: entry.key,
                    });
                }
            }
            Err(account.core.record(RenderAccountingError::Invariant))
        })
    }
    pub(crate) fn import_font(
        &self,
        font: &surgeist_font::FontResource,
        forward: Option<&Account>,
    ) -> Result<CpuImport> {
        let account = self.at(RenderCpuScope::FontPayloads);
        account.phase(|| {
            let owner = account.owner()?;
            let mut registry = owner.lock();
            let key = registry.font(&account, font, forward)?;
            Ok(CpuImport {
                owner: owner.clone(),
                key,
            })
        })
    }
    pub(crate) fn import_prepared(
        &self,
        prepared: &PreparedGlyphRun,
        forward: Option<&Account>,
    ) -> Result<CpuImport> {
        let account = self.at(RenderCpuScope::PreparedValues);
        account.phase(|| {
            let owner = account.owner()?;
            let mut registry = owner.lock();
            for entry in &mut registry.entries {
                account.probe(forward)?;
                if let Payload::Prepared(existing) = &entry.payload
                    && existing.is_same_preparation(prepared)
                {
                    entry.add_lease(&account.core)?;
                    return Ok(CpuImport {
                        owner: owner.clone(),
                        key: entry.key,
                    });
                }
            }
            registry.reserve(&account)?;
            // Reserve all key backing before publishing subordinate ownership.
            let mut bitmap_count = 0usize;
            for paint in prepared.paints() {
                account.visit(RenderWork::PaintVisits, forward)?;
                if matches!(paint, crate::text::GlyphPaint::Bitmap { .. }) {
                    bitmap_count = bitmap_count
                        .checked_add(1)
                        .ok_or_else(|| account.capacity_error(RenderResource::RetainedCpuBytes))?;
                }
            }
            let count = bitmap_count
                .checked_add(1)
                .ok_or_else(|| account.capacity_error(RenderResource::RetainedCpuBytes))?;
            let mut dependencies = DependencyBacking::new(&account, count)?;
            let result = (|| {
                let font = registry.font(
                    &account.at(RenderCpuScope::FontPayloads),
                    prepared.font().admitted_face().resource(),
                    forward,
                )?;
                dependencies.keys.append_slot()?.write(font);
                for paint in prepared.paints() {
                    account.visit(RenderWork::PaintVisits, forward)?;
                    if let crate::text::GlyphPaint::Bitmap { image, .. } = paint {
                        let key = registry.image(
                            &account.at(RenderCpuScope::ImagePayloads),
                            image,
                            forward,
                        )?;
                        dependencies.keys.append_slot()?.write(key);
                    }
                }
                // Subordinate insertion may have consumed the spare registry cell.
                registry.reserve(&account)?;
                let hold = account.acquire(
                    RenderResource::RetainedCpuBytes,
                    prepared
                        .own_backing_bytes()
                        .map_err(|_| account.capacity_error(RenderResource::RetainedCpuBytes))?,
                )?;
                let key = registry.next(&account.core)?;
                Ok((key, hold))
            })();
            match result {
                Ok((key, hold)) => {
                    registry.entries.push(Entry {
                        key,
                        leases: 1,
                        payload: Payload::Prepared(prepared.clone()),
                        dependencies: Some(dependencies),
                        hold,
                    });
                    Ok(CpuImport {
                        owner: owner.clone(),
                        key,
                    })
                }
                Err(error) => {
                    while let Some(key) = dependencies.keys.pop() {
                        registry.release(key, &account.core);
                    }
                    Err(error)
                }
            }
        })
    }
}

/// Noncloneable exact backing admission. Every release is attempted even after a
/// prior fault. The lifetime core outlives this hold without retaining a registry.
pub(crate) struct CpuHold {
    core: Arc<Core>,
    scope: RenderCpuScope,
    resource: RenderResource,
    amount: Option<u64>,
}
impl CpuHold {
    pub(crate) fn account(&self) -> CpuAccount {
        CpuAccount {
            core: self.core.clone(),
            owner: None,
            scope: self.scope,
        }
    }
    pub(crate) fn resource(&self) -> RenderResource {
        self.resource
    }
    pub(crate) fn active_amount(&self) -> Option<u64> {
        self.amount
    }
    /// The final checked operation before a complete readback leaves Renderer.
    /// Rejection preserves custody; success consumes it without destroying data.
    pub(crate) fn transfer_readback(&mut self) -> Result<()> {
        if self.scope != RenderCpuScope::ReadbackOutput
            || self.resource != RenderResource::RetainedCpuBytes
        {
            return Err(RenderAccountingError::Invariant.into());
        }
        let amount = self.amount.ok_or(RenderAccountingError::Invariant)?;
        let mut fault = self.core.lock();
        if let Some(error) = *fault {
            return Err(error.into());
        }
        self.core
            .ledger
            .apply_batch(
                self.core.scopes[self.scope.index()],
                &[
                    Change::Work(RenderWork::ReadbackTransferredBytes, amount),
                    Change::Release(self.resource, amount),
                ],
            )
            .map_err(|error| {
                let diagnostic = self.core.diagnostic(error);
                Error::from(*fault.get_or_insert(diagnostic))
            })?;
        self.amount = None;
        Ok(())
    }
}
impl Drop for CpuHold {
    fn drop(&mut self) {
        if let Some(amount) = self.amount.take() {
            self.core
                .mandatory_apply(self.scope, Change::Release(self.resource, amount));
        }
    }
}

/// One private semantic lease. The registry retains the exact producer backing;
/// public caller values and their clones carry no hidden renderer participant.
pub(crate) struct CpuImport {
    owner: Arc<State>,
    key: Key,
}
impl fmt::Debug for CpuImport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpuImport").finish_non_exhaustive()
    }
}
impl CpuImport {
    /// Destroys an owning value while this import's payload custody stays live.
    /// The import's own registry release follows separately in `Drop`.
    pub(crate) fn dispose_value<T>(&self, run: impl FnOnce() -> T) -> T {
        let account = CpuAccount {
            core: self.owner.core.clone(),
            owner: None,
            scope: RenderCpuScope::Cleanup,
        };
        account.maintenance_phase(run)
    }

    pub(crate) fn try_clone_with(&self, forward: Option<&Account>) -> Result<Self> {
        let mut registry = self.owner.lock();
        for entry in &mut registry.entries {
            match forward {
                Some(account) => account.work(RenderWork::ResourceProbes, 1)?,
                None => {
                    self.owner.core.mandatory_apply(
                        RenderCpuScope::Cleanup,
                        Change::Work(RenderWork::ResourceProbes, 1),
                    );
                }
            }
            if entry.key == self.key {
                entry.add_lease(&self.owner.core)?;
                return Ok(Self {
                    owner: self.owner.clone(),
                    key: self.key,
                });
            }
        }
        Err(self.owner.core.record(RenderAccountingError::Invariant))
    }
}
impl Drop for CpuImport {
    fn drop(&mut self) {
        let account = CpuAccount {
            core: self.owner.core.clone(),
            owner: None,
            scope: RenderCpuScope::Cleanup,
        };
        account.maintenance_phase(|| self.owner.lock().release(self.key, &self.owner.core));
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Key(u64);
enum Payload {
    Pixels(Arc<[u8]>),
    ImageAdapter {
        backing: Arc<dyn AsRef<[u8]> + Send + Sync>,
        pixels: Key,
    },
    Font(surgeist_font::ResourceId),
    Prepared(PreparedGlyphRun),
}
struct Entry {
    key: Key,
    leases: u64,
    payload: Payload,
    dependencies: Option<DependencyBacking>,
    // Payload and subordinate ownership drop before their own backing admission.
    hold: CpuHold,
}
impl Entry {
    fn add_lease(&mut self, core: &Core) -> Result<()> {
        self.leases = self
            .leases
            .checked_add(1)
            .ok_or_else(|| core.record(RenderAccountingError::Invariant))?;
        Ok(())
    }
}
struct DependencyBacking {
    keys: AccountedVec<Key>,
}
impl DependencyBacking {
    fn new(account: &CpuAccount, count: usize) -> Result<Self> {
        Ok(Self {
            keys: AccountedVec::new_in(
                &StorageAccount::Renderer(account.clone()),
                count,
                RenderResource::RetainedCpuBytes,
            )?,
        })
    }
}
#[derive(Default)]
struct Registry {
    entries: Vec<Entry>,
    backing: Option<CpuHold>,
    next_key: u64,
}
impl Drop for Registry {
    fn drop(&mut self) {
        if let Some(backing) = self.backing.take() {
            let entries = std::mem::take(&mut self.entries);
            let account = backing.account();
            if !entries.is_empty() {
                account.note_fault(RenderAccountingError::Invariant);
            }
            account.maintenance_phase(|| {
                drop(entries);
                drop(backing);
            });
        }
    }
}
impl Registry {
    fn reserve(&mut self, account: &CpuAccount) -> Result<()> {
        if self.entries.len() < self.entries.capacity() {
            return Ok(());
        }
        let account = account.at(RenderCpuScope::ImagePayloads);
        account.phase(|| {
            let existing = self.entries.len();
            let count = existing
                .checked_add(1)
                .ok_or_else(|| account.capacity_error(RenderResource::RetainedCpuBytes))?;
            let relocated = u64::try_from(existing)
                .map_err(|_| account.capacity_error(RenderResource::RetainedCpuBytes))?;
            account.preview(
                RenderResource::RetainedCpuBytes,
                account.sized::<Entry>(count, RenderResource::RetainedCpuBytes)?,
            )?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(count)
                .map_err(|_| account.allocation_error(RenderResource::RetainedCpuBytes))?;
            let hold = account.acquire(
                RenderResource::RetainedCpuBytes,
                account.sized::<Entry>(replacement.capacity(), RenderResource::RetainedCpuBytes)?,
            )?;
            if relocated != 0
                && let Err(error) = account.checked_work(RenderWork::StorageRelocations, relocated)
            {
                account.maintenance_phase(move || {
                    drop(replacement);
                    drop(hold);
                });
                return Err(error);
            }
            replacement.append(&mut self.entries);
            std::mem::swap(&mut self.entries, &mut replacement);
            if let Some(old) = self.backing.replace(hold) {
                let account = old.account();
                account.maintenance_phase(|| {
                    drop(replacement);
                    drop(old);
                });
            } else {
                drop(replacement);
            }
            Ok(())
        })
    }
    fn next(&mut self, core: &Core) -> Result<Key> {
        let next = self
            .next_key
            .checked_add(1)
            .ok_or_else(|| core.record(RenderAccountingError::Invariant))?;
        self.next_key = next;
        Ok(Key(next))
    }
    fn image(
        &mut self,
        account: &CpuAccount,
        image: &Image,
        forward: Option<&Account>,
    ) -> Result<Key> {
        let (backing, _) = image.data.data.clone().into_raw_parts();
        for entry in &mut self.entries {
            account.probe(forward)?;
            if let Payload::ImageAdapter {
                backing: existing, ..
            } = &entry.payload
                && Arc::ptr_eq(existing, &backing)
            {
                entry.add_lease(&account.core)?;
                return Ok(entry.key);
            }
        }
        let pixels = self.pixels(account, &image.bytes, forward)?;
        let result = (|| {
            self.reserve(account)?;
            // Image::from_rgba allocates Arc::new(Arc::clone(pixel_bytes)).
            // Count that known typed payload, excluding opaque Arc headers.
            let hold = account.acquire(
                RenderResource::RetainedCpuBytes,
                std::mem::size_of::<Arc<[u8]>>(),
            )?;
            let key = self.next(&account.core)?;
            self.entries.push(Entry {
                key,
                leases: 1,
                payload: Payload::ImageAdapter { backing, pixels },
                dependencies: None,
                hold,
            });
            Ok(key)
        })();
        if result.is_err() {
            self.release(pixels, &account.core);
        }
        result
    }
    fn pixels(
        &mut self,
        account: &CpuAccount,
        bytes: &Arc<[u8]>,
        forward: Option<&Account>,
    ) -> Result<Key> {
        for entry in &mut self.entries {
            account.probe(forward)?;
            if let Payload::Pixels(existing) = &entry.payload
                && Arc::ptr_eq(existing, bytes)
            {
                entry.add_lease(&account.core)?;
                return Ok(entry.key);
            }
        }
        self.reserve(account)?;
        let hold = account.acquire(RenderResource::RetainedCpuBytes, bytes.len())?;
        let key = self.next(&account.core)?;
        self.entries.push(Entry {
            key,
            leases: 1,
            payload: Payload::Pixels(bytes.clone()),
            dependencies: None,
            hold,
        });
        Ok(key)
    }
    fn font(
        &mut self,
        account: &CpuAccount,
        font: &surgeist_font::FontResource,
        forward: Option<&Account>,
    ) -> Result<Key> {
        let identity = font.identity();
        for entry in &mut self.entries {
            account.probe(forward)?;
            if let Payload::Font(existing) = &entry.payload
                && *existing == identity
            {
                entry.add_lease(&account.core)?;
                return Ok(entry.key);
            }
        }
        self.reserve(account)?;
        let hold = account.acquire(RenderResource::RetainedCpuBytes, font.retained_bytes())?;
        let key = self.next(&account.core)?;
        self.entries.push(Entry {
            key,
            leases: 1,
            payload: Payload::Font(identity),
            dependencies: None,
            hold,
        });
        Ok(key)
    }
    fn release(&mut self, key: Key, core: &Arc<Core>) {
        let mut found = None;
        for (index, entry) in self.entries.iter().enumerate() {
            core.mandatory_apply(
                RenderCpuScope::Cleanup,
                Change::Work(RenderWork::ResourceProbes, 1),
            );
            if entry.key == key {
                found = Some(index);
                break;
            }
        }
        let Some(index) = found else {
            core.remember(RenderAccountingError::Invariant);
            return;
        };
        let entry = &mut self.entries[index];
        let Some(leases) = entry.leases.checked_sub(1) else {
            core.remember(RenderAccountingError::Invariant);
            return;
        };
        entry.leases = leases;
        if leases != 0 {
            return;
        }
        let Entry {
            payload,
            mut dependencies,
            hold,
            ..
        } = self.entries.remove(index);
        // The entry is already absent while recursive subordinate removal runs.
        let account = hold.account();
        account.maintenance_phase(|| {
            if let Payload::ImageAdapter { backing, pixels } = payload {
                drop(backing);
                self.release(pixels, core);
            } else {
                drop(payload);
            }
            if let Some(dependencies) = &mut dependencies {
                while let Some(key) = dependencies.keys.pop() {
                    self.release(key, core);
                }
            }
            drop(dependencies);
            drop(hold);
        });
    }
}

pub(crate) struct CpuCheckpoint {
    cpu: RendererCpu,
    checkpoint: Option<Checkpoint>,
    report_backing: Option<ReportBacking<RenderCpuScope, IntervalNumbers>>,
}
impl CpuCheckpoint {
    pub(crate) fn finish(mut self) -> Result<RenderCpuInterval> {
        let checkpoint = self
            .checkpoint
            .take()
            .ok_or(RenderAccountingError::Invariant)?;
        let core = &self.cpu.0.core;
        let interval = core
            .ledger
            .end_checkpoint(&checkpoint)
            .map_err(|e| core.record(core.diagnostic(e)))?;
        Ok(RenderCpuInterval::new(
            RenderCpuIdentity(core.scopes[RenderCpuScope::Root.index()]),
            interval,
            &core.scopes,
            self.report_backing
                .take()
                .ok_or(RenderAccountingError::Invariant)?,
        )?)
    }
}
impl Drop for CpuCheckpoint {
    fn drop(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take() {
            let core = &self.cpu.0.core;
            if let Err(error) = core.ledger.end_checkpoint(&checkpoint) {
                core.remember(core.diagnostic(error));
            }
        }
    }
}

#[cfg(test)]
mod tests;
