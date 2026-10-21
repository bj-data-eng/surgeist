//! The actual destination entry owns one persistent finite consuming envelope.
use super::*;
use crate::parsed_association::DomParsedHtmlAssociationAdmissionError as Error;
use crate::{DomParsedHtmlAssociationLimits, DomParsedHtmlAssociationRuntimeUsage};
use std::cell::{Cell, RefCell};

#[derive(Default)]
pub(super) struct Control {
    bound: AtomicBool,
    active: AtomicBool,
    bytes_limit: AtomicUsize,
    units_limit: AtomicUsize,
    work_limit: AtomicU64,
    outputs_limit: AtomicUsize,
    work: AtomicU64,
    outputs: AtomicUsize,
    bytes: AtomicUsize,
    units: AtomicUsize,
    peak_bytes: AtomicUsize,
    peak_units: AtomicUsize,
    persistent_bytes: AtomicUsize,
    persistent_units: AtomicUsize,
    cleanup_atomic: AtomicUsize,
}
impl Control {
    pub(super) fn persistent_usage(&self) -> custody::Usage {
        custody::Usage::new(
            self.persistent_bytes.load(Ordering::Relaxed),
            self.persistent_units.load(Ordering::Relaxed),
        )
    }
    pub(super) fn reserve_cleanup(&self, bytes: usize) {
        self.cleanup_atomic.fetch_max(bytes, Ordering::Relaxed);
    }
    pub(super) fn add_persistent(&self, usage: custody::Usage) {
        self.persistent_bytes
            .fetch_add(usage.bytes(), Ordering::Relaxed);
        self.persistent_units
            .fetch_add(usage.units(), Ordering::Relaxed);
    }
    fn spend(&self, count: u64, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let limit = self.work_limit.load(Ordering::Relaxed);
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(count)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > limit {
            return Err(DomError::WorkLimit {
                resource: DomResource::ParsedAssociationProcessingWork,
                limit,
            });
        }
        self.work.store(next, Ordering::Relaxed);
        Ok(())
    }
    pub(super) fn bind(
        &self,
        id: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
    ) -> Result<(), Error> {
        if self.active.load(Ordering::Relaxed) {
            return Err(DomError::ParsedAssociationOperationPending { pending: id }.into());
        }
        if self.bound.load(Ordering::Relaxed)
            && (limits.retained_bytes() < self.bytes_limit.load(Ordering::Relaxed)
                || limits.retained_units() < self.units_limit.load(Ordering::Relaxed)
                || limits.work() < self.work_limit.load(Ordering::Relaxed)
                || limits.outputs() < self.outputs_limit.load(Ordering::Relaxed))
        {
            return Err(Error::LimitReduction { contribution: id });
        }
        self.bytes_limit
            .store(limits.retained_bytes(), Ordering::Relaxed);
        self.units_limit
            .store(limits.retained_units(), Ordering::Relaxed);
        self.work_limit.store(limits.work(), Ordering::Relaxed);
        self.outputs_limit
            .store(limits.outputs(), Ordering::Relaxed);
        self.bound.store(true, Ordering::Relaxed);
        Ok(())
    }
    pub(super) fn enter(&self) {
        self.active.store(true, Ordering::Relaxed);
    }
    pub(super) fn leave(&self) {
        self.active.store(false, Ordering::Relaxed);
    }
    pub(super) fn spent_work(&self) -> u64 {
        self.work.load(Ordering::Relaxed)
    }
    pub(super) fn limits(&self) -> DomParsedHtmlAssociationLimits {
        DomParsedHtmlAssociationLimits::new(
            self.bytes_limit.load(Ordering::Relaxed),
            self.units_limit.load(Ordering::Relaxed),
            self.work_limit.load(Ordering::Relaxed),
            self.outputs_limit.load(Ordering::Relaxed),
        )
        .expect("original validated finite policy")
    }
    fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let limit = self.work_limit.load(Ordering::Relaxed);
        if next > limit {
            return Err(DomError::WorkLimit {
                resource: DomResource::ParsedAssociationProcessingWork,
                limit,
            });
        }
        self.work.store(next, Ordering::Relaxed);
        Ok(())
    }
    fn check(&self, usage: custody::Usage) -> Result<(), DomError> {
        let bytes = self
            .bytes
            .load(Ordering::Relaxed)
            .checked_add(usage.bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        let units = self
            .units
            .load(Ordering::Relaxed)
            .checked_add(usage.units())
            .ok_or(DomError::RepresentationExhausted)?;
        let peak = bytes
            .checked_add(self.cleanup_atomic.load(Ordering::Relaxed))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            peak,
            self.bytes_limit.load(Ordering::Relaxed),
            DomResource::ParsedAssociationProcessingRetainedBytes,
        )?;
        admit(
            units,
            self.units_limit.load(Ordering::Relaxed),
            DomResource::ParsedAssociationProcessingRetainedUnits,
        )?;
        self.peak_bytes.fetch_max(peak, Ordering::Relaxed);
        self.peak_units.fetch_max(units, Ordering::Relaxed);
        Ok(())
    }
    fn add(&self, usage: custody::Usage) {
        self.bytes.fetch_add(usage.bytes(), Ordering::Relaxed);
        self.units.fetch_add(usage.units(), Ordering::Relaxed);
    }
    fn release(&self, usage: custody::Usage) {
        self.bytes.fetch_sub(usage.bytes(), Ordering::Relaxed);
        self.units.fetch_sub(usage.units(), Ordering::Relaxed);
    }
    fn outputs(&self, count: usize) -> Result<(), DomError> {
        let next = self
            .outputs
            .load(Ordering::Relaxed)
            .checked_add(count)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            next,
            self.outputs_limit.load(Ordering::Relaxed),
            DomResource::ParsedAssociationProcessingOutputs,
        )?;
        self.outputs.store(next, Ordering::Relaxed);
        Ok(())
    }
}

