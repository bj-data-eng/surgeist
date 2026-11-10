use super::*;
use crate::{
    PhysicalSize,
    accounting::{
        Generation, GenerationClass, ReadbackFixture, RenderAccountingError,
        RenderAccountingOptions, RenderAttemptStatus, RenderClock, RenderGenerationScope,
        RenderPhaseFilter, RenderProfile, RenderReadbackReport, RenderResource, RendererCpu,
    },
    readback::native::lifecycle::ReadbackStagingDisposition,
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
        let millis = clock.millis.load(Ordering::SeqCst);
        DROP_MAP.with(|map| {
            if let Some(map) = map.borrow().as_ref() {
                MAP_CLOCK_SAMPLES
                    .with(|samples| samples.borrow_mut().push((millis, Arc::strong_count(map))));
            }
        });
        Duration::from_millis(millis)
    })
}

struct PreparedMap {
    device: wgpu::Device,
    // WGPU's device stores a weak queue reference. Dropping the last Queue
    // makes map_async complete synchronously instead of waiting for progress.
    _queue: wgpu::Queue,
    generation: Generation,
    submitted: SubmittedAllocation,
    prefix: RenderReadbackReport,
}
impl PreparedMap {
    fn new(cpu: &RendererCpu, options: RenderAccountingOptions) -> Self {
        Self::with_generation(cpu, options, Generation::new(options).unwrap())
    }
    fn with_generation(
        cpu: &RendererCpu,
        options: RenderAccountingOptions,
        generation: Generation,
    ) -> Self {
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
                let submitted = ReadbackPlan::try_new(PhysicalSize::new(3, 2))
                    .unwrap()
                    .allocate(&generation, &device, &account)
                    .unwrap()
                    .submit_for_test(&device, &queue, &texture, &account)
                    .unwrap();
                drop(texture);
                Ok(submitted)
            })
            .unwrap()
            .into_parts();
        let submitted = parts.unwrap();
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
            submitted,
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
pub(super) struct PausedWake {
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
        submitted,
        prefix,
    } = PreparedMap::new(&cpu, options);
    let prefix_before = prefix.snapshot_for_test();
    let mut fixture = ReadbackFixture::new(&cpu, options);
    let staging_observation = std::cell::OnceCell::new();
    let latch = Arc::new(PausedWake::default());
    let waker = Waker::from(latch.clone());
    let submission = submitted.index.clone();
    let future = fixture.run(async |mut attempt| {
        attempt.attach_generation(&generation)?;
        let account = attempt.account().clone();
        let owner = submitted.bind(&account);
        assert!(staging_observation.set(owner.staging.observation()).is_ok());
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
        submitted,
        ..
    } = PreparedMap::new(&cpu, options);
    // Locked WGPU also supports mapping without a live Queue: this runs the
    // real callback synchronously. It explains the fixture's retained Queue.
    drop(_queue);
    let (output, _, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            let owner = submitted.bind(&account);
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
        submitted,
        ..
    } = PreparedMap::new(&cpu, options);
    let (output, _, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            let owner = submitted.bind(&account);
            let mut map = ReadbackMapFuture::start(owner, device).unwrap();
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

#[test]
fn completed_native_map_repoll_is_pending_without_clock_work_cleanup_or_wake() {
    use crate::accounting::RenderCpuScope;

    #[derive(Default)]
    struct CountWake(AtomicU64);
    impl Wake for CountWake {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let manual_clock = Arc::new(ManualClock::default());
    let _binding = ClockBinding::new(&manual_clock);
    let options = RenderAccountingOptions::new()
        .with_clock(RenderClock::Host(clock))
        .with_profile(RenderProfile::Summary { timing: true });
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device: _device,
        _queue,
        generation,
        submitted,
        prefix,
    } = PreparedMap::new(&cpu, options);
    let prefix_before = prefix.snapshot_for_test();
    assert_eq!(
        generation
            .class_usage(GenerationClass::ReadbackStaging)
            .unwrap()
            .live(RenderResource::GpuBytes),
        512
    );
    // The real provider invokes this registration's callback synchronously when
    // no Queue remains. No helper, synthesized completion or timing wait is used.
    drop(_queue);
    let wake = Arc::new(CountWake::default());
    let waker = Waker::from(wake.clone());
    let (output, started, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            manual_clock.millis.store(10, Ordering::SeqCst);
            account.phase(RenderPhase::ReadbackMap, || {
                let owner = submitted.bind(&account);
                assert_eq!(owner.plan.layout.mapped_range(), 0..512);
                let staging_map = owner.staging.observation();
                let mut map = Box::pin(ReadbackMapFuture::start_manual(owner).unwrap());
                manual_clock.millis.store(40, Ordering::SeqCst);
                let output = match map.as_mut().poll(&mut Context::from_waker(&waker)) {
                    Poll::Ready(Ok(output)) => output,
                    Poll::Ready(Err(error)) => panic!("synchronous native map failed: {error:?}"),
                    Poll::Pending => {
                        panic!("mapping without a live queue must complete synchronously")
                    }
                };
                assert_eq!(
                    staging_map.disposition_for_test(),
                    ReadbackStagingDisposition::Released
                );
                assert_eq!(
                    generation
                        .class_usage(GenerationClass::ReadbackStaging)
                        .unwrap()
                        .live(RenderResource::GpuBytes),
                    0
                );
                let work_before: Vec<_> = <RenderWork as surgeist_ledger::Dimension>::ALL
                    .iter()
                    .map(|&work| account.work_count(work).unwrap())
                    .collect();
                let reads_before = manual_clock.reads.load(Ordering::SeqCst);
                let wakes_before = wake.0.load(Ordering::SeqCst);

                // Poll the very same completed future through its actual Future
                // implementation. Terminal policy returns no second output.
                for _ in 0..3 {
                    assert!(matches!(
                        map.as_mut().poll(&mut Context::from_waker(&waker)),
                        Poll::Pending
                    ));
                }
                drop(map);
                assert_eq!(manual_clock.reads.load(Ordering::SeqCst), reads_before);
                assert_eq!(wake.0.load(Ordering::SeqCst), wakes_before);
                let work_after: Vec<_> = <RenderWork as surgeist_ledger::Dimension>::ALL
                    .iter()
                    .map(|&work| account.work_count(work).unwrap())
                    .collect();
                assert_eq!(work_after, work_before);
                assert_eq!(
                    staging_map.disposition_for_test(),
                    ReadbackStagingDisposition::Released
                );
                // Keep the actual guarded output until the driver has closed.
                Ok(output)
            })
        })
        .unwrap()
        .into_parts();
    let output = output.unwrap();
    assert!(started);
    assert_eq!(fault, None);
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(report.accounting_diagnostic(), None);
    assert_eq!(report.cleanup_fault(), None);
    let attempt = report.attempt().unwrap();
    for phase in [
        RenderPhase::ReadbackMap,
        RenderPhase::ReadbackDecode,
        RenderPhase::ReadbackCleanup,
    ] {
        assert_eq!(attempt.phase_count(phase), 1, "{phase:?} must run once");
    }
    let work = attempt.usage();
    assert_eq!(work.work(RenderWork::ReadbackMapRequests), 1);
    assert_eq!(work.work(RenderWork::ReadbackRows), 2);
    assert_eq!(work.work(RenderWork::ReadbackBytes), 24);
    for unit in [
        RenderWork::ReadbackCopies,
        RenderWork::Submissions,
        RenderWork::ResourceAllocationRequests,
        RenderWork::ReadbackTransferredBytes,
    ] {
        assert_eq!(
            work.work(unit),
            0,
            "map-only interval must not add {unit:?}"
        );
    }
    let capacity = report
        .renderer_cpu()
        .unwrap()
        .scope_usage(RenderCpuScope::ReadbackOutput)
        .live(RenderResource::RetainedCpuBytes);
    assert!(capacity >= 24);
    let closed = report.snapshot_for_test();
    let image = output.transfer().unwrap();
    assert_eq!(image.size(), PhysicalSize::new(3, 2));
    assert_eq!(image.rgba(), &[0; 24]);
    assert_eq!(image.into_rgba().capacity() as u64, capacity);
    assert_eq!(report, closed);
    assert_eq!(prefix, prefix_before);
    assert_eq!(wake.0.load(Ordering::SeqCst), 0);
    generation.ensure_healthy().unwrap();
}

pub(super) fn canceled_callback(generation: &Generation) -> impl FnOnce() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device,
        _queue,
        submitted,
        ..
    } = PreparedMap::with_generation(&cpu, options, generation.clone());
    let index = submitted.index.clone();
    let latch = Arc::new(PausedWake::default());
    let waker = Waker::from(latch.clone());
    let manual_clock = Arc::new(ManualClock::default());
    let (poll, started, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(generation).unwrap();
            let mut future = Box::pin(
                ReadbackMapFuture::start_manual(submitted.bind(attempt.account())).unwrap(),
            );
            assert!(matches!(
                future.as_mut().poll(&mut Context::from_waker(&waker)),
                Poll::Pending
            ));
            let poll = OwnedPoll::start(device, index, manual_clock, latch.clone());
            latch.await_entry();
            drop(future);
            Ok(poll)
        })
        .unwrap()
        .into_parts();
    let poll = poll.unwrap();
    assert!(started);
    assert_eq!(fault, None);
    assert_eq!(
        report
            .generation()
            .unwrap()
            .unwrap()
            .usage()
            .live(RenderResource::GpuBytes),
        512
    );
    move || {
        poll.finish();
        drop(_queue);
    }
}

