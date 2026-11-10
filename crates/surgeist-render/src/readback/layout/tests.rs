use super::*;
use crate::{
    accounting::{
        ReadbackFixture, RenderAccountingOptions, RenderAttemptStatus, RenderCpuScope,
        RenderLimits, RenderPhase, RenderQuantity, RenderReadbackReport, RenderResource,
        RenderWork, RendererCpu,
    },
    readback::output::ReadbackOutput,
};

fn mapped_rows() -> Vec<u8> {
    let mut mapped = vec![0xee; 512];
    mapped[..12].copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    mapped[256..268].copy_from_slice(&[12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]);
    mapped
}

fn decode(
    mapped: &[u8],
    limits: RenderLimits,
) -> (
    Result<(ReadbackOutput, u64)>,
    RenderReadbackReport,
    RendererCpu,
) {
    let options = RenderAccountingOptions::new().with_limits(limits);
    let cpu = RendererCpu::new(options).unwrap();
    let (result, _, report, fault) = ReadbackFixture::new(&cpu, options)
        .run_sync(|attempt| {
            let account = attempt.account().at(RenderPhase::ReadbackDecode);
            let size = PhysicalSize::new(3, 2);
            let layout = ReadbackLayout::try_new(size).unwrap();
            assert_eq!(layout.buffer_size(), 512);
            account.phase(RenderPhase::ReadbackDecode, || {
                let rows = decode_padded_rows(&layout, mapped, &account)?;
                let actual = rows.capacity() as u64;
                Ok((ReadbackOutput::try_new(size, rows)?, actual))
            })
        })
        .unwrap()
        .into_parts();
    assert_eq!(fault, None);
    (result, report, cpu)
}

#[test]
fn padded_rows_copy_only_logical_payload_and_transfer_actual_capacity_after_closing() {
    let mapped = mapped_rows();
    let original = mapped.clone();
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::ReadbackRows, 2)
        .unwrap()
        .with_work_limit(RenderWork::ReadbackBytes, 24)
        .unwrap();
    let (result, report, cpu) = decode(&mapped, limits);
    let (output, actual) = result.unwrap();
    assert!(actual >= 24);
    let forward = report.attempt().unwrap();
    assert_eq!(forward.usage().work(RenderWork::ReadbackRows), 2);
    assert_eq!(forward.usage().work(RenderWork::ReadbackBytes), 24);
    assert_eq!(
        forward.usage().work(RenderWork::ReadbackTransferredBytes),
        0
    );
    let closing = report
        .renderer_cpu()
        .unwrap()
        .scope_usage(RenderCpuScope::ReadbackOutput);
    assert_eq!(closing.live(RenderResource::RetainedCpuBytes), actual);
    assert_eq!(closing.peak(RenderResource::RetainedCpuBytes), actual);
    let image = output.transfer().unwrap();
    assert_eq!(image.rgba(), &(0..24).collect::<Vec<u8>>());
    let cloned = image.clone();
    drop(image);
    assert_eq!(cloned.rgba().len(), 24);
    let current = cpu.snapshot().unwrap();
    assert_eq!(
        current.usage().work(RenderWork::ReadbackTransferredBytes),
        actual
    );
    // The closed report retains its pretransfer boundary.
    assert_eq!(closing.live(RenderResource::RetainedCpuBytes), actual);
    assert_eq!(mapped, original);
}

#[test]
fn row_and_payload_limits_retain_only_the_executed_prefix_and_no_partial_output() {
    for (work, limit, rows, bytes, current, amount) in [
        (RenderWork::ReadbackRows, 1, 1, 12, 1, 1),
        (RenderWork::ReadbackBytes, 23, 2, 12, 12, 12),
    ] {
        let (result, report, cpu) = decode(
            &mapped_rows(),
            RenderLimits::new().with_work_limit(work, limit).unwrap(),
        );
        assert!(result.is_err());
        let forward = report.attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(forward.usage().work(RenderWork::ReadbackRows), rows);
        assert_eq!(forward.usage().work(RenderWork::ReadbackBytes), bytes);
        let refusal = forward.refusal().unwrap();
        assert_eq!(refusal.quantity, RenderQuantity::Work(work));
        assert_eq!((refusal.current, refusal.amount), (current, amount));
        assert_eq!(
            report
                .renderer_cpu()
                .unwrap()
                .scope_usage(RenderCpuScope::ReadbackOutput)
                .live(RenderResource::RetainedCpuBytes),
            0
        );
        assert_eq!(
            cpu.snapshot()
                .unwrap()
                .usage()
                .work(RenderWork::ReadbackTransferredBytes),
            0
        );
    }
}

