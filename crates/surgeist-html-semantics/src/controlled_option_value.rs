//! Controlled formation and one OptionValue read over an ordinary captured tree.
use crate::accounting::{LedgerError, wide};
use crate::capture::preflight::ActionPolicy;
use crate::*;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::{error::Error, fmt};
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapRefusal, HtmlCapScope, HtmlHostScope,
    HtmlLedgerOutcome, HtmlLimits, HtmlResource, HtmlWork, classify,
};
use surgeist_html::{
    construction::{HtmlDoctype, HtmlDocumentMode},
    values::{DocumentString, HtmlAttributesInput, HtmlNameInput, HtmlValueInterpreter},
};
use surgeist_ledger::{Change, Operation, Scope, SharedOperation, Tariff, Tariffs};

/// Total producer live allowance after excluding the producer's current graph
/// from the caller's other live storage. Work is the remaining allowance for
/// this call, rather than a reset of the owner's cumulative work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlOptionValueEnvelope {
    bytes: usize,
    units: usize,
    work: u64,
}
impl HtmlOptionValueEnvelope {
    pub const fn new(live_bytes: usize, live_utf16: usize, remaining_work: u64) -> Self {
        Self {
            bytes: live_bytes,
            units: live_utf16,
            work: remaining_work,
        }
    }
}

/// Represented storage, including spare Vec capacity and shared payloads per
/// occurrence; excludes Arc control blocks and allocator bookkeeping.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlOptionValueStorageUsage {
    pub(crate) bytes: usize,
    pub(crate) units: usize,
}
impl HtmlOptionValueStorageUsage {
    pub fn retained_bytes(self) -> usize {
        self.bytes
    }
    pub fn utf16_extent(self) -> usize {
        self.units
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOptionValueResource {
    LiveBytes,
    LiveUtf16,
    Work,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlOptionValueControlError {
    Cancelled,
    LimitExceeded {
        resource: HtmlOptionValueResource,
        limit: u64,
        required: u64,
    },
    RepresentationExhausted,
    /// The ledger that carries the owner's accounting failed in a way that is not a refused
    /// allowance: the session returned storage it never took from this execution, a ledger could
    /// not allocate, or a change the owner does not charge. Valid captures and valid use never
    /// reach the release and change causes; they report a defect in this crate's accounting.
    Accounting(HtmlAccountingFault),
}
impl HtmlOptionValueControlError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::Cancelled | Self::LimitExceeded { .. } | Self::RepresentationExhausted => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlControlledOptionValueError {
    Control(HtmlOptionValueControlError),
    Capture(HtmlCaptureError),
    ElementValues(HtmlCaptureElementError<std::convert::Infallible>),
    Context(HtmlSemanticContextError),
    Semantic(HtmlSemanticFailure),
    AllocationFailed,
    InvalidPhase,
}
impl HtmlControlledOptionValueError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Control(error) => error.accounting_fault(),
            Self::Capture(error) => error.accounting_fault(),
            Self::ElementValues(error) => error.accounting_fault(),
            Self::Context(error) => error.accounting_fault(),
            Self::Semantic(error) => error.accounting_fault(),
            Self::AllocationFailed | Self::InvalidPhase => None,
        }
    }
}

