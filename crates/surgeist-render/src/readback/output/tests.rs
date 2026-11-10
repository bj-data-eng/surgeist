use super::*;
use crate::accounting::{
    ReadbackFixture, RenderAccountingError, RenderAccountingOptions, RenderCpuScope,
    RenderResource, RenderWork, RendererCpu,
};

#[test]
fn rejected_image_validation_and_prior_health_fault_dispose_the_admitted_output() {
    for unhealthy in [false, true] {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let mut actual = 0;
        let (_, _, report, fault) = ReadbackFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let mut rows = attempt.account().retained::<u8>(24).unwrap();
                // An incomplete ImageBuffer is rejected independently of mapped-row
                // validation. The other case reaches the CPU phase's earlier health gate.
                rows.extend_from_slice(&[9; 23]).unwrap();
                actual = rows.capacity() as u64;
                if unhealthy {
                    cpu.at(RenderCpuScope::ReadbackOutput)
                        .note_fault(RenderAccountingError::Invariant);
                }
                let result = ReadbackOutput::try_new(PhysicalSize::new(3, 2), rows);
                let Err(error) = result else {
                    panic!("invalid or unhealthy output cannot escape its guard")
                };
                assert_eq!(
                    error.code(),
                    if unhealthy {
                        crate::ErrorCode::Accounting
                    } else {
                        crate::ErrorCode::ReadbackFailed
                    }
                );

                Err::<(), _>(error)
            })
            .unwrap()
            .into_parts();
        assert_eq!(fault, unhealthy.then_some(RenderAccountingError::Invariant));
        let usage = report
            .renderer_cpu()
            .unwrap()
            .scope_usage(RenderCpuScope::ReadbackOutput);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), actual);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ReadbackTransferredBytes),
            0
        );
    }
}

thread_local! {
    static CLOCK_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn counted_clock() -> std::time::Duration {
    CLOCK_READS.with(|reads| reads.set(reads.get() + 1));
    std::time::Duration::ZERO
}

#[test]
fn qualified_output_transfers_original_spare_capacity_without_post_handoff_observation() {
    use crate::accounting::{RenderClock, RenderProfile};
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        for (size, requested, bytes) in [
            (PhysicalSize::new(0, 4), 0, 0),
            (PhysicalSize::new(3, 2), 40, 24),
        ] {
            CLOCK_READS.with(|reads| reads.set(0));
            let options = RenderAccountingOptions::new()
                .with_profile(profile)
                .with_clock(RenderClock::Host(counted_clock));
            let cpu = RendererCpu::new(options).unwrap();
            let (output, _, _, fault) = ReadbackFixture::new(&cpu, options)
                .run_sync(|attempt| {
                    let mut rows = attempt.account().retained::<u8>(requested).unwrap();
                    rows.extend_from_slice(&vec![7; bytes]).unwrap();
                    let pointer = rows.as_slice().as_ptr();
                    let capacity = rows.capacity();
                    let output = ReadbackOutput::try_new(size, rows).unwrap();

                    Ok((output, pointer, capacity))
                })
                .unwrap()
                .into_parts();
            let (output, pointer, capacity) = output.unwrap();
            assert_eq!(fault, None);
            let image = output.transfer().unwrap();
            let after_transfer = CLOCK_READS.with(std::cell::Cell::get);
            assert_eq!(image.size(), size);
            assert_eq!(image.rgba(), vec![7; bytes]);
            assert_eq!(image.rgba().as_ptr(), pointer);
            let clone = image.clone();
            let values = image.into_rgba();
            assert_eq!(values.capacity(), capacity);
            drop(values);
            drop(clone);
            assert_eq!(CLOCK_READS.with(std::cell::Cell::get), after_transfer);
            if profile != (RenderProfile::Summary { timing: true }) {
                assert_eq!(after_transfer, 0);
            }
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .work(RenderWork::ReadbackTransferredBytes),
                capacity as u64
            );
        }
    }
}

#[test]
fn malformed_and_overflowing_output_preserve_the_numeric_source_and_release_capacity() {
    for (size, length) in [
        (PhysicalSize::new(3, 2), 23),
        (PhysicalSize::new(u32::MAX, u32::MAX), 0),
        (PhysicalSize::new(0, 7), 1),
    ] {
        let options = RenderAccountingOptions::new()
            .with_profile(crate::RenderProfile::Summary { timing: false });
        let cpu = RendererCpu::new(options).unwrap();
        let (_, _, _report, fault) = ReadbackFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let mut rows = attempt.account().retained::<u8>(length).unwrap();
                rows.extend_from_slice(&vec![0; length]).unwrap();
                let error = match ReadbackOutput::try_new(size, rows) {
                    Ok(_) => panic!("invalid bytes cannot form a complete readback"),
                    Err(error) => error,
                };
                assert_eq!(error.code(), crate::ErrorCode::ReadbackFailed);
                let source = std::error::Error::source(&error)
                    .unwrap()
                    .downcast_ref::<crate::Error>()
                    .unwrap();
                assert_eq!(source.code(), crate::ErrorCode::InvalidInput);
                assert_eq!(
                    cpu.snapshot()
                        .unwrap()
                        .scope_usage(RenderCpuScope::ReadbackOutput)
                        .unwrap()
                        .live(RenderResource::RetainedCpuBytes),
                    0
                );

                Err::<(), _>(error)
            })
            .unwrap()
            .into_parts();
        assert_eq!(fault, None);
    }
}