#[test]
fn incomplete_second_row_counts_inspection_but_never_copies_or_publishes_it() {
    let mapped = mapped_rows();
    let (result, report, cpu) = decode(&mapped[..267], RenderLimits::new());
    let Err(error) = result else {
        panic!("one missing logical byte must reject the complete output")
    };
    assert_eq!(error.code(), crate::ErrorCode::ReadbackFailed);
    assert_eq!(
        report.status(),
        RenderAttemptStatus::Failed(crate::ErrorCode::ReadbackFailed)
    );
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::ReadbackRows), 2);
    assert_eq!(usage.work(RenderWork::ReadbackBytes), 12);
    assert_eq!(
        report
            .renderer_cpu()
            .unwrap()
            .scope_usage(RenderCpuScope::ReadbackOutput)
            .live(RenderResource::RetainedCpuBytes),
        0
    );
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::ReadbackTransferredBytes),
        0
    );
}

#[test]
fn output_budget_refuses_before_allocation_and_creates_no_payload_peak() {
    let baseline = RendererCpu::new(RenderAccountingOptions::new())
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    let (result, report, cpu) = decode(
        &mapped_rows(),
        RenderLimits::new().with_retained_bytes(baseline + 23),
    );
    assert!(result.is_err());
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    let usage = report.attempt().unwrap().usage();
    assert_eq!(usage.work(RenderWork::ReadbackRows), 0);
    assert_eq!(usage.work(RenderWork::ReadbackBytes), 0);
    let output = report
        .renderer_cpu()
        .unwrap()
        .scope_usage(RenderCpuScope::ReadbackOutput);
    assert_eq!(output.peak(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(output.live(RenderResource::RetainedCpuBytes), 0);
    assert_eq!(
        cpu.snapshot()
            .unwrap()
            .usage()
            .work(RenderWork::ReadbackTransferredBytes),
        0
    );
}

#[test]
fn actual_output_capacity_has_an_exact_limit_and_a_one_under_refusal() {
    // Vec is the backing producer. Observe its public capacity independently of
    // Render's reports rather than assuming logical payload equals allocation.
    let mut producer = Vec::<u8>::new();
    producer.try_reserve_exact(24).unwrap();
    let capacity = producer.capacity() as u64;
    drop(producer);
    let baseline = RendererCpu::new(RenderAccountingOptions::new())
        .unwrap()
        .snapshot()
        .unwrap()
        .usage()
        .live(RenderResource::RetainedCpuBytes);
    for extra in [capacity, capacity - 1] {
        let (result, report, cpu) = decode(
            &mapped_rows(),
            RenderLimits::new().with_retained_bytes(baseline + extra),
        );
        if extra == capacity {
            let (output, actual) = result.unwrap();
            assert_eq!(actual, capacity);
            assert_eq!(output.transfer().unwrap().rgba().len(), 24);
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .work(RenderWork::ReadbackTransferredBytes),
                capacity
            );
        } else {
            assert!(result.is_err());
            assert_eq!(report.status(), RenderAttemptStatus::Refused);
            let output = report
                .renderer_cpu()
                .unwrap()
                .scope_usage(RenderCpuScope::ReadbackOutput);
            assert_eq!(output.live(RenderResource::RetainedCpuBytes), 0);
            assert_eq!(output.peak(RenderResource::RetainedCpuBytes), 0);
            assert_eq!(
                cpu.snapshot()
                    .unwrap()
                    .usage()
                    .work(RenderWork::ReadbackTransferredBytes),
                0
            );
        }
    }
}
