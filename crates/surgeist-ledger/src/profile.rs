#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use std::time::Instant;
use std::{fmt, time::Duration};

use crate::{Change, Defect, Dimension, Error, Refusal, Scope, Usage, push_reserved};

/// Selects which origins contribute detailed events. Summary accounting
/// includes every scope regardless of this filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventFilter {
    All,
    ScopeName(&'static str),
}

/// Optional observation for one operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProfileMode {
    #[default]
    Off,
    Summary {
        timing: bool,
    },
    Detailed {
        timing: bool,
        event_limit: usize,
        filter: EventFilter,
    },
}

/// Direct admission and phase observations, with inclusive ledger usage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeReport<W: Dimension, R: Dimension> {
    pub scope: Scope,
    pub parent: Option<Scope>,
    pub name: &'static str,
    pub usage: Usage<W, R>,
    pub accepted_batches: u64,
    pub accepted_items: u64,
    pub refused_batches: u64,
    pub first_refusal: Option<Refusal<W, R>>,
    pub phases: u64,
    /// Sum of inclusive phase durations at this scope. Nested invocations
    /// contribute individually, so this can exceed the operation elapsed time.
    pub phase_elapsed: Option<Duration>,
}

impl<W: Dimension, R: Dimension> ScopeReport<W, R> {
    fn new(scope: Scope, parent: Option<Scope>, name: &'static str, timing: bool) -> Self {
        Self {
            scope,
            parent,
            name,
            usage: Usage::zero(),
            accepted_batches: 0,
            accepted_items: 0,
            refused_batches: 0,
            first_refusal: None,
            phases: 0,
            phase_elapsed: timing.then_some(Duration::ZERO),
        }
    }
}

/// A retained observation. Timestamps are elapsed time since operation setup.
/// Accepted items in the same atomic batch share a timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event<W: Dimension, R: Dimension> {
    Accepted {
        scope: Scope,
        change: Change<W, R>,
        elapsed: Option<Duration>,
    },
    Refused {
        origin: Scope,
        refusal: Refusal<W, R>,
        elapsed: Option<Duration>,
    },
    Phase {
        scope: Scope,
        duration: Option<Duration>,
        elapsed: Option<Duration>,
    },
}

/// Structured report for an operation. The first scope holds the root totals.
/// Child usage is attribution and should be read in that child's units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report<W: Dimension, R: Dimension> {
    pub elapsed: Option<Duration>,
    pub scopes: Vec<ScopeReport<W, R>>,
    pub events: Vec<Event<W, R>>,
    pub dropped_events: u64,
    /// Some observational values exceeded their representable bound and were clamped.
    /// Ledger usage retains its normal checked arithmetic.
    pub saturated: bool,
}

impl<W: Dimension, R: Dimension> Report<W, R> {
    pub const fn terminal(&self) -> TerminalReport<'_, W, R> {
        TerminalReport(self)
    }
}

/// Display adapter for a structured report.
pub struct TerminalReport<'a, W: Dimension, R: Dimension>(&'a Report<W, R>);

impl<W: Dimension, R: Dimension> fmt::Display for TerminalReport<'_, W, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let report = self.0;
        if let Some(elapsed) = report.elapsed {
            writeln!(f, "Operation elapsed {elapsed:?}")?;
        }
        for scope in &report.scopes {
            write!(f, "[{}] {:?}", scope.scope.index, scope.name)?;
            if let Some(parent) = scope.parent {
                write!(f, " (parent {})", parent.index)?;
            }
            writeln!(
                f,
                " work={:?} live={:?} peak={:?} accepted={} items={} refused={} phases={}",
                scope.usage.work,
                scope.usage.live,
                scope.usage.peak,
                scope.accepted_batches,
                scope.accepted_items,
                scope.refused_batches,
                scope.phases,
            )?;
            if let Some(elapsed) = scope.phase_elapsed {
                writeln!(f, "  inclusive phase elapsed {elapsed:?}")?;
            }
            if let Some(refusal) = scope.first_refusal {
                writeln!(
                    f,
                    "  first refusal at scope {} item {}: {:?}, current {}, {:?}",
                    refusal.scope.index,
                    refusal.item,
                    refusal.change,
                    refusal.current,
                    refusal.reason
                )?;
            }
        }
        for event in &report.events {
            match event {
                Event::Accepted {
                    scope,
                    change,
                    elapsed,
                } => {
                    writeln!(
                        f,
                        "  [{:?}] scope {} accepted {change:?}",
                        elapsed, scope.index
                    )?;
                }
                Event::Refused {
                    origin,
                    refusal,
                    elapsed,
                } => {
                    writeln!(
                        f,
                        "  [{:?}] scope {} refused at scope {} item {}: {:?}, current {}, {:?}",
                        elapsed,
                        origin.index,
                        refusal.scope.index,
                        refusal.item,
                        refusal.change,
                        refusal.current,
                        refusal.reason
                    )?;
                }
                Event::Phase {
                    scope,
                    duration,
                    elapsed,
                } => {
                    writeln!(
                        f,
                        "  [{:?}] scope {} phase {duration:?}",
                        elapsed, scope.index
                    )?;
                }
            }
        }
        if report.dropped_events != 0 {
            writeln!(
                f,
                "Trace incomplete: {} events dropped",
                report.dropped_events
            )?;
        }
        if report.saturated {
            writeln!(f, "Observation counters saturated")?;
        }
        Ok(())
    }
}

