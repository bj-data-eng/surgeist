//! Independent rational anchors for Compositing and Blending Level 1 §10.2.
//! RGB is joint, Lum uses (0.3, 0.59, 0.11), and blend inputs/expected outputs
//! below are premultiplied. Public readback converts to straight RGBA8.

use super::{
    separable::{assert_near, masked_layer, render_pixels, straight_bytes, straight_color, tile},
    support::{graph_supported_working_formats_for_test, pixel_rgba},
};
use crate::{
    Antialiasing, BlendMode, EffectQualityPolicy, Layer, Options, Rect, Renderer, Scene, Shape,
    Size,
};

const SOURCE: [f32; 4] = [7.0 / 8.0, 1.0 / 8.0, 1.0 / 2.0, 1.0];
const BACKDROP: [f32; 4] = [1.0 / 4.0, 3.0 / 4.0, 1.0 / 2.0, 1.0];
const HALF_SOURCE: [f32; 4] = [7.0 / 16.0, 1.0 / 16.0, 1.0 / 4.0, 1.0 / 2.0];
const HALF_BACKDROP: [f32; 4] = [1.0 / 8.0, 3.0 / 8.0, 1.0 / 4.0, 1.0 / 2.0];
const GRAY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

struct ModeExpectations {
    mode: BlendMode,
    opaque: [f32; 3],
    partial: [f32; 3],
    reversed: [f32; 3],
    gray_source: [f32; 3],
    gray_backdrop: [f32; 3],
}

// The rational values follow the pinned equations, independently of the CPU
// reference and shaders. For as=ab=1/2, ao=3/4 and co=(Cs+Cb+B)/4.
const MODES: [ModeExpectations; 4] = [
    ModeExpectations {
        mode: BlendMode::Hue,
        opaque: [179.0 / 200.0, 79.0 / 200.0, 129.0 / 200.0],
        partial: [101.0 / 200.0, 127.0 / 400.0, 329.0 / 800.0],
        reversed: [0.0, 313.0 / 516.0, 313.0 / 1032.0],
        gray_source: [229.0 / 400.0; 3],
        gray_backdrop: [0.5; 3],
    },
    ModeExpectations {
        mode: BlendMode::Saturation,
        opaque: [71.0 / 800.0, 671.0 / 800.0, 371.0 / 800.0],
        partial: [971.0 / 3200.0, 1371.0 / 3200.0, 1171.0 / 3200.0],
        reversed: [571.0 / 800.0, 171.0 / 800.0, 371.0 / 800.0],
        gray_source: [229.0 / 400.0; 3],
        gray_backdrop: [0.5; 3],
    },
    ModeExpectations {
        mode: BlendMode::Color,
        opaque: [1.0, 29.0 / 86.0, 115.0 / 172.0],
        partial: [17.0 / 32.0, 417.0 / 1376.0, 287.0 / 688.0],
        reversed: [11.0 / 160.0, 91.0 / 160.0, 51.0 / 160.0],
        gray_source: [229.0 / 400.0; 3],
        gray_backdrop: [787.0 / 800.0, 187.0 / 800.0, 487.0 / 800.0],
    },
    ModeExpectations {
        mode: BlendMode::Luminosity,
        opaque: [11.0 / 160.0, 91.0 / 160.0, 51.0 / 160.0],
        partial: [191.0 / 640.0, 231.0 / 640.0, 211.0 / 640.0],
        reversed: [1.0, 29.0 / 86.0, 115.0 / 172.0],
        gray_source: [71.0 / 400.0, 271.0 / 400.0, 171.0 / 400.0],
        gray_backdrop: [313.0 / 800.0; 3],
    },
];

#[derive(Clone, Copy, Debug)]
struct Case {
    condition: &'static str,
    mode: BlendMode,
    source: [f32; 4],
    backdrop: [f32; 4],
    expected: [f32; 4],
}

fn with_alpha(rgb: [f32; 3], alpha: f32) -> [f32; 4] {
    [rgb[0], rgb[1], rgb[2], alpha]
}

fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for row in MODES {
        for (condition, source, backdrop, expected) in [
            ("opaque", SOURCE, BACKDROP, with_alpha(row.opaque, 1.0)),
            (
                "partial alpha",
                HALF_SOURCE,
                HALF_BACKDROP,
                with_alpha(row.partial, 0.75),
            ),
            (
                "exchanged roles",
                BACKDROP,
                SOURCE,
                with_alpha(row.reversed, 1.0),
            ),
            (
                "gray source",
                GRAY,
                BACKDROP,
                with_alpha(row.gray_source, 1.0),
            ),
            (
                "gray backdrop",
                SOURCE,
                GRAY,
                with_alpha(row.gray_backdrop, 1.0),
            ),
            ("transparent source", [0.0; 4], HALF_BACKDROP, HALF_BACKDROP),
            ("transparent backdrop", HALF_SOURCE, [0.0; 4], HALF_SOURCE),
            ("both transparent", [0.0; 4], [0.0; 4], [0.0; 4]),
        ] {
            cases.push(Case {
                condition,
                mode: row.mode,
                source,
                backdrop,
                expected,
            });
        }
        // Valid byte grays need exact finite endpoint behavior even when the
        // weighted luminosity sum rounds differently from the gray channel.
        for gray in [29.0 / 255.0, 153.0 / 255.0] {
            for endpoint in [0.0, 1.0] {
                cases.push(Case {
                    condition: "gray backdrop with black/white source",
                    mode: row.mode,
                    source: [endpoint, endpoint, endpoint, 1.0],
                    backdrop: [gray, gray, gray, 1.0],
                    expected: if row.mode == BlendMode::Luminosity {
                        [endpoint, endpoint, endpoint, 1.0]
                    } else {
                        [gray, gray, gray, 1.0]
                    },
                });
            }
        }
    }
    // These are valid public colors: SetLum shifts (0,1/2,1) to
    // (-1/5,3/10,4/5) or (1/5,7/10,6/5), then clips jointly.
    for (condition, luminosity, expected) in [
        (
            "negative clipping",
            41.0 / 200.0,
            [0.0, 41.0 / 162.0, 41.0 / 81.0, 1.0],
        ),
        (
            "upper clipping",
            121.0 / 200.0,
            [40.0 / 119.0, 159.0 / 238.0, 1.0, 1.0],
        ),
    ] {
        cases.push(Case {
            condition,
            mode: BlendMode::Color,
            source: [0.0, 0.5, 1.0, 1.0],
            backdrop: [luminosity, luminosity, luminosity, 1.0],
            expected,
        });
    }
    cases
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn nonseparable_graph_shader_preserves_joint_rgb_roles_and_partial_alpha() {
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
    // The real shader-vector harness also checks finite bounded premultiplied
    // output; its 3/255 tolerance accounts for working texture storage.
    super::gpu::assert_composition_blend_gpu_vectors(
        backend, identity, &requests, &vectors, &expected,
    );
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

#[test]
fn nonseparable_public_direct_and_masked_graph_match_rational_constants() {
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(16.0, 2.0), 1.0)).unwrap();
    // This existing helper asserts a nonempty real device-supported set.
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    for batch in cases().chunks(8) {
        let direct = render_pixels(
            &mut renderer,
            &mut surface,
            &scene_for_cases(batch, false),
            None,
        );
        for (index, case) in batch.iter().enumerate() {
            assert_near(
                pixel_rgba(&direct, u32::try_from(index).unwrap() * 2, 0),
                straight_bytes(case.expected),
                3,
                &format!("direct {} {case:?}", case.condition),
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
                let x = u32::try_from(index).unwrap() * 2;
                let actual = pixel_rgba(&graph, x, 0);
                // Capture, graph storage and final straight conversion round.
                assert_near(
                    actual,
                    straight_bytes(case.expected),
                    4,
                    &format!("graph {format:?} {} {case:?}", case.condition),
                );
                assert_near(
                    actual,
                    pixel_rgba(&direct, x, 0),
                    4,
                    &format!("route parity {format:?} {} {case:?}", case.condition),
                );
            }
        }
    }
}

