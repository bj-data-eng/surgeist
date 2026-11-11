use crate::PaintError;
use std::{mem::size_of, time::Duration};
pub use surgeist_ledger::RefusalReason;
use surgeist_ledger::{
    Change, Dimension, Error as LedgerError, Limits, Operation, Phase, ProfileMode, Report, Scope,
    Tariffs, Usage, dimensions,
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
        /// One border-image input scalar validity check.
        BorderImageValidation,
        /// One border-image coordinate calculation or inspection.
        BorderImageCoordinate,
    }
}

pub(crate) enum OperationKind {
    Stacking,
    PaintOrder,
    LinearGradient,
    GradientStops,
    RadialGradient,
    ZeroHeightGradient,
    BorderImageArea,
    BorderImage,
    BorderImageTiling,
    MaskBorder,
}
const MAX_OPERATION_SCOPES: usize = 10;

impl OperationKind {
    fn name(&self) -> &'static str {
        match self {
            Self::Stacking => "paint stacking",
            Self::PaintOrder => "paint order",
            Self::LinearGradient => "paint linear gradient",
            Self::GradientStops => "paint gradient stops",
            Self::RadialGradient => "paint radial gradient",
            Self::ZeroHeightGradient => "paint zero-height gradient colors",
            Self::BorderImageArea => "paint border image area",
            Self::BorderImage => "paint border image geometry",
            Self::BorderImageTiling => "paint border image tiling",
            Self::MaskBorder => "paint mask border",
        }
    }
    fn scope_limit(&self) -> usize {
        match self {
            Self::Stacking => 8,
            Self::PaintOrder => MAX_OPERATION_SCOPES,
            Self::LinearGradient => 5,
            Self::GradientStops => 7,
            Self::RadialGradient => 5,
            Self::ZeroHeightGradient => 7,
            Self::BorderImageArea | Self::BorderImage => 5,
            Self::BorderImageTiling => 4,
            Self::MaskBorder => 6,
        }
    }
}
impl WorkKind {
    /// Every work kind, in declaration order.
    pub const ALL: &'static [Self] = <Self as Dimension>::ALL;
}

