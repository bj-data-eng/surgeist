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
pub(crate) use readback::PendingReadbackSubmission;
#[cfg_attr(
    not(test),
    expect(
        unused_imports,
        reason = "preserves the existing crate-visible readback transaction front-door paths"
    )
)]
pub(crate) use readback::ReadbackSubmission;
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
    #[cfg(all(test, not(target_arch = "wasm32")))]
    cleanup_time: Option<std::rc::Rc<std::cell::Cell<u64>>>,
}

impl GpuOperationLease {
    pub(crate) fn begin(signal: Arc<DeviceSignal>, generation: u64) -> Self {
        signal.activate(generation);
        Self {
            signal,
            generation,
            #[cfg(all(test, not(target_arch = "wasm32")))]
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
        #[cfg(all(test, not(target_arch = "wasm32")))]
        if let Some(time) = &self.cleanup_time {
            time.set(time.get() + 7);
        }
        #[cfg(all(test, feature = "render-window", not(target_arch = "wasm32")))]
        test_support::observe_lease_drop_for_test(self);
    }
}

/// Real scopes in reverse push order, including during partial async moves.
#[must_use = "active GPU scopes must resolve or close on cancellation"]
struct ActiveGpuScopes {
    validation: wgpu::ErrorScopeGuard,
    out_of_memory: wgpu::ErrorScopeGuard,
    internal: wgpu::ErrorScopeGuard,
    #[cfg(all(test, not(target_arch = "wasm32")))]
    scope_gate: Option<std::rc::Rc<test_support::ScopePopGate>>,
}

impl ActiveGpuScopes {
    fn begin(device: &wgpu::Device) -> Self {
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let out_of_memory = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        Self {
            validation,
            out_of_memory,
            internal,
            #[cfg(all(test, not(target_arch = "wasm32")))]
            scope_gate: None,
        }
    }

    async fn pop(self) -> Option<wgpu::Error> {
        // Keep unpopped guards in this partially moved aggregate. Cancellation
        // drops the pending native result and remaining guards before the lease
        // held by the enclosing operation; do not destructure or pop eagerly.
        let validation = self.validation.pop();
        #[cfg(all(test, not(target_arch = "wasm32")))]
        if let Some(gate) = &self.scope_gate {
            gate.wait(test_support::ScopePopPosition::Validation).await;
        }
        let validation = validation.await;
        let out_of_memory = self.out_of_memory.pop();
        #[cfg(all(test, not(target_arch = "wasm32")))]
        if let Some(gate) = &self.scope_gate {
            gate.wait(test_support::ScopePopPosition::OutOfMemory).await;
        }
        let out_of_memory = out_of_memory.await;
        let internal = self.internal.pop();
        #[cfg(all(test, not(target_arch = "wasm32")))]
        if let Some(gate) = &self.scope_gate {
            gate.wait(test_support::ScopePopPosition::Internal).await;
        }
        let internal = internal.await;

        [validation, out_of_memory, internal]
            .into_iter()
            .flatten()
            .next()
    }
}

/// Nested WGPU scopes precede the lease so cancellation closes them first.
#[must_use = "GPU operation transactions must resolve scopes before publishing state"]
pub(crate) struct GpuOperationTransaction {
    scopes: ActiveGpuScopes,
    lease: GpuOperationLease,
    stage: GpuOperationStage,
}

/// Clean submission ownership retained through graph resource authorization.
#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
#[must_use = "clean graph submissions must begin presentation or abort on drop"]
struct CleanSubmissionContinuation {
    lease: GpuOperationLease,
    stage: GpuOperationStage,
}

impl GpuOperationTransaction {
    pub(crate) fn begin(
        device: &wgpu::Device,
        signal: Arc<DeviceSignal>,
        generation: u64,
        stage: GpuOperationStage,
    ) -> Self {
        let lease = GpuOperationLease::begin(signal, generation);
        Self {
            scopes: ActiveGpuScopes::begin(device),
            lease,
            stage,
        }
    }

    #[cfg(any(
        feature = "render-window",
        all(feature = "render-web", target_arch = "wasm32")
    ))]
    /// Resolve owns scope and lease cleanup on cancellation, inside the observed
    /// phase. The former borrowed resolver left remaining cleanup to its caller
    /// after Resolve closed. Clean success instead retains the lease for Present.
    async fn resolve_submission_phase(
        self,
        operation: RuntimeOperation,
        account: &Account,
    ) -> Result<CleanSubmissionContinuation> {
        account
            .async_phase(RenderPhase::Resolve, async move {
                let captured = self.scopes.pop().await;
                terminal_result(
                    &self.lease,
                    self.stage,
                    self.lease.signal.first_terminal(),
                    operation,
                )?;
                if let Some(error) = captured {
                    return Err(classify_captured_error(self.stage, error));
                }
                Ok(CleanSubmissionContinuation {
                    lease: self.lease,
                    stage: self.stage,
                })
            })
            .await
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

    async fn finish_inner(self, operation: RuntimeOperation) -> Result<()> {
        let captured = self.scopes.pop().await;

        terminal_result(&self.lease, self.stage, self.lease.finish(), operation)?;
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

#[cfg(any(
    feature = "render-window",
    all(feature = "render-web", target_arch = "wasm32")
))]
impl CleanSubmissionContinuation {
    fn begin_present_phase(
        self,
        device: &wgpu::Device,
        operation: RuntimeOperation,
    ) -> Result<GpuOperationTransaction> {
        terminal_result(
            &self.lease,
            self.stage,
            self.lease.signal.first_terminal(),
            operation,
        )?;
        Ok(GpuOperationTransaction {
            scopes: ActiveGpuScopes::begin(device),
            lease: self.lease,
            stage: GpuOperationStage::Present,
        })
    }
}

fn terminal_result(
    lease: &GpuOperationLease,
    stage: GpuOperationStage,
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
        } if *generation == lease.generation() => Err(stage.classify_fault(*kind, message)),
        DeviceTerminalSignal::Faulted { .. } => Err(terminal.error(operation)),
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
