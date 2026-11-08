#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]

use surgeist_render::{
    Color, EffectQualityPolicy, Gradient, GradientStop, Image, ImageBuffer, Layer, Options,
    Parameters, Point, Rect, RenderRoute, Renderer, ResolvedLayerAlphaMask, Scene, Size, Surface,
};

fn point(x: f64, y: f64) -> Point {
    Point::try_new(x, y).unwrap()
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect::try_new(x, y, width, height).unwrap()
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::try_rgba(r, g, b, a).unwrap()
}

fn stop(offset: f32, color: Color) -> GradientStop {
    GradientStop::try_new(f64::from(offset), color).unwrap()
}

fn renderer() -> Renderer {
    pollster::block_on(Renderer::new(
        Options::default().with_effect_quality_policy(EffectQualityPolicy::AllowReducedPrecision),
    ))
    .unwrap()
}

fn surface(renderer: &mut Renderer) -> Surface {
    pollster::block_on(renderer.create_headless(Size::try_new(16.0, 8.0).unwrap(), 1.0)).unwrap()
}

fn scene(graph: bool, draw: impl FnOnce(&mut Scene)) -> Scene {
    let mut scene = Scene::new();
    if graph {
        let mask =
            Image::from_rgba(Size::try_new(1.0, 1.0).unwrap(), vec![255, 255, 255, 128]).unwrap();
        scene.layer(
            Layer::new().with_resolved_alpha_mask(
                ResolvedLayerAlphaMask::try_new(mask, rect(0.0, 0.0, 16.0, 8.0)).unwrap(),
            ),
            draw,
        );
    } else {
        draw(&mut scene);
    }
    scene
}

fn read(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene, graph: bool) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(
        stats.route,
        Some(if graph {
            RenderRoute::GpuGraph
        } else {
            RenderRoute::DirectVello
        })
    );
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn assert_pixel(
    image: &ImageBuffer,
    x: u32,
    y: u32,
    expected: [u8; 4],
    tolerance: u8,
    context: &str,
) {
    let index = usize::try_from((y * image.size().width() + x) * 4).unwrap();
    let actual: [u8; 4] = image.rgba()[index..index + 4].try_into().unwrap();
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
        "{context}, pixel ({x},{y}): actual {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

#[derive(Clone, Copy, Debug)]
enum Field {
    Linear,
    Radial,
    Sweep,
}

#[test]
fn terminal_coincident_stops_keep_the_final_supplied_color_at_and_beyond_the_endpoint() {
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer);
    // Twelve frames share one Renderer and one 128-pixel surface. The sweep's
    // final position is .5 so an exterior field coordinate remains reachable.
    for (alpha, final_color, midpoint, masked_final, masked_midpoint) in [
        (
            1.0,
            [0, 255, 0, 255],
            [128, 0, 128, 255],
            [0, 255, 0, 128],
            [128, 0, 128, 128],
        ),
        (
            0.5,
            [0, 255, 0, 128],
            [85, 0, 170, 191],
            [0, 255, 0, 64],
            [85, 0, 170, 96],
        ),
    ] {
        for field in [Field::Linear, Field::Radial, Field::Sweep] {
            for graph in [false, true] {
                let final_offset = if matches!(field, Field::Sweep) {
                    0.5
                } else {
                    1.0
                };
                let stops = vec![
                    stop(0.0, rgba(1.0, 0.0, 0.0, alpha)),
                    stop(final_offset, rgba(0.0, 0.0, 1.0, 1.0)),
                    stop(final_offset, rgba(0.0, 1.0, 0.0, alpha)),
                ];
                let gradient = match field {
                    Field::Linear => Gradient::try_linear(point(0.0, 0.0), point(8.0, 0.0), stops),
                    Field::Radial => Gradient::try_radial(point(0.0, 4.0), 8.0, 8.0, stops),
                    Field::Sweep => Gradient::try_sweep(point(8.0, 4.0), stops),
                }
                .unwrap();
                let scene = scene(graph, |scene| {
                    scene.fill(rect(0.0, 0.0, 16.0, 8.0), gradient);
                });
                let image = read(&mut renderer, &mut surface, &scene, graph);
                // Native colors sample integer physical indices. Linear probes
                // have raw coordinates 1.25,1,.5; sweep probes .75,.5,.25.
                // Radial probes use those logical distance ratios, subject to
                // the retained native circle-coordinate approximation.
                let (beyond, endpoint, interior) = match field {
                    Field::Linear => ((10, 2), (8, 2), (4, 2)),
                    Field::Radial => ((10, 4), (8, 4), (4, 4)),
                    Field::Sweep => ((8, 2), (6, 4), (8, 6)),
                };
                let expected_final = if graph { masked_final } else { final_color };
                let context = format!("{field:?}, alpha={alpha}, graph={graph}");
                // Images3 exterior colors and the adopted native endpoint
                // equality policy select the last supplied green, not blue.
                assert_pixel(
                    &image,
                    beyond.0,
                    beyond.1,
                    expected_final,
                    0,
                    &format!("{context}, exterior"),
                );
                // The native circle calculation can place its logical radius
                // just below raw 1. Exact equality is covered by the shared GPU
                // endpoint helper; this public probe retains native geometry.
                if !matches!(field, Field::Radial) {
                    assert_pixel(
                        &image,
                        endpoint.0,
                        endpoint.1,
                        expected_final,
                        0,
                        &format!("{context}, endpoint equality"),
                    );
                }
                // The retained nonzero interval has its existing RGBA8 ramp:
                // halfway premul (.25,0,.5,.75) is straight (1/3,0,2/3,.75).
                assert_pixel(
                    &image,
                    interior.0,
                    interior.1,
                    if graph { masked_midpoint } else { midpoint },
                    if graph { 3 } else { 2 },
                    &format!("{context}, interval midpoint"),
                );
            }
        }
    }
}

