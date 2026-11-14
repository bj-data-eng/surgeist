//! Independent public GPU morphology oracles. CPU extrema below are literal
//! discrete definitions; they do not use production ranges, tables or queries.
use super::convolution::{assert_pixel, bounds, image, pixel, renderer_and_surface};
use crate::{
    Color, FilterColorSpace, FilterDomains, Image, ImageBuffer, ImageFit, Layer, MorphologyInput,
    MorphologyOperator as Op, MorphologySource, Parameters, PreparedMorphology, Rect,
    RenderAccountingContext, Renderer, Scene, Surface, prepare_morphology,
};

pub(super) fn operation(input: MorphologyInput, space: FilterColorSpace) -> PreparedMorphology {
    prepare_morphology(input, space, &mut RenderAccountingContext::default()).unwrap()
}
pub(super) fn source_scene(input: Image, destination: Rect) -> Scene {
    let mut scene = Scene::new();
    scene.image(input, destination, ImageFit::Stretch);
    scene
}
pub(super) fn finite_scene(
    source: Scene,
    domains: FilterDomains,
    effect: PreparedMorphology,
    layer: Layer,
) -> Scene {
    let mut scene = Scene::new();
    scene
        .morphology_layer(layer, effect, MorphologySource::finite(source, domains))
        .unwrap();
    scene
}
pub(super) fn render(renderer: &mut Renderer, surface: &mut Surface, scene: &Scene) -> ImageBuffer {
    let stats = pollster::block_on(renderer.render(surface, scene, Parameters::default())).unwrap();
    assert_eq!(stats, renderer.stats());
    assert_eq!(stats.route, Some(crate::RenderRoute::GpuGraph));
    pollster::block_on(renderer.read_headless(surface)).unwrap()
}
fn white(a: u8) -> [u8; 4] {
    if a == 0 { [0; 4] } else { [255, 255, 255, a] }
}
fn alpha_row(values: &[u8]) -> Image {
    image(
        values.len() as u32,
        1,
        values.iter().flat_map(|a| white(*a)).collect(),
    )
}
fn active(op: Op, x: f64, y: f64) -> PreparedMorphology {
    operation(
        MorphologyInput::new().with_operator(op).with_radii(x, y),
        FilterColorSpace::Srgb,
    )
}
pub(super) fn assert_premul(
    output: &ImageBuffer,
    x: u32,
    y: u32,
    expected: [f64; 4],
    tolerance: f64,
) {
    let actual = pixel(output, x, y);
    let a = f64::from(actual[3]) / 255.0;
    assert!(
        (a - expected[3]).abs() * 255.0 <= tolerance,
        "({x},{y}) {actual:?} expected premul {expected:?}"
    );
    for c in 0..3 {
        assert!(
            (f64::from(actual[c]) / 255.0 * a - expected[c]).abs() * 255.0 <= tolerance,
            "({x},{y}) {actual:?} expected premul {expected:?}"
        );
    }
}

#[test]
fn public_morphology_literal_finite_rows_include_transparent_exterior() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    for (op, r, expected) in [
        (Op::Dilate, 1.0, [191; 3]),
        (Op::Erode, 1.0, [0, 64, 0]),
        (Op::Dilate, 2.0, [191; 3]),
        (Op::Erode, 2.0, [0; 3]),
    ] {
        let scene = finite_scene(
            source_scene(alpha_row(&[64, 191, 128]), full),
            FilterDomains::try_new(full, full, full).unwrap(),
            active(op, r, 0.25),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for (x, a) in expected.into_iter().enumerate() {
            assert_pixel(&output, x as u32, 0, white(a), 2);
        }
    }
    // P one cell cannot cap R1 or omit finite exterior from erosion.
    let one = bounds(1, 1);
    let scene = finite_scene(
        source_scene(alpha_row(&[255]), one),
        FilterDomains::try_new(one, one, one).unwrap(),
        active(Op::Erode, 1.0, 0.25),
        Layer::new(),
    );
    let output = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&output, 0, 0, [0; 4], 1);
}