dimensions! {
    /// Owned algorithm/result storage, excluding caller input and the independently
    /// bounded Ledger observer and Paint report adapter.
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
        let scope_limit = kind.scope_limit();
        if scope_limit == 0 || scope_limit > MAX_OPERATION_SCOPES {
            return Err(invariant());
        }
        let backing = ReportBacking::new(scope_limit, self.profile)?;
        let operation = match self.clock {
            Some(clock) => Operation::with_profile_clock(
                kind.name(),
                limits,
                kind.scope_limit(),
                self.profile,
                clock,
            ),
            None => Operation::with_profile(kind.name(), limits, kind.scope_limit(), self.profile),
        }
        .map_err(setup_error)?;
        Ok(Meter {
            bindings: ScopeBindings::new(operation.root()),
            operation,
            backing,
        })
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
    BorderImageGeometry,
    BorderImageArea,
    BorderImageTiling,
}
impl PaintPhase {
    fn name(self) -> &'static str {
        match self {
            Self::Operation => "operation",
            Self::Validate => "validate",
            Self::Topology => "topology",
            Self::SiblingOrder => "sibling_order",
            Self::Membership => "membership",
            Self::ContextOrder => "context_order",
            Self::Publication => "publication",
            Self::Cleanup => "cleanup",
            Self::PaintValidation => "paint_validate",
            Self::PaintSequence => "paint_sequence",
            Self::GradientValidation => "gradient_validate",
            Self::GradientGeometry => "gradient_geometry",
            Self::GradientStopResolution => "gradient_stop_resolution",
            Self::GradientStopFixup => "gradient_stop_fixup",
            Self::GradientStopDistribution => "gradient_stop_distribution",
            Self::BorderImageGeometry => "border_image_geometry",
            Self::BorderImageArea => "border_image_area",
            Self::BorderImageTiling => "border_image_tiling",
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
    LedgerDefect(surgeist_ledger::Defect),
    Invariant,
}
fn invariant() -> PaintError {
    PaintError::Accounting(AccountingFailure::Invariant)
}
pub(crate) fn setup_error(error: LedgerError<WorkKind, ResourceKind>) -> PaintError {
    PaintError::Accounting(match error {
        LedgerError::Allocation => AccountingFailure::Allocation,
        LedgerError::IdentityExhausted => AccountingFailure::IdentityExhausted,
        LedgerError::ForeignScope => AccountingFailure::ForeignScope,
        LedgerError::ScopeLimit { limit } => AccountingFailure::ScopeLimit { limit },
        LedgerError::TimingUnavailable => AccountingFailure::TimingUnavailable,
        LedgerError::Defect(defect) => AccountingFailure::LedgerDefect(defect),
        LedgerError::Refused(_)
        | LedgerError::ForeignCheckpoint
        | LedgerError::StaleCheckpoint
        | LedgerError::CheckpointLimit { .. }
        | LedgerError::CheckpointsConfigured => AccountingFailure::Invariant,
    })
}

#[derive(Clone, Copy, Debug)]
struct ScopeBindings {
    root: Scope,
    children: [Option<(Scope, PaintPhase)>; MAX_OPERATION_SCOPES - 1],
}
impl ScopeBindings {
    fn new(root: Scope) -> Self {
        Self {
            root,
            children: [None; MAX_OPERATION_SCOPES - 1],
        }
    }
    fn phase(&self, scope: Scope) -> Result<PaintPhase, PaintError> {
        if scope == self.root {
            return Ok(PaintPhase::Operation);
        }
        self.children
            .iter()
            .flatten()
            .find_map(|&(handle, phase)| (handle == scope).then_some(phase))
            .ok_or_else(invariant)
    }
    fn len(&self) -> usize {
        1 + self.children.iter().flatten().count()
    }
}

fn resolve_refusal(
    origin: Scope,
    refusal: surgeist_ledger::Refusal<WorkKind, ResourceKind>,
    phase: impl Fn(Scope) -> Result<PaintPhase, PaintError>,
) -> Result<BudgetRefusal, PaintError> {
    Ok(BudgetRefusal {
        origin: phase(origin)?,
        limiting_phase: phase(refusal.scope)?,
        item: refusal.item,
        change: change(refusal.change),
        current: refusal.current,
        reason: refusal.reason,
    })
}
pub(crate) fn apply(
    meter: &mut PhaseMeter<'_, '_>,
    origin: Scope,
    value: Change<WorkKind, ResourceKind>,
) -> Result<(), PaintError> {
    match meter.phase.apply(origin, value) {
        Ok(()) => Ok(()),
        Err(LedgerError::Refused(refusal)) => Err(PaintError::Budget(resolve_refusal(
            origin,
            refusal,
            |scope| meter.bindings.phase(scope),
        )?)),
        Err(other) => Err(setup_error(other)),
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ResolvedPhaseReport {
    phase: PaintPhase,
    usage: Usage<WorkKind, ResourceKind>,
    accepted_batches: u64,
    refused_batches: u64,
    first_refusal: Option<BudgetRefusal>,
    invocations: u64,
    elapsed: Option<Duration>,
}

/// Borrowed inclusive usage/profiling view resolved from Ledger at construction.
#[derive(Clone, Copy, Debug)]
pub struct PhaseReport<'a>(&'a ResolvedPhaseReport);
impl PhaseReport<'_> {
    pub fn phase(self) -> PaintPhase {
        self.0.phase
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
        self.0.invocations
    }
    pub fn elapsed(self) -> Option<Duration> {
        self.0.elapsed
    }
    pub fn accepted_batches(self) -> u64 {
        self.0.accepted_batches
    }
    pub fn refused_batches(self) -> u64 {
        self.0.refused_batches
    }
}

/// Selected detailed observation, resolved once when the report is constructed.
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

// Observer backing follows the same setup bounds as Ledger's observer. It is
// separate from algorithm/result custody and never participates in admission.
#[derive(Debug)]
struct ReportBacking {
    scopes: Vec<ResolvedPhaseReport>,
    events: Vec<PaintEvent>,
    scope_limit: usize,
    event_limit: usize,
}
impl ReportBacking {
    fn new(scope_limit: usize, profile: ProfileMode) -> Result<Option<Self>, PaintError> {
        let event_limit = match profile {
            ProfileMode::Off => return Ok(None),
            ProfileMode::Summary { .. } => 0,
            ProfileMode::Detailed { event_limit, .. } => event_limit,
        };
        let mut scopes = Vec::new();
        let mut events = Vec::new();
        scopes
            .try_reserve_exact(scope_limit)
            .map_err(|_| PaintError::Accounting(AccountingFailure::Allocation))?;
        events
            .try_reserve_exact(event_limit)
            .map_err(|_| PaintError::Accounting(AccountingFailure::Allocation))?;
        Ok(Some(Self {
            scopes,
            events,
            scope_limit,
            event_limit,
        }))
    }
}
fn push_reserved<T>(values: &mut Vec<T>, value: T, limit: usize) -> Result<(), PaintError> {
    if values.len() >= limit || values.len() >= values.capacity() {
        return Err(invariant());
    }
    values.push(value);
    Ok(())
}

/// Ledger's accepted-prefix report, including failed operations. Resource live
/// at success includes returned plan custody; it is not a post-return tracker.
///
/// Construction resolves all scope handles through Paint's registered bindings.
/// The report retains only resolved summaries, refusals and events; readers need
/// neither a scope lookup nor a display-name interpretation.
#[derive(Debug, Eq, PartialEq)]
pub struct PaintReport {
    root: Usage<WorkKind, ResourceKind>,
    scopes: Vec<ResolvedPhaseReport>,
    events: Vec<PaintEvent>,
    elapsed: Option<Duration>,
    dropped_events: u64,
    saturated: bool,
}
impl PaintReport {
    fn new(
        report: Report<WorkKind, ResourceKind>,
        bindings: &ScopeBindings,
        mut backing: ReportBacking,
    ) -> Result<Self, PaintError> {
        let root = report
            .scopes
            .first()
            .filter(|entry| entry.scope == bindings.root && entry.parent.is_none())
            .map(|entry| entry.usage)
            .ok_or_else(invariant)?;
        if report.scopes.len() != bindings.len()
            || report.scopes.len() > backing.scope_limit
            || report.events.len() > backing.event_limit
            || !backing.scopes.is_empty()
            || !backing.events.is_empty()
        {
            return Err(invariant());
        }
        let phase = |scope| {
            if report.scopes.iter().any(|entry| entry.scope == scope) {
                bindings.phase(scope)
            } else {
                Err(invariant())
            }
        };
        for (index, entry) in report.scopes.iter().enumerate() {
            if report
                .scopes
                .iter()
                .take(index)
                .any(|other| other.scope == entry.scope)
            {
                return Err(invariant());
            }
            if let Some(parent) = entry.parent {
                phase(parent)?;
            }
            let summary = ResolvedPhaseReport {
                phase: phase(entry.scope)?,
                usage: entry.usage,
                accepted_batches: entry.accepted_batches,
                refused_batches: entry.refused_batches,
                first_refusal: entry
                    .first_refusal
                    .map(|refusal| resolve_refusal(entry.scope, refusal, phase))
                    .transpose()?,
                invocations: entry.phases,
                elapsed: entry.phase_elapsed,
            };
            push_reserved(&mut backing.scopes, summary, backing.scope_limit)?;
        }
        for event in report.events.iter().copied() {
            let event = match event {
                surgeist_ledger::Event::Accepted {
                    scope,
                    change: value,
                    elapsed,
                } => PaintEvent::Accepted {
                    phase: phase(scope)?,
                    change: change(value),
                    elapsed,
                },
                surgeist_ledger::Event::Refused {
                    origin,
                    refusal,
                    elapsed,
                } => PaintEvent::Refused {
                    refusal: resolve_refusal(origin, refusal, phase)?,
                    elapsed,
                },
                surgeist_ledger::Event::Phase {
                    scope,
                    duration,
                    elapsed,
                } => PaintEvent::PhaseCompleted {
                    phase: phase(scope)?,
                    duration,
                    elapsed,
                },
            };
            push_reserved(&mut backing.events, event, backing.event_limit)?;
        }
        Ok(Self {
            root,
            scopes: backing.scopes,
            events: backing.events,
            elapsed: report.elapsed,
            dropped_events: report.dropped_events,
            saturated: report.saturated,
        })
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
        self.elapsed
    }
    pub fn dropped_events(&self) -> u64 {
        self.dropped_events
    }
    pub fn saturated(&self) -> bool {
        self.saturated
    }
    pub fn phases(&self) -> impl ExactSizeIterator<Item = PhaseReport<'_>> {
        self.scopes.iter().map(PhaseReport)
    }
    pub fn refusals(&self) -> impl Iterator<Item = BudgetRefusal> + '_ {
        self.scopes.iter().filter_map(|entry| entry.first_refusal)
    }
    pub fn events(&self) -> impl ExactSizeIterator<Item = PaintEvent> + '_ {
        self.events.iter().copied()
    }
}

