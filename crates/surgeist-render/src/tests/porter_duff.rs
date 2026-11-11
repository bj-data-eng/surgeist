//! Independent §6/§9 Porter-Duff oracles and the adopted finite-image policy.
//! RGB expectations are premultiplied; headless readback is straight RGBA8.
//! No expectation calls the production or test-only reference compositor.

use super::{
    separable::{assert_near, masked_layer, render_pixels, straight_bytes, straight_color, tile},
    support::{graph_supported_working_formats_for_test, pixel_rgba},
};
use crate::{
    BlendMode, Color, EffectQualityPolicy, ErrorCode, Image, Layer, LayerComposition, Options,
    Path, Point, PorterDuffOperator as Op, Rect, Renderer, ResolvedLayerAlphaMask, Scene, Shape,
    Size, Surface, Transform,
};

const SOURCE: [f32; 4] = [3.0 / 16.0, 1.0 / 16.0, 1.0 / 8.0, 1.0 / 4.0];
const PARENT: [f32; 4] = [3.0 / 16.0, 9.0 / 16.0, 3.0 / 8.0, 3.0 / 4.0];
const MIX_SOURCE: [f32; 4] = [7.0 / 32.0, 1.0 / 32.0, 1.0 / 8.0, 1.0 / 4.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const HALF_GREEN: [f32; 4] = [0.0, 0.5, 0.0, 0.5];

struct OperatorOracle {
    operator: Op,
    normal: [f32; 4],
    // For Cs=(7/8,1/8,1/2),as=1/4,Cb=(1/4,3/4,1/2),ab=3/4:
    // co = offset + blend_weight*B. These coefficients are prederived from
    // Fa*(Cs/16+3B/16)+Fb*(3Cb/4), not obtained from a factor implementation.
    offset: [f32; 3],
    blend_weight: f32,
    alpha: f32,
    zero_source_preserves: bool,
    zero_parent_preserves: bool,
}

const OPERATORS: [OperatorOracle; 12] = [
    OperatorOracle {
        operator: Op::Clear,
        normal: [0.0; 4],
        offset: [0.0; 3],
        blend_weight: 0.0,
        alpha: 0.0,
        zero_source_preserves: false,
        zero_parent_preserves: false,
    },
    OperatorOracle {
        operator: Op::Copy,
        normal: [3.0 / 16.0, 1.0 / 16.0, 1.0 / 8.0, 1.0 / 4.0],
        offset: [7.0 / 128.0, 1.0 / 128.0, 1.0 / 32.0],
        blend_weight: 3.0 / 16.0,
        alpha: 1.0 / 4.0,
        zero_source_preserves: false,
        zero_parent_preserves: true,
    },
    OperatorOracle {
        operator: Op::Destination,
        normal: PARENT,
        offset: [3.0 / 16.0, 9.0 / 16.0, 3.0 / 8.0],
        blend_weight: 0.0,
        alpha: 3.0 / 4.0,
        zero_source_preserves: true,
        zero_parent_preserves: false,
    },
    OperatorOracle {
        operator: Op::SourceOver,
        normal: [21.0 / 64.0, 31.0 / 64.0, 13.0 / 32.0, 13.0 / 16.0],
        offset: [25.0 / 128.0, 55.0 / 128.0, 5.0 / 16.0],
        blend_weight: 3.0 / 16.0,
        alpha: 13.0 / 16.0,
        zero_source_preserves: true,
        zero_parent_preserves: true,
    },
    OperatorOracle {
        operator: Op::DestinationOver,
        normal: [15.0 / 64.0, 37.0 / 64.0, 13.0 / 32.0, 13.0 / 16.0],
        offset: [103.0 / 512.0, 289.0 / 512.0, 49.0 / 128.0],
        blend_weight: 3.0 / 64.0,
        alpha: 13.0 / 16.0,
        zero_source_preserves: true,
        zero_parent_preserves: true,
    },
    OperatorOracle {
        operator: Op::SourceIn,
        normal: [9.0 / 64.0, 3.0 / 64.0, 3.0 / 32.0, 3.0 / 16.0],
        offset: [21.0 / 512.0, 3.0 / 512.0, 3.0 / 128.0],
        blend_weight: 9.0 / 64.0,
        alpha: 3.0 / 16.0,
        zero_source_preserves: false,
        zero_parent_preserves: false,
    },
    OperatorOracle {
        operator: Op::DestinationIn,
        normal: [3.0 / 64.0, 9.0 / 64.0, 3.0 / 32.0, 3.0 / 16.0],
        offset: [3.0 / 64.0, 9.0 / 64.0, 3.0 / 32.0],
        blend_weight: 0.0,
        alpha: 3.0 / 16.0,
        zero_source_preserves: false,
        zero_parent_preserves: false,
    },
    OperatorOracle {
        operator: Op::SourceOut,
        normal: [3.0 / 64.0, 1.0 / 64.0, 1.0 / 32.0, 1.0 / 16.0],
        offset: [7.0 / 512.0, 1.0 / 512.0, 1.0 / 128.0],
        blend_weight: 3.0 / 64.0,
        alpha: 1.0 / 16.0,
        zero_source_preserves: false,
        zero_parent_preserves: true,
    },
    OperatorOracle {
        operator: Op::DestinationOut,
        normal: [9.0 / 64.0, 27.0 / 64.0, 9.0 / 32.0, 9.0 / 16.0],
        offset: [9.0 / 64.0, 27.0 / 64.0, 9.0 / 32.0],
        blend_weight: 0.0,
        alpha: 9.0 / 16.0,
        zero_source_preserves: true,
        zero_parent_preserves: false,
    },
    OperatorOracle {
        operator: Op::SourceAtop,
        normal: [9.0 / 32.0, 15.0 / 32.0, 3.0 / 8.0, 3.0 / 4.0],
        offset: [93.0 / 512.0, 219.0 / 512.0, 39.0 / 128.0],
        blend_weight: 9.0 / 64.0,
        alpha: 3.0 / 4.0,
        zero_source_preserves: true,
        zero_parent_preserves: false,
    },
    OperatorOracle {
        operator: Op::DestinationAtop,
        normal: [3.0 / 32.0, 5.0 / 32.0, 1.0 / 8.0, 1.0 / 4.0],
        offset: [31.0 / 512.0, 73.0 / 512.0, 13.0 / 128.0],
        blend_weight: 3.0 / 64.0,
        alpha: 1.0 / 4.0,
        zero_source_preserves: false,
        zero_parent_preserves: true,
    },
    OperatorOracle {
        operator: Op::Xor,
        normal: [3.0 / 16.0, 7.0 / 16.0, 5.0 / 16.0, 5.0 / 8.0],
        offset: [79.0 / 512.0, 217.0 / 512.0, 37.0 / 128.0],
        blend_weight: 3.0 / 64.0,
        alpha: 5.0 / 8.0,
        zero_source_preserves: true,
        zero_parent_preserves: true,
    },
];

// Independent B(Cs,Cb) anchors at straight Cs=(7/8,1/8,1/2),
// Cb=(1/4,3/4,1/2). Nonseparable values retain distinct Hue and Color.
const MIXES: [(BlendMode, [f32; 3]); 16] = [
    (BlendMode::Normal, [7.0 / 8.0, 1.0 / 8.0, 1.0 / 2.0]),
    (BlendMode::Multiply, [7.0 / 32.0, 3.0 / 32.0, 1.0 / 4.0]),
    (BlendMode::Screen, [29.0 / 32.0, 25.0 / 32.0, 3.0 / 4.0]),
    (BlendMode::Overlay, [7.0 / 16.0, 9.0 / 16.0, 1.0 / 2.0]),
    (BlendMode::Darken, [1.0 / 4.0, 1.0 / 8.0, 1.0 / 2.0]),
    (BlendMode::Lighten, [7.0 / 8.0, 3.0 / 4.0, 1.0 / 2.0]),
    (BlendMode::ColorDodge, [1.0, 6.0 / 7.0, 1.0]),
    (BlendMode::ColorBurn, [1.0 / 7.0, 0.0, 0.0]),
    (BlendMode::HardLight, [13.0 / 16.0, 3.0 / 16.0, 1.0 / 2.0]),
    (BlendMode::SoftLight, [7.0 / 16.0, 39.0 / 64.0, 1.0 / 2.0]),
    (BlendMode::Difference, [5.0 / 8.0, 5.0 / 8.0, 0.0]),
    (BlendMode::Exclusion, [11.0 / 16.0, 11.0 / 16.0, 1.0 / 2.0]),
    (BlendMode::Hue, [179.0 / 200.0, 79.0 / 200.0, 129.0 / 200.0]),
    (
        BlendMode::Saturation,
        [71.0 / 800.0, 671.0 / 800.0, 371.0 / 800.0],
    ),
    (BlendMode::Color, [1.0, 29.0 / 86.0, 115.0 / 172.0]),
    (
        BlendMode::Luminosity,
        [11.0 / 160.0, 91.0 / 160.0, 51.0 / 160.0],
    ),
];

#[derive(Clone, Copy, Debug)]
struct Case {
    operator: Op,
    mix: BlendMode,
    source: [f32; 4],
    parent: [f32; 4],
    expected: [f32; 4],
}

fn composition(operator: Op, mix: BlendMode) -> LayerComposition {
    LayerComposition::try_porter_duff(operator, mix).unwrap()
}

fn arithmetic_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for row in &OPERATORS {
        cases.push(Case {
            operator: row.operator,
            mix: BlendMode::Normal,
            source: SOURCE,
            parent: PARENT,
            expected: row.normal,
        });
        for (mix, b) in MIXES {
            cases.push(Case {
                operator: row.operator,
                mix,
                source: MIX_SOURCE,
                parent: PARENT,
                expected: [
                    row.offset[0] + row.blend_weight * b[0],
                    row.offset[1] + row.blend_weight * b[1],
                    row.offset[2] + row.blend_weight * b[2],
                    row.alpha,
                ],
            });
        }
    }
    cases
}

