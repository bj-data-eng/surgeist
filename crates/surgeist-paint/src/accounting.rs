use crate::PaintError;
use std::{mem::size_of, time::Duration};
pub use surgeist_ledger::RefusalReason;
use surgeist_ledger::{
    Change, Error as LedgerError, Limits, Operation, Phase, ProfileMode, Report, Resource, Scope,
    ScopeReport, Work,
};

/// Defined algorithm actions, not CPU instructions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkKind {
    BoxValidation,
    ParentEdge,
    TopologyVisit,
    SiblingComparison,
    MembershipVisit,
    ContextComparison,
    OrderMove,
    OutputRecord,
}
impl WorkKind {
    pub const ALL: [Self; 8] = [
        Self::BoxValidation,
        Self::ParentEdge,
        Self::TopologyVisit,
        Self::SiblingComparison,
        Self::MembershipVisit,
        Self::ContextComparison,
        Self::OrderMove,
        Self::OutputRecord,
    ];
    pub(crate) fn dimension(self) -> Work<8> {
        Work::new(self as usize).expect("closed work schema")
    }
}
/// Owned storage, excluding caller input and Ledger's independently bounded observer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    ScratchBytes,
    RetainedResultBytes,
}
impl ResourceKind {
    pub const ALL: [Self; 2] = [Self::ScratchBytes, Self::RetainedResultBytes];
    pub(crate) fn dimension(self) -> Resource<2> {
        Resource::new(self as usize).expect("closed resource schema")
    }
}

/// Named limits. An explicit unlimited policy still checks arithmetic.
#[derive(Clone, Copy, Debug)]
pub struct PaintLimits {
    work: [u64; 8],
    resources: [u64; 2],
}
impl PaintLimits {
    pub const fn unlimited() -> Self {
        Self {
            work: [u64::MAX; 8],
            resources: [u64::MAX; 2],
        }
    }
    pub const fn with_work_limit(mut self, kind: WorkKind, limit: u64) -> Self {
        self.work[kind as usize] = limit;
        self
    }
    pub const fn with_resource_limit(mut self, kind: ResourceKind, limit: u64) -> Self {
        self.resources[kind as usize] = limit;
        self
    }
    pub const fn work_limit(self, kind: WorkKind) -> u64 {
        self.work[kind as usize]
    }
    pub const fn resource_limit(self, kind: ResourceKind) -> u64 {
        self.resources[kind as usize]
    }
}

/// Explicit operation policy; profiling/timing do not alter admission or ordering.
#[derive(Clone, Copy, Debug)]
pub struct PaintOptions {
    limits: PaintLimits,
    profile: ProfileMode,
    clock: Option<fn() -> Duration>,
}
impl PaintOptions {
    pub const fn new(limits: PaintLimits, profile: ProfileMode) -> Self {
        Self {
            limits,
            profile,
            clock: None,
        }
    }
    pub const fn limits(self) -> PaintLimits {
        self.limits
    }
    pub const fn with_limits(mut self, limits: PaintLimits) -> Self {
        self.limits = limits;
        self
    }
    pub const fn with_clock(mut self, clock: fn() -> Duration) -> Self {
        self.clock = Some(clock);
        self
    }
    pub(crate) fn operation(self) -> Result<Meter, PaintError> {
        let limits = Limits::new(self.limits.work, self.limits.resources);
        match self.clock {
            Some(clock) => {
                Operation::with_profile_clock("paint stacking", limits, 8, self.profile, clock)
            }
            None => Operation::with_profile("paint stacking", limits, 8, self.profile),
        }
        .map_err(setup_error)
    }
}

/// Fixed operation scopes. Usage is inclusive; root totals must not be summed
/// with child attribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaintPhase {
    Operation,
    Validate,
    Topology,
    SiblingOrder,
    Membership,
    ContextOrder,
    Publication,
    Cleanup,
}
impl PaintPhase {
    fn from_name(name: &str) -> Self {
        match name {
            "validate" => Self::Validate,
            "topology" => Self::Topology,
            "sibling_order" => Self::SiblingOrder,
            "membership" => Self::Membership,
            "context_order" => Self::ContextOrder,
            "publication" => Self::Publication,
            "cleanup" => Self::Cleanup,
            _ => Self::Operation,
        }
    }
}