#[derive(Default)]
struct CountedWake(AtomicU64);
impl Wake for CountedWake {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn completion_terminal_polls_discard_late_results_without_waking() {
    for canceled in [false, true] {
        let completion = ReadbackCompletion::new();
        let wakes = Arc::new(CountedWake::default());
        let waker = Waker::from(wakes.clone());
        assert!(matches!(
            completion.poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        ));
        if canceled {
            completion.cancel();
        } else {
            completion.complete(ReadbackCompletionResult::Map(Err(wgpu::BufferAsyncError)));
            assert!(matches!(
                completion.poll(&mut Context::from_waker(&waker)),
                Poll::Ready(ReadbackCompletionResult::Map(Err(_)))
            ));
        }
        let before = wakes.0.load(Ordering::SeqCst);
        for _ in 0..3 {
            completion.complete(ReadbackCompletionResult::Map(Ok(())));
            assert!(matches!(
                completion.poll(&mut Context::from_waker(&waker)),
                Poll::Pending
            ));
        }
        assert_eq!(wakes.0.load(Ordering::SeqCst), before);
        assert_eq!(before, u64::from(!canceled));
    }
}

#[test]
fn real_callback_and_native_poll_failure_keep_first_source_and_latest_waker_in_both_orders() {
    use test_support::{CleanupObservation, ReadbackCleanupEventForTest as Event};
    for callback_success in [false, true] {
        for poll_first in [false, true] {
            let manual_clock = Arc::new(ManualClock::default());
            let _binding = ClockBinding::new(&manual_clock);
            let options = RenderAccountingOptions::new()
                .with_clock(RenderClock::Host(clock))
                .with_profile(RenderProfile::Summary { timing: true });
            let cpu = RendererCpu::new(options).unwrap();
            let PreparedMap {
                device,
                _queue,
                submitted,
                generation,
                prefix,
            } = PreparedMap::new(&cpu, options);
            let prefix_before = prefix.snapshot_for_test();
            let index = submitted.index.clone();
            let first = Arc::new(CountedWake::default());
            let latest = Arc::new(CountedWake::default());
            let first_waker = Waker::from(first.clone());
            let latest_waker = Waker::from(latest.clone());
            let events = CleanupObservation::default();
            let (result, started, report, fault) = ReadbackFixture::new(&cpu, options)
                .run_sync(|mut attempt| {
                    attempt.attach_generation(&generation).unwrap();
                    let account = attempt.account().clone();
                    account.phase(RenderPhase::ReadbackMap, || {
                        let mut owner = submitted.bind(&account);
                        owner.staging.probe.0 = Some(events.clone());
                        let observation = owner.staging.observation();
                        let mut map = Box::pin(ReadbackMapFuture::start_manual(owner).unwrap());
                        assert!(matches!(
                            map.as_mut().poll(&mut Context::from_waker(&first_waker)),
                            Poll::Pending
                        ));
                        assert!(matches!(
                            map.as_mut().poll(&mut Context::from_waker(&latest_waker)),
                            Poll::Pending
                        ));
                        let MapFutureState::Pending(registered) = &map.state else {
                            panic!("actual pending map")
                        };
                        let completion = registered.completion.clone();
                        if poll_first {
                            assert_eq!(
                                handle_native_poll_result(
                                    &completion,
                                    Err(wgpu::PollError::WrongSubmissionIndex(9, 8))
                                ),
                                NativePollAction::Stop
                            );
                        }
                        if !callback_success {
                            registered.staging.buffer.destroy();
                        }
                        device
                            .poll(wgpu::PollType::Wait {
                                submission_index: Some(index),
                                timeout: Some(DEADLINE),
                            })
                            .expect("bounded real provider callback");
                        assert_eq!(
                            observation.disposition_for_test(),
                            if callback_success {
                                ReadbackStagingDisposition::MappedActive
                            } else {
                                ReadbackStagingDisposition::Idle
                            }
                        );
                        if !poll_first {
                            assert_eq!(
                                handle_native_poll_result(
                                    &completion,
                                    Err(wgpu::PollError::WrongSubmissionIndex(9, 8))
                                ),
                                NativePollAction::Stop
                            );
                        }
                        assert_eq!(first.0.load(Ordering::SeqCst), 0);
                        assert_eq!(latest.0.load(Ordering::SeqCst), 1);
                        let result =
                            match map.as_mut().poll(&mut Context::from_waker(&latest_waker)) {
                                Poll::Ready(result) => result,
                                Poll::Pending => panic!("selected real completion must be ready"),
                            };
                        let reads = manual_clock.reads.load(Ordering::SeqCst);
                        for _ in 0..3 {
                            assert!(matches!(
                                map.as_mut().poll(&mut Context::from_waker(&latest_waker)),
                                Poll::Pending
                            ));
                        }
                        drop(map);
                        assert_eq!(manual_clock.reads.load(Ordering::SeqCst), reads);
                        assert_eq!(latest.0.load(Ordering::SeqCst), 1);
                        assert_eq!(
                            observation.disposition_for_test(),
                            ReadbackStagingDisposition::Released
                        );
                        result
                    })
                })
                .unwrap()
                .into_parts();
            assert!(started);
            assert_eq!(fault, None);
            let succeeded = callback_success && !poll_first;
            assert_eq!(
                report.status(),
                if succeeded {
                    RenderAttemptStatus::Succeeded
                } else {
                    RenderAttemptStatus::Failed(crate::ErrorCode::ReadbackFailed)
                }
            );
            assert_eq!(report.cleanup_fault(), None);
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .phase_count(RenderPhase::ReadbackCleanup),
                1
            );
            let work = report.attempt().unwrap().usage();
            assert_eq!(work.work(RenderWork::ReadbackMapRequests), 1);
            assert_eq!(
                work.work(RenderWork::ReadbackRows),
                if succeeded { 2 } else { 0 }
            );
            assert_eq!(
                work.work(RenderWork::ReadbackBytes),
                if succeeded { 24 } else { 0 }
            );
            match result {
                Ok(output) => assert_eq!(output.transfer().unwrap().rgba(), &[0; 24]),
                Err(error) => {
                    assert_eq!(error.code(), crate::ErrorCode::ReadbackFailed);
                    let source = std::error::Error::source(&error).unwrap();
                    if poll_first {
                        assert!(matches!(
                            source.downcast_ref::<wgpu::PollError>(),
                            Some(wgpu::PollError::WrongSubmissionIndex(9, 8))
                        ));
                    } else {
                        assert!(source.downcast_ref::<wgpu::BufferAsyncError>().is_some());
                    }
                }
            }
            let expected = match (succeeded, callback_success) {
                (true, _) => vec![
                    Event::MappedViewDropped,
                    Event::StagingUnmapped,
                    Event::StagingDropped,
                ],
                (false, true) => vec![Event::StagingUnmapped, Event::StagingDropped],
                (false, false) => vec![Event::StagingDropped],
            };
            assert_eq!(events.events(), expected);
            assert_eq!(
                generation
                    .class_usage(GenerationClass::ReadbackStaging)
                    .unwrap()
                    .live(RenderResource::GpuBytes),
                0
            );
            assert_eq!(prefix, prefix_before);
        }
    }
}