#[test]
fn porter_duff_constructor_rejects_plus_and_layer_selectors_replace_the_whole_operation() {
    for row in &OPERATORS {
        let error = LayerComposition::try_porter_duff(row.operator, BlendMode::Plus).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
    }
    let value = composition(Op::SourceIn, BlendMode::Color);
    assert_eq!(value.operator(), Some(Op::SourceIn));
    assert_eq!(value.blend_mode(), BlendMode::Color);
    let layer = Layer::new().with_composition(value);
    assert_eq!(layer.composition(), value);
    assert_eq!(
        layer
            .clone()
            .blend(BlendMode::Multiply)
            .composition()
            .operator(),
        Some(Op::SourceOver)
    );
    assert_eq!(
        layer
            .clone()
            .blend(BlendMode::Plus)
            .composition()
            .operator(),
        None
    );
    assert_eq!(
        layer.blend(BlendMode::Plus).composition().blend_mode(),
        BlendMode::Plus
    );
    assert!(!Layer::new().is_isolated());
    assert!(Layer::new().isolated().is_isolated());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn porter_duff_shader_matches_twelve_factors_and_all_sixteen_mix_functions() {
    use crate::{
        RuntimeOperation, backend::CompositionBlendVectorForTest,
        gpu_transaction::GpuOperationStage,
    };
    let (mut backend, identity, requests) =
        super::gpu::composition_selected_backend_and_requests_for_test();
    let mut cases = arithmetic_cases();
    for row in &OPERATORS {
        for (source, parent, expected) in [
            (
                [0.0; 4],
                PARENT,
                if row.zero_source_preserves {
                    PARENT
                } else {
                    [0.0; 4]
                },
            ),
            (
                SOURCE,
                [0.0; 4],
                if row.zero_parent_preserves {
                    SOURCE
                } else {
                    [0.0; 4]
                },
            ),
            ([0.0; 4], [0.0; 4], [0.0; 4]),
        ] {
            cases.push(Case {
                operator: row.operator,
                mix: BlendMode::Normal,
                source,
                parent,
                expected,
            });
        }
    }
    let mut vectors = cases
        .iter()
        .map(|case| CompositionBlendVectorForTest {
            composition: composition(case.operator, case.mix),
            source: case.source,
            parent: case.parent,
            opacity: 1.0,
            clip_alpha: None,
        })
        .collect::<Vec<_>>();
    let mut expected = cases.iter().map(|case| case.expected).collect::<Vec<_>>();
    for (coverage, result) in [
        (0.0, PARENT),
        (0.5, [3.0 / 16.0, 5.0 / 16.0, 1.0 / 4.0, 1.0 / 2.0]),
        (1.0, SOURCE),
    ] {
        vectors.push(CompositionBlendVectorForTest {
            composition: composition(Op::Copy, BlendMode::Normal),
            source: SOURCE,
            parent: PARENT,
            opacity: 1.0,
            clip_alpha: Some(coverage),
        });
        expected.push(result);
    }
    // All 243 draws use the actual production shader. Limit live vector
    // resources to eight draws while reusing the selected backend/device.
    for (batch, expected) in vectors.chunks(8).zip(expected.chunks(8)) {
        let observed = pollster::block_on(async {
            let transaction = backend.begin_gpu_operation(
                identity,
                GpuOperationStage::Render,
                RuntimeOperation::EffectRendering,
            )?;
            let prepared = backend
                .composition_shader_blend_preparation_for_test(identity, &requests, batch)?;
            super::gpu::composition_submit_and_read_gpu_vectors_for_test(
                &mut backend,
                identity,
                transaction,
                prepared,
            )
            .await
        })
        .unwrap();
        assert_eq!(observed.rgba.len(), expected.len());
        for ((actual, expected), vector) in observed.rgba.iter().zip(expected).zip(batch) {
            let tolerance = 3.0 / 255.0;
            assert!(
                actual.iter().all(|v| v.is_finite())
                    && actual[3] >= -tolerance
                    && actual[3] <= 1.0 + tolerance
                    && actual[..3]
                        .iter()
                        .all(|v| *v >= -tolerance && *v <= actual[3] + tolerance),
                "finite premultiplied vector {vector:?}: {actual:?}"
            );
            assert!(
                actual
                    .iter()
                    .zip(expected)
                    .all(|(a, e)| (a - e).abs() <= tolerance),
                "vector {vector:?}: actual={actual:?}, expected={expected:?}, format={:?}",
                observed.working_format
            );
        }
    }
}

#[test]
fn porter_duff_empty_foreground_preserves_bounded_backdrop_filter_composition() {
    use crate::{
        BackdropCaptureBounds, BackdropFilterInput, FilterList, FilterOp, UnitFilterAmount,
    };

    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    for (isolated, expected) in [
        // Inverting half-alpha red gives premultiplied (0,1/2,1/2), alpha1/2.
        // Source-over retains a quarter red; Copy replaces the parent entirely.
        (true, [0.25, 0.5, 0.5, 0.75]),
        (false, [0.0, 0.5, 0.5, 0.5]),
    ] {
        let filters = FilterList::try_ops(vec![FilterOp::invert(
            UnitFilterAmount::try_new(1.0).unwrap(),
        )])
        .unwrap();
        let layer = Layer::new()
            .try_backdrop_filter(
                BackdropFilterInput::try_new(
                    filters,
                    BackdropCaptureBounds::try_new(bounds).unwrap(),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        let layer = if isolated {
            layer.isolated()
        } else {
            layer.with_composition(composition(Op::Copy, BlendMode::Normal))
        };
        let mut scene = Scene::new();
        scene.fill(bounds, straight_color([0.5, 0.0, 0.0, 0.5]));
        scene.layer(layer, |_| {});
        for format in formats.iter().copied() {
            let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
            assert_pixel(
                pixel_rgba(&output, 0, 0),
                expected,
                &format!("empty backdrop foreground {isolated} {format:?}"),
            );
        }
    }
}

fn renderer_and_surface(size: Size) -> (Renderer, Surface) {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let surface = pollster::block_on(renderer.create_headless(size, 1.0)).unwrap();
    (renderer, surface)
}

fn graph_layer(operator: Op, mix: BlendMode, bounds: Rect) -> Layer {
    masked_layer(mix, bounds, 255).with_composition(composition(operator, mix))
}

fn render_noop_control(
    renderer: &mut Renderer,
    surface: &mut Surface,
    scene: &Scene,
) -> crate::ImageBuffer {
    // A mathematically empty operation can disappear during normalization.
    // Observe the ordinary public route without demanding a retained graph.
    let before = surface.headless_publication_count_for_test();
    let stats =
        pollster::block_on(renderer.render(surface, scene, crate::Parameters::default())).unwrap();
    assert!(stats.route.is_some());
    assert_eq!(stats, renderer.stats());
    assert_eq!(surface.headless_publication_count_for_test(), before + 1);
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn assert_pixel(actual: [u8; 4], expected: [f32; 4], context: &str) {
    // At ao=1/16, one RGBA8 premultiplied store quantum can become sixteen
    // straight RGB quanta. Compare independent premultiplied RGB within three
    // bytes, with a separate two-byte alpha bound. The raw GPU vector test
    // independently verifies normalized numerical results before readback.
    let alpha = f32::from(actual[3]) / 255.0;
    assert!(
        (f32::from(actual[3]) - expected[3] * 255.0).abs() <= 2.0,
        "alpha {context}: actual={actual:?}, expected={expected:?}"
    );
    for channel in 0..3 {
        let premultiplied_byte = f32::from(actual[channel]) * alpha;
        assert!(
            (premultiplied_byte - expected[channel] * 255.0).abs() <= 3.0,
            "premultiplied channel{channel} {context}: actual={actual:?}, expected={expected:?}"
        );
    }
}

fn scene_for_cases(cases: &[Case]) -> Scene {
    let mut scene = Scene::new();
    for (index, case) in cases.iter().enumerate() {
        let bounds = tile(index);
        scene.fill(bounds, straight_color(case.parent));
        // Each destructive operation has its OWN operation clip; transparent
        // source outside this tile must never erase a preceding independent case.
        scene.layer(
            graph_layer(case.operator, case.mix, bounds)
                .try_clip(Shape::rect(bounds))
                .unwrap(),
            |scene| {
                scene.fill(bounds, straight_color(case.source));
            },
        );
    }
    scene
}

#[test]
fn porter_duff_public_graph_executes_every_operator_mix_pair_with_independent_alpha() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(16.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    for batch in arithmetic_cases().chunks(8) {
        let scene = scene_for_cases(batch);
        for format in formats.iter().copied() {
            let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
            for (index, case) in batch.iter().enumerate() {
                assert_pixel(
                    pixel_rgba(&output, u32::try_from(index).unwrap() * 2, 0),
                    case.expected,
                    &format!("{format:?} {case:?}"),
                );
            }
        }
    }
}

#[test]
fn porter_duff_existing_blend_selector_restores_direct_source_over_or_local_plus() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    for (mix, expected) in [
        (
            BlendMode::Normal,
            [23.0 / 64.0, 29.0 / 64.0, 13.0 / 32.0, 13.0 / 16.0],
        ),
        (BlendMode::Plus, [13.0 / 32.0, 19.0 / 32.0, 1.0 / 2.0, 1.0]),
    ] {
        let mut scene = Scene::new();
        scene.fill(bounds, straight_color(PARENT));
        scene.layer(
            Layer::new()
                .with_composition(composition(Op::SourceIn, BlendMode::Color))
                .blend(mix),
            |scene| {
                scene.fill(bounds, straight_color(MIX_SOURCE));
            },
        );
        let output = render_pixels(&mut renderer, &mut surface, &scene, None);
        assert_near(
            pixel_rgba(&output, 0, 0),
            straight_bytes(expected),
            3,
            &format!("existing selector {mix:?}"),
        );
    }
}

#[test]
fn porter_duff_nonoverlap_regions_use_the_parent_domain_and_actual_accumulated_backdrop() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(6.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 6.0, 2.0);
    for row in &OPERATORS {
        let mut scene = Scene::new();
        scene.fill(Rect::new(0.0, 0.0, 3.0, 2.0), straight_color(PARENT));
        scene.layer(
            graph_layer(row.operator, BlendMode::Normal, bounds),
            |scene| {
                scene.fill(Rect::new(2.0, 0.0, 3.0, 2.0), straight_color(SOURCE));
            },
        );
        for format in formats.iter().copied() {
            let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
            for (x, expected) in [
                (
                    1,
                    if row.zero_source_preserves {
                        PARENT
                    } else {
                        [0.0; 4]
                    },
                ),
                (2, row.normal),
                (
                    4,
                    if row.zero_parent_preserves {
                        SOURCE
                    } else {
                        [0.0; 4]
                    },
                ),
                (5, [0.0; 4]),
            ] {
                assert_pixel(
                    pixel_rgba(&output, x, 0),
                    expected,
                    &format!("nonoverlap {format:?} {:?} x={x}", row.operator),
                );
            }
        }
    }
    let mut accumulated = Scene::new();
    accumulated.fill(bounds, straight_color([0.5, 0.0, 0.0, 0.5]));
    accumulated.fill(bounds, straight_color(HALF_GREEN));
    accumulated.layer(
        graph_layer(Op::SourceIn, BlendMode::Normal, bounds),
        |scene| {
            scene.fill(bounds, straight_color([0.0, 0.0, 0.25, 0.25]));
        },
    );
    // Previous red/green produces alpha3/4, not merely the last green's1/2.
    for format in formats {
        let output = render_pixels(&mut renderer, &mut surface, &accumulated, Some(format));
        assert_pixel(
            pixel_rgba(&output, 0, 0),
            [0.0, 0.0, 3.0 / 16.0, 3.0 / 16.0],
            &format!("accumulated {format:?}"),
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum EmptySource {
    NoChildren,
    ZeroOpacity,
    TransparentPaint,
    ZeroMask,
    EmptyMask,
    OutsideFrame,
    ClippedChild,
}

#[test]
fn porter_duff_empty_sources_still_execute_destructive_operations() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(16.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    // Interleave operators so each bounded batch contains destructive work;
    // identity controls share that real graph frame rather than forcing a
    // graph for a frame consisting solely of mathematically empty operations.
    let cases = [
        EmptySource::NoChildren,
        EmptySource::ZeroOpacity,
        EmptySource::TransparentPaint,
        EmptySource::ZeroMask,
        EmptySource::EmptyMask,
        EmptySource::OutsideFrame,
        EmptySource::ClippedChild,
    ]
    .into_iter()
    .flat_map(|source| OPERATORS.iter().map(move |row| (row, source)))
    .collect::<Vec<_>>();
    for batch in cases.chunks(8) {
        let mut scene = Scene::new();
        for (index, (row, source)) in batch.iter().enumerate() {
            let bounds = tile(index);
            scene.fill(bounds, straight_color(PARENT));
            let mut layer = graph_layer(row.operator, BlendMode::Normal, bounds)
                .try_clip(Shape::rect(bounds))
                .unwrap();
            match source {
                EmptySource::ZeroOpacity => layer = layer.try_opacity(0.0).unwrap(),
                EmptySource::ZeroMask => {
                    layer = masked_layer(BlendMode::Normal, bounds, 0)
                        .with_composition(composition(row.operator, BlendMode::Normal))
                        .try_clip(Shape::rect(bounds))
                        .unwrap()
                }
                EmptySource::EmptyMask => {
                    let empty = Image::from_rgba(Size::new(0.0, 1.0), Vec::<u8>::new()).unwrap();
                    layer = layer.with_resolved_alpha_mask(
                        ResolvedLayerAlphaMask::try_new(empty, bounds).unwrap(),
                    );
                }
                _ => {}
            }
            scene.layer(layer, |scene| match source {
                EmptySource::NoChildren => {}
                EmptySource::TransparentPaint => {
                    scene.fill(bounds, Color::TRANSPARENT);
                }
                EmptySource::OutsideFrame => {
                    scene.fill(Rect::new(100.0, 0.0, 2.0, 2.0), straight_color(BLUE));
                }
                EmptySource::ClippedChild => {
                    scene.layer(
                        Layer::new()
                            .try_clip(Shape::rect(Rect::new(100.0, 0.0, 2.0, 2.0)))
                            .unwrap(),
                        |scene| {
                            scene.fill(bounds, straight_color(BLUE));
                        },
                    );
                }
                _ => {
                    scene.fill(bounds, straight_color(BLUE));
                }
            });
        }
        for format in formats.iter().copied() {
            let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
            for (index, (row, source)) in batch.iter().enumerate() {
                let expected = if row.zero_source_preserves {
                    PARENT
                } else {
                    [0.0; 4]
                };
                assert_pixel(
                    pixel_rgba(&output, u32::try_from(index).unwrap() * 2, 0),
                    expected,
                    &format!("empty {format:?} {:?} {source:?}", row.operator),
                );
            }
        }
    }
}

#[test]
fn porter_duff_annihilated_source_needs_no_text_ink_but_retains_copy() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    for empty_mask in [false, true] {
        let layer = Layer::new().with_composition(composition(Op::Copy, BlendMode::Normal));
        let layer = if empty_mask {
            layer.with_resolved_alpha_mask(
                ResolvedLayerAlphaMask::try_new(
                    Image::from_rgba(Size::new(0.0, 1.0), Vec::<u8>::new()).unwrap(),
                    bounds,
                )
                .unwrap(),
            )
        } else {
            layer.try_opacity(0.0).unwrap()
        };
        let mut scene = Scene::new();
        scene.fill(bounds, straight_color(RED));
        scene.layer(layer, |scene| {
            super::support::add_planning_text(scene, crate::TextRunBounds::unspecified());
        });
        for format in formats.iter().copied() {
            let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
            assert_eq!(pixel_rgba(&output, 0, 0)[3], 0, "empty mask={empty_mask}");
        }
    }

    // The same unresolved ink is required when source is not annihilated.
    let mut visible = Scene::new();
    visible.layer(
        Layer::new().with_composition(composition(Op::Copy, BlendMode::Normal)),
        |scene| super::support::add_planning_text(scene, crate::TextRunBounds::unspecified()),
    );
    let error =
        pollster::block_on(renderer.render(&mut surface, &visible, crate::Parameters::default()))
            .unwrap_err();
    assert_eq!(error.code(), ErrorCode::UnresolvedResource);
}

#[test]
fn porter_duff_singular_source_discards_unresolved_ink_but_executes_copy() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let singular = Transform::scale(0.0, 1.0).unwrap();
    for inherited in [false, true] {
        let mut copy = Layer::new().with_composition(composition(Op::Copy, BlendMode::Normal));
        if !inherited {
            copy = copy.with_transform(singular);
        }
        let mut scene = Scene::new();
        let add_source = |scene: &mut Scene| {
            scene.layer(copy, |scene| {
                super::support::add_planning_text(scene, crate::TextRunBounds::unspecified());
            });
        };
        if inherited {
            scene.layer(Layer::new().with_transform(singular), add_source);
        } else {
            add_source(&mut scene);
        }
        for format in formats.iter().copied() {
            let before = surface.headless_publication_count_for_test();
            let stats =
                pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                    &mut surface,
                    &scene,
                    crate::Parameters {
                        base_color: Color::BLACK,
                        ..crate::Parameters::default()
                    },
                    format,
                ))
                .unwrap();
            assert_eq!(stats.route, Some(crate::RenderRoute::GpuGraph));
            assert_eq!(stats.vello_passes, 0);
            assert_eq!(surface.headless_publication_count_for_test(), before + 1);
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for y in 0..2 {
                for x in 0..2 {
                    assert_eq!(pixel_rgba(&pixels, x, y), [0; 4], "inherited={inherited}");
                }
            }
        }
    }
}

#[test]
fn porter_duff_singular_own_clip_preserves_parent_without_source_prerequisites() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let singular = Transform::scale(0.0, 1.0).unwrap();
    for inherited in [false, true] {
        for unspecified_text in [false, true] {
            let mut copy = Layer::new()
                .with_composition(composition(Op::Copy, BlendMode::Normal))
                .try_clip(Shape::rect(bounds))
                .unwrap();
            if !inherited {
                copy = copy.with_transform(singular);
            }
            let add_source = |scene: &mut Scene| {
                scene.layer(copy, |scene| {
                    if unspecified_text {
                        super::support::add_planning_text(
                            scene,
                            crate::TextRunBounds::unspecified(),
                        );
                    } else {
                        scene.fill(bounds, straight_color(BLUE));
                    }
                });
            };
            let mut scene = Scene::new();
            if inherited {
                scene.layer(Layer::new().with_transform(singular), add_source);
            } else {
                add_source(&mut scene);
            }
            // No root draw: an erased operation must not leave a graph containing
            // only root initialization and presentation outside the closed subset.
            let before = surface.headless_publication_count_for_test();
            pollster::block_on(renderer.render(
                &mut surface,
                &scene,
                crate::Parameters {
                    base_color: Color::BLACK,
                    ..crate::Parameters::default()
                },
            ))
            .unwrap();
            assert_eq!(surface.headless_publication_count_for_test(), before + 1);
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for y in 0..2 {
                for x in 0..2 {
                    assert_eq!(
                        pixel_rgba(&pixels, x, y),
                        [0, 0, 0, 255],
                        "inherited={inherited}, text={unspecified_text}"
                    );
                }
            }
        }
    }
}

