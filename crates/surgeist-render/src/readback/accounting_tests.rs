#![cfg(not(target_arch = "wasm32"))]

use super::*;
use crate::accounting::{
    ReadbackFixture, RenderAccountingOptions, RenderAttemptStatus, RenderLimits, RenderQuantity,
    RendererCpu,
};

#[test]
fn shared_constructor_and_submission_limits_refuse_before_the_actual_native_call() {
    pollster::block_on(async {
        for (work, limit, succeeds, requests, copies, submissions) in [
            (RenderWork::ResourceAllocationRequests, 0, false, 0, 0, 0),
            (RenderWork::ResourceAllocationRequests, 1, false, 1, 0, 0),
            (RenderWork::ResourceAllocationRequests, 2, false, 2, 1, 0),
            (RenderWork::ResourceAllocationRequests, 3, true, 3, 1, 1),
            (RenderWork::Submissions, 0, false, 3, 1, 0),
            (RenderWork::Submissions, 1, true, 3, 1, 1),
        ] {
            let options = RenderAccountingOptions::new()
                .with_limits(RenderLimits::new().with_work_limit(work, limit).unwrap());
            let cpu = RendererCpu::new(options).unwrap();
            let mut backend =
                Backend::new(crate::ResourceCacheBudget::DISABLED, cpu.clone(), options).unwrap();
            let identity = backend
                .select_device(None)
                .await
                .unwrap()
                .expect("real readback adapter");
            let generation = backend.generation(identity).unwrap().clone();
            let size = PhysicalSize::new(3, 2);
            let (texture, view) = {
                let (device, _) = backend
                    .device_queue(identity, RuntimeOperation::SurfaceReadback)
                    .unwrap();
                // This preexisting input belongs to setup, outside both fresh
                // attempts. Their limits govern only the actual readback body.
                crate::backend::create_headless_texture(
                    device,
                    &generation,
                    size,
                    crate::Format::Rgba8,
                    None,
                )
                .unwrap()
            };
            drop(view);
            for _ in 0..2 {
                let (result, _, report, fault) = ReadbackFixture::new(&cpu, options)
                    .run(async |mut attempt| {
                        attempt.attach_generation(&generation).unwrap();
                        let account = attempt.account().clone();
                        account
                            .async_phase(
                                RenderPhase::Readback,
                                read_texture_rgba(
                                    &mut backend,
                                    identity,
                                    &texture,
                                    size,
                                    RuntimeOperation::SurfaceReadback,
                                    &account,
                                ),
                            )
                            .await
                    })
                    .await
                    .unwrap()
                    .into_parts();
                assert_eq!(result.is_ok(), succeeds);
                assert_eq!(fault, None);
                let forward = report.attempt().unwrap();
                let usage = forward.usage();
                for (unit, count) in [
                    (RenderWork::ResourceAllocationRequests, requests),
                    (RenderWork::ReadbackCopies, copies),
                    (RenderWork::Submissions, submissions),
                    (RenderWork::ReadbackMapRequests, u64::from(succeeds)),
                    (RenderWork::ReadbackRows, if succeeds { 2 } else { 0 }),
                    (RenderWork::ReadbackBytes, if succeeds { 24 } else { 0 }),
                ] {
                    assert_eq!(usage.work(unit), count, "{work:?} cap {limit}: {unit:?}");
                }
                if succeeds {
                    assert_eq!(result.unwrap().transfer().unwrap().rgba().len(), 24);
                } else {
                    assert_eq!(report.status(), RenderAttemptStatus::Refused);
                    assert_eq!(
                        forward.refusal().unwrap().quantity(),
                        RenderQuantity::Work(work)
                    );
                }
            }
        }
    });
}
