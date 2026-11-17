//! A fixed chain of scopes kept in plain counters.
//!
//! [`Chain`] holds a root scope and a fixed number of scopes below it, one
//! under the other, and admits batches at the leaf by the rules of an
//! [`Operation`](crate::Operation) with the same scopes, caps and tariffs:
//!
//! - A batch is atomic and its items are checked in order, each from the leaf
//!   towards the root. A scope checks its own cap before the change is
//!   projected into its parent's units.
//! - The first refusal names the scope, the item, the change in that scope's
//!   units, the value the change was applied to, which includes the items of
//!   the batch before it, and the reason.
//! - A resource peak includes the usage an item reaches inside a batch.
//! - A refused batch and a checked batch leave usage and peaks as they were.
//!
//! Nothing observes a chain: it has no profile, no checkpoints, no phases and
//! no scope names, and it allocates nothing. The comparison tests in
//! `tests/chain.rs` run a chain and an operation on the same input.

use std::sync::atomic::Ordering;

use crate::{
    Change, Charge, Defect, Dimension, Error, Limits, NEXT_OPERATION, RefusalReason, Scope,
    Tariffs, Usage, increase, refused,
};

/// The number of values a batch can replace in place. A batch that replaces
/// more is admitted on a copy of the usage tables instead.
const JOURNAL: usize = 32;

/// One scope of a chain: its caps and what it holds.
#[derive(Clone, Copy)]
struct Level<W: Dimension, R: Dimension> {
    limits: Limits<W, R>,
    usage: Usage<W, R>,
}

impl<W: Dimension, R: Dimension> Level<W, R> {
    fn new(limits: Limits<W, R>) -> Self {
        Self {
            limits,
            usage: Usage::zero(),
        }
    }
}

/// A value of a scope as it was before a batch changed it. `depth` counts
/// scopes from the root, which is zero.
#[derive(Clone, Copy)]
enum Undo<W: Dimension, R: Dimension> {
    Unused,
    Work {
        depth: usize,
        dimension: W,
        total: u64,
    },
    Live {
        depth: usize,
        dimension: R,
        live: u64,
        peak: u64,
    },
}

/// A root scope and `BELOW` scopes below it, one under the other, kept in
/// plain counters. Batches are charged at the leaf, the scope furthest from
/// the root, which is the root itself when `BELOW` is zero.
///
/// A chain admits and refuses every batch as an [`Operation`](crate::Operation)
/// with the same scopes, caps and tariffs does when charged at its leaf, with
/// the same refusal figures and the same usage in every scope. It suits
/// accounting that nothing observes: it has no profile, checkpoints, phases or
/// scope names, and it allocates nothing at setup or admission.
///
/// ```
/// use surgeist_ledger::{Chain, Change, Limits, NoDimension, NoTable, Tariffs};
///
/// surgeist_ledger::dimensions! {
///     enum Work => WorkTable, WorkFields { Step }
/// }
///
/// // A scope without caps below a root that admits ten steps.
/// let mut chain = Chain::<Work, NoDimension, 1>::new(
///     Limits::new(WorkTable::new(WorkFields { Step: 10 }), NoTable::EMPTY),
///     [(Limits::unlimited(), Tariffs::identity())],
/// )?;
/// chain.apply(Change::Work(Work::Step, 7))?;
/// assert!(chain.apply(Change::Work(Work::Step, 4)).is_err());
/// assert_eq!(chain.usage(chain.root())?.work(Work::Step), 7);
/// # Ok::<(), surgeist_ledger::Error<Work, NoDimension>>(())
/// ```
pub struct Chain<W: Dimension, R: Dimension, const BELOW: usize> {
    identity: u64,
    root: Level<W, R>,
    /// The scopes below the root, from its child to the leaf, each with the
    /// tariffs into its parent.
    below: [(Level<W, R>, Tariffs<W, R>); BELOW],
    journal: [Undo<W, R>; JOURNAL],
}

