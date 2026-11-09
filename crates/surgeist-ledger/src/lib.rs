#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::arithmetic_side_effects,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

use std::{
    fmt,
    ops::{AsyncFnOnce, Deref},
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

mod charge;
mod checkpoint;
mod dimension;
mod in_place;
mod phase_observation;
mod scope_ledger;
mod shared;
#[cfg(test)]
mod tests;
pub use charge::{Charge, ScopeCharge, SharedScopeCharge};
pub use checkpoint::{Checkpoint, IntervalReport, IntervalScopeReport, IntervalUsage};
pub use dimension::{Dimension, NoDimension, NoTable, TableValue};
pub use phase_observation::PhaseObservation;
pub use scope_ledger::ScopeLedger;
pub use shared::SharedOperation;

mod profile;
use profile::Profile;
pub use profile::{Event, EventFilter, ProfileMode, Report, ScopeReport, TerminalReport};

/// Items the [`dimensions!`] expansion names. Not part of the supported API.
#[doc(hidden)]
pub mod __private {
    /// Marks a dimension set declared through [`dimensions!`](crate::dimensions).
    pub trait Sealed {}
}

/// Per-dimension caps in the scope's own units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits<W: Dimension, R: Dimension> {
    work: W::Table<u64>,
    live: R::Table<u64>,
}

impl<W: Dimension, R: Dimension> Limits<W, R> {
    /// Caps each work dimension's cumulative usage and each resource
    /// dimension's live usage.
    pub const fn new(work: W::Table<u64>, live: R::Table<u64>) -> Self {
        Self { work, live }
    }

    /// Builds caps by calling each function once per dimension.
    pub fn from_fn(work: impl FnMut(W) -> u64, live: impl FnMut(R) -> u64) -> Self {
        Self::new(W::from_fn(work), R::from_fn(live))
    }

    /// Removes configured caps. Arithmetic remains checked.
    pub fn unlimited() -> Self {
        Self::from_fn(|_| u64::MAX, |_| u64::MAX)
    }

    /// The cap on a work dimension's cumulative usage.
    pub fn work(&self, dimension: W) -> u64 {
        *W::get(&self.work, dimension)
    }

    /// The cap on a resource dimension's live usage.
    pub fn live(&self, dimension: R) -> u64 {
        *R::get(&self.live, dimension)
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

/// One mapping per local dimension into the parent's dimensions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tariffs<W: Dimension, R: Dimension> {
    work: W::Table<Tariff<W>>,
    resources: R::Table<Tariff<R>>,
}

impl<W: Dimension, R: Dimension> Tariffs<W, R> {
    pub const fn new(work: W::Table<Tariff<W>>, resources: R::Table<Tariff<R>>) -> Self {
        Self { work, resources }
    }

    /// Builds mappings by calling each function once per local dimension.
    pub fn from_fn(
        work: impl FnMut(W) -> Tariff<W>,
        resources: impl FnMut(R) -> Tariff<R>,
    ) -> Self {
        Self::new(W::from_fn(work), R::from_fn(resources))
    }

    /// Explicitly keeps the same dimensions and units in the parent.
    pub fn identity() -> Self {
        Self::from_fn(
            |dimension| Tariff::new(dimension, 1),
            |dimension| Tariff::new(dimension, 1),
        )
    }
}

/// One ordered ledger change, expressed in the originating scope's units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Change<W: Dimension, R: Dimension> {
    Work(W, u64),
    Acquire(R, u64),
    Release(R, u64),
}

/// An operation-owned scope. Its identity stays valid for the operation's lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Scope {
    operation: u64,
    index: usize,
}

/// Inclusive accepted usage in one scope's own units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage<W: Dimension, R: Dimension> {
    work: W::Table<u64>,
    live: R::Table<u64>,
    peak: R::Table<u64>,
}

impl<W: Dimension, R: Dimension> Usage<W, R> {
    fn zero() -> Self {
        Self {
            work: W::from_fn(|_| 0),
            live: R::from_fn(|_| 0),
            peak: R::from_fn(|_| 0),
        }
    }

    pub fn work(&self, dimension: W) -> u64 {
        *W::get(&self.work, dimension)
    }

    pub fn live(&self, dimension: R) -> u64 {
        *R::get(&self.live, dimension)
    }

