//! Actual native backing and pending uniform ownership through queue submission.
use super::convolution::{assert_pixel, bounds};
use super::displacement::{active, native_map};
use crate::{
    Color, DisplacementChannel as Channel, DisplacementSource, Layer, Options, Parameters, Rect,
    RenderAccountingError, RenderAccountingOptions, RenderGenerationScope, RenderProfile,
    RenderQuantity, RenderResource, Renderer, ResourceCacheBudget, Scene, Size,
    gpu_transaction::test_support::{
        GraphBeforeSubmissionGateGuardForTest, GraphSubmissionGateGuardForTest,
    },
    pass::{DisplacementUniformProbeForTest, DisplacementUniformStageForTest},
    resource::{DisplacementResourceDescriptorForTest, WorkingFormat},
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

// Both displacement uniforms remain2*256. The full graph also retains Layer
// spatial48, composite128 and Present spatial48 under the same actual owner.
const ALL_PARAMETER_BYTES: u64 = 2 * 256 + 48 + 128 + 48;

fn scene() -> Scene {
    let mut scene = Scene::new();
    scene
        .displacement_layer(
            Layer::new(),
            active(2.0, Channel::A, Channel::R),
            DisplacementSource::constant(Color::BLACK, bounds(4, 1), Rect::new(1.0, 0.0, 2.0, 1.0))
                .unwrap(),
            native_map(1, 1, vec![128, 0, 0, 255], bounds(4, 1)),
        )
        .unwrap();
    scene
}
fn poll_until<F: Future>(
    mut future: Pin<&mut F>,
    device: &wgpu::Device,
    reached: impl Fn() -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending()
        );
        if reached() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "graph did not reach its actual submission boundary"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}
