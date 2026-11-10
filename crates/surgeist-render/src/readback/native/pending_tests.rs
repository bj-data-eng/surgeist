use super::*;
use crate::{
    PhysicalSize,
    accounting::{
        Generation, GenerationClass, GpuObjectKind, ReadbackFixture, RenderAccountingError,
        RenderAccountingOptions, RenderAttemptStatus, RenderClock, RenderGenerationScope,
        RenderPhaseFilter, RenderProfile, RenderReadbackReport, RenderResource, RendererCpu,
    },
    readback::{layout::ReadbackLayout, lifecycle::ReadbackStagingDisposition},
};
use std::{
    cell::RefCell,
    sync::{
        Condvar,
        atomic::{AtomicU64, Ordering},
    },
    task::Wake,
    time::Instant,
};

const DEADLINE: Duration = Duration::from_secs(30);

#[derive(Default)]
struct ManualClock {
    millis: AtomicU64,
    reads: AtomicU64,
}
thread_local! {
    static CLOCK: RefCell<Option<Arc<ManualClock>>> = const { RefCell::new(None) };
}
struct ClockBinding;
impl ClockBinding {
    fn new(clock: &Arc<ManualClock>) -> Self {
        CLOCK.with(|binding| {
            assert!(binding.replace(Some(clock.clone())).is_none());
        });
        Self
    }
}
impl Drop for ClockBinding {
    fn drop(&mut self) {
        CLOCK.with(|binding| {
            binding.take();
        });
    }
}
fn clock() -> Duration {
    CLOCK.with(|binding| {
        let binding = binding.borrow();
        let clock = binding.as_ref().expect("invocation-owned clock binding");
        clock.reads.fetch_add(1, Ordering::SeqCst);
        Duration::from_millis(clock.millis.load(Ordering::SeqCst))
    })
}

struct PreparedMap {
    device: wgpu::Device,
    // WGPU's device stores a weak queue reference. Dropping the last Queue
    // makes map_async complete synchronously instead of waiting for progress.
    _queue: wgpu::Queue,
    generation: Generation,
    staging: OwnedGpu<wgpu::Buffer>,
    submission: wgpu::SubmissionIndex,
    prefix: RenderReadbackReport,
}
impl PreparedMap {
    fn new(cpu: &RendererCpu, options: RenderAccountingOptions) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            display: None,
            backends: wgpu::Backends::from_env().unwrap_or_default(),
            flags: wgpu::InstanceFlags::from_build_config().with_env(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default(),
        });
        let adapter = pollster::block_on(wgpu::util::initialize_adapter_from_env_or_default(
            &instance, None,
        ))
        .expect("native map tests require a compatible private GPU adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Surgeist invocation-owned readback device"),
            ..Default::default()
        }))
        .expect("native map tests require a private fixture device");
        let generation = Generation::new(options).unwrap();
        let (parts, _, prefix, fault) = ReadbackFixture::new(cpu, options)
            .run_sync(|mut prefix| {
                prefix.attach_generation(&generation).unwrap();
                let account = prefix.account().at(RenderPhase::ReadbackPrepare);
                let texture = generation
                    .allocation(GenerationClass::SurfaceTargets, 24, Some(&account), || {
                        device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("Surgeist native map fixture input"),
                            size: wgpu::Extent3d {
                                width: 3,
                                height: 2,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            usage: wgpu::TextureUsages::COPY_SRC,
                            view_formats: &[],
                        })
                    })
                    .unwrap();
                // 3 RGBA8 pixels are 12 useful bytes per row, aligned to 256: 512 total.
                let staging = generation
                    .allocation(
                        GenerationClass::ReadbackStaging,
                        512,
                        Some(&account),
                        || {
                            device.create_buffer(&wgpu::BufferDescriptor {
                                label: Some("Surgeist native map fixture staging"),
                                size: 512,
                                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            })
                        },
                    )
                    .unwrap();
                let mut encoder = generation
                    .opaque(GpuObjectKind::CommandEncoder, Some(&account), || {
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Surgeist native map fixture copy"),
                        })
                    })
                    .unwrap();
                account.work(RenderWork::ReadbackCopies, 1).unwrap();
                encoder.copy_texture_to_buffer(
                    texture.as_image_copy(),
                    wgpu::TexelCopyBufferInfo {
                        buffer: &staging,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(256),
                            rows_per_image: None,
                        },
                    },
                    wgpu::Extent3d {
                        width: 3,
                        height: 2,
                        depth_or_array_layers: 1,
                    },
                );
                let submission = encoder
                    .finish(Some(&account))
                    .unwrap()
                    .submit(&queue, Some(&account))
                    .unwrap();
                drop(texture);
                Ok((staging, submission))
            })
            .unwrap()
            .into_parts();
        let (staging, submission) = parts.unwrap();
        assert_eq!(fault, None);
        let work = prefix.attempt().unwrap().usage();
        assert_eq!(work.work(RenderWork::ResourceAllocationRequests), 4);
        assert_eq!(work.work(RenderWork::ReadbackCopies), 1);
        assert_eq!(work.work(RenderWork::Submissions), 1);
        assert_eq!(work.work(RenderWork::ReadbackMapRequests), 0);
        let native = prefix.generation().unwrap().unwrap().usage();
        assert_eq!(native.work(RenderWork::ResourceAllocations), 2);
        assert_eq!(native.work(RenderWork::GpuObjectsCreated), 2);
        assert_eq!(native.live(RenderResource::GpuBytes), 512);
        assert_eq!(native.live(RenderResource::OpaqueGpuObjects), 0);
        Self {
            device,
            _queue: queue,
            generation,
            staging,
            submission,
            prefix,
        }
    }
}