#[test]
fn porter_duff_transforms_preserve_the_parent_domain_and_active_operation_clips() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(6.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 6.0, 2.0);
    for scenario in 0..5 {
        let mut scene = Scene::new();
        scene.fill(bounds, straight_color(RED));
        match scenario {
            0 | 1 => {
                // No own clip: singular source becomes transparent, but Copy
                // still clears the complete parent. An own clip transformed
                // by the same singular map instead has zero operation coverage.
                let mut layer = Layer::new()
                    .with_composition(composition(Op::Copy, BlendMode::Normal))
                    .with_transform(Transform::scale(0.0, 1.0).unwrap());
                if scenario == 1 {
                    layer = layer.try_clip(Shape::rect(bounds)).unwrap();
                }
                scene.layer(layer, |scene| {
                    scene.fill(bounds, straight_color(BLUE));
                });
            }
            2 | 3 => {
                let mut layer = Layer::new()
                    .with_composition(composition(Op::Copy, BlendMode::Normal))
                    .with_transform(Transform::translation(2.0, 0.0).unwrap());
                if scenario == 2 {
                    layer = layer
                        .try_clip(Shape::rect(Rect::new(0.0, 0.0, 2.0, 2.0)))
                        .unwrap();
                }
                scene.layer(layer, |scene| {
                    scene.fill(Rect::new(0.0, 0.0, 1.0, 2.0), straight_color(BLUE));
                });
            }
            4 => {
                scene.layer(
                    Layer::new()
                        .try_clip(Shape::rect(Rect::new(1.0, 0.0, 4.0, 2.0)))
                        .unwrap(),
                    |scene| {
                        scene.layer(
                            Layer::new()
                                .with_composition(composition(Op::Copy, BlendMode::Normal))
                                .with_transform(Transform::scale(0.0, 1.0).unwrap()),
                            |scene| {
                                scene.fill(bounds, straight_color(BLUE));
                            },
                        );
                    },
                );
            }
            _ => unreachable!(),
        }
        let requested_formats = if scenario == 1 {
            vec![None]
        } else {
            formats.iter().copied().map(Some).collect()
        };
        for format in requested_formats {
            let output = if let Some(format) = format {
                render_pixels(&mut renderer, &mut surface, &scene, Some(format))
            } else {
                render_noop_control(&mut renderer, &mut surface, &scene)
            };
            for x in 0..6 {
                let expected = match scenario {
                    0 => [0.0; 4],
                    1 => RED,
                    2 => match x {
                        2 => BLUE,
                        3 => [0.0; 4],
                        _ => RED,
                    },
                    3 => {
                        if x == 2 {
                            BLUE
                        } else {
                            [0.0; 4]
                        }
                    }
                    4 => {
                        if x == 0 || x == 5 {
                            RED
                        } else {
                            [0.0; 4]
                        }
                    }
                    _ => unreachable!(),
                };
                assert_pixel(
                    pixel_rgba(&output, x, 0),
                    expected,
                    &format!("transform {format:?} scenario={scenario} x={x}"),
                );
            }
        }
    }
}

