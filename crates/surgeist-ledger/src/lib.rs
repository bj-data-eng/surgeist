#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use std::{
    fmt,
    ops::{AsyncFnOnce, Deref},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

mod checkpoint;
mod phase_observation;
mod shared;
pub use checkpoint::{Checkpoint, IntervalReport, IntervalScopeReport, IntervalUsage};
pub use phase_observation::PhaseObservation;
pub use shared::SharedOperation;

mod profile;
use profile::Profile;
pub use profile::{Event, EventFilter, ProfileMode, Report, ScopeReport, TerminalReport};

/// A work dimension in an operation's schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Work<const N: usize>(usize);

impl<const N: usize> Work<N> {
    /// Selects an index below the schema's work dimension count.
    pub const fn new(index: usize) -> Option<Self> {
        if index < N { Some(Self(index)) } else { None }
    }
}

/// A live resource dimension in an operation's schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resource<const N: usize>(usize);

impl<const N: usize> Resource<N> {
    /// Selects an index below the schema's resource dimension count.
    pub const fn new(index: usize) -> Option<Self> {
        if index < N { Some(Self(index)) } else { None }
    }
}

/// Per-dimension caps in the scope's own units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits<const W: usize, const R: usize> {
    work: [u64; W],
    live: [u64; R],
}

impl<const W: usize, const R: usize> Limits<W, R> {
    pub const fn new(work: [u64; W], live: [u64; R]) -> Self {
        Self { work, live }
    }

    /// Removes configured caps. Arithmetic remains checked.
    pub const fn unlimited() -> Self {
        Self::new([u64::MAX; W], [u64::MAX; R])
    }
}

/// Maps one dimension to an ancestor dimension with an integral scale.
/// A fixed scale makes arbitrary partial releases additive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tariff<D> {
    target: D,
    scale: u64,
}

impl<D> Tariff<D> {
    pub const fn new(target: D, scale: u64) -> Self {
        Self { target, scale }
    }
}

/// One mapping per local dimension, indexed in schema order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tariffs<const W: usize, const R: usize> {
    work: [Tariff<Work<W>>; W],
    resources: [Tariff<Resource<R>>; R],
}

impl<const W: usize, const R: usize> Tariffs<W, R> {
    pub const fn new(work: [Tariff<Work<W>>; W], resources: [Tariff<Resource<R>>; R]) -> Self {
        Self { work, resources }
    }

    /// Explicitly keeps the same dimensions and units in the parent.
    pub fn identity() -> Self {
        Self::new(
            std::array::from_fn(|i| Tariff::new(Work(i), 1)),
            std::array::from_fn(|i| Tariff::new(Resource(i), 1)),
        )
    }

    fn project(&self, change: Change<W, R>) -> Option<Change<W, R>> {
        Some(match change {
            Change::Work(dimension, amount) => {
                let tariff = self.work[dimension.0];
                Change::Work(tariff.target, amount.checked_mul(tariff.scale)?)
            }
            Change::Acquire(dimension, amount) => {
                let tariff = self.resources[dimension.0];
                Change::Acquire(tariff.target, amount.checked_mul(tariff.scale)?)
            }
            Change::Release(dimension, amount) => {
                let tariff = self.resources[dimension.0];
                Change::Release(tariff.target, amount.checked_mul(tariff.scale)?)
            }
        })
    }
}

/// One ordered ledger change, expressed in the originating scope's units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Change<const W: usize, const R: usize> {
    Work(Work<W>, u64),
    Acquire(Resource<R>, u64),
    Release(Resource<R>, u64),
}

/// An operation-owned scope. Its identity stays valid for the operation's lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Scope {
    operation: u64,
    index: usize,
}

/// Inclusive accepted usage in one scope's own units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage<const W: usize, const R: usize> {
    work: [u64; W],
    live: [u64; R],
    peak: [u64; R],
}

impl<const W: usize, const R: usize> Usage<W, R> {
    const ZERO: Self = Self {
        work: [0; W],
        live: [0; R],
        peak: [0; R],
    };

    pub const fn work(&self, dimension: Work<W>) -> u64 {
        self.work[dimension.0]
    }

    pub const fn live(&self, dimension: Resource<R>) -> u64 {
        self.live[dimension.0]
    }

    pub const fn peak(&self, dimension: Resource<R>) -> u64 {
        self.peak[dimension.0]
    }

    fn current(&self, change: Change<W, R>) -> u64 {
        match change {
            Change::Work(dimension, _) => self.work(dimension),
            Change::Acquire(dimension, _) | Change::Release(dimension, _) => self.live(dimension),
        }
    }

