use super::{
    BackendErrorCode, Error, PhysicalSize, Result,
    layout::{ReadbackLayout, decode_padded_rows},
    output::ReadbackOutput,
};
use crate::{
    RuntimeOperation,
    accounting::{
        Account, Generation, GenerationClass, GpuObjectKind, OwnedGpu, RenderPhase, RenderWork,
    },
    gpu_transaction::{GpuOperationTransaction, PendingReadbackSubmission},
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
#[cfg(not(target_arch = "wasm32"))]
use std::{thread, time::Duration};

#[path = "lifecycle.rs"]
mod lifecycle;
use lifecycle::{ReadbackStaging, ReadbackStagingMapState};

/// Size and validated layout are sealed before the actual native allocation.
pub(super) struct ReadbackPlan {
    size: PhysicalSize,
    layout: ReadbackLayout,
}
impl ReadbackPlan {
    pub(super) fn try_new(size: PhysicalSize) -> Result<Self> {
        Ok(Self {
            size,
            layout: ReadbackLayout::try_new(size)?,
        })
    }
    pub(super) fn allocate(
        self,
        generation: &Generation,
        device: &wgpu::Device,
        account: &Account,
    ) -> Result<ReadbackAllocation> {
        let buffer = generation.allocation(
            GenerationClass::ReadbackStaging,
            self.layout.buffer_size(),
            Some(account),
            || {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Surgeist scoped texture readback"),
                    size: self.layout.buffer_size(),
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            },
        )?;
        Ok(ReadbackAllocation { buffer, plan: self })
    }
}
pub(super) struct ReadbackAllocation {
    buffer: OwnedGpu<wgpu::Buffer>,
    plan: ReadbackPlan,
}
impl ReadbackAllocation {
    pub(super) fn bind(self, account: &Account) -> Readback<Allocated> {
        Readback {
            staging: ReadbackStaging::new(self.buffer, account),
            plan: self.plan,
            phase: Allocated,
        }
    }
    #[cfg(all(test, not(target_arch = "wasm32")))]
    fn submit_for_test(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        account: &Account,
    ) -> Result<SubmittedAllocation> {
        let command = encode_copy(&self.buffer, &self.plan, texture, device, account)?;
        let index = command.submit(queue, Some(account))?;
        Ok(SubmittedAllocation {
            allocation: self,
            index,
        })
    }
}
#[cfg(all(test, not(target_arch = "wasm32")))]
struct SubmittedAllocation {
    allocation: ReadbackAllocation,
    index: wgpu::SubmissionIndex,
}
#[cfg(all(test, not(target_arch = "wasm32")))]
impl SubmittedAllocation {
    fn bind(self, account: &Account) -> Readback<CopySubmitted<wgpu::SubmissionIndex>> {
        let Readback { staging, plan, .. } = self.allocation.bind(account);
        Readback {
            staging,
            plan,
            phase: CopySubmitted(self.index),
        }
    }
}
pub(super) struct Allocated;
pub(super) struct CopySubmitted<S>(S);
/// All constructors and fields stay within this native ownership boundary.
pub(super) struct Readback<P> {
    staging: ReadbackStaging,
    plan: ReadbackPlan,
    phase: P,
}
fn encode_copy(
    buffer: &OwnedGpu<wgpu::Buffer>,
    plan: &ReadbackPlan,
    texture: &wgpu::Texture,
    device: &wgpu::Device,
    account: &Account,
) -> Result<OwnedGpu<wgpu::CommandBuffer>> {
    let mut encoder =
        buffer
            .generation()
            .opaque(GpuObjectKind::CommandEncoder, Some(account), || {
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Surgeist scoped texture readback copy"),
                })
            })?;
    account.work(RenderWork::ReadbackCopies, 1)?;
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(plan.layout.padded_bytes_per_row()),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: plan.layout.width(),
            height: plan.layout.height(),
            depth_or_array_layers: 1,
        },
    );
    encoder.finish(Some(account))
}
impl Readback<Allocated> {
    pub(super) fn submit(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        transaction: GpuOperationTransaction,
    ) -> Result<Readback<CopySubmitted<PendingReadbackSubmission>>> {
        let account = self.staging.account();
        let command = encode_copy(&self.staging.buffer, &self.plan, texture, device, account)?;
        let pending = transaction.submit_readback(queue, command, account)?;
        Ok(Readback {
            staging: self.staging,
            plan: self.plan,
            phase: CopySubmitted(pending),
        })
    }
}
/// Scope resolution keeps its staging on both outcomes until the caller has
/// performed its existing backend terminal observation.
pub(super) struct ReadbackResolution(ResolvedCopy);
enum ResolvedCopy {
    Submitted(Readback<CopySubmitted<wgpu::SubmissionIndex>>),
    Failed {
        error: Error,
        staging: ReadbackStaging,
    },
}
impl Readback<CopySubmitted<PendingReadbackSubmission>> {
    pub(super) async fn resolve(
        self,
        operation: RuntimeOperation,
        account: &Account,
    ) -> ReadbackResolution {
        let Readback {
            staging,
            plan,
            phase: CopySubmitted(pending),
        } = self;
        match pending.finish(operation, account).await {
            Ok(submission) => ReadbackResolution(ResolvedCopy::Submitted(Readback {
                staging,
                plan,
                phase: CopySubmitted(submission.into_submission_index()),
            })),
            Err(error) => ReadbackResolution(ResolvedCopy::Failed { error, staging }),
        }
    }
}
impl ReadbackResolution {
    pub(super) fn into_result(self) -> Result<Readback<CopySubmitted<wgpu::SubmissionIndex>>> {
        match self.0 {
            ResolvedCopy::Submitted(owner) => Ok(owner),
            ResolvedCopy::Failed { error, staging } => {
                drop(staging);
                Err(error)
            }
        }
    }
}

