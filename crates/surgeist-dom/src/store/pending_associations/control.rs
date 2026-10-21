//! The actual destination entry owns one persistent finite consuming envelope.
use super::*;
use crate::parsed_meta::DomParsedHtmlMetaAdmissionError as Error;
use crate::{DomMetaAtomicLimits, DomMetaRuntimeUsage};
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
}
impl Control {
    pub(super) fn bind(
        &self,
        id: DomParsedHtmlAssociationId,
        limits: DomMetaAtomicLimits,
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
    pub(super) fn limits(&self) -> DomMetaAtomicLimits {
        DomMetaAtomicLimits::new(
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
            return Err(DomError::Meta(crate::DomMetaError::WorkLimit { limit }));
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
        admit(
            bytes,
            self.bytes_limit.load(Ordering::Relaxed),
            DomResource::MetaAtomicRetainedBytes,
        )?;
        admit(
            units,
            self.units_limit.load(Ordering::Relaxed),
            DomResource::MetaAtomicRetainedUnits,
        )?;
        self.peak_bytes.fetch_max(bytes, Ordering::Relaxed);
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
            DomResource::MetaAtomicOutputs,
        )?;
        self.outputs.store(next, Ordering::Relaxed);
        Ok(())
    }
}

/// Retains the actual original record and its custody, never a replacement issuer.
/// Owning graphs must be declared/destroyed before this receipt.
pub(super) struct Meter {
    entry: bank::ControlledRecord<Entry>,
    ledger: custody::Ledger,
    debit: RefCell<Option<custody::Debit>>,
    own: Cell<custody::Usage>,
    controlled: Cell<custody::Usage>,
    runtime: Cell<DomMetaRuntimeUsage>,
}
impl Drop for Meter {
    fn drop(&mut self) {
        self.entry.value().control.release(self.own.get());
        self.entry.value().control.release(self.controlled.get());
    }
}
impl Meter {
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
            controlled: Cell::new(custody::Usage::ZERO),
            runtime: Cell::new(DomMetaRuntimeUsage {
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
        Self::new(self.entry.clone(), self.ledger.clone())
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
        usage: DomMetaRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = self.runtime.get();
        let next = DomMetaRuntimeUsage::new(
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
        self.step(cancel)?;
        let old = self.controlled.get();
        let next = custody::Usage::new(
            old.bytes()
                .max(total.bytes().saturating_sub(baseline.bytes())),
            old.units()
                .max(total.units().saturating_sub(baseline.units())),
        );
        let growth = next.checked_sub(old)?;
        self.entry().control.check(growth)?;
        self.entry().control.add(growth);
        self.controlled.set(next);
        Ok(())
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