#[derive(Default)]
struct WakeState {
    entered: bool,
    released: bool,
    timed_out: bool,
}
#[derive(Default)]
struct PausedWake {
    state: Mutex<WakeState>,
    changed: Condvar,
}
impl PausedWake {
    fn await_entry(&self) {
        let start = Instant::now();
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (state, _) = self
            .changed
            .wait_timeout_while(state, DEADLINE, |s| !s.entered)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            state.entered,
            "real map callback did not enter wake within {DEADLINE:?}; elapsed {:?}, released={}, timed_out={}",
            start.elapsed(),
            state.released,
            state.timed_out
        );
        assert!(
            !state.timed_out,
            "map callback wake timed out before observation"
        );
    }
    fn release(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.released = true;
        self.changed.notify_all();
    }
}
impl Wake for PausedWake {
    fn wake(self: Arc<Self>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.entered = true;
        self.changed.notify_all();
        let (mut state, _) = self
            .changed
            .wait_timeout_while(state, DEADLINE, |s| !s.released)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.released {
            state.timed_out = true;
        }
    }
}

/// Always release the callback before joining, including assertion unwinds.
struct OwnedPoll {
    latch: Arc<PausedWake>,
    thread: Option<thread::JoinHandle<std::result::Result<wgpu::PollStatus, wgpu::PollError>>>,
}
impl OwnedPoll {
    fn start(
        device: wgpu::Device,
        submission: wgpu::SubmissionIndex,
        clock: Arc<ManualClock>,
        latch: Arc<PausedWake>,
    ) -> Self {
        let thread = thread::Builder::new()
            .name("surgeist-manual-map-test".to_owned())
            .spawn(move || {
                let _binding = ClockBinding::new(&clock);
                device.poll(wgpu::PollType::Wait {
                    submission_index: Some(submission),
                    timeout: Some(DEADLINE),
                })
            })
            .expect("owned map polling thread");
        Self {
            latch,
            thread: Some(thread),
        }
    }
    fn finish(mut self) {
        self.latch.release();
        self.thread
            .take()
            .unwrap()
            .join()
            .expect("owned map polling thread must not panic")
            .expect("bounded native map polling");
        let state = self
            .latch
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            !state.timed_out,
            "real callback outlived its bounded release latch"
        );
    }
}
impl Drop for OwnedPoll {
    fn drop(&mut self) {
        self.latch.release();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn pending_cancel_with_late_callback(profile: RenderProfile, retire: bool, timed: bool) {
    let manual_clock = Arc::new(ManualClock::default());
    let _binding = ClockBinding::new(&manual_clock);
    let options = RenderAccountingOptions::new()
        .with_clock(RenderClock::Host(clock))
        .with_profile(profile);
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device,
        _queue,
        generation,
        staging,
        submission,
        prefix,
    } = PreparedMap::new(&cpu, options);
    let prefix_before = prefix.snapshot_for_test();
    let mut fixture = ReadbackFixture::new(&cpu, options);
    let staging_observation = std::cell::OnceCell::new();
    let latch = Arc::new(PausedWake::default());
    let waker = Waker::from(latch.clone());
    let owner_submission = submission.clone();
    let future = fixture.run(async |mut attempt| {
        attempt.attach_generation(&generation)?;
        let account = attempt.account().clone();
        let mut owner = ReadbackOwner::allocated(
            staging,
            ReadbackLayout::try_new(PhysicalSize::new(3, 2)).unwrap(),
            PhysicalSize::new(3, 2),
            &account,
        );
        owner.copy_submitted(owner_submission);
        assert!(staging_observation.set(owner.staging_map()).is_ok());
        manual_clock.millis.store(10, Ordering::SeqCst);
        account
            .async_phase(RenderPhase::ReadbackMap, async move {
                ReadbackMapFuture::start_manual(owner)?.await
            })
            .await
    });
    let mut future = Box::pin(future);
    // This device has no progress helper, and no submit/poll occurs after real
    // registration. WGPU delivers map callbacks only from subsequent progress.
    assert!(matches!(
        future.as_mut().poll(&mut Context::from_waker(&waker)),
        Poll::Pending
    ));
    let staging_map = staging_observation.get().unwrap();
    assert_eq!(
        staging_map.disposition_for_test(),
        ReadbackStagingDisposition::MapPending
    );
    // The real driver has not published a terminal report.
    let poll = OwnedPoll::start(device, submission, manual_clock.clone(), latch.clone());
    latch.await_entry();
    // The real callback has published completion and unlocked it, but its native
    // staging clone remains alive until this wake returns.
    assert_eq!(
        staging_map.disposition_for_test(),
        ReadbackStagingDisposition::MappedActive
    );
    manual_clock.millis.store(70, Ordering::SeqCst);
    drop(future);
    assert_eq!(
        staging_map.disposition_for_test(),
        ReadbackStagingDisposition::Released
    );
    let canceled = fixture.report.take().unwrap();
    let closed = canceled.snapshot_for_test();
    assert_eq!(canceled.status(), RenderAttemptStatus::Canceled);
    let attempt = canceled.attempt().unwrap();
    let work = attempt.usage();
    assert_eq!(work.work(RenderWork::ReadbackMapRequests), 1);
    for unit in [
        RenderWork::ReadbackCopies,
        RenderWork::Submissions,
        RenderWork::ReadbackRows,
        RenderWork::ReadbackBytes,
        RenderWork::ReadbackTransferredBytes,
        RenderWork::ResourceAllocationRequests,
    ] {
        assert_eq!(
            work.work(unit),
            0,
            "map-only interval must not invent {unit:?}"
        );
    }
    assert_eq!(
        attempt.phase_count(RenderPhase::ReadbackMap),
        if matches!(profile, RenderProfile::Off) {
            0
        } else {
            1
        }
    );
    assert_eq!(
        attempt.phase_count(RenderPhase::ReadbackCleanup),
        if matches!(profile, RenderProfile::Off) {
            0
        } else {
            1
        }
    );
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::ReadbackMap),
        timed.then_some(Duration::from_millis(60))
    );
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::ReadbackCleanup),
        timed.then_some(Duration::ZERO)
    );
    let interval = canceled.generation().unwrap().unwrap();
    let native = interval.scope_usage(RenderGenerationScope::ReadbackStaging);
    assert_eq!(
        interval.scope_opening(
            RenderGenerationScope::ReadbackStaging,
            RenderResource::GpuBytes
        ),
        512
    );
    assert_eq!(native.live(RenderResource::GpuBytes), 512);
    assert_eq!(native.peak(RenderResource::GpuBytes), 512);
    assert_eq!(canceled.accounting_diagnostic(), None);
    assert_eq!(canceled.cleanup_fault(), None);
    assert_eq!(
        generation
            .class_usage(GenerationClass::ReadbackStaging)
            .unwrap()
            .live(RenderResource::GpuBytes),
        512
    );
    let first_fault = if retire {
        generation.retire();
        assert_eq!(
            generation
                .class_usage(GenerationClass::ReadbackStaging)
                .unwrap()
                .live(RenderResource::GpuBytes),
            0,
            "retirement releases logical custody while the real callback is still paused"
        );
        let fault = generation
            .ensure_healthy()
            .unwrap_err()
            .accounting_diagnostic();
        assert_eq!(fault, Some(RenderAccountingError::GenerationRetired));
        fault
    } else {
        None
    };
    poll.finish();
    assert_eq!(
        generation
            .class_usage(GenerationClass::ReadbackStaging)
            .unwrap()
            .live(RenderResource::GpuBytes),
        0
    );
    assert_eq!(
        generation
            .ensure_healthy()
            .err()
            .and_then(|e| e.accounting_diagnostic()),
        first_fault
    );
    assert_eq!(
        staging_map.disposition_for_test(),
        ReadbackStagingDisposition::Released
    );
    assert_eq!(
        canceled, closed,
        "late native disposal cannot rewrite closed reports"
    );
    assert_eq!(
        prefix, prefix_before,
        "map cleanup cannot rewrite its setup prefix"
    );
    if !timed {
        assert_eq!(
            manual_clock.reads.load(Ordering::SeqCst),
            0,
            "off/untimed paths, including late callback disposal, must not read the host clock"
        );
    }
}

