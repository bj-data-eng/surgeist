//! Independent numeric anchors from Compositing and Blending Level 1's
//! separable formulas. Inputs and expectations are premultiplied normalized
//! RGBA; public headless readback is straight RGBA8. No blend oracle below
//! evaluates a second copy of the production blend algorithm.

use super::support::{
    composition_mask_image_from_alpha_for_test, graph_supported_working_formats_for_test,
    pixel_rgba,
};
use crate::{
    BlendMode, Color, EffectPrecision, EffectQualityPolicy, Extend, ImageQuality, Layer, Options,
    Parameters, PhysicalSize, Rect, RenderRoute, Renderer, ResolvedLayerAlphaMask, Scene, Shape,
    Size, Surface, reference::PremultipliedRgba8, resource::WorkingFormat,
};

const SOURCE: [f32; 4] = [0.375, 0.125, 0.25, 0.5];
const BACKDROP: [f32; 4] = [0.125, 0.375, 0.25, 0.5];
const OPAQUE_SOURCE: [f32; 4] = [0.75, 0.25, 0.5, 1.0];
const OPAQUE_BACKDROP: [f32; 4] = [0.25, 0.75, 0.5, 1.0];

// Each row gives (mode, opaque B RGB, partial-alpha co RGB). For the
// partial-alpha case as=ab=1/2, ao=3/4 and co=(Cs+Cb+B)/4.
// The Plus row is the distinct saturated-add composition control.
const MODES: [(BlendMode, [f32; 3], [f32; 3]); 13] = [
    (
        BlendMode::Normal,
        [0.75, 0.25, 0.5],
        [0.4375, 0.3125, 0.375],
    ),
    (
        BlendMode::Multiply,
        [0.1875, 0.1875, 0.25],
        [0.296875, 0.296875, 0.3125],
    ),
    (
        BlendMode::Screen,
        [0.8125, 0.8125, 0.75],
        [0.453125, 0.453125, 0.4375],
    ),
    (
        BlendMode::Overlay,
        [0.375, 0.625, 0.5],
        [0.34375, 0.40625, 0.375],
    ),
    (
        BlendMode::Darken,
        [0.25, 0.25, 0.5],
        [0.3125, 0.3125, 0.375],
    ),
    (
        BlendMode::Lighten,
        [0.75, 0.75, 0.5],
        [0.4375, 0.4375, 0.375],
    ),
    (BlendMode::ColorDodge, [1.0, 1.0, 1.0], [0.5, 0.5, 0.5]),
    (BlendMode::ColorBurn, [0.0, 0.0, 0.0], [0.25, 0.25, 0.25]),
    (
        BlendMode::HardLight,
        [0.625, 0.375, 0.5],
        [0.40625, 0.34375, 0.375],
    ),
    (
        BlendMode::SoftLight,
        [0.375, 0.65625, 0.5],
        [0.34375, 0.4140625, 0.375],
    ),
    (BlendMode::Difference, [0.5, 0.5, 0.0], [0.375, 0.375, 0.25]),
    (
        BlendMode::Exclusion,
        [0.625, 0.625, 0.5],
        [0.40625, 0.40625, 0.375],
    ),
    (BlendMode::Plus, [1.0, 1.0, 1.0], [0.5, 0.5, 0.5]),
];

#[derive(Clone, Copy, Debug)]
struct Case {
    mode: BlendMode,
    source: [f32; 4],
    backdrop: [f32; 4],
    expected: [f32; 4],
}

fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (mode, opaque, partial) in MODES {
        for (source, backdrop, expected) in [
            (
                OPAQUE_SOURCE,
                OPAQUE_BACKDROP,
                [opaque[0], opaque[1], opaque[2], 1.0],
            ),
            (
                SOURCE,
                BACKDROP,
                [
                    partial[0],
                    partial[1],
                    partial[2],
                    if mode == BlendMode::Plus { 1.0 } else { 0.75 },
                ],
            ),
            (SOURCE, [0.0; 4], SOURCE),
            ([0.0; 4], BACKDROP, BACKDROP),
            ([0.0; 4], [0.0; 4], [0.0; 4]),
        ] {
            cases.push(Case {
                mode,
                source,
                backdrop,
                expected,
            });
        }
    }
    // RGB channels independently cross the guard and threshold branches.
    // Dodge(0,1)=0 and Burn(1,0)=1 distinguish endpoint guard precedence.
    for (mode, source, backdrop, expected) in [
        (
            BlendMode::ColorDodge,
            [1.0, 1.0, 0.5, 1.0],
            [0.0, 0.25, 0.75, 1.0],
            [0.0, 1.0, 1.0, 1.0],
        ),
        (
            BlendMode::ColorBurn,
            [0.0, 0.0, 0.5, 1.0],
            [1.0, 0.75, 0.25, 1.0],
            [1.0, 0.0, 0.0, 1.0],
        ),
        (
            BlendMode::HardLight,
            [0.25, 0.5, 0.75, 1.0],
            [0.25, 0.25, 0.25, 1.0],
            [0.125, 0.25, 0.625, 1.0],
        ),
        (
            BlendMode::Overlay,
            [0.25, 0.25, 0.25, 1.0],
            [0.25, 0.5, 0.75, 1.0],
            [0.125, 0.25, 0.625, 1.0],
        ),
        (
            BlendMode::SoftLight,
            [0.25, 0.5, 0.75, 1.0],
            [0.25, 0.25, 0.25, 1.0],
            [0.15625, 0.25, 0.375, 1.0],
        ),
        (
            BlendMode::SoftLight,
            [0.75, 0.75, 0.75, 1.0],
            [0.125, 0.25, 0.5625, 1.0],
            [0.234375, 0.375, 0.65625, 1.0],
        ),
        (
            BlendMode::SoftLight,
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 1.0, 0.5, 1.0],
            [0.0, 1.0, 0.25, 1.0],
        ),
        (
            BlendMode::Plus,
            [0.8, 0.2, 0.7, 0.8],
            [0.6, 0.9, 0.4, 0.9],
            [1.0, 1.0, 1.0, 1.0],
        ),
    ] {
        cases.push(Case {
            mode,
            source,
            backdrop,
            expected,
        });
    }
    cases
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn separable_graph_shader_preserves_guards_thresholds_and_partial_alpha() {
    use crate::backend::CompositionBlendVectorForTest;
    let (backend, identity, requests) =
        super::gpu::composition_selected_backend_and_requests_for_test();
    let cases = cases();
    let vectors = cases
        .iter()
        .map(|case| CompositionBlendVectorForTest {
            composition: crate::LayerComposition::from_blend(case.mode),
            clip_alpha: None,
            source: case.source,
            parent: case.backdrop,
            opacity: 1.0,
        })
        .collect::<Vec<_>>();
    let expected = cases.iter().map(|case| case.expected).collect::<Vec<_>>();
    // Existing harness checks actual GPU readback against the literals, plus
    // finite, bounded premultiplied output, within 3/255 for either format.
    super::gpu::assert_composition_blend_gpu_vectors(
        backend, identity, &requests, &vectors, &expected,
    );
}

pub(super) fn straight_color(pixel: [f32; 4]) -> Color {
    if pixel[3] == 0.0 {
        return Color::TRANSPARENT;
    }
    Color::try_rgba(
        pixel[0] / pixel[3],
        pixel[1] / pixel[3],
        pixel[2] / pixel[3],
        pixel[3],
    )
    .unwrap()
}

pub(super) fn straight_bytes(pixel: [f32; 4]) -> [u8; 4] {
    if pixel[3] == 0.0 {
        return [0; 4];
    }
    [
        (pixel[0] / pixel[3] * 255.0).round() as u8,
        (pixel[1] / pixel[3] * 255.0).round() as u8,
        (pixel[2] / pixel[3] * 255.0).round() as u8,
        (pixel[3] * 255.0).round() as u8,
    ]
}

