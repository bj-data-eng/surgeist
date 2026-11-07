//! Typed views of the renderer CPU owner's exact Ledger observations.
use super::{
    RESOURCE_COUNT, RenderAccountingError, RenderChangeKind, RenderQuantity, RenderResource,
    RenderWork, WORK_COUNT,
};
use std::{fmt, time::Duration};
use surgeist_ledger::{IntervalReport, IntervalUsage, Report, Scope, Usage};

/// A fixed CPU backing owner role. Child values are inclusive attribution and
/// must not be added to the root or one another to invent a physical peak.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderCpuScope {
    /// Complete renderer CPU lifetime.
    Root,
    /// Frame working, normalized, planned and encoded backing.
    FrameBacking,
    /// Persistent device shader and binding backing.
    DeviceBacking,
    /// Canonical history and publication draft record backing.
    ImageHistory,
    /// Canonical prepared runs and their directly owned allocations.
    PreparedValues,
    /// Producer-authoritative font resource payloads.
    FontPayloads,
    /// Shared image allocations and the import registry backing.
    ImagePayloads,
    /// Guarded explicit-readback output, retained until final caller handoff.
    ReadbackOutput,
    /// Mandatory disposal and rollback work.
    Cleanup,
}
impl RenderCpuScope {
    pub(crate) const ALL: [Self; 9] = [
        Self::Root,
        Self::FrameBacking,
        Self::DeviceBacking,
        Self::ImageHistory,
        Self::PreparedValues,
        Self::FontPayloads,
        Self::ImagePayloads,
        Self::ReadbackOutput,
        Self::Cleanup,
    ];
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
    /// Fixed owner-role name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Root => "render.cpu",
            Self::FrameBacking => "frame_backing",
            Self::DeviceBacking => "device_backing",
            Self::ImageHistory => "image_history",
            Self::PreparedValues => "prepared_values",
            Self::FontPayloads => "font_payloads",
            Self::ImagePayloads => "image_payloads",
            Self::ReadbackOutput => "readback_output",
            Self::Cleanup => "cleanup",
        }
    }
}

/// Identity of one renderer CPU owner, independent of device generations.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RenderCpuIdentity(pub(crate) Scope);
impl fmt::Debug for RenderCpuIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderCpuIdentity").finish_non_exhaustive()
    }
}

/// One owner's cumulative work, current live custody and lifetime maximum.
/// This has no opening balance and is not a sum of frame intervals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderLifetimeUsage(Usage<WORK_COUNT, RESOURCE_COUNT>);
impl RenderLifetimeUsage {
    pub(crate) const fn new(usage: Usage<WORK_COUNT, RESOURCE_COUNT>) -> Self {
        Self(usage)
    }
    /// Cumulative admitted work in this owner's named unit.
    pub const fn work(self, work: RenderWork) -> u64 {
        self.0.work(work.dimension())
    }
    /// Current admitted custody.
    pub const fn live(self, resource: RenderResource) -> u64 {
        self.0.live(resource.dimension())
    }
    /// Highest admitted lifetime custody, including actual allocation overlap.
    pub const fn peak(self, resource: RenderResource) -> u64 {
        self.0.peak(resource.dimension())
    }
}

/// Exact inclusive CPU interval usage, including its preexisting opening custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderCpuUsage(IntervalUsage<WORK_COUNT, RESOURCE_COUNT>);
impl RenderCpuUsage {
    /// Work admitted inside the interval.
    pub const fn work(self, work: RenderWork) -> u64 {
        self.0.work(work.dimension())
    }
    /// Custody already live when the interval opened.
    pub const fn opening_live(self, resource: RenderResource) -> u64 {
        self.0.opening_live(resource.dimension())
    }
    /// Custody live at the interval's actual closing boundary.
    pub const fn live(self, resource: RenderResource) -> u64 {
        self.0.live(resource.dimension())
    }
    /// Highest admitted custody during the interval, including its opening balance.
    pub const fn peak(self, resource: RenderResource) -> u64 {
        self.0.peak(resource.dimension())
    }
}