#[test]
fn real_pending_map_cancellation_counts_wait_time_and_retains_paused_callback_custody() {
    pending_cancel_with_late_callback(RenderProfile::Summary { timing: true }, false, true);
}

#[test]
fn retired_generation_late_callback_cannot_resurrect_custody_or_replace_first_fault() {
    pending_cancel_with_late_callback(RenderProfile::Summary { timing: true }, true, true);
}

#[test]
fn profiling_off_cancellation_and_late_callback_never_read_the_host_clock() {
    pending_cancel_with_late_callback(RenderProfile::Off, false, false);
}

#[test]
fn untimed_detailed_cancellation_and_retired_late_callback_never_read_the_host_clock() {
    pending_cancel_with_late_callback(
        RenderProfile::Detailed {
            timing: false,
            event_limit: 64,
            filter: RenderPhaseFilter::All,
        },
        true,
        false,
    );
}

#[test]
fn native_synchronous_completion_without_queue_decodes_real_rows() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device: _device,
        _queue,
        generation,
        staging,
        submission,
        ..
    } = PreparedMap::new(&cpu, options);
    // Locked WGPU also supports mapping without a live Queue: this runs the
    // real callback synchronously. It explains the fixture's retained Queue.
    drop(_queue);
    let (output, _, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            let mut owner = ReadbackOwner::allocated(
                staging,
                ReadbackLayout::try_new(PhysicalSize::new(3, 2)).unwrap(),
                PhysicalSize::new(3, 2),
                &account,
            );
            owner.copy_submitted(submission);
            let mut map = Box::pin(ReadbackMapFuture::start_manual(owner).unwrap());
            let output = match map.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
                Poll::Ready(Ok(output)) => output,
                Poll::Ready(Err(error)) => panic!("synchronous native map failed: {error:?}"),
                Poll::Pending => panic!("mapping without a live queue must complete synchronously"),
            };

            Ok(output)
        })
        .unwrap()
        .into_parts();
    let output = output.unwrap();
    assert_eq!(fault, None);
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ReadbackRows),
        2
    );
    assert_eq!(output.transfer().unwrap().rgba().len(), 24);
    generation.ensure_healthy().unwrap();
}

