//! Bounded observer adapters. Their backing is separate from algorithm custody,
//! like Ledger's observer: reserve before work, never grow during publication.
use super::*;
use std::{fmt, time::Duration};
use surgeist_ledger::{
    Event, IntervalReport, IntervalUsage, ProfileMode, Refusal, Report, ScopeReport, Usage,
};

pub(super) type IntervalNumbers = IntervalUsage<RenderWork, RenderResource>;
pub(super) type LifetimeNumbers = Usage<RenderWork, RenderResource>;
type Diagnostic = std::result::Result<(), RenderAccountingError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ObserverBudget {
    observed: bool,
    events: usize,
}
impl ObserverBudget {
    pub(super) fn new(profile: ProfileMode) -> Self {
        match profile {
            ProfileMode::Off => Self {
                observed: false,
                events: 0,
            },
            ProfileMode::Summary { .. } => Self {
                observed: true,
                events: 0,
            },
            ProfileMode::Detailed { event_limit, .. } => Self {
                observed: true,
                events: event_limit,
            },
        }
    }
}

pub(super) trait ScopeKey: Copy + Eq + fmt::Debug + 'static {
    type Event: Copy + Eq + fmt::Debug;
    fn all() -> &'static [Self];
    fn index(self) -> usize;
    fn parent(self) -> Option<Self>;
    fn event(event: ResolvedEvent<Self>) -> Self::Event;
}

pub(super) enum ResolvedEvent<K> {
    Change {
        scope: K,
        quantity: RenderQuantity,
        amount: u64,
        kind: RenderChangeKind,
        elapsed: Option<Duration>,
    },
    Phase {
        scope: K,
        duration: Option<Duration>,
        elapsed: Option<Duration>,
    },
    Refused {
        scope: K,
        elapsed: Option<Duration>,
    },
}

macro_rules! events_for {
    ($key:ty, $event:ident, $field:ident) => {
        fn event(event: ResolvedEvent<Self>) -> Self::Event {
            match event {
                ResolvedEvent::Change {
                    scope,
                    quantity,
                    amount,
                    kind,
                    elapsed,
                } => $event::Change {
                    $field: scope,
                    quantity,
                    amount,
                    kind,
                    elapsed,
                },
                ResolvedEvent::Phase {
                    scope,
                    duration,
                    elapsed,
                } => $event::Phase {
                    $field: scope,
                    duration,
                    elapsed,
                },
                ResolvedEvent::Refused { scope, elapsed } => $event::Refused {
                    $field: scope,
                    elapsed,
                },
            }
        }
    };
}
impl ScopeKey for RenderPhase {
    type Event = RenderEvent;
    fn all() -> &'static [Self] {
        &Self::ALL
    }
    fn index(self) -> usize {
        self.index()
    }
    fn parent(self) -> Option<Self> {
        self.parent()
    }
    events_for!(RenderPhase, RenderEvent, phase);
}
impl ScopeKey for RenderCpuScope {
    type Event = RenderCpuEvent;
    fn all() -> &'static [Self] {
        &Self::ALL
    }
    fn index(self) -> usize {
        self.index()
    }
    fn parent(self) -> Option<Self> {
        if self == Self::Root {
            None
        } else {
            Some(Self::Root)
        }
    }
    events_for!(RenderCpuScope, RenderCpuEvent, scope);
}
impl ScopeKey for RenderGenerationScope {
    type Event = RenderGenerationEvent;
    fn all() -> &'static [Self] {
        &Self::ALL
    }
    fn index(self) -> usize {
        self.index()
    }
    fn parent(self) -> Option<Self> {
        self.parent()
    }
    events_for!(RenderGenerationScope, RenderGenerationEvent, scope);
}

