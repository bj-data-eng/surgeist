#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
use std::time::Instant;
use std::{fmt, time::Duration};

use crate::{Change, Error, Refusal, Scope, Usage};

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
pub struct ScopeReport<const W: usize, const R: usize> {
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

impl<const W: usize, const R: usize> ScopeReport<W, R> {
    fn new(scope: Scope, parent: Option<Scope>, name: &'static str, timing: bool) -> Self {
        Self {
            scope,
            parent,
            name,
            usage: Usage::ZERO,
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
pub enum Event<const W: usize, const R: usize> {
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
pub struct Report<const W: usize, const R: usize> {
    pub elapsed: Option<Duration>,
    pub scopes: Vec<ScopeReport<W, R>>,
    pub events: Vec<Event<W, R>>,
    pub dropped_events: u64,
    /// Some observational values exceeded their representable bound and were clamped.
    /// Ledger usage retains its normal checked arithmetic.
    pub saturated: bool,
}

impl<const W: usize, const R: usize> Report<W, R> {
    pub const fn terminal(&self) -> TerminalReport<'_, W, R> {
        TerminalReport(self)
    }
}

/// Display adapter for a structured report.
pub struct TerminalReport<'a, const W: usize, const R: usize>(&'a Report<W, R>);

impl<const W: usize, const R: usize> fmt::Display for TerminalReport<'_, W, R> {
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

pub(super) struct Profile<const W: usize, const R: usize> {
    clock: Option<Clock>,
    detail: Option<(usize, EventFilter)>,
    report: Report<W, R>,
}

impl<const W: usize, const R: usize> Profile<W, R> {
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

    pub(super) fn add_scope(&mut self, scope: Scope, parent: Option<Scope>, name: &'static str) {
        self.report
            .scopes
            .push(ScopeReport::new(scope, parent, name, self.clock.is_some()));
    }

    pub(super) fn admission(
        &mut self,
        origin: Scope,
        changes: &[Change<W, R>],
        result: Result<(), Error<W, R>>,
    ) {
        let summary = &mut self.report.scopes[origin.index];
        let saturated = &mut self.report.saturated;
        match result {
            Ok(()) => {
                count(&mut summary.accepted_batches, 1, saturated);
                count(&mut summary.accepted_items, changes.len(), saturated);
                let keep = self.retain(origin, changes.len());
                if keep != 0 {
                    let elapsed = self.elapsed();
                    self.report
                        .events
                        .extend(changes[..keep].iter().map(|&change| Event::Accepted {
                            scope: origin,
                            change,
                            elapsed,
                        }));
                }
            }
            Err(Error::Refused(refusal)) => {
                count(&mut summary.refused_batches, 1, saturated);
                summary.first_refusal.get_or_insert(refusal);
                if self.retain(origin, 1) != 0 {
                    self.report.events.push(Event::Refused {
                        origin,
                        refusal,
                        elapsed: self.elapsed(),
                    });
                }
            }
            Err(_) => {}
        }
    }

    pub(super) fn elapsed(&self) -> Option<Duration> {
        self.clock.as_ref().map(Clock::elapsed)
    }

    pub(super) fn end_phase(&mut self, scope: Scope, start: Option<Duration>) {
        let end = self.elapsed();
        let duration = start.zip(end).map(|(start, end)| end.saturating_sub(start));
        let summary = &mut self.report.scopes[scope.index];
        count(&mut summary.phases, 1, &mut self.report.saturated);
        if let (Some(sum), Some(duration)) = (&mut summary.phase_elapsed, duration) {
            if let Some(total) = sum.checked_add(duration) {
                *sum = total;
            } else {
                *sum = Duration::MAX;
                self.report.saturated = true;
            }
        }
        if self.retain(scope, 1) != 0 {
            self.report.events.push(Event::Phase {
                scope,
                duration,
                elapsed: end,
            });
        }
    }

    fn retain(&mut self, scope: Scope, count_events: usize) -> usize {
        let Some((limit, filter)) = self.detail else {
            return 0;
        };
        if let EventFilter::ScopeName(name) = filter
            && self.report.scopes[scope.index].name != name
        {
            return 0;
        }
        let keep = count_events.min(limit - self.report.events.len());
        count(
            &mut self.report.dropped_events,
            count_events - keep,
            &mut self.report.saturated,
        );
        keep
    }

    pub(super) fn finish(mut self, usage: impl Iterator<Item = Usage<W, R>>) -> Report<W, R> {
        self.report.elapsed = self.elapsed();
        for (scope, usage) in self.report.scopes.iter_mut().zip(usage) {
            scope.usage = usage;
        }
        self.report
    }
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

    use crate::{Limits, Operation, Tariffs, Work};

    use super::*;

    thread_local! {
        static TICKS: Cell<u64> = const { Cell::new(0) };
    }

    fn clock() -> Duration {
        let tick = TICKS.get();
        TICKS.set(tick + 1);
        Duration::from_millis(tick)
    }

    fn operation(mode: ProfileMode) -> Operation<1, 0> {
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
                        .apply(root, Change::Work(Work::new(0).unwrap(), 1))
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
        let work = Work::new(0).unwrap();
        let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = ledger.with_phase(root, |phase| {
                phase.apply(root, Change::Work(work, 3)).unwrap();
                panic!("stop in phase");
            });
        }));
        assert!(stopped.is_err());
        let report = ledger.finish().unwrap();
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
        let charge = Change::Work(Work::new(0).unwrap(), 1);
        ledger
            .with_phase(root, |ledger| {
                ledger
                    .with_phase(child, |ledger| {
                        ledger.apply_batch(child, &[charge, charge]).unwrap();
                    })
                    .unwrap();
            })
            .unwrap();

        let report = ledger.finish().unwrap();
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
}