#[test]
fn public_morphology_nonpositive_disabled_and_positive_grid_point_preserve_numeric_input() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let bytes = vec![255, 0, 0, 64, 0, 170, 0, 191, 0, 0, 255, 128];
    for (x, y) in [
        (0.0, 0.0),
        (-0.0, 2.0),
        (-1.0, 1e300),
        (1e300, 0.0),
        (0.25, 0.25),
    ] {
        let scene = finite_scene(
            source_scene(image(3, 1, bytes.clone()), full),
            FilterDomains::try_new(full, full, full).unwrap(),
            operation(
                MorphologyInput::new().with_radii(x, y),
                FilterColorSpace::LinearSrgb,
            ),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for p in 0..3 {
            assert_pixel(
                &output,
                p,
                0,
                bytes[(p as usize) * 4..(p as usize + 1) * 4]
                    .try_into()
                    .unwrap(),
                2,
            );
        }
        assert_eq!(
            renderer
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(crate::RenderWork::KernelSampleVisits),
            0
        );
    }
}

#[test]
fn public_morphology_supplied_zero_disables_but_positive_quantized_zero_keeps_other_axis() {
    let (mut renderer, mut surface) = renderer_and_surface(1, 5);
    let full = bounds(1, 5);
    for (x, expected) in [(0.0, [0, 0, 255, 0, 0]), (0.25, [255; 5])] {
        let scene = finite_scene(
            source_scene(
                image(
                    1,
                    5,
                    [white(0), white(0), white(255), white(0), white(0)].concat(),
                ),
                full,
            ),
            FilterDomains::try_new(full, full, full).unwrap(),
            active(Op::Dilate, x, 2.0),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for (y, a) in expected.into_iter().enumerate() {
            assert_pixel(&output, 0, y as u32, white(a), 1);
        }
    }
}

#[test]
fn public_morphology_radius_thresholds_floor_before_closed_center_neighborhoods() {
    let (mut renderer, mut surface) = renderer_and_surface(7, 1);
    let full = bounds(7, 1);
    for (r, integer) in [(0.999, 0), (1.0, 1), (1.999, 1), (2.0, 2)] {
        let scene = finite_scene(
            source_scene(alpha_row(&[0, 0, 0, 255, 0, 0, 0]), full),
            FilterDomains::try_new(full, full, full).unwrap(),
            active(Op::Dilate, r, 0.25),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for x in 0..7 {
            assert_pixel(
                &output,
                x,
                0,
                white(if (i64::from(x) - 3).abs() <= integer {
                    255
                } else {
                    0
                }),
                1,
            );
        }
    }
}

#[test]
fn public_morphology_components_choose_independent_extrema_in_both_formats() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let full = bounds(5, 1);
    // A premul(.25,0,0,.5), B premul(0,.5,0,.75).
    let a = [128, 0, 0, 128];
    let b = [0, 170, 0, 191];
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        for (op, expected) in [
            (Op::Dilate, [0.25, 0.5, 0.0, 0.75]),
            (Op::Erode, [0.0, 0.0, 0.0, 0.5]),
        ] {
            let scene = finite_scene(
                source_scene(image(5, 1, [a, b, a, b, a].concat()), full),
                FilterDomains::try_new(full, full, full).unwrap(),
                active(op, 1.0, 0.25),
                Layer::new(),
            );
            pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                format,
            ))
            .unwrap();
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            assert_premul(&pixels, 2, 0, expected, 2.5);
        }
    }
}

fn decode(v: f64) -> f64 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn encode(v: f64) -> f64 {
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
#[test]
fn public_morphology_linear_color_extrema_are_premultiplied_and_converted_once() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let full = bounds(5, 1);
    let a = [255, 0, 0, 64];
    let b = [128, 0, 0, 255];
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        for space in [FilterColorSpace::Srgb, FilterColorSpace::LinearSrgb] {
            for op in [Op::Erode, Op::Dilate] {
                let scene = finite_scene(
                    source_scene(image(5, 1, [a, b, a, b, a].concat()), full),
                    FilterDomains::try_new(full, full, full).unwrap(),
                    operation(
                        MorphologyInput::new()
                            .with_operator(op)
                            .with_radii(1.0, 0.25),
                        space,
                    ),
                    Layer::new(),
                );
                pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                    &mut surface,
                    &scene,
                    Parameters::default(),
                    format,
                ))
                .unwrap();
                let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
                let aa: f64 = 64.0 / 255.0;
                let br = 128.0 / 255.0;
                let ar = aa;
                let br = if space == FilterColorSpace::Srgb {
                    br
                } else {
                    decode(br)
                };
                let (r, alpha) = match op {
                    Op::Erode => (ar.min(br), aa),
                    Op::Dilate => (ar.max(br), 1.0),
                };
                let r = if space == FilterColorSpace::Srgb {
                    r
                } else {
                    encode(r / alpha) * alpha
                };
                assert_premul(&pixels, 2, 0, [r, 0.0, 0.0, alpha], 2.5);
            }
        }
    }
}

