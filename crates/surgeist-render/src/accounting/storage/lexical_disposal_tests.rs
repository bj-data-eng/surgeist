use super::*;
use crate::accounting::{RenderAccountingOptions, RenderCpuScope, RendererCpu};
use crate::accounting::{RenderClock, RenderProfile};
use crate::{Error, ErrorCode};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

static DISPOSAL_TIME: AtomicU64 = AtomicU64::new(0);
static CLOCK_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn clock() -> Duration {
    Duration::from_millis(DISPOSAL_TIME.load(Ordering::SeqCst))
}

fn forbidden_clock() -> Duration {
    panic!("untimed text-value disposal read the clock");
}

struct DropProbe {
    buffers: [Vec<u8>; 2],
    cpu: RendererCpu,
    drops: Arc<AtomicUsize>,
    admitted: u64,
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        assert_eq!(
            self.cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            self.admitted,
        );
        drop(std::mem::take(&mut self.buffers));
        // Both real allocations have been destroyed, but their paired
        // admissions must still cover the complete value destructor.
        assert_eq!(
            self.cpu
                .snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            self.admitted,
        );
        DISPOSAL_TIME.fetch_add(7, Ordering::SeqCst);
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

fn probe(
    cpu: &RendererCpu,
    drops: &Arc<AtomicUsize>,
    count: usize,
) -> (DropProbe, [Option<Hold>; 2]) {
    let opening = cpu
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let buffers = [
        if count > 0 { vec![1; 3] } else { Vec::new() },
        if count > 1 { vec![2; 5] } else { Vec::new() },
    ];
    let account = cpu.at(RenderCpuScope::FrameBacking);
    let backing = std::array::from_fn(|index| {
        (index < count).then(|| {
            Hold::Renderer(
                account
                    .acquire(RenderResource::RetainedCpuBytes, buffers[index].capacity())
                    .unwrap(),
            )
        })
    });
    let admitted = opening + buffers.iter().map(Vec::capacity).sum::<usize>() as u64;
    (
        DropProbe {
            buffers,
            cpu: cpu.clone(),
            drops: drops.clone(),
            admitted,
        },
        backing,
    )
}

fn dispose_transferred_probe(value: (DropProbe,), backing: [Option<Hold>; 2]) {
    let mut completion = DisposalCompletion::default();
    let [first, second] = backing;
    let value = value;
    let _start = completion.start([first.as_ref(), second.as_ref()], None);
    let _ = &value;
}

#[derive(Clone, Copy, Debug)]
enum Exit {
    Drop,
    Error,
    Unwind,
    Transfer,
}

#[test]
fn text_value_disposal_keeps_all_backing_live_through_one_actual_destructor() {
    let _clock = CLOCK_LOCK.lock().unwrap();
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        for (count, exit) in [
            (0, Exit::Drop),
            (1, Exit::Drop),
            (1, Exit::Error),
            (2, Exit::Drop),
            (2, Exit::Unwind),
            (2, Exit::Transfer),
        ] {
            DISPOSAL_TIME.store(0, Ordering::SeqCst);
            let timed = matches!(profile, RenderProfile::Summary { timing: true });
            let options = RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(if timed {
                    clock
                } else {
                    forbidden_clock
                }));
            let cpu = RendererCpu::new(options).unwrap();
            let checkpoint = cpu.begin_interval().unwrap();
            let drops = Arc::new(AtomicUsize::new(0));
            let exercise = || -> Result<()> {
                let mut completion = DisposalCompletion::default();
                let (incoming, [first, second]) = probe(&cpu, &drops, count);
                let first = first;
                let second = second;
                let value = incoming;
                let start = completion.start([first.as_ref(), second.as_ref()], None);
                match exit {
                    Exit::Drop => Ok(()),
                    Exit::Error => Err(Error::invalid_value(
                        "text-value fixture",
                        "refused",
                        "must return the original error",
                    )),
                    Exit::Unwind => panic!("text-value fixture unwind"),
                    Exit::Transfer => {
                        start.disarm();
                        dispose_transferred_probe((value,), [first, second]);
                        Ok(())
                    }
                }
            };
            if matches!(exit, Exit::Unwind) {
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(exercise)).is_err());
            } else if matches!(exit, Exit::Error) {
                assert_eq!(exercise().unwrap_err().code(), ErrorCode::InvalidInput);
            } else {
                exercise().unwrap();
            }
            assert_eq!(
                drops.load(Ordering::SeqCst),
                1,
                "{count} backings, {exit:?}"
            );
            let interval = checkpoint.finish().unwrap();
            assert_eq!(
                interval.usage().live(RenderResource::RetainedCpuBytes),
                interval
                    .usage()
                    .opening_live(RenderResource::RetainedCpuBytes),
            );
            if timed {
                // The destructor advances the clock by seven milliseconds.
                // Two nested disposal phases around it would report fourteen.
                assert_eq!(
                    interval
                        .scope_elapsed(RenderCpuScope::FrameBacking)
                        .unwrap_or_default(),
                    Duration::from_millis(if count == 0 { 0 } else { 7 }),
                    "{count} backings, {exit:?}",
                );
            } else {
                assert_eq!(interval.scope_elapsed(RenderCpuScope::FrameBacking), None);
            }
        }
    }
}

