//! Typed views of the renderer CPU owner's exact Ledger observations.
use super::report::{IntervalNumbers, LifetimeNumbers, ReportBacking, ResolvedReport};
use super::{RenderAccountingError, RenderChangeKind, RenderQuantity, RenderResource, RenderWork};
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
pub struct RenderLifetimeUsage(Usage<RenderWork, RenderResource>);
impl RenderLifetimeUsage {
    pub(crate) const fn new(usage: Usage<RenderWork, RenderResource>) -> Self {
        Self(usage)
    }
    /// Cumulative admitted work in this owner's named unit.
    pub fn work(self, work: RenderWork) -> u64 {
        self.0.work(work)
    }
    /// Current admitted custody.
    pub fn live(self, resource: RenderResource) -> u64 {
        self.0.live(resource)
    }
    /// Highest admitted lifetime custody, including actual allocation overlap.
    pub fn peak(self, resource: RenderResource) -> u64 {
        self.0.peak(resource)
    }
}

/// Exact inclusive CPU interval usage, including its preexisting opening custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderCpuUsage(IntervalUsage<RenderWork, RenderResource>);
impl RenderCpuUsage {
    /// Work admitted inside the interval.
    pub fn work(self, work: RenderWork) -> u64 {
        self.0.work(work)
    }
    /// Custody already live when the interval opened.
    pub fn opening_live(self, resource: RenderResource) -> u64 {
        self.0.opening_live(resource)
    }
    /// Custody live at the interval's actual closing boundary.
    pub fn live(self, resource: RenderResource) -> u64 {
        self.0.live(resource)
    }
    /// Highest admitted custody during the interval, including its opening balance.
    pub fn peak(self, resource: RenderResource) -> u64 {
        self.0.peak(resource)
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

/// One exact closed renderer CPU interval. Later releases do not rewrite it.
#[derive(Debug, Eq, PartialEq)]
pub struct RenderCpuInterval {
    pub(crate) identity: RenderCpuIdentity,
    data: ResolvedReport<RenderCpuScope, IntervalNumbers>,
}
impl RenderCpuInterval {
    pub(super) fn new(
        identity: RenderCpuIdentity,
        interval: IntervalReport<RenderWork, RenderResource>,
        handles: &[Scope],
        backing: ReportBacking<RenderCpuScope, IntervalNumbers>,
    ) -> Result<Self, RenderAccountingError> {
        let (root, data) = ResolvedReport::interval(interval, handles, backing)?;
        if identity.0 != root {
            return Err(RenderAccountingError::Invariant);
        }
        Ok(Self { identity, data })
    }
    /// Owner represented by the interval.
    pub const fn identity(&self) -> RenderCpuIdentity {
        self.identity
    }
    /// Exact inclusive root usage in every profiling mode.
    pub fn usage(&self) -> RenderCpuUsage {
        self.scope_usage(RenderCpuScope::Root)
    }
    /// Inclusive fixed-role usage.
    pub fn scope_usage(&self, scope: RenderCpuScope) -> RenderCpuUsage {
        RenderCpuUsage(self.data.rows[scope.index()].usage)
    }
    /// Interval elapsed host time.
    pub fn elapsed(&self) -> Option<Duration> {
        self.data.elapsed
    }
    /// Inclusive ended-role duration.
    pub fn scope_elapsed(&self, scope: RenderCpuScope) -> Option<Duration> {
        self.data.rows[scope.index()].phase_elapsed
    }
    /// Ended role invocations; zero when profiling is off.
    pub fn scope_count(&self, scope: RenderCpuScope) -> u64 {
        self.data.rows[scope.index()].phases
    }
    /// Selected events lost to the per-domain bound.
    pub fn dropped_events(&self) -> u64 {
        self.data.dropped_events
    }
    /// Whether observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.data.saturated
    }
    /// Already resolved bounded detail.
    pub fn events(&self) -> impl Iterator<Item = RenderCpuEvent> + '_ {
        self.data.events.iter().copied()
    }
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self {
            identity: self.identity,
            data: self.data.snapshot_for_test(),
        }
    }
}

/// Coherent lifetime observation. Off retains only exact root usage and identity.
#[derive(Debug, Eq, PartialEq)]
pub struct RenderCpuSnapshot {
    pub(crate) identity: RenderCpuIdentity,
    pub(crate) usage: RenderLifetimeUsage,
    data: ResolvedReport<RenderCpuScope, LifetimeNumbers>,
    pub(crate) fault: Option<RenderAccountingError>,
}
impl RenderCpuSnapshot {
    pub(super) fn new(
        identity: RenderCpuIdentity,
        usage: RenderLifetimeUsage,
        profile: Option<Report<RenderWork, RenderResource>>,
        fault: Option<RenderAccountingError>,
        handles: &[Scope],
        backing: ReportBacking<RenderCpuScope, LifetimeNumbers>,
    ) -> Result<Self, RenderAccountingError> {
        let (root, data) = ResolvedReport::snapshot(profile, handles, backing)?;
        if identity.0 != root {
            return Err(RenderAccountingError::Invariant);
        }
        Ok(Self {
            identity,
            usage,
            data,
            fault,
        })
    }
    /// Owner represented by this immutable observation.
    pub const fn identity(&self) -> RenderCpuIdentity {
        self.identity
    }
    /// Exact root lifetime usage in every profiling mode.
    pub const fn usage(&self) -> RenderLifetimeUsage {
        self.usage
    }
    /// Scoped usage is unavailable when observation is Off.
    pub fn scope_usage(&self, scope: RenderCpuScope) -> Option<RenderLifetimeUsage> {
        self.data
            .observed
            .then(|| RenderLifetimeUsage::new(self.data.rows[scope.index()].usage))
    }
    /// First persistent accounting health fault.
    pub const fn fault(&self) -> Option<RenderAccountingError> {
        self.fault
    }
    /// Owner-lifetime elapsed host time.
    pub fn elapsed(&self) -> Option<Duration> {
        self.data.elapsed
    }
    /// Inclusive ended-role duration; unavailable when observation is Off.
    pub fn scope_elapsed(&self, scope: RenderCpuScope) -> Option<Duration> {
        if self.data.observed {
            self.data.rows[scope.index()].phase_elapsed
        } else {
            None
        }
    }
    /// Ended role invocations; zero when observation is Off.
    pub fn scope_count(&self, scope: RenderCpuScope) -> u64 {
        if self.data.observed {
            self.data.rows[scope.index()].phases
        } else {
            0
        }
    }
    /// Selected lifetime detail lost to the owner bound.
    pub fn dropped_events(&self) -> u64 {
        self.data.dropped_events
    }
    /// Whether optional observational counters saturated.
    pub fn saturated(&self) -> bool {
        self.data.saturated
    }
    /// Already resolved bounded lifetime detail.
    pub fn events(&self) -> impl Iterator<Item = RenderCpuEvent> + '_ {
        self.data.events.iter().copied()
    }
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self {
            identity: self.identity,
            usage: self.usage,
            data: self.data.snapshot_for_test(),
            fault: self.fault,
        }
    }
}
