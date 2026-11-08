#![cfg(not(target_arch = "wasm32"))]

use super::support::{
    AHEM_FONT_BYTES, AHEM_GLYPH_X, ahem_font, graph_supported_working_formats_for_test, pixel_rgba,
};
use crate::{
    Color, ErrorCode, FontData, GlyphRunLimits, GlyphSynthesis, Image, ImageBuffer, Layer, Options,
    Paint, Parameters, Point, Rect, RenderAccountingError, RenderAccountingOptions,
    RenderAttemptStatus, RenderLimits, RenderPhase, RenderProfile, RenderQuantity, RenderRoute,
    RenderWork, Renderer, RepeatingGradient, RepeatingGradientStop, ResolvedLayerAlphaMask, Scene,
    Shape, Size, Stroke, Surface, TextGlyph, TextInstance, TextPaint, TextRun, TextRunBounds,
    Transform, resource::WorkingFormat,
};

fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::try_rgba(r, g, b, a).unwrap()
}

fn red() -> Color {
    rgba(1.0, 0.0, 0.0, 1.0)
}

fn blue() -> Color {
    rgba(0.0, 0.0, 1.0, 1.0)
}

fn stops(values: &[(f64, Color)]) -> Vec<RepeatingGradientStop> {
    values
        .iter()
        .map(|&(position, color)| RepeatingGradientStop::try_new(position, color).unwrap())
        .collect()
}

fn linear(length: f64, values: &[(f64, Color)]) -> RepeatingGradient {
    RepeatingGradient::try_linear(Point::new(0.0, 0.0), Point::new(length, 0.0), stops(values))
        .unwrap()
}

fn renderer() -> Renderer {
    pollster::block_on(Renderer::new(Options::default())).unwrap()
}

fn surface(renderer: &mut Renderer, width: f64, height: f64, scale: f64) -> Surface {
    pollster::block_on(renderer.create_headless(Size::new(width, height), scale)).unwrap()
}

fn render(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(stats.route, Some(RenderRoute::DirectVello));
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}

fn assert_pixel(image: &ImageBuffer, x: u32, y: u32, expected: [u8; 4], tolerance: u8) {
    let actual = pixel_rgba(image, x, y);
    assert!(
        actual
            .into_iter()
            .zip(expected)
            .all(|(actual, expected)| actual.abs_diff(expected) <= tolerance),
        "pixel ({x}, {y}): {actual:?}, expected {expected:?}, tolerance {tolerance}"
    );
}

fn solid_scene(paint: impl Into<Paint>) -> Scene {
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 4.0, 4.0), paint);
    scene
}

// The pinned Images 3 average uses trapezoidal premultiplied-sRGBA weights.
// These byte literals are independently derived in the design audit. Solid
// readback permits two levels for RGBA8 premultiplication/unpremultiplication.
#[test]
fn zero_span_and_coincident_geometry_use_uniform_original_color_weights() {
    let green = rgba(0.0, 1.0, 0.0, 1.0);
    let white = rgba(1.0, 1.0, 1.0, 1.0);
    let minimum = f64::from_bits(1);
    // Distinct, ordered f64 stops survive validation, but their whole logical
    // period axis truly rounds to zero. This is not a device-density epsilon.
    assert_eq!(0.1 * (4.0 * minimum), 0.0);
    let cases = [
        (
            linear(40.0, &[(7.0, red()), (7.0, white), (7.0, blue())]),
            [191, 128, 191, 255],
        ),
        (
            linear(
                40.0,
                &[(2.0, red()), (2.0, green), (2.0, blue()), (2.0, white)],
            ),
            [85, 128, 128, 255],
        ),
        (
            linear(0.0, &[(0.0, red()), (1.0, white), (4.0, blue())]),
            [191, 128, 191, 255],
        ),
        (
            linear(0.1, &[(0.0, red()), (1.0, white), (4.0, blue())]),
            [159, 128, 223, 255],
        ),
        (
            linear(
                0.1,
                &[(0.0, red()), (minimum, white), (4.0 * minimum, blue())],
            ),
            [191, 128, 191, 255],
        ),
        (linear(0.0, &[(123.0, green)]), [0, 255, 0, 255]),
    ];
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 4.0, 4.0, 1.0);
    for (gradient, expected) in cases {
        let image = render(&mut renderer, &mut surface, &solid_scene(gradient));
        for y in 0..4 {
            for x in 0..4 {
                assert_pixel(&image, x, y, expected, 2);
            }
        }
    }
}