#[test]
fn destructor_unwind_closes_forward_before_disposal_and_releases_both_backings() {
    let _clock = CLOCK_LOCK.lock().unwrap();
    struct PanickingDrop(DropProbe);
    impl Drop for PanickingDrop {
        fn drop(&mut self) {
            let _ = &self.0;
            panic!("actual payload destructor unwind");
        }
    }
    DISPOSAL_TIME.store(0, Ordering::SeqCst);
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock)),
    )
    .unwrap();
    let checkpoint = cpu.begin_interval().unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let storage = StorageAccount::Renderer(cpu.at(RenderCpuScope::FrameBacking));
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut completion = DisposalCompletion::default();
            let (incoming, [first, second]) = probe(&cpu, &drops, 2);
            let first = first;
            let second = second;
            let value = PanickingDrop(incoming);
            let _start = completion.start([first.as_ref(), second.as_ref()], None);
            let _forward = storage.begin_phase().unwrap();
            let _ = &value;
        }))
        .is_err()
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        interval
            .usage()
            .opening_live(RenderResource::RetainedCpuBytes)
    );
    assert_eq!(
        interval.scope_elapsed(RenderCpuScope::FrameBacking),
        Some(Duration::from_millis(7))
    );
    assert_eq!(interval.scope_count(RenderCpuScope::FrameBacking), 2);
}

#[test]
fn mixed_lifetimes_select_the_first_actual_backing_and_keep_both_credits_live() {
    use crate::accounting::{RenderAccountingContext, RenderAttemptKind, RenderPhase};
    let _clock = CLOCK_LOCK.lock().unwrap();
    struct MixedProbe<'a> {
        buffers: [Vec<u8>; 2],
        preparation: &'a Account,
        cpu: &'a RendererCpu,
        prepared_bytes: u64,
        renderer_bytes: u64,
        drops: &'a AtomicUsize,
    }
    impl Drop for MixedProbe<'_> {
        fn drop(&mut self) {
            let preparation_live = || {
                self.preparation
                    .ledger
                    .usage(self.preparation.scopes[RenderPhase::Preparation.index()])
                    .unwrap()
                    .live(RenderResource::RetainedCpuBytes)
            };
            assert_eq!(preparation_live(), self.prepared_bytes);
            assert_eq!(
                self.cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                self.renderer_bytes
            );
            drop(std::mem::take(&mut self.buffers));
            assert_eq!(preparation_live(), self.prepared_bytes);
            assert_eq!(
                self.cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                self.renderer_bytes
            );
            DISPOSAL_TIME.fetch_add(7, Ordering::SeqCst);
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    for renderer_first in [false, true] {
        DISPOSAL_TIME.store(0, Ordering::SeqCst);
        let cpu = RendererCpu::new(
            RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: true })
                .with_clock(RenderClock::Host(clock)),
        )
        .unwrap();
        let renderer = cpu.at(RenderCpuScope::FrameBacking);
        let opening = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let checkpoint = cpu.begin_interval().unwrap();
        let drops = AtomicUsize::new(0);
        let mut context = RenderAccountingContext::default();
        context
            .run(RenderAttemptKind::Clip, RenderPhase::Clip, |preparation| {
                let buffers = [vec![1; 3], vec![2; 5]];
                let renderer_index = usize::from(!renderer_first);
                let holds = buffers.each_ref().map(|buffer| buffer.capacity());
                let renderer_bytes = holds[renderer_index] as u64;
                let prepared_bytes = holds[1 - renderer_index] as u64;
                let backing: [Option<Hold>; 2] = std::array::from_fn(|index| {
                    Some(if index == renderer_index {
                        Hold::Renderer(
                            renderer
                                .acquire(RenderResource::RetainedCpuBytes, holds[index])
                                .unwrap(),
                        )
                    } else {
                        preparation
                            .acquire(RenderResource::RetainedCpuBytes, holds[index])
                            .unwrap()
                    })
                });
                let mut completion = DisposalCompletion::default();
                let [first, second] = backing;
                let value = MixedProbe {
                    buffers,
                    preparation,
                    cpu: &cpu,
                    prepared_bytes,
                    renderer_bytes: opening + renderer_bytes,
                    drops: &drops,
                };
                let _start = completion.start([first.as_ref(), second.as_ref()], None);
                let _ = &value;
                Ok(())
            })
            .unwrap();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let interval = checkpoint.finish().unwrap();
        assert_eq!(
            interval
                .scope_elapsed(RenderCpuScope::FrameBacking)
                .unwrap_or_default(),
            Duration::from_millis(if renderer_first { 7 } else { 0 })
        );
        assert_eq!(
            interval.usage().live(RenderResource::RetainedCpuBytes),
            opening
        );
        assert_eq!(
            context
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            0
        );
    }
}