    fn stage(&mut self, limits: &Limits<W, R>, change: Change<W, R>) -> Result<(), RefusalReason> {
        match change {
            Change::Work(dimension, amount) => {
                let current = &mut self.work[dimension.0];
                *current = increase(*current, amount, limits.work[dimension.0])?;
            }
            Change::Acquire(dimension, amount) => {
                let current = &mut self.live[dimension.0];
                *current = increase(*current, amount, limits.live[dimension.0])?;
                let peak = &mut self.peak[dimension.0];
                *peak = (*peak).max(*current);
            }
            Change::Release(dimension, amount) => {
                let current = &mut self.live[dimension.0];
                *current = current
                    .checked_sub(amount)
                    .ok_or(RefusalReason::Underflow)?;
            }
        }
        Ok(())
    }
}

fn increase(current: u64, amount: u64, limit: u64) -> Result<u64, RefusalReason> {
    let next = current.checked_add(amount).ok_or(RefusalReason::Overflow)?;
    if next > limit {
        return Err(RefusalReason::Limit { limit });
    }
    Ok(next)
}

/// Why an ordered change was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalReason {
    Overflow,
    Underflow,
    Limit { limit: u64 },
}

/// The first refused item. Current usage includes the provisional prefix of the
/// batch and, for a custody underflow, describes only the origin's direct hold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Refusal<const W: usize, const R: usize> {
    pub scope: Scope,
    pub item: usize,
    pub change: Change<W, R>,
    pub current: u64,
    pub reason: RefusalReason,
}

/// A ledger admission or scope management failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<const W: usize, const R: usize> {
    Allocation,
    IdentityExhausted,
    ForeignScope,
    ForeignCheckpoint,
    StaleCheckpoint,
    CheckpointLimit { limit: usize },
    CheckpointsConfigured,
    ScopeLimit { limit: usize },
    TimingUnavailable,
    Refused(Refusal<W, R>),
}

impl<const W: usize, const R: usize> fmt::Display for Error<W, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allocation => f.write_str("could not allocate ledger storage"),
            Self::IdentityExhausted => f.write_str("ledger identities exhausted"),
            Self::ForeignCheckpoint => f.write_str("checkpoint belongs to another operation"),
            Self::StaleCheckpoint => f.write_str("checkpoint is no longer active"),
            Self::CheckpointLimit { limit } => {
                write!(f, "ledger checkpoint limit of {limit} reached")
            }
            Self::CheckpointsConfigured => f.write_str("checkpoint storage is already configured"),
            Self::ForeignScope => f.write_str("scope belongs to another operation"),
            Self::ScopeLimit { limit } => write!(f, "ledger scope limit of {limit} reached"),
            Self::TimingUnavailable => f.write_str("timed profiling requires a host clock"),
            Self::Refused(refusal) => write!(
                f,
                "ledger change {} refused: {:?}",
                refusal.item, refusal.reason
            ),
        }
    }
}

impl<const W: usize, const R: usize> std::error::Error for Error<W, R> {}

struct Parent<const W: usize, const R: usize> {
    index: usize,
    tariffs: Tariffs<W, R>,
}

struct ScopeEntry<const W: usize, const R: usize> {
    name: &'static str,
    parent: Option<Parent<W, R>>,
    limits: Limits<W, R>,
    usage: Usage<W, R>,
    staged: Usage<W, R>,
    batch_peak: [u64; R],
    direct: [u64; R],
}

impl<const W: usize, const R: usize> ScopeEntry<W, R> {
    const fn new(name: &'static str, parent: Option<Parent<W, R>>, limits: Limits<W, R>) -> Self {
        Self {
            name,
            parent,
            limits,
            usage: Usage::ZERO,
            staged: Usage::ZERO,
            batch_peak: [0; R],
            direct: [0; R],
        }
    }

    fn parent_index(&self) -> Option<usize> {
        self.parent.as_ref().map(|parent| parent.index)
    }
}

static NEXT_OPERATION: AtomicU64 = AtomicU64::new(0);

/// The ledger and scope tree for one operation.
///
/// Scopes are retained until this ledger is dropped. Only the originating scope
/// can release its direct custody, even when that scope has children. Consumers
/// own cancellation and terminal state and can issue cleanup after refusal.
pub struct Operation<const W: usize, const R: usize> {
    identity: u64,
    max_scopes: usize,
    scopes: Vec<ScopeEntry<W, R>>,
    // At most one observer. Disabled operations keep its storage unallocated,
    // while enabled setup reserves it fallibly alongside the report buffers.
    profile: Vec<Profile<W, R>>,
    checkpoints: checkpoint::Checkpoints<W, R>,
}

