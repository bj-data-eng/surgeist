use super::*;
use crate::{
    accounting::{FinishedReadback, ReadbackWork, RenderPhase, readback_lifecycle},
    readback::{ReadbackOutput, read_texture_rgba},
};

fn readback_setup(
    renderer: &Renderer,
) -> (&crate::accounting::RendererCpu, RenderAccountingOptions) {
    (&renderer.cpu, renderer.options.accounting())
}
fn readback_canceled(renderer: &mut Renderer, report: RenderReadbackReport) {
    renderer.readback_reports = Some(report);
}

impl Renderer {
    /// Performs explicit headless readback of the current complete publication.
    ///
    /// The returned [`ImageBuffer`] contains tightly packed straight-alpha RGBA8
    /// physical pixels. A zero-area available surface returns an empty validated
    /// image without GPU work. A nonzero surface without a publication returns
    /// its typed uninitialized diagnostic. Failed or canceled mapping never
    /// changes the published frame. The future is not promised to be `Send`.
    ///
    /// The attempt starts on first poll. Its separate latest-readback report
    /// closes while complete output remains guarded, before final caller transfer.
    pub async fn read_headless(&mut self, surface: &Surface) -> Result<ImageBuffer> {
        let finished = readback_lifecycle(
            self,
            readback_setup,
            readback_canceled,
            async |renderer, mut attempt, account| {
                account
                    .async_phase(
                        RenderPhase::Readback,
                        renderer.read_headless_inner(surface, &mut attempt),
                    )
                    .await
            },
        )
        .await?;
        self.finish_readback(finished)
    }

    async fn read_headless_inner(
        &mut self,
        surface: &Surface,
        attempt: &mut ReadbackWork<'_>,
    ) -> Result<ReadbackOutput> {
        let account = attempt.account().clone();
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Readback)?;
        match &surface.backend {
            SurfaceBackend::ContractOnly { physical_size } => {
                surface.ensure_available(SurfaceIoOperation::Readback)?;
                if physical_size.width() == 0 || physical_size.height() == 0 {
                    return ReadbackOutput::empty(*physical_size, &account);
                }
                Err(Error::runtime_unavailable(
                    RuntimeCapabilityUnavailable::readback_adapter_unavailable(),
                    "no compatible wgpu adapter is available",
                ))
            }
            SurfaceBackend::Headless {
                device_identity,
                resources,
                physical_size,
            } => {
                let device_identity = *device_identity;
                let backend = self.surface_backend_for_device(
                    device_identity,
                    SurfaceDiagnosticOperation::Readback,
                )?;
                surface.ensure_available(SurfaceIoOperation::Readback)?;
                let texture = match resources {
                    HeadlessResources::Empty => {
                        return ReadbackOutput::empty(*physical_size, &account);
                    }
                    HeadlessResources::Pending => {
                        return Err(Error::runtime_unavailable(
                            RuntimeCapabilityUnavailable::readback_uninitialized(),
                            "headless surface has no published texture",
                        ));
                    }
                    HeadlessResources::Ready { texture, .. } => texture,
                };
                if let Some(error) =
                    backend.terminal_error(device_identity, RuntimeOperation::SurfaceReadback)
                {
                    return Err(error);
                }
                attempt.attach_generation(backend.generation(device_identity)?)?;
                read_texture_rgba(
                    backend,
                    device_identity,
                    texture,
                    *physical_size,
                    RuntimeOperation::SurfaceReadback,
                    &account,
                )
                .await
            }
            #[cfg(any(
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            SurfaceBackend::Presented { .. } => Err(unsupported_surface_backend()),
        }
    }

    pub(super) fn finish_readback(
        &mut self,
        finished: FinishedReadback<ReadbackOutput>,
    ) -> Result<ImageBuffer> {
        let (result, started, mut report, fault) = finished.into_parts();
        let result = match result {
            Err(error) => Err(error),
            Ok(output) => {
                if let Some(fault) = fault {
                    drop(output);
                    let error = fault.into();
                    report.fail_after_close(&error, self.cpu.fault());
                    Err(error)
                } else {
                    match output.transfer() {
                        Ok(image) => Ok(image),
                        Err(error) => {
                            report.fail_after_close(&error, None);
                            Err(error)
                        }
                    }
                }
            }
        };
        // A successful transfer permits only these infallible moves. In
        // particular, no phase or snapshot reads occur after output handoff.
        if started {
            self.readback_reports = Some(report);
        }
        result
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod accounting_public_tests;