#[test]
fn positive_unresolved_periods_keep_segment_weights_and_premultiplied_alpha() {
    let green = rgba(0.0, 1.0, 0.0, 1.0);
    let white = rgba(1.0, 1.0, 1.0, 1.0);
    let cases = [
        (
            linear(
                0.1,
                &[
                    (0.0, red()),
                    (1.0, rgba(0.0, 1.0, 0.0, 0.5)),
                    (4.0, rgba(0.0, 0.0, 1.0, 0.0)),
                ],
            ),
            [85, 170, 0, 96],
        ),
        (
            linear(
                0.1,
                &[(0.0, red()), (1.0, green), (1.0, blue()), (4.0, white)],
            ),
            [128, 128, 191, 255],
        ),
        (
            linear(0.1, &[(0.0, rgba(1.0, 0.0, 0.0, 0.0)), (1.0, blue())]),
            [0, 0, 255, 128],
        ),
        (
            linear(
                0.1,
                &[
                    (0.0, rgba(1.0, 0.0, 0.0, 0.0)),
                    (1.0, rgba(0.0, 0.0, 1.0, 0.0)),
                ],
            ),
            [0, 0, 0, 0],
        ),
    ];
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 4.0, 4.0, 1.0);
    for (gradient, expected) in cases {
        let image = render(&mut renderer, &mut surface, &solid_scene(gradient));
        assert_pixel(&image, 1, 1, expected, 2);
        assert_pixel(&image, 2, 2, expected, 2);
    }
}

#[test]
fn selected_zero_height_radial_averages_but_zero_width_is_invalid() {
    let values = [(0.0, red()), (1.0, rgba(1.0, 1.0, 1.0, 1.0)), (4.0, blue())];
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 4.0, 4.0, 1.0);
    for (values, expected) in [
        (values.to_vec(), [159, 128, 223, 255]),
        (
            vec![(3.0, red()), (3.0, rgba(1.0, 1.0, 1.0, 1.0)), (3.0, blue())],
            [191, 128, 191, 255],
        ),
    ] {
        let gradient =
            RepeatingGradient::try_radial(Point::new(1.0, 1.0), 40.0, 0.0, stops(&values)).unwrap();
        let image = render(&mut renderer, &mut surface, &solid_scene(gradient));
        assert_pixel(&image, 1, 1, expected, 2);
    }
    for height in [0.0, 40.0] {
        let error =
            RepeatingGradient::try_radial(Point::new(1.0, 1.0), 0.0, height, stops(&values))
                .unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
    }
}

#[test]
fn linear_cycles_repeat_both_directions_with_sharp_joins_and_outside_unit_stops() {
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 128.0, 4.0, 1.0);
    let mut scene = Scene::new();
    scene.fill(
        Rect::new(0.0, 0.0, 128.0, 2.0),
        RepeatingGradient::try_linear(
            Point::new(50.0, 0.0),
            Point::new(90.0, 0.0),
            stops(&[(0.0, red()), (1.0, blue())]),
        )
        .unwrap(),
    );
    scene.fill(
        Rect::new(0.0, 2.0, 128.0, 2.0),
        RepeatingGradient::try_linear(
            Point::new(0.0, 0.0),
            Point::new(20.0, 0.0),
            stops(&[(-0.5, red()), (1.5, blue())]),
        )
        .unwrap(),
    );
    let image = render(&mut renderer, &mut surface, &scene);
    // Gradient samples use integer device coordinates. Each seam-side probe is
    // 1/40 of a cycle from the join, over twelve ramp samples away from it.
    for x in [30, 70, 110] {
        assert_pixel(&image, x, 0, [128, 0, 128, 255], 3);
    }
    for seam in [10, 50, 90] {
        assert_pixel(&image, seam - 1, 0, [6, 0, 249, 255], 3);
        assert_pixel(&image, seam + 1, 0, [249, 0, 6, 255], 3);
    }
    for x in [10, 50, 90] {
        assert_pixel(&image, x, 2, [128, 0, 128, 255], 3);
    }
    for seam in [30, 70, 110] {
        assert_pixel(&image, seam - 1, 2, [6, 0, 249, 255], 3);
        assert_pixel(&image, seam + 1, 2, [249, 0, 6, 255], 3);
    }
}