/// Usage remains available after every refused call, including setup failures.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlControlledOptionValueUsage {
    work: u64,
    storage: HtmlOptionValueStorageUsage,
    peak: HtmlOptionValueStorageUsage,
    semantic: HtmlSemanticUsage,
}
impl HtmlControlledOptionValueUsage {
    pub fn work(self) -> u64 {
        self.work
    }
    pub fn storage(self) -> HtmlOptionValueStorageUsage {
        self.storage
    }
    pub fn peak_storage(self) -> HtmlOptionValueStorageUsage {
        self.peak
    }
    pub fn semantic(self) -> HtmlSemanticUsage {
        self.semantic
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlControlledOptionValueFailure {
    kind: HtmlControlledOptionValueError,
    usage: HtmlControlledOptionValueUsage,
}
impl HtmlControlledOptionValueFailure {
    pub fn kind(&self) -> &HtmlControlledOptionValueError {
        &self.kind
    }
    /// The accounting fault this failure carries, directly or inside an error it wraps.
    pub fn accounting_fault(&self) -> Option<HtmlAccountingFault> {
        self.kind.accounting_fault()
    }
    pub fn usage(&self) -> HtmlControlledOptionValueUsage {
        self.usage
    }
    /// Moves the original typed error, including its qualified semantic failure.
    /// Copy `usage()` first when the caller also needs its accounting receipt.
    pub fn into_error(self) -> HtmlControlledOptionValueError {
        self.kind
    }
}
impl fmt::Display for HtmlOptionValueControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlOptionValueControlError {}
impl fmt::Display for HtmlControlledOptionValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for HtmlControlledOptionValueError {}
impl fmt::Display for HtmlControlledOptionValueFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}
impl Error for HtmlControlledOptionValueFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

/// Reads and writes one counter.
///
/// The cancellation test hook is an atomic so that a session reaching `Control` through `&Control`
/// stays `Sync`, which keeps `HtmlSemanticSession` `Send + Sync`. It is not for concurrent
/// mutation: one thread drives a session and its owner at a time, and every thread hand-off
/// already synchronizes. `Relaxed` is sufficient because no counter orders any other memory
/// access.
trait Counter<T> {
    fn get(&self) -> T;
    fn set(&self, value: T);
}
impl Counter<u64> for AtomicU64 {
    fn get(&self) -> u64 {
        self.load(Ordering::Relaxed)
    }
    fn set(&self, value: u64) {
        self.store(value, Ordering::Relaxed);
    }
}

fn narrow(value: u64) -> Result<usize, HtmlOptionValueControlError> {
    usize::try_from(value).map_err(|_| HtmlOptionValueControlError::RepresentationExhausted)
}

/// The caps one call's envelope sets: the live bytes and UTF-16 units it allows, and the work the
/// scope may reach, which is the work admitted so far plus the call's remaining work.
fn gate_limits(bytes: usize, units: usize, work: u64) -> HtmlLimits {
    HtmlLimits::from_fn(
        |dimension| match dimension {
            HtmlWork::SemanticStep => work,
            _ => u64::MAX,
        },
        |dimension| match dimension {
            HtmlResource::RetainedBytes => wide(bytes),
            HtmlResource::RetainedUtf16Units => wide(units),
            _ => u64::MAX,
        },
    )
}

/// The storage and work of one controlled option value capture, charged to a ledger operation of
/// its own.
///
/// The `gate` scope carries the caps of the current call's envelope. It admits every prospective
/// total of live and temporary storage, which records its peak, and every step of work, including
/// the nested session's: the session's scopes are created below it, so its work counts in the
/// call's work cap. The `retained` and `temporary` scopes hold the storage itself, and the
/// `observed` scope records peak evidence for storage that exceeded the allowance. Held storage is
/// never refused by the gate, because the caller has already decided its admission.
/// The ledger-backed figures of the usage the owner last read.
#[derive(Clone, Copy, Default)]
struct LastUsage {
    work: u64,
    storage: HtmlOptionValueStorageUsage,
    peak: HtmlOptionValueStorageUsage,
}

pub(crate) struct Control {
    ledger: SharedOperation<HtmlWork, HtmlResource>,
    gate: Scope,
    retained: Scope,
    temporary: Scope,
    observed: Scope,
    /// The remaining work the current call was given, which the work cap is measured from.
    allowance: AtomicU64,
    /// The ledger-backed figures of the last usage read, which a read that cannot reach the ledger
    /// returns. A mutex keeps the owner `Sync`, as `usage` takes `&self`.
    last: Mutex<LastUsage>,
    semantic: Mutex<HtmlSemanticUsage>,
    #[cfg(test)]
    cancel_after: AtomicU64,
}
impl Control {
    fn new(envelope: HtmlOptionValueEnvelope) -> Result<Self, HtmlOptionValueControlError> {
        let ledger_error = Self::error;
        let mut operation = Operation::new(
            HtmlCapScope::OptionValue.name(),
            gate_limits(envelope.bytes, envelope.units, envelope.work),
            8,
        )
        .map_err(ledger_error)?;
        let gate = operation.root();
        let mut child = |name: &'static str| {
            operation
                .child(
                    gate,
                    name,
                    HtmlLimits::unlimited(),
                    Tariffs::from_fn(
                        |work| Tariff::new(work, 0),
                        |resource| Tariff::new(resource, 0),
                    ),
                )
                .map_err(ledger_error)
        };
        let retained = child(HtmlAuxiliaryScope::OptionValueRetained.name())?;
        let temporary = child(HtmlAuxiliaryScope::OptionValueTemporary.name())?;
        let observed = child(HtmlAuxiliaryScope::OptionValueObserved.name())?;
        Ok(Self {
            ledger: operation.share(),
            gate,
            retained,
            temporary,
            observed,
            allowance: AtomicU64::new(envelope.work),
            last: Mutex::new(LastUsage::default()),
            semantic: Mutex::new(HtmlSemanticUsage::default()),
            #[cfg(test)]
            cancel_after: AtomicU64::new(u64::MAX),
        })
    }
    /// The control error of a ledger error that is not a refusal of the gate's caps: an overflow
    /// is exhaustion, and every other failure is the accounting fault.
    fn error(error: LedgerError) -> HtmlOptionValueControlError {
        match classify(error) {
            HtmlLedgerOutcome::Fault(fault) => HtmlOptionValueControlError::Accounting(fault),
            HtmlLedgerOutcome::Exhausted(_) => HtmlOptionValueControlError::RepresentationExhausted,
            HtmlLedgerOutcome::Cap(_) => {
                HtmlOptionValueControlError::Accounting(HtmlAccountingFault::UnexpectedChange)
            }
        }
    }
    /// Whether `scope` is the gate, whose refusals the nested session reports as this control's.
    pub(crate) fn is_gate(&self, scope: Scope) -> bool {
        scope == self.gate
    }
    /// The control error of a refusal by the gate: a refused storage cap keeps its resource, its
    /// limit and the total the storage would have reached, and a refused work cap the call's
    /// remaining work and the work spent since the call began.
    pub(crate) fn refusal(&self, refusal: &HtmlCapRefusal) -> HtmlOptionValueControlError {
        use HtmlOptionValueControlError as Error;
        let limit = refusal.limit();
        let Ok(required) = refusal.required() else {
            return Error::RepresentationExhausted;
        };
        match refusal.change() {
            Change::Acquire(HtmlResource::RetainedBytes, _) => Error::LimitExceeded {
                resource: HtmlOptionValueResource::LiveBytes,
                limit,
                required,
            },
            Change::Acquire(HtmlResource::RetainedUtf16Units, _) => Error::LimitExceeded {
                resource: HtmlOptionValueResource::LiveUtf16,
                limit,
                required,
            },
            Change::Work(HtmlWork::SemanticStep, _) => {
                let allowance = self.allowance.get();
                match limit
                    .checked_sub(allowance)
                    .and_then(|start| required.checked_sub(start))
                {
                    Some(spent) => Error::LimitExceeded {
                        resource: HtmlOptionValueResource::Work,
                        limit: allowance,
                        required: spent,
                    },
                    None => Error::Accounting(HtmlAccountingFault::UnexpectedChange),
                }
            }
            _ => Error::Accounting(HtmlAccountingFault::UnexpectedChange),
        }
    }
    fn refused(&self, error: LedgerError) -> HtmlOptionValueControlError {
        match classify(error) {
            HtmlLedgerOutcome::Cap(refusal) => self.refusal(&refusal),
            _ => Self::error(error),
        }
    }
    fn storage(
        &self,
        scope: Scope,
        peak: bool,
    ) -> Result<HtmlOptionValueStorageUsage, HtmlOptionValueControlError> {
        let usage = self.ledger.usage(scope).map_err(Self::error)?;
        let read = |dimension| {
            narrow(if peak {
                usage.peak(dimension)
            } else {
                usage.live(dimension)
            })
        };
        Ok(HtmlOptionValueStorageUsage {
            bytes: read(HtmlResource::RetainedBytes)?,
            units: read(HtmlResource::RetainedUtf16Units)?,
        })
    }
    fn live(&self) -> Result<HtmlOptionValueStorageUsage, HtmlOptionValueControlError> {
        self.storage(self.retained, false)
    }
    fn held_temporary(&self) -> Result<HtmlOptionValueStorageUsage, HtmlOptionValueControlError> {
        self.storage(self.temporary, false)
    }
    /// The largest total of live and temporary storage admitted or observed.
    fn peak(&self) -> Result<HtmlOptionValueStorageUsage, HtmlOptionValueControlError> {
        let admitted = self.storage(self.gate, true)?;
        let observed = self.storage(self.observed, true)?;
        Ok(HtmlOptionValueStorageUsage {
            bytes: admitted.bytes.max(observed.bytes),
            units: admitted.units.max(observed.units),
        })
    }
    fn work_done(&self) -> Result<u64, HtmlOptionValueControlError> {
        Ok(self
            .ledger
            .usage(self.gate)
            .map_err(Self::error)?
            .work(HtmlWork::SemanticStep))
    }
    /// The usage after every call, including refused ones. A figure that cannot be read from the
    /// ledger is the last one read: an unreadable ledger has latched a defect, which the next
    /// charge reports as an accounting fault.
    fn usage(&self) -> HtmlControlledOptionValueUsage {
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if let Ok(work) = self.work_done() {
            last.work = work;
        }
        if let Ok(storage) = self.live() {
            last.storage = storage;
        }
        if let Ok(peak) = self.peak() {
            last.peak = peak;
        }
        HtmlControlledOptionValueUsage {
            work: last.work,
            storage: last.storage,
            peak: last.peak,
            semantic: self
                .semantic
                .lock()
                .map_or_else(|poisoned| *poisoned.into_inner(), |usage| *usage),
        }
    }
    pub(crate) fn poll(&self, cancel: &AtomicBool) -> Result<(), HtmlOptionValueControlError> {
        if cancel.load(Ordering::Relaxed) {
            Err(HtmlOptionValueControlError::Cancelled)
        } else {
            Ok(())
        }
    }
    pub(crate) fn step(
        &self,
        units: usize,
        cancel: &AtomicBool,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.poll(cancel)?;
        self.ledger
            .apply(self.gate, Change::Work(HtmlWork::SemanticStep, wide(units)))
            .map_err(|error| self.refused(error))?;
        #[cfg(test)]
        self.after_work(cancel);
        Ok(())
    }
    /// Test scheduling only: cancels once this much work is admitted.
    #[cfg(test)]
    pub(crate) fn after_work(&self, cancel: &AtomicBool) {
        if self
            .work_done()
            .is_ok_and(|work| work >= self.cancel_after.get())
        {
            cancel.store(true, Ordering::Relaxed);
        }
    }
    /// The live and temporary storage and `extra`, which is what the allowance has to cover.
    fn required(
        &self,
        extra: HtmlOptionValueStorageUsage,
    ) -> Result<HtmlOptionValueStorageUsage, HtmlOptionValueControlError> {
        let live = self.live()?;
        let temporary = self.held_temporary()?;
        Ok(HtmlOptionValueStorageUsage {
            bytes: live
                .bytes
                .checked_add(temporary.bytes)
                .and_then(|n| n.checked_add(extra.bytes))
                .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?,
            units: live
                .units
                .checked_add(temporary.units)
                .and_then(|n| n.checked_add(extra.units))
                .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?,
        })
    }
    pub(crate) fn admit(
        &self,
        extra: HtmlOptionValueStorageUsage,
    ) -> Result<(), HtmlOptionValueControlError> {
        let required = self.required(extra)?;
        // The gate holds the total for the batch only: it checks the allowance and keeps the peak.
        self.ledger
            .apply_batch(
                self.gate,
                &[
                    Change::Acquire(HtmlResource::RetainedBytes, wide(required.bytes)),
                    Change::Acquire(HtmlResource::RetainedUtf16Units, wide(required.units)),
                    Change::Release(HtmlResource::RetainedBytes, wide(required.bytes)),
                    Change::Release(HtmlResource::RetainedUtf16Units, wide(required.units)),
                ],
            )
            .map_err(|error| self.refused(error))
    }
    /// Records the peak of a total the caller already holds or has, whatever the allowance says.
    fn observed(
        &self,
        extra: HtmlOptionValueStorageUsage,
    ) -> Result<(), HtmlOptionValueControlError> {
        let required = self.required(extra)?;
        self.ledger
            .apply_batch(
                self.observed,
                &[
                    Change::Acquire(HtmlResource::RetainedBytes, wide(required.bytes)),
                    Change::Acquire(HtmlResource::RetainedUtf16Units, wide(required.units)),
                    Change::Release(HtmlResource::RetainedBytes, wide(required.bytes)),
                    Change::Release(HtmlResource::RetainedUtf16Units, wide(required.units)),
                ],
            )
            .map_err(Self::error)
    }
    fn scope(&self, retained: bool) -> Scope {
        if retained {
            self.retained
        } else {
            self.temporary
        }
    }
    /// Takes storage into custody without checking the allowance.
    fn hold(
        &self,
        retained: bool,
        value: HtmlOptionValueStorageUsage,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.ledger
            .apply_batch(
                self.scope(retained),
                &[
                    Change::Acquire(HtmlResource::RetainedBytes, wide(value.bytes)),
                    Change::Acquire(HtmlResource::RetainedUtf16Units, wide(value.units)),
                ],
            )
            .map_err(Self::error)
    }
    pub(crate) fn add(
        &self,
        value: HtmlOptionValueStorageUsage,
        retained: bool,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.admit(value)?;
        self.hold(retained, value)
    }
    /// Returns storage that an earlier `add` took. Returning more than is held changes nothing and
    /// is a protocol breach.
    pub(crate) fn release(
        &self,
        value: HtmlOptionValueStorageUsage,
        retained: bool,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.ledger
            .apply_batch(
                self.scope(retained),
                &[
                    Change::Release(HtmlResource::RetainedBytes, wide(value.bytes)),
                    Change::Release(HtmlResource::RetainedUtf16Units, wide(value.units)),
                ],
            )
            .map_err(Self::error)
    }
    /// Gives every scope back what it holds, once. A refused give-back keeps the custody and
    /// latches nothing here; the ledger latches a defect itself.
    fn release_all(&self) {
        for scope in [self.temporary, self.retained] {
            let Ok(usage) = self.ledger.usage(scope) else {
                continue;
            };
            let _ = self.ledger.apply_batch(
                scope,
                &[
                    Change::Release(
                        HtmlResource::RetainedBytes,
                        usage.live(HtmlResource::RetainedBytes),
                    ),
                    Change::Release(
                        HtmlResource::RetainedUtf16Units,
                        usage.live(HtmlResource::RetainedUtf16Units),
                    ),
                ],
            );
        }
    }
    /// Records temporary storage whose admission the caller has already decided.
    fn hold_temporary(
        &self,
        value: HtmlOptionValueStorageUsage,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.hold(false, value)
    }
    /// Moves temporary storage into the retained total. Either both counts change or neither does.
    fn transfer(
        &self,
        value: HtmlOptionValueStorageUsage,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.release(value, false)?;
        if let Err(error) = self.hold(true, value) {
            // The retained total cannot take it: the temporary custody comes back.
            self.hold(false, value)?;
            return Err(error);
        }
        Ok(())
    }
    fn begin(
        &self,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<(), HtmlOptionValueControlError> {
        let work = self.work_done()?;
        self.allowance.set(envelope.work);
        self.ledger
            .set_limits(
                self.gate,
                gate_limits(
                    envelope.bytes,
                    envelope.units,
                    work.saturating_add(envelope.work),
                ),
            )
            .map_err(Self::error)?;
        self.poll(cancel)?;
        self.admit(Default::default())
    }
    fn failure(&self, kind: HtmlControlledOptionValueError) -> HtmlControlledOptionValueFailure {
        HtmlControlledOptionValueFailure {
            kind,
            usage: self.usage(),
        }
    }
}
impl Drop for Control {
    fn drop(&mut self) {
        self.release_all();
    }
}

#[derive(Clone, Copy)]
pub(crate) struct QueryControl<'a> {
    control: &'a Control,
    cancel: &'a AtomicBool,
}
impl QueryControl<'_> {
    pub(crate) fn poll(self) -> Result<(), HtmlOptionValueControlError> {
        self.control.poll(self.cancel)
    }
    pub(crate) fn step(self, work: usize) -> Result<(), HtmlOptionValueControlError> {
        self.control.step(work, self.cancel)
    }
    /// Test scheduling only: cancels once the configured work is admitted.
    #[cfg(test)]
    pub(crate) fn after_work(self) {
        self.control.after_work(self.cancel);
    }
    /// Whether `scope` is the control's gate, whose refusals it reports itself.
    pub(crate) fn is_gate(self, scope: Scope) -> bool {
        self.control.is_gate(scope)
    }
    pub(crate) fn refusal(self, refusal: &HtmlCapRefusal) -> HtmlOptionValueControlError {
        self.control.refusal(refusal)
    }
    fn bank(
        capacity: usize,
        record_size: usize,
        resource: HtmlSemanticResource,
    ) -> Result<HtmlOptionValueStorageUsage, HtmlOptionValueControlError> {
        Ok(HtmlOptionValueStorageUsage {
            bytes: capacity
                .checked_mul(record_size)
                .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?,
            units: if resource == HtmlSemanticResource::NestedOutputRecords {
                capacity
            } else {
                0
            },
        })
    }
    pub(crate) fn before_growth(
        self,
        old: usize,
        needed: usize,
        size: usize,
        resource: HtmlSemanticResource,
    ) -> Result<(), HtmlOptionValueControlError> {
        // The old backing stays in the current-live total while the complete
        // requested new backing and its copy work are admitted.
        self.step(old)?;
        self.control.admit(Self::bank(needed, size, resource)?)
    }
    pub(crate) fn after_growth(
        self,
        old: usize,
        actual: usize,
        size: usize,
        resource: HtmlSemanticResource,
    ) -> Result<(), HtmlOptionValueControlError> {
        let new = Self::bank(actual, size, resource)?;
        let outcome = self.control.admit(new);
        self.control.observed(new)?;
        let released = self
            .control
            .release(Self::bank(old, size, resource)?, false);
        let held = self.control.hold_temporary(new);
        outcome.and(released).and(held)
    }
    /// Returns the backing of scratch vectors that `after_growth` recorded, in bytes.
    pub(crate) fn release_scratch(self, bytes: usize) -> Result<(), HtmlOptionValueControlError> {
        self.control
            .release(HtmlOptionValueStorageUsage { bytes, units: 0 }, false)
    }
    pub(crate) fn string_copy(
        self,
        value: &DocumentString,
    ) -> Result<(), HtmlOptionValueControlError> {
        let usage = crate::capture::storage::option_value_string(value, &mut || self.step(1))
            .map_err(|error| match error {
                HtmlCaptureStorageError::Interrupted(error) => error,
                HtmlCaptureStorageError::RepresentationExhausted => {
                    HtmlOptionValueControlError::RepresentationExhausted
                }
            })?;
        self.control.add(
            HtmlOptionValueStorageUsage {
                bytes: usage.retained_bytes(),
                units: usage.utf16_extent(),
            },
            false,
        )
    }
    pub(crate) fn string_header(self) -> Result<(), HtmlOptionValueControlError> {
        self.step(1)?;
        self.control.add(
            HtmlOptionValueStorageUsage {
                bytes: size_of::<Vec<u16>>(),
                units: 0,
            },
            false,
        )
    }
}

