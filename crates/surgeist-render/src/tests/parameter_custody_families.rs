//! Public GPU controls for every graph parameter role. Only raw native
//! descriptors/identities are observed; no buffer handle extends its lifetime.
use super::convolution::{assert_pixel, bounds, image};
use crate::{
    accounting::RenderAccountingOptions,
    accounting::RenderGenerationScope,
    accounting::RenderProfile,
    accounting::RenderResource,
    convolution::ConvolutionInput,
    displacement::DisplacementChannel,
    effect::{Turbulence, TurbulenceKind, TurbulenceRegion},
    filter_domains::FilterDomains,
    filter_model::FilterColorSpace,
    filter_model::FilterEdgeMode,
    geometry::Point,
    geometry::Rect,
    geometry::Size,
    gpu_transaction::test_support::{
        GraphBeforeSubmissionGateGuardForTest, GraphSubmissionGateGuardForTest,
    },
    image::ImageFit,
    layer::Layer,
    layer::ResolvedLayerAlphaMask,
    morphology::MorphologyInput,
    morphology::MorphologyOperator,
    native_shadow::DropShadowInput,
    offset::OffsetInput,
    paint::Color,
    pass::{
        ParameterFamilyForTest as Family, ParameterProbeForTest, ParameterSnapshotForTest,
        ParameterStageForTest as Stage,
    },
    renderer::Options,
    renderer::Renderer,
    renderer::ResourceCacheBudget,
    resource::WorkingFormat,
    scene::Scene,
    shape::Shape,
    stats::RenderRoute,
    style::clip::ClipInput,
    style::filter::BackdropCaptureBounds,
    style::filter::BackdropFilterInput,
    style::filter::FilterAmount,
    style::filter::FilterBlur,
    style::filter::FilterDropShadow,
    style::filter::FilterList,
    style::filter::FilterOp,
    style::filter::UnitFilterAmount,
    surface::Parameters,
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

const WIDTH: u32 = 8;
const HEIGHT: u32 = 4;
const FORMATS: [WorkingFormat; 2] = [
    WorkingFormat::HighPrecision,
    WorkingFormat::ReducedPrecision,
];

struct Fixture {
    name: &'static str,
    scene: Scene,
    expected: Vec<[u8; 4]>,
    required: Vec<Family>,
    color_operations: Option<u64>,
    core_only: bool,
}
impl Fixture {
    fn new(name: &'static str, scene: Scene, required: Vec<Family>) -> Self {
        Self {
            name,
            scene,
            expected: vec![[0; 4]; (WIDTH * HEIGHT) as usize],
            required,
            color_operations: None,
            core_only: false,
        }
    }
    fn pixel(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        self.expected[(y * WIDTH + x) as usize] = rgba;
    }
    fn rectangle(&mut self, width: u32, height: u32, rgba: [u8; 4]) {
        for y in 0..height {
            for x in 0..width {
                self.pixel(x, y, rgba);
            }
        }
    }
}

fn solid(rect: Rect, color: Color) -> Scene {
    let mut scene = Scene::new();
    scene.fill(rect, color);
    scene
}

fn color_fixture() -> Fixture {
    let full = bounds(4, 1);
    let filters = FilterList::try_ops(vec![
        FilterOp::brightness(FilterAmount::try_new(0.5).unwrap()),
        FilterOp::invert(UnitFilterAmount::try_new(1.0).unwrap()),
    ])
    .unwrap();
    let mut scene = Scene::new();
    scene.layer(Layer::new().with_filter_list(filters), |s| {
        s.fill(full, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
    });
    let mut fixture = Fixture::new(
        "ordered color operations",
        scene,
        vec![Family::ColorOperation, Family::Spatial],
    );
    fixture.color_operations = Some(2);
    // Encoded red halves to(.5,0,0), then invert gives(.5,1,1).
    fixture.rectangle(4, 1, [128, 255, 255, 255]);
    fixture
}

fn gaussian_fixture() -> Fixture {
    let full = bounds(4, 2);
    let scene = super::gaussian::finite_scene(
        solid(full, Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap()),
        FilterDomains::try_new(full, full, full).unwrap(),
        FilterEdgeMode::Duplicate,
        super::gaussian::operation(1.0, 1.0, FilterColorSpace::Srgb),
        Layer::new(),
    );
    let mut fixture = Fixture::new("active finite Gaussian", scene, vec![Family::Gaussian]);
    // Every duplicate-extended sample is opaque blue, so a normalized kernel
    // preserves it independently of its radius, coefficients and batching.
    fixture.rectangle(4, 2, [0, 0, 255, 255]);
    fixture
}

fn morphology_fixture() -> Fixture {
    let full = bounds(4, 2);
    let scene = super::morphology::finite_scene(
        solid(full, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap()),
        FilterDomains::try_new(full, full, full).unwrap(),
        super::morphology::operation(
            MorphologyInput::new()
                .with_operator(MorphologyOperator::Dilate)
                .with_radii(1.0, 1.0),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let mut fixture = Fixture::new("active finite morphology", scene, vec![Family::Morphology]);
    // Each rectangular maximum includes its own opaque red source cell;
    // all other samples are red or transparent, including finite exterior.
    fixture.rectangle(4, 2, [255, 0, 0, 255]);
    fixture
}

fn native_shadow_fixture() -> Fixture {
    let full = bounds(4, 1);
    let source = super::native_shadow::source_scene(
        image(
            4,
            1,
            vec![255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ),
        full,
    );
    let scene = super::native_shadow::finite_scene(
        source,
        FilterDomains::try_new(full, full, full).unwrap(),
        super::native_shadow::operation(
            DropShadowInput::new()
                .with_standard_deviation(0.0)
                .with_dx(1.0)
                .with_dy(0.0)
                .with_flood_color(Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap()),
            FilterColorSpace::Srgb,
        ),
        Layer::new(),
    );
    let mut fixture = Fixture::new(
        "native shifted zero-sigma shadow",
        scene,
        vec![Family::NativeShadow],
    );
    fixture.pixel(0, 0, [255, 0, 0, 255]);
    fixture.pixel(1, 0, [0, 255, 0, 255]);
    fixture
}

fn offset_fixture() -> Fixture {
    let source = bounds(3, 1);
    let full = bounds(4, 1);
    let scene = super::offset::image_scene(
        image(3, 1, vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]),
        source,
        super::offset::operation(OffsetInput::new().with_dx(1.0), source, full, full),
    );
    let mut fixture = Fixture::new("native integer offset", scene, vec![Family::Offset]);
    for (x, p) in [
        (1, [255, 0, 0, 255]),
        (2, [0, 255, 0, 255]),
        (3, [0, 0, 255, 255]),
    ] {
        fixture.pixel(x, 0, p);
    }
    fixture
}

fn convolution_fixture() -> Fixture {
    let full = bounds(4, 1);
    let operation = super::convolution::operation(
        ConvolutionInput::new(&[1.0]).with_order(1.0, None),
        full,
        FilterColorSpace::Srgb,
    );
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_convolution(operation)
            .try_opacity(0.5)
            .unwrap(),
        |s| {
            s.image(
                image(
                    4,
                    1,
                    vec![
                        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
                    ],
                ),
                full,
                ImageFit::Stretch,
            );
        },
    );
    let mut fixture = Fixture::new(
        "convolution and bounded composition",
        scene,
        vec![Family::Convolution, Family::Spatial, Family::Composite],
    );
    for (x, p) in [
        [255, 0, 0, 128],
        [0, 255, 0, 128],
        [0, 0, 255, 128],
        [255, 255, 0, 128],
    ]
    .into_iter()
    .enumerate()
    {
        fixture.pixel(x as u32, 0, p);
    }
    fixture
}

fn css_shadow_fixture() -> Fixture {
    let shadow = FilterDropShadow::try_new(
        Point::new(1.0, 0.0),
        FilterBlur::try_new(0.0).unwrap(),
        Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.layer(
        Layer::new()
            .with_filter_list(FilterList::try_ops(vec![FilterOp::drop_shadow(shadow)]).unwrap()),
        |s| {
            s.fill(
                Rect::new(1.0, 1.0, 1.0, 1.0),
                Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
            );
        },
    );
    let mut fixture = Fixture::new(
        "CSS shifted zero-blur shadow",
        scene,
        vec![Family::CssShadow, Family::Spatial],
    );
    fixture.pixel(1, 1, [255, 0, 0, 255]);
    fixture.pixel(2, 1, [0, 255, 0, 255]);
    fixture
}

fn mirror_backdrop_fixture() -> Fixture {
    let full = bounds(4, 3);
    let backdrop = BackdropFilterInput::try_new(
        FilterList::try_ops(vec![FilterOp::blur(FilterBlur::try_new(1.0).unwrap())]).unwrap(),
        BackdropCaptureBounds::try_new(full).unwrap(),
        Some(ClipInput::try_shape(Shape::rect(full)).unwrap()),
    )
    .unwrap();
    let mut scene = solid(full, Color::try_rgba(1.0, 1.0, 0.0, 1.0).unwrap());
    scene.layer(Layer::new().try_backdrop_filter(backdrop).unwrap(), |_| {});
    let mut fixture = Fixture::new(
        "semantic mirror backdrop blur",
        scene,
        vec![Family::BlurEdge, Family::Spatial],
    );
    // Mirroring a constant finite capture keeps all blur samples yellow.
    // Capture bounds constrain reads; the explicit post-filter clip separately
    // limits the inflated blur result to this4×3 contribution.
    fixture.rectangle(4, 3, [255, 255, 0, 255]);
    fixture
}

fn core_fixture() -> Fixture {
    let full = bounds(4, 1);
    let mask =
        ResolvedLayerAlphaMask::try_new(image(4, 1, [255, 255, 255, 128].repeat(4)), full).unwrap();
    let mut scene = Scene::new();
    scene.layer(Layer::new().with_resolved_alpha_mask(mask), |s| {
        s.fill(full, Color::try_rgba(1.0, 0.0, 0.0, 1.0).unwrap());
    });
    let mut fixture = Fixture::new(
        "effect-free core mask and Present",
        scene,
        vec![Family::Spatial, Family::Composite],
    );
    fixture.core_only = true;
    fixture.rectangle(4, 1, [255, 0, 0, 128]);
    fixture
}

fn mixed_fixture() -> Fixture {
    let full = bounds(4, 3);
    let output = bounds(2, 1);
    let row = [
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
    ];
    let source = crate::displacement::DisplacementSource::finite(
        super::displacement::source_scene(image(4, 3, row.repeat(3)), full),
        FilterDomains::try_new(full, full, output).unwrap(),
    );
    let mut scene = Scene::new();
    scene.layer(
        Layer::new().with_filter_list(
            FilterList::try_ops(vec![FilterOp::brightness(
                FilterAmount::try_new(0.5).unwrap(),
            )])
            .unwrap(),
        ),
        |s| {
            s.displacement_layer(
                Layer::new(),
                super::displacement::active(4.0, DisplacementChannel::R, DisplacementChannel::G),
                source,
                super::displacement::native_map(1, 1, vec![255, 128, 0, 64], output),
            )
            .unwrap();
        },
    );
    let mut fixture = Fixture::new(
        "mixed native displacement and CSS color",
        scene,
        vec![
            Family::Displacement,
            Family::ColorOperation,
            Family::Spatial,
        ],
    );
    fixture.color_operations = Some(1);
    // Native +2 byte addressing selects blue/yellow; encoded brightness halves
    // their straight RGB while preserving their opaque source alpha.
    fixture.pixel(0, 0, [0, 0, 128, 255]);
    fixture.pixel(1, 0, [128, 128, 0, 255]);
    fixture
}

fn lighting_fixture() -> Fixture {
    use crate::lighting::DiffuseLightingInput;
    let full = bounds(2, 1);
    let scene = super::lighting::diffuse(
        super::native_shadow::source_scene(image(2, 1, vec![255, 0, 0, 0, 0, 0, 255, 255]), full),
        FilterDomains::try_new(full, full, full).unwrap(),
        DiffuseLightingInput::new(),
        super::lighting::VERTICAL,
        FilterColorSpace::Srgb,
        Layer::new(),
    );
    let mut fixture = Fixture::new("diffuse lighting", scene, vec![Family::Lighting]);
    // Both ends of this two-cell bump use slope 2, giving z = 1/sqrt(5).
    fixture.rectangle(2, 1, [114, 114, 114, 255]);
    fixture
}

fn turbulence_fixture() -> Fixture {
    let full = bounds(2, 1);
    let mut scene = Scene::new();
    scene
        .turbulence_layer(
            Layer::new(),
            Turbulence::try_new([0.125, 0.25], 1.0)
                .unwrap()
                .with_octaves(0)
                .with_kind(TurbulenceKind::FractalNoise)
                .with_color_space(FilterColorSpace::Srgb),
            TurbulenceRegion::try_new(full, full).unwrap(),
        )
        .unwrap();
    let mut fixture = Fixture::new("source-free turbulence", scene, vec![Family::Turbulence]);
    // The empty noise sum maps to straight .5 for each fractal-noise channel.
    fixture.rectangle(2, 1, [128, 128, 128, 128]);
    fixture
}

fn wait<F: Future>(mut future: Pin<&mut F>, device: &wgpu::Device, reached: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            future
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending(),
            "real graph must remain held at selected gate"
        );
        if reached() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "parameter-family graph did not reach its owned gate"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}
fn finish<F: Future>(mut future: Pin<&mut F>, device: &wgpu::Device) -> F::Output {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Poll::Ready(value) = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "parameter-family graph did not complete before deadline"
        );
        device.poll(wgpu::PollType::Poll).unwrap();
        std::thread::yield_now();
    }
}

fn assert_descriptors(fixture: &Fixture, snapshots: &[ParameterSnapshotForTest]) {
    let prepared = snapshots
        .iter()
        .find(|s| s.stage == Stage::Prepared)
        .expect("actual PreparedGraph preparation must be observed");
    let mut originals = prepared
        .buffers
        .clone()
        .expect("prepared owner must expose raw descriptors");
    assert!(
        !originals.is_empty(),
        "{} must acquire native parameter buffers",
        fixture.name
    );
    assert!(originals.iter().all(|b| !b.consumed));
    for required in &fixture.required {
        assert!(
            originals.iter().any(|b| b.family == *required),
            "{} missing actual {required:?}: {snapshots:?}",
            fixture.name
        );
    }
    for buffer in &originals {
        let (size, usage) = match buffer.family {
            Family::ColorOperation => (
                16 + 32
                    * fixture
                        .color_operations
                        .expect("fixture must declare actual ordered color operation count"),
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            ),
            Family::Gaussian
            | Family::Lighting
            | Family::Morphology
            | Family::NativeShadow
            | Family::Offset
            | Family::Displacement
            | Family::Convolution => (
                256,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            Family::Spatial => (
                48,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            Family::Turbulence => (
                9296,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            Family::Composite => (
                128,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            Family::CssShadow => (
                32,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            Family::BlurEdge => (
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
        };
        assert_eq!(
            buffer.bytes, size,
            "{} actual {:?} wire size",
            fixture.name, buffer.family
        );
        assert_eq!(
            buffer.usage, usage,
            "{} actual {:?} exact usage",
            fixture.name, buffer.family
        );
        if fixture.core_only {
            assert!(matches!(buffer.family, Family::Spatial | Family::Composite));
        }
    }
    if fixture.required.contains(&Family::Lighting) {
        assert_eq!(
            originals
                .iter()
                .filter(|b| b.family == Family::Lighting)
                .count(),
            3
        );
    }
    if fixture.required.contains(&Family::Turbulence) {
        assert_eq!(
            originals
                .iter()
                .filter(|b| b.family == Family::Turbulence)
                .count(),
            1
        );
    }
    originals.sort_unstable_by_key(|b| b.native_identity);
    assert!(
        originals
            .windows(2)
            .all(|pair| pair[0].native_identity != pair[1].native_identity),
        "each actual parameter occurrence owns a distinct native buffer"
    );
    let bytes = originals.iter().map(|b| b.bytes).sum::<u64>();
    assert!(prepared.pass_buffer_bytes >= bytes);
    for stage in [
        Stage::Encoded,
        Stage::BeforeSubmit,
        Stage::Submitted,
        Stage::AccountingReady,
    ] {
        let at_stage = snapshots
            .iter()
            .filter(|s| s.stage == stage)
            .collect::<Vec<_>>();
        assert!(
            !at_stage.is_empty(),
            "{} missing {stage:?}: {snapshots:?}",
            fixture.name
        );
        for snapshot in at_stage {
            let mut retained = snapshot
                .buffers
                .clone()
                .expect("actual pending/ready owner descriptors must remain observable");
            retained.sort_unstable_by_key(|b| b.native_identity);
            assert_eq!(
                retained.len(),
                originals.len(),
                "{} buffers disappeared at {stage:?}",
                fixture.name
            );
            for (before, after) in originals.iter().zip(&retained) {
                assert_eq!(
                    (
                        after.family,
                        after.pass,
                        after.native_identity,
                        after.bytes,
                        after.usage
                    ),
                    (
                        before.family,
                        before.pass,
                        before.native_identity,
                        before.bytes,
                        before.usage
                    ),
                    "{} actual descriptor changed at {stage:?}",
                    fixture.name
                );
                assert!(
                    after.consumed,
                    "{} logical completion must consume {:?} by {stage:?}",
                    fixture.name, after.family
                );
            }
            assert!(
                snapshot.pass_buffer_bytes >= bytes,
                "{} parameter backing refunded at {stage:?}: {snapshots:?}",
                fixture.name
            );
        }
    }
}

fn exercise(fixture: Fixture, format: WorkingFormat) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default()
            .with_resource_cache_budget(ResourceCacheBudget::DISABLED)
            .with_accounting(
                RenderAccountingOptions::new()
                    .with_profile(RenderProfile::Summary { timing: false }),
            ),
    ))
    .unwrap();
    let mut surface = pollster::block_on(
        renderer.create_headless(Size::new(f64::from(WIDTH), f64::from(HEIGHT)), 1.0),
    )
    .unwrap();
    let baseline = solid(bounds(WIDTH, HEIGHT), Color::BLACK);
    pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default())).unwrap();
    let publications = surface.headless_publication_count_for_test();
    let ready = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap();
    let device = ready.device_for_test().clone();
    let probe = ParameterProbeForTest::new(ready.generation_for_test());
    let reader = probe.reader();
    let signal = renderer.default_device_signal_for_test().unwrap();
    let before = GraphBeforeSubmissionGateGuardForTest::install(Arc::clone(&signal));
    let submitted = GraphSubmissionGateGuardForTest::install(signal);
    {
        let mut future = Box::pin(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &fixture.scene,
            Parameters::default(),
            format,
        ));
        wait(future.as_mut(), &device, || {
            before.reached_generation().is_some()
        });
        assert_eq!(before.submissions(), 0);
        before.release();
        wait(future.as_mut(), &device, || {
            submitted.submitted_generation().is_some()
        });
        assert_eq!(submitted.submissions(), 1);
        submitted.release();
        assert_eq!(
            finish(future.as_mut(), &device).unwrap().route,
            Some(RenderRoute::GpuGraph)
        );
    }
    drop((before, submitted, probe));
    let snapshots = reader();
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    // Controls use primary colors and binary-exact .5 alpha/RGB. One byte
    // allows the final RGBA8 rounding in either supported working format.
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            assert_pixel(&output, x, y, fixture.expected[(y * WIDTH + x) as usize], 1);
        }
    }
    assert_eq!(
        surface.headless_publication_count_for_test(),
        publications + 1
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
    let resources = renderer
        .default_ready_device_state_borrow_for_test()
        .unwrap()
        .internal_resource_manager_observation_for_test();
    assert_eq!(
        (
            resources.active_frame_count,
            resources.leased_count,
            resources.retained_bytes
        ),
        (0, 0, 0)
    );
    assert_eq!(resources.accounting_fault_for_test(), None);
    // Pixel and identical-input retry controls complete before the lifetime
    // assertions, so a custody failure cannot masquerade as missing execution.
    pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
        &mut surface,
        &fixture.scene,
        Parameters::default(),
        format,
    ))
    .unwrap();
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        output
    );
    assert_descriptors(&fixture, &snapshots);
}

