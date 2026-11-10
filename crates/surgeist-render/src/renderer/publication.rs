use super::Renderer;
use crate::{
    EffectPrecision, Parameters, RenderRoute, Stats, Surface,
    accounting::{HistoryCommit, RenderFrameReport},
    backend::SurfaceFrameCommit,
};

/// All fallible accounting and metadata preparation precedes this final move.
pub(super) struct RenderPublication {
    pub(super) frame: SurfaceFrameCommit,
    parameters: Parameters,
    route: RenderRoute,
}
impl RenderPublication {
    pub(super) fn new(
        frame: SurfaceFrameCommit,
        parameters: Parameters,
        route: RenderRoute,
    ) -> Self {
        Self {
            frame,
            parameters,
            route,
        }
    }
    pub(super) fn route(&self) -> RenderRoute {
        self.route
    }
    pub(super) fn precision(&self) -> Option<EffectPrecision> {
        self.frame.working_format().map(|format| match format {
            crate::resource::WorkingFormat::HighPrecision => EffectPrecision::High,
            crate::resource::WorkingFormat::ReducedPrecision => EffectPrecision::Reduced,
        })
    }
    pub(super) fn commit(
        self,
        renderer: &mut Renderer,
        surface: &mut Surface,
        stats: Stats,
        history: HistoryCommit,
        report: RenderFrameReport,
    ) -> Stats {
        // No setup, fallible extraction, copy, allocation, admission or phase end
        // remains. Later native releases retain their lifetime observation policy.
        history.commit();
        self.frame.commit(surface);
        renderer.stats = stats;
        surface.last_parameters = Some(self.parameters);
        renderer.frame_reports = Some(report);
        stats
    }
}

struct FrameContext<'r, 's> {
    renderer: &'r mut Renderer,
    surface: &'s mut Surface,
}
fn frame_setup<'a>(
    context: &'a FrameContext<'_, '_>,
) -> (
    &'a crate::accounting::RendererCpu,
    crate::RenderAccountingOptions,
) {
    (&context.renderer.cpu, context.renderer.options.accounting())
}
fn frame_canceled(context: &mut FrameContext<'_, '_>, report: RenderFrameReport) {
    context.renderer.frame_reports = Some(report);
}
struct ReadyFrame {
    publication: RenderPublication,
    history: HistoryCommit,
    _account: crate::accounting::Account,
}
impl Renderer {
    /// One shared owning gate for real frame execution, including forced-route
    /// fixtures. The callback cannot publish; all terminal paths retain reports.
    pub(super) async fn frame_operation(
        &mut self,
        surface: &mut Surface,
        run: impl AsyncFnOnce(
            &mut Self,
            &mut Surface,
            &crate::accounting::Account,
        ) -> crate::Result<RenderPublication>,
    ) -> crate::Result<Stats> {
        use crate::accounting::{RenderPhase, frame_lifecycle};
        let mut context = FrameContext {
            renderer: self,
            surface,
        };
        let finished = frame_lifecycle(
            &mut context,
            frame_setup,
            frame_canceled,
            async move |context, mut attempt, account| {
                let renderer = &mut *context.renderer;
                let surface = &mut *context.surface;
                let (mut publication, draft, account) = account
                    .async_phase(RenderPhase::Frame, async {
                        let identity = renderer.render_device_identity(surface)?;
                        let generation = renderer
                            .backend
                            .as_ref()
                            .ok_or(crate::RenderAccountingError::Invariant)?
                            .generation(identity)?
                            .clone();
                        attempt.attach_generation(&generation)?;
                        let draft = renderer.image_history.begin(attempt.account())?;
                        let account = account.with_history(draft.observer());
                        let publication = run(renderer, surface, &account).await?;
                        Ok((publication, draft, account))
                    })
                    .await?;
                // Preparation remains outside Frame and before report extraction.
                let preparation = account.phase(RenderPhase::Publish, || {
                    account.phase(RenderPhase::Cleanup, || {
                        publication.frame.finish_cpu_cleanup();
                        Ok(())
                    })?;
                    let ready = draft.into_publication()?.retire_old()?;
                    renderer.cpu.ensure_healthy()?;
                    Ok(ready)
                });
                match preparation {
                    Ok(history) => Ok(ReadyFrame {
                        publication,
                        history,
                        _account: account,
                    }),
                    Err(error) => {
                        drop(publication);
                        Err(error)
                    }
                }
            },
        )
        .await?;
        let (result, started, mut report, fault) = finished.into_parts();
        let ReadyFrame {
            publication,
            history,
            _account,
        } = match result {
            Ok(ready) => ready,
            Err(error) => {
                if started {
                    self.frame_reports = Some(report);
                }
                return Err(error);
            }
        };
        if let Some(fault) = fault {
            drop(history);
            drop(publication);
            report.note_cleanup_fault(self.cpu.fault());
            self.frame_reports = Some(report);
            return Err(fault.into());
        }
        let stats =
            match Stats::from_frame_report(&report, publication.route(), publication.precision()) {
                Ok(stats) => stats,
                Err(error) => {
                    drop(history);
                    drop(publication);
                    report.note_cleanup_fault(error.accounting_diagnostic().or(self.cpu.fault()));
                    self.frame_reports = Some(report);
                    return Err(error);
                }
            };
        Ok(publication.commit(self, surface, stats, history, report))
    }
}
