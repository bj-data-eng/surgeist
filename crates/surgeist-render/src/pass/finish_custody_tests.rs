//! Submission failures retain real morphology and displacement graph assets.
//! Observations retain no cloned graph GPU handles.
use super::*;
use crate::{
    accounting::RenderAccountingContext,
    accounting::{
        Account, FrameFixture, Generation, GenerationClass, GpuObjectKind, RenderAccountingError,
        RenderAccountingOptions, RenderCpuScope, RenderLimits, RenderQuantity, RenderRefusalReason,
        RenderResource, RenderWork, RendererCpu,
    },
    backend::Backend,
    capability::Capabilities,
    filter_domains::FilterDomains,
    filter_model::FilterColorSpace,
    frame::{FrameContext, FramePlan},
    geometry::PhysicalSize,
    geometry::Rect,
    geometry::Size,
    layer::Layer,
    morphology::MorphologyInput,
    morphology::MorphologyOperator,
    morphology::MorphologySource,
    paint::Color,
    pass::LoweredGraphPlan,
    renderer::Antialiasing,
    renderer::EffectQualityPolicy,
    renderer::ResourceCacheBudget,
    resource::ResourceManager,
    scene::Scene,
    shader::DevicePassCache,
    surface::Format,
};

fn physical_bytes(generation: &Generation) -> [u64; 3] {
    [
        GenerationClass::CacheEffects,
        GenerationClass::CacheVello,
        GenerationClass::PassBuffers,
    ]
    .map(|class| {
        generation
            .class_usage(class)
            .unwrap()
            .live(RenderResource::GpuBytes)
    })
}

fn assert_refusal(error: &crate::Error, work: RenderWork) {
    let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
        panic!("the selected work limit must produce a typed refusal: {error:?}");
    };
    assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
    assert_eq!((refusal.current(), refusal.amount()), (0, 1));
    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
}

