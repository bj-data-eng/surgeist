use super::*;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Weak;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadbackPhaseForTest {
    Allocated,
    CopySubmitted { submission_index: u64 },
    MapPending,
    Mapped,
    PublishedBytes,
    Failed,
    Canceled,
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadbackStagingDispositionForTest {
    Idle,
    MapPending,
    MappedActive,
    Released,
}
#[cfg(not(target_arch = "wasm32"))]
impl From<lifecycle::ReadbackStagingDisposition> for ReadbackStagingDispositionForTest {
    fn from(value: lifecycle::ReadbackStagingDisposition) -> Self {
        use lifecycle::ReadbackStagingDisposition as D;
        match value {
            D::Idle => Self::Idle,
            D::MapPending => Self::MapPending,
            D::MappedActive => Self::MappedActive,
            D::Released => Self::Released,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReadbackCleanupEventForTest {
    MappedViewDropped,
    StagingUnmapped,
    StagingDropped,
    #[cfg(not(target_arch = "wasm32"))]
    PublishedBytes,
}
#[derive(Clone, Default)]
pub(super) struct CleanupObservation {
    events: Arc<Mutex<Vec<ReadbackCleanupEventForTest>>>,
    hook: Option<fn(ReadbackCleanupEventForTest)>,
}
impl CleanupObservation {
    pub(super) fn record(&self, event: ReadbackCleanupEventForTest) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
        if let Some(hook) = self.hook {
            hook(event);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn with_hook(hook: fn(ReadbackCleanupEventForTest)) -> Self {
        Self {
            hook: Some(hook),
            ..Self::default()
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn events(&self) -> Vec<ReadbackCleanupEventForTest> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}
#[derive(Default)]
pub(super) struct BufferDropProbe(pub(super) Option<CleanupObservation>);
impl Drop for BufferDropProbe {
    fn drop(&mut self) {
        if let Some(observation) = &self.0 {
            observation.record(ReadbackCleanupEventForTest::StagingDropped);
        }
    }
}
pub(super) struct ViewDropProbe(pub(super) Option<CleanupObservation>);
impl Drop for ViewDropProbe {
    fn drop(&mut self) {
        if let Some(observation) = &self.0 {
            observation.record(ReadbackCleanupEventForTest::MappedViewDropped);
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
struct ObservedView {
    view: wgpu::BufferView,
    _after_view: ViewDropProbe,
}

/// Pure cell saturation fixture; offers are not native callback capabilities.
pub(crate) struct ReadbackCompletionForTest {
    completion: ReadbackCompletion,
    counts: Mutex<(usize, usize)>,
}
impl ReadbackCompletionForTest {
    pub(crate) fn new() -> Self {
        Self {
            completion: ReadbackCompletion::new(),
            counts: Mutex::new((0, 0)),
        }
    }
    pub(crate) fn poll_for_test(&self, context: &mut Context<'_>) -> Poll<Result<()>> {
        self.completion.poll(context).map(completion_result)
    }
    pub(crate) fn offer_map_result_for_test(
        &self,
        result: std::result::Result<(), wgpu::BufferAsyncError>,
    ) {
        let accepted = self.completion.result_will_be_accepted_for_test();
        self.completion
            .complete(ReadbackCompletionResult::Map(result));
        self.record(accepted);
    }
    pub(crate) fn deliver_late_map_result_for_test(
        &self,
        result: std::result::Result<(), wgpu::BufferAsyncError>,
    ) {
        self.offer_map_result_for_test(result);
    }
    pub(crate) fn cancel_for_test(&self) {
        self.completion.cancel();
    }
    pub(crate) fn is_canceled_for_test(&self) -> bool {
        self.completion.is_canceled_for_test()
    }
    pub(crate) fn accepted_result_count_for_test(&self) -> usize {
        self.counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .0
    }
    pub(crate) fn discarded_result_count_for_test(&self) -> usize {
        self.counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .1
    }
    fn record(&self, accepted: bool) {
        let mut counts = self
            .counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if accepted {
            counts.0 += 1;
        } else {
            counts.1 += 1;
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn timeout_slice_for_test(&self) -> bool {
        handle_native_poll_result(&self.completion, Err(wgpu::PollError::Timeout))
            == NativePollAction::Continue
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn wrong_submission_index_for_test(&self, requested: u64, successful: u64) {
        let accepted = self.completion.result_will_be_accepted_for_test();
        assert_eq!(
            handle_native_poll_result(
                &self.completion,
                Err(wgpu::PollError::WrongSubmissionIndex(requested, successful))
            ),
            NativePollAction::Stop
        );
        self.record(accepted);
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use native_support::*;
#[cfg(not(target_arch = "wasm32"))]
mod native_support {
    use super::*;
    use crate::{
        ImageBuffer,
        accounting::{ReadbackFixture, RenderAccountingOptions, RendererCpu},
    };
    use std::sync::OnceLock;
    const DEADLINE: Duration = Duration::from_secs(30);

    fn device_queue() -> &'static (wgpu::Device, wgpu::Queue) {
        static GPU: OnceLock<(wgpu::Device, wgpu::Queue)> = OnceLock::new();
        GPU.get_or_init(|| {
            let instance = wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            );
            let adapter = pollster::block_on(wgpu::util::initialize_adapter_from_env_or_default(
                &instance, None,
            ))
            .expect("readback ownership fixtures require a compatible tiny native adapter");
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("Surgeist shared tiny readback ownership fixture"),
                ..Default::default()
            }))
            .unwrap()
        })
    }
    fn account() -> Account {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let (account, started, _, fault) = ReadbackFixture::new(&cpu, options)
            .run_sync(|attempt| Ok(attempt.account().clone()))
            .unwrap()
            .into_parts();
        assert!(started);
        assert_eq!(fault, None);
        account.unwrap()
    }
    fn copied_allocation(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        size: PhysicalSize,
        account: &Account,
    ) -> SubmittedAllocation {
        let generation = Generation::new(RenderAccountingOptions::new()).unwrap();
        ReadbackPlan::try_new(size)
            .unwrap()
            .allocate(&generation, device, account)
            .unwrap()
            .submit_for_test(device, queue, texture, account)
            .unwrap()
    }
    fn poll(device: &wgpu::Device, index: wgpu::SubmissionIndex) {
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: Some(DEADLINE),
            })
            .expect("bounded real native ownership fixture progress");
    }
    enum State {
        Allocated(Readback<Allocated>),
        Submitted(Readback<CopySubmitted<u64>>),
        Pending(ReadbackMapFuture),
        Mapped(MappedReadback),
        Terminal(ReadbackPhaseForTest),
    }
    #[derive(Clone)]
    pub(crate) struct ReadbackStateMachineObservationForTest {
        terminal: Arc<Mutex<Option<ReadbackPhaseForTest>>>,
        events: CleanupObservation,
        map: Arc<ReadbackStagingMapState>,
    }
    impl ReadbackStateMachineObservationForTest {
        pub(crate) fn terminal_phase_for_test(&self) -> Option<ReadbackPhaseForTest> {
            *self
                .terminal
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }
        pub(crate) fn cleanup_events_for_test(&self) -> Vec<ReadbackCleanupEventForTest> {
            self.events.events()
        }
        pub(crate) fn staging_disposition_for_test(&self) -> ReadbackStagingDispositionForTest {
            self.map.disposition_for_test().into()
        }
    }
    /// Dynamic test storage contains the actual concrete consuming owners.
    /// Sentinel submissions never enter native registration.
    pub(crate) struct ReadbackStateMachineForTest {
        state: State,
        observation: ReadbackStateMachineObservationForTest,
        index: Option<wgpu::SubmissionIndex>,
    }
    impl ReadbackStateMachineForTest {
        pub(crate) fn at(phase: ReadbackPhaseForTest) -> Self {
            let (device, queue) = device_queue();
            let account = account();
            let generation = Generation::new(RenderAccountingOptions::new()).unwrap();
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Surgeist 1x2 lifecycle fixture input"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 2,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                texture.as_image_copy(),
                &[1, 2, 3, 4, 5, 6, 7, 8],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: None,
                },
                texture.size(),
            );
            let allocation = ReadbackPlan::try_new(PhysicalSize::new(1, 2))
                .unwrap()
                .allocate(&generation, device, &account)
                .unwrap();
            let events = CleanupObservation::default();
            let mut owner = allocation.bind(&account);
            owner.staging.probe.0 = Some(events.clone());
            let observation = ReadbackStateMachineObservationForTest {
                terminal: Arc::new(Mutex::new(None)),
                events,
                map: owner.staging.observation(),
            };
            let (state, index) = match phase {
                ReadbackPhaseForTest::Allocated => (State::Allocated(owner), None),
                ReadbackPhaseForTest::CopySubmitted { submission_index } => {
                    let command = encode_copy(
                        &owner.staging.buffer,
                        &owner.plan,
                        &texture,
                        device,
                        &account,
                    )
                    .unwrap();
                    command.submit(queue, Some(&account)).unwrap();
                    (
                        State::Submitted(Readback {
                            staging: owner.staging,
                            plan: owner.plan,
                            phase: CopySubmitted(submission_index),
                        }),
                        None,
                    )
                }
                ReadbackPhaseForTest::MapPending | ReadbackPhaseForTest::Mapped => {
                    let command = encode_copy(
                        &owner.staging.buffer,
                        &owner.plan,
                        &texture,
                        device,
                        &account,
                    )
                    .unwrap();
                    let index = command.submit(queue, Some(&account)).unwrap();
                    let resolved = Readback {
                        staging: owner.staging,
                        plan: owner.plan,
                        phase: CopySubmitted(index.clone()),
                    };
                    (
                        State::Pending(ReadbackMapFuture::start_manual(resolved).unwrap()),
                        Some(index),
                    )
                }
                _ => panic!("ownership fixture starts only in an active phase"),
            };
            let mut fixture = Self {
                state,
                observation,
                index,
            };
            if phase == ReadbackPhaseForTest::Mapped {
                fixture.map_callback_succeeded_for_test();
                fixture.consume_success();
            }
            fixture
        }
        fn consume_success(&mut self) {
            let State::Pending(mut future) = std::mem::replace(
                &mut self.state,
                State::Terminal(ReadbackPhaseForTest::Failed),
            ) else {
                panic!("real registered owner")
            };
            let MapFutureState::Pending(registered) =
                std::mem::replace(&mut future.state, MapFutureState::Finished)
            else {
                panic!("pending registration")
            };
            assert!(matches!(
                registered
                    .completion
                    .poll(&mut Context::from_waker(Waker::noop())),
                Poll::Ready(ReadbackCompletionResult::Map(Ok(())))
            ));
            self.state = State::Mapped(MappedReadback {
                staging: registered.staging,
                plan: registered.plan,
            });
        }
        pub(crate) fn phase_for_test(&self) -> ReadbackPhaseForTest {
            match &self.state {
                State::Allocated(owner) => {
                    assert_eq!(owner.plan.size, PhysicalSize::new(1, 2));
                    ReadbackPhaseForTest::Allocated
                }
                State::Submitted(owner) => ReadbackPhaseForTest::CopySubmitted {
                    submission_index: owner.phase.0,
                },
                State::Pending(_) => ReadbackPhaseForTest::MapPending,
                State::Mapped(_) => ReadbackPhaseForTest::Mapped,
                State::Terminal(phase) => *phase,
            }
        }
        fn terminate(&mut self, phase: ReadbackPhaseForTest) {
            if matches!(self.state, State::Terminal(_)) {
                return;
            }
            drop(std::mem::replace(&mut self.state, State::Terminal(phase)));
            *self
                .observation
                .terminal
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(phase);
        }
        pub(crate) fn fail_for_test(&mut self) {
            self.terminate(ReadbackPhaseForTest::Failed);
        }
        pub(crate) fn cancel_for_test(&mut self) {
            self.terminate(ReadbackPhaseForTest::Canceled);
        }
        pub(crate) fn map_callback_succeeded_for_test(&self) {
            poll(
                &device_queue().0,
                self.index
                    .clone()
                    .expect("a real registered fixture submission"),
            );
        }
        pub(crate) fn map_callback_failed_for_test(&self) {
            let State::Pending(future) = &self.state else {
                panic!("pending actual request")
            };
            let MapFutureState::Pending(registered) = &future.state else {
                panic!("pending native owner")
            };
            registered.staging.buffer.destroy();
            poll(
                &device_queue().0,
                self.index
                    .clone()
                    .expect("a real registered fixture submission"),
            );
        }
        pub(crate) fn finish_mapped_for_test(
            &mut self,
            mapped_bytes: usize,
        ) -> Result<ImageBuffer> {
            let State::Mapped(mapped) = std::mem::replace(
                &mut self.state,
                State::Terminal(ReadbackPhaseForTest::Failed),
            ) else {
                panic!("real mapped owner")
            };
            let account = mapped.staging.account().at(RenderPhase::ReadbackDecode);
            let result = account
                .phase(RenderPhase::ReadbackDecode, || {
                    let decoded = {
                        let view = ObservedView {
                            view: mapped
                                .staging
                                .buffer
                                .slice(mapped.plan.layout.mapped_range())
                                .get_mapped_range(),
                            _after_view: ViewDropProbe(Some(self.observation.events.clone())),
                        };
                        assert_eq!(view.view.len(), 512);
                        decode_padded_rows(
                            &mapped.plan.layout,
                            &view.view[..mapped_bytes],
                            &account,
                        )
                    };
                    drop(mapped.staging);
                    ReadbackOutput::try_new(mapped.plan.size, decoded?)
                })
                .and_then(ReadbackOutput::transfer);
            let phase = if result.is_ok() {
                ReadbackPhaseForTest::PublishedBytes
            } else {
                ReadbackPhaseForTest::Failed
            };
            if result.is_ok() {
                self.observation
                    .events
                    .record(ReadbackCleanupEventForTest::PublishedBytes);
            }
            self.state = State::Terminal(phase);
            *self
                .observation
                .terminal
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(phase);
            result
        }
        pub(crate) fn staging_disposition_for_test(&self) -> ReadbackStagingDispositionForTest {
            self.observation.staging_disposition_for_test()
        }
        pub(crate) fn cleanup_events_for_test(&self) -> Vec<ReadbackCleanupEventForTest> {
            self.observation.cleanup_events_for_test()
        }
        pub(crate) fn observation_for_test(&self) -> ReadbackStateMachineObservationForTest {
            self.observation.clone()
        }
    }
    impl Drop for ReadbackStateMachineForTest {
        fn drop(&mut self) {
            self.cancel_for_test();
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum NativeReadbackStagePhaseForTest {
        MapPending,
        PublishedBytes,
        Failed,
        Canceled,
    }
    pub(crate) struct NativeReadbackStageForTest {
        future: Option<ReadbackMapFuture>,
        staging_map: Weak<ReadbackStagingMapState>,
        submission_index: wgpu::SubmissionIndex,
        phase: NativeReadbackStagePhaseForTest,
    }
    impl NativeReadbackStageForTest {
        pub(crate) fn begin(
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            texture: &wgpu::Texture,
            size: PhysicalSize,
        ) -> Result<Self> {
            let account = account();
            let submitted = copied_allocation(device, queue, texture, size, &account);
            let submission_index = submitted.index.clone();
            let owner = submitted.bind(&account);
            let staging_map = Arc::downgrade(&owner.staging.observation());
            let future = ReadbackMapFuture::start(owner, device.clone())?;
            Ok(Self {
                future: Some(future),
                staging_map,
                submission_index,
                phase: NativeReadbackStagePhaseForTest::MapPending,
            })
        }
        pub(crate) const fn phase_for_test(&self) -> NativeReadbackStagePhaseForTest {
            self.phase
        }
        pub(crate) fn submission_index_for_test(&self) -> wgpu::SubmissionIndex {
            self.submission_index.clone()
        }
        pub(crate) fn staging_disposition_for_test(
            &self,
        ) -> Option<ReadbackStagingDispositionForTest> {
            self.staging_map
                .upgrade()
                .map(|m| m.disposition_for_test().into())
        }
        pub(crate) fn staging_state_dropped_for_test(&self) -> bool {
            self.staging_map.upgrade().is_none()
        }
        pub(crate) fn cancel_for_test(&mut self) {
            if let Some(future) = self.future.take() {
                drop(future);
                self.phase = NativeReadbackStagePhaseForTest::Canceled;
            }
        }
    }
    impl Future for NativeReadbackStageForTest {
        type Output = Result<ImageBuffer>;
        fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            let Some(future) = &mut this.future else {
                return Poll::Pending;
            };
            match Pin::new(future).poll(context) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(result) => {
                    let result = result.and_then(ReadbackOutput::transfer);
                    this.phase = if result.is_ok() {
                        NativeReadbackStagePhaseForTest::PublishedBytes
                    } else {
                        NativeReadbackStagePhaseForTest::Failed
                    };
                    drop(this.future.take());
                    Poll::Ready(result)
                }
            }
        }
    }
    impl Drop for NativeReadbackStageForTest {
        fn drop(&mut self) {
            self.cancel_for_test();
        }
    }
    pub(crate) struct NativeReadbackLateCallbackStageForTest {
        completion: Arc<ReadbackCompletion>,
        device: wgpu::Device,
        staging_map: Weak<ReadbackStagingMapState>,
        submission_index: wgpu::SubmissionIndex,
    }
    impl NativeReadbackLateCallbackStageForTest {
        pub(crate) fn cancel_before_poll(
            device: &wgpu::Device,
            queue: &wgpu::Queue,
            texture: &wgpu::Texture,
            size: PhysicalSize,
        ) -> Result<Self> {
            let account = account();
            let submitted = copied_allocation(device, queue, texture, size, &account);
            let submission_index = submitted.index.clone();
            let owner = submitted.bind(&account);
            let staging_map = Arc::downgrade(&owner.staging.observation());
            let future = ReadbackMapFuture::start_manual(owner)?;
            let MapFutureState::Pending(registered) = &future.state else {
                unreachable!()
            };
            let completion = registered.completion.clone();
            drop(future);
            Ok(Self {
                completion,
                device: device.clone(),
                staging_map,
                submission_index,
            })
        }
        pub(crate) fn staging_disposition_for_test(
            &self,
        ) -> Option<ReadbackStagingDispositionForTest> {
            self.staging_map
                .upgrade()
                .map(|m| m.disposition_for_test().into())
        }
        pub(crate) fn deliver_late_callback_for_test(
            &self,
        ) -> std::result::Result<wgpu::PollStatus, wgpu::PollError> {
            self.device.poll(wgpu::PollType::Wait {
                submission_index: Some(self.submission_index.clone()),
                timeout: Some(DEADLINE),
            })
        }
        pub(crate) fn callback_result_was_discarded_for_test(&self) -> bool {
            self.completion.is_canceled_for_test() && self.staging_map.upgrade().is_none()
        }
    }
}