    pub fn peak(&self, dimension: R) -> u64 {
        *R::get(&self.peak, dimension)
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
pub struct Refusal<W: Dimension, R: Dimension> {
    pub scope: Scope,
    pub item: usize,
    pub change: Change<W, R>,
    pub current: u64,
    pub reason: RefusalReason,
}

/// A ledger state that the ledger's own sequencing rules out. No valid use
/// produces one. Each variant names the invariant that was found broken.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Defect {
    /// A scope identity of this operation names no scope entry.
    ScopeEntry,
    /// A profile or interval report has no record for an existing scope, or
    /// holds a different number of records than the operation has scopes.
    ReportScope,
    /// A buffer reserved at setup was full where the operation's bounds leave room.
    ReservedCapacity,
    /// A scope's direct custody exceeded the custody its admissions accepted.
    DirectCustody,
    /// Cumulative accepted work fell below its value when an interval opened.
    IntervalWork,
    /// Retained detail events exceeded the configured event limit.
    EventLimit,
    /// A checkpoint slot found free was missing when the interval was stored.
    CheckpointSlot,
}

impl fmt::Display for Defect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ScopeEntry => "scope identity names no scope entry",
            Self::ReportScope => "report records do not match the operation's scopes",
            Self::ReservedCapacity => "reserved buffer full within its bound",
            Self::DirectCustody => "direct custody exceeds accepted custody",
            Self::IntervalWork => "cumulative work fell below its interval opening",
            Self::EventLimit => "retained events exceed the event limit",
            Self::CheckpointSlot => "free checkpoint slot missing",
        })
    }
}

/// A ledger admission or scope management failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<W: Dimension, R: Dimension> {
    Allocation,
    IdentityExhausted,
    ForeignScope,
    ForeignCheckpoint,
    StaleCheckpoint,
    CheckpointLimit {
        limit: usize,
    },
    CheckpointsConfigured,
    ScopeLimit {
        limit: usize,
    },
    TimingUnavailable,
    Refused(Refusal<W, R>),
    /// The ledger found a broken internal invariant. The operation stays in
    /// this state: every later fallible call returns the same defect.
    Defect(Defect),
}

impl<W: Dimension, R: Dimension> fmt::Display for Error<W, R> {
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
            Self::Defect(defect) => write!(f, "ledger defect: {defect}"),
        }
    }
}

impl<W: Dimension, R: Dimension> std::error::Error for Error<W, R> {}

/// Pushes into a buffer whose capacity was reserved for this item, without
/// reallocating. A full buffer is a defect in the reservation bound.
fn push_reserved<T>(buffer: &mut Vec<T>, item: T) -> Result<(), Defect> {
    if buffer.len() < buffer.capacity() {
        buffer.push(item);
        Ok(())
    } else {
        Err(Defect::ReservedCapacity)
    }
}

struct Parent<W: Dimension, R: Dimension> {
    index: usize,
    tariffs: Tariffs<W, R>,
}

struct ScopeEntry<W: Dimension, R: Dimension> {
    name: &'static str,
    parent: Option<Parent<W, R>>,
    limits: Limits<W, R>,
    usage: Usage<W, R>,
    staged: Usage<W, R>,
    batch_peak: R::Table<u64>,
    direct: R::Table<u64>,
}

