//! Typed frame reports and their renderer-owned destination.
use super::{
    renderer_attempt::{RendererAttempt, RendererReport},
    *,
};
use std::ops::Deref;

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
///
/// Multiple observations, including the other report domain, can coexist.
/// The report is borrowed directly; reading it does not clone its buffers.
/// Mutable Renderer operations require those borrows to end.
///
/// ```no_run
/// use surgeist_render::Renderer;
/// fn observe(renderer: &Renderer) {
///     let frame = renderer.last_attempt().unwrap();
///     let frame_again = renderer.last_attempt().unwrap();
///     let readback = renderer.last_readback_attempt().unwrap();
///     let readback_again = renderer.last_readback_attempt().unwrap();
///     assert!(std::ptr::eq(&*frame, &*frame_again));
///     assert!(std::ptr::eq(&*readback, &*readback_again));
/// }
/// fn take(mut renderer: Renderer) {
///     let frame = renderer.take_last_attempt().unwrap();
///     let readback = renderer.take_last_readback_attempt().unwrap();
///     drop(renderer);
///     assert!(frame.attempt().is_ok());
///     assert!(readback.attempt().is_ok());
/// }
/// ```
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn render_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_attempt().unwrap();
///     let pending = renderer.render(surface, scene, Parameters::default());
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn readback_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_attempt().unwrap();
///     let pending = renderer.read_headless(surface);
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn take_frame_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_attempt().unwrap();
///     let taken = renderer.take_last_attempt();
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
///
/// ```compile_fail,E0502
/// use surgeist_render::{Renderer, Surface, Scene, Parameters};
/// fn take_readback_while_observed(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) {
///     let report = renderer.last_attempt().unwrap();
///     let taken = renderer.take_last_readback_attempt();
///     let _status = report.status(); // A real final use keeps the borrow live.
/// }
/// ```
pub struct RenderFrameReportRef<'a>(&'a RenderFrameReport);
impl<'a> RenderFrameReportRef<'a> {
    pub(crate) fn new(report: &'a RenderFrameReport) -> Self {
        Self(report)
    }
}
impl Deref for RenderFrameReportRef<'_> {
    type Target = RenderFrameReport;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Mandatory driver-local checkpoint owner; work never receives this value.
struct FrameAttempt {
    core: RendererAttempt,
}
impl FrameAttempt {
    fn begin(cpu: &RendererCpu, options: RenderAccountingOptions) -> Result<Self> {
        Ok(Self {
            core: RendererAttempt::begin(RenderAttemptKind::Frame, cpu, options)?,
        })
    }
    fn account(&self) -> &Account {
        self.core.account()
    }
}
/// Short work capability borrowing the actual owner's core. Moving this view
/// cannot replace either driver's mandatory checkpoint owner.
pub(crate) struct FrameWork<'a> {
    core: &'a mut RendererAttempt,
}
impl FrameWork<'_> {
    pub(crate) fn account(&self) -> &Account {
        self.core.account()
    }
    pub(crate) fn attach_generation(&mut self, generation: &Generation) -> Result<()> {
        self.core.attach_generation(generation)
    }
}

/// Terminal data, without a checkpoint owner or lifecycle destructor.
pub(crate) struct FinishedFrame<T> {
    result: Result<T>,
    started: bool,
    report: RenderFrameReport,
    fault: Option<RenderAccountingError>,
}
impl<T> FinishedFrame<T> {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Result<T>,
        bool,
        RenderFrameReport,
        Option<RenderAccountingError>,
    ) {
        (self.result, self.started, self.report, self.fault)
    }
}
struct FrameGuard<'g, C> {
    context: &'g mut C,
    attempt: &'g mut FrameAttempt,
    publish_canceled: fn(&mut C, RenderFrameReport),
}
impl<C> FrameGuard<'_, C> {
    fn parts(&mut self) -> (&mut C, &mut FrameAttempt) {
        (self.context, self.attempt)
    }
    fn finish<T>(self, result: Result<T>) -> FinishedFrame<T> {
        let started = self.attempt.core.started();
        let (status, diagnostic) = match &result {
            Ok(_) => (RenderAttemptStatus::Succeeded, None),
            Err(error) => (
                self.attempt.core.status_for(error),
                error.accounting_diagnostic(),
            ),
        };
        let (data, fault) = self.attempt.core.finish(status, diagnostic);
        FinishedFrame {
            result,
            started,
            report: RenderFrameReport { data },
            fault,
        }
    }
}
impl<C> Drop for FrameGuard<'_, C> {
    fn drop(&mut self) {
        // Explicit finish consumed the checkpoint, making this path inert.
        if let Some(data) = self.attempt.core.cancel() {
            (self.publish_canceled)(self.context, RenderFrameReport { data });
        }
    }
}
pub(crate) async fn frame_lifecycle<C, T>(
    context: &mut C,
    setup: for<'s> fn(&'s C) -> (&'s RendererCpu, RenderAccountingOptions),
    publish_canceled: fn(&mut C, RenderFrameReport),
    work: impl AsyncFnOnce(&mut C, FrameWork<'_>, &Account) -> Result<T>,
) -> Result<FinishedFrame<T>> {
    let mut attempt = {
        let (cpu, options) = setup(context);
        FrameAttempt::begin(cpu, options)?
    };
    let account = attempt.account().clone();
    let mut guard = FrameGuard {
        context,
        attempt: &mut attempt,
        publish_canceled,
    };
    // The child future is destroyed before cancellation closes the checkpoints.
    let result = {
        let (context, attempt) = guard.parts();
        work(
            context,
            FrameWork {
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