/// Retains the actual original record and its custody, never a replacement issuer.
/// Owning graphs must be declared/destroyed before this receipt.
pub(in crate::store) struct Meter {
    entry: bank::ControlledRecord<Entry>,
    ledger: custody::Ledger,
    debit: RefCell<Option<custody::Debit>>,
    own: Cell<custody::Usage>,
    controlled: RefCell<Option<Admission>>,
    runtime: Cell<DomParsedHtmlAssociationRuntimeUsage>,
    pending_limit: Option<(custody::Usage, DomLimits)>,
}
impl Drop for Meter {
    fn drop(&mut self) {
        self.entry.value().control.release(self.own.get());
    }
}
impl Meter {
    pub(super) fn spend(&self, count: u64, cancel: &AtomicBool) -> Result<(), DomError> {
        self.entry().control.spend(count, cancel)
    }
    pub(super) fn check_peak(&self, usage: custody::Usage) -> Result<(), DomError> {
        self.entry().control.check(usage)
    }
    /// The already-admitted graph moves into the original Entry, never a Meter
    /// or ControlledRecord backreference. Its control extent stays charged.
    pub(super) fn detach(&self, usage: custody::Usage) -> custody::Debit {
        let left = self
            .own
            .get()
            .checked_sub(usage)
            .expect("preadmitted persistent subset");
        let debit = self
            .debit
            .borrow_mut()
            .as_mut()
            .expect("admitted graph")
            .split(usage)
            .expect("preadmitted persistent subset");
        self.own.set(left);
        debit
    }
    /// Transfer already charged persistent backing into an actual retirement
    /// owner. No new retention or fallible work occurs at publication.
    pub(super) fn retire_persistent(&self, debit: custody::Debit, usage: custody::Usage) {
        self.entry()
            .control
            .persistent_bytes
            .fetch_sub(usage.bytes(), Ordering::Relaxed);
        self.entry()
            .control
            .persistent_units
            .fetch_sub(usage.units(), Ordering::Relaxed);
        let mut own = self.debit.borrow_mut();
        if let Some(own) = own.as_mut() {
            own.absorb(debit).expect("same original custody ledger");
        } else {
            *own = Some(debit);
        }
        self.own.set(
            self.own
                .get()
                .checked_add(usage)
                .expect("preadmitted retirement union"),
        );
    }
    pub(super) fn shrink_to(&self, usage: custody::Usage) {
        self.release(
            self.own
                .get()
                .checked_sub(usage)
                .expect("surviving preadmitted output subset"),
        );
    }
    pub(super) fn new(entry: bank::ControlledRecord<Entry>, ledger: custody::Ledger) -> Self {
        Self {
            entry,
            ledger,
            debit: RefCell::new(None),
            own: Cell::new(custody::Usage::ZERO),
            controlled: RefCell::new(None),
            pending_limit: None,
            runtime: Cell::new(DomParsedHtmlAssociationRuntimeUsage {
                bytes: 0,
                units: 0,
                outputs: 0,
            }),
        }
    }
    pub(super) fn entry(&self) -> &Entry {
        self.entry.value()
    }
    pub(super) fn fork(&self) -> Self {
        let mut next = Self::new(self.entry.clone(), self.ledger.clone());
        next.pending_limit = self.pending_limit;
        next
    }
    pub(super) fn constrain_pending(mut self, baseline: custody::Usage, limits: DomLimits) -> Self {
        self.pending_limit = Some((baseline, limits));
        self
    }

