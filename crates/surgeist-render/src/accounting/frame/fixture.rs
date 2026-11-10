//! Fixture context using the actual lifecycle and cancellation guard.
use super::*;
pub(crate) struct FrameFixture<'a> {
    cpu: &'a RendererCpu,
    pub(crate) options: RenderAccountingOptions,
    pub(crate) report: Option<RenderFrameReport>,
}
impl<'a> FrameFixture<'a> {
    pub(crate) fn new(cpu: &'a RendererCpu, options: RenderAccountingOptions) -> Self {
        Self {
            cpu,
            options,
            report: None,
        }
    }
    pub(crate) async fn run<T>(
        &mut self,
        work: impl AsyncFnOnce(FrameWork<'_>) -> Result<T>,
    ) -> Result<FinishedFrame<T>> {
        frame_lifecycle(self, setup, canceled, async move |_, attempt, _| {
            work(attempt).await
        })
        .await
    }
    pub(crate) fn run_sync<T>(
        &mut self,
        work: impl FnOnce(FrameWork<'_>) -> Result<T>,
    ) -> Result<FinishedFrame<T>> {
        pollster::block_on(self.run(async move |attempt| work(attempt)))
    }
}
fn setup<'s>(context: &'s FrameFixture<'_>) -> (&'s RendererCpu, RenderAccountingOptions) {
    (context.cpu, context.options)
}
fn canceled(context: &mut FrameFixture<'_>, report: RenderFrameReport) {
    context.report = Some(report);
}