pub(super) enum ReadbackCompletionResult {
    Map(std::result::Result<(), wgpu::BufferAsyncError>),
    #[cfg(not(target_arch = "wasm32"))]
    PollFailure(wgpu::PollError),
}
enum ReadbackCompletionStatus {
    Pending { waker: Option<Waker> },
    Ready(ReadbackCompletionResult),
    Consumed,
    Canceled,
}
pub(super) struct ReadbackCompletion {
    state: Mutex<ReadbackCompletionStatus>,
}
impl ReadbackCompletion {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(ReadbackCompletionStatus::Pending { waker: None }),
        }
    }
    pub(super) fn complete(&self, result: ReadbackCompletionResult) {
        let waker = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match std::mem::replace(&mut *state, ReadbackCompletionStatus::Consumed) {
                ReadbackCompletionStatus::Pending { waker } => {
                    *state = ReadbackCompletionStatus::Ready(result);
                    waker
                }
                terminal => {
                    *state = terminal;
                    None
                }
            }
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
    pub(super) fn poll(&self, context: &mut Context<'_>) -> Poll<ReadbackCompletionResult> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match std::mem::replace(&mut *state, ReadbackCompletionStatus::Consumed) {
            ReadbackCompletionStatus::Pending { .. } => {
                *state = ReadbackCompletionStatus::Pending {
                    waker: Some(context.waker().clone()),
                };
                Poll::Pending
            }
            ReadbackCompletionStatus::Ready(result) => Poll::Ready(result),
            terminal => {
                *state = terminal;
                Poll::Pending
            }
        }
    }
    pub(super) fn cancel(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(
            *state,
            ReadbackCompletionStatus::Pending { .. } | ReadbackCompletionStatus::Ready(_)
        ) {
            *state = ReadbackCompletionStatus::Canceled;
        }
    }
    #[cfg(test)]
    pub(super) fn is_canceled_for_test(&self) -> bool {
        matches!(
            *self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ReadbackCompletionStatus::Canceled
        )
    }
    #[cfg(test)]
    pub(super) fn result_will_be_accepted_for_test(&self) -> bool {
        self.is_pending()
    }
    #[cfg(any(test, not(target_arch = "wasm32")))]
    fn is_pending(&self) -> bool {
        matches!(
            *self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ReadbackCompletionStatus::Pending { .. }
        )
    }
}
/// Actual registration creates this single, consuming callback capability.
struct MapCallback {
    completion: Arc<ReadbackCompletion>,
    map: Arc<ReadbackStagingMapState>,
    staging: OwnedGpu<wgpu::Buffer>,
    #[cfg(all(test, not(target_arch = "wasm32")))]
    entry_gate: Option<Arc<pending_tests::PausedWake>>,
}
impl MapCallback {
    fn complete(self, result: std::result::Result<(), wgpu::BufferAsyncError>) {
        #[cfg(all(test, not(target_arch = "wasm32")))]
        if let Some(gate) = self.entry_gate {
            std::task::Wake::wake(gate);
        }
        self.map.completed(result.is_ok());
        self.completion
            .complete(ReadbackCompletionResult::Map(result));
        // Keep the native clone through wake return, including paused wakes.
        drop(self.staging);
    }
}
pub(super) fn completion_result(result: ReadbackCompletionResult) -> Result<()> {
    match result {
        ReadbackCompletionResult::Map(Ok(())) => Ok(()),
        ReadbackCompletionResult::Map(Err(source)) => Err(Error::new(
            BackendErrorCode::ReadbackFailed,
            "failed to map the texture readback staging buffer",
        )
        .with_source(source)),
        #[cfg(not(target_arch = "wasm32"))]
        ReadbackCompletionResult::PollFailure(source) => Err(Error::new(
            BackendErrorCode::ReadbackFailed,
            "the readback helper received the wrong submission index",
        )
        .with_source(source)),
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativePollAction {
    Continue,
    Stop,
}
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn handle_native_poll_result(
    completion: &ReadbackCompletion,
    result: std::result::Result<wgpu::PollStatus, wgpu::PollError>,
) -> NativePollAction {
    match result {
        Ok(_) | Err(wgpu::PollError::Timeout) => {
            if completion.is_pending() {
                NativePollAction::Continue
            } else {
                NativePollAction::Stop
            }
        }
        Err(source @ wgpu::PollError::WrongSubmissionIndex(_, _)) => {
            completion.complete(ReadbackCompletionResult::PollFailure(source));
            NativePollAction::Stop
        }
    }
}
#[cfg(all(test, not(target_arch = "wasm32")))]
thread_local! {
    static REFUSE_HELPER_SPAWN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CALLBACK_ENTRY_GATE: std::cell::RefCell<Option<Arc<pending_tests::PausedWake>>> = const { std::cell::RefCell::new(None) };
}
#[cfg(not(target_arch = "wasm32"))]
fn spawn_native_poll_helper(
    device: wgpu::Device,
    submission_index: wgpu::SubmissionIndex,
    completion: Arc<ReadbackCompletion>,
) -> std::io::Result<thread::JoinHandle<()>> {
    #[cfg(test)]
    if REFUSE_HELPER_SPAWN.with(|refuse| refuse.replace(false)) {
        return Err(std::io::Error::other(
            "injected native helper spawn failure",
        ));
    }
    thread::Builder::new()
        .name("surgeist-readback-poll".to_owned())
        .spawn(move || {
            while completion.is_pending() {
                let result = device.poll(wgpu::PollType::Wait {
                    submission_index: Some(submission_index.clone()),
                    timeout: Some(Duration::from_millis(50)),
                });
                if handle_native_poll_result(&completion, result) == NativePollAction::Stop {
                    break;
                }
            }
        })
}
struct RegisteredReadback {
    staging: ReadbackStaging,
    plan: ReadbackPlan,
    completion: Arc<ReadbackCompletion>,
}
struct MappedReadback {
    staging: ReadbackStaging,
    plan: ReadbackPlan,
}
impl MappedReadback {
    fn decode(self) -> Result<ReadbackOutput> {
        let account = self.staging.account().at(RenderPhase::ReadbackDecode);
        account.phase(RenderPhase::ReadbackDecode, || {
            let decoded = {
                #[cfg(test)]
                let _after_view = test_support::ViewDropProbe(self.staging.probe.0.clone());
                let view = self
                    .staging
                    .buffer
                    .slice(self.plan.layout.mapped_range())
                    .get_mapped_range();
                decode_padded_rows(&self.plan.layout, &view, &account)
            };
            let Self { staging, plan } = self;
            drop(staging);
            ReadbackOutput::try_new(plan.size, decoded?)
        })
    }
}
#[expect(
    clippy::large_enum_variant,
    reason = "the pending state retains mandatory staging inline without another allocation"
)]
enum MapFutureState {
    Pending(RegisteredReadback),
    Finished,
}
pub(super) struct ReadbackMapFuture {
    state: MapFutureState,
    #[cfg(not(target_arch = "wasm32"))]
    _poll_helper: Option<thread::JoinHandle<()>>,
}
impl ReadbackMapFuture {
    pub(super) fn start(
        owner: Readback<CopySubmitted<wgpu::SubmissionIndex>>,
        device: wgpu::Device,
    ) -> Result<Self> {
        let (registered, submission_index) = Self::register(owner)?;
        #[cfg(not(target_arch = "wasm32"))]
        let helper =
            match spawn_native_poll_helper(device, submission_index, registered.completion.clone())
            {
                Ok(helper) => helper,
                Err(source) => {
                    registered.completion.cancel();
                    return Err(Error::new(
                        BackendErrorCode::ReadbackFailed,
                        "failed to start the native readback progress helper",
                    )
                    .with_source(source));
                }
            };
        #[cfg(target_arch = "wasm32")]
        let _ = (device, submission_index);
        Ok(Self {
            state: MapFutureState::Pending(registered),
            #[cfg(not(target_arch = "wasm32"))]
            _poll_helper: Some(helper),
        })
    }
    fn register(
        owner: Readback<CopySubmitted<wgpu::SubmissionIndex>>,
    ) -> Result<(RegisteredReadback, wgpu::SubmissionIndex)> {
        owner
            .staging
            .account()
            .at(RenderPhase::ReadbackMap)
            .work(RenderWork::ReadbackMapRequests, 1)?;
        let Readback {
            staging,
            plan,
            phase: CopySubmitted(index),
        } = owner;
        let completion = Arc::new(ReadbackCompletion::new());
        let callback = MapCallback {
            completion: completion.clone(),
            map: staging.register(),
            staging: staging.buffer.clone(),
            #[cfg(all(test, not(target_arch = "wasm32")))]
            entry_gate: CALLBACK_ENTRY_GATE.with(|gate| gate.take()),
        };
        staging
            .buffer
            .slice(plan.layout.mapped_range())
            .map_async(wgpu::MapMode::Read, move |result| callback.complete(result));
        Ok((
            RegisteredReadback {
                staging,
                plan,
                completion,
            },
            index,
        ))
    }
    #[cfg(all(test, not(target_arch = "wasm32")))]
    fn start_manual(owner: Readback<CopySubmitted<wgpu::SubmissionIndex>>) -> Result<Self> {
        Self::register(owner).map(|(registered, _)| Self {
            state: MapFutureState::Pending(registered),
            _poll_helper: None,
        })
    }
}
impl Future for ReadbackMapFuture {
    type Output = Result<ReadbackOutput>;
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match std::mem::replace(&mut this.state, MapFutureState::Finished) {
            MapFutureState::Finished => Poll::Pending,
            MapFutureState::Pending(registered) => match registered.completion.poll(context) {
                Poll::Pending => {
                    this.state = MapFutureState::Pending(registered);
                    Poll::Pending
                }
                Poll::Ready(result) => {
                    if let Err(error) = completion_result(result) {
                        return Poll::Ready(Err(error));
                    }
                    let RegisteredReadback {
                        staging,
                        plan,
                        completion: _,
                    } = registered;
                    Poll::Ready(MappedReadback { staging, plan }.decode())
                }
            },
        }
    }
}
impl Drop for ReadbackMapFuture {
    fn drop(&mut self) {
        if let MapFutureState::Pending(registered) = &self.state {
            registered.completion.cancel();
        }
        // Pending staging fields drop only after logical cancellation.
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod pending_tests;

#[cfg(test)]
#[path = "test_support.rs"]
pub(super) mod test_support;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod custody_tests {
    use super::*;
    use crate::accounting::{Generation, GenerationClass, RenderAccountingError, RenderResource};

    #[test]
    fn canceled_callback_retains_staging_custody_until_delivery() {
        let generation = Generation::new(crate::RenderAccountingOptions::default()).unwrap();
        let callback = pending_tests::canceled_callback(&generation);
        assert_eq!(
            generation.usage().unwrap().live(RenderResource::GpuBytes),
            512,
        );
        assert_eq!(
            generation
                .class_usage(GenerationClass::ReadbackStaging)
                .unwrap()
                .live(RenderResource::GpuBytes),
            512,
        );
        callback();
        assert_eq!(
            generation.usage().unwrap().live(RenderResource::GpuBytes),
            0,
        );
        generation.ensure_healthy().unwrap();
    }

    #[test]
    fn retired_staging_custody_ignores_late_callback_drop() {
        let generation = Generation::new(crate::RenderAccountingOptions::default()).unwrap();
        let callback = pending_tests::canceled_callback(&generation);
        generation.retire();
        assert_eq!(
            generation.usage().unwrap().live(RenderResource::GpuBytes),
            0,
        );
        callback();
        generation.retire();
        assert_eq!(
            generation
                .class_usage(GenerationClass::ReadbackStaging)
                .unwrap()
                .live(RenderResource::GpuBytes),
            0,
        );
        assert_eq!(
            generation
                .ensure_healthy()
                .unwrap_err()
                .accounting_diagnostic(),
            Some(RenderAccountingError::GenerationRetired),
        );
    }
}