fn triangular_clip() -> Shape {
    let mut path = Path::new();
    path.move_to(Point::new(0.0, 0.0))
        .line_to(Point::new(2.0, 0.0))
        .line_to(Point::new(0.0, 2.0))
        .close();
    Shape::path(path)
}

#[test]
fn plus_lighter_is_a_complete_composition() {
    let value = composition(Op::PlusLighter, BlendMode::Normal);
    assert_eq!(value.operator(), Some(Op::PlusLighter));
    assert_eq!(value.blend_mode(), BlendMode::Normal);
    for (mix, _) in MIXES
        .into_iter()
        .filter(|(mix, _)| *mix != BlendMode::Normal)
    {
        let error = LayerComposition::try_porter_duff(Op::PlusLighter, mix).unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
    }
    let error = LayerComposition::try_porter_duff(Op::PlusLighter, BlendMode::Plus).unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidInput);
    assert!(
        crate::Capabilities::CURRENT
            .compositing()
            .supports_plus_lighter()
    );
}

#[test]
fn plus_lighter_cross_fades_within_the_isolated_group() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let fade = Layer::new()
        .with_composition(composition(Op::PlusLighter, BlendMode::Normal))
        .try_opacity(0.5)
        .unwrap();
    let mut scene = Scene::new();
    scene.fill(bounds, straight_color(HALF_GREEN));
    scene.layer(Layer::new().isolated(), |scene| {
        scene.layer(fade.clone(), |scene| {
            scene.fill(bounds, straight_color([0.5, 0.0, 0.0, 0.5]));
        });
        scene.layer(fade, |scene| {
            scene.fill(bounds, straight_color([0.0, 0.0, 0.5, 0.5]));
        });
    });
    // The group is half-opaque purple. Source-over retains half of the
    // external green backdrop, yielding three equal premultiplied channels.
    for format in formats {
        let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
        assert_pixel(
            pixel_rgba(&output, 0, 0),
            [0.25, 0.25, 0.25, 0.75],
            &format!("isolated plus-lighter {format:?}"),
        );
    }
}