/// A named admission action, expressed in its originating scope's units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaintChange {
    Work { kind: WorkKind, amount: u64 },
    Acquire { kind: ResourceKind, amount: u64 },
    Release { kind: ResourceKind, amount: u64 },
}
fn change(value: Change<8, 2>) -> PaintChange {
    match value {
        Change::Work(dimension, amount) => PaintChange::Work {
            kind: WorkKind::ALL
                .into_iter()
                .find(|kind| kind.dimension() == dimension)
                .expect("owned work dimension"),
            amount,
        },
        Change::Acquire(dimension, amount) => PaintChange::Acquire {
            kind: ResourceKind::ALL
                .into_iter()
                .find(|kind| kind.dimension() == dimension)
                .expect("owned resource dimension"),
            amount,
        },
        Change::Release(dimension, amount) => PaintChange::Release {
            kind: ResourceKind::ALL
                .into_iter()
                .find(|kind| kind.dimension() == dimension)
                .expect("owned resource dimension"),
            amount,
        },
    }
}
/// Exact ordered refusal, without exposing Ledger's numeric schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetRefusal {
    pub origin: PaintPhase,
    pub limiting_phase: PaintPhase,
    pub item: usize,
    pub change: PaintChange,
    pub current: u64,
    pub reason: RefusalReason,
}
/// Accounting setup/invariant failure, distinct from a budget refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountingFailure {
    Allocation,
    IdentityExhausted,
    ForeignScope,
    ScopeLimit { limit: usize },
    TimingUnavailable,
    Invariant,
}
pub(crate) fn setup_error(error: LedgerError<8, 2>) -> PaintError {
    PaintError::Accounting(match error {
        LedgerError::Allocation => AccountingFailure::Allocation,
        LedgerError::IdentityExhausted => AccountingFailure::IdentityExhausted,
        LedgerError::ForeignScope => AccountingFailure::ForeignScope,
        LedgerError::ScopeLimit { limit } => AccountingFailure::ScopeLimit { limit },
        LedgerError::TimingUnavailable => AccountingFailure::TimingUnavailable,
        LedgerError::Refused(_)
        | LedgerError::ForeignCheckpoint
        | LedgerError::StaleCheckpoint
        | LedgerError::CheckpointLimit { .. }
        | LedgerError::CheckpointsConfigured => AccountingFailure::Invariant,
    })
}
fn phase_name(meter: &Meter, scope: Scope) -> PaintPhase {
    PaintPhase::from_name(meter.scope_name(scope).expect("owned accounting scope"))
}
pub(crate) fn apply(
    meter: &mut PhaseMeter<'_>,
    origin: Scope,
    value: Change<8, 2>,
) -> Result<(), PaintError> {
    meter.apply(origin, value).map_err(|error| match error {
        LedgerError::Refused(refusal) => PaintError::Budget(BudgetRefusal {
            origin: phase_name(meter, origin),
            limiting_phase: phase_name(meter, refusal.scope),
            item: refusal.item,
            change: change(refusal.change),
            current: refusal.current,
            reason: refusal.reason,
        }),
        other => setup_error(other),
    })
}

/// Borrowed inclusive usage/profiling view, directly backed by Ledger.
#[derive(Clone, Copy, Debug)]
pub struct PhaseReport<'a>(&'a ScopeReport<8, 2>);
impl PhaseReport<'_> {
    pub fn phase(self) -> PaintPhase {
        PaintPhase::from_name(self.0.name)
    }
    pub fn work(self, kind: WorkKind) -> u64 {
        self.0.usage.work(kind.dimension())
    }
    pub fn live(self, kind: ResourceKind) -> u64 {
        self.0.usage.live(kind.dimension())
    }
    pub fn peak(self, kind: ResourceKind) -> u64 {
        self.0.usage.peak(kind.dimension())
    }
    pub fn invocations(self) -> u64 {
        self.0.phases
    }
    pub fn elapsed(self) -> Option<Duration> {
        self.0.phase_elapsed
    }
    pub fn accepted_batches(self) -> u64 {
        self.0.accepted_batches
    }
    pub fn refused_batches(self) -> u64 {
        self.0.refused_batches
    }
}

/// Selected detailed observation. No separate event buffer is maintained by Paint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaintEvent {
    Accepted {
        phase: PaintPhase,
        change: PaintChange,
        elapsed: Option<Duration>,
    },
    Refused {
        refusal: BudgetRefusal,
        elapsed: Option<Duration>,
    },
    PhaseCompleted {
        phase: PaintPhase,
        duration: Option<Duration>,
        elapsed: Option<Duration>,
    },
}

