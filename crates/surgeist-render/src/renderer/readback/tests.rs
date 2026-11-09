#![cfg(not(target_arch = "wasm32"))]

use super::*;
use crate::accounting::{
    RenderAccountingError, RenderAccountingOptions, RenderCpuScope, RenderProfile, RenderQuantity,
    RenderRefusalReason, RenderResource, RenderWork,
};

#[test]
fn failed_finalization_disposes_real_decoded_output_without_caller_transfer() {
    pollster::block_on(async {
        for overflow in [false, true] {
            let options = Options::default().with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            );
            let mut renderer = Renderer::new(options).await.unwrap();
            let mut surface = renderer
                .create_headless(Size::new(3.0, 2.0), 1.0)
                .await
                .unwrap();
            let mut scene = Scene::new();
            scene.fill(Rect::new(0.0, 0.0, 3.0, 2.0), Color::BLACK);
            let stats = renderer
                .render(&mut surface, &scene, Parameters::default())
                .await
                .unwrap();
            let frame = renderer.last_attempt().unwrap().snapshot_for_test();
            if overflow {
                // Seed cumulative lifetime history before the readback interval;
                // the next real transfer must overflow atomically.
                renderer
                    .cpu
                    .seed_readback_transfer_for_test(u64::MAX)
                    .unwrap();
            }
            let mut attempt = ReadbackAttempt::begin(
                renderer.readback_reports.clone(),
                &renderer.cpu,
                renderer.options.accounting(),
            )
            .unwrap();
            let account = attempt.account().clone();
            let output = account
                .async_phase(
                    RenderPhase::Readback,
                    renderer.read_headless_inner(&surface, &mut attempt),
                )
                .await
                .unwrap();
            if !overflow {
                // Inject a lifetime health fault after real decoding and final
                // device validation, while output remains guarded.
                renderer
                    .cpu
                    .at(RenderCpuScope::ReadbackOutput)
                    .note_fault(RenderAccountingError::Invariant);
            }
            let error = renderer.finish_readback(attempt, Ok(output)).unwrap_err();
            assert_eq!(error.code(), crate::ErrorCode::Accounting);
            let report = renderer.last_readback_attempt().unwrap();
            assert_eq!(
                report.status(),
                RenderAttemptStatus::Failed(crate::ErrorCode::Accounting)
            );
            assert_eq!(report.attempt().unwrap().status(), report.status());
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::ReadbackBytes),
                24
            );
            let closed = report
                .renderer_cpu()
                .unwrap()
                .scope_usage(RenderCpuScope::ReadbackOutput);
            let held = closed.live(RenderResource::RetainedCpuBytes);
            assert!(
                held >= 24,
                "report must retain its actual pretransfer boundary"
            );
            assert_eq!(closed.work(RenderWork::ReadbackTransferredBytes), 0);
            if overflow {
                assert!(
                    matches!(error.accounting_diagnostic(), Some(RenderAccountingError::CpuRefused {
                    scope: RenderCpuScope::ReadbackOutput,
                    quantity: RenderQuantity::Work(RenderWork::ReadbackTransferredBytes),
                    current: u64::MAX, amount, reason: RenderRefusalReason::Overflow,
                }) if amount == held)
                );
            } else {
                assert_eq!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Invariant)
                );
            }
            drop(report);
            let current = renderer.cpu_accounting_snapshot().unwrap();
            assert_eq!(
                current
                    .scope_usage(RenderCpuScope::ReadbackOutput)
                    .unwrap()
                    .live(RenderResource::RetainedCpuBytes),
                0
            );
            assert_eq!(
                current.usage().work(RenderWork::ReadbackTransferredBytes),
                if overflow { u64::MAX } else { 0 }
            );
            assert_eq!(renderer.stats(), stats);
            assert_eq!(*renderer.last_attempt().unwrap(), frame);
        }
    });
}