#[test]
fn radial_negative_phase_preserves_concentric_elliptical_cycles() {
    let gradient = RepeatingGradient::try_radial(
        Point::new(64.0, 32.0),
        40.0,
        20.0,
        stops(&[(-0.25, red()), (0.75, blue())]),
    )
    .unwrap();
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 128.0, 64.0), gradient);
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 128.0, 64.0, 1.0);
    let image = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&image, 64, 32, [191, 0, 64, 255], 3);
    for (x, y) in [
        (74, 32),
        (54, 32),
        (64, 37),
        (64, 27),
        (114, 32),
        (14, 32),
        (64, 57),
        (64, 7),
    ] {
        assert_pixel(&image, x, y, [128, 0, 128, 255], 3);
    }
    for (x, y, expected) in [
        (92, 32, [13, 0, 242, 255]),
        (96, 32, [242, 0, 13, 255]),
        (64, 46, [13, 0, 242, 255]),
        (64, 48, [242, 0, 13, 255]),
        // (24/40)^2 + (8/20)^2 = .52; phase sqrt(.52)+.25.
        (88, 40, [7, 0, 248, 255]),
        (40, 24, [7, 0, 248, 255]),
    ] {
        assert_pixel(&image, x, y, expected, 3);
    }
}

fn magnified_scene(gradient: &RepeatingGradient, scale: f64, bounds: Rect) -> Scene {
    let mut scene = Scene::new();
    scene.transform(Transform::scale(scale, 1.0).unwrap(), |scene| {
        scene.fill(bounds, gradient.clone());
    });
    scene
}

fn assert_twenty_pixel_cycle(image: &ImageBuffer, y: u32, tolerance: u8) {
    // Native gradient samples use integer device coordinates: x=2 is phase
    // 1/10, and x=17 is phase 17/20. The pixel's coverage area is separate.
    for (x, expected) in [
        (2, [230, 0, 26, 255]),
        (17, [38, 0, 217, 255]),
        (22, [230, 0, 26, 255]),
        (37, [38, 0, 217, 255]),
        (42, [230, 0, 26, 255]),
        (57, [38, 0, 217, 255]),
    ] {
        assert_pixel(image, x, y, expected, tolerance);
    }
}

#[test]
fn same_tiny_input_is_averaged_or_resolved_per_transform_and_surface_scale() {
    let gradient = linear(0.2, &[(0.0, red()), (1.0, blue())]);
    let mut renderer = renderer();
    let mut regular = surface(&mut renderer, 64.0, 8.0, 1.0);
    let unresolved = magnified_scene(&gradient, 1.0, Rect::new(0.0, 0.0, 64.0, 8.0));
    let image = render(&mut renderer, &mut regular, &unresolved);
    assert_pixel(&image, 2, 3, [128, 0, 128, 255], 2);
    let resolved = magnified_scene(&gradient, 100.0, Rect::new(0.0, 0.0, 0.64, 8.0));
    assert_twenty_pixel_cycle(&render(&mut renderer, &mut regular, &resolved), 3, 3);
    let mut dense = surface(&mut renderer, 0.64, 0.08, 100.0);
    let scaled_surface_scene = magnified_scene(&gradient, 1.0, Rect::new(0.0, 0.0, 0.64, 0.08));
    assert_twenty_pixel_cycle(
        &render(&mut renderer, &mut dense, &scaled_surface_scene),
        3,
        3,
    );
    // Reusing the exact logical value at its original output density must not
    // inherit a cached decision from either magnified use.
    let image = render(&mut renderer, &mut regular, &unresolved);
    assert_pixel(&image, 17, 3, [128, 0, 128, 255], 2);
}