#[test]
fn native_helper_spawn_failure_preserves_io_source_and_cleans_the_real_registration_once() {
    let options =
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false });
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device,
        _queue,
        submitted,
        generation,
        prefix,
    } = PreparedMap::new(&cpu, options);
    let prefix_before = prefix.snapshot_for_test();
    let (result, started, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            REFUSE_HELPER_SPAWN.with(|refuse| assert!(!refuse.replace(true)));
            ReadbackMapFuture::start(submitted.bind(attempt.account()), device)
        })
        .unwrap()
        .into_parts();
    assert!(started);
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("actual helper spawn was refused"),
    };
    assert_eq!(error.code(), crate::ErrorCode::ReadbackFailed);
    assert_eq!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::Other
    );
    assert_eq!(fault, None);
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::ReadbackFailed)
    );
    assert_eq!(report.cleanup_fault(), None);
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .phase_count(RenderPhase::ReadbackCleanup),
        1
    );
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::ReadbackMapRequests), 1);
    assert_eq!(usage.work(RenderWork::ReadbackRows), 0);
    assert_eq!(usage.work(RenderWork::ReadbackBytes), 0);
    assert_eq!(
        generation
            .class_usage(GenerationClass::ReadbackStaging)
            .unwrap()
            .live(RenderResource::GpuBytes),
        0
    );
    assert_eq!(prefix, prefix_before);
    assert!(!REFUSE_HELPER_SPAWN.with(std::cell::Cell::get));
}