impl<W: Dimension, R: Dimension> ScopeEntry<W, R> {
    fn new(name: &'static str, parent: Option<Parent<W, R>>, limits: Limits<W, R>) -> Self {
        Self {
            name,
            parent,
            limits,
            usage: Usage::zero(),
            staged: Usage::zero(),
            batch_peak: R::from_fn(|_| 0),
            direct: R::from_fn(|_| 0),
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
pub struct Operation<W: Dimension, R: Dimension> {
    identity: u64,
    max_scopes: usize,
    scopes: Vec<ScopeEntry<W, R>>,
    // At most one observer. Disabled operations keep its storage unallocated,
    // while enabled setup reserves it fallibly alongside the report buffers.
    profile: Vec<Profile<W, R>>,
    checkpoints: checkpoint::Checkpoints<W, R>,
    // The first defect found. Once set, every fallible call returns it. A
    // `OnceLock` lets read-only calls latch a defect they find.
    defect: OnceLock<Defect>,
}

impl<W: Dimension, R: Dimension> Operation<W, R> {
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
            defect: OnceLock::new(),
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
            profile.add_scope(root, None, name).map_err(Error::Defect)?;
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
        if index >= self.max_scopes {
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
        let observed = self.observe_scope(scope, parent, name);
        self.note(observed)?;
        Ok(scope)
    }

    fn observe_scope(
        &mut self,
        scope: Scope,
        parent: Scope,
        name: &'static str,
    ) -> Result<(), Error<W, R>> {
        if let Some(profile) = self.profile.first_mut() {
            profile
                .add_scope(scope, Some(parent), name)
                .map_err(Error::Defect)?;
        }
        self.checkpoints
            .add_scope(scope, Some(parent), name)
            .map_err(Error::Defect)
    }

    pub fn usage(&self, scope: Scope) -> Result<&Usage<W, R>, Error<W, R>> {
        Ok(&self.entry(self.index(scope)?)?.usage)
    }

    /// The caps configured for a scope, in its own units. Usage includes the
    /// scope's descendants, so a cap applies to them too; this reads only the
    /// scope's own caps and not those of its ancestors.
    pub fn limits(&self, scope: Scope) -> Result<&Limits<W, R>, Error<W, R>> {
        Ok(&self.entry(self.index(scope)?)?.limits)
    }

    /// Replaces the caps of a scope, for the batches admitted after this call.
    ///
    /// Usage, peaks, direct custody, the profile and open intervals stay as they
    /// are. The new caps are not checked against the usage: a scope that already
    /// holds more than a new cap refuses its next acquisition of that dimension,
    /// even of zero, and still accepts releases. Use it when a consumer's
    /// allowance changes between calls.
    pub fn set_limits(&mut self, scope: Scope, limits: Limits<W, R>) -> Result<(), Error<W, R>> {
        let index = self.index(scope)?;
        let Some(entry) = self.scopes.get_mut(index) else {
            return Err(self.latch(Defect::ScopeEntry));
        };
        entry.limits = limits;
        Ok(())
    }

    pub fn scope_name(&self, scope: Scope) -> Result<&'static str, Error<W, R>> {
        Ok(self.entry(self.index(scope)?)?.name)
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
        self.healthy()?;
        let snapshot = self
            .profile
            .first()
            .map(|profile| profile.snapshot(self.scopes.iter().map(|entry| entry.usage)))
            .transpose();
        // A defect is latched; an allocation failure leaves the operation usable.
        match snapshot {
            Err(Error::Defect(defect)) => Err(self.latch(defect)),
            other => other,
        }
    }

    /// Consumes the ledger and returns its structured report if enabled.
    /// Consumers release live resources before finishing when cleanup is part
    /// of the operation; outstanding balances remain visible in the report.
    /// A defect found earlier, including on a path with no error return, is
    /// returned instead of a report.
    pub fn finish(mut self) -> Result<Option<Report<W, R>>, Error<W, R>> {
        self.healthy()?;
        self.profile
            .pop()
            .map(|profile| {
                profile
                    .finish(self.scopes.into_iter().map(|entry| entry.usage))
                    .map_err(Error::Defect)
            })
            .transpose()
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
        if const { in_place::wide::<W, R>() }
            && in_place::preferred::<W, R>(changes, self.chain(origin))
            && let Some(checked) =
                in_place::check(&mut self.scopes, &self.defect, scope, origin, changes)
        {
            return self.note(checked);
        }
        let staged = Admission::new(self, scope)
            .stage(origin, changes)
            .map(|_| ());
        self.note(staged)
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
        if !self.profile.is_empty() || self.checkpoints.configured() {
            self.observe_admission(scope, changes, result)?;
        }
        self.note(result)
    }

    /// Records `value` as the largest reached by a gauge resource at `scope`: one batch releases
    /// the value held and acquires `value`, so a refusal leaves the held value as it was.
    pub(crate) fn record_largest(
        &mut self,
        scope: Scope,
        resource: R,
        value: u64,
    ) -> Result<(), Error<W, R>> {
        let held = self.usage(scope)?.live(resource);
        self.apply_batch(
            scope,
            &[
                Change::Release(resource, held),
                Change::Acquire(resource, value),
            ],
        )
    }

    /// Records an admission in the profile and every open interval. A defect
    /// found while recording is latched and returned.
    fn observe_admission(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
        result: Result<(), Error<W, R>>,
    ) -> Result<(), Error<W, R>> {
        if let Some(profile) = self.profile.first_mut()
            && let Err(defect) = profile.admission(scope, changes, result)
        {
            return Err(Error::Defect(latch_defect(&self.defect, defect)));
        }
        if let Err(defect) = self.checkpoints.admission(scope, changes, result) {
            return Err(Error::Defect(latch_defect(&self.defect, defect)));
        }
        Ok(())
    }

    fn admit_batch(
        &mut self,
        scope: Scope,
        origin: usize,
        changes: &[Change<W, R>],
    ) -> Result<(), Error<W, R>> {
        // An open interval records the batch peak of every resource, which only
        // the dense admission tracks.
        if const { in_place::wide::<W, R>() }
            && !self.checkpoints.configured()
            && in_place::preferred::<W, R>(changes, self.chain(origin))
            && in_place::admit(&mut self.scopes, &self.defect, scope, origin, changes)?
        {
            return Ok(());
        }
        let direct = Admission::new(self, scope).stage(origin, changes)?;
        let Self {
            scopes,
            defect,
            checkpoints,
            ..
        } = self;
        let scopes = scopes.as_mut_slice();
        let mut cursor = Some(origin);
        while let Some(index) = cursor {
            let Some(entry) = scopes.get_mut(index) else {
                return Err(missing_scope(defect));
            };
            entry.usage = entry.staged;
            let peak = entry.batch_peak;
            cursor = entry.parent_index();
            if checkpoints.configured()
                && let Err(found) = checkpoints.commit_peak(index, peak)
            {
                return Err(Error::Defect(latch_defect(defect, found)));
            }
        }
        let Some(entry) = scopes.get_mut(origin) else {
            return Err(missing_scope(defect));
        };
        entry.direct = direct;
        Ok(())
    }

    /// The number of scopes from `index` to the root, both included. A chain
    /// that cannot be read is reported as one no journal holds, which selects
    /// the dense admission.
    fn chain(&self, index: usize) -> usize {
        let mut chain = 0_usize;
        let mut cursor = Some(index);
        while let Some(at) = cursor {
            let Some(entry) = self.scopes.get(at) else {
                return usize::MAX;
            };
            chain = chain.saturating_add(1);
            cursor = entry.parent_index();
            if chain > self.scopes.len() {
                return usize::MAX;
            }
        }
        chain
    }

    fn record_phase(&mut self, scope: Scope, start: Option<Duration>) {
        let end = self.profile.first().and_then(Profile::elapsed);
        let profiled = self
            .profile
            .first_mut()
            .map_or(Ok(()), |profile| profile.end_phase(scope, start, end));
        if let Err(defect) = profiled {
            latch_defect(&self.defect, defect);
        }
        if self.checkpoints.configured()
            && let Err(defect) = self.checkpoints.end_phase(scope, start, end)
        {
            latch_defect(&self.defect, defect);
        }
    }

    const fn scope(&self, index: usize) -> Scope {
        Scope {
            operation: self.identity,
            index,
        }
    }

    /// Returns the latched defect, if any.
    fn healthy(&self) -> Result<(), Error<W, R>> {
        self.defect
            .get()
            .map_or(Ok(()), |&defect| Err(Error::Defect(defect)))
    }

    /// Latches a defect carried by `result` and passes the result on.
    fn note<T>(&mut self, result: Result<T, Error<W, R>>) -> Result<T, Error<W, R>> {
        match result {
            Err(Error::Defect(defect)) => Err(self.latch(defect)),
            other => other,
        }
    }

    /// Records `defect` unless an earlier one is latched, and returns it as an error.
    fn latch(&self, defect: Defect) -> Error<W, R> {
        Error::Defect(latch_defect(&self.defect, defect))
    }

    fn index(&self, scope: Scope) -> Result<usize, Error<W, R>> {
        self.healthy()?;
        if scope.operation != self.identity {
            return Err(Error::ForeignScope);
        }
        if scope.index < self.scopes.len() {
            Ok(scope.index)
        } else {
            Err(self.latch(Defect::ScopeEntry))
        }
    }

    fn entry(&self, index: usize) -> Result<&ScopeEntry<W, R>, Error<W, R>> {
        self.scopes
            .get(index)
            .ok_or_else(|| self.latch(Defect::ScopeEntry))
    }
}

/// Borrowed access to an operation within a profiled phase. The owning ledger
/// remains in place through phase completion.
///
/// ```compile_fail,E0596
/// use surgeist_ledger::{Limits, NoDimension, Operation};
///
/// let mut operation =
///     Operation::<NoDimension, NoDimension>::new("original", Limits::unlimited(), 1).unwrap();
/// let replacement =
///     Operation::<NoDimension, NoDimension>::new("replacement", Limits::unlimited(), 1).unwrap();
/// operation.with_phase(operation.root(), |phase| {
///     std::mem::replace(&mut **phase, replacement);
/// }).unwrap();
/// ```
pub struct Phase<'a, W: Dimension, R: Dimension> {
    start: Option<Duration>,
    scope: Scope,
    operation: &'a mut Operation<W, R>,
}

impl<W: Dimension, R: Dimension> Phase<'_, W, R> {
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

impl<W: Dimension, R: Dimension> Deref for Phase<'_, W, R> {
    type Target = Operation<W, R>;

