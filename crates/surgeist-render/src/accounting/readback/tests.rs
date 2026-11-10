use super::*;

#[test]
fn completed_readback_retains_pretransfer_output_and_uses_its_own_destination() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let opening = cpu
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let (_, _, frame, _) = FrameFixture::new(&cpu, options)
        .run_sync(|_| Ok(()))
        .unwrap()
        .into_parts();
    let frames = Some(frame.snapshot_for_test());
    let mut readbacks = ReadbackFixture::new(&cpu, options);
    let (output, started, report, fault) = readbacks
        .run_sync(|attempt| {
            let output = attempt.account().retained::<u8>(7)?;
            attempt.account().work(RenderWork::ReadbackRows, 2)?;
            Ok(output)
        })
        .unwrap()
        .into_parts();
    let output = output.unwrap();
    assert!(started);
    assert_eq!(fault, None);
    assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(
        report.attempt().unwrap().kind(),
        RenderAttemptKind::Readback
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ReadbackRows),
        2
    );
    assert_eq!(report.generation(), None);
    let interval = report.renderer_cpu().unwrap();
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        opening + 7
    );
    assert_eq!(
        interval
            .scope_usage(RenderCpuScope::ReadbackOutput)
            .live(RenderResource::RetainedCpuBytes),
        7
    );
    assert_eq!(
        interval
            .scope_usage(RenderCpuScope::FrameBacking)
            .live(RenderResource::RetainedCpuBytes),
        0
    );
    assert_eq!(
        interval.usage().work(RenderWork::ReadbackTransferredBytes),
        0
    );
    drop(output);
    assert_eq!(
        interval.usage().live(RenderResource::RetainedCpuBytes),
        opening + 7
    );
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .live(RenderResource::RetainedCpuBytes),
        opening
    );
    assert!(
        readbacks.report.is_none(),
        "finish returns the report for final handoff"
    );
    readbacks.report = Some(report.snapshot_for_test());
    {
        let first = readbacks.report.as_ref().unwrap();
        let second = readbacks.report.as_ref().unwrap();
        assert_eq!(first, &report);
        assert_eq!(second, &report);
        assert_eq!(frames.as_ref().unwrap(), &frame);
    }
    assert_eq!(readbacks.report.take(), Some(report));
    assert!(readbacks.report.take().is_none());
    assert_eq!(frames, Some(frame));
}

#[test]
fn cancellation_closes_after_inner_output_and_generation_cleanup_and_allows_retry() {
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let opening = cpu
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let generation = Generation::new(options).unwrap();
    let mut fixture = ReadbackFixture::new(&cpu, options);
    {
        let future = fixture.run(async |mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            let account = attempt.account().clone();
            account
                .async_phase(RenderPhase::ReadbackMap, async {
                    let _output = account.retained::<u8>(5)?;
                    let _native =
                        generation.opaque(GpuObjectKind::Sampler, Some(&account), || ())?;
                    account.work(RenderWork::ReadbackMapRequests, 1)?;
                    std::future::pending::<()>().await;
                    Ok(())
                })
                .await
                .unwrap();
            Ok(())
        });
        let mut future = std::pin::pin!(future);
        assert!(matches!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop())),
            Poll::Pending
        ));
        // Report publication remains exclusively terminal.
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .live(RenderResource::RetainedCpuBytes),
            opening + 5
        );
    }
    let canceled = fixture.report.take().unwrap();
    assert_eq!(canceled.status(), RenderAttemptStatus::Canceled);
    assert_eq!(
        canceled.attempt().unwrap().kind(),
        RenderAttemptKind::Readback
    );
    assert_eq!(
        canceled
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ReadbackMapRequests),
        1
    );
    let output = canceled.renderer_cpu().unwrap().usage();
    assert_eq!(output.peak(RenderResource::RetainedCpuBytes), opening + 5);
    assert_eq!(output.live(RenderResource::RetainedCpuBytes), opening);
    let native = canceled.generation().unwrap().unwrap().usage();
    assert_eq!(native.peak(RenderResource::OpaqueGpuObjects), 1);
    assert_eq!(native.live(RenderResource::OpaqueGpuObjects), 0);
    let (_, _, retry, fault) = fixture
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation)?;
            Ok(())
        })
        .unwrap()
        .into_parts();
    assert_eq!(fault, None);
    assert_eq!(retry.status(), RenderAttemptStatus::Succeeded);
    assert_eq!(
        retry
            .renderer_cpu()
            .unwrap()
            .usage()
            .peak(RenderResource::RetainedCpuBytes),
        opening
    );
}

