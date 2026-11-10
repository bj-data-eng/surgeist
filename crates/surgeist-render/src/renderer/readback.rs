use super::*;
use crate::{
    accounting::{ReadbackAttempt, RenderAttemptStatus, RenderPhase},
    readback::{ReadbackOutput, read_texture_rgba},
};

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
        let mut attempt = ReadbackAttempt::begin(
            self.readback_reports.clone(),
            &self.cpu,
            self.options.accounting(),
        )?;
        let account = attempt.account().clone();
        let result = account
            .async_phase(
                RenderPhase::Readback,
                self.read_headless_inner(surface, &mut attempt),
            )
            .await;
        self.finish_readback(attempt, result)
    }

    async fn read_headless_inner(
        &mut self,
        surface: &Surface,
        attempt: &mut ReadbackAttempt,
    ) -> Result<ReadbackOutput> {
        let account = attempt.account().clone();
        self.validate_surface_renderer_identity(surface, SurfaceDiagnosticOperation::Readback)?;
        self.validate_surface_operation_backend(surface, SurfaceDiagnosticOperation::Readback)?;
        self.validate_surface_device_identity(surface, SurfaceDiagnosticOperation::Readback)?;
        surface.ensure_available(SurfaceIoOperation::Readback)?;
        let (device_identity, texture, physical_size) = match &surface.backend {
            SurfaceBackend::ContractOnly { physical_size }
                if physical_size.width() == 0 || physical_size.height() == 0 =>
            {
                return ReadbackOutput::empty(*physical_size, &account);
            }
            SurfaceBackend::ContractOnly { .. } => {
                return Err(Error::runtime_unavailable(
                    RuntimeCapabilityUnavailable::readback_adapter_unavailable(),
                    "no compatible wgpu adapter is available",
                ));
            }
            SurfaceBackend::Headless {
                physical_size,
                resources: HeadlessResources::Empty,
                ..
            } => {
                return ReadbackOutput::empty(*physical_size, &account);
            }
            SurfaceBackend::Headless {
                resources: HeadlessResources::Pending,
                ..
            } => {
                return Err(Error::runtime_unavailable(
                    RuntimeCapabilityUnavailable::readback_uninitialized(),
                    "headless surface has no published texture",
                ));
            }
            SurfaceBackend::Headless {
                device_identity,
                resources: HeadlessResources::Ready { texture, .. },
                physical_size,
            } => (*device_identity, texture, *physical_size),
            #[cfg(any(
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            SurfaceBackend::Presented { .. } => unreachable!(),
        };
        self.validate_surface_device_terminal(surface, RuntimeOperation::SurfaceReadback)?;
        let Some(backend) = self.backend.as_mut() else {
            return Err(Error::runtime_unavailable(
                RuntimeCapabilityUnavailable::readback_adapter_unavailable(),
                "no compatible wgpu adapter is available",
            ));
        };
        attempt.attach_generation(backend.generation(device_identity)?)?;
        read_texture_rgba(
            backend,
            device_identity,
            texture,
            physical_size,
            RuntimeOperation::SurfaceReadback,
            &account,
        )
        .await
    }

    pub(super) fn finish_readback(
        &mut self,
        attempt: ReadbackAttempt,
        result: Result<ReadbackOutput>,
    ) -> Result<ImageBuffer> {
        let started = attempt.started();
        let (status, diagnostic) = match &result {
            Ok(_) => (RenderAttemptStatus::Succeeded, None),
            Err(error) => (attempt.status_for(error), error.accounting_diagnostic()),
        };
        let (mut report, fault) = attempt.finish(status, diagnostic);
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
            self.readback_reports.store(report);
        }
        result
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod accounting_public_tests;