pub(crate) struct Meter {
    operation: Operation<WorkKind, ResourceKind>,
    bindings: ScopeBindings,
    backing: Option<ReportBacking>,
}
impl Meter {
    pub(crate) fn root(&self) -> Scope {
        self.bindings.root
    }
    pub(crate) fn child(
        &mut self,
        parent: Scope,
        phase: PaintPhase,
        limits: Limits<WorkKind, ResourceKind>,
        tariffs: Tariffs<WorkKind, ResourceKind>,
    ) -> Result<Scope, PaintError> {
        // The inline table covers every OperationKind's bound. Keep Ledger's
        // parent/defect/scope-limit error precedence even when the table is full.
        let slot = self
            .bindings
            .children
            .iter_mut()
            .find(|entry| entry.is_none());
        let scope = self
            .operation
            .child(parent, phase.name(), limits, tariffs)
            .map_err(setup_error)?;
        let slot = slot.ok_or_else(invariant)?;
        *slot = Some((scope, phase));
        Ok(scope)
    }
    #[cfg(test)]
    pub(crate) fn scope_name(
        &self,
        scope: Scope,
    ) -> Result<&'static str, LedgerError<WorkKind, ResourceKind>> {
        self.operation.scope_name(scope)
    }
    pub(crate) fn with_phase<T>(
        &mut self,
        scope: Scope,
        run: impl FnOnce(&mut PhaseMeter<'_, '_>) -> T,
    ) -> Result<T, LedgerError<WorkKind, ResourceKind>> {
        let bindings = &self.bindings;
        self.operation
            .with_phase(scope, |phase| run(&mut PhaseMeter { phase, bindings }))
    }
    pub(crate) fn finish(self) -> Result<Option<PaintReport>, PaintError> {
        // Ledger latches bookkeeping defects, including failed child observation,
        // and refuses finish before any projection of an untrusted report.
        let report = self.operation.finish().map_err(setup_error)?;
        match (report, self.backing) {
            (None, None) => Ok(None),
            (Some(report), Some(backing)) => {
                PaintReport::new(report, &self.bindings, backing).map(Some)
            }
            (None, Some(_)) | (Some(_), None) => Err(invariant()),
        }
    }
}