thread_local! {
    static DROP_GENERATION: RefCell<Option<Generation>> = const { RefCell::new(None) };
    static DROP_MAP: RefCell<Option<Arc<ReadbackStagingMapState>>> = const { RefCell::new(None) };
    static MAP_CLOCK_SAMPLES: RefCell<Vec<(u64, usize)>> = const { RefCell::new(Vec::new()) };
}
fn cleanup_milestone(event: test_support::ReadbackCleanupEventForTest) {
    use test_support::ReadbackCleanupEventForTest as Event;
    let millis = match event {
        Event::MappedViewDropped => 20,
        Event::StagingUnmapped => 30,
        Event::StagingDropped => {
            DROP_GENERATION.with(|generation| {
                assert_eq!(
                    generation
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .class_usage(GenerationClass::ReadbackStaging)
                        .unwrap()
                        .live(RenderResource::GpuBytes),
                    0,
                    "the real native buffer and final local hold dropped before this trailing field"
                );
            });
            40
        }
        Event::PublishedBytes => return,
    };
    CLOCK.with(|clock| {
        clock
            .borrow()
            .as_ref()
            .unwrap()
            .millis
            .store(millis, Ordering::SeqCst)
    });
}
struct DropGenerationBinding;
impl DropGenerationBinding {
    fn new(generation: &Generation) -> Self {
        DROP_GENERATION.with(|slot| assert!(slot.replace(Some(generation.clone())).is_none()));
        Self
    }
}
impl Drop for DropGenerationBinding {
    fn drop(&mut self) {
        DROP_GENERATION.with(|slot| {
            slot.take();
        });
        DROP_MAP.with(|slot| {
            slot.take();
        });
        MAP_CLOCK_SAMPLES.with(|samples| samples.borrow_mut().clear());
    }
}