#[test]
fn plus_lighter_caps_rgb_and_alpha_before_later_draws() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(4.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 4.0, 2.0);
    let mut scene = Scene::new();
    scene.fill(bounds, straight_color([0.75, 0.25, 0.0, 0.75]));
    scene.layer(
        Layer::new().with_composition(composition(Op::PlusLighter, BlendMode::Normal)),
        |scene| {
            scene.fill(
                Rect::new(0.0, 0.0, 2.0, 2.0),
                straight_color([0.75, 0.0, 0.25, 0.75]),
            );
        },
    );
    scene.fill(bounds, straight_color([0.0, 0.0, 0.0, 0.5]));
    // The additive result is (1, 1/4, 1/4, 1), before half-black.
    for format in formats {
        let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
        assert_pixel(
            pixel_rgba(&output, 0, 0),
            [0.5, 0.125, 0.125, 1.0],
            &format!("plus-lighter before later drawing {format:?}"),
        );
        assert_pixel(
            pixel_rgba(&output, 3, 0),
            [0.375, 0.125, 0.0, 0.875],
            &format!("transparent plus-lighter source {format:?}"),
        );
    }
}

#[test]
fn plus_lighter_attenuates_clip_and_mask_before_saturation() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(4.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 4.0, 2.0);
    let composition = composition(Op::PlusLighter, BlendMode::Normal);
    let plain = Layer::new().with_composition(composition);
    let masked = masked_layer(BlendMode::Normal, bounds, 128).with_composition(composition);
    for (layer, half) in [(plain, 1.0), (masked, 0.9375)] {
        let mut scene = Scene::new();
        scene.fill(bounds, straight_color([0.75, 0.0, 0.0, 1.0]));
        scene.layer(layer.try_clip(triangular_clip()).unwrap(), |scene| {
            scene.fill(bounds, straight_color([0.75, 0.0, 0.0, 1.0]));
        });
        for format in formats.iter().copied() {
            let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
            for (x, y, red) in [(0, 0, 1.0), (1, 0, half), (3, 0, 0.75)] {
                assert_pixel(
                    pixel_rgba(&output, x, y),
                    [red, 0.0, 0.0, 1.0],
                    &format!("plus-lighter clip {format:?} half={half} ({x},{y})"),
                );
            }
        }
    }
}