#[test]
fn rejected_length_ends_validation_before_separate_real_vector_disposal() {
    use crate::accounting::{RenderChangeKind, RenderCpuEvent, RenderPhaseFilter, RenderProfile};
    let options = RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
        timing: false,
        event_limit: 128,
        filter: RenderPhaseFilter::All,
    });
    let cpu = RendererCpu::new(options).unwrap();
    let (_, _, _report, fault) = ReadbackFixture::new(&cpu, options).run_sync(|attempt| {
    let mut rows = attempt.account().retained::<u8>(24).unwrap();
    rows.extend_from_slice(&[9; 23]).unwrap();
    let actual = rows.capacity() as u64;
    let before = cpu.snapshot().unwrap();
    let events_before = before.events().count();
    let count_before = before.scope_count(RenderCpuScope::ReadbackOutput);
    let error = match ReadbackOutput::try_new(PhysicalSize::new(3, 2), rows) {
        Err(error) => error,
        Ok(_) => panic!("incomplete pixels must refuse"),
    };
    assert_eq!(error.code(), crate::ErrorCode::ReadbackFailed);
    let after = cpu.snapshot().unwrap();
    assert_eq!(
        after.scope_count(RenderCpuScope::ReadbackOutput) - count_before,
        2
    );
    let events: Vec<_> = after.events().skip(events_before).collect();
    assert!(matches!(events.as_slice(), [
        RenderCpuEvent::Phase { scope: RenderCpuScope::ReadbackOutput, duration: None, .. },
        RenderCpuEvent::Change { scope: RenderCpuScope::ReadbackOutput,
            kind: RenderChangeKind::Release, quantity: crate::RenderQuantity::Resource(RenderResource::RetainedCpuBytes), amount, .. },
        RenderCpuEvent::Phase { scope: RenderCpuScope::ReadbackOutput, duration: None, .. },
    ] if *amount == actual));

Err::<(), _>(error)
}).unwrap().into_parts();
    assert_eq!(fault, None);
}

#[test]
fn releasing_a_real_blocker_allows_the_same_readback_owner_to_retry_unchanged_pixels() {
    use crate::accounting::{RenderLimits, RenderProfile};
    let mut producer = Vec::<u8>::new();
    producer.try_reserve_exact(24).unwrap();
    let capacity = producer.capacity();
    let baseline = RendererCpu::new(RenderAccountingOptions::new())
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let options = RenderAccountingOptions::new()
        .with_profile(RenderProfile::Summary { timing: false })
        .with_limits(RenderLimits::new().with_retained_bytes(baseline + capacity as u64));
    let cpu = RendererCpu::new(options).unwrap();
    let (output, _, _, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|attempt| {
            let account = attempt.account();
            let mut blocker = account.retained::<u8>(1).unwrap();
            blocker.extend_from_slice(&[99]).unwrap();
            let error = account.retained::<u8>(24).unwrap_err();
            assert!(matches!(
                error.accounting_diagnostic(),
                Some(RenderAccountingError::CpuRefused { .. })
            ));
            assert_eq!(blocker.as_slice(), &[99]);
            assert_eq!(cpu.snapshot().unwrap().fault(), None);
            drop(blocker);
            let mut rows = account.retained::<u8>(24).unwrap();
            rows.extend_from_slice(&[7; 24]).unwrap();
            let pointer = rows.as_slice().as_ptr();
            let output = ReadbackOutput::try_new(PhysicalSize::new(3, 2), rows).unwrap();

            Ok((output, pointer))
        })
        .unwrap()
        .into_parts();
    let (output, pointer) = output.unwrap();
    assert_eq!(fault, None);
    let image = output.transfer().unwrap();
    assert_eq!(image.rgba(), &[7; 24]);
    assert_eq!(image.rgba().as_ptr(), pointer);
    assert_eq!(image.into_rgba().capacity(), capacity);
    let after = cpu.snapshot().unwrap();
    assert_eq!(
        after.usage().live(RenderResource::RetainedCpuBytes),
        baseline
    );
    assert_eq!(
        after.usage().work(RenderWork::ReadbackTransferredBytes),
        capacity as u64
    );
}