    pub(super) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.entry().control.step(cancel)
    }
    pub(super) fn retain(
        &self,
        bytes: usize,
        units: usize,
        outputs: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let usage = custody::Usage::new(bytes, units);
        let next = self.own.get().checked_add(usage)?;
        self.entry().control.check(usage)?;
        if let Some((baseline, limits)) = self.pending_limit {
            let control = &self.entry().control;
            let live = custody::Usage::new(
                control.bytes.load(Ordering::Relaxed),
                control.units.load(Ordering::Relaxed),
            );
            let extra = live
                .checked_add(usage)?
                .checked_sub(control.persistent_usage())?;
            Bank::admit_usage(baseline.checked_add(extra)?, limits)?;
        }
        self.entry().control.outputs(outputs)?;
        let debit = self.ledger.retain(usage, |_| Ok(()))?;
        let mut own = self.debit.borrow_mut();
        if let Some(own) = own.as_mut() {
            own.absorb(debit)
                .expect("one original pending custody ledger");
        } else {
            *own = Some(debit);
        }
        self.entry().control.add(usage);
        self.own.set(next);
        Ok(())
    }
    fn release(&self, usage: custody::Usage) {
        if usage == custody::Usage::ZERO {
            return;
        }
        drop(
            self.debit
                .borrow_mut()
                .as_mut()
                .expect("owned consuming payload")
                .split(usage)
                .expect("owned consuming subset"),
        );
        self.entry().control.release(usage);
        self.own.set(
            self.own
                .get()
                .checked_sub(usage)
                .expect("owned consuming subset"),
        );
    }
    pub(super) fn runtime(
        &self,
        usage: DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = self.runtime.get();
        let next = DomParsedHtmlAssociationRuntimeUsage::new(
            old.bytes().max(usage.bytes()),
            old.units().max(usage.units()),
            old.outputs().max(usage.outputs()),
        )?;
        self.retain(
            next.bytes() - old.bytes(),
            next.units() - old.units(),
            next.outputs() - old.outputs(),
            cancel,
        )?;
        self.runtime.set(next);
        Ok(())
    }
    pub(super) fn controlled(
        &self,
        total: custody::Usage,
        baseline: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let fresh = self.controlled.borrow().is_none();
        let admission = self.admission(baseline, cancel)?;
        // The producer supplied its prospective ledger total before the lazy
        // receipt existed. Add that new real debit exactly once to this first
        // report, matching the receipt's equally adjusted baseline.
        let total = if fresh {
            total.checked_add(custody::Usage::new(size_of::<Charged>(), 0))?
        } else {
            total
        };
        admission.controlled(total, cancel)
    }
    pub(in crate::store) fn admission(
        &self,
        baseline: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<Admission, DomError> {
        if let Some(value) = self.controlled.borrow().as_ref() {
            return Ok(value.clone());
        }
        let usage = custody::Usage::new(size_of::<Charged>(), 0);
        let baseline = baseline.checked_add(usage)?;
        self.retain(usage.bytes(), 0, 0, cancel)?;
        let debit = self.detach(usage);
        let result = Admission(Arc::new(Charged {
            entry: self.entry.clone(),
            baseline,
            bytes: AtomicUsize::new(0),
            units: AtomicUsize::new(0),
            metadata: usage,
            _debit: debit,
        }));
        *self.controlled.borrow_mut() = Some(result.clone());
        Ok(result)
    }
    fn absorb(&self, other: Self) -> Result<(), DomError> {
        let usage = other.own.replace(custody::Usage::ZERO);
        self.own.set(self.own.get().checked_add(usage)?);
        if let Some(incoming) = other.debit.borrow_mut().take() {
            let mut debit = self.debit.borrow_mut();
            if let Some(debit) = debit.as_mut() {
                debit
                    .absorb(incoming)
                    .expect("one original pending custody ledger");
            } else {
                *debit = Some(incoming);
            }
        }
        Ok(())
    }
    pub(super) fn reserve<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.reserve_with(values, additional, false, cancel)
    }
    pub(super) fn reserve_units(
        &self,
        values: &mut Vec<u16>,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.reserve_with(values, additional, true, cancel)
    }
    fn reserve_with<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
        utf16: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        let required = values
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if required <= values.capacity() {
            return Ok(());
        }
        let count = required.max(
            values
                .capacity()
                .checked_mul(2)
                .ok_or(DomError::RepresentationExhausted)?,
        );
        let bytes = count
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        let replacement_meter = self.fork();
        replacement_meter.retain(bytes, if utf16 { count } else { 0 }, 0, cancel)?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(count)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = replacement
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        replacement_meter.retain(
            actual
                .checked_sub(bytes)
                .ok_or(DomError::RepresentationExhausted)?,
            if utf16 {
                replacement.capacity() - count
            } else {
                0
            },
            0,
            cancel,
        )?;
        for _ in values.iter() {
            self.step(cancel)?;
        }
        let old = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        let old_units = if utf16 { values.capacity() } else { 0 };
        replacement.append(values);
        drop(std::mem::replace(values, replacement));
        self.release(custody::Usage::new(old, old_units));
        self.absorb(replacement_meter)
    }
}

