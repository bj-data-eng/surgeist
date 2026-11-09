//! Typed readback reports and their renderer-owned destination.
use super::{
    renderer_attempt::{RendererAttempt, RendererReport, ReportSlot},
    *,
};
use std::{ops::Deref, sync::RwLockReadGuard};

/// A completed explicit readback attempt, separate from frame diagnostics.
/// Each domain retains its actual closing boundary. Successful extraction closes
/// before the final output transfer while guarded output is still renderer-owned;
/// later transfer failure or disposal does not rewrite the closed intervals.
#[derive(Eq, PartialEq)]
pub struct RenderReadbackReport(RendererReport);
impl std::fmt::Debug for RenderReadbackReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt("RenderReadbackReport", formatter)
    }
}
impl RenderReadbackReport {
    #[cfg(test)]
    pub(crate) fn snapshot_for_test(&self) -> Self {
        Self(self.0.snapshot_for_test())
    }
    /// Terminal status after inner cleanup and required finalization.
    pub const fn status(&self) -> RenderAttemptStatus {
        self.0.status()
    }
    /// Exact forward-work interval, or its typed extraction fault.
    pub fn attempt(&self) -> std::result::Result<&RenderAttemptReport, RenderAccountingError> {
        self.0.attempt()
    }
    /// Renderer CPU interval, or its typed extraction fault.
    pub fn renderer_cpu(&self) -> std::result::Result<&RenderCpuInterval, RenderAccountingError> {
        self.0.renderer_cpu()
    }
    /// Selected generation interval. None means no generation was associated,
    /// including empty output and requests rejected before native work.
    pub fn generation(
        &self,
    ) -> Option<std::result::Result<&RenderGenerationInterval, RenderAccountingError>> {
        self.0.generation()
    }
    /// First returned accounting cause, without replacing an earlier backend cause.
    pub const fn accounting_diagnostic(&self) -> Option<RenderAccountingError> {
        self.0.accounting_diagnostic()
    }
    /// Final CPU transfer failure changes outcome, never the closed observations.
    pub(crate) fn fail_after_close(
        &mut self,
        error: &Error,
        cleanup_fault: Option<RenderAccountingError>,
    ) {
        self.0.fail_after_close(error, cleanup_fault);
    }
    /// First mandatory cleanup or finalization fault, independently of an earlier
    /// causal backend failure or refusal.
    pub const fn cleanup_fault(&self) -> Option<RenderAccountingError> {
        self.0.cleanup_fault()
    }
}

/// Borrowed immutable latest-readback observation. Holding it borrows Renderer and
/// prevents another mutable operation, just like borrowing ordinary renderer data.
pub struct RenderReadbackReportRef<'a>(RwLockReadGuard<'a, Option<RenderReadbackReport>>);
impl Deref for RenderReadbackReportRef<'_> {
    type Target = RenderReadbackReport;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("occupied borrowed report")
    }
}
#[derive(Clone, Default)]
pub(crate) struct ReadbackReportSlot(ReportSlot<RenderReadbackReport>);
impl ReadbackReportSlot {
    pub(crate) fn read(&self) -> Option<RenderReadbackReportRef<'_>> {
        self.0.read().map(RenderReadbackReportRef)
    }
    pub(crate) fn take(&self) -> Option<RenderReadbackReport> {
        self.0.take()
    }
    pub(crate) fn store(&self, report: RenderReadbackReport) {
        self.0.store(report);
    }
}

pub(crate) struct ReadbackAttempt {
    core: RendererAttempt,
    slot: ReadbackReportSlot,
}
impl ReadbackAttempt {
    pub(crate) fn begin(
        slot: ReadbackReportSlot,
        cpu: &RendererCpu,
        options: RenderAccountingOptions,
    ) -> Result<Self> {
        Ok(Self {
            core: RendererAttempt::begin(RenderAttemptKind::Readback, cpu, options)?,
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
    ) -> (RenderReadbackReport, Option<RenderAccountingError>) {
        let (report, fault) = self.core.finish(status, diagnostic);
        (RenderReadbackReport(report), fault)
    }
}
impl Drop for ReadbackAttempt {
    fn drop(&mut self) {
        if let Some(report) = self.core.cancel() {
            self.slot.store(RenderReadbackReport(report));
        }
    }
}

#[cfg(test)]
mod tests;