impl crate::capture::preflight::ActionPolicy for QueryControl<'_> {
    type Error = HtmlOptionValueControlError;
    fn on_work(&mut self, units: usize) -> Result<(), Self::Error> {
        self.control.step(units, self.cancel)
    }
    fn on_reserve(&mut self, _entries: usize) -> Result<(), Self::Error> {
        self.control.poll(self.cancel)
    }
    fn on_storage_before(&mut self, requested: usize) -> Result<(), Self::Error> {
        self.control.poll(self.cancel)?;
        self.control.admit(HtmlOptionValueStorageUsage {
            bytes: requested,
            units: 0,
        })
    }
    fn on_storage_after(
        &mut self,
        old: usize,
        actual: usize,
        retained: bool,
    ) -> Result<(), Self::Error> {
        let control = self.control;
        // Requested backing was admitted before allocation. An allocator
        // may expose extra capacity; record it even on immediate refusal.
        let outcome = control.admit(HtmlOptionValueStorageUsage {
            bytes: actual,
            units: 0,
        });
        control.observed(HtmlOptionValueStorageUsage {
            bytes: actual,
            units: 0,
        })?;
        let released = control.release(
            HtmlOptionValueStorageUsage {
                bytes: old,
                units: 0,
            },
            retained,
        );
        let held = control.hold(
            retained,
            HtmlOptionValueStorageUsage {
                bytes: actual,
                units: 0,
            },
        );
        outcome.and(released).and(held)
    }
    fn on_storage_transfer(&mut self, bytes: usize) -> Result<(), Self::Error> {
        self.control
            .transfer(HtmlOptionValueStorageUsage { bytes, units: 0 })
    }
    fn on_element_values(
        &mut self,
        bytes: usize,
        units: usize,
        observed: bool,
    ) -> Result<(), Self::Error> {
        let control = self.control;
        let payload = HtmlOptionValueStorageUsage { bytes, units };
        let result = control
            .poll(self.cancel)
            .and_then(|()| control.admit(payload));
        if observed {
            // An actual destination capacity remains peak evidence even
            // when it immediately exceeds the prospective allowance.
            control.observed(payload)?;
        }
        result
    }
}