#[test]
fn nonseparable_direct_gray_endpoints_are_stable_for_every_antialiasing_policy() {
    let mut endpoints = Vec::new();
    for mode in [
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
    ] {
        for gray in [29.0 / 255.0, 153.0 / 255.0] {
            for endpoint in [0.0, 1.0] {
                endpoints.push(Case {
                    condition: "gray backdrop with black/white source",
                    mode,
                    source: [endpoint, endpoint, endpoint, 1.0],
                    backdrop: [gray, gray, gray, 1.0],
                    expected: if mode == BlendMode::Luminosity {
                        [endpoint, endpoint, endpoint, 1.0]
                    } else {
                        [gray, gray, gray, 1.0]
                    },
                });
            }
        }
    }
    for antialiasing in [
        Antialiasing::Area,
        Antialiasing::Msaa8,
        Antialiasing::Msaa16,
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default().with_antialiasing(antialiasing),
        ))
        .unwrap();
        let mut surface =
            pollster::block_on(renderer.create_headless(Size::new(16.0, 2.0), 1.0)).unwrap();
        for batch in endpoints.chunks(8) {
            // The common public-render helper asserts DirectVello, one Vello
            // pass, no graph composites, and one atomic pixel/stat publication.
            let output = render_pixels(
                &mut renderer,
                &mut surface,
                &scene_for_cases(batch, false),
                None,
            );
            for (index, case) in batch.iter().enumerate() {
                let actual = pixel_rgba(&output, u32::try_from(index).unwrap() * 2, 0);
                assert_near(
                    actual,
                    straight_bytes(case.expected),
                    3,
                    &format!("direct endpoints {antialiasing:?} {case:?}"),
                );
                assert_eq!(actual[3], 255, "opaque endpoints {antialiasing:?} {case:?}");
            }
        }
    }
}