fn finish<F: Future>(mut future: Pin<&mut F>, device: &wgpu::Device) -> F::Output {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Poll::Ready(result) = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            return result;
        }
        assert!(
            Instant::now() < deadline,
            "real displacement transaction did not resolve"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

fn assert_disposed_after_commands(
    observed: &[crate::pass::DisplacementUniformSnapshotForTest],
    expected: &[u64],
) {
    let disposed = observed
        .iter()
        .filter(|s| s.stage == DisplacementUniformStageForTest::UniformDisposing)
        .collect::<Vec<_>>();
    let mut ids = Vec::new();
    for snapshot in disposed {
        let uniform = snapshot.disposing.as_ref().unwrap();
        assert!(uniform.consumed);
        assert_eq!(uniform.bytes, 256);
        assert!(uniform.usage.contains(wgpu::BufferUsages::UNIFORM));
        assert!(snapshot.pass_buffer_bytes >= uniform.bytes);
        assert_eq!(snapshot.command_encoder_objects, 0);
        assert_eq!(snapshot.command_buffer_objects, 0);
        ids.push(uniform.native_identity);
    }
    ids.sort_unstable();
    assert_eq!(ids, expected);
}

#[test]
fn public_displacement_exact_backing_and_uniforms_survive_both_submission_boundaries() {
    let options = Options::default()
        .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
        .with_accounting(
            RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
        );
    let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
    let stats =
        pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
            .unwrap();
    let prior = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let reader = ready.resource_manager_observation_reader_for_test();
    let opening = reader();
    assert_eq!(opening.effects_gpu_bytes, 0);
    let scene = scene();
    // High-precision root4x1 + realizationW3x1 + resultO2x1 + composite4x1
    // uses(4+3+2+4)*8=104 bytes. Raw map4, address tables2*256*8=4096,
    // and axis records(2+1)*16=48 give exactly4252 Effects bytes.
    const BYTES: u64 = 4252;
    renderer
        .set_default_device_effects_gpu_byte_limit_for_test(BYTES - 1)
        .unwrap();
    let error = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
        &mut surface,
        &scene,
        Parameters::default(),
        WorkingFormat::HighPrecision,
    ))
    .unwrap_err();
    assert!(
        matches!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::GenerationRefused {
                quantity: RenderQuantity::Resource(RenderResource::GpuBytes),
                ..
            })
        ),
        "{error:?}"
    );
    let refused = reader();
    assert_eq!(
        refused.payload_creation_attempts,
        opening.payload_creation_attempts
    );
    assert_eq!(refused.effects_allocations, opening.effects_allocations);
    assert_eq!(refused.effects_gpu_bytes, 0);
    assert!(refused.displacement_resources.is_empty());
    assert_eq!(renderer.stats(), stats);
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        prior
    );
    renderer
        .set_default_device_effects_gpu_byte_limit_for_test(BYTES)
        .unwrap();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
    let submitted = GraphSubmissionGateGuardForTest::install(signal);
    let probe = DisplacementUniformProbeForTest::new();
    let uniforms = probe.reader();
    let expected_uniform_ids = {
        let mut future = Box::pin(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            WorkingFormat::HighPrecision,
        ));
        poll_until(future.as_mut(), &device, || {
            before.reached_generation().is_some()
        });
        assert_eq!(before.submissions(), 0);
        let encoded = reader();
        assert_eq!(encoded.effects_gpu_bytes, BYTES);
        let mut planes = encoded
            .effect_textures
            .iter()
            .map(|p| (p.extent.width(), p.extent.height(), p.bytes))
            .collect::<Vec<_>>();
        planes.sort_unstable();
        assert_eq!(planes, [(2, 1, 16), (3, 1, 24), (4, 1, 32), (4, 1, 32)]);
        assert!(
            encoded
                .effect_textures
                .iter()
                .all(|p| p.leased && p.resolved && p.format == wgpu::TextureFormat::Rgba16Float)
        );
        assert_eq!(encoded.displacement_resources.len(), 3);
        let mut kinds = Vec::new();
        for resource in &encoded.displacement_resources {
            assert!(resource.leased && resource.resolved);
            assert_eq!(
                resource.class,
                crate::accounting::GenerationClass::CacheEffects
            );
            match resource.descriptor {
                DisplacementResourceDescriptorForTest::RawMap {
                    extent,
                    format,
                    usage,
                } => {
                    assert_eq!(extent, crate::PhysicalSize::new(1, 1));
                    assert_eq!(format, wgpu::TextureFormat::Rgba8Unorm);
                    assert!(usage.contains(
                        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST
                    ));
                    assert_eq!(resource.bytes, 4);
                    assert!(
                        encoded
                            .effect_textures
                            .iter()
                            .all(|p| p.native_identity != resource.native_identity)
                    );
                    kinds.push(0);
                }
                DisplacementResourceDescriptorForTest::AddressTables { size, usage } => {
                    assert_eq!((size, resource.bytes), (4096, 4096));
                    assert!(
                        usage.contains(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST)
                    );
                    kinds.push(1);
                }
                DisplacementResourceDescriptorForTest::MapRecords { size, usage } => {
                    assert_eq!((size, resource.bytes), (48, 48));
                    assert!(
                        usage.contains(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST)
                    );
                    kinds.push(2);
                }
            }
        }
        kinds.sort_unstable();
        assert_eq!(kinds, [0, 1, 2]);
        let buffer_ids = encoded
            .displacement_resources
            .iter()
            .filter(|entry| {
                matches!(
                    entry.descriptor,
                    DisplacementResourceDescriptorForTest::AddressTables { .. }
                        | DisplacementResourceDescriptorForTest::MapRecords { .. }
                )
            })
            .map(|entry| entry.native_identity)
            .collect::<Vec<_>>();
        assert_eq!(buffer_ids.len(), 2);
        assert_ne!(buffer_ids[0], buffer_ids[1]);
        for (i, entry) in encoded.displacement_resources.iter().enumerate() {
            for other in &encoded.displacement_resources[i + 1..] {
                assert_ne!(entry.resource, other.resource);
            }
        }
        let observed = uniforms();
        let prepared = observed
            .iter()
            .find(|s| s.stage == DisplacementUniformStageForTest::Prepared)
            .unwrap();
        let pending = observed
            .iter()
            .find(|s| s.stage == DisplacementUniformStageForTest::BeforeSubmit)
            .unwrap();
        assert_eq!(prepared.pass_buffer_bytes, ALL_PARAMETER_BYTES);
        assert_eq!(pending.pass_buffer_bytes, ALL_PARAMETER_BYTES);
        assert_eq!(pending.command_encoder_objects, 0);
        assert_eq!(pending.command_buffer_objects, 1);
        assert_eq!(pending.uniforms.len(), 2);
        assert_ne!(pending.uniforms[0].pass, pending.uniforms[1].pass);
        let mut identities = prepared
            .uniforms
            .iter()
            .map(|u| u.native_identity)
            .collect::<Vec<_>>();
        identities.sort_unstable();
        let mut pending_ids = pending
            .uniforms
            .iter()
            .map(|u| u.native_identity)
            .collect::<Vec<_>>();
        pending_ids.sort_unstable();
        assert_eq!(identities, pending_ids);
        assert_ne!(identities[0], identities[1]);
        assert!(
            prepared
                .uniforms
                .iter()
                .all(|u| !u.consumed && u.bytes == 256)
        );
        assert!(pending.uniforms.iter().all(|u| {
            u.consumed
                && u.bytes == 256
                && u.usage
                    .contains(wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
        }));
        before.release();
        poll_until(future.as_mut(), &device, || {
            submitted.submitted_generation().is_some()
        });
        assert_eq!(submitted.submissions(), 1);
        let held = reader();
        assert_eq!(held.effects_gpu_bytes, BYTES);
        assert_eq!(held.displacement_resources, encoded.displacement_resources);
        assert_eq!(
            held.displacement_resources
                .iter()
                .map(|entry| (entry.resource, entry.allocation_generation))
                .collect::<Vec<_>>(),
            encoded
                .displacement_resources
                .iter()
                .map(|entry| (entry.resource, entry.allocation_generation))
                .collect::<Vec<_>>()
        );
        let observed = uniforms();
        let post = observed
            .iter()
            .find(|s| s.stage == DisplacementUniformStageForTest::Submitted)
            .unwrap();
        assert_eq!(post.pass_buffer_bytes, ALL_PARAMETER_BYTES);
        assert_eq!(post.command_buffer_objects, 0);
        let mut post_ids = post
            .uniforms
            .iter()
            .map(|u| u.native_identity)
            .collect::<Vec<_>>();
        post_ids.sort_unstable();
        assert_eq!(post_ids, identities);
        assert!(post.uniforms.iter().all(|u| u.consumed && u.bytes == 256));
        submitted.release();
        finish(future.as_mut(), &device).unwrap();
        identities
    };
    assert_disposed_after_commands(&uniforms(), &expected_uniform_ids);
    drop((probe, before, submitted));
    let final_resources = reader();
    assert_eq!(final_resources.effects_gpu_bytes, 0);
    assert!(final_resources.displacement_resources.is_empty());
    assert_eq!(
        (
            final_resources.leased_count,
            final_resources.active_frame_count
        ),
        (0, 0)
    );
    assert_eq!(
        renderer
            .device_accounting_snapshot(Some(&surface))
            .unwrap()
            .scope_usage(RenderGenerationScope::PassBuffers)
            .unwrap()
            .live(RenderResource::GpuBytes),
        0
    );
    let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    for (x, expected) in [
        (0, [0; 4]),
        (1, [0, 0, 0, 255]),
        (2, [0, 0, 0, 255]),
        (3, [0; 4]),
    ] {
        assert_pixel(&pixels, x, 0, expected, 1);
    }
}