#[test]
fn second_only_backing_observes_the_real_destructor() {
    let _clock = CLOCK_LOCK.lock().unwrap();
    DISPOSAL_TIME.store(0, Ordering::SeqCst);
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock)),
    )
    .unwrap();
    let checkpoint = cpu.begin_interval().unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    {
        let mut completion = DisposalCompletion::default();
        let (incoming, [backing, _]) = probe(&cpu, &drops, 1);
        let second = backing;
        let value = incoming;
        let _start = completion.start([None, second.as_ref()], None);
        let _ = &value;
    }
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let interval = checkpoint.finish().unwrap();
    assert_eq!(
        interval.scope_elapsed(RenderCpuScope::FrameBacking),
        Some(Duration::from_millis(7))
    );
    assert_eq!(interval.scope_count(RenderCpuScope::FrameBacking), 1);
}

#[test]
fn import_only_lexical_disposal_keeps_original_custody_then_releases_separately() {
    use crate::accounting::{RenderCpuEvent, RenderPhaseFilter};
    let _clock = CLOCK_LOCK.lock().unwrap();
    struct ImportedProbe<'a> {
        image: Option<crate::Image>,
        cpu: &'a RendererCpu,
        admitted: u64,
        drops: &'a AtomicUsize,
    }
    impl Drop for ImportedProbe<'_> {
        fn drop(&mut self) {
            drop(self.image.take());
            assert_eq!(
                self.cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                self.admitted
            );
            self.drops.fetch_add(1, Ordering::SeqCst);
            DISPOSAL_TIME.fetch_add(7, Ordering::SeqCst);
        }
    }
    for exit in [Exit::Drop, Exit::Error, Exit::Unwind] {
        DISPOSAL_TIME.store(0, Ordering::SeqCst);
        let options = RenderAccountingOptions::new()
            .with_profile(RenderProfile::Detailed {
                timing: true,
                event_limit: 128,
                filter: RenderPhaseFilter::All,
            })
            .with_clock(RenderClock::Host(clock));
        let cpu = RendererCpu::new(options).unwrap();
        let image =
            crate::Image::from_rgba(crate::Size::new(1.0, 1.0), vec![1, 2, 3, 255]).unwrap();
        let import = cpu
            .at(RenderCpuScope::FrameBacking)
            .import_image(&image, None)
            .unwrap();
        let admitted = cpu
            .snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes);
        let checkpoint = cpu.begin_interval().unwrap();
        let drops = AtomicUsize::new(0);
        let exercise = || -> Result<()> {
            let import = import;
            let mut completion = DisposalCompletion::default();
            let value = ImportedProbe {
                image: Some(image.clone()),
                cpu: &cpu,
                admitted,
                drops: &drops,
            };
            let _start = completion.start([None, None], Some(&import));
            let _ = &value;
            match exit {
                Exit::Error => Err(Error::invalid_value(
                    "import fixture",
                    "refused",
                    "original semantic error",
                )),
                Exit::Unwind => panic!("import fixture unwind"),
                _ => Ok(()),
            }
        };
        if matches!(exit, Exit::Unwind) {
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(exercise)).is_err());
        } else if matches!(exit, Exit::Error) {
            assert_eq!(exercise().unwrap_err().code(), ErrorCode::InvalidInput);
        } else {
            exercise().unwrap();
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let interval = checkpoint.finish().unwrap();
        let events: Vec<_> = interval.events().collect();
        let first = events
            .iter()
            .position(|event| {
                matches!(event, RenderCpuEvent::Phase {
            scope: RenderCpuScope::Cleanup, duration: Some(duration), ..
        } if *duration == Duration::from_millis(7))
            })
            .unwrap();
        assert!(events[first + 1..].iter().any(|event| matches!(
            event,
            RenderCpuEvent::Change {
                scope: RenderCpuScope::ImagePayloads,
                kind: crate::RenderChangeKind::Release,
                ..
            }
        )));
        assert!(matches!(
            events.last(),
            Some(RenderCpuEvent::Phase {
                scope: RenderCpuScope::Cleanup,
                duration: Some(Duration::ZERO),
                ..
            })
        ));
        assert_eq!(
            interval.scope_elapsed(RenderCpuScope::Cleanup),
            Some(Duration::from_millis(7))
        );
        assert_eq!(image.bytes.as_ref(), &[1, 2, 3, 255]);
        assert_eq!(cpu.snapshot().unwrap().fault(), None);
    }
}