fn bands(period: f64) -> RepeatingGradient {
    linear(
        period,
        &[(0.0, red()), (0.5, red()), (0.5, blue()), (1.0, blue())],
    )
}

#[test]
fn linear_two_pixel_threshold_uses_inverse_transpose_spacing_under_shear() {
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 16.0, 4.0, 1.0);
    for (period, shear, expected) in [
        (1.99, 0.0, [[128, 0, 128, 255]; 3]),
        (
            2.0,
            0.0,
            [[255, 0, 0, 255], [0, 0, 255, 255], [255, 0, 0, 255]],
        ),
        // A = [[1,3/4],[0,1]] gives physical spacing 4P/5.
        (2.4, 0.75, [[128, 0, 128, 255]; 3]),
        (
            2.5,
            0.75,
            [[255, 0, 0, 255], [255, 0, 0, 255], [0, 0, 255, 255]],
        ),
    ] {
        let mut scene = Scene::new();
        scene.transform(
            Transform::try_new([1.0, 0.0, shear, 1.0, 0.0, 0.0]).unwrap(),
            |scene| {
                scene.fill(Rect::new(-8.0, -8.0, 40.0, 20.0), bands(period));
            },
        );
        let image = render(&mut renderer, &mut surface, &scene);
        for (x, expected) in expected.into_iter().enumerate() {
            assert_pixel(&image, x as u32, 0, expected, 2);
        }
        if period == 2.5 && shear == 0.75 {
            // Integer device (x,y) maps to local x - 3y/4. At y=1,
            // x=0,1,3 therefore have phases .7,.1,.9. These samples
            // distinguish the field normal from transformed endpoint length.
            assert_pixel(&image, 0, 1, [0, 0, 255, 255], 2);
            assert_pixel(&image, 1, 1, [255, 0, 0, 255], 2);
            assert_pixel(&image, 3, 1, [0, 0, 255, 255], 2);
        }
    }
}

#[test]
fn radial_any_unresolved_axis_averages_and_exact_two_pixel_axis_repeats() {
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 32.0, 8.0, 1.0);
    for (radius_y, transform, expected) in [
        (1.99, Transform::identity(), [128, 0, 128, 255]),
        (2.0, Transform::identity(), [204, 0, 51, 255]),
        (
            10.0,
            Transform::scale(2.0, 0.1).unwrap(),
            [128, 0, 128, 255],
        ),
    ] {
        let gradient = RepeatingGradient::try_radial(
            Point::new(0.0, 0.0),
            10.0,
            radius_y,
            stops(&[(0.0, red()), (1.0, blue())]),
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.transform(transform, |scene| {
            scene.fill(Rect::new(-8.0, -8.0, 64.0, 128.0), gradient);
        });
        assert_pixel(
            &render(&mut renderer, &mut surface, &scene),
            2,
            0,
            expected,
            3,
        );
    }
}

fn opaque_mask(bounds: Rect) -> Layer {
    Layer::new().with_resolved_alpha_mask(
        ResolvedLayerAlphaMask::try_new(
            Image::from_rgba(Size::new(1.0, 1.0), vec![255; 4]).unwrap(),
            bounds,
        )
        .unwrap(),
    )
}

