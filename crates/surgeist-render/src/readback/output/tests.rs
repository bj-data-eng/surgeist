use super::*;
use crate::accounting::{
    ReadbackAttempt, ReadbackReportSlot, RenderAccountingError, RenderAccountingOptions,
    RenderCpuScope, RenderResource, RenderWork, RendererCpu,
};

#[test]
fn rejected_image_validation_and_prior_health_fault_dispose_the_admitted_output() {
    for unhealthy in [false, true] {
        let options = RenderAccountingOptions::new();
        let cpu = RendererCpu::new(options).unwrap();
        let attempt = ReadbackAttempt::begin(ReadbackReportSlot::default(), &cpu, options).unwrap();
        let mut rows = attempt.account().retained::<u8>(24).unwrap();
        // An incomplete ImageBuffer is rejected independently of mapped-row
        // validation. The other case reaches the CPU phase's earlier health gate.
        rows.extend_from_slice(&vec![9; if unhealthy { 24 } else { 23 }])
            .unwrap();
        let actual = rows.capacity() as u64;
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
        let status = attempt.status_for(&error);
        let (report, fault) = attempt.finish(status, error.accounting_diagnostic());
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
