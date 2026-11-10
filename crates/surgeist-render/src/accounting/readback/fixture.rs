//! Fixture context using the actual lifecycle and cancellation guard.
use super::*;
pub(crate) struct ReadbackFixture<'a> {
    cpu: &'a RendererCpu,
    pub(crate) options: RenderAccountingOptions,
    pub(crate) report: Option<RenderReadbackReport>,
}
impl<'a> ReadbackFixture<'a> {
    pub(crate) fn new(cpu: &'a RendererCpu, options: RenderAccountingOptions) -> Self {
        Self {
            cpu,
            options,
            report: None,
        }
    }
    pub(crate) async fn run<T>(
        &mut self,
        work: impl AsyncFnOnce(ReadbackWork<'_>) -> Result<T>,
    ) -> Result<FinishedReadback<T>> {
        readback_lifecycle(self, setup, canceled, async move |_, attempt, _| {
            work(attempt).await
        })
        .await
    }
    pub(crate) fn run_sync<T>(
        &mut self,
        work: impl FnOnce(ReadbackWork<'_>) -> Result<T>,
    ) -> Result<FinishedReadback<T>> {
        pollster::block_on(self.run(async move |attempt| work(attempt)))
    }
}
fn setup<'s>(context: &'s ReadbackFixture<'_>) -> (&'s RendererCpu, RenderAccountingOptions) {
    (context.cpu, context.options)
}
fn canceled(context: &mut ReadbackFixture<'_>, report: RenderReadbackReport) {
    context.report = Some(report);
}