#[cfg(test)]
impl Control {
    pub(crate) fn for_tests(envelope: HtmlOptionValueEnvelope) -> Self {
        Self::new(envelope).unwrap()
    }
    pub(crate) fn temporary_bytes(&self) -> usize {
        self.held_temporary().unwrap_or_default().bytes
    }
    pub(crate) fn temporary_usage(&self) -> HtmlOptionValueStorageUsage {
        self.held_temporary().unwrap_or_default()
    }
    pub(crate) fn live_usage(&self) -> HtmlOptionValueStorageUsage {
        self.live().unwrap_or_default()
    }
    pub(crate) fn peak_usage(&self) -> HtmlOptionValueStorageUsage {
        self.peak().unwrap_or_default()
    }
    /// Takes storage into the temporary or retained custody, whatever the allowance says.
    pub(crate) fn hold_for_tests(&self, retained: bool, value: HtmlOptionValueStorageUsage) {
        self.hold(retained, value).unwrap();
    }
    /// Replaces the allowance the way a new call's envelope does, keeping the work cap.
    pub(crate) fn set_envelope(&self, envelope: HtmlOptionValueEnvelope) {
        let work = self.work_done().unwrap();
        self.ledger
            .set_limits(
                self.gate,
                gate_limits(
                    envelope.bytes,
                    envelope.units,
                    work.saturating_add(self.allowance.get()),
                ),
            )
            .unwrap();
    }
}
#[cfg(test)]
impl<'a> QueryControl<'a> {
    pub(crate) fn for_tests(control: &'a Control, cancel: &'a AtomicBool) -> Self {
        Self { control, cancel }
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "The builder holds its records inline, and boxing it would add an allocation the controlled option-value operation does not admit."
)]
enum Phase {
    Building(HtmlCaptureBuilder),
    Finished {
        capture: HtmlCapture,
        report: Option<HtmlSemanticReport>,
        failure: Option<HtmlControlledOptionValueError>,
    },
}

