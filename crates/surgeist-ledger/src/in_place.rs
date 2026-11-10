//! Admission that changes the scopes in place and journals what it touches.
//!
//! The dense admission in this crate stages a full copy of the usage of every
//! scope on the chain, so its cost grows with the width of the dimension sets.
//! A batch of a few changes touches a few values. This store changes those values
//! directly and records each old value in a small array on the stack, and a
//! refused batch is rolled back from it. The two paths apply the same rules:
//! items in input order, each from the origin towards the root, each scope
//! checking its own cap before its tariff projects the change into its parent.
//!
//! The rules live in two places: here and in `Admission` in `lib.rs`. A change to
//! an admission rule changes both, and the comparison test in `tests.rs` runs the
//! two on the same random input.

use std::sync::OnceLock;

use crate::{
    Change, Defect, Dimension, Error, RefusalReason, Scope, ScopeEntry, increase, missing_scope,
    refused,
};

/// Which store a test forces.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Force {
    /// The rule of [`preferred`].
    Auto,
    /// The dense admission for every batch.
    Dense,
    /// The in-place store whenever the journal has room.
    InPlace,
}

#[cfg(test)]
thread_local! {
    /// Makes this thread's batches use one store, so a test can compare the
    /// two stores on the same input.
    pub(crate) static FORCE: std::cell::Cell<Force> = const { std::cell::Cell::new(Force::Auto) };
}

/// Words of usage a dimension pair holds: one per work dimension, and a live
/// value and a peak per resource dimension.
const fn words<W: Dimension, R: Dimension>() -> usize {
    W::ALL.len().saturating_add(R::ALL.len().saturating_mul(2))
}

/// Whether a dimension pair is wide enough for the in-place store to pay.
/// Staging a dense copy costs in proportion to the width of the dimension
/// sets, so a set of a few dimensions is cheaper to copy than to journal.
/// The result is a constant for each pair, which lets the compiler drop the
/// in-place store from a narrow pair's admission entirely.
pub(crate) const fn wide<W: Dimension, R: Dimension>() -> bool {
    words::<W, R>() >= 8
}

/// Whether a batch, admitted at a scope `chain` scopes from the root, goes
/// through the in-place store. It must pay and it must fit.
///
/// The store pays from about eight words of usage per change. A change journals
/// one value in each scope of the chain, and an acquisition or release one more
/// for the origin's direct custody. A batch the journal cannot hold would be
/// rolled back and staged densely, which costs more than staging it densely at
/// once.
pub(crate) fn preferred<W: Dimension, R: Dimension>(
    changes: &[Change<W, R>],
    chain: usize,
) -> bool {
    #[cfg(test)]
    match FORCE.get() {
        Force::Dense => return false,
        Force::InPlace => return true,
        Force::Auto => {}
    }
    if words::<W, R>() < changes.len().saturating_mul(8) {
        return false;
    }
    let mut needed = 0_usize;
    for change in changes {
        let own = match change {
            Change::Work(..) => 0,
            Change::Acquire(..) | Change::Release(..) => 1,
        };
        needed = needed.saturating_add(chain).saturating_add(own);
    }
    needed <= JOURNAL
}

/// The number of values the journal restores.
const JOURNAL: usize = 24;

/// The journal of a one-change batch on a chain of up to three scopes.
const SHORT_JOURNAL: usize = 4;

/// The journal of a batch of up to four changes on a chain of up to two scopes.
const MID_JOURNAL: usize = 12;

/// What a journal entry restores.
#[derive(Clone, Copy)]
enum Key<W: Dimension, R: Dimension> {
    Work(usize, W),
    Live(usize, R),
    Direct(R),
}

/// A value as it was before the batch first changed it. `peak` is used by live
/// entries only.
#[derive(Clone, Copy)]
struct Saved<W: Dimension, R: Dimension> {
    key: Key<W, R>,
    value: u64,
    peak: u64,
}

/// Why admission stopped.
enum Stop<W: Dimension, R: Dimension> {
    /// The batch is refused or the ledger found a defect.
    Error(Error<W, R>),
    /// The journal has no room; the dense admission handles the batch.
    Full,
}

/// One batch changing the scopes in place.
struct Batch<'a, W: Dimension, R: Dimension, const J: usize> {
    scopes: &'a mut [ScopeEntry<W, R>],
    defect: &'a OnceLock<Defect>,
    scope: Scope,
    origin: usize,
    journal: [Option<Saved<W, R>>; J],
    used: usize,
}