#[test]
fn public_morphology_variable_queries_cross_every_reached_power_class() {
    let (mut renderer, mut surface) = renderer_and_surface(11, 1);
    let source = bounds(9, 1);
    let full = bounds(11, 1);
    let values = [26, 230, 51, 204, 77, 179, 102, 153, 128];
    for radius in [1_i64, 2, 3, 4, 8] {
        let scene = finite_scene(
            source_scene(alpha_row(&values), source),
            FilterDomains::try_new(source, full, full).unwrap(),
            active(Op::Dilate, radius as f64, 0.25),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        for x in 0..11 {
            let expected = values
                .iter()
                .enumerate()
                .filter(|(i, _)| (i64::from(x) - *i as i64).abs() <= radius)
                .map(|(_, a)| *a)
                .max()
                .unwrap_or(0);
            assert_pixel(&output, x, 0, white(expected), 2);
        }
    }
}

#[test]
fn public_morphology_unequal_rectangles_match_literal_two_dimensional_extrema() {
    let (mut renderer, mut surface) = renderer_and_surface(7, 6);
    let source = bounds(5, 4);
    let full = bounds(7, 6);
    let values: [u8; 20] = [
        20, 180, 90, 240, 60, 200, 40, 160, 80, 220, 70, 230, 30, 140, 100, 190, 50, 210, 110, 150,
    ];
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        for op in [Op::Erode, Op::Dilate] {
            let scene = finite_scene(
                source_scene(
                    image(5, 4, values.iter().flat_map(|a| white(*a)).collect()),
                    source,
                ),
                FilterDomains::try_new(source, full, full).unwrap(),
                active(op, 2.0, 1.0),
                Layer::new(),
            );
            pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                format,
            ))
            .unwrap();
            let output = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for y in 0..6 {
                for x in 0..7 {
                    let mut extrema = if op == Op::Erode { 255 } else { 0 };
                    for dy in -1_i64..=1 {
                        for dx in -2_i64..=2 {
                            let sx = i64::from(x) + dx;
                            let sy = i64::from(y) + dy;
                            let v = if (0..5).contains(&sx) && (0..4).contains(&sy) {
                                values[(sy * 5 + sx) as usize]
                            } else {
                                0
                            };
                            extrema = if op == Op::Erode {
                                extrema.min(v)
                            } else {
                                extrema.max(v)
                            };
                        }
                    }
                    assert_pixel(&output, x, y, white(extrema), 2);
                }
            }
        }
    }
}

#[test]
fn public_morphology_input_outside_small_result_and_vertical_rows_still_contribute() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 5);
    let full = bounds(5, 5);
    let result = Rect::new(2.0, 2.0, 1.0, 1.0);
    let mut source = Scene::new();
    source.image(
        alpha_row(&[255]),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        ImageFit::Stretch,
    );
    let scene = finite_scene(
        source,
        FilterDomains::try_new(full, full, result).unwrap(),
        active(Op::Dilate, 2.0, 2.0),
        Layer::new(),
    );
    let output = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&output, 2, 2, [255; 4], 1);
    assert_pixel(&output, 0, 0, [0; 4], 1);
}

#[test]
fn public_morphology_fractional_input_and_output_coverage_are_applied_once() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let input = Rect::new(0.0, 0.0, 0.5, 1.0);
    let full = bounds(3, 1);
    let output = Rect::new(1.0, 0.0, 0.5, 1.0);
    let scene = finite_scene(
        source_scene(alpha_row(&[255]), bounds(1, 1)),
        FilterDomains::try_new(input, full, output).unwrap(),
        active(Op::Dilate, 1.0, 0.25),
        Layer::new(),
    );
    let pixels = render(&mut renderer, &mut surface, &scene);
    assert_pixel(&pixels, 1, 0, white(64), 2);
    assert_pixel(&pixels, 0, 0, [0; 4], 1);
}