#[test]
fn plus_lighter_obeys_the_ledger_composite_budget() {
    use crate::{
        Parameters, RenderAccountingError, RenderAccountingOptions, RenderAttemptStatus,
        RenderLimits, RenderQuantity, RenderRefusalReason, RenderWork,
    };

    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::CompositePasses, 0)
        .unwrap();
    let options = Options::default()
        .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision)
        .with_accounting(RenderAccountingOptions::new().with_limits(limits));
    let mut renderer = pollster::block_on(Renderer::new(options)).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut baseline = Scene::new();
    baseline.fill(bounds, straight_color(RED));
    let before = render_noop_control(&mut renderer, &mut surface, &baseline);
    let before_stats = renderer.stats();
    let before_publications = surface.headless_publication_count_for_test();

    let mut scene = Scene::new();
    scene.fill(bounds, straight_color(RED));
    scene.layer(
        Layer::new().with_composition(composition(Op::PlusLighter, BlendMode::Normal)),
        |scene| {
            scene.fill(bounds, straight_color(BLUE));
        },
    );
    let error = pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
        .unwrap_err();
    let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
        panic!("expected a composite pass refusal, got {error:?}");
    };
    assert_eq!(
        refusal.quantity(),
        RenderQuantity::Work(RenderWork::CompositePasses)
    );
    assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit: 0 });
    let report = renderer.last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(report.cleanup_fault(), None);
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Submissions),
        0
    );
    assert_eq!(renderer.stats(), before_stats);
    assert_eq!(
        surface.headless_publication_count_for_test(),
        before_publications
    );
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    assert_eq!(
        render_noop_control(&mut renderer, &mut surface, &baseline),
        before
    );
}

