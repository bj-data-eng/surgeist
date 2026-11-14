//! Submission failures retain the existing finite morphology encoder's real assets.
//! This fixture uses no displacement API and retains no cloned graph GPU handles.
use super::*;
use crate::{
    Antialiasing, Capabilities, Color, EffectQualityPolicy, FilterColorSpace, FilterDomains,
    Format, Layer, MorphologyInput, MorphologyOperator, MorphologySource, PhysicalSize, Rect,
    RenderAccountingContext, ResourceCacheBudget, Scene, Size,
    accounting::{
        Account, FrameFixture, Generation, GenerationClass, GpuObjectKind, RenderAccountingError,
        RenderAccountingOptions, RenderCpuScope, RenderLimits, RenderQuantity, RenderRefusalReason,
        RenderResource, RenderWork, RendererCpu,
    },
    backend::Backend,
    frame::{FrameContext, FramePlan},
    resource::ResourceManager,
    shader::DevicePassCache,
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

fn exercise_failure(finish_proof: bool) {
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
    let operation = crate::prepare_morphology(
        MorphologyInput::new()
            .with_operator(MorphologyOperator::Dilate)
            .with_radius(1.0),
        FilterColorSpace::Srgb,
        &mut RenderAccountingContext::default(),
    )
    .unwrap();
    let mut input = Scene::new();
    input.fill(bounds, Color::BLACK);
    let mut scene = Scene::new();
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
    let commands = scene.normalize(Capabilities::CURRENT, &setup).unwrap();
    let context = FrameContext::try_new(
        Size::new(2.0, 2.0),
        1.0,
        Antialiasing::Area,
        Color::TRANSPARENT,
    )
    .unwrap();
    let FramePlan::GpuGraph(graph) = commands.plan_for(context, &setup).unwrap() else {
        panic!("finite morphology requires its actual graph");
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
    let close::PrePreparationGraphClassification::ExactSpatialFilter(spatial) =
        close::PrePreparationGraphClassification::classify(lowered, &setup).unwrap()
    else {
        panic!("finite morphology must close as a spatial graph");
    };
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
    // Existing morphology uniforms retire at logical completion; this fixture
    // covers the frame and pending capture custody beyond that point.
    assert_eq!(encoded[2], baseline[2]);
    let limited_work = if finish_proof {
        RenderWork::ResourceProbes
    } else {
        RenderWork::ResourceAllocationRequests
    };
    let options = RenderAccountingOptions::new().with_limits(
        RenderLimits::new()
            .with_work_limit(limited_work, 0)
            .unwrap(),
    );
    FrameFixture::new(&cpu, options)
        .run_sync(|attempt| {
            if finish_proof {
                let result = prepared.finish_graph_submission(pending, attempt.account());
                assert_refusal(
                    result
                        .as_ref()
                        .err()
                        .expect("the finish proof must reach its zero probe allowance"),
                    limited_work,
                );
                // The error must retain all native references until the caller's
                // unsubmitted encoder is disposed, including pending captures.
                assert_eq!(
                    physical_bytes(generation),
                    encoded,
                    "finish rejection refunded graph backing while the caller encoder was alive"
                );
                drop(encoder);
                drop(result);
            } else {
                let submission = prepared.finish_graph_submission(pending, &setup).unwrap();
                let result = encoder.finish(Some(attempt.account()));
                assert_refusal(
                    result
                        .as_ref()
                        .err()
                        .expect("command-buffer creation must reach its zero request allowance"),
                    limited_work,
                );
                assert_eq!(
                    physical_bytes(generation),
                    encoded,
                    "encoder finish failure lost pending graph or capture custody"
                );
                drop(result);
                drop(submission);
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
}

#[test]
fn finish_probe_refusal_keeps_real_effect_and_capture_backing_until_encoder_disposal() {
    exercise_failure(true);
}

#[test]
fn encoder_finish_refusal_keeps_real_effect_and_capture_backing_until_submission_abort() {
    exercise_failure(false);
}