#[derive(Clone)]
enum Clock {
    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    System(Instant),
    Host {
        start: Duration,
        now: fn() -> Duration,
    },
}

impl Clock {
    fn new(now: Option<fn() -> Duration>) -> Option<Self> {
        if let Some(now) = now {
            return Some(Self::Host { start: now(), now });
        }
        #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
        {
            Some(Self::System(Instant::now()))
        }
        #[cfg(all(target_family = "wasm", target_os = "unknown"))]
        {
            None
        }
    }

    fn elapsed(&self) -> Duration {
        match self {
            #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
            Self::System(start) => start.elapsed(),
            Self::Host { start, now } => now().saturating_sub(*start),
        }
    }
}

pub(super) struct Profile<W: Dimension, R: Dimension> {
    clock: Option<Clock>,
    offset: Duration,
    detail: Option<(usize, EventFilter)>,
    report: Report<W, R>,
}

impl<W: Dimension, R: Dimension> Profile<W, R> {
    pub(super) fn new(
        mode: ProfileMode,
        scope_limit: usize,
        now: Option<fn() -> Duration>,
    ) -> Result<Option<Self>, Error<W, R>> {
        let (timing, detail) = match mode {
            ProfileMode::Off => return Ok(None),
            ProfileMode::Summary { timing } => (timing, None),
            ProfileMode::Detailed {
                timing,
                event_limit,
                filter,
            } => (timing, Some((event_limit, filter))),
        };
        let clock = if timing {
            Some(Clock::new(now).ok_or(Error::TimingUnavailable)?)
        } else {
            None
        };
        let mut scopes = Vec::new();
        let mut events = Vec::new();
        scopes
            .try_reserve_exact(scope_limit)
            .map_err(|_| Error::Allocation)?;
        if let Some((limit, _)) = detail {
            events
                .try_reserve_exact(limit)
                .map_err(|_| Error::Allocation)?;
        }
        Ok(Some(Self {
            clock,
            offset: Duration::ZERO,
            detail,
            report: Report {
                elapsed: None,
                scopes,
                events,
                dropped_events: 0,
                saturated: false,
            },
        }))
    }

    pub(super) fn checkpoint(&self, scope_limit: usize) -> Result<Self, Error<W, R>> {
        let mut scopes = Vec::new();
        let mut events = Vec::new();
        scopes
            .try_reserve_exact(scope_limit)
            .map_err(|_| Error::Allocation)?;
        if let Some((limit, _)) = self.detail {
            events
                .try_reserve_exact(limit)
                .map_err(|_| Error::Allocation)?;
        }
        Ok(Self {
            clock: self.clock.clone(),
            offset: self.clock.as_ref().map_or(Duration::ZERO, Clock::elapsed),
            detail: self.detail,
            report: Report {
                elapsed: None,
                scopes,
                events,
                dropped_events: 0,
                saturated: false,
            },
        })
    }

    pub(super) fn end_checkpoint_phase(
        &mut self,
        scope: Scope,
        start: Option<Duration>,
        end: Option<Duration>,
    ) -> Result<(), Defect> {
        self.end_phase(
            scope,
            start.map(|start| start.saturating_sub(self.offset)),
            end.map(|end| end.saturating_sub(self.offset)),
        )
    }