    fn deref(&self) -> &Self::Target {
        self.operation
    }
}

impl<W: Dimension, R: Dimension> Drop for Phase<'_, W, R> {
    fn drop(&mut self) {
        self.operation.record_phase(self.scope, self.start);
    }
}

/// One batch's view of the scope list, borrowed once so that every walk
/// along a scope chain works on the same slice.
///
/// This is the dense admission. `in_place.rs` applies the same rules to a small
/// batch on a wide pair; a change to an admission rule here changes there too,
/// and the comparison test in `tests.rs` runs both on the same input.
struct Admission<'a, W: Dimension, R: Dimension> {
    scopes: &'a mut [ScopeEntry<W, R>],
    defect: &'a OnceLock<Defect>,
    scope: Scope,
}

impl<'a, W: Dimension, R: Dimension> Admission<'a, W, R> {
    fn new(operation: &'a mut Operation<W, R>, scope: Scope) -> Self {
        Self {
            scopes: operation.scopes.as_mut_slice(),
            defect: &operation.defect,
            scope,
        }
    }

    fn at(&self, index: usize) -> Scope {
        Scope {
            operation: self.scope.operation,
            index,
        }
    }

    /// Stages a batch along the origin's chain and returns the origin's
    /// direct custody after it. Staged usage is committed by the caller.
    fn stage(
        &mut self,
        origin: usize,
        changes: &[Change<W, R>],
    ) -> Result<R::Table<u64>, Error<W, R>> {
        let mut cursor = Some(origin);
        while let Some(index) = cursor {
            let Some(entry) = self.scopes.get_mut(index) else {
                return Err(missing_scope(self.defect));
            };
            entry.staged = entry.usage;
            entry.batch_peak = entry.usage.live;
            cursor = entry.parent_index();
        }

        let Some(origin_entry) = self.scopes.get(origin) else {
            return Err(missing_scope(self.defect));
        };
        let mut direct = origin_entry.direct;
        for (item, &change) in changes.iter().enumerate() {
            match change {
                Change::Work(dimension, amount) => {
                    self.work(origin, item, dimension, amount)?;
                }
                Change::Acquire(dimension, amount) => {
                    self.acquire(origin, item, dimension, amount)?;
                    // Direct holdings are a subset of the admitted inclusive total.
                    let held = R::get_mut(&mut direct, dimension);
                    let Some(total) = held.checked_add(amount) else {
                        return Err(Error::Defect(latch_defect(
                            self.defect,
                            Defect::DirectCustody,
                        )));
                    };
                    *held = total;
                }
                Change::Release(dimension, amount) => {
                    let held = R::get_mut(&mut direct, dimension);
                    let Some(remaining) = held.checked_sub(amount) else {
                        return Err(refused(
                            self.scope,
                            item,
                            change,
                            *held,
                            RefusalReason::Underflow,
                        ));
                    };
                    *held = remaining;
                    self.release(origin, item, dimension, amount)?;
                }
            }
        }
        Ok(direct)
    }