impl<'a, W: Dimension, R: Dimension, const J: usize> Batch<'a, W, R, J> {
    fn new(
        scopes: &'a mut [ScopeEntry<W, R>],
        defect: &'a OnceLock<Defect>,
        scope: Scope,
        origin: usize,
    ) -> Self {
        Self {
            scopes,
            defect,
            scope,
            origin,
            journal: [None; J],
            used: 0,
        }
    }

    /// Records the old value of `key` before the batch changes it.
    fn save(&mut self, key: Key<W, R>, value: u64, peak: u64) -> Result<(), Stop<W, R>> {
        let slot = self.journal.get_mut(self.used).ok_or(Stop::Full)?;
        *slot = Some(Saved { key, value, peak });
        self.used = self.used.saturating_add(1);
        Ok(())
    }

    /// Restores every journaled value, newest first.
    fn rollback(&mut self) {
        let Some(saved) = self.journal.get(..self.used) else {
            return;
        };
        for saved in saved.iter().rev().flatten() {
            match saved.key {
                Key::Work(index, dimension) => {
                    if let Some(entry) = self.scopes.get_mut(index) {
                        *W::get_mut(&mut entry.usage.work, dimension) = saved.value;
                    }
                }
                Key::Live(index, dimension) => {
                    if let Some(entry) = self.scopes.get_mut(index) {
                        *R::get_mut(&mut entry.usage.live, dimension) = saved.value;
                        *R::get_mut(&mut entry.usage.peak, dimension) = saved.peak;
                    }
                }
                Key::Direct(dimension) => {
                    if let Some(entry) = self.scopes.get_mut(self.origin) {
                        *R::get_mut(&mut entry.direct, dimension) = saved.value;
                    }
                }
            }
        }
        self.used = 0;
    }

    fn run(&mut self, changes: &[Change<W, R>]) -> Result<(), Stop<W, R>> {
        for (item, &change) in changes.iter().enumerate() {
            match change {
                Change::Work(dimension, amount) => {
                    self.work(item, dimension, amount)?;
                }
                Change::Acquire(dimension, amount) => {
                    self.acquire(item, dimension, amount)?;
                    // Direct holdings are a subset of the admitted inclusive total.
                    let Some(entry) = self.scopes.get(self.origin) else {
                        return Err(Stop::Error(missing_scope(self.defect)));
                    };
                    let held = *R::get(&entry.direct, dimension);
                    let Some(total) = held.checked_add(amount) else {
                        return Err(Stop::Error(Error::Defect(crate::latch_defect(
                            self.defect,
                            Defect::DirectCustody,
                        ))));
                    };
                    self.save(Key::Direct(dimension), held, 0)?;
                    if let Some(entry) = self.scopes.get_mut(self.origin) {
                        *R::get_mut(&mut entry.direct, dimension) = total;
                    }
                }
                Change::Release(dimension, amount) => {
                    let Some(entry) = self.scopes.get(self.origin) else {
                        return Err(Stop::Error(missing_scope(self.defect)));
                    };
                    let held = *R::get(&entry.direct, dimension);
                    let Some(remaining) = held.checked_sub(amount) else {
                        return Err(Stop::Error(refused(
                            self.scope,
                            item,
                            change,
                            held,
                            RefusalReason::Underflow,
                        )));
                    };
                    self.save(Key::Direct(dimension), held, 0)?;
                    if let Some(entry) = self.scopes.get_mut(self.origin) {
                        *R::get_mut(&mut entry.direct, dimension) = remaining;
                    }
                    self.release(item, dimension, amount)?;
                }
            }
        }
        Ok(())
    }

    /// Applies one work change from the origin to the root. Each scope checks
    /// its own limit before the change is projected into its parent's units.
    /// A hop saves a value before it changes it, so a full journal stops the
    /// batch with every earlier change restorable.
    fn work(&mut self, item: usize, mut dimension: W, mut amount: u64) -> Result<(), Stop<W, R>> {
        let Self {
            scopes,
            defect,
            scope,
            origin,
            journal,
            used,
        } = self;
        let mut index = *origin;
        loop {
            let at = Scope {
                operation: scope.operation,
                index,
            };
            let change = Change::Work(dimension, amount);
            let slot = journal.get_mut(*used).ok_or(Stop::Full)?;
            let Some(entry) = scopes.get_mut(index) else {
                return Err(Stop::Error(missing_scope(defect)));
            };
            let current = W::get_mut(&mut entry.usage.work, dimension);
            let held = *current;
            let next = increase(held, amount, entry.limits.work(dimension))
                .map_err(|reason| Stop::Error(refused(at, item, change, held, reason)))?;
            *slot = Some(Saved {
                key: Key::Work(index, dimension),
                value: held,
                peak: 0,
            });
            *used = used.saturating_add(1);
            *current = next;
            let Some(parent) = &entry.parent else {
                return Ok(());
            };
            let tariff = *W::get(&parent.tariffs.work, dimension);
            amount = amount.checked_mul(tariff.scale).ok_or_else(|| {
                Stop::Error(refused(at, item, change, held, RefusalReason::Overflow))
            })?;
            dimension = tariff.target;
            index = parent.index;
        }
    }

