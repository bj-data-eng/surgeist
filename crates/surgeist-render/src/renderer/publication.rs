use super::Renderer;
use crate::{
    BackendErrorCode, EffectPrecision, Error, Parameters, RenderAccountingOptions, RenderRoute,
    Result, Scene, Stats, Surface,
    accounting::{
        Account, HistoryCommit, RenderFrameReport, RenderPhase, RendererCpu, frame_lifecycle,
    },
    backend::SurfaceFrameCommit,
    resource::{FrameCleanup, WorkingFormat},
    surface::{HeadlessPublication, HeadlessResources, SurfaceBackend},
};

pub(super) struct RenderPublication {
    frame: SurfaceFrameCommit,
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
    fn bind(self, surface: &mut Surface) -> Result<BoundPublication<'_>> {
        let (headless, cleanup, working_format) = self.frame.into_parts();
        let headless = match headless {
            Some(draft) => Some(BoundHeadlessPublication::bind(
                &mut surface.backend,
                #[cfg(test)]
                &mut surface.headless_publication_count,
                draft,
            )?),
            None => None,
        };
        Ok(BoundPublication {
            headless,
            cleanup,
            last_parameters: &mut surface.last_parameters,
            parameters: self.parameters,
            route: self.route,
            working_format,
        })
    }
}

struct BoundHeadlessPublication<'s> {
    draft: HeadlessPublication,
    resources: &'s mut HeadlessResources,
    #[cfg(test)]
    count: &'s mut usize,
}
impl<'s> BoundHeadlessPublication<'s> {
    fn bind(
        backend: &'s mut SurfaceBackend,
        #[cfg(test)] count: &'s mut usize,
        draft: HeadlessPublication,
    ) -> Result<Self> {
        match backend {
            SurfaceBackend::Headless { resources, .. } => Ok(Self {
                draft,
                resources,
                #[cfg(test)]
                count,
            }),
            _ => Err(Error::new(
                BackendErrorCode::RenderFailed,
                "headless publication requires a headless frame surface",
            )),
        }
    }

    fn install(self) {
        *self.resources = HeadlessResources::Ready {
            texture: self.draft.texture,
        };
        #[cfg(test)]
        {
            *self.count = self.count.saturating_add(1);
        }
    }
}

#[cfg(test)]
impl Surface {
    pub(crate) fn commit_headless_publication(&mut self, draft: HeadlessPublication) {
        BoundHeadlessPublication::bind(
            &mut self.backend,
            &mut self.headless_publication_count,
            draft,
        )
        .expect("headless publication fixture requires a headless surface")
        .install();
    }
}

struct BoundPublication<'s> {
    headless: Option<BoundHeadlessPublication<'s>>,
    cleanup: Option<FrameCleanup>,
    last_parameters: &'s mut Option<Parameters>,
    parameters: Parameters,
    route: RenderRoute,
    working_format: Option<WorkingFormat>,
}
impl BoundPublication<'_> {
    fn precision(&self) -> Option<EffectPrecision> {
        self.working_format.map(|format| match format {
            WorkingFormat::HighPrecision => EffectPrecision::High,
            WorkingFormat::ReducedPrecision => EffectPrecision::Reduced,
        })
    }
    fn commit(
        self,
        renderer: &mut Renderer,
        stats: Stats,
        history: HistoryCommit,
        report: RenderFrameReport,
    ) -> Stats {
        // No setup, fallible extraction, copy, allocation, admission or phase end
        // remains. Later native releases retain their lifetime observation policy.
        history.commit();
        if let Some(headless) = self.headless {
            headless.install();
        }
        renderer.stats = stats;
        *self.last_parameters = Some(self.parameters);
        renderer.frame_reports = Some(report);
        stats
    }
}

fn frame_setup(renderer: &Renderer) -> (&RendererCpu, RenderAccountingOptions) {
    (&renderer.cpu, renderer.options.accounting())
}
fn frame_canceled(renderer: &mut Renderer, report: RenderFrameReport) {
    renderer.frame_reports = Some(report);
}
struct ReadyFrame<'s> {
    publication: BoundPublication<'s>,
    history: HistoryCommit,
    _account: Account,
}
impl Renderer {
    pub(super) async fn render_frame(
        &mut self,
        surface: &mut Surface,
        scene: &Scene,
        parameters: Parameters,
    ) -> Result<Stats> {
        self.run_frame(surface, async |renderer, surface, account| {
            let (_, publication) = renderer
                .dispatch_render_frame(surface, scene, parameters, account)
                .await?;
            Ok(publication)
        })
        .await
    }

    #[cfg(test)]
    pub(super) async fn frame_operation(
        &mut self,
        surface: &mut Surface,
        run: impl AsyncFnOnce(&mut Self, &mut Surface, &Account) -> Result<RenderPublication>,
    ) -> Result<Stats> {
        self.run_frame(surface, run).await
    }

    async fn run_frame(
        &mut self,
        surface: &mut Surface,
        run: impl AsyncFnOnce(&mut Self, &mut Surface, &Account) -> Result<RenderPublication>,
    ) -> Result<Stats> {
        let finished = frame_lifecycle(
            self,
            frame_setup,
            frame_canceled,
            async move |renderer, mut attempt, account| {
                let (publication, draft, account) = account
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
                        let publication = run(renderer, &mut *surface, &account).await?;
                        Ok((publication, draft, account))
                    })
                    .await?;
                let mut publication = publication.bind(surface)?;
                // Hold the same surface through preparation. On refusal, the
                // draft drops after the Publish call returns.
                let preparation = account.phase(RenderPhase::Publish, || {
                    account.phase(RenderPhase::Cleanup, || {
                        drop(publication.cleanup.take());
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
            match Stats::from_frame_report(&report, publication.route, publication.precision()) {
                Ok(stats) => stats,
                Err(error) => {
                    drop(history);
                    drop(publication);
                    report.note_cleanup_fault(error.accounting_diagnostic().or(self.cpu.fault()));
                    self.frame_reports = Some(report);
                    return Err(error);
                }
            };
        Ok(publication.commit(self, stats, history, report))
    }
}