impl<const W: usize, const R: usize> Operation<W, R> {
    /// Creates an operation. The scope bound includes the root.
    pub fn new(
        name: &'static str,
        limits: Limits<W, R>,
        max_scopes: usize,
    ) -> Result<Self, Error<W, R>> {
        if max_scopes == 0 {
            return Err(Error::ScopeLimit { limit: 0 });
        }
        let mut scopes = Vec::new();
        scopes.try_reserve(1).map_err(|_| Error::Allocation)?;
        let identity = NEXT_OPERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::IdentityExhausted)?;
        scopes.push(ScopeEntry::new(name, None, limits));
        Ok(Self {
            identity,
            max_scopes,
            scopes,
            profile: Vec::new(),
            checkpoints: checkpoint::Checkpoints::default(),
        })
    }

    /// Creates an operation with optional profiling. Observer storage is
    /// reserved during setup and does not participate in later admissions.
    pub fn with_profile(
        name: &'static str,
        limits: Limits<W, R>,
        max_scopes: usize,
        mode: ProfileMode,
    ) -> Result<Self, Error<W, R>> {
        Self::profiled(name, limits, max_scopes, mode, None)
    }

    /// Uses a host-supplied monotonic clock for timed modes. The clock reports
    /// a duration from an arbitrary fixed epoch and must return without panic
    /// or observable side effects. Untimed modes never call it.
    pub fn with_profile_clock(
        name: &'static str,
        limits: Limits<W, R>,
        max_scopes: usize,
        mode: ProfileMode,
        clock: fn() -> Duration,
    ) -> Result<Self, Error<W, R>> {
        Self::profiled(name, limits, max_scopes, mode, Some(clock))
    }

    fn profiled(
        name: &'static str,
        limits: Limits<W, R>,
        max_scopes: usize,
        mode: ProfileMode,
        clock: Option<fn() -> Duration>,
    ) -> Result<Self, Error<W, R>> {
        let mut operation = Self::new(name, limits, max_scopes)?;
        let root = operation.root();
        if let Some(mut profile) = Profile::new(mode, max_scopes, clock)? {
            operation
                .profile
                .try_reserve_exact(1)
                .map_err(|_| Error::Allocation)?;
            profile.add_scope(root, None, name);
            operation.profile.push(profile);
        }
        Ok(operation)
    }

    pub const fn root(&self) -> Scope {
        self.scope(0)
    }

    /// Opens a child with its own caps and explicit projection into its parent.
    pub fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>> {
        let parent_index = self.index(parent)?;
        let index = self.scopes.len();
        if index == self.max_scopes {
            return Err(Error::ScopeLimit {
                limit: self.max_scopes,
            });
        }
        self.scopes.try_reserve(1).map_err(|_| Error::Allocation)?;
        self.scopes.push(ScopeEntry::new(
            name,
            Some(Parent {
                index: parent_index,
                tariffs,
            }),
            limits,
        ));
        let scope = self.scope(index);
        if let Some(profile) = self.profile.first_mut() {
            profile.add_scope(scope, Some(parent), name);
        }
        self.checkpoints.add_scope(scope, Some(parent), name);
        Ok(scope)
    }

    pub fn usage(&self, scope: Scope) -> Result<&Usage<W, R>, Error<W, R>> {
        Ok(&self.scopes[self.index(scope)?].usage)
    }

    pub fn scope_name(&self, scope: Scope) -> Result<&'static str, Error<W, R>> {
        Ok(self.scopes[self.index(scope)?].name)
    }

    /// Observes an inclusive phase at a scope while running the caller's
    /// operation. A phase ends when the callback returns or unwinds.
    pub fn with_phase<T>(
        &mut self,
        scope: Scope,
        run: impl FnOnce(&mut Phase<'_, W, R>) -> T,
    ) -> Result<T, Error<W, R>> {
        let mut phase = self.enter_phase(scope)?;
        Ok(run(&mut phase))
    }

    /// Observes a phase while polling and awaiting the supplied asynchronous work.
    ///
    /// The phase starts when this future is first polled, not when it is created.
    /// Completion, unwinding, or dropping a pending future ends the phase and
    /// records its elapsed time. Admissions made before cancellation remain in
    /// the operation. Dropping an unpolled future records no phase.
    ///
    /// The callback borrows a [`Phase`], allowing nested admissions and phases
    /// without moving the owning operation. The caller owns operation status,
    /// cleanup and report custody after the returned future is dropped.
    pub async fn with_async_phase<T>(
        &mut self,
        scope: Scope,
        run: impl AsyncFnOnce(&mut Phase<'_, W, R>) -> T,
    ) -> Result<T, Error<W, R>> {
        let mut phase = self.enter_phase(scope)?;
        Ok(run(&mut phase).await)
    }

    fn enter_phase(&mut self, scope: Scope) -> Result<Phase<'_, W, R>, Error<W, R>> {
        self.index(scope)?;
        Ok(Phase {
            start: self.profile.first().and_then(Profile::elapsed),
            scope,
            operation: self,
        })
    }

    /// Copies the current structured profile without consuming the operation.
    ///
    /// Returns `None` when profiling is off. The copy has independent, fallibly
    /// allocated storage bounded by the already retained scope/event counts.
    /// Current usage, lifetime peaks and accepted observations are unchanged.
    /// Elapsed operation time is observed at the snapshot; phase summaries include
    /// only phases that have ended. This is not an interval-peak calculation.
    pub fn snapshot(&self) -> Result<Option<Report<W, R>>, Error<W, R>> {
        self.profile
            .first()
            .map(|profile| profile.snapshot(self.scopes.iter().map(|entry| entry.usage)))
            .transpose()
    }

    /// Consumes the ledger and returns its structured report if enabled.
    /// Consumers release live resources before finishing when cleanup is part
    /// of the operation; outstanding balances remain visible in the report.
    pub fn finish(mut self) -> Option<Report<W, R>> {
        self.profile
            .pop()
            .map(|profile| profile.finish(self.scopes.into_iter().map(|entry| entry.usage)))
    }

    /// Checks the same ordered admission rules without committing usage, peaks
    /// or observations. This reserves nothing: a later apply must check again,
    /// and an intervening shared admission may invalidate a successful check.
    pub fn check_batch(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
    ) -> Result<(), Error<W, R>> {
        let origin = self.index(scope)?;
        if changes.is_empty() {
            return Ok(());
        }
        self.stage_batch(scope, origin, changes).map(|_| ())
    }

    pub fn apply(&mut self, scope: Scope, change: Change<W, R>) -> Result<(), Error<W, R>> {
        self.apply_batch(scope, &[change])
    }

    /// Admits a batch atomically, checking items in input order and each item
    /// from the origin towards the root. A scope is checked before applying its
    /// tariff to reach the next scope. Intermediate accepted live usage counts
    /// towards peak, including acquire/release pairs for temporary resources.
    ///
    /// Any refusal discards the entire batch. Earlier batches are retained.
    /// Callers can issue a separate release after refusal or cancellation.
    pub fn apply_batch(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
    ) -> Result<(), Error<W, R>> {
        let origin = self.index(scope)?;
        if changes.is_empty() {
            return Ok(());
        }
        let result = self.admit_batch(scope, origin, changes);
        if let Some(profile) = self.profile.first_mut() {
            profile.admission(scope, changes, result);
        }
        self.checkpoints.admission(scope, changes, result);
        result
    }

    fn admit_batch(
        &mut self,
        scope: Scope,
        origin: usize,
        changes: &[Change<W, R>],
    ) -> Result<(), Error<W, R>> {
        let direct = self.stage_batch(scope, origin, changes)?;
        let mut cursor = Some(origin);
        while let Some(index) = cursor {
            let entry = &mut self.scopes[index];
            entry.usage = entry.staged;
            self.checkpoints.commit_peak(index, entry.batch_peak);
            cursor = entry.parent_index();
        }
        self.scopes[origin].direct = direct;
        Ok(())
    }

    fn stage_batch(
        &mut self,
        scope: Scope,
        origin: usize,
        changes: &[Change<W, R>],
    ) -> Result<[u64; R], Error<W, R>> {
        let mut cursor = Some(origin);
        while let Some(index) = cursor {
            let entry = &mut self.scopes[index];
            entry.staged = entry.usage;
            entry.batch_peak = entry.usage.live;
            cursor = entry.parent_index();
        }

        let mut direct = self.scopes[origin].direct;
        for (item, &change) in changes.iter().enumerate() {
            if let Change::Release(dimension, amount) = change {
                let current = &mut direct[dimension.0];
                *current = current.checked_sub(amount).ok_or(Error::Refused(Refusal {
                    scope,
                    item,
                    change,
                    current: *current,
                    reason: RefusalReason::Underflow,
                }))?;
            }

            let mut index = origin;
            let mut projected = change;
            loop {
                let entry = &mut self.scopes[index];
                let current = entry.staged.current(projected);
                let refusal = |reason| {
                    Error::Refused(Refusal {
                        scope: Scope {
                            operation: scope.operation,
                            index,
                        },
                        item,
                        change: projected,
                        current,
                        reason,
                    })
                };
                entry
                    .staged
                    .stage(&entry.limits, projected)
                    .map_err(refusal)?;
                if let Change::Acquire(dimension, _) = projected {
                    entry.batch_peak[dimension.0] =
                        entry.batch_peak[dimension.0].max(entry.staged.live[dimension.0]);
                }
                let Some(parent) = entry.parent.as_ref() else {
                    break;
                };
                projected = parent
                    .tariffs
                    .project(projected)
                    .ok_or_else(|| refusal(RefusalReason::Overflow))?;
                index = parent.index;
            }

            if let Change::Acquire(dimension, amount) = change {
                // Direct holdings are a subset of the admitted inclusive total.
                direct[dimension.0] += amount;
            }
        }

        Ok(direct)
    }

    fn record_phase(&mut self, scope: Scope, start: Option<Duration>) {
        let end = self.profile.first().and_then(Profile::elapsed);
        if let Some(profile) = self.profile.first_mut() {
            profile.end_phase(scope, start, end);
        }
        self.checkpoints.end_phase(scope, start, end);
    }

    const fn scope(&self, index: usize) -> Scope {
        Scope {
            operation: self.identity,
            index,
        }
    }

    fn index(&self, scope: Scope) -> Result<usize, Error<W, R>> {
        if scope.operation == self.identity {
            Ok(scope.index)
        } else {
            Err(Error::ForeignScope)
        }
    }
}