#[test]
fn porter_duff_fractional_nonrectangular_clip_interpolates_the_complete_operation() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(4.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 4.0, 2.0);
    for operator in [Op::Clear, Op::Copy] {
        let mut scene = Scene::new();
        scene.fill(bounds, straight_color(RED));
        scene.layer(
            Layer::new()
                .with_composition(composition(operator, BlendMode::Normal))
                .try_clip(triangular_clip())
                .unwrap(),
            |scene| {
                // Smaller source than the operation clip. The right half-covered
                // pixel has no source, but still loses half its backdrop under Copy.
                scene.fill(Rect::new(0.0, 0.0, 1.0, 2.0), straight_color(BLUE));
            },
        );
        for format in formats.iter().copied() {
            let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
            let full = if operator == Op::Clear {
                [0.0; 4]
            } else {
                BLUE
            };
            let source_half = if operator == Op::Clear {
                [0.5, 0.0, 0.0, 0.5]
            } else {
                [0.5, 0.0, 0.5, 1.0]
            };
            for (x, y, expected) in [
                (0, 0, full),
                (1, 0, [0.5, 0.0, 0.0, 0.5]),
                (0, 1, source_half),
                (1, 1, RED),
                (3, 0, RED),
            ] {
                assert_pixel(
                    pixel_rgba(&output, x, y),
                    expected,
                    &format!("fractional clip {format:?} {operator:?} ({x},{y})"),
                );
            }
        }
    }
}