#[test]
fn clip_forward_completion_keeps_custody_and_preserves_failure_partition() {
    use crate::accounting::{FrameAttempt, FrameReportSlot, RenderPhase};
    let _clock = CLOCK_LOCK.lock().unwrap();
    for success in [false, true] {
        DISPOSAL_TIME.store(0, Ordering::SeqCst);
        let options = RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock));
        let cpu = RendererCpu::new(options).unwrap();
        let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
        let drops = Arc::new(AtomicUsize::new(0));
        let result = (|| -> Result<()> {
            let entered = attempt.account().begin_phase(RenderPhase::Clip)?;
            let mut completion = DisposalCompletion::default();
            let (incoming, [first, second]) = probe(&cpu, &drops, 2);
            let first = first;
            let second = second;
            let value = incoming;
            let start = completion.start([first.as_ref(), second.as_ref()], None);
            if success {
                // Observation completion occurs with the mandatory payload armed.
                drop(entered);
                start.disarm();
                dispose_transferred_probe((value,), [first, second]);
                Ok(())
            } else {
                Err(Error::invalid_value(
                    "clip fixture",
                    "refused",
                    "original validation error",
                ))
            }
        })();
        let status = if success {
            result.unwrap();
            crate::RenderAttemptStatus::Succeeded
        } else {
            assert_eq!(result.unwrap_err().code(), ErrorCode::InvalidInput);
            crate::RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
        };
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let (report, fault) = attempt.finish(status, None);
        assert_eq!(fault, None);
        assert_eq!(
            report.attempt().unwrap().phase_elapsed(RenderPhase::Clip),
            Some(Duration::from_millis(if success { 0 } else { 7 }))
        );
        assert_eq!(
            report
                .renderer_cpu()
                .unwrap()
                .scope_elapsed(RenderCpuScope::FrameBacking),
            Some(Duration::from_millis(7))
        );
    }
}