/// Borrowed access to an operation within a profiled phase. The owning ledger
/// remains in place through phase completion.
///
/// ```compile_fail,E0596
/// use surgeist_ledger::{Limits, Operation};
///
/// let mut operation = Operation::<0, 0>::new("original", Limits::unlimited(), 1).unwrap();
/// let replacement = Operation::<0, 0>::new("replacement", Limits::unlimited(), 1).unwrap();
/// operation.with_phase(operation.root(), |phase| {
///     std::mem::replace(&mut **phase, replacement);
/// }).unwrap();
/// ```
pub struct Phase<'a, const W: usize, const R: usize> {
    start: Option<Duration>,
    scope: Scope,
    operation: &'a mut Operation<W, R>,
}

impl<const W: usize, const R: usize> Phase<'_, W, R> {
    pub fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>> {
        self.operation.child(parent, name, limits, tariffs)
    }

    pub fn with_phase<T>(
        &mut self,
        scope: Scope,
        run: impl FnOnce(&mut Phase<'_, W, R>) -> T,
    ) -> Result<T, Error<W, R>> {
        self.operation.with_phase(scope, run)
    }

    /// Observes nested asynchronous work within this phase.
    pub async fn with_async_phase<T>(
        &mut self,
        scope: Scope,
        run: impl AsyncFnOnce(&mut Phase<'_, W, R>) -> T,
    ) -> Result<T, Error<W, R>> {
        self.operation.with_async_phase(scope, run).await
    }

    pub fn check_batch(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
    ) -> Result<(), Error<W, R>> {
        self.operation.check_batch(scope, changes)
    }

    /// Opens an interval without moving the operation out of its phase.
    pub fn begin_checkpoint(&mut self) -> Result<Checkpoint, Error<W, R>> {
        self.operation.begin_checkpoint()
    }

    pub fn end_checkpoint(
        &mut self,
        checkpoint: &Checkpoint,
    ) -> Result<IntervalReport<W, R>, Error<W, R>> {
        self.operation.end_checkpoint(checkpoint)
    }

    pub fn apply(&mut self, scope: Scope, change: Change<W, R>) -> Result<(), Error<W, R>> {
        self.operation.apply(scope, change)
    }

    pub fn apply_batch(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
    ) -> Result<(), Error<W, R>> {
        self.operation.apply_batch(scope, changes)
    }
}

impl<const W: usize, const R: usize> Deref for Phase<'_, W, R> {
    type Target = Operation<W, R>;

    fn deref(&self) -> &Self::Target {
        self.operation
    }
}

impl<const W: usize, const R: usize> Drop for Phase<'_, W, R> {
    fn drop(&mut self) {
        self.operation.record_phase(self.scope, self.start);
    }
}