/// Ledger's accepted-prefix report, including failed operations. Resource live
/// at success includes returned plan custody; it is not a post-return tracker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaintReport(pub(crate) Report<8, 2>);
impl PaintReport {
    pub fn work(&self, kind: WorkKind) -> u64 {
        self.0.scopes[0].usage.work(kind.dimension())
    }
    pub fn live(&self, kind: ResourceKind) -> u64 {
        self.0.scopes[0].usage.live(kind.dimension())
    }
    pub fn peak(&self, kind: ResourceKind) -> u64 {
        self.0.scopes[0].usage.peak(kind.dimension())
    }
    pub fn elapsed(&self) -> Option<Duration> {
        self.0.elapsed
    }
    pub fn dropped_events(&self) -> u64 {
        self.0.dropped_events
    }
    pub fn saturated(&self) -> bool {
        self.0.saturated
    }
    pub fn phases(&self) -> impl ExactSizeIterator<Item = PhaseReport<'_>> {
        self.0.scopes.iter().map(PhaseReport)
    }
    fn phase(&self, scope: Scope) -> PaintPhase {
        PaintPhase::from_name(
            self.0
                .scopes
                .iter()
                .find(|entry| entry.scope == scope)
                .expect("owned report scope")
                .name,
        )
    }
    pub fn refusals(&self) -> impl Iterator<Item = BudgetRefusal> + '_ {
        self.0.scopes.iter().filter_map(|entry| {
            entry.first_refusal.map(|refusal| BudgetRefusal {
                origin: self.phase(entry.scope),
                limiting_phase: self.phase(refusal.scope),
                item: refusal.item,
                change: change(refusal.change),
                current: refusal.current,
                reason: refusal.reason,
            })
        })
    }
    pub fn events(&self) -> impl ExactSizeIterator<Item = PaintEvent> + '_ {
        self.0.events.iter().map(|event| match *event {
            surgeist_ledger::Event::Accepted {
                scope,
                change: value,
                elapsed,
            } => PaintEvent::Accepted {
                phase: self.phase(scope),
                change: change(value),
                elapsed,
            },
            surgeist_ledger::Event::Refused {
                origin,
                refusal,
                elapsed,
            } => PaintEvent::Refused {
                refusal: BudgetRefusal {
                    origin: self.phase(origin),
                    limiting_phase: self.phase(refusal.scope),
                    item: refusal.item,
                    change: change(refusal.change),
                    current: refusal.current,
                    reason: refusal.reason,
                },
                elapsed,
            },
            surgeist_ledger::Event::Phase {
                scope,
                duration,
                elapsed,
            } => PaintEvent::PhaseCompleted {
                phase: self.phase(scope),
                duration,
                elapsed,
            },
        })
    }
}

pub(crate) type Meter = Operation<8, 2>;
pub(crate) type PhaseMeter<'a> = Phase<'a, 8, 2>;
pub(crate) fn charge(
    meter: &mut PhaseMeter<'_>,
    scope: Scope,
    kind: WorkKind,
    amount: usize,
) -> Result<(), PaintError> {
    let amount = u64::try_from(amount).map_err(|_| PaintError::CapacityOverflow)?;
    apply(meter, scope, Change::Work(kind.dimension(), amount))
}
pub(crate) fn bytes<T>(capacity: usize) -> Result<u64, PaintError> {
    capacity
        .checked_mul(size_of::<T>())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(PaintError::CapacityOverflow)
}
/// Preview requested bytes without mutating usage. Commit only actual backing
/// after a successful allocation. Rejected backing is dropped without publication.
pub(crate) fn allocate<T>(
    meter: &mut PhaseMeter<'_>,
    root: Scope,
    capacity: usize,
    kind: ResourceKind,
) -> Result<Vec<T>, PaintError> {
    let request = Change::Acquire(kind.dimension(), bytes::<T>(capacity)?);
    if meter.check_batch(root, &[request]).is_err() {
        // Record the refusal, never a speculative accepted allocation/peak.
        apply(meter, root, request)?;
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| PaintError::Allocation { resource: kind })?;
    apply(
        meter,
        root,
        Change::Acquire(kind.dimension(), bytes::<T>(values.capacity())?),
    )?;
    Ok(values)
}
pub(crate) fn release<T>(
    meter: &mut PhaseMeter<'_>,
    root: Scope,
    values: Vec<T>,
    kind: ResourceKind,
) {
    let amount = bytes::<T>(values.capacity()).expect("admitted capacity remains representable");
    drop(values);
    if amount != 0 {
        apply(meter, root, Change::Release(kind.dimension(), amount))
            .expect("release exact admitted custody");
    }
}
