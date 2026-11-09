use crate::PaintError;
use std::{mem::size_of, time::Duration};
pub use surgeist_ledger::RefusalReason;
use surgeist_ledger::{
    Change, Dimension, Error as LedgerError, Limits, Operation, Phase, ProfileMode, Report, Scope,
    ScopeReport, Usage, dimensions,
};

dimensions! {
    /// Defined algorithm actions, not CPU instructions.
    pub enum WorkKind => WorkKindTable {
        BoxValidation,
        ParentEdge,
        TopologyVisit,
        SiblingComparison,
        MembershipVisit,
        ContextComparison,
        OrderMove,
        OutputRecord,
        /// One paint-input record or reference validation action.
        PaintValidation,
        /// One fragment inspection, including repeated passes.
        FragmentVisit,
        /// One line inspection.
        LineVisit,
        /// One mixed text, child, or occurrence traversal action.
        InlineMemberVisit,
        /// One primitive record inspection.
        PrimitiveVisit,
        /// One recursive paint-scope work-stack action or visit.
        PaintScopeVisit,
        /// One source, occurrence, or primitive ordering comparison.
        PaintOrderComparison,
        /// One decoration applicator or ordering comparison.
        DecorationComparison,
        /// One input scalar, gradient request discriminant, or stop-list grammar validation.
        GradientValidation,
        /// One direction dispatch, scalar calculation, or scalar inspection.
        GradientDirection,
        /// One scalar gradient geometry or used-position calculation/inspection,
        /// including repeated checks and bounded scalar representation-proof actions.
        GradientCoordinate,
        /// One source-item or expanded stop-record visit, including repeated walks.
        GradientStopVisit,
    }
}

pub(crate) enum OperationKind {
    Stacking,
    PaintOrder,
    LinearGradient,
    GradientStops,
    RadialGradient,
    ZeroHeightGradient,
}
impl OperationKind {
    fn name(&self) -> &'static str {
        match self {
            Self::Stacking => "paint stacking",
            Self::PaintOrder => "paint order",
            Self::LinearGradient => "paint linear gradient",
            Self::GradientStops => "paint gradient stops",
            Self::RadialGradient => "paint radial gradient",
            Self::ZeroHeightGradient => "paint zero-height gradient colors",
        }
    }
    fn scope_limit(&self) -> usize {
        match self {
            Self::Stacking => 8,
            Self::PaintOrder => 10,
            Self::LinearGradient => 5,
            Self::GradientStops => 7,
            Self::RadialGradient => 5,
            Self::ZeroHeightGradient => 7,
        }
    }
}
impl WorkKind {
    /// Every work kind, in declaration order.
    pub const ALL: &'static [Self] = <Self as Dimension>::ALL;
}

dimensions! {
    /// Owned storage, excluding caller input and Ledger's independently bounded observer.
    pub enum ResourceKind => ResourceKindTable {
        ScratchBytes,
        RetainedResultBytes,
    }
}