pub(super) fn masked_layer(mode: BlendMode, rect: Rect, alpha: u8) -> Layer {
    let mask = composition_mask_image_from_alpha_for_test(
        PhysicalSize::new(1, 1),
        &[alpha],
        ImageQuality::Low,
        Extend::Pad,
    );
    Layer::new()
        .blend(mode)
        .with_resolved_alpha_mask(ResolvedLayerAlphaMask::try_new(mask, rect).unwrap())
}

pub(super) fn tile(index: usize) -> Rect {
    Rect::new(
        f64::from(u32::try_from(index % 8).unwrap() * 2),
        f64::from(u32::try_from(index / 8).unwrap() * 2),
        2.0,
        2.0,
    )
}

fn scene_for_cases(cases: &[Case], masked: bool) -> Scene {
    let mut scene = Scene::new();
    for (index, case) in cases.iter().enumerate() {
        let rect = tile(index);
        scene.fill(rect, straight_color(case.backdrop));
        let layer = if masked {
            masked_layer(case.mode, rect, 255)
        } else {
            Layer::new().blend(case.mode)
        };
        scene.layer(layer, |scene| {
            scene.fill(rect, straight_color(case.source));
        });
    }
    scene
}

pub(super) fn assert_near(actual: [u8; 4], expected: [u8; 4], tolerance: u8, context: &str) {
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(a, e)| a.abs_diff(e) <= tolerance),
        "{context}: actual={actual:?}, expected={expected:?}, tolerance={tolerance}"
    );
}

pub(super) fn render_pixels(
    renderer: &mut Renderer,
    surface: &mut Surface,
    scene: &Scene,
    format: Option<WorkingFormat>,
) -> crate::ImageBuffer {
    let before = surface.headless_publication_count_for_test();
    let stats = if let Some(format) = format {
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            surface,
            scene,
            Parameters::default(),
            format,
        ))
        .unwrap()
    } else {
        pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap()
    };
    assert_eq!(stats, renderer.stats());
    assert_eq!(surface.headless_publication_count_for_test(), before + 1);
    if let Some(format) = format {
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        assert_eq!(
            stats.effect_precision,
            Some(match format {
                WorkingFormat::HighPrecision => EffectPrecision::High,
                WorkingFormat::ReducedPrecision => EffectPrecision::Reduced,
            })
        );
        assert!(stats.composite_passes > 0);
        assert_eq!(stats.custom_present_passes, 1);
    } else {
        assert_eq!(stats.route, Some(RenderRoute::DirectVello));
        assert_eq!(stats.effect_precision, None);
        assert_eq!(stats.vello_passes, 1);
        assert_eq!(stats.composite_passes, 0);
    }
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

#[test]
fn separable_direct_clamps_blend_result_before_composition_after_local_plus() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    for antialiasing in [
        crate::Antialiasing::Area,
        crate::Antialiasing::Msaa8,
        crate::Antialiasing::Msaa16,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_antialiasing(antialiasing),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
        for (mode, expected) in [
            // Normal's B=clamp(Cs,0,1) has the same five-eighths result.
            (BlendMode::Normal, [159, 159, 159, 255]),
            // B=clamp(max(1/4,2),0,1)=1, then co=1/2*1+1/2*1/4=5/8.
            // This does not assume that local Plus saturates its own source.
            (BlendMode::Lighten, [159, 159, 159, 255]),
        ] {
            let mut scene = Scene::new();
            draw_local_plus_blend(&mut scene, mode, bounds);
            let pixels = render_pixels(&mut renderer, &mut surface, &scene, None);
            for y in 0..2 {
                for x in 0..2 {
                    assert_near(
                        pixel_rgba(&pixels, x, y),
                        expected,
                        2,
                        &format!("direct {mode:?}, {antialiasing:?}"),
                    );
                }
            }
        }
    }
}