#[test]
fn public_displacement_cancel_disposes_native_commands_before_the_same_retained_uniforms() {
    for after_submission in [false, true] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
                .with_accounting(
                    RenderAccountingOptions::new()
                        .with_profile(RenderProfile::Summary { timing: false }),
                ),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
        let stats =
            pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
                .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let ready = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap();
        let device = ready.device_for_test().clone();
        let reader = ready.resource_manager_observation_reader_for_test();
        let signal = renderer.default_device_signal_for_test().unwrap();
        let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
        let submitted = GraphSubmissionGateGuardForTest::install(signal);
        let probe = DisplacementUniformProbeForTest::new();
        let uniforms = probe.reader();
        let scene = scene();
        let ids = {
            let mut future = Box::pin(renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                WorkingFormat::HighPrecision,
            ));
            poll_until(future.as_mut(), &device, || {
                before.reached_generation().is_some()
            });
            if after_submission {
                before.release();
                poll_until(future.as_mut(), &device, || {
                    submitted.submitted_generation().is_some()
                });
                assert_eq!(submitted.submissions(), 1);
            }
            let observed = uniforms();
            let stage = if after_submission {
                DisplacementUniformStageForTest::Submitted
            } else {
                DisplacementUniformStageForTest::BeforeSubmit
            };
            let held = observed.iter().find(|s| s.stage == stage).unwrap();
            assert_eq!(held.pass_buffer_bytes, ALL_PARAMETER_BYTES);
            assert_eq!(held.command_buffer_objects, u64::from(!after_submission));
            let mut ids = held
                .uniforms
                .iter()
                .map(|u| u.native_identity)
                .collect::<Vec<_>>();
            ids.sort_unstable();
            assert_eq!(ids.len(), 2);
            drop(future);
            ids
        };
        assert_disposed_after_commands(&uniforms(), &ids);
        drop((probe, before, submitted));
        assert_eq!(renderer.stats(), stats);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            pixels
        );
        assert_eq!(reader().effects_gpu_bytes, 0);
        assert_eq!(
            renderer
                .device_accounting_snapshot(Some(&surface))
                .unwrap()
                .scope_usage(RenderGenerationScope::PassBuffers)
                .unwrap()
                .live(RenderResource::GpuBytes),
            0
        );
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            WorkingFormat::HighPrecision,
        ))
        .unwrap();
        assert_pixel(
            &pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            1,
            0,
            [0, 0, 0, 255],
            1,
        );
    }
}