#[test]
fn rejected_setup_preserves_the_previous_readback_and_cpu_lifetime() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let mut fixture = ReadbackFixture::new(&cpu, options);
    let (_, _, prior, _) = fixture.run_sync(|_| Ok(())).unwrap().into_parts();
    fixture.report = Some(prior.snapshot_for_test());
    let before = cpu.snapshot().unwrap().usage();
    let invalid = RenderAccountingOptions::new().with_limits(
        RenderLimits::new()
            .with_work_limit(RenderWork::AuthoredCommands, 0)
            .unwrap(),
    );
    let unavailable = RenderAccountingOptions::new().with_profile(RenderProfile::Detailed {
        timing: false,
        event_limit: usize::MAX,
        filter: RenderPhaseFilter::All,
    });
    for (configuration, expected) in [
        (
            invalid,
            RenderAccountingError::InvalidLimit(RenderWork::AuthoredCommands),
        ),
        (
            unavailable,
            RenderAccountingError::Setup(RenderAccountingSetupError::Allocation),
        ),
    ] {
        fixture.options = configuration;
        let error = match fixture.run_sync(|_| Ok(())) {
            Err(error) => error,
            Ok(_) => panic!("rejected setup unexpectedly started"),
        };
        assert_eq!(error.accounting_diagnostic(), Some(expected));
        assert_eq!(fixture.report.as_ref().unwrap(), &prior);
        assert_eq!(cpu.snapshot().unwrap().usage(), before);
    }
    // Failure to open the renderer CPU checkpoint also closes the fresh attempt
    // checkpoint without replacing the previous completed report.
    let busy = cpu.begin_interval().unwrap();
    fixture.options = options;
    assert!(fixture.run_sync(|_| Ok(())).is_err());
    assert_eq!(fixture.report.as_ref().unwrap(), &prior);
    drop(busy);
    let (_, _, _, fault) = fixture.run_sync(|_| Ok(())).unwrap().into_parts();
    assert_eq!(fault, None);
}

#[test]
fn unavailable_cpu_interval_preserves_exact_readback_and_generation_domains() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let generation = Generation::new(options).unwrap();
    let (_, _, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|mut attempt| {
            attempt.attach_generation(&generation).unwrap();
            attempt
                .account()
                .work(RenderWork::ReadbackCopies, 1)
                .unwrap();
            drop(attempt.core.cpu_checkpoint.take());

            Ok(())
        })
        .unwrap()
        .into_parts();
    assert_eq!(fault, Some(RenderAccountingError::Invariant));
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
    );
    assert_eq!(report.attempt().unwrap().status(), report.status());
    assert_eq!(report.renderer_cpu(), Err(RenderAccountingError::Invariant));
    assert!(report.generation().unwrap().is_ok());
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ReadbackCopies),
        1
    );
}

#[test]
fn ordinary_cpu_refusal_is_recoverable_and_persistent_fault_keeps_the_first_cause() {
    let options =
        RenderAccountingOptions::new().with_limits(RenderLimits::new().with_scratch_bytes(0));
    let cpu = RendererCpu::new(options).unwrap();
    for unhealthy in [false, true] {
        let (result, _, report, fault) = ReadbackFixture::new(&cpu, options)
            .run_sync(|attempt| {
                let error = attempt.account().temporary::<u8>(1).unwrap_err();
                assert!(matches!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::CpuRefused { .. })
                ));
                assert_eq!(cpu.fault(), None);
                if unhealthy {
                    cpu.at(RenderCpuScope::ReadbackOutput)
                        .note_fault(RenderAccountingError::Invariant);
                }
                Err::<(), _>(error)
            })
            .unwrap()
            .into_parts();
        let first = result.unwrap_err().accounting_diagnostic().unwrap();
        assert_eq!(
            report.status(),
            if unhealthy {
                RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
            } else {
                RenderAttemptStatus::Refused
            }
        );
        assert_eq!(report.accounting_diagnostic(), Some(first));
        assert_eq!(fault, unhealthy.then_some(RenderAccountingError::Invariant));
        assert_eq!(report.cleanup_fault(), fault);
    }
}

#[test]
fn failed_final_transfer_changes_both_statuses_without_rewriting_closed_observations() {
    let options = RenderAccountingOptions::new();
    let cpu = RendererCpu::new(options).unwrap();
    let (output, _, mut report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|attempt| {
            let output = attempt.account().retained::<u8>(4).unwrap();
            attempt
                .account()
                .work(RenderWork::ReadbackBytes, 4)
                .unwrap();

            Ok(output)
        })
        .unwrap()
        .into_parts();
    let output = output.unwrap();
    assert_eq!(fault, None);
    let before = report.snapshot_for_test();
    let first = RenderAccountingError::CpuRefused {
        scope: RenderCpuScope::ReadbackOutput,
        quantity: RenderQuantity::Work(RenderWork::ReadbackTransferredBytes),
        current: u64::MAX,
        amount: 4,
        reason: RenderRefusalReason::Overflow,
    };
    let cleanup = RenderAccountingError::Invariant;
    report.fail_after_close(&first.into(), Some(cleanup));
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
    );
    assert_eq!(report.attempt().unwrap().status(), report.status());
    assert_eq!(report.accounting_diagnostic(), Some(first));
    assert_eq!(report.cleanup_fault(), Some(cleanup));
    assert_eq!(report.renderer_cpu(), before.renderer_cpu());
    assert_eq!(report.generation(), before.generation());
    assert_eq!(
        report.attempt().unwrap().usage(),
        before.attempt().unwrap().usage()
    );
    assert_eq!(
        report.attempt().unwrap().elapsed(),
        before.attempt().unwrap().elapsed()
    );
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::ReadbackTransferredBytes),
        0
    );
    drop(output);
    assert_eq!(report.renderer_cpu(), before.renderer_cpu());
    report.fail_after_close(&cleanup.into(), Some(first));
    assert_eq!(report.accounting_diagnostic(), Some(first));
    assert_eq!(report.cleanup_fault(), Some(cleanup));
}