/// One charged positive-delta receipt, shared by the actual private Draft and
/// its Formation forks. It retains original authority, never a second counter
/// or an Entry-owned backreference. Owners dispose graphs before this receipt.
#[derive(Clone)]
pub(in crate::store) struct Admission(Arc<Charged>);
struct Charged {
    entry: bank::ControlledRecord<Entry>,
    baseline: custody::Usage,
    bytes: AtomicUsize,
    units: AtomicUsize,
    metadata: custody::Usage,
    _debit: custody::Debit,
}
impl Drop for Charged {
    fn drop(&mut self) {
        self.entry.value().control.release(custody::Usage::new(
            self.bytes.load(Ordering::Relaxed),
            self.units.load(Ordering::Relaxed),
        ));
        self.entry.value().control.release(self.metadata);
    }
}
impl Admission {
    pub(in crate::store) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.0.entry.value().control.step(cancel)
    }
    pub(in crate::store) fn controlled(
        &self,
        total: custody::Usage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.step(cancel)?;
        self.controlled_unpolled(total)
    }
    pub(in crate::store) fn controlled_unpolled(
        &self,
        total: custody::Usage,
    ) -> Result<(), DomError> {
        let old = custody::Usage::new(
            self.0.bytes.load(Ordering::Relaxed),
            self.0.units.load(Ordering::Relaxed),
        );
        let next = custody::Usage::new(
            old.bytes()
                .max(total.bytes().saturating_sub(self.0.baseline.bytes())),
            old.units()
                .max(total.units().saturating_sub(self.0.baseline.units())),
        );
        let growth = next.checked_sub(old)?;
        self.0.entry.value().control.check(growth)?;
        self.0.entry.value().control.add(growth);
        self.0.bytes.store(next.bytes(), Ordering::Relaxed);
        self.0.units.store(next.units(), Ordering::Relaxed);
        Ok(())
    }
    pub(in crate::store) fn spent_work(&self) -> u64 {
        self.0.entry.value().control.spent_work()
    }
    pub(in crate::store) fn remaining_bytes(&self) -> Result<usize, DomError> {
        let c = &self.0.entry.value().control;
        c.limits()
            .retained_bytes()
            .checked_sub(c.bytes.load(Ordering::Relaxed))
            .and_then(|n| n.checked_sub(c.cleanup_atomic.load(Ordering::Relaxed)))
            .ok_or(DomError::ResourceLimit {
                resource: DomResource::ParsedAssociationProcessingRetainedBytes,
                limit: c.limits().retained_bytes(),
            })
    }
    pub(in crate::store) fn limits(&self) -> DomParsedHtmlAssociationLimits {
        self.0.entry.value().control.limits()
    }
}

#[cfg(test)]
mod tests;