#[test]
fn porter_duff_isolated_bottom_children_start_transparent_and_later_children_accumulate() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    for bottom in [
        Op::Destination,
        Op::SourceIn,
        Op::DestinationIn,
        Op::DestinationOut,
        Op::SourceAtop,
    ] {
        for later_green in [false, true] {
            let mut scene = Scene::new();
            scene.fill(bounds, straight_color(RED));
            scene.layer(
                graph_layer(Op::Copy, BlendMode::Normal, bounds).isolated(),
                |scene| {
                    scene.layer(
                        Layer::new().with_composition(composition(bottom, BlendMode::Normal)),
                        |scene| {
                            scene.fill(bounds, straight_color([0.0, 0.0, 0.5, 0.5]));
                        },
                    );
                    if later_green {
                        scene.fill(bounds, straight_color(HALF_GREEN));
                    }
                },
            );
            for format in formats.iter().copied() {
                let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
                assert_pixel(
                    pixel_rgba(&output, 0, 0),
                    if later_green { HALF_GREEN } else { [0.0; 4] },
                    &format!("isolated {format:?} {bottom:?} later_green={later_green}"),
                );
            }
        }
    }
    // The default wrapper forwards red to its SourceIn child: blue1/2 survives,
    // then green1/2 yields premultiplied (0,1/2,1/4), alpha3/4.
    let mut forwarded = Scene::new();
    forwarded.fill(bounds, straight_color(RED));
    forwarded.layer(Layer::new(), |scene| {
        scene.layer(
            Layer::new().with_composition(composition(Op::SourceIn, BlendMode::Normal)),
            |scene| {
                scene.fill(bounds, straight_color([0.0, 0.0, 0.5, 0.5]));
            },
        );
        scene.fill(bounds, straight_color(HALF_GREEN));
    });
    for format in formats {
        let output = render_pixels(&mut renderer, &mut surface, &forwarded, Some(format));
        assert_pixel(
            pixel_rgba(&output, 0, 0),
            [0.0, 0.5, 0.25, 0.75],
            &format!("default wrapper {format:?}"),
        );
    }
}

#[test]
fn porter_duff_empty_isolated_source_and_nested_mask_opacity_reach_the_outer_operation() {
    let (mut renderer, mut surface) = renderer_and_surface(Size::new(2.0, 2.0));
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    for outer in [Op::Copy, Op::SourceOver] {
        let mut scene = Scene::new();
        scene.fill(bounds, straight_color(RED));
        scene.layer(
            graph_layer(outer, BlendMode::Normal, bounds).isolated(),
            |scene| {
                scene.layer(
                    Layer::new().with_composition(composition(Op::SourceIn, BlendMode::Normal)),
                    |scene| {
                        scene.fill(bounds, straight_color(BLUE));
                    },
                );
            },
        );
        let requested_formats = if outer == Op::SourceOver {
            vec![None]
        } else {
            formats.iter().copied().map(Some).collect()
        };
        for format in requested_formats {
            let output = if let Some(format) = format {
                render_pixels(&mut renderer, &mut surface, &scene, Some(format))
            } else {
                render_noop_control(&mut renderer, &mut surface, &scene)
            };
            assert_pixel(
                pixel_rgba(&output, 0, 0),
                if outer == Op::Copy { [0.0; 4] } else { RED },
                &format!("empty outer {format:?} {outer:?}"),
            );
        }
    }
    let attenuated = || {
        masked_layer(BlendMode::Normal, bounds, 128)
            .with_composition(composition(Op::Copy, BlendMode::Normal))
            .try_opacity(0.5)
            .unwrap()
    };
    let mut nested = Scene::new();
    nested.fill(bounds, straight_color(RED));
    nested.layer(attenuated().isolated(), |scene| {
        scene.layer(attenuated(), |scene| {
            scene.fill(bounds, straight_color([0.0, 1.0, 0.0, 1.0]));
        });
    });
    // Each mask128/255 and opacity1/2 contributes64/255; outer Copy replaces
    // red with the nested premultiplied green source, not source-over red.
    for format in formats {
        let output = render_pixels(&mut renderer, &mut surface, &nested, Some(format));
        assert_pixel(
            pixel_rgba(&output, 0, 0),
            [0.0, 4096.0 / 65025.0, 0.0, 4096.0 / 65025.0],
            &format!("nested attenuation {format:?}"),
        );
    }
}
