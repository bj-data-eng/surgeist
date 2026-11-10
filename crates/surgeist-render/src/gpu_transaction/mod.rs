use crate::accounting::{Account, RenderPhase};
mod graph;
mod readback;
#[cfg(test)]
mod scope_tests;
#[cfg(test)]
pub(crate) mod test_support;
mod vello;

#[expect(
    unused_imports,
    reason = "preserves the existing crate-visible transaction front-door path"
)]
pub(crate) use graph::GraphSubmissionCommit;
pub(crate) use graph::{GraphOutputCommit, GraphSubmissionPayload};
#[expect(
    unused_imports,
    reason = "preserves the existing crate-visible readback transaction front-door paths"
)]
pub(crate) use readback::{PendingReadbackSubmission, ReadbackSubmission};
pub(crate) use vello::{InternalVelloPayload, VelloResourceCommitProof};

use super::{
    BackendErrorCode, Error, GpuFaultKind, Result, RuntimeOperation,
    backend::{DeviceSignal, DeviceTerminalSignal},
};

use std::sync::Arc;

/// Private ownership stage for a render-owned GPU operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GpuOperationStage {
    Render,
    Readback,
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    Configure,
    #[cfg(any(
        test,
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    Present,
}

impl GpuOperationStage {
    pub(crate) const fn error_code(self) -> BackendErrorCode {
        match self {
            Self::Render => BackendErrorCode::RenderFailed,
            Self::Readback => BackendErrorCode::ReadbackFailed,
            #[cfg(any(
                test,
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::Configure => BackendErrorCode::SurfaceConfigureFailed,
            #[cfg(any(
                test,
                feature = "render-window",
                all(feature = "render-web", target_arch = "wasm32")
            ))]
            Self::Present => BackendErrorCode::PresentFailed,
        }
    }

    fn classify_fault(self, kind: GpuFaultKind, message: &str) -> Error {
        let code = if kind == GpuFaultKind::OutOfMemory {
            BackendErrorCode::SurfaceOutOfMemory
        } else {
            self.error_code()
        };
        Error::new(code, message)
    }

    #[cfg(test)]
    pub(crate) fn classify_fault_for_test(self, kind: GpuFaultKind, message: &str) -> Error {
        self.classify_fault(kind, message)
    }
}

/// Test-owned sentinel publication committed after the real transaction resolves.
#[cfg(test)]
#[must_use = "GPU draft state must be committed or dropped"]
pub(crate) struct GpuOperationDraft<'a, T> {
    target: &'a mut Option<T>,
    value: Option<T>,
}

#[cfg(test)]
impl<'a, T> GpuOperationDraft<'a, T> {
    pub(crate) fn new(target: &'a mut Option<T>, value: T) -> Self {
        Self {
            target,
            value: Some(value),
        }
    }

    pub(crate) fn commit(mut self) {
        *self.target = self.value.take();
    }
}

/// Owns one active generation and clears only that generation on every exit.
#[must_use = "GPU operation leases must remain alive until scopes resolve"]
pub(crate) struct GpuOperationLease {
    signal: Arc<DeviceSignal>,
    generation: u64,
    #[cfg(test)]
    cleanup_time: Option<std::rc::Rc<std::cell::Cell<u64>>>,
}