/// A bounded selected CPU-domain observation. Timing uses this owner's epoch,
/// or this interval's opening, and is independent of the frame timing authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderCpuEvent {
    /// An admitted work, acquire or release transition.
    Change {
        /// Actual origin role.
        scope: RenderCpuScope,
        /// Named unit.
        quantity: RenderQuantity,
        /// Admitted amount.
        amount: u64,
        /// Transition meaning.
        kind: RenderChangeKind,
        /// Optional time from observation opening.
        elapsed: Option<Duration>,
    },
    /// One ended inclusive owner-role transaction.
    Phase {
        /// Ended role.
        scope: RenderCpuScope,
        /// Inclusive transaction duration.
        duration: Option<Duration>,
        /// Completion time from observation opening.
        elapsed: Option<Duration>,
    },
    /// An acquisition or checked transition was refused.
    Refused {
        /// Origin role.
        scope: RenderCpuScope,
        /// Optional time from observation opening.
        elapsed: Option<Duration>,
    },
}

fn events<'a>(
    profile: Option<&'a Report<WORK_COUNT, RESOURCE_COUNT>>,
    scope: impl Fn(Scope) -> RenderCpuScope + 'a,
) -> impl Iterator<Item = RenderCpuEvent> + 'a {
    profile
        .into_iter()
        .flat_map(|p| p.events.iter())
        .map(move |event| match *event {
            surgeist_ledger::Event::Phase {
                scope: origin,
                duration,
                elapsed,
            } => RenderCpuEvent::Phase {
                scope: scope(origin),
                duration,
                elapsed,
            },
            surgeist_ledger::Event::Refused {
                origin, elapsed, ..
            } => RenderCpuEvent::Refused {
                scope: scope(origin),
                elapsed,
            },
            surgeist_ledger::Event::Accepted {
                scope: origin,
                change,
                elapsed,
            } => {
                let (quantity, amount, kind) = match change {
                    surgeist_ledger::Change::Work(d, n) => (
                        RenderQuantity::Work(
                            RenderWork::ALL
                                .into_iter()
                                .find(|w| w.dimension() == d)
                                .expect("CPU work schema"),
                        ),
                        n,
                        RenderChangeKind::Work,
                    ),
                    surgeist_ledger::Change::Acquire(d, n) => (
                        RenderQuantity::Resource(
                            RenderResource::ALL
                                .into_iter()
                                .find(|r| r.dimension() == d)
                                .expect("CPU resource schema"),
                        ),
                        n,
                        RenderChangeKind::Acquire,
                    ),
                    surgeist_ledger::Change::Release(d, n) => (
                        RenderQuantity::Resource(
                            RenderResource::ALL
                                .into_iter()
                                .find(|r| r.dimension() == d)
                                .expect("CPU resource schema"),
                        ),
                        n,
                        RenderChangeKind::Release,
                    ),
                };
                RenderCpuEvent::Change {
                    scope: scope(origin),
                    quantity,
                    amount,
                    kind,
                    elapsed,
                }
            }
        })
}

