//! Typed readback reports and their renderer-owned destination.
use super::{
    renderer_attempt::{RendererAttempt, RendererReport},
    *,
};
use std::ops::Deref;

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
///
/// Repeated immutable observations can coexist. The returned reference borrows
/// the stored report directly, including its resolved buffers. Both report
/// reference types inherit Send and Sync from their immutable report values.
/// Mutable Renderer operations require a later-used observation to end first.
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn render_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_readback_attempt().unwrap();
///     let pending = renderer.render(surface, scene, Parameters::default());
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn readback_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_readback_attempt().unwrap();
///     let pending = renderer.read_headless(surface);
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn take_frame_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_readback_attempt().unwrap();
///     let taken = renderer.take_last_attempt();
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn take_readback_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_readback_attempt().unwrap();
///     let taken = renderer.take_last_readback_attempt();
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
pub struct RenderReadbackReportRef<'a>(&'a RenderReadbackReport);
impl<'a> RenderReadbackReportRef<'a> {
    pub(crate) fn new(report: &'a RenderReadbackReport) -> Self {
        Self(report)
    }
}
impl Deref for RenderReadbackReportRef<'_> {
    type Target = RenderReadbackReport;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Mandatory driver-local checkpoint owner; work never receives this value.
struct ReadbackAttempt {
    core: RendererAttempt,
}
impl ReadbackAttempt {
    fn begin(cpu: &RendererCpu, options: RenderAccountingOptions) -> Result<Self> {
        Ok(Self {
            core: RendererAttempt::begin(RenderAttemptKind::Readback, cpu, options)?,
        })
    }
    fn account(&self) -> &Account {
        self.core.account()
    }
}
/// Short work capability borrowing the actual owner's core. Moving this view
/// cannot replace either driver's mandatory checkpoint owner.
pub(crate) struct ReadbackWork<'a> {
    core: &'a mut RendererAttempt,
}
impl ReadbackWork<'_> {
    pub(crate) fn account(&self) -> &Account {
        self.core.account()
    }
    pub(crate) fn attach_generation(&mut self, generation: &Generation) -> Result<()> {
        self.core.attach_generation(generation)
    }
}

/// Terminal data, without a checkpoint owner or lifecycle destructor.
pub(crate) struct FinishedReadback<T> {
    result: Result<T>,
    started: bool,
    report: RenderReadbackReport,
    fault: Option<RenderAccountingError>,
}
impl<T> FinishedReadback<T> {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Result<T>,
        bool,
        RenderReadbackReport,
        Option<RenderAccountingError>,
    ) {
        (self.result, self.started, self.report, self.fault)
    }
}
struct ReadbackGuard<'g, C> {
    context: &'g mut C,
    attempt: &'g mut ReadbackAttempt,
    publish_canceled: fn(&mut C, RenderReadbackReport),
}
impl<C> ReadbackGuard<'_, C> {
    fn parts(&mut self) -> (&mut C, &mut ReadbackAttempt) {
        (self.context, self.attempt)
    }
    fn finish<T>(self, result: Result<T>) -> FinishedReadback<T> {
        let started = self.attempt.core.started();
        let (status, diagnostic) = match &result {
            Ok(_) => (RenderAttemptStatus::Succeeded, None),
            Err(error) => (
                self.attempt.core.status_for(error),
                error.accounting_diagnostic(),
            ),
        };
        let (data, fault) = self.attempt.core.finish(status, diagnostic);
        FinishedReadback {
            result,
            started,
            report: RenderReadbackReport(data),
            fault,
        }
    }
}
impl<C> Drop for ReadbackGuard<'_, C> {
    fn drop(&mut self) {
        // Explicit finish consumed the checkpoint, making this path inert.
        if let Some(data) = self.attempt.core.cancel() {
            (self.publish_canceled)(self.context, RenderReadbackReport(data));
        }
    }
}
pub(crate) async fn readback_lifecycle<C, T>(
    context: &mut C,
    setup: for<'s> fn(&'s C) -> (&'s RendererCpu, RenderAccountingOptions),
    publish_canceled: fn(&mut C, RenderReadbackReport),
    work: impl AsyncFnOnce(&mut C, ReadbackWork<'_>, &Account) -> Result<T>,
) -> Result<FinishedReadback<T>> {
    let mut attempt = {
        let (cpu, options) = setup(context);
        ReadbackAttempt::begin(cpu, options)?
    };
    let account = attempt.account().clone();
    let mut guard = ReadbackGuard {
        context,
        attempt: &mut attempt,
        publish_canceled,
    };
    // The child future is destroyed before cancellation closes the checkpoints.
    let result = {
        let (context, attempt) = guard.parts();
        work(
            context,
            ReadbackWork {
                core: &mut attempt.core,
            },
            &account,
        )
        .await
    };
    let finished = guard.finish(result);
    // Neither mandatory local may reach publication or final output transfer.
    drop(attempt);
    drop(account);
    Ok(finished)
}

#[cfg(test)]
pub(crate) mod fixture;
#[cfg(test)]
mod tests;