#[test]
fn public_morphology_huge_finite_radius_uses_bounded_semantic_proofs_before_casts() {
    let (mut renderer, mut surface) = renderer_and_surface(5, 1);
    let source = bounds(3, 1);
    let full = bounds(5, 1);
    for op in [Op::Erode, Op::Dilate] {
        let scene = finite_scene(
            source_scene(alpha_row(&[64, 191, 128]), source),
            FilterDomains::try_new(source, full, full).unwrap(),
            active(op, 1e300, 0.25),
            Layer::new(),
        );
        let pixels = render(&mut renderer, &mut surface, &scene);
        for x in 0..5 {
            assert_pixel(
                &pixels,
                x,
                0,
                white(if op == Op::Erode { 0 } else { 191 }),
                2,
            );
        }
        assert_eq!(
            renderer
                .last_attempt()
                .unwrap()
                .attempt()
                .unwrap()
                .usage()
                .work(crate::RenderWork::KernelSampleVisits),
            0
        );
    }
}

#[test]
fn public_morphology_constants_are_analytic_while_finite_erosion_includes_boundary_zero() {
    let (mut renderer, mut surface) = renderer_and_surface(3, 1);
    let full = bounds(3, 1);
    let color = Color::try_rgba(0.25, 0.5, 0.75, 0.5).unwrap();
    for op in [Op::Erode, Op::Dilate] {
        let mut scene = Scene::new();
        scene
            .morphology_layer(
                Layer::new(),
                active(op, 1e300, 1e300),
                MorphologySource::constant(color, full, full).unwrap(),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        for x in 0..3 {
            assert_pixel(&pixels, x, 0, [64, 128, 191, 128], 2);
        }
    }
}

#[test]
fn public_morphology_periodic_extrema_wrap_the_authentic_fundamental_canvas() {
    let (mut renderer, mut surface) = renderer_and_surface(9, 1);
    let period = bounds(3, 1);
    let full = bounds(9, 1);
    for (op, r, expected) in [
        (Op::Erode, 1.0, 64),
        (Op::Dilate, 1.0, 191),
        (Op::Erode, 1e300, 64),
        (Op::Dilate, 1e300, 191),
    ] {
        let mut scene = Scene::new();
        scene
            .morphology_layer(
                Layer::new(),
                active(op, r, 0.25),
                MorphologySource::periodic(
                    source_scene(alpha_row(&[64, 191, 128]), period),
                    period,
                    full,
                    full,
                )
                .unwrap(),
            )
            .unwrap();
        let pixels = render(&mut renderer, &mut surface, &scene);
        for x in 0..9 {
            assert_pixel(&pixels, x, 0, white(expected), 2);
        }
    }
}

#[test]
fn public_morphology_density_two_threshold_uses_actual_axis_grid() {
    use crate::{Options, Size};
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(4.0, 1.0), 2.0)).unwrap();
    let full = bounds(4, 1);
    for (radius, r) in [(0.499, 0_i64), (0.5, 1_i64)] {
        let mut source = Scene::new();
        source.fill(
            Rect::new(1.0, 0.0, 1.0, 1.0),
            Color::try_rgba(1.0, 1.0, 1.0, 1.0).unwrap(),
        );
        let scene = finite_scene(
            source,
            FilterDomains::try_new(full, full, full).unwrap(),
            active(Op::Dilate, radius, 0.125),
            Layer::new(),
        );
        let output = render(&mut renderer, &mut surface, &scene);
        // Aligned vector occupies actual device columns2,3. rho2 makes .5
        // a one-texel radius and .499 a zero-texel radius.
        for x in 0..8 {
            for y in 0..2 {
                let inside = (2 - r..=3 + r).contains(&i64::from(x));
                assert_pixel(&output, x, y, white(if inside { 255 } else { 0 }), 1);
            }
        }
    }
}