    pub(super) fn add_scope(
        &mut self,
        scope: Scope,
        parent: Option<Scope>,
        name: &'static str,
    ) -> Result<(), Defect> {
        let timing = self.clock.is_some();
        push_reserved(
            &mut self.report.scopes,
            ScopeReport::new(scope, parent, name, timing),
        )
    }

    pub(super) fn admission(
        &mut self,
        origin: Scope,
        changes: &[Change<W, R>],
        result: Result<(), Error<W, R>>,
    ) -> Result<(), Defect> {
        match result {
            Ok(()) => {
                let keep = self.retain(origin, changes.len())?;
                let summary = self
                    .report
                    .scopes
                    .get_mut(origin.index)
                    .ok_or(Defect::ReportScope)?;
                let saturated = &mut self.report.saturated;
                count(&mut summary.accepted_batches, 1, saturated);
                count(&mut summary.accepted_items, changes.len(), saturated);
                if keep != 0 {
                    let elapsed = self.elapsed();
                    for &change in changes.iter().take(keep) {
                        push_reserved(
                            &mut self.report.events,
                            Event::Accepted {
                                scope: origin,
                                change,
                                elapsed,
                            },
                        )?;
                    }
                }
            }
            Err(Error::Refused(refusal)) => {
                let keep = self.retain(origin, 1)?;
                let summary = self
                    .report
                    .scopes
                    .get_mut(origin.index)
                    .ok_or(Defect::ReportScope)?;
                count(&mut summary.refused_batches, 1, &mut self.report.saturated);
                summary.first_refusal.get_or_insert(refusal);
                if keep != 0 {
                    let elapsed = self.elapsed();
                    push_reserved(
                        &mut self.report.events,
                        Event::Refused {
                            origin,
                            refusal,
                            elapsed,
                        },
                    )?;
                }
            }
            Err(_) => {}
        }
        Ok(())
    }

    pub(super) fn elapsed(&self) -> Option<Duration> {
        self.clock
            .as_ref()
            .map(|clock| clock.elapsed().saturating_sub(self.offset))
    }

    pub(super) fn end_phase(
        &mut self,
        scope: Scope,
        start: Option<Duration>,
        end: Option<Duration>,
    ) -> Result<(), Defect> {
        let duration = start.zip(end).map(|(start, end)| end.saturating_sub(start));
        let keep = self.retain(scope, 1)?;
        let summary = self
            .report
            .scopes
            .get_mut(scope.index)
            .ok_or(Defect::ReportScope)?;
        count(&mut summary.phases, 1, &mut self.report.saturated);
        if let (Some(sum), Some(duration)) = (&mut summary.phase_elapsed, duration) {
            if let Some(total) = sum.checked_add(duration) {
                *sum = total;
            } else {
                *sum = Duration::MAX;
                self.report.saturated = true;
            }
        }
        if keep != 0 {
            push_reserved(
                &mut self.report.events,
                Event::Phase {
                    scope,
                    duration,
                    elapsed: end,
                },
            )?;
        }
        Ok(())
    }

    /// How many of `count_events` new events fit in the detail stream. The
    /// rest are counted as dropped. Filtered origins retain and drop nothing.
    fn retain(&mut self, scope: Scope, count_events: usize) -> Result<usize, Defect> {
        let Some((limit, filter)) = self.detail else {
            return Ok(0);
        };
        if let EventFilter::ScopeName(name) = filter {
            let origin = self
                .report
                .scopes
                .get(scope.index)
                .ok_or(Defect::ReportScope)?;
            if origin.name != name {
                return Ok(0);
            }
        }
        let room = limit
            .checked_sub(self.report.events.len())
            .ok_or(Defect::EventLimit)?;
        let keep = count_events.min(room);
        let dropped = count_events.checked_sub(keep).ok_or(Defect::EventLimit)?;
        count(
            &mut self.report.dropped_events,
            dropped,
            &mut self.report.saturated,
        );
        Ok(keep)
    }

    pub(super) fn finish(
        mut self,
        usage: impl Iterator<Item = Usage<W, R>>,
    ) -> Result<Report<W, R>, Defect> {
        self.report.elapsed = self.elapsed();
        fill_usage(&mut self.report.scopes, usage)?;
        Ok(self.report)
    }