#[test]
fn shared_factory_allocation_refusal_disposes_after_its_forward_phase() {
    let _clock = CLOCK_LOCK.lock().unwrap();
    let baseline = RendererCpu::new(RenderAccountingOptions::default())
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let bytes = vec![1u8, 2, 3];
    let byte_capacity = bytes.capacity();
    let limit = baseline + byte_capacity as u64;
    DISPOSAL_TIME.store(0, Ordering::SeqCst);
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock))
            .with_limits(crate::RenderLimits::new().with_retained_bytes(limit)),
    )
    .unwrap();
    let account = cpu.at(RenderCpuScope::FrameBacking);
    let storage = StorageAccount::Renderer(account.clone());
    let hold = Hold::Renderer(
        account
            .acquire(RenderResource::RetainedCpuBytes, byte_capacity)
            .unwrap(),
    );
    let drops = Arc::new(AtomicUsize::new(0));
    let value = DropProbe {
        buffers: [bytes, Vec::new()],
        cpu: cpu.clone(),
        drops: Arc::clone(&drops),
        admitted: limit,
    };
    let error = match SharedValue::with_parts(value, Some(hold), None, &storage) {
        Err(error) => error,
        Ok(_) => panic!("the fixed shared payload must exceed the retained cap"),
    };
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::CpuRefused {
            scope: RenderCpuScope::Root,
            quantity: crate::RenderQuantity::Resource(RenderResource::RetainedCpuBytes),
            current: limit,
            amount: std::mem::size_of::<SharedValueData<DropProbe>>() as u64,
            reason: crate::RenderRefusalReason::Limit { limit },
        })
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let snapshot = cpu.snapshot().unwrap();
    assert_eq!(snapshot.fault(), None);
    assert_eq!(
        snapshot.usage().live(RenderResource::RetainedCpuBytes),
        baseline
    );
    // Disposal occurs after the refused forward callback has ended,
    // rather than measuring the same destructor in two nested phases.
    assert_eq!(
        snapshot.scope_elapsed(RenderCpuScope::FrameBacking),
        Some(Duration::from_millis(7))
    );
}

#[test]
fn shared_factory_health_refusal_keeps_import_only_custody_through_value_disposal() {
    let _clock = CLOCK_LOCK.lock().unwrap();
    struct ImportedProbe {
        image: Option<crate::Image>,
        cpu: RendererCpu,
        drops: Arc<AtomicUsize>,
        observed_live: Arc<AtomicU64>,
    }
    impl Drop for ImportedProbe {
        fn drop(&mut self) {
            drop(self.image.take());
            self.observed_live.store(
                self.cpu
                    .snapshot()
                    .unwrap()
                    .usage()
                    .live(RenderResource::RetainedCpuBytes),
                Ordering::SeqCst,
            );
            DISPOSAL_TIME.store(7, Ordering::SeqCst);
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    DISPOSAL_TIME.store(0, Ordering::SeqCst);
    let cpu = RendererCpu::new(
        RenderAccountingOptions::new()
            .with_profile(RenderProfile::Summary { timing: true })
            .with_clock(RenderClock::Host(clock)),
    )
    .unwrap();
    let image = crate::Image::from_rgba(crate::Size::new(1.0, 1.0), vec![0, 0, 0, 255]).unwrap();
    let account = cpu.at(RenderCpuScope::FrameBacking);
    let import = account.import_image(&image, None).unwrap();
    let imported_live = cpu
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let drops = Arc::new(AtomicUsize::new(0));
    let observed_live = Arc::new(AtomicU64::new(u64::MAX));
    let value = ImportedProbe {
        image: Some(image.clone()),
        cpu: cpu.clone(),
        drops: Arc::clone(&drops),
        observed_live: Arc::clone(&observed_live),
    };
    account.note_fault(RenderAccountingError::Invariant);
    let error = match SharedValue::with_parts(
        value,
        None,
        Some(import),
        &StorageAccount::Renderer(account),
    ) {
        Err(error) => error,
        Ok(_) => panic!("the existing CPU health fault must be preserved"),
    };
    assert_eq!(
        error.accounting_diagnostic(),
        Some(RenderAccountingError::Invariant)
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(observed_live.load(Ordering::SeqCst), imported_live);
    let snapshot = cpu.snapshot().unwrap();
    assert_eq!(snapshot.fault(), Some(RenderAccountingError::Invariant));
    assert_eq!(
        snapshot.scope_elapsed(RenderCpuScope::Cleanup),
        Some(Duration::from_millis(7))
    );
    // The registry keeps its real capacity, but releases its pixel and
    // typed adapter custody despite the caller's Image still existing.
    assert_eq!(
        snapshot.usage().live(RenderResource::RetainedCpuBytes)
            + image.bytes.len() as u64
            + std::mem::size_of::<Arc<[u8]>>() as u64,
        imported_live,
    );
}