pub(crate) struct PhaseMeter<'a, 'operation> {
    phase: &'a mut Phase<'operation, WorkKind, ResourceKind>,
    bindings: &'a ScopeBindings,
}
impl PhaseMeter<'_, '_> {
    fn check_batch(
        &mut self,
        scope: Scope,
        changes: &[Change<WorkKind, ResourceKind>],
    ) -> Result<(), LedgerError<WorkKind, ResourceKind>> {
        self.phase.check_batch(scope, changes)
    }
}
pub(crate) fn charge(
    meter: &mut PhaseMeter<'_, '_>,
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
    meter: &mut PhaseMeter<'_, '_>,
    root: Scope,
    capacity: usize,
    kind: ResourceKind,
) -> Result<Vec<T>, PaintError> {
    let request = Change::Acquire(kind, bytes::<T>(capacity)?);
    if meter.check_batch(root, &[request]).is_err() {
        // Record the refusal, never a speculative accepted allocation/peak.
        apply(meter, root, request)?;
    }
    admit_capacity(meter, root, reserve_exact(capacity, kind)?, kind)
}

/// Prefer spare capacity when its requested and actual backing both fit.
/// Only the selected allocation commits custody or records a refusal.
pub(crate) fn allocate_with_headroom<T>(
    meter: &mut PhaseMeter<'_, '_>,
    root: Scope,
    required: usize,
    preferred: usize,
    kind: ResourceKind,
) -> Result<Vec<T>, PaintError> {
    if preferred > required
        && preview_optional_capacity::<T>(meter, root, preferred, kind)?
        && let Ok(values) = reserve_exact::<T>(preferred, kind)
        && (values.capacity() == preferred
            || preview_optional_capacity::<T>(meter, root, values.capacity(), kind)?)
    {
        return admit_capacity(meter, root, values, kind);
    }
    allocate(meter, root, required, kind)
}