#[test]
fn tiny_radial_geometry_keeps_negative_phase_after_final_magnification_and_mask_capture() {
    // The untransformed radial period is below the backend's native radius
    // epsilon. Its final period axes are 10 and 5 pixels, so neither averaging
    // nor transparent native-radius collapse is permitted.
    let gradient = RepeatingGradient::try_radial(
        Point::new(1.6e-6, 0.8e-6),
        1.0e-6,
        0.5e-6,
        stops(&[(-0.25, red()), (0.75, blue())]),
    )
    .unwrap();
    let bounds = Rect::new(0.0, 0.0, 3.2e-6, 1.6e-6);
    let mut direct = Scene::new();
    direct.transform(Transform::scale(1.0e7, 1.0e7).unwrap(), |scene| {
        scene.fill(bounds, gradient.clone());
    });
    let mut masked = Scene::new();
    masked.transform(Transform::scale(1.0e7, 1.0e7).unwrap(), |scene| {
        scene.layer(opaque_mask(bounds), |scene| {
            scene.fill(bounds, gradient.clone());
        });
    });
    let mut renderer = renderer();
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let mut surface = surface(&mut renderer, 32.0, 16.0, 1.0);
    let image = render(&mut renderer, &mut surface, &direct);
    for (x, y, expected) in [
        (16, 8, [191, 0, 64, 255]),
        (21, 8, [64, 0, 191, 255]),
        (11, 8, [64, 0, 191, 255]),
        (28, 8, [140, 0, 115, 255]),
        (16, 11, [38, 0, 217, 255]),
    ] {
        assert_pixel(&image, x, y, expected, 3);
    }
    for format in formats {
        let stats = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &masked,
            Parameters::default(),
            format,
        ))
        .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for (x, y, expected) in [
            (16, 8, [191, 0, 64, 255]),
            (21, 8, [64, 0, 191, 255]),
            (11, 8, [64, 0, 191, 255]),
            (28, 8, [140, 0, 115, 255]),
            (16, 11, [38, 0, 217, 255]),
        ] {
            assert_pixel(&image, x, y, expected, 4);
        }
    }
}

#[test]
fn large_resolved_stop_positions_retain_period_and_phase_without_absolute_narrowing() {
    let gradient = linear(20.0, &[(1.0e12, red()), (1.0e12 + 1.0, blue())]);
    let mut scene = Scene::new();
    scene.fill(Rect::new(0.0, 0.0, 64.0, 4.0), gradient);
    let mut renderer = renderer();
    let mut surface = surface(&mut renderer, 64.0, 4.0, 1.0);
    assert_twenty_pixel_cycle(&render(&mut renderer, &mut surface, &scene), 1, 3);
}

#[test]
fn nested_opaque_mask_captures_preserve_final_output_magnified_cycles_in_all_formats() {
    let gradient = linear(0.2, &[(0.0, red()), (1.0, blue())]);
    let bounds = Rect::new(0.0, 0.0, 0.64, 8.0);
    let direct = magnified_scene(&gradient, 100.0, bounds);
    let mut nested = Scene::new();
    nested.transform(Transform::scale(10.0, 1.0).unwrap(), |scene| {
        scene.transform(Transform::scale(10.0, 1.0).unwrap(), |scene| {
            scene.layer(opaque_mask(bounds), |scene| {
                scene.layer(opaque_mask(bounds), |scene| {
                    scene.fill(bounds, gradient.clone());
                });
            });
        });
    });
    let mut renderer = renderer();
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let mut surface = surface(&mut renderer, 64.0, 8.0, 1.0);
    let direct_image = render(&mut renderer, &mut surface, &direct);
    assert_twenty_pixel_cycle(&direct_image, 3, 3);
    for format in formats {
        let stats = pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &nested,
            Parameters::default(),
            format,
        ))
        .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::GpuGraph));
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        // A ramp channel has <1 byte sampling error; each RGBA8 mask/capture
        // adds at most one rounding level. Probe interiors, never softened joins.
        assert_twenty_pixel_cycle(&image, 3, 4);
        for x in [2, 17, 22, 37, 42, 57] {
            assert_pixel(&image, x, 3, pixel_rgba(&direct_image, x, 3), 3);
        }
    }
}