/// Named limits. An explicit unlimited policy still checks arithmetic.
#[derive(Clone, Copy, Debug)]
pub struct PaintLimits {
    work: WorkKindTable<u64>,
    resources: ResourceKindTable<u64>,
}
impl PaintLimits {
    pub const fn unlimited() -> Self {
        Self {
            work: WorkKindTable::splat(u64::MAX),
            resources: ResourceKindTable::splat(u64::MAX),
        }
    }
    pub fn with_work_limit(mut self, kind: WorkKind, limit: u64) -> Self {
        *WorkKind::get_mut(&mut self.work, kind) = limit;
        self
    }
    pub fn with_resource_limit(mut self, kind: ResourceKind, limit: u64) -> Self {
        *ResourceKind::get_mut(&mut self.resources, kind) = limit;
        self
    }
    pub fn work_limit(self, kind: WorkKind) -> u64 {
        *WorkKind::get(&self.work, kind)
    }
    pub fn resource_limit(self, kind: ResourceKind) -> u64 {
        *ResourceKind::get(&self.resources, kind)
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
    pub(crate) fn operation(self, kind: OperationKind) -> Result<Meter, PaintError> {
        let limits = Limits::new(self.limits.work, self.limits.resources);
        match self.clock {
            Some(clock) => Operation::with_profile_clock(
                kind.name(),
                limits,
                kind.scope_limit(),
                self.profile,
                clock,
            ),
            None => Operation::with_profile(kind.name(), limits, kind.scope_limit(), self.profile),
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
    PaintValidation,
    PaintSequence,
    GradientValidation,
    GradientGeometry,
    GradientStopResolution,
    GradientStopFixup,
    GradientStopDistribution,
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
            "paint_validate" => Self::PaintValidation,
            "paint_sequence" => Self::PaintSequence,
            "gradient_validate" => Self::GradientValidation,
            "gradient_geometry" => Self::GradientGeometry,
            "gradient_stop_resolution" => Self::GradientStopResolution,
            "gradient_stop_fixup" => Self::GradientStopFixup,
            "gradient_stop_distribution" => Self::GradientStopDistribution,
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
fn change(value: Change<WorkKind, ResourceKind>) -> PaintChange {
    match value {
        Change::Work(kind, amount) => PaintChange::Work { kind, amount },
        Change::Acquire(kind, amount) => PaintChange::Acquire { kind, amount },
        Change::Release(kind, amount) => PaintChange::Release { kind, amount },
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
pub(crate) fn setup_error(error: LedgerError<WorkKind, ResourceKind>) -> PaintError {
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
        | LedgerError::CheckpointsConfigured
        | LedgerError::Defect(_) => AccountingFailure::Invariant,
    })
}
fn phase_name(meter: &Meter, scope: Scope) -> Result<PaintPhase, PaintError> {
    meter
        .scope_name(scope)
        .map(PaintPhase::from_name)
        .map_err(setup_error)
}
pub(crate) fn apply(
    meter: &mut PhaseMeter<'_>,
    origin: Scope,
    value: Change<WorkKind, ResourceKind>,
) -> Result<(), PaintError> {
    match meter.apply(origin, value) {
        Ok(()) => Ok(()),
        Err(LedgerError::Refused(refusal)) => Err(PaintError::Budget(BudgetRefusal {
            origin: phase_name(meter, origin)?,
            limiting_phase: phase_name(meter, refusal.scope)?,
            item: refusal.item,
            change: change(refusal.change),
            current: refusal.current,
            reason: refusal.reason,
        })),
        Err(other) => Err(setup_error(other)),
    }
}

/// Borrowed inclusive usage/profiling view, directly backed by Ledger.
#[derive(Clone, Copy, Debug)]
pub struct PhaseReport<'a>(&'a ScopeReport<WorkKind, ResourceKind>);
impl PhaseReport<'_> {
    pub fn phase(self) -> PaintPhase {
        PaintPhase::from_name(self.0.name)
    }
    pub fn work(self, kind: WorkKind) -> u64 {
        self.0.usage.work(kind)
    }
    pub fn live(self, kind: ResourceKind) -> u64 {
        self.0.usage.live(kind)
    }
    pub fn peak(self, kind: ResourceKind) -> u64 {
        self.0.usage.peak(kind)
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
///
/// Construction verifies that the report has its root scope and that every
/// scope its refusals and events name has an entry, so reading it cannot fail.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaintReport {
    report: Report<WorkKind, ResourceKind>,
    root: Usage<WorkKind, ResourceKind>,
}
impl PaintReport {
    /// Takes a finished ledger report, or refuses one that does not name its own scopes.
    pub(crate) fn new(report: Report<WorkKind, ResourceKind>) -> Result<Self, PaintError> {
        let Some(root) = report.scopes.first().map(|entry| entry.usage) else {
            return Err(PaintError::Accounting(AccountingFailure::Invariant));
        };
        let known = |scope| report.scopes.iter().any(|entry| entry.scope == scope);
        let scopes_resolve = report
            .scopes
            .iter()
            .filter_map(|entry| entry.first_refusal)
            .all(|refusal| known(refusal.scope))
            && report.events.iter().all(|event| match *event {
                surgeist_ledger::Event::Accepted { scope, .. }
                | surgeist_ledger::Event::Phase { scope, .. } => known(scope),
                surgeist_ledger::Event::Refused {
                    origin, refusal, ..
                } => known(origin) && known(refusal.scope),
            });
        if scopes_resolve {
            Ok(Self { report, root })
        } else {
            Err(PaintError::Accounting(AccountingFailure::Invariant))
        }
    }
    pub fn work(&self, kind: WorkKind) -> u64 {
        self.root.work(kind)
    }
    pub fn live(&self, kind: ResourceKind) -> u64 {
        self.root.live(kind)
    }
    pub fn peak(&self, kind: ResourceKind) -> u64 {
        self.root.peak(kind)
    }
    pub fn elapsed(&self) -> Option<Duration> {
        self.report.elapsed
    }
    pub fn dropped_events(&self) -> u64 {
        self.report.dropped_events
    }
    pub fn saturated(&self) -> bool {
        self.report.saturated
    }
    pub fn phases(&self) -> impl ExactSizeIterator<Item = PhaseReport<'_>> {
        self.report.scopes.iter().map(PhaseReport)
    }
    /// The phase of a scope that construction verified the report names.
    fn phase(&self, scope: Scope) -> PaintPhase {
        self.report
            .scopes
            .iter()
            .find(|entry| entry.scope == scope)
            .map_or(PaintPhase::Operation, |entry| {
                PaintPhase::from_name(entry.name)
            })
    }
    pub fn refusals(&self) -> impl Iterator<Item = BudgetRefusal> + '_ {
        self.report.scopes.iter().filter_map(|entry| {
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
        self.report.events.iter().map(|event| match *event {
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

pub(crate) type Meter = Operation<WorkKind, ResourceKind>;
pub(crate) type PhaseMeter<'a> = Phase<'a, WorkKind, ResourceKind>;
pub(crate) fn charge(
    meter: &mut PhaseMeter<'_>,
    scope: Scope,
    kind: WorkKind,
    amount: usize,
) -> Result<(), PaintError> {
    let amount = u64::try_from(amount).map_err(|_| PaintError::CapacityOverflow)?;
    apply(meter, scope, Change::Work(kind, amount))
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
    let request = Change::Acquire(kind, bytes::<T>(capacity)?);
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
        Change::Acquire(kind, bytes::<T>(values.capacity())?),
    )?;
    Ok(values)
}
pub(crate) fn release<T>(
    meter: &mut PhaseMeter<'_>,
    root: Scope,
    values: Vec<T>,
    kind: ResourceKind,
) -> Result<(), PaintError> {
    let amount = bytes::<T>(values.capacity())?;
    drop(values);
    if amount != 0 {
        apply(meter, root, Change::Release(kind, amount))?;
    }
    Ok(())
}

/// Closes the operation. A ledger defect found while it ran replaces the result,
/// because accepted counts can no longer be trusted.
pub(crate) fn finish<T>(
    meter: Meter,
    result: Result<T, PaintError>,
) -> (Result<T, PaintError>, Option<PaintReport>) {
    match meter
        .finish()
        .map_err(setup_error)
        .and_then(|report| report.map(PaintReport::new).transpose())
    {
        Ok(report) => (result, report),
        Err(error) => (Err(error), None),
    }
}

impl ResourceKind {
    /// Every resource kind, in declaration order.
    pub const ALL: &'static [Self] = <Self as Dimension>::ALL;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_ledger_defects_remain_distinct_accounting_failures() {
        assert_ne!(
            setup_error(LedgerError::Defect(surgeist_ledger::Defect::ScopeEntry)),
            setup_error(LedgerError::Defect(surgeist_ledger::Defect::ReportScope))
        );
    }

    fn report_with_validate_scope() -> (Report<WorkKind, ResourceKind>, Scope) {
        let mut meter = PaintOptions::new(
            PaintLimits::unlimited(),
            ProfileMode::Summary { timing: false },
        )
        .operation(OperationKind::Stacking)
        .unwrap();
        let validate = meter
            .child(
                meter.root(),
                "validate",
                Limits::unlimited(),
                surgeist_ledger::Tariffs::identity(),
            )
            .unwrap();
        (meter.finish().unwrap().unwrap(), validate)
    }

    #[test]
    fn known_scope_keeps_its_phase_when_its_display_label_changes() {
        let (mut report, validate) = report_with_validate_scope();
        report
            .scopes
            .iter_mut()
            .find(|entry| entry.scope == validate)
            .unwrap()
            .name = "unrelated display label";

        let report = PaintReport::new(report).unwrap();
        assert_eq!(
            report.phases().map(PhaseReport::phase).collect::<Vec<_>>(),
            [PaintPhase::Operation, PaintPhase::Validate]
        );
    }

    #[test]
    fn foreign_scope_with_recognized_label_is_an_accounting_invariant_failure() {
        let (mut report, validate) = report_with_validate_scope();
        let foreign = Operation::<WorkKind, ResourceKind>::new(
            "foreign operation",
            Limits::unlimited(),
            1,
        )
        .unwrap();
        report
            .scopes
            .iter_mut()
            .find(|entry| entry.scope == validate)
            .unwrap()
            .scope = foreign.root();

        assert_eq!(
            PaintReport::new(report),
            Err(PaintError::Accounting(AccountingFailure::Invariant))
        );
    }

    #[test]
    fn report_without_its_root_scope_is_an_accounting_invariant_failure() {
        let report = Report {
            elapsed: None,
            scopes: Vec::new(),
            events: Vec::new(),
            dropped_events: 0,
            saturated: false,
        };
        assert_eq!(
            PaintReport::new(report),
            Err(PaintError::Accounting(AccountingFailure::Invariant))
        );
    }
}