#[test]
fn public_native_prepared_families_retain_exact_parameter_descriptors_until_terminal_commit() {
    for format in FORMATS {
        for fixture in [
            color_fixture(),
            gaussian_fixture(),
            morphology_fixture(),
            native_shadow_fixture(),
            lighting_fixture(),
            turbulence_fixture(),
            offset_fixture(),
        ] {
            exercise(fixture, format);
        }
    }
}

#[test]
fn public_convolution_css_shadow_and_mirror_blur_retain_former_encode_local_parameters() {
    for format in FORMATS {
        for fixture in [
            convolution_fixture(),
            css_shadow_fixture(),
            mirror_backdrop_fixture(),
        ] {
            exercise(fixture, format);
        }
    }
}

#[test]
fn public_effect_free_core_and_mixed_displacement_graphs_share_parameter_custody() {
    for format in FORMATS {
        for fixture in [core_fixture(), mixed_fixture()] {
            exercise(fixture, format);
        }
    }
}

#[test]
fn public_direct_empty_scene_avoids_graph_parameter_preparation() {
    let mut renderer = pollster::block_on(Renderer::new(Options::default().with_accounting(
        RenderAccountingOptions::new().with_profile(RenderProfile::Summary { timing: false }),
    )))
    .unwrap();
    let surface_size = Size::new(2.0, 1.0);
    let mut surface = pollster::block_on(renderer.create_headless(surface_size, 1.0)).unwrap();
    pollster::block_on(renderer.render(
        &mut surface,
        &solid(bounds(2, 1), Color::BLACK),
        Parameters::default(),
    ))
    .unwrap();
    let probe = ParameterProbeForTest::new(
        renderer
            .default_ready_device_state_borrow_for_test()
            .unwrap()
            .generation_for_test(),
    );
    let reader = probe.reader();
    pollster::block_on(renderer.render(&mut surface, &Scene::new(), Parameters::default()))
        .unwrap();
    let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    assert_eq!(output.rgba(), &[0; 8]);
    drop(probe);
    assert!(
        reader().is_empty(),
        "a direct empty scene never prepares a graph"
    );
    // This is a real no-parameter public control, not a fabricated empty
    // PreparedGraph. A public graph still needs its Present spatial block.
    assert_eq!(
        renderer
            .device_accounting_snapshot(Some(&surface))
            .unwrap()
            .scope_usage(RenderGenerationScope::PassBuffers)
            .unwrap()
            .live(RenderResource::GpuBytes),
        0
    );
}
