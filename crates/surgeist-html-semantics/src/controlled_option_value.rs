//! Controlled formation and one OptionValue read over an ordinary captured tree.
use crate::*;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::{error::Error, fmt};

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

trait Counter<T> {
    fn get(&self) -> T;
    fn set(&self, value: T);
    fn replace(&self, value: T) -> T;
}
impl Counter<u64> for AtomicU64 {
    fn get(&self) -> u64 {
        self.load(Ordering::Relaxed)
    }
    fn set(&self, value: u64) {
        self.store(value, Ordering::Relaxed);
    }
    fn replace(&self, value: u64) -> u64 {
        self.swap(value, Ordering::Relaxed)
    }
}
impl Counter<usize> for AtomicUsize {
    fn get(&self) -> usize {
        self.load(Ordering::Relaxed)
    }
    fn set(&self, value: usize) {
        self.store(value, Ordering::Relaxed);
    }
    fn replace(&self, value: usize) -> usize {
        self.swap(value, Ordering::Relaxed)
    }
}
struct StorageCounter {
    bytes: AtomicUsize,
    units: AtomicUsize,
}
impl StorageCounter {
    fn new() -> Self {
        Self {
            bytes: AtomicUsize::new(0),
            units: AtomicUsize::new(0),
        }
    }
    fn get(&self) -> HtmlOptionValueStorageUsage {
        HtmlOptionValueStorageUsage {
            bytes: self.bytes.get(),
            units: self.units.get(),
        }
    }
    fn set(&self, value: HtmlOptionValueStorageUsage) {
        self.bytes.set(value.bytes);
        self.units.set(value.units);
    }
}
struct EnvelopeCounter {
    storage: StorageCounter,
    work: AtomicU64,
}
impl EnvelopeCounter {
    fn new(value: HtmlOptionValueEnvelope) -> Self {
        let out = Self {
            storage: StorageCounter::new(),
            work: AtomicU64::new(0),
        };
        out.set(value);
        out
    }
    fn get(&self) -> HtmlOptionValueEnvelope {
        let storage = self.storage.get();
        HtmlOptionValueEnvelope::new(storage.bytes, storage.units, self.work.get())
    }
    fn set(&self, value: HtmlOptionValueEnvelope) {
        self.storage.set(HtmlOptionValueStorageUsage {
            bytes: value.bytes,
            units: value.units,
        });
        self.work.set(value.work);
    }
}
struct SemanticCounter {
    words: [AtomicUsize; 6],
}
impl SemanticCounter {
    fn new() -> Self {
        Self {
            words: std::array::from_fn(|_| AtomicUsize::new(0)),
        }
    }
    fn get(&self) -> HtmlSemanticUsage {
        HtmlSemanticUsage::controlled_snapshot(std::array::from_fn(|i| self.words[i].get()))
    }
    fn set(&self, value: HtmlSemanticUsage) {
        let words = [
            value.work(),
            value.output_records(),
            value.scratch_records(),
            value.allocated_bytes(),
            value.peak_scratch_records(),
            value.nested_output_records(),
        ];
        for (to, from) in self.words.iter().zip(words) {
            to.set(from);
        }
    }
}