fn render_route(
    renderer: &mut Renderer,
    surface: &mut Surface,
    scene: &Scene,
    format: Option<WorkingFormat>,
) -> ImageBuffer {
    if let Some(format) = format {
        let result = pollster::block_on(renderer.render_forced_base_graph_for_test(
            surface,
            scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        assert_eq!(result.stats.route, Some(RenderRoute::GpuGraph));
        pollster::block_on(renderer.read_headless(surface)).unwrap()
    } else {
        render(renderer, surface, scene)
    }
}

#[test]
fn transformed_stroke_prepares_repeating_paint_with_its_actual_device_scale() {
    let gradient = linear(0.2, &[(0.0, red()), (1.0, blue())]);
    let mut scene = Scene::new();
    scene.transform(Transform::scale(100.0, 1.0).unwrap(), |scene| {
        scene.stroke(
            Shape::rect(Rect::new(0.0, 2.0, 0.64, 4.0)),
            Stroke::try_new(2.0).unwrap(),
            gradient,
        );
    });
    let mut renderer = renderer();
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let mut surface = surface(&mut renderer, 64.0, 8.0, 1.0);
    for format in std::iter::once(None).chain(formats.into_iter().map(Some)) {
        let image = render_route(&mut renderer, &mut surface, &scene, format);
        assert_twenty_pixel_cycle(&image, 1, 4);
    }
}

#[test]
fn prepared_and_native_text_keep_run_local_repeating_paint_and_final_scale() {
    let font = FontData::try_from_bytes(
        AHEM_FONT_BYTES.to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    let glyphs = [
        TextGlyph::try_new(AHEM_GLYPH_X, 0.0, 0.8, 1.0).unwrap(),
        TextGlyph::try_new(AHEM_GLYPH_X, 2.0, 0.8, 1.0).unwrap(),
    ];
    let prepared = font
        .prepare_glyph_run(
            &TextInstance::try_new(1.0, vec![], GlyphSynthesis::none()).unwrap(),
            &glyphs,
            GlyphRunLimits::new(2, 16, 1 << 20, 1 << 20),
            &mut crate::RenderAccountingContext::default(),
        )
        .unwrap();
    let paint = TextPaint::try_fill(linear(0.7, &[(0.0, red()), (1.0, blue())]).into()).unwrap();
    let transform = Transform::try_new([20.0, 0.0, 0.0, 4.0, 2.0, 0.0]).unwrap();
    let mut renderer = renderer();
    let formats = graph_supported_working_formats_for_test(&mut renderer);
    let mut surface = surface(&mut renderer, 64.0, 4.0, 1.0);
    for native in [false, true] {
        let mut scene = Scene::new();
        if native {
            scene.text_run(
                TextRun::try_new(
                    ahem_font("repeating run-local paint"),
                    1.0,
                    transform,
                    paint.clone(),
                    &glyphs,
                    // Ahem X has font bounds [0,-200,1000,800] at
                    // 1000 units/em. Size 1 and baseline .8 place the two
                    // run-local squares at [0,1] and [2,3], both y=[0,1].
                    TextRunBounds::try_ink(Rect::new(0.0, 0.0, 3.0, 1.0)).unwrap(),
                )
                .unwrap(),
            );
        } else {
            scene.prepared_glyph_run(prepared.clone(), transform, paint.clone());
        }
        for format in std::iter::once(None).chain(formats.iter().copied().map(Some)) {
            let image = render_route(&mut renderer, &mut surface, &scene, format);
            // Integer device x=3,43 maps to run-local .05,2.05. Dividing
            // by .7 gives phases 1/14,13/14. Both coverage pixels are
            // inside their squares; restarting paint per glyph cannot pass.
            assert_pixel(&image, 3, 1, [237, 0, 18, 255], 4);
            assert_pixel(&image, 43, 1, [18, 0, 237, 255], 4);
            assert_pixel(&image, 0, 1, [0, 0, 0, 0], 2);
            assert_pixel(&image, 30, 1, [0, 0, 0, 0], 2);
        }
    }
}

#[test]
fn repeating_profiles_preserve_every_work_dimension_pixels_and_optional_timing() {
    let mut expected_work = None;
    let mut expected_pixels = None;
    for profile in [
        RenderProfile::Off,
        RenderProfile::Summary { timing: false },
        RenderProfile::Summary { timing: true },
    ] {
        let mut renderer = pollster::block_on(Renderer::new(
            Options::default()
                .with_accounting(RenderAccountingOptions::new().with_profile(profile)),
        ))
        .unwrap();
        let mut surface = surface(&mut renderer, 4.0, 4.0, 1.0);
        let gradient = linear(
            0.1,
            &[
                (0.0, red()),
                (1.0, rgba(0.0, 1.0, 0.0, 0.5)),
                (4.0, rgba(0.0, 0.0, 1.0, 0.0)),
            ],
        );
        pollster::block_on(renderer.render(
            &mut surface,
            &solid_scene(gradient),
            Parameters::default(),
        ))
        .unwrap();
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        let attempt = report.attempt().unwrap();
        let usage = attempt.usage();
        assert!(usage.work(RenderWork::GradientStopVisits) >= 3);
        let gradient_work = attempt
            .phase_usage(RenderPhase::GradientPrepare)
            .work(RenderWork::GradientStopVisits);
        assert!(gradient_work > 0);
        assert_eq!(
            attempt
                .phase_usage(RenderPhase::Encode)
                .work(RenderWork::GradientStopVisits),
            gradient_work,
            "Encode includes its gradient preparation child"
        );
        assert!(usage.work(RenderWork::GradientStopVisits) > gradient_work);
        let work = RenderWork::ALL.map(|unit| usage.work(unit));
        if let Some(expected) = expected_work {
            assert_eq!(
                work, expected,
                "profiling must preserve all actual work dimensions"
            );
        } else {
            expected_work = Some(work);
        }
        let timed = matches!(profile, RenderProfile::Summary { timing: true });
        assert_eq!(attempt.elapsed().is_some(), timed);
        if profile != RenderProfile::Off {
            assert!(attempt.phase_count(RenderPhase::GradientPrepare) > 0);
            assert!(attempt.phase_count(RenderPhase::Encode) > 0);
            assert_eq!(
                attempt
                    .phase_elapsed(RenderPhase::GradientPrepare)
                    .is_some(),
                timed
            );
        }
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_pixel(&image, 1, 1, [85, 170, 0, 96], 2);
        if let Some(expected) = expected_pixels.as_ref() {
            assert_eq!(&image, expected);
        } else {
            expected_pixels = Some(image);
        }
    }
}

#[test]
fn refused_repeating_stop_work_preserves_publication_stats_and_fresh_retry() {
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::GradientStopVisits, 0)
        .unwrap();
    let mut renderer = pollster::block_on(Renderer::new(
        Options::default().with_accounting(RenderAccountingOptions::new().with_limits(limits)),
    ))
    .unwrap();
    let mut surface = surface(&mut renderer, 4.0, 4.0, 1.0);
    let baseline = solid_scene(red());
    let successful =
        pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default()))
            .unwrap();
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let publication = surface.headless_publication_count_for_test();
    let gradient = linear(0.2, &[(0.0, red()), (1.0, blue())]);
    let error = pollster::block_on(renderer.render(
        &mut surface,
        &solid_scene(gradient),
        Parameters::default(),
    ))
    .unwrap_err();
    assert!(
        matches!(error.accounting_diagnostic(), Some(RenderAccountingError::Refused(refusal))
        if refusal.quantity() == RenderQuantity::Work(RenderWork::GradientStopVisits))
    );
    assert_eq!(renderer.stats(), successful);
    assert_eq!(surface.headless_publication_count_for_test(), publication);
    let report = renderer.take_last_attempt().unwrap();
    assert_eq!(report.status(), RenderAttemptStatus::Refused);
    assert_eq!(
        report
            .attempt()
            .unwrap()
            .usage()
            .work(RenderWork::Submissions),
        0
    );
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
    pollster::block_on(renderer.render(&mut surface, &baseline, Parameters::default())).unwrap();
    assert_eq!(
        surface.headless_publication_count_for_test(),
        publication + 1
    );
    assert_eq!(
        renderer.last_attempt().unwrap().status(),
        RenderAttemptStatus::Succeeded
    );
    assert_eq!(
        pollster::block_on(renderer.read_headless(&surface)).unwrap(),
        before
    );
}