struct Bindings<'a, K: 'static> {
    handles: &'a [Scope],
    root: Scope,
    keys: &'static [K],
}
impl<'a, K: ScopeKey> Bindings<'a, K> {
    fn new(handles: &'a [Scope]) -> std::result::Result<Self, RenderAccountingError> {
        let keys = K::all();
        let root = handles
            .first()
            .copied()
            .ok_or(RenderAccountingError::Invariant)?;
        if handles.len() != keys.len()
            || keys
                .first()
                .is_none_or(|key| key.index() != 0 || key.parent().is_some())
        {
            return Err(RenderAccountingError::Invariant);
        }
        for (index, handle) in handles.iter().enumerate() {
            if handles
                .iter()
                .take(index)
                .any(|previous| previous == handle)
            {
                return Err(RenderAccountingError::Invariant);
            }
        }
        for (index, key) in keys.iter().enumerate() {
            if key.index() >= keys.len()
                || keys
                    .iter()
                    .take(index)
                    .any(|previous| previous.index() == key.index())
            {
                return Err(RenderAccountingError::Invariant);
            }
        }
        Ok(Self {
            handles,
            root,
            keys,
        })
    }
    fn resolve(&self, scope: Scope) -> std::result::Result<K, RenderAccountingError> {
        self.keys
            .iter()
            .copied()
            .find(|key| self.handles.get(key.index()) == Some(&scope))
            .ok_or(RenderAccountingError::Invariant)
    }
    fn row(&self, key: K, scope: Scope, parent: Option<Scope>) -> Diagnostic {
        let expected = self
            .handles
            .get(key.index())
            .ok_or(RenderAccountingError::Invariant)?;
        let expected_parent = key
            .parent()
            .map(|parent| {
                self.handles
                    .get(parent.index())
                    .copied()
                    .ok_or(RenderAccountingError::Invariant)
            })
            .transpose()?;
        if scope != *expected || parent != expected_parent {
            return Err(RenderAccountingError::Invariant);
        }
        Ok(())
    }
    fn refusal(
        &self,
        refusal: Refusal<RenderWork, RenderResource>,
    ) -> std::result::Result<ResolvedRefusal<K>, RenderAccountingError> {
        Ok(ResolvedRefusal {
            scope: self.resolve(refusal.scope)?,
            item: refusal.item,
            change: refusal.change,
            current: refusal.current,
            reason: refusal.reason,
        })
    }
    fn event(
        &self,
        event: Event<RenderWork, RenderResource>,
    ) -> std::result::Result<K::Event, RenderAccountingError> {
        let event = match event {
            Event::Accepted {
                scope,
                change,
                elapsed,
            } => {
                let (quantity, amount, kind) = match change {
                    Change::Work(work, amount) => {
                        (RenderQuantity::Work(work), amount, RenderChangeKind::Work)
                    }
                    Change::Acquire(resource, amount) => (
                        RenderQuantity::Resource(resource),
                        amount,
                        RenderChangeKind::Acquire,
                    ),
                    Change::Release(resource, amount) => (
                        RenderQuantity::Resource(resource),
                        amount,
                        RenderChangeKind::Release,
                    ),
                };
                ResolvedEvent::Change {
                    scope: self.resolve(scope)?,
                    quantity,
                    amount,
                    kind,
                    elapsed,
                }
            }
            Event::Phase {
                scope,
                duration,
                elapsed,
            } => ResolvedEvent::Phase {
                scope: self.resolve(scope)?,
                duration,
                elapsed,
            },
            Event::Refused {
                origin,
                refusal,
                elapsed,
            } => {
                let scope = self.resolve(origin)?;
                self.refusal(refusal)?;
                ResolvedEvent::Refused { scope, elapsed }
            }
        };
        Ok(K::event(event))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResolvedRefusal<K> {
    scope: K,
    item: usize,
    change: Change<RenderWork, RenderResource>,
    current: u64,
    reason: surgeist_ledger::RefusalReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Summary<K, U> {
    key: K,
    pub(super) usage: U,
    parent: Option<K>,
    accepted_batches: u64,
    accepted_items: u64,
    refused_batches: u64,
    first_refusal: Option<ResolvedRefusal<K>>,
    pub(super) phases: u64,
    pub(super) phase_elapsed: Option<Duration>,
}
impl<K: ScopeKey, U> Summary<K, U> {
    fn numeric(key: K, usage: U) -> Self {
        Self {
            key,
            usage,
            parent: key.parent(),
            accepted_batches: 0,
            accepted_items: 0,
            refused_batches: 0,
            first_refusal: None,
            phases: 0,
            phase_elapsed: None,
        }
    }
    fn observe(
        &mut self,
        row: ScopeReport<RenderWork, RenderResource>,
        bindings: &Bindings<'_, K>,
    ) -> Diagnostic {
        bindings.row(self.key, row.scope, row.parent)?;
        self.accepted_batches = row.accepted_batches;
        self.accepted_items = row.accepted_items;
        self.refused_batches = row.refused_batches;
        self.first_refusal = row
            .first_refusal
            .map(|refusal| bindings.refusal(refusal))
            .transpose()?;
        self.phases = row.phases;
        self.phase_elapsed = row.phase_elapsed;
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct ReportBacking<K: ScopeKey, U> {
    rows: Vec<Summary<K, U>>,
    events: Vec<K::Event>,
    row_limit: usize,
    budget: ObserverBudget,
}
impl<K: ScopeKey, U> ReportBacking<K, U> {
    pub(super) fn interval(
        budget: ObserverBudget,
    ) -> std::result::Result<Self, RenderAccountingError> {
        Self::reserve(K::all().len(), budget)
    }
    pub(super) fn snapshot(
        budget: ObserverBudget,
    ) -> std::result::Result<Self, RenderAccountingError> {
        Self::reserve(if budget.observed { K::all().len() } else { 0 }, budget)
    }
    fn reserve(
        row_limit: usize,
        budget: ObserverBudget,
    ) -> std::result::Result<Self, RenderAccountingError> {
        let mut rows = Vec::new();
        let mut events = Vec::new();
        rows.try_reserve_exact(row_limit)
            .map_err(|_| RenderAccountingError::Setup(RenderAccountingSetupError::Allocation))?;
        events
            .try_reserve_exact(budget.events)
            .map_err(|_| RenderAccountingError::Setup(RenderAccountingSetupError::Allocation))?;
        Ok(Self {
            rows,
            events,
            row_limit,
            budget,
        })
    }
    fn empty(&self, row_limit: usize, observed: bool) -> Diagnostic {
        if !self.rows.is_empty()
            || !self.events.is_empty()
            || self.row_limit != row_limit
            || self.budget.observed != observed
        {
            return Err(RenderAccountingError::Invariant);
        }
        Ok(())
    }
    fn finish(
        self,
        elapsed: Option<Duration>,
        dropped_events: u64,
        saturated: bool,
    ) -> ResolvedReport<K, U> {
        ResolvedReport {
            rows: self.rows,
            events: self.events,
            observed: self.budget.observed,
            elapsed,
            dropped_events,
            saturated,
        }
    }
}
fn push_reserved<T>(values: &mut Vec<T>, value: T, limit: usize) -> Diagnostic {
    if values.len() >= limit || values.len() >= values.capacity() {
        return Err(RenderAccountingError::Invariant);
    }
    values.push(value);
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct ResolvedReport<K: ScopeKey, U> {
    pub(super) rows: Vec<Summary<K, U>>,
    pub(super) events: Vec<K::Event>,
    pub(super) observed: bool,
    pub(super) elapsed: Option<Duration>,
    pub(super) dropped_events: u64,
    pub(super) saturated: bool,
}
impl<K: ScopeKey> ResolvedReport<K, IntervalNumbers> {
    pub(super) fn interval(
        raw: IntervalReport<RenderWork, RenderResource>,
        handles: &[Scope],
        mut backing: ReportBacking<K, IntervalNumbers>,
    ) -> std::result::Result<(Scope, Self), RenderAccountingError> {
        let bindings = Bindings::<K>::new(handles)?;
        backing.empty(bindings.keys.len(), raw.profile.is_some())?;
        if raw.scopes.len() != bindings.keys.len() {
            return Err(RenderAccountingError::Invariant);
        }
        for (row, &key) in raw.scopes.into_iter().zip(bindings.keys) {
            bindings.row(key, row.scope, row.parent)?;
            push_reserved(
                &mut backing.rows,
                Summary::numeric(key, row.usage),
                backing.row_limit,
            )?;
        }
        let mut elapsed = None;
        let mut dropped_events = 0;
        let mut saturated = false;
        if let Some(profile) = raw.profile {
            if profile.scopes.len() != bindings.keys.len() {
                return Err(RenderAccountingError::Invariant);
            }
            for (row, summary) in profile.scopes.into_iter().zip(&mut backing.rows) {
                summary.observe(row, &bindings)?;
            }
            for event in profile.events {
                push_reserved(
                    &mut backing.events,
                    bindings.event(event)?,
                    backing.budget.events,
                )?;
            }
            elapsed = profile.elapsed;
            dropped_events = profile.dropped_events;
            saturated = profile.saturated;
        }
        backing.rows.sort_unstable_by_key(|row| row.key.index());
        Ok((
            bindings.root,
            backing.finish(elapsed, dropped_events, saturated),
        ))
    }
}
impl<K: ScopeKey> ResolvedReport<K, LifetimeNumbers> {
    pub(super) fn snapshot(
        raw: Option<Report<RenderWork, RenderResource>>,
        handles: &[Scope],
        mut backing: ReportBacking<K, LifetimeNumbers>,
    ) -> std::result::Result<(Scope, Self), RenderAccountingError> {
        let bindings = Bindings::<K>::new(handles)?;
        backing.empty(
            if raw.is_some() {
                bindings.keys.len()
            } else {
                0
            },
            raw.is_some(),
        )?;
        let Some(profile) = raw else {
            return Ok((bindings.root, backing.finish(None, 0, false)));
        };
        if profile.scopes.len() != bindings.keys.len() {
            return Err(RenderAccountingError::Invariant);
        }
        for (row, &key) in profile.scopes.into_iter().zip(bindings.keys) {
            let mut summary = Summary::numeric(key, row.usage);
            summary.observe(row, &bindings)?;
            push_reserved(&mut backing.rows, summary, backing.row_limit)?;
        }
        for event in profile.events {
            push_reserved(
                &mut backing.events,
                bindings.event(event)?,
                backing.budget.events,
            )?;
        }
        backing.rows.sort_unstable_by_key(|row| row.key.index());
        Ok((
            bindings.root,
            backing.finish(profile.elapsed, profile.dropped_events, profile.saturated),
        ))
    }
}
impl<K: ScopeKey, U: Copy> ResolvedReport<K, U> {
    #[cfg(test)]
    pub(super) fn snapshot_for_test(&self) -> Self {
        Self {
            rows: self.rows.clone(),
            events: self.events.clone(),
            observed: self.observed,
            elapsed: self.elapsed,
            dropped_events: self.dropped_events,
            saturated: self.saturated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_reservations_fail_with_typed_allocation_errors() {
        let detailed = ObserverBudget {
            observed: true,
            events: usize::MAX,
        };
        assert!(matches!(
            ReportBacking::<RenderPhase, IntervalNumbers>::interval(detailed),
            Err(RenderAccountingError::Setup(
                RenderAccountingSetupError::Allocation
            ))
        ));
        assert!(matches!(
            ReportBacking::<RenderPhase, IntervalNumbers>::reserve(
                usize::MAX,
                ObserverBudget::new(ProfileMode::Off)
            ),
            Err(RenderAccountingError::Setup(
                RenderAccountingSetupError::Allocation
            ))
        ));
    }

    #[test]
    fn off_snapshot_backing_has_no_role_or_event_allocation() {
        let backing = ReportBacking::<RenderCpuScope, LifetimeNumbers>::snapshot(
            ObserverBudget::new(ProfileMode::Off),
        )
        .unwrap();
        assert_eq!(backing.rows.capacity(), 0);
        assert_eq!(backing.events.capacity(), 0);
        let interval = ReportBacking::<RenderCpuScope, IntervalNumbers>::interval(
            ObserverBudget::new(ProfileMode::Off),
        )
        .unwrap();
        assert!(interval.rows.capacity() >= RenderCpuScope::ALL.len());
        assert_eq!(interval.events.capacity(), 0);
    }

    #[test]
    fn logical_bounds_and_real_capacity_both_guard_projection() {
        for mutation in 0..4 {
            let options = RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
                timing: false,
                event_limit: 8,
                filter: RenderPhaseFilter::All,
            });
            let (account, checkpoint, mut backing) = build_account(
                options,
                RenderAttemptKind::Frame,
                GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
                #[cfg(feature = "render-text")]
                None,
                None,
            )
            .unwrap();
            account
                .at(RenderPhase::Kernel)
                .work(RenderWork::CoordinateVisits, 1)
                .unwrap();
            let interval = account.ledger.end_checkpoint(&checkpoint).unwrap();
            match mutation {
                0 => backing.row_limit = 0,
                1 => backing.rows = Vec::new(),
                2 => backing.budget.events = 0,
                3 => backing.events = Vec::new(),
                _ => unreachable!(),
            }
            match ResolvedReport::interval(interval, account.scopes.as_ref(), backing) {
                Err(error) => assert_eq!(error, RenderAccountingError::Invariant),
                Ok(_) => panic!("projection exceeded its logical or actual backing bound"),
            }
        }
    }

    #[test]
    fn renderer_intervals_use_their_owner_profile_budgets() {
        let cpu = RendererCpu::new(RenderAccountingOptions::new().with_profile(
            RenderProfile::Detailed {
                timing: false,
                event_limit: 16,
                filter: RenderPhaseFilter::All,
            },
        ))
        .unwrap();
        let mut attempt = FrameAttempt::begin(
            FrameReportSlot::default(),
            &cpu,
            RenderAccountingOptions::default(),
        )
        .unwrap();
        let generation = Generation::new(RenderAccountingOptions::new().with_profile(
            RenderProfile::Detailed {
                timing: false,
                event_limit: 16,
                filter: RenderPhaseFilter::All,
            },
        ))
        .unwrap();
        attempt.attach_generation(&generation).unwrap();
        let native = generation
            .allocation(GenerationClass::CacheVello, 3, None, || ())
            .unwrap();
        cpu.at(RenderCpuScope::FrameBacking)
            .checked_work(RenderWork::CoordinateVisits, 1)
            .unwrap();
        let (report, fault) = attempt.finish(RenderAttemptStatus::Succeeded, None);
        assert_eq!(fault, None);
        assert_eq!(report.attempt().unwrap().events().count(), 0);
        assert!(report.renderer_cpu().unwrap().events().next().is_some());
        assert!(
            report
                .generation()
                .unwrap()
                .unwrap()
                .events()
                .next()
                .is_some()
        );
        drop(native);
    }
}