fn exercise_failure(failure: u8) {
    let cpu = RendererCpu::new(RenderAccountingOptions::new()).unwrap();
    let mut backend = Backend::new(
        ResourceCacheBudget::DISABLED,
        cpu.clone(),
        RenderAccountingOptions::new(),
    )
    .unwrap();
    let identity = pollster::block_on(backend.select_device(None))
        .unwrap()
        .expect("real GPU adapter");
    let capabilities = backend.device_capabilities(identity).unwrap();
    let ready = backend
        .ready_device_state_borrow_for_test(identity)
        .unwrap();
    let generation = ready.generation_for_test();
    let resources = ResourceManager::with_generation(
        ResourceCacheBudget::DISABLED,
        generation.clone(),
        cpu.at(RenderCpuScope::DeviceBacking),
    );
    let cache = DevicePassCache::new(generation.clone(), cpu.at(RenderCpuScope::DeviceBacking));
    let baseline = physical_bytes(generation);
    let setup = Account::for_test();
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut input = Scene::new();
    input.fill(bounds, Color::BLACK);
    let mut scene = Scene::new();
    if failure < 3 {
        let operation = crate::morphology::prepare_morphology(
            MorphologyInput::new()
                .with_operator(MorphologyOperator::Dilate)
                .with_radius(1.0),
            FilterColorSpace::Srgb,
            &mut RenderAccountingContext::default(),
        )
        .unwrap();
        scene
            .morphology_layer(
                Layer::new(),
                operation,
                MorphologySource::finite(
                    input,
                    FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
                ),
            )
            .unwrap();
    } else {
        let mut context = RenderAccountingContext::default();
        let operation = crate::displacement::prepare_displacement(
            crate::displacement::DisplacementInput::new().with_scale(2.0),
            FilterColorSpace::Srgb,
            &mut context,
        )
        .unwrap();
        let map = crate::displacement::prepare_native_displacement_map(
            Size::new(1.0, 1.0),
            std::sync::Arc::from([255, 0, 0, 255]),
            bounds,
            &mut context,
        )
        .unwrap();
        scene
            .displacement_layer(
                Layer::new(),
                operation,
                crate::displacement::DisplacementSource::finite(
                    input,
                    FilterDomains::try_new(bounds, bounds, bounds).unwrap(),
                ),
                map,
            )
            .unwrap();
    }
    let commands = scene.normalize(Capabilities::CURRENT, &setup).unwrap();
    let context = FrameContext::try_new(
        Size::new(2.0, 2.0),
        1.0,
        Antialiasing::Area,
        Color::TRANSPARENT,
    )
    .unwrap();
    let FramePlan::GpuGraph(graph) = commands.plan_for(context, &setup).unwrap() else {
        panic!("the finite native source requires its actual graph");
    };
    let working = capabilities
        .resolve_effect_working_format(EffectQualityPolicy::AllowReducedPrecision)
        .unwrap();
    let lowered = LoweredGraphPlan::try_lower_validated_graph(
        &graph,
        working,
        Format::Rgba8,
        &capabilities,
        &setup,
    )
    .unwrap();
    let crate::pass::close::PrePreparationGraphClassification::ExactSpatialFilter(spatial) =
        crate::pass::close::PrePreparationGraphClassification::classify(lowered, &setup).unwrap()
    else {
        panic!("the finite native source must close as a spatial graph");
    };
    let probe = crate::pass::DisplacementUniformProbeForTest::new();
    let observations = probe.reader();
    let parameter_probe = crate::pass::ParameterProbeForTest::new(generation);
    let parameters = parameter_probe.reader();
    let mut prepared = PreparedGraph::try_prepare_spatial_filter(
        &spatial,
        working,
        &capabilities,
        (ready.device_for_test(), ready.queue_for_test()),
        &resources,
        (&cache, true),
        &setup,
    )
    .unwrap()
    .with_vello_engine(ready.engine_for_test());
    prepared.observe_displacement_uniforms_for_test();
    prepared.observe_parameter_custody_for_test(crate::pass::ParameterStageForTest::Prepared);
    drop(spatial);
    drop(graph);
    drop(scene);
    // The external output is independent of the graph's Effects/Vello resources.
    let output = ready
        .device_for_test()
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("finish custody external output"),
            size: wgpu::Extent3d {
                width: 2,
                height: 2,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
    let view = output.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = generation
        .opaque(GpuObjectKind::CommandEncoder, Some(&setup), || {
            ready
                .device_for_test()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("finite morphology custody encoder"),
                })
        })
        .unwrap();
    let pending = pollster::block_on(prepared.encode_custom_spine(
        &mut encoder,
        GraphExternalOutputView::try_new(&view, Format::Rgba8, PhysicalSize::new(2, 2)).unwrap(),
        &setup,
    ))
    .unwrap();
    let encoded = physical_bytes(generation);
    assert!(
        encoded[0] > baseline[0],
        "the real encoder references Effects textures"
    );
    assert!(
        encoded[1] > baseline[1],
        "the real finite source retains pending Vello capture assets"
    );
    use crate::pass::{ParameterFamilyForTest as Family, ParameterStageForTest as Stage};
    // For the 2x2/radius1 finite source, each morphology axis builds Table1
    // and Query1; Prepare + two pairs + Resolve is six 256-byte blocks.
    // Displacement uses RealizeSource + Displace instead: two 256-byte blocks.
    // Both graphs also have four spatial blocks (capture canonicalization,
    // source span composite, bounded outer Layer composite, Present) and one
    // outer Layer parameter block. These literals precede raw-owner inspection.
    let native_count = if failure < 3 { 6 } else { 2 };
    let native_family = if failure < 3 {
        Family::Morphology
    } else {
        Family::Displacement
    };
    let expected_parameter_bytes = native_count * 256 + 4 * 48 + 128;
    assert_eq!(encoded[2], baseline[2] + expected_parameter_bytes);
    let parameter_observations = parameters();
    let prepared_parameters = parameter_observations
        .iter()
        .find(|s| s.stage == Stage::Prepared)
        .unwrap()
        .buffers
        .as_ref()
        .unwrap();
    let encoded_parameters = parameter_observations
        .iter()
        .find(|s| s.stage == Stage::Encoded)
        .unwrap()
        .buffers
        .as_ref()
        .unwrap();
    for (family, bytes, count) in [
        (native_family, 256, native_count),
        (Family::Spatial, 48, 4),
        (Family::Composite, 128, 1),
    ] {
        let actual: Vec<_> = prepared_parameters
            .iter()
            .filter(|b| b.family == family)
            .collect();
        assert_eq!(actual.len() as u64, count);
        assert!(actual.iter().all(|b| b.bytes == bytes
            && b.usage == (wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
            && !b.consumed));
    }
    assert_eq!(prepared_parameters.len() as u64, native_count + 5);
    assert_eq!(encoded_parameters.len(), prepared_parameters.len());
    for before in prepared_parameters {
        let after = encoded_parameters
            .iter()
            .find(|b| b.native_identity == before.native_identity)
            .unwrap();
        assert_eq!(
            (after.family, after.pass, after.bytes, after.usage),
            (before.family, before.pass, before.bytes, before.usage)
        );
        assert!(after.consumed);
    }
    let limited_work = if failure == 2 {
        RenderWork::ResourceAllocationRequests
    } else {
        RenderWork::ResourceProbes
    };
    let options = RenderAccountingOptions::new().with_limits(
        RenderLimits::new()
            .with_work_limit(limited_work, 0)
            .unwrap(),
    );
    FrameFixture::new(&cpu, options)
        .run_sync(|attempt| {
            if failure == 0 {
                let draft = GraphSubmissionDraft { encoder, pending, prepared };
                let result = draft.validate(attempt.account());
                assert_refusal(
                    result
                        .as_ref()
                        .expect_err("the finish proof must reach its zero probe allowance"),
                    limited_work,
                );
                // The unchanged zero-probe stimulus rejects the proof while
                // the coupled owner still retains the real unsubmitted encoder.
                assert_eq!(
                    physical_bytes(generation),
                    encoded,
                    "finish rejection refunded graph backing while the caller encoder was alive"
                );
                drop(result);
                drop(draft);
            } else if failure <= 2 {
                let result = prepared.finish_graph_submission(pending, encoder, attempt.account());
                assert_refusal(
                    result
                        .as_ref()
                        .err()
                        .expect("the selected finish boundary must reach its zero allowance"),
                    limited_work,
                );
                drop(result);
            } else {
                // Remove the real owner rather than retaining a cloned GPU
                // handle. Pending finite-source captures remain discriminating.
                if failure == 3 || failure == 5 { drop(prepared.frame_scope.take()); }
                else { drop(prepared.pass_cache_update.take()); }
                assert!(physical_bytes(generation)[1] > baseline[1]);
                let result = if failure <= 4 {
                    prepared.finish_graph_submission(pending, encoder, &setup)
                } else {
                    let command_buffer = encoder.finish(Some(&setup)).unwrap();
                    FinishedGraphSubmissionDraft { command_buffer, pending, prepared }.into_submission()
                };
                let error = result.as_ref().err().expect("a missing real submission owner must be rejected");
                assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
                assert_eq!(error.accounting_diagnostic(), None);
                drop(result);
            }
            if failure != 0 {
                assert_eq!(
                    physical_bytes(generation),
                    baseline,
                    "the complete boundary must dispose commands before releasing graph and capture custody"
                );
            }
            assert_eq!(
                physical_bytes(generation),
                baseline,
                "abort must release the exact real backing after encoder disposal"
            );
            Ok(())
        })
        .unwrap()
        .into_parts()
        .0
        .unwrap();
    let terminal_parameters = parameters();
    let disposals: Vec<_> = terminal_parameters
        .iter()
        .filter(|s| s.stage == Stage::Disposing)
        .collect();
    assert_eq!(disposals.len(), prepared_parameters.len());
    for snapshot in disposals {
        let disposed = snapshot.disposing.as_ref().unwrap();
        assert!(
            prepared_parameters
                .iter()
                .any(|before| before.native_identity == disposed.native_identity)
        );
        assert!(disposed.consumed && snapshot.pass_buffer_bytes > baseline[2]);
        assert_eq!(
            (
                snapshot.command_encoder_objects,
                snapshot.command_buffer_objects
            ),
            (0, 0),
            "commands dispose before every actual parameter buffer"
        );
    }
    if failure >= 3 {
        let observations = observations();
        let prepared_ids: Vec<_> = observations
            .iter()
            .find(|snapshot| {
                snapshot.stage == crate::pass::DisplacementUniformStageForTest::Prepared
            })
            .unwrap()
            .uniforms
            .iter()
            .map(|uniform| uniform.native_identity)
            .collect();
        assert_eq!(prepared_ids.len(), 2);
        let disposals: Vec<_> = observations
            .iter()
            .filter(|snapshot| {
                snapshot.stage == crate::pass::DisplacementUniformStageForTest::UniformDisposing
            })
            .collect();
        assert_eq!(disposals.len(), 2);
        for snapshot in disposals {
            let buffer = snapshot.disposing.as_ref().unwrap();
            assert!(prepared_ids.contains(&buffer.native_identity));
            assert_eq!(buffer.bytes, 256);
            assert!(buffer.consumed && snapshot.pass_buffer_bytes > baseline[2]);
            assert_eq!(
                (
                    snapshot.command_encoder_objects,
                    snapshot.command_buffer_objects
                ),
                (0, 0),
                "native commands must dispose before the actual uniform refunds"
            );
        }
    }
}

#[test]
fn finish_probe_refusal_keeps_real_effect_and_capture_backing_until_encoder_disposal() {
    exercise_failure(0);
}

#[test]
fn encoder_finish_refusal_keeps_real_effect_and_capture_backing_until_submission_abort() {
    exercise_failure(2);
}

#[test]
fn complete_finish_boundary_refusal_disposes_encoder_and_both_resource_owners() {
    exercise_failure(1);
}
#[test]
fn missing_frame_scope_disposes_encoder_before_remaining_capture_custody() {
    exercise_failure(3);
}
#[test]
fn missing_pass_cache_disposes_encoder_before_frame_and_capture_custody() {
    exercise_failure(4);
}
#[test]
fn late_missing_frame_scope_disposes_command_buffer_before_capture_custody() {
    exercise_failure(5);
}
#[test]
fn late_missing_pass_cache_disposes_command_buffer_before_frame_and_capture_custody() {
    exercise_failure(6);
}
