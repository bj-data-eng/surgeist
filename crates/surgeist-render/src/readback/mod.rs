use self::native::{ReadbackMapFuture, ReadbackPlan};
use super::{
    BackendErrorCode, Error, PhysicalSize, Result, RuntimeOperation,
    accounting::{Account, RenderPhase},
    backend::{Backend, DeviceSlotIdentity},
    gpu_transaction::GpuOperationStage,
};

mod layout;
mod native;
mod output;
pub(crate) use output::ReadbackOutput;
#[cfg(test)]
mod accounting_tests;
#[cfg(test)]
use native::test_support;

#[cfg(test)]
pub(crate) use test_support::ReadbackCompletionForTest;

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) use test_support::{
    NativeReadbackLateCallbackStageForTest, NativeReadbackStageForTest,
    NativeReadbackStagePhaseForTest, ReadbackCleanupEventForTest, ReadbackPhaseForTest,
    ReadbackStagingDispositionForTest, ReadbackStateMachineForTest,
};

pub(crate) async fn read_texture_rgba(
    backend: &mut Backend,
    device_identity: DeviceSlotIdentity,
    texture: &wgpu::Texture,
    physical_size: PhysicalSize,
    operation: RuntimeOperation,
    account: &Account,
) -> Result<ReadbackOutput> {
    if physical_size.width() == 0 || physical_size.height() == 0 {
        return ReadbackOutput::empty(physical_size, account);
    }

    let transaction =
        backend.begin_gpu_operation(device_identity, GpuOperationStage::Readback, operation)?;
    let plan = match account.phase(RenderPhase::ReadbackPrepare, || {
        ReadbackPlan::try_new(physical_size)
    }) {
        Ok(layout) => layout,
        Err(error) => {
            let scope_result = transaction.finish(operation, Some(account)).await;
            backend.observe_device_terminal(device_identity);
            scope_result?;
            return Err(error);
        }
    };
    let submitted = account.phase(RenderPhase::ReadbackPrepare, || {
        let account = &account.at(RenderPhase::ReadbackPrepare);
        let generation = backend.generation(device_identity)?.clone();
        let (device, queue) = backend.gpu_operation_device_queue(
            device_identity,
            operation,
            GpuOperationStage::Readback,
        )?;
        plan.allocate(&generation, device, account)?
            .bind(account)
            .submit(device, queue, texture, transaction)
    })?;
    let resolution = submitted.resolve(operation, account).await;
    backend.observe_device_terminal(device_identity);
    let owner = resolution.into_result()?;

    let device = match backend.gpu_operation_device_queue(
        device_identity,
        operation,
        GpuOperationStage::Readback,
    ) {
        Ok((device, _)) => device.clone(),
        Err(error) => return Err(error),
    };
    let readback_result = account
        .async_phase(RenderPhase::ReadbackMap, async {
            ReadbackMapFuture::start(owner, device)?.await
        })
        .await;

    backend.observe_device_terminal(device_identity);
    if let Some(error) = backend.terminal_error(device_identity, operation) {
        return Err(error);
    }
    readback_result
}