    /// Stages one work change from `index` to the root. Each scope checks its
    /// own limit before the change is projected into its parent's units.
    fn work(
        &mut self,
        mut index: usize,
        item: usize,
        mut dimension: W,
        mut amount: u64,
    ) -> Result<(), Error<W, R>> {
        loop {
            let at = self.at(index);
            let Some(entry) = self.scopes.get_mut(index) else {
                return Err(missing_scope(self.defect));
            };
            let change = Change::Work(dimension, amount);
            let current = W::get_mut(&mut entry.staged.work, dimension);
            let held = *current;
            *current = increase(held, amount, entry.limits.work(dimension))
                .map_err(|reason| refused(at, item, change, held, reason))?;
            let Some(parent) = &entry.parent else {
                return Ok(());
            };
            let tariff = W::get(&parent.tariffs.work, dimension);
            amount = amount
                .checked_mul(tariff.scale)
                .ok_or_else(|| refused(at, item, change, held, RefusalReason::Overflow))?;
            dimension = tariff.target;
            index = parent.index;
        }
    }

    /// Stages one acquisition from `index` to the root, raising each scope's
    /// lifetime and batch peaks.
    fn acquire(
        &mut self,
        mut index: usize,
        item: usize,
        mut dimension: R,
        mut amount: u64,
    ) -> Result<(), Error<W, R>> {
        loop {
            let at = self.at(index);
            let Some(entry) = self.scopes.get_mut(index) else {
                return Err(missing_scope(self.defect));
            };
            let change = Change::Acquire(dimension, amount);
            let current = R::get_mut(&mut entry.staged.live, dimension);
            let held = *current;
            let live = increase(held, amount, entry.limits.live(dimension))
                .map_err(|reason| refused(at, item, change, held, reason))?;
            *current = live;
            let peak = R::get_mut(&mut entry.staged.peak, dimension);
            *peak = (*peak).max(live);
            let batch_peak = R::get_mut(&mut entry.batch_peak, dimension);
            *batch_peak = (*batch_peak).max(live);
            let Some(parent) = &entry.parent else {
                return Ok(());
            };
            let tariff = R::get(&parent.tariffs.resources, dimension);
            amount = amount
                .checked_mul(tariff.scale)
                .ok_or_else(|| refused(at, item, change, held, RefusalReason::Overflow))?;
            dimension = tariff.target;
            index = parent.index;
        }
    }