impl<W: Dimension, R: Dimension, const BELOW: usize> Chain<W, R, BELOW> {
    /// Creates a chain from the caps of its root and, for each scope below it
    /// from the root's child to the leaf, that scope's caps and its tariffs
    /// into its parent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IdentityExhausted`] when no operation identity is left.
    pub fn new(
        root: Limits<W, R>,
        below: [(Limits<W, R>, Tariffs<W, R>); BELOW],
    ) -> Result<Self, Error<W, R>> {
        let identity = NEXT_OPERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::IdentityExhausted)?;
        Ok(Self {
            identity,
            root: Level::new(root),
            below: below.map(|(limits, tariffs)| (Level::new(limits), tariffs)),
            journal: [Undo::Unused; JOURNAL],
        })
    }

    /// The scope at the top of the chain.
    pub const fn root(&self) -> Scope {
        self.at(0)
    }

    /// The scope batches are charged at.
    pub const fn leaf(&self) -> Scope {
        self.at(BELOW)
    }

    /// The scope `depth` scopes below the root, or `None` past the leaf.
    pub const fn scope(&self, depth: usize) -> Option<Scope> {
        if depth <= BELOW {
            Some(self.at(depth))
        } else {
            None
        }
    }

    const fn at(&self, depth: usize) -> Scope {
        Scope {
            operation: self.identity,
            index: depth,
        }
    }

    /// What a scope of this chain holds, in its own units.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ForeignScope`] for a scope of another chain or of an
    /// operation.
    pub fn usage(&self, scope: Scope) -> Result<&Usage<W, R>, Error<W, R>> {
        Ok(&self.level(scope)?.usage)
    }

    /// The caps of a scope of this chain, in its own units.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ForeignScope`] for a scope of another chain or of an
    /// operation.
    pub fn limits(&self, scope: Scope) -> Result<&Limits<W, R>, Error<W, R>> {
        Ok(&self.level(scope)?.limits)
    }

    fn level(&self, scope: Scope) -> Result<&Level<W, R>, Error<W, R>> {
        if scope.operation != self.identity {
            return Err(Error::ForeignScope);
        }
        match scope.index.checked_sub(1) {
            None => Some(&self.root),
            Some(index) => self.below.get(index).map(|(level, _)| level),
        }
        .ok_or(Error::ForeignScope)
    }

    /// Admits one change at the leaf.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Refused`] with the first scope that refuses it.
    #[inline]
    pub fn apply(&mut self, change: Change<W, R>) -> Result<(), Error<W, R>> {
        self.step(0, change, None)
    }

    /// Admits a batch at the leaf, all or nothing, checking its items in order
    /// and each item from the leaf towards the root.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Refused`] with the first refused item. The chain is
    /// then as it was before the call.
    #[inline]
    pub fn apply_batch(&mut self, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        match *changes {
            [] => Ok(()),
            [change] => self.step(0, change, None),
            _ => self.run(changes, false),
        }
    }

    /// Whether [`Chain::apply_batch`] would admit the batch. Nothing changes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Refused`] with the first refused item.
    pub fn check_batch(&mut self, changes: &[Change<W, R>]) -> Result<(), Error<W, R>> {
        match *changes {
            [] => Ok(()),
            [change] => match change {
                Change::Work(dimension, amount) => self.work(0, dimension, amount).map(|_| ()),
                Change::Acquire(dimension, amount) => {
                    self.acquire(0, dimension, amount).map(|_| ())
                }
                Change::Release(dimension, amount) => {
                    self.release(0, dimension, amount).map(|_| ())
                }
            },
            _ => self.run(changes, true),
        }
    }

    /// Applies a batch of several changes in place and takes it back when an
    /// item is refused or when the batch is only checked.
    fn run(&mut self, changes: &[Change<W, R>], check_only: bool) -> Result<(), Error<W, R>> {
        let scopes = BELOW.saturating_add(1);
        if changes.len().saturating_mul(scopes) > JOURNAL {
            return self.run_on_copy(changes, check_only);
        }
        let mut recorded = 0_usize;
        let mut outcome = Ok(());
        for (item, &change) in changes.iter().enumerate() {
            outcome = self.step(item, change, Some(&mut recorded));
            if outcome.is_err() {
                break;
            }
        }
        if outcome.is_err() || check_only {
            self.roll_back(recorded);
        }
        outcome
    }

    /// Applies a batch that the journal cannot hold, keeping a copy of what
    /// every scope holds to restore.
    fn run_on_copy(
        &mut self,
        changes: &[Change<W, R>],
        check_only: bool,
    ) -> Result<(), Error<W, R>> {
        let root = self.root.usage;
        let below = self.below.each_ref().map(|(level, _)| level.usage);
        let mut outcome = Ok(());
        for (item, &change) in changes.iter().enumerate() {
            outcome = self.step(item, change, None);
            if outcome.is_err() {
                break;
            }
        }
        if outcome.is_err() || check_only {
            self.root.usage = root;
            for ((level, _), usage) in self.below.iter_mut().zip(below) {
                level.usage = usage;
            }
        }
        outcome
    }

    /// Restores the first `recorded` journal entries, newest first.
    fn roll_back(&mut self, recorded: usize) {
        let Self {
            root,
            below,
            journal,
            ..
        } = self;
        for &undo in journal.iter().take(recorded).rev() {
            let depth = match undo {
                Undo::Unused => continue,
                Undo::Work { depth, .. } | Undo::Live { depth, .. } => depth,
            };
            let level = match depth.checked_sub(1) {
                None => Some(&mut *root),
                Some(index) => below.get_mut(index).map(|(level, _)| level),
            };
            let Some(level) = level else {
                continue;
            };
            match undo {
                Undo::Unused => {}
                Undo::Work {
                    dimension, total, ..
                } => *W::get_mut(&mut level.usage.work, dimension) = total,
                Undo::Live {
                    dimension,
                    live,
                    peak,
                    ..
                } => {
                    *R::get_mut(&mut level.usage.live, dimension) = live;
                    *R::get_mut(&mut level.usage.peak, dimension) = peak;
                }
            }
        }
    }

    /// Checks one work change against every scope from the leaf to the root
    /// and returns, for each scope, the dimension it counts in there and the
    /// total it reaches. Nothing changes.
    #[inline]
    fn work(
        &self,
        item: usize,
        mut dimension: W,
        mut amount: u64,
    ) -> Result<Reached<W, BELOW>, Error<W, R>> {
        let mut below = [(dimension, 0_u64); BELOW];
        for ((index, (level, tariffs)), reached) in
            self.below.iter().enumerate().zip(&mut below).rev()
        {
            let held = level.usage.work(dimension);
            let total = match increase(held, amount, level.limits.work(dimension)) {
                Ok(total) => total,
                Err(reason) => {
                    let change = Change::Work(dimension, amount);
                    return Err(refused(self.below_root(index), item, change, held, reason));
                }
            };
            *reached = (dimension, total);
            let tariff = W::get(&tariffs.work, dimension);
            let Some(scaled) = amount.checked_mul(tariff.scale) else {
                let change = Change::Work(dimension, amount);
                let reason = RefusalReason::Overflow;
                return Err(refused(self.below_root(index), item, change, held, reason));
            };
            amount = scaled;
            dimension = tariff.target;
        }
        let held = self.root.usage.work(dimension);
        match increase(held, amount, self.root.limits.work(dimension)) {
            Ok(total) => Ok(Reached {
                below,
                root: (dimension, total),
            }),
            Err(reason) => {
                let change = Change::Work(dimension, amount);
                Err(refused(self.root(), item, change, held, reason))
            }
        }
    }

    /// As [`Chain::work`], for an acquisition and the live value it reaches.
    #[inline]
    fn acquire(
        &self,
        item: usize,
        mut dimension: R,
        mut amount: u64,
    ) -> Result<Reached<R, BELOW>, Error<W, R>> {
        let mut below = [(dimension, 0_u64); BELOW];
        for ((index, (level, tariffs)), reached) in
            self.below.iter().enumerate().zip(&mut below).rev()
        {
            let held = level.usage.live(dimension);
            let live = match increase(held, amount, level.limits.live(dimension)) {
                Ok(live) => live,
                Err(reason) => {
                    let change = Change::Acquire(dimension, amount);
                    return Err(refused(self.below_root(index), item, change, held, reason));
                }
            };
            *reached = (dimension, live);
            let tariff = R::get(&tariffs.resources, dimension);
            let Some(scaled) = amount.checked_mul(tariff.scale) else {
                let change = Change::Acquire(dimension, amount);
                let reason = RefusalReason::Overflow;
                return Err(refused(self.below_root(index), item, change, held, reason));
            };
            amount = scaled;
            dimension = tariff.target;
        }
        let held = self.root.usage.live(dimension);
        match increase(held, amount, self.root.limits.live(dimension)) {
            Ok(live) => Ok(Reached {
                below,
                root: (dimension, live),
            }),
            Err(reason) => {
                let change = Change::Acquire(dimension, amount);
                Err(refused(self.root(), item, change, held, reason))
            }
        }
    }

    /// As [`Chain::work`], for a release and the live value it leaves.
    #[inline]
    fn release(
        &self,
        item: usize,
        mut dimension: R,
        mut amount: u64,
    ) -> Result<Reached<R, BELOW>, Error<W, R>> {
        let mut below = [(dimension, 0_u64); BELOW];
        for ((index, (level, tariffs)), reached) in
            self.below.iter().enumerate().zip(&mut below).rev()
        {
            let held = level.usage.live(dimension);
            let Some(live) = held.checked_sub(amount) else {
                let change = Change::Release(dimension, amount);
                let reason = RefusalReason::Underflow;
                return Err(refused(self.below_root(index), item, change, held, reason));
            };
            *reached = (dimension, live);
            let tariff = R::get(&tariffs.resources, dimension);
            let Some(scaled) = amount.checked_mul(tariff.scale) else {
                let change = Change::Release(dimension, amount);
                let reason = RefusalReason::Overflow;
                return Err(refused(self.below_root(index), item, change, held, reason));
            };
            amount = scaled;
            dimension = tariff.target;
        }
        let held = self.root.usage.live(dimension);
        match held.checked_sub(amount) {
            Some(live) => Ok(Reached {
                below,
                root: (dimension, live),
            }),
            None => {
                let change = Change::Release(dimension, amount);
                let reason = RefusalReason::Underflow;
                Err(refused(self.root(), item, change, held, reason))
            }
        }
    }

    /// The scope at `index` of the scopes below the root.
    const fn below_root(&self, index: usize) -> Scope {
        self.at(index.saturating_add(1))
    }

    /// Checks one change against every scope and, when all accept it, writes
    /// the values it reaches. With `recorded` it journals each value it
    /// replaces and counts the entries.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Refused`] with nothing changed, or
    /// [`Defect::ReservedCapacity`] when the journal is full, which the
    /// caller's bound on the batch rules out; the values replaced so far are
    /// then in the journal.
    #[inline]
    fn step(
        &mut self,
        item: usize,
        change: Change<W, R>,
        mut recorded: Option<&mut usize>,
    ) -> Result<(), Error<W, R>> {
        match change {
            Change::Work(dimension, amount) => {
                let reached = self.work(item, dimension, amount)?;
                let Self {
                    root,
                    below,
                    journal,
                    ..
                } = self;
                let levels = below.iter_mut().map(|(level, _)| level);
                for ((index, level), (dimension, total)) in levels.enumerate().zip(reached.below) {
                    let held = W::get_mut(&mut level.usage.work, dimension);
                    if let Some(recorded) = recorded.as_deref_mut() {
                        let depth = index.saturating_add(1);
                        let undo = Undo::Work {
                            depth,
                            dimension,
                            total: *held,
                        };
                        record(journal, recorded, undo)?;
                    }
                    *held = total;
                }
                let (dimension, total) = reached.root;
                let held = W::get_mut(&mut root.usage.work, dimension);
                if let Some(recorded) = recorded {
                    let undo = Undo::Work {
                        depth: 0,
                        dimension,
                        total: *held,
                    };
                    record(journal, recorded, undo)?;
                }
                *held = total;
                Ok(())
            }
            Change::Acquire(dimension, amount) => {
                let reached = self.acquire(item, dimension, amount)?;
                self.write_live(reached, recorded, true)
            }
            Change::Release(dimension, amount) => {
                let reached = self.release(item, dimension, amount)?;
                self.write_live(reached, recorded, false)
            }
        }
    }

    /// Writes the live values a resource change reaches, raising the peaks for
    /// an acquisition.
    #[inline]
    fn write_live(
        &mut self,
        reached: Reached<R, BELOW>,
        mut recorded: Option<&mut usize>,
        raise_peak: bool,
    ) -> Result<(), Error<W, R>> {
        let Self {
            root,
            below,
            journal,
            ..
        } = self;
        let levels = below.iter_mut().map(|(level, _)| level);
        let depths = (1..).zip(levels).zip(reached.below);
        let scopes = depths.chain(std::iter::once(((0, root), reached.root)));
        for ((depth, level), (dimension, live)) in scopes {
            if let Some(recorded) = recorded.as_deref_mut() {
                let undo = Undo::Live {
                    depth,
                    dimension,
                    live: level.usage.live(dimension),
                    peak: level.usage.peak(dimension),
                };
                record(journal, recorded, undo)?;
            }
            *R::get_mut(&mut level.usage.live, dimension) = live;
            if raise_peak {
                let peak = R::get_mut(&mut level.usage.peak, dimension);
                *peak = (*peak).max(live);
            }
        }
        Ok(())
    }
}