/// One controlled ordinary capture and one original OptionValue query.
/// Storage usage is cached and includes this owner's fixed inline header once.
/// Context/session borrow the capture locally; report identities do not own its graph.
pub struct HtmlControlledOptionValueCapture {
    phase: Phase,
    control: Control,
}

impl Control {
    /// The query control that applies this storage policy under `cancel`.
    fn query<'a>(&'a self, cancel: &'a AtomicBool) -> QueryControl<'a> {
        QueryControl {
            control: self,
            cancel,
        }
    }
    fn capture_failure(
        &self,
        failure: crate::capture::preflight::Failure<HtmlOptionValueControlError>,
    ) -> HtmlControlledOptionValueFailure {
        use crate::capture::preflight::Failure;
        self.failure(match failure {
            Failure::Admission(error) => HtmlControlledOptionValueError::Control(error),
            Failure::Capture(error) => HtmlControlledOptionValueError::Capture(error),
            Failure::AllocationFailed => HtmlControlledOptionValueError::AllocationFailed,
        })
    }
    fn payload_failure(
        &self,
        error: HtmlCaptureStorageError<HtmlOptionValueControlError>,
    ) -> HtmlControlledOptionValueFailure {
        self.failure(HtmlControlledOptionValueError::Control(match error {
            HtmlCaptureStorageError::Interrupted(error) => error,
            HtmlCaptureStorageError::RepresentationExhausted => {
                HtmlOptionValueControlError::RepresentationExhausted
            }
        }))
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    fn add_payload(
        &self,
        usage: HtmlCaptureRetainedStorageUsage,
    ) -> Result<HtmlOptionValueStorageUsage, HtmlControlledOptionValueFailure> {
        let payload = HtmlOptionValueStorageUsage {
            bytes: usage.retained_bytes(),
            units: usage.utf16_extent(),
        };
        self.add(payload, false)
            .map_err(|e| self.failure(HtmlControlledOptionValueError::Control(e)))?;
        Ok(payload)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    fn string(
        &self,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<HtmlOptionValueStorageUsage, HtmlControlledOptionValueFailure> {
        let usage =
            crate::capture::storage::option_value_string(value, &mut || self.step(1, cancel))
                .map_err(|e| self.payload_failure(e))?;
        self.add_payload(usage)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    fn finish_add<T>(
        &self,
        result: Result<T, crate::capture::preflight::Failure<HtmlOptionValueControlError>>,
        payload: HtmlOptionValueStorageUsage,
    ) -> Result<T, HtmlControlledOptionValueFailure> {
        let transferred = if result.is_ok() {
            self.transfer(payload)
        } else {
            Ok(())
        };
        self.clear_temporaries();
        let value = result.map_err(|e| self.capture_failure(e))?;
        transferred.map_err(|e| self.failure(HtmlControlledOptionValueError::Control(e)))?;
        Ok(value)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    fn strings(
        &self,
        values: &[Option<&DocumentString>],
        cancel: &AtomicBool,
    ) -> Result<HtmlOptionValueStorageUsage, HtmlControlledOptionValueFailure> {
        let result = (|| {
            let mut payload = HtmlOptionValueStorageUsage::default();
            for value in values.iter().flatten() {
                let next = self.string(value, cancel)?;
                payload.bytes = payload.bytes.checked_add(next.bytes).ok_or_else(|| {
                    self.failure(HtmlControlledOptionValueError::Control(
                        HtmlOptionValueControlError::RepresentationExhausted,
                    ))
                })?;
                payload.units = payload.units.checked_add(next.units).ok_or_else(|| {
                    self.failure(HtmlControlledOptionValueError::Control(
                        HtmlOptionValueControlError::RepresentationExhausted,
                    ))
                })?;
            }
            Ok(payload)
        })();
        if result.is_err() {
            self.clear_temporaries();
        }
        result
    }
    /// Gives the temporary scope back what it holds. A refused give-back is a ledger defect,
    /// which stays latched in the operation and fails the next charge.
    fn clear_temporaries(&self) {
        if let Ok(held) = self.held_temporary() {
            let _ = self.release(held, false);
        }
    }
}

impl HtmlControlledOptionValueCapture {
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn new(
        limits: HtmlCaptureLimits,
        host: HtmlHostFactsRevision,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<Self, HtmlControlledOptionValueFailure> {
        let control = Control::new(envelope).map_err(|error| HtmlControlledOptionValueFailure {
            kind: HtmlControlledOptionValueError::Control(error),
            usage: HtmlControlledOptionValueUsage::default(),
        })?;
        let setup = (|| {
            control.step(1, cancel)?;
            control.add(
                HtmlOptionValueStorageUsage {
                    bytes: size_of::<Self>(),
                    units: 0,
                },
                true,
            )
        })();
        if let Err(error) = setup {
            return Err(control.failure(HtmlControlledOptionValueError::Control(error)));
        }
        let mut builder = HtmlCaptureBuilder::new(limits, host);
        builder.enable_controlled_storage();
        Ok(Self {
            phase: Phase::Building(builder),
            control,
        })
    }
    pub fn storage_usage(&self) -> HtmlOptionValueStorageUsage {
        self.control.live().unwrap_or_default()
    }
    pub fn usage(&self) -> HtmlControlledOptionValueUsage {
        self.control.usage()
    }
    /// Borrows the actual successful retained report without repeating the query.
    /// This permits the caller to join usage before admitting its output copy.
    pub fn report(&self) -> Option<&HtmlSemanticReport> {
        match &self.phase {
            Phase::Finished { report, .. } => report.as_ref(),
            Phase::Building(_) => None,
        }
    }
    pub fn capture_usage(&self) -> HtmlCaptureUsage {
        match &self.phase {
            Phase::Building(builder) => builder.usage(),
            Phase::Finished { capture, .. } => capture.usage(),
        }
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    fn begin(
        &self,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<(), HtmlControlledOptionValueFailure> {
        self.control.begin(envelope, cancel).map_err(|e| {
            self.control
                .failure(HtmlControlledOptionValueError::Control(e))
        })
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn add_document(
        &mut self,
        mode: HtmlDocumentMode,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
        self.begin(envelope, cancel)?;
        let Phase::Building(builder) = &mut self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        let result = builder.add_document_controlled(
            mode,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
            &mut |action| self.control.query(cancel).action(action),
        );
        self.control.finish_add(result, Default::default())
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn add_element(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        name: &(impl HtmlNameInput + ?Sized),
        attributes: &(impl HtmlAttributesInput + ?Sized),
        saved_is: Option<&DocumentString>,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
        self.begin(envelope, cancel)?;
        let Phase::Building(builder) = &mut self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        let formed = builder
            .form_element_inputs_controlled(
                document,
                name,
                attributes,
                saved_is,
                HtmlCaptureRegistryInput::Null,
                &mut |action| self.control.query(cancel).action(action),
            )
            .map_err(|error| {
                // The failed helper has already dropped its unpublished storage.
                // Only the original capture banks remain retained by this owner.
                self.control.clear_temporaries();
                self.control.failure(match error {
                    crate::capture::FormFailure::AllocationFailed => {
                        HtmlControlledOptionValueError::AllocationFailed
                    }
                    crate::capture::FormFailure::Input(error) => match error {
                        HtmlCaptureElementError::Capture(error) => {
                            HtmlControlledOptionValueError::Capture(error)
                        }
                        HtmlCaptureElementError::Interrupted(error) => {
                            HtmlControlledOptionValueError::Control(error)
                        }
                        HtmlCaptureElementError::Attributes(error) => {
                            HtmlControlledOptionValueError::ElementValues(
                                HtmlCaptureElementError::Attributes(error),
                            )
                        }
                        HtmlCaptureElementError::InconsistentAttributes { index } => {
                            HtmlControlledOptionValueError::ElementValues(
                                HtmlCaptureElementError::InconsistentAttributes { index },
                            )
                        }
                    },
                })
            })?;
        let payload = self.control.add_payload(formed.usage)?;
        let result = builder.add_element_controlled(
            document,
            formed.name,
            formed.attributes,
            saved_is.map_or(HtmlCapturedIsValue::Absent, |value| {
                HtmlCapturedIsValue::Value(value.clone())
            }),
            HtmlCaptureRegistryInput::Null,
            &mut |action| self.control.query(cancel).action(action),
        );
        self.control.finish_add(result, payload)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    fn add_string(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        value: &DocumentString,
        text: bool,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
        self.begin(envelope, cancel)?;
        let Phase::Building(builder) = &mut self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        self.control.step(1, cancel).map_err(|e| {
            self.control
                .failure(HtmlControlledOptionValueError::Control(e))
        })?;
        builder
            .validate_controlled_document(document)
            .map_err(|e| {
                self.control
                    .failure(HtmlControlledOptionValueError::Capture(e))
            })?;
        let payload = self.control.string(value, cancel)?;
        let kind = if text {
            HtmlCapturedNodeKind::Text(value.clone())
        } else {
            HtmlCapturedNodeKind::Comment(value.clone())
        };
        let result = builder.add_leaf_controlled(document, kind, value.len(), &mut |action| {
            self.control.query(cancel).action(action)
        });
        self.control.finish_add(result, payload)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn add_text(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        text: &DocumentString,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
        self.add_string(document, text, true, envelope, cancel)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn add_comment(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        text: &DocumentString,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
        self.add_string(document, text, false, envelope, cancel)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn add_processing_instruction(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        target: &DocumentString,
        data: &DocumentString,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
        self.begin(envelope, cancel)?;
        let Phase::Building(builder) = &mut self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        self.control.step(1, cancel).map_err(|e| {
            self.control
                .failure(HtmlControlledOptionValueError::Control(e))
        })?;
        builder
            .validate_controlled_document(document)
            .map_err(|e| {
                self.control
                    .failure(HtmlControlledOptionValueError::Capture(e))
            })?;
        let payload = self.control.strings(&[Some(target), Some(data)], cancel)?;
        let units = target.len().checked_add(data.len()).ok_or_else(|| {
            self.control
                .failure(HtmlControlledOptionValueError::Control(
                    HtmlOptionValueControlError::RepresentationExhausted,
                ))
        });
        let units = match units {
            Ok(units) => units,
            Err(error) => {
                self.control.clear_temporaries();
                return Err(error);
            }
        };
        let kind = HtmlCapturedNodeKind::ProcessingInstruction(
            HtmlCapturedProcessingInstruction::controlled(target.clone(), data.clone()),
        );
        let result = builder.add_leaf_controlled(document, kind, units, &mut |action| {
            self.control.query(cancel).action(action)
        });
        self.control.finish_add(result, payload)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn add_doctype(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        name: &DocumentString,
        public_id: Option<&DocumentString>,
        system_id: Option<&DocumentString>,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
        self.begin(envelope, cancel)?;
        let Phase::Building(builder) = &mut self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        self.control.step(1, cancel).map_err(|e| {
            self.control
                .failure(HtmlControlledOptionValueError::Control(e))
        })?;
        builder
            .validate_controlled_document(document)
            .map_err(|e| {
                self.control
                    .failure(HtmlControlledOptionValueError::Capture(e))
            })?;
        let payload = self
            .control
            .strings(&[Some(name), public_id, system_id], cancel)?;
        let units = name
            .len()
            .checked_add(public_id.map_or(0, DocumentString::len))
            .and_then(|n| n.checked_add(system_id.map_or(0, DocumentString::len)))
            .ok_or_else(|| {
                self.control
                    .failure(HtmlControlledOptionValueError::Control(
                        HtmlOptionValueControlError::RepresentationExhausted,
                    ))
            });
        let units = match units {
            Ok(units) => units,
            Err(error) => {
                self.control.clear_temporaries();
                return Err(error);
            }
        };
        let kind = HtmlCapturedNodeKind::Doctype(HtmlDoctype::new(
            name.clone(),
            public_id.cloned(),
            system_id.cloned(),
        ));
        let result = builder.add_leaf_controlled(document, kind, units, &mut |action| {
            self.control.query(cancel).action(action)
        });
        self.control.finish_add(result, payload)
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn establish_children(
        &mut self,
        parent: &HtmlCaptureBuilderSubject,
        children: &[HtmlCaptureBuilderSubject],
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<(), HtmlControlledOptionValueFailure> {
        self.begin(envelope, cancel)?;
        let Phase::Building(builder) = &mut self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        let result = builder.establish_children_controlled(parent, children, &mut |action| {
            self.control.query(cancel).action(action)
        });
        self.control.finish_add(result, Default::default())
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn finish(
        &mut self,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<(), HtmlControlledOptionValueFailure> {
        self.begin(envelope, cancel)?;
        let Phase::Building(builder) = &mut self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        let result = {
            let mut scratch = crate::capture::preflight::DepthScratch::new();
            builder.preflight_finish(&mut scratch, &mut |action| {
                self.control.query(cancel).action(action)
            })
        };
        self.control.clear_temporaries();
        result.map_err(|e| self.control.capture_failure(e))?;
        self.control.step(1, cancel).map_err(|e| {
            self.control
                .failure(HtmlControlledOptionValueError::Control(e))
        })?;
        self.control
            .add(
                HtmlOptionValueStorageUsage {
                    bytes: HtmlCaptureBuilder::controlled_data_header_bytes(),
                    units: 0,
                },
                true,
            )
            .map_err(|e| {
                self.control
                    .failure(HtmlControlledOptionValueError::Control(e))
            })?;
        self.phase = Phase::Finished {
            capture: builder.publish_taking(),
            report: None,
            failure: None,
        };
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "Refusal preserves the original qualified error and complete usage without allocation."
    )]
    pub fn query_option_value(
        &mut self,
        subject: &HtmlCaptureBuilderSubject,
        interpreter: HtmlValueInterpreter,
        limits: HtmlSemanticLimits,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<&HtmlSemanticReport, HtmlControlledOptionValueFailure> {
        if let Phase::Finished {
            failure: Some(error),
            ..
        } = &self.phase
        {
            return Err(self.control.failure(error.clone()));
        }
        if let Err(error) = self.begin(envelope, cancel) {
            // A Finished owner's first query attempt is terminal even when
            // admission stops before constructing the original session.
            // Wrong-phase calls do not acquire a query failure slot.
            if let Phase::Finished {
                report: None,
                failure,
                ..
            } = &mut self.phase
            {
                *failure = Some(error.kind.clone());
            }
            return Err(error);
        }
        // The one match proves a Finished owner with no report and no failure, and it keeps the
        // slots for the outcome, so nothing re-matches the phase afterwards.
        let Phase::Finished {
            capture,
            report: report_slot @ None,
            failure: failure_slot @ None,
        } = &mut self.phase
        else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        let capture = &*capture;
        let result = (|| {
            // These empty-input constructors perform fixed work. All variable
            // work remains in the original session. Three qualification payload
            // occurrences cover session, sticky failure and returned failure.
            self.control
                .step(4, cancel)
                .map_err(HtmlControlledOptionValueError::Control)?;
            let fixed = size_of::<HtmlSemanticContext>()
                .checked_add(size_of::<HtmlSemanticContextInput>())
                .and_then(|n| n.checked_add(size_of::<HtmlSemanticSession<'_>>()))
                .and_then(|n| n.checked_add(size_of::<HtmlSemanticRequest>()))
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .and_then(|n| n.checked_add(size_of::<AtomicBool>()))
                .and_then(|n| n.checked_add(size_of::<HtmlSemanticQualification>().checked_mul(3)?))
                .ok_or(HtmlControlledOptionValueError::Control(
                    HtmlOptionValueControlError::RepresentationExhausted,
                ))?;
            self.control
                .add(
                    HtmlOptionValueStorageUsage {
                        bytes: fixed,
                        units: 0,
                    },
                    false,
                )
                .map_err(HtmlControlledOptionValueError::Control)?;
            let subject = capture
                .subject_id(subject)
                .map_err(HtmlControlledOptionValueError::Capture)?;
            let context = HtmlSemanticContext::new(
                capture,
                capture.host_facts_revision().clone(),
                HtmlSemanticContextInput::new(Vec::new()),
                0,
            )
            .map_err(HtmlControlledOptionValueError::Context)?;
            let accounting = HtmlHostScope::new(self.control.ledger.clone(), self.control.gate);
            let mut session = HtmlSemanticSession::with_accounting(
                capture,
                &context,
                interpreter,
                limits,
                HtmlSemanticCancellation::new(),
                &accounting,
            )
            .map_err(HtmlControlledOptionValueError::Semantic)?;
            session.control_option_value(QueryControl {
                control: &self.control,
                cancel,
            });
            let result = session.query(&[HtmlSemanticRequest::new(
                subject,
                HtmlSemanticQuery::OptionValue,
            )]);
            *self
                .control
                .semantic
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = session.usage();
            result.map_err(HtmlControlledOptionValueError::Semantic)
        })();
        match result {
            Ok(report) => {
                // Transfer the report's real result bank, one qualification
                // occurrence and actual result string occurrence while all
                // query temporary custody is still charged.
                let storage = (|| {
                    self.control.step(1, cancel)?;
                    let mut value = HtmlOptionValueStorageUsage {
                        bytes: report
                            .results_capacity_bytes()?
                            .checked_add(size_of::<HtmlSemanticQualification>())
                            .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?,
                        units: 0,
                    };
                    if let Some(result) = report.results().first()
                        && let HtmlSemanticFact::OptionValue(HtmlSemanticValue::Known(readback)) =
                            result.fact()
                    {
                        let string = crate::capture::storage::option_value_string(
                            readback.value(),
                            &mut || self.control.step(1, cancel),
                        )
                        .map_err(|error| match error {
                            HtmlCaptureStorageError::Interrupted(error) => error,
                            HtmlCaptureStorageError::RepresentationExhausted => {
                                HtmlOptionValueControlError::RepresentationExhausted
                            }
                        })?;
                        value.bytes = value
                            .bytes
                            .checked_add(string.retained_bytes())
                            .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?;
                        value.units = string.utf16_extent();
                    }
                    Ok::<_, HtmlOptionValueControlError>(value)
                })();
                // The storage moves into the retained total only after it was measured.
                let stored = storage.and_then(|storage| self.control.transfer(storage));
                if let Err(error) = stored {
                    let error = HtmlControlledOptionValueError::Control(error);
                    *failure_slot = Some(error.clone());
                    self.control.clear_temporaries();
                    return Err(self.control.failure(error));
                }
                self.control.clear_temporaries();
                Ok(&*report_slot.insert(report))
            }
            Err(error) => {
                // The query's failure is the first one and holds the sticky slot before any
                // accounting below runs; a later breach is kept only if the slot were empty.
                failure_slot.get_or_insert_with(|| error.clone());
                if let HtmlControlledOptionValueError::Semantic(failure) = &error
                    && failure.qualification().is_some()
                    && let Err(breach) = self.control.transfer(HtmlOptionValueStorageUsage {
                        bytes: size_of::<HtmlSemanticQualification>(),
                        units: 0,
                    })
                {
                    // A refusal moved nothing, so it cannot corrupt the counts.
                    failure_slot
                        .get_or_insert_with(|| HtmlControlledOptionValueError::Control(breach));
                }
                self.control.clear_temporaries();
                Err(self.control.failure(error))
            }
        }
    }
}

#[cfg(test)]
mod borrowed_tests;
#[cfg(test)]
mod tests;