#[test]
fn actual_view_and_staging_fields_finish_cleanup_inside_decode_before_output_close() {
    let manual_clock = Arc::new(ManualClock::default());
    let _binding = ClockBinding::new(&manual_clock);
    let options = RenderAccountingOptions::new()
        .with_clock(RenderClock::Host(clock))
        .with_profile(RenderProfile::Summary { timing: true });
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device: _device,
        _queue,
        generation,
        submitted,
        ..
    } = PreparedMap::new(&cpu, options);
    let _drop_binding = DropGenerationBinding::new(&generation);
    let events = test_support::CleanupObservation::with_hook(cleanup_milestone);
    drop(_queue);
    let (output, started, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account();
            manual_clock.millis.store(10, Ordering::SeqCst);
            account.phase(RenderPhase::ReadbackMap, || {
                let mut owner = submitted.bind(account);
                owner.staging.probe.0 = Some(events.clone());
                DROP_MAP.with(|slot| {
                    assert!(slot.replace(Some(owner.staging.observation())).is_none())
                });
                let mut map = Box::pin(ReadbackMapFuture::start_manual(owner).unwrap());
                match map.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
                    Poll::Ready(output) => output,
                    Poll::Pending => panic!("real no-Queue provider completion"),
                }
            })
        })
        .unwrap()
        .into_parts();
    assert!(started);
    assert_eq!(fault, None);
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    let attempt = report.attempt().unwrap();
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::ReadbackCleanup),
        Some(Duration::from_millis(20))
    );
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::ReadbackDecode),
        Some(Duration::from_millis(30))
    );
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::ReadbackMap),
        Some(Duration::from_millis(30))
    );
    assert_eq!(
        events.events(),
        vec![
            test_support::ReadbackCleanupEventForTest::MappedViewDropped,
            test_support::ReadbackCleanupEventForTest::StagingUnmapped,
            test_support::ReadbackCleanupEventForTest::StagingDropped,
        ]
    );
    MAP_CLOCK_SAMPLES.with(|samples| {
        let samples = samples.borrow();
        let tail_end = samples
            .iter()
            .position(|sample| *sample == (40, 2))
            .expect("cleanup end samples while its metadata Arc field is still retained");
        assert!(
            samples[tail_end + 1..].contains(&(40, 1)),
            "metadata Arc drops after cleanup end, before decode/output completion: {samples:?}"
        );
    });
    let closed = report.snapshot_for_test();
    assert_eq!(output.unwrap().transfer().unwrap().rgba(), &[0; 24]);
    assert_eq!(report, closed);
}