    /// Applies one acquisition from the origin to the root, raising each
    /// scope's lifetime peak.
    fn acquire(
        &mut self,
        item: usize,
        mut dimension: R,
        mut amount: u64,
    ) -> Result<(), Stop<W, R>> {
        let Self {
            scopes,
            defect,
            scope,
            origin,
            journal,
            used,
        } = self;
        let mut index = *origin;
        loop {
            let at = Scope {
                operation: scope.operation,
                index,
            };
            let change = Change::Acquire(dimension, amount);
            let slot = journal.get_mut(*used).ok_or(Stop::Full)?;
            let Some(entry) = scopes.get_mut(index) else {
                return Err(Stop::Error(missing_scope(defect)));
            };
            let usage = &mut entry.usage;
            let current = R::get_mut(&mut usage.live, dimension);
            let held = *current;
            let live = increase(held, amount, entry.limits.live(dimension))
                .map_err(|reason| Stop::Error(refused(at, item, change, held, reason)))?;
            let peak = R::get_mut(&mut usage.peak, dimension);
            *slot = Some(Saved {
                key: Key::Live(index, dimension),
                value: held,
                peak: *peak,
            });
            *used = used.saturating_add(1);
            *current = live;
            *peak = (*peak).max(live);
            let Some(parent) = &entry.parent else {
                return Ok(());
            };
            let tariff = *R::get(&parent.tariffs.resources, dimension);
            amount = amount.checked_mul(tariff.scale).ok_or_else(|| {
                Stop::Error(refused(at, item, change, held, RefusalReason::Overflow))
            })?;
            dimension = tariff.target;
            index = parent.index;
        }
    }

    /// Applies one release from the origin to the root.
    fn release(
        &mut self,
        item: usize,
        mut dimension: R,
        mut amount: u64,
    ) -> Result<(), Stop<W, R>> {
        let Self {
            scopes,
            defect,
            scope,
            origin,
            journal,
            used,
        } = self;
        let mut index = *origin;
        loop {
            let at = Scope {
                operation: scope.operation,
                index,
            };
            let change = Change::Release(dimension, amount);
            let slot = journal.get_mut(*used).ok_or(Stop::Full)?;
            let Some(entry) = scopes.get_mut(index) else {
                return Err(Stop::Error(missing_scope(defect)));
            };
            let usage = &mut entry.usage;
            let current = R::get_mut(&mut usage.live, dimension);
            let held = *current;
            let remaining = held.checked_sub(amount).ok_or_else(|| {
                Stop::Error(refused(at, item, change, held, RefusalReason::Underflow))
            })?;
            let peak = *R::get(&usage.peak, dimension);
            *slot = Some(Saved {
                key: Key::Live(index, dimension),
                value: held,
                peak,
            });
            *used = used.saturating_add(1);
            *current = remaining;
            let Some(parent) = &entry.parent else {
                return Ok(());
            };
            let tariff = *R::get(&parent.tariffs.resources, dimension);
            amount = amount.checked_mul(tariff.scale).ok_or_else(|| {
                Stop::Error(refused(at, item, change, held, RefusalReason::Overflow))
            })?;
            dimension = tariff.target;
            index = parent.index;
        }
    }
}

/// Admits a batch in place. Returns `false`, with nothing changed, when the
/// batch touches more values than the journal holds. A batch of one change
/// uses a journal sized for a short chain, so it does not initialize the
/// full journal.
#[inline(never)]
pub(crate) fn admit<W: Dimension, R: Dimension>(
    scopes: &mut [ScopeEntry<W, R>],
    defect: &OnceLock<Defect>,
    scope: Scope,
    origin: usize,
    changes: &[Change<W, R>],
) -> Result<bool, Error<W, R>> {
    if changes.len() == 1 {
        match admit_with::<W, R, SHORT_JOURNAL>(scopes, defect, scope, origin, changes) {
            Ok(false) => {}
            other => return other,
        }
    }
    if changes.len() <= 4 {
        match admit_with::<W, R, MID_JOURNAL>(scopes, defect, scope, origin, changes) {
            Ok(false) => {}
            other => return other,
        }
    }
    admit_with::<W, R, JOURNAL>(scopes, defect, scope, origin, changes)
}