#[test]
fn public_displacement_partial_raw_table_and_record_upload_refusals_release_their_actual_leases() {
    use crate::{
        RenderAttemptStatus, RenderLimits, RenderRefusalReason, RenderWork,
        pass::DisplacementAcquiredReadRoleForTest as Role,
    };
    let scene = scene();
    // The two actual pass uniforms upload256 bytes each. The three concrete
    // read descriptors are rawRGBA8=4, addresses2*256*8=4096, and records3*16=48.
    // Each cap accepts the complete preceding payloads and rejects this one.
    for (prefix, bytes, acquired_roles) in [
        (512, 4, vec![]),
        (516, 4096, vec![Role::RawMap]),
        (4612, 48, vec![Role::RawMap, Role::AddressTables]),
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
                .with_accounting(
                    RenderAccountingOptions::new().with_limits(
                        RenderLimits::new()
                            .with_work_limit(RenderWork::UploadBytes, prefix)
                            .unwrap(),
                    ),
                ),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
        let stats =
            pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
                .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        let publications = surface.headless_publication_count_for_test();
        let reader = renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap()
            .resource_manager_observation_reader_for_test();
        let opening = reader();
        let probe = DisplacementUniformProbeForTest::new();
        let uniforms = probe.reader();
        let error = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            WorkingFormat::HighPrecision,
        ))
        .unwrap_err();
        let report = renderer.last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        assert_eq!(report.cleanup_fault(), None);
        let refusal = report.attempt().unwrap().refusal().unwrap();
        assert_eq!(
            refusal.quantity(),
            RenderQuantity::Work(RenderWork::UploadBytes)
        );
        assert_eq!((refusal.current(), refusal.amount()), (prefix, bytes));
        assert_eq!(
            refusal.reason(),
            RenderRefusalReason::Limit { limit: prefix }
        );
        assert_eq!(
            error.accounting_diagnostic(),
            Some(RenderAccountingError::Refused(refusal))
        );
        let observed = uniforms();
        assert_eq!(observed.len(), 2);
        assert!(
            observed
                .iter()
                .all(|s| s.stage == DisplacementUniformStageForTest::UniformDisposing)
        );
        let failed = observed
            .iter()
            .find(|s| s.disposing.as_ref().is_some_and(|u| u.declared_reads == 3))
            .unwrap();
        let data = failed.disposing.as_ref().unwrap();
        assert!(!data.consumed);
        assert_eq!(data.bytes, 256);
        assert_eq!(failed.pass_buffer_bytes, 512);
        assert_eq!(
            (
                failed.command_encoder_objects,
                failed.command_buffer_objects
            ),
            (0, 0)
        );
        assert_eq!(
            data.acquired_reads
                .iter()
                .map(|r| r.role)
                .collect::<Vec<_>>(),
            acquired_roles
        );
        for (index, read) in data.acquired_reads.iter().enumerate() {
            assert!(!opening.entry_identities_for_test().contains(&read.resource));
            for other in &data.acquired_reads[index + 1..] {
                assert_ne!(
                    (read.resource, read.allocation_generation),
                    (other.resource, other.allocation_generation)
                );
            }
        }
        drop(probe);
        assert_eq!(renderer.stats(), stats);
        assert_eq!(surface.headless_publication_count_for_test(), publications);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            pixels
        );
        let released = reader();
        assert_eq!(released.effects_gpu_bytes, opening.effects_gpu_bytes);
        assert_eq!(released.retained_bytes, opening.retained_bytes);
        assert_eq!((released.leased_count, released.active_frame_count), (0, 0));
        assert!(released.displacement_resources.is_empty());
        assert_eq!(released.accounting_fault_for_test(), None);
        assert!(renderer.default_device_has_no_terminal_signal_for_test());
        drop((surface, renderer, reader));
        // The identical immutable input is valid with the complete policy.
        let mut retry = pollster::block_on(Renderer::new(
            Options::default().with_resource_cache_budget(ResourceCacheBudget::DISABLED),
        ))
        .unwrap();
        let mut target =
            pollster::block_on(retry.create_headless(Size::new(4.0, 1.0), 1.0)).unwrap();
        pollster::block_on(retry.render_with_exact_graph_working_format_for_test(
            &mut target,
            &scene,
            Parameters::default(),
            WorkingFormat::HighPrecision,
        ))
        .unwrap();
        assert_pixel(
            &pollster::block_on(retry.read_headless(&target)).unwrap(),
            1,
            0,
            [0, 0, 0, 255],
            1,
        );
    }
}