#[test]
fn separable_capture_local_clamps_blend_result_before_composition_after_local_plus() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    for antialiasing in [
        crate::Antialiasing::Area,
        crate::Antialiasing::Msaa8,
        crate::Antialiasing::Msaa16,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_antialiasing(antialiasing)
                .with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
        let formats = graph_supported_working_formats_for_test(&mut renderer);
        for mode in [BlendMode::Normal, BlendMode::Lighten] {
            let mut scene = Scene::new();
            // Keep gray, mixing and local Plus together inside one raster
            // capture. Masking the mixing layer itself would canonicalize Plus
            // before the blend and miss this live-accumulator defect.
            scene.layer(masked_layer(BlendMode::Normal, bounds, 255), |scene| {
                draw_local_plus_blend(scene, mode, bounds);
            });
            for format in formats.iter().copied() {
                let pixels = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
                for y in 0..2 {
                    for x in 0..2 {
                        assert_near(
                            pixel_rgba(&pixels, x, y),
                            [159, 159, 159, 255],
                            2,
                            &format!("capture local {mode:?}, {antialiasing:?}, {format:?}"),
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn separable_bounding_preserves_local_plus_accumulation_before_later_paint() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let white = Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap();
    for antialiasing in [
        crate::Antialiasing::Area,
        crate::Antialiasing::Msaa8,
        crate::Antialiasing::Msaa16,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_antialiasing(antialiasing),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
        let mut scene = Scene::new();
        scene.fill(bounds, white);
        scene.layer(Layer::new().blend(BlendMode::Plus), |scene| {
            scene.fill(bounds, white);
        });
        scene.fill(bounds, Color::try_rgba(0.0, 0.0, 0.0, 0.5).unwrap());
        let pixels = render_pixels(&mut renderer, &mut surface, &scene, None);
        // Existing local Plus retains RGB=2, alpha=1 before later paint.
        // Half-black then leaves RGB=1, unlike prematurely saturating Plus.
        // Raster alpha quantization can move that endpoint by one byte.
        for y in 0..2 {
            for x in 0..2 {
                let pixel = pixel_rgba(&pixels, x, y);
                assert_near(pixel, [255; 4], 2, &format!("Plus {antialiasing:?}"));
                assert_eq!(pixel[3], 255);
            }
        }
    }
}

fn draw_local_plus_blend(scene: &mut Scene, mode: BlendMode, bounds: Rect) {
    let white = Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap();
    scene.fill(bounds, Color::try_rgba(0.25, 0.25, 0.25, 1.0).unwrap());
    scene.layer(
        Layer::new().blend(mode).try_opacity(0.5).unwrap(),
        |scene| {
            scene.fill(bounds, white);
            scene.layer(Layer::new().blend(BlendMode::Plus), |scene| {
                scene.fill(bounds, white);
            });
        },
    );
}

#[test]
fn separable_public_direct_and_masked_graph_pixels_match_independent_constants() {
    let cases = cases();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(16.0, 2.0), 1.0)).unwrap();
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    // Bound simultaneously live capture/pass allocations while reusing the
    // renderer and surface across every batch and supported working format.
    for batch in cases.chunks(8) {
        let direct = render_pixels(
            &mut renderer,
            &mut surface,
            &scene_for_cases(batch, false),
            None,
        );
        for (index, case) in batch.iter().enumerate() {
            let rect = tile(index);
            assert_near(
                pixel_rgba(&direct, rect.x() as u32, rect.y() as u32),
                straight_bytes(case.expected),
                3,
                &format!("direct {case:?}"),
            );
        }
        for format in formats.iter().copied() {
            let graph = render_pixels(
                &mut renderer,
                &mut surface,
                &scene_for_cases(batch, true),
                Some(format),
            );
            for (index, case) in batch.iter().enumerate() {
                let rect = tile(index);
                let actual = pixel_rgba(&graph, rect.x() as u32, rect.y() as u32);
                // Capture, graph storage and final straight conversion each round.
                assert_near(
                    actual,
                    straight_bytes(case.expected),
                    4,
                    &format!("graph {format:?} {case:?}"),
                );
                assert_near(
                    actual,
                    pixel_rgba(&direct, rect.x() as u32, rect.y() as u32),
                    4,
                    &format!("route parity {format:?} {case:?}"),
                );
            }
        }
    }
}

#[test]
fn separable_fractional_mask_opacity_and_clip_attenuate_before_blending() {
    // Opaque Cs=(3/4,1/4,1/2), ab=1/2, mask=128/255 and opacity=1/2:
    // effective as=64/255, ao=319/510. Literal straight readback bytes follow
    // co=(1-ab)*as*Cs+(1-as)*ab*Cb+as*ab*B, rounded only at output.
    let expected = [
        (BlendMode::ColorDodge, [128, 178, 153, 160]),
        (BlendMode::ColorBurn, [77, 127, 102, 160]),
        (BlendMode::HardLight, [109, 146, 128, 160]),
        (BlendMode::SoftLight, [96, 161, 128, 160]),
        (BlendMode::Difference, [102, 153, 102, 160]),
        (BlendMode::Exclusion, [109, 159, 128, 160]),
    ];
    let mut scene = Scene::new();
    for (index, (mode, _)) in expected.iter().enumerate() {
        let rect = tile(index);
        scene.fill(rect, straight_color(BACKDROP));
        scene.layer(
            masked_layer(*mode, rect, 128)
                .try_opacity(0.5)
                .unwrap()
                .try_clip(Shape::rect(rect))
                .unwrap(),
            |scene| {
                scene.fill(rect, straight_color(OPAQUE_SOURCE));
            },
        );
    }
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(12.0, 2.0), 1.0)).unwrap();
    for format in graph_supported_working_formats_for_test(&mut renderer) {
        let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
        for (index, (mode, expected)) in expected.iter().enumerate() {
            assert_near(
                pixel_rgba(&output, u32::try_from(index).unwrap() * 2, 0),
                *expected,
                4,
                &format!("attenuated {format:?} {mode:?}"),
            );
        }
    }
}

#[test]
fn separable_capture_local_dodge_and_outer_difference_compose_in_graph_order() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.fill(bounds, straight_color(OPAQUE_BACKDROP));
    scene.layer(masked_layer(BlendMode::Difference, bounds, 255), |scene| {
        scene.fill(bounds, straight_color(OPAQUE_BACKDROP));
        scene.layer(Layer::new().blend(BlendMode::ColorDodge), |scene| {
            scene.fill(bounds, straight_color(OPAQUE_SOURCE));
        });
    });
    // Capture-local dodge is white; external Difference(1,Cb)=(3/4,1/4,1/2).
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(2.0, 2.0), 1.0)).unwrap();
    for format in graph_supported_working_formats_for_test(&mut renderer) {
        let output = render_pixels(&mut renderer, &mut surface, &scene, Some(format));
        assert_near(
            pixel_rgba(&output, 0, 0),
            [191, 64, 128, 255],
            3,
            &format!("nested {format:?}"),
        );
        assert!(renderer.stats().vello_passes >= 2);
    }
}

#[test]
fn separable_reference_preserves_independent_opaque_anchors_and_endpoint_precedence() {
    let source = PremultipliedRgba8::try_new(191, 64, 128, 255).unwrap();
    let backdrop = PremultipliedRgba8::try_new(64, 191, 128, 255).unwrap();
    for (mode, opaque, _) in MODES {
        let actual = source.blend_over(backdrop, mode);
        assert_near(
            [actual.red(), actual.green(), actual.blue(), actual.alpha()],
            straight_bytes([opaque[0], opaque[1], opaque[2], 1.0]),
            2,
            &format!("reference {mode:?}"),
        );
        assert_eq!(
            source.blend_over(PremultipliedRgba8::TRANSPARENT, mode),
            source
        );
        assert_eq!(
            PremultipliedRgba8::TRANSPARENT.blend_over(backdrop, mode),
            backdrop
        );
    }
    let white = PremultipliedRgba8::try_new(255, 255, 255, 255).unwrap();
    let black = PremultipliedRgba8::try_new(0, 0, 0, 255).unwrap();
    assert_eq!(white.blend_over(black, BlendMode::ColorDodge), black);
    assert_eq!(black.blend_over(white, BlendMode::ColorBurn), white);
}