    /// Stages one release from `index` to the root.
    fn release(
        &mut self,
        mut index: usize,
        item: usize,
        mut dimension: R,
        mut amount: u64,
    ) -> Result<(), Error<W, R>> {
        loop {
            let at = self.at(index);
            let Some(entry) = self.scopes.get_mut(index) else {
                return Err(missing_scope(self.defect));
            };
            let change = Change::Release(dimension, amount);
            let current = R::get_mut(&mut entry.staged.live, dimension);
            let held = *current;
            *current = held
                .checked_sub(amount)
                .ok_or_else(|| refused(at, item, change, held, RefusalReason::Underflow))?;
            let Some(parent) = &entry.parent else {
                return Ok(());
            };
            let tariff = R::get(&parent.tariffs.resources, dimension);
            amount = amount
                .checked_mul(tariff.scale)
                .ok_or_else(|| refused(at, item, change, held, RefusalReason::Overflow))?;
            dimension = tariff.target;
            index = parent.index;
        }
    }
}

/// Builds a refusal. Kept out of line so admission loops stay small.
#[cold]
#[inline(never)]
fn refused<W: Dimension, R: Dimension>(
    scope: Scope,
    item: usize,
    change: Change<W, R>,
    current: u64,
    reason: RefusalReason,
) -> Error<W, R> {
    Error::Refused(Refusal {
        scope,
        item,
        change,
        current,
        reason,
    })
}

/// Latches and returns a missing-scope defect, out of line.
#[cold]
#[inline(never)]
fn missing_scope<W: Dimension, R: Dimension>(defect: &OnceLock<Defect>) -> Error<W, R> {
    Error::Defect(latch_defect(defect, Defect::ScopeEntry))
}

/// Latches `defect` unless an earlier one is latched and returns the latched
/// defect. Out of line: a defect is never on the admission fast path.
#[cold]
#[inline(never)]
fn latch_defect(latched: &OnceLock<Defect>, defect: Defect) -> Defect {
    *latched.get_or_init(|| defect)
}