#[test]
fn public_morphology_positive_scaling_overflow_is_consumed_only_by_proved_huge_routes() {
    use crate::{Options, Size};
    let mut renderer = pollster::block_on(Renderer::new(Options::default())).unwrap();
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(5.0, 1.0), 2.0)).unwrap();
    let source = bounds(3, 1);
    let output = bounds(5, 1);
    assert!((f64::MAX * 2.0).is_infinite());
    for format in [
        crate::resource::WorkingFormat::HighPrecision,
        crate::resource::WorkingFormat::ReducedPrecision,
    ] {
        for op in [Op::Erode, Op::Dilate] {
            let mut input = Scene::new();
            input.fill(source, Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap());
            let scene = finite_scene(
                input,
                FilterDomains::try_new(source, output, output).unwrap(),
                active(op, f64::MAX, 0.125),
                Layer::new(),
            );
            pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
                &mut surface,
                &scene,
                Parameters::default(),
                format,
            ))
            .unwrap();
            let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
            for x in 0..10 {
                for y in 0..2 {
                    assert_pixel(
                        &pixels,
                        x,
                        y,
                        if op == Op::Erode {
                            [0; 4]
                        } else {
                            [255, 0, 0, 128]
                        },
                        2,
                    );
                }
            }
        }
        let period = Rect::new(0.0, 0.0, 2.5, 1.0);
        let mut input = Scene::new();
        input.fill(period, Color::try_rgba(0.0, 0.0, 1.0, 0.5).unwrap());
        let mut scene = Scene::new();
        scene
            .morphology_layer(
                Layer::new(),
                active(Op::Erode, f64::MAX, f64::MAX),
                MorphologySource::periodic(input, period, output, output).unwrap(),
            )
            .unwrap();
        pollster::block_on(renderer.render_with_exact_graph_working_format_for_test(
            &mut surface,
            &scene,
            Parameters::default(),
            format,
        ))
        .unwrap();
        let pixels = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        for x in 0..10 {
            for y in 0..2 {
                assert_pixel(&pixels, x, y, [0, 0, 255, 128], 2);
            }
        }
    }
}

#[test]
fn public_morphology_reached_capture_and_period_mapping_refusals_keep_the_owning_diagnostic() {
    use crate::{MorphologyNumericPhase, MorphologyPreparationError, Transform};
    let (mut renderer, mut surface) = renderer_and_surface(4, 1);
    let full = bounds(4, 1);
    let mut old = Scene::new();
    old.fill(full, Color::try_rgba(0.0, 1.0, 0.0, 1.0).unwrap());
    let stats =
        pollster::block_on(renderer.render(&mut surface, &old, Parameters::default())).unwrap();
    let before = pollster::block_on(renderer.read_headless(&surface)).unwrap();
    let origin = 1_099_511_627_776.0;
    let distant = Rect::new(origin, 0.0, 4.0, 1.0);
    let finite = finite_scene(
        source_scene(alpha_row(&[64, 191, 128, 255]), distant),
        FilterDomains::try_new(distant, distant, distant).unwrap(),
        active(Op::Dilate, 1.0, 0.25),
        Layer::new().with_transform(Transform::translation(-origin, 0.0).unwrap()),
    );
    let period = Rect::new(0.0, 0.0, f64::MIN_POSITIVE, 1.0);
    let mut tile = Scene::new();
    tile.fill(period, Color::BLACK);
    let mut periodic = Scene::new();
    periodic
        .morphology_layer(
            Layer::new(),
            active(Op::Dilate, 1.0, 0.25),
            MorphologySource::periodic(tile, period, full, full).unwrap(),
        )
        .unwrap();
    for scene in [finite, periodic] {
        let error =
            pollster::block_on(renderer.render(&mut surface, &scene, Parameters::default()))
                .unwrap_err();
        assert!(
            matches!(
                error.morphology_diagnostic(),
                Some(MorphologyPreparationError::Numeric {
                    phase: MorphologyNumericPhase::EndpointMapping,
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(error.gaussian_diagnostic(), None);
        assert_eq!(renderer.stats(), stats);
        assert_eq!(
            pollster::block_on(renderer.read_headless(&surface)).unwrap(),
            before
        );
    }
    // The same healthy renderer accepts an ordinary admitted scene after both
    // pre-runtime refusals; neither failure changes the existing publication.
    let valid = finite_scene(
        source_scene(alpha_row(&[64, 191, 128, 64]), full),
        FilterDomains::try_new(full, full, full).unwrap(),
        active(Op::Dilate, 1.0, 0.25),
        Layer::new(),
    );
    let after = render(&mut renderer, &mut surface, &valid);
    assert_pixel(&after, 1, 0, white(191), 2);
}