pub(crate) struct Control {
    live: StorageCounter,
    temporary: StorageCounter,
    peak: StorageCounter,
    work: AtomicU64,
    start: AtomicU64,
    envelope: EnvelopeCounter,
    semantic: SemanticCounter,
    scratch_bytes: AtomicUsize,
    #[cfg(test)]
    cancel_after: AtomicU64,
}
impl Control {
    fn new(envelope: HtmlOptionValueEnvelope) -> Self {
        Self {
            live: StorageCounter::new(),
            temporary: StorageCounter::new(),
            peak: StorageCounter::new(),
            work: AtomicU64::new(0),
            start: AtomicU64::new(0),
            envelope: EnvelopeCounter::new(envelope),
            semantic: SemanticCounter::new(),
            scratch_bytes: AtomicUsize::new(0),
            #[cfg(test)]
            cancel_after: AtomicU64::new(u64::MAX),
        }
    }
    fn usage(&self) -> HtmlControlledOptionValueUsage {
        HtmlControlledOptionValueUsage {
            work: self.work.get(),
            storage: self.live.get(),
            peak: self.peak.get(),
            semantic: self.semantic.get(),
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
        let next = self
            .work
            .get()
            .checked_add(
                u64::try_from(units)
                    .map_err(|_| HtmlOptionValueControlError::RepresentationExhausted)?,
            )
            .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?;
        let spent = next - self.start.get();
        if spent > self.envelope.get().work {
            return Err(HtmlOptionValueControlError::LimitExceeded {
                resource: HtmlOptionValueResource::Work,
                limit: self.envelope.get().work,
                required: spent,
            });
        }
        self.work.set(next);
        #[cfg(test)]
        if next >= self.cancel_after.get() {
            cancel.store(true, Ordering::Relaxed);
        }
        Ok(())
    }
    fn required(
        &self,
        extra: HtmlOptionValueStorageUsage,
    ) -> Result<HtmlOptionValueStorageUsage, HtmlOptionValueControlError> {
        let live = self.live.get();
        let temporary = self.temporary.get();
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
        for (resource, value, limit) in [
            (
                HtmlOptionValueResource::LiveBytes,
                required.bytes,
                self.envelope.get().bytes,
            ),
            (
                HtmlOptionValueResource::LiveUtf16,
                required.units,
                self.envelope.get().units,
            ),
        ] {
            if value > limit {
                return Err(HtmlOptionValueControlError::LimitExceeded {
                    resource,
                    limit: u64::try_from(limit)
                        .map_err(|_| HtmlOptionValueControlError::RepresentationExhausted)?,
                    required: u64::try_from(value)
                        .map_err(|_| HtmlOptionValueControlError::RepresentationExhausted)?,
                });
            }
        }
        self.record_peak(required);
        Ok(())
    }
    fn record_peak(&self, required: HtmlOptionValueStorageUsage) {
        let old = self.peak.get();
        self.peak.set(HtmlOptionValueStorageUsage {
            bytes: old.bytes.max(required.bytes),
            units: old.units.max(required.units),
        });
    }
    fn observed(
        &self,
        extra: HtmlOptionValueStorageUsage,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.record_peak(self.required(extra)?);
        Ok(())
    }
    pub(crate) fn add(
        &self,
        value: HtmlOptionValueStorageUsage,
        retained: bool,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.admit(value)?;
        let cell = if retained {
            &self.live
        } else {
            &self.temporary
        };
        let old = cell.get();
        cell.set(HtmlOptionValueStorageUsage {
            bytes: old.bytes + value.bytes,
            units: old.units + value.units,
        });
        Ok(())
    }
    pub(crate) fn release(&self, value: HtmlOptionValueStorageUsage, retained: bool) {
        let cell = if retained {
            &self.live
        } else {
            &self.temporary
        };
        let old = cell.get();
        cell.set(HtmlOptionValueStorageUsage {
            bytes: old.bytes - value.bytes,
            units: old.units - value.units,
        });
    }
    fn begin(
        &self,
        envelope: HtmlOptionValueEnvelope,
        cancel: &AtomicBool,
    ) -> Result<(), HtmlOptionValueControlError> {
        self.start.set(self.work.get());
        self.envelope.set(envelope);
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
        self.control
            .release(Self::bank(old, size, resource)?, false);
        let previous = self.control.temporary.get();
        self.control.temporary.set(HtmlOptionValueStorageUsage {
            bytes: previous
                .bytes
                .checked_add(new.bytes)
                .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?,
            units: previous
                .units
                .checked_add(new.units)
                .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?,
        });
        if resource == HtmlSemanticResource::ScratchRecords {
            self.control.scratch_bytes.set(new.bytes);
        }
        outcome
    }
    pub(crate) fn release_scratch(self) {
        self.control.release(
            HtmlOptionValueStorageUsage {
                bytes: self.control.scratch_bytes.replace(0),
                units: 0,
            },
            false,
        );
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

enum Phase {
    Building(HtmlCaptureBuilder),
    Finished {
        capture: HtmlCapture,
        report: Option<HtmlSemanticReport>,
        failure: Option<HtmlControlledOptionValueError>,
    },
    Moving,
}

/// One controlled ordinary capture and one original OptionValue query.
/// Storage usage is cached and includes this owner's fixed inline header once.
/// Context/session borrow the capture locally; report identities do not own its graph.
pub struct HtmlControlledOptionValueCapture {
    phase: Phase,
    control: Control,
}

impl Control {
    pub(crate) fn action(
        &self,
        action: crate::capture::preflight::Action,
        cancel: &AtomicBool,
    ) -> Result<(), HtmlOptionValueControlError> {
        use crate::capture::preflight::Action;
        match action {
            Action::Work(work) => self.step(work, cancel),
            Action::Reserve { .. } => self.poll(cancel),
            Action::StorageBefore { requested } => {
                self.poll(cancel)?;
                self.admit(HtmlOptionValueStorageUsage {
                    bytes: requested,
                    units: 0,
                })
            }
            Action::StorageAfter {
                old,
                actual,
                retained,
            } => {
                // Requested backing was admitted before allocation. An allocator
                // may expose extra capacity; record it even on immediate refusal.
                let outcome = self.admit(HtmlOptionValueStorageUsage {
                    bytes: actual,
                    units: 0,
                });
                self.observed(HtmlOptionValueStorageUsage {
                    bytes: actual,
                    units: 0,
                })?;
                self.release(
                    HtmlOptionValueStorageUsage {
                        bytes: old,
                        units: 0,
                    },
                    retained,
                );
                let cell = if retained {
                    &self.live
                } else {
                    &self.temporary
                };
                let previous = cell.get();
                cell.set(HtmlOptionValueStorageUsage {
                    bytes: previous
                        .bytes
                        .checked_add(actual)
                        .ok_or(HtmlOptionValueControlError::RepresentationExhausted)?,
                    units: previous.units,
                });
                outcome
            }
            Action::StorageTransfer { bytes } => {
                self.transfer(HtmlOptionValueStorageUsage { bytes, units: 0 });
                Ok(())
            }
            Action::ElementValues {
                bytes,
                units,
                observed,
            } => {
                let payload = HtmlOptionValueStorageUsage { bytes, units };
                let result = self.poll(cancel).and_then(|()| self.admit(payload));
                if observed {
                    // An actual destination capacity remains peak evidence even
                    // when it immediately exceeds the prospective allowance.
                    self.observed(payload)?;
                }
                result
            }
        }
    }
    fn transfer(&self, value: HtmlOptionValueStorageUsage) {
        self.release(value, false);
        let previous = self.live.get();
        self.live.set(HtmlOptionValueStorageUsage {
            bytes: previous.bytes + value.bytes,
            units: previous.units + value.units,
        });
    }
    fn clear_temporaries(&self) {
        self.temporary.set(Default::default());
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
        let control = Control::new(envelope);
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
        self.control.live.get()
    }
    pub fn usage(&self) -> HtmlControlledOptionValueUsage {
        self.control.usage()
    }
    /// Borrows the actual successful retained report without repeating the query.
    /// This permits the caller to join usage before admitting its output copy.
    pub fn report(&self) -> Option<&HtmlSemanticReport> {
        match &self.phase {
            Phase::Finished { report, .. } => report.as_ref(),
            Phase::Building(_) | Phase::Moving => None,
        }
    }
    pub fn capture_usage(&self) -> HtmlCaptureUsage {
        match &self.phase {
            Phase::Building(builder) => builder.usage(),
            Phase::Finished { capture, .. } => capture.usage(),
            Phase::Moving => unreachable!(),
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
    fn capture_failure(
        &self,
        failure: crate::capture::preflight::Failure<HtmlOptionValueControlError>,
    ) -> HtmlControlledOptionValueFailure {
        use crate::capture::preflight::Failure;
        self.control.failure(match failure {
            Failure::Admission(error) => HtmlControlledOptionValueError::Control(error),
            Failure::Capture(error) => HtmlControlledOptionValueError::Capture(error),
            Failure::AllocationFailed => HtmlControlledOptionValueError::AllocationFailed,
        })
    }
    fn payload_failure(
        &self,
        error: HtmlCaptureStorageError<HtmlOptionValueControlError>,
    ) -> HtmlControlledOptionValueFailure {
        self.control
            .failure(HtmlControlledOptionValueError::Control(match error {
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
        self.control.add(payload, false).map_err(|e| {
            self.control
                .failure(HtmlControlledOptionValueError::Control(e))
        })?;
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
        let usage = crate::capture::storage::option_value_string(value, &mut || {
            self.control.step(1, cancel)
        })
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
        if result.is_ok() {
            self.control.transfer(payload);
        }
        self.control.clear_temporaries();
        result.map_err(|e| self.capture_failure(e))
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
            &mut |action| self.control.action(action, cancel),
        );
        self.finish_add(result, Default::default())
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
        let Phase::Building(builder) = &self.phase else {
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
                &mut |action| self.control.action(action, cancel),
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
        let payload = self.add_payload(formed.usage)?;
        let Phase::Building(builder) = &mut self.phase else {
            unreachable!()
        };
        let result = builder.add_element_controlled(
            document,
            formed.name,
            formed.attributes,
            saved_is.map_or(HtmlCapturedIsValue::Absent, |value| {
                HtmlCapturedIsValue::Value(value.clone())
            }),
            HtmlCaptureRegistryInput::Null,
            &mut |action| self.control.action(action, cancel),
        );
        self.finish_add(result, payload)
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
        let Phase::Building(builder) = &self.phase else {
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
        let payload = self.string(value, cancel)?;
        let kind = if text {
            HtmlCapturedNodeKind::Text(value.clone())
        } else {
            HtmlCapturedNodeKind::Comment(value.clone())
        };
        let Phase::Building(builder) = &mut self.phase else {
            unreachable!()
        };
        let result = builder.add_leaf_controlled(document, kind, value.len(), &mut |action| {
            self.control.action(action, cancel)
        });
        self.finish_add(result, payload)
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
        let Phase::Building(builder) = &self.phase else {
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
        let payload = self.strings(&[Some(target), Some(data)], cancel)?;
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
        let Phase::Building(builder) = &mut self.phase else {
            unreachable!()
        };
        let result = builder.add_leaf_controlled(document, kind, units, &mut |action| {
            self.control.action(action, cancel)
        });
        self.finish_add(result, payload)
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
                    self.control
                        .failure(HtmlControlledOptionValueError::Control(
                            HtmlOptionValueControlError::RepresentationExhausted,
                        ))
                })?;
                payload.units = payload.units.checked_add(next.units).ok_or_else(|| {
                    self.control
                        .failure(HtmlControlledOptionValueError::Control(
                            HtmlOptionValueControlError::RepresentationExhausted,
                        ))
                })?;
            }
            Ok(payload)
        })();
        if result.is_err() {
            self.control.clear_temporaries();
        }
        result
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
        let Phase::Building(builder) = &self.phase else {
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
        let payload = self.strings(&[Some(name), public_id, system_id], cancel)?;
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
        let Phase::Building(builder) = &mut self.phase else {
            unreachable!()
        };
        let result = builder.add_leaf_controlled(document, kind, units, &mut |action| {
            self.control.action(action, cancel)
        });
        self.finish_add(result, payload)
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
            self.control.action(action, cancel)
        });
        self.finish_add(result, Default::default())
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
        let Phase::Building(builder) = &self.phase else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
        let result = {
            let mut scratch = crate::capture::preflight::DepthScratch::new();
            builder.preflight_finish(&mut scratch, &mut |action| {
                self.control.action(action, cancel)
            })
        };
        self.control.clear_temporaries();
        result.map_err(|e| self.capture_failure(e))?;
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
        let Phase::Building(builder) = std::mem::replace(&mut self.phase, Phase::Moving) else {
            unreachable!()
        };
        self.phase = Phase::Finished {
            capture: builder.publish(),
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
        let Phase::Finished {
            capture,
            report: None,
            failure: None,
        } = &self.phase
        else {
            return Err(self
                .control
                .failure(HtmlControlledOptionValueError::InvalidPhase));
        };
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
            let mut session = HtmlSemanticSession::new(
                capture,
                &context,
                interpreter,
                limits,
                HtmlSemanticCancellation::new(),
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
            self.control.semantic.set(session.usage());
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
                let storage = match storage {
                    Ok(storage) => storage,
                    Err(error) => {
                        let error = HtmlControlledOptionValueError::Control(error);
                        let Phase::Finished { failure, .. } = &mut self.phase else {
                            unreachable!()
                        };
                        *failure = Some(error.clone());
                        self.control.clear_temporaries();
                        return Err(self.control.failure(error));
                    }
                };
                self.control.transfer(storage);
                self.control.clear_temporaries();
                let Phase::Finished { report: slot, .. } = &mut self.phase else {
                    unreachable!()
                };
                *slot = Some(report);
                let Phase::Finished {
                    report: Some(report),
                    ..
                } = &self.phase
                else {
                    unreachable!()
                };
                Ok(report)
            }
            Err(error) => {
                if let HtmlControlledOptionValueError::Semantic(failure) = &error
                    && failure.qualification().is_some()
                {
                    self.control.transfer(HtmlOptionValueStorageUsage {
                        bytes: size_of::<HtmlSemanticQualification>(),
                        units: 0,
                    });
                }
                let Phase::Finished { failure: slot, .. } = &mut self.phase else {
                    unreachable!()
                };
                *slot = Some(error.clone());
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
