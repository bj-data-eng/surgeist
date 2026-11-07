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
        renderer.frame_reports.store(report);
        stats
    }
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
        use crate::accounting::{FrameAttempt, RenderPhase};
        let mut attempt = FrameAttempt::begin(
            self.frame_reports.clone(),
            &self.cpu,
            self.options.accounting(),
        )?;
        let account = attempt.account().clone();
        let result = account
            .async_phase(RenderPhase::Frame, async {
                let identity = self.render_device_identity(surface)?;
                let generation = self
                    .backend
                    .as_ref()
                    .ok_or(crate::RenderAccountingError::Invariant)?
                    .generation(identity)?
                    .clone();
                attempt.attach_generation(&generation)?;
                let draft = self.image_history.begin(&account)?;
                let account = account.with_history(draft.observer());
                let publication = run(self, surface, &account).await?;
                Ok((publication, draft, account))
            })
            .await;
        match result {
            Ok((publication, draft, account)) => {
                self.publish_clean_render_frame(surface, publication, draft, attempt, &account)
            }
            Err(error) => {
                let started = attempt.started();
                let status = attempt.status_for(&error);
                let (report, _) = attempt.finish(status, error.accounting_diagnostic());
                if started {
                    self.frame_reports.store(report);
                }
                Err(error)
            }
        }
    }

    pub(super) fn publish_clean_render_frame(
        &mut self,
        surface: &mut Surface,
        mut publication: RenderPublication,
        draft: crate::accounting::HistoryDraft,
        attempt: crate::accounting::FrameAttempt,
        account: &crate::accounting::Account,
    ) -> crate::Result<Stats> {
        use crate::accounting::{RenderAttemptStatus, RenderPhase};
        let preparation = account.phase(RenderPhase::Publish, || {
            account.phase(RenderPhase::Cleanup, || {
                publication.frame.finish_cpu_cleanup();
                Ok(())
            })?;
            let ready = draft.into_publication()?.retire_old()?;
            self.cpu.ensure_healthy()?;
            Ok(ready)
        });
        let history = match preparation {
            Ok(history) => history,
            Err(error) => {
                drop(publication);
                let status = attempt.status_for(&error);
                let (report, _) = attempt.finish(status, error.accounting_diagnostic());
                self.frame_reports.store(report);
                return Err(error);
            }
        };
        let (mut report, fault) = attempt.finish(RenderAttemptStatus::Succeeded, None);
        if let Some(fault) = fault {
            drop(history); // restores canonical baseline using already owned backing
            drop(publication);
            report.note_cleanup_fault(self.cpu.fault());
            self.frame_reports.store(report);
            return Err(fault.into());
        }
        let stats =
            match Stats::from_frame_report(&report, publication.route(), publication.precision()) {
                Ok(stats) => stats,
                Err(error) => {
                    drop(history);
                    drop(publication);
                    report.note_cleanup_fault(error.accounting_diagnostic().or(self.cpu.fault()));
                    self.frame_reports.store(report);
                    return Err(error);
                }
            };
        Ok(publication.commit(self, surface, stats, history, report))
    }
}
