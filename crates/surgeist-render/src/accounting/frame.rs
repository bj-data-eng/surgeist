//! Typed frame reports and their renderer-owned destination.
use super::{
    renderer_attempt::{RendererAttempt, RendererReport, ReportSlot},
    *,
};
use std::{ops::Deref, sync::RwLockReadGuard};

/// A completed frame attempt, distinct from last-successful frame statistics.
/// Each domain keeps its actual closing boundary. Failed extraction is typed
/// unavailability; an already closed interval does not include later rollback.
#[derive(Eq, PartialEq)]
pub struct RenderFrameReport {
    data: RendererReport,
}
impl std::fmt::Debug for RenderFrameReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt("RenderFrameReport", formatter)
    }
}
impl RenderFrameReport {
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self {
            data: self.data.snapshot_for_test(),
        }
    }
    /// Terminal status after inner cleanup and required finalization.
    pub const fn status(&self) -> RenderAttemptStatus {
        self.data.status()
    }
    /// Exact forward-work interval, or its typed extraction fault.
    pub fn attempt(&self) -> std::result::Result<&RenderAttemptReport, RenderAccountingError> {
        self.data.attempt()
    }
    /// Renderer CPU interval, or its typed extraction fault.
    pub fn renderer_cpu(&self) -> std::result::Result<&RenderCpuInterval, RenderAccountingError> {
        self.data.renderer_cpu()
    }
    /// Selected generation interval, when one was associated with this attempt.
    pub fn generation(
        &self,
    ) -> Option<std::result::Result<&RenderGenerationInterval, RenderAccountingError>> {
        self.data.generation()
    }
    /// First returned accounting cause, without replacing an earlier backend cause.
    pub const fn accounting_diagnostic(&self) -> Option<RenderAccountingError> {
        self.data.accounting_diagnostic()
    }
    /// Mandatory cleanup/finalization fault, separate from the causal outcome.
    pub(crate) fn note_cleanup_fault(&mut self, fault: Option<RenderAccountingError>) {
        self.data.note_cleanup_fault(fault);
    }
    /// First mandatory cleanup or finalization fault, independently of an earlier
    /// causal backend failure or refusal.
    pub const fn cleanup_fault(&self) -> Option<RenderAccountingError> {
        self.data.cleanup_fault()
    }
}

/// Borrowed immutable latest-frame observation. Holding it borrows Renderer and
/// prevents another mutable operation, just like borrowing ordinary renderer data.
pub struct RenderFrameReportRef<'a>(RwLockReadGuard<'a, Option<RenderFrameReport>>);
impl Deref for RenderFrameReportRef<'_> {
    type Target = RenderFrameReport;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("occupied borrowed report")
    }
}
#[derive(Clone, Default)]
pub(crate) struct FrameReportSlot(ReportSlot<RenderFrameReport>);
impl FrameReportSlot {
    pub(crate) fn read(&self) -> Option<RenderFrameReportRef<'_>> {
        self.0.read().map(RenderFrameReportRef)
    }
    pub(crate) fn take(&self) -> Option<RenderFrameReport> {
        self.0.take()
    }
    pub(crate) fn store(&self, report: RenderFrameReport) {
        self.0.store(report);
    }
}

pub(crate) struct FrameAttempt {
    core: RendererAttempt,
    slot: FrameReportSlot,
}
impl FrameAttempt {
    pub(crate) fn begin(
        slot: FrameReportSlot,
        cpu: &RendererCpu,
        options: RenderAccountingOptions,
    ) -> Result<Self> {
        Ok(Self {
            core: RendererAttempt::begin(RenderAttemptKind::Frame, cpu, options)?,
            slot,
        })
    }
    pub(crate) fn started(&self) -> bool {
        self.core.started()
    }
    pub(crate) fn account(&self) -> &Account {
        self.core.account()
    }
    pub(crate) fn attach_generation(&mut self, generation: &Generation) -> Result<()> {
        self.core.attach_generation(generation)
    }
    pub(crate) fn status_for(&self, error: &Error) -> RenderAttemptStatus {
        self.core.status_for(error)
    }
    pub(crate) fn finish(
        mut self,
        status: RenderAttemptStatus,
        diagnostic: Option<RenderAccountingError>,
    ) -> (RenderFrameReport, Option<RenderAccountingError>) {
        let (report, fault) = self.core.finish(status, diagnostic);
        (RenderFrameReport { data: report }, fault)
    }
}
impl Drop for FrameAttempt {
    fn drop(&mut self) {
        if let Some(report) = self.core.cancel() {
            self.slot.store(RenderFrameReport { data: report });
        }
    }
}

#[cfg(test)]
mod tests;