#[test]
fn unrepresentable_radial_phase_is_refused_without_publishing_and_allows_valid_retry() {
    let mut renderer = renderer();
    let graph_format = graph_supported_working_formats_for_test(&mut renderer)
        .into_iter()
        .next()
        .unwrap();
    let mut surface = surface(&mut renderer, 4.0, 4.0, 1.0);
    let baseline = solid_scene(rgba(0.0, 1.0, 0.0, 1.0));
    let far_center = Point::new(268_435_456.0, 0.0);
    let unresolved_native = solid_scene(
        RepeatingGradient::try_radial(
            far_center,
            16.0,
            16.0,
            stops(&[(0.0, red()), (1.0, blue())]),
        )
        .unwrap(),
    );
    let valid = solid_scene(
        RepeatingGradient::try_radial(
            Point::new(0.0, 0.0),
            16.0,
            16.0,
            stops(&[(0.0, red()), (1.0, blue())]),
        )
        .unwrap(),
    );
    let unresolved_period = solid_scene(
        RepeatingGradient::try_radial(
            far_center,
            0.25,
            0.25,
            stops(&[(0.0, red()), (1.0, blue())]),
        )
        .unwrap(),
    );
    for format in [None, Some(graph_format)] {
        let before = render_route(&mut renderer, &mut surface, &baseline, format);
        assert_pixel(&before, 1, 0, [0, 255, 0, 255], 2);
        let successful = renderer.stats();
        let publication = surface.headless_publication_count_for_test();

        // At integer device (1,0), the true distance is exactly 2^28-1.
        // Its 16-pixel period has phase 15/16, hence [16,0,239,255]. The
        // required native field cannot retain that phase: reject it rather
        // than publishing a different field or averaging a resolved period.
        let result = if let Some(format) = format {
            pollster::block_on(renderer.render_forced_base_graph_for_test(
                &mut surface,
                &unresolved_native,
                Parameters::default(),
                format,
            ))
            .map(|result| result.stats)
        } else {
            pollster::block_on(renderer.render(
                &mut surface,
                &unresolved_native,
                Parameters::default(),
            ))
        };
        let error = result.unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidInput);
        assert!(error.invalid_value_diagnostic().is_some());
        assert_eq!(renderer.stats(), successful);
        assert_eq!(surface.headless_publication_count_for_test(), publication);
        let report = renderer.take_last_attempt().unwrap();
        assert_eq!(
            report.status(),
            RenderAttemptStatus::Failed(ErrorCode::InvalidInput)
        );
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::Submissions),
            0
        );
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );

        // A representable center-zero field must still render. At (1,0),
        // phase 1/16 gives the independently derived opposite color ratio.
        let retried = render_route(&mut renderer, &mut surface, &valid, format);
        assert_pixel(&retried, 1, 0, [239, 0, 16, 255], 3);
        assert_eq!(
            surface.headless_publication_count_for_test(),
            publication + 1
        );
        assert_eq!(
            renderer.last_attempt().unwrap().status(),
            RenderAttemptStatus::Succeeded
        );

        // A positive .25-pixel period selects the average before any unused
        // native-pattern qualification, even with the same distant center.
        let averaged = render_route(&mut renderer, &mut surface, &unresolved_period, format);
        assert_pixel(&averaged, 1, 0, [128, 0, 128, 255], 2);
        assert_eq!(
            surface.headless_publication_count_for_test(),
            publication + 2
        );
    }
}