#[test]
fn canceled_actual_map_child_drops_staging_before_map_phase_and_report_close() {
    let manual_clock = Arc::new(ManualClock::default());
    let _binding = ClockBinding::new(&manual_clock);
    let options = RenderAccountingOptions::new()
        .with_clock(RenderClock::Host(clock))
        .with_profile(RenderProfile::Summary { timing: true });
    let cpu = RendererCpu::new(options).unwrap();
    let PreparedMap {
        device: _device,
        _queue,
        generation,
        submitted,
        ..
    } = PreparedMap::new(&cpu, options);
    let _drop_binding = DropGenerationBinding::new(&generation);
    let events = test_support::CleanupObservation::with_hook(cleanup_milestone);
    let mut fixture = ReadbackFixture::new(&cpu, options);
    manual_clock.millis.store(10, Ordering::SeqCst);
    let future = fixture.run(async |mut attempt| {
        attempt.attach_generation(&generation).unwrap();
        let account = attempt.account().clone();
        let mut owner = submitted.bind(&account);
        owner.staging.probe.0 = Some(events.clone());
        account
            .async_phase(RenderPhase::ReadbackMap, async move {
                ReadbackMapFuture::start_manual(owner)?.await
            })
            .await
    });
    let mut future = Box::pin(future);
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    manual_clock.millis.store(20, Ordering::SeqCst);
    drop(future);
    let report = fixture.report.take().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Canceled);
    assert_eq!(report.cleanup_fault(), None);
    let attempt = report.attempt().unwrap();
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::ReadbackCleanup),
        Some(Duration::from_millis(20))
    );
    assert_eq!(
        attempt.phase_elapsed(RenderPhase::ReadbackMap),
        Some(Duration::from_millis(30))
    );
    assert_eq!(attempt.phase_count(RenderPhase::ReadbackCleanup), 1);
    assert_eq!(
        events.events(),
        vec![
            test_support::ReadbackCleanupEventForTest::StagingUnmapped,
            test_support::ReadbackCleanupEventForTest::StagingDropped,
        ]
    );
    assert_eq!(
        report
            .generation()
            .unwrap()
            .unwrap()
            .usage()
            .live(RenderResource::GpuBytes),
        0
    );
}

