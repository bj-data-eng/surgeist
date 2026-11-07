use self::{layout::ReadbackLayout, lifecycle::ReadbackOwner, native::ReadbackMapFuture};
use super::{
    BackendErrorCode, Error, PhysicalSize, Result, RuntimeOperation,
    accounting::{Account, GenerationClass, GpuObjectKind, RenderPhase, RenderWork},
    backend::{Backend, DeviceSlotIdentity},
    gpu_transaction::GpuOperationStage,
};
#[cfg(test)]
use crate::ImageBuffer;

mod layout;
mod lifecycle;
mod native;
mod output;
pub(crate) use output::ReadbackOutput;
#[cfg(test)]
mod accounting_tests;
#[cfg(test)]
mod test_support;

#[cfg(test)]
pub(crate) use test_support::{
    ReadbackCleanupEventForTest, ReadbackCompletionForTest, ReadbackPhaseForTest,
    ReadbackStagingDispositionForTest, ReadbackStateMachineForTest,
};

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) use test_support::{
    NativeReadbackLateCallbackStageForTest, NativeReadbackStageForTest,
    NativeReadbackStagePhaseForTest,
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
    let layout = match account.phase(RenderPhase::ReadbackPrepare, || {
        ReadbackLayout::try_new(physical_size)
    }) {
        Ok(layout) => layout,
        Err(error) => {
            let scope_result = transaction.finish(operation, Some(account)).await;
            backend.observe_device_terminal(device_identity);
            scope_result?;
            return Err(error);
        }
    };
    let (mut owner, pending_submission) = account.phase(RenderPhase::ReadbackPrepare, || {
        let account = &account.at(RenderPhase::ReadbackPrepare);
        let generation = backend.generation(device_identity)?.clone();
        let (device, queue) = backend.gpu_operation_device_queue(
            device_identity,
            operation,
            GpuOperationStage::Readback,
        )?;
        let buffer = generation.allocation(
            GenerationClass::ReadbackStaging,
            layout.buffer_size(),
            Some(account),
            || {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist scoped texture readback"),
                    size: layout.buffer_size(),
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            },
        )?;
        let mut owner = ReadbackOwner::allocated(buffer, layout, physical_size, account);
        let mut encoder =
            generation.opaque(GpuObjectKind::CommandEncoder, Some(account), || {
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surgeist scoped texture readback copy"),
                })
            })?;
        account.work(RenderWork::ReadbackCopies, 1)?;
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: owner.staging(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(owner.layout.padded_bytes_per_row()),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: owner.layout.width(),
                height: owner.layout.height(),
                depth_or_array_layers: 1,
            },
        );
        let pending_submission =
            transaction.submit_readback(queue, encoder.finish(Some(account))?, account)?;
        let submission_index = pending_submission.submission_index();
        owner.copy_submitted(submission_index);
        Ok((owner, pending_submission))
    })?;

    let submission_result = pending_submission.finish(operation, account).await;
    backend.observe_device_terminal(device_identity);
    let submission = match submission_result {
        Ok(submission) => submission,
        Err(error) => {
            owner.fail();
            return Err(error);
        }
    };

    let device = match backend.gpu_operation_device_queue(
        device_identity,
        operation,
        GpuOperationStage::Readback,
    ) {
        Ok((device, _)) => device.clone(),
        Err(error) => {
            owner.fail();
            return Err(error);
        }
    };
    let readback_result = account
        .async_phase(RenderPhase::ReadbackMap, async {
            ReadbackMapFuture::start(owner, device, submission.into_submission_index())?.await
        })
        .await;

    backend.observe_device_terminal(device_identity);
    if let Some(error) = backend.terminal_error(device_identity, operation) {
        return Err(error);
    }
    readback_result
}