/// One exact closed renderer CPU interval. Its boundary is not rewritten when a
/// later rollback or release occurs; a later lifetime snapshot observes that work.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderCpuInterval {
    pub(crate) identity: RenderCpuIdentity,
    pub(crate) interval: IntervalReport<WORK_COUNT, RESOURCE_COUNT>,
}
impl RenderCpuInterval {
    /// Owner represented by the interval.
    pub const fn identity(&self) -> RenderCpuIdentity {
        self.identity
    }
    /// Exact inclusive root usage, available with profiling off.
    pub fn usage(&self) -> RenderCpuUsage {
        self.scope_usage(RenderCpuScope::Root)
    }
    /// Inclusive fixed-role usage.
    pub fn scope_usage(&self, scope: RenderCpuScope) -> RenderCpuUsage {
        RenderCpuUsage(self.interval.scopes[scope.index()].usage)
    }
    /// Interval elapsed host time; not the frame's elapsed time.
    pub fn elapsed(&self) -> Option<Duration> {
        self.interval.profile.as_ref().and_then(|p| p.elapsed)
    }
    /// Inclusive ended-role duration; do not add it to ancestors.
    pub fn scope_elapsed(&self, scope: RenderCpuScope) -> Option<Duration> {
        self.interval
            .profile
            .as_ref()
            .and_then(|p| p.scopes[scope.index()].phase_elapsed)
    }
    /// Ended role invocations; zero when profiling is off.
    pub fn scope_count(&self, scope: RenderCpuScope) -> u64 {
        self.interval
            .profile
            .as_ref()
            .map_or(0, |p| p.scopes[scope.index()].phases)
    }
    /// Selected events lost to the per-domain bound.
    pub fn dropped_events(&self) -> u64 {
        self.interval
            .profile
            .as_ref()
            .map_or(0, |p| p.dropped_events)
    }
    /// Whether observational counters saturated; authoritative usage remains exact.
    pub fn saturated(&self) -> bool {
        self.interval.profile.as_ref().is_some_and(|p| p.saturated)
    }
    /// Bounded events projected without another allocation.
    pub fn events(&self) -> impl Iterator<Item = RenderCpuEvent> + '_ {
        events(self.interval.profile.as_ref(), |scope| {
            RenderCpuScope::ALL
                .into_iter()
                .find(|s| self.interval.scopes[s.index()].scope == scope)
                .expect("CPU interval scope")
        })
    }
}

/// Pure coherent lifetime observation of one renderer CPU owner. Off exposes the
/// exact root; enabled profiles additionally expose the fixed-role breakdown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderCpuSnapshot {
    pub(crate) identity: RenderCpuIdentity,
    pub(crate) usage: RenderLifetimeUsage,
    pub(crate) profile: Option<Report<WORK_COUNT, RESOURCE_COUNT>>,
    pub(crate) fault: Option<RenderAccountingError>,
}
impl RenderCpuSnapshot {
    /// Owner represented by this immutable observation.
    pub const fn identity(&self) -> RenderCpuIdentity {
        self.identity
    }
    /// Exact root lifetime usage in every profiling mode.
    pub const fn usage(&self) -> RenderLifetimeUsage {
        self.usage
    }
    /// Coherent scoped lifetime usage, available only when profiling is enabled.
    pub fn scope_usage(&self, scope: RenderCpuScope) -> Option<RenderLifetimeUsage> {
        self.profile
            .as_ref()
            .map(|p| RenderLifetimeUsage::new(p.scopes[scope.index()].usage))
    }
    /// First persistent accounting health fault, if one has occurred.
    pub const fn fault(&self) -> Option<RenderAccountingError> {
        self.fault
    }
    /// Owner-lifetime elapsed host time, including idle gaps.
    pub fn elapsed(&self) -> Option<Duration> {
        self.profile.as_ref().and_then(|p| p.elapsed)
    }
    /// Inclusive ended-role duration over the lifetime.
    pub fn scope_elapsed(&self, scope: RenderCpuScope) -> Option<Duration> {
        self.profile
            .as_ref()
            .and_then(|p| p.scopes[scope.index()].phase_elapsed)
    }
    /// Ended role invocations; zero when profiling is off.
    pub fn scope_count(&self, scope: RenderCpuScope) -> u64 {
        self.profile
            .as_ref()
            .map_or(0, |p| p.scopes[scope.index()].phases)
    }
    /// Selected lifetime events lost to the per-owner bound.
    pub fn dropped_events(&self) -> u64 {
        self.profile.as_ref().map_or(0, |p| p.dropped_events)
    }
    /// Whether optional observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.profile.as_ref().is_some_and(|p| p.saturated)
    }
    /// Bounded lifetime events without another event-buffer copy.
    pub fn events(&self) -> impl Iterator<Item = RenderCpuEvent> + '_ {
        events(self.profile.as_ref(), |scope| {
            let profile = self.profile.as_ref().expect("event requires profile");
            RenderCpuScope::ALL
                .into_iter()
                .find(|s| profile.scopes[s.index()].scope == scope)
                .expect("CPU snapshot scope")
        })
    }
}