impl GpuOperationLease {
    pub(crate) fn begin(signal: Arc<DeviceSignal>, generation: u64) -> Self {
        signal.activate(generation);
        Self {
            signal,
            generation,
            #[cfg(test)]
            cleanup_time: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn begin_for_test(signal: &Arc<DeviceSignal>) -> Result<Self> {
        let generation = signal.next_test_generation()?;
        Ok(Self::begin(Arc::clone(signal), generation))
    }

    const fn generation(&self) -> u64 {
        self.generation
    }

    fn finish(&self) -> Option<Arc<DeviceTerminalSignal>> {
        self.signal.finish_active_generation(self.generation)
    }

    #[cfg(test)]
    pub(crate) const fn generation_for_test(&self) -> u64 {
        self.generation
    }
}

impl Drop for GpuOperationLease {
    fn drop(&mut self) {
        self.signal.clear_active(self.generation);
        #[cfg(test)]
        if let Some(time) = &self.cleanup_time {
            time.set(time.get() + 7);
        }
    }
}

/// Nested WGPU scopes and the active-generation lease for one operation.
///
/// Scope fields are stored in reverse pop order. The explicit `Drop` below
/// preserves that order when an async operation future is canceled.
#[must_use = "GPU operation transactions must resolve scopes before publishing state"]
pub(crate) struct GpuOperationTransaction {
    validation: Option<wgpu::ErrorScopeGuard>,
    out_of_memory: Option<wgpu::ErrorScopeGuard>,
    internal: Option<wgpu::ErrorScopeGuard>,
    lease: GpuOperationLease,
    stage: GpuOperationStage,
    #[cfg(test)]
    scope_gate: Option<std::rc::Rc<test_support::ScopePopGate>>,
}

impl GpuOperationTransaction {
    pub(crate) fn begin(
        device: &wgpu::Device,
        signal: Arc<DeviceSignal>,
        generation: u64,
        stage: GpuOperationStage,
    ) -> Self {
        let lease = GpuOperationLease::begin(signal, generation);
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let out_of_memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        Self {
            validation: Some(validation),
            out_of_memory: Some(out_of_memory),
            internal: Some(internal),
            lease,
            stage,
            #[cfg(test)]
            scope_gate: None,
        }
    }

    async fn pop_active_scopes(&mut self) -> Option<wgpu::Error> {
        let validation = self
            .validation
            .take()
            .expect("transaction validation scope must be present")
            .pop();
        #[cfg(test)]
        if let Some(gate) = &self.scope_gate {
            gate.wait(test_support::ScopePopPosition::Validation).await;
        }
        let validation = validation.await;
        let out_of_memory = self
            .out_of_memory
            .take()
            .expect("transaction out-of-memory scope must be present")
            .pop();
        #[cfg(test)]
        if let Some(gate) = &self.scope_gate {
            gate.wait(test_support::ScopePopPosition::OutOfMemory).await;
        }
        let out_of_memory = out_of_memory.await;
        let internal = self
            .internal
            .take()
            .expect("transaction internal scope must be present")
            .pop();
        #[cfg(test)]
        if let Some(gate) = &self.scope_gate {
            gate.wait(test_support::ScopePopPosition::Internal).await;
        }
        let internal = internal.await;

        [validation, out_of_memory, internal]
            .into_iter()
            .flatten()
            .next()
    }

    fn terminal_result(
        &self,
        terminal: Option<Arc<DeviceTerminalSignal>>,
        operation: RuntimeOperation,
    ) -> Result<()> {
        let Some(terminal) = terminal else {
            return Ok(());
        };
        match terminal.as_ref() {
            DeviceTerminalSignal::Lost { .. } => Err(terminal.error(operation)),
            DeviceTerminalSignal::Faulted {
                kind,
                message,
                operation_generation: Some(generation),
            } if *generation == self.lease.generation() => {
                Err(self.stage.classify_fault(*kind, message))
            }
            DeviceTerminalSignal::Faulted { .. } => Err(terminal.error(operation)),
        }
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    async fn resolve_submission_phase(
        &mut self,
        operation: RuntimeOperation,
        account: &Account,
    ) -> Result<()> {
        account
            .async_phase(RenderPhase::Resolve, async {
                let captured = self.pop_active_scopes().await;
                self.terminal_result(self.lease.signal.first_terminal(), operation)?;
                if let Some(error) = captured {
                    return Err(classify_captured_error(self.stage, error));
                }
                Ok(())
            })
            .await
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    fn begin_present_phase(
        &mut self,
        device: &wgpu::Device,
        operation: RuntimeOperation,
    ) -> Result<()> {
        if self.validation.is_some() || self.out_of_memory.is_some() || self.internal.is_some() {
            return Err(Error::new(
                BackendErrorCode::PresentFailed,
                "the graph presentation phase started before submission scopes resolved",
            ));
        }
        self.terminal_result(self.lease.signal.first_terminal(), operation)?;
        self.stage = GpuOperationStage::Present;
        self.internal = Some(device.push_error_scope(wgpu::ErrorFilter::Internal));
        self.out_of_memory = Some(device.push_error_scope(wgpu::ErrorFilter::OutOfMemory));
        self.validation = Some(device.push_error_scope(wgpu::ErrorFilter::Validation));
        Ok(())
    }

    /// Resolves all error scopes before the caller may publish its draft state.
    pub(crate) async fn finish(
        self,
        operation: RuntimeOperation,
        observer: Option<&Account>,
    ) -> Result<()> {
        match observer {
            Some(account) => {
                let phase = if matches!(self.stage, GpuOperationStage::Readback) {
                    RenderPhase::ReadbackResolve
                } else {
                    RenderPhase::Resolve
                };
                account
                    .async_phase(phase, self.finish_inner(operation))
                    .await
            }
            None => self.finish_inner(operation).await,
        }
    }

    async fn finish_inner(mut self, operation: RuntimeOperation) -> Result<()> {
        let captured = self.pop_active_scopes().await;

        self.terminal_result(self.lease.finish(), operation)?;
        if let Some(error) = captured {
            return Err(classify_captured_error(self.stage, error));
        }
        Ok(())
    }

    /// Submits output work and applies its non-rollbackable host effect while scoped.
    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    pub(crate) async fn submit_command_buffer_with_host_effect(
        self,
        queue: &wgpu::Queue,
        command_buffer: crate::accounting::OwnedGpu<wgpu::CommandBuffer>,
        host_effect: impl FnOnce(),
        operation: RuntimeOperation,
        account: &Account,
    ) -> Result<()> {
        account.phase(RenderPhase::Submit, || {
            command_buffer
                .submit(queue, Some(&account.at(RenderPhase::Submit)))
                .map(|_| ())
        })?;
        account.phase(RenderPhase::Present, || {
            account
                .at(RenderPhase::Present)
                .work(crate::accounting::RenderWork::Presentations, 1)?;
            host_effect();
            Ok(())
        })?;
        self.finish(operation, Some(account)).await
    }
}

impl Drop for GpuOperationTransaction {
    fn drop(&mut self) {
        drop(self.validation.take());
        drop(self.out_of_memory.take());
        drop(self.internal.take());
    }
}

fn classify_captured_error(stage: GpuOperationStage, error: wgpu::Error) -> Error {
    let kind = match error {
        wgpu::Error::Validation { .. } => GpuFaultKind::Validation,
        wgpu::Error::OutOfMemory { .. } => GpuFaultKind::OutOfMemory,
        wgpu::Error::Internal { .. } => GpuFaultKind::Internal,
    };
    stage
        .classify_fault(kind, &error.to_string())
        .with_source(error)
}