#[inline(always)]
fn admit_with<W: Dimension, R: Dimension, const J: usize>(
    scopes: &mut [ScopeEntry<W, R>],
    defect: &OnceLock<Defect>,
    scope: Scope,
    origin: usize,
    changes: &[Change<W, R>],
) -> Result<bool, Error<W, R>> {
    let mut batch = Batch::<W, R, J>::new(scopes, defect, scope, origin);
    match batch.run(changes) {
        Ok(()) => Ok(true),
        Err(Stop::Error(error)) => {
            batch.rollback();
            Err(error)
        }
        Err(Stop::Full) => {
            batch.rollback();
            Ok(false)
        }
    }
}

/// Checks a batch without committing it. Returns `None`, with nothing changed,
/// when the batch touches more values than the journal holds.
#[inline(never)]
pub(crate) fn check<W: Dimension, R: Dimension>(
    scopes: &mut [ScopeEntry<W, R>],
    defect: &OnceLock<Defect>,
    scope: Scope,
    origin: usize,
    changes: &[Change<W, R>],
) -> Option<Result<(), Error<W, R>>> {
    if changes.len() == 1
        && let Some(done) =
            check_with::<W, R, SHORT_JOURNAL>(scopes, defect, scope, origin, changes)
    {
        return Some(done);
    }
    if changes.len() <= 4
        && let Some(done) = check_with::<W, R, MID_JOURNAL>(scopes, defect, scope, origin, changes)
    {
        return Some(done);
    }
    check_with::<W, R, JOURNAL>(scopes, defect, scope, origin, changes)
}

#[inline(always)]
fn check_with<W: Dimension, R: Dimension, const J: usize>(
    scopes: &mut [ScopeEntry<W, R>],
    defect: &OnceLock<Defect>,
    scope: Scope,
    origin: usize,
    changes: &[Change<W, R>],
) -> Option<Result<(), Error<W, R>>> {
    let mut batch = Batch::<W, R, J>::new(scopes, defect, scope, origin);
    let outcome = batch.run(changes);
    batch.rollback();
    match outcome {
        Ok(()) => Some(Ok(())),
        Err(Stop::Error(error)) => Some(Err(error)),
        Err(Stop::Full) => None,
    }
}

/// Admits one work change without a journal: a first pass checks every hop from
/// the origin to the root and a second pass commits them. Returns `None`, with
/// nothing changed, when the chain is longer than the hops it holds.
#[inline(never)]
pub(crate) fn admit_one_work<W: Dimension, R: Dimension>(
    scopes: &mut [ScopeEntry<W, R>],
    defect: &OnceLock<Defect>,
    scope: Scope,
    origin: usize,
    dimension: W,
    amount: u64,
) -> Option<Result<(), Error<W, R>>> {
    const HOPS: usize = 4;
    let mut hops: [(usize, W, u64); HOPS] = [(origin, dimension, amount); HOPS];
    let mut count = 0_usize;
    let mut index = origin;
    let mut dimension = dimension;
    let mut amount = amount;
    loop {
        let at = Scope {
            operation: scope.operation,
            index,
        };
        let change = Change::Work(dimension, amount);
        let slot = hops.get_mut(count)?;
        let Some(entry) = scopes.get(index) else {
            return Some(Err(missing_scope(defect)));
        };
        let held = *W::get(&entry.usage.work, dimension);
        let next = match increase(held, amount, entry.limits.work(dimension)) {
            Ok(next) => next,
            Err(reason) => return Some(Err(refused(at, 0, change, held, reason))),
        };
        *slot = (index, dimension, next);
        count += 1;
        let Some(parent) = &entry.parent else {
            break;
        };
        let tariff = *W::get(&parent.tariffs.work, dimension);
        let Some(scaled) = amount.checked_mul(tariff.scale) else {
            return Some(Err(refused(at, 0, change, held, RefusalReason::Overflow)));
        };
        amount = scaled;
        dimension = tariff.target;
        index = parent.index;
    }
    for &(index, dimension, next) in hops.iter().take(count) {
        if let Some(entry) = scopes.get_mut(index) {
            *W::get_mut(&mut entry.usage.work, dimension) = next;
        }
    }
    Some(Ok(()))
}