#[test]
fn actual_callback_after_local_release_cannot_resurrect_closed_custody_or_reports() {
    for retire in [false, true] {
        let manual_clock = Arc::new(ManualClock::default());
        let _binding = ClockBinding::new(&manual_clock);
        let options = RenderAccountingOptions::new()
            .with_clock(RenderClock::Host(clock))
            .with_profile(RenderProfile::Summary { timing: true });
        let cpu = RendererCpu::new(options).unwrap();
        let PreparedMap {
            device,
            _queue,
            generation,
            submitted,
            prefix,
        } = PreparedMap::new(&cpu, options);
        let prefix_before = prefix.snapshot_for_test();
        let index = submitted.index.clone();
        let gate = Arc::new(PausedWake::default());
        let wakes = Arc::new(CountedWake::default());
        let waker = Waker::from(wakes.clone());
        let observation = std::cell::OnceCell::new();
        let mut fixture = ReadbackFixture::new(&cpu, options);
        let future = fixture.run(async |mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            let owner = submitted.bind(&account);
            assert!(observation.set(owner.staging.observation()).is_ok());
            CALLBACK_ENTRY_GATE.with(|slot| assert!(slot.replace(Some(gate.clone())).is_none()));
            manual_clock.millis.store(10, Ordering::SeqCst);
            account
                .async_phase(RenderPhase::ReadbackMap, async move {
                    ReadbackMapFuture::start_manual(owner)?.await
                })
                .await
        });
        let mut future = Box::pin(future);
        assert!(matches!(
            future.as_mut().poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        ));
        let poll = OwnedPoll::start(device, index, manual_clock.clone(), gate.clone());
        gate.await_entry();
        // This is the actual provider's once-callback before either physical or
        // logical publication, unlike the separate after-wake custody cases.
        let map = observation.get().unwrap();
        assert_eq!(
            map.disposition_for_test(),
            ReadbackStagingDisposition::MapPending
        );
        assert_eq!(wakes.0.load(Ordering::SeqCst), 0);
        manual_clock.millis.store(70, Ordering::SeqCst);
        drop(future);
        assert_eq!(
            map.disposition_for_test(),
            ReadbackStagingDisposition::Released
        );
        let canceled = fixture.report.take().unwrap();
        let closed = canceled.snapshot_for_test();
        assert_eq!(canceled.status(), RenderAttemptStatus::Canceled);
        assert_eq!(canceled.cleanup_fault(), None);
        assert_eq!(
            canceled
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ReadbackMapRequests),
            1
        );
        assert_eq!(
            canceled
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::ReadbackRows),
            0
        );
        assert_eq!(
            canceled
                .attempt()
                .unwrap()
                .phase_count(RenderPhase::ReadbackCleanup),
            1
        );
        assert_eq!(
            canceled
                .attempt()
                .unwrap()
                .phase_elapsed(RenderPhase::ReadbackMap),
            Some(Duration::from_millis(60))
        );
        assert_eq!(
            canceled
                .generation()
                .unwrap()
                .unwrap()
                .usage()
                .live(RenderResource::GpuBytes),
            512
        );
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
                0
            );
            Some(RenderAccountingError::GenerationRetired)
        } else {
            None
        };
        poll.finish();
        assert_eq!(
            wakes.0.load(Ordering::SeqCst),
            0,
            "canceled logical completion does not wake"
        );
        assert_eq!(
            map.disposition_for_test(),
            ReadbackStagingDisposition::Released
        );
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
        assert_eq!(canceled, closed);
        assert_eq!(prefix, prefix_before);
        assert!(CALLBACK_ENTRY_GATE.with(|slot| slot.borrow().is_none()));
    }
}