    pub(super) fn snapshot(
        &self,
        usage: impl Iterator<Item = Usage<W, R>>,
    ) -> Result<Report<W, R>, Error<W, R>> {
        let mut scopes = Vec::new();
        let mut events = Vec::new();
        scopes
            .try_reserve_exact(self.report.scopes.len())
            .map_err(|_| Error::Allocation)?;
        events
            .try_reserve_exact(self.report.events.len())
            .map_err(|_| Error::Allocation)?;
        scopes.extend(self.report.scopes.iter().cloned());
        events.extend_from_slice(&self.report.events);
        fill_usage(&mut scopes, usage).map_err(Error::Defect)?;
        Ok(Report {
            elapsed: self.elapsed(),
            scopes,
            events,
            dropped_events: self.report.dropped_events,
            saturated: self.report.saturated,
        })
    }
}

/// Pairs each report record with its scope's usage. The operation creates both
/// in step, so their counts are equal.
fn fill_usage<W: Dimension, R: Dimension>(
    scopes: &mut [ScopeReport<W, R>],
    mut usage: impl Iterator<Item = Usage<W, R>>,
) -> Result<(), Defect> {
    for scope in scopes {
        scope.usage = usage.next().ok_or(Defect::ReportScope)?;
    }
    if usage.next().is_some() {
        return Err(Defect::ReportScope);
    }
    Ok(())
}