#[test]
fn native_progress_helper_completes_the_same_registration_path_and_decodes_real_rows() {
    struct HelperJoin(Option<thread::JoinHandle<()>>);
    impl Drop for HelperJoin {
        fn drop(&mut self) {
            if let Some(helper) = self.0.take() {
                let _ = helper.join();
            }
        }
    }
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device,
        _queue,
        generation,
        staging,
        submission,
        ..
    } = PreparedMap::new(&cpu, options);
    let (output, _, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            let mut owner = ReadbackOwner::allocated(
                staging,
                ReadbackLayout::try_new(PhysicalSize::new(3, 2)).unwrap(),
                PhysicalSize::new(3, 2),
                &account,
            );
            owner.copy_submitted(submission.clone());
            let mut map = ReadbackMapFuture::start(owner, device, submission).unwrap();
            let mut helper = HelperJoin(map._poll_helper.take());
            let output = pollster::block_on(account.async_phase(RenderPhase::ReadbackMap, map));
            helper
                .0
                .take()
                .unwrap()
                .join()
                .expect("native production progress helper must finish");
            let output = output.unwrap();

            Ok(output)
        })
        .unwrap()
        .into_parts();
    let output = output.unwrap();
    assert_eq!(fault, None);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::ReadbackMapRequests), 1);
    assert_eq!(usage.work(RenderWork::ReadbackRows), 2);
    assert_eq!(usage.work(RenderWork::ReadbackBytes), 24);
    assert_eq!(usage.work(RenderWork::ReadbackCopies), 0);
    assert_eq!(usage.work(RenderWork::Submissions), 0);
    assert_eq!(output.transfer().unwrap().rgba().len(), 24);
    generation.ensure_healthy().unwrap();
}