#[test]
fn real_readback_cleanup_keeps_work_and_output_invariant_across_profiles_filters_and_detail_loss() {
    use crate::accounting::{RenderCpuScope, RenderEvent};
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 0,
            filter: RenderPhaseFilter::All,
        },
        RenderProfile::Detailed {
            timing: true,
            event_limit: 16,
            filter: RenderPhaseFilter::Phase(RenderPhase::ReadbackCleanup),
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 64,
            filter: RenderPhaseFilter::RendererCpuScope(RenderCpuScope::ReadbackOutput),
        },
        RenderProfile::Detailed {
            timing: false,
            event_limit: 64,
            filter: RenderPhaseFilter::GenerationScope(RenderGenerationScope::ReadbackStaging),
        },
    ] {
        let manual_clock = Arc::new(ManualClock::default());
        let _binding = ClockBinding::new(&manual_clock);
        let options = RenderAccountingOptions::new()
            .with_clock(RenderClock::Host(clock))
            .with_profile(profile);
        let cpu = RendererCpu::new(options).unwrap();
        let PreparedMap {
            device: _device,
            _queue,
            generation,
            submitted,
            ..
        } = PreparedMap::new(&cpu, options);
        drop(_queue);
        let (output, started, report, fault) = ReadbackFixture::new(&cpu, options)
            .run_sync(|mut attempt| {
                attempt.attach_generation(&generation).unwrap();
                let account = attempt.account();
                account.phase(RenderPhase::ReadbackMap, || {
                    let mut map =
                        Box::pin(ReadbackMapFuture::start_manual(submitted.bind(account)).unwrap());
                    match map.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
                        Poll::Ready(output) => output,
                        Poll::Pending => panic!("actual synchronous native completion"),
                    }
                })
            })
            .unwrap()
            .into_parts();
        assert!(started);
        assert_eq!(fault, None);
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.cleanup_fault(), None);
        let attempt = report.attempt().unwrap();
        let work = attempt.usage();
        for (unit, count) in [
            (RenderWork::ReadbackMapRequests, 1),
            (RenderWork::ReadbackRows, 2),
            (RenderWork::ReadbackBytes, 24),
            (RenderWork::ReadbackCopies, 0),
            (RenderWork::Submissions, 0),
            (RenderWork::ReadbackTransferredBytes, 0),
            (RenderWork::ResourceAllocationRequests, 0),
        ] {
            assert_eq!(work.work(unit), count, "{profile:?}: {unit:?}");
        }
        assert_eq!(
            attempt.phase_count(RenderPhase::ReadbackCleanup),
            u64::from(profile != RenderProfile::Off)
        );
        let native = report.generation().unwrap().unwrap();
        assert_eq!(
            native.scope_opening(
                RenderGenerationScope::ReadbackStaging,
                RenderResource::GpuBytes
            ),
            512
        );
        assert_eq!(
            native
                .scope_usage(RenderGenerationScope::ReadbackStaging)
                .live(RenderResource::GpuBytes),
            0
        );
        assert_eq!(
            native
                .scope_usage(RenderGenerationScope::ReadbackStaging)
                .peak(RenderResource::GpuBytes),
            512
        );
        if let RenderProfile::Detailed { event_limit: 0, .. } = profile {
            assert_eq!(attempt.events().count(), 0);
            assert!(attempt.dropped_events() > 0);
        }
        if let RenderProfile::Detailed {
            filter: RenderPhaseFilter::Phase(RenderPhase::ReadbackCleanup),
            ..
        } = profile
        {
            let events: Vec<_> = attempt.events().collect();
            assert!(matches!(
                events.as_slice(),
                [RenderEvent::Phase {
                    phase: RenderPhase::ReadbackCleanup,
                    ..
                }]
            ));
        }
        if let RenderProfile::Detailed {
            filter: RenderPhaseFilter::RendererCpuScope(_),
            ..
        } = profile
        {
            assert_eq!(attempt.events().count(), 0);
            assert!(report.renderer_cpu().unwrap().events().count() > 0);
        }
        if let RenderProfile::Detailed {
            filter: RenderPhaseFilter::GenerationScope(_),
            ..
        } = profile
        {
            assert_eq!(attempt.events().count(), 0);
            assert!(native.events().count() > 0);
        }
        let capacity = report
            .renderer_cpu()
            .unwrap()
            .scope_usage(RenderCpuScope::ReadbackOutput)
            .live(RenderResource::RetainedCpuBytes);
        let closed = report.snapshot_for_test();
        let image = output.unwrap().transfer().unwrap();
        let reads_after_handoff = manual_clock.reads.load(Ordering::SeqCst);
        assert_eq!(image.rgba(), &[0; 24]);
        assert_eq!(image.into_rgba().capacity() as u64, capacity);
        assert_eq!(
            manual_clock.reads.load(Ordering::SeqCst),
            reads_after_handoff
        );
        assert_eq!(report, closed);
        if !matches!(
            profile,
            RenderProfile::Summary { timing: true } | RenderProfile::Detailed { timing: true, .. }
        ) {
            assert_eq!(
                reads_after_handoff, 0,
                "all actual untimed cleanup/transfer paths avoid clocks: {profile:?}"
            );
        }
        generation.ensure_healthy().unwrap();
    }
}