#[test]
fn nonseparable_fractional_mask_opacity_and_clip_preserve_straight_blend_inputs() {
    // Effective as=64/255, ab=1/2, ao=319/510. Before byte rounding the
    // straight output is (64*Cs+64*B+191*Cb)/319, from source-over expansion.
    let expected = [
        (BlendMode::Hue, [129, 141, 135, 160]),
        (BlendMode::Saturation, [87, 164, 126, 160]),
        (BlendMode::Color, [134, 138, 136, 160]),
        (BlendMode::Luminosity, [86, 150, 118, 160]),
    ];
    let mut scene = Scene::new();
    for (index, (mode, _)) in expected.iter().enumerate() {
        let rect = tile(index);
        scene.fill(rect, straight_color(HALF_BACKDROP));
        scene.layer(
            masked_layer(*mode, rect, 128)
                .try_opacity(0.5)
                .unwrap()
                .try_clip(Shape::rect(rect))
                .unwrap(),
            |scene| {
                scene.fill(rect, straight_color(SOURCE));
            },
        );
    }
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(8.0, 2.0), 1.0)).unwrap();
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
fn nonseparable_capture_local_hue_and_external_color_compose_in_graph_order() {
    let bounds = Rect::new(0.0, 0.0, 2.0, 2.0);
    let mut scene = Scene::new();
    scene.fill(bounds, straight_color(GRAY));
    scene.layer(masked_layer(BlendMode::Color, bounds, 255), |scene| {
        scene.fill(bounds, straight_color(BACKDROP));
        scene.layer(Layer::new().blend(BlendMode::Hue), |scene| {
            scene.fill(bounds, straight_color(SOURCE));
        });
    });
    // Inner Hue=(179/200,79/200,129/200), luminosity229/400.
    // Outer Color shifts it by -29/400 to the gray backdrop luminosity1/2:
    // (329/400,129/400,229/400), distinct from either uncomposited source.
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
            [210, 82, 146, 255],
            3,
            &format!("nested {format:?}"),
        );
        assert!(renderer.stats().vello_passes >= 2);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn nonseparable_gpu_helpers_preserve_original_extrema_and_channel_identity() {
    use crate::backend::NonseparableHelperInputForTest::{ClipColor, SetLum, SetSat};
    let mut fixtures = vec![
        (
            ClipColor([-1.0 / 5.0, 3.0 / 10.0, 4.0 / 5.0]),
            [0.0, 41.0 / 162.0, 41.0 / 81.0],
        ),
        (
            ClipColor([1.0 / 5.0, 7.0 / 10.0, 6.0 / 5.0]),
            [40.0 / 119.0, 159.0 / 238.0, 1.0],
        ),
        // This span>1 is a synthetic private helper input, never a public Color.
        // L=31/100,n=-1/2,x=3/2 are snapshotted once. The lower branch yields
        // (0,31/81,62/81); the upper branch must then use ORIGINAL L and x.
        (
            ClipColor([-0.5, 0.5, 1.5]),
            [31.0 / 238.0, 2263.0 / 6426.0, 31.0 / 54.0],
        ),
        (
            SetLum {
                color: [0.0, 0.5, 1.0],
                luminosity: 41.0 / 200.0,
            },
            [0.0, 41.0 / 162.0, 41.0 / 81.0],
        ),
        (
            SetLum {
                color: [0.0, 0.5, 1.0],
                luminosity: 121.0 / 200.0,
            },
            [40.0 / 119.0, 159.0 / 238.0, 1.0],
        ),
        (
            SetLum {
                color: [1.0, 0.0, 0.0],
                luminosity: 3.0 / 10.0,
            },
            [1.0, 0.0, 0.0],
        ),
        (
            SetLum {
                color: [0.0, 1.0, 0.0],
                luminosity: 59.0 / 100.0,
            },
            [0.0, 1.0, 0.0],
        ),
        (
            SetLum {
                color: [0.0, 0.0, 1.0],
                luminosity: 11.0 / 100.0,
            },
            [0.0, 0.0, 1.0],
        ),
        // Both increments are exactly representable f32 values. This is not
        // gray: an epsilon shortcut would incorrectly erase its saturation.
        (
            SetSat {
                color: [0.5, 0.5 + 1.0 / 4_194_304.0, 0.5 + 1.0 / 2_097_152.0],
                saturation: 0.5,
            },
            [0.0, 0.25, 0.5],
        ),
    ];
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for permutation in permutations {
        let color = permutation.map(|index| [1.0 / 8.0, 3.0 / 8.0, 7.0 / 8.0][index]);
        let expected = permutation.map(|index| [0.0, 1.0 / 6.0, 1.0 / 2.0][index]);
        fixtures.push((
            SetSat {
                color,
                saturation: 0.5,
            },
            expected,
        ));
    }
    for permutation in [[0, 1, 2], [1, 2, 0], [2, 0, 1]] {
        for (color, expected) in [
            ([0.25, 0.25, 0.75], [0.0, 0.0, 0.5]),
            ([0.25, 0.75, 0.75], [0.0, 0.5, 0.5]),
        ] {
            fixtures.push((
                SetSat {
                    color: permutation.map(|index| color[index]),
                    saturation: 0.5,
                },
                permutation.map(|index| expected[index]),
            ));
        }
    }
    for amount in [0.0, 0.5, 1.0] {
        fixtures.push((
            SetSat {
                color: [0.5; 3],
                saturation: amount,
            },
            [0.0; 3],
        ));
        fixtures.push((
            SetLum {
                color: [0.5; 3],
                luminosity: amount,
            },
            [amount; 3],
        ));
    }
    // Reachable valid-domain inputs, distinct from the synthetic ClipColor
    // fixture: gray SetLum must remain finite at both luminosity endpoints.
    for gray in [29.0 / 255.0, 153.0 / 255.0] {
        for luminosity in [0.0, 1.0] {
            fixtures.push((
                SetLum {
                    color: [gray; 3],
                    luminosity,
                },
                [luminosity; 3],
            ));
        }
    }
    fixtures.push((
        SetLum {
            color: [1.0e-39, 0.0, 0.0],
            luminosity: 0.0,
        },
        [0.0; 3],
    ));
    let (mut backend, identity, _) =
        super::gpu::composition_selected_backend_and_requests_for_test();
    let inputs = fixtures.iter().map(|(input, _)| *input).collect::<Vec<_>>();
    let observed =
        pollster::block_on(backend.nonseparable_shader_helpers_for_test(identity, &inputs))
            .unwrap();
    assert_eq!(observed.len(), fixtures.len());
    // Raw storage f32 readback bypasses float16/RGBA8 quantization. The small
    // absolute tolerance covers this bounded chain of f32 arithmetic only.
    let tolerance = 2.0e-6;
    for (actual, (input, expected)) in observed.iter().zip(&fixtures) {
        assert!(
            actual.iter().all(|channel| channel.is_finite()
                && *channel >= -tolerance
                && *channel <= 1.0 + tolerance),
            "helper {input:?}: nonfinite/unbounded {actual:?}"
        );
        assert!(
            actual
                .iter()
                .zip(expected)
                .all(|(a, e)| (a - e).abs() <= tolerance),
            "helper {input:?}: actual={actual:?}, expected={expected:?}"
        );
    }
}