#[test]
fn all_coincident_stops_are_true_steps_at_exact_near_quantum_and_subnormal_thresholds() {
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer);
    let quarter = 0.25_f32;
    // Each row is a distinct field. Shifted lines expose raw negative values
    // before threshold zero; unshifted 0->4 lines sample exactly .25 at x=1.
    let cases = [
        (0.0, 4.0, [(2, false), (4, true), (6, true)]),
        (0.5, 4.0, [(5, false), (6, true), (7, true)]),
        (1.0, 4.0, [(7, false), (8, true), (10, true)]),
        (
            f32::from_bits(quarter.to_bits() - 1),
            0.0,
            [(0, false), (1, true), (2, true)],
        ),
        (quarter, 0.0, [(0, false), (1, true), (2, true)]),
        (
            f32::from_bits(quarter.to_bits() + 1),
            0.0,
            [(0, false), (1, false), (2, true)],
        ),
        (f32::from_bits(1), 0.0, [(0, false), (1, true), (2, true)]),
    ];
    // Four frames exercise opaque and independent endpoint alphas on both
    // routes. A finite ramp texel, average or FTZ threshold cannot fit the rows.
    for (first_alpha, last_alpha, first, last, masked_first, masked_last) in [
        (
            1.0,
            1.0,
            [255, 0, 0, 255],
            [0, 0, 255, 255],
            [255, 0, 0, 128],
            [0, 0, 255, 128],
        ),
        (
            0.5,
            0.25,
            [255, 0, 0, 128],
            [0, 0, 255, 64],
            [255, 0, 0, 64],
            [0, 0, 255, 32],
        ),
    ] {
        for graph in [false, true] {
            let scene = scene(graph, |scene| {
                for (row, &(threshold, start, _)) in cases.iter().enumerate() {
                    let stops = vec![
                        stop(threshold, rgba(1.0, 0.0, 0.0, first_alpha)),
                        stop(threshold, rgba(0.0, 1.0, 0.0, 1.0)),
                        stop(threshold, rgba(0.0, 0.0, 1.0, last_alpha)),
                    ];
                    let gradient =
                        Gradient::try_linear(point(start, 0.0), point(start + 4.0, 0.0), stops)
                            .unwrap();
                    scene.fill(
                        rect(0.0, f64::from(u32::try_from(row).unwrap()), 12.0, 1.0),
                        gradient,
                    );
                }
                // One stop remains solid even with coincident line endpoints;
                // it must not acquire unused step/gradient geometry.
                let solid = Gradient::try_linear(
                    point(14.0, 4.0),
                    point(14.0, 4.0),
                    vec![stop(0.75, rgba(1.0, 0.0, 0.0, first_alpha))],
                )
                .unwrap();
                scene.fill(rect(12.0, 0.0, 4.0, 8.0), solid);
            });
            let image = read(&mut renderer, &mut surface, &scene, graph);
            for (row, &(threshold, _, probes)) in cases.iter().enumerate() {
                for (x, last_side) in probes {
                    let expected = if graph {
                        if last_side { masked_last } else { masked_first }
                    } else if last_side {
                        last
                    } else {
                        first
                    };
                    assert_pixel(
                        &image,
                        x,
                        u32::try_from(row).unwrap(),
                        expected,
                        0,
                        &format!(
                            "threshold bits={:#010x}, last_side={last_side}, graph={graph}",
                            threshold.to_bits()
                        ),
                    );
                }
            }
            assert_pixel(
                &image,
                14,
                3,
                if graph { masked_first } else { first },
                0,
                "one-stop coincident-line control",
            );
            assert_pixel(
                &image,
                14,
                7,
                if graph { masked_first } else { first },
                0,
                "one-stop coincident-line control second row",
            );
        }
    }
}