fn preview_optional_capacity<T>(
    meter: &mut PhaseMeter<'_, '_>,
    root: Scope,
    capacity: usize,
    kind: ResourceKind,
) -> Result<bool, PaintError> {
    let Ok(amount) = bytes::<T>(capacity) else {
        return Ok(false);
    };
    match meter.check_batch(root, &[Change::Acquire(kind, amount)]) {
        Ok(()) => Ok(true),
        Err(LedgerError::Refused(_)) => Ok(false),
        Err(error) => Err(setup_error(error)),
    }
}

fn reserve_exact<T>(capacity: usize, kind: ResourceKind) -> Result<Vec<T>, PaintError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| PaintError::Allocation { resource: kind })?;
    Ok(values)
}

fn admit_capacity<T>(
    meter: &mut PhaseMeter<'_, '_>,
    root: Scope,
    values: Vec<T>,
    kind: ResourceKind,
) -> Result<Vec<T>, PaintError> {
    apply(
        meter,
        root,
        Change::Acquire(kind, bytes::<T>(values.capacity())?),
    )?;
    Ok(values)
}
pub(crate) fn release<T>(
    meter: &mut PhaseMeter<'_, '_>,
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
    match meter.finish() {
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

    fn report_with_validate_scope() -> (
        Report<WorkKind, ResourceKind>,
        Scope,
        ScopeBindings,
        ReportBacking,
    ) {
        report_with_validate_scope_profile(ProfileMode::Summary { timing: false })
    }

    fn report_with_validate_scope_profile(
        profile: ProfileMode,
    ) -> (
        Report<WorkKind, ResourceKind>,
        Scope,
        ScopeBindings,
        ReportBacking,
    ) {
        let mut meter = PaintOptions::new(PaintLimits::unlimited(), profile)
            .operation(OperationKind::Stacking)
            .unwrap();
        let validate = meter
            .child(
                meter.root(),
                PaintPhase::Validate,
                Limits::unlimited(),
                surgeist_ledger::Tariffs::identity(),
            )
            .unwrap();
        let Meter {
            operation,
            bindings,
            backing,
            ..
        } = meter;
        (
            operation.finish().unwrap().unwrap(),
            validate,
            bindings,
            backing.unwrap(),
        )
    }

    #[test]
    fn known_scope_keeps_its_phase_when_its_display_label_changes() {
        let (mut report, validate, bindings, backing) = report_with_validate_scope();
        report
            .scopes
            .iter_mut()
            .find(|entry| entry.scope == validate)
            .unwrap()
            .name = "unrelated display label";

        let report = PaintReport::new(report, &bindings, backing).unwrap();
        assert_eq!(
            report.phases().map(PhaseReport::phase).collect::<Vec<_>>(),
            [PaintPhase::Operation, PaintPhase::Validate]
        );
    }

    #[test]
    fn foreign_scope_with_recognized_label_is_an_accounting_invariant_failure() {
        let (mut report, validate, bindings, backing) = report_with_validate_scope();
        let foreign =
            Operation::<WorkKind, ResourceKind>::new("foreign operation", Limits::unlimited(), 1)
                .unwrap();
        report
            .scopes
            .iter_mut()
            .find(|entry| entry.scope == validate)
            .unwrap()
            .scope = foreign.root();

        assert_eq!(
            PaintReport::new(report, &bindings, backing),
            Err(PaintError::Accounting(AccountingFailure::Invariant))
        );
    }

    #[test]
    fn report_without_its_root_scope_is_an_accounting_invariant_failure() {
        let (_, _, bindings, backing) = report_with_validate_scope();
        let report = Report {
            elapsed: None,
            scopes: Vec::new(),
            events: Vec::new(),
            dropped_events: 0,
            saturated: false,
        };
        assert_eq!(
            PaintReport::new(report, &bindings, backing),
            Err(PaintError::Accounting(AccountingFailure::Invariant))
        );
    }

    #[test]
    fn named_ledger_defects_survive_accounting_conversion() {
        use surgeist_ledger::Defect;
        for defect in [
            Defect::ScopeEntry,
            Defect::ReportScope,
            Defect::ReservedCapacity,
            Defect::DirectCustody,
            Defect::IntervalWork,
            Defect::EventLimit,
            Defect::CheckpointSlot,
        ] {
            assert_eq!(
                setup_error(LedgerError::Defect(defect)),
                PaintError::Accounting(AccountingFailure::LedgerDefect(defect))
            );
        }
    }

    fn detailed(event_limit: usize) -> ProfileMode {
        ProfileMode::Detailed {
            timing: false,
            event_limit,
            filter: surgeist_ledger::EventFilter::All,
        }
    }

    fn refusal(scope: Scope) -> surgeist_ledger::Refusal<WorkKind, ResourceKind> {
        surgeist_ledger::Refusal {
            scope,
            item: 2,
            change: Change::Work(WorkKind::BoxValidation, 3),
            current: 5,
            reason: RefusalReason::Limit { limit: 6 },
        }
    }

    #[test]
    fn unknown_observation_and_parent_handles_fail_report_construction() {
        enum Reference {
            Parent,
            SummaryRefusal,
            Accepted,
            RefusedOrigin,
            RefusedLimit,
            Phase,
        }
        let foreign =
            Operation::<WorkKind, ResourceKind>::new("validate", Limits::unlimited(), 1).unwrap();
        let unknown = foreign.root();
        for reference in [
            Reference::Parent,
            Reference::SummaryRefusal,
            Reference::Accepted,
            Reference::RefusedOrigin,
            Reference::RefusedLimit,
            Reference::Phase,
        ] {
            let (mut raw, validate, bindings, backing) =
                report_with_validate_scope_profile(detailed(1));
            match reference {
                Reference::Parent => raw.scopes.last_mut().unwrap().parent = Some(unknown),
                Reference::SummaryRefusal => {
                    raw.scopes.last_mut().unwrap().first_refusal = Some(refusal(unknown))
                }
                Reference::Accepted => raw.events.push(surgeist_ledger::Event::Accepted {
                    scope: unknown,
                    change: Change::Work(WorkKind::BoxValidation, 1),
                    elapsed: None,
                }),
                Reference::RefusedOrigin => raw.events.push(surgeist_ledger::Event::Refused {
                    origin: unknown,
                    refusal: refusal(bindings.root),
                    elapsed: None,
                }),
                Reference::RefusedLimit => raw.events.push(surgeist_ledger::Event::Refused {
                    origin: validate,
                    refusal: refusal(unknown),
                    elapsed: None,
                }),
                Reference::Phase => raw.events.push(surgeist_ledger::Event::Phase {
                    scope: unknown,
                    duration: None,
                    elapsed: None,
                }),
            }
            assert_eq!(PaintReport::new(raw, &bindings, backing), Err(invariant()));
        }
    }

    #[test]
    fn registered_event_handle_without_a_report_row_is_an_invariant_failure() {
        let (mut raw, validate, bindings, backing) =
            report_with_validate_scope_profile(detailed(1));
        raw.scopes.retain(|entry| entry.scope != validate);
        raw.events.push(surgeist_ledger::Event::Accepted {
            scope: validate,
            change: Change::Work(WorkKind::BoxValidation, 1),
            elapsed: None,
        });
        assert_eq!(PaintReport::new(raw, &bindings, backing), Err(invariant()));
    }

    #[test]
    fn child_row_cannot_replace_the_registered_root() {
        let (mut raw, _, bindings, backing) = report_with_validate_scope();
        raw.scopes.swap(0, 1);
        assert_eq!(PaintReport::new(raw, &bindings, backing), Err(invariant()));
    }

    #[test]
    fn duplicate_scope_rows_fail_report_construction() {
        let (mut raw, _, bindings, backing) = report_with_validate_scope();
        raw.scopes[1] = raw.scopes[0].clone();
        assert_eq!(PaintReport::new(raw, &bindings, backing), Err(invariant()));
    }

    #[test]
    fn known_observations_keep_phases_payloads_and_order_despite_display_labels() {
        let (mut raw, validate, bindings, backing) =
            report_with_validate_scope_profile(detailed(3));
        let elapsed = Some(Duration::from_millis(7));
        let duration = Some(Duration::from_millis(2));
        let raw_refusal = refusal(bindings.root);
        raw.scopes.last_mut().unwrap().first_refusal = Some(raw_refusal);
        for row in &mut raw.scopes {
            row.name = "unrelated display label";
        }
        raw.events = vec![
            surgeist_ledger::Event::Accepted {
                scope: validate,
                change: Change::Work(WorkKind::BoxValidation, 4),
                elapsed,
            },
            surgeist_ledger::Event::Refused {
                origin: validate,
                refusal: raw_refusal,
                elapsed,
            },
            surgeist_ledger::Event::Phase {
                scope: validate,
                duration,
                elapsed,
            },
        ];
        raw.elapsed = elapsed;
        raw.dropped_events = 13;
        raw.saturated = true;
        let report = PaintReport::new(raw, &bindings, backing).unwrap();
        let expected_refusal = BudgetRefusal {
            origin: PaintPhase::Validate,
            limiting_phase: PaintPhase::Operation,
            item: 2,
            change: PaintChange::Work {
                kind: WorkKind::BoxValidation,
                amount: 3,
            },
            current: 5,
            reason: RefusalReason::Limit { limit: 6 },
        };
        assert_eq!(report.refusals().collect::<Vec<_>>(), [expected_refusal]);
        assert_eq!(
            report.events().collect::<Vec<_>>(),
            [
                PaintEvent::Accepted {
                    phase: PaintPhase::Validate,
                    change: PaintChange::Work {
                        kind: WorkKind::BoxValidation,
                        amount: 4
                    },
                    elapsed,
                },
                PaintEvent::Refused {
                    refusal: expected_refusal,
                    elapsed
                },
                PaintEvent::PhaseCompleted {
                    phase: PaintPhase::Validate,
                    duration,
                    elapsed
                },
            ]
        );
        assert_eq!(report.events().len(), 3);
        assert_eq!(report.elapsed(), elapsed);
        assert_eq!(report.dropped_events(), 13);
        assert!(report.saturated());
    }

    #[test]
    fn failed_child_creation_preserves_a_report_of_partial_setup() {
        let mut meter = PaintOptions::new(
            PaintLimits::unlimited(),
            ProfileMode::Summary { timing: false },
        )
        .operation(OperationKind::Stacking)
        .unwrap();
        meter
            .child(
                meter.root(),
                PaintPhase::Validate,
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .unwrap();
        let foreign =
            Operation::<WorkKind, ResourceKind>::new("foreign", Limits::unlimited(), 1).unwrap();
        let error = meter
            .child(
                foreign.root(),
                PaintPhase::Topology,
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .unwrap_err();
        assert_eq!(
            error,
            PaintError::Accounting(AccountingFailure::ForeignScope)
        );
        let (result, report) = finish(meter, Err::<(), _>(error));
        assert_eq!(
            result,
            Err(PaintError::Accounting(AccountingFailure::ForeignScope))
        );
        let report = report.unwrap();
        assert_eq!(
            report.phases().map(PhaseReport::phase).collect::<Vec<_>>(),
            [PaintPhase::Operation, PaintPhase::Validate]
        );
        for &kind in WorkKind::ALL {
            assert_eq!(report.work(kind), 0);
        }
    }

    #[test]
    fn live_refusals_use_registered_origin_and_limiter_despite_display_labels() {
        for local_limit in [false, true] {
            let limits = PaintLimits::unlimited().with_work_limit(WorkKind::BoxValidation, 0);
            let root_limits = if local_limit {
                Limits::unlimited()
            } else {
                Limits::new(limits.work, limits.resources)
            };
            let child_limits = if local_limit {
                Limits::new(limits.work, limits.resources)
            } else {
                Limits::unlimited()
            };
            let mut operation =
                Operation::with_profile("gradient_geometry", root_limits, 2, detailed(2)).unwrap();
            let root = operation.root();
            let validate = operation
                .child(root, "cleanup", child_limits, Tariffs::identity())
                .unwrap();
            let mut bindings = ScopeBindings::new(root);
            *bindings.children.first_mut().unwrap() = Some((validate, PaintPhase::Validate));
            let mut meter = Meter {
                operation,
                bindings,
                backing: ReportBacking::new(2, detailed(2)).unwrap(),
            };
            let error = meter
                .with_phase(validate, |phase| {
                    charge(phase, validate, WorkKind::BoxValidation, 1)
                })
                .unwrap()
                .unwrap_err();
            let expected = BudgetRefusal {
                origin: PaintPhase::Validate,
                limiting_phase: if local_limit {
                    PaintPhase::Validate
                } else {
                    PaintPhase::Operation
                },
                item: 0,
                change: PaintChange::Work {
                    kind: WorkKind::BoxValidation,
                    amount: 1,
                },
                current: 0,
                reason: RefusalReason::Limit { limit: 0 },
            };
            assert_eq!(error, PaintError::Budget(expected));
            let report = meter.finish().unwrap().unwrap();
            assert_eq!(report.refusals().collect::<Vec<_>>(), [expected]);
            assert_eq!(
                report.events().next(),
                Some(PaintEvent::Refused {
                    refusal: expected,
                    elapsed: None
                })
            );
        }
    }

    #[test]
    fn unregistered_refusal_origin_fails_without_admitting_work() {
        let limits = PaintLimits::unlimited().with_work_limit(WorkKind::BoxValidation, 0);
        let mut meter = PaintOptions::new(limits, ProfileMode::Summary { timing: false })
            .operation(OperationKind::Stacking)
            .unwrap();
        let root = meter.root();
        let unregistered = meter
            .operation
            .child(root, "validate", Limits::unlimited(), Tariffs::identity())
            .unwrap();
        let error = meter
            .with_phase(root, |phase| {
                charge(phase, unregistered, WorkKind::BoxValidation, 1)
            })
            .unwrap()
            .unwrap_err();
        assert_eq!(error, invariant());
        assert_eq!(
            meter
                .operation
                .usage(root)
                .unwrap()
                .work(WorkKind::BoxValidation),
            0
        );
    }

    #[test]
    fn impossible_observer_reservations_are_typed_allocation_failures() {
        for (scope_limit, profile) in [
            (usize::MAX, ProfileMode::Summary { timing: false }),
            (8, detailed(usize::MAX)),
        ] {
            assert_eq!(
                ReportBacking::new(scope_limit, profile).unwrap_err(),
                PaintError::Accounting(AccountingFailure::Allocation)
            );
        }
        assert!(
            ReportBacking::new(usize::MAX, ProfileMode::Off)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            PaintOptions::new(PaintLimits::unlimited(), detailed(usize::MAX))
                .operation(OperationKind::Stacking),
            Err(PaintError::Accounting(AccountingFailure::Allocation))
        ));
    }

    #[test]
    fn logical_observer_bounds_and_actual_reserved_capacity_are_both_required() {
        enum Bound {
            ScopeLimit,
            ScopeCapacity,
            EventLimit,
            EventCapacity,
        }
        for bound in [
            Bound::ScopeLimit,
            Bound::ScopeCapacity,
            Bound::EventLimit,
            Bound::EventCapacity,
        ] {
            let (mut raw, validate, bindings, mut backing) =
                report_with_validate_scope_profile(detailed(1));
            raw.events.push(surgeist_ledger::Event::Accepted {
                scope: validate,
                change: Change::Work(WorkKind::BoxValidation, 1),
                elapsed: None,
            });
            match bound {
                Bound::ScopeLimit => backing.scope_limit = 1,
                Bound::ScopeCapacity => backing.scopes = Vec::new(),
                Bound::EventLimit => backing.event_limit = 0,
                Bound::EventCapacity => backing.events = Vec::new(),
            }
            assert_eq!(PaintReport::new(raw, &bindings, backing), Err(invariant()));
        }
    }

    #[test]
    fn foreign_live_origin_preserves_the_ledger_error_kind() {
        let mut meter = PaintOptions::new(
            PaintLimits::unlimited(),
            ProfileMode::Summary { timing: false },
        )
        .operation(OperationKind::Stacking)
        .unwrap();
        let root = meter.root();
        let foreign =
            Operation::<WorkKind, ResourceKind>::new("foreign", Limits::unlimited(), 1).unwrap();
        let error = meter
            .with_phase(root, |phase| {
                charge(phase, foreign.root(), WorkKind::BoxValidation, 1)
            })
            .unwrap()
            .unwrap_err();
        assert_eq!(
            error,
            PaintError::Accounting(AccountingFailure::ForeignScope)
        );
        assert_eq!(
            meter
                .operation
                .usage(root)
                .unwrap()
                .work(WorkKind::BoxValidation),
            0
        );
    }
}