/// For each scope of a chain, the dimension a change counts in there and the
/// value it reaches.
struct Reached<D, const BELOW: usize> {
    below: [(D, u64); BELOW],
    root: (D, u64),
}

/// Journals a value a batch is about to replace.
#[inline]
fn record<W: Dimension, R: Dimension>(
    journal: &mut [Undo<W, R>; JOURNAL],
    recorded: &mut usize,
    undo: Undo<W, R>,
) -> Result<(), Error<W, R>> {
    let slot = journal
        .get_mut(*recorded)
        .ok_or(Error::Defect(Defect::ReservedCapacity))?;
    *slot = undo;
    *recorded = recorded.saturating_add(1);
    Ok(())
}

impl<W: Dimension, R: Dimension, const BELOW: usize> Charge<W, R> for Chain<W, R, BELOW> {
    type Stop = Error<W, R>;

    fn charge(&mut self, batch: &[Change<W, R>]) -> Result<(), Self::Stop> {
        self.apply_batch(batch)
    }

    fn record_largest(&mut self, resource: R, value: u64) -> Result<(), Self::Stop> {
        let held = self.usage(self.leaf())?.live(resource);
        self.apply_batch(&[
            Change::Release(resource, held),
            Change::Acquire(resource, value),
        ])
    }
}
