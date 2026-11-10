use super::*;

#[test]
fn cleanup_tail_rejects_foreign_completion_without_replacing_an_originating_fault() {
    let foreign = Account::for_test();
    for prior in [
        None,
        Some(RenderAccountingError::LedgerDefect(
            surgeist_ledger::Defect::ReportScope,
        )),
    ] {
        let account = Account::for_test();
        *account.fault.lock().unwrap() = prior;
        let mut tail = account.clone().readback_cleanup_tail();
        tail.begin();
        account
            .ledger
            .end_phase(tail.observation.take().unwrap())
            .unwrap();
        // Private association injection at the actual Account owner. There is
        // no production constructor that associates these foreign markers.
        tail.observation = Some(
            foreign
                .ledger
                .begin_phase(foreign.scopes[RenderPhase::ReadbackCleanup.index()])
                .unwrap(),
        );
        drop(tail);
        assert_eq!(
            account.latched_fault(),
            prior.or(Some(RenderAccountingError::Invariant))
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn rejected_cleanup_entry_still_drops_actual_staging_and_preserves_backend_or_refusal_cause() {
    use crate::{RuntimeOperation, backend::Backend, gpu_transaction::GpuOperationStage};
    pollster::block_on(async {
        for invalid_copy in [false, true] {
            let mut options = RenderAccountingOptions::new()
                .with_profile(RenderProfile::Summary { timing: false });
            if !invalid_copy {
                options = options.with_limits(
                    RenderLimits::new()
                        .with_work_limit(RenderWork::ReadbackRows, 0)
                        .unwrap(),
                );
            }
            let cpu = RendererCpu::new(options).unwrap();
            let mut backend =
                Backend::new(crate::ResourceCacheBudget::DISABLED, cpu.clone(), options).unwrap();
            let identity = backend.select_device(None).await.unwrap().unwrap();
            let generation = backend.generation(identity).unwrap().clone();
            let texture = {
                let (device, _) = backend
                    .gpu_operation_device_queue(
                        identity,
                        RuntimeOperation::SurfaceReadback,
                        GpuOperationStage::Readback,
                    )
                    .unwrap();
                generation
                    .allocation(GenerationClass::SurfaceTargets, 8, None, || {
                        device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("Surgeist cleanup-entry defensive fixture"),
                            size: wgpu::Extent3d {
                                width: 1,
                                height: 2,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            // Missing COPY_SRC produces a real scoped validation
                            // error before mapping in the backend-cause case.
                            usage: if invalid_copy {
                                wgpu::TextureUsages::COPY_DST
                            } else {
                                wgpu::TextureUsages::COPY_SRC
                            },
                            view_formats: &[],
                        })
                    })
                    .unwrap()
            };
            let foreign = Account::for_test();
            let (result, started, report, fault) = ReadbackFixture::new(&cpu, options)
                .run(async |mut attempt| {
                    attempt.attach_generation(&generation).unwrap();
                    let mut account = attempt.account().clone();
                    let mut scopes = *account.scopes;
                    scopes[RenderPhase::ReadbackCleanup.index()] =
                        foreign.scopes[RenderPhase::ReadbackCleanup.index()];
                    account.scopes = Arc::new(scopes);
                    crate::readback::read_texture_rgba(
                        &mut backend,
                        identity,
                        &texture,
                        crate::PhysicalSize::new(1, 2),
                        RuntimeOperation::SurfaceReadback,
                        &account,
                    )
                    .await
                })
                .await
                .unwrap()
                .into_parts();
            assert!(started);
            let error = match result {
                Err(error) => error,
                Ok(_) => panic!("the real readback must fail"),
            };
            if invalid_copy {
                assert_eq!(error.code(), crate::ErrorCode::ReadbackFailed);
                assert!(std::error::Error::source(&error).is_some());
                assert_eq!(
                    report.status(),
                    RenderAttemptStatus::Failed(crate::ErrorCode::ReadbackFailed)
                );
                assert_eq!(
                    report.accounting_diagnostic(),
                    Some(RenderAccountingError::Invariant)
                );
            } else {
                assert!(matches!(
                    error.accounting_diagnostic(),
                    Some(RenderAccountingError::Refused(_))
                ));
                assert_eq!(report.status(), RenderAttemptStatus::Refused);
                assert_eq!(
                    report.accounting_diagnostic(),
                    error.accounting_diagnostic()
                );
                assert_eq!(
                    report.attempt().unwrap().refusal().unwrap().quantity(),
                    RenderQuantity::Work(RenderWork::ReadbackRows)
                );
            }
            assert_eq!(fault, Some(RenderAccountingError::Invariant));
            assert_eq!(report.cleanup_fault(), fault);
            assert_eq!(
                generation
                    .class_usage(GenerationClass::ReadbackStaging)
                    .unwrap()
                    .live(RenderResource::GpuBytes),
                0,
                "rejected observation must not skip real cleanup"
            );
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::ReadbackCopies), 1);
            assert_eq!(usage.work(RenderWork::Submissions), 1);
            assert_eq!(
                usage.work(RenderWork::ReadbackMapRequests),
                u64::from(!invalid_copy)
            );
            assert_eq!(usage.work(RenderWork::ReadbackRows), 0);
            assert_eq!(usage.work(RenderWork::ReadbackBytes), 0);
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .phase_count(RenderPhase::ReadbackCleanup),
                0
            );
            drop(texture);
            generation.ensure_healthy().unwrap();
        }
    });
}