fn count(total: &mut u64, increment: usize, saturated: &mut bool) {
    if let Some(next) = u64::try_from(increment)
        .ok()
        .and_then(|amount| total.checked_add(amount))
    {
        *total = next;
    } else {
        *total = u64::MAX;
        *saturated = true;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::{Limits, NoDimension, Operation, Tariffs};

    use super::*;

    crate::dimensions! {
        enum TestWork => TestWorkTable { Visit }
    }

    thread_local! {
        static TICKS: Cell<u64> = const { Cell::new(0) };
    }

    fn clock() -> Duration {
        let tick = TICKS.get();
        TICKS.set(tick + 1);
        Duration::from_millis(tick)
    }

    fn operation(mode: ProfileMode) -> Operation<TestWork, NoDimension> {
        TICKS.set(0);
        Operation::with_profile_clock("root", Limits::unlimited(), 2, mode, clock).unwrap()
    }

    #[test]
    fn disabled_timing_does_not_read_the_clock() {
        for mode in [
            ProfileMode::Off,
            ProfileMode::Summary { timing: false },
            ProfileMode::Detailed {
                timing: false,
                event_limit: 2,
                filter: EventFilter::All,
            },
        ] {
            let mut ledger = operation(mode);
            let root = ledger.root();
            ledger
                .with_phase(root, |ledger| {
                    ledger
                        .apply(root, Change::Work(TestWork::Visit, 1))
                        .unwrap();
                })
                .unwrap();
            let _report = ledger.finish();
            assert_eq!(TICKS.get(), 0);
        }
    }

    #[test]
    fn unwinding_records_the_phase_and_preserves_accepted_work() {
        let mut ledger = operation(ProfileMode::Summary { timing: true });
        let root = ledger.root();
        let work = TestWork::Visit;
        let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = ledger.with_phase(root, |phase| {
                phase.apply(root, Change::Work(work, 3)).unwrap();
                panic!("stop in phase");
            });
        }));
        assert!(stopped.is_err());
        let report = ledger.finish().unwrap().unwrap();
        assert_eq!(report.scopes[0].usage.work(work), 3);
        assert_eq!(report.scopes[0].phases, 1);
        assert_eq!(
            report.scopes[0].phase_elapsed,
            Some(Duration::from_millis(1))
        );
    }

    #[test]
    fn nested_phases_have_inclusive_time_and_batches_share_a_timestamp() {
        let mut ledger = operation(ProfileMode::Detailed {
            timing: true,
            event_limit: 4,
            filter: EventFilter::All,
        });
        let root = ledger.root();
        let child = ledger
            .child(root, "child", Limits::unlimited(), Tariffs::identity())
            .unwrap();
        let charge = Change::Work(TestWork::Visit, 1);
        ledger
            .with_phase(root, |ledger| {
                ledger
                    .with_phase(child, |ledger| {
                        ledger.apply_batch(child, &[charge, charge]).unwrap();
                    })
                    .unwrap();
            })
            .unwrap();

        let report = ledger.finish().unwrap().unwrap();
        let ms = |n| Some(Duration::from_millis(n));
        assert_eq!(report.elapsed, ms(6));
        assert_eq!(report.scopes[0].phase_elapsed, ms(4));
        assert_eq!(report.scopes[1].phase_elapsed, ms(2));
        assert_eq!(
            report.events,
            [
                Event::Accepted {
                    scope: child,
                    change: charge,
                    elapsed: ms(3)
                },
                Event::Accepted {
                    scope: child,
                    change: charge,
                    elapsed: ms(3)
                },
                Event::Phase {
                    scope: child,
                    duration: ms(2),
                    elapsed: ms(4)
                },
                Event::Phase {
                    scope: root,
                    duration: ms(4),
                    elapsed: ms(5)
                },
            ]
        );
        assert_eq!(TICKS.get(), 7);
    }
    #[test]
    fn interval_observer_saturation_is_bounded_and_does_not_change_usage() {
        let mut operation = operation(ProfileMode::Detailed {
            timing: false,
            event_limit: 0,
            filter: EventFilter::All,
        });
        let root = operation.root();
        let mut interval = operation.profile[0].checkpoint(2).unwrap();
        interval.add_scope(root, None, "root").unwrap();
        interval.report.scopes[0].accepted_batches = u64::MAX;
        interval.report.scopes[0].accepted_items = u64::MAX;
        interval.report.scopes[0].phases = u64::MAX;
        interval.report.dropped_events = u64::MAX;
        let change = Change::Work(TestWork::Visit, 1);
        operation.apply(root, change).unwrap();
        interval.admission(root, &[change], Ok(())).unwrap();
        interval.end_checkpoint_phase(root, None, None).unwrap();
        let report = interval
            .finish([*operation.usage(root).unwrap()].into_iter())
            .unwrap();
        assert!(report.saturated);
        assert_eq!(report.dropped_events, u64::MAX);
        assert_eq!(report.scopes[0].accepted_batches, u64::MAX);
        assert_eq!(report.scopes[0].accepted_items, u64::MAX);
        assert_eq!(report.scopes[0].phases, u64::MAX);
        assert_eq!(report.scopes[0].usage.work(TestWork::Visit), 1);
        assert!(report.events.is_empty());
    }

    #[test]
    fn full_scope_record_buffer_latches_a_reserved_capacity_defect() {
        let mut operation = operation(ProfileMode::Summary { timing: false });
        let root = operation.root();
        // Fill the buffer reserved at setup, so the next scope has no room.
        let records = &mut operation.profile[0].report.scopes;
        let record = records[0].clone();
        records.resize(records.capacity(), record);
        assert_eq!(
            operation.child(root, "child", Limits::unlimited(), Tariffs::identity()),
            Err(Error::Defect(Defect::ReservedCapacity))
        );
        crate::tests::assert_latched(operation, Defect::ReservedCapacity);
    }

    #[test]
    fn full_event_buffer_latches_a_reserved_capacity_defect() {
        let mut operation = operation(ProfileMode::Detailed {
            timing: false,
            event_limit: 1,
            filter: EventFilter::All,
        });
        let root = operation.root();
        // Raise the limit past the reservation and fill the reservation, so
        // the limit promises room that the buffer does not have.
        let profile = &mut operation.profile[0];
        let event = Event::Phase {
            scope: root,
            duration: None,
            elapsed: None,
        };
        let capacity = profile.report.events.capacity();
        profile.report.events.resize(capacity, event);
        profile.detail = Some((capacity + 1, EventFilter::All));
        assert_eq!(
            operation.apply(root, Change::Work(TestWork::Visit, 1)),
            Err(Error::Defect(Defect::ReservedCapacity))
        );
        crate::tests::assert_latched(operation, Defect::ReservedCapacity);
    }

    #[test]
    fn retained_events_above_the_limit_latch_an_event_limit_defect() {
        let mut operation = operation(ProfileMode::Detailed {
            timing: false,
            event_limit: 1,
            filter: EventFilter::All,
        });
        let root = operation.root();
        let event = Event::Phase {
            scope: root,
            duration: None,
            elapsed: None,
        };
        operation.profile[0].report.events.extend([event, event]);
        assert_eq!(
            operation.apply(root, Change::Work(TestWork::Visit, 1)),
            Err(Error::Defect(Defect::EventLimit))
        );
        crate::tests::assert_latched(operation, Defect::EventLimit);
    }
}
